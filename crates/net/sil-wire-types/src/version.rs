//! Support for representing the version of the `sil`

use crate::alloc::string::ToString;
use alloc::string::String;
use alloy_rlp::{Decodable, Encodable, Error as RlpError};
use bytes::BufMut;
use core::{fmt, str::FromStr};
use derive_more::Display;
use rsil_codecs_derive::add_arbitrary_tests;

/// Error thrown when failed to parse a valid [`SilVersion`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Unknown sil protocol version: {0}")]
pub struct ParseVersionError(String);

/// The `sil` protocol version.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(test, feature = "arbitrary"), derive(arbitrary::Arbitrary))]
pub enum SilVersion {
    /// The `sil` protocol version 66.
    Sil66 = 66,
    /// The `sil` protocol version 67.
    Sil67 = 67,
    /// The `sil` protocol version 68.
    Sil68 = 68,
    /// The `sil` protocol version 69.
    Sil69 = 69,
    /// The `sil` protocol version 70.
    ///
    /// [SIP-7975](https://sips.sila.org/SIPS/sip-7975) adds partial block receipt
    /// lists by extending `GetReceipts` and `Receipts` with pagination fields.
    Sil70 = 70,
    /// The `sil` protocol version 71.
    ///
    /// [SIP-8159](https://sips.sila.org/SIPS/sip-8159) adds block access list
    /// exchange with `GetBlockAccessLists` and `BlockAccessLists`.
    Sil71 = 71,
    /// The `sil` protocol version 72.
    ///
    /// [SIP-8070](https://sips.sila.org/SIPS/sip-8070) adds sparse blobpool
    /// support by extending `NewPooledTransactionHashes` with `cell_mask` and adding
    /// `GetCells` and `Cells`.
    Sil72 = 72,
}

impl SilVersion {
    /// The latest known sil version
    pub const LATEST: Self = Self::Sil69;

    /// All known sil versions
    pub const ALL_VERSIONS: &'static [Self] = &[Self::Sil69, Self::Sil68, Self::Sil67, Self::Sil66];

    /// Returns true if the version is sil/66
    pub const fn is_eth66(&self) -> bool {
        matches!(self, Self::Sil66)
    }

    /// Returns true if the version is sil/67
    pub const fn is_eth67(&self) -> bool {
        matches!(self, Self::Sil67)
    }

    /// Returns true if the version is sil/68
    pub const fn is_eth68(&self) -> bool {
        matches!(self, Self::Sil68)
    }

    /// Returns true if the version carries sil/68 transaction announcement metadata.
    pub const fn has_eth68_metadata(&self) -> bool {
        matches!(self, Self::Sil68 | Self::Sil69 | Self::Sil70 | Self::Sil71 | Self::Sil72)
    }

    /// Returns true if the version is sil/69
    pub const fn is_eth69(&self) -> bool {
        matches!(self, Self::Sil69)
    }

    /// Returns true if the version is sil/70
    pub const fn is_eth70(&self) -> bool {
        matches!(self, Self::Sil70)
    }

    /// Returns true if the version is sil/71
    pub const fn is_eth71(&self) -> bool {
        matches!(self, Self::Sil71)
    }

    /// Returns true if the version is sil/72
    pub const fn is_eth72(&self) -> bool {
        matches!(self, Self::Sil72)
    }

    /// Returns true if the version is sil/69 or newer.
    pub const fn is_eth69_or_newer(&self) -> bool {
        matches!(self, Self::Sil69 | Self::Sil70 | Self::Sil71 | Self::Sil72)
    }
}

/// RLP encodes `SilVersion` as a single byte (66-72).
impl Encodable for SilVersion {
    fn encode(&self, out: &mut dyn BufMut) {
        (*self as u8).encode(out)
    }

    fn length(&self) -> usize {
        (*self as u8).length()
    }
}

/// RLP decodes a single byte into `SilVersion`.
/// Returns error if byte is not a valid version (66-72).
impl Decodable for SilVersion {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let version = u8::decode(buf)?;
        Self::try_from(version).map_err(|_| RlpError::Custom("invalid sil version"))
    }
}

/// Allow for converting from a `&str` to an `SilVersion`.
///
/// # Example
/// ```
/// use rsil_eth_wire_types::SilVersion;
///
/// let version = SilVersion::try_from("67").unwrap();
/// assert_eq!(version, SilVersion::Sil67);
/// ```
impl TryFrom<&str> for SilVersion {
    type Error = ParseVersionError;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "66" => Ok(Self::Sil66),
            "67" => Ok(Self::Sil67),
            "68" => Ok(Self::Sil68),
            "69" => Ok(Self::Sil69),
            "70" => Ok(Self::Sil70),
            "71" => Ok(Self::Sil71),
            "72" => Ok(Self::Sil72),
            _ => Err(ParseVersionError(s.to_string())),
        }
    }
}

/// Allow for converting from a u8 to an `SilVersion`.
///
/// # Example
/// ```
/// use rsil_eth_wire_types::SilVersion;
///
/// let version = SilVersion::try_from(67).unwrap();
/// assert_eq!(version, SilVersion::Sil67);
/// ```
impl TryFrom<u8> for SilVersion {
    type Error = ParseVersionError;

    #[inline]
    fn try_from(u: u8) -> Result<Self, Self::Error> {
        match u {
            66 => Ok(Self::Sil66),
            67 => Ok(Self::Sil67),
            68 => Ok(Self::Sil68),
            69 => Ok(Self::Sil69),
            70 => Ok(Self::Sil70),
            71 => Ok(Self::Sil71),
            72 => Ok(Self::Sil72),
            _ => Err(ParseVersionError(u.to_string())),
        }
    }
}

impl FromStr for SilVersion {
    type Err = ParseVersionError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl From<SilVersion> for u8 {
    #[inline]
    fn from(v: SilVersion) -> Self {
        v as Self
    }
}

impl From<SilVersion> for &'static str {
    #[inline]
    fn from(v: SilVersion) -> &'static str {
        match v {
            SilVersion::Sil66 => "66",
            SilVersion::Sil67 => "67",
            SilVersion::Sil68 => "68",
            SilVersion::Sil69 => "69",
            SilVersion::Sil70 => "70",
            SilVersion::Sil71 => "71",
            SilVersion::Sil72 => "72",
        }
    }
}

/// `RLPx` `p2p` protocol version
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(test, feature = "arbitrary"), derive(arbitrary::Arbitrary))]
#[add_arbitrary_tests(rlp)]
pub enum ProtocolVersion {
    /// `p2p` version 4
    V4 = 4,
    /// `p2p` version 5
    #[default]
    V5 = 5,
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", *self as u8)
    }
}

impl Encodable for ProtocolVersion {
    fn encode(&self, out: &mut dyn BufMut) {
        (*self as u8).encode(out)
    }
    fn length(&self) -> usize {
        // the version should be a single byte
        (*self as u8).length()
    }
}

impl Decodable for ProtocolVersion {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let version = u8::decode(buf)?;
        match version {
            4 => Ok(Self::V4),
            5 => Ok(Self::V5),
            _ => Err(RlpError::Custom("unknown p2p protocol version")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SilVersion;
    use alloy_rlp::{Decodable, Encodable, Error as RlpError};
    use bytes::BytesMut;

    #[test]
    fn test_eth_version_try_from_str() {
        assert_eq!(SilVersion::Sil66, SilVersion::try_from("66").unwrap());
        assert_eq!(SilVersion::Sil67, SilVersion::try_from("67").unwrap());
        assert_eq!(SilVersion::Sil68, SilVersion::try_from("68").unwrap());
        assert_eq!(SilVersion::Sil69, SilVersion::try_from("69").unwrap());
        assert_eq!(SilVersion::Sil70, SilVersion::try_from("70").unwrap());
        assert_eq!(SilVersion::Sil71, SilVersion::try_from("71").unwrap());
        assert_eq!(SilVersion::Sil72, SilVersion::try_from("72").unwrap());
    }

    #[test]
    fn test_eth_version_from_str() {
        assert_eq!(SilVersion::Sil66, "66".parse().unwrap());
        assert_eq!(SilVersion::Sil67, "67".parse().unwrap());
        assert_eq!(SilVersion::Sil68, "68".parse().unwrap());
        assert_eq!(SilVersion::Sil69, "69".parse().unwrap());
        assert_eq!(SilVersion::Sil70, "70".parse().unwrap());
        assert_eq!(SilVersion::Sil71, "71".parse().unwrap());
        assert_eq!(SilVersion::Sil72, "72".parse().unwrap());
    }

    #[test]
    fn test_has_eth68_metadata() {
        assert!(!SilVersion::Sil66.has_eth68_metadata());
        assert!(!SilVersion::Sil67.has_eth68_metadata());
        assert!(SilVersion::Sil68.has_eth68_metadata());
        assert!(SilVersion::Sil69.has_eth68_metadata());
        assert!(SilVersion::Sil70.has_eth68_metadata());
        assert!(SilVersion::Sil71.has_eth68_metadata());
        assert!(SilVersion::Sil72.has_eth68_metadata());
    }

    #[test]
    fn test_eth_version_rlp_encode() {
        let versions = [
            SilVersion::Sil66,
            SilVersion::Sil67,
            SilVersion::Sil68,
            SilVersion::Sil69,
            SilVersion::Sil70,
            SilVersion::Sil71,
            SilVersion::Sil72,
        ];

        for version in versions {
            let mut encoded = BytesMut::new();
            version.encode(&mut encoded);

            assert_eq!(encoded.len(), 1);
            assert_eq!(encoded[0], version as u8);
        }
    }
    #[test]
    fn test_eth_version_rlp_decode() {
        let test_cases = [
            (66_u8, Ok(SilVersion::Sil66)),
            (67_u8, Ok(SilVersion::Sil67)),
            (68_u8, Ok(SilVersion::Sil68)),
            (69_u8, Ok(SilVersion::Sil69)),
            (70_u8, Ok(SilVersion::Sil70)),
            (71_u8, Ok(SilVersion::Sil71)),
            (72_u8, Ok(SilVersion::Sil72)),
            (65_u8, Err(RlpError::Custom("invalid sil version"))),
        ];

        for (input, expected) in test_cases {
            let mut encoded = BytesMut::new();
            input.encode(&mut encoded);

            let mut slice = encoded.as_ref();
            let result = SilVersion::decode(&mut slice);
            assert_eq!(result, expected);
        }
    }
}
