use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::ui::theme::Theme;

/// Create a centered popup area with the given width and height percentages.
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

pub fn render_projects_popup(
    f: &mut Frame,
    projects: &[std::path::PathBuf],
    list_state: &mut ListState,
    filter: &str,
) {
    // Calculate popup size (centered, 70% width, 70% height)
    let popup_area = centered_popup_area(f.area(), 70, 70);

    // Clear the area behind the popup with solid background
    f.render_widget(ratatui::widgets::Clear, popup_area);

    // Split popup into filter input, list, and help sections
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input
            Constraint::Min(1),    // List
            Constraint::Length(3), // Help
        ])
        .split(popup_area);

    // Render filter input
    let filter_block = Block::default()
        .title("Filter (type to search)")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);
    let filter_text = if filter.is_empty() {
        Line::from("Type to filter projects...")
    } else {
        Line::from(filter.to_string())
    };
    let filter_para = Paragraph::new(filter_text).block(filter_block);
    f.render_widget(filter_para, popup_chunks[0]);

    // Create list items from projects
    let items: Vec<ListItem> = projects
        .iter()
        .map(|p| {
            let display = p.to_string_lossy().to_string();
            ListItem::new(Line::from(display))
        })
        .collect();

    let title = if projects.is_empty() {
        "Recent Projects (no matches)".to_string()
    } else {
        format!("Recent Projects ({} matches)", projects.len())
    };

    let list_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);

    let list = List::new(items)
        .block(list_block)
        .style(Theme::DEFAULT_STYLE)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, popup_chunks[1], list_state);

    // Render help text
    let help_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Theme::DEFAULT_STYLE);
    let help_text = "Type: Filter | ↑↓: Navigate | Enter: Select | Esc: Cancel";
    let help = Paragraph::new(Line::from(help_text))
        .block(help_block)
        .style(Theme::FOOTER_SECTION_STYLE);
    f.render_widget(help, popup_chunks[2]);
}
