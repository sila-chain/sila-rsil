//! E2E test for forkchoice update with finalized blocks.
//!
//! This test verifies the behavior when attempting to reorg behind a finalized block.

use eyre::Result;
use rsil_chainspec::{ChainSpecBuilder, SILA_MAINNET};
use rsil_e2e_test_utils::testsuite::{
    actions::{
        BlockReference, CaptureBlock, CreateFork, FinalizeBlock, MakeCanonical, ProduceBlocks,
        SendForkchoiceUpdate,
    },
    setup::{NetworkSetup, Setup},
    TestBuilder,
};
use rsil_engine_tree::tree::TreeConfig;
use rsil_sila_engine_primitives::SilEngineTypes;
use rsil_node_sila::SilaNode;
use std::sync::Arc;

/// Creates the standard setup for engine tree e2e tests.
fn default_engine_tree_setup() -> Setup<SilEngineTypes> {
    Setup::default()
        .with_chain_spec(Arc::new(
            ChainSpecBuilder::default()
                .chain(SILA_MAINNET.chain)
                .genesis(
                    serde_json::from_str(include_str!(
                        "../../../../e2e-test-utils/src/testsuite/assets/genesis.json"
                    ))
                    .unwrap(),
                )
                .cancun_activated()
                .build(),
        ))
        .with_network(NetworkSetup::single_node())
        .with_tree_config(TreeConfig::default().with_has_enough_parallelism(true))
}

/// This test:
/// 1. Creates a main chain and finalizes a block
/// 2. Creates a fork that branches BEFORE the finalized block
/// 3. Attempts to switch to that fork (which would require changing history behind finalized)
#[tokio::test]
async fn test_reorg_to_fork_behind_finalized() -> Result<()> {
    rsil_tracing::init_test_tracing();

    let test = TestBuilder::new()
        .with_setup(default_engine_tree_setup())
        // Build main chain: blocks 1-10
        .with_action(ProduceBlocks::<SilEngineTypes>::new(10))
        .with_action(MakeCanonical::new())
        // Capture blocks for the test
        .with_action(CaptureBlock::new("block_5")) // Will be fork point
        .with_action(CaptureBlock::new("block_7")) // Will be finalized
        .with_action(CaptureBlock::new("block_10")) // Current head
        // Create a fork from block 5 (before block 7 which will be finalized)
        .with_action(CreateFork::<SilEngineTypes>::new(5, 5)) // Fork from block 5, add 5 blocks
        .with_action(CaptureBlock::new("fork_tip"))
        // Step 1: Finalize block 7 with head at block 10
        .with_action(
            FinalizeBlock::<SilEngineTypes>::new(BlockReference::Tag("block_7".to_string()))
                .with_head(BlockReference::Tag("block_10".to_string())),
        )
        // Step 2: Attempt to reorg to a fork that doesn't contain the finalized block
        .with_action(
            SendForkchoiceUpdate::<SilEngineTypes>::new(
                BlockReference::Tag("block_7".to_string()), // Keep finalized
                BlockReference::Tag("fork_tip".to_string()), // New safe
                BlockReference::Tag("fork_tip".to_string()), // New head
            )
            .with_expected_status(alloy_rpc_types_engine::PayloadStatusEnum::Valid),
        );

    test.run::<SilaNode>().await?;

    Ok(())
}
