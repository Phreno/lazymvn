//! Maven command keyboard event handlers
//!
//! This module handles keyboard events for executing Maven commands
//! and Spring Boot operations.

use crate::ui::state::TuiState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle Maven and Spring Boot command keys
pub fn handle_maven_command(key: KeyEvent, state: &mut TuiState) -> bool {
    // Only handle keys without modifiers (except for Ctrl+Shift+S)
    // This allows Ctrl+K to be handled by navigation_keys for cache refresh
    let has_modifiers = key.modifiers != KeyModifiers::NONE;
    
    match key.code {
        KeyCode::Char('b') if !has_modifiers => {
            log::info!("Execute: clean install");
            state.run_selected_module_command(&["clean", "install"]);
            true
        }
        KeyCode::Char('C') if !has_modifiers => {
            log::info!("Execute: clean");
            state.run_selected_module_command(&["clean"]);
            true
        }
        KeyCode::Char('c') if !has_modifiers => {
            log::info!("Execute: compile");
            state.run_selected_module_command(&["compile"]);
            true
        }
        KeyCode::Char('k') if !has_modifiers => {
            log::info!("Execute: package");
            state.run_selected_module_command(&["package"]);
            true
        }
        KeyCode::Char('t') if !has_modifiers => {
            log::info!("Execute: test");
            state.run_selected_module_command(&["test"]);
            true
        }
        KeyCode::Char('i') if !has_modifiers => {
            log::info!("Execute: install");
            state.run_selected_module_command(&["install"]);
            true
        }
        KeyCode::Char('d') if !has_modifiers => {
            log::info!("Execute: dependency:tree");
            state.run_selected_module_command(&["dependency:tree"]);
            true
        }
        KeyCode::Char('s') if !has_modifiers => {
            log::info!("Run Spring Boot starter");
            state.run_preferred_starter();
            true
        }
        KeyCode::Char('S')
            if key.modifiers.contains(
                crossterm::event::KeyModifiers::CONTROL | crossterm::event::KeyModifiers::SHIFT,
            ) =>
        {
            log::info!("Open starter manager");
            state.show_starter_manager();
            true
        }
        // Custom goals: Alt+1 to Alt+9
        KeyCode::Char(c @ '1'..='9') if key.modifiers.contains(KeyModifiers::ALT) => {
            let goal_index = c.to_digit(10).unwrap() as usize - 1;
            let tab = state.get_active_tab();
            if goal_index < tab.custom_goals.len() {
                log::info!("Execute custom goal {}: {}", goal_index + 1, tab.custom_goals[goal_index].name);
                state.run_custom_goal(goal_index);
                true
            } else {
                log::warn!("No custom goal defined at index {}", goal_index + 1);
                false
            }
        }
        _ => false,
    }
}
