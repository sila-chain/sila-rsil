use alloy_primitives::BlockNumber;
use rsil_errors::ProviderResult;
use rsil_static_file_types::StaticFileSegment;
use rsil_storage_api::NodePrimitivesProvider;

use crate::providers::{StaticFileProvider, StaticFileProviderRWRefMut};

/// Static file provider factory.
pub trait StaticFileProviderFactory: NodePrimitivesProvider {
    /// Create new instance of static file provider.
    fn static_file_provider(&self) -> StaticFileProvider<Self::Primitives>;

    /// Returns a mutable reference to a
    /// [`StaticFileProviderRW`](`crate::providers::StaticFileProviderRW`) of a
    /// [`StaticFileSegment`].
    fn get_static_file_writer(
        &self,
        block: BlockNumber,
        segment: StaticFileSegment,
    ) -> ProviderResult<StaticFileProviderRWRefMut<'_, Self::Primitives>>;
}
