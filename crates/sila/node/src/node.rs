//! Sila Node types config.

use crate::{SilEngineTypes, SilEvmConfig};
use alloy_eips::{merge::EPOCH_SLOTS, sip7840::BlobParams};
use alloy_network::Sila;
use alloy_rpc_types_engine::ExecutionData;
use revm::context::TxEnv;
use rsil_chainspec::{ChainSpec, Hardforks, SilChainSpec, SilaHardforks};
use rsil_engine_local::LocalPayloadAttributesBuilder;
use rsil_engine_primitives::EngineTypes;
use rsil_evm::{
    sil::spec::SilExecutorSpec, ConfigureEvm, NextBlockEnvAttributes, SavmFactory, SavmFactoryFor,
};
use rsil_evm_sila::factory::RsilEvmFactory;
#[cfg(feature = "jit")]
use rsil_evm_sila::factory::{JitBackend, JitMode, RevmcMetrics, RuntimeConfig, RuntimeTuning};
use rsil_network::{primitives::BasicNetworkPrimitives, NetworkHandle, PeersInfo};
use rsil_node_api::{
    AddOnsContext, FullNodeComponents, HeaderTy, NodeAddOns, NodePrimitives,
    PayloadAttributesBuilder, PrimitivesTy, TxTy,
};
use rsil_node_builder::{
    components::{
        BasicPayloadServiceBuilder, ComponentsBuilder, ConsensusBuilder, ExecutorBuilder,
        NetworkBuilder, PoolBuilder, TxPoolBuilder,
    },
    node::{FullNodeTypes, NodeTypes},
    rpc::{
        BasicEngineApiBuilder, BasicEngineValidatorBuilder, Either, EngineApiBuilder,
        EngineValidatorAddOn, EngineValidatorBuilder, Identity, PayloadValidatorBuilder, RpcAddOns,
        RpcHandle, RsilAuthHttpMiddleware, RsilRpcAddOns, RsilRpcMiddleware, SilApiBuilder,
        SilApiCtx, Stack,
    },
    BuilderContext, DebugNode, Node, NodeAdapter, PayloadBuilderConfig,
};
use rsil_node_core::args::JitArgs;
use rsil_payload_primitives::PayloadTypes;
use rsil_provider::{providers::ProviderFactoryBuilder, SilStorage};
use rsil_rpc::{
    sil::core::{SilApiFor, SilRpcConverterFor},
    TestingApi, ValidationApi,
};
use rsil_rpc_api::servers::{BlockSubmissionValidationApiServer, TestingApiServer};
use rsil_rpc_builder::config::RsilRpcServerConfig;
use rsil_rpc_eth_api::{
    helpers::{
        config::{SilConfigApiServer, SilConfigHandler},
        pending_block::BuildPendingEnv,
    },
    RpcConvert, RpcTypes, SignableTxRequest,
};
use rsil_rpc_eth_types::{error::FromEvmError, SilApiError};
use rsil_rpc_server_types::RsilRpcModule;
use rsil_sila_consensus::SilBeaconConsensus;
use rsil_sila_engine_primitives::{SilBuiltPayload, SilPayloadAttributes};
use rsil_sila_primitives::{SilPrimitives, TransactionSigned};
use rsil_tracing::tracing::{debug, info};
use rsil_transaction_pool::{
    blobstore::DiskFileBlobStore, PoolPooledTx, PoolTransaction, SilTransactionPool,
    TransactionPool, TransactionValidationTaskExecutor,
};
use std::{marker::PhantomData, sync::Arc, time::SystemTime};

pub use crate::{payload::SilaPayloadBuilder, SilaEngineValidator};
#[cfg(feature = "jit")]
pub use rsil_evm_sila::factory::maybe_run_jit_helper;

/// Type configuration for a regular Sila node.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct SilaNode;

impl SilaNode {
    /// Returns a [`ComponentsBuilder`] configured for a regular Sila node.
    pub fn components<Node>() -> ComponentsBuilder<
        Node,
        SilaPoolBuilder,
        BasicPayloadServiceBuilder<SilaPayloadBuilder>,
        SilaNetworkBuilder,
        SilaExecutorBuilder,
        SilaConsensusBuilder,
    >
    where
        Node: FullNodeTypes<
            Types: NodeTypes<
                ChainSpec: Hardforks + SilaHardforks + SilExecutorSpec,
                Primitives = SilPrimitives,
            >,
        >,
        <Node::Types as NodeTypes>::Payload:
            PayloadTypes<BuiltPayload = SilBuiltPayload, PayloadAttributes = SilPayloadAttributes>,
    {
        ComponentsBuilder::default()
            .node_types::<Node>()
            .pool(SilaPoolBuilder::default())
            .executor(SilaExecutorBuilder::default())
            .payload(BasicPayloadServiceBuilder::default())
            .network(SilaNetworkBuilder::default())
            .consensus(SilaConsensusBuilder::default())
    }

    /// Instantiates the [`ProviderFactoryBuilder`] for an sila node.
    ///
    /// # Open a Providerfactory in read-only mode from a datadir
    ///
    /// See also: [`ProviderFactoryBuilder`] and
    /// [`ReadOnlyConfig`](rsil_provider::providers::ReadOnlyConfig).
    ///
    /// ```no_run
    /// use rsil_chainspec::SILA_MAINNET;
    /// use rsil_node_sila::SilaNode;
    ///
    /// fn demo(runtime: rsil_tasks::Runtime) {
    ///     let factory = SilaNode::provider_factory_builder()
    ///         .open_read_only(SILA_MAINNET.clone(), "datadir", runtime)
    ///         .unwrap();
    /// }
    /// ```
    ///
    /// See also [`ProviderFactory::new`](rsil_provider::ProviderFactory::new) for constructing
    /// a [`ProviderFactory`](rsil_provider::ProviderFactory) manually with all required
    /// components.
    pub fn provider_factory_builder() -> ProviderFactoryBuilder<Self> {
        ProviderFactoryBuilder::default()
    }
}

impl NodeTypes for SilaNode {
    type Primitives = SilPrimitives;
    type ChainSpec = ChainSpec;
    type Storage = SilStorage;
    type Payload = SilEngineTypes;
}

/// Builds [`SilApi`](rsil_rpc::SilApi) for Sila.
#[derive(Debug)]
pub struct SilaEthApiBuilder<NetworkT = Sila>(PhantomData<NetworkT>);

impl<NetworkT> Default for SilaEthApiBuilder<NetworkT> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<N, NetworkT> SilApiBuilder<N> for SilaEthApiBuilder<NetworkT>
where
    N: FullNodeComponents<
        Types: NodeTypes<ChainSpec: Hardforks + SilaHardforks>,
        Savm: ConfigureEvm<NextBlockEnvCtx: BuildPendingEnv<HeaderTy<N::Types>>>,
    >,
    NetworkT: RpcTypes<TransactionRequest: SignableTxRequest<TxTy<N::Types>>>,
    SilRpcConverterFor<N, NetworkT>: RpcConvert<
        Primitives = PrimitivesTy<N::Types>,
        Error = SilApiError,
        Network = NetworkT,
        Savm = N::Savm,
    >,
    SilApiError: FromEvmError<N::Savm>,
{
    type SilApi = SilApiFor<N, NetworkT>;

    async fn build_eth_api(self, ctx: SilApiCtx<'_, N>) -> eyre::Result<Self::SilApi> {
        Ok(ctx.eth_api_builder().map_converter(|r| r.with_network()).build())
    }
}

/// Add-ons w.r.t. l1 sila.
#[derive(Debug)]
pub struct SilaAddOns<
    N: FullNodeComponents,
    SilB: SilApiBuilder<N>,
    PVB,
    EB = BasicEngineApiBuilder<PVB>,
    EVB = BasicEngineValidatorBuilder<PVB>,
    RpcMiddleware = Identity,
    AuthHttpMiddleware = Identity,
> {
    inner: RpcAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>,
}

impl<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>
    SilaAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>
where
    N: FullNodeComponents,
    SilB: SilApiBuilder<N>,
{
    /// Creates a new instance from the inner `RpcAddOns`.
    pub const fn new(
        inner: RpcAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>,
    ) -> Self {
        Self { inner }
    }
}

impl<N> Default for SilaAddOns<N, SilaEthApiBuilder, SilaEngineValidatorBuilder>
where
    N: FullNodeComponents<
        Types: NodeTypes<
            ChainSpec: SilaHardforks + Clone + 'static,
            Payload: EngineTypes<ExecutionData = ExecutionData>
                         + PayloadTypes<PayloadAttributes = SilPayloadAttributes>,
            Primitives = SilPrimitives,
        >,
    >,
    SilaEthApiBuilder: SilApiBuilder<N>,
{
    fn default() -> Self {
        Self::new(RpcAddOns::new(
            SilaEthApiBuilder::default(),
            SilaEngineValidatorBuilder::default(),
            BasicEngineApiBuilder::default(),
            BasicEngineValidatorBuilder::default(),
            Default::default(),
            Identity::new(),
        ))
    }
}

impl<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>
    SilaAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>
where
    N: FullNodeComponents,
    SilB: SilApiBuilder<N>,
{
    /// Replace the engine API builder.
    pub fn with_engine_api<T>(
        self,
        engine_api_builder: T,
    ) -> SilaAddOns<N, SilB, PVB, T, EVB, RpcMiddleware, AuthHttpMiddleware>
    where
        T: Send,
    {
        let Self { inner } = self;
        SilaAddOns::new(inner.with_engine_api(engine_api_builder))
    }

    /// Replace the payload validator builder.
    pub fn with_payload_validator<V, T>(
        self,
        payload_validator_builder: T,
    ) -> SilaAddOns<N, SilB, T, EB, EVB, RpcMiddleware, AuthHttpMiddleware> {
        let Self { inner } = self;
        SilaAddOns::new(inner.with_payload_validator(payload_validator_builder))
    }

    /// Sets rpc middleware
    pub fn with_rpc_middleware<T>(
        self,
        rpc_middleware: T,
    ) -> SilaAddOns<N, SilB, PVB, EB, EVB, T, AuthHttpMiddleware>
    where
        T: Send,
    {
        let Self { inner } = self;
        SilaAddOns::new(inner.with_rpc_middleware(rpc_middleware))
    }

    /// Configures the HTTP transport middleware for the auth / Engine API server.
    pub fn with_auth_http_middleware<T>(
        self,
        auth_http_middleware: T,
    ) -> SilaAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, T>
    where
        T: Send,
    {
        let Self { inner } = self;
        SilaAddOns::new(inner.with_auth_http_middleware(auth_http_middleware))
    }

    /// Stacks an additional HTTP transport middleware layer for the auth / Engine API server.
    pub fn layer_auth_http_middleware<T>(
        self,
        layer: T,
    ) -> SilaAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, Stack<AuthHttpMiddleware, T>> {
        let Self { inner } = self;
        SilaAddOns::new(inner.layer_auth_http_middleware(layer))
    }

    /// Conditionally stacks an HTTP transport middleware layer for the auth / Engine API server.
    #[expect(clippy::type_complexity)]
    pub fn option_layer_auth_http_middleware<T>(
        self,
        layer: Option<T>,
    ) -> SilaAddOns<
        N,
        SilB,
        PVB,
        EB,
        EVB,
        RpcMiddleware,
        Stack<AuthHttpMiddleware, Either<T, Identity>>,
    > {
        let Self { inner } = self;
        SilaAddOns::new(inner.option_layer_auth_http_middleware(layer))
    }

    /// Sets the tokio runtime for the RPC servers.
    ///
    /// Caution: This runtime must not be created from within asynchronous context.
    pub fn with_tokio_runtime(self, tokio_runtime: Option<tokio::runtime::Handle>) -> Self {
        let Self { inner } = self;
        Self { inner: inner.with_tokio_runtime(tokio_runtime) }
    }
}

impl<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware> NodeAddOns<N>
    for SilaAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>
where
    N: FullNodeComponents<
        Types: NodeTypes<
            ChainSpec: SilChainSpec + Hardforks + SilaHardforks,
            Primitives = SilPrimitives,
            Payload: EngineTypes<ExecutionData = ExecutionData>,
        >,
        Savm: ConfigureEvm<NextBlockEnvCtx = NextBlockEnvAttributes>,
    >,
    SilB: SilApiBuilder<N>,
    PVB: Send,
    EB: EngineApiBuilder<N>,
    EVB: EngineValidatorBuilder<N>,
    SilApiError: FromEvmError<N::Savm>,
    SavmFactoryFor<N::Savm>: SavmFactory<Tx = TxEnv>,
    RpcMiddleware: RsilRpcMiddleware,
    AuthHttpMiddleware: RsilAuthHttpMiddleware<Identity>,
{
    type Handle = RpcHandle<N, SilB::SilApi>;

    async fn launch_add_ons(
        self,
        ctx: rsil_node_api::AddOnsContext<'_, N>,
    ) -> eyre::Result<Self::Handle> {
        let validation_api = ValidationApi::<_, _, <N::Types as NodeTypes>::Payload>::new(
            ctx.node.provider().clone(),
            Arc::new(ctx.node.consensus().clone()),
            ctx.node.evm_config().clone(),
            ctx.config.rpc.flashbots_config(),
            ctx.node.task_executor().clone(),
            Arc::new(SilaEngineValidator::new(ctx.config.chain.clone())),
        );

        let eth_config =
            SilConfigHandler::new(ctx.node.provider().clone(), ctx.node.evm_config().clone());

        let testing_skip_invalid_transactions = ctx.config.rpc.testing_skip_invalid_transactions;
        let testing_gas_limit_override = ctx.config.rpc.testing_gas_limit;
        let testing_desired_gas_limit = ctx.config.builder.gas_limit_for(ctx.config.chain.chain());
        let testing_engine_handle = ctx.beacon_engine_handle.clone();

        self.inner
            .launch_add_ons_with(ctx, move |container| {
                container.modules.merge_if_module_configured(
                    RsilRpcModule::Flashbots,
                    validation_api.into_rpc(),
                )?;

                container
                    .modules
                    .merge_if_module_configured(RsilRpcModule::Sil, eth_config.into_rpc())?;

                // testing_buildBlockV1: only wire when the hidden testing module is explicitly
                // requested on any transport. Default stays disabled to honor security guidance.
                let mut testing_api = TestingApi::new(
                    container.registry.eth_api().clone(),
                    container.registry.evm_config().clone(),
                    testing_desired_gas_limit,
                    testing_engine_handle,
                );
                if testing_skip_invalid_transactions {
                    testing_api = testing_api.with_skip_invalid_transactions();
                }
                if let Some(gas_limit) = testing_gas_limit_override {
                    testing_api = testing_api.with_gas_limit_override(gas_limit);
                }
                container
                    .modules
                    .merge_if_module_configured(RsilRpcModule::Testing, testing_api.into_rpc())?;

                Ok(())
            })
            .await
    }
}

impl<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware> RsilRpcAddOns<N>
    for SilaAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>
where
    N: FullNodeComponents<
        Types: NodeTypes<
            ChainSpec: Hardforks + SilaHardforks,
            Primitives = SilPrimitives,
            Payload: EngineTypes<ExecutionData = ExecutionData>,
        >,
        Savm: ConfigureEvm<NextBlockEnvCtx = NextBlockEnvAttributes>,
    >,
    SilB: SilApiBuilder<N>,
    PVB: PayloadValidatorBuilder<N>,
    EB: EngineApiBuilder<N>,
    EVB: EngineValidatorBuilder<N>,
    SilApiError: FromEvmError<N::Savm>,
    SavmFactoryFor<N::Savm>: SavmFactory<Tx = TxEnv>,
    RpcMiddleware: RsilRpcMiddleware,
    AuthHttpMiddleware: RsilAuthHttpMiddleware<Identity>,
{
    type SilApi = SilB::SilApi;

    fn hooks_mut(&mut self) -> &mut rsil_node_builder::rpc::RpcHooks<N, Self::SilApi> {
        self.inner.hooks_mut()
    }
}

impl<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware> EngineValidatorAddOn<N>
    for SilaAddOns<N, SilB, PVB, EB, EVB, RpcMiddleware, AuthHttpMiddleware>
where
    N: FullNodeComponents<
        Types: NodeTypes<
            ChainSpec: SilChainSpec + SilaHardforks,
            Primitives = SilPrimitives,
            Payload: EngineTypes<ExecutionData = ExecutionData>,
        >,
        Savm: ConfigureEvm<NextBlockEnvCtx = NextBlockEnvAttributes>,
    >,
    SilB: SilApiBuilder<N>,
    PVB: Send,
    EB: EngineApiBuilder<N>,
    EVB: EngineValidatorBuilder<N>,
    SilApiError: FromEvmError<N::Savm>,
    SavmFactoryFor<N::Savm>: SavmFactory<Tx = TxEnv>,
    RpcMiddleware: Send,
    AuthHttpMiddleware: Send,
{
    type ValidatorBuilder = EVB;

    fn engine_validator_builder(&self) -> Self::ValidatorBuilder {
        self.inner.engine_validator_builder()
    }
}

impl<N> Node<N> for SilaNode
where
    N: FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        SilaPoolBuilder,
        BasicPayloadServiceBuilder<SilaPayloadBuilder>,
        SilaNetworkBuilder,
        SilaExecutorBuilder,
        SilaConsensusBuilder,
    >;

    type AddOns = SilaAddOns<NodeAdapter<N>, SilaEthApiBuilder, SilaEngineValidatorBuilder>;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        Self::components()
    }

    fn add_ons(&self) -> Self::AddOns {
        SilaAddOns::default()
    }
}

impl<N: FullNodeComponents<Types = Self>> DebugNode<N> for SilaNode {
    type RpcBlock = alloy_rpc_types_eth::Block;

    fn rpc_to_primitive_block(rpc_block: Self::RpcBlock) -> rsil_sila_primitives::Block {
        rpc_block.into_consensus().convert_transactions()
    }

    fn local_payload_attributes_builder(
        chain_spec: &Self::ChainSpec,
    ) -> impl PayloadAttributesBuilder<<Self::Payload as PayloadTypes>::PayloadAttributes> {
        LocalPayloadAttributesBuilder::new(Arc::new(chain_spec.clone()))
    }
}

/// Builds a [`RuntimeConfig`] from CLI [`JitArgs`].
#[cfg(feature = "jit")]
fn jit_runtime_config(jit: &JitArgs) -> RuntimeConfig {
    let default_tuning = RuntimeTuning::default();
    let tuning = RuntimeTuning {
        channel_capacity: jit.channel_capacity,
        jit_hot_threshold: jit.hot_threshold,
        jit_max_bytecode_len: jit.max_bytecode_len,
        jit_max_pending_jobs: jit.max_pending_jobs,
        jit_worker_count: jit.worker_count.unwrap_or(default_tuning.jit_worker_count),
        jit_timeout: default_tuning.jit_timeout,
        jit_helper_memory_limit_bytes: default_tuning.jit_helper_memory_limit_bytes,
        jit_helper_cpu_count: default_tuning.jit_helper_cpu_count,
        resident_code_cache_bytes: jit.code_cache_bytes,
        idle_evict_duration: Some(jit.idle_evict_duration),

        max_events_per_drain: default_tuning.max_events_per_drain,
        event_drain_interval: default_tuning.event_drain_interval,
        shutdown_timeout: default_tuning.shutdown_timeout,
        jit_worker_queue_capacity: default_tuning.jit_worker_queue_capacity,
        jit_opt_level: default_tuning.jit_opt_level,
        aot_opt_level: default_tuning.aot_opt_level,
        eviction_sweep_interval: default_tuning.eviction_sweep_interval,
        compiler_recycle_threshold: default_tuning.compiler_recycle_threshold,
    };

    let default_config = RuntimeConfig::default();
    RuntimeConfig {
        enabled: jit.enabled,
        thread_name: default_config.thread_name,
        store: default_config.store,
        tuning,
        dump_dir: default_config.dump_dir,
        debug_assertions: jit.debug,
        blocking: jit.blocking,
        no_dedup: default_config.no_dedup,
        no_dse: default_config.no_dse,
        gas_params: default_config.gas_params,
        aot: default_config.aot,
        jit_mode: JitMode::OutOfProcess,
        jit_helper_path: default_config.jit_helper_path,
        on_compilation: default_config.on_compilation,
    }
}

/// Builds an [`SilEvmConfig`] with revmc JIT from CLI [`JitArgs`].
///
/// This is the shared setup used by both [`SilaExecutorBuilder`] and `rsil re-execute`.
///
/// Returns the savm config and metrics recorder if JIT starts enabled.
#[cfg(feature = "jit")]
#[allow(clippy::type_complexity)]
pub fn build_evm_config<C: SilaHardforks>(
    chain_spec: Arc<C>,
    jit: &JitArgs,
    dump_dir: Option<std::path::PathBuf>,
) -> eyre::Result<(SilEvmConfig<C, RsilEvmFactory>, Option<Arc<RevmcMetrics>>)> {
    if !jit.enabled {
        let factory = RsilEvmFactory::disabled();
        return Ok((SilEvmConfig::new_with_evm_factory(chain_spec, factory), None));
    }

    let mut config = jit_runtime_config(jit);
    config.dump_dir = dump_dir;

    let revmc_metrics = Arc::new(RevmcMetrics::default());
    let compilation_metrics = revmc_metrics.clone();
    config.on_compilation = Some(Arc::new(move |event| {
        compilation_metrics.record_compilation(&event);
    }));

    let tuning = config.tuning;
    let jit_mode = config.jit_mode;
    let backend = JitBackend::new(config)?;

    rsil_tracing::tracing::warn!(target: "rsil::cli",
        hot_threshold = tuning.jit_hot_threshold,
        workers = tuning.jit_worker_count,
        mode = ?jit_mode,
        blocking = jit.blocking,
        "Started experimental revmc JIT backend; this may cause instability",
    );

    let factory = RsilEvmFactory::new_with_metrics(backend, revmc_metrics.as_ref().clone());
    let evm_config = SilEvmConfig::new_with_evm_factory(chain_spec, factory);

    Ok((evm_config, Some(revmc_metrics)))
}

/// Builds an [`SilEvmConfig`] from CLI [`JitArgs`].
///
/// This is the shared setup used by both [`SilaExecutorBuilder`] and `rsil re-execute`.
///
/// Compiled without the `jit` feature: errors if JIT was requested via [`JitArgs`] and otherwise
/// returns a plain interpreter-backed config.
#[cfg(not(feature = "jit"))]
#[allow(clippy::type_complexity)]
pub fn build_evm_config<C: SilaHardforks>(
    chain_spec: Arc<C>,
    jit: &JitArgs,
    _dump_dir: Option<std::path::PathBuf>,
) -> eyre::Result<(SilEvmConfig<C, RsilEvmFactory>, Option<()>)> {
    if jit.enabled {
        eyre::bail!(
            "JIT compilation was requested but this binary was compiled without the `jit` feature"
        );
    }
    let factory = RsilEvmFactory::default();
    Ok((SilEvmConfig::new_with_evm_factory(chain_spec, factory), None))
}

/// A regular sila savm and executor builder.
///
/// Uses [`RsilEvmFactory`].
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct SilaExecutorBuilder;

impl<Types, Node> ExecutorBuilder<Node> for SilaExecutorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + SilExecutorSpec + SilaHardforks,
        Primitives = SilPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
{
    type SAVM = SilEvmConfig<Types::ChainSpec, RsilEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::SAVM> {
        let jit = &ctx.config().jit;
        let dump_dir = jit.debug.then(|| ctx.config().datadir().data_dir().join("jit"));

        let (evm_config, revmc_metrics) = build_evm_config(ctx.chain_spec(), jit, dump_dir)?;

        #[cfg(not(feature = "jit"))]
        let _ = revmc_metrics;

        #[cfg(feature = "jit")]
        if let Some(revmc_metrics) = revmc_metrics {
            let metrics_backend = evm_config.executor_factory.evm_factory().backend().clone();
            ctx.task_executor().spawn_with_graceful_shutdown_signal(|shutdown| async move {
                let mut shutdown = std::pin::pin!(shutdown);
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                            revmc_metrics.record(&metrics_backend.stats());
                        }
                        _ = &mut shutdown => break,
                    }
                }
            });
        }

        Ok(evm_config)
    }
}

/// A basic sila transaction pool.
///
/// This contains various settings that can be configured and take precedence over the node's
/// config.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct SilaPoolBuilder {
    // TODO add options for txpool args
}

impl<Types, Node, Savm> PoolBuilder<Node, Savm> for SilaPoolBuilder
where
    Types: NodeTypes<
        ChainSpec: SilaHardforks,
        Primitives: NodePrimitives<SignedTx = TransactionSigned>,
    >,
    Node: FullNodeTypes<Types = Types>,
    Savm: ConfigureEvm<Primitives = PrimitivesTy<Types>> + Clone + 'static,
{
    type Pool = SilTransactionPool<Node::Provider, DiskFileBlobStore, Savm>;

    async fn build_pool(
        self,
        ctx: &BuilderContext<Node>,
        evm_config: Savm,
    ) -> eyre::Result<Self::Pool> {
        let pool_config = ctx.pool_config();

        let blobs_disabled = ctx.config().txpool.disable_blobs_support
            || ctx.config().txpool.blobpool_max_count == 0;

        let blob_cache_size = if let Some(blob_cache_size) = pool_config.blob_cache_size {
            Some(blob_cache_size)
        } else {
            // get the current blob params for the current timestamp, fallback to default SilaCancun
            // params
            let current_timestamp =
                SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
            let blob_params = ctx
                .chain_spec()
                .blob_params_at_timestamp(current_timestamp)
                .unwrap_or_else(BlobParams::cancun);

            // Derive the blob cache size from the target blob count, to auto scale it by
            // multiplying it with the slot count for 2 epochs: 384 for pectra
            Some((blob_params.target_blob_count * EPOCH_SLOTS * 2) as u32)
        };

        let blob_store =
            rsil_node_builder::components::create_blob_store_with_cache(ctx, blob_cache_size)?;

        let validator =
            TransactionValidationTaskExecutor::eth_builder(ctx.provider().clone(), evm_config)
                .set_eip4844(!blobs_disabled)
                .kzg_settings(ctx.kzg_settings()?)
                .with_max_tx_input_bytes(ctx.config().txpool.max_tx_input_bytes)
                .with_local_transactions_config(pool_config.local_transactions_config.clone())
                .set_tx_fee_cap(ctx.config().rpc.rpc_tx_fee_cap)
                .with_max_tx_gas_limit(ctx.config().txpool.max_tx_gas_limit)
                .with_minimum_priority_fee(ctx.config().txpool.minimum_priority_fee)
                .with_additional_tasks(ctx.config().txpool.additional_validation_tasks)
                .build_with_tasks(ctx.task_executor().clone(), blob_store.clone());

        if validator.validator().sip4844() {
            // initializing the KZG settings can be expensive, this should be done upfront so that
            // it doesn't impact the first block or the first gossiped blob transaction, so we
            // initialize this in the background
            let kzg_settings = validator.validator().kzg_settings().clone();
            ctx.task_executor().spawn_blocking_task(async move {
                let _ = kzg_settings.get();
                debug!(target: "rsil::cli", "Initialized KZG settings");
            });
        }

        let transaction_pool = TxPoolBuilder::new(ctx)
            .with_validator(validator)
            .build_and_spawn_maintenance_task(blob_store, pool_config)?;

        info!(target: "rsil::cli", "Transaction pool initialized");
        debug!(target: "rsil::cli", "Spawned txpool maintenance task");

        Ok(transaction_pool)
    }
}

/// A basic sila payload service.
#[derive(Debug, Default, Clone, Copy)]
pub struct SilaNetworkBuilder {
    // TODO add closure to modify network
}

impl<Node, Pool> NetworkBuilder<Node, Pool> for SilaNetworkBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec: Hardforks>>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Node::Types>>>
        + Unpin
        + 'static,
{
    type Network =
        NetworkHandle<BasicNetworkPrimitives<PrimitivesTy<Node::Types>, PoolPooledTx<Pool>>>;

    async fn build_network(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<Self::Network> {
        let network = ctx.network_builder().await?;
        let handle = ctx.start_network(network, pool);
        info!(target: "rsil::cli", enode=%handle.local_node_record(), "P2P networking initialized");
        Ok(handle)
    }
}

/// A basic sila consensus builder.
#[derive(Debug, Default, Clone, Copy)]
pub struct SilaConsensusBuilder {
    // TODO add closure to modify consensus
}

impl<Node> ConsensusBuilder<Node> for SilaConsensusBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<ChainSpec: SilChainSpec + SilaHardforks, Primitives = SilPrimitives>,
    >,
{
    type Consensus = Arc<SilBeaconConsensus<<Node::Types as NodeTypes>::ChainSpec>>;

    async fn build_consensus(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::Consensus> {
        Ok(Arc::new(SilBeaconConsensus::new(ctx.chain_spec())))
    }
}

/// Builder for [`SilaEngineValidator`].
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct SilaEngineValidatorBuilder;

impl<Node, Types> PayloadValidatorBuilder<Node> for SilaEngineValidatorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + SilaHardforks + Clone + 'static,
        Payload: EngineTypes<ExecutionData = ExecutionData>
                     + PayloadTypes<PayloadAttributes = SilPayloadAttributes>,
        Primitives = SilPrimitives,
    >,
    Node: FullNodeComponents<Types = Types>,
{
    type Validator = SilaEngineValidator<Types::ChainSpec>;

    async fn build(self, ctx: &AddOnsContext<'_, Node>) -> eyre::Result<Self::Validator> {
        Ok(SilaEngineValidator::new(ctx.config.chain.clone()))
    }
}
