//! The implementation of Engine API.
//! [Read more](https://github.com/sila-chain/execution-apis/tree/main/src/engine).

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// The Engine API implementation.
mod engine_api;

/// Rsil-specific engine API extensions.
mod rsil_engine_api;

/// Engine API capabilities.
pub mod capabilities;
pub use capabilities::EngineCapabilities;

/// Engine API error.
mod error;

/// Engine API metrics.
mod metrics;

pub use engine_api::{EngineApi, EngineApiSender};
pub use error::*;
pub use rsil_engine_api::RsilEngineApi;

// re-export server trait for convenience
pub use rsil_rpc_api::EngineApiServer;

#[cfg(test)]
mod tests {
    // silence unused import warning
    use alloy_rlp as _;
}
