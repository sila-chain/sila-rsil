use crate::{providers::NodeTypesForProvider, DatabaseProvider};
use rsil_db_api::transaction::{DbTx, DbTxMut};
use rsil_node_types::NodePrimitives;

use rsil_primitives_traits::{FullBlockHeader, FullSignedTx};
use rsil_storage_api::{ChainStorageReader, ChainStorageWriter, EmptyBodyStorage, SilStorage};

/// Trait that provides access to implementations of [`ChainStorage`]
pub trait ChainStorage<Primitives: NodePrimitives>: Send + Sync {
    /// Provides access to the chain reader.
    fn reader<TX, Types>(&self) -> impl ChainStorageReader<DatabaseProvider<TX, Types>, Primitives>
    where
        TX: DbTx + 'static,
        Types: NodeTypesForProvider<Primitives = Primitives>;

    /// Provides access to the chain writer.
    fn writer<TX, Types>(&self) -> impl ChainStorageWriter<DatabaseProvider<TX, Types>, Primitives>
    where
        TX: DbTxMut + DbTx + 'static,
        Types: NodeTypesForProvider<Primitives = Primitives>;
}

impl<N, T, H> ChainStorage<N> for SilStorage<T, H>
where
    T: FullSignedTx,
    H: FullBlockHeader,
    N: NodePrimitives<
        Block = alloy_consensus::Block<T, H>,
        BlockHeader = H,
        BlockBody = alloy_consensus::BlockBody<T, H>,
        SignedTx = T,
    >,
{
    fn reader<TX, Types>(&self) -> impl ChainStorageReader<DatabaseProvider<TX, Types>, N>
    where
        TX: DbTx + 'static,
        Types: NodeTypesForProvider<Primitives = N>,
    {
        self
    }

    fn writer<TX, Types>(&self) -> impl ChainStorageWriter<DatabaseProvider<TX, Types>, N>
    where
        TX: DbTxMut + DbTx + 'static,
        Types: NodeTypesForProvider<Primitives = N>,
    {
        self
    }
}

impl<N, T, H> ChainStorage<N> for EmptyBodyStorage<T, H>
where
    T: FullSignedTx,
    H: FullBlockHeader,
    N: NodePrimitives<
        Block = alloy_consensus::Block<T, H>,
        BlockHeader = H,
        BlockBody = alloy_consensus::BlockBody<T, H>,
        SignedTx = T,
    >,
{
    fn reader<TX, Types>(&self) -> impl ChainStorageReader<DatabaseProvider<TX, Types>, N>
    where
        TX: DbTx + 'static,
        Types: NodeTypesForProvider<Primitives = N>,
    {
        self
    }

    fn writer<TX, Types>(&self) -> impl ChainStorageWriter<DatabaseProvider<TX, Types>, N>
    where
        TX: DbTxMut + DbTx + 'static,
        Types: NodeTypesForProvider<Primitives = N>,
    {
        self
    }
}
