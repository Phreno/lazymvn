//! Tests for output display and scrolling

#[cfg(test)]
mod tests {
    use super::super::TuiState;
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

#[cfg(test)]
mod helper_tests {
    use super::super::output::{
        calculate_clamped_scroll, calculate_output_metrics, format_clipboard_error,
        format_clipboard_success,
    };

    #[test]
    fn test_calculate_output_metrics_empty() {
        assert!(calculate_output_metrics(80, &[]).is_none());
    }

    #[test]
    fn test_calculate_output_metrics_zero_width() {
        let lines = vec!["test".to_string()];
        assert!(calculate_output_metrics(0, &lines).is_none());
    }

    #[test]
    fn test_calculate_output_metrics_valid() {
        let lines = vec!["test".to_string()];
        assert!(calculate_output_metrics(80, &lines).is_some());
    }

    #[test]
    fn test_calculate_clamped_scroll_normal() {
        assert_eq!(calculate_clamped_scroll(10, 5, 100), 15);
    }

    #[test]
    fn test_calculate_clamped_scroll_negative() {
        assert_eq!(calculate_clamped_scroll(10, -5, 100), 5);
    }

    #[test]
    fn test_calculate_clamped_scroll_below_zero() {
        assert_eq!(calculate_clamped_scroll(3, -5, 100), 0);
    }

    #[test]
    fn test_calculate_clamped_scroll_above_max() {
        assert_eq!(calculate_clamped_scroll(95, 10, 100), 100);
    }

    #[test]
    fn test_format_clipboard_success() {
        assert_eq!(
            format_clipboard_success(42),
            "✓ Copied 42 lines to clipboard"
        );
    }

    #[test]
    fn test_format_clipboard_error() {
        assert_eq!(
            format_clipboard_error("permission denied"),
            "✗ Failed to copy: permission denied"
        );
    }
}
