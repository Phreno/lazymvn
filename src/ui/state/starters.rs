//! Spring Boot starters management for TUI

use super::TuiState;

impl TuiState {
    pub fn show_starter_selector(&mut self) {
        log::info!("Showing starter selector");

        // Always refresh starters for the active tab (in case we switched tabs)
        let tab = self.get_active_tab();
        self.starter_candidates =
            crate::features::starters::find_potential_starters(&tab.project_root);
        log::debug!("Found {} potential starters for current tab", self.starter_candidates.len());

        self.show_starter_selector = true;
        self.starter_filter.clear();

        if !self.starter_candidates.is_empty() {
            self.starters_list_state.select(Some(0));
        }
    }

    pub fn hide_starter_selector(&mut self) {
        log::info!("Hiding starter selector");
        self.show_starter_selector = false;
        self.starter_filter.clear();
    }

    pub fn show_starter_manager(&mut self) {
        log::info!("Showing starter manager");
        self.show_starter_manager = true;

        let tab = self.get_active_tab();
        if !tab.starters_cache.starters.is_empty() {
            self.starters_list_state.select(Some(0));
        }
    }

    pub fn hide_starter_manager(&mut self) {
        log::info!("Hiding starter manager");
        self.show_starter_manager = false;
    }

    pub fn select_and_run_starter(&mut self) {
        if let Some(idx) = self.starters_list_state.selected() {
            let filtered = self.get_filtered_starter_candidates();

            if let Some(fqcn) = filtered.get(idx) {
                log::info!("Selected starter: {}", fqcn);
                self.run_spring_boot_starter(fqcn);
                self.hide_starter_selector();
            }
        }
    }

    pub fn run_spring_boot_starter(&mut self, fqcn: &str) {
        log::info!("Running Spring Boot starter: {}", fqcn);
        let tab = self.get_active_tab_mut();

        // Check if already in cache by iterating through starters
        let already_cached = tab.starters_cache.starters.iter()
            .any(|s| s.fully_qualified_class_name == fqcn);
        
        if !already_cached {
            // Add to cache
            let label = fqcn.split('.').last().unwrap_or(fqcn).to_string();
            tab.starters_cache.add_starter(crate::features::starters::Starter {
                fully_qualified_class_name: fqcn.to_string(),
                label,
                is_default: false,
            });
            let project_root = tab.project_root.clone();
            if let Err(e) = tab.starters_cache.save(&project_root) {
                log::error!("Failed to save starters cache: {}", e);
            }
        }

        // Find the main class name from FQCN
        let main_class = fqcn
            .split('.')
            .last()
            .unwrap_or(fqcn);

        // Build command: mvn spring-boot:run -Dspring-boot.run.main-class=<FQCN>
        let spring_boot_arg = format!("-Dspring-boot.run.main-class={}", fqcn);
        let args = vec![
            "spring-boot:run",
            &spring_boot_arg,
        ];

        log::info!(
            "Launching starter {} with command: mvn {}",
            main_class,
            args.join(" ")
        );

        // Use the existing command execution method
        self.run_selected_module_command_with_options(&args, true);
    }

    pub fn run_preferred_starter(&mut self) {
        // Always show selector popup when 's' is pressed
        log::info!("Showing starter selector");
        self.show_starter_selector();
    }

    pub fn get_filtered_starter_candidates(&self) -> Vec<String> {
        use fuzzy_matcher::FuzzyMatcher;

        if self.starter_filter.is_empty() {
            return self.starter_candidates.clone();
        }

        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let mut scored: Vec<_> = self
            .starter_candidates
            .iter()
            .filter_map(|candidate| {
                matcher
                    .fuzzy_match(candidate, &self.starter_filter)
                    .map(|score| (candidate.clone(), score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(candidate, _)| candidate).collect()
    }

    pub fn push_starter_filter_char(&mut self, ch: char) {
        self.starter_filter.push(ch);
        // Reset selection to first match
        if !self.get_filtered_starter_candidates().is_empty() {
            self.starters_list_state.select(Some(0));
        }
    }

    pub fn pop_starter_filter_char(&mut self) {
        self.starter_filter.pop();
        // Reset selection to first match
        if !self.get_filtered_starter_candidates().is_empty() {
            self.starters_list_state.select(Some(0));
        }
    }

    pub fn next_starter(&mut self) {
        let candidates = if self.show_starter_selector {
            self.get_filtered_starter_candidates()
        } else {
            let tab = self.get_active_tab();
            tab.starters_cache
                .starters
                .iter()
                .map(|s| s.fully_qualified_class_name.clone())
                .collect()
        };

        if candidates.is_empty() {
            return;
        }

        let i = match self.starters_list_state.selected() {
            Some(i) => (i + 1) % candidates.len(),
            None => 0,
        };
        self.starters_list_state.select(Some(i));
    }

    pub fn previous_starter(&mut self) {
        let candidates = if self.show_starter_selector {
            self.get_filtered_starter_candidates()
        } else {
            let tab = self.get_active_tab();
            tab.starters_cache
                .starters
                .iter()
                .map(|s| s.fully_qualified_class_name.clone())
                .collect()
        };

        if candidates.is_empty() {
            return;
        }

        let i = match self.starters_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    candidates.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.starters_list_state.select(Some(i));
    }

    pub fn toggle_starter_default(&mut self) {
        if let Some(idx) = self.starters_list_state.selected() {
            let tab = self.get_active_tab_mut();
            if let Some(starter) = tab.starters_cache.starters.get(idx) {
                let fqcn = starter.fully_qualified_class_name.clone();
                let project_root = tab.project_root.clone();
                tab.starters_cache.set_default(&fqcn);

                if let Err(e) = tab.starters_cache.save(&project_root) {
                    log::error!("Failed to save starters cache: {}", e);
                }
            }
        }
    }

    pub fn remove_selected_starter(&mut self) {
        if let Some(idx) = self.starters_list_state.selected() {
            // First, try to remove the starter and get the resulting state
            let (removed, new_len) = {
                let tab = self.get_active_tab_mut();
                if let Some(starter) = tab.starters_cache.starters.get(idx) {
                    let fqcn = starter.fully_qualified_class_name.clone();
                    let project_root = tab.project_root.clone();
                    let removed = tab.starters_cache.remove_starter(&fqcn);

                    if removed {
                        log::info!("Removed starter: {}", fqcn);
                        if let Err(e) = tab.starters_cache.save(&project_root) {
                            log::error!("Failed to save starters cache: {}", e);
                        }
                    }

                    (removed, tab.starters_cache.starters.len())
                } else {
                    (false, 0)
                }
            };

            // Now adjust selection without holding a borrow to tab
            if removed {
                if new_len == 0 {
                    self.starters_list_state.select(None);
                } else if idx >= new_len {
                    self.starters_list_state.select(Some(new_len - 1));
                }
            }
        }
    }
}
