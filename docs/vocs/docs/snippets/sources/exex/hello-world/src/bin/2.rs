use rsil::api::FullNodeComponents;
use rsil_exex::ExExContext;
use rsil_node_sila::SilaNode;

async fn my_exex<Node: FullNodeComponents>(mut _ctx: ExExContext<Node>) -> eyre::Result<()> {
    #[expect(clippy::empty_loop)]
    loop {}
}

fn main() -> eyre::Result<()> {
    rsil::cli::Cli::parse_args().run(async move |builder, _| {
        let handle = builder
            .node(SilaNode::default())
            .install_exex("my-exex", async move |ctx| Ok(my_exex(ctx)))
            .launch_with_debug_capabilities()
            .await?;

        handle.wait_for_node_exit().await
    })
}
