//! TUI State management
//!
//! This module manages the state of the terminal UI including module selection,
//! profiles, flags, command execution, and output display.

// Sub-modules
mod commands;
mod config_reload;
mod custom_goals;
mod favorites;
mod flags;
mod help;
mod history;
mod launcher_config;
mod navigation;
mod output;
mod packages;
mod preferences_io;
mod profiles;
mod project_tab;
mod projects;
mod search;
mod starters;
mod tabs;
mod types;
mod utilities;

pub use project_tab::ProjectTab;

// Re-export types
pub use types::{
    BuildFlag, MavenProfile, ModuleOutput, OutputMetrics, ProfileLoadingStatus, ProfileState,
};

use crate::maven;
use crate::ui::keybindings::{CurrentView, Focus, SearchMode};
use crate::ui::search::{SearchMatch, SearchState};
use ratatui::widgets::ListState;
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

/// Main state structure for the TUI
pub struct TuiState {
    // Tabs management
    tabs: Vec<ProjectTab>,
    active_tab_index: usize,
    next_tab_id: usize,

    // Global UI state
    pub current_view: CurrentView,
    pub focus: Focus,

    // Search state (operates on active tab)
    search_state: Option<SearchState>,
    search_input: Option<String>,
    search_history: Vec<String>,
    search_history_index: Option<usize>,
    search_error: Option<String>,
    pending_center: Option<SearchMatch>,
    pub search_mod: Option<SearchMode>,

    // Debouncing for navigation keys
    last_nav_key_time: Option<Instant>,
    nav_debounce_duration: Duration,

    // Async profile loading (for active tab)
    profiles_receiver: Option<mpsc::Receiver<Result<Vec<String>, String>>>,
    pub profile_loading_status: ProfileLoadingStatus,
    profile_loading_start_time: Option<Instant>,
    profile_spinner_frame: usize,

    // Recent projects (global)
    pub recent_projects: Vec<PathBuf>,
    pub projects_list_state: ListState,
    pub show_projects_popup: bool,
    pub projects_filter: String,

    // Spring Boot starters UI (global UI state, cache is per-tab)
    pub show_starter_selector: bool,
    pub show_starter_manager: bool,
    pub starter_candidates: Vec<String>,
    pub starter_filter: String,
    pub starters_list_state: ListState,

    // Package selector UI (global UI state)
    pub show_package_selector: bool,
    pub package_candidates: Vec<String>,
    pub package_filter: String,
    pub packages_list_state: ListState,

    // Custom goals popup (global UI state)
    pub show_custom_goals_popup: bool,

    // Clipboard - keep it alive to prevent "dropped too quickly" errors (global)
    clipboard: Option<arboard::Clipboard>,

    // Command history (global)
    pub command_history: crate::features::history::CommandHistory,
    pub show_history_popup: bool,
    pub history_list_state: ListState,
    pub history_filter: String,

    // Favorites (global)
    pub favorites: crate::features::favorites::Favorites,
    pub show_favorites_popup: bool,
    pub favorites_list_state: ListState,
    pub favorites_filter: String,
    pub show_save_favorite_popup: bool,
    pub favorite_name_input: String,
    pub pending_favorite: Option<crate::features::history::HistoryEntry>,

    // Help popup (global)
    pub show_help_popup: bool,
    pub help_list_state: ListState,
    pub help_search_query: String,

    // Editor command to execute (global)
    pub editor_command: Option<(String, String)>,
}

impl TuiState {
    /// Legacy constructor - creates state with tabs system and initial project tab
    /// This is a compatibility wrapper for the old API
    pub fn new(
        modules: Vec<String>,
        project_root: PathBuf,
        config: crate::core::config::Config,
    ) -> Self {
        // Create empty state with tabs system
        let mut state = Self::new_with_tabs();

        // Create initial tab with the provided project
        let tab = ProjectTab::new(1, project_root, modules, config);
        state.tabs.push(tab);
        state.next_tab_id = 2;
        state.active_tab_index = 0;

        // Load preferences for the initially selected module
        state.load_module_preferences();

        state
    }

    /// Create a new TuiState with tabs system
    pub fn new_with_tabs() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_index: 0,
            next_tab_id: 1,

            current_view: CurrentView::Modules,
            focus: Focus::Modules,

            search_state: None,
            search_input: None,
            search_history: Vec::new(),
            search_history_index: None,
            search_error: None,
            pending_center: None,
            search_mod: None,

            last_nav_key_time: None,
            nav_debounce_duration: Duration::from_millis(50),

            profiles_receiver: None,
            profile_loading_status: ProfileLoadingStatus::Loaded,
            profile_loading_start_time: None,
            profile_spinner_frame: 0,

            recent_projects: crate::core::config::RecentProjects::load().get_projects(),
            projects_list_state: ListState::default(),
            show_projects_popup: false,
            projects_filter: String::new(),

            show_starter_selector: false,
            show_starter_manager: false,
            starter_candidates: Vec::new(),
            starter_filter: String::new(),
            starters_list_state: ListState::default(),

            show_package_selector: false,
            package_candidates: Vec::new(),
            package_filter: String::new(),
            packages_list_state: ListState::default(),

            show_custom_goals_popup: false,

            clipboard: arboard::Clipboard::new().ok(),

            command_history: crate::features::history::CommandHistory::load(),
            show_history_popup: false,
            history_list_state: ListState::default(),
            history_filter: String::new(),

            favorites: crate::features::favorites::Favorites::load(),
            show_favorites_popup: false,
            favorites_list_state: ListState::default(),
            favorites_filter: String::new(),
            show_save_favorite_popup: false,
            favorite_name_input: String::new(),
            pending_favorite: None,

            show_help_popup: false,
            help_list_state: ListState::default(),
            help_search_query: String::new(),

            editor_command: None,
        }
    }

    // Live search - performs search as user types without storing in history

    // Module output management

    /// Kill the currently running Maven process
    pub fn kill_running_process(&mut self) {
        self.terminate_running_process("Stopping process at user request", |pid| {
            format!("⚠ Process {pid} killed by user")
        });
    }

    fn terminate_running_process<F>(&mut self, log_context: &str, message_fn: F) -> bool
    where
        F: FnOnce(u32) -> String,
    {
        let mut killed = false;
        let mut pending_reload = false;

        {
            let tab = self.get_active_tab_mut();
            let Some(pid) = tab.running_process_pid else {
                log::warn!("No running process to terminate ({})", log_context);
                return false;
            };

            log::info!("{} (PID: {})", log_context, pid);
            match maven::kill_process(pid) {
                Ok(()) => {
                    tab.command_output.push(String::new());
                    tab.command_output.push(message_fn(pid));
                    tab.is_command_running = false;
                    tab.command_receiver = None;
                    tab.running_process_pid = None;
                    tab.output_metrics = None;
                    killed = true;
                    pending_reload = tab.pending_watch_rerun;
                }
                Err(e) => {
                    log::error!("Failed to kill process {}: {}", pid, e);
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push(format!("✗ Failed to stop process {}: {}", pid, e));
                }
            }
        }

        if killed {
            if pending_reload {
                let tab = self.get_active_tab_mut();
                tab.pending_watch_rerun = false;
            }
            self.store_current_module_output();
        }

        killed
    }

    /// Check file watcher and re-run command if files changed
    pub fn check_file_watcher(&mut self) {
        let watch_config = match self.get_active_tab().config.watch.clone() {
            Some(cfg) if cfg.enabled => cfg,
            _ => return,
        };

        let mut rerun_args: Option<Vec<String>> = None;
        let mut restart_running_command = false;

        {
            let tab = self.get_active_tab_mut();

            if tab.pending_watch_rerun && !tab.is_command_running {
                if let Some(last_cmd) = tab.last_command.clone() {
                    tab.pending_watch_rerun = false;
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push("🔄 Files changed, reloading...".to_string());
                    tab.command_output.push(String::new());
                    rerun_args = Some(last_cmd);
                } else {
                    log::warn!("Pending reload flagged but no last command recorded");
                    tab.pending_watch_rerun = false;
                }
            }
        }

        if let Some(args) = rerun_args.take() {
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            self.run_selected_module_command(&arg_refs);
            return;
        }

        {
            let tab = self.get_active_tab_mut();

            let Some(watcher) = tab.file_watcher.as_mut() else {
                return;
            };

            if !watcher.check_changes() {
                return;
            }

            log::info!("File changes detected, evaluating auto-reload");

            let Some(last_cmd) = tab.last_command.clone() else {
                log::warn!("File changes detected but no command to rerun");
                return;
            };

            let should_rerun = Self::command_matches_watch_list(&last_cmd, &watch_config);

            if !should_rerun {
                log::debug!("Last command is not configured for auto-reload, skipping rerun");
                return;
            }

            if tab.is_command_running {
                tab.command_output.push(String::new());
                tab.command_output
                    .push("🔁 Files changed, restarting command...".to_string());
                tab.command_output.push(String::new());
                tab.pending_watch_rerun = false;
                restart_running_command = true;
                rerun_args = Some(last_cmd);
            } else {
                tab.command_output.push(String::new());
                tab.command_output
                    .push("🔄 Files changed, reloading...".to_string());
                tab.command_output.push(String::new());
                tab.pending_watch_rerun = false;
                rerun_args = Some(last_cmd);
            }
        }

        if restart_running_command
            && !self.terminate_running_process("Stopping running command for auto-reload", |pid| {
                format!("🔁 Process {pid} stopped for auto-reload")
            })
        {
            let tab = self.get_active_tab_mut();
            tab.pending_watch_rerun = true;
            return;
        }

        if let Some(args) = rerun_args {
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            self.run_selected_module_command(&arg_refs);
        }
    }

    fn command_matches_watch_list(
        last_cmd: &[String],
        watch_config: &crate::core::config::WatchConfig,
    ) -> bool {
        last_cmd.iter().any(|arg| {
            let arg_lower = arg.to_ascii_lowercase();
            watch_config
                .commands
                .iter()
                .any(|cmd| Self::watch_term_matches(&arg_lower, cmd))
        })
    }

    fn watch_term_matches(arg_lower: &str, term: &str) -> bool {
        let term_lower = term.to_ascii_lowercase();
        if arg_lower.contains(&term_lower) {
            return true;
        }

        match term_lower.as_str() {
            // Treat "start" / "run" aliases as the common Maven launch goals
            "start" | "run" => {
                arg_lower.contains("spring-boot:run") || arg_lower.contains("exec:java")
            }
            _ => false,
        }
    }

    // Debug, clipboard, notification, and config editing methods moved to utilities.rs

    // Output display and metrics

    pub fn command_elapsed_seconds(&self) -> Option<u64> {
        let tab = self.get_active_tab();
        tab.command_start_time
            .map(|start| start.elapsed().as_secs())
    }

    // Scrolling methods

    fn total_display_rows(&self) -> usize {
        let tab = self.get_active_tab();
        if let Some(metrics) = tab.output_metrics.as_ref() {
            metrics.total_rows()
        } else {
            tab.command_output.len()
        }
    }

    // Search functionality
    // Recent projects methods moved to projects.rs

    /// Refresh caches (profiles and starters) by forcing a reload
    pub fn refresh_caches(&mut self) {
        log::info!("Refreshing caches (profiles and starters)");
        
        // Get project root before any mutable borrows
        let project_root = self.get_active_tab().project_root.clone();
        
        // Refresh profiles by reloading from Maven
        self.reload_profiles_from_maven();
        
        // Refresh starters by rescanning dependencies
        let tab = self.get_active_tab_mut();
        tab.starters_cache = crate::features::starters::StartersCache::rescan(&project_root);
        log::info!("Starters cache refreshed successfully");

        // Show confirmation message
        tab.command_output = vec![
            "🔄 Caches refreshed successfully".to_string(),
            String::new(),
            "✅ Maven profiles reloaded".to_string(),
            "✅ Spring Boot starters rescanned".to_string(),
        ];
    }

    /// Cleanup resources and kill any running Maven processes
    /// This should be called before the application exits
    pub fn cleanup(&mut self) {
        log::info!("Cleaning up application resources");

        let tab = self.get_active_tab_mut();

        // Kill any running Maven process
        if let Some(pid) = tab.running_process_pid {
            log::info!("Killing running Maven process with PID: {}", pid);
            match crate::maven::kill_process(pid) {
                Ok(()) => {
                    log::info!("Successfully killed Maven process {}", pid);
                }
                Err(e) => {
                    log::error!("Failed to kill Maven process {}: {}", pid, e);
                }
            }
            tab.running_process_pid = None;
            tab.is_command_running = false;
        }

        // Save module preferences
        if let Err(e) = tab.module_preferences.save(&tab.project_root) {
            log::error!("Failed to save module preferences: {}", e);
        }

        log::info!("Cleanup completed");
    }

    // History and favorites methods moved to history.rs and favorites.rs

    // Starters, packages, and custom goals methods moved to their respective modules

    // Module preferences methods

    /// Save current profiles and flags for the selected module
    pub fn save_module_preferences(&mut self) {
        let module = self.selected_module().map(|m| m.to_string());
        let enabled_flags = self.enabled_flag_names();
        let tab = self.get_active_tab_mut();
        preferences_io::save_module_preferences(tab, module.as_deref(), enabled_flags);
    }

    /// Load preferences for the selected module
    pub fn load_module_preferences(&mut self) {
        let module = self.selected_module().map(|m| m.to_string());
        let tab = self.get_active_tab_mut();
        preferences_io::load_module_preferences(tab, module.as_deref());
    }
}

// Helper functions

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_loading_status_initial_state() {
        let config = crate::core::config::Config::default();
        let mut state = TuiState::new(
            vec!["test-module".to_string()],
            PathBuf::from("/tmp/test"),
            config,
        );

        // Create a fake cache to avoid spawning thread that calls Maven
        let cache = crate::core::config::ProfilesCache {
            profiles: vec!["dev".to_string()],
            auto_activated: vec![],
        };
        let _ = cache.save(&state.get_active_tab().project_root);

        // Start loading profiles (should load from cache immediately)
        state.start_loading_profiles();

        // Profiles should be loaded from cache
        assert!(matches!(
            state.profile_loading_status,
            ProfileLoadingStatus::Loaded
        ));
        assert_eq!(state.get_active_tab().profiles.len(), 1);
        
        // Cleanup cache
        let _ = crate::core::config::ProfilesCache::invalidate(&state.get_active_tab().project_root);
    }

    #[test]
    fn test_profile_loading_spinner_frames() {
        let config = crate::core::config::Config::default();
        let mut state = TuiState::new(
            vec!["test-module".to_string()],
            PathBuf::from("/tmp/test"),
            config,
        );

        // Test spinner cycles through frames
        let frame1 = state.profile_loading_spinner();
        state.profile_spinner_frame = 1;
        let frame2 = state.profile_loading_spinner();
        state.profile_spinner_frame = 7;
        let frame3 = state.profile_loading_spinner();

        // Should have different frames
        assert_ne!(frame1, frame2);
        assert_ne!(frame2, frame3);

        // Should cycle back after 8 frames
        state.profile_spinner_frame = 8;
        let frame_cycled = state.profile_loading_spinner();
        assert_eq!(frame1, frame_cycled);
    }
}

