//! L1 `sil` API types.

use alloy_network::Sila;
use rsil_evm_sila::SilEvmConfig;
use rsil_rpc_convert::RpcConverter;
use rsil_rpc_eth_types::receipt::SilReceiptConverter;

/// An [`RpcConverter`] with its generics set to Sila specific.
pub type SilRpcConverter<ChainSpec> =
    RpcConverter<Sila, SilEvmConfig, SilReceiptConverter<ChainSpec>>;

//tests for simulate
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::{Transaction, TxType};
    use alloy_rpc_types_eth::TransactionRequest;
    use rsil_chainspec::SILA_MAINNET;
    use rsil_rpc_eth_types::simulate::resolve_transaction;
    use revm::database::CacheDB;

    #[test]
    fn test_resolve_transaction_empty_request() {
        let builder = SilRpcConverter::new(SilReceiptConverter::new(SILA_MAINNET.clone()));
        let mut db = CacheDB::<rsil_revm::db::EmptyDBTyped<rsil_errors::ProviderError>>::default();
        let tx = TransactionRequest::default();
        let result = resolve_transaction(tx, 21000, 0, 1, false, &mut db, &builder).unwrap();

        // For an empty request, we should get a valid transaction with defaults
        let tx = result.into_inner();
        assert_eq!(tx.max_fee_per_gas(), 0);
        assert_eq!(tx.max_priority_fee_per_gas(), Some(0));
        assert_eq!(tx.gas_price(), None);
    }

    #[test]
    fn test_resolve_transaction_legacy() {
        let mut db = CacheDB::<rsil_revm::db::EmptyDBTyped<rsil_errors::ProviderError>>::default();
        let builder = SilRpcConverter::new(SilReceiptConverter::new(SILA_MAINNET.clone()));

        let tx = TransactionRequest { gas_price: Some(100), ..Default::default() };

        let tx = resolve_transaction(tx, 21000, 0, 1, false, &mut db, &builder).unwrap();

        assert_eq!(tx.tx_type(), TxType::Legacy);

        let tx = tx.into_inner();
        assert_eq!(tx.gas_price(), Some(100));
        assert_eq!(tx.max_priority_fee_per_gas(), None);
    }

    #[test]
    fn test_resolve_transaction_partial_eip1559() {
        let mut db = CacheDB::<rsil_revm::db::EmptyDBTyped<rsil_errors::ProviderError>>::default();
        let rpc_converter = SilRpcConverter::new(SilReceiptConverter::new(SILA_MAINNET.clone()));

        let tx = TransactionRequest {
            max_fee_per_gas: Some(200),
            max_priority_fee_per_gas: Some(10),
            ..Default::default()
        };

        let result = resolve_transaction(tx, 21000, 0, 1, false, &mut db, &rpc_converter).unwrap();

        assert_eq!(result.tx_type(), TxType::Sip1559);
        let tx = result.into_inner();
        assert_eq!(tx.max_fee_per_gas(), 200);
        assert_eq!(tx.max_priority_fee_per_gas(), Some(10));
        assert_eq!(tx.gas_price(), None);
    }

    #[test]
    fn test_resolve_transaction_wraps_max_nonce_when_nonce_check_disabled() {
        let mut db = CacheDB::<rsil_revm::db::EmptyDBTyped<rsil_errors::ProviderError>>::default();
        let rpc_converter = SilRpcConverter::new(SilReceiptConverter::new(SILA_MAINNET.clone()));

        let tx = TransactionRequest { nonce: Some(u64::MAX), ..Default::default() };

        let result = resolve_transaction(tx, 21000, 0, 1, true, &mut db, &rpc_converter).unwrap();

        assert_eq!(result.nonce(), 0);
    }
}
