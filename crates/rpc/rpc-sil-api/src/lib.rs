//! Rsil RPC `eth_` API implementation
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

pub mod bundle;
pub mod core;
pub mod ext;
pub mod filter;
pub mod helpers;
pub mod node;
pub mod pubsub;
pub mod types;

pub use bundle::{SilBundleApiServer, SilCallBundleApiServer};
pub use core::{SilApiServer, FullEthApiServer};
pub use ext::L2EthApiExtServer;
pub use filter::{EngineEthFilter, SilFilterApiServer, QueryLimits};
pub use helpers::config::SilConfigApiServer;
pub use node::{RpcNodeCore, RpcNodeCoreExt};
pub use pubsub::SilPubSubApiServer;
pub use rsil_rpc_convert::*;
pub use rsil_rpc_eth_types::error::{
    AsEthApiError, FromEthApiError, FromEvmError, IntoEthApiError,
};
pub use types::{SilApiTypes, FullEthApiTypes, RpcBlock, RpcHeader, RpcReceipt, RpcTransaction};

#[cfg(feature = "client")]
pub use bundle::{SilBundleApiClient, SilCallBundleApiClient};
#[cfg(feature = "client")]
pub use core::SilApiClient;
#[cfg(feature = "client")]
pub use ext::L2EthApiExtClient;
#[cfg(feature = "client")]
pub use filter::SilFilterApiClient;
#[cfg(feature = "client")]
pub use helpers::config::SilConfigApiClient;

use rsil_trie_common as _;
