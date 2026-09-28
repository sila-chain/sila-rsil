//! `eth_` RPC API for pubsub subscription.

use alloy_json_rpc::RpcObject;
use alloy_rpc_types_eth::pubsub::{Params, SubscriptionKind};
use jsonrpsee::proc_macros::rpc;

/// Sila pub-sub rpc interface.
#[rpc(server, namespace = "sil")]
pub trait SilPubSubApi<T: RpcObject> {
    /// Create an sila subscription for the given params
    #[subscription(
        name = "subscribe" => "subscription",
        unsubscribe = "unsubscribe",
        item = alloy_rpc_types::pubsub::SubscriptionResult
    )]
    async fn subscribe(
        &self,
        kind: SubscriptionKind,
        params: Option<Params>,
    ) -> jsonrpsee::core::SubscriptionResult;
}
