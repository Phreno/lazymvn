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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_next_from_projects() {
        assert_eq!(Focus::Projects.next(), Focus::Modules);
    }

    #[test]
    fn test_focus_next_from_modules() {
        assert_eq!(Focus::Modules.next(), Focus::Profiles);
    }

    #[test]
    fn test_focus_next_from_profiles() {
        assert_eq!(Focus::Profiles.next(), Focus::Flags);
    }

    #[test]
    fn test_focus_next_from_flags() {
        assert_eq!(Focus::Flags.next(), Focus::Output);
    }

    #[test]
    fn test_focus_next_from_output_wraps() {
        assert_eq!(Focus::Output.next(), Focus::Projects);
    }

    #[test]
    fn test_focus_previous_from_projects_wraps() {
        assert_eq!(Focus::Projects.previous(), Focus::Output);
    }

    #[test]
    fn test_focus_previous_from_modules() {
        assert_eq!(Focus::Modules.previous(), Focus::Projects);
    }

    #[test]
    fn test_focus_previous_from_profiles() {
        assert_eq!(Focus::Profiles.previous(), Focus::Modules);
    }

    #[test]
    fn test_focus_previous_from_flags() {
        assert_eq!(Focus::Flags.previous(), Focus::Profiles);
    }

    #[test]
    fn test_focus_previous_from_output() {
        assert_eq!(Focus::Output.previous(), Focus::Flags);
    }

    #[test]
    fn test_focus_cycle_full_loop_next() {
        let start = Focus::Projects;
        let after_5 = start.next().next().next().next().next();
        assert_eq!(after_5, start);
    }

    #[test]
    fn test_focus_cycle_full_loop_previous() {
        let start = Focus::Projects;
        let after_5 = start.previous().previous().previous().previous().previous();
        assert_eq!(after_5, start);
    }
}
