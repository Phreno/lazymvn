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

pub fn render_help_popup(f: &mut Frame, search_query: &str, list_state: &mut ListState) {
    use crate::ui::keybindings::get_all_keybindings;
    
    let popup_area = centered_popup_area(f.area(), 80, 90);
    f.render_widget(ratatui::widgets::Clear, popup_area);

    // Split into main area and search bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search bar
            Constraint::Min(1),     // Keybindings list
            Constraint::Length(3),  // Controls
        ])
        .split(popup_area);

    // Render search bar
    let search_block = Block::default()
        .title(" Search (type to filter) ")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);
    
    let search_text = if search_query.is_empty() {
        Span::styled("Type to filter keybindings...", Style::default().fg(ratatui::style::Color::DarkGray))
    } else {
        Span::raw(search_query)
    };
    
    let search_paragraph = Paragraph::new(search_text)
        .block(search_block)
        .style(Theme::DEFAULT_STYLE);
    f.render_widget(search_paragraph, chunks[0]);

    // Get and filter keybindings
    let all_keybindings = get_all_keybindings();
    let filtered_keybindings: Vec<_> = if search_query.is_empty() {
        all_keybindings
    } else {
        let query_lower = search_query.to_lowercase();
        all_keybindings.into_iter().filter(|kb| {
            kb.keys.to_lowercase().contains(&query_lower) ||
            kb.description.to_lowercase().contains(&query_lower) ||
            kb.category.to_lowercase().contains(&query_lower)
        }).collect()
    };

    // Group by category and create list items
    // Note: Ne pas afficher les en-têtes de catégorie quand un filtre est actif
    // pour éviter les problèmes de sélection avec les indices
    let mut items = Vec::new();
    let mut current_category = "";
    let show_headers = search_query.is_empty();
    
    for keybinding in &filtered_keybindings {
        // Add category header if changed (only when no filter)
        if show_headers && keybinding.category != current_category {
            if !current_category.is_empty() {
                items.push(ListItem::new(Line::from(""))); // Empty line between categories
            }
            current_category = keybinding.category;
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("═══ {} ═══", keybinding.category),
                    Style::default()
                        .fg(ratatui::style::Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                )
            ])));
        }
        
        // Add keybinding item
        let prefix = if show_headers { "  " } else { "" };
        
        // En mode filtré, afficher la catégorie entre parenthèses
        let line_spans = if show_headers {
            vec![
                Span::raw(prefix),
                Span::styled(
                    format!("{:12}", keybinding.keys),
                    Style::default().fg(ratatui::style::Color::Cyan)
                ),
                Span::raw(" "),
                Span::raw(keybinding.description),
            ]
        } else {
            vec![
                Span::styled(
                    format!("{:12}", keybinding.keys),
                    Style::default().fg(ratatui::style::Color::Cyan)
                ),
                Span::raw(" "),
                Span::raw(keybinding.description),
                Span::raw(" "),
                Span::styled(
                    format!("({})", keybinding.category),
                    Style::default().fg(ratatui::style::Color::DarkGray)
                ),
            ]
        };
        
        items.push(ListItem::new(Line::from(line_spans)));
    }

    let list_block = Block::default()
        .title(format!(" LazyMVN - Keyboard Shortcuts ({} bindings) ", filtered_keybindings.len()))
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOCUS_STYLE);

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, chunks[1], list_state);

    // Render controls
    let controls_block = Block::default()
        .title("Controls")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let controls_text = vec![
        Line::from(vec![
            Span::styled("Type", Style::default().fg(ratatui::style::Color::Yellow)),
            Span::raw(" Filter  "),
            Span::styled("↑↓", Style::default().fg(ratatui::style::Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(ratatui::style::Color::Yellow)),
            Span::raw(" Execute  "),
            Span::styled("Esc/?/q", Style::default().fg(ratatui::style::Color::Yellow)),
            Span::raw(" Close"),
        ]),
    ];

    let controls_paragraph = Paragraph::new(controls_text)
        .block(controls_block)
        .style(Theme::DEFAULT_STYLE);

    f.render_widget(controls_paragraph, chunks[2]);
}
