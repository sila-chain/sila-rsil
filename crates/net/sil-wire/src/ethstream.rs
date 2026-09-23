//! Sila protocol stream implementations.
//!
//! Provides stream types for the Sila wire protocol.
//! It separates protocol logic [`SilStreamInner`] from transport concerns [`SilStream`].
//! Handles handshaking, message processing, and RLP serialization.

use crate::{
    errors::{SilHandshakeError, SilStreamError},
    handshake::SilaEthHandshake,
    message::{SilBroadcastMessage, MAX_MESSAGE_SIZE, TX_MEMORY_BUDGET_MULTIPLIER},
    p2pstream::HANDSHAKE_TIMEOUT,
    CanDisconnect, DisconnectReason, ProtocolMessage, SilMessage, SilNetworkPrimitives, SilVersion,
    UnifiedStatus,
};
use alloy_primitives::bytes::{Bytes, BytesMut};
use futures::{ready, Sink, SinkExt};
use pin_project::pin_project;
use rsil_eth_wire_types::{NetworkPrimitives, RawCapabilityMessage, SilMessageID};
use rsil_sila_forks::ForkFilter;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::timeout;
use tokio_stream::Stream;
use tracing::{debug, trace};

/// An un-authenticated [`SilStream`]. This is consumed and returns a [`SilStream`] after the
/// `Status` handshake is completed.
#[pin_project]
#[derive(Debug)]
pub struct UnauthedEthStream<S> {
    #[pin]
    inner: S,
}

impl<S> UnauthedEthStream<S> {
    /// Create a new `UnauthedEthStream` from a type `S` which implements `Stream` and `Sink`.
    pub const fn new(inner: S) -> Self {
        Self { inner }
    }

    /// Consumes the type and returns the wrapped stream
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S, E> UnauthedEthStream<S>
where
    S: Stream<Item = Result<BytesMut, E>> + CanDisconnect<Bytes> + Send + Unpin,
    SilStreamError: From<E> + From<<S as Sink<Bytes>>::Error>,
{
    /// Consumes the [`UnauthedEthStream`] and returns an [`SilStream`] after the `Status`
    /// handshake is completed successfully. This also returns the `Status` message sent by the
    /// remote peer.
    ///
    /// Caution: This expects that the [`UnifiedStatus`] has the proper sil version configured, with
    /// ETH69 the initial status message changed.
    pub async fn handshake<N: NetworkPrimitives>(
        self,
        status: UnifiedStatus,
        fork_filter: ForkFilter,
    ) -> Result<(SilStream<S, N>, UnifiedStatus), SilStreamError> {
        self.handshake_with_timeout(status, fork_filter, HANDSHAKE_TIMEOUT).await
    }

    /// Wrapper around handshake which enforces a timeout.
    pub async fn handshake_with_timeout<N: NetworkPrimitives>(
        self,
        status: UnifiedStatus,
        fork_filter: ForkFilter,
        timeout_limit: Duration,
    ) -> Result<(SilStream<S, N>, UnifiedStatus), SilStreamError> {
        timeout(timeout_limit, Self::handshake_without_timeout(self, status, fork_filter))
            .await
            .map_err(|_| SilStreamError::StreamTimeout)?
    }

    /// Handshake with no timeout
    pub async fn handshake_without_timeout<N: NetworkPrimitives>(
        mut self,
        status: UnifiedStatus,
        fork_filter: ForkFilter,
    ) -> Result<(SilStream<S, N>, UnifiedStatus), SilStreamError> {
        trace!(
            status = %status.into_message(),
            "sending sil status to peer"
        );
        let their_status =
            SilaEthHandshake(&mut self.inner).eth_handshake(status, fork_filter).await?;

        // now we can create the `SilStream` because the peer has successfully completed
        // the handshake
        let stream = SilStream::new(status.version, self.inner);

        Ok((stream, their_status))
    }
}

/// Contains sil protocol specific logic for processing messages
#[derive(Debug)]
pub struct SilStreamInner<N> {
    /// Negotiated sil version
    version: SilVersion,
    /// Maximum allowed SIL message size.
    max_message_size: usize,
    /// When true, `NewBlock` (0x07) and `NewBlockHashes` (0x01) messages are rejected before RLP
    /// decoding to avoid any memory impact for non-PoW networks.
    reject_block_announcements: bool,
    _pd: std::marker::PhantomData<N>,
}

impl<N> SilStreamInner<N>
where
    N: NetworkPrimitives,
{
    /// Creates a new [`SilStreamInner`] with the given sil version
    pub const fn new(version: SilVersion) -> Self {
        Self::with_max_message_size(version, MAX_MESSAGE_SIZE)
    }

    /// Creates a new [`SilStreamInner`] with the given sil version and message size limit.
    pub const fn with_max_message_size(version: SilVersion, max_message_size: usize) -> Self {
        Self {
            version,
            max_message_size,
            reject_block_announcements: false,
            _pd: std::marker::PhantomData,
        }
    }

    /// Returns the sil version
    #[inline]
    pub const fn version(&self) -> SilVersion {
        self.version
    }

    /// Sets whether to reject block announcement messages (`NewBlock`, `NewBlockHashes`) before
    /// RLP decoding.
    pub const fn set_reject_block_announcements(&mut self, reject: bool) {
        self.reject_block_announcements = reject;
    }

    /// Decodes incoming bytes into an [`SilMessage`].
    pub fn decode_message(&self, bytes: BytesMut) -> Result<SilMessage<N>, SilStreamError> {
        if bytes.len() > self.max_message_size {
            return Err(SilStreamError::MessageTooBig(bytes.len()));
        }

        if self.reject_block_announcements
            && let Some(&id) = bytes.first()
            && (id == SilMessageID::NewBlock.to_u8() || id == SilMessageID::NewBlockHashes.to_u8())
        {
            return Err(SilStreamError::UnsupportedMessage { message_id: id });
        }

        let msg = match ProtocolMessage::decode_message_with_tx_memory_budget(
            self.version,
            &mut bytes.as_ref(),
            self.max_message_size * TX_MEMORY_BUDGET_MULTIPLIER,
        ) {
            Ok(m) => m,
            Err(err) => {
                let msg = if bytes.len() > 50 {
                    format!("{:02x?}...{:x?}", &bytes[..10], &bytes[bytes.len() - 10..])
                } else {
                    format!("{bytes:02x?}")
                };
                debug!(
                    version=?self.version,
                    %msg,
                    "failed to decode protocol message"
                );
                return Err(SilStreamError::InvalidMessage(err));
            }
        };

        if matches!(msg.message, SilMessage::Status(_)) {
            return Err(SilStreamError::SilHandshakeError(SilHandshakeError::StatusNotInHandshake));
        }

        Ok(msg.message)
    }

    /// Encodes an [`SilMessage`] to bytes.
    ///
    /// Validates that Status messages are not sent after handshake, enforcing protocol rules.
    pub fn encode_message(&self, item: SilMessage<N>) -> Result<Bytes, SilStreamError> {
        if matches!(item, SilMessage::Status(_)) {
            return Err(SilStreamError::SilHandshakeError(SilHandshakeError::StatusNotInHandshake));
        }

        Ok(Bytes::from(alloy_rlp::encode(ProtocolMessage::from(item))))
    }
}

/// An `SilStream` wraps over any `Stream` that yields bytes and makes it
/// compatible with sil-networking protocol messages, which get RLP encoded/decoded.
#[pin_project]
#[derive(Debug)]
pub struct SilStream<S, N = SilNetworkPrimitives> {
    /// Sil-specific logic
    sil: SilStreamInner<N>,
    #[pin]
    inner: S,
}

impl<S, N: NetworkPrimitives> SilStream<S, N> {
    /// Creates a new unauthed [`SilStream`] from a provided stream. You will need
    /// to manually handshake a peer.
    #[inline]
    pub const fn new(version: SilVersion, inner: S) -> Self {
        Self::with_max_message_size(version, inner, MAX_MESSAGE_SIZE)
    }

    /// Creates a new unauthed [`SilStream`] with a custom max message size.
    #[inline]
    pub const fn with_max_message_size(
        version: SilVersion,
        inner: S,
        max_message_size: usize,
    ) -> Self {
        Self { sil: SilStreamInner::with_max_message_size(version, max_message_size), inner }
    }

    /// Returns the sil version.
    #[inline]
    pub const fn version(&self) -> SilVersion {
        self.sil.version()
    }

    /// Sets whether to reject block announcement messages (`NewBlock`, `NewBlockHashes`) before
    /// RLP decoding.
    pub const fn set_reject_block_announcements(&mut self, reject: bool) {
        self.sil.set_reject_block_announcements(reject);
    }

    /// Returns the underlying stream.
    #[inline]
    pub const fn inner(&self) -> &S {
        &self.inner
    }

    /// Returns mutable access to the underlying stream.
    #[inline]
    pub const fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Consumes this type and returns the wrapped stream.
    #[inline]
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S, E, N> SilStream<S, N>
where
    S: Sink<Bytes, Error = E> + Unpin,
    SilStreamError: From<E>,
    N: NetworkPrimitives,
{
    /// Same as [`Sink::start_send`] but accepts a [`SilBroadcastMessage`] instead.
    pub fn start_send_broadcast(
        &mut self,
        item: SilBroadcastMessage<N>,
    ) -> Result<(), SilStreamError> {
        self.inner.start_send_unpin(item.encoded())?;
        Ok(())
    }

    /// Sends a raw capability message directly over the stream
    pub fn start_send_raw(&mut self, msg: RawCapabilityMessage) -> Result<(), SilStreamError> {
        self.inner.start_send_unpin(msg.encoded())?;
        Ok(())
    }
}

impl<S, E, N> Stream for SilStream<S, N>
where
    S: Stream<Item = Result<BytesMut, E>> + Unpin,
    SilStreamError: From<E>,
    N: NetworkPrimitives,
{
    type Item = Result<SilMessage<N>, SilStreamError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let res = ready!(this.inner.poll_next(cx));

        match res {
            Some(Ok(bytes)) => Poll::Ready(Some(this.sil.decode_message(bytes))),
            Some(Err(err)) => Poll::Ready(Some(Err(err.into()))),
            None => Poll::Ready(None),
        }
    }
}

impl<S, N> Sink<SilMessage<N>> for SilStream<S, N>
where
    S: CanDisconnect<Bytes> + Unpin,
    SilStreamError: From<<S as Sink<Bytes>>::Error>,
    N: NetworkPrimitives,
{
    type Error = SilStreamError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx).map_err(Into::into)
    }

    fn start_send(self: Pin<&mut Self>, item: SilMessage<N>) -> Result<(), Self::Error> {
        if matches!(item, SilMessage::Status(_)) {
            // Attempt to disconnect the peer for protocol breach when trying to send Status
            // message after handshake is complete
            let mut this = self.project();
            // We can't await the disconnect future here since this is a synchronous method,
            // but we can start the disconnect process. The actual disconnect will be handled
            // asynchronously by the caller or the stream's poll methods.
            let _disconnect_future = this.inner.disconnect(DisconnectReason::ProtocolBreach);
            return Err(SilStreamError::SilHandshakeError(SilHandshakeError::StatusNotInHandshake));
        }

        self.project()
            .inner
            .start_send(Bytes::from(alloy_rlp::encode(ProtocolMessage::from(item))))?;

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx).map_err(Into::into)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx).map_err(Into::into)
    }
}

impl<S, N> CanDisconnect<SilMessage<N>> for SilStream<S, N>
where
    S: CanDisconnect<Bytes> + Send,
    SilStreamError: From<<S as Sink<Bytes>>::Error>,
    N: NetworkPrimitives,
{
    fn disconnect(
        &mut self,
        reason: DisconnectReason,
    ) -> Pin<Box<dyn Future<Output = Result<(), SilStreamError>> + Send + '_>> {
        Box::pin(async move { self.inner.disconnect(reason).await.map_err(Into::into) })
    }
}

#[cfg(test)]
mod tests {
    use super::UnauthedEthStream;
    use crate::{
        broadcast::BlockHashNumber,
        errors::{SilHandshakeError, SilStreamError},
        ethstream::RawCapabilityMessage,
        hello::DEFAULT_TCP_PORT,
        p2pstream::UnauthedP2PStream,
        HelloMessageWithProtocols, PassthroughCodec, ProtocolVersion, SilMessage, SilStream,
        SilVersion, Status, StatusMessage,
    };
    use alloy_chains::NamedChain;
    use alloy_primitives::{bytes::Bytes, B256, U256};
    use alloy_rlp::Decodable;
    use futures::{SinkExt, StreamExt};
    use rsil_ecies::stream::ECIESStream;
    use rsil_eth_wire_types::{SilNetworkPrimitives, UnifiedStatus};
    use rsil_network_peers::pk2id;
    use rsil_sila_forks::{ForkFilter, Head};
    use secp256k1::{SecretKey, SECP256K1};
    use std::time::Duration;
    use tokio::net::{TcpListener, TcpStream};
    use tokio_util::codec::Decoder;

    #[tokio::test]
    async fn can_handshake() {
        let genesis = B256::random();
        let fork_filter = ForkFilter::new(Head::default(), genesis, 0, Vec::new());

        let status = Status {
            version: SilVersion::Sil67,
            chain: NamedChain::SilaMainnet.into(),
            total_difficulty: U256::ZERO,
            blockhash: B256::random(),
            genesis,
            // Pass the current fork id.
            forkid: fork_filter.current(),
        };
        let unified_status = UnifiedStatus::from_message(StatusMessage::Legacy(status));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let status_clone = unified_status;
        let fork_filter_clone = fork_filter.clone();
        let handle = tokio::spawn(async move {
            // roughly based off of the design of tokio::net::TcpListener
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = PassthroughCodec::default().framed(incoming);
            let (_, their_status) = UnauthedEthStream::new(stream)
                .handshake::<SilNetworkPrimitives>(status_clone, fork_filter_clone)
                .await
                .unwrap();

            // just make sure it equals our status (our status is a clone of their status)
            assert_eq!(their_status, status_clone);
        });

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let sink = PassthroughCodec::default().framed(outgoing);

        // try to connect
        let (_, their_status) = UnauthedEthStream::new(sink)
            .handshake::<SilNetworkPrimitives>(unified_status, fork_filter)
            .await
            .unwrap();

        // their status is a clone of our status, these should be equal
        assert_eq!(their_status, unified_status);

        // wait for it to finish
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn pass_handshake_on_low_td_bitlen() {
        let genesis = B256::random();
        let fork_filter = ForkFilter::new(Head::default(), genesis, 0, Vec::new());

        let status = Status {
            version: SilVersion::Sil67,
            chain: NamedChain::SilaMainnet.into(),
            total_difficulty: U256::from(2).pow(U256::from(100)) - U256::from(1),
            blockhash: B256::random(),
            genesis,
            // Pass the current fork id.
            forkid: fork_filter.current(),
        };
        let unified_status = UnifiedStatus::from_message(StatusMessage::Legacy(status));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let status_clone = unified_status;
        let fork_filter_clone = fork_filter.clone();
        let handle = tokio::spawn(async move {
            // roughly based off of the design of tokio::net::TcpListener
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = PassthroughCodec::default().framed(incoming);
            let (_, their_status) = UnauthedEthStream::new(stream)
                .handshake::<SilNetworkPrimitives>(status_clone, fork_filter_clone)
                .await
                .unwrap();

            // just make sure it equals our status, and that the handshake succeeded
            assert_eq!(their_status, status_clone);
        });

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let sink = PassthroughCodec::default().framed(outgoing);

        // try to connect
        let (_, their_status) = UnauthedEthStream::new(sink)
            .handshake::<SilNetworkPrimitives>(unified_status, fork_filter)
            .await
            .unwrap();

        // their status is a clone of our status, these should be equal
        assert_eq!(their_status, unified_status);

        // await the other handshake
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn fail_handshake_on_high_td_bitlen() {
        let genesis = B256::random();
        let fork_filter = ForkFilter::new(Head::default(), genesis, 0, Vec::new());

        let status = Status {
            version: SilVersion::Sil67,
            chain: NamedChain::SilaMainnet.into(),
            total_difficulty: U256::from(2).pow(U256::from(164)),
            blockhash: B256::random(),
            genesis,
            // Pass the current fork id.
            forkid: fork_filter.current(),
        };
        let unified_status = UnifiedStatus::from_message(StatusMessage::Legacy(status));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let status_clone = unified_status;
        let fork_filter_clone = fork_filter.clone();
        let handle = tokio::spawn(async move {
            // roughly based off of the design of tokio::net::TcpListener
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = PassthroughCodec::default().framed(incoming);
            let handshake_res = UnauthedEthStream::new(stream)
                .handshake::<SilNetworkPrimitives>(status_clone, fork_filter_clone)
                .await;

            // make sure the handshake fails due to td too high
            assert!(matches!(
                handshake_res,
                Err(SilStreamError::SilHandshakeError(
                    SilHandshakeError::TotalDifficultyBitLenTooLarge { got: 165, maximum: 160 }
                ))
            ));
        });

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let sink = PassthroughCodec::default().framed(outgoing);

        // try to connect
        let handshake_res = UnauthedEthStream::new(sink)
            .handshake::<SilNetworkPrimitives>(unified_status, fork_filter)
            .await;

        // this handshake should also fail due to td too high
        assert!(matches!(
            handshake_res,
            Err(SilStreamError::SilHandshakeError(
                SilHandshakeError::TotalDifficultyBitLenTooLarge { got: 165, maximum: 160 }
            ))
        ));

        // await the other handshake
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn can_write_and_read_cleartext() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        let test_msg = SilMessage::<SilNetworkPrimitives>::NewBlockHashes(
            vec![
                BlockHashNumber { hash: B256::random(), number: 5 },
                BlockHashNumber { hash: B256::random(), number: 6 },
            ]
            .into(),
        );

        let test_msg_clone = test_msg.clone();
        let handle = tokio::spawn(async move {
            // roughly based off of the design of tokio::net::TcpListener
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = PassthroughCodec::default().framed(incoming);
            let mut stream = SilStream::new(SilVersion::Sil67, stream);

            // use the stream to get the next message
            let message = stream.next().await.unwrap().unwrap();
            assert_eq!(message, test_msg_clone);
        });

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let sink = PassthroughCodec::default().framed(outgoing);
        let mut client_stream = SilStream::new(SilVersion::Sil67, sink);

        client_stream.send(test_msg).await.unwrap();

        // make sure the server receives the message and asserts before ending the test
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn can_write_and_read_ecies() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        let server_key = SecretKey::new(&mut rand_08::thread_rng());
        let test_msg = SilMessage::<SilNetworkPrimitives>::NewBlockHashes(
            vec![
                BlockHashNumber { hash: B256::random(), number: 5 },
                BlockHashNumber { hash: B256::random(), number: 6 },
            ]
            .into(),
        );

        let test_msg_clone = test_msg.clone();
        let handle = tokio::spawn(async move {
            // roughly based off of the design of tokio::net::TcpListener
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = ECIESStream::incoming(incoming, server_key).await.unwrap();
            let mut stream = SilStream::new(SilVersion::Sil67, stream);

            // use the stream to get the next message
            let message = stream.next().await.unwrap().unwrap();
            assert_eq!(message, test_msg_clone);
        });

        // create the server pubkey
        let server_id = pk2id(&server_key.public_key(SECP256K1));

        let client_key = SecretKey::new(&mut rand_08::thread_rng());

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let outgoing = ECIESStream::connect(outgoing, client_key, server_id).await.unwrap();
        let mut client_stream = SilStream::new(SilVersion::Sil67, outgoing);

        client_stream.send(test_msg).await.unwrap();

        // make sure the server receives the message and asserts before ending the test
        handle.await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ethstream_over_p2p() {
        // create a p2p stream and server, then confirm that the two are authed
        // create tcpstream
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        let server_key = SecretKey::new(&mut rand_08::thread_rng());
        let test_msg = SilMessage::<SilNetworkPrimitives>::NewBlockHashes(
            vec![
                BlockHashNumber { hash: B256::random(), number: 5 },
                BlockHashNumber { hash: B256::random(), number: 6 },
            ]
            .into(),
        );

        let genesis = B256::random();
        let fork_filter = ForkFilter::new(Head::default(), genesis, 0, Vec::new());

        let status = Status {
            version: SilVersion::Sil67,
            chain: NamedChain::SilaMainnet.into(),
            total_difficulty: U256::ZERO,
            blockhash: B256::random(),
            genesis,
            // Pass the current fork id.
            forkid: fork_filter.current(),
        };
        let unified_status = UnifiedStatus::from_message(StatusMessage::Legacy(status));

        let status_copy = unified_status;
        let fork_filter_clone = fork_filter.clone();
        let test_msg_clone = test_msg.clone();
        let handle = tokio::spawn(async move {
            // roughly based off of the design of tokio::net::TcpListener
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = ECIESStream::incoming(incoming, server_key).await.unwrap();

            let server_hello = HelloMessageWithProtocols {
                protocol_version: ProtocolVersion::V5,
                client_version: "bitcoind/1.0.0".to_string(),
                protocols: vec![SilVersion::Sil67.into()],
                port: DEFAULT_TCP_PORT,
                id: pk2id(&server_key.public_key(SECP256K1)),
            };

            let unauthed_stream = UnauthedP2PStream::new(stream);
            let (p2p_stream, _) = unauthed_stream.handshake(server_hello).await.unwrap();
            let (mut eth_stream, _) = UnauthedEthStream::new(p2p_stream)
                .handshake(status_copy, fork_filter_clone)
                .await
                .unwrap();

            // use the stream to get the next message
            let message = eth_stream.next().await.unwrap().unwrap();
            assert_eq!(message, test_msg_clone);
        });

        // create the server pubkey
        let server_id = pk2id(&server_key.public_key(SECP256K1));

        let client_key = SecretKey::new(&mut rand_08::thread_rng());

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let sink = ECIESStream::connect(outgoing, client_key, server_id).await.unwrap();

        let client_hello = HelloMessageWithProtocols {
            protocol_version: ProtocolVersion::V5,
            client_version: "bitcoind/1.0.0".to_string(),
            protocols: vec![SilVersion::Sil67.into()],
            port: DEFAULT_TCP_PORT,
            id: pk2id(&client_key.public_key(SECP256K1)),
        };

        let unauthed_stream = UnauthedP2PStream::new(sink);
        let (p2p_stream, _) = unauthed_stream.handshake(client_hello).await.unwrap();

        let (mut client_stream, _) = UnauthedEthStream::new(p2p_stream)
            .handshake(unified_status, fork_filter)
            .await
            .unwrap();

        client_stream.send(test_msg).await.unwrap();

        // make sure the server receives the message and asserts before ending the test
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn handshake_should_timeout() {
        let genesis = B256::random();
        let fork_filter = ForkFilter::new(Head::default(), genesis, 0, Vec::new());

        let status = Status {
            version: SilVersion::Sil67,
            chain: NamedChain::SilaMainnet.into(),
            total_difficulty: U256::ZERO,
            blockhash: B256::random(),
            genesis,
            // Pass the current fork id.
            forkid: fork_filter.current(),
        };
        let unified_status = UnifiedStatus::from_message(StatusMessage::Legacy(status));

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let status_clone = unified_status;
        let fork_filter_clone = fork_filter.clone();
        let _handle = tokio::spawn(async move {
            // Delay accepting the connection for longer than the client's timeout period
            tokio::time::sleep(Duration::from_secs(11)).await;
            // roughly based off of the design of tokio::net::TcpListener
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = PassthroughCodec::default().framed(incoming);
            let (_, their_status) = UnauthedEthStream::new(stream)
                .handshake::<SilNetworkPrimitives>(status_clone, fork_filter_clone)
                .await
                .unwrap();

            // just make sure it equals our status (our status is a clone of their status)
            assert_eq!(their_status, status_clone);
        });

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let sink = PassthroughCodec::default().framed(outgoing);

        // try to connect
        let handshake_result = UnauthedEthStream::new(sink)
            .handshake_with_timeout::<SilNetworkPrimitives>(
                unified_status,
                fork_filter,
                Duration::from_secs(1),
            )
            .await;

        // Assert that a timeout error occurred
        assert!(
            matches!(handshake_result, Err(e) if e.to_string() == SilStreamError::StreamTimeout.to_string())
        );
    }

    #[tokio::test]
    async fn can_write_and_read_raw_capability() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let test_msg = RawCapabilityMessage { id: 0x1234, payload: Bytes::from(vec![1, 2, 3, 4]) };

        let test_msg_clone = test_msg.clone();
        let handle = tokio::spawn(async move {
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = PassthroughCodec::default().framed(incoming);
            let mut stream = SilStream::<_, SilNetworkPrimitives>::new(SilVersion::Sil67, stream);

            let bytes = stream.inner_mut().next().await.unwrap().unwrap();

            // Create a cursor to track position while decoding
            let mut id_bytes = &bytes[..];
            let decoded_id = <usize as Decodable>::decode(&mut id_bytes).unwrap();
            assert_eq!(decoded_id, test_msg_clone.id);

            // Get remaining bytes after ID decoding
            let remaining = id_bytes;
            assert_eq!(remaining, &test_msg_clone.payload[..]);
        });

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let sink = PassthroughCodec::default().framed(outgoing);
        let mut client_stream = SilStream::<_, SilNetworkPrimitives>::new(SilVersion::Sil67, sink);

        client_stream.start_send_raw(test_msg).unwrap();
        client_stream.inner_mut().flush().await.unwrap();

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn status_message_after_handshake_triggers_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let handle = tokio::spawn(async move {
            let (incoming, _) = listener.accept().await.unwrap();
            let stream = PassthroughCodec::default().framed(incoming);
            let mut stream = SilStream::<_, SilNetworkPrimitives>::new(SilVersion::Sil67, stream);

            // Try to send a Status message after handshake - this should trigger disconnect
            let status = Status {
                version: SilVersion::Sil67,
                chain: NamedChain::SilaMainnet.into(),
                total_difficulty: U256::ZERO,
                blockhash: B256::random(),
                genesis: B256::random(),
                forkid: ForkFilter::new(Head::default(), B256::random(), 0, Vec::new()).current(),
            };
            let status_message =
                SilMessage::<SilNetworkPrimitives>::Status(StatusMessage::Legacy(status));

            // This should return an error and trigger disconnect
            let result = stream.send(status_message).await;
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                SilStreamError::SilHandshakeError(SilHandshakeError::StatusNotInHandshake)
            ));
        });

        let outgoing = TcpStream::connect(local_addr).await.unwrap();
        let sink = PassthroughCodec::default().framed(outgoing);
        let mut client_stream = SilStream::<_, SilNetworkPrimitives>::new(SilVersion::Sil67, sink);

        // Send a valid message to keep the connection alive
        let test_msg = SilMessage::<SilNetworkPrimitives>::NewBlockHashes(
            vec![BlockHashNumber { hash: B256::random(), number: 5 }].into(),
        );
        client_stream.send(test_msg).await.unwrap();

        handle.await.unwrap();
    }
}
