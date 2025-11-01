use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::ui::theme::Theme;

/// Create a centered popup area with the given width and height percentages.
#[allow(dead_code)]
pub(super) fn centered_popup_area(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let popup_width = (area.width * width_percent) / 100;
    let popup_height = (area.height * height_percent) / 100;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    }
}

pub fn render_custom_goals_popup(
    f: &mut Frame,
    custom_goals: &[crate::core::config::CustomGoal],
    list_state: &mut ListState,
) {
    // Calculate popup size (centered, 60% width, 50% height)
    let area = f.area();
    let popup_width = (area.width * 60) / 100;
    let popup_height = (area.height * 50) / 100;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear the area behind the popup with solid background
    f.render_widget(ratatui::widgets::Clear, popup_area);

    // Split popup into list and help sections
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // List
            Constraint::Length(3), // Help
        ])
        .split(popup_area);

    // Create list items from custom goals
    let items: Vec<ListItem> = custom_goals
        .iter()
        .enumerate()
        .map(|(idx, goal)| {
            let line = vec![
                Span::styled(
                    format!("[{}] ", idx + 1),
                    Style::default().fg(ratatui::style::Color::Yellow),
                ),
                Span::styled(
                    &goal.name,
                    Style::default()
                        .fg(ratatui::style::Color::Cyan)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::raw(" → "),
                Span::styled(
                    &goal.goal,
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ),
            ];
            ListItem::new(Line::from(line))
        })
        .collect();

    let title = format!("Custom Maven Goals ({} available)", custom_goals.len());

    let list_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, popup_chunks[0], list_state);

    // Render help section
    let help_block = Block::default()
        .title("Controls")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let help_text = vec![
        Line::from(vec![
            Span::styled("↑↓", Style::default().fg(ratatui::style::Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(ratatui::style::Color::Yellow)),
            Span::raw(" Execute  "),
            Span::styled("Esc/q", Style::default().fg(ratatui::style::Color::Yellow)),
            Span::raw(" Close"),
        ]),
    ];

    let help_para = Paragraph::new(help_text).block(help_block);
    f.render_widget(help_para, popup_chunks[1]);
}
