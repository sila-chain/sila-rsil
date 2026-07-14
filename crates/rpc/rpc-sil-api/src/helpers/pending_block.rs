//! Loads a pending block from database. Helper trait for `eth_` block, transaction, call and trace
//! RPC methods.

use super::SpawnBlocking;
use crate::{SilApiTypes, FromEthApiError, FromEvmError, RpcNodeCore};
use alloy_consensus::{BlockHeader, Transaction};
use alloy_eips::sip7840::BlobParams;
use alloy_primitives::{B256, U256};
use alloy_rpc_types_eth::{BlockNumberOrTag, BlockOverrides};
use futures::Future;
use rsil_chain_state::{BlockState, ExecutedBlock};
use rsil_chainspec::{ChainSpecProvider, SilChainSpec, SilaHardforks};
use rsil_errors::{BlockExecutionError, BlockValidationError, ProviderError, RsilError};
use rsil_evm::{
    block::TxResult,
    execute::{BlockBuilder, BlockBuilderOutcome, BlockExecutionOutput},
    ConfigureEvm, Savm, SavmEnvFor, NextBlockEnvAttributes,
};
use rsil_primitives_traits::{transaction::error::InvalidTransactionError, HeaderTy, SealedHeader};
use rsil_revm::{database::StateProviderDatabase, db::State};
use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_types::{
    block::BlockAndReceipts, builder::config::PendingBlockKind, SilApiError, PendingBlock,
    PendingBlockEnv, PendingBlockEnvOrigin,
};
use rsil_storage_api::{
    noop::NoopProvider, BlockReader, BlockReaderIdExt, ProviderHeader, ProviderTx,
    StateProviderBox, StateProviderFactory,
};
use rsil_transaction_pool::{
    error::InvalidPoolTransactionError, BestTransactions, BestTransactionsAttributes,
    PoolTransaction, TransactionPool,
};
use rsil_trie_common::ComputedTrieData;
use revm::context_interface::{Block, Cfg as _};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::debug;

/// Loads a pending block from database.
///
/// Behaviour shared by several `eth_` RPC methods, not exclusive to `eth_` blocks RPC methods.
pub trait LoadPendingBlock:
    SilApiTypes<
        Error: FromEvmError<Self::Savm>,
        RpcConvert: RpcConvert<Network = Self::NetworkTypes>,
    > + RpcNodeCore
{
    /// Returns a handle to the pending block.
    ///
    /// Data access in default (L1) trait method implementations.
    fn pending_block(&self) -> &Mutex<Option<PendingBlock<Self::Primitives>>>;

    /// Returns a [`PendingEnvBuilder`] for the pending block.
    fn pending_env_builder(&self) -> &dyn PendingEnvBuilder<Self::Savm>;

    /// Returns the pending block kind
    fn pending_block_kind(&self) -> PendingBlockKind;

    /// Configures the [`PendingBlockEnv`] for the pending block
    ///
    /// If no pending block is available, this will derive it from the `latest` block
    fn pending_block_env_and_cfg(&self) -> Result<PendingBlockEnv<Self::Savm>, Self::Error> {
        if let Some((block, receipts)) =
            self.provider().pending_block_and_receipts().map_err(Self::Error::from_eth_err)?
        {
            // Note: for the PENDING block we assume it is past the known merge block and
            // thus this will not fail when looking up the total
            // difficulty value for the blockenv.
            let evm_env = self
                .evm_config()
                .evm_env(block.header())
                .map_err(RsilError::other)
                .map_err(Self::Error::from_eth_err)?;

            return Ok(PendingBlockEnv::new(
                evm_env,
                PendingBlockEnvOrigin::ActualPending(Arc::new(block), Arc::new(receipts)),
            ));
        }

        // no pending block from the CL yet, so we use the latest block and modify the env
        // values that we can
        let latest = self
            .provider()
            .latest_header()
            .map_err(Self::Error::from_eth_err)?
            .ok_or(SilApiError::HeaderNotFound(BlockNumberOrTag::Latest.into()))?;

        let evm_env = self
            .evm_config()
            .next_evm_env(&latest, &self.next_env_attributes(&latest)?)
            .map_err(RsilError::other)
            .map_err(Self::Error::from_eth_err)?;

        Ok(PendingBlockEnv::new(evm_env, PendingBlockEnvOrigin::DerivedFromLatest(latest)))
    }

    /// Returns [`ConfigureEvm::NextBlockEnvCtx`] for building a local pending block.
    fn next_env_attributes(
        &self,
        parent: &SealedHeader<ProviderHeader<Self::Provider>>,
    ) -> Result<<Self::Savm as ConfigureEvm>::NextBlockEnvCtx, Self::Error> {
        Ok(self.pending_env_builder().pending_env_attributes(parent, None)?)
    }

    /// Returns a [`StateProviderBox`] on a mem-pool built pending block overlaying latest.
    fn local_pending_state(
        &self,
    ) -> impl Future<Output = Result<Option<StateProviderBox>, Self::Error>> + Send
    where
        Self: SpawnBlocking,
    {
        async move {
            let Some(pending_block) = self.pool_pending_block().await? else {
                return Ok(None);
            };

            let latest_historical = self
                .provider()
                .history_by_block_hash(pending_block.block().parent_hash())
                .map_err(Self::Error::from_eth_err)?;

            let state = BlockState::from(pending_block);

            Ok(Some(Box::new(state.state_provider(latest_historical)) as StateProviderBox))
        }
    }

    /// Returns a mem-pool built pending block.
    fn pool_pending_block(
        &self,
    ) -> impl Future<Output = Result<Option<PendingBlock<Self::Primitives>>, Self::Error>> + Send
    where
        Self: SpawnBlocking,
    {
        async move {
            if self.pending_block_kind().is_none() {
                return Ok(None);
            }
            let pending = self.pending_block_env_and_cfg()?;
            let parent = match pending.origin {
                PendingBlockEnvOrigin::ActualPending(..) => return Ok(None),
                PendingBlockEnvOrigin::DerivedFromLatest(parent) => parent,
            };

            self.build_pool_pending_block(parent, pending.evm_env).await
        }
    }

    /// Builds or returns a cached pending block from the transaction pool.
    ///
    /// This is the shared implementation used by both [`Self::pool_pending_block`] and
    /// [`Self::local_pending_block`] to avoid resolving the pending block environment twice.
    fn build_pool_pending_block(
        &self,
        parent: SealedHeader<ProviderHeader<Self::Provider>>,
        evm_env: SavmEnvFor<Self::Savm>,
    ) -> impl Future<Output = Result<Option<PendingBlock<Self::Primitives>>, Self::Error>> + Send
    where
        Self: SpawnBlocking,
    {
        async move {
            // we couldn't find the real pending block, so we need to build it ourselves
            let mut lock = self.pending_block().lock().await;

            let now = Instant::now();

            // Is the pending block cached?
            if let Some(pending_block) = lock.as_ref() {
                // Is the cached block not expired and latest is its parent?
                if evm_env.block_env.number() == U256::from(pending_block.block().number()) &&
                    parent.hash() == pending_block.block().parent_hash() &&
                    now <= pending_block.expires_at
                {
                    return Ok(Some(pending_block.clone()));
                }
            }

            let executed_block = match self
                .spawn_blocking_io(move |this| {
                    // we rebuild the block
                    this.build_block(&parent)
                })
                .await
            {
                Ok(block) => block,
                Err(err) => {
                    debug!(target: "rpc", "Failed to build pending block: {:?}", err);
                    return Ok(None)
                }
            };

            let pending = PendingBlock::with_executed_block(
                Instant::now() + Duration::from_secs(1),
                executed_block,
            );

            *lock = Some(pending.clone());

            Ok(Some(pending))
        }
    }

    /// Returns the locally built pending block
    fn local_pending_block(
        &self,
    ) -> impl Future<Output = Result<Option<BlockAndReceipts<Self::Primitives>>, Self::Error>> + Send
    where
        Self: SpawnBlocking,
        Self::Pool:
            TransactionPool<Transaction: PoolTransaction<Consensus = ProviderTx<Self::Provider>>>,
    {
        async move {
            if self.pending_block_kind().is_none() {
                return Ok(None);
            }

            let pending = self.pending_block_env_and_cfg()?;

            Ok(match pending.origin {
                PendingBlockEnvOrigin::ActualPending(block, receipts) => {
                    Some(BlockAndReceipts { block, receipts })
                }
                PendingBlockEnvOrigin::DerivedFromLatest(parent) => self
                    .build_pool_pending_block(parent, pending.evm_env)
                    .await?
                    .map(PendingBlock::into_block_and_receipts),
            })
        }
    }

    /// Builds a locally derived pending block using the configured provider and pool.
    ///
    /// This is used when no execution-layer pending block is available and a pending block is
    /// derived from the latest canonical header, using the provided parent.
    ///
    /// Withdrawals and any fork-specific behavior (such as SIP-4788 pre-block contract calls) are
    /// determined by the SAVM environment and chain specification used during construction.
    fn build_block(
        &self,
        parent: &SealedHeader<ProviderHeader<Self::Provider>>,
    ) -> Result<ExecutedBlock<Self::Primitives>, Self::Error>
    where
        Self::Pool:
            TransactionPool<Transaction: PoolTransaction<Consensus = ProviderTx<Self::Provider>>>,
        SilApiError: From<ProviderError>,
    {
        let state_provider = self
            .provider()
            .history_by_block_hash(parent.hash())
            .map_err(Self::Error::from_eth_err)?;
        let state = StateProviderDatabase::new(state_provider);
        let mut db = State::builder().with_database(state).with_bundle_update().build();

        let mut builder = self
            .evm_config()
            .builder_for_next_block(&mut db, parent, self.next_env_attributes(parent)?)
            .map_err(RsilError::other)
            .map_err(Self::Error::from_eth_err)?;

        builder.apply_pre_execution_changes().map_err(Self::Error::from_eth_err)?;

        let block_gas_limit: u64 = builder.savm().block().gas_limit();
        let is_amsterdam = self
            .provider()
            .chain_spec()
            .is_amsterdam_active_at_timestamp(builder.savm().block().timestamp().saturating_to());
        let basefee = builder.savm().block().basefee();
        let blob_gasprice = builder.savm().block().blob_gasprice().map(|p| p as u64);

        let blob_params = self
            .provider()
            .chain_spec()
            .blob_params_at_timestamp(parent.timestamp())
            .unwrap_or_else(BlobParams::cancun);
        let mut cumulative_tx_gas_used = 0;
        let mut block_regular_gas_used = 0;
        let mut block_state_gas_used = 0;
        let mut sum_blob_gas_used = 0;
        let tx_gas_limit_cap = builder.savm().cfg_env().tx_gas_limit_cap();

        // Only include transactions if not configured as Empty
        if !self.pending_block_kind().is_empty() {
            let mut best_txs = self
                .pool()
                .best_transactions_with_attributes(BestTransactionsAttributes::new(
                    basefee,
                    blob_gasprice,
                ))
                // freeze to get a block as fast as possible
                .without_updates();

            while let Some(pool_tx) = best_txs.next() {
                // ensure we still have capacity for this transaction
                let exceeds_gas_limit = if is_amsterdam {
                    let regular_available_gas =
                        block_gas_limit.saturating_sub(block_regular_gas_used);
                    let state_available_gas = block_gas_limit.saturating_sub(block_state_gas_used);
                    let regular_tx_gas_limit = pool_tx.gas_limit().min(tx_gas_limit_cap);

                    if regular_tx_gas_limit > regular_available_gas {
                        Some((regular_tx_gas_limit, regular_available_gas))
                    } else if pool_tx.gas_limit() > state_available_gas {
                        Some((pool_tx.gas_limit(), state_available_gas))
                    } else {
                        None
                    }
                } else {
                    let block_available_gas =
                        block_gas_limit.saturating_sub(cumulative_tx_gas_used);
                    (pool_tx.gas_limit() > block_available_gas)
                        .then_some((pool_tx.gas_limit(), block_available_gas))
                };

                if let Some((transaction_gas_limit, block_available_gas)) = exceeds_gas_limit {
                    // we can't fit this transaction into the block, so we need to mark it as
                    // invalid which also removes all dependent transaction from
                    // the iterator before we can continue
                    best_txs.mark_invalid(
                        &pool_tx,
                        InvalidPoolTransactionError::ExceedsGasLimit(
                            transaction_gas_limit,
                            block_available_gas,
                        ),
                    );
                    continue
                }

                if pool_tx.origin.is_private() {
                    // we don't want to leak any state changes made by private transactions, so we
                    // mark them as invalid here which removes all dependent
                    // transactions from the iteratorbefore we can continue
                    best_txs.mark_invalid(
                        &pool_tx,
                        InvalidPoolTransactionError::Consensus(
                            InvalidTransactionError::TxTypeNotSupported,
                        ),
                    );
                    continue
                }

                // convert tx to a signed transaction
                let tx = pool_tx.to_consensus();

                // There's only limited amount of blob space available per block, so we need to
                // check if the SIP-4844 can still fit in the block
                let tx_blob_gas = tx.blob_gas_used();
                if let Some(tx_blob_gas) = tx_blob_gas &&
                    sum_blob_gas_used + tx_blob_gas > blob_params.max_blob_gas_per_block()
                {
                    // we can't fit this _blob_ transaction into the block, so we mark it as
                    // invalid, which removes its dependent transactions from
                    // the iterator. This is similar to the gas limit condition
                    // for regular transactions above.
                    best_txs.mark_invalid(
                        &pool_tx,
                        InvalidPoolTransactionError::ExceedsGasLimit(
                            tx_blob_gas,
                            blob_params.max_blob_gas_per_block(),
                        ),
                    );
                    continue
                }

                let mut tx_regular_gas_used = 0;
                let gas_output =
                    match builder.execute_transaction_with_result_closure(tx, |result| {
                        tx_regular_gas_used = result.result().result.gas().block_regular_gas_used();
                    }) {
                        Ok(gas_output) => gas_output,
                        Err(BlockExecutionError::Validation(BlockValidationError::InvalidTx {
                            error,
                            ..
                        })) => {
                            if error.is_nonce_too_low() {
                                // if the nonce is too low, we can skip this transaction
                            } else {
                                // if the transaction is invalid, we can skip it and all of its
                                // descendants
                                best_txs.mark_invalid(
                                    &pool_tx,
                                    InvalidPoolTransactionError::Consensus(
                                        InvalidTransactionError::TxTypeNotSupported,
                                    ),
                                );
                            }
                            continue
                        }
                        Err(BlockExecutionError::Validation(
                            BlockValidationError::TransactionGasLimitMoreThanAvailableBlockGas {
                                transaction_gas_limit,
                                block_available_gas,
                            },
                        )) => {
                            best_txs.mark_invalid(
                                &pool_tx,
                                InvalidPoolTransactionError::ExceedsGasLimit(
                                    transaction_gas_limit,
                                    block_available_gas,
                                ),
                            );
                            continue
                        }
                        // this is an error that we should treat as fatal for this attempt
                        Err(err) => return Err(Self::Error::from_eth_err(err)),
                    };

                // add to the total blob gas used if the transaction successfully executed
                if let Some(tx_blob_gas) = tx_blob_gas {
                    sum_blob_gas_used += tx_blob_gas;

                    // if we've reached the max data gas per block, we can skip blob txs entirely
                    if sum_blob_gas_used == blob_params.max_blob_gas_per_block() {
                        best_txs.skip_blobs();
                    }
                }

                // Track receipt gas and the SilaAmsterdam block-capacity counter separately.
                let gas_used = gas_output.tx_gas_used();
                cumulative_tx_gas_used += gas_used;
                block_regular_gas_used += tx_regular_gas_used;
                block_state_gas_used += gas_output.state_gas_used();
            }
        }

        let BlockBuilderOutcome { execution_result, block, hashed_state, trie_updates, .. } =
            builder.finish(NoopProvider::default(), None).map_err(Self::Error::from_eth_err)?;

        let execution_outcome =
            BlockExecutionOutput { state: db.take_bundle(), result: execution_result };

        Ok(ExecutedBlock::new(
            block.into(),
            Arc::new(execution_outcome),
            ComputedTrieData::new(
                Arc::new(hashed_state.into_sorted()),
                Arc::new(trie_updates.into_sorted()),
            ),
        ))
    }
}

/// A type that knows how to build a [`ConfigureEvm::NextBlockEnvCtx`] for a pending block.
pub trait PendingEnvBuilder<Savm: ConfigureEvm>: Send + Sync + Unpin + 'static {
    /// Builds a [`ConfigureEvm::NextBlockEnvCtx`] for a pending block.
    ///
    /// `block_overrides` can be used for values that need to be part of the next block context
    /// before the SAVM environment is constructed. Other block overrides are applied directly to the
    /// SAVM environment after construction.
    fn pending_env_attributes(
        &self,
        parent: &SealedHeader<HeaderTy<Savm::Primitives>>,
        block_overrides: Option<&BlockOverrides>,
    ) -> Result<Savm::NextBlockEnvCtx, SilApiError>;
}

/// Trait that should be implemented on [`ConfigureEvm::NextBlockEnvCtx`] to provide a way for it to
/// build an environment for pending block.
///
/// This assumes that next environment building doesn't require any additional context, for more
/// complex implementations one should implement [`PendingEnvBuilder`] on their custom type.
pub trait BuildPendingEnv<Header> {
    /// Builds a [`ConfigureEvm::NextBlockEnvCtx`] for a pending block.
    ///
    /// `block_overrides` can be used for values that need to be part of the next block context
    /// before the SAVM environment is constructed. Other block overrides are applied directly to the
    /// SAVM environment after construction.
    fn build_pending_env(
        parent: &SealedHeader<Header>,
        block_overrides: Option<&BlockOverrides>,
    ) -> Self;
}

impl<Savm> PendingEnvBuilder<Savm> for ()
where
    Savm: ConfigureEvm<NextBlockEnvCtx: BuildPendingEnv<HeaderTy<Savm::Primitives>>>,
{
    fn pending_env_attributes(
        &self,
        parent: &SealedHeader<HeaderTy<Savm::Primitives>>,
        block_overrides: Option<&BlockOverrides>,
    ) -> Result<Savm::NextBlockEnvCtx, SilApiError> {
        Ok(Savm::NextBlockEnvCtx::build_pending_env(parent, block_overrides))
    }
}

impl<H: BlockHeader> BuildPendingEnv<H> for NextBlockEnvAttributes {
    fn build_pending_env(
        parent: &SealedHeader<H>,
        block_overrides: Option<&BlockOverrides>,
    ) -> Self {
        let mut attributes = Self {
            timestamp: parent.timestamp().saturating_add(12),
            suggested_fee_recipient: parent.beneficiary(),
            prev_randao: B256::random(),
            gas_limit: parent.gas_limit(),
            parent_beacon_block_root: parent.parent_beacon_block_root().map(|_| B256::ZERO),
            withdrawals: parent.withdrawals_root().map(|_| Default::default()),
            extra_data: parent.extra_data().clone(),
            slot_number: parent.slot_number().map(|slot| slot.saturating_add(1)),
        };

        if attributes.parent_beacon_block_root.is_some() &&
            let Some(beacon_root) = block_overrides.and_then(|overrides| overrides.beacon_root)
        {
            attributes.parent_beacon_block_root = Some(beacon_root);
        }

        attributes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::Header;
    use alloy_primitives::B256;
    use rsil_primitives_traits::SealedHeader;

    #[test]
    fn pending_env_defaults_parent_beacon_root() {
        let mut header = Header::default();
        let beacon_root = B256::repeat_byte(0x42);
        header.parent_beacon_block_root = Some(beacon_root);
        let sealed = SealedHeader::new(header, B256::ZERO);

        let attrs = NextBlockEnvAttributes::build_pending_env(&sealed, None);

        assert_eq!(attrs.parent_beacon_block_root, Some(B256::ZERO));
    }

    #[test]
    fn pending_env_applies_parent_beacon_root_override() {
        let header = Header { parent_beacon_block_root: Some(B256::ZERO), ..Default::default() };
        let sealed = SealedHeader::new(header, B256::ZERO);
        let beacon_root = B256::repeat_byte(0x42);
        let block_overrides =
            BlockOverrides { beacon_root: Some(beacon_root), ..Default::default() };

        let attrs = NextBlockEnvAttributes::build_pending_env(&sealed, Some(&block_overrides));

        assert_eq!(attrs.parent_beacon_block_root, Some(beacon_root));
    }

    #[test]
    fn pending_env_ignores_parent_beacon_root_override_before_fork() {
        let sealed = SealedHeader::new(Header::default(), B256::ZERO);
        let beacon_root = B256::repeat_byte(0x42);
        let block_overrides =
            BlockOverrides { beacon_root: Some(beacon_root), ..Default::default() };

        let attrs = NextBlockEnvAttributes::build_pending_env(&sealed, Some(&block_overrides));

        assert_eq!(attrs.parent_beacon_block_root, None);
    }

    #[test]
    fn pending_env_increments_parent_slot_number() {
        let header = Header { slot_number: Some(7), ..Default::default() };
        let sealed = SealedHeader::new(header, B256::ZERO);

        let attrs = NextBlockEnvAttributes::build_pending_env(&sealed, None);

        assert_eq!(attrs.slot_number, Some(8));
    }
}
