use crate::upgrade_status::{UpgradeStatus, UpgradeStatusExtension};
use alloy_rlp::Decodable;
use futures::SinkExt;
use rsil_eth_wire::{
    errors::{SilHandshakeError, SilStreamError},
    handshake::{SilRlpxHandshake, SilaEthHandshake, UnauthEth},
    UnifiedStatus,
};
use rsil_eth_wire_types::{DisconnectReason, SilVersion};
use rsil_sila_forks::ForkFilter;
use std::{future::Future, pin::Pin};
use tokio::time::{timeout, Duration};
use tokio_stream::StreamExt;
use tracing::debug;

#[derive(Debug, Default)]
/// The Binance Smart Chain (BSC) P2P handshake.
#[non_exhaustive]
pub struct BscHandshake;

impl BscHandshake {
    /// Negotiate the upgrade status message.
    pub async fn upgrade_status(
        unauth: &mut dyn UnauthEth,
        negotiated_status: UnifiedStatus,
    ) -> Result<UnifiedStatus, SilStreamError> {
        if negotiated_status.version > SilVersion::Sil66 {
            // Send upgrade status message allowing peer to broadcast transactions
            let upgrade_msg = UpgradeStatus {
                extension: UpgradeStatusExtension { disable_peer_tx_broadcast: false },
            };
            unauth.start_send_unpin(upgrade_msg.into_rlpx())?;

            // Receive peer's upgrade status response
            let their_msg = match unauth.next().await {
                Some(Ok(msg)) => msg,
                Some(Err(e)) => return Err(SilStreamError::from(e)),
                None => {
                    unauth.disconnect(DisconnectReason::DisconnectRequested).await?;
                    return Err(SilStreamError::SilHandshakeError(SilHandshakeError::NoResponse));
                }
            };

            // Decode their response
            match UpgradeStatus::decode(&mut their_msg.as_ref()).map_err(|e| {
                debug!("Decode error in BSC handshake: msg={their_msg:x}");
                SilStreamError::InvalidMessage(e.into())
            }) {
                Ok(_) => {
                    // Successful handshake
                    return Ok(negotiated_status);
                }
                Err(_) => {
                    unauth.disconnect(DisconnectReason::ProtocolBreach).await?;
                    return Err(SilStreamError::SilHandshakeError(
                        SilHandshakeError::NonStatusMessageInHandshake,
                    ));
                }
            }
        }

        Ok(negotiated_status)
    }
}

impl SilRlpxHandshake for BscHandshake {
    fn handshake<'a>(
        &'a self,
        unauth: &'a mut dyn UnauthEth,
        status: UnifiedStatus,
        fork_filter: ForkFilter,
        timeout_limit: Duration,
    ) -> Pin<Box<dyn Future<Output = Result<UnifiedStatus, SilStreamError>> + 'a + Send>> {
        Box::pin(async move {
            let fut = async {
                let negotiated_status =
                    SilaEthHandshake(unauth).eth_handshake(status, fork_filter).await?;
                Self::upgrade_status(unauth, negotiated_status).await
            };
            timeout(timeout_limit, fut).await.map_err(|_| SilStreamError::StreamTimeout)?
        })
    }
}
