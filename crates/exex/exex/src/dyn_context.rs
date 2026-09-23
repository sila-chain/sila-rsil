//! Mirrored version of [`ExExContext`](`crate::ExExContext`)
//! without generic abstraction over [Node](`rsil_node_api::FullNodeComponents`)

use alloy_eips::BlockNumHash;
use rsil_chainspec::SilChainSpec;
use rsil_node_api::{FullNodeComponents, HeaderTy, NodePrimitives, NodeTypes, PrimitivesTy};
use rsil_node_core::node_config::NodeConfig;
use rsil_provider::BlockReader;
use rsil_sila_primitives::SilPrimitives;
use std::fmt::Debug;
use tokio::sync::mpsc;

use crate::{ExExContext, ExExEvent, ExExNotificationsStream};

// TODO(0xurb) - add `node` after abstractions
/// Captures the context that an `ExEx` has access to.
pub struct ExExContextDyn<N: NodePrimitives = SilPrimitives> {
    /// The current head of the blockchain at launch.
    pub head: BlockNumHash,
    /// The config of the node
    pub config: NodeConfig<Box<dyn SilChainSpec<Header = N::BlockHeader> + 'static>>,
    /// The loaded node config
    pub rsil_config: rsil_config::Config,
    /// Channel used to send [`ExExEvent`]s to the rest of the node.
    ///
    /// # Important
    ///
    /// The exex should emit a `FinishedHeight` whenever a processed block is safe to prune.
    /// Additionally, the exex can preemptively emit a `FinishedHeight` event to specify what
    /// blocks to receive notifications for.
    pub events: mpsc::UnboundedSender<ExExEvent>,
    /// Channel to receive [`ExExNotification`](crate::ExExNotification)s.
    ///
    /// # Important
    ///
    /// Once an [`ExExNotification`](crate::ExExNotification) is sent over the channel, it is
    /// considered delivered by the node.
    pub notifications: Box<dyn ExExNotificationsStream<N>>,
}

impl<N: NodePrimitives> Debug for ExExContextDyn<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExExContext")
            .field("head", &self.head)
            .field("config", &self.config)
            .field("rsil_config", &self.rsil_config)
            .field("events", &self.events)
            .field("notifications", &"...")
            .finish()
    }
}

impl<Node> From<ExExContext<Node>> for ExExContextDyn<PrimitivesTy<Node::Types>>
where
    Node: FullNodeComponents<Types: NodeTypes<Primitives: NodePrimitives>>,
    Node::Provider: Debug + BlockReader,
{
    fn from(ctx: ExExContext<Node>) -> Self {
        let config = ctx.config.map_chainspec(|chainspec| {
            Box::new(chainspec) as Box<dyn SilChainSpec<Header = HeaderTy<Node::Types>>>
        });
        let notifications = Box::new(ctx.notifications) as Box<_>;

        Self {
            head: ctx.head,
            config,
            rsil_config: ctx.rsil_config,
            events: ctx.events,
            notifications,
        }
    }
}
