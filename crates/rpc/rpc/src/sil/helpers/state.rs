//! Contains RPC handler implementations specific to state.

use crate::SilApi;
use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_api::{
    helpers::{LoadPendingBlock, LoadState, SilState},
    RpcNodeCore,
};
use rsil_rpc_eth_types::SilApiError;

impl<N, Rpc> SilState for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    Rpc: RpcConvert<Primitives = N::Primitives, Error = SilApiError>,
    Self: LoadPendingBlock,
{
    fn max_proof_window(&self) -> u64 {
        self.inner.eth_proof_window()
    }
}

impl<N, Rpc> LoadState for SilApi<N, Rpc>
where
    N: RpcNodeCore,
    Rpc: RpcConvert<Primitives = N::Primitives>,
    Self: LoadPendingBlock,
{
}

#[cfg(test)]
mod tests {
    use crate::sil::helpers::types::SilRpcConverter;

    use super::*;
    use alloy_primitives::{
        map::{AddressMap, B256Map},
        Address, StorageKey, StorageValue, U256,
    };
    use rsil_chainspec::ChainSpec;
    use rsil_evm_sila::SilEvmConfig;
    use rsil_network_api::noop::NoopNetwork;
    use rsil_provider::{
        test_utils::{ExtendedAccount, MockEthProvider, NoopProvider},
        ChainSpecProvider,
    };
    use rsil_rpc_eth_api::{helpers::SilState, node::RpcNodeCoreAdapter};
    use rsil_transaction_pool::test_utils::{testing_pool, TestPool};

    fn noop_eth_api() -> SilApi<
        RpcNodeCoreAdapter<NoopProvider, TestPool, NoopNetwork, SilEvmConfig>,
        SilRpcConverter<ChainSpec>,
    > {
        let provider = NoopProvider::default();
        let pool = testing_pool();
        let evm_config = SilEvmConfig::sila_mainnet();

        SilApi::builder(provider, pool, NoopNetwork::default(), evm_config).build()
    }

    fn mock_eth_api(
        accounts: AddressMap<ExtendedAccount>,
    ) -> SilApi<
        RpcNodeCoreAdapter<MockEthProvider, TestPool, NoopNetwork, SilEvmConfig>,
        SilRpcConverter<ChainSpec>,
    > {
        let pool = testing_pool();
        let mock_provider = MockEthProvider::default();

        let evm_config = SilEvmConfig::new(mock_provider.chain_spec());
        mock_provider.extend_accounts(accounts);

        SilApi::builder(mock_provider, pool, NoopNetwork::default(), evm_config).build()
    }

    #[tokio::test]
    async fn test_storage() {
        // === Noop ===
        let eth_api = noop_eth_api();
        let address = Address::random();
        let storage = eth_api.storage_at(address, U256::ZERO.into(), None).await.unwrap();
        assert_eq!(storage, U256::ZERO.to_be_bytes());

        // === Mock ===
        let storage_value = StorageValue::from(1337);
        let storage_key = StorageKey::random();
        let storage: B256Map<_> = core::iter::once((storage_key, storage_value)).collect();

        let accounts = AddressMap::from_iter([(
            address,
            ExtendedAccount::new(0, U256::ZERO).extend_storage(storage),
        )]);
        let eth_api = mock_eth_api(accounts);

        let storage_key: U256 = storage_key.into();
        let storage = eth_api.storage_at(address, storage_key.into(), None).await.unwrap();
        assert_eq!(storage, storage_value.to_be_bytes());
    }

    #[tokio::test]
    async fn test_get_account_missing() {
        let eth_api = noop_eth_api();
        let address = Address::random();
        let account = eth_api.get_account(address, Default::default()).await.unwrap();
        assert!(account.is_none());
    }
}
