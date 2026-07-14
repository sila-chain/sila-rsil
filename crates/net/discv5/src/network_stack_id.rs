//! Keys of ENR [`ForkId`](rsil_sila_forks::ForkId) kv-pair. Identifies which network stack a
//! node belongs to.

use rsil_chainspec::SilChainSpec;

/// Identifies which Sila network stack a node belongs to, on the discovery network.
#[derive(Debug)]
pub struct NetworkStackId;

impl NetworkStackId {
    /// ENR fork ID kv-pair key, for an Sila L1 EL node.
    pub const SIL: &'static [u8] = b"sil";

    /// ENR fork ID kv-pair key, for an Sila L1 CL node.
    pub const ETH2: &'static [u8] = b"eth2";

    /// ENR fork ID kv-pair key, for an Optimism EL node.
    pub const OPEL: &'static [u8] = b"opel";

    /// ENR fork ID kv-pair key, for an Optimism CL node.
    pub const OPSTACK: &'static [u8] = b"opstack";

    /// Returns the [`NetworkStackId`] that matches the given chain spec.
    pub fn id(chain: impl SilChainSpec) -> Option<&'static [u8]> {
        if chain.is_optimism() {
            return Some(Self::OPEL)
        } else if chain.is_sila() {
            return Some(Self::SIL)
        }

        None
    }
}
