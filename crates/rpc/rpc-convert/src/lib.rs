//! Rsil compatibility and utils for RPC types
//!
//! This crate various helper functions to convert between rsil primitive types and rpc types.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod rpc;
pub mod transaction;

pub use rpc::*;
pub use transaction::{RpcConvert, RpcConverter, TransactionConversionError};

pub use alloy_evm::rpc::{CallFees, CallFeesError, SilTxEnvError, TryIntoTxEnv};

// Re-export traits from rsil-rpc-traits
pub use rsil_rpc_traits::{
    FromConsensusHeader, FromConsensusTx, SignTxRequestError, SignableTxRequest, TryIntoSimTx,
    TxInfoMapper,
};
