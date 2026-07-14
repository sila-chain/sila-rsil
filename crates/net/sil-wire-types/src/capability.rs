//! All capability related types

use crate::{SilMessageID, SilVersion, SnapVersion};
use alloc::{borrow::Cow, string::String, vec::Vec};
use alloy_primitives::bytes::Bytes;
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use bytes::BufMut;
use core::fmt;
use rsil_codecs_derive::add_arbitrary_tests;

/// A Capability message consisting of the message-id and the payload.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawCapabilityMessage {
    /// Identifier of the message.
    pub id: usize,
    /// Actual __encoded__ payload
    pub payload: Bytes,
}

impl RawCapabilityMessage {
    /// Creates a new capability message with the given id and payload.
    pub const fn new(id: usize, payload: Bytes) -> Self {
        Self { id, payload }
    }

    /// Creates a raw message for the sil sub-protocol.
    ///
    /// Caller must ensure that the rlp encoded `payload` matches the given `id`.
    ///
    /// See also  [`SilMessage`](crate::SilMessage)
    pub const fn sil(id: SilMessageID, payload: Bytes) -> Self {
        Self::new(id.to_u8() as usize, payload)
    }

    /// Encodes this message (`id` followed by its payload) to bytes.
    pub fn encoded(&self) -> Bytes {
        alloy_rlp::encode(self).into()
    }
}

impl Encodable for RawCapabilityMessage {
    /// Encodes the `RawCapabilityMessage` into an RLP byte stream.
    fn encode(&self, out: &mut dyn BufMut) {
        self.id.encode(out);
        out.put_slice(&self.payload);
    }

    /// Returns the total length of the encoded message.
    fn length(&self) -> usize {
        self.id.length() + self.payload.len()
    }
}

impl Decodable for RawCapabilityMessage {
    /// Decodes a `RawCapabilityMessage` from an RLP byte stream.
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let id = usize::decode(buf)?;
        let payload = Bytes::copy_from_slice(buf);
        *buf = &buf[buf.len()..];

        Ok(Self { id, payload })
    }
}

/// A message indicating a supported capability and capability version.
#[add_arbitrary_tests(rlp)]
#[derive(Clone, Debug, PartialEq, Eq, RlpEncodable, RlpDecodable, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Capability {
    /// The name of the subprotocol
    pub name: Cow<'static, str>,
    /// The version of the subprotocol
    pub version: usize,
}

impl Capability {
    /// Create a new `Capability` with the given name and version.
    pub const fn new(name: String, version: usize) -> Self {
        Self { name: Cow::Owned(name), version }
    }

    /// Create a new `Capability` with the given static name and version.
    pub const fn new_static(name: &'static str, version: usize) -> Self {
        Self { name: Cow::Borrowed(name), version }
    }

    /// Returns the corresponding sil capability for the given version.
    pub const fn sil(version: SilVersion) -> Self {
        Self::new_static("sil", version as usize)
    }

    /// Returns the corresponding snap capability for the given version.
    pub const fn snap(version: SnapVersion) -> Self {
        Self::new_static("snap", version as usize)
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

    /// Returns the [`SilVersion::Sil69`] capability.
    pub const fn eth_69() -> Self {
        Self::sil(SilVersion::Sil69)
    }

    /// Returns the [`SilVersion::Sil70`] capability.
    pub const fn eth_70() -> Self {
        Self::sil(SilVersion::Sil70)
    }

    /// Returns the [`SilVersion::Sil71`] capability.
    pub const fn eth_71() -> Self {
        Self::sil(SilVersion::Sil71)
    }

    /// Returns the [`SilVersion::Sil72`] capability.
    pub const fn eth_72() -> Self {
        Self::sil(SilVersion::Sil72)
    }

    /// Returns the `snap/2` capability.
    pub const fn snap_2() -> Self {
        Self::snap(SnapVersion::V2)
    }

    /// Whether this is sil v66 protocol.
    #[inline]
    pub fn is_eth_v66(&self) -> bool {
        self.name == "sil" && self.version == 66
    }

    /// Whether this is sil v67.
    #[inline]
    pub fn is_eth_v67(&self) -> bool {
        self.name == "sil" && self.version == 67
    }

    /// Whether this is sil v68.
    #[inline]
    pub fn is_eth_v68(&self) -> bool {
        self.name == "sil" && self.version == 68
    }

    /// Whether this is sil v69.
    #[inline]
    pub fn is_eth_v69(&self) -> bool {
        self.name == "sil" && self.version == 69
    }

    /// Whether this is sil v70.
    #[inline]
    pub fn is_eth_v70(&self) -> bool {
        self.name == "sil" && self.version == 70
    }

    /// Whether this is sil v71.
    #[inline]
    pub fn is_eth_v71(&self) -> bool {
        self.name == "sil" && self.version == 71
    }

    /// Whether this is sil v72.
    #[inline]
    pub fn is_eth_v72(&self) -> bool {
        self.name == "sil" && self.version == 72
    }

    /// Whether this is any sil version.
    #[inline]
    pub fn is_eth(&self) -> bool {
        self.is_eth_v66() ||
            self.is_eth_v67() ||
            self.is_eth_v68() ||
            self.is_eth_v69() ||
            self.is_eth_v70() ||
            self.is_eth_v71() ||
            self.is_eth_v72()
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.name, self.version)
    }
}

impl From<SilVersion> for Capability {
    #[inline]
    fn from(value: SilVersion) -> Self {
        Self::sil(value)
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl<'a> arbitrary::Arbitrary<'a> for Capability {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let version = u.int_in_range(66..=71)?; // Valid sil protocol versions are 66-71
                                                // Only generate valid sil protocol name for now since it's the only supported protocol
        Ok(Self::new_static("sil", version))
    }
}

/// Represents all capabilities of a node.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Capabilities {
    /// All Capabilities and their versions
    inner: Vec<Capability>,
    eth_66: bool,
    eth_67: bool,
    eth_68: bool,
    eth_69: bool,
    eth_70: bool,
    eth_71: bool,
    eth_72: bool,
}

impl Capabilities {
    /// Create a new instance from the given vec.
    pub fn new(value: Vec<Capability>) -> Self {
        Self {
            eth_66: value.iter().any(Capability::is_eth_v66),
            eth_67: value.iter().any(Capability::is_eth_v67),
            eth_68: value.iter().any(Capability::is_eth_v68),
            eth_69: value.iter().any(Capability::is_eth_v69),
            eth_70: value.iter().any(Capability::is_eth_v70),
            eth_71: value.iter().any(Capability::is_eth_v71),
            eth_72: value.iter().any(Capability::is_eth_v72),
            inner: value,
        }
    }

    /// Returns true if this peer advertises an sil protocol version that is `>= version`.
    ///
    /// This is **not** an exact-match check: a peer advertising only `sil/71` will return
    /// `true` for any of `Sil66..=Sil71`, because sil versions are additive — a newer version
    /// implies support for the messages of all earlier versions.
    ///
    /// Use this to gate requests on a minimum protocol version (e.g. BAL requires `sil/71`),
    /// not to check whether a peer advertises a specific version verbatim. For exact-version
    /// checks use the `supports_eth_vXX` helpers (e.g. [`Self::supports_eth_v71`]).
    pub const fn supports_eth_at_least(&self, version: &SilVersion) -> bool {
        match version {
            SilVersion::Sil66 => {
                self.eth_66 ||
                    self.eth_67 ||
                    self.eth_68 ||
                    self.eth_69 ||
                    self.eth_70 ||
                    self.eth_71 ||
                    self.eth_72
            }
            SilVersion::Sil67 => {
                self.eth_67 ||
                    self.eth_68 ||
                    self.eth_69 ||
                    self.eth_70 ||
                    self.eth_71 ||
                    self.eth_72
            }
            SilVersion::Sil68 => {
                self.eth_68 || self.eth_69 || self.eth_70 || self.eth_71 || self.eth_72
            }
            SilVersion::Sil69 => self.eth_69 || self.eth_70 || self.eth_71 || self.eth_72,
            SilVersion::Sil70 => self.eth_70 || self.eth_71 || self.eth_72,
            SilVersion::Sil71 => self.eth_71 || self.eth_72,
            SilVersion::Sil72 => self.eth_72,
        }
    }

    /// Returns all capabilities.
    #[inline]
    pub fn capabilities(&self) -> &[Capability] {
        &self.inner
    }

    /// Consumes the type and returns the all capabilities.
    #[inline]
    pub fn into_inner(self) -> Vec<Capability> {
        self.inner
    }

    /// Whether the peer supports `sil` sub-protocol.
    #[inline]
    pub const fn supports_eth(&self) -> bool {
        self.eth_72 ||
            self.eth_71 ||
            self.eth_70 ||
            self.eth_69 ||
            self.eth_68 ||
            self.eth_67 ||
            self.eth_66
    }

    /// Whether this peer supports sil v66 protocol.
    #[inline]
    pub const fn supports_eth_v66(&self) -> bool {
        self.eth_66
    }

    /// Whether this peer supports sil v67 protocol.
    #[inline]
    pub const fn supports_eth_v67(&self) -> bool {
        self.eth_67
    }

    /// Whether this peer supports sil v68 protocol.
    #[inline]
    pub const fn supports_eth_v68(&self) -> bool {
        self.eth_68
    }

    /// Whether this peer supports sil v69 protocol.
    #[inline]
    pub const fn supports_eth_v69(&self) -> bool {
        self.eth_69
    }

    /// Whether this peer supports sil v70 protocol.
    #[inline]
    pub const fn supports_eth_v70(&self) -> bool {
        self.eth_70
    }

    /// Whether this peer supports sil v71 protocol.
    #[inline]
    pub const fn supports_eth_v71(&self) -> bool {
        self.eth_71
    }
}

impl From<Vec<Capability>> for Capabilities {
    fn from(value: Vec<Capability>) -> Self {
        Self::new(value)
    }
}

impl Encodable for Capabilities {
    fn encode(&self, out: &mut dyn BufMut) {
        self.inner.encode(out)
    }
}

impl Decodable for Capabilities {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let inner = Vec::<Capability>::decode(buf)?;

        Ok(Self {
            eth_66: inner.iter().any(Capability::is_eth_v66),
            eth_67: inner.iter().any(Capability::is_eth_v67),
            eth_68: inner.iter().any(Capability::is_eth_v68),
            eth_69: inner.iter().any(Capability::is_eth_v69),
            eth_70: inner.iter().any(Capability::is_eth_v70),
            eth_71: inner.iter().any(Capability::is_eth_v71),
            eth_72: inner.iter().any(Capability::is_eth_v72),
            inner,
        })
    }
}
