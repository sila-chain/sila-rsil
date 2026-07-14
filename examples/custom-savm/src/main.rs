//! This example shows how to implement a node with a custom SAVM

#![warn(unused_crate_dependencies)]

use alloy_evm::{
    sil::SilEvmContext,
    precompiles::PrecompilesMap,
    revm::{
        context::DBErrorMarker,
        handler::SilPrecompiles,
        precompile::{Precompile, PrecompileId},
    },
    SavmFactory,
};
use alloy_genesis::Genesis;
use alloy_primitives::{address, Bytes};
use rsil_sila::{
    chainspec::{Chain, ChainSpec},
    savm::{
        primitives::{Database, SavmEnv},
        revm::{
            context::{BlockEnv, Context, TxEnv},
            context_interface::result::{EVMError, HaltReason},
            inspector::{Inspector, NoOpInspector},
            interpreter::interpreter::SilInterpreter,
            precompile::{PrecompileOutput, Precompiles},
            primitives::hardfork::SpecId,
            MainBuilder, MainContext,
        },
        SilEvm, SilEvmConfig,
    },
    node::{
        api::{FullNodeTypes, NodeTypes},
        builder::{components::ExecutorBuilder, BuilderContext, NodeBuilder},
        core::{args::RpcServerArgs, node_config::NodeConfig},
        node::SilaAddOns,
        SilaNode,
    },
    tasks::Runtime,
    SilPrimitives,
};
use rsil_tracing::{RsilTracer, Tracer};
use std::sync::OnceLock;

/// Custom SAVM configuration.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct MyEvmFactory;

impl SavmFactory for MyEvmFactory {
    type Savm<DB: Database, I: Inspector<SilEvmContext<DB>, SilInterpreter>> =
        SilEvm<DB, I, Self::Precompiles>;
    type Tx = TxEnv;
    type Error<DBError: DBErrorMarker> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = SilEvmContext<DB>;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: SavmEnv) -> Self::Savm<DB, NoOpInspector> {
        let spec = input.cfg_env.spec;
        let mut savm = Context::sila-mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(PrecompilesMap::from_static(SilPrecompiles::new(spec).precompiles));

        if spec == SpecId::PRAGUE {
            savm = savm.with_precompiles(PrecompilesMap::from_static(prague_custom()));
        }

        SilEvm::new(savm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, SilInterpreter>>(
        &self,
        db: DB,
        input: SavmEnv,
        inspector: I,
    ) -> Self::Savm<DB, I> {
        SilEvm::new(self.create_evm(db, input).into_inner().with_inspector(inspector), true)
    }
}

/// Builds a regular sila block executor that uses the custom SAVM.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct MyExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for MyExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = SilPrimitives>>,
{
    type SAVM = SilEvmConfig<ChainSpec, MyEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::SAVM> {
        let evm_config =
            SilEvmConfig::new_with_evm_factory(ctx.chain_spec(), MyEvmFactory::default());
        Ok(evm_config)
    }
}

/// Returns precompiles for SilaPrague spec.
pub fn prague_custom() -> &'static Precompiles {
    static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = Precompiles::prague().clone();
        // Custom precompile.
        let precompile = Precompile::new(
            PrecompileId::custom("custom"),
            address!("0x0000000000000000000000000000000000000999"),
            |_, _, _| Ok(PrecompileOutput::new(0, Bytes::new(), 0)),
        );
        precompiles.extend([precompile]);
        precompiles
    })
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = RsilTracer::new().init()?;

    let runtime = Runtime::test();

    // create a custom chain spec
    let spec = ChainSpec::builder()
        .chain(Chain::sila-mainnet())
        .genesis(Genesis::default())
        .london_activated()
        .paris_activated()
        .shanghai_activated()
        .cancun_activated()
        .prague_activated()
        .build();

    let node_config =
        NodeConfig::test().with_rpc(RpcServerArgs::default().with_http()).with_chain(spec);

    let handle = NodeBuilder::new(node_config)
        .testing_node(runtime)
        // configure the node with regular sila types
        .with_types::<SilaNode>()
        // use default sila components but with our executor
        .with_components(SilaNode::components().executor(MyExecutorBuilder::default()))
        .with_add_ons(SilaAddOns::default())
        .launch()
        .await
        .unwrap();

    println!("Node started");

    handle.node_exit_future.await
}
