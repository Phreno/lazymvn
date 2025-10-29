//! Popup keyboard event handlers
//!
//! This module handles keyboard events for all popup dialogs.

use crate::ui::keybindings::{get_all_keybindings, KeybindingAction};
use crate::ui::state::TuiState;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle keyboard events for save favorite popup
pub fn handle_save_favorite_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char(ch) => {
            state.favorite_name_input.push(ch);
        }
        KeyCode::Backspace => {
            state.favorite_name_input.pop();
        }
        KeyCode::Enter => {
            if !state.favorite_name_input.trim().is_empty() {
                // Get current goal from the last executed command or default
                let goal = state
                    .get_last_executed_command()
                    .map(|(cmd, _, _)| cmd)
                    .unwrap_or_else(|| "install".to_string());
                state.save_pending_favorite(goal);
                log::info!("Favorite saved");
            }
        }
        KeyCode::Esc => {
            state.cancel_save_favorite();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for favorites popup
pub fn handle_favorites_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            log::info!("Close favorites popup");
            state.show_favorites_popup = false;
            state.favorites_filter.clear();
        }
        KeyCode::Delete | KeyCode::Char('d') => {
            log::info!("Delete favorite");
            state.delete_selected_favorite();
        }
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("Favorites filter input: '{}'", ch);
            state.push_favorites_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("Favorites filter backspace");
            state.pop_favorites_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Navigate down in favorites");
            let len = state.get_filtered_favorites().len();
            if len > 0 {
                let i = match state.favorites_list_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                state.favorites_list_state.select(Some(i));
            }
        }
        KeyCode::Up => {
            log::debug!("Navigate up in favorites");
            let len = state.get_filtered_favorites().len();
            if len > 0 {
                let i = match state.favorites_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                state.favorites_list_state.select(Some(i));
            }
        }
        KeyCode::Enter => {
            log::info!("Execute favorite");
            if let Some(selected) = state.favorites_list_state.selected()
                && let Some(fav) = state.get_filtered_favorites().get(selected).cloned()
            {
                state.apply_favorite(&fav);
                state.show_favorites_popup = false;
                state.favorites_filter.clear();
            }
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for history popup
pub fn handle_history_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            log::info!("Close history popup");
            state.show_history_popup = false;
            state.history_filter.clear();
        }
        KeyCode::Char('s')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::info!("Save selected history entry as favorite");
            if let Some(selected) = state.history_list_state.selected()
                && let Some(entry) = state.get_filtered_history().get(selected).cloned()
            {
                state.pending_favorite = Some(entry);
                state.show_history_popup = false;
                state.show_save_favorite_popup = true;
                state.favorite_name_input.clear();
            }
        }
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("History filter input: '{}'", ch);
            state.push_history_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("History filter backspace");
            state.pop_history_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Navigate down in history");
            let len = state.get_filtered_history().len();
            if len > 0 {
                let i = match state.history_list_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                state.history_list_state.select(Some(i));
            }
        }
        KeyCode::Up => {
            log::debug!("Navigate up in history");
            let len = state.get_filtered_history().len();
            if len > 0 {
                let i = match state.history_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                state.history_list_state.select(Some(i));
            }
        }
        KeyCode::Enter => {
            log::info!("Execute command from history");
            if let Some(selected) = state.history_list_state.selected()
                && let Some(entry) = state.get_filtered_history().get(selected)
            {
                // Apply the command's configuration
                state.apply_history_entry(entry.clone());
                state.show_history_popup = false;
            }
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for projects popup
pub fn handle_projects_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            log::info!("Cancel project selection");
            state.hide_recent_projects();
        }
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("Projects filter input: '{}'", ch);
            state.push_projects_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("Projects filter backspace");
            state.pop_projects_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Navigate down in projects list");
            state.next_project();
        }
        KeyCode::Up => {
            log::debug!("Navigate up in projects list");
            state.previous_project();
        }
        KeyCode::Enter => {
            log::info!("Select project from recent list");
            state.select_current_project();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for custom goals popup
pub fn handle_custom_goals_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            log::info!("Close custom goals popup");
            state.close_custom_goals_popup();
        }
        KeyCode::Down => {
            log::debug!("Navigate down in custom goals");
            let tab = state.get_active_tab();
            let len = tab.custom_goals.len();
            if len > 0 {
                let tab = state.get_active_tab_mut();
                let i = match tab.custom_goals_list_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                tab.custom_goals_list_state.select(Some(i));
            }
        }
        KeyCode::Up => {
            log::debug!("Navigate up in custom goals");
            let tab = state.get_active_tab();
            let len = tab.custom_goals.len();
            if len > 0 {
                let tab = state.get_active_tab_mut();
                let i = match tab.custom_goals_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                tab.custom_goals_list_state.select(Some(i));
            }
        }
        KeyCode::Enter => {
            log::info!("Execute custom goal");
            let tab = state.get_active_tab();
            if let Some(selected) = tab.custom_goals_list_state.selected()
                && selected < tab.custom_goals.len()
            {
                state.close_custom_goals_popup();
                state.run_custom_goal(selected);
            }
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for starter selector
pub fn handle_starter_selector(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("Starter filter input: '{}'", ch);
            state.push_starter_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("Starter filter backspace");
            state.pop_starter_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Next starter");
            state.next_starter();
        }
        KeyCode::Up => {
            log::debug!("Previous starter");
            state.previous_starter();
        }
        KeyCode::Enter => {
            log::info!("Select and run starter");
            state.select_and_run_starter();
        }
        KeyCode::Esc => {
            log::info!("Cancel starter selection");
            state.hide_starter_selector();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for package selector
pub fn handle_package_selector(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char(ch)
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::debug!("Package filter input: '{}'", ch);
            state.push_package_filter_char(ch);
        }
        KeyCode::Backspace => {
            log::debug!("Package filter backspace");
            state.pop_package_filter_char();
        }
        KeyCode::Down => {
            log::debug!("Next package");
            state.next_package();
        }
        KeyCode::Up => {
            log::debug!("Previous package");
            state.previous_package();
        }
        KeyCode::Enter => {
            log::info!("Select and add package to config");
            state.select_and_add_package();
        }
        KeyCode::Esc => {
            log::info!("Cancel package selection");
            state.hide_package_selector();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for starter manager
pub fn handle_starter_manager(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Down => {
            log::debug!("Next starter in manager");
            state.next_starter();
        }
        KeyCode::Up => {
            log::debug!("Previous starter in manager");
            state.previous_starter();
        }
        KeyCode::Enter => {
            log::info!("Run selected starter from manager");
            let tab = state.get_active_tab();
            if let Some(idx) = state.starters_list_state.selected()
                && let Some(starter) = tab.starters_cache.starters.get(idx)
            {
                let fqcn = starter.fully_qualified_class_name.clone();
                state.run_spring_boot_starter(&fqcn);
                state.hide_starter_manager();
            }
        }
        KeyCode::Char(' ') => {
            log::info!("Toggle starter default");
            state.toggle_starter_default();
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            log::info!("Delete starter");
            state.remove_selected_starter();
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            log::info!("Close starter manager");
            state.hide_starter_manager();
        }
        _ => {}
    }
    true
}

/// Handle keyboard events for help popup
pub fn handle_help_popup(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        // Close popup
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            log::info!("Close help popup");
            state.hide_help_popup();
        }
        
        // Navigation - Up
        KeyCode::Up => {
            log::trace!("Help popup: Navigate up");
            let filtered_count = get_filtered_keybindings_count(state);
            if filtered_count == 0 {
                return true;
            }
            
            let i = match state.help_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        filtered_count - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            state.help_list_state.select(Some(i));
        }
        
        // Navigation - Down
        KeyCode::Down => {
            log::trace!("Help popup: Navigate down");
            let filtered_count = get_filtered_keybindings_count(state);
            if filtered_count == 0 {
                return true;
            }
            
            let i = match state.help_list_state.selected() {
                Some(i) => {
                    if i >= filtered_count - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            state.help_list_state.select(Some(i));
        }
        
        // Execute selected keybinding action
        KeyCode::Enter => {
            log::info!("Execute help keybinding");
            if let Some(selected) = state.help_list_state.selected() {
                let all_keybindings = get_all_keybindings();
                let filtered: Vec<_> = if state.help_search_query.is_empty() {
                    all_keybindings
                } else {
                    let query_lower = state.help_search_query.to_lowercase();
                    all_keybindings.into_iter().filter(|kb| {
                        kb.keys.to_lowercase().contains(&query_lower) ||
                        kb.description.to_lowercase().contains(&query_lower) ||
                        kb.category.to_lowercase().contains(&query_lower)
                    }).collect()
                };
                
                if let Some(keybinding) = filtered.get(selected)
                    && let Some(action) = &keybinding.action
                {
                    state.hide_help_popup();
                    execute_keybinding_action(action, state);
                }
            }
        }
        
        // Clear search filter
        KeyCode::Backspace if !state.help_search_query.is_empty() => {
            state.help_search_query.pop();
            state.help_list_state.select(Some(0)); // Reset to first item
        }
        
        // Type to filter
        KeyCode::Char(ch) if !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
            state.help_search_query.push(ch);
            state.help_list_state.select(Some(0)); // Reset to first item after filter
        }
        
        _ => {}
    }
    true
}

/// Get the count of filtered keybindings for help popup
fn get_filtered_keybindings_count(state: &TuiState) -> usize {
    use crate::ui::keybindings::get_all_keybindings;
    
    let all_keybindings = get_all_keybindings();
    if state.help_search_query.is_empty() {
        all_keybindings.len()
    } else {
        let query_lower = state.help_search_query.to_lowercase();
        all_keybindings.into_iter().filter(|kb| {
            kb.keys.to_lowercase().contains(&query_lower) ||
            kb.description.to_lowercase().contains(&query_lower) ||
            kb.category.to_lowercase().contains(&query_lower)
        }).count()
    }
}

/// Execute a keybinding action from the help popup
fn execute_keybinding_action(action: &KeybindingAction, state: &mut TuiState) {
    match action {
        // Navigation
        KeybindingAction::FocusPane(focus) => {
            state.focus = *focus;
        }
        
        // Maven Commands
        KeybindingAction::Build => {
            state.run_selected_module_command(&["clean", "install"]);
        }
        KeybindingAction::Compile => {
            state.run_selected_module_command(&["compile"]);
        }
        KeybindingAction::Clean => {
            state.run_selected_module_command(&["clean"]);
        }
        KeybindingAction::Package => {
            state.run_selected_module_command(&["package"]);
        }
        KeybindingAction::Test => {
            state.run_selected_module_command(&["test"]);
        }
        KeybindingAction::Install => {
            state.run_selected_module_command(&["install"]);
        }
        KeybindingAction::Dependencies => {
            state.run_selected_module_command(&["dependency:tree"]);
        }
        
        // Spring Boot
        KeybindingAction::RunStarter => {
            state.run_preferred_starter();
        }
        KeybindingAction::ManageStarters => {
            state.show_starter_manager();
        }
        
        // Workflow
        KeybindingAction::ShowFavorites => {
            state.show_favorites_popup = true;
        }
        KeybindingAction::SaveFavorite => {
            state.show_save_favorite_dialog_from_current();
        }
        KeybindingAction::ShowHistory => {
            state.show_history_popup = true;
        }
        KeybindingAction::ShowRecentProjects => {
            state.show_recent_projects();
        }
        KeybindingAction::EditConfig => {
            state.edit_config();
        }
        KeybindingAction::RefreshCaches => {
            state.refresh_caches();
        }
        KeybindingAction::ShowCustomGoals => {
            state.show_custom_goals_popup();
        }
        
        // Tab Management
        KeybindingAction::NewTab => {
            state.show_recent_projects(); // Open project selector for new tab
        }
        KeybindingAction::CloseTab => {
            let current_idx = state.get_active_tab_index();
            let _ = state.close_tab(current_idx);
        }
        
        // Search & Selection
        KeybindingAction::StartSearch => {
            state.search_mod = Some(crate::ui::keybindings::SearchMode::Input);
        }
        KeybindingAction::YankOutput => {
            state.yank_output();
        }
        KeybindingAction::YankDebugReport => {
            state.yank_debug_info();
        }
        
        // General
        KeybindingAction::ShowHelp => {
            state.show_help_popup();
        }
        KeybindingAction::Quit => {
            // Quit is handled at a higher level, just close the help
        }
        KeybindingAction::KillProcess => {
            state.kill_running_process();
        }
        
        // Complex actions (not directly executable)
        KeybindingAction::FocusPreviousPane | 
        KeybindingAction::FocusNextPane | 
        KeybindingAction::PreviousTab | 
        KeybindingAction::NextTab => {
            // These require more complex logic, skip
        }
    }
}

