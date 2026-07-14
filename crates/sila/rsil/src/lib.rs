//! Sila meta crate that provides access to commonly used rsil dependencies.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

/// Re-exported sila types
#[doc(inline)]
pub use rsil_sila_primitives::*;

/// Re-exported rsil primitives
pub mod primitives {
    #[doc(inline)]
    pub use rsil_primitives_traits::*;
}

/// Re-exported cli types
#[cfg(feature = "cli")]
pub mod cli {
    #[doc(inline)]
    pub use rsil_cli_util::*;
    #[doc(inline)]
    pub use rsil_sila_cli::*;
}

/// Re-exported pool types
#[cfg(feature = "pool")]
pub use rsil_transaction_pool as pool;

/// Re-exported consensus types
#[cfg(feature = "consensus")]
pub mod consensus {
    #[doc(inline)]
    pub use rsil_consensus::*;
    pub use rsil_consensus_common::*;
    pub use rsil_sila_consensus::*;
}

/// Re-exported from `rsil_chainspec`
pub mod chainspec {
    #[doc(inline)]
    pub use rsil_chainspec::*;
}

/// Re-exported savm types
#[cfg(feature = "savm")]
pub mod savm {
    #[doc(inline)]
    pub use rsil_evm_sila::*;

    #[doc(inline)]
    pub use rsil_evm as primitives;

    #[doc(inline)]
    pub use rsil_revm as revm;
}

/// Re-exported exex types
#[cfg(feature = "exex")]
pub use rsil_exex as exex;

/// Re-exported from `tasks`.
#[cfg(feature = "tasks")]
pub mod tasks {
    pub use rsil_tasks::*;
}

/// Re-exported rsil network types
#[cfg(feature = "network")]
pub mod network {
    #[doc(inline)]
    pub use rsil_eth_wire as eth_wire;
    #[doc(inline)]
    pub use rsil_network::*;
    #[doc(inline)]
    pub use rsil_network_api as api;
}

/// Re-exported rsil provider types
#[cfg(feature = "provider")]
pub mod provider {
    #[doc(inline)]
    pub use rsil_provider::*;

    #[doc(inline)]
    pub use rsil_db as db;
}

/// Re-exported codec crate
#[cfg(feature = "provider")]
pub use rsil_codecs as codec;

/// Re-exported rsil storage api types
#[cfg(feature = "storage-api")]
pub mod storage {
    #[doc(inline)]
    pub use rsil_storage_api::*;
}

/// Re-exported sila node
#[cfg(feature = "node-api")]
pub mod node {
    #[doc(inline)]
    pub use rsil_node_api as api;
    #[cfg(feature = "node")]
    pub use rsil_node_builder as builder;
    #[doc(inline)]
    pub use rsil_node_core as core;
    #[cfg(feature = "node")]
    pub use rsil_node_sila::*;
}

/// Re-exported sila engine types
#[cfg(feature = "node")]
pub mod engine {
    #[doc(inline)]
    pub use rsil_engine_local as local;
    #[doc(inline)]
    pub use rsil_node_sila::engine::*;
}

/// Re-exported rsil trie types
#[cfg(feature = "trie")]
pub mod trie {
    #[doc(inline)]
    pub use rsil_trie::*;

    #[cfg(feature = "trie-db")]
    #[doc(inline)]
    pub use rsil_trie_db::*;
}

/// Re-exported rpc types
#[cfg(feature = "rpc")]
pub mod rpc {
    #[doc(inline)]
    pub use rsil_rpc::*;

    #[doc(inline)]
    pub use rsil_rpc_api as api;
    #[doc(inline)]
    pub use rsil_rpc_builder as builder;

    /// Re-exported sil types
    #[allow(ambiguous_glob_reexports)]
    pub mod sil {
        #[doc(inline)]
        pub use alloy_rpc_types_eth as primitives;
        #[doc(inline)]
        pub use rsil_rpc_eth_types::*;

        pub use rsil_rpc::sil::*;
    }

    /// Re-exported types
    pub mod types {
        #[doc(inline)]
        pub use alloy_rpc_types_engine as engine;
    }
}
