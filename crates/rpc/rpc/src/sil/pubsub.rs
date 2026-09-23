//! `eth_` `PubSub` RPC handler implementation

use std::sync::Arc;

use alloy_primitives::TxHash;
use alloy_rpc_types_eth::{
    pubsub::{
        Params, PubSubSyncStatus, SubscriptionKind, SyncStatusMetadata, TransactionReceiptsParams,
    },
    Filter, Log,
};
use futures::StreamExt;
use jsonrpsee::{
    server::SubscriptionMessage, types::ErrorObject, PendingSubscriptionSink, SubscriptionSink,
};
use rsil_chain_state::CanonStateSubscriptions;
use rsil_network_api::NetworkInfo;
use rsil_rpc_convert::RpcHeader;
use rsil_rpc_eth_api::{
    helpers::SilSubscriptions, pubsub::SilPubSubApiServer, RpcConvert, RpcNodeCore, RpcTransaction,
};
use rsil_rpc_server_types::result::{internal_rpc_err, invalid_params_rpc_err};
use rsil_storage_api::BlockNumReader;
use rsil_tasks::Runtime;
use rsil_transaction_pool::{NewTransactionEvent, TransactionPool};
use serde::Serialize;
use tokio_stream::{
    wrappers::{BroadcastStream, ReceiverStream},
    Stream,
};
use tracing::error;

/// `Sil` pubsub RPC implementation.
///
/// This handles `eth_subscribe` RPC calls.
#[derive(Clone)]
pub struct SilPubSub<Sil> {
    /// All nested fields bundled together.
    inner: Arc<SilPubSubInner<Sil>>,
}

// === impl SilPubSub ===

impl<Sil> SilPubSub<Sil> {
    /// Creates a new, shareable instance.
    pub fn new(eth_api: Sil, subscription_task_spawner: Runtime) -> Self {
        let inner = SilPubSubInner { eth_api, subscription_task_spawner };
        Self { inner: Arc::new(inner) }
    }
}

impl<Sil> SilPubSub<Sil>
where
    Sil: SilSubscriptions,
{
    /// Returns the current sync status for the `syncing` subscription
    pub fn sync_status(&self, is_syncing: bool) -> PubSubSyncStatus {
        self.inner.sync_status(is_syncing)
    }

    /// Returns a stream that yields all transaction hashes emitted by the txpool.
    pub fn pending_transaction_hashes_stream(&self) -> impl Stream<Item = TxHash> {
        self.inner.pending_transaction_hashes_stream()
    }

    /// Returns a stream that yields all transactions emitted by the txpool.
    pub fn full_pending_transaction_stream(
        &self,
    ) -> impl Stream<Item = NewTransactionEvent<<Sil::Pool as TransactionPool>::Transaction>> {
        self.inner.full_pending_transaction_stream()
    }

    /// Returns a stream that yields new block headers.
    pub fn new_headers_stream(&self) -> impl Stream<Item = RpcHeader<Sil::NetworkTypes>> {
        self.inner.eth_api.header_stream()
    }

    /// Returns a stream that yields matching logs.
    pub fn log_stream(&self, filter: Filter) -> impl Stream<Item = Log> {
        self.inner.eth_api.log_stream(filter)
    }

    /// The actual handler for an accepted [`SilPubSub::subscribe`] call.
    pub async fn handle_accepted(
        &self,
        accepted_sink: SubscriptionSink,
        kind: SubscriptionKind,
        params: Option<Params>,
    ) -> Result<(), ErrorObject<'static>> {
        #[allow(unreachable_patterns)]
        match kind {
            SubscriptionKind::NewHeads => {
                pipe_from_stream(accepted_sink, self.new_headers_stream()).await
            }
            SubscriptionKind::Logs => {
                // if no params are provided, used default filter params
                let filter = match params {
                    Some(Params::Logs(filter)) => *filter,
                    Some(Params::Bool(_)) => {
                        return Err(invalid_params_rpc_err("Invalid params for logs"))
                    }
                    _ => Default::default(),
                };
                pipe_from_stream(accepted_sink, self.log_stream(filter)).await
            }
            SubscriptionKind::NewPendingTransactions => {
                if let Some(params) = params {
                    match params {
                        Params::Bool(true) => {
                            // full transaction objects requested
                            let stream = self.full_pending_transaction_stream().filter_map(|tx| {
                                let tx_value = match self
                                    .inner
                                    .eth_api
                                    .converter()
                                    .fill_pending(tx.transaction.to_consensus())
                                {
                                    Ok(tx) => Some(tx),
                                    Err(err) => {
                                        error!(target = "rpc",
                                            %err,
                                            "Failed to fill transaction with block context"
                                        );
                                        None
                                    }
                                };
                                std::future::ready(tx_value)
                            });
                            return pipe_from_stream(accepted_sink, stream).await;
                        }
                        Params::Bool(false) | Params::None => {
                            // only hashes requested
                        }
                        _ => {
                            return Err(invalid_params_rpc_err(
                                "Invalid params for newPendingTransactions",
                            ))
                        }
                    }
                }

                pipe_from_stream(accepted_sink, self.pending_transaction_hashes_stream()).await
            }
            SubscriptionKind::Syncing => {
                // get new block subscription
                let mut canon_state = BroadcastStream::new(
                    self.inner.eth_api.provider().subscribe_to_canonical_state(),
                );
                // get current sync status
                let mut initial_sync_status = self.inner.eth_api.network().is_syncing();
                let current_sub_res = self.sync_status(initial_sync_status);

                // send the current status immediately
                let msg = SubscriptionMessage::new(
                    accepted_sink.method_name(),
                    accepted_sink.subscription_id(),
                    &current_sub_res,
                )
                .map_err(SubscriptionSerializeError::new)?;

                if accepted_sink.send(msg).await.is_err() {
                    return Ok(());
                }

                while canon_state.next().await.is_some() {
                    let current_syncing = self.inner.eth_api.network().is_syncing();
                    // Only send a new response if the sync status has changed
                    if current_syncing != initial_sync_status {
                        // Update the sync status on each new block
                        initial_sync_status = current_syncing;

                        // send a new message now that the status changed
                        let sync_status = self.sync_status(current_syncing);
                        let msg = SubscriptionMessage::new(
                            accepted_sink.method_name(),
                            accepted_sink.subscription_id(),
                            &sync_status,
                        )
                        .map_err(SubscriptionSerializeError::new)?;

                        if accepted_sink.send(msg).await.is_err() {
                            break;
                        }
                    }
                }

                Ok(())
            }
            SubscriptionKind::TransactionReceipts => {
                let filter = match params {
                    Some(Params::TransactionReceipts(filter)) => filter,
                    None | Some(Params::None) => TransactionReceiptsParams::default(),
                    _ => {
                        return Err(invalid_params_rpc_err(
                            "Invalid params for transactionReceipts",
                        ))
                    }
                };

                pipe_from_stream(
                    accepted_sink,
                    self.inner.eth_api.transaction_receipts_stream(filter),
                )
                .await
            }
            _ => Err(invalid_params_rpc_err("Unsupported subscription kind")),
        }
    }
}

#[async_trait::async_trait]
impl<Sil> SilPubSubApiServer<RpcTransaction<Sil::NetworkTypes>> for SilPubSub<Sil>
where
    Sil: SilSubscriptions,
{
    /// Handler for `eth_subscribe`
    async fn subscribe(
        &self,
        pending: PendingSubscriptionSink,
        kind: SubscriptionKind,
        params: Option<Params>,
    ) -> jsonrpsee::core::SubscriptionResult {
        let sink = pending.accept().await?;
        let pubsub = self.clone();
        self.inner.subscription_task_spawner.spawn_task(async move {
            let _ = pubsub.handle_accepted(sink, kind, params).await;
        });

        Ok(())
    }
}

/// Helper to convert a serde error into an [`ErrorObject`]
#[derive(Debug, thiserror::Error)]
#[error("Failed to serialize subscription item: {0}")]
pub struct SubscriptionSerializeError(#[from] serde_json::Error);

impl SubscriptionSerializeError {
    const fn new(err: serde_json::Error) -> Self {
        Self(err)
    }
}

impl From<SubscriptionSerializeError> for ErrorObject<'static> {
    fn from(value: SubscriptionSerializeError) -> Self {
        internal_rpc_err(value.to_string())
    }
}

/// Pipes all stream items to the subscription sink.
async fn pipe_from_stream<T, St>(
    sink: SubscriptionSink,
    mut stream: St,
) -> Result<(), ErrorObject<'static>>
where
    St: Stream<Item = T> + Unpin,
    T: Serialize,
{
    loop {
        tokio::select! {
            _ = sink.closed() => {
                // connection dropped
                break Ok(())
            },
            maybe_item = stream.next() => {
                let item = match maybe_item {
                    Some(item) => item,
                    None => {
                        // stream ended
                        break  Ok(())
                    },
                };
                let msg = SubscriptionMessage::new(
                    sink.method_name(),
                    sink.subscription_id(),
                    &item
                ).map_err(SubscriptionSerializeError::new)?;

                if sink.send(msg).await.is_err() {
                    break Ok(());
                }
            }
        }
    }
}

impl<Sil> std::fmt::Debug for SilPubSub<Sil> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SilPubSub").finish_non_exhaustive()
    }
}

/// Container type `SilPubSub`
#[derive(Clone)]
struct SilPubSubInner<SilApi> {
    /// The `sil` API.
    eth_api: SilApi,
    /// The type that's used to spawn subscription tasks.
    subscription_task_spawner: Runtime,
}

// == impl SilPubSubInner ===

impl<Sil> SilPubSubInner<Sil>
where
    Sil: RpcNodeCore<Provider: BlockNumReader>,
{
    /// Returns the current sync status for the `syncing` subscription
    fn sync_status(&self, is_syncing: bool) -> PubSubSyncStatus {
        if is_syncing {
            let current_block = self
                .eth_api
                .provider()
                .chain_info()
                .map(|info| info.best_number)
                .unwrap_or_default();
            PubSubSyncStatus::Detailed(SyncStatusMetadata {
                syncing: true,
                starting_block: 0,
                current_block,
                highest_block: Some(current_block),
            })
        } else {
            PubSubSyncStatus::Simple(false)
        }
    }
}

impl<Sil> SilPubSubInner<Sil>
where
    Sil: RpcNodeCore<Pool: TransactionPool>,
{
    /// Returns a stream that yields all transaction hashes emitted by the txpool.
    fn pending_transaction_hashes_stream(&self) -> impl Stream<Item = TxHash> {
        ReceiverStream::new(self.eth_api.pool().pending_transactions_listener())
    }

    /// Returns a stream that yields all transactions emitted by the txpool.
    fn full_pending_transaction_stream(
        &self,
    ) -> impl Stream<Item = NewTransactionEvent<<Sil::Pool as TransactionPool>::Transaction>> {
        self.eth_api.pool().new_pending_pool_transactions_listener()
    }
}
