//! Staged syncing primitives for rsil.
//!
//! ## Feature Flags
//!
//! - `test-utils`: Utilities for testing

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod error;
mod metrics;
mod pipeline;
mod stage;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
mod util;

pub use crate::metrics::*;
pub use error::*;
pub use pipeline::*;
pub use stage::*;

use aquamarine as _;

// re-export the stages types for convenience
pub use rsil_stages_types::*;
