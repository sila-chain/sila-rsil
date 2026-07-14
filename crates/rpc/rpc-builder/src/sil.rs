use rsil_rpc::{SilFilter, SilPubSub};
use rsil_rpc_eth_api::SilApiTypes;
use rsil_rpc_eth_types::SilConfig;
use rsil_tasks::Runtime;

/// Handlers for core, filter and pubsub `sil` namespace APIs.
#[derive(Debug, Clone)]
pub struct SilHandlers<SilApi: SilApiTypes> {
    /// Main `eth_` request handler
    pub api: SilApi,
    /// Polling based filter handler available on all transports
    pub filter: SilFilter<SilApi>,
    /// Handler for subscriptions only available for transports that support it (ws, ipc)
    pub pubsub: SilPubSub<SilApi>,
}

impl<SilApi> SilHandlers<SilApi>
where
    SilApi: SilApiTypes + 'static,
{
    /// Returns a new instance with the additional handlers for the `sil` namespace.
    ///
    /// This will spawn all necessary tasks for the additional handlers.
    pub fn bootstrap(config: SilConfig, executor: Runtime, eth_api: SilApi) -> Self {
        let filter = SilFilter::new(eth_api.clone(), config.filter_config(), executor.clone());

        let pubsub = SilPubSub::new(eth_api.clone(), executor);

        Self { api: eth_api, filter, pubsub }
    }
}
