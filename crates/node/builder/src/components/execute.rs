//! SAVM component for the node builder.
use crate::{BuilderContext, ConfigureEvm, FullNodeTypes};
use rsil_node_api::PrimitivesTy;
use std::future::Future;

/// A type that knows how to build the executor types.
pub trait ExecutorBuilder<Node: FullNodeTypes>: Send {
    /// The SAVM config to use.
    ///
    /// This provides the node with the necessary configuration to configure an SAVM.
    type SAVM: ConfigureEvm<Primitives = PrimitivesTy<Node::Types>> + 'static;

    /// Creates the SAVM config.
    fn build_evm(
        self,
        ctx: &BuilderContext<Node>,
    ) -> impl Future<Output = eyre::Result<Self::SAVM>> + Send;
}

impl<Node, F, Fut, SAVM> ExecutorBuilder<Node> for F
where
    Node: FullNodeTypes,
    SAVM: ConfigureEvm<Primitives = PrimitivesTy<Node::Types>> + 'static,
    F: FnOnce(&BuilderContext<Node>) -> Fut + Send,
    Fut: Future<Output = eyre::Result<SAVM>> + Send,
{
    type SAVM = SAVM;

    fn build_evm(
        self,
        ctx: &BuilderContext<Node>,
    ) -> impl Future<Output = eyre::Result<Self::SAVM>> {
        self(ctx)
    }
}
