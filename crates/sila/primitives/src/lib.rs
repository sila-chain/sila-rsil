//! Standalone crate for sila-specific Rsil primitive types.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

// Feature-only dep: activated by `rsil-codec` feature for downstream consumers.
#[cfg(feature = "rsil-codec")]
use rsil_codecs as _;

mod receipt;
pub use receipt::*;

pub use alloy_consensus::{transaction::PooledTransaction, TxType};
use alloy_consensus::{TxEip4844, TxEip4844WithSidecar};
use alloy_eips::sip7594::BlobTransactionSidecarVariant;

/// Typed Transaction type without a signature
pub type Transaction = alloy_consensus::SilaTypedTransaction<TxEip4844>;

/// Signed transaction.
pub type TransactionSigned = alloy_consensus::SilaTxEnvelope<TxEip4844>;

/// A type alias for [`PooledTransaction`] that's also generic over blob sidecar.
pub type PooledTransactionVariant =
    alloy_consensus::SilaTxEnvelope<TxEip4844WithSidecar<BlobTransactionSidecarVariant>>;

/// Type alias for the sila block
pub type Block = alloy_consensus::Block<TransactionSigned>;

/// Type alias for the sila blockbody
pub type BlockBody = alloy_consensus::BlockBody<TransactionSigned>;

/// Helper struct that specifies the sila
/// [`NodePrimitives`](rsil_primitives_traits::NodePrimitives) types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct SilPrimitives;

impl rsil_primitives_traits::NodePrimitives for SilPrimitives {
    type Block = crate::Block;
    type BlockHeader = alloy_consensus::Header;
    type BlockBody = crate::BlockBody;
    type SignedTx = crate::TransactionSigned;
    type Receipt = crate::Receipt;
}
