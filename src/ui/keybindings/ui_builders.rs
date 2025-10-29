//! UI builders for footer and navigation displays
//!
//! Provides functions to build navigation lines, footer content, and command hints.

use super::helpers::{append_bracketed_word, key_token};
use super::types::CurrentView;
use crate::ui::theme::Theme;
use ratatui::text::{Line, Span};

/// Data structure for module action display
pub(crate) struct ModuleAction {
    pub key_display: &'static str,
    pub prefix: &'static str,
    pub suffix: &'static str,
}

/// Module action key bindings
pub(crate) const MODULE_ACTIONS: [ModuleAction; 10] = [
    ModuleAction {
        key_display: "b",
        prefix: "",
        suffix: "uild",
    },
    ModuleAction {
        key_display: "C",
        prefix: "",
        suffix: "lean",
    },
    ModuleAction {
        key_display: "c",
        prefix: "",
        suffix: "ompile",
    },
    ModuleAction {
        key_display: "k",
        prefix: "pac",
        suffix: "age",
    },
    ModuleAction {
        key_display: "t",
        prefix: "",
        suffix: "est",
    },
    ModuleAction {
        key_display: "i",
        prefix: "",
        suffix: "nstall",
    },
    ModuleAction {
        key_display: "s",
        prefix: "",
        suffix: "tart",
    },
    ModuleAction {
        key_display: "d",
        prefix: "",
        suffix: "eps",
    },
    ModuleAction {
        key_display: "y",
        prefix: "",
        suffix: "ank output",
    },
    ModuleAction {
        key_display: "?",
        prefix: "",
        suffix: "",
    },
];

/// Create a blank line for spacing in the footer
pub fn blank_line() -> Line<'static> {
    Line::raw("")
}

/// Build the main navigation help lines
///
/// Returns a vector of lines showing:
/// - Views (0-4 keys)
/// - Focus and Navigate (arrow keys)
/// - Tabs (Ctrl+T/W, Ctrl+arrows)
/// - Actions (Ctrl+F/H/R/E)
pub fn build_navigation_line() -> Vec<Line<'static>> {
    vec![
        // Line 1: Views
        Line::from(vec![
            Span::styled("Views: ", Theme::FOOTER_SECTION_STYLE),
            key_token("0"),
            Span::raw(" Output  "),
            key_token("1"),
            Span::raw(" Projects  "),
            key_token("2"),
            Span::raw(" Modules  "),
            key_token("3"),
            Span::raw(" Profiles  "),
            key_token("4"),
            Span::raw(" Flags"),
        ]),
        // Line 2: Focus & Navigate
        Line::from(vec![
            Span::styled("Focus: ", Theme::FOOTER_SECTION_STYLE),
            key_token("←→"),
            Span::raw("   "),
            Span::styled("Navigate: ", Theme::FOOTER_SECTION_STYLE),
            key_token("↑↓"),
            Span::raw("   "),
        ]),
        // Line 3: Tabulations
        Line::from(vec![
            Span::styled("Tabs: ", Theme::FOOTER_SECTION_STYLE),
            key_token("Ctrl+T"),
            Span::raw(" New "),
            key_token("Ctrl+W"),
            Span::raw(" Close "),
            key_token("Ctrl+←→"),
            Span::raw(" Switch "),
            key_token("Esc"),
            Span::raw(" Kill"),
        ]),
        // Actions
        Line::from(vec![
            Span::styled("Actions: ", Theme::FOOTER_SECTION_STYLE),
            key_token("Ctrl+F"),
            Span::raw(" Favs  "),
            key_token("Ctrl+H"),
            Span::raw(" History  "),
            key_token("Ctrl+R"),
            Span::raw(" Recent  "),
            key_token("Ctrl+E"),
            Span::raw(" Edit  "),
            key_token("Ctrl+K"),
            Span::raw(" Refresh"),
        ]),
    ]
}

/// Build the simplified footer title
///
/// Returns a styled span with "Commands" text
/// Style depends on current view (modules/projects vs profiles/flags)
pub fn simplified_footer_title(
    view: CurrentView,
    _module_name: Option<&str>,
    _active_profiles: &[String],
    _enabled_flags: &[String],
) -> Span<'static> {
    let text = "Commands".to_string();

    let style = match view {
        CurrentView::Projects | CurrentView::Modules => Theme::FOOTER_SECTION_STYLE,
        CurrentView::Profiles | CurrentView::Flags => Theme::FOOTER_SECTION_FOCUSED_STYLE,
    };

    Span::styled(text, style)
}

/// Build the simplified footer body with module commands
///
/// Shows all available module commands like [b]uild, [c]ompile, [t]est, etc.
pub fn simplified_footer_body(_view: CurrentView) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::raw(" "));

    // Module commands
    for (idx, action) in MODULE_ACTIONS.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw(" "));
        }
        append_bracketed_word(&mut spans, action.prefix, action.key_display, action.suffix);
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blank_line_creates_empty_line() {
        let line = blank_line();
        // Line::raw("") may have 0 or 1 spans depending on implementation
        // What matters is the rendered content is empty
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.is_empty());
    }

    #[test]
    fn test_build_navigation_line_has_four_lines() {
        let lines = build_navigation_line();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn test_build_navigation_line_contains_views() {
        let lines = build_navigation_line();
        let first_line = &lines[0];
        let text: String = first_line
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(text.contains("Views:"));
        assert!(text.contains("Output"));
        assert!(text.contains("Modules"));
    }

    #[test]
    fn test_build_navigation_line_contains_tabs() {
        let lines = build_navigation_line();
        let third_line = &lines[2];
        let text: String = third_line
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(text.contains("Tabs:"));
        assert!(text.contains("New"));
        assert!(text.contains("Close"));
    }

    #[test]
    fn test_simplified_footer_title_modules_view() {
        let span = simplified_footer_title(CurrentView::Modules, None, &[], &[]);
        assert_eq!(span.content, "Commands");
        assert_eq!(span.style, Theme::FOOTER_SECTION_STYLE);
    }

    #[test]
    fn test_simplified_footer_title_profiles_view() {
        let span = simplified_footer_title(CurrentView::Profiles, None, &[], &[]);
        assert_eq!(span.content, "Commands");
        assert_eq!(span.style, Theme::FOOTER_SECTION_FOCUSED_STYLE);
    }

    #[test]
    fn test_simplified_footer_body_contains_all_actions() {
        let line = simplified_footer_body(CurrentView::Modules);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();

        // Check all actions are present (check substrings because brackets split spans)
        assert!(text.contains("uild")); // [b]uild
        assert!(text.contains("lean")); // [C]lean
        assert!(text.contains("ompile")); // [c]ompile
        assert!(text.contains("pac") && text.contains("age")); // pac[k]age
        assert!(text.contains("est")); // [t]est
        assert!(text.contains("nstall")); // [i]nstall
        assert!(text.contains("tart")); // [s]tart
        assert!(text.contains("eps")); // [d]eps
        assert!(text.contains("ank")); // [y]ank output
    }

    #[test]
    fn test_module_actions_count() {
        assert_eq!(MODULE_ACTIONS.len(), 9);
    }

    #[test]
    fn test_module_actions_keys() {
        let keys: Vec<&str> = MODULE_ACTIONS.iter().map(|a| a.key_display).collect();
        assert_eq!(keys, vec!["b", "C", "c", "k", "t", "i", "s", "d", "y"]);
    }
}
