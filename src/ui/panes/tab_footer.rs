//! Tab bar and footer rendering
//!
//! This module handles rendering of the tab bar (for multi-project support)
//! and the footer (navigation help and command shortcuts).

use crate::ui::keybindings::{CurrentView, Focus};
use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Render the tab bar showing open projects
pub fn render_tab_bar(
    f: &mut Frame,
    area: Rect,
    tabs: &[crate::ui::state::ProjectTab],
    active_tab_index: usize,
) {
    if tabs.is_empty() {
        return;
    }

    // Build tab labels
    let mut tab_spans = Vec::new();
    for (idx, tab) in tabs.iter().enumerate() {
        let is_active = idx == active_tab_index;

        // Get short project name
        let project_name = tab
            .project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("???");

        // Format: " [1] project-name " or " 1 project-name " (active has brackets)
        let tab_label = if is_active {
            format!(" [{}] {} ", idx + 1, project_name)
        } else {
            format!("  {}  {} ", idx + 1, project_name)
        };

        // Style
        let style = if is_active {
            Theme::ACTIVE_PROFILE_STYLE
        } else {
            Theme::DEFAULT_STYLE
        };

        tab_spans.push(Span::styled(tab_label, style));

        // Separator
        if idx < tabs.len() - 1 {
            tab_spans.push(Span::raw("│"));
        }
    }

    // Add indicator of total tabs at the end
    if tabs.len() > 1 {
        tab_spans.push(Span::styled(
            format!(" ({}/{}) ", active_tab_index + 1, tabs.len()),
            Theme::DIM_STYLE,
        ));
    }

    let line = Line::from(tab_spans);
    let paragraph = Paragraph::new(line).style(Theme::DEFAULT_STYLE).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Theme::DEFAULT_STYLE),
    );

    f.render_widget(paragraph, area);
}

/// Render the footer with navigation help and command shortcuts
#[allow(clippy::too_many_arguments)]
pub fn render_footer(
    f: &mut Frame,
    area: Rect,
    view: CurrentView,
    focus: Focus,
    module_name: Option<&str>,
    active_profiles: &[String],
    enabled_flags: &[String],
    search_status_line: Option<Line<'static>>,
    last_command_status: Option<&crate::ui::state::LastCommandStatus>,
) {
    let _ = focus; // Not needed in simplified footer

    let mut constraints = vec![
        Constraint::Length(3), // navigation (3 lines)
        Constraint::Length(1), // spacer
        Constraint::Length(3), // commands box
    ];
    if search_status_line.is_some() {
        constraints.push(Constraint::Length(1));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Navigation lines (multi-line)
    let navigation_lines = crate::ui::keybindings::build_navigation_line();
    let navigation = Paragraph::new(navigation_lines);
    f.render_widget(navigation, chunks[0]);

    // Spacer
    f.render_widget(
        Paragraph::new(crate::ui::keybindings::blank_line()),
        chunks[1],
    );

    // Simplified commands box - single row with all commands
    let title = crate::ui::keybindings::simplified_footer_title(
        view,
        module_name,
        active_profiles,
        enabled_flags,
    );
    let commands_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Theme::FOOTER_BOX_BORDER_STYLE);
    let commands_paragraph =
        Paragraph::new(crate::ui::keybindings::simplified_footer_body(view, last_command_status)).block(commands_block);
    f.render_widget(commands_paragraph, chunks[2]);

    // Optional search status line
    if let Some(status_line) = search_status_line {
        let status_paragraph = Paragraph::new(status_line);
        let idx = chunks.len() - 1;
        f.render_widget(status_paragraph, chunks[idx]);
    }
}
