use alloy_eips::BlockId;
use alloy_primitives::{map::AddressMap, U256, U64};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde::{Deserialize, Serialize};

// Required for the subscription attributes below
use rsil_chain_state as _;

/// Rsil API namespace for rsil-specific methods
#[cfg_attr(not(feature = "client"), rpc(server, namespace = "rsil"))]
#[cfg_attr(feature = "client", rpc(server, client, namespace = "rsil"))]
pub trait RsilApi {
    /// Returns all SIL balance changes in a block
    #[method(name = "getBalanceChangesInBlock")]
    async fn rsil_get_balance_changes_in_block(
        &self,
        block_id: BlockId,
    ) -> RpcResult<AddressMap<U256>>;

    /// Re-executes a block (or a range of blocks) and returns the execution outcome including
    /// receipts, state changes, and SIP-7685 requests.
    ///
    /// If `count` is provided, re-executes `count` consecutive blocks starting from `block_id`
    /// and returns the merged execution outcome.
    #[method(name = "getBlockExecutionOutcome")]
    async fn rsil_get_block_execution_outcome(
        &self,
        block_id: BlockId,
        count: Option<U64>,
    ) -> RpcResult<Option<serde_json::Value>>;

    /// Controls the revmc JIT backend.
    #[method(name = "jit")]
    async fn rsil_jit(&self, action: RsilJitAction) -> RpcResult<()>;

    /// Subscribe to json `ChainNotifications`
    #[subscription(
        name = "subscribeChainNotifications",
        unsubscribe = "unsubscribeChainNotifications",
        item = rsil_chain_state::CanonStateNotification
    )]
    async fn rsil_subscribe_chain_notifications(&self) -> jsonrpsee::core::SubscriptionResult;

    /// Subscribe to persisted block notifications.
    ///
    /// Emits a notification with the block number and hash when a new block is persisted to disk.
    #[subscription(
        name = "subscribePersistedBlock",
        unsubscribe = "unsubscribePersistedBlock",
        item = alloy_eips::BlockNumHash
    )]
    async fn rsil_subscribe_persisted_block(&self) -> jsonrpsee::core::SubscriptionResult;

    /// Subscribe to finalized chain notifications.
    ///
    /// Buffers committed chain notifications and emits them once a new finalized block is received.
    /// Each notification contains all committed chain segments up to the finalized block.
    #[subscription(
        name = "subscribeFinalizedChainNotifications",
        unsubscribe = "unsubscribeFinalizedChainNotifications",
        item = Vec<rsil_chain_state::CanonStateNotification>
    )]
    async fn rsil_subscribe_finalized_chain_notifications(
        &self,
    ) -> jsonrpsee::core::SubscriptionResult;
}

/// Supported `rsil_jit` control actions.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RsilJitAction {
    /// Enable JIT compilation for the backend.
    Enable,
    /// Disable JIT compilation for the backend.
    Disable,
    /// Pause background JIT compilation.
    Pause,
    /// Resume background JIT compilation.
    Unpause,
    /// Clear resident and persisted JIT artifacts.
    Clear,
}
