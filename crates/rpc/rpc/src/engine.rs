use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_primitives::{Address, Bytes, B256, U256, U64};
use alloy_rpc_types_eth::{
    state::StateOverride, BlockOverrides, SIP1186AccountProofResponse, Filter, Log, SyncStatus,
};
use alloy_serde::JsonStorageKey;
use jsonrpsee::core::RpcResult as Result;
use rsil_primitives_traits::TxTy;
use rsil_rpc_api::{EngineEthApiServer, SilApiServer};
use rsil_rpc_convert::RpcTxReq;
/// Re-export for convenience
pub use rsil_rpc_engine_api::EngineApi;
use rsil_rpc_eth_api::{
    EngineEthFilter, FullEthApiTypes, QueryLimits, RpcBlock, RpcHeader, RpcReceipt, RpcTransaction,
};
use serde_json::Value;
use tracing_futures::Instrument;

macro_rules! engine_span {
    () => {
        tracing::info_span!(target: "rpc", "engine")
    };
}

/// A wrapper type for the `SilApi` and `SilFilter` implementations that only expose the required
/// subset for the `eth_` namespace used in auth server alongside the `engine_` namespace.
#[derive(Debug, Clone)]
pub struct EngineEthApi<Sil, SilFilter> {
    sil: Sil,
    eth_filter: SilFilter,
}

impl<Sil, SilFilter> EngineEthApi<Sil, SilFilter> {
    /// Create a new `EngineEthApi` instance.
    pub const fn new(sil: Sil, eth_filter: SilFilter) -> Self {
        Self { sil, eth_filter }
    }
}

#[async_trait::async_trait]
impl<Sil, SilFilter>
    EngineEthApiServer<
        RpcTxReq<Sil::NetworkTypes>,
        RpcBlock<Sil::NetworkTypes>,
        RpcReceipt<Sil::NetworkTypes>,
    > for EngineEthApi<Sil, SilFilter>
where
    Sil: SilApiServer<
            RpcTxReq<Sil::NetworkTypes>,
            RpcTransaction<Sil::NetworkTypes>,
            RpcBlock<Sil::NetworkTypes>,
            RpcReceipt<Sil::NetworkTypes>,
            RpcHeader<Sil::NetworkTypes>,
            TxTy<Sil::Primitives>,
        > + FullEthApiTypes,
    SilFilter: EngineEthFilter,
{
    /// Handler for: `eth_syncing`
    fn syncing(&self) -> Result<SyncStatus> {
        let span = engine_span!();
        let _enter = span.enter();
        self.sil.syncing()
    }

    /// Handler for: `eth_chainId`
    async fn chain_id(&self) -> Result<Option<U64>> {
        let span = engine_span!();
        let _enter = span.enter();
        self.sil.chain_id().await
    }

    /// Handler for: `eth_blockNumber`
    fn block_number(&self) -> Result<U256> {
        let span = engine_span!();
        let _enter = span.enter();
        self.sil.block_number()
    }

    /// Handler for: `eth_call`
    async fn call(
        &self,
        request: RpcTxReq<Sil::NetworkTypes>,
        block_id: Option<BlockId>,
        state_overrides: Option<StateOverride>,
        block_overrides: Option<Box<BlockOverrides>>,
    ) -> Result<Bytes> {
        self.sil
            .call(request, block_id, state_overrides, block_overrides)
            .instrument(engine_span!())
            .await
    }

    /// Handler for: `eth_getCode`
    async fn get_code(&self, address: Address, block_id: Option<BlockId>) -> Result<Bytes> {
        self.sil.get_code(address, block_id).instrument(engine_span!()).await
    }

    /// Handler for: `eth_getBlockByHash`
    async fn block_by_hash(
        &self,
        hash: B256,
        full: bool,
    ) -> Result<Option<RpcBlock<Sil::NetworkTypes>>> {
        self.sil.block_by_hash(hash, full).instrument(engine_span!()).await
    }

    /// Handler for: `eth_getBlockByNumber`
    async fn block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> Result<Option<RpcBlock<Sil::NetworkTypes>>> {
        self.sil.block_by_number(number, full).instrument(engine_span!()).await
    }

    async fn block_receipts(
        &self,
        block_id: BlockId,
    ) -> Result<Option<Vec<RpcReceipt<Sil::NetworkTypes>>>> {
        self.sil.block_receipts(block_id).instrument(engine_span!()).await
    }

    /// Handler for: `eth_sendRawTransaction`
    async fn send_raw_transaction(&self, bytes: Bytes) -> Result<B256> {
        self.sil.send_raw_transaction(bytes).instrument(engine_span!()).await
    }

    async fn transaction_receipt(
        &self,
        hash: B256,
    ) -> Result<Option<RpcReceipt<Sil::NetworkTypes>>> {
        self.sil.transaction_receipt(hash).instrument(engine_span!()).await
    }

    /// Handler for `eth_getLogs`
    async fn logs(&self, filter: Filter) -> Result<Vec<Log>> {
        self.eth_filter.logs(filter, QueryLimits::no_limits()).instrument(engine_span!()).await
    }

    /// Handler for `eth_getProof`
    async fn get_proof(
        &self,
        address: Address,
        keys: Vec<JsonStorageKey>,
        block_number: Option<BlockId>,
    ) -> Result<SIP1186AccountProofResponse> {
        self.sil.get_proof(address, keys, block_number).instrument(engine_span!()).await
    }

    /// Handler for `eth_getBlockAccessListByBlockHash`
    async fn block_access_list_by_block_hash(&self, hash: B256) -> Result<Option<Value>> {
        self.sil.block_access_list_by_block_hash(hash).instrument(engine_span!()).await
    }

    /// Handler for `eth_getBlockAccessListByBlockNumber`
    async fn block_access_list_by_block_number(
        &self,
        block_number: BlockNumberOrTag,
    ) -> Result<Option<Value>> {
        self.sil.block_access_list_by_block_number(block_number).instrument(engine_span!()).await
    }

    /// Handler for `eth_getBlockAccessList`
    async fn block_access_list(&self, block_id: BlockId) -> Result<Option<Value>> {
        self.sil.block_access_list(block_id).instrument(engine_span!()).await
    }

    /// Handler for `getBlockAccessListRaw`
    async fn block_access_list_raw(&self, block: BlockId) -> Result<Option<Bytes>> {
        self.sil.block_access_list_raw(block).instrument(engine_span!()).await
    }
}
