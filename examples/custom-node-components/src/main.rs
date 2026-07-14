//! This example shows how to configure custom components for a rsil node.

#![warn(unused_crate_dependencies)]

use rsil_sila::{
    chainspec::ChainSpec,
    cli::interface::Cli,
    savm::primitives::ConfigureEvm,
    node::{
        api::{FullNodeTypes, NodeTypes},
        builder::{components::PoolBuilder, BuilderContext},
        node::SilaAddOns,
        SilaNode,
    },
    pool::{
        blobstore::InMemoryBlobStore, CoinbaseTipOrdering, SilTransactionPool, Pool, PoolConfig,
        TransactionValidationTaskExecutor,
    },
    provider::CanonStateSubscriptions,
    SilPrimitives,
};
use rsil_tracing::tracing::{debug, info};

fn main() {
    Cli::parse_args()
        .run(async move |builder, _| {
            let handle = builder
                // use the default sila node types
                .with_types::<SilaNode>()
                // Configure the components of the node
                // use default sila components but use our custom pool
                .with_components(SilaNode::components().pool(CustomPoolBuilder::default()))
                .with_add_ons(SilaAddOns::default())
                .launch()
                .await?;

            handle.wait_for_node_exit().await
        })
        .unwrap();
}

/// A custom pool builder
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CustomPoolBuilder {
    /// Use custom pool config
    pool_config: PoolConfig,
}

/// Implement the [`PoolBuilder`] trait for the custom pool builder
///
/// This will be used to build the transaction pool and its maintenance tasks during launch.
impl<Node, Savm> PoolBuilder<Node, Savm> for CustomPoolBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = SilPrimitives>>,
    Savm: ConfigureEvm<Primitives = SilPrimitives> + Clone + 'static,
{
    type Pool = SilTransactionPool<Node::Provider, InMemoryBlobStore, Savm>;

    async fn build_pool(
        self,
        ctx: &BuilderContext<Node>,
        evm_config: Savm,
    ) -> eyre::Result<Self::Pool> {
        let data_dir = ctx.config().datadir();
        let blob_store = InMemoryBlobStore::default();
        let validator =
            TransactionValidationTaskExecutor::eth_builder(ctx.provider().clone(), evm_config)
                .kzg_settings(ctx.kzg_settings()?)
                .with_additional_tasks(ctx.config().txpool.additional_validation_tasks)
                .build_with_tasks(ctx.task_executor().clone(), blob_store.clone());

        let transaction_pool =
            Pool::new(validator, CoinbaseTipOrdering::default(), blob_store, self.pool_config);
        info!(target: "rsil::cli", "Transaction pool initialized");
        let transactions_path = data_dir.txpool_transactions();

        // spawn txpool maintenance task
        {
            let pool = transaction_pool.clone();
            let chain_events = ctx.provider().canonical_state_stream();
            let client = ctx.provider().clone();
            let transactions_backup_config =
                rsil_sila::pool::maintain::LocalTransactionBackupConfig::with_local_txs_backup(
                    transactions_path,
                );

            ctx.task_executor().spawn_critical_with_graceful_shutdown_signal(
                "local transactions backup task",
                |shutdown| {
                    rsil_sila::pool::maintain::backup_local_transactions_task(
                        shutdown,
                        pool.clone(),
                        transactions_backup_config,
                    )
                },
            );

            // spawn the maintenance task
            ctx.task_executor().spawn_critical_task(
                "txpool maintenance task",
                rsil_sila::pool::maintain::maintain_transaction_pool_future(
                    client,
                    pool,
                    chain_events,
                    ctx.task_executor().clone(),
                    rsil_sila::pool::maintain::MaintainPoolConfig {
                        max_tx_lifetime: transaction_pool.config().max_queued_lifetime,
                        ..Default::default()
                    },
                ),
            );
            debug!(target: "rsil::cli", "Spawned txpool maintenance task");
        }

        Ok(transaction_pool)
    }
}
