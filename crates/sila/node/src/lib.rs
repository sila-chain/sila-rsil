//! Standalone crate for sila-specific Rsil configuration and builder types.
//!
//! # features
//! - `js-tracer`: Enable the `JavaScript` tracer for the `debug_trace` endpoints

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

use rsil_revm as _;
use revm as _;

pub use rsil_sila_engine_primitives::{SilEngineTypes, SilPayloadTypes};

pub mod savm;
pub use savm::SilEvmConfig;

#[allow(deprecated)]
pub use savm::SilExecutorProvider;

pub use rsil_sila_consensus as consensus;
pub mod node;
pub use node::*;

pub mod payload;

pub mod engine;
pub use engine::SilaEngineValidator;

pub mod engine_ssz_containers;
pub mod engine_ssz_proxy;
