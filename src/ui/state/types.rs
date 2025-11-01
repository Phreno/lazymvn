//! Type definitions for TUI state management
//!
//! This module contains core data structures used throughout the UI state system.

use crate::ui::search::SearchMatch;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Output data for a specific module
#[derive(Clone, Debug, Default)]
pub struct ModuleOutput {
    pub lines: Vec<String>,
    pub scroll_offset: usize,
    pub command: Option<String>,
    pub profiles: Vec<String>,
    pub flags: Vec<String>,
}

/// Metrics for calculating output display and scrolling
#[derive(Clone, Debug, Default)]
pub struct OutputMetrics {
    width: usize,
    line_display: Vec<String>,
    line_start_rows: Vec<usize>,
    total_rows: usize,
}

impl OutputMetrics {
    pub fn new(width: usize, lines: &[String]) -> Self {
        if width == 0 {
            return Self::default();
        }
        let mut line_display = Vec::with_capacity(lines.len());
        let mut line_start_rows = Vec::with_capacity(lines.len());
        let mut cumulative = 0usize;

        for line in lines {
            line_start_rows.push(cumulative);
            let display = crate::utils::clean_log_line(line).unwrap_or_default();
            let rows = visual_rows(&display, width);
            cumulative += rows;
            line_display.push(display);
        }

        Self {
            width,
            line_display,
            line_start_rows,
            total_rows: cumulative,
        }
    }

    pub fn total_rows(&self) -> usize {
        self.total_rows
    }

    pub fn row_for_match(&self, m: &SearchMatch) -> Option<usize> {
        if self.width == 0 {
            return Some(0);
        }
        let line_index = m.line_index;
        let start_rows = self.line_start_rows.get(line_index)?;
        let display = self.line_display.get(line_index)?;
        let col = column_for_byte_index(display, m.start);
        let row_in_line = col / self.width;
        Some(start_rows + row_in_line)
    }
}

/// Profile loading state
pub enum ProfileLoadingStatus {
    /// Profiles are being loaded asynchronously
    Loading,
    /// Profiles have been loaded successfully
    Loaded,
    /// Failed to load profiles
    Error(String),
}

/// State of a Maven profile
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileState {
    /// Profile follows Maven's auto-activation rules
    Default,
    /// Profile is explicitly enabled (will add to -P)
    ExplicitlyEnabled,
    /// Profile is explicitly disabled (will add !profile to -P)
    ExplicitlyDisabled,
}

/// Maven profile with activation state
#[derive(Clone, Debug)]
pub struct MavenProfile {
    pub name: String,
    pub state: ProfileState,
    /// Whether this profile is auto-activated by Maven (file, JDK, OS, etc.)
    pub auto_activated: bool,
}

impl MavenProfile {
    pub fn new(name: String, auto_activated: bool) -> Self {
        Self {
            name,
            state: ProfileState::Default,
            auto_activated,
        }
    }

    /// Returns true if this profile will be active when running Maven
    pub fn is_active(&self) -> bool {
        match self.state {
            ProfileState::Default => self.auto_activated,
            ProfileState::ExplicitlyEnabled => true,
            ProfileState::ExplicitlyDisabled => false,
        }
    }

    /// Returns the profile argument string for Maven (-P flag)
    /// Returns None if profile is in Default state
    pub fn to_maven_arg(&self) -> Option<String> {
        match self.state {
            ProfileState::Default => None,
            ProfileState::ExplicitlyEnabled => Some(self.name.clone()),
            ProfileState::ExplicitlyDisabled => Some(format!("!{}", self.name)),
        }
    }

    /// Cycle through states when toggled
    pub fn toggle(&mut self) {
        self.state = match self.state {
            ProfileState::Default => {
                if self.auto_activated {
                    // Auto-activated: Default → Disabled
                    ProfileState::ExplicitlyDisabled
                } else {
                    // Not auto-activated: Default → Enabled
                    ProfileState::ExplicitlyEnabled
                }
            }
            ProfileState::ExplicitlyEnabled => ProfileState::Default,
            ProfileState::ExplicitlyDisabled => ProfileState::Default,
        };
    }
}

/// Maven build flags that can be toggled
#[derive(Clone, Debug)]
pub struct BuildFlag {
    pub name: String,
    pub flag: String,
    pub enabled: bool,
}

// Helper functions for OutputMetrics

fn visual_rows(line: &str, width: usize) -> usize {
    if width == 0 {
        return 1;
    }
    let display_width = UnicodeWidthStr::width(line);
    let rows = display_width.div_ceil(width);
    rows.max(1)
}

fn column_for_byte_index(s: &str, byte_index: usize) -> usize {
    let mut column = 0usize;
    for (idx, ch) in s.char_indices() {
        if idx >= byte_index {
            break;
        }
        column += UnicodeWidthChar::width(ch).unwrap_or(0);
    }
    column
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_state_transitions() {
        let mut profile = MavenProfile::new("test-profile".to_string(), false);

        // Default state for non-auto profile
        assert_eq!(profile.state, ProfileState::Default);
        assert!(!profile.is_active());

        // Toggle should enable
        profile.toggle();
        assert_eq!(profile.state, ProfileState::ExplicitlyEnabled);
        assert!(profile.is_active());

        // Toggle again should return to default
        profile.toggle();
        assert_eq!(profile.state, ProfileState::Default);
        assert!(!profile.is_active());
    }

    #[test]
    fn test_auto_activated_profile_state_transitions() {
        let mut profile = MavenProfile::new("auto-profile".to_string(), true);

        // Default state for auto-activated profile
        assert_eq!(profile.state, ProfileState::Default);
        assert!(profile.is_active()); // Auto-activated, so active by default

        // Toggle should disable
        profile.toggle();
        assert_eq!(profile.state, ProfileState::ExplicitlyDisabled);
        assert!(!profile.is_active());

        // Toggle again should return to default (auto-activated)
        profile.toggle();
        assert_eq!(profile.state, ProfileState::Default);
        assert!(profile.is_active());
    }

    #[test]
    fn test_profile_maven_arg_generation() {
        let mut profile = MavenProfile::new("test".to_string(), false);

        // Default state: no arg
        assert_eq!(profile.to_maven_arg(), None);

        // Explicitly enabled: returns profile name
        profile.state = ProfileState::ExplicitlyEnabled;
        assert_eq!(profile.to_maven_arg(), Some("test".to_string()));

        // Explicitly disabled: returns !profile
        profile.state = ProfileState::ExplicitlyDisabled;
        assert_eq!(profile.to_maven_arg(), Some("!test".to_string()));
    }
}
