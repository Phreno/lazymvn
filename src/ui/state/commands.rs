//! Maven command execution management
//!
//! This module handles executing Maven commands, collecting output,
//! and managing command state.

use super::{TuiState, ModuleOutput};
use crate::maven;
use std::time::Instant;
use std::sync::mpsc;

impl TuiState {
    /// Get list of enabled flag names
    pub fn enabled_flag_names(&self) -> Vec<String> {
        let tab = self.get_active_tab();
        tab.flags
            .iter()
            .filter(|f| f.enabled)
            .map(|f| f.flag.clone()) // Use flag.flag instead of flag.name
            .collect()
    }

    /// Get the last executed command for the selected module
    pub fn get_last_executed_command(&self) -> Option<(String, Vec<String>, Vec<String>)> {
        let tab = self.get_active_tab();
        self.selected_module().and_then(|module| {
            tab.module_outputs.get(module).and_then(|output| {
                output
                    .command
                    .clone()
                    .map(|cmd| (cmd, output.profiles.clone(), output.flags.clone()))
            })
        })
    }

    /// Get list of active profile names for display
    pub fn active_profile_names(&self) -> Vec<String> {
        let tab = self.get_active_tab();
        tab.profiles
            .iter()
            .filter(|p| p.is_active())
            .map(|p| p.name.clone())
            .collect()
    }

    /// Get the current output context (command, profiles, flags)
    pub fn current_output_context(&self) -> Option<(String, Vec<String>, Vec<String>)> {
        let tab = self.get_active_tab();
        self.selected_module().and_then(|module| {
            tab.module_outputs.get(module).and_then(|output| {
                output
                    .command
                    .clone()
                    .map(|cmd| (cmd, output.profiles.clone(), output.flags.clone()))
            })
        })
    }

    /// Store the current command output for the selected module
    pub(super) fn store_current_module_output(&mut self) {
        let module = self.selected_module().map(|m| m.to_string());
        if let Some(module) = module {
            let tab = self.get_active_tab_mut();
            // Get the current execution context from the most recent command
            let module_output = if let Some(existing) = tab.module_outputs.get(&module) {
                ModuleOutput {
                    lines: tab.command_output.clone(),
                    scroll_offset: tab.output_offset,
                    command: existing.command.clone(),
                    profiles: existing.profiles.clone(),
                    flags: existing.flags.clone(),
                }
            } else {
                ModuleOutput {
                    lines: tab.command_output.clone(),
                    scroll_offset: tab.output_offset,
                    ..Default::default()
                }
            };
            tab.module_outputs.insert(module.to_string(), module_output);
        }
    }

    /// Run Maven command for the selected module
    pub fn run_selected_module_command(&mut self, args: &[&str]) {
        self.run_selected_module_command_with_options(args, false);
    }

    /// Run Maven command with option to use -f instead of -pl
    pub fn run_selected_module_command_with_options(&mut self, args: &[&str], use_file_flag: bool) {
        log::debug!(
            "run_selected_module_command called with args: {:?}, use_file_flag: {}",
            args,
            use_file_flag
        );

        let tab = self.get_active_tab_mut();

        // Don't start a new command if one is already running
        if tab.is_command_running {
            log::warn!("Command already running, ignoring new command request");
            return;
        }

        let module = self.selected_module().map(|m| m.to_string());
        let tab = self.get_active_tab_mut();

        if let Some(module) = module {
            log::info!("Running async command for module: {}", module);

            // Collect enabled flags
            let enabled_flags: Vec<String> = tab
                .flags
                .iter()
                .filter(|f| f.enabled)
                .map(|f| f.flag.clone())
                .collect();

            let enabled_flag_names: Vec<String> = tab
                .flags
                .iter()
                .filter(|f| f.enabled)
                .map(|f| f.name.clone())
                .collect();

            // Collect profiles that need to be passed to Maven
            // Only profiles that are not in Default state
            let profile_args: Vec<String> = tab
                .profiles
                .iter()
                .filter_map(|p| p.to_maven_arg())
                .collect();

            // Get list of active profile names for display
            let active_profile_names: Vec<String> = tab
                .profiles
                .iter()
                .filter(|p| p.is_active())
                .map(|p| p.name.clone())
                .collect();

            log::debug!("Enabled flags: {:?}", enabled_flag_names);
            log::debug!("Profile args for Maven: {:?}", profile_args);
            log::debug!("Active profiles (display): {:?}", active_profile_names);

            // Clear previous output and prepare for new command
            tab.command_output = vec![format!("Running: {} ...", args.join(" "))];
            tab.output_offset = 0;

            match maven::execute_maven_command_async_with_options(
                &tab.project_root,
                Some(&module),
                args,
                &profile_args,
                tab.config.maven_settings.as_deref(),
                &enabled_flags,
                use_file_flag,
                tab.config.logging.as_ref(),
            ) {
                Ok(receiver) => {
                    log::info!("Async command started successfully");
                    tab.command_receiver = Some(receiver);
                    tab.is_command_running = true;
                    tab.command_start_time = Some(Instant::now());

                    // Save last command for watch mode
                    tab.last_command = Some(args.iter().map(|s| s.to_string()).collect());

                    // Store metadata about this command execution
                    let module_output = ModuleOutput {
                        lines: tab.command_output.clone(),
                        scroll_offset: tab.output_offset,
                        command: Some(args.join(" ")),
                        profiles: active_profile_names.clone(),
                        flags: enabled_flag_names.clone(),
                    };
                    tab.module_outputs.insert(module.clone(), module_output);

                    // Add to command history
                    let history_entry = crate::features::history::HistoryEntry::new(
                        module,
                        args.join(" "),
                        active_profile_names,
                        enabled_flag_names,
                    );
                    self.command_history.add(history_entry);
                    log::debug!("Command added to history");
                    return;
                }
                Err(e) => {
                    log::error!("Failed to start async command: {}", e);
                    tab.command_output = vec![format!("Error starting command: {e}")];
                    tab.output_offset = 0;
                }
            }
        } else {
            log::warn!("No module selected for command execution");
            tab.command_output = vec!["No module selected".to_string()];
            tab.output_offset = 0;
        }
        tab.output_metrics = None;
        self.clamp_output_offset();
    }

    /// Check for and process any pending command updates
    /// Should be called regularly from the main event loop
    pub fn poll_command_updates(&mut self) {
        let tab = self.get_active_tab_mut();

        // Get output configuration from config or use defaults
        let output_config = tab.config.output.as_ref().cloned().unwrap_or_default();
        let max_output_lines = output_config.max_lines;
        let max_updates_per_poll = output_config.max_updates_per_poll;

        // Collect all pending updates first to avoid borrowing issues
        let mut updates = Vec::new();
        let mut should_clear_receiver = false;

        if let Some(receiver) = tab.command_receiver.as_ref() {
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
        let output_offset = tab.output_offset;
        let was_at_bottom = output_offset >= self.max_scroll_offset();

        // Re-get tab for processing updates
        let tab = self.get_active_tab_mut();
        let mut had_output_lines = false;
        let mut need_notification = None; // (title, body, success)

        // Now process all updates
        for update in updates {
            match update {
                maven::CommandUpdate::Started(pid) => {
                    log::info!("Command started with PID: {}", pid);
                    tab.running_process_pid = Some(pid);
                }
                maven::CommandUpdate::OutputLine(line) => {
                    tab.command_output.push(line);
                    had_output_lines = true;

                    // Trim buffer if it exceeds max size
                    if tab.command_output.len() > max_output_lines {
                        let excess = tab.command_output.len() - max_output_lines;
                        tab.command_output.drain(0..excess);
                        log::debug!(
                            "Trimmed {} lines from output buffer (max: {})",
                            excess,
                            max_output_lines
                        );
                    }
                }
                maven::CommandUpdate::Completed => {
                    log::info!("Command completed successfully");
                    tab.command_output.push(String::new());
                    tab.command_output
                        .push("✓ Command completed successfully".to_string());
                    tab.is_command_running = false;
                    tab.command_receiver = None;
                    tab.running_process_pid = None;
                    tab.output_metrics = None;

                    need_notification = Some((
                        "LazyMVN - Build Complete".to_string(),
                        "Maven command completed successfully ✓".to_string(),
                        true,
                    ));
                }
                maven::CommandUpdate::Error(msg) => {
                    log::error!("Command failed: {}", msg);
                    tab.command_output.push(String::new());
                    tab.command_output.push(format!("✗ {}", msg));
                    tab.is_command_running = false;
                    tab.command_receiver = None;
                    tab.running_process_pid = None;
                    tab.output_metrics = None;

                    need_notification = Some((
                        "LazyMVN - Build Failed".to_string(),
                        format!("Maven command failed: {}", msg),
                        false,
                    ));
                }
            }
        }

        // Get is_command_running before dropping tab
        let is_command_running = tab.is_command_running;

        if should_clear_receiver {
            tab.is_command_running = false;
            tab.command_receiver = None;
        }

        // Store module output if we had a command completion
        if need_notification.is_some() {
            self.store_current_module_output();
        }

        // Only update scroll and metrics once at the end if we had output lines
        if had_output_lines {
            // Auto-scroll to bottom while command is running (always follow logs)
            // Only respect user's scroll position when command is not running
            if was_at_bottom || is_command_running {
                self.scroll_output_to_end();
            }
            self.store_current_module_output();
            let tab = self.get_active_tab_mut();
            tab.output_metrics = None;
        }

        // Send notification if needed
        if let Some((title, body, success)) = need_notification {
            self.send_notification(&title, &body, success);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use std::path::PathBuf;

    fn create_test_state() -> TuiState {
        TuiState::new(
            vec!["module1".to_string(), "module2".to_string()],
            PathBuf::from("/test"),
            Config::default(),
        )
    }

    #[test]
    fn test_enabled_flag_names_empty() {
        let state = create_test_state();
        let flags = state.enabled_flag_names();
        assert_eq!(flags, Vec::<String>::new());
    }

    #[test]
    fn test_enabled_flag_names_with_enabled_flags() {
        let mut state = create_test_state();
        // Enable some flags
        let tab = state.get_active_tab_mut();
        if !tab.flags.is_empty() {
            tab.flags[0].enabled = true;
        }
        if tab.flags.len() > 1 {
            tab.flags[1].enabled = true;
        }
        
        let flags = state.enabled_flag_names();
        assert_eq!(flags.len(), 2);
    }

    #[test]
    fn test_active_profile_names_empty() {
        let state = create_test_state();
        let profiles = state.active_profile_names();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_get_last_executed_command_none() {
        let state = create_test_state();
        let result = state.get_last_executed_command();
        assert!(result.is_none());
    }

    #[test]
    fn test_current_output_context_none() {
        let state = create_test_state();
        let result = state.current_output_context();
        assert!(result.is_none());
    }

    #[test]
    fn test_store_current_module_output() {
        let mut state = create_test_state();
        {
            let tab = state.get_active_tab_mut();
            tab.command_output.push("Test output line".to_string());
        }
        
        state.store_current_module_output();
        
        let tab = state.get_active_tab();
        let module = state.selected_module().unwrap();
        assert!(tab.module_outputs.contains_key(module));
        let output = tab.module_outputs.get(module).unwrap();
        assert!(!output.lines.is_empty());
        assert!(output.lines.contains(&"Test output line".to_string()));
    }

    #[test]
    fn test_store_current_module_output_preserves_command() {
        let mut state = create_test_state();
        
        // Set up initial output with command
        let module = state.selected_module().unwrap().to_string();
        let initial_output = ModuleOutput {
            lines: vec!["Old output".to_string()],
            command: Some("mvn test".to_string()),
            profiles: vec!["dev".to_string()],
            flags: vec!["-X".to_string()],
            ..Default::default()
        };
        
        {
            let tab = state.get_active_tab_mut();
            tab.module_outputs.insert(module.clone(), initial_output);
            
            // Add new output and store
            tab.command_output = vec!["New output".to_string()];
        }
        
        state.store_current_module_output();
        
        // Verify command context preserved
        let tab = state.get_active_tab();
        let output = tab.module_outputs.get(&module).unwrap();
        assert_eq!(output.command, Some("mvn test".to_string()));
        assert_eq!(output.profiles, vec!["dev".to_string()]);
        assert_eq!(output.flags, vec!["-X".to_string()]);
        assert_eq!(output.lines, vec!["New output".to_string()]);
    }
}
