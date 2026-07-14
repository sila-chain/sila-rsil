use crate::{
    providers::{NodeTypesForProvider, ProviderNodeTypes, RocksDBBuilder, StaticFileProvider},
    HashingWriter, ProviderFactory, TrieWriter,
};
use alloy_primitives::B256;
use rsil_chainspec::{ChainSpec, SILA_MAINNET};
use rsil_db::{mdbx::DatabaseArguments, test_utils::TempDatabase, DatabaseEnv};
use rsil_errors::ProviderResult;
use rsil_sila_engine_primitives::SilEngineTypes;
use rsil_node_types::NodeTypesWithDBAdapter;
use rsil_primitives_traits::{Account, StorageEntry};
use rsil_storage_api::StorageSettingsCache;
use rsil_trie::StateRoot;
use rsil_trie_db::DatabaseStateRoot;
use std::sync::Arc;

type DbStateRoot<'a, TX, A> = StateRoot<
    rsil_trie_db::DatabaseTrieCursorFactory<&'a TX, A>,
    rsil_trie_db::DatabaseHashedCursorFactory<&'a TX>,
>;

pub mod blocks;
mod mock;
mod noop;

pub use mock::{ExtendedAccount, MockEthProvider};
pub use noop::NoopProvider;
pub use rsil_chain_state::test_utils::TestCanonStateSubscriptions;

/// Mock [`rsil_node_types::NodeTypes`] for testing.
pub type MockNodeTypes = rsil_node_types::AnyNodeTypesWithEngine<
    rsil_sila_primitives::SilPrimitives,
    rsil_sila_engine_primitives::SilEngineTypes,
    rsil_chainspec::ChainSpec,
    crate::SilStorage,
    SilEngineTypes,
>;

/// Mock [`rsil_node_types::NodeTypesWithDB`] for testing.
pub type MockNodeTypesWithDB<DB = Arc<TempDatabase<DatabaseEnv>>> =
    NodeTypesWithDBAdapter<MockNodeTypes, DB>;

/// Creates test provider factory with sila-mainnet chain spec.
pub fn create_test_provider_factory() -> ProviderFactory<MockNodeTypesWithDB> {
    create_test_provider_factory_with_chain_spec(SILA_MAINNET.clone())
}

/// Creates test provider factory with provided chain spec.
pub fn create_test_provider_factory_with_chain_spec(
    chain_spec: Arc<ChainSpec>,
) -> ProviderFactory<MockNodeTypesWithDB> {
    create_test_provider_factory_with_node_types::<MockNodeTypes>(chain_spec)
}

/// Creates test provider factory with provided chain spec.
pub fn create_test_provider_factory_with_node_types<N: NodeTypesForProvider>(
    chain_spec: Arc<N::ChainSpec>,
) -> ProviderFactory<NodeTypesWithDBAdapter<N, Arc<TempDatabase<DatabaseEnv>>>> {
    // Create a single temp directory that contains all data dirs (db, static_files, rocksdb).
    // TempDatabase will clean up the entire directory on drop.
    let datadir_path = rsil_db::test_utils::tempdir_path();

    let static_files_path = datadir_path.join("static_files");
    let rocksdb_path = datadir_path.join("rocksdb");

    // Create static_files directory
    std::fs::create_dir_all(&static_files_path).expect("failed to create static_files dir");

    // Create database with the datadir path so TempDatabase cleans up everything on drop
    let db = rsil_db::test_utils::create_test_rw_db_with_datadir(&datadir_path);

    ProviderFactory::new(
        db,
        chain_spec,
        StaticFileProvider::read_write(static_files_path).expect("static file provider"),
        RocksDBBuilder::new(&rocksdb_path)
            .with_default_tables()
            .build()
            .expect("failed to create test RocksDB provider"),
        rsil_tasks::Runtime::test(),
    )
    .expect("failed to create test provider factory")
}

/// Creates test provider factory with provided chain spec and custom database arguments.
///
/// Same as [`create_test_provider_factory_with_chain_spec`] but allows overriding the default
/// test database arguments (e.g. to increase the MDBX geometry for heavy benchmarks).
pub fn create_test_provider_factory_with_chain_spec_and_db_args(
    chain_spec: Arc<ChainSpec>,
    db_args: DatabaseArguments,
) -> ProviderFactory<MockNodeTypesWithDB> {
    let datadir_path = rsil_db::test_utils::tempdir_path();

    let db_path = datadir_path.join("db");
    let static_files_path = datadir_path.join("static_files");
    let rocksdb_path = datadir_path.join("rocksdb");

    std::fs::create_dir_all(&static_files_path).expect("failed to create static_files dir");

    let db = rsil_db::init_db(&db_path, db_args).expect("failed to init db");
    let db = Arc::new(TempDatabase::new(db, datadir_path));

    ProviderFactory::new(
        db,
        chain_spec,
        StaticFileProvider::read_write(static_files_path).expect("static file provider"),
        RocksDBBuilder::new(&rocksdb_path)
            .with_default_tables()
            .build()
            .expect("failed to create test RocksDB provider"),
        rsil_tasks::Runtime::test(),
    )
    .expect("failed to create test provider factory")
}

/// Inserts the genesis alloc from the provided chain spec into the trie.
pub fn insert_genesis<N: ProviderNodeTypes<ChainSpec = ChainSpec>>(
    provider_factory: &ProviderFactory<N>,
    chain_spec: Arc<N::ChainSpec>,
) -> ProviderResult<B256> {
    let provider = provider_factory.provider_rw()?;

    // Hash accounts and insert them into hashing table.
    let genesis = chain_spec.genesis();
    let alloc_accounts =
        genesis.alloc.iter().map(|(addr, account)| (*addr, Some(Account::from(account))));
    provider.insert_account_for_hashing(alloc_accounts).unwrap();

    let alloc_storage = genesis.alloc.clone().into_iter().filter_map(|(addr, account)| {
        // Only return `Some` if there is storage.
        account.storage.map(|storage| {
            (
                addr,
                storage.into_iter().map(|(key, value)| StorageEntry { key, value: value.into() }),
            )
        })
    });
    provider.insert_storage_for_hashing(alloc_storage)?;

    let (root, updates) = rsil_trie_db::with_adapter!(provider, |A| {
        DbStateRoot::<_, A>::from_tx(provider.tx_ref()).root_with_updates()?
    });
    provider.write_trie_updates(updates).unwrap();

    provider.commit()?;

    Ok(root)
}
