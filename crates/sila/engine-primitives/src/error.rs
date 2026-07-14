use thiserror::Error;

/// Error during [`SilBuiltPayload`](crate::SilBuiltPayload) to execution payload envelope
/// conversion.
#[derive(Error, Debug)]
pub enum BuiltPayloadConversionError {
    /// Unexpected SIP-4844 sidecars in the built payload.
    #[error("unexpected SIP-4844 sidecars")]
    UnexpectedEip4844Sidecars,
    /// Unexpected SIP-7594 sidecars in the built payload.
    #[error("unexpected SIP-7594 sidecars")]
    UnexpectedEip7594Sidecars,
    /// Missing block access list (required for V6 envelope).
    #[error("missing block access list")]
    MissingBlockAccessList,
}
