//! TUI State management
//!
//! This module manages the state of the terminal UI including module selection,
//! profiles, flags, command execution, and output display.

// Sub-modules
mod project_tab;
mod tabs;
mod navigation;
mod commands;
mod profiles;
mod flags;
mod search;

pub use project_tab::ProjectTab;

// Re-export types

use crate::maven;
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

    // Spring Boot starters UI (global UI state, cache is per-tab)
    pub show_starter_selector: bool,
    pub show_starter_manager: bool,
    pub starter_candidates: Vec<String>,
    pub starter_filter: String,
    pub starters_list_state: ListState,

    // Clipboard - keep it alive to prevent "dropped too quickly" errors (global)
    clipboard: Option<arboard::Clipboard>,

    // Command history (global)
    pub command_history: crate::features::history::CommandHistory,
    pub show_history_popup: bool,
    pub history_list_state: ListState,

    // Favorites (global)
    pub favorites: crate::features::favorites::Favorites,
    pub show_favorites_popup: bool,
    pub favorites_list_state: ListState,
    pub show_save_favorite_popup: bool,
    pub favorite_name_input: String,
    pub pending_favorite: Option<crate::features::history::HistoryEntry>,

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
    pub fn new(modules: Vec<String>, project_root: PathBuf, config: crate::core::config::Config) -> Self {
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

            show_starter_selector: false,
            show_starter_manager: false,
            starter_candidates: Vec::new(),
            starter_filter: String::new(),
            starters_list_state: ListState::default(),

            clipboard: arboard::Clipboard::new().ok(),

            command_history: crate::features::history::CommandHistory::load(),
            show_history_popup: false,
            history_list_state: ListState::default(),

            favorites: crate::features::favorites::Favorites::load(),
            show_favorites_popup: false,
            favorites_list_state: ListState::default(),
            show_save_favorite_popup: false,
            favorite_name_input: String::new(),
            pending_favorite: None,

            editor_command: None,
        }
    }








































    // Live search - performs search as user types without storing in history


    // Module output management
    pub(crate) fn sync_selected_module_output(&mut self) {
        let module = self.selected_module().map(|m| m.to_string());
        {
            let tab = self.get_active_tab_mut();
            if let Some(module) = module.as_deref() {
                if let Some(module_output) = tab.module_outputs.get(module) {
                    tab.command_output = module_output.lines.clone();
                    tab.output_offset = module_output.scroll_offset;
                } else {
                    tab.command_output.clear();
                    tab.output_offset = 0;
                }
            } else {
                tab.command_output.clear();
                tab.output_offset = 0;
            }
            tab.output_metrics = None;
        }
        self.clamp_output_offset();
        self.refresh_search_matches();
    }















    /// Start loading profiles asynchronously

    /// Kill the currently running Maven process
    pub fn kill_running_process(&mut self) {
        let tab = self.get_active_tab_mut();
        if let Some(pid) = tab.running_process_pid {
            log::info!("Attempting to kill process with PID: {}", pid);
            match maven::kill_process(pid) {
                Ok(()) => {
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push(format!("⚠ Process {} killed by user", pid));
                    tab.is_command_running = false;
                    tab.command_receiver = None;
                    tab.running_process_pid = None;
                    tab.output_metrics = None;
                    self.store_current_module_output();
                }
                Err(e) => {
                    log::error!("Failed to kill process: {}", e);
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push(format!("✗ Failed to kill process: {}", e));
                }
            }
        } else {
            log::warn!("No running process to kill");
        }
    }

    /// Check file watcher and re-run command if files changed
    pub fn check_file_watcher(&mut self) {
        let tab = self.get_active_tab_mut();
        if !tab.watch_enabled || tab.is_command_running {
            return;
        }

        if let Some(watcher) = &mut tab.file_watcher
            && watcher.check_changes()
        {
            log::info!("File changes detected, checking if should re-run command");

            // Clone last command to avoid borrow issues
            let last_cmd = tab.last_command.clone();

            // Check if last command is watchable
            if let Some(last_cmd) = last_cmd {
                let watch_config = tab.config.watch.as_ref().unwrap();

                // Check if this command should trigger auto-reload
                let should_rerun = last_cmd
                    .iter()
                    .any(|arg| watch_config.commands.iter().any(|cmd| arg.contains(cmd)));

                if should_rerun {
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push("🔄 Files changed, reloading...".to_string());
                    tab.command_output.push(String::new());

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
        // Extract data we need from tab first
        let (output_text, lines) = {
            let tab = self.get_active_tab();
            if tab.command_output.is_empty() {
                log::info!("No output to copy");
                let tab = self.get_active_tab_mut();
                tab.command_output.push(String::new());
                tab.command_output.push("⚠ No output to copy".to_string());
                return;
            }
            (tab.command_output.join("\n"), tab.command_output.len())
        };

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
                    let tab = self.get_active_tab_mut();
                    tab.command_output.push(String::new());
                    tab.command_output
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
                    let tab = self.get_active_tab_mut();
                    tab.command_output.push(String::new());
                    tab.command_output
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
                    let tab = self.get_active_tab_mut();
                    tab.command_output.push(String::new());
                    tab.command_output
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
                            let tab = self.get_active_tab_mut();
                            tab.command_output.push(String::new());
                            tab.command_output
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
                            let tab = self.get_active_tab_mut();
                            tab.command_output.push(String::new());
                            tab.command_output
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
                            let tab = self.get_active_tab_mut();
                            tab.command_output.push(String::new());
                            tab.command_output
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
                    let tab = self.get_active_tab_mut();
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push(format!("✗ Clipboard not available: {}", e));
                    return;
                }
            }
        };

        let tab = self.get_active_tab_mut();
        match clipboard_result {
            Ok(()) => {
                log::info!("Copied {} lines to clipboard via arboard", lines);
                tab.command_output.push(String::new());
                tab.command_output
                    .push(format!("✓ Copied {} lines to clipboard", lines));
            }
            Err(e) => {
                log::error!("Failed to copy to clipboard: {}", e);
                tab.command_output.push(String::new());
                tab.command_output.push(format!("✗ Failed to copy: {}", e));
            }
        }
    }

    /// Yank (copy) comprehensive debug information to clipboard
    /// Includes: version info, git hash, logs, output from all tabs, and config file
    pub fn yank_debug_info(&mut self) {
        log::info!("Collecting comprehensive debug information");
        
        let mut debug_info = Vec::new();
        
        // Header
        debug_info.push("=".repeat(80));
        debug_info.push("LazyMVN Debug Report".to_string());
        debug_info.push("=".repeat(80));
        debug_info.push(format!("Generated: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        debug_info.push(String::new());
        
        // Version information
        debug_info.push("=== Version Information ===".to_string());
        debug_info.push(format!("LazyMVN Version: {}", env!("CARGO_PKG_VERSION")));
        
        // Git information (if available from build)
        if let Some(date) = option_env!("VERGEN_BUILD_DATE") {
            debug_info.push(format!("Build Date: {}", date));
        }
        if let Some(branch) = option_env!("VERGEN_GIT_BRANCH") {
            debug_info.push(format!("Git Branch: {}", branch));
        }
        if let Some(sha) = option_env!("VERGEN_GIT_SHA") {
            debug_info.push(format!("Git Commit: {}", sha));
        }
        debug_info.push(String::new());
        
        // System information
        debug_info.push("=== System Information ===".to_string());
        debug_info.push(format!("OS: {}", std::env::consts::OS));
        debug_info.push(format!("Architecture: {}", std::env::consts::ARCH));
        debug_info.push(String::new());
        
        // Configuration file
        debug_info.push("=== Configuration (config.toml) ===".to_string());
        let config_path = crate::core::config::get_project_config_path(&self.get_active_tab().project_root);
        if config_path.exists() {
            debug_info.push(format!("Location: {}", config_path.display()));
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    debug_info.push(content);
                }
                Err(e) => {
                    debug_info.push(format!("Error reading config file: {}", e));
                }
            }
        } else {
            debug_info.push("(No config.toml configuration file found)".to_string());
            debug_info.push(format!("Expected location: {}", config_path.display()));
            debug_info.push("Run 'lazymvn --setup' to create configuration".to_string());
        }
        debug_info.push(String::new());
        
        // Output from all tabs
        debug_info.push("=== Output from All Tabs ===".to_string());
        for (idx, tab) in self.tabs.iter().enumerate() {
            let is_active = idx == self.active_tab_index;
            let marker = if is_active { " [ACTIVE]" } else { "" };
            let tab_name = tab.project_root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            debug_info.push(format!("--- Tab {}: {}{} ---", idx + 1, tab_name, marker));
            debug_info.push(format!("Project Root: {}", tab.project_root.display()));
            
            // Get selected module for this tab
            let selected_module = tab.modules_list_state
                .selected()
                .and_then(|i| tab.modules.get(i))
                .map(|s| s.as_str())
                .unwrap_or("<none>");
            debug_info.push(format!("Module: {}", selected_module));
            debug_info.push(format!("Output Lines: {}", tab.command_output.len()));
            
            if tab.command_output.is_empty() {
                debug_info.push("(No output)".to_string());
            } else {
                // Include the last 100 lines of output to keep it manageable
                let start_idx = if tab.command_output.len() > 100 {
                    tab.command_output.len() - 100
                } else {
                    0
                };
                if start_idx > 0 {
                    debug_info.push(format!("(Showing last {} lines of {})", 
                        tab.command_output.len() - start_idx, 
                        tab.command_output.len()));
                }
                for line in &tab.command_output[start_idx..] {
                    debug_info.push(line.clone());
                }
            }
            debug_info.push(String::new());
        }
        
        // LazyMVN logs
        debug_info.push("=== LazyMVN Logs ===".to_string());
        let logs = crate::utils::logger::get_all_logs();
        debug_info.push(logs);
        debug_info.push(String::new());
        
        // Footer
        debug_info.push("=".repeat(80));
        debug_info.push("End of Debug Report".to_string());
        debug_info.push("=".repeat(80));
        
        let debug_text = debug_info.join("\n");
        let lines = debug_info.len();
        
        log::info!("Collected {} lines of debug information", lines);
        
        // Copy to clipboard using the same mechanism as yank_output
        #[cfg(target_os = "linux")]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};

            // Try wl-copy (Wayland) first
            if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn()
                && let Some(mut stdin) = child.stdin.take()
                && stdin.write_all(debug_text.as_bytes()).is_ok()
            {
                drop(stdin);
                if child.wait().is_ok() {
                    log::info!("Copied debug info via wl-copy");
                    let tab = self.get_active_tab_mut();
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push(format!("✓ Copied debug report ({} lines) to clipboard", lines));
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
                && stdin.write_all(debug_text.as_bytes()).is_ok()
            {
                drop(stdin);
                if child.wait().is_ok() {
                    log::info!("Copied debug info via xclip");
                    let tab = self.get_active_tab_mut();
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push(format!("✓ Copied debug report ({} lines) to clipboard", lines));
                    return;
                }
            }

            // Try xsel as another X11 fallback
            if let Ok(mut child) = Command::new("xsel")
                .arg("--clipboard")
                .stdin(Stdio::piped())
                .spawn()
                && let Some(mut stdin) = child.stdin.take()
                && stdin.write_all(debug_text.as_bytes()).is_ok()
            {
                drop(stdin);
                if child.wait().is_ok() {
                    log::info!("Copied debug info via xsel");
                    let tab = self.get_active_tab_mut();
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push(format!("✓ Copied debug report ({} lines) to clipboard", lines));
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
                    if stdin.write_all(debug_text.as_bytes()).is_ok() {
                        drop(stdin);
                        if child.wait().is_ok() {
                            log::info!("Copied debug info via PowerShell Set-Clipboard");
                            let tab = self.get_active_tab_mut();
                            tab.command_output.push(String::new());
                            tab.command_output
                                .push(format!("✓ Copied debug report ({} lines) to clipboard", lines));
                            return;
                        }
                    }
                }
            }

            // Try clip.exe as fallback (built-in Windows command)
            if let Ok(mut child) = Command::new("clip").stdin(Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(debug_text.as_bytes()).is_ok() {
                        drop(stdin);
                        if child.wait().is_ok() {
                            log::info!("Copied debug info via clip.exe");
                            let tab = self.get_active_tab_mut();
                            tab.command_output.push(String::new());
                            tab.command_output
                                .push(format!("✓ Copied debug report ({} lines) to clipboard", lines));
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
                    if stdin.write_all(debug_text.as_bytes()).is_ok() {
                        drop(stdin);
                        if child.wait().is_ok() {
                            log::info!("Copied debug info via pbcopy");
                            let tab = self.get_active_tab_mut();
                            tab.command_output.push(String::new());
                            tab.command_output
                                .push(format!("✓ Copied debug report ({} lines) to clipboard", lines));
                            return;
                        }
                    }
                }
            }
        }

        // Fallback to arboard if all system tools failed
        let clipboard_result = if let Some(ref mut clipboard) = self.clipboard {
            clipboard.set_text(debug_text)
        } else {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    let result = clipboard.set_text(debug_text);
                    self.clipboard = Some(clipboard);
                    result
                }
                Err(e) => {
                    log::error!("Failed to initialize clipboard: {}", e);
                    let tab = self.get_active_tab_mut();
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push(format!("✗ Clipboard not available: {}", e));
                    return;
                }
            }
        };

        let tab = self.get_active_tab_mut();
        match clipboard_result {
            Ok(()) => {
                log::info!("Copied debug report to clipboard via arboard");
                tab.command_output.push(String::new());
                tab.command_output
                    .push(format!("✓ Copied debug report ({} lines) to clipboard", lines));
            }
            Err(e) => {
                log::error!("Failed to copy debug report to clipboard: {}", e);
                tab.command_output.push(String::new());
                tab.command_output.push(format!("✗ Failed to copy debug report: {}", e));
            }
        }
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
    pub fn update_output_metrics(&mut self, width: u16) {
        let tab = self.get_active_tab_mut();
        tab.output_area_width = width;
        if width == 0 || tab.command_output.is_empty() {
            tab.output_metrics = None;
            return;
        }
        let width_usize = width as usize;
        tab.output_metrics = Some(OutputMetrics::new(width_usize, &tab.command_output));
    }

    pub fn set_output_view_dimensions(&mut self, height: u16, width: u16) {
        let tab = self.get_active_tab_mut();
        tab.output_view_height = height;
        tab.output_area_width = width;
        self.clamp_output_offset();
        self.apply_pending_center();
        self.ensure_current_match_visible();
    }

    // Scrolling methods
    fn clamp_output_offset(&mut self) {
        let max_offset = self.max_scroll_offset();
        let tab = self.get_active_tab_mut();
        if tab.output_offset > max_offset {
            tab.output_offset = max_offset;
        }
    }

    pub fn scroll_output_lines(&mut self, delta: isize) {
        if !self.should_allow_navigation() {
            return;
        }
        let is_empty = self.get_active_tab().command_output.is_empty();
        if is_empty {
            return;
        }
        let max_offset = self.max_scroll_offset();
        let tab = self.get_active_tab_mut();
        let current = tab.output_offset as isize;
        let next = (current + delta).clamp(0, max_offset as isize) as usize;
        if next != tab.output_offset {
            tab.output_offset = next;
            self.store_current_module_output();
        }
    }

    pub fn scroll_output_pages(&mut self, delta: isize) {
        let tab = self.get_active_tab();
        let page = tab.output_view_height.max(1) as isize;
        self.scroll_output_lines(delta * page);
    }

    pub fn scroll_output_to_start(&mut self) {
        let tab = self.get_active_tab_mut();
        if tab.command_output.is_empty() {
            return;
        }
        tab.output_offset = 0;
        self.store_current_module_output();
    }

    pub fn scroll_output_to_end(&mut self) {
        let max_offset = self.max_scroll_offset();
        let tab = self.get_active_tab_mut();
        tab.output_offset = max_offset;
        self.store_current_module_output();
    }

    fn max_scroll_offset(&self) -> usize {
        let tab = self.get_active_tab();
        let height = tab.output_view_height as usize;
        if height == 0 {
            return 0;
        }
        let total = self.total_display_rows();
        total.saturating_sub(height)
    }

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

    /// Reload configuration and apply changes
    /// Returns true if configuration actually changed
    pub fn reload_config(&mut self, new_config: crate::core::config::Config) -> bool {
        log::info!("Reloading configuration");

        let tab = self.get_active_tab_mut();

        // Check what changed
        let mut changed = false;
        let mut changes = Vec::new();

        // Check launch_mode
        if tab.config.launch_mode != new_config.launch_mode {
            changes.push(format!(
                "  • Launch mode: {:?} → {:?}",
                tab.config.launch_mode, new_config.launch_mode
            ));
            changed = true;
        }

        // Check maven_settings
        if tab.config.maven_settings != new_config.maven_settings {
            changes.push(format!(
                "  • Maven settings: {:?} → {:?}",
                tab.config.maven_settings, new_config.maven_settings
            ));
            changed = true;
        }

        // Check notifications
        if tab.config.notifications_enabled != new_config.notifications_enabled {
            changes.push(format!(
                "  • Notifications: {:?} → {:?}",
                tab.config.notifications_enabled, new_config.notifications_enabled
            ));
            changed = true;
        }

        // Check watch configuration
        let watch_changed = match (&tab.config.watch, &new_config.watch) {
            (Some(old), Some(new)) => {
                old.enabled != new.enabled
                    || old.commands != new.commands
                    || old.patterns != new.patterns
                    || old.debounce_ms != new.debounce_ms
            }
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        };

        if watch_changed {
            changes.push("  • Watch configuration changed".to_string());
            changed = true;

            // Reinitialize file watcher if watch config changed
            if let Some(watch_config) = &new_config.watch {
                if watch_config.enabled {
                    match crate::utils::watcher::FileWatcher::new(
                        &tab.project_root,
                        watch_config.debounce_ms,
                    ) {
                        Ok(watcher) => {
                            tab.file_watcher = Some(watcher);
                            tab.watch_enabled = true;
                            log::info!(
                                "File watcher reinitialized with {} patterns",
                                watch_config.patterns.len()
                            );
                        }
                        Err(e) => {
                            log::error!("Failed to reinitialize file watcher: {}", e);
                            tab.file_watcher = None;
                            tab.watch_enabled = false;
                        }
                    }
                } else {
                    tab.file_watcher = None;
                    tab.watch_enabled = false;
                    log::info!("File watcher disabled");
                }
            } else {
                tab.file_watcher = None;
                tab.watch_enabled = false;
            }
        }

        // Check output configuration
        if tab.config.output != new_config.output {
            changes.push("  • Output configuration changed".to_string());
            changed = true;
        }

        // Check logging configuration
        if tab.config.logging != new_config.logging {
            changes.push("  • Logging configuration changed".to_string());
            changed = true;
        }

        // Apply the new configuration
        tab.config = new_config;

        // Log changes
        if changed {
            log::info!("Configuration changes detected:");
            for change in &changes {
                log::info!("{}", change);
            }
        }

        changed
    }

    /// Apply a history entry: select module, set profiles, flags, and run command
    pub fn apply_history_entry(&mut self, entry: crate::features::history::HistoryEntry) {
        log::info!("Applying history entry for module: {}", entry.module);

        let tab = self.get_active_tab_mut();

        // Find and select the module
        if let Some(module_idx) = tab.modules.iter().position(|m| m == &entry.module) {
            tab.modules_list_state.select(Some(module_idx));
            log::debug!("Selected module at index {}", module_idx);
        } else {
            log::warn!("Module '{}' not found in current project", entry.module);
            tab.command_output = vec![format!(
                "Error: Module '{}' not found in current project",
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

        // Scan for potential starters if candidates list is empty
        if self.starter_candidates.is_empty() {
            let tab = self.get_active_tab();
            self.starter_candidates = crate::features::starters::find_potential_starters(&tab.project_root);
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
                    let starter =
                        crate::features::starters::Starter::new(fqcn_clone.clone(), label, is_default);
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

        // Get selected module
        let module = self.selected_module();

        let tab = self.get_active_tab();
        let project_root = tab.project_root.clone();
        let config_clone = tab.config.clone();

        // Detect Spring Boot capabilities for this module
        match crate::maven::detect_spring_boot_capabilities(&project_root, module) {
            Ok(detection) => {
                // Decide launch strategy based on detection and config
                let launch_mode = config_clone
                    .launch_mode
                    .unwrap_or(crate::core::config::LaunchMode::Auto);
                let strategy = crate::maven::decide_launch_strategy(&detection, launch_mode);

                log::info!(
                    "Launch strategy decided: {:?} (mode={:?}, has_sb_plugin={}, packaging={:?})",
                    strategy,
                    launch_mode,
                    detection.has_spring_boot_plugin,
                    detection.packaging
                );

                let tab = self.get_active_tab();
                // Collect active profile names (those that need to be passed to Maven)
                let active_profiles: Vec<String> = tab
                    .profiles
                    .iter()
                    .filter_map(|p| p.to_maven_arg())
                    .collect();

                // Build JVM args from logging configuration
                let mut jvm_args: Vec<String> = if let Some(ref logging_config) = tab.config.logging {
                    log::debug!("Found logging config with {} packages", logging_config.packages.len());
                    
                    // Convert LoggingPackage to (String, String) tuples for Log4j generation
                    let logging_overrides: Vec<(String, String)> = logging_config
                        .packages
                        .iter()
                        .map(|pkg| (pkg.name.clone(), pkg.level.clone()))
                        .collect();
                    
                    // Generate Log4j 1.x config file if logging overrides exist
                    // This is automatically used by Log4j 1.x applications
                    if !logging_overrides.is_empty() {
                        if let Some(log4j_config_path) = crate::maven::generate_log4j_config(
                            &tab.project_root,
                            &logging_overrides,
                        ) {
                            // Convert path to URL format for Log4j configuration
                            let config_url = if cfg!(windows) {
                                // Windows: file:///C:/path/to/file
                                format!("file:///{}", log4j_config_path.display().to_string().replace('\\', "/"))
                            } else {
                                // Unix: file:///path/to/file
                                format!("file://{}", log4j_config_path.display())
                            };
                            
                            log::info!("Injecting Log4j 1.x configuration: {}", config_url);
                            
                            // Add Log4j configuration argument at the beginning
                            // This ensures Log4j 1.x picks it up before loading default config
                            vec![format!("-Dlog4j.configuration={}", config_url)]
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    log::debug!("No logging config found in tab.config");
                    Vec::new()
                };
                
                // Also add traditional JVM args for Spring Boot (Logback) compatibility
                if let Some(ref logging_config) = tab.config.logging {
                    for pkg in &logging_config.packages {
                        // Add Logback/Spring Boot style logging levels
                        jvm_args.push(format!("-Dlogging.level.{}={}", pkg.name, pkg.level));
                    }
                }

                log::debug!("Generated {} JVM args from logging config", jvm_args.len());
                for arg in &jvm_args {
                    log::debug!("  JVM arg: {}", arg);
                }

                // Generate Spring Boot properties override file if [spring] config exists
                if let Some(ref spring_config) = tab.config.spring {
                    log::debug!("Found spring config with {} properties and {} profiles", 
                        spring_config.properties.len(),
                        spring_config.active_profiles.len()
                    );
                    
                    // Convert SpringProperty to (String, String) tuples
                    let spring_properties: Vec<(String, String)> = spring_config
                        .properties
                        .iter()
                        .map(|prop| (prop.name.clone(), prop.value.clone()))
                        .collect();
                    
                    // Generate Spring properties file
                    if let Some(spring_config_path) = crate::maven::generate_spring_properties(
                        &tab.project_root,
                        &spring_properties,
                        &spring_config.active_profiles,
                    ) {
                        // Convert path to URL format for Spring configuration
                        let config_url = if cfg!(windows) {
                            // Windows: file:///C:/path/to/file
                            format!("file:///{}", spring_config_path.display().to_string().replace('\\', "/"))
                        } else {
                            // Unix: file:///path/to/file
                            format!("file://{}", spring_config_path.display())
                        };
                        
                        log::info!("Injecting Spring Boot properties override: {}", config_url);
                        log::debug!("Spring properties will OVERRIDE project defaults (LazyMVN has the last word)");
                        
                        // Add Spring config location argument
                        // Using spring.config.additional-location ensures our config has highest priority
                        jvm_args.push(format!("-Dspring.config.additional-location={}", config_url));
                        
                        // Log each property for debugging
                        for (name, value) in &spring_properties {
                            log::debug!("  Spring property override: {}={}", name, value);
                        }
                        if !spring_config.active_profiles.is_empty() {
                            log::debug!("  Spring active profiles: {}", spring_config.active_profiles.join(","));
                        }
                    }
                } else {
                    log::debug!("No spring config found in tab.config");
                }

                // Build launch command with the strategy
                let command_parts = crate::maven::build_launch_command(
                    strategy,
                    Some(fqcn),
                    &active_profiles,
                    &jvm_args,
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
                {
                    let tab = self.get_active_tab_mut();
                    tab.command_output = vec![
                        format!("Error detecting launch strategy: {}", e),
                        String::new(),
                        "Falling back to spring-boot:run...".to_string(),
                    ];
                }

                // Fallback to old behavior
                let main_class_arg = format!("-Dspring-boot.run.mainClass={}", fqcn);
                let args = vec!["spring-boot:run", &main_class_arg];
                // Use -pl for spring-boot:run to inherit parent plugin config
                self.run_selected_module_command_with_options(&args, false);
            }
        }
    }

    pub fn run_preferred_starter(&mut self) {
        let tab = self.get_active_tab();
        if let Some(starter) = tab.starters_cache.get_preferred_starter() {
            let fqcn = starter.fully_qualified_class_name.clone();
            log::info!("Running preferred starter: {}", fqcn);
            self.run_spring_boot_starter(&fqcn);
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

        // Start loading profiles to set status to Loading
        state.start_loading_profiles();

        // Now profiles should be in Loading state
        assert!(matches!(
            state.profile_loading_status,
            ProfileLoadingStatus::Loading
        ));
        assert_eq!(state.get_active_tab().profiles.len(), 0);
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
