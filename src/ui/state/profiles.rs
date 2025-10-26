//! Maven profile management
//!
//! This module handles loading, toggling, and displaying Maven profiles.

use super::{TuiState, ProfileLoadingStatus, MavenProfile, Focus};
use crate::maven;
use std::sync::mpsc;
use std::time::{Duration, Instant};

impl TuiState {
    /// Set the list of available Maven profiles
    pub fn set_profiles(&mut self, profile_names: Vec<String>) {
        log::info!("set_profiles: Loading {} profiles", profile_names.len());

        let tab = self.get_active_tab_mut();

        // Get auto-activated profiles
        let auto_activated = maven::get_active_profiles(&tab.project_root).unwrap_or_else(|e| {
            log::warn!("Failed to get active profiles: {}", e);
            vec![]
        });

        log::debug!("Auto-activated profiles: {:?}", auto_activated);

        // Create MavenProfile structs
        tab.profiles = profile_names
            .into_iter()
            .map(|name| {
                let is_auto = auto_activated.contains(&name);
                log::debug!("Profile '{}' auto-activated: {}", name, is_auto);
                MavenProfile::new(name, is_auto)
            })
            .collect();

        if !tab.profiles.is_empty() {
            tab.profiles_list_state.select(Some(0));
        }

        log::info!(
            "Loaded {} profiles ({} auto-activated)",
            tab.profiles.len(),
            auto_activated.len()
        );

        // Load saved preferences for the current module after profiles are created
        self.load_module_preferences();
    }

    /// Toggle the selected profile (enable/disable/default)
    pub fn toggle_profile(&mut self) {
        if self.focus != Focus::Profiles {
            return;
        }
        let tab = self.get_active_tab_mut();
        if let Some(selected) = tab.profiles_list_state.selected()
            && let Some(profile) = tab.profiles.get_mut(selected)
        {
            let old_state = profile.state.clone();
            profile.toggle();
            log::info!(
                "Profile '{}': {:?} → {:?} (auto: {})",
                profile.name,
                old_state,
                profile.state,
                profile.auto_activated
            );

            // Save preferences after toggling
            self.save_module_preferences();
        }
    }

    /// Check for and process any pending profile loading updates
    /// Should be called regularly from the main event loop
    pub fn poll_profiles_updates(&mut self) {
        // Update spinner animation
        if matches!(self.profile_loading_status, ProfileLoadingStatus::Loading) {
            self.profile_spinner_frame = (self.profile_spinner_frame + 1) % 8;
        }

        // Check for timeout (30 seconds)
        if let Some(start_time) = self.profile_loading_start_time
            && start_time.elapsed() > Duration::from_secs(30)
        {
            log::warn!("Profile loading timed out after 30 seconds");
            self.profile_loading_status = ProfileLoadingStatus::Error(
                "Timeout: Profile loading took too long (>30s)".to_string(),
            );
            self.profiles_receiver = None;
            self.profile_loading_start_time = None;
            return;
        }

        if let Some(receiver) = self.profiles_receiver.as_ref() {
            match receiver.try_recv() {
                Ok(Ok(profile_names)) => {
                    log::info!(
                        "Profiles loaded asynchronously: {} profiles",
                        profile_names.len()
                    );
                    self.set_profiles(profile_names);
                    self.profile_loading_status = ProfileLoadingStatus::Loaded;
                    self.profiles_receiver = None;
                    self.profile_loading_start_time = None;
                }
                Ok(Err(error)) => {
                    log::error!("Failed to load profiles: {}", error);
                    self.profile_loading_status = ProfileLoadingStatus::Error(error);
                    self.profiles_receiver = None;
                    self.profile_loading_start_time = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Still loading, nothing to do
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    log::warn!("Profiles channel disconnected unexpectedly");
                    self.profile_loading_status = ProfileLoadingStatus::Error(
                        "Profile loading channel disconnected".to_string(),
                    );
                    self.profiles_receiver = None;
                    self.profile_loading_start_time = None;
                }
            }
        }
    }

    /// Get the current spinner character for profile loading animation
    pub fn profile_loading_spinner(&self) -> &'static str {
        const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        SPINNER_FRAMES[self.profile_spinner_frame % SPINNER_FRAMES.len()]
    }

    /// Start loading profiles asynchronously
    pub fn start_loading_profiles(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.profiles_receiver = Some(rx);
        self.profile_loading_status = ProfileLoadingStatus::Loading;
        self.profile_loading_start_time = Some(Instant::now());
        self.profile_spinner_frame = 0;

        let tab = self.get_active_tab();
        let project_root = tab.project_root.clone();
        let project_root_display = project_root.clone(); // Clone for logging
        
        std::thread::spawn(move || {
            let result = maven::get_profiles(&project_root).map_err(|e| e.to_string());

            if let Err(e) = tx.send(result) {
                log::error!("Failed to send profiles result: {}", e);
            }
        });

        log::info!("Started async profile loading for {:?}", project_root_display);
    }

    /// Sync output to show the selected profile's XML
    pub(super) fn sync_selected_profile_output(&mut self) {
        let tab = self.get_active_tab_mut();
        if let Some(selected) = tab.profiles_list_state.selected() {
            if let Some(profile) = tab.profiles.get(selected) {
                if let Some((xml, pom_path)) =
                    crate::maven::get_profile_xml(&tab.project_root, &profile.name)
                {
                    // Build output with header and XML
                    let relative_path = pom_path
                        .strip_prefix(&tab.project_root)
                        .unwrap_or(&pom_path)
                        .to_string_lossy();

                    let mut output = vec![
                        format!("Profile: {}", profile.name),
                        format!("From: {}", relative_path),
                        String::new(),
                    ];

                    // Add XML lines
                    for line in xml.lines() {
                        output.push(line.to_string());
                    }

                    tab.command_output = output;
                    tab.output_offset = 0;
                } else {
                    tab.command_output = vec![
                        format!("Profile: {}", profile.name),
                        String::new(),
                        "XML not found in POM files.".to_string(),
                    ];
                    tab.output_offset = 0;
                }
            } else {
                tab.command_output = vec!["No profile selected.".to_string()];
                tab.output_offset = 0;
            }
        } else {
            tab.command_output = vec!["No profile selected.".to_string()];
            tab.output_offset = 0;
        }
        tab.output_metrics = None;
        self.clamp_output_offset();
        self.refresh_search_matches();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use std::path::PathBuf;

    fn create_test_state() -> TuiState {
        TuiState::new(
            vec!["module1".to_string()],
            PathBuf::from("/test"),
            Config::default(),
        )
    }

    #[test]
    fn test_set_profiles_empty() {
        let mut state = create_test_state();
        
        state.set_profiles(vec![]);
        
        let tab = state.get_active_tab();
        assert!(tab.profiles.is_empty());
    }

    #[test]
    fn test_set_profiles_with_names() {
        let mut state = create_test_state();
        
        state.set_profiles(vec!["dev".to_string(), "prod".to_string()]);
        
        let tab = state.get_active_tab();
        assert_eq!(tab.profiles.len(), 2);
        assert_eq!(tab.profiles[0].name, "dev");
        assert_eq!(tab.profiles[1].name, "prod");
    }

    #[test]
    fn test_set_profiles_selects_first() {
        let mut state = create_test_state();
        
        state.set_profiles(vec!["dev".to_string()]);
        
        let tab = state.get_active_tab();
        assert_eq!(tab.profiles_list_state.selected(), Some(0));
    }

    #[test]
    fn test_toggle_profile_not_in_profiles_focus() {
        let mut state = create_test_state();
        state.focus = Focus::Modules;
        state.set_profiles(vec!["dev".to_string()]);
        
        let initial_state = state.get_active_tab().profiles[0].state.clone();
        state.toggle_profile();
        
        // Should not toggle when not in Profiles focus
        assert_eq!(state.get_active_tab().profiles[0].state, initial_state);
    }

    #[test]
    fn test_toggle_profile_no_selection() {
        let mut state = create_test_state();
        state.focus = Focus::Profiles;
        state.set_profiles(vec!["dev".to_string()]);
        
        {
            let tab = state.get_active_tab_mut();
            tab.profiles_list_state.select(None);
        }
        
        state.toggle_profile();
        
        // Should not panic with no selection
    }

    #[test]
    fn test_profile_loading_spinner_frames() {
        let mut state = create_test_state();
        
        let frame1 = state.profile_loading_spinner();
        state.profile_spinner_frame = 1;
        let frame2 = state.profile_loading_spinner();
        
        assert_ne!(frame1, frame2);
    }

    #[test]
    fn test_profile_loading_spinner_wraps() {
        let mut state = create_test_state();
        state.profile_spinner_frame = 0;
        let first = state.profile_loading_spinner();
        
        state.profile_spinner_frame = 8; // Should wrap
        let wrapped = state.profile_loading_spinner();
        
        assert_eq!(first, wrapped);
    }

    #[test]
    fn test_start_loading_profiles() {
        let mut state = create_test_state();
        
        state.start_loading_profiles();
        
        assert!(state.profiles_receiver.is_some());
        assert!(matches!(state.profile_loading_status, ProfileLoadingStatus::Loading));
        assert!(state.profile_loading_start_time.is_some());
        assert_eq!(state.profile_spinner_frame, 0);
    }

    #[test]
    fn test_poll_profiles_updates_advances_spinner() {
        let mut state = create_test_state();
        state.profile_loading_status = ProfileLoadingStatus::Loading;
        state.profile_spinner_frame = 0;
        
        state.poll_profiles_updates();
        
        assert_eq!(state.profile_spinner_frame, 1);
    }

    #[test]
    fn test_poll_profiles_updates_no_spinner_when_not_loading() {
        let mut state = create_test_state();
        state.profile_loading_status = ProfileLoadingStatus::Loaded;
        state.profile_spinner_frame = 0;
        
        state.poll_profiles_updates();
        
        assert_eq!(state.profile_spinner_frame, 0);
    }

    #[test]
    fn test_sync_selected_profile_output_no_selection() {
        let mut state = create_test_state();
        state.set_profiles(vec!["dev".to_string()]);
        
        {
            let tab = state.get_active_tab_mut();
            tab.profiles_list_state.select(None);
        }
        
        state.sync_selected_profile_output();
        
        let tab = state.get_active_tab();
        assert_eq!(tab.command_output.len(), 1);
        assert_eq!(tab.command_output[0], "No profile selected.");
    }

    #[test]
    fn test_sync_selected_profile_output_with_selection() {
        let mut state = create_test_state();
        state.set_profiles(vec!["dev".to_string()]);
        
        {
            let tab = state.get_active_tab_mut();
            tab.profiles_list_state.select(Some(0));
        }
        
        state.sync_selected_profile_output();
        
        // Should generate output (may not find XML but should not panic)
        let tab = state.get_active_tab();
        assert!(!tab.command_output.is_empty());
    }

    #[test]
    fn test_sync_selected_profile_output_resets_offset() {
        let mut state = create_test_state();
        state.set_profiles(vec!["dev".to_string()]);
        
        {
            let tab = state.get_active_tab_mut();
            tab.profiles_list_state.select(Some(0));
            tab.output_offset = 100;
        }
        
        state.sync_selected_profile_output();
        
        let tab = state.get_active_tab();
        assert_eq!(tab.output_offset, 0);
    }

    #[test]
    fn test_sync_selected_profile_output_clears_metrics() {
        let mut state = create_test_state();
        state.set_profiles(vec!["dev".to_string()]);
        
        {
            let tab = state.get_active_tab_mut();
            tab.profiles_list_state.select(Some(0));
            tab.output_metrics = Some(crate::ui::state::OutputMetrics::new(80, &[]));
        }
        
        state.sync_selected_profile_output();
        
        let tab = state.get_active_tab();
        assert!(tab.output_metrics.is_none());
    }
}

