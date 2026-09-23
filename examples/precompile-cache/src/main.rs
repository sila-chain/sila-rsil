//! This example shows how to implement a node with a custom SAVM that uses a stateful precompile

#![warn(unused_crate_dependencies)]

use alloy_evm::{
    precompiles::{DynPrecompile, Precompile, PrecompileInput, PrecompilesMap},
    revm::{context::DBErrorMarker, handler::SilPrecompiles, precompile::PrecompileId},
    sil::SilEvmContext,
    Savm, SavmFactory,
};
use alloy_genesis::Genesis;
use alloy_primitives::Bytes;
use parking_lot::RwLock;
use rsil_sila::{
    chainspec::{Chain, ChainSpec},
    node::{
        api::{FullNodeTypes, NodeTypes},
        builder::{components::ExecutorBuilder, BuilderContext, NodeBuilder},
        core::{args::RpcServerArgs, node_config::NodeConfig},
        node::SilaAddOns,
        savm::SilEvm,
        SilEvmConfig, SilaNode,
    },
    savm::{
        primitives::{Database, SavmEnv},
        revm::{
            context::{BlockEnv, Context, TxEnv},
            context_interface::result::{EVMError, HaltReason},
            inspector::{Inspector, NoOpInspector},
            interpreter::interpreter::SilInterpreter,
            precompile::PrecompileResult,
            primitives::hardfork::SpecId,
            MainBuilder, MainContext,
        },
    },
    tasks::Runtime,
    SilPrimitives,
};
use rsil_tracing::{RsilTracer, Tracer};
use schnellru::{ByLength, LruMap};
use std::sync::Arc;

/// Type alias for the LRU cache used within the [`PrecompileCache`].
type PrecompileLRUCache = LruMap<(Bytes, u64), PrecompileResult>;

/// A cache for precompile inputs / outputs.
///
/// This cache works with standard precompiles that take input data and gas limit as parameters.
/// The cache key is composed of the input bytes and gas limit, and the cached value is the
/// precompile execution result.
#[derive(Debug)]
pub struct PrecompileCache {
    /// Caches for each precompile input / output.
    cache: PrecompileLRUCache,
}

/// Custom SAVM factory.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MyEvmFactory {
    precompile_cache: Arc<RwLock<PrecompileCache>>,
}

impl SavmFactory for MyEvmFactory {
    type Savm<DB: Database, I: Inspector<SilEvmContext<DB>, SilInterpreter>> =
        SilEvm<DB, I, PrecompilesMap>;
    type Tx = TxEnv;
    type Error<DBError: DBErrorMarker> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = SilEvmContext<DB>;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: SavmEnv) -> Self::Savm<DB, NoOpInspector> {
        let new_cache = self.precompile_cache.clone();
        let spec = input.cfg_env.spec;

        let savm = Context::sila_mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(PrecompilesMap::from_static(SilPrecompiles::new(spec).precompiles));

        let mut savm = SilEvm::new(savm, false);

        savm.precompiles_mut().map_precompiles(|_, precompile| {
            WrappedPrecompile::wrap(precompile, new_cache.clone())
        });

        savm
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

/// A custom precompile that contains the cache and precompile it wraps.
pub struct WrappedPrecompile {
    /// The precompile to wrap.
    precompile: DynPrecompile,
    /// The cache to use.
    cache: Arc<RwLock<PrecompileCache>>,
}

impl WrappedPrecompile {
    fn new(precompile: DynPrecompile, cache: Arc<RwLock<PrecompileCache>>) -> Self {
        Self { precompile, cache }
    }

    /// Given a [`DynPrecompile`] and cache for a specific precompiles, create a
    /// wrapper that can be used inside Savm.
    fn wrap(precompile: DynPrecompile, cache: Arc<RwLock<PrecompileCache>>) -> DynPrecompile {
        let precompile_id = precompile.precompile_id().clone();
        let wrapped = Self::new(precompile, cache);
        (precompile_id, move |input: PrecompileInput<'_>| -> PrecompileResult {
            wrapped.call(input)
        })
            .into()
    }
}

impl Precompile for WrappedPrecompile {
    fn precompile_id(&self) -> &PrecompileId {
        self.precompile.precompile_id()
    }

    fn call(&self, input: PrecompileInput<'_>) -> PrecompileResult {
        let mut cache = self.cache.write();
        let key = (Bytes::copy_from_slice(input.data), input.gas);

        // get the result if it exists
        if let Some(result) = cache.cache.get(&key) {
            return result.clone();
        }

        // call the precompile if cache miss
        let output = self.precompile.call(input);

        // insert the result into the cache
        cache.cache.insert(key, output.clone());

        output
    }
}

/// Builds a regular sila block executor that uses the custom SAVM.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MyExecutorBuilder {
    /// The precompile cache to use for all executors.
    precompile_cache: Arc<RwLock<PrecompileCache>>,
}

impl Default for MyExecutorBuilder {
    fn default() -> Self {
        let precompile_cache = PrecompileCache {
            cache: LruMap::<(Bytes, u64), PrecompileResult>::new(ByLength::new(100)),
        };
        Self { precompile_cache: Arc::new(RwLock::new(precompile_cache)) }
    }
}

impl<Node> ExecutorBuilder<Node> for MyExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = SilPrimitives>>,
{
    type SAVM = SilEvmConfig<ChainSpec, MyEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::SAVM> {
        let evm_config = SilEvmConfig::new_with_evm_factory(
            ctx.chain_spec(),
            MyEvmFactory { precompile_cache: self.precompile_cache },
        );
        Ok(evm_config)
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = RsilTracer::new().init()?;

    let runtime = Runtime::test();

    // create a custom chain spec
    let spec = ChainSpec::builder()
        .chain(Chain::sila_mainnet())
        .genesis(Genesis::default())
        .london_activated()
        .paris_activated()
        .shanghai_activated()
        .cancun_activated()
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
