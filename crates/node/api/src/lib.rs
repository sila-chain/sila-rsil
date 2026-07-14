//! Standalone crate for Rsil configuration traits and builder types.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Traits, validation methods, and helper types used to abstract over engine types.
pub use rsil_engine_primitives as engine;
pub use rsil_engine_primitives::*;

/// Traits and helper types used to abstract over payload types.
pub use rsil_payload_primitives as payload;
pub use rsil_payload_primitives::*;

/// Traits and helper types used to abstract over payload builder types.
pub use rsil_payload_builder_primitives as payload_builder;
pub use rsil_payload_builder_primitives::*;

/// Traits and helper types used to abstract over SAVM methods and types.
pub use rsil_evm::{ConfigureEvm, NextBlockEnvAttributes};

pub mod node;
pub use node::*;

// re-export for convenience
pub use rsil_node_types::*;
pub use rsil_provider::FullProvider;
