#![allow(missing_docs)]

#[global_allocator]
static ALLOC: rsil_cli_util::allocator::Allocator = rsil_cli_util::allocator::new_allocator();

// Required for "override_allocator_on_supported_platforms".
#[cfg(all(feature = "jemalloc", unix))]
use rsil_cli_util::allocator::tikv_jemalloc_sys as _;

#[cfg(all(feature = "jemalloc-prof", unix))]
#[unsafe(export_name = "malloc_conf")]
static MALLOC_CONF: &[u8] = b"prof:true,prof_active:true,lg_prof_sample:19\0";

use clap::Parser;
use rsil::cli::Cli;
use rsil_sila_cli::chainspec::SilaChainSpecParser;
use rsil_node_sila::SilaNode;
use tracing::info;

fn main() {
    #[cfg(feature = "jit")]
    {
        match rsil_node_sila::node::maybe_run_jit_helper() {
            Ok(std::ops::ControlFlow::Break(())) => return,
            Ok(std::ops::ControlFlow::Continue(())) => {}
            Err(err) => {
                eprintln!("Error: {err:?}");
                std::process::exit(1);
            }
        }
    }

    rsil_cli_util::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    if let Err(err) = Cli::<SilaChainSpecParser>::parse().run(async move |builder, _| {
        info!(target: "rsil::cli", "Launching node");
        let handle = builder.node(SilaNode::default()).launch_with_debug_capabilities().await?;

        handle.wait_for_node_exit().await
    }) {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
