use crate::{ExExContextDyn, ExExEvent, ExExNotifications, ExExNotificationsStream};
use alloy_eips::BlockNumHash;
use rsil_exex_types::ExExHead;
use rsil_node_api::{FullNodeComponents, NodePrimitives, NodeTypes, PrimitivesTy};
use rsil_node_core::node_config::NodeConfig;
use rsil_payload_builder::PayloadBuilderHandle;
use rsil_provider::BlockReader;
use rsil_tasks::TaskExecutor;
use std::fmt::Debug;
use tokio::sync::mpsc::{error::SendError, UnboundedSender};

/// Captures the context that an `ExEx` has access to.
///
/// This type wraps various node components that the `ExEx` has access to.
pub struct ExExContext<Node: FullNodeComponents> {
    /// The current head of the blockchain at launch.
    pub head: BlockNumHash,
    /// The config of the node
    pub config: NodeConfig<<Node::Types as NodeTypes>::ChainSpec>,
    /// The loaded node config
    pub rsil_config: rsil_config::Config,
    /// Channel used to send [`ExExEvent`]s to the rest of the node.
    ///
    /// # Important
    ///
    /// The exex should emit a `FinishedHeight` whenever a processed block is safe to prune.
    /// Additionally, the exex can preemptively emit a `FinishedHeight` event to specify what
    /// blocks to receive notifications for.
    pub events: UnboundedSender<ExExEvent>,
    /// Channel to receive [`ExExNotification`](crate::ExExNotification)s.
    ///
    /// # Important
    ///
    /// Once an [`ExExNotification`](crate::ExExNotification) is sent over the channel, it is
    /// considered delivered by the node.
    pub notifications: ExExNotifications<Node::Provider, Node::Savm>,

    /// Node components
    pub components: Node,
}

impl<Node> Debug for ExExContext<Node>
where
    Node: FullNodeComponents,
    Node::Provider: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExExContext")
            .field("head", &self.head)
            .field("config", &self.config)
            .field("rsil_config", &self.rsil_config)
            .field("events", &self.events)
            .field("notifications", &self.notifications)
            .field("components", &"...")
            .finish()
    }
}

impl<Node> ExExContext<Node>
where
    Node: FullNodeComponents,
    Node::Provider: Debug + BlockReader,
    Node::Types: NodeTypes<Primitives: NodePrimitives>,
{
    /// Returns dynamic version of the context
    pub fn into_dyn(self) -> ExExContextDyn<PrimitivesTy<Node::Types>> {
        ExExContextDyn::from(self)
    }
}

impl<Node> ExExContext<Node>
where
    Node: FullNodeComponents,
    Node::Types: NodeTypes<Primitives: NodePrimitives>,
{
    /// Returns the transaction pool of the node.
    pub fn pool(&self) -> &Node::Pool {
        self.components.pool()
    }

    /// Returns the node's savm config.
    pub fn evm_config(&self) -> &Node::Savm {
        self.components.evm_config()
    }

    /// Returns the provider of the node.
    pub fn provider(&self) -> &Node::Provider {
        self.components.provider()
    }

    /// Returns the handle to the network
    pub fn network(&self) -> &Node::Network {
        self.components.network()
    }

    /// Returns the handle to the payload builder service.
    pub fn payload_builder_handle(
        &self,
    ) -> &PayloadBuilderHandle<<Node::Types as NodeTypes>::Payload> {
        self.components.payload_builder_handle()
    }

    /// Returns the task executor.
    ///
    /// This type should be used to spawn (critical) tasks.
    pub fn task_executor(&self) -> &TaskExecutor {
        self.components.task_executor()
    }

    /// Sets notifications stream to [`crate::ExExNotificationsWithoutHead`], a stream of
    /// notifications without a head.
    pub fn set_notifications_without_head(&mut self) {
        self.notifications.set_without_head();
    }

    /// Sets notifications stream to [`crate::ExExNotificationsWithHead`], a stream of notifications
    /// with the provided head.
    pub fn set_notifications_with_head(&mut self, head: ExExHead) {
        self.notifications.set_with_head(head);
    }

    /// Sends an [`ExExEvent::FinishedHeight`] to the ExEx task manager letting it know that this
    /// ExEx has processed the corresponding block.
    ///
    /// Returns an error if the channel was closed (ExEx task manager panicked).
    pub fn send_finished_height(
        &self,
        height: BlockNumHash,
    ) -> Result<(), SendError<BlockNumHash>> {
        self.events.send(ExExEvent::FinishedHeight(height)).map_err(|_| SendError(height))
    }
}

#[cfg(test)]
mod tests {
    use crate::ExExContext;
    use rsil_exex_types::ExExHead;
    use rsil_node_api::FullNodeComponents;
    use rsil_provider::BlockReader;

    /// <https://github.com/sila-chain/sila-rsil/issues/12054>
    #[test]
    const fn issue_12054() {
        #[expect(dead_code)]
        struct ExEx<Node: FullNodeComponents> {
            ctx: ExExContext<Node>,
        }

        impl<Node: FullNodeComponents> ExEx<Node>
        where
            Node::Provider: BlockReader,
        {
            async fn _test_bounds(mut self) -> eyre::Result<()> {
                self.ctx.pool();
                self.ctx.evm_config();
                self.ctx.provider();
                self.ctx.network();
                self.ctx.payload_builder_handle();
                self.ctx.task_executor();
                self.ctx.set_notifications_without_head();
                self.ctx.set_notifications_with_head(ExExHead { block: Default::default() });
                Ok(())
            }
        }
    }
}
