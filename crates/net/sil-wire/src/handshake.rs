use crate::{
    errors::{SilHandshakeError, SilStreamError, P2PStreamError},
    message::MAX_MESSAGE_SIZE,
    CanDisconnect,
};
use bytes::{Bytes, BytesMut};
use futures::{Sink, SinkExt, Stream};
use rsil_eth_wire_types::{
    DisconnectReason, SilMessage, SilNetworkPrimitives, ProtocolMessage, StatusMessage,
    UnifiedStatus,
};
use rsil_sila_forks::ForkFilter;
use rsil_primitives_traits::GotExpected;
use std::{fmt::Debug, future::Future, pin::Pin, time::Duration};
use tokio::time::timeout;
use tokio_stream::StreamExt;
use tracing::{debug, trace};

/// A trait that knows how to perform the P2P handshake.
pub trait SilRlpxHandshake: Debug + Send + Sync + 'static {
    /// Perform the P2P handshake for the `sil` protocol.
    fn handshake<'a>(
        &'a self,
        unauth: &'a mut dyn UnauthEth,
        status: UnifiedStatus,
        fork_filter: ForkFilter,
        timeout_limit: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<UnifiedStatus, SilStreamError>> + 'a + Send>>;
}

/// An unauthenticated stream that can send and receive messages.
pub trait UnauthEth:
    Stream<Item = Result<BytesMut, P2PStreamError>>
    + Sink<Bytes, Error = P2PStreamError>
    + CanDisconnect<Bytes>
    + Unpin
    + Send
{
}

impl<T> UnauthEth for T where
    T: Stream<Item = Result<BytesMut, P2PStreamError>>
        + Sink<Bytes, Error = P2PStreamError>
        + CanDisconnect<Bytes>
        + Unpin
        + Send
{
}

/// The Sila P2P handshake.
///
/// This performs the regular sila `sil` rlpx handshake.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct SilHandshake;

impl SilRlpxHandshake for SilHandshake {
    fn handshake<'a>(
        &'a self,
        unauth: &'a mut dyn UnauthEth,
        status: UnifiedStatus,
        fork_filter: ForkFilter,
        timeout_limit: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<UnifiedStatus, SilStreamError>> + 'a + Send>> {
        Box::pin(async move {
            timeout(timeout_limit, SilaEthHandshake(unauth).eth_handshake(status, fork_filter))
                .await
                .map_err(|_| SilStreamError::StreamTimeout)?
        })
    }
}

/// A type that performs the sila specific `sil` protocol handshake.
#[derive(Debug)]
pub struct SilaEthHandshake<'a, S: ?Sized>(pub &'a mut S);

impl<S: ?Sized, E> SilaEthHandshake<'_, S>
where
    S: Stream<Item = Result<BytesMut, E>> + CanDisconnect<Bytes> + Send + Unpin,
    SilStreamError: From<E> + From<<S as Sink<Bytes>>::Error>,
{
    /// Performs the `sil` rlpx protocol handshake using the given input stream.
    pub async fn eth_handshake(
        self,
        unified_status: UnifiedStatus,
        fork_filter: ForkFilter,
    ) -> Result<UnifiedStatus, SilStreamError> {
        let unauth = self.0;

        let status = unified_status.into_message();

        // Send our status message
        let status_msg = alloy_rlp::encode(ProtocolMessage::<SilNetworkPrimitives>::from(
            SilMessage::Status(status),
        ))
        .into();
        unauth.send(status_msg).await.map_err(SilStreamError::from)?;

        // Receive peer's response
        let their_msg_res = unauth.next().await;
        let their_msg = match their_msg_res {
            Some(Ok(msg)) => msg,
            Some(Err(e)) => return Err(SilStreamError::from(e)),
            None => {
                unauth
                    .disconnect(DisconnectReason::DisconnectRequested)
                    .await
                    .map_err(SilStreamError::from)?;
                return Err(SilStreamError::SilHandshakeError(SilHandshakeError::NoResponse));
            }
        };

        if their_msg.len() > MAX_MESSAGE_SIZE {
            unauth
                .disconnect(DisconnectReason::ProtocolBreach)
                .await
                .map_err(SilStreamError::from)?;
            return Err(SilStreamError::MessageTooBig(their_msg.len()));
        }

        let version = status.version();
        let their_status_message = match ProtocolMessage::<SilNetworkPrimitives>::decode_status(
            version,
            &mut their_msg.as_ref(),
        ) {
            Ok(status) => status,
            Err(err) => {
                debug!("decode error in sil handshake: msg={their_msg:x}");
                unauth
                    .disconnect(DisconnectReason::ProtocolBreach)
                    .await
                    .map_err(SilStreamError::from)?;
                return Err(SilStreamError::InvalidMessage(err));
            }
        };

        trace!("Validating incoming SIL status from peer");

        if status.genesis() != their_status_message.genesis() {
            unauth
                .disconnect(DisconnectReason::ProtocolBreach)
                .await
                .map_err(SilStreamError::from)?;
            return Err(SilHandshakeError::MismatchedGenesis(
                GotExpected { expected: status.genesis(), got: their_status_message.genesis() }
                    .into(),
            )
            .into());
        }

        if status.version() != their_status_message.version() {
            unauth
                .disconnect(DisconnectReason::ProtocolBreach)
                .await
                .map_err(SilStreamError::from)?;
            return Err(SilHandshakeError::MismatchedProtocolVersion(GotExpected {
                got: their_status_message.version(),
                expected: status.version(),
            })
            .into());
        }

        if *status.chain() != *their_status_message.chain() {
            unauth
                .disconnect(DisconnectReason::ProtocolBreach)
                .await
                .map_err(SilStreamError::from)?;
            return Err(SilHandshakeError::MismatchedChain(GotExpected {
                got: *their_status_message.chain(),
                expected: *status.chain(),
            })
            .into());
        }

        // Ensure peer's total difficulty is reasonable
        if let StatusMessage::Legacy(s) = &their_status_message &&
            s.total_difficulty.bit_len() > 160
        {
            unauth
                .disconnect(DisconnectReason::ProtocolBreach)
                .await
                .map_err(SilStreamError::from)?;
            return Err(SilHandshakeError::TotalDifficultyBitLenTooLarge {
                got: s.total_difficulty.bit_len(),
                maximum: 160,
            }
            .into());
        }

        // Fork validation
        if let Err(err) = fork_filter
            .validate(their_status_message.forkid())
            .map_err(SilHandshakeError::InvalidFork)
        {
            unauth
                .disconnect(DisconnectReason::ProtocolBreach)
                .await
                .map_err(SilStreamError::from)?;
            return Err(err.into());
        }

        if let StatusMessage::Sil69(s) = &their_status_message {
            if s.earliest > s.latest {
                return Err(SilHandshakeError::EarliestBlockGreaterThanLatestBlock {
                    got: s.earliest,
                    latest: s.latest,
                }
                .into());
            }

            if s.blockhash.is_zero() {
                return Err(SilHandshakeError::BlockhashZero.into());
            }
        }

        Ok(UnifiedStatus::from_message(their_status_message))
    }
}
