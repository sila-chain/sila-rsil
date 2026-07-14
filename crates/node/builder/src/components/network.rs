//! Network component for the node builder.

use crate::{BuilderContext, FullNodeTypes};
use rsil_network::types::NetPrimitivesFor;
use rsil_network_api::FullNetwork;
use rsil_node_api::PrimitivesTy;
use rsil_transaction_pool::TransactionPool;
use std::future::Future;

/// A type that knows how to build the network implementation.
pub trait NetworkBuilder<Node: FullNodeTypes, Pool: TransactionPool>: Send {
    /// The network built.
    type Network: FullNetwork<Primitives: NetPrimitivesFor<PrimitivesTy<Node::Types>>>;

    /// Launches the network implementation and returns the handle to it.
    fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> impl Future<Output = eyre::Result<Self::Network>> + Send;
}

impl<Node, Net, F, Fut, Pool> NetworkBuilder<Node, Pool> for F
where
    Node: FullNodeTypes,
    Net: FullNetwork<Primitives: NetPrimitivesFor<PrimitivesTy<Node::Types>>>,
    Pool: TransactionPool,
    F: Fn(&BuilderContext<Node>, Pool) -> Fut + Send,
    Fut: Future<Output = eyre::Result<Net>> + Send,
{
    type Network = Net;

    fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> impl Future<Output = eyre::Result<Net>> + Send {
        self(ctx, pool)
    }
}
