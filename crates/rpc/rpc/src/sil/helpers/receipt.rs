//! Builds an RPC receipt response w.r.t. data layout of network.

use crate::SilApi;
use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{helpers::LoadReceipt, FromEvmError, RpcNodeCore};
use rsil_rpc_eth_types::SilApiError;

impl<N, Rpc> LoadReceipt for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
{
}
