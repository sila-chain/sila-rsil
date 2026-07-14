use rsil_node_sila::SilaNode;

fn main() -> eyre::Result<()> {
    rsil::cli::Cli::parse_args().run(async move |builder, _| {
        let handle = builder.node(SilaNode::default()).launch_with_debug_capabilities().await?;

        handle.wait_for_node_exit().await
    })
}
