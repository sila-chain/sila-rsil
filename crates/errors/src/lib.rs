//! High level error types for the rsil in general.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

extern crate alloc;

mod error;
pub use error::{RsilError, RsilResult};

pub use rsil_consensus::ConsensusError;
pub use rsil_execution_errors::{BlockExecutionError, BlockValidationError};
pub use rsil_storage_errors::{
    db::DatabaseError,
    provider::{ProviderError, ProviderResult},
};
