//! Validates execution payload wrt Sila Execution Engine API version.

use alloy_rpc_types_engine::ExecutionData;
pub use alloy_rpc_types_engine::{
    ExecutionPayloadEnvelopeV2, ExecutionPayloadEnvelopeV3, ExecutionPayloadEnvelopeV4,
    ExecutionPayloadV1, PayloadAttributes as SilPayloadAttributes,
};
use rsil_chainspec::{SilChainSpec, SilaHardforks};
use rsil_engine_primitives::{EngineApiValidator, PayloadValidator};
use rsil_sila_payload_builder::SilaExecutionPayloadValidator;
use rsil_sila_primitives::Block;
use rsil_node_api::PayloadTypes;
use rsil_payload_primitives::{
    validate_execution_requests, validate_version_specific_fields, EngineApiMessageVersion,
    EngineObjectValidationError, NewPayloadError, PayloadOrAttributes,
};
use rsil_primitives_traits::SealedBlock;
use std::sync::Arc;

/// Validator for the sila engine API.
#[derive(Debug, Clone)]
pub struct SilaEngineValidator<ChainSpec = rsil_chainspec::ChainSpec> {
    inner: SilaExecutionPayloadValidator<ChainSpec>,
}

impl<ChainSpec> SilaEngineValidator<ChainSpec> {
    /// Instantiates a new validator.
    pub const fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self { inner: SilaExecutionPayloadValidator::new(chain_spec) }
    }

    /// Returns the chain spec used by the validator.
    #[inline]
    fn chain_spec(&self) -> &ChainSpec {
        self.inner.chain_spec()
    }
}

impl<ChainSpec, Types> PayloadValidator<Types> for SilaEngineValidator<ChainSpec>
where
    ChainSpec: SilChainSpec + SilaHardforks + 'static,
    Types: PayloadTypes<ExecutionData = ExecutionData>,
{
    type Block = Block;

    fn convert_payload_to_block(
        &self,
        payload: ExecutionData,
    ) -> Result<SealedBlock<Self::Block>, NewPayloadError> {
        self.inner.ensure_well_formed_payload(payload).map_err(Into::into)
    }
}

impl<ChainSpec, Types> EngineApiValidator<Types> for SilaEngineValidator<ChainSpec>
where
    ChainSpec: SilChainSpec + SilaHardforks + 'static,
    Types: PayloadTypes<PayloadAttributes = SilPayloadAttributes, ExecutionData = ExecutionData>,
{
    fn validate_version_specific_fields(
        &self,
        version: EngineApiMessageVersion,
        payload_or_attrs: PayloadOrAttributes<'_, Types::ExecutionData, SilPayloadAttributes>,
    ) -> Result<(), EngineObjectValidationError> {
        payload_or_attrs
            .execution_requests()
            .map(|requests| validate_execution_requests(requests))
            .transpose()?;

        validate_version_specific_fields(self.chain_spec(), version, payload_or_attrs)
    }

    fn ensure_well_formed_attributes(
        &self,
        version: EngineApiMessageVersion,
        attributes: &SilPayloadAttributes,
    ) -> Result<(), EngineObjectValidationError> {
        validate_version_specific_fields(
            self.chain_spec(),
            version,
            PayloadOrAttributes::<Types::ExecutionData, SilPayloadAttributes>::PayloadAttributes(
                attributes,
            ),
        )
    }
}
