//! Support for configuring the components of a node.
//!
//! Customizable components of the node include:
//!  - The transaction pool.
//!  - The network implementation.
//!  - The payload builder service.
//!
//! Components depend on a fully type configured node: [`FullNodeTypes`].

mod builder;
mod consensus;
mod execute;
mod network;
mod payload;
mod pool;

pub use builder::*;
pub use consensus::*;
pub use execute::*;
pub use network::*;
pub use payload::*;
pub use pool::*;

use crate::{ConfigureEvm, FullNodeTypes};
use rsil_consensus::FullConsensus;
use rsil_network::types::NetPrimitivesFor;
use rsil_network_api::FullNetwork;
use rsil_node_api::{NodeTypes, PrimitivesTy, TxTy};
use rsil_payload_builder::PayloadBuilderHandle;
use rsil_transaction_pool::{PoolPooledTx, PoolTransaction, TransactionPool};
use std::fmt::Debug;

/// An abstraction over the components of a node, consisting of:
///  - savm and executor
///  - transaction pool
///  - network
///  - payload builder.
pub trait NodeComponents<T: FullNodeTypes>: Clone + Debug + Unpin + Send + Sync + 'static {
    /// The transaction pool of the node.
    type Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<T::Types>>> + Unpin;

    /// The node's SAVM configuration, defining settings for the Sila Virtual Machine.
    type Savm: ConfigureEvm<Primitives = <T::Types as NodeTypes>::Primitives>;

    /// The consensus type of the node.
    type Consensus: FullConsensus<<T::Types as NodeTypes>::Primitives> + Clone + Unpin + 'static;

    /// Network API.
    type Network: FullNetwork<Primitives: NetPrimitivesFor<<T::Types as NodeTypes>::Primitives>>;

    /// Returns the transaction pool of the node.
    fn pool(&self) -> &Self::Pool;

    /// Returns the node's savm config.
    fn evm_config(&self) -> &Self::Savm;

    /// Returns the node's consensus type.
    fn consensus(&self) -> &Self::Consensus;

    /// Returns the handle to the network
    fn network(&self) -> &Self::Network;

    /// Returns the handle to the payload builder service handling payload building requests from
    /// the engine.
    fn payload_builder_handle(&self) -> &PayloadBuilderHandle<<T::Types as NodeTypes>::Payload>;
}

/// All the components of the node.
///
/// This provides access to all the components of the node.
#[derive(Debug)]
pub struct Components<Node: FullNodeTypes, Network, Pool, SAVM, Consensus> {
    /// The transaction pool of the node.
    pub transaction_pool: Pool,
    /// The node's SAVM configuration, defining settings for the Sila Virtual Machine.
    pub evm_config: SAVM,
    /// The consensus implementation of the node.
    pub consensus: Consensus,
    /// The network implementation of the node.
    pub network: Network,
    /// The handle to the payload builder service.
    pub payload_builder_handle: PayloadBuilderHandle<<Node::Types as NodeTypes>::Payload>,
}

impl<Node, Pool, SAVM, Cons, Network> NodeComponents<Node>
    for Components<Node, Network, Pool, SAVM, Cons>
where
    Node: FullNodeTypes,
    Network: FullNetwork<
        Primitives: NetPrimitivesFor<
            PrimitivesTy<Node::Types>,
            PooledTransaction = PoolPooledTx<Pool>,
        >,
    >,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Node::Types>>>
        + Unpin
        + 'static,
    SAVM: ConfigureEvm<Primitives = PrimitivesTy<Node::Types>> + 'static,
    Cons: FullConsensus<PrimitivesTy<Node::Types>> + Clone + Unpin + 'static,
{
    type Pool = Pool;
    type Savm = SAVM;
    type Consensus = Cons;
    type Network = Network;

    fn pool(&self) -> &Self::Pool {
        &self.transaction_pool
    }

    fn evm_config(&self) -> &Self::Savm {
        &self.evm_config
    }

    fn consensus(&self) -> &Self::Consensus {
        &self.consensus
    }

    fn network(&self) -> &Self::Network {
        &self.network
    }

    fn payload_builder_handle(&self) -> &PayloadBuilderHandle<<Node::Types as NodeTypes>::Payload> {
        &self.payload_builder_handle
    }
}

impl<Node, N, Pool, SAVM, Cons> Clone for Components<Node, N, Pool, SAVM, Cons>
where
    N: Clone,
    Node: FullNodeTypes,
    Pool: TransactionPool,
    SAVM: ConfigureEvm,
    Cons: Clone,
{
    fn clone(&self) -> Self {
        Self {
            transaction_pool: self.transaction_pool.clone(),
            evm_config: self.evm_config.clone(),
            consensus: self.consensus.clone(),
            network: self.network.clone(),
            payload_builder_handle: self.payload_builder_handle.clone(),
        }
    }
}
