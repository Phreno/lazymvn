//! TUI State management
//!
//! This module manages the state of the terminal UI including module selection,
//! profiles, flags, command execution, and output display.


// Re-export types

use crate::maven;
use crate::ui::keybindings::{CurrentView, Focus, SearchMode};
use crate::ui::search::{SearchMatch, SearchState, collect_search_matches};
use ratatui::widgets::ListState;
use regex::Regex;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Output data for a specific module
#[derive(Clone, Debug, Default)]
pub struct ModuleOutput {
    pub lines: Vec<String>,
    pub scroll_offset: usize,
    pub command: Option<String>,
    pub profiles: Vec<String>,
    pub flags: Vec<String>,
}

/// Metrics for calculating output display and scrolling
#[derive(Clone, Debug, Default)]
pub struct OutputMetrics {
    width: usize,
    line_display: Vec<String>,
    line_start_rows: Vec<usize>,
    total_rows: usize,
}

impl OutputMetrics {
    pub fn new(width: usize, lines: &[String]) -> Self {
        if width == 0 {
            return Self::default();
        }
        let mut line_display = Vec::with_capacity(lines.len());
        let mut line_start_rows = Vec::with_capacity(lines.len());
        let mut cumulative = 0usize;

        for line in lines {
            line_start_rows.push(cumulative);
            let display = crate::utils::clean_log_line(line).unwrap_or_default();
            let rows = visual_rows(&display, width);
            cumulative += rows;
            line_display.push(display);
        }

        Self {
            width,
            line_display,
            line_start_rows,
            total_rows: cumulative,
        }
    }

    pub fn total_rows(&self) -> usize {
        self.total_rows
    }

    pub fn row_for_match(&self, m: &SearchMatch) -> Option<usize> {
        if self.width == 0 {
            return Some(0);
        }
        let line_index = m.line_index;
        let start_rows = self.line_start_rows.get(line_index)?;
        let display = self.line_display.get(line_index)?;
        let col = column_for_byte_index(display, m.start);
        let row_in_line = col / self.width;
        Some(start_rows + row_in_line)
    }
}

/// Main state structure for the TUI
pub struct TuiState {
    pub current_view: CurrentView,
    pub focus: Focus,
    pub modules: Vec<String>,
    pub profiles: Vec<MavenProfile>,
    pub flags: Vec<BuildFlag>,
    pub modules_list_state: ListState,
    pub profiles_list_state: ListState,
    pub flags_list_state: ListState,
    pub command_output: Vec<String>,
    pub output_offset: usize,
    pub output_view_height: u16,
    module_outputs: HashMap<String, ModuleOutput>,
    pub project_root: PathBuf,
    search_state: Option<SearchState>,
    search_input: Option<String>,
    search_history: Vec<String>,
    search_history_index: Option<usize>,
    search_error: Option<String>,
    output_area_width: u16,
    output_metrics: Option<OutputMetrics>,
    pending_center: Option<SearchMatch>,
    pub search_mod: Option<SearchMode>,
    pub config: crate::config::Config,
    // Debouncing for navigation keys
    last_nav_key_time: Option<Instant>,
    nav_debounce_duration: Duration,
    // Async command execution
    command_receiver: Option<mpsc::Receiver<maven::CommandUpdate>>,
    pub is_command_running: bool,
    command_start_time: Option<Instant>,
    running_process_pid: Option<u32>,
    // Async profile loading
    profiles_receiver: Option<mpsc::Receiver<Result<Vec<String>, String>>>,
    pub profile_loading_status: ProfileLoadingStatus,
    profile_loading_start_time: Option<Instant>,
    profile_spinner_frame: usize,
    // Recent projects
    pub recent_projects: Vec<PathBuf>,
    pub projects_list_state: ListState,
    pub show_projects_popup: bool,
    pub switch_to_project: Option<PathBuf>,
    // Spring Boot starters
    pub starters_cache: crate::starters::StartersCache,
    pub show_starter_selector: bool,
    pub show_starter_manager: bool,
    pub starter_candidates: Vec<String>,
    pub starter_filter: String,
    pub starters_list_state: ListState,
    // Module preferences
    module_preferences: crate::config::ProjectPreferences,
    // Clipboard - keep it alive to prevent "dropped too quickly" errors
    clipboard: Option<arboard::Clipboard>,
    // File watcher for auto-reload
    file_watcher: Option<crate::watcher::FileWatcher>,
    last_command: Option<Vec<String>>,
    watch_enabled: bool,
    // Git branch
    pub git_branch: Option<String>,
}

/// Status of profile loading
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileLoadingStatus {
    /// Profiles are being loaded asynchronously
    Loading,
    /// Profiles have been loaded successfully
    Loaded,
    /// Failed to load profiles
    Error(String),
}

/// State of a Maven profile
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileState {
    /// Profile follows Maven's auto-activation rules
    Default,
    /// Profile is explicitly enabled (will add to -P)
    ExplicitlyEnabled,
    /// Profile is explicitly disabled (will add !profile to -P)
    ExplicitlyDisabled,
}

/// Maven profile with activation state
#[derive(Clone, Debug)]
pub struct MavenProfile {
    pub name: String,
    pub state: ProfileState,
    /// Whether this profile is auto-activated by Maven (file, JDK, OS, etc.)
    pub auto_activated: bool,
}

impl MavenProfile {
    pub fn new(name: String, auto_activated: bool) -> Self {
        Self {
            name,
            state: ProfileState::Default,
            auto_activated,
        }
    }

    /// Returns true if this profile will be active when running Maven
    pub fn is_active(&self) -> bool {
        match self.state {
            ProfileState::Default => self.auto_activated,
            ProfileState::ExplicitlyEnabled => true,
            ProfileState::ExplicitlyDisabled => false,
        }
    }

    /// Returns the profile argument string for Maven (-P flag)
    /// Returns None if profile is in Default state
    pub fn to_maven_arg(&self) -> Option<String> {
        match self.state {
            ProfileState::Default => None,
            ProfileState::ExplicitlyEnabled => Some(self.name.clone()),
            ProfileState::ExplicitlyDisabled => Some(format!("!{}", self.name)),
        }
    }

    /// Cycle through states when toggled
    pub fn toggle(&mut self) {
        self.state = match self.state {
            ProfileState::Default => {
                if self.auto_activated {
                    // Auto-activated: Default → Disabled
                    ProfileState::ExplicitlyDisabled
                } else {
                    // Not auto-activated: Default → Enabled
                    ProfileState::ExplicitlyEnabled
                }
            }
            ProfileState::ExplicitlyEnabled => ProfileState::Default,
            ProfileState::ExplicitlyDisabled => ProfileState::Default,
        };
    }
}

/// Maven build flags that can be toggled
#[derive(Clone, Debug)]
pub struct BuildFlag {
    pub name: String,
    pub flag: String,
    pub enabled: bool,
}

impl TuiState {
    pub fn new(modules: Vec<String>, project_root: PathBuf, config: crate::config::Config) -> Self {
        let mut modules_list_state = ListState::default();
        let profiles_list_state = ListState::default();
        let flags_list_state = ListState::default();
        if !modules.is_empty() {
            modules_list_state.select(Some(0));
        }

        // Initialize common Maven build flags
        let flags = vec![
            BuildFlag {
                name: "Also Make".to_string(),
                flag: "--also-make".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Also Make Dependents".to_string(),
                flag: "--also-make-dependents".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Update Snapshots".to_string(),
                flag: "--update-snapshots".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Skip Tests".to_string(),
                flag: "-DskipTests".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Offline".to_string(),
                flag: "--offline".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Fail Fast".to_string(),
                flag: "--fail-fast".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Fail At End".to_string(),
                flag: "--fail-at-end".to_string(),
                enabled: false,
            },
        ];

        // Load recent projects
        let mut recent_projects_manager = crate::config::RecentProjects::load();
        recent_projects_manager.remove_invalid();
        let recent_projects = recent_projects_manager.get_projects();

        let mut projects_list_state = ListState::default();
        if !recent_projects.is_empty() {
            projects_list_state.select(Some(0));
        }

        // Load starters cache for this project
        let starters_cache = crate::starters::StartersCache::load(&project_root);
        let mut starters_list_state = ListState::default();
        if !starters_cache.starters.is_empty() {
            starters_list_state.select(Some(0));
        }

        // Load module preferences for this project
        let module_preferences = crate::config::ProjectPreferences::load(&project_root);

        // Get Git branch
        let git_branch = crate::utils::get_git_branch(&project_root);

        let mut state = Self {
            current_view: CurrentView::Modules,
            focus: Focus::Modules,
            modules,
            profiles: vec![],
            flags,
            modules_list_state,
            profiles_list_state,
            flags_list_state,
            command_output: vec![],
            output_offset: 0,
            output_view_height: 0,
            module_outputs: HashMap::new(),
            project_root,
            search_state: None,
            search_input: None,
            search_history: Vec::new(),
            search_history_index: None,
            search_error: None,
            output_area_width: 0,
            output_metrics: None,
            pending_center: None,
            search_mod: None,
            config,
            last_nav_key_time: None,
            nav_debounce_duration: Duration::from_millis(100),
            command_receiver: None,
            is_command_running: false,
            command_start_time: None,
            running_process_pid: None,
            profiles_receiver: None,
            profile_loading_status: ProfileLoadingStatus::Loading,
            profile_loading_start_time: None,
            profile_spinner_frame: 0,
            recent_projects,
            projects_list_state,
            show_projects_popup: false,
            switch_to_project: None,
            starters_cache,
            show_starter_selector: false,
            show_starter_manager: false,
            starter_candidates: vec![],
            starter_filter: String::new(),
            starters_list_state,
            module_preferences,
            clipboard: None,
            file_watcher: None,
            last_command: None,
            watch_enabled: false,
            git_branch,
        };

        // Initialize file watcher if configured
        if let Some(watch_config) = &state.config.watch
            && watch_config.enabled
        {
            match crate::watcher::FileWatcher::new(&state.project_root, watch_config.debounce_ms) {
                Ok(watcher) => {
                    state.file_watcher = Some(watcher);
                    state.watch_enabled = true;
                    log::info!("File watcher enabled with {} patterns", watch_config.patterns.len());
                }
                Err(e) => {
                    log::error!("Failed to initialize file watcher: {}", e);
                }
            }
        }

        // Pre-select first flag to ensure alignment
        if !state.flags.is_empty() {
            state.flags_list_state.select(Some(0));
        }

        state.sync_selected_module_output();

        // Load preferences for the initially selected module
        state.load_module_preferences();

        state
    }

    pub fn set_profiles(&mut self, profile_names: Vec<String>) {
        log::info!("set_profiles: Loading {} profiles", profile_names.len());

        // Get auto-activated profiles
        let auto_activated = maven::get_active_profiles(&self.project_root).unwrap_or_else(|e| {
            log::warn!("Failed to get active profiles: {}", e);
            vec![]
        });

        log::debug!("Auto-activated profiles: {:?}", auto_activated);

        // Create MavenProfile structs
        self.profiles = profile_names
            .into_iter()
            .map(|name| {
                let is_auto = auto_activated.contains(&name);
                log::debug!("Profile '{}' auto-activated: {}", name, is_auto);
                MavenProfile::new(name, is_auto)
            })
            .collect();

        if !self.profiles.is_empty() {
            self.profiles_list_state.select(Some(0));
        }

        log::info!(
            "Loaded {} profiles ({} auto-activated)",
            self.profiles.len(),
            auto_activated.len()
        );

        // Load saved preferences for the current module after profiles are created
        self.load_module_preferences();
    }

    /// Check if enough time has passed since last navigation key
    /// Returns true if navigation should be allowed
    fn should_allow_navigation(&mut self) -> bool {
        let now = Instant::now();

        if let Some(last_time) = self.last_nav_key_time
            && now.duration_since(last_time) < self.nav_debounce_duration
        {
            log::debug!("Navigation debounced (too fast)");
            return false;
        }

        self.last_nav_key_time = Some(now);
        true
    }

    // Navigation methods
    pub fn next_item(&mut self) {
        if !self.should_allow_navigation() {
            return;
        }

        match self.focus {
            Focus::Projects => {
                // Projects view is static, no navigation needed
            }
            Focus::Modules => {
                if self.modules.is_empty() {
                    return;
                }
                // Save current module preferences before switching
                self.save_module_preferences();

                let i = match self.modules_list_state.selected() {
                    Some(i) => (i + 1) % self.modules.len(),
                    None => 0,
                };
                self.modules_list_state.select(Some(i));
                self.sync_selected_module_output();

                // Load preferences for the new module
                self.load_module_preferences();
            }
            Focus::Profiles => {
                if !self.profiles.is_empty() {
                    let i = match self.profiles_list_state.selected() {
                        Some(i) => (i + 1) % self.profiles.len(),
                        None => 0,
                    };
                    self.profiles_list_state.select(Some(i));
                    // Update output to show new profile XML
                    self.sync_selected_profile_output();
                }
            }
            Focus::Flags => {
                if !self.flags.is_empty() {
                    let i = match self.flags_list_state.selected() {
                        Some(i) => (i + 1) % self.flags.len(),
                        None => 0,
                    };
                    self.flags_list_state.select(Some(i));
                }
            }
            Focus::Output => {
                // No item navigation in output
            }
        }
    }

    pub fn previous_item(&mut self) {
        if !self.should_allow_navigation() {
            return;
        }

        match self.focus {
            Focus::Projects => {
                // Projects view is static, no navigation needed
            }
            Focus::Modules => {
                if self.modules.is_empty() {
                    return;
                }
                // Save current module preferences before switching
                self.save_module_preferences();

                let i = match self.modules_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.modules.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.modules_list_state.select(Some(i));
                self.sync_selected_module_output();

                // Load preferences for the new module
                self.load_module_preferences();
            }
            Focus::Profiles => {
                if !self.profiles.is_empty() {
                    let i = match self.profiles_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.profiles.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.profiles_list_state.select(Some(i));
                    // Update output to show new profile XML
                    self.sync_selected_profile_output();
                }
            }
            Focus::Flags => {
                if !self.flags.is_empty() {
                    let i = match self.flags_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.flags.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.flags_list_state.select(Some(i));
                }
            }
            Focus::Output => {
                // No item navigation in output
            }
        }
    }

    pub fn toggle_profile(&mut self) {
        if self.focus != Focus::Profiles {
            return;
        }
        if let Some(selected) = self.profiles_list_state.selected()
            && let Some(profile) = self.profiles.get_mut(selected)
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

    pub fn toggle_flag(&mut self) {
        if self.focus != Focus::Flags {
            return;
        }
        if let Some(selected) = self.flags_list_state.selected()
            && let Some(flag) = self.flags.get_mut(selected)
        {
            flag.enabled = !flag.enabled;
            log::info!(
                "Toggled flag '{}' ({}): {}",
                flag.name,
                flag.flag,
                flag.enabled
            );

            // Save preferences after toggling
            self.save_module_preferences();
        }
    }

    pub fn selected_module(&self) -> Option<&str> {
        self.modules_list_state
            .selected()
            .and_then(|i| self.modules.get(i))
            .map(|s| s.as_str())
    }

    pub fn enabled_flag_names(&self) -> Vec<String> {
        self.flags
            .iter()
            .filter(|f| f.enabled)
            .map(|f| f.flag.clone()) // Use flag.flag instead of flag.name
            .collect()
    }

    /// Get list of active profile names for display
    pub fn active_profile_names(&self) -> Vec<String> {
        self.profiles
            .iter()
            .filter(|p| p.is_active())
            .map(|p| p.name.clone())
            .collect()
    }

    pub fn current_output_context(&self) -> Option<(String, Vec<String>, Vec<String>)> {
        self.selected_module().and_then(|module| {
            self.module_outputs.get(module).and_then(|output| {
                output
                    .command
                    .clone()
                    .map(|cmd| (cmd, output.profiles.clone(), output.flags.clone()))
            })
        })
    }

    pub fn switch_to_projects(&mut self) {
        self.current_view = CurrentView::Projects;
        self.focus = Focus::Projects;
    }

    pub fn switch_to_modules(&mut self) {
        self.current_view = CurrentView::Modules;
        self.focus = Focus::Modules;
        self.sync_selected_module_output();
    }

    pub fn switch_to_profiles(&mut self) {
        self.current_view = CurrentView::Profiles;
        if self.profiles_list_state.selected().is_none() && !self.profiles.is_empty() {
            self.profiles_list_state.select(Some(0));
        }
        self.focus = Focus::Profiles;
        // Sync profile XML to output
        self.sync_selected_profile_output();
    }

    pub fn switch_to_flags(&mut self) {
        self.current_view = CurrentView::Flags;
        if self.flags_list_state.selected().is_none() && !self.flags.is_empty() {
            self.flags_list_state.select(Some(0));
        }
        self.focus = Focus::Flags;
    }

    // Focus management
    pub fn focus_output(&mut self) {
        self.focus = Focus::Output;
        self.ensure_current_match_visible();
    }

    /// Cycle focus to the next pane (right arrow)
    pub fn cycle_focus_right(&mut self) {
        let old_focus = self.focus;
        self.focus = self.focus.next();

        // When leaving Profiles focus, restore module output
        if old_focus == Focus::Profiles && self.focus != Focus::Profiles {
            self.sync_selected_module_output();
        }
        // When entering Profiles focus, show profile XML
        else if self.focus == Focus::Profiles {
            self.sync_selected_profile_output();
        }

        if self.focus == Focus::Output {
            self.ensure_current_match_visible();
        }
    }

    /// Cycle focus to the previous pane (left arrow)
    pub fn cycle_focus_left(&mut self) {
        let old_focus = self.focus;
        self.focus = self.focus.previous();

        // When leaving Profiles focus, restore module output
        if old_focus == Focus::Profiles && self.focus != Focus::Profiles {
            self.sync_selected_module_output();
        }
        // When entering Profiles focus, show profile XML
        else if self.focus == Focus::Profiles {
            self.sync_selected_profile_output();
        }

        if self.focus == Focus::Output {
            self.ensure_current_match_visible();
        }
    }

    pub fn has_search_results(&self) -> bool {
        self.search_state
            .as_ref()
            .map(|s| s.has_matches())
            .unwrap_or(false)
    }

    // Live search - performs search as user types without storing in history
    pub fn live_search(&mut self) {
        if let Some(pattern) = self.search_input.as_ref() {
            if pattern.is_empty() {
                self.search_state = None;
                self.search_error = None;
                return;
            }

            match self.apply_search_query(pattern.clone(), false) {
                Ok(_) => {
                    self.search_error = None;
                }
                Err(err) => {
                    self.search_error = Some(err.to_string());
                    self.search_state = None;
                }
            }
        }
    }

    // Module output management
    pub(crate) fn sync_selected_module_output(&mut self) {
        if let Some(module) = self.selected_module() {
            if let Some(module_output) = self.module_outputs.get(module) {
                self.command_output = module_output.lines.clone();
                self.output_offset = module_output.scroll_offset;
            } else {
                self.command_output.clear();
                self.output_offset = 0;
            }
        } else {
            self.command_output.clear();
            self.output_offset = 0;
        }
        self.clamp_output_offset();
        self.output_metrics = None;
        self.refresh_search_matches();
    }

    /// Sync output to show the selected profile's XML
    pub(crate) fn sync_selected_profile_output(&mut self) {
        if let Some(selected) = self.profiles_list_state.selected() {
            if let Some(profile) = self.profiles.get(selected) {
                if let Some((xml, pom_path)) =
                    crate::maven::get_profile_xml(&self.project_root, &profile.name)
                {
                    // Build output with header and XML
                    let relative_path = pom_path
                        .strip_prefix(&self.project_root)
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

                    self.command_output = output;
                    self.output_offset = 0;
                } else {
                    self.command_output = vec![
                        format!("Profile: {}", profile.name),
                        String::new(),
                        "XML not found in POM files.".to_string(),
                    ];
                    self.output_offset = 0;
                }
            } else {
                self.command_output = vec!["No profile selected.".to_string()];
                self.output_offset = 0;
            }
        } else {
            self.command_output = vec!["No profile selected.".to_string()];
            self.output_offset = 0;
        }
        self.clamp_output_offset();
        self.output_metrics = None;
        self.refresh_search_matches();
    }

    fn store_current_module_output(&mut self) {
        if let Some(module) = self.selected_module() {
            // Get the current execution context from the most recent command
            let module_output = if let Some(existing) = self.module_outputs.get(module) {
                ModuleOutput {
                    lines: self.command_output.clone(),
                    scroll_offset: self.output_offset,
                    command: existing.command.clone(),
                    profiles: existing.profiles.clone(),
                    flags: existing.flags.clone(),
                }
            } else {
                ModuleOutput {
                    lines: self.command_output.clone(),
                    scroll_offset: self.output_offset,
                    ..Default::default()
                }
            };
            self.module_outputs
                .insert(module.to_string(), module_output);
        }
    }

    // Command execution
    pub fn run_selected_module_command(&mut self, args: &[&str]) {
        self.run_selected_module_command_with_options(args, false);
    }

    /// Run command with option to use -f instead of -pl
    pub fn run_selected_module_command_with_options(
        &mut self,
        args: &[&str],
        use_file_flag: bool,
    ) {
        log::debug!(
            "run_selected_module_command called with args: {:?}, use_file_flag: {}",
            args,
            use_file_flag
        );

        // Don't start a new command if one is already running
        if self.is_command_running {
            log::warn!("Command already running, ignoring new command request");
            return;
        }

        if let Some(module) = self.selected_module().map(|m| m.to_string()) {
            log::info!("Running async command for module: {}", module);

            // Collect enabled flags
            let enabled_flags: Vec<String> = self
                .flags
                .iter()
                .filter(|f| f.enabled)
                .map(|f| f.flag.clone())
                .collect();

            let enabled_flag_names: Vec<String> = self
                .flags
                .iter()
                .filter(|f| f.enabled)
                .map(|f| f.name.clone())
                .collect();

            // Collect profiles that need to be passed to Maven
            // Only profiles that are not in Default state
            let profile_args: Vec<String> = self
                .profiles
                .iter()
                .filter_map(|p| p.to_maven_arg())
                .collect();

            // Get list of active profile names for display
            let active_profile_names: Vec<String> = self
                .profiles
                .iter()
                .filter(|p| p.is_active())
                .map(|p| p.name.clone())
                .collect();

            log::debug!("Enabled flags: {:?}", enabled_flag_names);
            log::debug!("Profile args for Maven: {:?}", profile_args);
            log::debug!("Active profiles (display): {:?}", active_profile_names);

            // Clear previous output and prepare for new command
            self.command_output = vec![format!("Running: {} ...", args.join(" "))];
            self.output_offset = 0;

            match maven::execute_maven_command_async_with_options(
                &self.project_root,
                Some(&module),
                args,
                &profile_args,
                self.config.maven_settings.as_deref(),
                &enabled_flags,
                use_file_flag,
            ) {
                Ok(receiver) => {
                    log::info!("Async command started successfully");
                    self.command_receiver = Some(receiver);
                    self.is_command_running = true;
                    self.command_start_time = Some(Instant::now());

                    // Save last command for watch mode
                    self.last_command = Some(args.iter().map(|s| s.to_string()).collect());

                    // Store metadata about this command execution
                    let module_output = ModuleOutput {
                        lines: self.command_output.clone(),
                        scroll_offset: self.output_offset,
                        command: Some(args.join(" ")),
                        profiles: active_profile_names,
                        flags: enabled_flag_names,
                    };
                    self.module_outputs.insert(module, module_output);
                }
                Err(e) => {
                    log::error!("Failed to start async command: {}", e);
                    self.command_output = vec![format!("Error starting command: {e}")];
                    self.output_offset = 0;
                }
            }
        } else {
            log::warn!("No module selected for command execution");
            self.command_output = vec!["No module selected".to_string()];
            self.output_offset = 0;
        }
        self.clamp_output_offset();
        self.output_metrics = None;
    }

    /// Check for and process any pending command updates
    /// Should be called regularly from the main event loop
    pub fn poll_command_updates(&mut self) {
        // Get output configuration from config or use defaults
        let output_config = self.config.output.as_ref().cloned().unwrap_or_default();
        let max_output_lines = output_config.max_lines;
        let max_updates_per_poll = output_config.max_updates_per_poll;

        // Collect all pending updates first to avoid borrowing issues
        let mut updates = Vec::new();
        let mut should_clear_receiver = false;

        if let Some(receiver) = self.command_receiver.as_ref() {
            let mut count = 0;
            loop {
                if count >= max_updates_per_poll {
                    // Limit updates per poll to prevent UI freeze
                    break;
                }
                match receiver.try_recv() {
                    Ok(update) => {
                        updates.push(update);
                        count += 1;
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        log::warn!("Command channel disconnected unexpectedly");
                        should_clear_receiver = true;
                        break;
                    }
                }
            }
        }

        // Check if we're currently at the bottom (for auto-scroll)
        let was_at_bottom = self.output_offset >= self.max_scroll_offset();
        let mut had_output_lines = false;

        // Now process all updates
        for update in updates {
            match update {
                maven::CommandUpdate::Started(pid) => {
                    log::info!("Command started with PID: {}", pid);
                    self.running_process_pid = Some(pid);
                }
                maven::CommandUpdate::OutputLine(line) => {
                    self.command_output.push(line);
                    had_output_lines = true;
                    
                    // Trim buffer if it exceeds max size
                    if self.command_output.len() > max_output_lines {
                        let excess = self.command_output.len() - max_output_lines;
                        self.command_output.drain(0..excess);
                        log::debug!("Trimmed {} lines from output buffer (max: {})", excess, max_output_lines);
                    }
                }
                maven::CommandUpdate::Completed => {
                    log::info!("Command completed successfully");
                    self.command_output.push(String::new());
                    self.command_output
                        .push("✓ Command completed successfully".to_string());
                    self.is_command_running = false;
                    self.command_receiver = None;
                    self.running_process_pid = None;
                    self.store_current_module_output();
                    self.output_metrics = None;

                    // Send desktop notification
                    self.send_notification(
                        "LazyMVN - Build Complete",
                        "Maven command completed successfully ✓",
                        true,
                    );
                }
                maven::CommandUpdate::Error(msg) => {
                    log::error!("Command failed: {}", msg);
                    self.command_output.push(String::new());
                    self.command_output.push(format!("✗ {}", msg));
                    self.is_command_running = false;
                    self.command_receiver = None;
                    self.running_process_pid = None;
                    self.store_current_module_output();
                    self.output_metrics = None;

                    // Send desktop notification for error
                    self.send_notification(
                        "LazyMVN - Build Failed",
                        &format!("Maven command failed: {}", msg),
                        false,
                    );
                }
            }
        }

        // Only update scroll and metrics once at the end if we had output lines
        if had_output_lines {
            // Auto-scroll to bottom while command is running (always follow logs)
            // Only respect user's scroll position when command is not running
            if was_at_bottom || self.is_command_running {
                self.scroll_output_to_end();
            }
            self.store_current_module_output();
            self.output_metrics = None;
        }

        if should_clear_receiver {
            self.is_command_running = false;
            self.command_receiver = None;
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
                "Timeout: Profile loading took too long (>30s)".to_string()
            );
            self.profiles_receiver = None;
            self.profile_loading_start_time = None;
            return;
        }

        if let Some(receiver) = self.profiles_receiver.as_ref() {
            match receiver.try_recv() {
                Ok(Ok(profile_names)) => {
                    log::info!("Profiles loaded asynchronously: {} profiles", profile_names.len());
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
                        "Profile loading channel disconnected".to_string()
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

        let project_root = self.project_root.clone();
        std::thread::spawn(move || {
            let result = maven::get_profiles(&project_root)
                .map_err(|e| e.to_string());
            
            if let Err(e) = tx.send(result) {
                log::error!("Failed to send profiles result: {}", e);
            }
        });

        log::info!("Started asynchronous profile loading");
    }

    /// Kill the currently running Maven process
    pub fn kill_running_process(&mut self) {
        if let Some(pid) = self.running_process_pid {
            log::info!("Attempting to kill process with PID: {}", pid);
            match maven::kill_process(pid) {
                Ok(()) => {
                    self.command_output.push(String::new());
                    self.command_output
                        .push(format!("⚠ Process {} killed by user", pid));
                    self.is_command_running = false;
                    self.command_receiver = None;
                    self.running_process_pid = None;
                    self.store_current_module_output();
                    self.output_metrics = None;
                }
                Err(e) => {
                    log::error!("Failed to kill process: {}", e);
                    self.command_output.push(String::new());
                    self.command_output
                        .push(format!("✗ Failed to kill process: {}", e));
                }
            }
        } else {
            log::warn!("No running process to kill");
        }
    }

    /// Check file watcher and re-run command if files changed
    pub fn check_file_watcher(&mut self) {
        if !self.watch_enabled || self.is_command_running {
            return;
        }

        if let Some(watcher) = &mut self.file_watcher
            && watcher.check_changes()
        {
            log::info!("File changes detected, checking if should re-run command");
            
            // Clone last command to avoid borrow issues
            let last_cmd = self.last_command.clone();
            
            // Check if last command is watchable
            if let Some(last_cmd) = last_cmd {
                let watch_config = self.config.watch.as_ref().unwrap();
                
                // Check if this command should trigger auto-reload
                let should_rerun = last_cmd.iter().any(|arg| {
                    watch_config.commands.iter().any(|cmd| arg.contains(cmd))
                });
                
                if should_rerun {
                    self.command_output.push(String::new());
                    self.command_output.push("🔄 Files changed, reloading...".to_string());
                    self.command_output.push(String::new());
                    
                    // Re-run the last command
                    log::info!("Re-running command due to file changes");
                    let args: Vec<&str> = last_cmd.iter().map(|s| s.as_str()).collect();
                    self.run_selected_module_command(&args);
                }
            }
        }
    }

    /// Yank (copy) the output to clipboard
    pub fn yank_output(&mut self) {
        if self.command_output.is_empty() {
            log::info!("No output to copy");
            self.command_output.push(String::new());
            self.command_output.push("⚠ No output to copy".to_string());
            return;
        }

        let output_text = self.command_output.join("\n");
        let lines = self.command_output.len();

        // Try to use system clipboard tools first (more reliable for terminal apps)
        #[cfg(target_os = "linux")]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};

            // Try wl-copy (Wayland) first
            if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn()
                && let Some(mut stdin) = child.stdin.take()
                && stdin.write_all(output_text.as_bytes()).is_ok()
            {
                drop(stdin);
                if child.wait().is_ok() {
                    log::info!("Copied {} lines via wl-copy", lines);
                    self.command_output.push(String::new());
                    self.command_output
                        .push(format!("✓ Copied {} lines to clipboard", lines));
                    return;
                }
            }

            // Try xclip (X11) as fallback
            if let Ok(mut child) = Command::new("xclip")
                .arg("-selection")
                .arg("clipboard")
                .stdin(Stdio::piped())
                .spawn()
                && let Some(mut stdin) = child.stdin.take()
                && stdin.write_all(output_text.as_bytes()).is_ok()
            {
                drop(stdin);
                if child.wait().is_ok() {
                    log::info!("Copied {} lines via xclip", lines);
                    self.command_output.push(String::new());
                    self.command_output
                        .push(format!("✓ Copied {} lines to clipboard", lines));
                    return;
                }
            }

            // Try xsel as another X11 fallback
            if let Ok(mut child) = Command::new("xsel")
                .arg("--clipboard")
                .stdin(Stdio::piped())
                .spawn()
                && let Some(mut stdin) = child.stdin.take()
                && stdin.write_all(output_text.as_bytes()).is_ok()
            {
                drop(stdin);
                if child.wait().is_ok() {
                    log::info!("Copied {} lines via xsel", lines);
                    self.command_output.push(String::new());
                    self.command_output
                        .push(format!("✓ Copied {} lines to clipboard", lines));
                    return;
                }
            }
        }

        // Windows: Use PowerShell Set-Clipboard
        #[cfg(target_os = "windows")]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};

            // Try PowerShell Set-Clipboard
            if let Ok(mut child) = Command::new("powershell")
                .arg("-Command")
                .arg("$input | Set-Clipboard")
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(output_text.as_bytes()).is_ok() {
                        drop(stdin);
                        if child.wait().is_ok() {
                            log::info!("Copied {} lines via PowerShell Set-Clipboard", lines);
                            self.command_output.push(String::new());
                            self.command_output
                                .push(format!("✓ Copied {} lines to clipboard", lines));
                            return;
                        }
                    }
                }
            }

            // Try clip.exe as fallback (built-in Windows command)
            if let Ok(mut child) = Command::new("clip").stdin(Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(output_text.as_bytes()).is_ok() {
                        drop(stdin);
                        if child.wait().is_ok() {
                            log::info!("Copied {} lines via clip.exe", lines);
                            self.command_output.push(String::new());
                            self.command_output
                                .push(format!("✓ Copied {} lines to clipboard", lines));
                            return;
                        }
                    }
                }
            }
        }

        // macOS: Use pbcopy
        #[cfg(target_os = "macos")]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};

            if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(output_text.as_bytes()).is_ok() {
                        drop(stdin);
                        if child.wait().is_ok() {
                            log::info!("Copied {} lines via pbcopy", lines);
                            self.command_output.push(String::new());
                            self.command_output
                                .push(format!("✓ Copied {} lines to clipboard", lines));
                            return;
                        }
                    }
                }
            }
        }

        // Fallback to arboard if all system tools failed
        let clipboard_result = if let Some(ref mut clipboard) = self.clipboard {
            clipboard.set_text(output_text)
        } else {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    let result = clipboard.set_text(output_text);
                    self.clipboard = Some(clipboard);
                    result
                }
                Err(e) => {
                    log::error!("Failed to initialize clipboard: {}", e);
                    self.command_output.push(String::new());
                    self.command_output
                        .push(format!("✗ Clipboard not available: {}", e));
                    return;
                }
            }
        };

        match clipboard_result {
            Ok(()) => {
                log::info!("Copied {} lines to clipboard via arboard", lines);
                self.command_output.push(String::new());
                self.command_output
                    .push(format!("✓ Copied {} lines to clipboard", lines));
            }
            Err(e) => {
                log::error!("Failed to copy to clipboard: {}", e);
                self.command_output.push(String::new());
                self.command_output.push(format!("✗ Failed to copy: {}", e));
            }
        }
    }

    /// Get elapsed time of current command in seconds
    pub fn command_elapsed_seconds(&self) -> Option<u64> {
        self.command_start_time
            .map(|start| start.elapsed().as_secs())
    }

    /// Send desktop notification
    fn send_notification(&self, title: &str, body: &str, success: bool) {
        // Check if notifications are enabled (default: true)
        let enabled = self.config.notifications_enabled.unwrap_or(true);
        if !enabled {
            log::debug!("Notifications disabled in config, skipping notification");
            return;
        }

        use notify_rust::{Notification, Timeout};

        log::debug!("Sending notification: {} - {}", title, body);

        let mut notification = Notification::new();
        notification
            .summary(title)
            .body(body)
            .timeout(Timeout::Milliseconds(5000)); // 5 seconds

        // Set icon based on success/failure (platform-specific)
        #[cfg(target_os = "linux")]
        {
            if success {
                notification.icon("dialog-information");
            } else {
                notification.icon("dialog-error");
            }
        }

        // Try to show the notification
        if let Err(e) = notification.show() {
            log::warn!("Failed to send desktop notification: {}", e);
            // Don't show error to user, notifications are optional
        }
    }

    // Output display and metrics
    pub fn update_output_metrics(&mut self, width: u16) {
        self.output_area_width = width;
        if width == 0 || self.command_output.is_empty() {
            self.output_metrics = None;
            return;
        }
        let width_usize = width as usize;
        self.output_metrics = Some(OutputMetrics::new(width_usize, &self.command_output));
    }

    pub fn set_output_view_dimensions(&mut self, height: u16, width: u16) {
        self.output_view_height = height;
        self.output_area_width = width;
        self.clamp_output_offset();
        self.apply_pending_center();
        self.ensure_current_match_visible();
    }

    // Scrolling methods
    fn clamp_output_offset(&mut self) {
        let max_offset = self.max_scroll_offset();
        if self.output_offset > max_offset {
            self.output_offset = max_offset;
        }
    }

    pub fn scroll_output_lines(&mut self, delta: isize) {
        if !self.should_allow_navigation() {
            return;
        }
        if self.command_output.is_empty() {
            return;
        }
        let max_offset = self.max_scroll_offset();
        let current = self.output_offset as isize;
        let next = (current + delta).clamp(0, max_offset as isize) as usize;
        if next != self.output_offset {
            self.output_offset = next;
            self.store_current_module_output();
        }
    }

    pub fn scroll_output_pages(&mut self, delta: isize) {
        let page = self.output_view_height.max(1) as isize;
        self.scroll_output_lines(delta * page);
    }

    pub fn scroll_output_to_start(&mut self) {
        if self.command_output.is_empty() {
            return;
        }
        self.output_offset = 0;
        self.store_current_module_output();
    }

    pub fn scroll_output_to_end(&mut self) {
        let max_offset = self.max_scroll_offset();
        self.output_offset = max_offset;
        self.store_current_module_output();
    }

    fn max_scroll_offset(&self) -> usize {
        let height = self.output_view_height as usize;
        if height == 0 {
            return 0;
        }
        let total = self.total_display_rows();
        total.saturating_sub(height)
    }

    fn total_display_rows(&self) -> usize {
        if let Some(metrics) = self.output_metrics.as_ref() {
            metrics.total_rows()
        } else {
            self.command_output.len()
        }
    }

    // Search functionality
    pub fn begin_search_input(&mut self) {
        self.search_input = Some(String::new());
        self.search_history_index = None;
        self.search_error = None;
    }

    pub fn cancel_search_input(&mut self) {
        self.search_input = None;
        self.search_history_index = None;
        self.search_error = None;
    }

    pub fn push_search_char(&mut self, ch: char) {
        if let Some(buffer) = self.search_input.as_mut() {
            buffer.push(ch);
            self.search_history_index = None;
        }
    }

    pub fn backspace_search_char(&mut self) {
        if let Some(buffer) = self.search_input.as_mut() {
            buffer.pop();
            self.search_history_index = None;
        }
    }

    pub fn recall_previous_search(&mut self) {
        if self.search_history.is_empty() {
            return;
        }
        let len = self.search_history.len();
        let next_index = match self.search_history_index {
            None => Some(len - 1),
            Some(0) => Some(0),
            Some(i) => Some(i - 1),
        };
        if let Some(idx) = next_index
            && let Some(query) = self.search_history.get(idx)
        {
            self.search_input = Some(query.clone());
            self.search_history_index = Some(idx);
        }
    }

    pub fn recall_next_search(&mut self) {
        if self.search_history.is_empty() {
            return;
        }
        if let Some(idx) = self.search_history_index {
            if idx + 1 < self.search_history.len() {
                let query = self.search_history[idx + 1].clone();
                self.search_input = Some(query);
                self.search_history_index = Some(idx + 1);
            } else {
                self.search_input = Some(String::new());
                self.search_history_index = None;
            }
        }
    }

    pub fn submit_search(&mut self) {
        if let Some(pattern) = self.search_input.clone() {
            if !pattern.is_empty() {
                match self.apply_search_query(pattern.clone(), false) {
                    Ok(_) => {
                        if !self.search_history.contains(&pattern) {
                            self.search_history.push(pattern);
                        }
                        self.search_input = None;
                        self.search_error = None;
                        self.search_history_index = None;
                    }
                    Err(e) => {
                        self.search_error = Some(e.to_string());
                    }
                }
            } else {
                self.search_input = None;
                self.search_state = None;
                self.search_error = None;
                self.search_history_index = None;
            }
        }
    }

    fn apply_search_query(
        &mut self,
        query: String,
        keep_current: bool,
    ) -> Result<(), regex::Error> {
        let regex = Regex::new(&query)?;
        let matches = collect_search_matches(&self.command_output, &regex);
        let mut current_index = 0usize;

        if keep_current && let Some(existing) = self.search_state.as_ref() {
            current_index = existing.current.min(matches.len().saturating_sub(1));
        }

        self.search_state = Some(SearchState::new(query, matches));

        let current_match = if let Some(search) = self.search_state.as_mut() {
            search.jump_to_match(current_index);
            search.current_match().cloned()
        } else {
            None
        };

        if let Some(match_to_center) = current_match {
            self.center_on_match(match_to_center);
        }

        Ok(())
    }

    fn refresh_search_matches(&mut self) {
        if let Some(existing) = self.search_state.as_ref().cloned() {
            let _ = self.apply_search_query(existing.query, true);
        } else {
            self.search_state = None;
        }
    }

    pub fn next_search_match(&mut self) {
        let current_match = if let Some(search) = self.search_state.as_mut() {
            if search.has_matches() {
                search.next_match();
                search.current_match().cloned()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(match_to_center) = current_match {
            self.center_on_match(match_to_center);
        }
    }

    pub fn previous_search_match(&mut self) {
        let current_match = if let Some(search) = self.search_state.as_mut() {
            if search.has_matches() {
                search.previous_match();
                search.current_match().cloned()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(match_to_center) = current_match {
            self.center_on_match(match_to_center);
        }
    }

    fn center_on_match(&mut self, target: SearchMatch) {
        self.pending_center = Some(target);
        self.apply_pending_center();
    }

    fn apply_pending_center(&mut self) {
        let target = match self.pending_center.clone() {
            Some(t) => t,
            None => return,
        };
        if self.output_view_height == 0 || self.output_area_width == 0 {
            return;
        }
        let metrics = match self.output_metrics.as_ref() {
            Some(m) => m,
            None => return,
        };
        let total_rows = metrics.total_rows();
        if total_rows == 0 {
            self.pending_center = None;
            return;
        }
        if let Some(target_row) = metrics.row_for_match(&target) {
            let view_height = self.output_view_height as usize;
            let desired_offset = target_row.saturating_sub(view_height / 2);
            let max_offset = total_rows.saturating_sub(view_height);
            self.output_offset = desired_offset.min(max_offset);
            self.store_current_module_output();
        }
        self.pending_center = None;
    }

    fn ensure_current_match_visible(&mut self) {
        if let Some(search) = self.search_state.as_ref()
            && let Some(current_match) = search.current_match()
        {
            self.center_on_match(current_match.clone());
        }
    }

    // Getters for search state
    pub fn search_line_style(
        &self,
        line_index: usize,
    ) -> Option<Vec<(ratatui::style::Style, std::ops::Range<usize>)>> {
        self.search_state
            .as_ref()
            .and_then(|search| crate::ui::search::search_line_style(line_index, search))
    }

    pub fn search_status_line(&self) -> Option<ratatui::text::Line<'static>> {
        crate::ui::search::search_status_line(
            self.search_input.as_deref(),
            self.search_error.as_deref(),
            self.search_state.as_ref(),
        )
    }

    // Recent projects methods
    pub fn show_recent_projects(&mut self) {
        log::info!("Showing recent projects popup");
        self.show_projects_popup = true;
        if self.focus != Focus::Projects {
            self.focus = Focus::Projects;
        }
    }

    pub fn hide_recent_projects(&mut self) {
        log::info!("Hiding recent projects popup");
        self.show_projects_popup = false;
    }

    pub fn select_current_project(&mut self) {
        if let Some(idx) = self.projects_list_state.selected()
            && let Some(project) = self.recent_projects.get(idx)
        {
            log::info!("Selected project: {:?}", project);
            self.switch_to_project = Some(project.clone());
            self.hide_recent_projects();
        }
    }

    pub fn next_project(&mut self) {
        if self.recent_projects.is_empty() {
            return;
        }
        let i = match self.projects_list_state.selected() {
            Some(i) => (i + 1) % self.recent_projects.len(),
            None => 0,
        };
        self.projects_list_state.select(Some(i));
    }

    pub fn previous_project(&mut self) {
        if self.recent_projects.is_empty() {
            return;
        }
        let i = match self.projects_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.recent_projects.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.projects_list_state.select(Some(i));
    }

    // Spring Boot starter methods
    pub fn show_starter_selector(&mut self) {
        log::info!("Showing starter selector");

        // Scan for potential starters if candidates list is empty
        if self.starter_candidates.is_empty() {
            self.starter_candidates = crate::starters::find_potential_starters(&self.project_root);
            log::debug!("Found {} potential starters", self.starter_candidates.len());
        }

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

        if !self.starters_cache.starters.is_empty() {
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

                // Create a new starter entry if not already cached
                if !self
                    .starters_cache
                    .starters
                    .iter()
                    .any(|s| &s.fully_qualified_class_name == fqcn)
                {
                    let label = fqcn.split('.').next_back().unwrap_or(fqcn).to_string();
                    let is_default = self.starters_cache.starters.is_empty();
                    let starter = crate::starters::Starter::new(fqcn.clone(), label, is_default);
                    self.starters_cache.add_starter(starter);

                    // Save the cache
                    if let Err(e) = self.starters_cache.save(&self.project_root) {
                        log::error!("Failed to save starters cache: {}", e);
                    }
                }

                // Update last used
                self.starters_cache.set_last_used(fqcn.clone());
                if let Err(e) = self.starters_cache.save(&self.project_root) {
                    log::error!("Failed to save last used starter: {}", e);
                }

                // Run the starter
                self.run_spring_boot_starter(fqcn);
                self.hide_starter_selector();
            }
        }
    }

    pub fn run_spring_boot_starter(&mut self, fqcn: &str) {
        log::info!("Running Spring Boot starter: {}", fqcn);

        // Get selected module
        let module = self.selected_module();

        // Detect Spring Boot capabilities for this module
        match crate::maven::detect_spring_boot_capabilities(&self.project_root, module) {
            Ok(detection) => {
                // Decide launch strategy based on detection and config
                let launch_mode = self
                    .config
                    .launch_mode
                    .unwrap_or(crate::config::LaunchMode::Auto);
                let strategy = crate::maven::decide_launch_strategy(&detection, launch_mode);

                log::info!(
                    "Launch strategy decided: {:?} (mode={:?}, has_sb_plugin={}, packaging={:?})",
                    strategy,
                    launch_mode,
                    detection.has_spring_boot_plugin,
                    detection.packaging
                );

                // Collect active profile names (those that need to be passed to Maven)
                let active_profiles: Vec<String> = self
                    .profiles
                    .iter()
                    .filter_map(|p| p.to_maven_arg())
                    .collect();

                // Build launch command with the strategy
                let command_parts = crate::maven::build_launch_command(
                    strategy,
                    Some(fqcn),
                    &active_profiles,
                    &[], // JVM args could be added here in the future
                    detection.packaging.as_deref(),
                );

                // Convert to &str references
                let args: Vec<&str> = command_parts.iter().map(|s| s.as_str()).collect();

                // For Spring Boot projects, use -pl instead of -f to inherit parent plugin config
                // but add --also-make to ensure dependencies are built
                let use_file_flag = strategy == crate::maven::LaunchStrategy::ExecJava;
                self.run_selected_module_command_with_options(&args, use_file_flag);
            }
            Err(e) => {
                log::error!("Failed to detect Spring Boot capabilities: {}", e);
                self.command_output = vec![
                    format!("Error detecting launch strategy: {}", e),
                    String::new(),
                    "Falling back to spring-boot:run...".to_string(),
                ];

                // Fallback to old behavior
                let main_class_arg = format!("-Dspring-boot.run.mainClass={}", fqcn);
                let args = vec!["spring-boot:run", &main_class_arg];
                // Use -pl for spring-boot:run to inherit parent plugin config
                self.run_selected_module_command_with_options(&args, false);
            }
        }
    }

    pub fn run_preferred_starter(&mut self) {
        if let Some(starter) = self.starters_cache.get_preferred_starter() {
            log::info!(
                "Running preferred starter: {}",
                starter.fully_qualified_class_name
            );
            self.run_spring_boot_starter(&starter.fully_qualified_class_name.clone());
        } else {
            // No cached starter, show selector
            log::info!("No preferred starter found, showing selector");
            self.show_starter_selector();
        }
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
            self.starters_cache
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
            self.starters_cache
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
        if let Some(idx) = self.starters_list_state.selected()
            && let Some(starter) = self.starters_cache.starters.get(idx)
        {
            let fqcn = starter.fully_qualified_class_name.clone();
            self.starters_cache.set_default(&fqcn);

            if let Err(e) = self.starters_cache.save(&self.project_root) {
                log::error!("Failed to save starters cache: {}", e);
            }
        }
    }

    pub fn remove_selected_starter(&mut self) {
        if let Some(idx) = self.starters_list_state.selected()
            && let Some(starter) = self.starters_cache.starters.get(idx)
        {
            let fqcn = starter.fully_qualified_class_name.clone();
            if self.starters_cache.remove_starter(&fqcn) {
                log::info!("Removed starter: {}", fqcn);

                if let Err(e) = self.starters_cache.save(&self.project_root) {
                    log::error!("Failed to save starters cache: {}", e);
                }

                // Adjust selection
                if self.starters_cache.starters.is_empty() {
                    self.starters_list_state.select(None);
                } else if idx >= self.starters_cache.starters.len() {
                    self.starters_list_state
                        .select(Some(self.starters_cache.starters.len() - 1));
                }
            }
        }
    }

    // Module preferences methods

    /// Save current profiles and flags for the selected module
    pub fn save_module_preferences(&mut self) {
        if let Some(module) = self.selected_module() {
            // Save only explicitly set profiles (not Default state)
            let explicit_profiles: Vec<String> = self
                .profiles
                .iter()
                .filter_map(|p| match p.state {
                    ProfileState::ExplicitlyEnabled => Some(p.name.clone()),
                    ProfileState::ExplicitlyDisabled => Some(format!("!{}", p.name)),
                    ProfileState::Default => None,
                })
                .collect();

            let prefs = crate::config::ModulePreferences {
                active_profiles: explicit_profiles.clone(),
                enabled_flags: self.enabled_flag_names(),
            };

            log::info!(
                "Saving preferences for module '{}': profiles={:?}, flags={:?}",
                module,
                prefs.active_profiles,
                prefs.enabled_flags
            );

            self.module_preferences
                .set_module_prefs(module.to_string(), prefs);

            if let Err(e) = self.module_preferences.save(&self.project_root) {
                log::error!("Failed to save module preferences: {}", e);
            }
        }
    }

    /// Load preferences for the selected module
    pub fn load_module_preferences(&mut self) {
        if let Some(module) = self.selected_module() {
            if let Some(prefs) = self.module_preferences.get_module_prefs(module) {
                log::info!(
                    "Loading preferences for module '{}': profiles={:?}, flags={:?}",
                    module,
                    prefs.active_profiles,
                    prefs.enabled_flags
                );

                // Restore profile states
                for profile in &mut self.profiles {
                    // Check if profile is explicitly enabled or disabled
                    let disabled_name = format!("!{}", profile.name);

                    if prefs.active_profiles.contains(&profile.name) {
                        profile.state = ProfileState::ExplicitlyEnabled;
                        log::debug!("Restored profile '{}' as ExplicitlyEnabled", profile.name);
                    } else if prefs.active_profiles.contains(&disabled_name) {
                        profile.state = ProfileState::ExplicitlyDisabled;
                        log::debug!("Restored profile '{}' as ExplicitlyDisabled", profile.name);
                    } else {
                        profile.state = ProfileState::Default;
                        log::debug!("Profile '{}' in Default state", profile.name);
                    }
                }

                // Restore enabled flags
                for flag in &mut self.flags {
                    flag.enabled = prefs.enabled_flags.contains(&flag.flag);
                }
            } else {
                log::debug!("No saved preferences for module '{}'", module);
                // Reset all profiles to Default state
                for profile in &mut self.profiles {
                    profile.state = ProfileState::Default;
                }
            }
        }
    }
}

// Helper functions
fn visual_rows(line: &str, width: usize) -> usize {
    if width == 0 {
        return 1;
    }
    let display_width = UnicodeWidthStr::width(line);
    let rows = display_width.div_ceil(width);
    rows.max(1)
}

fn column_for_byte_index(s: &str, byte_index: usize) -> usize {
    let mut column = 0usize;
    for (idx, ch) in s.char_indices() {
        if idx >= byte_index {
            break;
        }
        column += UnicodeWidthChar::width(ch).unwrap_or(0);
    }
    column
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_loading_status_initial_state() {
        let config = crate::config::Config::default();
        let state = TuiState::new(
            vec!["test-module".to_string()],
            PathBuf::from("/tmp/test"),
            config,
        );
        
        // Initially, profiles should be in Loading state
        assert!(matches!(state.profile_loading_status, ProfileLoadingStatus::Loading));
        assert_eq!(state.profiles.len(), 0);
    }

    #[test]
    fn test_profile_loading_spinner_frames() {
        let config = crate::config::Config::default();
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

    #[test]
    fn test_profile_state_transitions() {
        let mut profile = MavenProfile::new("test-profile".to_string(), false);
        
        // Default state for non-auto profile
        assert_eq!(profile.state, ProfileState::Default);
        assert!(!profile.is_active());
        
        // Toggle should enable
        profile.toggle();
        assert_eq!(profile.state, ProfileState::ExplicitlyEnabled);
        assert!(profile.is_active());
        
        // Toggle again should return to default
        profile.toggle();
        assert_eq!(profile.state, ProfileState::Default);
        assert!(!profile.is_active());
    }

    #[test]
    fn test_auto_activated_profile_state_transitions() {
        let mut profile = MavenProfile::new("auto-profile".to_string(), true);
        
        // Default state for auto-activated profile
        assert_eq!(profile.state, ProfileState::Default);
        assert!(profile.is_active()); // Auto-activated, so active by default
        
        // Toggle should disable
        profile.toggle();
        assert_eq!(profile.state, ProfileState::ExplicitlyDisabled);
        assert!(!profile.is_active());
        
        // Toggle again should return to default (auto-activated)
        profile.toggle();
        assert_eq!(profile.state, ProfileState::Default);
        assert!(profile.is_active());
    }

    #[test]
    fn test_profile_maven_arg_generation() {
        let mut profile = MavenProfile::new("test".to_string(), false);
        
        // Default state: no arg
        assert_eq!(profile.to_maven_arg(), None);
        
        // Explicitly enabled: returns profile name
        profile.state = ProfileState::ExplicitlyEnabled;
        assert_eq!(profile.to_maven_arg(), Some("test".to_string()));
        
        // Explicitly disabled: returns !profile
        profile.state = ProfileState::ExplicitlyDisabled;
        assert_eq!(profile.to_maven_arg(), Some("!test".to_string()));
    }
}
