//! Helper trait for full rpc provider

use rsil_chainspec::{ChainSpecProvider, SilaHardforks};

use crate::{
    BlockReaderIdExt, HeaderProvider, StageCheckpointReader, StateProviderFactory,
    TransactionsProvider,
};

/// Helper trait to unify all provider traits required to support `sil` RPC server behaviour, for
/// simplicity.
pub trait FullRpcProvider:
    StateProviderFactory
    + ChainSpecProvider<ChainSpec: SilaHardforks>
    + BlockReaderIdExt
    + HeaderProvider
    + TransactionsProvider
    + StageCheckpointReader
    + Clone
    + Unpin
    + 'static
{
}

impl<T> FullRpcProvider for T where
    T: StateProviderFactory
        + ChainSpecProvider<ChainSpec: SilaHardforks>
        + BlockReaderIdExt
        + HeaderProvider
        + TransactionsProvider
        + StageCheckpointReader
        + Clone
        + Unpin
        + 'static
{
}
