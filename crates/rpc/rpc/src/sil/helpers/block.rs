//! Contains RPC handler implementations specific to blocks.

use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{
    helpers::{SilBlocks, LoadBlock, LoadPendingBlock},
    FromEvmError, RpcNodeCore,
};
use rsil_rpc_eth_types::SilApiError;

use crate::SilApi;

impl<N, Rpc> SilBlocks for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
{
}

impl<N, Rpc> LoadBlock for SilApi<N, Rpc>
where
    Self: LoadPendingBlock,
    N: RpcNodeCore,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
{
}
