//! Payload component configuration for the Sila node.

use rsil_chainspec::{SilChainSpec, SilaHardforks};
use rsil_evm::ConfigureEvm;
use rsil_node_api::{FullNodeTypes, NodeTypes, PrimitivesTy, TxTy};
use rsil_node_builder::{
    components::PayloadBuilderBuilder, BuilderContext, PayloadBuilderConfig, PayloadTypes,
};
use rsil_sila_engine_primitives::{SilBuiltPayload, SilPayloadAttributes};
use rsil_sila_payload_builder::SilaBuilderConfig;
use rsil_sila_primitives::SilPrimitives;
use rsil_transaction_pool::{PoolTransaction, TransactionPool};

/// A basic sila payload service.
#[derive(Clone, Default, Debug)]
#[non_exhaustive]
pub struct SilaPayloadBuilder;

impl<Types, Node, Pool, Savm> PayloadBuilderBuilder<Node, Pool, Savm> for SilaPayloadBuilder
where
    Types: NodeTypes<ChainSpec: SilaHardforks, Primitives = SilPrimitives>,
    Node: FullNodeTypes<Types = Types>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Node::Types>>>
        + Unpin
        + 'static,
    Savm: ConfigureEvm<
            Primitives = PrimitivesTy<Types>,
            NextBlockEnvCtx = rsil_evm::NextBlockEnvAttributes,
        > + 'static,
    Types::Payload:
        PayloadTypes<BuiltPayload = SilBuiltPayload, PayloadAttributes = SilPayloadAttributes>,
{
    type PayloadBuilder = rsil_sila_payload_builder::SilaPayloadBuilder<Pool, Node::Provider, Savm>;

    async fn build_payload_builder(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
        evm_config: Savm,
    ) -> eyre::Result<Self::PayloadBuilder> {
        let conf = ctx.payload_builder_config();
        let chain = ctx.chain_spec().chain();
        let gas_limit = conf.gas_limit_for(chain);
        let skip_state_root = ctx.config().tree_config().skip_state_root();

        Ok(rsil_sila_payload_builder::SilaPayloadBuilder::new(
            ctx.provider().clone(),
            pool,
            evm_config,
            SilaBuilderConfig::new()
                .with_gas_limit(gas_limit)
                .with_max_blobs_per_block(conf.max_blobs_per_block())
                .with_extra_data(conf.extra_data())
                .with_skip_state_root(skip_state_root),
        ))
    }
}
