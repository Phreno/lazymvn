use crate::ui::keybindings::{CurrentView, Focus};
use crate::ui::state::MenuState;
use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

/// Render the modules pane
pub fn render_modules_pane(
    f: &mut Frame,
    area: Rect,
    modules: &[String],
    list_state: &mut ListState,
    is_focused: bool,
) {
    let block = Block::default()
        .title("Modules")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Theme::FOCUS_STYLE
        } else {
            Theme::DEFAULT_STYLE
        });

    let items: Vec<ListItem> = modules
        .iter()
        .map(|m| ListItem::new(Line::from(m.as_str())))
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Theme::DEFAULT_STYLE)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

/// Render the profiles pane
pub fn render_profiles_pane(
    f: &mut Frame,
    area: Rect,
    profiles: &[String],
    active_profiles: &[String],
    list_state: &mut ListState,
    is_focused: bool,
) {
    let block = Block::default()
        .title("Profiles")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Theme::FOCUS_STYLE
        } else {
            Theme::DEFAULT_STYLE
        });

    let items: Vec<ListItem> = profiles
        .iter()
        .map(|p| {
            let prefix = if active_profiles.contains(p) {
                "* "
            } else {
                "  "
            };
            let style = if active_profiles.contains(p) {
                Theme::ACTIVE_PROFILE_STYLE
            } else {
                Theme::DEFAULT_STYLE
            };
            ListItem::new(Line::from(Span::styled(format!("{prefix}{p}"), style)))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Theme::DEFAULT_STYLE)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

/// Render the output pane
pub fn render_output_pane(
    f: &mut Frame,
    area: Rect,
    command_output: &[String],
    output_offset: usize,
    is_focused: bool,
    search_line_style_fn: impl Fn(usize) -> Option<Vec<(Style, std::ops::Range<usize>)>>,
    is_search_active: bool,
) {
    let output_block = Block::default()
        .title("Output")
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Theme::FOCUS_STYLE
        } else {
            Theme::DEFAULT_STYLE
        });

    let output_lines = if command_output.is_empty() {
        vec![Line::from("Run a command to see Maven output.")]
    } else {
        command_output
            .iter()
            .enumerate()
            .map(|(line_index, line)| {
                if is_search_active {
                    // In search mode: use search highlighting over cleaned text
                    let cleaned = crate::utils::clean_log_line(line).unwrap_or_default();
                    if let Some(highlights) = search_line_style_fn(line_index) {
                        let mut spans = Vec::new();
                        let mut last_end = 0;
                        for (style, range) in highlights {
                            if range.start > last_end {
                                spans.push(Span::raw(cleaned[last_end..range.start].to_string()));
                            }
                            if range.end <= cleaned.len() {
                                spans.push(Span::styled(cleaned[range.clone()].to_string(), style));
                                last_end = range.end;
                            }
                        }
                        if last_end < cleaned.len() {
                            spans.push(Span::raw(cleaned[last_end..].to_string()));
                        }
                        Line::from(spans)
                    } else {
                        Line::from(cleaned)
                    }
                } else {
                    // Normal mode: use keyword-based coloring
                    let cleaned = crate::utils::clean_log_line(line).unwrap_or_default();
                    crate::utils::colorize_log_line(&cleaned)
                }
            })
            .collect()
    };

    let output_paragraph = Paragraph::new(output_lines)
        .block(output_block)
        .wrap(Wrap { trim: true })
        .scroll((output_offset.min(u16::MAX as usize) as u16, 0));

    f.render_widget(output_paragraph, area);
}

/// Render the footer with key hints and search status
pub fn render_footer(
    f: &mut Frame,
    area: Rect,
    view: CurrentView,
    focus: Focus,
    menu_state: MenuState,
    search_status_line: Option<Line<'static>>,
) {
    let mut footer_lines = crate::ui::keybindings::footer_lines(view, focus, menu_state);
    if let Some(status_line) = search_status_line {
        footer_lines.push(status_line);
    }
    let footer = Paragraph::new(footer_lines).block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, area);
}

/// Create the main layout for the TUI
pub fn create_layout(area: Rect) -> (Rect, Rect, Rect) {
    let footer_height = 9; // accommodates multi-line footer including optional search status
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(footer_height)].as_ref())
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(vertical[0]);

    (content_chunks[0], content_chunks[1], vertical[1])
}
