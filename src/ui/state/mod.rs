//! TUI State management
//!
//! This module manages the state of the terminal UI including module selection,
//! profiles, flags, command execution, and output display.

// Sub-modules
mod commands;
mod config_reload;
mod flags;
mod launcher_config;
mod navigation;
mod output;
mod profiles;
mod project_tab;
mod search;
mod tabs;

pub use project_tab::ProjectTab;

// Re-export types

use crate::maven;
use crate::maven::detection::SpringBootDetection;
use crate::ui::keybindings::{CurrentView, Focus, SearchMode};
use crate::ui::search::{SearchMatch, SearchState};
use ratatui::widgets::ListState;
use std::{
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

    /// Yank (copy) comprehensive debug information to clipboard
    /// Includes: version info, git hash, logs, output from all tabs, and config file
    pub fn yank_debug_info(&mut self) {
        log::info!("Collecting comprehensive debug information");

        let mut debug_info = Vec::new();

        Self::add_debug_header(&mut debug_info);
        debug_info.extend(Self::collect_version_info());
        debug_info.extend(Self::collect_system_info());
        debug_info.extend(self.collect_config_info());
        debug_info.extend(self.collect_all_tabs_output());
        debug_info.extend(Self::collect_logs());
        Self::add_debug_footer(&mut debug_info);

        let debug_text = debug_info.join("\n");
        let lines = debug_info.len();

        log::info!("Collected {} lines of debug information", lines);

        self.copy_to_clipboard(&debug_text, lines, "debug report");
    }

    /// Add debug report header
    fn add_debug_header(debug_info: &mut Vec<String>) {
        debug_info.push("=".repeat(80));
        debug_info.push("LazyMVN Debug Report".to_string());
        debug_info.push("=".repeat(80));
        debug_info.push(format!(
            "Generated: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
        debug_info.push(String::new());
    }

    /// Collect version information (LazyMVN version, build date, git info)
    fn collect_version_info() -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== Version Information ===".to_string());
        info.push(format!(
            "LazyMVN Version: {}",
            crate::utils::version::current()
        ));

        if let Some(channel) = crate::utils::version::build_channel() {
            info.push(format!("Channel: {}", channel));
        }
        if let Some(tag) = crate::utils::version::build_tag() {
            info.push(format!("Build Tag: {}", tag));
        }
        if let Some(commit) = crate::utils::version::commit_sha() {
            info.push(format!("Commit SHA: {}", commit));
        }

        if let Some(date) = option_env!("VERGEN_BUILD_DATE") {
            info.push(format!("Build Date: {}", date));
        }
        if let Some(branch) = option_env!("VERGEN_GIT_BRANCH") {
            info.push(format!("Git Branch: {}", branch));
        }
        if let Some(sha) = option_env!("VERGEN_GIT_SHA") {
            info.push(format!("Git Commit: {}", sha));
        }
        info.push(String::new());
        info
    }

    /// Collect system information (OS, architecture)
    fn collect_system_info() -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== System Information ===".to_string());
        info.push(format!("OS: {}", std::env::consts::OS));
        info.push(format!("Architecture: {}", std::env::consts::ARCH));
        info.push(String::new());
        info
    }

    /// Collect configuration file content
    fn collect_config_info(&self) -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== Configuration (config.toml) ===".to_string());

        let config_path =
            crate::core::config::get_project_config_path(&self.get_active_tab().project_root);
        if config_path.exists() {
            info.push(format!("Location: {}", config_path.display()));
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    // Filter out comments and empty lines to reduce size
                    let filtered_lines: Vec<String> = content
                        .lines()
                        .filter(|line| {
                            let trimmed = line.trim();
                            // Keep non-empty lines that don't start with #
                            !trimmed.is_empty() && !trimmed.starts_with('#')
                        })
                        .map(String::from)
                        .collect();
                    
                    if filtered_lines.is_empty() {
                        info.push("(Config file is empty or contains only comments)".to_string());
                    } else {
                        info.push("(Comments and empty lines removed for brevity)".to_string());
                        info.extend(filtered_lines);
                    }
                }
                Err(e) => {
                    info.push(format!("Error reading config file: {}", e));
                }
            }
        } else {
            info.push("(No config.toml configuration file found)".to_string());
            info.push(format!("Expected location: {}", config_path.display()));
            info.push("Run 'lazymvn --setup' to create configuration".to_string());
        }
        info.push(String::new());
        info
    }

    /// Collect output from all tabs
    fn collect_all_tabs_output(&self) -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== Output from All Tabs ===".to_string());

        for (idx, tab) in self.tabs.iter().enumerate() {
            info.extend(Self::collect_tab_output(
                tab,
                idx,
                idx == self.active_tab_index,
            ));
        }

        info
    }

    /// Collect output from a single tab
    fn collect_tab_output(tab: &ProjectTab, idx: usize, is_active: bool) -> Vec<String> {
        let mut info = Vec::new();

        let marker = if is_active { " [ACTIVE]" } else { "" };
        let tab_name = tab
            .project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        info.push(format!("--- Tab {}: {}{} ---", idx + 1, tab_name, marker));
        info.push(format!("Project Root: {}", tab.project_root.display()));

        // Get selected module for this tab
        let selected_module = tab
            .modules_list_state
            .selected()
            .and_then(|i| tab.modules.get(i))
            .map(|s| s.as_str())
            .unwrap_or("<none>");
        info.push(format!("Module: {}", selected_module));
        info.push(format!("Output Lines: {}", tab.command_output.len()));

        if tab.command_output.is_empty() {
            info.push("(No output)".to_string());
        } else {
            // Include the last 100 lines of output to keep it manageable
            let start_idx = if tab.command_output.len() > 100 {
                tab.command_output.len() - 100
            } else {
                0
            };
            if start_idx > 0 {
                info.push(format!(
                    "(Showing last {} lines of {})",
                    tab.command_output.len() - start_idx,
                    tab.command_output.len()
                ));
            }
            for line in &tab.command_output[start_idx..] {
                info.push(line.clone());
            }
        }
        info.push(String::new());

        info
    }

    /// Collect LazyMVN logs
    fn collect_logs() -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== LazyMVN Logs (Current Session) ===".to_string());
        let logs = crate::utils::logger::get_logs_for_debug_report();
        info.push(logs);
        info.push(String::new());
        info
    }

    /// Add debug report footer
    fn add_debug_footer(debug_info: &mut Vec<String>) {
        debug_info.push("=".repeat(80));
        debug_info.push("End of Debug Report".to_string());
        debug_info.push("=".repeat(80));
    }

    /// Copy text to clipboard using platform-specific tools
    fn copy_to_clipboard(&mut self, text: &str, lines: usize, content_type: &str) {
        // Try platform-specific clipboard tools first
        if self.try_platform_clipboard(text, lines, content_type) {
            return;
        }

        // Fallback to arboard if all system tools failed
        self.copy_via_arboard(text, lines, content_type);
    }

    /// Try platform-specific clipboard tools (wl-copy, xclip, xsel, PowerShell, pbcopy)
    fn try_platform_clipboard(&mut self, text: &str, lines: usize, content_type: &str) -> bool {
        #[cfg(target_os = "linux")]
        {
            if Self::try_clipboard_tool("wl-copy", &[], text).is_ok() {
                self.show_clipboard_success(lines, content_type, "wl-copy");
                return true;
            }
            if Self::try_clipboard_tool("xclip", &["-selection", "clipboard"], text).is_ok() {
                self.show_clipboard_success(lines, content_type, "xclip");
                return true;
            }
            if Self::try_clipboard_tool("xsel", &["--clipboard"], text).is_ok() {
                self.show_clipboard_success(lines, content_type, "xsel");
                return true;
            }
        }

        #[cfg(target_os = "windows")]
        {
            if Self::try_clipboard_tool("powershell", &["-Command", "$input | Set-Clipboard"], text)
                .is_ok()
            {
                self.show_clipboard_success(lines, content_type, "PowerShell");
                return true;
            }
            if Self::try_clipboard_tool("clip", &[], text).is_ok() {
                self.show_clipboard_success(lines, content_type, "clip.exe");
                return true;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if Self::try_clipboard_tool("pbcopy", &[], text).is_ok() {
                self.show_clipboard_success(lines, content_type, "pbcopy");
                return true;
            }
        }

        false
    }

    /// Try to copy text using a specific clipboard tool
    fn try_clipboard_tool(tool: &str, args: &[&str], text: &str) -> Result<(), std::io::Error> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new(tool)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
            drop(stdin);
            let status = child.wait()?;
            if status.success() {
                return Ok(());
            }
        }

        Err(std::io::Error::other(format!("{} failed", tool)))
    }

    /// Copy via arboard library (fallback)
    fn copy_via_arboard(&mut self, text: &str, lines: usize, content_type: &str) {
        let clipboard_result = if let Some(ref mut clipboard) = self.clipboard {
            clipboard.set_text(text)
        } else {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    let result = clipboard.set_text(text);
                    self.clipboard = Some(clipboard);
                    result
                }
                Err(e) => {
                    log::error!("Failed to initialize clipboard: {}", e);
                    self.show_clipboard_error(&format!("Clipboard not available: {}", e));
                    return;
                }
            }
        };

        match clipboard_result {
            Ok(()) => {
                self.show_clipboard_success(lines, content_type, "arboard");
            }
            Err(e) => {
                log::error!("Failed to copy {} to clipboard: {}", content_type, e);
                self.show_clipboard_error(&format!("Failed to copy {}: {}", content_type, e));
            }
        }
    }

    /// Show clipboard success message
    fn show_clipboard_success(&mut self, lines: usize, content_type: &str, tool: &str) {
        log::info!("Copied {} via {}", content_type, tool);
        let tab = self.get_active_tab_mut();
        tab.command_output.push(String::new());
        tab.command_output.push(format!(
            "✓ Copied {} ({} lines) to clipboard",
            content_type, lines
        ));
    }

    /// Show clipboard error message
    fn show_clipboard_error(&mut self, error: &str) {
        let tab = self.get_active_tab_mut();
        tab.command_output.push(String::new());
        tab.command_output.push(format!("✗ {}", error));
    }

    /// Get elapsed time of current command in seconds
    pub fn command_elapsed_seconds(&self) -> Option<u64> {
        let tab = self.get_active_tab();
        tab.command_start_time
            .map(|start| start.elapsed().as_secs())
    }

    /// Send desktop notification
    fn send_notification(&self, title: &str, body: &str, success: bool) {
        // Check if notifications are enabled (default: true)
        let tab = self.get_active_tab();
        let enabled = tab.config.notifications_enabled.unwrap_or(true);
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

    // Recent projects methods
    pub fn show_recent_projects(&mut self) {
        log::info!("Showing recent projects popup");
        self.show_projects_popup = true;
        self.projects_filter.clear();
        if self.focus != Focus::Projects {
            self.focus = Focus::Projects;
        }
    }

    pub fn hide_recent_projects(&mut self) {
        log::info!("Hiding recent projects popup");
        self.show_projects_popup = false;
        self.projects_filter.clear();
    }

    /// Get filtered recent projects based on current filter
    pub fn get_filtered_projects(&self) -> Vec<PathBuf> {
        if self.projects_filter.is_empty() {
            self.recent_projects.clone()
        } else {
            let filter = self.projects_filter.to_lowercase();
            self.recent_projects
                .iter()
                .filter(|p| {
                    p.to_string_lossy().to_lowercase().contains(&filter)
                })
                .cloned()
                .collect()
        }
    }

    /// Push character to projects filter
    pub fn push_projects_filter_char(&mut self, ch: char) {
        self.projects_filter.push(ch);
        // Reset selection when filter changes
        if !self.get_filtered_projects().is_empty() {
            self.projects_list_state.select(Some(0));
        }
    }

    /// Pop character from projects filter
    pub fn pop_projects_filter_char(&mut self) {
        self.projects_filter.pop();
        // Reset selection when filter changes
        if !self.get_filtered_projects().is_empty() {
            self.projects_list_state.select(Some(0));
        }
    }

    pub fn select_current_project(&mut self) {
        if let Some(idx) = self.projects_list_state.selected() {
            let filtered_projects = self.get_filtered_projects();
            if let Some(project) = filtered_projects.get(idx) {
                log::info!("Selected project: {:?}", project);
                match self.create_tab(project.clone()) {
                    Ok(tab_idx) => {
                        log::info!("Opened project in tab {}", tab_idx);
                    }
                    Err(e) => {
                        log::error!("Failed to create tab: {}", e);
                        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                            tab.command_output = vec![format!("❌ {}", e)];
                        }
                    }
                }
                self.hide_recent_projects();
            }
        }
    }

    pub fn next_project(&mut self) {
        let filtered = self.get_filtered_projects();
        if filtered.is_empty() {
            return;
        }
        let i = match self.projects_list_state.selected() {
            Some(i) => (i + 1) % filtered.len(),
            None => 0,
        };
        self.projects_list_state.select(Some(i));
    }

    pub fn previous_project(&mut self) {
        let filtered = self.get_filtered_projects();
        if filtered.is_empty() {
            return;
        }
        let i = match self.projects_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    filtered.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.projects_list_state.select(Some(i));
    }

    /// Edit the project configuration file in the system editor
    pub fn edit_config(&mut self) {
        let config_path = {
            let tab = self.get_active_tab();
            crate::core::config::get_project_config_path(&tab.project_root)
        };

        // Generate config if it doesn't exist
        if !config_path.exists() {
            log::info!("Configuration file not found, creating: {:?}", config_path);
            let project_root = self.get_active_tab().project_root.clone();
            match crate::core::config::create_project_config(&project_root) {
                Ok(path) => {
                    log::info!("Created config file at: {:?}", path);
                }
                Err(e) => {
                    log::error!("Failed to generate config file: {}", e);
                    let tab = self.get_active_tab_mut();
                    tab.command_output = vec![
                        format!("❌ Failed to generate config file: {}", e),
                        String::new(),
                        "Please run 'lazymvn --setup' to create configuration".to_string(),
                    ];
                    return;
                }
            }
        }

        // Get system editor
        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| {
                // Platform-specific defaults
                if cfg!(target_os = "windows") {
                    "notepad".to_string()
                } else {
                    "vi".to_string()
                }
            });

        log::info!("Opening config with editor: {}", editor);
        let tab = self.get_active_tab_mut();
        tab.command_output = vec![
            format!("📝 Opening configuration with {}...", editor),
            format!("   File: {}", config_path.display()),
            String::new(),
            "The TUI will resume after you close the editor.".to_string(),
        ];

        // We need to exit raw mode before opening the editor
        self.editor_command = Some((editor, config_path.to_string_lossy().to_string()));
    }

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

    /// Apply a history entry: select module, set profiles, flags, and run command
    /// Automatically switches to the correct project tab or creates a new one if needed
    pub fn apply_history_entry(&mut self, entry: crate::features::history::HistoryEntry) {
        log::info!(
            "Applying history entry for project: {:?}, module: {}",
            entry.project_root,
            entry.module
        );

        // Check if we need to switch project context
        let current_project_root = self.get_active_tab().project_root.clone();

        if current_project_root != entry.project_root {
            log::info!(
                "History entry is for a different project. Current: {:?}, Required: {:?}",
                current_project_root,
                entry.project_root
            );

            // Check if there's already a tab with this project
            let existing_tab_index = self
                .tabs
                .iter()
                .position(|tab| tab.project_root == entry.project_root);

            if let Some(tab_index) = existing_tab_index {
                // Switch to existing tab
                log::info!("Switching to existing tab at index {}", tab_index);
                self.active_tab_index = tab_index;
            } else {
                // Try to open a new tab with this project
                log::info!("Opening new tab for project: {:?}", entry.project_root);

                // Load project modules
                match crate::core::project::get_project_modules_for_path(&entry.project_root) {
                    Ok((modules, root)) => {
                        // Load config for this project
                        let config = crate::core::config::load_config(&root);

                        // Create new tab - ProjectTab::new doesn't return Result, it's infallible
                        let new_tab_id = self.tabs.len();
                        let new_tab = crate::ui::state::ProjectTab::new(
                            new_tab_id,
                            root,
                            modules,
                            config,
                        );

                        if self.tabs.len() < 10 {
                            self.tabs.push(new_tab);
                            self.active_tab_index = self.tabs.len() - 1;
                            log::info!("New tab created successfully");
                        } else {
                            log::error!("Cannot create new tab: maximum 10 tabs reached");
                            let tab = self.get_active_tab_mut();
                            tab.command_output = vec![
                                format!(
                                    "Error: Cannot switch to project {:?}",
                                    entry.project_root
                                ),
                                "Maximum 10 tabs reached. Close some tabs first.".to_string(),
                            ];
                            return;
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to load project modules: {}", e);
                        let tab = self.get_active_tab_mut();
                        tab.command_output = vec![
                            format!("Error: Failed to load project {:?}: {}", entry.project_root, e),
                        ];
                        return;
                    }
                }
            }
        }

        // Now we're in the correct project context, apply the command
        let tab = self.get_active_tab_mut();

        // Find and select the module
        if let Some(module_idx) = tab.modules.iter().position(|m| m == &entry.module) {
            tab.modules_list_state.select(Some(module_idx));
            log::debug!("Selected module at index {}", module_idx);
        } else {
            log::warn!("Module '{}' not found in project", entry.module);
            tab.command_output = vec![format!(
                "Error: Module '{}' not found in this project",
                entry.module
            )];
            return;
        }

        // Set profiles
        for profile in &mut tab.profiles {
            if entry.profiles.contains(&profile.name) {
                // Should be enabled
                if !profile.is_active() {
                    profile.state = crate::ui::state::ProfileState::ExplicitlyEnabled;
                }
            } else {
                // Should be disabled or default
                if profile.auto_activated {
                    // If auto-activated but not in history, explicitly disable
                    profile.state = crate::ui::state::ProfileState::ExplicitlyDisabled;
                } else {
                    // Otherwise set to default
                    profile.state = crate::ui::state::ProfileState::Default;
                }
            }
        }

        // Set flags
        for flag in &mut tab.flags {
            flag.enabled = entry.flags.contains(&flag.name);
        }

        // Switch to modules view
        self.switch_to_modules();

        // Execute the command
        let goal_parts: Vec<&str> = entry.goal.split_whitespace().collect();
        self.run_selected_module_command(&goal_parts);

        log::info!("History entry applied and command executed");
    }

    /// Get filtered history entries based on current filter
    pub fn get_filtered_history(&self) -> Vec<crate::features::history::HistoryEntry> {
        if self.history_filter.is_empty() {
            self.command_history.entries().to_vec()
        } else {
            let filter = self.history_filter.to_lowercase();
            self.command_history
                .entries()
                .iter()
                .filter(|entry| {
                    entry.module.to_lowercase().contains(&filter)
                        || entry.goal.to_lowercase().contains(&filter)
                        || entry.profiles.iter().any(|p| p.to_lowercase().contains(&filter))
                        || entry.flags.iter().any(|f| f.to_lowercase().contains(&filter))
                })
                .cloned()
                .collect()
        }
    }

    /// Push character to history filter
    pub fn push_history_filter_char(&mut self, ch: char) {
        self.history_filter.push(ch);
        // Reset selection when filter changes
        if !self.get_filtered_history().is_empty() {
            self.history_list_state.select(Some(0));
        }
    }

    /// Pop character from history filter
    pub fn pop_history_filter_char(&mut self) {
        self.history_filter.pop();
        // Reset selection when filter changes
        if !self.get_filtered_history().is_empty() {
            self.history_list_state.select(Some(0));
        }
    }

    /// Show save favorite dialog with current context
    pub fn show_save_favorite_dialog_from_current(&mut self) {
        if let Some(module) = self.selected_module() {
            let tab = self.get_active_tab();

            // Get active profiles
            let active_profiles: Vec<String> = tab
                .profiles
                .iter()
                .filter(|p| p.is_active())
                .map(|p| p.name.clone())
                .collect();

            // Get enabled flags
            let enabled_flags: Vec<String> = tab
                .flags
                .iter()
                .filter(|f| f.enabled)
                .map(|f| f.name.clone())
                .collect();

            // Create a pending favorite entry
            let entry = crate::features::history::HistoryEntry::new(
                self.get_active_tab().project_root.clone(),
                module.to_string(),
                "".to_string(), // Will be filled with goal when saving
                active_profiles,
                enabled_flags,
            );

            self.pending_favorite = Some(entry);
            self.favorite_name_input.clear();
            self.show_save_favorite_popup = true;
            log::info!("Opened save favorite dialog");
        }
    }

    /// Save the pending favorite with the entered name
    pub fn save_pending_favorite(&mut self, goal: String) {
        if let Some(mut entry) = self.pending_favorite.take() {
            entry.goal = goal;

            let favorite = crate::features::favorites::Favorite::new(
                self.favorite_name_input.clone(),
                entry.module,
                entry.goal,
                entry.profiles,
                entry.flags,
            );

            self.favorites.add(favorite);
            self.show_save_favorite_popup = false;
            self.favorite_name_input.clear();
            log::info!("Favorite saved successfully");
        }
    }

    /// Cancel saving favorite
    pub fn cancel_save_favorite(&mut self) {
        self.show_save_favorite_popup = false;
        self.favorite_name_input.clear();
        self.pending_favorite = None;
        log::info!("Canceled save favorite");
    }

    /// Apply a favorite: select module, set profiles, flags, and show in modules view
    pub fn apply_favorite(&mut self, favorite: &crate::features::favorites::Favorite) {
        log::info!("Applying favorite: {}", favorite.name);

        let tab = self.get_active_tab_mut();

        // Find and select the module
        if let Some(module_idx) = tab.modules.iter().position(|m| m == &favorite.module) {
            tab.modules_list_state.select(Some(module_idx));
            log::debug!("Selected module at index {}", module_idx);
        } else {
            log::warn!("Module '{}' not found in current project", favorite.module);
            tab.command_output = vec![format!(
                "Error: Module '{}' not found in current project",
                favorite.module
            )];
            return;
        }

        // Set profiles
        for profile in &mut tab.profiles {
            if favorite.profiles.contains(&profile.name) {
                if !profile.is_active() {
                    profile.state = crate::ui::state::ProfileState::ExplicitlyEnabled;
                }
            } else if profile.auto_activated {
                profile.state = crate::ui::state::ProfileState::ExplicitlyDisabled;
            } else {
                profile.state = crate::ui::state::ProfileState::Default;
            }
        }

        // Set flags
        for flag in &mut tab.flags {
            flag.enabled = favorite.flags.contains(&flag.name);
        }

        // Switch to modules view
        self.switch_to_modules();

        // Execute the command
        let goal_parts: Vec<&str> = favorite.goal.split_whitespace().collect();
        self.run_selected_module_command(&goal_parts);

        log::info!("Favorite applied and command executed");
    }

    /// Get filtered favorites based on current filter
    pub fn get_filtered_favorites(&self) -> Vec<crate::features::favorites::Favorite> {
        if self.favorites_filter.is_empty() {
            self.favorites.list().to_vec()
        } else {
            let filter = self.favorites_filter.to_lowercase();
            self.favorites
                .list()
                .iter()
                .filter(|fav| {
                    fav.name.to_lowercase().contains(&filter)
                        || fav.module.to_lowercase().contains(&filter)
                        || fav.goal.to_lowercase().contains(&filter)
                        || fav.profiles.iter().any(|p| p.to_lowercase().contains(&filter))
                        || fav.flags.iter().any(|f| f.to_lowercase().contains(&filter))
                })
                .cloned()
                .collect()
        }
    }

    /// Push character to favorites filter
    pub fn push_favorites_filter_char(&mut self, ch: char) {
        self.favorites_filter.push(ch);
        // Reset selection when filter changes
        if !self.get_filtered_favorites().is_empty() {
            self.favorites_list_state.select(Some(0));
        }
    }

    /// Pop character from favorites filter
    pub fn pop_favorites_filter_char(&mut self) {
        self.favorites_filter.pop();
        // Reset selection when filter changes
        if !self.get_filtered_favorites().is_empty() {
            self.favorites_list_state.select(Some(0));
        }
    }

    /// Delete the selected favorite
    pub fn delete_selected_favorite(&mut self) {
        if let Some(selected) = self.favorites_list_state.selected()
            && let Some(removed) = self.favorites.remove(selected)
        {
            log::info!("Deleted favorite: {}", removed.name);

            // Adjust selection
            let new_len = self.favorites.list().len();
            if new_len == 0 {
                self.favorites_list_state.select(None);
            } else if selected >= new_len {
                self.favorites_list_state.select(Some(new_len - 1));
            }
        }
    }

    // Spring Boot starter methods
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

    // Custom goals popup methods
    pub fn show_custom_goals_popup(&mut self) {
        log::info!("Showing custom goals popup");
        
        let tab = self.get_active_tab_mut();
        if tab.custom_goals.is_empty() {
            log::warn!("No custom goals defined in configuration");
            tab.command_output = vec![
                "No custom goals defined.".to_string(),
                "Add goals to your lazymvn.toml config:".to_string(),
                "[maven]".to_string(),
                "custom_goals = [".to_string(),
                "  { name = \"Format\", goal = \"formatter:format\" }".to_string(),
                "]".to_string(),
            ];
            return;
        }

        self.show_custom_goals_popup = true;

        // Select first goal by default
        let tab = self.get_active_tab_mut();
        if !tab.custom_goals.is_empty() {
            tab.custom_goals_list_state.select(Some(0));
        }
    }

    pub fn close_custom_goals_popup(&mut self) {
        log::info!("Closing custom goals popup");
        self.show_custom_goals_popup = false;
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

    pub fn show_help_popup(&mut self) {
        log::info!("Showing help popup");
        self.show_help_popup = true;
        self.help_search_query.clear();
        // Select first item
        self.help_list_state.select(Some(0));
    }

    pub fn hide_help_popup(&mut self) {
        log::info!("Hiding help popup");
        self.show_help_popup = false;
        self.help_search_query.clear();
    }

    pub fn select_and_run_starter(&mut self) {
        if let Some(idx) = self.starters_list_state.selected() {
            let filtered = self.get_filtered_starter_candidates();

            if let Some(fqcn) = filtered.get(idx) {
                log::info!("Selected starter: {}", fqcn);

                let fqcn_clone = fqcn.clone();
                let tab = self.get_active_tab_mut();
                let project_root = tab.project_root.clone();

                // Create a new starter entry if not already cached
                if !tab
                    .starters_cache
                    .starters
                    .iter()
                    .any(|s| s.fully_qualified_class_name == fqcn_clone)
                {
                    let label = fqcn_clone
                        .split('.')
                        .next_back()
                        .unwrap_or(&fqcn_clone)
                        .to_string();
                    let is_default = tab.starters_cache.starters.is_empty();
                    let starter = crate::features::starters::Starter::new(
                        fqcn_clone.clone(),
                        label,
                        is_default,
                    );
                    tab.starters_cache.add_starter(starter);

                    // Save the cache
                    if let Err(e) = tab.starters_cache.save(&project_root) {
                        log::error!("Failed to save starters cache: {}", e);
                    }
                }

                // Update last used
                tab.starters_cache.set_last_used(fqcn_clone.clone());
                if let Err(e) = tab.starters_cache.save(&project_root) {
                    log::error!("Failed to save last used starter: {}", e);
                }

                // Run the starter
                self.run_spring_boot_starter(&fqcn_clone);
                self.hide_starter_selector();
            }
        }
    }

    pub fn run_spring_boot_starter(&mut self, fqcn: &str) {
        log::info!("Running Spring Boot starter: {}", fqcn);

        let module = self.selected_module();
        let tab = self.get_active_tab();
        let project_root = tab.project_root.clone();

        // Detect Spring Boot capabilities and decide launch strategy
        match crate::maven::detect_spring_boot_capabilities(&project_root, module) {
            Ok(detection) => {
                let strategy = self.decide_launch_strategy(&detection);
                let active_profiles = self.collect_active_maven_profiles();
                let jvm_args = self.build_jvm_args_for_launcher();

                self.execute_launch_command(
                    strategy,
                    fqcn,
                    &active_profiles,
                    &jvm_args,
                    detection.packaging.as_deref(),
                    detection.spring_boot_version.as_deref(),
                );
            }
            Err(e) => {
                self.handle_detection_error(e.to_string(), fqcn);
            }
        }
    }

    /// Decide launch strategy based on detection and configuration
    fn decide_launch_strategy(
        &self,
        detection: &SpringBootDetection,
    ) -> crate::maven::LaunchStrategy {
        let tab = self.get_active_tab();
        let launch_mode = tab
            .config
            .launch_mode
            .unwrap_or(crate::core::config::LaunchMode::Auto);
        let strategy = crate::maven::decide_launch_strategy(detection, launch_mode);

        log::info!(
            "Launch strategy decided: {:?} (mode={:?}, has_sb_plugin={}, packaging={:?})",
            strategy,
            launch_mode,
            detection.has_spring_boot_plugin,
            detection.packaging
        );

        strategy
    }

    /// Collect active Maven profile names
    fn collect_active_maven_profiles(&self) -> Vec<String> {
        let tab = self.get_active_tab();
        tab.profiles
            .iter()
            .filter_map(|p| p.to_maven_arg())
            .collect()
    }

    /// Execute the launch command with the given parameters
    fn execute_launch_command(
        &mut self,
        strategy: crate::maven::LaunchStrategy,
        fqcn: &str,
        active_profiles: &[String],
        jvm_args: &[String],
        packaging: Option<&str>,
        spring_boot_version: Option<&str>,
    ) {
        let command_parts = crate::maven::build_launch_command(
            strategy,
            Some(fqcn),
            active_profiles,
            jvm_args,
            packaging,
            spring_boot_version,
        );

        let args: Vec<&str> = command_parts.iter().map(|s| s.as_str()).collect();
        let use_file_flag = strategy == crate::maven::LaunchStrategy::ExecJava;

        self.run_selected_module_command_with_options(&args, use_file_flag);
    }

    /// Handle Spring Boot detection error with fallback
    fn handle_detection_error(&mut self, error: String, fqcn: &str) {
        log::error!("Failed to detect Spring Boot capabilities: {}", error);

        {
            let tab = self.get_active_tab_mut();
            tab.command_output = vec![
                format!("Error detecting launch strategy: {}", error),
                String::new(),
                "Falling back to spring-boot:run...".to_string(),
            ];
        }

        // Fallback to traditional spring-boot:run
        let main_class_arg = format!("-Dspring-boot.run.mainClass={}", fqcn);
        let args = vec!["spring-boot:run", &main_class_arg];
        self.run_selected_module_command_with_options(&args, false);
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

    // Package selector methods

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
                    String::new(),
                    "Configuration saved to lazymvn.toml".to_string(),
                    format!("[logging.packages]"),
                    format!("  name = \"{}\"", package_name),
                    format!("  level = \"{}\"", level),
                    String::new(),
                    "You can edit the level in lazymvn.toml if needed.".to_string(),
                ];
                true
            }
            Err(e) => {
                log::error!("Failed to save config: {}", e);
                tab.command_output = vec![
                    format!("✗ Failed to save configuration: {}", e),
                ];
                false
            }
        }
    }

    // Module preferences methods

    /// Save current profiles and flags for the selected module
    pub fn save_module_preferences(&mut self) {
        let module = self.selected_module().map(|m| m.to_string());
        let enabled_flags = self.enabled_flag_names();
        let tab = self.get_active_tab_mut();
        if let Some(module) = module.as_deref() {
            // Save only explicitly set profiles (not Default state)
            let explicit_profiles: Vec<String> = tab
                .profiles
                .iter()
                .filter_map(|p| match p.state {
                    ProfileState::ExplicitlyEnabled => Some(p.name.clone()),
                    ProfileState::ExplicitlyDisabled => Some(format!("!{}", p.name)),
                    ProfileState::Default => None,
                })
                .collect();

            let prefs = crate::core::config::ModulePreferences {
                active_profiles: explicit_profiles.clone(),
                enabled_flags,
            };

            log::info!(
                "Saving preferences for module '{}': profiles={:?}, flags={:?}",
                module,
                prefs.active_profiles,
                prefs.enabled_flags
            );

            tab.module_preferences
                .set_module_prefs(module.to_string(), prefs);

            if let Err(e) = tab.module_preferences.save(&tab.project_root) {
                log::error!("Failed to save module preferences: {}", e);
            }
        }
    }

    /// Load preferences for the selected module
    pub fn load_module_preferences(&mut self) {
        let module = self.selected_module().map(|m| m.to_string());
        let tab = self.get_active_tab_mut();
        if let Some(module) = module.as_deref() {
            if let Some(prefs) = tab.module_preferences.get_module_prefs(module) {
                log::info!(
                    "Loading preferences for module '{}': profiles={:?}, flags={:?}",
                    module,
                    prefs.active_profiles,
                    prefs.enabled_flags
                );

                // Restore profile states
                for profile in &mut tab.profiles {
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
                for flag in &mut tab.flags {
                    flag.enabled = prefs.enabled_flags.contains(&flag.flag);
                }
            } else {
                log::debug!("No saved preferences for module '{}'", module);
                // Reset all profiles to Default state
                for profile in &mut tab.profiles {
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
