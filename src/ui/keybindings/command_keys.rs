//! Maven command keyboard event handlers
//!
//! This module handles keyboard events for executing Maven commands
//! and Spring Boot operations.

use crate::ui::state::TuiState;
use crossterm::event::{KeyCode, KeyEvent};

/// Handle Maven and Spring Boot command keys
pub fn handle_maven_command(key: KeyEvent, state: &mut TuiState) -> bool {
    match key.code {
        KeyCode::Char('b') => {
            log::info!("Execute: clean install");
            state.run_selected_module_command(&["clean", "install"]);
            true
        }
        KeyCode::Char('C') => {
            log::info!("Execute: clean");
            state.run_selected_module_command(&["clean"]);
            true
        }
        KeyCode::Char('c') => {
            log::info!("Execute: compile");
            state.run_selected_module_command(&["compile"]);
            true
        }
        KeyCode::Char('k') => {
            log::info!("Execute: package");
            state.run_selected_module_command(&["package"]);
            true
        }
        KeyCode::Char('t') => {
            log::info!("Execute: test");
            state.run_selected_module_command(&["test"]);
            true
        }
        KeyCode::Char('i') => {
            log::info!("Execute: install");
            state.run_selected_module_command(&["install"]);
            true
        }
        KeyCode::Char('d') => {
            log::info!("Execute: dependency:tree");
            state.run_selected_module_command(&["dependency:tree"]);
            true
        }
        KeyCode::Char('s') => {
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
        _ => false,
    }
}
