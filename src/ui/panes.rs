use crate::ui::keybindings::{CurrentView, Focus};
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
        .map(|m| {
            let display_name = if m == "." {
                "(root project)"
            } else {
                m.as_str()
            };
            ListItem::new(Line::from(display_name))
        })
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
    let title = if active_profiles.is_empty() {
        "Profiles".to_string()
    } else {
        format!("Profiles ({})", active_profiles.len())
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Theme::FOCUS_STYLE
        } else {
            Theme::DEFAULT_STYLE
        });

    let items: Vec<ListItem> = profiles
        .iter()
        .map(|p| {
            let checkbox = if active_profiles.contains(p) {
                "☑"
            } else {
                "☐"
            };
            let style = if active_profiles.contains(p) {
                Theme::ACTIVE_PROFILE_STYLE
            } else {
                Theme::DEFAULT_STYLE
            };
            ListItem::new(Line::from(Span::styled(
                format!("{} {}", checkbox, p),
                style,
            )))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Theme::DEFAULT_STYLE)
        .highlight_style(Theme::SELECTED_STYLE)
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

/// Render the build flags pane
pub fn render_flags_pane(
    f: &mut Frame,
    area: Rect,
    flags: &[crate::ui::state::BuildFlag],
    list_state: &mut ListState,
    is_focused: bool,
) {
    let enabled_count = flags.iter().filter(|f| f.enabled).count();
    let title = if enabled_count == 0 {
        "Build Flags".to_string()
    } else {
        format!("Build Flags ({})", enabled_count)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if is_focused {
            Theme::FOCUS_STYLE
        } else {
            Theme::DEFAULT_STYLE
        });

    let items: Vec<ListItem> = flags
        .iter()
        .map(|flag| {
            let checkbox = if flag.enabled { "☑" } else { "☐" };
            let style = if flag.enabled {
                Theme::ACTIVE_PROFILE_STYLE
            } else {
                Theme::DEFAULT_STYLE
            };
            ListItem::new(Line::from(Span::styled(
                format!("{} {}", checkbox, flag.name),
                style,
            )))
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
#[allow(clippy::too_many_arguments)]
pub fn render_output_pane(
    f: &mut Frame,
    area: Rect,
    command_output: &[String],
    output_offset: usize,
    is_focused: bool,
    search_line_style_fn: impl Fn(usize) -> Option<Vec<(Style, std::ops::Range<usize>)>>,
    is_search_active: bool,
    module_name: Option<&str>,
    output_context: Option<(String, Vec<String>, Vec<String>)>,
) {
    // Build title with context
    let title = if let (Some(module), Some((cmd, profiles, flags))) = (module_name, output_context)
    {
        let mut parts = vec![module.to_string(), cmd];
        if !profiles.is_empty() {
            parts.push(profiles.join(", "));
        }
        if !flags.is_empty() {
            parts.push(flags.join(", "));
        }
        format!("Output: {}", parts.join(" • "))
    } else if let Some(module) = module_name {
        format!("Output: {}", module)
    } else {
        "Output".to_string()
    };

    let output_block = Block::default()
        .title(title)
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
) {
    let _ = focus; // Not needed in simplified footer

    let mut constraints = vec![
        Constraint::Length(1), // navigation
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

    // Navigation line
    let navigation = Paragraph::new(crate::ui::keybindings::build_navigation_line());
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
        .border_style(Theme::FOOTER_BOX_BORDER_STYLE);
    let commands_paragraph =
        Paragraph::new(crate::ui::keybindings::simplified_footer_body(view)).block(commands_block);
    f.render_widget(commands_paragraph, chunks[2]);

    // Optional search status line
    if let Some(status_line) = search_status_line {
        let status_paragraph = Paragraph::new(status_line);
        let idx = chunks.len() - 1;
        f.render_widget(status_paragraph, chunks[idx]);
    }
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
