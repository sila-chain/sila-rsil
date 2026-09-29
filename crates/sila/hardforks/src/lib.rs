//! Sila fork types used in rsil.
//!
//! This crate contains Sila fork types and helper functions.
//!
//! ## Feature Flags
//!
//! - `arbitrary`: Adds `arbitrary` support for primitive types.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// Re-exported [SIP-2124](https://sips.sila.org/SIPS/sip-2124) forkid types.
pub use alloy_sip2124::*;

mod display;
mod hardforks;

pub use alloy_hardforks::*;
pub use alloy_hardforks::{
    ethereum as sila, mainnet as sila_mainnet, EthereumHardfork as SilaHardfork,
};

/// Sila-facing hardfork activation adapter over the real Alloy hardfork authority.
pub trait SilaHardforks: alloy_hardforks::EthereumHardforks {
    /// Returns the activation condition for the requested Sila hardfork.
    fn sila_fork_activation(&self, fork: SilaHardfork) -> ForkCondition {
        self.ethereum_fork_activation(fork)
    }
}

impl<T> SilaHardforks for T where T: alloy_hardforks::EthereumHardforks + ?Sized {}

pub use display::DisplayHardforks;
pub use hardforks::*;

#[cfg(any(test, feature = "arbitrary"))]
pub use arbitrary;
