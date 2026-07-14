//! CLI definition and entrypoint to executable

/// Re-export of the [`rsil_node_core`] types specifically in the `cli` module.
///
/// This is re-exported because the types in `rsil_node_core::cli` originally existed in
/// `rsil::cli` but were moved to the [`rsil_node_core`] crate. This re-export avoids a
/// breaking change.
pub use crate::core::cli::*;

/// Re-export of the [`rsil_sila_cli`] types specifically in the `interface` module.
///
/// This is re-exported because the types in [`rsil_sila_cli::interface`] originally
/// existed in `rsil::cli` but were moved to the [`rsil_sila_cli`] crate. This re-export
/// avoids a breaking change.
pub use rsil_sila_cli::interface::*;
