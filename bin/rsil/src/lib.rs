//! Rust Sila (rsil) binary executable.
//!
//! ## Feature Flags
//!
//! ### Default Features
//!
//! - `jemalloc`: Uses [jemallocator](https://github.com/tikv/jemallocator) as the global allocator.
//!   This is **not recommended on Windows**. See [here](https://rust-lang.github.io/rfcs/1974-global-allocators.html#jemalloc)
//!   for more info.
//! - `otlp`: Enables [OpenTelemetry](https://opentelemetry.io/) metrics export to a configured OTLP
//!   collector endpoint.
//! - `js-tracer`: Enables the `JavaScript` tracer for the `debug_trace` endpoints, allowing custom
//!   `JavaScript`-based transaction tracing.
//! - `keccak-cache-global`: Enables global caching for Keccak256 hashes to improve performance.
//! - `asm-keccak`: Replaces the default, pure-Rust implementation of Keccak256 with one implemented
//!   in assembly; see [the `keccak-asm` crate](https://github.com/DaniPopes/keccak-asm) for more
//!   details and supported targets.
//!
//! ### Allocator Features
//!
//! - `jemalloc-prof`: Enables [jemallocator's](https://github.com/tikv/jemallocator) heap profiling
//!   and leak detection functionality. See [jemalloc's opt.prof](https://jemalloc.net/jemalloc.3.html#opt.prof)
//!   documentation for usage details. This is **not recommended on Windows**.
//! - `jemalloc-symbols`: Enables jemalloc symbols for profiling. Includes `jemalloc-prof`.
//! - `tracy-allocator`: Enables [Tracy](https://github.com/wolfpld/tracy) profiler allocator
//!   integration for memory profiling.
//! - `snmalloc`: Uses [snmalloc](https://github.com/microsoft/snmalloc) as the global allocator.
//!   Use `--no-default-features` when enabling this, as jemalloc takes precedence.
//! - `snmalloc-native`: Uses snmalloc with native CPU optimizations. Use `--no-default-features`
//!   when enabling this.
//!
//! ### Log Level Features
//!
//! - `min-error-logs`: Disables all logs below `error` level.
//! - `min-warn-logs`: Disables all logs below `warn` level.
//! - `min-info-logs`: Disables all logs below `info` level. This can speed up the node, since fewer
//!   calls to the logging component are made.
//! - `min-debug-logs`: Disables all logs below `debug` level.
//! - `min-trace-logs`: Disables all logs below `trace` level.
//!
//! ### Development Features
//!
//! - `dev`: Enables development mode features, including test vector generation commands.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Used in feature flags only (`asm-keccak`, `keccak-cache-global`)
use alloy_primitives as _;

pub mod cli;

/// Re-exported utils.
pub mod utils {
    pub use rsil_db::open_db_read_only;

    /// Re-exported from `rsil_node_core`, also to prevent a breaking change. See the comment
    /// on the `rsil_node_core::args` re-export for more details.
    pub use rsil_node_core::utils::*;
}

/// Re-exported payload related types
pub mod payload {
    pub use rsil_payload_builder::*;
    pub use rsil_payload_primitives::*;
    pub use rsil_sila_payload_builder::SilaExecutionPayloadValidator;
}

/// Re-exported from `rsil_node_api`.
pub mod api {
    pub use rsil_node_api::*;
}

/// Re-exported from `rsil_node_core`.
pub mod core {
    pub use rsil_node_core::*;
}

/// Re-exported from `rsil_node_metrics`.
pub mod prometheus_exporter {
    pub use rsil_node_metrics::recorder::*;
}

/// Re-export of the `rsil_node_core` types specifically in the `args` module.
///
/// This is re-exported because the types in `rsil_node_core::args` originally existed in
/// `rsil::args` but were moved to the `rsil_node_core` crate. This re-export avoids a breaking
/// change.
pub mod args {
    pub use rsil_node_core::args::*;
}

/// Re-exported from `rsil_node_core`, also to prevent a breaking change. See the comment on
/// the `rsil_node_core::args` re-export for more details.
pub mod version {
    pub use rsil_node_core::version::*;
}

/// Re-exported from `rsil_node_builder`
pub mod builder {
    pub use rsil_node_builder::*;
}

/// Re-exported from `rsil_node_core`, also to prevent a breaking change. See the comment on
/// the `rsil_node_core::args` re-export for more details.
pub mod dirs {
    pub use rsil_node_core::dirs::*;
}

/// Re-exported from `rsil_chainspec`
pub mod chainspec {
    pub use rsil_chainspec::*;
    pub use rsil_sila_cli::chainspec::*;
}

/// Re-exported from `rsil_provider`.
pub mod providers {
    pub use rsil_provider::*;
}

/// Re-exported primitives.
#[allow(ambiguous_glob_reexports)]
pub mod primitives {
    pub use rsil_primitives_traits::*;
    pub use rsil_sila_primitives::*;
}

/// Re-exported from `rsil_sila_consensus`.
pub mod beacon_consensus {
    pub use rsil_node_sila::consensus::*;
}

/// Re-exported from `rsil_consensus`.
pub mod consensus {
    pub use rsil_consensus::*;
}

/// Re-exported from `rsil_consensus_common`.
pub mod consensus_common {
    pub use rsil_consensus_common::*;
}

/// Re-exported from `rsil_revm`.
pub mod revm {
    pub use rsil_revm::*;
}

/// Re-exported from `rsil_tasks`.
pub mod tasks {
    pub use rsil_tasks::*;
}

/// Re-exported from `rsil_network`.
pub mod network {
    pub use rsil_network::*;
    pub use rsil_network_api::{
        noop, test_utils::PeersHandleProvider, NetworkInfo, Peers, PeersInfo,
    };
}

/// Re-exported from `rsil_transaction_pool`.
pub mod transaction_pool {
    pub use rsil_transaction_pool::*;
}

/// Re-export of `rsil_rpc_*` crates.
pub mod rpc {
    /// Re-exported from `rsil_rpc_builder`.
    pub mod builder {
        pub use rsil_rpc_builder::*;
    }

    /// Re-exported from `alloy_rpc_types`.
    pub mod types {
        pub use alloy_rpc_types::*;
    }

    /// Re-exported from `rsil_rpc_server_types`.
    pub mod server_types {
        pub use rsil_rpc_server_types::*;
        /// Re-exported from `rsil_rpc_eth_types`.
        pub mod sil {
            pub use rsil_rpc_eth_types::*;
        }
    }

    /// Re-exported from `rsil_rpc_api`.
    pub mod api {
        pub use rsil_rpc_api::*;
    }
    /// Re-exported from `rsil_rpc::sil`.
    pub mod sil {
        pub use rsil_rpc::sil::*;
    }

    /// Re-exported from `rsil_rpc_server_types::result`.
    pub mod result {
        pub use rsil_rpc_server_types::result::*;
    }

    /// Re-exported from `rsil_rpc_convert`.
    pub mod compat {
        pub use rsil_rpc_convert::*;
    }
}

// re-export for convenience
#[doc(inline)]
pub use rsil_cli_runner::{CliContext, CliRunner};

// for rendering diagrams
use aquamarine as _;

// used in main
use clap as _;
use rsil_cli_util as _;
use tracing as _;
