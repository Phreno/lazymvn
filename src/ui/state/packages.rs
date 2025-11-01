//! Package selector for logging configuration

use super::TuiState;

impl TuiState {
    /// Show package selector popup with packages extracted from current output
    pub fn show_package_selector(&mut self) {
        log::info!("Showing package selector");

        // Extract packages from current tab's output
        let tab = self.get_active_tab();
        let log_format = tab.config.logging.as_ref().and_then(|l| l.log_format.as_deref());
        
        self.package_candidates = crate::utils::extract_unique_packages(&tab.command_output, log_format);
        log::debug!("Found {} unique packages in output", self.package_candidates.len());

        self.show_package_selector = true;
        self.package_filter.clear();

        if !self.package_candidates.is_empty() {
            self.packages_list_state.select(Some(0));
        }
    }

    /// Hide package selector popup
    pub fn hide_package_selector(&mut self) {
        log::info!("Hiding package selector");
        self.show_package_selector = false;
        self.package_filter.clear();
    }

    /// Get filtered package candidates based on current filter
    pub fn get_filtered_package_candidates(&self) -> Vec<String> {
        use fuzzy_matcher::FuzzyMatcher;

        if self.package_filter.is_empty() {
            return self.package_candidates.clone();
        }

        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let mut scored: Vec<_> = self
            .package_candidates
            .iter()
            .filter_map(|candidate| {
                matcher
                    .fuzzy_match(candidate, &self.package_filter)
                    .map(|score| (candidate.clone(), score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(candidate, _)| candidate).collect()
    }

    /// Add character to package filter
    pub fn push_package_filter_char(&mut self, ch: char) {
        self.package_filter.push(ch);
        // Reset selection to first match
        if !self.get_filtered_package_candidates().is_empty() {
            self.packages_list_state.select(Some(0));
        }
    }

    /// Remove character from package filter
    pub fn pop_package_filter_char(&mut self) {
        self.package_filter.pop();
        // Reset selection to first match
        if !self.get_filtered_package_candidates().is_empty() {
            self.packages_list_state.select(Some(0));
        }
    }

    /// Navigate to next package in filtered list
    pub fn next_package(&mut self) {
        let candidates = self.get_filtered_package_candidates();
        if candidates.is_empty() {
            return;
        }

        let i = match self.packages_list_state.selected() {
            Some(i) => (i + 1) % candidates.len(),
            None => 0,
        };
        self.packages_list_state.select(Some(i));
    }

    /// Navigate to previous package in filtered list
    pub fn previous_package(&mut self) {
        let candidates = self.get_filtered_package_candidates();
        if candidates.is_empty() {
            return;
        }

        let i = match self.packages_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    candidates.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.packages_list_state.select(Some(i));
    }

    /// Select package and add to project config with INFO level
    pub fn select_and_add_package(&mut self) {
        if let Some(idx) = self.packages_list_state.selected() {
            let candidates = self.get_filtered_package_candidates();
            if let Some(package_name) = candidates.get(idx) {
                log::info!("Adding package to config: {}", package_name);
                
                // Add package to current tab's config
                if self.add_package_to_config(package_name, "INFO") {
                    self.hide_package_selector();
                }
            }
        }
    }

    /// Add a package to the project's logging configuration
    /// Returns true if successful
    fn add_package_to_config(&mut self, package_name: &str, level: &str) -> bool {
        let tab = self.get_active_tab_mut();
        let project_root = tab.project_root.clone();
        
        // Ensure logging config exists
        if tab.config.logging.is_none() {
            tab.config.logging = Some(crate::core::config::LoggingConfig::default());
        }
        
        let logging_config = tab.config.logging.as_mut().unwrap();
        
        // Check if package already exists
        if logging_config.packages.iter().any(|p| p.name == package_name) {
            log::warn!("Package {} already exists in config", package_name);
            tab.command_output = vec![
                format!("Package '{}' is already in the configuration.", package_name),
                "You can edit the level in lazymvn.toml manually.".to_string(),
            ];
            return false;
        }
        
        // Add the package
        logging_config.packages.push(crate::core::config::PackageLogLevel {
            name: package_name.to_string(),
            level: level.to_string(),
        });
        
        // Save the config
        match crate::core::config::save_config(&project_root, &tab.config) {
            Ok(_) => {
                log::info!("Successfully added package {} with level {}", package_name, level);
                tab.command_output = vec![
                    format!("✓ Added package '{}' with level '{}'", package_name, level),
                    "Package has been added to lazymvn.toml".to_string(),
                    "Run the command again to see filtered logs.".to_string(),
                ];
                true
            }
            Err(e) => {
                log::error!("Failed to save config: {}", e);
                tab.command_output = vec![
                    format!("❌ Failed to save configuration: {}", e),
                ];
                false
            }
        }
    }
}
