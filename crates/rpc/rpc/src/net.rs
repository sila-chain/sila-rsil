use alloy_primitives::U64;
use jsonrpsee::core::RpcResult as Result;
use rsil_network_api::PeersInfo;
use rsil_rpc_api::NetApiServer;
use rsil_rpc_eth_api::helpers::SilApiSpec;

/// `Net` API implementation.
///
/// This type provides the functionality for handling `net` related requests.
pub struct NetApi<Net, Sil> {
    /// An interface to interact with the network
    network: Net,
    /// The implementation of `sil` API
    sil: Sil,
}

// === impl NetApi ===

impl<Net, Sil> NetApi<Net, Sil> {
    /// Returns a new instance with the given network and sil interface implementations
    pub const fn new(network: Net, sil: Sil) -> Self {
        Self { network, sil }
    }
}

/// Net rpc implementation
impl<Net, Sil> NetApiServer for NetApi<Net, Sil>
where
    Net: PeersInfo + 'static,
    Sil: SilApiSpec + 'static,
{
    /// Handler for `net_version`
    fn version(&self) -> Result<String> {
        // Note: net_version is numeric: <https://github.com/sila-chain/sila-rsil/issues/5569>
        Ok(self.sil.chain_id().to::<u64>().to_string())
    }

    /// Handler for `net_peerCount`
    fn peer_count(&self) -> Result<U64> {
        Ok(U64::from(self.network.num_connected_peers()))
    }

    /// Handler for `net_listening`
    fn is_listening(&self) -> Result<bool> {
        Ok(true)
    }
}

impl<Net, Sil> std::fmt::Debug for NetApi<Net, Sil> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetApi").finish_non_exhaustive()
    }
}
