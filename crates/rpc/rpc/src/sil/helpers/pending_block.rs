//! Support for building a pending block with transactions from local view of mempool.

use crate::SilApi;
use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{
    helpers::{pending_block::PendingEnvBuilder, LoadPendingBlock},
    FromEvmError, RpcNodeCore,
};
use rsil_rpc_eth_types::{builder::config::PendingBlockKind, PendingBlock, SilApiError};

impl<N, Rpc> LoadPendingBlock for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
{
    #[inline]
    fn pending_block(&self) -> &tokio::sync::Mutex<Option<PendingBlock<Self::Primitives>>> {
        self.inner.pending_block()
    }

    #[inline]
    fn pending_env_builder(&self) -> &dyn PendingEnvBuilder<Self::Savm> {
        self.inner.pending_env_builder()
    }

    #[inline]
    fn pending_block_kind(&self) -> PendingBlockKind {
        self.inner.pending_block_kind()
    }
}
