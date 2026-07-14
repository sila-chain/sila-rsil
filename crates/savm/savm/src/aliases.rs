//! Helper aliases when working with [`ConfigureEvm`] and the traits in this crate.

use crate::ConfigureEvm;
use alloy_evm::{
    block::{BlockExecutorFactory, BlockExecutorFor},
    Database, SavmEnv, SavmFactory,
};
use revm::{database::State, inspector::NoOpInspector, Inspector};

/// Helper to access [`SavmFactory`] for a given [`ConfigureEvm`].
pub type SavmFactoryFor<Savm> =
    <<Savm as ConfigureEvm>::BlockExecutorFactory as BlockExecutorFactory>::SavmFactory;

/// Helper to access [`SavmFactory::Spec`] for a given [`ConfigureEvm`].
pub type SpecFor<Savm> = <SavmFactoryFor<Savm> as SavmFactory>::Spec;

/// Helper to access [`SavmFactory::BlockEnv`] for a given [`ConfigureEvm`].
pub type BlockEnvFor<Savm> = <SavmFactoryFor<Savm> as SavmFactory>::BlockEnv;

/// Helper to access [`SavmFactory::Savm`] for a given [`ConfigureEvm`].
pub type SavmFor<Savm, DB, I = NoOpInspector> = <SavmFactoryFor<Savm> as SavmFactory>::Savm<DB, I>;

/// Helper to access [`SavmFactory::Error`] for a given [`ConfigureEvm`].
pub type SavmErrorFor<Savm, DB> = <SavmFactoryFor<Savm> as SavmFactory>::Error<DB>;

/// Helper to access [`SavmFactory::Context`] for a given [`ConfigureEvm`].
pub type SavmContextFor<Savm, DB> = <SavmFactoryFor<Savm> as SavmFactory>::Context<DB>;

/// Helper to access [`SavmFactory::HaltReason`] for a given [`ConfigureEvm`].
pub type HaltReasonFor<Savm> = <SavmFactoryFor<Savm> as SavmFactory>::HaltReason;

/// Helper to access [`SavmFactory::Tx`] for a given [`ConfigureEvm`].
pub type TxEnvFor<Savm> = <SavmFactoryFor<Savm> as SavmFactory>::Tx;

/// Helper to access [`BlockExecutorFactory::ExecutionCtx`] for a given [`ConfigureEvm`].
pub type ExecutionCtxFor<'a, Savm> =
    <<Savm as ConfigureEvm>::BlockExecutorFactory as BlockExecutorFactory>::ExecutionCtx<'a>;

/// Helper to access [`alloy_evm::block::BlockExecutor`] for a given [`ConfigureEvm`].
pub type BlockExecutorForEvm<'a, Savm, DB, I = NoOpInspector> =
    BlockExecutorFor<'a, <Savm as ConfigureEvm>::BlockExecutorFactory, &'a mut State<DB>, I>;

/// Type alias for [`SavmEnv`] for a given [`ConfigureEvm`].
pub type SavmEnvFor<Savm> = SavmEnv<SpecFor<Savm>, BlockEnvFor<Savm>>;

/// Helper trait to bound [`Inspector`] for a [`ConfigureEvm`].
pub trait InspectorFor<Savm: ConfigureEvm, DB: Database>: Inspector<SavmContextFor<Savm, DB>> {}
impl<T, Savm, DB> InspectorFor<Savm, DB> for T
where
    Savm: ConfigureEvm,
    DB: Database,
    T: Inspector<SavmContextFor<Savm, DB>>,
{
}
