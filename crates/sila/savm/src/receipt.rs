use alloy_consensus::TxType;
use alloy_savm::eth::receipt_builder::{ReceiptBuilder, ReceiptBuilderCtx};
use rsil_sila_primitives::{Receipt, TransactionSigned};
use rsil_savm::Savm;

/// A builder that operates on Rsil primitive types, specifically [`TransactionSigned`] and
/// [`Receipt`].
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct RsilReceiptBuilder;

impl ReceiptBuilder for RsilReceiptBuilder {
    type Transaction = TransactionSigned;
    type Receipt = Receipt;

    fn build_receipt<E: Savm>(&self, ctx: ReceiptBuilderCtx<'_, TxType, E>) -> Self::Receipt {
        let ReceiptBuilderCtx { tx_type, result, cumulative_gas_used, .. } = ctx;
        Receipt {
            tx_type,
            // Success flag was added in `SIP-658: Embedding transaction status code in
            // receipts`.
            success: result.is_success(),
            cumulative_gas_used,
            logs: result.into_logs(),
        }
    }
}
