use core::fmt;
use std::collections::BTreeMap;

use alloy_consensus::Transaction;
use alloy_primitives::Address;
use alloy_rpc_types_txpool::{
    TxpoolContent, TxpoolContentFrom, TxpoolInspect, TxpoolInspectSummary, TxpoolStatus,
};
use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use rsil_primitives_traits::NodePrimitives;
use rsil_rpc_api::TxPoolApiServer;
use rsil_rpc_convert::{RpcConvert, RpcTypes};
use rsil_rpc_eth_api::RpcTransaction;
use rsil_transaction_pool::{
    AllPoolTransactions, PoolConsensusTx, PoolTransaction, TransactionPool,
};
use tracing::trace;

/// `txpool` API implementation.
///
/// This type provides the functionality for handling `txpool` related requests.
#[derive(Clone)]
pub struct TxPoolApi<Pool, Sil> {
    /// An interface to interact with the pool
    pool: Pool,
    converter: Sil,
}

impl<Pool, Sil> TxPoolApi<Pool, Sil> {
    /// Creates a new instance of `TxpoolApi`.
    pub const fn new(pool: Pool, converter: Sil) -> Self {
        Self { pool, converter }
    }
}

impl<Pool, Sil> TxPoolApi<Pool, Sil>
where
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus: Transaction>> + 'static,
    Sil: RpcConvert<Primitives: NodePrimitives<SignedTx = PoolConsensusTx<Pool>>>,
{
    fn content(&self) -> Result<TxpoolContent<RpcTransaction<Sil::Network>>, Sil::Error> {
        #[inline]
        fn insert<Tx, RpcTxB>(
            tx: &Tx,
            content: &mut BTreeMap<
                Address,
                BTreeMap<String, <RpcTxB::Network as RpcTypes>::TransactionResponse>,
            >,
            resp_builder: &RpcTxB,
        ) -> Result<(), RpcTxB::Error>
        where
            Tx: PoolTransaction,
            RpcTxB: RpcConvert<Primitives: NodePrimitives<SignedTx = Tx::Consensus>>,
        {
            content.entry(tx.sender()).or_default().insert(
                tx.nonce().to_string(),
                resp_builder.fill_pending(tx.clone_into_consensus())?,
            );

            Ok(())
        }

        let AllPoolTransactions { pending, queued } = self.pool.all_transactions();

        let mut content = TxpoolContent::default();
        for pending in pending {
            insert::<_, Sil>(&pending.transaction, &mut content.pending, &self.converter)?;
        }
        for queued in queued {
            insert::<_, Sil>(&queued.transaction, &mut content.queued, &self.converter)?;
        }

        Ok(content)
    }
}

#[async_trait]
impl<Pool, Sil> TxPoolApiServer<RpcTransaction<Sil::Network>> for TxPoolApi<Pool, Sil>
where
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus: Transaction>> + 'static,
    Sil: RpcConvert<Primitives: NodePrimitives<SignedTx = PoolConsensusTx<Pool>>> + 'static,
{
    /// Returns the number of transactions currently pending for inclusion in the next block(s), as
    /// well as the ones that are being scheduled for future execution only.
    /// Ref: [Here](https://geth.sila.org/docs/rpc/ns-txpool#txpool_status)
    ///
    /// Handler for `txpool_status`
    async fn txpool_status(&self) -> RpcResult<TxpoolStatus> {
        trace!(target: "rpc::sil", "Serving txpool_status");
        let (pending, queued) = self.pool.pending_and_queued_txn_count();
        Ok(TxpoolStatus { pending: pending as u64, queued: queued as u64 })
    }

    /// Returns a summary of all the transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    ///
    /// See [here](https://geth.sila.org/docs/rpc/ns-txpool#txpool_inspect) for more details
    ///
    /// Handler for `txpool_inspect`
    async fn txpool_inspect(&self) -> RpcResult<TxpoolInspect> {
        trace!(target: "rpc::sil", "Serving txpool_inspect");

        #[inline]
        fn insert<T: PoolTransaction<Consensus: Transaction>>(
            tx: &T,
            inspect: &mut BTreeMap<Address, BTreeMap<String, TxpoolInspectSummary>>,
        ) {
            let entry = inspect.entry(tx.sender()).or_default();
            let tx = tx.clone_into_consensus();
            entry.insert(tx.nonce().to_string(), tx.into_inner().into());
        }

        let AllPoolTransactions { pending, queued } = self.pool.all_transactions();

        Ok(TxpoolInspect {
            pending: pending.iter().fold(Default::default(), |mut acc, tx| {
                insert(&tx.transaction, &mut acc);
                acc
            }),
            queued: queued.iter().fold(Default::default(), |mut acc, tx| {
                insert(&tx.transaction, &mut acc);
                acc
            }),
        })
    }

    /// Retrieves the transactions contained within the txpool, returning pending as well as queued
    /// transactions of this address, grouped by nonce.
    ///
    /// See [here](https://geth.sila.org/docs/rpc/ns-txpool#txpool_contentFrom) for more details
    /// Handler for `txpool_contentFrom`
    async fn txpool_content_from(
        &self,
        from: Address,
    ) -> RpcResult<TxpoolContentFrom<RpcTransaction<Sil::Network>>> {
        trace!(target: "rpc::sil", ?from, "Serving txpool_contentFrom");
        Ok(self.content().map_err(Into::into)?.remove_from(&from))
    }

    /// Returns the details of all transactions currently pending for inclusion in the next
    /// block(s), as well as the ones that are being scheduled for future execution only.
    ///
    /// See [here](https://geth.sila.org/docs/rpc/ns-txpool#txpool_content) for more details
    /// Handler for `txpool_content`
    async fn txpool_content(&self) -> RpcResult<TxpoolContent<RpcTransaction<Sil::Network>>> {
        trace!(target: "rpc::sil", "Serving txpool_content");
        Ok(self.content().map_err(Into::into)?)
    }
}

impl<Pool, Sil> fmt::Debug for TxPoolApi<Pool, Sil> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TxpoolApi").finish_non_exhaustive()
    }
}
