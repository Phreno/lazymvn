use crate::maven;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use regex::Regex;
use std::{collections::HashMap, path::PathBuf};

pub fn draw<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut TuiState,
) -> Result<(), std::io::Error> {
    terminal.draw(|f| {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.area());

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(vertical[0]);

        match state.current_view {
            CurrentView::Modules => {
                // Modules panel
                let modules_block = Block::default()
                    .title("Modules")
                    .borders(Borders::ALL)
                    .border_style(if state.focus == Focus::Modules {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });
                let list_items: Vec<ListItem> = state
                    .modules
                    .iter()
                    .map(|m| ListItem::new(m.as_str()))
                    .collect();
                let list = List::new(list_items)
                    .block(modules_block)
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    )
                    .highlight_symbol("> ");
                f.render_stateful_widget(list, content_chunks[0], &mut state.modules_list_state);
            }
            CurrentView::Profiles => {
                // Profiles panel
                let profiles_block = Block::default()
                    .title("Profiles")
                    .borders(Borders::ALL)
                    .border_style(if state.focus == Focus::Modules {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });
                let list_items: Vec<ListItem> = state
                    .profiles
                    .iter()
                    .map(|p| {
                        let line = if state.active_profiles.contains(p) {
                            format!("* {}", p)
                        } else {
                            format!("  {}", p)
                        };
                        ListItem::new(line)
                    })
                    .collect();
                let list = List::new(list_items)
                    .block(profiles_block)
                    .highlight_style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Yellow),
                    )
                    .highlight_symbol("> ");
                f.render_stateful_widget(list, content_chunks[0], &mut state.profiles_list_state);
            }
        }

        // Command output panel
        let output_block = Block::default()
            .title("Output")
            .borders(Borders::ALL)
            .border_style(if state.focus == Focus::Output {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        let output_area = content_chunks[1];
        let visible_height = output_area.height.saturating_sub(2);
        state.set_output_view_height(visible_height);
        let total_lines = state.command_output.len();
        let max_offset = total_lines.saturating_sub(visible_height as usize);
        if state.output_offset > max_offset {
            state.output_offset = max_offset;
        }
        let output_lines = if state.command_output.is_empty() {
            vec![Line::from("Run a command to see Maven output.")]
        } else {
            state
                .command_output
                .iter()
                .enumerate()
                .map(|(idx, line)| {
                    let mut rendered = if line.starts_with("[ERR]") {
                        Line::from(vec![
                            Span::styled("[ERR]", Style::default().fg(Color::Red)),
                            Span::raw(format!(
                                " {}",
                                line.trim_start_matches("[ERR]").trim_start()
                            )),
                        ])
                    } else {
                        Line::from(Span::raw(line.as_str()))
                    };
                    if let Some(style) = state.search_line_style(idx) {
                        rendered = rendered.style(style);
                    }
                    rendered
                })
                .collect()
        };
        let output_paragraph = Paragraph::new(output_lines)
            .block(output_block)
            .wrap(Wrap { trim: true })
            .scroll((state.output_offset.min(u16::MAX as usize) as u16, 0));
        f.render_widget(output_paragraph, output_area);

        // Footer with key hints
        let mut footer_lines = vec![Line::from(footer_spans(state.current_view, state.focus))];
        if let Some(status_line) = state.search_status_line() {
            footer_lines.push(status_line);
        }
        let footer = Paragraph::new(footer_lines).block(Block::default().borders(Borders::TOP));
        f.render_widget(footer, vertical[1]);
    })?;
    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CurrentView {
    Modules,
    Profiles,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Focus {
    Modules,
    Output,
}

#[derive(Clone, Debug, Default)]
struct ModuleOutput {
    lines: Vec<String>,
    scroll_offset: usize,
}

#[derive(Clone, Debug)]
struct SearchMatch {
    line_index: usize,
}

#[derive(Clone, Debug)]
struct SearchState {
    query: String,
    matches: Vec<SearchMatch>,
    current: usize,
}

impl SearchState {
    fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    fn current_match(&self) -> Option<&SearchMatch> {
        self.matches.get(self.current)
    }

    fn total_matches(&self) -> usize {
        self.matches.len()
    }

    fn line_contains_current(&self, line_index: usize) -> bool {
        self.current_match()
            .map(|m| m.line_index == line_index)
            .unwrap_or(false)
    }

    fn line_has_match(&self, line_index: usize) -> bool {
        self.matches.iter().any(|m| m.line_index == line_index)
    }
}

pub struct TuiState {
    pub current_view: CurrentView,
    pub focus: Focus,
    pub modules: Vec<String>,
    pub profiles: Vec<String>,
    pub active_profiles: Vec<String>,
    pub modules_list_state: ListState,
    pub profiles_list_state: ListState,
    pub command_output: Vec<String>,
    pub output_offset: usize,
    pub output_view_height: u16,
    module_outputs: HashMap<String, ModuleOutput>,
    pub project_root: PathBuf,
    search_state: Option<SearchState>,
    search_input: Option<String>,
    search_error: Option<String>,
}

impl TuiState {
    pub fn new(modules: Vec<String>, project_root: PathBuf) -> Self {
        let mut modules_list_state = ListState::default();
        let profiles_list_state = ListState::default();
        if !modules.is_empty() {
            modules_list_state.select(Some(0));
        }
        let mut state = Self {
            current_view: CurrentView::Modules,
            focus: Focus::Modules,
            modules,
            profiles: vec![],
            active_profiles: vec![],
            modules_list_state,
            profiles_list_state,
            command_output: vec![],
            output_offset: 0,
            output_view_height: 0,
            module_outputs: HashMap::new(),
            project_root,
            search_state: None,
            search_input: None,
            search_error: None,
        };
        state.sync_selected_module_output();
        state
    }

    pub fn next_item(&mut self) {
        match self.current_view {
            CurrentView::Modules => {
                if self.modules.is_empty() {
                    return;
                }
                let i = match self.modules_list_state.selected() {
                    Some(i) => (i + 1) % self.modules.len(),
                    None => 0,
                };
                self.modules_list_state.select(Some(i));
                self.sync_selected_module_output();
            }
            CurrentView::Profiles => {
                if !self.profiles.is_empty() {
                    let i = match self.profiles_list_state.selected() {
                        Some(i) => (i + 1) % self.profiles.len(),
                        None => 0,
                    };
                    self.profiles_list_state.select(Some(i));
                }
            }
        }
    }

    pub fn previous_item(&mut self) {
        match self.current_view {
            CurrentView::Modules => {
                if self.modules.is_empty() {
                    return;
                }
                let i = match self.modules_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.modules.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.modules_list_state.select(Some(i));
                self.sync_selected_module_output();
            }
            CurrentView::Profiles => {
                if !self.profiles.is_empty() {
                    let i = match self.profiles_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.profiles.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.profiles_list_state.select(Some(i));
                }
            }
        }
    }

    pub fn toggle_profile(&mut self) {
        if let Some(selected) = self.profiles_list_state.selected() {
            let profile = &self.profiles[selected];
            if self.active_profiles.contains(profile) {
                self.active_profiles.retain(|p| p != profile);
            } else {
                self.active_profiles.push(profile.clone());
            }
        }
    }

    pub fn selected_module(&self) -> Option<&str> {
        self.modules_list_state
            .selected()
            .and_then(|i| self.modules.get(i).map(|s| s.as_str()))
    }

    fn sync_selected_module_output(&mut self) {
        if let Some(module) = self.selected_module() {
            if let Some(stored) = self.module_outputs.get(module) {
                self.command_output = stored.lines.clone();
                self.output_offset = stored.scroll_offset;
            } else {
                self.command_output.clear();
                self.output_offset = 0;
            }
        } else {
            self.command_output.clear();
            self.output_offset = 0;
        }
        self.clamp_output_offset();
        self.refresh_search_matches();
    }

    fn store_current_module_output(&mut self) {
        if let Some(module) = self.selected_module() {
            self.module_outputs.insert(
                module.to_string(),
                ModuleOutput {
                    lines: self.command_output.clone(),
                    scroll_offset: self.output_offset,
                },
            );
        }
    }

    fn clear_current_module_output(&mut self) {
        if let Some(module) = self.selected_module().map(|m| m.to_string()) {
            self.command_output.clear();
            self.output_offset = 0;
            self.module_outputs.insert(module, ModuleOutput::default());
        } else {
            self.command_output.clear();
            self.output_offset = 0;
        }
        self.clamp_output_offset();
        self.refresh_search_matches();
    }

    fn run_selected_module_command(&mut self, args: &[&str]) {
        if let Some(module) = self.selected_module().map(|m| m.to_string()) {
            self.clear_current_module_output();
            let output = maven::execute_maven_command(
                &self.project_root,
                Some(module.as_str()),
                args,
                &self.active_profiles,
            )
            .unwrap_or_else(|e| vec![format!("[ERR] {e}")]);
            self.command_output = output;
            self.output_offset = self
                .command_output
                .len()
                .saturating_sub(self.output_view_height as usize);
        } else {
            self.command_output = vec!["Select a module to run commands.".to_string()];
            self.output_offset = 0;
        }
        self.clamp_output_offset();
        self.refresh_search_matches();
        self.store_current_module_output();
        self.clamp_output_offset();
    }

    pub fn set_output_view_height(&mut self, height: u16) {
        self.output_view_height = height;
        self.clamp_output_offset();
        self.ensure_current_match_visible();
    }

    fn clamp_output_offset(&mut self) {
        let max_offset = self
            .command_output
            .len()
            .saturating_sub(self.output_view_height as usize);
        if self.output_offset > max_offset {
            self.output_offset = max_offset;
            self.store_current_module_output();
        }
    }

    fn scroll_output_lines(&mut self, delta: isize) {
        if self.command_output.is_empty() {
            return;
        }
        let max_offset = self
            .command_output
            .len()
            .saturating_sub(self.output_view_height as usize);
        let current = self.output_offset as isize;
        let next = (current + delta).clamp(0, max_offset as isize) as usize;
        if next != self.output_offset {
            self.output_offset = next;
            self.store_current_module_output();
        }
    }

    fn scroll_output_pages(&mut self, delta: isize) {
        let page = self.output_view_height.max(1) as isize;
        self.scroll_output_lines(delta * page);
    }

    fn scroll_output_to_start(&mut self) {
        if self.command_output.is_empty() {
            return;
        }
        self.output_offset = 0;
        self.store_current_module_output();
    }

    fn scroll_output_to_end(&mut self) {
        let max_offset = self
            .command_output
            .len()
            .saturating_sub(self.output_view_height as usize);
        self.output_offset = max_offset;
        self.store_current_module_output();
    }

    pub fn focus_modules(&mut self) {
        self.focus = Focus::Modules;
    }

    pub fn focus_output(&mut self) {
        self.focus = Focus::Output;
    }

    fn is_search_input_active(&self) -> bool {
        self.search_input.is_some()
    }

    fn begin_search_input(&mut self) {
        let initial = self
            .search_input
            .take()
            .or_else(|| self.search_state.as_ref().map(|s| s.query.clone()))
            .unwrap_or_default();
        self.search_input = Some(initial);
        self.search_error = None;
    }

    fn cancel_search_input(&mut self) {
        self.search_input = None;
        if self
            .search_state
            .as_ref()
            .map(|s| s.has_matches())
            .unwrap_or(false)
        {
            self.search_error = None;
        } else if self.search_state.is_none() {
            self.search_error = None;
        }
    }

    fn push_search_char(&mut self, ch: char) {
        if let Some(buffer) = self.search_input.as_mut() {
            buffer.push(ch);
            self.search_error = None;
        }
    }

    fn backspace_search_char(&mut self) {
        if let Some(buffer) = self.search_input.as_mut() {
            buffer.pop();
            self.search_error = None;
        }
    }

    fn submit_search(&mut self) {
        if let Some(pattern) = self.search_input.clone() {
            if pattern.is_empty() {
                self.search_state = None;
                self.search_input = None;
                self.search_error = None;
                return;
            }
            match self.apply_search_query(pattern.clone(), false) {
                Ok(_) => {
                    self.search_input = None;
                }
                Err(err) => {
                    self.search_error = Some(err.to_string());
                    self.search_input = Some(pattern);
                }
            }
        }
    }

    fn apply_search_query(
        &mut self,
        query: String,
        keep_current: bool,
    ) -> Result<(), regex::Error> {
        let regex = Regex::new(&query)?;
        let matches = self.collect_search_matches(&regex);
        let mut current_index = 0usize;

        if keep_current {
            if let Some(existing) = self.search_state.as_ref() {
                if existing.query == query && !matches.is_empty() {
                    current_index = existing.current.min(matches.len() - 1);
                }
            }
        }

        self.search_state = Some(SearchState {
            query,
            matches,
            current: current_index,
        });

        if let Some(search) = self.search_state.as_ref() {
            if search.has_matches() {
                self.search_error = None;
                self.jump_to_match(current_index);
            } else {
                self.search_error = Some("No matches found".to_string());
            }
        }

        Ok(())
    }

    fn collect_search_matches(&self, regex: &Regex) -> Vec<SearchMatch> {
        self.command_output
            .iter()
            .enumerate()
            .flat_map(|(line_index, line)| {
                regex
                    .find_iter(line)
                    .map(move |_| SearchMatch { line_index })
            })
            .collect()
    }

    fn refresh_search_matches(&mut self) {
        if let Some(existing) = self.search_state.as_ref().cloned() {
            let _ = self.apply_search_query(existing.query, true);
        } else {
            self.search_error = None;
        }
    }

    fn next_search_match(&mut self) {
        if let Some(search) = self.search_state.as_ref() {
            if search.matches.is_empty() {
                return;
            }
            let next = (search.current + 1) % search.matches.len();
            self.jump_to_match(next);
        }
    }

    fn previous_search_match(&mut self) {
        if let Some(search) = self.search_state.as_ref() {
            if search.matches.is_empty() {
                return;
            }
            let len = search.matches.len();
            let next = if search.current == 0 {
                len - 1
            } else {
                search.current - 1
            };
            self.jump_to_match(next);
        }
    }

    fn jump_to_match(&mut self, index: usize) {
        if let Some(search) = self.search_state.as_mut() {
            if search.matches.is_empty() {
                return;
            }
            let idx = index % search.matches.len();
            search.current = idx;
            let line_index = search.matches[idx].line_index;
            self.scroll_match_into_view(line_index);
        }
    }

    fn scroll_match_into_view(&mut self, line_index: usize) {
        if self.output_view_height == 0 {
            return;
        }
        let height = self.output_view_height as usize;
        let max_offset = self.command_output.len().saturating_sub(height);
        let mut new_offset = self.output_offset;
        if line_index < self.output_offset {
            new_offset = line_index;
        } else if line_index >= self.output_offset + height {
            new_offset = line_index + 1 - height;
        }
        if new_offset > max_offset {
            new_offset = max_offset;
        }
        if new_offset != self.output_offset {
            self.output_offset = new_offset;
            self.store_current_module_output();
        }
    }

    fn ensure_current_match_visible(&mut self) {
        if let Some(search) = self.search_state.as_ref() {
            if let Some(current) = search.current_match() {
                self.scroll_match_into_view(current.line_index);
            }
        }
    }

    fn search_line_style(&self, line_index: usize) -> Option<Style> {
        self.search_state.as_ref().and_then(|search| {
            if !search.has_matches() {
                return None;
            }
            if search.line_contains_current(line_index) {
                Some(
                    Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else if search.line_has_match(line_index) {
                Some(Style::default().bg(Color::DarkGray))
            } else {
                None
            }
        })
    }

    fn search_status_line(&self) -> Option<Line<'static>> {
        if let Some(buffer) = self.search_input.as_ref() {
            let mut display = buffer.clone();
            display.push('_');
            return Some(Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::from(display),
            ]));
        }
        if let Some(error) = self.search_error.as_ref() {
            return Some(Line::from(vec![
                Span::styled("Search error:", Style::default().fg(Color::Red)),
                Span::from(format!(" {error}")),
            ]));
        }
        if let Some(search) = self.search_state.as_ref() {
            if search.query.is_empty() {
                return None;
            }
            if search.has_matches() {
                return Some(Line::from(vec![
                    Span::styled("Search", Style::default().fg(Color::Yellow)),
                    Span::from(format!(
                        " Match {}/{}   /{}",
                        search.current + 1,
                        search.total_matches(),
                        search.query
                    )),
                ]));
            } else {
                return Some(Line::from(vec![
                    Span::styled("Search", Style::default().fg(Color::Yellow)),
                    Span::from(format!(" No matches   /{}", search.query)),
                ]));
            }
        }
        None
    }
}

pub fn handle_key_event(key: KeyEvent, state: &mut TuiState) {
    if state.is_search_input_active() {
        match key.code {
            KeyCode::Enter => state.submit_search(),
            KeyCode::Esc => state.cancel_search_input(),
            KeyCode::Backspace => state.backspace_search_char(),
            KeyCode::Char(ch) => state.push_search_char(ch),
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Left => state.focus_modules(),
        KeyCode::Right => state.focus_output(),
        KeyCode::Down => match state.focus {
            Focus::Modules => state.next_item(),
            Focus::Output => state.scroll_output_lines(1),
        },
        KeyCode::Up => match state.focus {
            Focus::Modules => state.previous_item(),
            Focus::Output => state.scroll_output_lines(-1),
        },
        KeyCode::Char('p') => match state.current_view {
            CurrentView::Modules => {
                if state.profiles.is_empty() {
                    state.profiles = maven::get_profiles(&state.project_root)
                        .unwrap_or_else(|e| vec![e.to_string()]);
                }
                state.current_view = CurrentView::Profiles;
                state.focus_modules();
            }
            CurrentView::Profiles => {
                state.current_view = CurrentView::Modules;
                state.focus_modules();
                state.sync_selected_module_output();
            }
        },
        KeyCode::Char('b') => {
            let args = &["-T1C", "-DskipTests", "package"];
            state.run_selected_module_command(args);
        }
        KeyCode::Char('t') => {
            let args = &["test"];
            state.run_selected_module_command(args);
        }
        KeyCode::Char('c') => {
            let args = &["clean"];
            state.run_selected_module_command(args);
        }
        KeyCode::Char('i') => {
            let args = &["-DskipTests", "install"];
            state.run_selected_module_command(args);
        }
        KeyCode::Char('d') => {
            let args = &["dependency:tree"];
            state.run_selected_module_command(args);
        }
        KeyCode::Char('/') => {
            if state.focus == Focus::Output {
                state.begin_search_input();
            }
        }
        KeyCode::Char('n') => {
            if state.focus == Focus::Output {
                state.next_search_match();
            }
        }
        KeyCode::Char('N') => {
            if state.focus == Focus::Output {
                state.previous_search_match();
            }
        }
        KeyCode::PageUp => {
            if state.focus == Focus::Output {
                state.scroll_output_pages(-1);
            }
        }
        KeyCode::PageDown => {
            if state.focus == Focus::Output {
                state.scroll_output_pages(1);
            }
        }
        KeyCode::Home => {
            if state.focus == Focus::Output {
                state.scroll_output_to_start();
            }
        }
        KeyCode::End => {
            if state.focus == Focus::Output {
                state.scroll_output_to_end();
            }
        }
        KeyCode::Enter => {
            if state.current_view == CurrentView::Profiles {
                state.toggle_profile();
            }
        }
        _ => {}
    }
}

fn footer_spans(view: CurrentView, focus: Focus) -> Vec<Span<'static>> {
    let mut hints: Vec<(&str, &str)> = vec![("←/→", "Focus")];

    match focus {
        Focus::Modules => {
            let label = match view {
                CurrentView::Modules => "Select",
                CurrentView::Profiles => "Move",
            };
            hints.push(("↑/↓", label));
            hints.push((
                "p",
                match view {
                    CurrentView::Modules => "Profiles",
                    CurrentView::Profiles => "Back to modules",
                },
            ));
            if matches!(view, CurrentView::Profiles) {
                hints.push(("Enter", "Toggle profile"));
            }
        }
        Focus::Output => {
            hints.push(("↑/↓", "Scroll"));
            hints.push(("PgUp", "Page up"));
            hints.push(("PgDn", "Page down"));
            hints.push(("Home", "Top"));
            hints.push(("End", "Bottom"));
            hints.push(("/", "Search"));
            hints.push(("n", "Next match"));
            hints.push(("N", "Prev match"));
        }
    }

    hints.extend_from_slice(&[
        ("b", "Package"),
        ("t", "Test"),
        ("c", "Clean"),
        ("i", "Install"),
        ("d", "Deps"),
        ("q", "Quit"),
    ]);

    let mut spans = Vec::with_capacity(hints.len() * 3);
    for (idx, (key, label)) in hints.iter().enumerate() {
        spans.push(Span::styled(
            format!(" {key} "),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(format!(" {label} ")));
        if idx < hints.len() - 1 {
            spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use tempfile::tempdir;

    #[test]
    fn test_draw_ui() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let modules = vec!["module1".to_string(), "module2".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = TuiState::new(modules, project_root);
        state.command_output = vec!["output1".to_string(), "output2".to_string()];
        state.store_current_module_output();

        // Modules view renders expected sections and footer hints
        draw(&mut terminal, &mut state).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(rendered.contains("Modules"));
        assert!(rendered.contains("Output"));
        assert!(rendered.contains("Focus"));
        assert!(rendered.contains("Select"));

        // Switching focus to output updates footer copy
        handle_key_event(KeyEvent::from(KeyCode::Right), &mut state);
        draw(&mut terminal, &mut state).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(rendered.contains("Scroll"));

        // Profiles view toggles footer copy and highlights active profile
        handle_key_event(KeyEvent::from(KeyCode::Left), &mut state);
        state.current_view = CurrentView::Profiles;
        state.profiles = vec!["profile1".to_string(), "profile2".to_string()];
        state.active_profiles = vec!["profile1".to_string()];
        draw(&mut terminal, &mut state).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(rendered.contains("Profiles"));
        assert!(rendered.contains("* profile1"));
        assert!(rendered.contains("Toggle profile"));
    }

    #[test]
    fn test_key_events() {
        let modules = vec![
            "module1".to_string(),
            "module2".to_string(),
            "module3".to_string(),
        ];
        let project_root = PathBuf::from("/");
        let mut state = TuiState::new(modules, project_root);

        // Test initial state
        assert_eq!(state.modules_list_state.selected(), Some(0));

        // Test moving down
        handle_key_event(KeyEvent::from(KeyCode::Down), &mut state);
        assert_eq!(state.modules_list_state.selected(), Some(1));

        // Test moving down again
        handle_key_event(KeyEvent::from(KeyCode::Down), &mut state);
        assert_eq!(state.modules_list_state.selected(), Some(2));

        // Test wrapping around
        handle_key_event(KeyEvent::from(KeyCode::Down), &mut state);
        assert_eq!(state.modules_list_state.selected(), Some(0));

        // Test moving up
        handle_key_event(KeyEvent::from(KeyCode::Up), &mut state);
        assert_eq!(state.modules_list_state.selected(), Some(2));

        // Test moving up again
        handle_key_event(KeyEvent::from(KeyCode::Up), &mut state);
        assert_eq!(state.modules_list_state.selected(), Some(1));
    }

    #[test]
    fn test_footer_spans_content() {
        let modules_text: String = footer_spans(CurrentView::Modules, Focus::Modules)
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(modules_text.contains("Select"));
        assert!(modules_text.contains("Profiles"));

        let output_text: String = footer_spans(CurrentView::Modules, Focus::Output)
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(output_text.contains("Scroll"));
        assert!(output_text.contains("Page down"));
        assert!(output_text.contains("Search"));
        assert!(output_text.contains("Next match"));
        assert!(output_text.contains("Prev match"));

        let profiles_text: String = footer_spans(CurrentView::Profiles, Focus::Modules)
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(profiles_text.contains("Toggle profile"));
        assert!(profiles_text.contains("Back to modules"));
    }

    #[test]
    fn test_output_scroll_controls() {
        let backend = TestBackend::new(80, 18);
        let mut terminal = Terminal::new(backend).unwrap();
        let modules = vec!["module".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = TuiState::new(modules, project_root);
        state.command_output = (0..40).map(|i| format!("line {i}")).collect();
        state.output_offset = state.command_output.len();
        state.store_current_module_output();
        handle_key_event(KeyEvent::from(KeyCode::Right), &mut state);

        // Initial draw snaps scroll to bottom
        draw(&mut terminal, &mut state).unwrap();
        let max_scroll = state
            .command_output
            .len()
            .saturating_sub(state.output_view_height as usize);
        assert_eq!(state.output_offset, max_scroll);
        state.store_current_module_output();

        // Page up moves toward the top
        handle_key_event(KeyEvent::from(KeyCode::PageUp), &mut state);
        draw(&mut terminal, &mut state).unwrap();
        assert!(state.output_offset < max_scroll);

        // End jumps back to the bottom
        handle_key_event(KeyEvent::from(KeyCode::End), &mut state);
        draw(&mut terminal, &mut state).unwrap();
        assert_eq!(state.output_offset, max_scroll);

        // Home goes to the top
        handle_key_event(KeyEvent::from(KeyCode::Home), &mut state);
        draw(&mut terminal, &mut state).unwrap();
        assert_eq!(state.output_offset, 0);
    }

    #[test]
    fn test_output_search_navigation() {
        let modules = vec!["module".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = TuiState::new(modules, project_root);
        state.command_output = vec![
            "alpha".to_string(),
            "beta".to_string(),
            "gamma beta".to_string(),
            "delta".to_string(),
        ];
        state.set_output_view_height(2);
        state.scroll_output_to_end();
        state.focus_output();

        handle_key_event(KeyEvent::from(KeyCode::Char('/')), &mut state);
        assert!(state.is_search_input_active());
        for ch in ['b', 'e', 't', 'a'] {
            handle_key_event(KeyEvent::from(KeyCode::Char(ch)), &mut state);
        }
        handle_key_event(KeyEvent::from(KeyCode::Enter), &mut state);

        let search = state.search_state.as_ref().expect("search state");
        assert_eq!(search.matches.len(), 2);
        assert_eq!(search.matches[0].line_index, 1);
        assert_eq!(search.current, 0);
        assert!(state.search_error.is_none());

        let status = state.search_status_line().expect("status line");
        let status_text: String = status
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(status_text.contains("Match 1/2"));

        let height = state.output_view_height as usize;
        let current_line = search.matches[search.current].line_index;
        assert!(current_line < state.output_offset + height);

        handle_key_event(KeyEvent::from(KeyCode::Char('n')), &mut state);
        let search = state.search_state.as_ref().unwrap();
        assert_eq!(search.current, 1);
        let current_line = search.matches[search.current].line_index;
        assert!(current_line >= state.output_offset);
        assert!(current_line < state.output_offset + height);

        handle_key_event(KeyEvent::from(KeyCode::Char('N')), &mut state);
        let search = state.search_state.as_ref().unwrap();
        assert_eq!(search.current, 0);
    }

    #[test]
    fn test_output_search_error_handling() {
        let modules = vec!["module".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = TuiState::new(modules, project_root);
        state.command_output = vec!["alpha".to_string()];
        state.set_output_view_height(2);
        state.focus_output();

        handle_key_event(KeyEvent::from(KeyCode::Char('/')), &mut state);
        handle_key_event(KeyEvent::from(KeyCode::Char('[')), &mut state);
        handle_key_event(KeyEvent::from(KeyCode::Enter), &mut state);

        assert!(state.is_search_input_active());
        assert!(state.search_error.is_some());

        handle_key_event(KeyEvent::from(KeyCode::Esc), &mut state);
        assert!(state.search_input.is_none());
        assert!(state.search_error.is_none());
    }

    #[test]
    fn test_build_command() {
        // 1. Setup temp project
        let project_dir = tempdir().unwrap();
        let project_root = project_dir.path();

        // 2. Create mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        let mut mvnw_file = std::fs::File::create(&mvnw_path).unwrap();
        use std::io::Write;
        mvnw_file.write_all(b"#!/bin/sh\necho $@").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = mvnw_file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        mvnw_file.set_permissions(perms).unwrap();
        drop(mvnw_file);

        // 3. Create TuiState
        let modules = vec!["module1".to_string()];
        let mut state = TuiState::new(modules, project_root.to_path_buf());
        state.active_profiles = vec!["p1".to_string()];

        // 4. Simulate 'b' key press
        handle_key_event(KeyEvent::from(KeyCode::Char('b')), &mut state);

        // 5. Assert command output
        assert_eq!(
            state.command_output,
            vec!["-P p1 -pl module1 -T1C -DskipTests package"]
        );
    }

    #[test]
    fn test_other_commands() {
        // 1. Setup temp project
        let project_dir = tempdir().unwrap();
        let project_root = project_dir.path();

        // 2. Create mock mvnw script
        let mvnw_path = project_root.join("mvnw");
        let mut mvnw_file = std::fs::File::create(&mvnw_path).unwrap();
        use std::io::Write;
        mvnw_file.write_all(b"#!/bin/sh\necho $@").unwrap(); // The script will echo all arguments
        use std::os::unix::fs::PermissionsExt;
        let mut perms = mvnw_file.metadata().unwrap().permissions();
        perms.set_mode(0o755);
        mvnw_file.set_permissions(perms).unwrap();
        drop(mvnw_file);

        // 3. Create TuiState
        let modules = vec!["module1".to_string()];
        let mut state = TuiState::new(modules, project_root.to_path_buf());
        state.active_profiles = vec!["p1".to_string()];

        // 4. Simulate key presses and assert command output
        handle_key_event(KeyEvent::from(KeyCode::Char('t')), &mut state);
        assert_eq!(state.command_output, vec!["-P p1 -pl module1 test"]);

        handle_key_event(KeyEvent::from(KeyCode::Char('c')), &mut state);
        assert_eq!(state.command_output, vec!["-P p1 -pl module1 clean"]);

        handle_key_event(KeyEvent::from(KeyCode::Char('i')), &mut state);
        assert_eq!(
            state.command_output,
            vec!["-P p1 -pl module1 -DskipTests install"]
        );

        handle_key_event(KeyEvent::from(KeyCode::Char('d')), &mut state);
        assert_eq!(
            state.command_output,
            vec!["-P p1 -pl module1 dependency:tree"]
        );
    }

    #[test]
    fn test_view_switching() {
        let modules = vec!["module1".to_string()];
        let project_root = PathBuf::from("/");
        let mut state = TuiState::new(modules, project_root);

        // Initial view is Modules
        assert_eq!(state.current_view, CurrentView::Modules);

        // Press 'p' to switch to Profiles
        handle_key_event(KeyEvent::from(KeyCode::Char('p')), &mut state);
        assert_eq!(state.current_view, CurrentView::Profiles);

        // Press 'p' again to switch back to Modules
        handle_key_event(KeyEvent::from(KeyCode::Char('p')), &mut state);
        assert_eq!(state.current_view, CurrentView::Modules);
    }

    #[test]
    fn test_profile_activation() {
        let modules = vec![];
        let project_root = PathBuf::from("/");
        let mut state = TuiState::new(modules, project_root);
        state.profiles = vec!["profile1".to_string(), "profile2".to_string()];
        state.current_view = CurrentView::Profiles;
        state.profiles_list_state.select(Some(0));

        // Activate profile1
        handle_key_event(KeyEvent::from(KeyCode::Enter), &mut state);
        assert_eq!(state.active_profiles, vec!["profile1"]);

        // Activate profile2
        state.profiles_list_state.select(Some(1));
        handle_key_event(KeyEvent::from(KeyCode::Enter), &mut state);
        assert_eq!(state.active_profiles, vec!["profile1", "profile2"]);

        // Deactivate profile1
        state.profiles_list_state.select(Some(0));
        handle_key_event(KeyEvent::from(KeyCode::Enter), &mut state);
        assert_eq!(state.active_profiles, vec!["profile2"]);
    }
}
