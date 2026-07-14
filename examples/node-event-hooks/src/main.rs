//! Example for how to hook into the node via the CLI extension mechanism without registering
//! additional arguments
//!
//! Run with
//!
//! ```sh
//! cargo run -p node-event-hooks -- node
//! ```
//!
//! This launches a regular rsil node and also print:
//! > "All components initialized" – once all components have been initialized
//! > "Node started" – once the node has been started.

#![warn(unused_crate_dependencies)]

use rsil_sila::{cli::interface::Cli, node::SilaNode};

fn main() {
    Cli::parse_args()
        .run(async move |builder, _| {
            let handle = builder
                .node(SilaNode::default())
                .on_node_started(|_ctx| {
                    println!("Node started");
                    Ok(())
                })
                .on_rpc_started(|_ctx, _handles| {
                    println!("RPC started");
                    Ok(())
                })
                .on_component_initialized(|_ctx| {
                    println!("All components initialized");
                    Ok(())
                })
                .launch()
                .await?;

            handle.wait_for_node_exit().await
        })
        .unwrap();
}
