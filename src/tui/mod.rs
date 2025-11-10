//! Main TUI coordination module
//!
//! This module provides the main TUI functionality by coordinating between
//! rendering, event handling, and state management.

mod mouse;
mod renderer;

// Re-export public API
pub use mouse::handle_mouse_event;
pub use renderer::draw;

// Re-export commonly used types for backward compatibility
pub use crate::ui::state::TuiState;

use crossterm::event::KeyEvent;

/// Handle key events by delegating to the keybindings module
pub fn handle_key_event(key: KeyEvent, state: &mut TuiState) {
    crate::ui::keybindings::handle_key_event(key, state);
}


#[cfg(test)]
mod tests;
