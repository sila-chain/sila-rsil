//! Contains RPC handler implementations for fee history.

use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{
    helpers::{LoadFee, SilFees},
    FromEvmError, RpcNodeCore,
};
use rsil_rpc_eth_types::{FeeHistoryCache, GasPriceOracle, SilApiError};
use rsil_storage_api::ProviderHeader;

use crate::SilApi;

impl<N, Rpc> SilFees for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
{
}

impl<N, Rpc> LoadFee for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    SilApiError: FromEvmError<N::Savm>,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
{
    #[inline]
    fn gas_oracle(&self) -> &GasPriceOracle<Self::Provider> {
        self.inner.gas_oracle()
    }

    #[inline]
    fn fee_history_cache(&self) -> &FeeHistoryCache<ProviderHeader<N::Provider>> {
        self.inner.fee_history_cache()
    }
}
