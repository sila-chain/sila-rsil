//! Example for how to modify a block post-execution step. It credits beacon withdrawals with a
//! custom mechanism instead of minting native tokens

#![warn(unused_crate_dependencies)]

use alloy_eips::sip4895::Withdrawal;
use alloy_evm::{
    block::{BlockExecutorFactory, ExecutableTx, GasOutput},
    precompiles::PrecompilesMap,
    revm::context::Block as _,
    sil::{SilBlockExecutionCtx, SilBlockExecutor, SilTxResult},
    SavmFactory, SilEvm, SilEvmFactory,
};
use alloy_sol_types::{sol, SolCall};
use rsil_sila::{
    chainspec::ChainSpec,
    cli::interface::Cli,
    node::{
        api::{ConfigureEngineEvm, ConfigureEvm, ExecutableTxIterator, FullNodeTypes, NodeTypes},
        builder::{components::ExecutorBuilder, BuilderContext},
        node::SilaAddOns,
        SilaNode,
    },
    primitives::{Header, SealedBlock, SealedHeader},
    provider::BlockExecutionResult,
    rpc::types::engine::ExecutionData,
    savm::{
        primitives::{
            block::StateDB,
            execute::{BlockExecutionError, BlockExecutor, InternalBlockExecutionError},
            ExecutionCtxFor, InspectorFor, NextBlockEnvAttributes, Savm, SavmEnv, SavmEnvFor,
        },
        revm::{
            context::TxEnv,
            primitives::{address, hardfork::SpecId, Address},
            DatabaseCommit,
        },
        RsilReceiptBuilder, SilBlockAssembler, SilEvmConfig,
    },
    Block, Receipt, SilPrimitives, TransactionSigned, TxType,
};
use std::{fmt::Display, sync::Arc};

pub const SYSTEM_ADDRESS: Address = address!("0xfffffffffffffffffffffffffffffffffffffffe");
pub const WITHDRAWALS_ADDRESS: Address = address!("0x4200000000000000000000000000000000000000");

fn main() {
    Cli::parse_args()
        .run(async move |builder, _| {
            let handle = builder
                // use the default sila node types
                .with_types::<SilaNode>()
                // Configure the components of the node
                // use default sila components but use our custom pool
                .with_components(SilaNode::components().executor(CustomExecutorBuilder::default()))
                .with_add_ons(SilaAddOns::default())
                .launch()
                .await?;

            handle.wait_for_node_exit().await
        })
        .unwrap();
}

/// A custom executor builder
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct CustomExecutorBuilder;

impl<Types, Node> ExecutorBuilder<Node> for CustomExecutorBuilder
where
    Types: NodeTypes<ChainSpec = ChainSpec, Primitives = SilPrimitives>,
    Node: FullNodeTypes<Types = Types>,
{
    type SAVM = CustomEvmConfig;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::SAVM> {
        let evm_config = CustomEvmConfig { inner: SilEvmConfig::new(ctx.chain_spec()) };

        Ok(evm_config)
    }
}

#[derive(Debug, Clone)]
pub struct CustomEvmConfig {
    inner: SilEvmConfig,
}

impl BlockExecutorFactory for CustomEvmConfig {
    type SavmFactory = SilEvmFactory;
    type ExecutionCtx<'a> = SilBlockExecutionCtx<'a>;
    type Transaction = TransactionSigned;
    type Receipt = Receipt;
    type TxExecutionResult = SilTxResult<<SilEvmFactory as SavmFactory>::HaltReason, TxType>;
    type Executor<'a, DB: StateDB, I: InspectorFor<Self, DB>> =
        CustomBlockExecutor<'a, SilEvm<DB, I, PrecompilesMap>>;

    fn evm_factory(&self) -> &Self::SavmFactory {
        self.inner.evm_factory()
    }

    fn create_executor<'a, DB, I>(
        &'a self,
        savm: SilEvm<DB, I, PrecompilesMap>,
        ctx: SilBlockExecutionCtx<'a>,
    ) -> Self::Executor<'a, DB, I>
    where
        DB: StateDB,
        I: InspectorFor<Self, DB>,
    {
        CustomBlockExecutor {
            inner: SilBlockExecutor::new(
                savm,
                ctx,
                self.inner.chain_spec(),
                self.inner.executor_factory.receipt_builder(),
            ),
        }
    }
}

impl ConfigureEvm for CustomEvmConfig {
    type Primitives = <SilEvmConfig as ConfigureEvm>::Primitives;
    type Error = <SilEvmConfig as ConfigureEvm>::Error;
    type NextBlockEnvCtx = <SilEvmConfig as ConfigureEvm>::NextBlockEnvCtx;
    type BlockExecutorFactory = Self;
    type BlockAssembler = SilBlockAssembler<ChainSpec>;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        self
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        self.inner.block_assembler()
    }

    fn evm_env(&self, header: &Header) -> Result<SavmEnv<SpecId>, Self::Error> {
        self.inner.evm_env(header)
    }

    fn next_evm_env(
        &self,
        parent: &Header,
        attributes: &NextBlockEnvAttributes,
    ) -> Result<SavmEnv<SpecId>, Self::Error> {
        self.inner.next_evm_env(parent, attributes)
    }

    fn context_for_block<'a>(
        &self,
        block: &'a SealedBlock<Block>,
    ) -> Result<SilBlockExecutionCtx<'a>, Self::Error> {
        self.inner.context_for_block(block)
    }

    fn context_for_next_block(
        &self,
        parent: &SealedHeader,
        attributes: Self::NextBlockEnvCtx,
    ) -> Result<SilBlockExecutionCtx<'_>, Self::Error> {
        self.inner.context_for_next_block(parent, attributes)
    }
}

impl ConfigureEngineEvm<ExecutionData> for CustomEvmConfig {
    fn evm_env_for_payload(
        &self,
        payload: &ExecutionData,
    ) -> Result<SavmEnvFor<Self>, Self::Error> {
        self.inner.evm_env_for_payload(payload)
    }

    fn context_for_payload<'a>(
        &self,
        payload: &'a ExecutionData,
    ) -> Result<ExecutionCtxFor<'a, Self>, Self::Error> {
        self.inner.context_for_payload(payload)
    }

    fn tx_iterator_for_payload(
        &self,
        payload: &ExecutionData,
    ) -> Result<impl ExecutableTxIterator<Self>, Self::Error> {
        self.inner.tx_iterator_for_payload(payload)
    }
}

pub struct CustomBlockExecutor<'a, Savm> {
    /// Inner Sila execution strategy.
    inner: SilBlockExecutor<'a, Savm, &'a Arc<ChainSpec>, &'a RsilReceiptBuilder>,
}

impl<E> BlockExecutor for CustomBlockExecutor<'_, E>
where
    E: Savm<DB: StateDB, Tx = TxEnv>,
{
    type Transaction = TransactionSigned;
    type Receipt = Receipt;
    type Savm = E;
    type Result = SilTxResult<E::HaltReason, TxType>;

    fn apply_pre_execution_changes(&mut self) -> Result<(), BlockExecutionError> {
        self.inner.apply_pre_execution_changes()
    }

    fn receipts(&self) -> &[Self::Receipt] {
        self.inner.receipts()
    }

    fn execute_transaction_without_commit(
        &mut self,
        tx: impl ExecutableTx<Self>,
    ) -> Result<Self::Result, BlockExecutionError> {
        self.inner.execute_transaction_without_commit(tx)
    }

    fn commit_transaction(&mut self, output: Self::Result) -> GasOutput {
        self.inner.commit_transaction(output)
    }

    fn finish(
        mut self,
    ) -> Result<(Self::Savm, BlockExecutionResult<Receipt>), BlockExecutionError> {
        if let Some(withdrawals) = self.inner.ctx.withdrawals.clone() {
            apply_withdrawals_contract_call(withdrawals.as_ref(), self.inner.evm_mut())?;
        }

        // Invoke inner finish method to apply Sila post-execution changes
        self.inner.finish()
    }

    fn evm_mut(&mut self) -> &mut Self::Savm {
        self.inner.evm_mut()
    }

    fn savm(&self) -> &Self::Savm {
        self.inner.savm()
    }
}

sol!(
    function withdrawals(
        uint64[] calldata amounts,
        address[] calldata addresses
    );
);

/// Applies the post-block call to the withdrawal / deposit contract, using the given block,
/// [`ChainSpec`], SAVM.
pub fn apply_withdrawals_contract_call(
    withdrawals: &[Withdrawal],
    savm: &mut impl Savm<Error: Display, DB: DatabaseCommit>,
) -> Result<(), BlockExecutionError> {
    let mut state = match savm.transact_system_call(
        SYSTEM_ADDRESS,
        WITHDRAWALS_ADDRESS,
        withdrawalsCall {
            amounts: withdrawals.iter().map(|w| w.amount).collect::<Vec<_>>(),
            addresses: withdrawals.iter().map(|w| w.address).collect::<Vec<_>>(),
        }
        .abi_encode()
        .into(),
    ) {
        Ok(res) => res.state,
        Err(e) => {
            return Err(BlockExecutionError::Internal(InternalBlockExecutionError::Other(
                format!("withdrawal contract system call revert: {e}").into(),
            )))
        }
    };

    // Clean-up post system tx context
    state.remove(&SYSTEM_ADDRESS);
    state.remove(&savm.block().beneficiary());

    savm.db_mut().commit(state);

    Ok(())
}
