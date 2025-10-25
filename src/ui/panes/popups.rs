use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::ui::theme::Theme;

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
) {
    // Calculate popup size (centered, 60% width, 60% height)
    let area = f.area();
    let popup_width = (area.width * 60) / 100;
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

    // Create the popup block with rounded borders
    let block = Block::default()
        .title("Recent Projects [Ctrl+R]")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);

    // Create list items from projects
    let items: Vec<ListItem> = projects
        .iter()
        .map(|p| {
            let display = p.to_string_lossy().to_string();
            ListItem::new(Line::from(display))
        })
        .collect();

    let help_text = if projects.is_empty() {
        "No recent projects. Open Maven projects to add them to this list."
    } else {
        "↑↓: Navigate | Enter: Select | Esc: Cancel"
    };

    // Split popup into list and help sections
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(popup_area);

    // Render the list
    let list = List::new(items)
        .block(block)
        .style(Theme::DEFAULT_STYLE)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, popup_chunks[0], list_state);

    // Render help text
    let help_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Theme::DEFAULT_STYLE);
    let help = Paragraph::new(Line::from(help_text))
        .block(help_block)
        .style(Theme::FOOTER_SECTION_STYLE);
    f.render_widget(help, popup_chunks[1]);
}

/// Render command history popup
pub fn render_history_popup(
    f: &mut Frame,
    history: &[crate::features::history::HistoryEntry],
    list_state: &mut ListState,
) {
    // Calculate popup size (centered, 80% width, 80% height)
    let area = f.area();
    let popup_width = (area.width * 80) / 100;
    let popup_height = (area.height * 80) / 100;
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

    // Split popup into title, list, preview, and help sections
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Title
            Constraint::Percentage(60), // List
            Constraint::Min(1),         // Preview
            Constraint::Length(2),      // Help
        ])
        .split(popup_area);

    // Title
    let title_block = Block::default()
        .title("Command History (Ctrl+H)")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);
    f.render_widget(title_block, popup_chunks[0]);

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

    let list_block = Block::default()
        .title("Recent Commands")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let list = List::new(items)
        .block(list_block)
        .style(Theme::DEFAULT_STYLE)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, popup_chunks[1], list_state);

    // Preview of selected command
    let preview_text = if let Some(selected) = list_state.selected() {
        if let Some(entry) = history.get(selected) {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Module: ",
                        Style::default().fg(ratatui::style::Color::Yellow),
                    ),
                    Span::raw(&entry.module),
                ]),
                Line::from(vec![
                    Span::styled("Goal: ", Style::default().fg(ratatui::style::Color::Yellow)),
                    Span::raw(&entry.goal),
                ]),
            ];

            if !entry.profiles.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Profiles: ",
                        Style::default().fg(ratatui::style::Color::Yellow),
                    ),
                    Span::raw(entry.profiles.join(", ")),
                ]));
            }

            if !entry.flags.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Flags: ",
                        Style::default().fg(ratatui::style::Color::Yellow),
                    ),
                    Span::raw(entry.flags.join(", ")),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Time: ", Style::default().fg(ratatui::style::Color::Yellow)),
                Span::raw(entry.format_time()),
            ]));

            lines
        } else {
            vec![Line::from("No command selected")]
        }
    } else {
        vec![Line::from("No command selected")]
    };

    let preview_block = Block::default()
        .title("Command Details")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let preview = Paragraph::new(preview_text).block(preview_block);
    f.render_widget(preview, popup_chunks[2]);

    // Help
    let help_text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(ratatui::style::Color::Green)),
        Span::raw(" Run | "),
        Span::styled("↑↓", Style::default().fg(ratatui::style::Color::Cyan)),
        Span::raw(" Navigate | "),
        Span::styled("Esc", Style::default().fg(ratatui::style::Color::Red)),
        Span::raw(" Close"),
    ]);

    let help_paragraph = Paragraph::new(help_text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Theme::DEFAULT_STYLE);
    f.render_widget(help_paragraph, popup_chunks[3]);
}

/// Render favorites popup
pub fn render_favorites_popup(
    f: &mut Frame,
    favorites: &[crate::features::favorites::Favorite],
    list_state: &mut ListState,
) {
    // Calculate popup size
    let area = f.area();
    let popup_width = (area.width * 80) / 100;
    let popup_height = (area.height * 80) / 100;
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

    // Split popup
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Title
            Constraint::Percentage(60), // List
            Constraint::Min(1),         // Preview
            Constraint::Length(2),      // Help
        ])
        .split(popup_area);

    // Title
    let title_block = Block::default()
        .title("Favorite Commands (Ctrl+F)")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);
    f.render_widget(title_block, popup_chunks[0]);

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

    let list_block = Block::default()
        .title("Saved Favorites")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let list = List::new(items)
        .block(list_block)
        .style(Theme::DEFAULT_STYLE)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, popup_chunks[1], list_state);

    // Preview
    let preview_text = if favorites.is_empty() {
        vec![Line::from("No favorites to preview")]
    } else if let Some(selected) = list_state.selected() {
        if let Some(fav) = favorites.get(selected) {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(ratatui::style::Color::Yellow)),
                    Span::raw(&fav.name),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Module: ",
                        Style::default().fg(ratatui::style::Color::Yellow),
                    ),
                    Span::raw(&fav.module),
                ]),
                Line::from(vec![
                    Span::styled("Goal: ", Style::default().fg(ratatui::style::Color::Yellow)),
                    Span::raw(&fav.goal),
                ]),
            ];

            if !fav.profiles.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Profiles: ",
                        Style::default().fg(ratatui::style::Color::Yellow),
                    ),
                    Span::raw(fav.profiles.join(", ")),
                ]));
            }

            if !fav.flags.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Flags: ",
                        Style::default().fg(ratatui::style::Color::Yellow),
                    ),
                    Span::raw(fav.flags.join(", ")),
                ]));
            }

            lines
        } else {
            vec![Line::from("No favorite selected")]
        }
    } else {
        vec![Line::from("No favorite selected")]
    };

    let preview_block = Block::default()
        .title("Command Details")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let preview = Paragraph::new(preview_text).block(preview_block);
    f.render_widget(preview, popup_chunks[2]);

    // Help
    let help_text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(ratatui::style::Color::Green)),
        Span::raw(" Run | "),
        Span::styled("Del", Style::default().fg(ratatui::style::Color::Red)),
        Span::raw(" Delete | "),
        Span::styled("↑↓", Style::default().fg(ratatui::style::Color::Cyan)),
        Span::raw(" Navigate | "),
        Span::styled("Esc", Style::default().fg(ratatui::style::Color::Red)),
        Span::raw(" Close"),
    ]);

    let help_paragraph = Paragraph::new(help_text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Theme::DEFAULT_STYLE);
    f.render_widget(help_paragraph, popup_chunks[3]);
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
