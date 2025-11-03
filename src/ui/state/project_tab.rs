//! Project tab management
//!
//! Each tab represents a complete Maven project with its own state,
//! including modules, profiles, output, and running processes.

use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Instant;

use crate::core::config;
use crate::maven;
use crate::ui::state::{BuildFlag, MavenProfile, ModuleOutput, OutputMetrics};
use crate::utils::watcher::FileWatcher;

/// Status of a command execution
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandExecutionState {
    /// Command is currently running
    Running,
    /// Command completed successfully
    Success,
    /// Command failed
    Failure,
}

/// Information about the last executed command
#[derive(Clone, Debug)]
pub struct LastCommandStatus {
    /// The key that triggered the command (b, c, t, etc.)
    pub command_key: char,
    /// The execution state
    pub state: CommandExecutionState,
}

/// A project tab representing a complete Maven project
pub struct ProjectTab {
    pub id: usize,
    pub project_root: PathBuf,

    // Project data
    pub modules: Vec<String>,
    pub profiles: Vec<MavenProfile>,
    pub flags: Vec<BuildFlag>,
    pub custom_goals: Vec<config::CustomGoal>,

    // List states
    pub modules_list_state: ListState,
    pub profiles_list_state: ListState,
    pub flags_list_state: ListState,
    pub custom_goals_list_state: ListState,

    // Command execution
    pub command_output: Vec<String>,
    pub output_offset: usize,
    pub is_command_running: bool,
    pub command_start_time: Option<Instant>,
    pub running_process_pid: Option<u32>,
    pub command_receiver: Option<mpsc::Receiver<maven::CommandUpdate>>,
    pub last_command_status: Option<LastCommandStatus>,

    // Module outputs cache
    pub module_outputs: HashMap<String, ModuleOutput>,

    // Project config
    pub config: config::Config,
    pub module_preferences: config::ProjectPreferences,

    // File watcher
    pub file_watcher: Option<FileWatcher>,
    pub last_command: Option<Vec<String>>,
    pub watch_enabled: bool,
    pub pending_watch_rerun: bool,

    // Git info
    pub git_branch: Option<String>,

    // UI state
    pub output_view_height: u16,
    pub output_area_width: u16,
    pub output_metrics: Option<OutputMetrics>,

    // Spring Boot starters (tab-specific)
    pub starters_cache: crate::features::starters::StartersCache,
}

impl ProjectTab {
    /// Create a new project tab
    pub fn new(
        id: usize,
        project_root: PathBuf,
        modules: Vec<String>,
        config: config::Config,
    ) -> Self {
        // Get git branch
        let git_branch = crate::utils::get_git_branch(&project_root);

        // Load module preferences
        let module_preferences = config::ProjectPreferences::load(&project_root);

        // Initialize profiles
        let profiles = Vec::new(); // Will be loaded asynchronously

        // Initialize flags (built-in flags)
        let mut flags = vec![
            BuildFlag {
                name: "Work offline".to_string(),
                flag: "-o".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Force update snapshots".to_string(),
                flag: "-U".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Debug output".to_string(),
                flag: "-X".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Skip tests".to_string(),
                flag: "-DskipTests".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Build with 4 threads".to_string(),
                flag: "-T 4".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Build dependencies".to_string(),
                flag: "--also-make".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Build dependents".to_string(),
                flag: "--also-make-dependents".to_string(),
                enabled: false,
            },
        ];

        // Add custom flags from configuration
        if let Some(maven_config) = &config.maven {
            for custom_flag in &maven_config.custom_flags {
                flags.push(BuildFlag {
                    name: custom_flag.name.clone(),
                    flag: custom_flag.flag.clone(),
                    enabled: custom_flag.enabled,
                });
            }
        }

        // Load custom goals from configuration
        let custom_goals = if let Some(maven_config) = &config.maven {
            maven_config.custom_goals.clone()
        } else {
            Vec::new()
        };

        // Initialize list states
        let mut modules_list_state = ListState::default();
        if !modules.is_empty() {
            modules_list_state.select(Some(0));
        }

        let mut flags_list_state = ListState::default();
        if !flags.is_empty() {
            flags_list_state.select(Some(0));
        }

        let mut custom_goals_list_state = ListState::default();
        if !custom_goals.is_empty() {
            custom_goals_list_state.select(Some(0));
        }

        // Create file watcher if enabled in config
        let (file_watcher, watch_enabled) = if let Some(watch_config) = &config.watch {
            if watch_config.enabled {
                match FileWatcher::new(
                    &project_root,
                    watch_config.patterns.clone(),
                    watch_config.debounce_ms,
                ) {
                    Ok(watcher) => {
                        log::info!("File watcher initialized for tab {}", id);
                        (Some(watcher), true)
                    }
                    Err(e) => {
                        log::error!("Failed to initialize file watcher for tab {}: {}", id, e);
                        (None, false)
                    }
                }
            } else {
                (None, false)
            }
        } else {
            (None, false)
        };
        // Load starters cache for this project (will auto-scan if empty)
        let starters_cache = crate::features::starters::StartersCache::load_or_scan(&project_root);

        Self {
            id,
            project_root,
            modules,
            profiles,
            flags,
            custom_goals,
            modules_list_state,
            profiles_list_state: ListState::default(),
            flags_list_state,
            custom_goals_list_state,
            command_output: vec!["Ready. Select a module and press a command key.".to_string()],
            output_offset: 0,
            is_command_running: false,
            command_start_time: None,
            running_process_pid: None,
            command_receiver: None,
            last_command_status: None,
            module_outputs: HashMap::new(),
            config,
            module_preferences,
            file_watcher,
            last_command: None,
            watch_enabled,
            pending_watch_rerun: false,
            git_branch,
            output_view_height: 0,
            output_area_width: 0,
            output_metrics: None,
            starters_cache,
        }
    }

    /// Get the display title for this tab
    pub fn get_title(&self) -> String {
        let project_name = self
            .project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("???");

        // Optionally include git branch
        if let Some(ref branch) = self.git_branch {
            format!("{} ({})", project_name, branch)
        } else {
            project_name.to_string()
        }
    }

    /// Get a short title for the tab (max 20 chars)
    #[allow(dead_code)]
    pub fn get_short_title(&self, max_len: usize) -> String {
        let title = self.get_title();
        if title.len() <= max_len {
            title
        } else {
            format!("{}...", &title[..max_len.saturating_sub(3)])
        }
    }

    /// Check if this tab has a running process
    pub fn has_running_process(&self) -> bool {
        self.is_command_running && self.running_process_pid.is_some()
    }

    /// Cleanup resources (kill process, save preferences)
    pub fn cleanup(&mut self) {
        log::info!("Cleaning up tab {} ({})", self.id, self.get_title());

        // Kill running process
        if let Some(pid) = self.running_process_pid {
            log::info!("Killing process {} in tab {}", pid, self.id);
            match maven::kill_process(pid) {
                Ok(()) => {
                    log::info!("Successfully killed process {} in tab {}", pid, self.id);
                }
                Err(e) => {
                    log::error!("Failed to kill process {} in tab {}: {}", pid, self.id, e);
                }
            }
            self.running_process_pid = None;
            self.is_command_running = false;
        }

        // Save module preferences
        if let Err(e) = self.module_preferences.save(&self.project_root) {
            log::error!(
                "Failed to save module preferences for tab {}: {}",
                self.id,
                e
            );
        }

        log::info!("Tab {} cleanup completed", self.id);
    }

    /// Get the currently selected module
    #[allow(dead_code)]
    pub fn get_selected_module(&self) -> Option<&String> {
        self.modules_list_state
            .selected()
            .and_then(|i| self.modules.get(i))
    }

    /// Get the currently selected module index
    #[allow(dead_code)]
    pub fn get_selected_module_index(&self) -> Option<usize> {
        self.modules_list_state.selected()
    }
}

impl Drop for ProjectTab {
    fn drop(&mut self) {
        // Ensure cleanup on drop
        if self.has_running_process() {
            log::warn!("Tab {} dropped with running process, cleaning up", self.id);
            self.cleanup();
        }
    }
}
