//! Plugin registry — resolves plugin names to instances.

use std::{collections::HashMap, sync::Arc};
use super::chain::{GatewayPlugin, PluginChain};

/// A registry that maps plugin names to their instances.
///
/// Used by the config system to build a `PluginChain` from the list of
/// enabled plugin names in `default.toml`.
pub struct PluginRegistry {
    plugins: HashMap<&'static str, Arc<dyn GatewayPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin. Panics in debug mode if the same name is registered twice.
    pub fn register(&mut self, plugin: Arc<dyn GatewayPlugin>) {
        debug_assert!(
            !self.plugins.contains_key(plugin.name()),
            "Duplicate plugin registration: {}",
            plugin.name()
        );
        self.plugins.insert(plugin.name(), plugin);
    }

    /// Build a `PluginChain` from a list of enabled plugin names.
    ///
    /// Unknown plugin names are logged and skipped rather than panicking.
    pub fn build_chain(&self, enabled: &[String]) -> PluginChain {
        let mut selected = Vec::with_capacity(enabled.len());
        for name in enabled {
            match self.plugins.get(name.as_str()) {
                Some(p) => selected.push(Arc::clone(p)),
                None => {
                    tracing::warn!(plugin = %name, "Unknown plugin name — skipping");
                }
            }
        }
        PluginChain::new(selected)
    }

    /// Returns the names of all registered plugins.
    pub fn available(&self) -> Vec<&str> {
        self.plugins.keys().copied().collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
