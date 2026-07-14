//! Contains RPC handler implementations specific to block access lists.

use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{helpers::bal::GetBlockAccessList, FromEvmError, RpcNodeCore};
use rsil_rpc_eth_types::SilApiError;

use crate::SilApi;

impl<N, Rpc> GetBlockAccessList for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError, Savm = N::Savm>,
{
}
