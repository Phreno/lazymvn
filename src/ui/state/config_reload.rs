//! Configuration reload helpers
//!
//! Helper functions for detecting and applying configuration changes.

use super::TuiState;

impl TuiState {
    /// Returns true if configuration actually changed
    pub fn reload_config(&mut self, new_config: crate::core::config::Config) -> bool {
        log::info!("Reloading configuration");

        let changes = self.detect_config_changes(&new_config);
        let changed = !changes.is_empty();

        if changed {
            self.apply_watch_config_changes(&new_config);
            self.apply_new_config(new_config, &changes);
        }

        changed
    }

    /// Detect all configuration changes
    fn detect_config_changes(&self, new_config: &crate::core::config::Config) -> Vec<String> {
        let tab = self.get_active_tab();
        let mut changes = Vec::new();

        // Check launch_mode
        if tab.config.launch_mode != new_config.launch_mode {
            changes.push(format!(
                "  • Launch mode: {:?} → {:?}",
                tab.config.launch_mode, new_config.launch_mode
            ));
        }

        // Check maven_settings
        if tab.config.maven_settings != new_config.maven_settings {
            changes.push(format!(
                "  • Maven settings: {:?} → {:?}",
                tab.config.maven_settings, new_config.maven_settings
            ));
        }

        // Check notifications
        if tab.config.notifications_enabled != new_config.notifications_enabled {
            changes.push(format!(
                "  • Notifications: {:?} → {:?}",
                tab.config.notifications_enabled, new_config.notifications_enabled
            ));
        }

        // Check watch configuration
        if self.has_watch_config_changed(new_config) {
            changes.push("  • Watch configuration changed".to_string());
        }

        // Check output configuration
        if tab.config.output != new_config.output {
            changes.push("  • Output configuration changed".to_string());
        }

        // Check logging configuration
        if tab.config.logging != new_config.logging {
            changes.push("  • Logging configuration changed".to_string());
        }

        changes
    }

    /// Check if watch configuration has changed
    fn has_watch_config_changed(&self, new_config: &crate::core::config::Config) -> bool {
        let tab = self.get_active_tab();

        match (&tab.config.watch, &new_config.watch) {
            (Some(old), Some(new)) => {
                old.enabled != new.enabled
                    || old.commands != new.commands
                    || old.patterns != new.patterns
                    || old.debounce_ms != new.debounce_ms
            }
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        }
    }

    /// Apply watch configuration changes (reinitialize file watcher)
    fn apply_watch_config_changes(&mut self, new_config: &crate::core::config::Config) {
        if !self.has_watch_config_changed(new_config) {
            return;
        }

        // Clone the project root to avoid borrow checker issues
        let project_root = self.get_active_tab().project_root.clone();

        // Reinitialize file watcher if watch config changed
        if let Some(watch_config) = &new_config.watch {
            if watch_config.enabled {
                self.reinitialize_file_watcher(&project_root, watch_config);
            } else {
                self.disable_file_watcher();
            }
        } else {
            self.disable_file_watcher();
        }
    }

    /// Reinitialize file watcher with new configuration
    fn reinitialize_file_watcher(
        &mut self,
        project_root: &std::path::Path,
        watch_config: &crate::core::config::WatchConfig,
    ) {
        match crate::utils::watcher::FileWatcher::new(project_root, watch_config.debounce_ms) {
            Ok(watcher) => {
                let tab = self.get_active_tab_mut();
                tab.file_watcher = Some(watcher);
                tab.watch_enabled = true;
                log::info!(
                    "File watcher reinitialized with {} patterns",
                    watch_config.patterns.len()
                );
            }
            Err(e) => {
                log::error!("Failed to reinitialize file watcher: {}", e);
                self.disable_file_watcher();
            }
        }
    }

    /// Disable file watcher
    fn disable_file_watcher(&mut self) {
        let tab = self.get_active_tab_mut();
        tab.file_watcher = None;
        tab.watch_enabled = false;
        log::info!("File watcher disabled");
    }

    /// Apply the new configuration and log changes
    fn apply_new_config(&mut self, new_config: crate::core::config::Config, changes: &[String]) {
        let tab = self.get_active_tab_mut();
        tab.config = new_config;

        log::info!("Configuration changes detected:");
        for change in changes {
            log::info!("{}", change);
        }
    }
}
