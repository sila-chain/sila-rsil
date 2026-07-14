//! rsil P2P networking.
//!
//! Sila's networking protocol is specified in [devp2p](https://github.com/sila-chain/devp2p).
//!
//! In order for a node to join the sila p2p network it needs to know what nodes are already
//! part of that network. This includes public identities (public key) and addresses (where to reach
//! them).
//!
//! ## Bird's Eye View
//!
//! See also diagram in [`NetworkManager`]
//!
//! The `Network` is made up of several, separate tasks:
//!
//!    - `Transactions Task`: is a spawned
//!      [`TransactionsManager`](crate::transactions::TransactionsManager) future that:
//!
//!        * Responds to incoming transaction related requests
//!        * Requests missing transactions from the `Network`
//!        * Broadcasts new transactions received from the
//!          [`TransactionPool`](rsil_transaction_pool::TransactionPool) over the `Network`
//!
//!    - `SIL request Task`: is a spawned
//!      [`SilRequestHandler`](crate::eth_requests::SilRequestHandler) future that:
//!
//!        * Responds to incoming SIL related requests: `Headers`, `Bodies`
//!
//!    - `Discovery Task`: is a spawned [`Discv4`](rsil_discv4::Discv4) future that handles peer
//!      discovery and emits new peers to the `Network`
//!
//!    - [`NetworkManager`] task advances the state of the `Network`, which includes:
//!
//!        * Initiating new _outgoing_ connections to discovered peers
//!        * Handling _incoming_ TCP connections from peers
//!        * Peer management
//!        * Route requests:
//!             - from remote peers to corresponding tasks
//!             - from local to remote peers
//!
//! ## Usage
//!
//! ### Configure and launch a standalone network
//!
//! The [`NetworkConfig`] is used to configure the network.
//! It requires an instance of [`BlockReader`](rsil_storage_api::BlockReader).
//!
//! ```
//! # async fn launch() {
//! use rsil_network::{
//!     config::rng_secret_key, SilNetworkPrimitives, NetworkConfig, NetworkManager,
//! };
//! use rsil_network_peers::mainnet_nodes;
//! use rsil_storage_api::noop::NoopProvider;
//! use rsil_tasks::Runtime;
//!
//! // This block provider implementation is used for testing purposes.
//! let client = NoopProvider::default();
//!
//! // The key that's used for encrypting sessions and to identify our node.
//! let local_key = rng_secret_key();
//!
//! let config = NetworkConfig::<_, SilNetworkPrimitives>::builder(local_key, Runtime::test())
//!     .boot_nodes(mainnet_nodes())
//!     .build(client);
//!
//! // create the network instance
//! let network = NetworkManager::new(config).await.unwrap();
//!
//! // keep a handle to the network and spawn it
//! let handle = network.handle().clone();
//! tokio::task::spawn(network);
//!
//! # }
//! ```
//!
//! ### Configure all components of the Network with the [`NetworkBuilder`]
//!
//! ```
//! use rsil_network::{
//!     config::rng_secret_key, SilNetworkPrimitives, NetworkConfig, NetworkManager,
//! };
//! use rsil_network_peers::mainnet_nodes;
//! use rsil_storage_api::noop::NoopProvider;
//! use rsil_tasks::Runtime;
//! use rsil_transaction_pool::TransactionPool;
//! async fn launch<Pool: TransactionPool>(pool: Pool) {
//!     // This block provider implementation is used for testing purposes.
//!     let client = NoopProvider::default();
//!
//!     // The key that's used for encrypting sessions and to identify our node.
//!     let local_key = rng_secret_key();
//!
//!     let config = NetworkConfig::<_, SilNetworkPrimitives>::builder(local_key, Runtime::test())
//!         .boot_nodes(mainnet_nodes())
//!         .build(client.clone());
//!     let transactions_manager_config = config.transactions_manager_config.clone();
//!
//!     // create the network instance
//!     let (handle, network, transactions, request_handler) = NetworkManager::builder(config)
//!         .await
//!         .unwrap()
//!         .transactions(pool, transactions_manager_config)
//!         .request_handler(client)
//!         .split_with_handle();
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `serde` (default): Enable serde support for configuration types.
//! - `test-utils`: Various utilities helpful for writing tests

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/sila-chain/sila-rsil/main/assets/rsil-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/sila-chain/sila-rsil/issues/"
)]
#![allow(unreachable_pub)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(test, feature = "test-utils"))]
/// Common helpers for network testing.
pub mod test_utils;

pub mod cache;
pub mod config;
pub mod error;
pub mod eth_requests;
pub mod import;
pub mod message;
pub mod peers;
pub mod protocol;
pub mod transactions;

mod budget;
mod builder;
mod discovery;
mod fetch;
mod flattened_response;
mod listener;
mod manager;
mod metrics;
mod network;
mod required_block_filter;
mod session;
mod state;
mod swarm;
mod trusted_peers_resolver;

pub use rsil_eth_wire::{DisconnectReason, HelloMessageWithProtocols};
pub use rsil_eth_wire_types::{primitives, SilNetworkPrimitives, NetworkPrimitives};
pub use rsil_network_api::{
    events, BlockDownloaderProvider, DiscoveredEvent, DiscoveryEvent, NetworkEvent,
    NetworkEventListenerProvider, NetworkInfo, PeerRequest, PeerRequestSender, Peers, PeersInfo,
};
pub use rsil_network_p2p::sync::{NetworkSyncUpdater, SyncState};
pub use rsil_network_types::{PeersConfig, SessionsConfig};
pub use session::{
    ActiveSessionHandle, ActiveSessionMessage, Direction, SilRlpxConnection, PeerInfo,
    PendingSessionEvent, PendingSessionHandle, PendingSessionHandshakeError, SessionCommand,
    SessionEvent, SessionId, SessionManager,
};

pub use builder::NetworkBuilder;
pub use config::{NetworkConfig, NetworkConfigBuilder};
pub use discovery::Discovery;
pub use fetch::FetchClient;
pub use flattened_response::FlattenedResponse;
pub use manager::NetworkManager;
pub use metrics::TxTypesCounter;
pub use network::{NetworkHandle, NetworkProtocols};
pub use swarm::NetworkConnectionState;

/// re-export p2p interfaces
pub use rsil_network_p2p as p2p;

/// re-export types crates
pub mod types {
    pub use rsil_discv4::NatResolver;
    pub use rsil_eth_wire_types::*;
    pub use rsil_network_types::*;
}

use aquamarine as _;

use smallvec as _;
