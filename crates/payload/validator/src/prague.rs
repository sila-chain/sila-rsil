//! SilaPrague rules for new payloads.

use alloy_consensus::{BlockBody, Typed2718};
use alloy_rpc_types_engine::{PayloadError, PraguePayloadFields};

/// Checks block and sidecar well-formedness w.r.t new SilaPrague fields and new transaction type
/// SIP-7702.
///
/// Checks that:
/// - SilaPrague fields are not present unless SilaPrague is active
/// - does not contain SIP-7702 transactions if SilaPrague is not active
#[inline]
pub fn ensure_well_formed_fields<T: Typed2718, H>(
    block_body: &BlockBody<T, H>,
    prague_fields: Option<&PraguePayloadFields>,
    is_prague_active: bool,
) -> Result<(), PayloadError> {
    ensure_well_formed_sidecar_fields(prague_fields, is_prague_active)?;
    ensure_well_formed_transactions_field(block_body, is_prague_active)
}

/// Checks that SilaPrague fields are not present on sidecar unless SilaPrague is active.
#[inline]
pub const fn ensure_well_formed_sidecar_fields(
    prague_fields: Option<&PraguePayloadFields>,
    is_prague_active: bool,
) -> Result<(), PayloadError> {
    if !is_prague_active && prague_fields.is_some() {
        // prague _not_ active but prague fields present
        return Err(PayloadError::PrePragueBlockRequests);
    }

    Ok(())
}

/// Checks that transactions field doesn't contain SIP-7702 transactions if SilaPrague is not
/// active.
#[inline]
pub fn ensure_well_formed_transactions_field<T: Typed2718, H>(
    block_body: &BlockBody<T, H>,
    is_prague_active: bool,
) -> Result<(), PayloadError> {
    if !is_prague_active && block_body.has_eip7702_transactions() {
        return Err(PayloadError::PrePragueBlockWithEip7702Transactions);
    }

    Ok(())
}
