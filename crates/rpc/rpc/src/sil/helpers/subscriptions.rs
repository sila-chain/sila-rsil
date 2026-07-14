//! Contains RPC handler implementations specific to streams subscriptions.

use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{helpers::SilSubscriptions, RpcNodeCore};
use rsil_rpc_eth_types::SilApiError;

use crate::SilApi;

impl<N, Rpc> SilSubscriptions for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
{
}
