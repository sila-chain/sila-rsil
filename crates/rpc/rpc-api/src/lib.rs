//! Rsil RPC interface definitions
//!
//! Provides all RPC interfaces.
//!
//! ## Feature Flags
//!
//! - `client`: Enables JSON-RPC client support.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod admin;
mod anvil;
mod debug;
mod engine;
mod hardhat;
mod mev;
mod miner;
mod net;
mod otterscan;
mod rsil;
mod rsil_engine;
mod rpc;
mod testing;
mod trace;
mod txpool;
mod validation;
mod web3;

pub use rsil::RsilJitAction;
pub use testing::{TestingBuildBlockRequestV1, TESTING_BUILD_BLOCK_V1, TESTING_COMMIT_BLOCK_V1};

/// re-export of all server traits
pub use servers::*;

/// Aggregates all server traits.
pub mod servers {
    pub use crate::{
        admin::AdminApiServer,
        anvil::AnvilApiServer,
        debug::DebugApiServer,
        engine::{EngineApiServer, EngineEthApiServer, IntoEngineApiRpcModule},
        hardhat::HardhatApiServer,
        mev::{MevFullApiServer, MevSimApiServer},
        miner::MinerApiServer,
        net::NetApiServer,
        otterscan::OtterscanServer,
        rsil::RsilApiServer,
        rsil_engine::{RsilEngineApiServer, RsilNewPayloadInput, RsilPayloadStatus},
        rpc::RpcApiServer,
        testing::TestingApiServer,
        trace::TraceApiServer,
        txpool::TxPoolApiServer,
        validation::BlockSubmissionValidationApiServer,
        web3::Web3ApiServer,
    };
    pub use rsil_rpc_eth_api::{
        self as sil, SilApiServer, SilBundleApiServer, SilCallBundleApiServer, SilConfigApiServer,
        SilFilterApiServer, SilPubSubApiServer, L2EthApiExtServer,
    };
}

/// re-export of all client traits
#[cfg(feature = "client")]
pub use clients::*;

/// Aggregates all client traits.
#[cfg(feature = "client")]
pub mod clients {
    pub use crate::{
        admin::AdminApiClient,
        anvil::AnvilApiClient,
        debug::DebugApiClient,
        engine::{EngineApiClient, EngineEthApiClient},
        hardhat::HardhatApiClient,
        mev::{MevFullApiClient, MevSimApiClient},
        miner::MinerApiClient,
        net::NetApiClient,
        otterscan::OtterscanClient,
        rsil::RsilApiClient,
        rsil_engine::RsilEngineApiClient,
        rpc::RpcApiClient,
        testing::TestingApiClient,
        trace::TraceApiClient,
        txpool::TxPoolApiClient,
        validation::BlockSubmissionValidationApiClient,
        web3::Web3ApiClient,
    };
    pub use rsil_rpc_eth_api::{
        SilApiClient, SilBundleApiClient, SilCallBundleApiClient, SilConfigApiClient,
        SilFilterApiClient, L2EthApiExtClient,
    };
}
