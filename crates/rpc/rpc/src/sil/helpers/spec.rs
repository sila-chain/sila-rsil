use alloy_primitives::U256;
use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{helpers::SilApiSpec, RpcNodeCore};
use rsil_rpc_eth_types::SilApiError;

use crate::SilApi;

impl<N, Rpc> SilApiSpec for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
{
    fn starting_block(&self) -> U256 {
        self.inner.starting_block()
    }
}
