//! Utility functions for TUI state (clipboard, debug info, notifications, config editing)

use super::TuiState;

impl TuiState {
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

    /// Collect system information (OS, architecture, etc.)
    fn collect_system_info() -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== System Information ===".to_string());
        info.push(format!("OS: {}", std::env::consts::OS));
        info.push(format!("Architecture: {}", std::env::consts::ARCH));
        info.push(String::new());
        info
    }

    /// Collect configuration information from current tab
    fn collect_config_info(&self) -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== Configuration ===".to_string());
        
        let tab = self.get_active_tab();
        let config = &tab.config;
        
        info.push(format!("Project root: {:?}", tab.project_root));
        info.push(format!("Settings file: {:?}", config.maven_settings));
        info.push(format!("Notifications: {}", config.notifications_enabled.unwrap_or(true)));
        
        if let Some(logging) = &config.logging {
            info.push(format!("Log format: {:?}", logging.log_format));
            info.push(format!("Package filters: {} configured", logging.packages.len()));
        }
        
        info.push(String::new());
        info
    }

    /// Collect output from all tabs
    fn collect_all_tabs_output(&self) -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== Tab Outputs ===".to_string());
        
        for (idx, tab) in self.tabs.iter().enumerate() {
            info.push(format!("--- Tab {} ---", idx + 1));
            info.push(format!("Project: {:?}", tab.project_root));
            info.push(format!("Modules: {}", tab.modules.len()));
            info.push(format!("Output lines: {}", tab.command_output.len()));
            
            if !tab.command_output.is_empty() {
                info.push("Last 10 lines:".to_string());
                let start = tab.command_output.len().saturating_sub(10);
                for line in &tab.command_output[start..] {
                    info.push(format!("  {}", line));
                }
            }
            info.push(String::new());
        }
        
        info
    }

    /// Collect recent log entries from current session
    /// Note: Uses get_logs_for_debug_report() which filters out TRACE level logs
    /// to keep the debug report manageable and focused on relevant information.
    fn collect_logs() -> Vec<String> {
        let mut info = Vec::new();
        info.push("=== Recent Logs ===".to_string());
        info.push("(Showing DEBUG, INFO, WARN, ERROR - TRACE logs excluded)".to_string());
        info.push(String::new());
        
        // Get logs from current session, with TRACE filtering
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
        }

        child.wait()?;
        Ok(())
    }

    /// Fallback to arboard crate for clipboard
    fn copy_via_arboard(&mut self, text: &str, lines: usize, content_type: &str) {
        use arboard::Clipboard;

        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(text) {
                Ok(()) => {
                    self.show_clipboard_success(lines, content_type, "system clipboard");
                }
                Err(e) => {
                    self.show_clipboard_error(&e.to_string());
                }
            },
            Err(e) => {
                self.show_clipboard_error(&e.to_string());
            }
        }
    }

    fn show_clipboard_success(&mut self, lines: usize, content_type: &str, tool: &str) {
        log::info!("Copied {} lines to clipboard using {}", lines, tool);
        let tab = self.get_active_tab_mut();
        tab.command_output = vec![
            format!("✓ Copied {} to clipboard ({} lines)", content_type, lines),
            format!("  Tool: {}", tool),
        ];
    }

    fn show_clipboard_error(&mut self, error: &str) {
        log::error!("Failed to copy to clipboard: {}", error);
        let tab = self.get_active_tab_mut();
        tab.command_output = vec![
            "❌ Failed to copy to clipboard".to_string(),
            format!("Error: {}", error),
        ];
    }

    pub(crate) fn send_notification(&self, title: &str, body: &str, success: bool) {
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
}
