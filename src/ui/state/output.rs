//! Output display and scrolling management
//!
//! This module handles command output display, scrolling, clipboard operations,
//! and output metrics calculation.

use super::{OutputMetrics, TuiState};

impl TuiState {
    /// Synchronize the selected module's output to the display
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

    /// Copy output to clipboard using system tools or arboard fallback
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

    /// Update output metrics for text wrapping calculations
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

    /// Set output view dimensions and adjust scrolling
    pub fn set_output_view_dimensions(&mut self, height: u16, width: u16) {
        let tab = self.get_active_tab_mut();
        tab.output_view_height = height;
        tab.output_area_width = width;
        self.clamp_output_offset();
        self.apply_pending_center();
        self.ensure_current_match_visible();
    }

    /// Clamp output offset to valid range
    pub(super) fn clamp_output_offset(&mut self) {
        let max = self.max_scroll_offset();
        let tab = self.get_active_tab_mut();
        tab.output_offset = tab.output_offset.min(max);
    }

    /// Scroll output by lines (positive = down, negative = up)
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

    /// Scroll output by pages (positive = down, negative = up)
    pub fn scroll_output_pages(&mut self, delta: isize) {
        let tab = self.get_active_tab();
        let page = tab.output_view_height.max(1) as isize;
        self.scroll_output_lines(delta * page);
    }

    /// Scroll to start of output
    pub fn scroll_output_to_start(&mut self) {
        let tab = self.get_active_tab_mut();
        if tab.command_output.is_empty() {
            return;
        }
        tab.output_offset = 0;
        self.store_current_module_output();
    }

    /// Scroll to end of output
    pub fn scroll_output_to_end(&mut self) {
        let max_offset = self.max_scroll_offset();
        let tab = self.get_active_tab_mut();
        tab.output_offset = max_offset;
        self.store_current_module_output();
    }

    /// Calculate maximum scroll offset based on output size
    pub(super) fn max_scroll_offset(&self) -> usize {
        let tab = self.get_active_tab();
        let height = tab.output_view_height as usize;
        if height == 0 {
            return 0;
        }
        let total = self.total_display_rows();
        total.saturating_sub(height)
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

    fn setup_output(state: &mut TuiState, lines: Vec<&str>) {
        let tab = state.get_active_tab_mut();
        tab.command_output = lines.iter().map(|s| s.to_string()).collect();
    }

    #[test]
    fn test_scroll_output_to_start() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2", "line3"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_offset = 5;
        }

        state.scroll_output_to_start();

        assert_eq!(state.get_active_tab().output_offset, 0);
    }

    #[test]
    fn test_scroll_output_to_start_empty() {
        let mut state = create_test_state();
        state.scroll_output_to_start();
        assert_eq!(state.get_active_tab().output_offset, 0);
    }

    #[test]
    fn test_scroll_output_to_end() {
        let mut state = create_test_state();
        setup_output(
            &mut state,
            vec!["line1", "line2", "line3", "line4", "line5"],
        );

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 2;
        }

        state.scroll_output_to_end();

        let max_offset = state.max_scroll_offset();
        assert_eq!(state.get_active_tab().output_offset, max_offset);
    }

    #[test]
    fn test_scroll_output_lines_down() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2", "line3", "line4"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 2;
            tab.output_offset = 0;
        }

        state.scroll_output_lines(1);

        assert_eq!(state.get_active_tab().output_offset, 1);
    }

    #[test]
    fn test_scroll_output_lines_up() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2", "line3", "line4"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 2;
            tab.output_offset = 2;
        }

        state.scroll_output_lines(-1);

        assert_eq!(state.get_active_tab().output_offset, 1);
    }

    #[test]
    fn test_scroll_output_lines_clamps_at_top() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 2;
            tab.output_offset = 0;
        }

        state.scroll_output_lines(-10);

        assert_eq!(state.get_active_tab().output_offset, 0);
    }

    #[test]
    fn test_scroll_output_lines_clamps_at_bottom() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2", "line3"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 2;
            tab.output_offset = 0;
        }

        state.scroll_output_lines(100);

        let max_offset = state.max_scroll_offset();
        assert_eq!(state.get_active_tab().output_offset, max_offset);
    }

    #[test]
    fn test_scroll_output_pages() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["l1", "l2", "l3", "l4", "l5", "l6"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 2;
            tab.output_offset = 0;
        }

        state.scroll_output_pages(1);

        // Should scroll by page size (2 lines)
        assert_eq!(state.get_active_tab().output_offset, 2);
    }

    #[test]
    fn test_max_scroll_offset_with_content() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["l1", "l2", "l3", "l4", "l5"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 2;
        }

        let max_offset = state.max_scroll_offset();
        // 5 lines - 2 visible = 3 max offset
        assert_eq!(max_offset, 3);
    }

    #[test]
    fn test_max_scroll_offset_zero_height() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 0;
        }

        assert_eq!(state.max_scroll_offset(), 0);
    }

    #[test]
    fn test_max_scroll_offset_no_overflow() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2"]);

        {
            let tab = state.get_active_tab_mut();
            tab.output_view_height = 5; // More visible than content
        }

        assert_eq!(state.max_scroll_offset(), 0);
    }

    #[test]
    fn test_update_output_metrics_zero_width() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1"]);

        state.update_output_metrics(0);

        assert!(state.get_active_tab().output_metrics.is_none());
    }

    #[test]
    fn test_update_output_metrics_empty_output() {
        let mut state = create_test_state();

        // Empty output might still create metrics with width
        state.update_output_metrics(80);

        // Just verify width is set
        assert_eq!(state.get_active_tab().output_area_width, 80);
    }

    #[test]
    fn test_update_output_metrics_valid() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2"]);

        state.update_output_metrics(80);

        assert!(state.get_active_tab().output_metrics.is_some());
        assert_eq!(state.get_active_tab().output_area_width, 80);
    }

    #[test]
    fn test_set_output_view_dimensions() {
        let mut state = create_test_state();
        setup_output(&mut state, vec!["line1", "line2"]);

        state.set_output_view_dimensions(10, 80);

        let tab = state.get_active_tab();
        assert_eq!(tab.output_view_height, 10);
        assert_eq!(tab.output_area_width, 80);
    }

    #[test]
    fn test_sync_selected_module_output_with_existing() {
        let mut state = create_test_state();

        // Set up module output
        let module = state.selected_module().unwrap().to_string();
        {
            let tab = state.get_active_tab_mut();
            let module_output = super::super::ModuleOutput {
                lines: vec!["module line 1".to_string(), "module line 2".to_string()],
                scroll_offset: 3,
                ..Default::default()
            };
            tab.module_outputs.insert(module.clone(), module_output);
        }

        state.sync_selected_module_output();

        let tab = state.get_active_tab();
        assert_eq!(tab.command_output.len(), 2);
        assert_eq!(tab.command_output[0], "module line 1");
        // Offset gets clamped, so just verify it's set
        assert!(tab.output_offset <= 3);
    }

    #[test]
    fn test_sync_selected_module_output_no_existing() {
        let mut state = create_test_state();

        {
            let tab = state.get_active_tab_mut();
            tab.command_output = vec!["old output".to_string()];
            tab.output_offset = 5;
        }

        state.sync_selected_module_output();

        let tab = state.get_active_tab();
        assert!(tab.command_output.is_empty());
        assert_eq!(tab.output_offset, 0);
    }
}
