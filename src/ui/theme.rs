use ratatui::style::{Color, Modifier, Style};

/// Theme constants for the TUI
pub struct Theme;

impl Theme {
    /// Style for focused borders and highlighted elements
    pub const FOCUS_STYLE: Style = Style::new().fg(Color::Yellow);

    /// Style for default borders and text
    pub const DEFAULT_STYLE: Style = Style::new();

    /// Style for selected list items
    pub const SELECTED_STYLE: Style = Style::new()
        .bg(Color::LightBlue)
        .add_modifier(Modifier::BOLD);

    /// Style for active profiles (explicitly enabled)
    pub const ACTIVE_PROFILE_STYLE: Style =
        Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);

    /// Style for auto-activated profiles (default state, auto-enabled)
    pub const AUTO_PROFILE_STYLE: Style = Style::new().fg(Color::Cyan);

    /// Style for disabled profiles (explicitly disabled)
    pub const DISABLED_PROFILE_STYLE: Style = Style::new().fg(Color::Red);

    /// Style for key hints in footer
    pub const KEY_HINT_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    /// Style for footer section labels
    pub const FOOTER_SECTION_STYLE: Style = Style::new()
        .fg(Color::LightMagenta)
        .add_modifier(Modifier::BOLD);

    /// Style for focused footer section titles
    pub const FOOTER_SECTION_FOCUSED_STYLE: Style = Style::new()
        .fg(Color::LightCyan)
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED);

    /// Style for footer box borders
    pub const FOOTER_BOX_BORDER_STYLE: Style = Style::new().fg(Color::Blue);

    /// Style for focused footer box borders
    /// Style for disabled footer text
    /// Style for dimmed/secondary text
    pub const DIM_STYLE: Style = Style::new().fg(Color::DarkGray);

    /// Style for search match highlights
    pub const SEARCH_MATCH_STYLE: Style = Style::new().bg(Color::Yellow).fg(Color::Black);

    /// Style for current search match
    pub const CURRENT_SEARCH_MATCH_STYLE: Style = Style::new().bg(Color::Red).fg(Color::White);

    /// Style for error messages
    pub const ERROR_STYLE: Style = Style::new().fg(Color::Red);

    /// Style for success/info messages
    pub const INFO_STYLE: Style = Style::new().fg(Color::Green);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_style_is_yellow() {
        assert_eq!(Theme::FOCUS_STYLE.fg, Some(Color::Yellow));
    }

    #[test]
    fn test_selected_style_has_background() {
        assert_eq!(Theme::SELECTED_STYLE.bg, Some(Color::LightBlue));
    }

    #[test]
    fn test_active_profile_style_is_green_bold() {
        assert_eq!(Theme::ACTIVE_PROFILE_STYLE.fg, Some(Color::Green));
        assert!(Theme::ACTIVE_PROFILE_STYLE.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_auto_profile_style_is_cyan() {
        assert_eq!(Theme::AUTO_PROFILE_STYLE.fg, Some(Color::Cyan));
    }

    #[test]
    fn test_disabled_profile_style_is_red() {
        assert_eq!(Theme::DISABLED_PROFILE_STYLE.fg, Some(Color::Red));
    }

    #[test]
    fn test_search_match_style() {
        assert_eq!(Theme::SEARCH_MATCH_STYLE.bg, Some(Color::Yellow));
        assert_eq!(Theme::SEARCH_MATCH_STYLE.fg, Some(Color::Black));
    }

    #[test]
    fn test_current_search_match_style() {
        assert_eq!(Theme::CURRENT_SEARCH_MATCH_STYLE.bg, Some(Color::Red));
        assert_eq!(Theme::CURRENT_SEARCH_MATCH_STYLE.fg, Some(Color::White));
    }

    #[test]
    fn test_error_style_is_red() {
        assert_eq!(Theme::ERROR_STYLE.fg, Some(Color::Red));
    }

    #[test]
    fn test_info_style_is_green() {
        assert_eq!(Theme::INFO_STYLE.fg, Some(Color::Green));
    }

    #[test]
    fn test_dim_style_is_dark_gray() {
        assert_eq!(Theme::DIM_STYLE.fg, Some(Color::DarkGray));
    }
}

