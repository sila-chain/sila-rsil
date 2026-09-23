//! Helper trait for interfacing with [`FullNodeComponents`].

use rsil_chain_state::CanonStateSubscriptions;
use rsil_chainspec::{ChainSpecProvider, Hardforks, SilChainSpec, SilaHardforks};
use rsil_evm::ConfigureEvm;
use rsil_network_api::NetworkInfo;
use rsil_node_api::{FullNodeComponents, NodePrimitives, PrimitivesTy};
use rsil_primitives_traits::{BlockTy, HeaderTy, ReceiptTy, TxTy};
use rsil_rpc_eth_types::SilStateCache;
use rsil_storage_api::{
    BalProvider, BlockReader, BlockReaderIdExt, PruneCheckpointReader, StageCheckpointReader,
    StateProviderFactory,
};
use rsil_transaction_pool::{PoolTransaction, TransactionPool};

/// Helper trait that provides the same interface as [`FullNodeComponents`] but without requiring
/// implementation of trait bounds.
///
/// This trait is structurally equivalent to [`FullNodeComponents`], exposing the same associated
/// types and methods. However, it doesn't enforce the trait bounds required by
/// [`FullNodeComponents`]. This makes it useful for RPC types that need access to node components
/// where the full trait bounds of the components are not necessary.
///
/// Every type that is a [`FullNodeComponents`] also implements this trait.
pub trait RpcNodeCore: Clone + Send + Sync + Unpin + 'static {
    /// Blockchain data primitives.
    type Primitives: NodePrimitives;
    /// The provider type used to interact with the node.
    type Provider: BlockReaderIdExt<
            Block = BlockTy<Self::Primitives>,
            Receipt = ReceiptTy<Self::Primitives>,
            Header = HeaderTy<Self::Primitives>,
            Transaction = TxTy<Self::Primitives>,
        > + ChainSpecProvider<
            ChainSpec: SilChainSpec<Header = HeaderTy<Self::Primitives>>
                           + Hardforks
                           + SilaHardforks,
        > + StateProviderFactory
        + CanonStateSubscriptions<Primitives = Self::Primitives>
        + StageCheckpointReader
        + PruneCheckpointReader
        + BalProvider
        + Send
        + Sync
        + Clone
        + Unpin
        + 'static;
    /// The transaction pool of the node.
    type Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Self::Primitives>>>;
    /// The node's SAVM configuration, defining settings for the Sila Virtual Machine.
    type Savm: ConfigureEvm<Primitives = Self::Primitives> + Send + Sync + 'static;
    /// Network API.
    type Network: NetworkInfo + Clone;

    /// Returns the transaction pool of the node.
    fn pool(&self) -> &Self::Pool;

    /// Returns the node's savm config.
    fn evm_config(&self) -> &Self::Savm;

    /// Returns the handle to the network
    fn network(&self) -> &Self::Network;

    /// Returns the provider of the node.
    fn provider(&self) -> &Self::Provider;
}

impl<T> RpcNodeCore for T
where
    T: FullNodeComponents<Provider: ChainSpecProvider<ChainSpec: Hardforks + SilaHardforks>>,
{
    type Primitives = PrimitivesTy<T::Types>;
    type Provider = T::Provider;
    type Pool = T::Pool;
    type Savm = T::Savm;
    type Network = T::Network;

    #[inline]
    fn pool(&self) -> &Self::Pool {
        FullNodeComponents::pool(self)
    }

    #[inline]
    fn evm_config(&self) -> &Self::Savm {
        FullNodeComponents::evm_config(self)
    }

    #[inline]
    fn network(&self) -> &Self::Network {
        FullNodeComponents::network(self)
    }

    #[inline]
    fn provider(&self) -> &Self::Provider {
        FullNodeComponents::provider(self)
    }
}

/// Additional components, asides the core node components, needed to run `eth_` namespace API
/// server.
pub trait RpcNodeCoreExt: RpcNodeCore<Provider: BlockReader> {
    /// Returns handle to RPC cache service.
    fn cache(&self) -> &SilStateCache<Self::Primitives>;
}

/// An adapter that allows to construct [`RpcNodeCore`] from components.
#[derive(Debug, Clone)]
pub struct RpcNodeCoreAdapter<Provider, Pool, Network, Savm> {
    provider: Provider,
    pool: Pool,
    network: Network,
    evm_config: Savm,
}

impl<Provider, Pool, Network, Savm> RpcNodeCoreAdapter<Provider, Pool, Network, Savm> {
    /// Creates a new `RpcNodeCoreAdapter` instance.
    pub const fn new(provider: Provider, pool: Pool, network: Network, evm_config: Savm) -> Self {
        Self { provider, pool, network, evm_config }
    }
}

impl<Provider, Pool, Network, Savm> RpcNodeCore
    for RpcNodeCoreAdapter<Provider, Pool, Network, Savm>
where
    Provider: BlockReaderIdExt<
            Block = BlockTy<Savm::Primitives>,
            Receipt = ReceiptTy<Savm::Primitives>,
            Header = HeaderTy<Savm::Primitives>,
            Transaction = TxTy<Savm::Primitives>,
        > + ChainSpecProvider<
            ChainSpec: SilChainSpec<Header = HeaderTy<Savm::Primitives>>
                           + Hardforks
                           + SilaHardforks,
        > + StateProviderFactory
        + CanonStateSubscriptions<Primitives = Savm::Primitives>
        + StageCheckpointReader
        + PruneCheckpointReader
        + BalProvider
        + Send
        + Sync
        + Unpin
        + Clone
        + 'static,
    Savm: ConfigureEvm + Clone + 'static,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Savm::Primitives>>>
        + Unpin
        + 'static,
    Network: NetworkInfo + Clone + Unpin + 'static,
{
    type Primitives = Savm::Primitives;
    type Provider = Provider;
    type Pool = Pool;
    type Savm = Savm;
    type Network = Network;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }

    fn evm_config(&self) -> &Self::Savm {
        &self.evm_config
    }

    fn network(&self) -> &Self::Network {
        &self.network
    }

    fn provider(&self) -> &Self::Provider {
        &self.provider
    }
}
