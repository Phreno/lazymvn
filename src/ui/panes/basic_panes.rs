//! Basic UI pane rendering
//!
//! This module contains rendering functions for the main application panes:
//! projects, modules, profiles, flags, and output display.

use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

/// Render the projects pane (shows project root name and Git branch if available)
pub fn render_projects_pane(
    f: &mut Frame,
    area: Rect,
    project_root: &str,
    git_branch: Option<&str>,
    is_focused: bool,
) {
    let block = Block::default()
        .title("[1] Projects")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(if is_focused {
            Theme::FOCUS_STYLE
        } else {
            Theme::DEFAULT_STYLE
        });

    // Build display text with branch on same line if available
    let text = if let Some(branch) = git_branch {
        Line::from(vec![
            Span::styled(project_root, Style::default()),
            Span::styled("  ", Style::default()),
            Span::styled("", Style::default().fg(ratatui::style::Color::Green)),
            Span::styled(
                format!(" {}", branch),
                Style::default()
                    .fg(ratatui::style::Color::Green)
                    .add_modifier(ratatui::style::Modifier::ITALIC),
            ),
        ])
    } else {
        Line::from(project_root)
    };

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

/// Render the modules pane
pub fn render_modules_pane(
    f: &mut Frame,
    area: Rect,
    modules: &[String],
    list_state: &mut ListState,
    is_focused: bool,
) {
    let block = Block::default()
        .title("[2] Modules")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
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
    profiles: &[crate::ui::state::MavenProfile],
    list_state: &mut ListState,
    is_focused: bool,
    loading_status: &crate::ui::state::ProfileLoadingStatus,
    spinner: &str,
) {
    use crate::ui::state::ProfileLoadingStatus;

    let active_count = profiles.iter().filter(|p| p.is_active()).count();

    let title = match loading_status {
        ProfileLoadingStatus::Loading => format!("[3] Profiles ({} loading...)", spinner),
        ProfileLoadingStatus::Error(_) => "[3] Profiles (error)".to_string(),
        ProfileLoadingStatus::Loaded => {
            if active_count == 0 {
                "[3] Profiles".to_string()
            } else {
                format!("[3] Profiles ({})", active_count)
            }
        }
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(if is_focused {
            Theme::FOCUS_STYLE
        } else {
            Theme::DEFAULT_STYLE
        });

    // Show loading/error message if profiles are not loaded yet
    if matches!(loading_status, ProfileLoadingStatus::Loading) {
        let loading_text = Paragraph::new(format!("{} Discovering Maven profiles...", spinner))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Theme::INFO_STYLE);
        f.render_widget(loading_text, area);
        return;
    } else if let ProfileLoadingStatus::Error(err) = loading_status {
        let error_text = Paragraph::new(format!("✗ Error loading profiles:\n{}", err))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Theme::ERROR_STYLE);
        f.render_widget(error_text, area);
        return;
    }

    let items: Vec<ListItem> = profiles
        .iter()
        .map(|p| {
            use crate::ui::state::ProfileState;

            let (checkbox, suffix, style) = match p.state {
                ProfileState::Default => {
                    if p.auto_activated {
                        ("☑", " (auto)", Theme::AUTO_PROFILE_STYLE)
                    } else {
                        ("☐", "", Theme::DEFAULT_STYLE)
                    }
                }
                ProfileState::ExplicitlyEnabled => ("☑", "", Theme::ACTIVE_PROFILE_STYLE),
                ProfileState::ExplicitlyDisabled => {
                    ("☒", " (disabled)", Theme::DISABLED_PROFILE_STYLE)
                }
            };

            ListItem::new(Line::from(Span::styled(
                format!("{} {}{}", checkbox, p.name, suffix),
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
        "[4] Build Flags".to_string()
    } else {
        format!("[4] Build Flags ({})", enabled_count)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
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
    is_command_running: bool,
    elapsed_seconds: Option<u64>,
    log_format: Option<&str>,
) {
    // Build title with context and running indicator
    let mut title =
        if let (Some(module), Some((cmd, profiles, flags))) = (module_name, output_context) {
            let mut parts = vec![module.to_string(), cmd];
            if !profiles.is_empty() {
                parts.push(profiles.join(", "));
            }
            if !flags.is_empty() {
                parts.push(flags.join(", "));
            }
            format!("[0] Output: {}", parts.join(" • "))
        } else if let Some(module) = module_name {
            format!("[0] Output: {}", module)
        } else {
            "Output".to_string()
        };

    // Add running indicator with spinner
    if is_command_running {
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame_idx = elapsed_seconds.unwrap_or(0) as usize % spinner_frames.len();
        let spinner = spinner_frames[frame_idx];
        title = format!(
            "{} {} Running ({}s)",
            title,
            spinner,
            elapsed_seconds.unwrap_or(0)
        );
    }

    let output_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(if is_focused {
            Theme::FOCUS_STYLE
        } else {
            Theme::DEFAULT_STYLE
        });

    let output_lines = if command_output.is_empty() {
        vec![Line::from("Run a command to see Maven output.")]
    } else {
        // Check if we're displaying XML (profile XML starts with "Profile:")
        let is_xml_mode = command_output
            .first()
            .map(|s| s.starts_with("Profile:"))
            .unwrap_or(false);

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
                } else if is_xml_mode && line_index >= 3 {
                    // XML mode: colorize XML syntax (skip first 3 lines: header)
                    // Don't use clean_log_line as it may trim - use raw line
                    crate::utils::colorize_xml_line(line)
                } else {
                    // Normal mode: use keyword-based coloring with log format for package extraction
                    let cleaned = crate::utils::clean_log_line(line).unwrap_or_default();
                    crate::utils::colorize_log_line_with_format(&cleaned, log_format)
                }
            })
            .collect()
    };

    // Check if we're in XML mode to disable trim
    let is_xml_mode = command_output
        .first()
        .map(|s| s.starts_with("Profile:"))
        .unwrap_or(false);

    let output_paragraph = Paragraph::new(output_lines)
        .block(output_block)
        .wrap(Wrap { trim: !is_xml_mode }) // Don't trim in XML mode to preserve indentation
        .scroll((output_offset.min(u16::MAX as usize) as u16, 0));

    f.render_widget(output_paragraph, area);
}
