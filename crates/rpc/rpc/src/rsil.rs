use std::{future::Future, sync::Arc};

use alloy_consensus::BlockHeader;
use alloy_eips::BlockId;
use alloy_primitives::{map::AddressMap, U256, U64};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use jsonrpsee::{core::RpcResult, PendingSubscriptionSink, SubscriptionMessage, SubscriptionSink};
use rsil_chain_state::{
    CanonStateNotification, CanonStateSubscriptions, ForkChoiceSubscriptions,
    PersistedBlockSubscriptions,
};
use rsil_errors::{RsilError, RsilResult};
use rsil_evm::{execute::Executor, ConfigureEvm};
use rsil_execution_types::ExecutionOutcome;
use rsil_primitives_traits::{NodePrimitives, SealedHeader};
use rsil_rpc_api::{RsilApiServer, RsilJitAction};
use rsil_rpc_eth_types::{SilApiError, SilResult};
use rsil_storage_api::{
    BlockReader, BlockReaderIdExt, ChangeSetReader, StateProviderFactory, TransactionVariant,
};
use rsil_tasks::{pool::BlockingTaskGuard, Runtime};
use serde::Serialize;
use tokio::sync::oneshot;

/// `rsil` API implementation.
///
/// This type provides the functionality for handling `rsil` prototype RPC requests.
pub struct RsilApi<Provider, SavmConfig> {
    inner: Arc<RsilApiInner<Provider, SavmConfig>>,
}

// === impl RsilApi ===

impl<Provider, SavmConfig> RsilApi<Provider, SavmConfig> {
    /// The provider that can interact with the chain.
    pub fn provider(&self) -> &Provider {
        &self.inner.provider
    }

    /// The savm config.
    pub fn evm_config(&self) -> &SavmConfig {
        &self.inner.evm_config
    }

    /// Create a new instance of the [`RsilApi`]
    pub fn new(
        provider: Provider,
        evm_config: SavmConfig,
        blocking_task_guard: BlockingTaskGuard,
        task_spawner: Runtime,
    ) -> Self {
        let inner =
            Arc::new(RsilApiInner { provider, evm_config, blocking_task_guard, task_spawner });
        Self { inner }
    }
}

impl<Provider, SavmConfig> RsilApi<Provider, SavmConfig>
where
    Provider: BlockReaderIdExt + ChangeSetReader + StateProviderFactory + 'static,
    SavmConfig: Send + Sync + 'static,
{
    /// Executes the future on a new blocking task.
    async fn on_blocking_task<C, F, R>(&self, c: C) -> SilResult<R>
    where
        C: FnOnce(Self) -> F,
        F: Future<Output = SilResult<R>> + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let this = self.clone();
        let f = c(this);
        self.inner.task_spawner.spawn_blocking_task(async move {
            let res = f.await;
            let _ = tx.send(res);
        });
        rx.await.map_err(|_| SilApiError::InternalEthError)?
    }

    /// Returns a map of addresses to changed account balanced for a particular block.
    pub async fn balance_changes_in_block(&self, block_id: BlockId) -> SilResult<AddressMap<U256>> {
        self.on_blocking_task(async move |this| this.try_balance_changes_in_block(block_id)).await
    }

    fn try_balance_changes_in_block(&self, block_id: BlockId) -> SilResult<AddressMap<U256>> {
        let Some(block_number) = self.provider().block_number_for_id(block_id)? else {
            return Err(SilApiError::HeaderNotFound(block_id))
        };

        let state = self.provider().state_by_block_id(block_id)?;
        let accounts_before = self.provider().account_block_changeset(block_number)?;
        let hash_map = accounts_before.iter().try_fold(
            AddressMap::default(),
            |mut hash_map, account_before| -> RsilResult<_> {
                let current_balance = state.account_balance(&account_before.address)?;
                let prev_balance = account_before.info.map(|info| info.balance);
                if current_balance != prev_balance {
                    hash_map.insert(account_before.address, current_balance.unwrap_or_default());
                }
                Ok(hash_map)
            },
        )?;
        Ok(hash_map)
    }
}

impl<N, Provider, SavmConfig> RsilApi<Provider, SavmConfig>
where
    N: NodePrimitives,
    Provider: BlockReaderIdExt
        + ChangeSetReader
        + StateProviderFactory
        + BlockReader<Block = N::Block>
        + CanonStateSubscriptions<Primitives = N>
        + 'static,
    SavmConfig: ConfigureEvm<Primitives = N> + 'static,
{
    /// Re-executes one or more consecutive blocks and returns the execution outcome.
    pub async fn block_execution_outcome(
        &self,
        block_id: BlockId,
        count: Option<U64>,
    ) -> SilResult<Option<ExecutionOutcome<N::Receipt>>> {
        const MAX_BLOCK_COUNT: u64 = 128;

        let block_count = count.map(|c| c.to::<u64>()).unwrap_or(1);
        if block_count == 0 || block_count > MAX_BLOCK_COUNT {
            return Err(SilApiError::InvalidParams(format!(
                "block count must be between 1 and {MAX_BLOCK_COUNT}, got {block_count}"
            )))
        }

        let permit = self
            .inner
            .blocking_task_guard
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| SilApiError::InternalEthError)?;
        self.on_blocking_task(async move |this| {
            let _permit = permit;
            this.try_block_execution_outcome(block_id, block_count)
        })
        .await
    }

    fn try_block_execution_outcome(
        &self,
        block_id: BlockId,
        block_count: u64,
    ) -> SilResult<Option<ExecutionOutcome<N::Receipt>>> {
        let Some(start_block) = self.provider().block_number_for_id(block_id)? else {
            return Ok(None)
        };

        if start_block == 0 {
            return Ok(Some(ExecutionOutcome::default()))
        }

        let state_provider = self.provider().history_by_block_number(start_block - 1)?;
        let db = rsil_revm::database::StateProviderDatabase::new(&state_provider);

        let mut blocks = Vec::with_capacity(block_count as usize);
        for block_number in start_block..start_block + block_count {
            let Some(block) = self
                .provider()
                .recovered_block(block_number.into(), TransactionVariant::WithHash)?
            else {
                if block_number == start_block {
                    return Ok(None)
                }
                break;
            };
            blocks.push(block);
        }

        let outcome = self.evm_config().executor(db).execute_batch(&blocks).map_err(
            |e: rsil_evm::execute::BlockExecutionError| {
                SilApiError::Internal(rsil_errors::RsilError::Other(e.into()))
            },
        )?;

        Ok(Some(outcome))
    }
}

#[async_trait]
impl<Provider, SavmConfig> RsilApiServer for RsilApi<Provider, SavmConfig>
where
    Provider: BlockReaderIdExt
        + ChangeSetReader
        + StateProviderFactory
        + BlockReader<Block = <Provider::Primitives as NodePrimitives>::Block>
        + CanonStateSubscriptions
        + ForkChoiceSubscriptions<Header = <Provider::Primitives as NodePrimitives>::BlockHeader>
        + PersistedBlockSubscriptions
        + 'static,
    SavmConfig: ConfigureEvm<Primitives = Provider::Primitives> + 'static,
{
    /// Handler for `rsil_getBalanceChangesInBlock`
    async fn rsil_get_balance_changes_in_block(
        &self,
        block_id: BlockId,
    ) -> RpcResult<AddressMap<U256>> {
        Ok(Self::balance_changes_in_block(self, block_id).await?)
    }

    /// Handler for `rsil_getBlockExecutionOutcome`
    async fn rsil_get_block_execution_outcome(
        &self,
        block_id: BlockId,
        count: Option<U64>,
    ) -> RpcResult<Option<serde_json::Value>> {
        let outcome = Self::block_execution_outcome(self, block_id, count).await?;
        match outcome {
            Some(outcome) => {
                let value = serde_json::to_value(&outcome).map_err(|e| {
                    SilApiError::Internal(rsil_errors::RsilError::msg(e.to_string()))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Handler for `rsil_jit`
    async fn rsil_jit(&self, action: RsilJitAction) -> RpcResult<()> {
        let Some(jit_backend) = self.evm_config().jit_backend() else {
            return Ok(());
        };

        match action {
            RsilJitAction::Enable => jit_backend
                .set_enabled(true)
                .map_err(|err| SilApiError::Internal(RsilError::msg(err)))?,
            RsilJitAction::Disable => jit_backend
                .set_enabled(false)
                .map_err(|err| SilApiError::Internal(RsilError::msg(err)))?,
            RsilJitAction::Pause => jit_backend.pause(),
            RsilJitAction::Unpause => jit_backend.resume(),
            RsilJitAction::Clear => jit_backend.clear(),
        }

        Ok(())
    }

    /// Handler for `rsil_subscribeChainNotifications`
    async fn rsil_subscribe_chain_notifications(
        &self,
        pending: PendingSubscriptionSink,
    ) -> jsonrpsee::core::SubscriptionResult {
        let sink = pending.accept().await?;
        let stream = self.provider().canonical_state_stream();
        self.inner.task_spawner.spawn_task(pipe_from_stream(sink, stream));

        Ok(())
    }

    /// Handler for `rsil_subscribePersistedBlock`
    async fn rsil_subscribe_persisted_block(
        &self,
        pending: PendingSubscriptionSink,
    ) -> jsonrpsee::core::SubscriptionResult {
        let sink = pending.accept().await?;
        let stream = self.provider().persisted_block_stream();
        self.inner.task_spawner.spawn_task(pipe_from_stream(sink, stream));

        Ok(())
    }

    /// Handler for `rsil_subscribeFinalizedChainNotifications`
    async fn rsil_subscribe_finalized_chain_notifications(
        &self,
        pending: PendingSubscriptionSink,
    ) -> jsonrpsee::core::SubscriptionResult {
        let sink = pending.accept().await?;
        let canon_stream = self.provider().canonical_state_stream();
        let finalized_stream = self.provider().finalized_block_stream();
        self.inner.task_spawner.spawn_task(finalized_chain_notifications(
            sink,
            canon_stream,
            finalized_stream,
        ));

        Ok(())
    }
}

/// Pipes all stream items to the subscription sink.
async fn pipe_from_stream<S, T>(sink: SubscriptionSink, mut stream: S)
where
    S: Stream<Item = T> + Unpin,
    T: Serialize,
{
    loop {
        tokio::select! {
            _ = sink.closed() => {
                break
            }
            maybe_item = stream.next() => {
                let Some(item) = maybe_item else {
                    break
                };
                let msg = match SubscriptionMessage::new(sink.method_name(), sink.subscription_id(), &item) {
                    Ok(msg) => msg,
                    Err(err) => {
                        tracing::error!(target: "rpc::rsil", %err, "Failed to serialize subscription message");
                        break
                    }
                };
                if sink.send(msg).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// Buffers committed chain notifications and emits them when a new finalized block is received.
async fn finalized_chain_notifications<N>(
    sink: SubscriptionSink,
    mut canon_stream: rsil_chain_state::CanonStateNotificationStream<N>,
    mut finalized_stream: rsil_chain_state::ForkChoiceStream<SealedHeader<N::BlockHeader>>,
) where
    N: NodePrimitives,
{
    let mut buffered: Vec<CanonStateNotification<N>> = Vec::new();

    loop {
        tokio::select! {
            _ = sink.closed() => {
                break
            }
            maybe_canon = canon_stream.next() => {
                let Some(notification) = maybe_canon else { break };
                match &notification {
                    CanonStateNotification::Commit { .. } => {
                        buffered.push(notification);
                    }
                    CanonStateNotification::Reorg { .. } => {
                        buffered.clear();
                    }
                }
            }
            maybe_finalized = finalized_stream.next() => {
                let Some(finalized_header) = maybe_finalized else { break };
                let finalized_num = finalized_header.number();

                let mut committed = Vec::new();
                buffered.retain(|n| {
                    if *n.committed().range().end() <= finalized_num {
                        committed.push(n.clone());
                        false
                    } else {
                        true
                    }
                });

                if committed.is_empty() {
                    continue;
                }

                committed.sort_by_key(|n| *n.committed().range().start());

                let msg = match SubscriptionMessage::new(
                    sink.method_name(),
                    sink.subscription_id(),
                    &committed,
                ) {
                    Ok(msg) => msg,
                    Err(err) => {
                        tracing::error!(target: "rpc::rsil", %err, "Failed to serialize finalized chain notification");
                        break
                    }
                };
                if sink.send(msg).await.is_err() {
                    break;
                }
            }
        }
    }
}

impl<Provider, SavmConfig> std::fmt::Debug for RsilApi<Provider, SavmConfig> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RsilApi").finish_non_exhaustive()
    }
}

impl<Provider, SavmConfig> Clone for RsilApi<Provider, SavmConfig> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

struct RsilApiInner<Provider, SavmConfig> {
    /// The provider that can interact with the chain.
    provider: Provider,
    /// The SAVM configuration used to create block executors.
    evm_config: SavmConfig,
    /// Guard to restrict the number of concurrent block re-execution requests.
    blocking_task_guard: BlockingTaskGuard,
    /// The type that can spawn tasks which would otherwise block.
    task_spawner: Runtime,
}
