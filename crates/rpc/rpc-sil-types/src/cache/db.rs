//! Helper types to workaround 'higher-ranked lifetime error'
//! <https://github.com/rust-lang/rust/issues/100013> in default implementation of
//! `rsil_rpc_eth_api::helpers::Call`.

use alloy_primitives::{Address, B256, U256};
use rsil_errors::ProviderResult;
use rsil_revm::database::StateProviderDatabase;
use rsil_storage_api::{BytecodeReader, HashedPostStateProvider, StateProvider, StateProviderBox};
use rsil_trie::{HashedStorage, MultiProofTargets};
use revm::database::{BundleState, State};

/// Helper alias type for the state's [`State`]
pub type StateCacheDb = State<StateProviderDatabase<StateProviderTraitObjWrapper>>;

/// Hack to get around 'higher-ranked lifetime error', see
/// <https://github.com/rust-lang/rust/issues/100013>
///
/// Apparently, when dealing with our RPC code, compiler is struggling to prove lifetimes around
/// [`StateProvider`] trait objects. This type is a workaround which should help the compiler to
/// understand that there are no lifetimes involved.
#[expect(missing_debug_implementations)]
pub struct StateProviderTraitObjWrapper(pub StateProviderBox);

impl rsil_storage_api::StateRootProvider for StateProviderTraitObjWrapper {
    fn state_root(
        &self,
        hashed_state: rsil_trie::HashedPostState,
    ) -> rsil_errors::ProviderResult<B256> {
        self.0.state_root(hashed_state)
    }

    fn state_root_from_nodes(
        &self,
        input: rsil_trie::TrieInput,
    ) -> rsil_errors::ProviderResult<B256> {
        self.0.state_root_from_nodes(input)
    }

    fn state_root_with_updates(
        &self,
        hashed_state: rsil_trie::HashedPostState,
    ) -> rsil_errors::ProviderResult<(B256, rsil_trie::updates::TrieUpdates)> {
        self.0.state_root_with_updates(hashed_state)
    }

    fn state_root_from_nodes_with_updates(
        &self,
        input: rsil_trie::TrieInput,
    ) -> rsil_errors::ProviderResult<(B256, rsil_trie::updates::TrieUpdates)> {
        self.0.state_root_from_nodes_with_updates(input)
    }
}

impl rsil_storage_api::StorageRootProvider for StateProviderTraitObjWrapper {
    fn storage_root(
        &self,
        address: Address,
        hashed_storage: HashedStorage,
    ) -> ProviderResult<B256> {
        self.0.storage_root(address, hashed_storage)
    }

    fn storage_proof(
        &self,
        address: Address,
        slot: B256,
        hashed_storage: HashedStorage,
    ) -> ProviderResult<rsil_trie::StorageProof> {
        self.0.storage_proof(address, slot, hashed_storage)
    }

    fn storage_multiproof(
        &self,
        address: Address,
        slots: &[B256],
        hashed_storage: HashedStorage,
    ) -> ProviderResult<rsil_trie::StorageMultiProof> {
        self.0.storage_multiproof(address, slots, hashed_storage)
    }
}

impl rsil_storage_api::StateProofProvider for StateProviderTraitObjWrapper {
    fn proof(
        &self,
        input: rsil_trie::TrieInput,
        address: Address,
        slots: &[B256],
    ) -> rsil_errors::ProviderResult<rsil_trie::AccountProof> {
        self.0.proof(input, address, slots)
    }

    fn multiproof(
        &self,
        input: rsil_trie::TrieInput,
        targets: MultiProofTargets,
    ) -> ProviderResult<rsil_trie::MultiProof> {
        self.0.multiproof(input, targets)
    }

    fn witness(
        &self,
        input: rsil_trie::TrieInput,
        target: rsil_trie::HashedPostState,
        mode: rsil_trie::ExecutionWitnessMode,
    ) -> rsil_errors::ProviderResult<Vec<alloy_primitives::Bytes>> {
        self.0.witness(input, target, mode)
    }
}

impl rsil_storage_api::AccountReader for StateProviderTraitObjWrapper {
    fn basic_account(
        &self,
        address: &Address,
    ) -> rsil_errors::ProviderResult<Option<rsil_primitives_traits::Account>> {
        self.0.basic_account(address)
    }
}

impl rsil_storage_api::BlockHashReader for StateProviderTraitObjWrapper {
    fn block_hash(
        &self,
        block_number: alloy_primitives::BlockNumber,
    ) -> rsil_errors::ProviderResult<Option<B256>> {
        self.0.block_hash(block_number)
    }

    fn convert_block_hash(
        &self,
        hash_or_number: alloy_rpc_types_eth::BlockHashOrNumber,
    ) -> rsil_errors::ProviderResult<Option<B256>> {
        self.0.convert_block_hash(hash_or_number)
    }

    fn canonical_hashes_range(
        &self,
        start: alloy_primitives::BlockNumber,
        end: alloy_primitives::BlockNumber,
    ) -> rsil_errors::ProviderResult<Vec<B256>> {
        self.0.canonical_hashes_range(start, end)
    }
}

impl HashedPostStateProvider for StateProviderTraitObjWrapper {
    fn hashed_post_state(&self, bundle_state: &BundleState) -> rsil_trie::HashedPostState {
        self.0.hashed_post_state(bundle_state)
    }
}

impl StateProvider for StateProviderTraitObjWrapper {
    fn storage(
        &self,
        account: Address,
        storage_key: alloy_primitives::StorageKey,
    ) -> rsil_errors::ProviderResult<Option<alloy_primitives::StorageValue>> {
        self.0.storage(account, storage_key)
    }

    fn account_code(
        &self,
        addr: &Address,
    ) -> rsil_errors::ProviderResult<Option<rsil_primitives_traits::Bytecode>> {
        self.0.account_code(addr)
    }

    fn account_balance(&self, addr: &Address) -> rsil_errors::ProviderResult<Option<U256>> {
        self.0.account_balance(addr)
    }

    fn account_nonce(&self, addr: &Address) -> rsil_errors::ProviderResult<Option<u64>> {
        self.0.account_nonce(addr)
    }
}

impl BytecodeReader for StateProviderTraitObjWrapper {
    fn bytecode_by_hash(
        &self,
        code_hash: &B256,
    ) -> rsil_errors::ProviderResult<Option<rsil_primitives_traits::Bytecode>> {
        self.0.bytecode_by_hash(code_hash)
    }
}
