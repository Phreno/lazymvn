use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::keybindings::{KeybindingAction, get_all_keybindings};
use crate::ui::state::TuiState;

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
