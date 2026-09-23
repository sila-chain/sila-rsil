//! A dedicated `sil` + `snap/2` stream.
//!
//! [`SilSnapStream`] carries `sil` as the primary protocol and `snap/2` (SIP-8189) as a typed
//! side-channel over a single `RLPx` connection. It owns the raw [`P2PStream`] directly and reuses
//! the transport-free [`SilStreamInner`] codec for `sil` framing, demultiplexing the two protocols
//! by capability message-id offset.

use crate::{
    capability::SharedCapabilities,
    errors::{P2PStreamError, SilStreamError},
    handshake::SilRlpxHandshake,
    message::SilBroadcastMessage,
    Capability, NetworkPrimitives, P2PStream, SilMessage, SilNetworkPrimitives, SilStreamInner,
    SilVersion, UnifiedStatus, HANDSHAKE_TIMEOUT,
};
use alloy_primitives::bytes::{Bytes, BytesMut};
use futures::{Sink, SinkExt, Stream, StreamExt};
use rsil_eth_wire_types::{
    snap::{SnapProtocolMessage, SnapVersion},
    RawCapabilityMessage,
};
use rsil_sila_forks::ForkFilter;
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
};

/// A dedicated stream that carries `sil` as the primary protocol and `snap/2` (SIP-8189) as a typed
/// side-channel on the same `RLPx` connection.
///
/// Both protocols are exposed through the [`Stream`]/[`Sink`] impls, which yield and accept
/// [`SilSnapMessage`] (an `sil` message or a `snap/2` message). A single poll surfaces whichever
/// protocol the next inbound frame belongs to, so the owner drives one stream rather than two.
#[derive(Debug)]
pub struct SilSnapStream<St, N: NetworkPrimitives = SilNetworkPrimitives> {
    /// The raw `RLPx` stream carrying both `sil` and `snap/2` frames.
    conn: P2PStream<St>,
    /// Transport-free `sil` codec, reused from [`SilStream`](crate::SilStream).
    sil: SilStreamInner<N>,
    /// Relative message-id offset of the `snap/2` capability in the combined message space. Ids at
    /// or above it are snap, rebased to snap-relative and validated by
    /// [`SnapProtocolMessage::decode_versioned`].
    snap_offset: u8,
}

impl<St, N> SilSnapStream<St, N>
where
    St: Stream<Item = io::Result<BytesMut>> + Sink<Bytes, Error = io::Error> + Unpin + Send + Sync,
    N: NetworkPrimitives,
{
    /// Performs the `sil` status handshake over a connection that negotiated both `sil` and
    /// `snap/2`, returning the established [`SilSnapStream`] and the remote's status.
    ///
    /// The handshake runs directly on the underlying [`P2PStream`]; a `snap/2` message arriving
    /// before the status exchange completes is a protocol violation and surfaces as a handshake
    /// error.
    pub async fn handshake(
        mut conn: P2PStream<St>,
        status: UnifiedStatus,
        fork_filter: ForkFilter,
        handshake: Arc<dyn SilRlpxHandshake>,
        eth_max_message_size: usize,
    ) -> Result<(Self, UnifiedStatus), SilStreamError> {
        let eth_version = conn.shared_capabilities().eth_version()?;
        let snap_offset = eth_snap_layout(conn.shared_capabilities())?;

        let their_status =
            handshake.handshake(&mut conn, status, fork_filter, HANDSHAKE_TIMEOUT).await?;

        let sil = SilStreamInner::with_max_message_size(eth_version, eth_max_message_size);
        Ok((Self { conn, sil, snap_offset }, their_status))
    }
}

impl<St, N: NetworkPrimitives> SilSnapStream<St, N> {
    /// Returns the negotiated `sil` version.
    #[inline]
    pub const fn version(&self) -> SilVersion {
        self.sil.version()
    }

    /// Sets whether to reject block announcement messages before RLP decoding.
    #[inline]
    pub const fn set_reject_block_announcements(&mut self, reject: bool) {
        self.sil.set_reject_block_announcements(reject);
    }

    /// Returns a reference to the underlying [`P2PStream`].
    #[inline]
    pub const fn inner(&self) -> &P2PStream<St> {
        &self.conn
    }

    /// Returns mutable access to the underlying [`P2PStream`].
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut P2PStream<St> {
        &mut self.conn
    }

    /// Consumes this type and returns the underlying [`P2PStream`].
    #[inline]
    pub fn into_inner(self) -> P2PStream<St> {
        self.conn
    }
}

impl<St, N> SilSnapStream<St, N>
where
    St: Stream<Item = io::Result<BytesMut>> + Sink<Bytes, Error = io::Error> + Unpin,
    N: NetworkPrimitives,
{
    /// Queues an [`SilBroadcastMessage`] to be sent on the wire.
    pub fn start_send_broadcast(
        &mut self,
        item: SilBroadcastMessage<N>,
    ) -> Result<(), SilStreamError> {
        self.conn.start_send_unpin(item.encoded()).map_err(Into::into)
    }

    /// Sends a raw capability message over the connection.
    pub fn start_send_raw(&mut self, msg: RawCapabilityMessage) -> Result<(), SilStreamError> {
        self.conn.start_send_unpin(msg.encoded()).map_err(Into::into)
    }
}

/// A message carried by an [`SilSnapStream`]: either an `sil` message or a `snap/2` message.
#[derive(Debug)]
pub enum SilSnapMessage<N: NetworkPrimitives = SilNetworkPrimitives> {
    /// An `sil` protocol message.
    Sil(SilMessage<N>),
    /// A `snap/2` (SIP-8189) protocol message.
    Snap(SnapProtocolMessage),
}

impl<St, N> Stream for SilSnapStream<St, N>
where
    St: Stream<Item = io::Result<BytesMut>> + Sink<Bytes, Error = io::Error> + Unpin,
    N: NetworkPrimitives,
{
    type Item = Result<SilSnapMessage<N>, SilStreamError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let mut bytes = match ready!(this.conn.poll_next_unpin(cx)) {
            Some(Ok(bytes)) => bytes,
            Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
            None => return Poll::Ready(None),
        };

        let Some(&id) = bytes.first() else {
            return Poll::Ready(Some(Err(P2PStreamError::EmptyProtocolMessage.into())));
        };

        // `sil` occupies ids below the snap offset and is decoded by the shared codec. Ids at or
        // above it are snap: rebased to snap-relative (`0x00..`) and validated by
        // `decode_versioned`, which rejects ids that are out of range or invalid for `snap/2`.
        let Some(snap_id) = id.checked_sub(this.snap_offset) else {
            return Poll::Ready(Some(this.sil.decode_message(bytes).map(SilSnapMessage::Sil)));
        };
        bytes[0] = snap_id;
        Poll::Ready(Some(
            SnapProtocolMessage::decode_versioned(SnapVersion::V2, &bytes)
                .map(SilSnapMessage::Snap)
                .map_err(Into::into),
        ))
    }
}

impl<St, N> Sink<SilSnapMessage<N>> for SilSnapStream<St, N>
where
    St: Stream<Item = io::Result<BytesMut>> + Sink<Bytes, Error = io::Error> + Unpin,
    N: NetworkPrimitives,
{
    type Error = SilStreamError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().conn.poll_ready_unpin(cx).map_err(Into::into)
    }

    fn start_send(self: Pin<&mut Self>, item: SilSnapMessage<N>) -> Result<(), Self::Error> {
        let this = self.get_mut();
        let bytes = match item {
            SilSnapMessage::Sil(msg) => this.sil.encode_message(msg)?,
            SilSnapMessage::Snap(msg) => {
                // Reclaim the freshly-encoded buffer as mutable without copying the payload, then
                // rebase the snap-relative id into the combined message space.
                let mut buf =
                    msg.encode().0.try_into_mut().unwrap_or_else(|b| BytesMut::from(b.as_ref()));
                mask_snap(&mut buf, this.snap_offset)?;
                buf.freeze()
            }
        };
        this.conn.start_send_unpin(bytes).map_err(Into::into)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().conn.poll_flush_unpin(cx).map_err(Into::into)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().conn.poll_close_unpin(cx).map_err(Into::into)
    }
}

/// Resolves the `snap/2` message-id offset from the negotiated capabilities, accepting only a
/// connection that shares exactly `sil` at relative offset 0 immediately followed by `snap/2`.
///
/// The stream forwards every combined id below the snap offset to the sil codec and treats every
/// id at or above it as snap, so any other shared capability before `sil`, between `sil` and
/// `snap/2`, or after `snap/2` would be routed incorrectly. Such layouts belong on the
/// general-purpose satellite multiplexer and are rejected here.
fn eth_snap_layout(caps: &SharedCapabilities) -> Result<u8, SilStreamError> {
    if !caps.is_exact_eth_snap_v2() {
        return Err(P2PStreamError::CapabilityNotShared.into());
    }
    let snap = caps
        .ensure_matching_capability(&Capability::snap_2())
        .map_err(|_| SilStreamError::from(P2PStreamError::CapabilityNotShared))?;
    let snap_offset = snap.relative_message_id_offset();

    let sil = caps.sil()?;
    if sil.relative_message_id_offset() != 0 || sil.num_messages() != snap_offset {
        return Err(P2PStreamError::CapabilityNotShared.into());
    }
    Ok(snap_offset)
}

/// Rebases a snap-relative message id to the combined message space.
#[inline]
fn mask_snap(bytes: &mut BytesMut, snap_offset: u8) -> Result<(), io::Error> {
    let id = bytes.first().ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    bytes[0] =
        id.checked_add(snap_offset).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        handshake::SilHandshake,
        message::MAX_MESSAGE_SIZE,
        protocol::Protocol,
        test_utils::{connect_passthrough, eth_handshake, eth_hello},
        UnauthedP2PStream,
    };
    use rsil_eth_wire_types::{
        snap::{BlockAccessListsMessage, GetBlockAccessListsMessage},
        SilVersion,
    };
    use tokio::net::TcpListener;
    use tokio_util::codec::Decoder;

    /// Builds shared capabilities from matching local protocols and peer capabilities.
    fn shared_caps(local: Vec<Protocol>, peer: Vec<Capability>) -> SharedCapabilities {
        SharedCapabilities::try_new(local, peer).unwrap()
    }

    #[test]
    fn eth_snap_layout_accepts_eth_then_snap() {
        let caps = shared_caps(
            vec![SilVersion::Sil68.into(), Protocol::snap_2()],
            vec![SilVersion::Sil68.into(), Capability::snap_2()],
        );
        let offset = eth_snap_layout(&caps).unwrap();
        assert_eq!(offset, caps.sil().unwrap().num_messages());
    }

    #[test]
    fn eth_snap_layout_rejects_missing_snap() {
        let caps = shared_caps(vec![SilVersion::Sil68.into()], vec![SilVersion::Sil68.into()]);
        assert!(eth_snap_layout(&caps).is_err());
    }

    #[test]
    fn eth_snap_layout_rejects_capability_before_eth() {
        // "aaa" sorts before "sil", so it takes relative offset 0 and sil no longer starts at 0.
        let cap = Capability::new_static("aaa", 1);
        let caps = shared_caps(
            vec![Protocol::new(cap.clone(), 5), SilVersion::Sil68.into(), Protocol::snap_2()],
            vec![cap, SilVersion::Sil68.into(), Capability::snap_2()],
        );
        assert!(eth_snap_layout(&caps).is_err());
    }

    #[test]
    fn eth_snap_layout_rejects_capability_between_eth_and_snap() {
        // "les" sorts between "sil" and "snap", leaving a gap so snap no longer directly follows
        // sil.
        let cap = Capability::new_static("les", 1);
        let caps = shared_caps(
            vec![SilVersion::Sil68.into(), Protocol::new(cap.clone(), 5), Protocol::snap_2()],
            vec![SilVersion::Sil68.into(), cap, Capability::snap_2()],
        );
        assert!(eth_snap_layout(&caps).is_err());
    }

    #[test]
    fn eth_snap_layout_rejects_capability_after_snap() {
        // "zzz" sorts after "snap"; sil+snap still line up, but its frames would be routed
        // incorrectly as snap, so the layout must be rejected (it belongs on the satellite
        // multiplexer).
        let cap = Capability::new_static("zzz", 1);
        let caps = shared_caps(
            vec![SilVersion::Sil68.into(), Protocol::snap_2(), Protocol::new(cap.clone(), 5)],
            vec![SilVersion::Sil68.into(), Capability::snap_2(), cap],
        );
        assert!(eth_snap_layout(&caps).is_err());
    }

    #[test]
    fn mask_snap_rebases_into_combined_space() {
        let mut bytes = BytesMut::from(&[8u8, 0xcc][..]);
        mask_snap(&mut bytes, 17).unwrap();
        assert_eq!(bytes[0], 17 + 8);
    }

    /// End-to-end: two peers negotiate `sil` + `snap/2`, then a `GetBlockAccessLists` request and
    /// its `BlockAccessLists` response round-trip over live [`SilSnapStream`]s.
    #[tokio::test(flavor = "multi_thread")]
    async fn snap_request_response_round_trips_over_the_wire() {
        rsil_tracing::init_test_tracing();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        let (status, fork_filter) = eth_handshake();
        let server_status = status;
        let server_fork_filter = fork_filter.clone();

        // Server: accept, negotiate, and answer one GetBlockAccessLists echoing the request id.
        let server = tokio::spawn(async move {
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = crate::PassthroughCodec::default().framed(incoming);
            let server_hello = eth_snap_hello();
            let (conn, _) = UnauthedP2PStream::new(stream).handshake(server_hello).await.unwrap();

            let (mut stream, _) = SilSnapStream::<_, SilNetworkPrimitives>::handshake(
                conn,
                server_status,
                server_fork_filter,
                Arc::new(SilHandshake::default()),
                MAX_MESSAGE_SIZE,
            )
            .await
            .unwrap();

            while let Some(Ok(msg)) = stream.next().await {
                if let SilSnapMessage::Snap(SnapProtocolMessage::GetBlockAccessLists(req)) = msg {
                    let response = SnapProtocolMessage::BlockAccessLists(BlockAccessListsMessage {
                        request_id: req.request_id,
                        block_access_lists: rsil_eth_wire_types::BlockAccessLists(vec![None]),
                    });
                    stream.send(SilSnapMessage::Snap(response)).await.unwrap();
                }
            }
        });

        // Client: connect, negotiate, send the request, and await the correlated response.
        let conn = connect_passthrough(local_addr, eth_snap_hello()).await;
        let (mut stream, _) = SilSnapStream::<_, SilNetworkPrimitives>::handshake(
            conn,
            status,
            fork_filter,
            Arc::new(SilHandshake::default()),
            MAX_MESSAGE_SIZE,
        )
        .await
        .unwrap();

        stream
            .send(SilSnapMessage::Snap(SnapProtocolMessage::GetBlockAccessLists(
                GetBlockAccessListsMessage {
                    request_id: 7,
                    block_hashes: Vec::new(),
                    response_bytes: u64::MAX,
                },
            )))
            .await
            .unwrap();

        let response = loop {
            if let SilSnapMessage::Snap(SnapProtocolMessage::BlockAccessLists(resp)) =
                stream.next().await.unwrap().unwrap()
            {
                break resp;
            }
        };
        assert_eq!(response.request_id, 7);

        server.abort();
    }

    /// Builds a hello advertising `sil` + `snap/2`.
    fn eth_snap_hello() -> crate::HelloMessageWithProtocols {
        let mut hello = eth_hello().0;
        hello.try_add_protocol(Protocol::snap_2()).unwrap();
        hello
    }
}
