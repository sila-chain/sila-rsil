//! Example illustrating how to run the SIL JSON RPC API as a standalone over a DB file.
//!
//! Run with
//!
//! ```sh
//! cargo run -p rpc-db
//! ```
//!
//! This installs an additional RPC method `myrpcExt_customMethod` that can be queried via [cast](https://github.com/foundry-rs/foundry)
//!
//! ```sh
//! cast rpc myrpcExt_customMethod
//! ```

#![warn(unused_crate_dependencies)]

use std::{path::Path, sync::Arc};

use rsil_sila::{
    chainspec::ChainSpecBuilder,
    consensus::SilBeaconConsensus,
    network::api::noop::NoopNetwork,
    node::{api::NodeTypesWithDBAdapter, SilEvmConfig, SilaNode},
    pool::noop::NoopTransactionPool,
    provider::{
        db::{mdbx::DatabaseArguments, open_db_read_only, ClientVersion, DatabaseEnv},
        providers::{BlockchainProvider, RocksDBProvider, StaticFileProvider},
        ProviderFactory,
    },
    rpc::{
        builder::{RsilRpcModule, RpcModuleBuilder, RpcServerConfig, TransportRpcModuleConfig},
        SilApiBuilder,
    },
    tasks::Runtime,
};
// Configuring the network parts, ideally also wouldn't need to think about this.
use myrpc_ext::{MyRpcExt, MyRpcExtApiServer};

// Custom rpc extension
pub mod myrpc_ext;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // 1. Set up the DB
    let db_path = std::env::var("RSIL_DB_PATH")?;
    let db_path = Path::new(&db_path);
    let db = open_db_read_only(
        db_path.join("db").as_path(),
        DatabaseArguments::new(ClientVersion::default()),
    )?;
    let spec = Arc::new(ChainSpecBuilder::sila-mainnet().build());
    let runtime = Runtime::test();
    let factory = ProviderFactory::<NodeTypesWithDBAdapter<SilaNode, DatabaseEnv>>::new(
        db.clone(),
        spec.clone(),
        StaticFileProvider::read_only(db_path.join("static_files"))?,
        RocksDBProvider::builder(db_path.join("rocksdb")).build().unwrap(),
        runtime.clone(),
    )?;

    // 2. Set up the blockchain provider using only the database provider and a noop for the tree to
    //    satisfy trait bounds. Tree is not used in this example since we are only operating on the
    //    disk and don't handle new blocks/live sync etc, which is done by the blockchain tree.
    let provider = BlockchainProvider::new(factory)?;

    let rpc_builder = RpcModuleBuilder::default()
        .with_provider(provider.clone())
        // Rest is just noops that do nothing
        .with_noop_pool()
        .with_noop_network()
        .with_executor(runtime)
        .with_evm_config(SilEvmConfig::new(spec.clone()))
        .with_consensus(SilBeaconConsensus::new(spec.clone()));

    let eth_api = SilApiBuilder::new(
        provider.clone(),
        NoopTransactionPool::default(),
        NoopNetwork::default(),
        SilEvmConfig::sila-mainnet(),
    )
    .build();

    // Pick which namespaces to expose.
    let config = TransportRpcModuleConfig::default().with_http([RsilRpcModule::Sil]);

    let mut server = rpc_builder.build(config, eth_api, Default::default());

    // Add a custom rpc namespace
    let custom_rpc = MyRpcExt { provider };
    server.merge_configured(custom_rpc.into_rpc())?;

    // Start the server & keep it alive
    let server_args =
        RpcServerConfig::http(Default::default()).with_http_address("0.0.0.0:8545".parse()?);
    let _handle = server_args.start(&server).await?;
    futures::future::pending::<()>().await;

    Ok(())
}
