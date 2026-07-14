//! Contains RPC handler implementations specific to tracing.

use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{helpers::Trace, FromEvmError, RpcNodeCore};
use rsil_rpc_eth_types::SilApiError;

use crate::SilApi;

impl<N, Rpc> Trace for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError, Savm = N::Savm>,
{
}
