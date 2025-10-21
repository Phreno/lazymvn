//! Type definitions for keybindings and navigation

/// Represents the current view in the TUI
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CurrentView {
    Projects,
    Modules,
    Profiles,
    Flags,
}

/// Represents which pane currently has focus
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Focus {
    Projects,
    Modules,
    Profiles,
    Flags,
    Output,
}

impl Focus {
    /// Get the next focus in the cycle (right arrow)
    pub fn next(self) -> Self {
        match self {
            Focus::Projects => Focus::Modules,
            Focus::Modules => Focus::Profiles,
            Focus::Profiles => Focus::Flags,
            Focus::Flags => Focus::Output,
            Focus::Output => Focus::Projects,
        }
    }

    /// Get the previous focus in the cycle (left arrow)
    pub fn previous(self) -> Self {
        match self {
            Focus::Projects => Focus::Output,
            Focus::Modules => Focus::Projects,
            Focus::Profiles => Focus::Modules,
            Focus::Flags => Focus::Profiles,
            Focus::Output => Focus::Flags,
        }
    }
}

/// Search mode for input vs cycling through matches
pub enum SearchMode {
    Input,
    Cycling,
}
