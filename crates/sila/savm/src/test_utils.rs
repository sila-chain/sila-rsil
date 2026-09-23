use crate::SilEvmConfig;
use rsil_savm::noop::NoopEvmConfig;

/// A helper type alias for mocked block executor provider.
pub type MockExecutorProvider = MockEvmConfig;

/// Mock for SAVM config.
pub type MockEvmConfig = NoopEvmConfig<SilEvmConfig>;
