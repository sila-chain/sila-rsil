//! This example showcases various Nodebuilder use cases

use rsil_sila::{
    cli::interface::Cli,
    node::{builder::components::NoopNetworkBuilder, node::SilaAddOns, SilaNode},
};

/// Maps the sila node's network component to the noop implementation.
///
/// This installs the [`NoopNetworkBuilder`] that does not launch a real network.
pub fn noop_network() {
    Cli::parse_args()
        .run(async move |builder, _| {
            let handle = builder
                // use the default sila node types
                .with_types::<SilaNode>()
                // Configure the components of the node
                // use default sila components but use the Noop network that does nothing but
                .with_components(SilaNode::components().network(NoopNetworkBuilder::sil()))
                .with_add_ons(SilaAddOns::default())
                .launch()
                .await?;

            handle.wait_for_node_exit().await
        })
        .unwrap();
}

fn main() {}
