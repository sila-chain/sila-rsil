//! Rsil RPC server types, used in server implementation of `sil` namespace API.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

// `url` is needed for serde support on `reqwest::Url`
use url as _;

pub mod block;
pub mod builder;
pub mod cache;
pub mod capabilities;
pub mod error;
pub mod fee_history;
pub mod gas_oracle;
pub mod id_provider;
pub mod logs_utils;
pub mod pending_block;
pub mod receipt;
pub mod simulate;
pub mod transaction;
pub mod tx_forward;
pub mod utils;

pub use alloy_rpc_types_eth::FillTransaction;
pub use block::CachedTransaction;
pub use builder::config::{SilConfig, SilFilterConfig};
pub use cache::{
    config::SilStateCacheConfig, db::StateCacheDb, multi_consumer::MultiConsumerLruCache,
    SilStateCache,
};
pub use capabilities::{SilCapabilities, SilCapabilitiesHead, SilCapabilitiesResource};
pub use error::{RevertError, RpcInvalidTransactionError, SignError, SilApiError, SilResult};
pub use fee_history::{FeeHistoryCache, FeeHistoryCacheConfig, FeeHistoryEntry};
pub use gas_oracle::{
    GasCap, GasPriceOracle, GasPriceOracleConfig, GasPriceOracleResult, RPC_DEFAULT_GAS_CAP,
};
pub use id_provider::SilSubscriptionIdProvider;
pub use pending_block::{PendingBlock, PendingBlockEnv, PendingBlockEnvOrigin};
pub use transaction::TransactionSource;
pub use tx_forward::ForwardConfig;
