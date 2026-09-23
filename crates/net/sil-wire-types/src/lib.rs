//! Types for the sil wire protocol: <https://github.com/sila-chain/devp2p/blob/master/caps/sil.md>

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod status;
pub use status::{Status, StatusBuilder, StatusEth69, StatusMessage, UnifiedStatus};

pub mod version;
pub use version::{ProtocolVersion, SilVersion};

pub mod message;
pub use message::{ProtocolMessage, SilMessage, SilMessageID};

pub mod header;
pub use header::*;

pub mod blocks;
pub use blocks::*;

pub mod broadcast;
pub use broadcast::*;

pub mod transactions;
pub use transactions::*;

pub mod state;
pub use state::*;

pub mod receipts;
pub use receipts::*;

pub mod block_access_lists;
pub use block_access_lists::*;

pub mod disconnect_reason;
pub use disconnect_reason::*;

pub mod capability;
pub use capability::*;

pub mod primitives;
pub use primitives::*;

pub mod snap;
pub use snap::*;

/// re-export for convenience
pub use alloy_sips::eip1898::{BlockHashOrNumber, HashOrNumber};
pub use alloy_sips::eip2718::Encodable2718;
