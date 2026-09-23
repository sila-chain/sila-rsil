use alloy_rpc_types_engine::{ClientCode, ClientVersionV1};
use rsil_chainspec::SILA_MAINNET;
use rsil_consensus::noop::NoopConsensus;
use rsil_engine_primitives::ConsensusEngineHandle;
use rsil_sila_engine_primitives::SilEngineTypes;
use rsil_sila_primitives::SilPrimitives;
use rsil_tokio_util::EventSender;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use rsil_evm_sila::SilEvmConfig;
use rsil_network_api::noop::NoopNetwork;
use rsil_node_sila::SilaEngineValidator;
use rsil_payload_builder::test_utils::spawn_test_payload_service;
use rsil_provider::test_utils::NoopProvider;
use rsil_rpc_builder::{
    auth::{AuthRpcModule, AuthServerConfig, AuthServerHandle},
    middleware::{RsilAuthHttpMiddleware, RsilRpcMiddleware},
    RpcModuleBuilder, RpcServerConfig, RpcServerHandle, TransportRpcModuleConfig,
};
use rsil_rpc_engine_api::{capabilities::EngineCapabilities, EngineApi};
use rsil_rpc_layer::JwtSecret;
use rsil_rpc_server_types::RpcModuleSelection;
use rsil_tasks::Runtime;
use rsil_transaction_pool::{
    noop::NoopTransactionPool,
    test_utils::{TestPool, TestPoolBuilder},
};
use tokio::sync::mpsc::unbounded_channel;

/// Localhost with port 0 so a free port is used.
pub const fn test_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
}

/// Launches a new server for the auth module
pub async fn launch_auth(secret: JwtSecret) -> AuthServerHandle {
    let config = AuthServerConfig::builder(secret).socket_addr(test_address()).build();
    launch_auth_with_config(config).await
}

/// Launches a new server for the auth module with the given config.
pub async fn launch_auth_with_config<RpcMiddleware, HttpMiddleware>(
    config: AuthServerConfig<RpcMiddleware, HttpMiddleware>,
) -> AuthServerHandle
where
    RpcMiddleware: RsilRpcMiddleware,
    HttpMiddleware: RsilAuthHttpMiddleware<RpcMiddleware>,
{
    let (tx, _rx) = unbounded_channel();
    let beacon_engine_handle = ConsensusEngineHandle::<SilEngineTypes>::new(tx);
    let client = ClientVersionV1 {
        code: ClientCode::RH,
        name: "Rsil".to_string(),
        version: "v0.2.0-beta.5".to_string(),
        commit: "defa64b2".to_string(),
    };

    let engine_api = EngineApi::new(
        NoopProvider::default(),
        SILA_MAINNET.clone(),
        beacon_engine_handle,
        spawn_test_payload_service().into(),
        NoopTransactionPool::default(),
        Runtime::test(),
        client,
        EngineCapabilities::default(),
        SilaEngineValidator::new(SILA_MAINNET.clone()),
        false,
        NoopNetwork::default(),
    );
    let module = AuthRpcModule::new(engine_api);
    module.start_server(config).await.unwrap()
}

/// Launches a new server with http only with the given modules
pub async fn launch_http(modules: impl Into<RpcModuleSelection>) -> RpcServerHandle {
    let builder = test_rpc_builder();
    let eth_api = builder.bootstrap_eth_api();
    let server =
        builder.build(TransportRpcModuleConfig::set_http(modules), eth_api, EventSender::new(1));
    RpcServerConfig::http(Default::default())
        .with_http_address(test_address())
        .start(&server)
        .await
        .unwrap()
}

/// Launches a new server with ws only with the given modules
pub async fn launch_ws(modules: impl Into<RpcModuleSelection>) -> RpcServerHandle {
    let builder = test_rpc_builder();
    let eth_api = builder.bootstrap_eth_api();
    let server =
        builder.build(TransportRpcModuleConfig::set_ws(modules), eth_api, EventSender::new(1));
    RpcServerConfig::ws(Default::default())
        .with_ws_address(test_address())
        .start(&server)
        .await
        .unwrap()
}

/// Launches a new server with http and ws and with the given modules
pub async fn launch_http_ws(modules: impl Into<RpcModuleSelection>) -> RpcServerHandle {
    let builder = test_rpc_builder();
    let eth_api = builder.bootstrap_eth_api();
    let modules = modules.into();
    let server = builder.build(
        TransportRpcModuleConfig::set_ws(modules.clone()).with_http(modules),
        eth_api,
        EventSender::new(1),
    );
    RpcServerConfig::ws(Default::default())
        .with_ws_address(test_address())
        .with_ws_address(test_address())
        .with_http(Default::default())
        .with_http_address(test_address())
        .start(&server)
        .await
        .unwrap()
}

/// Launches a new server with http and ws and with the given modules on the same port.
pub async fn launch_http_ws_same_port(modules: impl Into<RpcModuleSelection>) -> RpcServerHandle {
    let builder = test_rpc_builder();
    let modules = modules.into();
    let eth_api = builder.bootstrap_eth_api();
    let server = builder.build(
        TransportRpcModuleConfig::set_ws(modules.clone()).with_http(modules),
        eth_api,
        EventSender::new(1),
    );
    let addr = test_address();
    RpcServerConfig::ws(Default::default())
        .with_ws_address(addr)
        .with_http(Default::default())
        .with_http_address(addr)
        .start(&server)
        .await
        .unwrap()
}

/// Returns an [`RpcModuleBuilder`] with testing components.
pub fn test_rpc_builder(
) -> RpcModuleBuilder<SilPrimitives, NoopProvider, TestPool, NoopNetwork, SilEvmConfig, NoopConsensus>
{
    RpcModuleBuilder::default()
        .with_provider(NoopProvider::default())
        .with_pool(TestPoolBuilder::default().into())
        .with_network(NoopNetwork::default())
        .with_executor(Runtime::test())
        .with_evm_config(SilEvmConfig::sila_mainnet())
        .with_consensus(NoopConsensus::default())
}
