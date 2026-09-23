//! The implementation of the [`PayloadAttributesBuilder`] for the
//! [`LocalMiner`](super::LocalMiner).

use alloy_consensus::BlockHeader;
use alloy_primitives::{Address, B256};
use rsil_chainspec::{SilChainSpec, SilaHardforks};
use rsil_payload_primitives::PayloadAttributesBuilder;
use rsil_primitives_traits::SealedHeader;
use rsil_sila_engine_primitives::SilPayloadAttributes;
use std::sync::Arc;

/// The attributes builder for local Sila payload.
#[derive(Debug)]
#[non_exhaustive]
pub struct LocalPayloadAttributesBuilder<ChainSpec> {
    /// The chainspec
    pub chain_spec: Arc<ChainSpec>,

    /// Whether to enforce increasing timestamp.
    pub enforce_increasing_timestamp: bool,
}

impl<ChainSpec> LocalPayloadAttributesBuilder<ChainSpec> {
    /// Creates a new instance of the builder.
    pub const fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self { chain_spec, enforce_increasing_timestamp: true }
    }

    /// Creates a new instance of the builder without enforcing increasing timestamps.
    pub fn without_increasing_timestamp(self) -> Self {
        Self { enforce_increasing_timestamp: false, ..self }
    }
}

impl<ChainSpec> PayloadAttributesBuilder<SilPayloadAttributes, ChainSpec::Header>
    for LocalPayloadAttributesBuilder<ChainSpec>
where
    ChainSpec: SilChainSpec + SilaHardforks + 'static,
{
    fn build(&self, parent: &SealedHeader<ChainSpec::Header>) -> SilPayloadAttributes {
        let mut timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        if self.enforce_increasing_timestamp {
            timestamp = std::cmp::max(parent.timestamp().saturating_add(1), timestamp);
        }

        SilPayloadAttributes {
            timestamp,
            prev_randao: B256::random(),
            suggested_fee_recipient: Address::random(),
            withdrawals: self
                .chain_spec
                .is_shanghai_active_at_timestamp(timestamp)
                .then(Default::default),
            parent_beacon_block_root: self
                .chain_spec
                .is_cancun_active_at_timestamp(timestamp)
                .then(B256::random),
            slot_number: self.chain_spec.is_amsterdam_active_at_timestamp(timestamp).then_some(0),
            ..Default::default()
        }
    }
}
