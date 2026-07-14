//! The core of the Sila node. Collection of utilities and libraries that are used by the node.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod args;
pub mod cli;
pub mod dirs;
pub mod exit;
pub mod node_config;
pub mod utils;
pub mod version;

/// Re-exported primitive types
pub mod primitives {
    pub use rsil_sila_forks::*;
    pub use rsil_primitives_traits::*;
}

/// Re-export of `rsil_rpc_*` crates.
pub mod rpc {
    /// Re-exported from `rsil_rpc_server_types::result`.
    pub mod result {
        pub use rsil_rpc_server_types::result::*;
    }

    /// Re-exported from `rsil_rpc_convert`.
    pub mod compat {
        pub use rsil_rpc_convert::*;
    }
}
