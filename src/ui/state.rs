use crate::maven;
use crate::ui::keybindings::{CurrentView, Focus, SearchMode};
use crate::ui::search::{SearchMatch, SearchState, collect_search_matches};
use ratatui::widgets::ListState;
use regex::Regex;
use std::{collections::HashMap, path::PathBuf};
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
    pub profiles: Vec<String>,
    pub active_profiles: Vec<String>,
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
}

/// Maven build flags that can be toggled
#[derive(Clone, Debug)]
pub struct BuildFlag {
    pub name: String,
    pub flag: String,
    #[allow(dead_code)]
    pub description: String,
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
                description: "Build dependencies of specified modules".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Also Make Dependents".to_string(),
                flag: "--also-make-dependents".to_string(),
                description: "Build modules that depend on specified modules".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Update Snapshots".to_string(),
                flag: "--update-snapshots".to_string(),
                description: "Force update of snapshot dependencies".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Skip Tests".to_string(),
                flag: "-DskipTests".to_string(),
                description: "Skip running tests".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Offline".to_string(),
                flag: "--offline".to_string(),
                description: "Work offline (don't download dependencies)".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Fail Fast".to_string(),
                flag: "--fail-fast".to_string(),
                description: "Stop at first failure in multi-module build".to_string(),
                enabled: false,
            },
            BuildFlag {
                name: "Fail At End".to_string(),
                flag: "--fail-at-end".to_string(),
                description: "Fail build at end; allow all non-impacted builds to continue"
                    .to_string(),
                enabled: false,
            },
        ];

        let mut state = Self {
            current_view: CurrentView::Modules,
            focus: Focus::Modules,
            modules,
            profiles: vec![],
            active_profiles: vec![],
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
        };
        state.sync_selected_module_output();
        state
    }

    pub fn set_profiles(&mut self, profiles: Vec<String>) {
        self.profiles = profiles;
        if !self.profiles.is_empty() {
            self.profiles_list_state.select(Some(0));
        }
    }

    // Navigation methods
    pub fn next_item(&mut self) {
        match self.current_view {
            CurrentView::Modules => {
                if self.modules.is_empty() {
                    return;
                }
                let i = match self.modules_list_state.selected() {
                    Some(i) => (i + 1) % self.modules.len(),
                    None => 0,
                };
                self.modules_list_state.select(Some(i));
                self.sync_selected_module_output();
            }
            CurrentView::Profiles => {
                if !self.profiles.is_empty() {
                    let i = match self.profiles_list_state.selected() {
                        Some(i) => (i + 1) % self.profiles.len(),
                        None => 0,
                    };
                    self.profiles_list_state.select(Some(i));
                }
            }
            CurrentView::Flags => {
                if !self.flags.is_empty() {
                    let i = match self.flags_list_state.selected() {
                        Some(i) => (i + 1) % self.flags.len(),
                        None => 0,
                    };
                    self.flags_list_state.select(Some(i));
                }
            }
        }
    }

    pub fn previous_item(&mut self) {
        match self.current_view {
            CurrentView::Modules => {
                if self.modules.is_empty() {
                    return;
                }
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
            }
            CurrentView::Profiles => {
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
                }
            }
            CurrentView::Flags => {
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
        }
    }

    pub fn toggle_profile(&mut self) {
        if self.current_view != CurrentView::Profiles {
            return;
        }
        if let Some(selected) = self.profiles_list_state.selected() {
            if let Some(profile) = self.profiles.get(selected) {
                if let Some(pos) = self.active_profiles.iter().position(|p| p == profile) {
                    log::info!("Deactivating profile: {}", profile);
                    self.active_profiles.remove(pos);
                } else {
                    log::info!("Activating profile: {}", profile);
                    self.active_profiles.push(profile.clone());
                }
                log::debug!("Active profiles now: {:?}", self.active_profiles);
            }
        }
    }

    pub fn toggle_flag(&mut self) {
        if self.current_view != CurrentView::Flags {
            return;
        }
        if let Some(selected) = self.flags_list_state.selected() {
            if let Some(flag) = self.flags.get_mut(selected) {
                flag.enabled = !flag.enabled;
                log::info!(
                    "Toggled flag '{}' ({}): {}",
                    flag.name,
                    flag.flag,
                    flag.enabled
                );
            }
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
            .map(|f| f.name.clone())
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

    pub fn switch_to_modules(&mut self) {
        self.current_view = CurrentView::Modules;
        self.focus_modules();
        self.sync_selected_module_output();
    }

    pub fn switch_to_profiles(&mut self) {
        self.current_view = CurrentView::Profiles;
        if self.profiles_list_state.selected().is_none() && !self.profiles.is_empty() {
            self.profiles_list_state.select(Some(0));
        }
        self.focus_modules();
    }

    pub fn switch_to_flags(&mut self) {
        self.current_view = CurrentView::Flags;
        if self.flags_list_state.selected().is_none() && !self.flags.is_empty() {
            self.flags_list_state.select(Some(0));
        }
        self.focus_modules();
    }

    // Focus management
    pub fn focus_modules(&mut self) {
        self.focus = Focus::Modules;
    }

    pub fn focus_output(&mut self) {
        self.focus = Focus::Output;
        self.ensure_current_match_visible();
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
    fn sync_selected_module_output(&mut self) {
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

    #[allow(dead_code)]
    pub fn clear_current_module_output(&mut self) {
        if let Some(module) = self.selected_module().map(|m| m.to_string()) {
            self.command_output.clear();
            self.output_offset = 0;
            self.module_outputs.insert(module, ModuleOutput::default());
        } else {
            self.command_output.clear();
            self.output_offset = 0;
        }
        self.clamp_output_offset();
        self.output_metrics = None;
        self.refresh_search_matches();
    }

    // Command execution
    pub fn run_selected_module_command(&mut self, args: &[&str]) {
        log::debug!("run_selected_module_command called with args: {:?}", args);

        if let Some(module) = self.selected_module().map(|m| m.to_string()) {
            log::info!("Running command for module: {}", module);

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

            log::debug!("Enabled flags: {:?}", enabled_flag_names);
            log::debug!("Active profiles: {:?}", self.active_profiles);

            match maven::execute_maven_command(
                &self.project_root,
                Some(&module),
                args,
                &self.active_profiles,
                self.config.maven_settings.as_deref(),
                &enabled_flags,
            ) {
                Ok(output) => {
                    log::info!(
                        "Command completed successfully, {} lines of output",
                        output.len()
                    );
                    self.command_output = output;
                    self.output_offset = self.command_output.len();

                    // Store metadata about this command execution
                    let module_output = ModuleOutput {
                        lines: self.command_output.clone(),
                        scroll_offset: self.output_offset,
                        command: Some(args.join(" ")),
                        profiles: self.active_profiles.clone(),
                        flags: enabled_flag_names,
                    };
                    self.module_outputs.insert(module, module_output);
                }
                Err(e) => {
                    log::error!("Command failed: {}", e);
                    self.command_output = vec![format!("Error: {e}")];
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
        self.refresh_search_matches();
        self.clamp_output_offset();
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
        if let Some(idx) = next_index {
            if let Some(query) = self.search_history.get(idx) {
                self.search_input = Some(query.clone());
                self.search_history_index = Some(idx);
            }
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

        if keep_current {
            if let Some(existing) = self.search_state.as_ref() {
                current_index = existing.current.min(matches.len().saturating_sub(1));
            }
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
            let desired_offset = if target_row >= view_height / 2 {
                target_row - view_height / 2
            } else {
                0
            };
            let max_offset = total_rows.saturating_sub(view_height);
            self.output_offset = desired_offset.min(max_offset);
            self.store_current_module_output();
        }
        self.pending_center = None;
    }

    fn ensure_current_match_visible(&mut self) {
        if let Some(search) = self.search_state.as_ref() {
            if let Some(current_match) = search.current_match() {
                self.center_on_match(current_match.clone());
            }
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
}

// Helper functions
fn visual_rows(line: &str, width: usize) -> usize {
    if width == 0 {
        return 1;
    }
    let display_width = UnicodeWidthStr::width(line);
    let rows = (display_width + width - 1) / width;
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
