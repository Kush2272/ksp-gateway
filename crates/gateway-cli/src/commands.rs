//! CLI command definitions and dispatch.

use std::{path::PathBuf, sync::Arc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use anyhow::Result;

use gateway_config::load_config;
use gateway_monitor::init_tracing;
use gateway_ksp::SessionManager;
use gateway_monitor::GatewayMetrics;
use gateway_dashboard::DashboardServer;
use gateway_ksp::KspListener;

#[derive(Parser)]
#[command(
    name    = "ksp-gateway",
    version = env!("CARGO_PKG_VERSION"),
    about   = "KSP Gateway — protocol bridge between KSP and HTTPS/WSS",
    long_about = "KSP Gateway translates between the Kush Secure Protocol (KSP) and\n\
                  standard HTTPS/HTTP2/WebSocket, allowing KSP Browser to access\n\
                  any HTTPS website through an encrypted KSP tunnel.",
)]
pub struct Cli {
    /// Path to the TOML configuration file.
    #[arg(short, long, default_value = "config/default.toml", global = true)]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the KSP Gateway
    Start,
    /// Show status of a running gateway instance
    Status,
    /// Run a built-in throughput benchmark
    Bench,
    /// Print the active configuration (with defaults applied)
    Config,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Start  => cmd_start(self.config).await,
            Commands::Status => cmd_status().await,
            Commands::Bench  => cmd_bench().await,
            Commands::Config => cmd_config(self.config).await,
        }
    }
}

// ─── Command implementations ──────────────────────────────────────────────────

async fn cmd_start(config_path: PathBuf) -> Result<()> {
    let cfg = load_config(&config_path)?;
    init_tracing(&cfg.monitor);

    println!(
        "{} {} {}",
        "KSP Gateway".bold().bright_yellow(),
        "v".dimmed(),
        env!("CARGO_PKG_VERSION").bold(),
    );
    println!("{} {}", "KSP listener:".dimmed(), cfg.ksp.listen.bold());
    println!("{} {}", "Dashboard:   ".dimmed(),
        format!("http://localhost:{}", cfg.monitor.prometheus_port).bold());
    println!();

    let session_manager = Arc::new(SessionManager::new());
    let metrics         = Arc::new(GatewayMetrics::default());

    let ksp_addr: std::net::SocketAddr = cfg.ksp.listen.parse()?;

    // Spawn dashboard server
    let dashboard = DashboardServer::new(
        cfg.monitor.prometheus_port,
        Arc::clone(&metrics),
        Arc::clone(&session_manager),
    );
    tokio::spawn(async move {
        if let Err(e) = dashboard.run().await {
            tracing::error!(error = %e, "Dashboard server error");
        }
    });

    // Run KSP listener (blocks until shutdown)
    let listener = KspListener::new(ksp_addr, Arc::clone(&session_manager));
    listener.run().await?;

    Ok(())
}

async fn cmd_status() -> Result<()> {
    println!("{}", "Gateway status: not yet implemented (Milestone 2)".yellow());
    println!("Run 'ksp-gateway start' first, then check http://localhost:9090");
    Ok(())
}

async fn cmd_bench() -> Result<()> {
    println!("{}", "Benchmark: not yet implemented (Milestone 4)".yellow());
    Ok(())
}

async fn cmd_config(config_path: PathBuf) -> Result<()> {
    let cfg = load_config(&config_path)?;
    let json = serde_json::to_string_pretty(&cfg)
        .unwrap_or_else(|_| "Failed to serialize config".into());
    println!("{json}");
    Ok(())
}
