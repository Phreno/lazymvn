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

    /// Style for active profiles (with asterisk)
    pub const ACTIVE_PROFILE_STYLE: Style =
        Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);

    /// Style for key hints in footer
    pub const KEY_HINT_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    /// Style for footer section labels
    pub const FOOTER_SECTION_STYLE: Style = Style::new()
        .fg(Color::LightMagenta)
        .add_modifier(Modifier::BOLD);

    /// Style for active footer text
    pub const FOOTER_ACTIVE_TEXT_STYLE: Style = Style::new()
        .fg(Color::White)
        .add_modifier(Modifier::UNDERLINED);

    /// Style for footer pointer indicators
    pub const FOOTER_POINTER_STYLE: Style =
        Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);

    /// Style for search match highlights
    pub const SEARCH_MATCH_STYLE: Style = Style::new().bg(Color::Yellow).fg(Color::Black);

    /// Style for current search match
    pub const CURRENT_SEARCH_MATCH_STYLE: Style = Style::new().bg(Color::Red).fg(Color::White);

    /// Style for error messages
    pub const ERROR_STYLE: Style = Style::new().fg(Color::Red);

    /// Style for success/info messages
    pub const INFO_STYLE: Style = Style::new().fg(Color::Green);
}
