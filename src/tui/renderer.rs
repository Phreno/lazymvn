//! TUI rendering module
//!
//! This module provides the main drawing function that coordinates between
//! all the UI modules (panes, state, keybindings, etc.) to render the complete
//! terminal user interface.

use crate::ui::{
    keybindings::Focus,
    panes::{
        create_adaptive_layout, render_favorites_popup, render_flags_pane, render_footer,
        render_history_popup, render_modules_pane, render_output_pane, render_profiles_pane,
        render_projects_pane, render_projects_popup, render_save_favorite_popup,
        render_starter_manager_popup, render_starter_selector_popup, render_tab_bar,
    },
    state::TuiState,
};
use ratatui::{Terminal, backend::Backend};

/// Main drawing function that renders the complete TUI
pub fn draw<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut TuiState,
) -> Result<(), std::io::Error> {
    terminal.draw(|f| {
        // Extract data from state that doesn't require mutable access
        let focus = state.focus;
        let search_active = state.search_mod.is_some();

        let (
            tab_bar_area,
            projects_area,
            modules_area,
            profiles_area,
            flags_area,
            output_area,
            footer_area,
        ) = create_adaptive_layout(f.area(), Some(focus));

        // Render tab bar (only if multiple tabs exist)
        if state.get_tab_count() > 1 {
            render_tab_bar(
                f,
                tab_bar_area,
                state.get_tabs(),
                state.get_active_tab_index(),
            );
        }

        // Get active tab for rendering (in a scoped block to release borrow)
        let tab = state.get_active_tab_mut();

        // Get project root name for display
        let project_name = tab
            .project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Render all left panes
        render_projects_pane(
            f,
            projects_area,
            project_name,
            tab.git_branch.as_deref(),
            focus == Focus::Projects,
        );

        render_modules_pane(
            f,
            modules_area,
            &tab.modules,
            &mut tab.modules_list_state,
            focus == Focus::Modules,
        );

        // Need to clone profiles data to avoid borrow issues with profile_loading_status
        let profiles_clone = tab.profiles.clone();
        let mut profiles_list_state_clone = tab.profiles_list_state.clone();

        render_flags_pane(
            f,
            flags_area,
            &tab.flags,
            &mut tab.flags_list_state,
            focus == Focus::Flags,
        );

        // Extract is_running before releasing borrow
        let is_running = tab.is_command_running;

        // Now render profiles with state data
        let spinner = state.profile_loading_spinner();
        let profile_loading_status = &state.profile_loading_status;
        render_profiles_pane(
            f,
            profiles_area,
            &profiles_clone,
            &mut profiles_list_state_clone,
            focus == Focus::Profiles,
            profile_loading_status,
            spinner,
        );

        // Sync list state back if it changed
        {
            let tab = state.get_active_tab_mut();
            tab.profiles_list_state = profiles_list_state_clone;
        }

        // Update output metrics for proper scrolling calculations
        let inner_area = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .inner(output_area);
        state.update_output_metrics(inner_area.width);
        state.set_output_view_dimensions(inner_area.height, inner_area.width);

        // Render output pane
        let selected_module = state.selected_module();
        let current_context = state.current_output_context();
        let elapsed = state.command_elapsed_seconds();

        // Get tab again for rendering output (immutable borrow is OK with closure accessing state)
        let tab = state.get_active_tab();
        let command_output = &tab.command_output;
        let output_offset = tab.output_offset;
        render_output_pane(
            f,
            output_area,
            command_output,
            output_offset,
            focus == Focus::Output,
            |line_index| state.search_line_style(line_index),
            search_active,
            selected_module,
            current_context,
            is_running,
            elapsed,
        );

        // Render footer
        render_footer(
            f,
            footer_area,
            state.current_view,
            state.focus,
            state.selected_module(),
            &state.active_profile_names(),
            &state.enabled_flag_names(),
            state.search_status_line(),
        );

        // Render projects popup on top if shown
        if state.show_projects_popup {
            render_projects_popup(f, &state.recent_projects, &mut state.projects_list_state);
        }

        // Render starter selector popup if shown
        if state.show_starter_selector {
            let candidates = state.get_filtered_starter_candidates();
            render_starter_selector_popup(
                f,
                &candidates,
                &state.starter_filter,
                &mut state.starters_list_state,
            );
        }

        // Render starter manager popup if shown
        if state.show_starter_manager {
            let starters = state.get_active_tab().starters_cache.starters.clone();
            render_starter_manager_popup(f, &starters, &mut state.starters_list_state);
        }

        // Render command history popup if shown
        if state.show_history_popup {
            render_history_popup(
                f,
                state.command_history.entries(),
                &mut state.history_list_state,
            );
        }

        // Render favorites popup if shown
        if state.show_favorites_popup {
            render_favorites_popup(f, state.favorites.list(), &mut state.favorites_list_state);
        }

        // Render save favorite popup if shown
        if state.show_save_favorite_popup {
            render_save_favorite_popup(f, &state.favorite_name_input);
        }
    })?;
    Ok(())
}
