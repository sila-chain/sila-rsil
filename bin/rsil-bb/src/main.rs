//! rsil-bb: a modified rsil node for benchmarking big block execution.
#![allow(missing_docs)]

#[global_allocator]
static ALLOC: rsil_cli_util::allocator::Allocator = rsil_cli_util::allocator::new_allocator();

mod savm;
mod evm_config;

use alloy_primitives::Bytes;
use alloy_rpc_types::engine::ExecutionData;
use clap::Parser;
use evm_config::{BbEvmConfig, BigBlockData};
use rsil_chainspec::{ChainSpec, SilaHardforks};
use rsil_consensus::noop::NoopConsensus;
use rsil_sila_cli::{chainspec::SilaChainSpecParser, interface::Cli};
use rsil_sila_primitives::{Block, SilPrimitives};
use rsil_evm_sila::SilEvmConfig;
use rsil_node_api::{
    AddOnsContext, FullNodeComponents, NewPayloadError, NodeTypes, PayloadTypes, PayloadValidator,
};
use rsil_node_builder::{
    components::{
        BasicPayloadServiceBuilder, ComponentsBuilder, ConsensusBuilder, ExecutorBuilder,
    },
    node::FullNodeTypes,
    rpc::{NoopEngineApiBuilder, PayloadValidatorBuilder, RpcAddOns},
    BuilderContext, Node, NodeAdapter,
};
use rsil_node_core::args::DefaultEngineValues;
use rsil_node_sila::{
    SilPayloadTypes, SilaEngineValidator, SilaEthApiBuilder, SilaNetworkBuilder,
    SilaNode, SilaPayloadBuilder, SilaPoolBuilder,
};
use rsil_primitives_traits::SealedBlock;
use rsil_provider::SilStorage;
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct BbPayloadTypes;

impl PayloadTypes for BbPayloadTypes {
    type ExecutionData = BigBlockData<ExecutionData>;
    type BuiltPayload = <SilPayloadTypes as PayloadTypes>::BuiltPayload;
    type PayloadAttributes = <SilPayloadTypes as PayloadTypes>::PayloadAttributes;

    fn block_to_payload(
        _block: SealedBlock<
                <<Self::BuiltPayload as rsil_node_api::BuiltPayload>::Primitives as rsil_node_api::NodePrimitives>::Block,
            >,
        _bal: Option<Bytes>,
    ) -> Self::ExecutionData {
        unreachable!()
    }
}

#[derive(Debug, Default, Clone)]
pub struct BbEngineValidatorBuilder;

impl<Node> PayloadValidatorBuilder<Node> for BbEngineValidatorBuilder
where
    Node: FullNodeComponents<Types = BbNode>,
{
    type Validator = BbEngineValidator;

    async fn build(self, ctx: &AddOnsContext<'_, Node>) -> eyre::Result<Self::Validator> {
        Ok(BbEngineValidator { inner: SilaEngineValidator::new(ctx.config.chain.clone()) })
    }
}

#[derive(Debug, Clone)]
pub struct BbEngineValidator {
    inner: SilaEngineValidator,
}

impl PayloadValidator<BbPayloadTypes> for BbEngineValidator {
    type Block = Block;

    fn convert_payload_to_block(
        &self,
        payload: BigBlockData<ExecutionData>,
    ) -> Result<SealedBlock<Block>, NewPayloadError> {
        let mut blocks = payload
            .env_switches
            .into_iter()
            .map(|data| {
                PayloadValidator::<SilPayloadTypes>::convert_payload_to_block(&self.inner, data)
            })
            .collect::<Result<Vec<SealedBlock<Block>>, NewPayloadError>>()?;

        let (mut block, hash) = blocks.pop().unwrap().split();

        // Override the block number
        block.header.number = payload.block_number;

        // Set block's parent hash to the parent of the first block in this batch so that engine
        // tree state is consistent.
        if let Some(first) = blocks.first() {
            block.header.parent_hash = first.parent_hash;
        }

        // Update block's gas usage to make sure metrics are correct
        block.header.gas_used += blocks.iter().map(|b| b.gas_used).sum::<u64>();
        block.header.gas_limit += blocks.iter().map(|b| b.gas_limit).sum::<u64>();

        // Prepend transactions from previous blocks to make sure that persistence indices are
        // correct.
        block.body.transactions = blocks
            .into_iter()
            .flat_map(|b| b.into_body().transactions)
            .chain(core::mem::take(&mut block.body.transactions))
            .collect();

        // Use `new_unchecked` to preserve the hash
        Ok(SealedBlock::new_unchecked(block, hash))
    }
}

// ---------------------------------------------------------------------------
// Custom executor builder
// ---------------------------------------------------------------------------

/// Executor builder that creates a [`BbEvmConfig`].
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct BbExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for BbExecutorBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            ChainSpec: rsil_sila_forks::Hardforks
                           + alloy_evm::sil::spec::SilExecutorSpec
                           + SilaHardforks,
            Primitives = SilPrimitives,
        >,
    >,
{
    type SAVM = BbEvmConfig<<Node::Types as NodeTypes>::ChainSpec>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::SAVM> {
        Ok(BbEvmConfig::new(SilEvmConfig::new(ctx.chain_spec())))
    }
}

// ---------------------------------------------------------------------------
// Node type
// ---------------------------------------------------------------------------

/// Node type for big block execution.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct BbNode;

impl NodeTypes for BbNode {
    type Primitives = SilPrimitives;
    type ChainSpec = ChainSpec;
    type Storage = SilStorage;
    type Payload = BbPayloadTypes;
}

impl<N> Node<N> for BbNode
where
    N: FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        SilaPoolBuilder,
        BasicPayloadServiceBuilder<SilaPayloadBuilder>,
        SilaNetworkBuilder,
        BbExecutorBuilder,
        BbConsensusBuilder,
    >;

    type AddOns = RpcAddOns<
        NodeAdapter<N>,
        SilaEthApiBuilder,
        BbEngineValidatorBuilder,
        NoopEngineApiBuilder,
    >;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        SilaNode::components()
            .executor(BbExecutorBuilder::default())
            .consensus(BbConsensusBuilder)
    }

    fn add_ons(&self) -> Self::AddOns {
        Default::default()
    }
}

// ---------------------------------------------------------------------------
// Consensus builder
// ---------------------------------------------------------------------------

/// Consensus builder for big block execution.
#[derive(Debug, Default, Clone, Copy)]
pub struct BbConsensusBuilder;

impl<Node> ConsensusBuilder<Node> for BbConsensusBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<Primitives = SilPrimitives>>,
{
    type Consensus = NoopConsensus;

    async fn build_consensus(self, _ctx: &BuilderContext<Node>) -> eyre::Result<Self::Consensus> {
        Ok(NoopConsensus::default())
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    rsil_cli_util::sigsegv_handler::install();

    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    let _ = DefaultEngineValues::default().with_bal_parallel_execution_disabled(false).try_init();

    if let Err(err) = Cli::<SilaChainSpecParser>::parse().run(async move |builder, _| {
        info!(target: "rsil::cli", "Launching big block node");
        let handle = builder.launch_node(BbNode::default()).await?;

        handle.wait_for_node_exit().await
    }) {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
