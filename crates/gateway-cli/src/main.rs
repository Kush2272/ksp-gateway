//! KSP Gateway — CLI entrypoint.
//!
//! ```
//! ksp-gateway start   Start the gateway
//! ksp-gateway stop    Send graceful shutdown signal
//! ksp-gateway status  Show active sessions and stats
//! ksp-gateway reload  Hot-reload configuration
//! ksp-gateway bench   Run built-in throughput benchmark
//! ```

mod commands;

use std::process;
use clap::Parser;
use colored::Colorize;
use commands::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.run().await {
        eprintln!("{} {e}", "error:".red().bold());
        process::exit(1);
    }
}
