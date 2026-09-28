//! Storage metadata models.

use bytes::{BufMut, BytesMut};
use modular_bitfield::prelude::*;
use reth_codecs::{add_arbitrary_tests, Compact};
use serde::{Deserialize, Serialize};

/// Storage configuration settings for this node.
///
/// Controls whether this node uses v2 storage layout (static files + `RocksDB` routing)
/// or v1/legacy layout (everything in MDBX).
///
/// These should be set during `init_genesis` or `init_db` depending on whether we want dictate
/// behaviour of new or old nodes respectively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "arbitrary"), derive(arbitrary::Arbitrary))]
#[add_arbitrary_tests(compact)]
pub struct StorageSettings {
    /// Whether this node uses v2 storage layout.
    ///
    /// When `true`, enables all v2 storage features:
    /// - Receipts and transaction senders in static files
    /// - History indices in `RocksDB` (accounts, storages, transaction hashes)
    /// - Account and storage changesets in static files
    /// - Hashed state tables as canonical state representation
    ///
    /// When `false`, uses v1/legacy layout (everything in MDBX).
    pub storage_v2: bool,
}

#[bitfield]
#[derive(Clone, Copy, Debug, Default)]
struct StorageSettingsFlags {
    storage_v2_len: B1,
    #[skip]
    unused: B7,
}

impl StorageSettings {
    /// Used bytes by the compact bitfield.
    pub const fn bitflag_encoded_bytes() -> usize {
        1
    }

    /// Unused bits available in the compact bitfield.
    pub const fn bitflag_unused_bits() -> usize {
        7
    }
}

impl Compact for StorageSettings {
    fn to_compact<B>(&self, buf: &mut B) -> usize
    where
        B: BufMut + AsMut<[u8]>,
    {
        let mut payload = BytesMut::new();
        let mut flags = StorageSettingsFlags::default();
        flags.set_storage_v2_len(self.storage_v2.to_compact(&mut payload) as u8);
        buf.put_slice(&flags.into_bytes());
        buf.put_slice(&payload);
        1 + payload.len()
    }

    fn from_compact(buf: &[u8], _len: usize) -> (Self, &[u8]) {
        let flags = StorageSettingsFlags::from_bytes([buf[0]]);
        let payload = &buf[1..];
        let (storage_v2, payload) =
            bool::from_compact(payload, flags.storage_v2_len() as usize);
        (Self { storage_v2 }, payload)
    }
}

impl StorageSettings {
    /// Returns the default base `StorageSettings`.
    pub const fn base() -> Self {
        Self::v2()
    }

    /// Creates `StorageSettings` for v2 nodes with all storage features enabled:
    /// - Receipts and transaction senders in static files
    /// - History indices in `RocksDB` (storages, accounts, transaction hashes)
    /// - Account and storage changesets in static files
    /// - Hashed state as canonical state representation
    ///
    /// Use this when the `--storage.v2` CLI flag is set.
    pub const fn v2() -> Self {
        Self { storage_v2: true }
    }

    /// Creates `StorageSettings` for v1/legacy nodes.
    ///
    /// This keeps all data in MDBX, matching the original storage layout.
    pub const fn v1() -> Self {
        Self { storage_v2: false }
    }

    /// Returns `true` if this node uses v2 storage layout.
    pub const fn is_v2(&self) -> bool {
        self.storage_v2
    }

    /// Whether receipts are stored in static files.
    pub const fn receipts_in_static_files(&self) -> bool {
        self.storage_v2
    }

    /// Whether transaction senders are stored in static files.
    pub const fn transaction_senders_in_static_files(&self) -> bool {
        self.storage_v2
    }

    /// Whether storages history is stored in `RocksDB`.
    pub const fn storages_history_in_rocksdb(&self) -> bool {
        self.storage_v2
    }

    /// Whether transaction hash numbers are stored in `RocksDB`.
    pub const fn transaction_hash_numbers_in_rocksdb(&self) -> bool {
        self.storage_v2
    }

    /// Whether account history is stored in `RocksDB`.
    pub const fn account_history_in_rocksdb(&self) -> bool {
        self.storage_v2
    }

    /// Whether to use hashed state tables (`HashedAccounts`/`HashedStorages`) as the canonical
    /// state representation instead of plain state tables. Implied by v2 storage layout.
    pub const fn use_hashed_state(&self) -> bool {
        self.storage_v2
    }

    /// Returns `true` if any tables are configured to be stored in `RocksDB`.
    pub const fn any_in_rocksdb(&self) -> bool {
        self.storage_v2
    }
}
