use alloc::vec;

use alloy_primitives::U256;

use once_cell as _;
#[cfg(not(feature = "std"))]
use once_cell::sync::Lazy as LazyLock;
#[cfg(feature = "std")]
use std::sync::LazyLock;

use crate::{ChainHardforks, ForkCondition, Hardfork, SilaHardfork};

/// Dev hardforks
pub static DEV_HARDFORKS: LazyLock<ChainHardforks> = LazyLock::new(|| {
    ChainHardforks::new(vec![
        (SilaHardfork::Frontier.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::Homestead.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::Dao.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::SpuriousDragon.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::Constantinople.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::Petersburg.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::Berlin.boxed(), ForkCondition::Block(0)),
        (SilaHardfork::London.boxed(), ForkCondition::Block(0)),
        (
            SilaHardfork::Paris.boxed(),
            ForkCondition::TTD {
                activation_block_number: 0,
                fork_block: None,
                total_difficulty: U256::ZERO,
            },
        ),
        (SilaHardfork::Shanghai.boxed(), ForkCondition::Timestamp(0)),
        (SilaHardfork::Cancun.boxed(), ForkCondition::Timestamp(0)),
        (SilaHardfork::Prague.boxed(), ForkCondition::Timestamp(0)),
        (SilaHardfork::Osaka.boxed(), ForkCondition::Timestamp(0)),
    ])
});
