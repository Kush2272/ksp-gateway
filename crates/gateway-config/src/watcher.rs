//! File-system watcher for live config reload.
//!
//! When the config file changes on disk, the watcher sends the new
//! `GatewayConfig` through the provided `tokio::sync::watch` channel.
//! All gateway components that hold a `watch::Receiver<GatewayConfig>`
//! automatically pick up the new config without restarting.

use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::watch;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use gateway_core::error::GatewayResult;
use crate::{loader::load_config, schema::GatewayConfig};

/// Spawn a background task that watches `config_path` for changes.
///
/// Returns a `watch::Receiver<Arc<GatewayConfig>>` that always holds the
/// most recently loaded valid configuration. Invalid configs after reload
/// are logged and discarded — the last valid config remains active.
pub fn watch_config(
    config_path: PathBuf,
) -> GatewayResult<watch::Receiver<Arc<GatewayConfig>>> {
    // Load initial config.
    let initial = Arc::new(load_config(&config_path)?);
    let (tx, rx) = watch::channel(initial);

    let path = config_path.clone();
    tokio::spawn(async move {
        // Use a channel to bridge the notify callback (sync) into async.
        let (fs_tx, mut fs_rx) = tokio::sync::mpsc::channel::<()>(4);

        let mut watcher: RecommendedWatcher = {
            let fs_tx = fs_tx.clone();
            match notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                if let Ok(event) = result {
                    use notify::EventKind::*;
                    if matches!(event.kind, Modify(_) | Create(_)) {
                        let _ = fs_tx.try_send(());
                    }
                }
            }) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to create config file watcher");
                    return;
                }
            }
        };

        if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
            tracing::error!(error = %e, "Failed to start watching config file");
            return;
        }

        tracing::info!(path = %path.display(), "Watching config file for changes");

        while fs_rx.recv().await.is_some() {
            // Debounce — wait a short time for file writes to complete.
            tokio::time::sleep(Duration::from_millis(200)).await;
            // Drain any additional events.
            while fs_rx.try_recv().is_ok() {}

            match load_config(&path) {
                Ok(new_cfg) => {
                    tracing::info!("Config reloaded successfully");
                    let _ = tx.send(Arc::new(new_cfg));
                }
                Err(e) => {
                    tracing::error!(error = %e, "Config reload failed — keeping previous config");
                }
            }
        }
    });

    Ok(rx)
}
