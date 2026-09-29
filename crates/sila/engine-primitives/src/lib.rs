//! Sila specific engine API types and impls.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod payload;
use alloy_primitives::Bytes;
pub use payload::{BlobSidecars, SilBuiltPayload};

mod error;
pub use error::*;

use alloy_rpc_types_engine::{ExecutionData, ExecutionPayload};
pub use alloy_rpc_types_engine::{
    ExecutionPayloadEnvelopeV2, ExecutionPayloadEnvelopeV3, ExecutionPayloadEnvelopeV4,
    ExecutionPayloadEnvelopeV5, ExecutionPayloadEnvelopeV6, ExecutionPayloadV1,
    PayloadAttributes as SilPayloadAttributes,
};
use rsil_engine_primitives::EngineTypes;
use rsil_payload_primitives::{BuiltPayload, PayloadTypes};
use rsil_primitives_traits::{NodePrimitives, SealedBlock};

/// The types used in the default sila-mainnet sila beacon consensus engine.
#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct SilEngineTypes<T: PayloadTypes = SilPayloadTypes> {
    _marker: core::marker::PhantomData<T>,
}

impl<T> PayloadTypes for SilEngineTypes<T>
where
    T: PayloadTypes<
        ExecutionData = ExecutionData,
        BuiltPayload: BuiltPayload<Primitives: NodePrimitives<Block = rsil_sila_primitives::Block>>,
    >,
    ExecutionData: From<T::BuiltPayload>,
{
    type ExecutionData = T::ExecutionData;
    type BuiltPayload = T::BuiltPayload;
    type PayloadAttributes = T::PayloadAttributes;

    fn block_to_payload(
        block: SealedBlock<
            <<Self::BuiltPayload as BuiltPayload>::Primitives as NodePrimitives>::Block,
        >,
        bal: Option<Bytes>,
    ) -> Self::ExecutionData {
        T::block_to_payload(block, bal)
    }
}

impl<T> EngineTypes for SilEngineTypes<T>
where
    T: PayloadTypes<ExecutionData = ExecutionData>,
    ExecutionData: From<T::BuiltPayload>,
    T::BuiltPayload: BuiltPayload<Primitives: NodePrimitives<Block = rsil_sila_primitives::Block>>
        + TryInto<ExecutionPayloadV1>
        + TryInto<ExecutionPayloadEnvelopeV2>
        + TryInto<ExecutionPayloadEnvelopeV3>
        + TryInto<ExecutionPayloadEnvelopeV4>
        + TryInto<ExecutionPayloadEnvelopeV5>
        + TryInto<ExecutionPayloadEnvelopeV6>,
{
    type ExecutionPayloadEnvelopeV1 = ExecutionPayloadV1;
    type ExecutionPayloadEnvelopeV2 = ExecutionPayloadEnvelopeV2;
    type ExecutionPayloadEnvelopeV3 = ExecutionPayloadEnvelopeV3;
    type ExecutionPayloadEnvelopeV4 = ExecutionPayloadEnvelopeV4;
    type ExecutionPayloadEnvelopeV5 = ExecutionPayloadEnvelopeV5;
    type ExecutionPayloadEnvelopeV6 = ExecutionPayloadEnvelopeV6;
}

/// A default payload type for [`SilEngineTypes`]
#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct SilPayloadTypes;

impl PayloadTypes for SilPayloadTypes {
    type BuiltPayload = SilBuiltPayload;
    type PayloadAttributes = SilPayloadAttributes;
    type ExecutionData = ExecutionData;

    fn block_to_payload(
        block: SealedBlock<
            <<Self::BuiltPayload as BuiltPayload>::Primitives as NodePrimitives>::Block,
        >,
        bal: Option<Bytes>,
    ) -> Self::ExecutionData {
        let (payload, sidecar) = ExecutionPayload::from_block_unchecked_with_extras(
            block.hash(),
            &block.into_block(),
            bal,
        );
        ExecutionData { payload, sidecar }
    }
}
