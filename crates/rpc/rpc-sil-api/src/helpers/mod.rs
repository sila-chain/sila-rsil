//! Behaviour needed to serve `eth_` RPC requests, divided into general database reads and
//! specific database access.
//!
//! Traits with `Load` prefix, read atomic data from database, e.g. a block or transaction. Any
//! database read done in more than one default `Sil` trait implementation, is defined in a `Load`
//! trait.
//!
//! Traits with `Sil` prefix, compose specific data needed to serve RPC requests in the `sil`
//! namespace. They use `Load` traits as building blocks. [`SilTransactions`] also writes data
//! (submits transactions). Based on the `eth_` request method semantics, request methods are
//! divided into: [`SilTransactions`], [`SilBlocks`], [`SilFees`], [`SilState`] and [`SilCall`].
//! Default implementation of the `Sil` traits, is done w.r.t. L1.
//!
//! [`SilApiServer`](crate::SilApiServer), is implemented for any type that implements
//! all the `Sil` traits, e.g. `rsil_rpc::SilApi`.

pub mod bal;
pub mod block;
pub mod blocking_task;
pub mod call;
pub mod config;
pub mod estimate;
pub mod fee;
pub mod pending_block;
pub mod receipt;
pub mod signer;
pub mod spec;
pub mod state;
pub mod subscriptions;
pub mod trace;
pub mod transaction;

pub use bal::GetBlockAccessList;
pub use block::{LoadBlock, SilBlocks};
pub use blocking_task::SpawnBlocking;
pub use call::{Call, SilCall};
pub use fee::{LoadFee, SilFees};
pub use pending_block::LoadPendingBlock;
pub use receipt::LoadReceipt;
pub use signer::SilSigner;
pub use spec::SilApiSpec;
pub use state::{LoadState, SilState};
pub use subscriptions::SilSubscriptions;
pub use trace::Trace;
pub use transaction::{LoadTransaction, SilTransactions};

use crate::FullEthApiTypes;

/// Extension trait that bundles traits needed for tracing transactions.
pub trait TraceExt:
    LoadTransaction + LoadBlock + SpawnBlocking + Trace + Call + GetBlockAccessList
{
}

impl<T> TraceExt for T where T: LoadTransaction + LoadBlock + Trace + Call + GetBlockAccessList {}

/// Helper trait to unify all `sil` rpc server building block traits, for simplicity.
///
/// This trait is automatically implemented for any type that implements all the `Sil` traits.
pub trait FullEthApi:
    FullEthApiTypes
    + SilApiSpec
    + SilTransactions
    + SilBlocks
    + SilState
    + SilCall
    + SilFees
    + SilSubscriptions
    + Trace
    + LoadReceipt
    + GetBlockAccessList
{
}

impl<T> FullEthApi for T where
    T: FullEthApiTypes
        + SilApiSpec
        + SilTransactions
        + SilBlocks
        + SilState
        + SilCall
        + SilFees
        + SilSubscriptions
        + Trace
        + LoadReceipt
        + GetBlockAccessList
{
}
