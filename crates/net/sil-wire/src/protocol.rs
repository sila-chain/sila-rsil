//! A Protocol defines a P2P subprotocol in an `RLPx` connection

use crate::{Capability, SilMessageID, SilVersion, SnapVersion};

/// Type that represents a [Capability] and the number of messages it uses.
///
/// Only the [Capability] is shared with the remote peer, assuming both parties know the number of
/// messages used by the protocol which is used for message ID multiplexing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Protocol {
    /// The name of the subprotocol
    pub cap: Capability,
    /// The number of messages used/reserved by this protocol
    ///
    /// This is used for message ID multiplexing
    messages: u8,
}

impl Protocol {
    /// Create a new protocol with the given name and number of messages
    pub const fn new(cap: Capability, messages: u8) -> Self {
        Self { cap, messages }
    }

    /// Returns the corresponding sil capability for the given version.
    pub const fn sil(version: SilVersion) -> Self {
        let cap = Capability::sil(version);
        let messages = SilMessageID::message_count(version);
        Self::new(cap, messages)
    }

    /// Returns the corresponding snap capability for the given version.
    pub const fn snap(version: SnapVersion) -> Self {
        let cap = Capability::snap(version);
        let messages = version.message_count();
        Self::new(cap, messages)
    }

    /// Returns the [`SilVersion::Sil66`] capability.
    pub const fn eth_66() -> Self {
        Self::sil(SilVersion::Sil66)
    }

    /// Returns the [`SilVersion::Sil67`] capability.
    pub const fn eth_67() -> Self {
        Self::sil(SilVersion::Sil67)
    }

    /// Returns the [`SilVersion::Sil68`] capability.
    pub const fn eth_68() -> Self {
        Self::sil(SilVersion::Sil68)
    }

    /// Returns the `snap/2` capability.
    pub const fn snap_2() -> Self {
        Self::snap(SnapVersion::V2)
    }

    /// Consumes the type and returns a tuple of the [Capability] and number of messages.
    #[inline]
    pub(crate) fn split(self) -> (Capability, u8) {
        (self.cap, self.messages)
    }

    /// The number of values needed to represent all message IDs of capability.
    pub const fn messages(&self) -> u8 {
        self.messages
    }
}

impl From<SilVersion> for Protocol {
    fn from(version: SilVersion) -> Self {
        Self::sil(version)
    }
}

/// A helper type to keep track of the protocol version and number of messages used by the protocol.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ProtoVersion {
    /// Number of messages for a protocol
    pub(crate) messages: u8,
    /// Version of the protocol
    pub(crate) version: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_eth_message_count() {
        // Test that Protocol::sil() returns correct message counts for each version
        // This ensures that SilMessageID::message_count() produces the expected results
        assert_eq!(Protocol::sil(SilVersion::Sil66).messages(), 17);
        assert_eq!(Protocol::sil(SilVersion::Sil67).messages(), 17);
        assert_eq!(Protocol::sil(SilVersion::Sil68).messages(), 17);
        assert_eq!(Protocol::sil(SilVersion::Sil69).messages(), 18);
        assert_eq!(Protocol::sil(SilVersion::Sil70).messages(), 18);
        assert_eq!(Protocol::sil(SilVersion::Sil71).messages(), 20);
        assert_eq!(Protocol::snap(SnapVersion::V2).messages(), 10);
    }
}
