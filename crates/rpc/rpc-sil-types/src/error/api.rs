//! Helper traits to wrap generic l1 errors, in network specific error type configured in
//! `rsil_rpc_eth_api::SilApiTypes`.

use crate::{simulate::SilSimulateError, RevertError, SilApiError};
use alloy_primitives::Bytes;
use revm::{context::result::ExecutionResult, context_interface::result::HaltReason};
use rsil_errors::ProviderError;
use rsil_evm::{ConfigureEvm, HaltReasonFor, SavmErrorFor};
use rsil_revm::db::bal::SavmDatabaseError;

use super::RpcInvalidTransactionError;

/// Helper trait to wrap core [`SilApiError`].
pub trait FromEthApiError: From<SilApiError> {
    /// Converts from error via [`SilApiError`].
    fn from_eth_err<E>(err: E) -> Self
    where
        SilApiError: From<E>;
}

impl<T> FromEthApiError for T
where
    T: From<SilApiError>,
{
    fn from_eth_err<E>(err: E) -> Self
    where
        SilApiError: From<E>,
    {
        T::from(SilApiError::from(err))
    }
}

/// Helper trait to wrap core [`SilApiError`].
pub trait IntoEthApiError: Into<SilApiError> {
    /// Converts into error via [`SilApiError`].
    fn into_eth_err<E>(self) -> E
    where
        E: FromEthApiError;
}

impl<T> IntoEthApiError for T
where
    SilApiError: From<T>,
{
    fn into_eth_err<E>(self) -> E
    where
        E: FromEthApiError,
    {
        E::from_eth_err(self)
    }
}

/// Helper trait to access wrapped core error.
pub trait AsEthApiError {
    /// Returns a reference to [`SilApiError`] if this is an error variant inherited from core
    /// functionality.
    fn as_err(&self) -> Option<&SilApiError>;

    /// Returns `true` if error is
    /// [`RpcInvalidTransactionError::GasTooHigh`].
    fn is_gas_too_high(&self) -> bool {
        if let Some(err) = self.as_err() {
            return err.is_gas_too_high();
        }

        false
    }

    /// Returns `true` if error is
    /// [`RpcInvalidTransactionError::GasTooLow`].
    fn is_gas_too_low(&self) -> bool {
        if let Some(err) = self.as_err() {
            return err.is_gas_too_low();
        }

        false
    }

    /// Returns [`SilSimulateError`] if this error maps to a simulate-specific error code.
    fn as_simulate_error(&self) -> Option<SilSimulateError> {
        let err = self.as_err()?;
        match err {
            SilApiError::InvalidTransaction(tx_err) => match tx_err {
                RpcInvalidTransactionError::NonceTooLow { tx, state } => {
                    Some(SilSimulateError::NonceTooLow { tx: *tx, state: *state })
                }
                RpcInvalidTransactionError::NonceTooHigh => Some(SilSimulateError::NonceTooHigh),
                RpcInvalidTransactionError::NonceMaxValue => Some(SilSimulateError::NonceMaxValue),
                RpcInvalidTransactionError::FeeCapTooLow => {
                    Some(SilSimulateError::BaseFeePerGasTooLow)
                }
                RpcInvalidTransactionError::GasTooLow => Some(SilSimulateError::IntrinsicGasTooLow),
                RpcInvalidTransactionError::InsufficientFunds { cost, balance } => {
                    Some(SilSimulateError::InsufficientFunds { cost: *cost, balance: *balance })
                }
                RpcInvalidTransactionError::SenderNoEOA => Some(SilSimulateError::SenderNotEOA),
                RpcInvalidTransactionError::MaxInitCodeSizeExceeded => {
                    Some(SilSimulateError::MaxInitCodeSizeExceeded)
                }
                _ => None,
            },
            _ => None,
        }
    }
}

impl AsEthApiError for SilApiError {
    fn as_err(&self) -> Option<&SilApiError> {
        Some(self)
    }
}

/// Helper trait to convert from revm errors.
pub trait FromEvmError<Savm: ConfigureEvm>:
    From<SavmErrorFor<Savm, SavmDatabaseError<ProviderError>>>
    + FromEvmHalt<HaltReasonFor<Savm>>
    + FromRevert
{
    /// Converts from SAVM error to this type.
    fn from_evm_err(err: SavmErrorFor<Savm, SavmDatabaseError<ProviderError>>) -> Self {
        err.into()
    }

    /// Ensures the execution result is successful or returns an error,
    fn ensure_success(result: ExecutionResult<HaltReasonFor<Savm>>) -> Result<Bytes, Self> {
        match result {
            ExecutionResult::Success { output, .. } => Ok(output.into_data()),
            ExecutionResult::Revert { output, .. } => Err(Self::from_revert(output)),
            ExecutionResult::Halt { reason, gas, .. } => {
                Err(Self::from_evm_halt(reason, gas.tx_gas_used()))
            }
        }
    }
}

impl<T, Savm> FromEvmError<Savm> for T
where
    T: From<SavmErrorFor<Savm, SavmDatabaseError<ProviderError>>>
        + FromEvmHalt<HaltReasonFor<Savm>>
        + FromRevert,
    Savm: ConfigureEvm,
{
}

/// Helper trait to convert from revm errors.
pub trait FromEvmHalt<Halt> {
    /// Converts from SAVM halt to this type.
    fn from_evm_halt(halt: Halt, gas_limit: u64) -> Self;
}

impl FromEvmHalt<HaltReason> for SilApiError {
    fn from_evm_halt(halt: HaltReason, gas_limit: u64) -> Self {
        RpcInvalidTransactionError::halt(halt, gas_limit).into()
    }
}

/// Helper trait to construct errors from unexpected reverts.
pub trait FromRevert {
    /// Constructs an error from revert bytes.
    ///
    /// This is only invoked when revert was unexpected (`eth_call`, `eth_estimateGas`, etc).
    fn from_revert(output: Bytes) -> Self;
}

impl FromRevert for SilApiError {
    fn from_revert(output: Bytes) -> Self {
        RpcInvalidTransactionError::Revert(RevertError::new(output)).into()
    }
}
