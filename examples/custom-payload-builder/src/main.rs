//! Example for how to hook into the node via the CLI extension mechanism without registering
//! additional arguments
//!
//! Run with
//!
//! ```sh
//! cargo run -p custom-payload-builder -- node
//! ```
//!
//! This launches a regular rsil node overriding the engine api payload builder with our custom.

#![warn(unused_crate_dependencies)]

use crate::generator::EmptyBlockPayloadJobGenerator;
use rsil_basic_payload_builder::BasicPayloadJobGeneratorConfig;
use rsil_payload_builder::{PayloadBuilderHandle, PayloadBuilderService};
use rsil_sila::{
    chainspec::ChainSpec,
    cli::interface::Cli,
    node::{
        api::{node::FullNodeTypes, NodeTypes},
        builder::{components::PayloadServiceBuilder, BuilderContext},
        core::cli::config::PayloadBuilderConfig,
        node::SilaAddOns,
        SilEngineTypes, SilaNode,
    },
    pool::{PoolTransaction, TransactionPool},
    provider::CanonStateSubscriptions,
    savm::primitives::{ConfigureEvm, NextBlockEnvAttributes},
    SilPrimitives, TransactionSigned,
};
use rsil_sila_payload_builder::SilaBuilderConfig;

pub mod generator;
pub mod job;

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct CustomPayloadBuilder;

impl<Node, Pool, Savm> PayloadServiceBuilder<Node, Pool, Savm> for CustomPayloadBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            Payload = SilEngineTypes,
            ChainSpec = ChainSpec,
            Primitives = SilPrimitives,
        >,
    >,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TransactionSigned>>
        + Unpin
        + 'static,
    Savm: ConfigureEvm<Primitives = SilPrimitives, NextBlockEnvCtx = NextBlockEnvAttributes>
        + 'static,
{
    async fn spawn_payload_builder_service(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
        evm_config: Savm,
    ) -> eyre::Result<PayloadBuilderHandle<<Node::Types as NodeTypes>::Payload>> {
        tracing::info!("Spawning a custom payload builder");

        let payload_builder = rsil_sila_payload_builder::SilaPayloadBuilder::new(
            ctx.provider().clone(),
            pool,
            evm_config,
            SilaBuilderConfig::new().with_extra_data(ctx.payload_builder_config().extra_data()),
        );

        let conf = ctx.payload_builder_config();

        let payload_job_config = BasicPayloadJobGeneratorConfig::default()
            .interval(conf.interval())
            .deadline(conf.deadline())
            .max_payload_tasks(conf.max_payload_tasks());

        let payload_generator = EmptyBlockPayloadJobGenerator::with_builder(
            ctx.provider().clone(),
            ctx.task_executor().clone(),
            payload_job_config,
            payload_builder,
        );

        let (payload_service, payload_builder) =
            PayloadBuilderService::new(payload_generator, ctx.provider().canonical_state_stream());

        ctx.task_executor().spawn_critical_task("custom payload builder service", payload_service);

        Ok(payload_builder)
    }
}

fn main() {
    Cli::parse_args()
        .run(async move |builder, _| {
            let handle = builder
                .with_types::<SilaNode>()
                // Configure the components of the node
                // use default sila components but use our custom payload builder
                .with_components(SilaNode::components().payload(CustomPayloadBuilder::default()))
                .with_add_ons(SilaAddOns::default())
                .launch()
                .await?;

            handle.wait_for_node_exit().await
        })
        .unwrap();
}
