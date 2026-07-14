//! Contains RPC handler implementations specific to endpoints that call/execute within savm.

use crate::SilApi;
use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{
    helpers::{estimate::EstimateCall, Call, SilCall},
    FromEvmError, RpcNodeCore,
};
use rsil_rpc_eth_types::SilApiError;

impl<N, Rpc> SilCall for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError, Savm = N::Savm>,
{
}

impl<N, Rpc> Call for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError, Savm = N::Savm>,
{
    #[inline]
    fn call_gas_limit(&self) -> u64 {
        self.inner.gas_cap()
    }

    #[inline]
    fn max_simulate_blocks(&self) -> u64 {
        self.inner.max_simulate_blocks()
    }

    #[inline]
    fn compute_state_root_for_eth_simulate(&self) -> bool {
        self.inner.compute_state_root_for_eth_simulate()
    }

    #[inline]
    fn evm_memory_limit(&self) -> u64 {
        self.inner.evm_memory_limit()
    }
}

impl<N, Rpc> EstimateCall for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError, Savm = N::Savm>,
{
}
