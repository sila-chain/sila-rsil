//! Helper macros for implementing traits for various `StateProvider`
//! implementations

/// A macro that delegates trait implementations to the `as_ref` function of the type.
///
/// Used to implement provider traits.
#[macro_export]
macro_rules! delegate_impls_to_as_ref {
    (for $target:ty => $($trait:ident $(where [$($generics:tt)*])? {  $(fn $func:ident$(<$($generic_arg:ident: $generic_arg_ty:path),*>)?(&self, $($arg:ident: $argty:ty),*) -> $ret:path;)* })* ) => {

        $(
          impl<'a, $($($generics)*)?> $trait for $target {
              $(
                  fn $func$(<$($generic_arg: $generic_arg_ty),*>)?(&self, $($arg: $argty),*) -> $ret {
                    self.as_ref().$func($($arg),*)
                  }
              )*
          }
        )*
    };
}

pub use delegate_impls_to_as_ref;

/// Delegates the provider trait implementations to the `as_ref` function of the type:
///
/// [`AccountReader`](crate::AccountReader)
/// [`BlockHashReader`](crate::BlockHashReader)
/// [`StateProvider`](crate::StateProvider)
#[macro_export]
macro_rules! delegate_provider_impls {
    ($target:ty $(where [$($generics:tt)*])?) => {
        $crate::macros::delegate_impls_to_as_ref!(
            for $target =>
            AccountReader $(where [$($generics)*])? {
                fn basic_account(&self, address: &alloy_primitives::Address) -> rsil_storage_api::errors::provider::ProviderResult<Option<rsil_primitives_traits::Account>>;
            }
            BlockHashReader $(where [$($generics)*])? {
                fn block_hash(&self, number: u64) -> rsil_storage_api::errors::provider::ProviderResult<Option<alloy_primitives::B256>>;
                fn canonical_hashes_range(&self, start: alloy_primitives::BlockNumber, end: alloy_primitives::BlockNumber) -> rsil_storage_api::errors::provider::ProviderResult<Vec<alloy_primitives::B256>>;
            }
            StateProvider $(where [$($generics)*])? {
                fn storage(&self, account: alloy_primitives::Address, storage_key: alloy_primitives::StorageKey) -> rsil_storage_api::errors::provider::ProviderResult<Option<alloy_primitives::StorageValue>>;
            }
            BytecodeReader $(where [$($generics)*])? {
                fn bytecode_by_hash(&self, code_hash: &alloy_primitives::B256) -> rsil_storage_api::errors::provider::ProviderResult<Option<rsil_primitives_traits::Bytecode>>;
            }
            StateRootProvider $(where [$($generics)*])? {
                fn state_root(&self, state: rsil_trie::HashedPostState) -> rsil_storage_api::errors::provider::ProviderResult<alloy_primitives::B256>;
                fn state_root_from_nodes(&self, input: rsil_trie::TrieInput) -> rsil_storage_api::errors::provider::ProviderResult<alloy_primitives::B256>;
                fn state_root_with_updates(&self, state: rsil_trie::HashedPostState) -> rsil_storage_api::errors::provider::ProviderResult<(alloy_primitives::B256, rsil_trie::updates::TrieUpdates)>;
                fn state_root_from_nodes_with_updates(&self, input: rsil_trie::TrieInput) -> rsil_storage_api::errors::provider::ProviderResult<(alloy_primitives::B256, rsil_trie::updates::TrieUpdates)>;
            }
            StorageRootProvider $(where [$($generics)*])? {
                fn storage_root(&self, address: alloy_primitives::Address, storage: rsil_trie::HashedStorage) -> rsil_storage_api::errors::provider::ProviderResult<alloy_primitives::B256>;
                fn storage_proof(&self, address: alloy_primitives::Address, slot: alloy_primitives::B256, storage: rsil_trie::HashedStorage) -> rsil_storage_api::errors::provider::ProviderResult<rsil_trie::StorageProof>;
                fn storage_multiproof(&self, address: alloy_primitives::Address, slots: &[alloy_primitives::B256], storage: rsil_trie::HashedStorage) -> rsil_storage_api::errors::provider::ProviderResult<rsil_trie::StorageMultiProof>;
            }
            StateProofProvider $(where [$($generics)*])? {
                fn proof(&self, input: rsil_trie::TrieInput, address: alloy_primitives::Address, slots: &[alloy_primitives::B256]) -> rsil_storage_api::errors::provider::ProviderResult<rsil_trie::AccountProof>;
                fn multiproof(&self, input: rsil_trie::TrieInput, targets: rsil_trie::MultiProofTargets) -> rsil_storage_api::errors::provider::ProviderResult<rsil_trie::MultiProof>;
                fn witness(&self, input: rsil_trie::TrieInput, target: rsil_trie::HashedPostState, mode: rsil_trie::ExecutionWitnessMode) -> rsil_storage_api::errors::provider::ProviderResult<Vec<alloy_primitives::Bytes>>;
            }
            HashedPostStateProvider $(where [$($generics)*])? {
                fn hashed_post_state(&self, bundle_state: &revm::database::BundleState) -> rsil_trie::HashedPostState;
            }
        );
    }
}

pub use delegate_provider_impls;
