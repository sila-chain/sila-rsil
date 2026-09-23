//! Connection types for a session

use futures::{Sink, SinkExt, Stream, StreamExt};
use rsil_ecies::stream::ECIESStream;
use rsil_eth_wire::{
    errors::{P2PStreamError, SilStreamError},
    message::SilBroadcastMessage,
    multiplex::{ProtocolProxy, RlpxSatelliteStream},
    snap::SnapProtocolMessage,
    NetworkPrimitives, P2PStream, SilMessage, SilNetworkPrimitives, SilSnapMessage, SilSnapStream,
    SilStream, SilVersion,
};
use rsil_eth_wire_types::RawCapabilityMessage;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::net::TcpStream;

/// The type of the underlying peer network connection.
pub type SilPeerConnection<N> = SilStream<P2PStream<ECIESStream<TcpStream>>, N>;

/// Various connection types that at least support the SIL protocol.
pub type SilSatelliteConnection<N = SilNetworkPrimitives> =
    RlpxSatelliteStream<ECIESStream<TcpStream>, SilStream<ProtocolProxy, N>>;

/// A dedicated `sil` + `snap/2` connection.
pub type SilSnapConnection<N = SilNetworkPrimitives> = SilSnapStream<ECIESStream<TcpStream>, N>;

/// Connection types that support the SIL protocol.
///
/// This can be either:
/// - A connection that only supports the SIL protocol
/// - A connection that supports the SIL protocol and `snap/2` ([`SilSnapStream`])
/// - A connection that supports the SIL protocol and at least one other `RLPx` protocol
// This type is boxed because the underlying stream is ~6KB,
// mostly coming from `P2PStream`'s `snap::Encoder` (2072), and `ECIESStream` (3600).
#[derive(Debug)]
pub enum SilRlpxConnection<N: NetworkPrimitives = SilNetworkPrimitives> {
    /// A connection that only supports the SIL protocol.
    SilOnly(Box<SilPeerConnection<N>>),
    /// A dedicated connection that supports the SIL protocol and `snap/2` (SIP-8189).
    SilSnap(Box<SilSnapConnection<N>>),
    /// A connection that supports the SIL protocol and __at least one other__ `RLPx` protocol.
    Satellite(Box<SilSatelliteConnection<N>>),
}

impl<N: NetworkPrimitives> SilRlpxConnection<N> {
    /// Returns the negotiated SIL version.
    #[inline]
    pub(crate) const fn version(&self) -> SilVersion {
        match self {
            Self::SilOnly(conn) => conn.version(),
            Self::SilSnap(conn) => conn.version(),
            Self::Satellite(conn) => conn.primary().version(),
        }
    }

    /// Returns `true` if `snap/2` was negotiated on this connection.
    #[inline]
    pub(crate) const fn supports_snap(&self) -> bool {
        matches!(self, Self::SilSnap(_))
    }

    /// Consumes this type and returns the wrapped [`P2PStream`].
    #[inline]
    pub(crate) fn into_inner(self) -> P2PStream<ECIESStream<TcpStream>> {
        match self {
            Self::SilOnly(conn) => conn.into_inner(),
            Self::SilSnap(conn) => conn.into_inner(),
            Self::Satellite(conn) => conn.into_inner(),
        }
    }

    /// Returns mutable access to the underlying stream.
    #[inline]
    pub(crate) fn inner_mut(&mut self) -> &mut P2PStream<ECIESStream<TcpStream>> {
        match self {
            Self::SilOnly(conn) => conn.inner_mut(),
            Self::SilSnap(conn) => conn.inner_mut(),
            Self::Satellite(conn) => conn.inner_mut(),
        }
    }

    /// Returns access to the underlying stream.
    #[inline]
    pub(crate) const fn inner(&self) -> &P2PStream<ECIESStream<TcpStream>> {
        match self {
            Self::SilOnly(conn) => conn.inner(),
            Self::SilSnap(conn) => conn.inner(),
            Self::Satellite(conn) => conn.inner(),
        }
    }

    /// Same as [`Sink::start_send`] but accepts a [`SilBroadcastMessage`] instead.
    #[inline]
    pub fn start_send_broadcast(
        &mut self,
        item: SilBroadcastMessage<N>,
    ) -> Result<(), SilStreamError> {
        match self {
            Self::SilOnly(conn) => conn.start_send_broadcast(item),
            Self::SilSnap(conn) => conn.start_send_broadcast(item),
            Self::Satellite(conn) => conn.primary_mut().start_send_broadcast(item),
        }
    }

    /// Sends a raw capability message over the connection
    pub fn start_send_raw(&mut self, msg: RawCapabilityMessage) -> Result<(), SilStreamError> {
        match self {
            Self::SilOnly(conn) => conn.start_send_raw(msg),
            Self::SilSnap(conn) => conn.start_send_raw(msg),
            Self::Satellite(conn) => conn.primary_mut().start_send_raw(msg),
        }
    }

    /// Queues a `snap/2` message to be sent on the wire.
    ///
    /// Returns an error on connections that did not negotiate `snap/2`, so a caller never believes
    /// a request was sent when it was discarded.
    pub fn start_send_snap(&mut self, msg: SnapProtocolMessage) -> Result<(), SilStreamError> {
        match self {
            Self::SilSnap(conn) => conn.start_send_unpin(SilSnapMessage::Snap(msg)),
            Self::SilOnly(_) | Self::Satellite(_) => {
                Err(P2PStreamError::CapabilityNotShared.into())
            }
        }
    }

    /// Sets whether to reject block announcement messages (`NewBlock`, `NewBlockHashes`) before
    /// RLP decoding to avoid memory amplification from deserializing blocks that will be discarded.
    pub fn set_reject_block_announcements(&mut self, reject: bool) {
        match self {
            Self::SilOnly(conn) => conn.set_reject_block_announcements(reject),
            Self::SilSnap(conn) => conn.set_reject_block_announcements(reject),
            Self::Satellite(conn) => conn.primary_mut().set_reject_block_announcements(reject),
        }
    }
}

impl<N: NetworkPrimitives> From<SilPeerConnection<N>> for SilRlpxConnection<N> {
    #[inline]
    fn from(conn: SilPeerConnection<N>) -> Self {
        Self::SilOnly(Box::new(conn))
    }
}

impl<N: NetworkPrimitives> From<SilSnapConnection<N>> for SilRlpxConnection<N> {
    #[inline]
    fn from(conn: SilSnapConnection<N>) -> Self {
        Self::SilSnap(Box::new(conn))
    }
}

impl<N: NetworkPrimitives> From<SilSatelliteConnection<N>> for SilRlpxConnection<N> {
    #[inline]
    fn from(conn: SilSatelliteConnection<N>) -> Self {
        Self::Satellite(Box::new(conn))
    }
}

/// Delegates a call to the active variant's boxed stream (every variant is `Unpin`).
///
/// The second form runs `$adapt` on the sil-only variants to lift their result into the shared
/// item type; the snap variant already yields it.
macro_rules! delegate_call {
    ($self:ident.$method:ident($($args:ident),+)) => {
        match $self.get_mut() {
            Self::SilOnly(l) => l.$method($($args),+),
            Self::SilSnap(s) => s.$method($($args),+),
            Self::Satellite(r) => r.$method($($args),+),
        }
    };
    ($self:ident.$method:ident($($args:ident),+) => $adapt:expr) => {
        match $self.get_mut() {
            Self::SilOnly(l) => $adapt(l.$method($($args),+)),
            Self::Satellite(r) => $adapt(r.$method($($args),+)),
            Self::SilSnap(s) => s.$method($($args),+),
        }
    };
}

impl<N: NetworkPrimitives> Stream for SilRlpxConnection<N> {
    type Item = Result<SilSnapMessage<N>, SilStreamError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        delegate_call!(self.poll_next_unpin(cx) => lift_eth)
    }
}

impl<N: NetworkPrimitives> Sink<SilMessage<N>> for SilRlpxConnection<N> {
    type Error = SilStreamError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        delegate_call!(self.poll_ready_unpin(cx))
    }

    fn start_send(self: Pin<&mut Self>, item: SilMessage<N>) -> Result<(), Self::Error> {
        match self.get_mut() {
            Self::SilOnly(l) => l.start_send_unpin(item),
            Self::Satellite(r) => r.start_send_unpin(item),
            Self::SilSnap(s) => s.start_send_unpin(SilSnapMessage::Sil(item)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        delegate_call!(self.poll_flush_unpin(cx))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        delegate_call!(self.poll_close_unpin(cx))
    }
}

/// Lifts a polled `sil` item into the shared [`SilSnapMessage`] item type.
#[inline]
fn lift_eth<N: NetworkPrimitives>(
    poll: Poll<Option<Result<SilMessage<N>, SilStreamError>>>,
) -> Poll<Option<Result<SilSnapMessage<N>, SilStreamError>>> {
    poll.map(|opt| opt.map(|res| res.map(SilSnapMessage::Sil)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn assert_eth_stream<N, St>()
    where
        N: NetworkPrimitives,
        St: Stream<Item = Result<SilMessage<N>, SilStreamError>> + Sink<SilMessage<N>>,
    {
    }

    const fn assert_eth_snap_stream<N, St>()
    where
        N: NetworkPrimitives,
        St: Stream<Item = Result<SilSnapMessage<N>, SilStreamError>> + Sink<SilMessage<N>>,
    {
    }

    #[test]
    const fn test_eth_stream_variants() {
        assert_eth_stream::<SilNetworkPrimitives, SilSatelliteConnection<SilNetworkPrimitives>>();
        assert_eth_snap_stream::<SilNetworkPrimitives, SilRlpxConnection<SilNetworkPrimitives>>();
    }
}
