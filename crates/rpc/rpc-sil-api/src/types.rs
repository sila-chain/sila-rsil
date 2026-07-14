//! Trait for specifying `sil` network dependent API types.

use crate::{AsEthApiError, FromEthApiError, RpcNodeCore};
use alloy_rpc_types_eth::Block;
use rsil_rpc_convert::{RpcConvert, SignableTxRequest};
pub use rsil_rpc_convert::{RpcTransaction, RpcTxReq, RpcTypes};
use rsil_storage_api::ProviderTx;
use std::error::Error;

/// Network specific `sil` API types.
///
/// This trait defines the network specific rpc types and helpers required for the `eth_` and
/// adjacent endpoints. `NetworkTypes` is [`alloy_network::Network`] as defined by the alloy crate,
/// see also [`alloy_network::Sila`].
///
/// This type is stateful so that it can provide additional context if necessary, e.g. populating
/// receipts with additional data.
pub trait SilApiTypes: Send + Sync + Clone {
    /// Extension of [`FromEthApiError`], with network specific errors.
    type Error: Into<jsonrpsee_types::error::ErrorObject<'static>>
        + FromEthApiError
        + AsEthApiError
        + From<<Self::RpcConvert as RpcConvert>::Error>
        + Error
        + Send
        + Sync;
    /// Blockchain primitive types, specific to network, e.g. block and transaction.
    type NetworkTypes: RpcTypes;
    /// Conversion methods for transaction RPC type.
    type RpcConvert: RpcConvert<Network = Self::NetworkTypes>;

    /// Returns reference to transaction response builder.
    fn converter(&self) -> &Self::RpcConvert;
}

/// Adapter for network specific block type.
pub type RpcBlock<T> = Block<RpcTransaction<T>, RpcHeader<T>>;

/// Adapter for network specific receipt type.
pub type RpcReceipt<T> = <T as RpcTypes>::Receipt;

/// Adapter for network specific header type.
pub type RpcHeader<T> = <T as RpcTypes>::Header;

/// Adapter for network specific error type.
pub type RpcError<T> = <T as SilApiTypes>::Error;

/// Helper trait holds necessary trait bounds on [`SilApiTypes`] to implement `sil` API.
pub trait FullEthApiTypes
where
    Self: RpcNodeCore
        + SilApiTypes<
            NetworkTypes: RpcTypes<
                TransactionRequest: SignableTxRequest<ProviderTx<Self::Provider>>,
            >,
            RpcConvert: RpcConvert<Primitives = Self::Primitives>,
        >,
{
}

impl<T> FullEthApiTypes for T where
    T: RpcNodeCore
        + SilApiTypes<
            NetworkTypes: RpcTypes<
                TransactionRequest: SignableTxRequest<ProviderTx<Self::Provider>>,
            >,
            RpcConvert: RpcConvert<Primitives = Self::Primitives>,
        >
{
}
