use crate::ui::keybindings::{CurrentView, Focus};
use crate::ui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
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
                    // Normal mode: use keyword-based coloring
                    let cleaned = crate::utils::clean_log_line(line).unwrap_or_default();
                    crate::utils::colorize_log_line(&cleaned)
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
        Constraint::Length(2), // navigation (2 lines)
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
        Paragraph::new(crate::ui::keybindings::simplified_footer_body(view)).block(commands_block);
    f.render_widget(commands_paragraph, chunks[2]);

    // Optional search status line
    if let Some(status_line) = search_status_line {
        let status_paragraph = Paragraph::new(status_line);
        let idx = chunks.len() - 1;
        f.render_widget(status_paragraph, chunks[idx]);
    }
}

/// Create an adaptive layout that responds to terminal size and focused pane
pub fn create_adaptive_layout(
    area: Rect,
    focused_pane: Option<Focus>,
) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    let footer_height = 9;

    // Determine layout mode based on terminal size
    let is_narrow = area.width < 80; // Narrow width threshold
    let is_short = area.height < 30; // Short height threshold

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(footer_height)].as_ref())
        .split(area);

    // Adaptive width layout
    if is_narrow {
        // Single column mode - stack everything vertically
        create_single_column_layout(vertical[0], vertical[1], focused_pane, is_short)
    } else {
        // Two column mode - left panes and output
        create_two_column_layout(vertical[0], vertical[1], focused_pane, is_short)
    }
}

/// Create single column layout for narrow terminals
fn create_single_column_layout(
    content_area: Rect,
    footer_area: Rect,
    focused_pane: Option<crate::ui::keybindings::Focus>,
    is_short: bool,
) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    use crate::ui::keybindings::Focus;

    // In single column, show focused pane expanded, others collapsed
    let constraints = if is_short {
        // Very restrictive - only show focused pane
        match focused_pane {
            Some(Focus::Projects) => vec![
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
            Some(Focus::Modules) => vec![
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
            Some(Focus::Profiles) => vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
            Some(Focus::Flags) => vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
            ],
            Some(Focus::Output) | None => vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
            ],
        }
    } else {
        // Normal single column - show all with focus expanded
        match focused_pane {
            Some(Focus::Projects) => vec![
                Constraint::Percentage(40),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ],
            Some(Focus::Modules) => vec![
                Constraint::Percentage(15),
                Constraint::Percentage(40),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ],
            Some(Focus::Profiles) => vec![
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(40),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ],
            Some(Focus::Flags) => vec![
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(40),
                Constraint::Percentage(15),
            ],
            Some(Focus::Output) | None => vec![
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(40),
            ],
        }
    };

    let blocks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(content_area);

    (
        blocks[0],
        blocks[1],
        blocks[2],
        blocks[3],
        blocks[4],
        footer_area,
    )
}

/// Create two column layout for normal/wide terminals
fn create_two_column_layout(
    content_area: Rect,
    footer_area: Rect,
    focused_pane: Option<crate::ui::keybindings::Focus>,
    is_short: bool,
) -> (Rect, Rect, Rect, Rect, Rect, Rect) {
    use crate::ui::keybindings::Focus;

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(content_area);

    // Adaptive left pane layout based on height and focus
    let left_constraints = if is_short {
        // Short height - expand focused pane, collapse others
        match focused_pane {
            Some(Focus::Projects) => vec![
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ],
            Some(Focus::Modules) => vec![
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
            ],
            Some(Focus::Profiles) => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ],
            Some(Focus::Flags) => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
            ],
            Some(Focus::Output) | None => vec![
                Constraint::Length(3),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ],
        }
    } else {
        // Normal height - standard layout
        vec![
            Constraint::Length(3),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]
    };

    let left_blocks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(left_constraints)
        .split(content_chunks[0]);

    (
        left_blocks[0],
        left_blocks[1],
        left_blocks[2],
        left_blocks[3],
        content_chunks[1],
        footer_area,
    )
}

/// Render popup for selecting a Spring Boot starter
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
    starters: &[crate::starters::Starter],
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
    history: &[crate::history::HistoryEntry],
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
    favorites: &[crate::favorites::Favorite],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_layout_narrow_terminal() {
        // Narrow terminal (< 80 cols) should use single column layout
        let area = Rect {
            x: 0,
            y: 0,
            width: 60,
            height: 40,
        };

        let (_, modules_area, _, _, output_area, _) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // In single column, all panes should have the same x position (stacked vertically)
        assert_eq!(modules_area.x, output_area.x);
        // Modules should be below projects
        assert!(modules_area.y > 0);
        // Output should be below modules
        assert!(output_area.y > modules_area.y);
    }

    #[test]
    fn test_adaptive_layout_wide_terminal() {
        // Wide terminal (>= 80 cols) should use two column layout
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        };

        let (_, modules_area, _, _, output_area, _) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // In two column, output should be to the right of modules
        assert!(output_area.x > modules_area.x);
    }

    #[test]
    fn test_adaptive_layout_short_height_expands_focused() {
        // Short terminal (< 30 rows) should collapse non-focused panes
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 20,
        };

        // Focus on modules
        let (projects_area, modules_area, profiles_area, _, _, _) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // Modules (focused) should have more height than others
        assert!(modules_area.height > projects_area.height);
        assert!(modules_area.height > profiles_area.height);
    }

    #[test]
    fn test_adaptive_layout_normal_height_standard_layout() {
        // Normal height terminal should use standard layout
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 40,
        };

        let (projects_area, modules_area, profiles_area, flags_area, _, _) =
            create_adaptive_layout(area, Some(Focus::Modules));

        // Projects should be small (length 3)
        assert_eq!(projects_area.height, 3);
        // Other panes should have reasonable sizes
        assert!(modules_area.height > 5);
        assert!(profiles_area.height > 5);
        assert!(flags_area.height > 5);
    }
}
