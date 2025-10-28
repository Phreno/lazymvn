use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::ui::theme::Theme;

/// Create a centered popup area with the given width and height percentages.
/// Returns a Rect centered in the terminal area.
fn centered_popup_area(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
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

pub fn render_starter_selector_popup(
    f: &mut Frame,
    candidates: &[String],
    filter: &str,
    list_state: &mut ListState,
) {
    // Calculate popup size (centered, 70% width, 70% height)
    let area = f.area();
    let popup_width = (area.width * 70) / 100;
    let popup_height = (area.height * 70) / 100;
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
        Line::from("Type to filter starters...")
    } else {
        Line::from(filter.to_string())
    };
    let filter_para = Paragraph::new(filter_text).block(filter_block);
    f.render_widget(filter_para, popup_chunks[0]);

    // Create list items from candidates
    let items: Vec<ListItem> = candidates
        .iter()
        .map(|c| {
            // Show class name highlighted
            let parts: Vec<&str> = c.rsplitn(2, '.').collect();
            let class_name = parts[0];
            let package = if parts.len() > 1 { parts[1] } else { "" };

            let line = if !package.is_empty() {
                vec![
                    Span::styled(
                        package,
                        Style::default().fg(ratatui::style::Color::DarkGray),
                    ),
                    Span::raw("."),
                    Span::styled(
                        class_name,
                        Style::default()
                            .fg(ratatui::style::Color::Cyan)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                ]
            } else {
                vec![Span::styled(
                    class_name,
                    Style::default()
                        .fg(ratatui::style::Color::Cyan)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                )]
            };

            ListItem::new(Line::from(line))
        })
        .collect();

    let title = if candidates.is_empty() {
        "Select Spring Boot Starter (no matches)".to_string()
    } else {
        format!("Select Spring Boot Starter ({} matches)", candidates.len())
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
    let help_text = "Type: Filter | ↑↓: Navigate | Enter: Select & Run | Esc: Cancel";
    let help = Paragraph::new(Line::from(help_text))
        .block(help_block)
        .style(Theme::FOOTER_SECTION_STYLE);
    f.render_widget(help, popup_chunks[2]);
}

/// Render popup for managing cached starters
pub fn render_starter_manager_popup(
    f: &mut Frame,
    starters: &[crate::features::starters::Starter],
    list_state: &mut ListState,
) {
    // Calculate popup size (centered, 70% width, 60% height)
    let area = f.area();
    let popup_width = (area.width * 70) / 100;
    let popup_height = (area.height * 60) / 100;
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
            Constraint::Length(4), // Help
        ])
        .split(popup_area);

    // Create list items from starters
    let items: Vec<ListItem> = starters
        .iter()
        .map(|starter| {
            let mut spans = vec![];

            // Default indicator
            if starter.is_default {
                spans.push(Span::styled(
                    "★ ",
                    Style::default().fg(ratatui::style::Color::Yellow),
                ));
            } else {
                spans.push(Span::raw("  "));
            }

            // Label
            spans.push(Span::styled(
                &starter.label,
                Style::default()
                    .fg(ratatui::style::Color::Green)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ));

            // FQCN in gray
            spans.push(Span::raw(" ("));
            spans.push(Span::styled(
                &starter.fully_qualified_class_name,
                Style::default().fg(ratatui::style::Color::DarkGray),
            ));
            spans.push(Span::raw(")"));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if starters.is_empty() {
        "Manage Starters (empty)".to_string()
    } else {
        format!("Manage Starters ({} cached)", starters.len())
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

    f.render_stateful_widget(list, popup_chunks[0], list_state);

    // Render help text
    let help_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Theme::DEFAULT_STYLE);
    let help_lines = vec![
        Line::from("↑↓: Navigate | Enter: Run | Space: Toggle Default"),
        Line::from("d: Delete | Esc: Close"),
    ];
    let help = Paragraph::new(help_lines)
        .block(help_block)
        .style(Theme::FOOTER_SECTION_STYLE);
    f.render_widget(help, popup_chunks[1]);
}

/// Render popup for recent projects selection
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

/// Render command history popup
pub fn render_history_popup(
    f: &mut Frame,
    history: &[crate::features::history::HistoryEntry],
    list_state: &mut ListState,
    filter: &str,
) {
    let popup_area = centered_popup_area(f.area(), 70, 70);
    f.render_widget(ratatui::widgets::Clear, popup_area);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input
            Constraint::Min(1),    // List
            Constraint::Length(3), // Help
        ])
        .split(popup_area);

    // Filter input section
    let filter_block = Block::default()
        .title("Filter (type to search)")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);

    let filter_text = if filter.is_empty() {
        Span::styled("Type to filter commands...", Theme::DEFAULT_STYLE.add_modifier(Modifier::DIM))
    } else {
        Span::raw(filter)
    };

    let filter_para = Paragraph::new(filter_text).block(filter_block);
    f.render_widget(filter_para, popup_chunks[0]);

    // Command list
    let items: Vec<ListItem> = history
        .iter()
        .map(|entry| {
            let time = entry.format_time();
            let cmd = entry.format_command();
            let line = format!("{} | {}", time, cmd);
            ListItem::new(Line::from(line))
        })
        .collect();

    let title = if history.is_empty() {
        "Command History (no matches)".to_string()
    } else {
        format!("Command History ({} matches)", history.len())
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
    let help_text = "Type: Filter | ↑↓: Navigate | Enter: Run | Ctrl+S: Save | Esc: Cancel";
    let help = Paragraph::new(Line::from(help_text))
        .block(help_block)
        .style(Theme::FOOTER_SECTION_STYLE);
    f.render_widget(help, popup_chunks[2]);
}

/// Render favorites popup
pub fn render_favorites_popup(
    f: &mut Frame,
    favorites: &[crate::features::favorites::Favorite],
    list_state: &mut ListState,
    filter: &str,
) {
    let popup_area = centered_popup_area(f.area(), 70, 70);
    f.render_widget(ratatui::widgets::Clear, popup_area);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input
            Constraint::Min(1),    // List
            Constraint::Length(3), // Help
        ])
        .split(popup_area);

    // Filter input section
    let filter_block = Block::default()
        .title("Filter (type to search)")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);

    let filter_text = if filter.is_empty() {
        Span::styled("Type to filter favorites...", Theme::DEFAULT_STYLE.add_modifier(Modifier::DIM))
    } else {
        Span::raw(filter)
    };

    let filter_para = Paragraph::new(filter_text).block(filter_block);
    f.render_widget(filter_para, popup_chunks[0]);

    // List
    let items: Vec<ListItem> = if favorites.is_empty() {
        vec![ListItem::new(Line::from(
            "No favorites yet. Use Ctrl+S in history to save one!",
        ))]
    } else {
        favorites
            .iter()
            .map(|fav| {
                let line = fav.format_summary();
                ListItem::new(Line::from(line))
            })
            .collect()
    };

    let title = if favorites.is_empty() {
        "Favorite Commands (no matches)".to_string()
    } else {
        format!("Favorite Commands ({} matches)", favorites.len())
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
    let help_text = "Type: Filter | ↑↓: Navigate | Enter: Run | d/Del: Delete | Esc: Cancel";
    let help = Paragraph::new(Line::from(help_text))
        .block(help_block)
        .style(Theme::FOOTER_SECTION_STYLE);
    f.render_widget(help, popup_chunks[2]);
}

/// Render save favorite popup (name input)
pub fn render_save_favorite_popup(f: &mut Frame, name_input: &str) {
    // Calculate popup size (smaller, centered)
    let area = f.area();
    let popup_width = 60.min(area.width - 4);
    let popup_height = 7;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear background
    f.render_widget(ratatui::widgets::Clear, popup_area);

    // Main block
    let block = Block::default()
        .title("Save as Favorite")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    // Split into prompt and input
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Prompt
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Input
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // Prompt
    let prompt = Paragraph::new("Enter a name for this favorite:");
    f.render_widget(prompt, chunks[0]);

    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);
    let input = Paragraph::new(name_input).block(input_block);
    f.render_widget(input, chunks[2]);

    // Help
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(ratatui::style::Color::Green)),
        Span::raw(" Save | "),
        Span::styled("Esc", Style::default().fg(ratatui::style::Color::Red)),
        Span::raw(" Cancel"),
    ]))
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(help, chunks[4]);
}
