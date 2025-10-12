use crate::maven;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::PathBuf;

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
                let modules_block = Block::default().title("Modules").borders(Borders::ALL);
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
                let profiles_block = Block::default().title("Profiles").borders(Borders::ALL);
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
        let output_block = Block::default().title("Output").borders(Borders::ALL);
        let output_items: Vec<ListItem> = state
            .command_output
            .iter()
            .map(|m| ListItem::new(m.as_str()))
            .collect();
        let output_list = List::new(output_items).block(output_block);
        f.render_widget(output_list, content_chunks[1]);

        // Footer with key hints
        let footer_spans = footer_spans(state.current_view);
        let footer =
            Paragraph::new(Line::from(footer_spans)).block(Block::default().borders(Borders::TOP));
        f.render_widget(footer, vertical[1]);
    })?;
    Ok(())
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CurrentView {
    Modules,
    Profiles,
}

pub struct TuiState {
    pub current_view: CurrentView,
    pub modules: Vec<String>,
    pub profiles: Vec<String>,
    pub active_profiles: Vec<String>,
    pub modules_list_state: ListState,
    pub profiles_list_state: ListState,
    pub command_output: Vec<String>,
    pub project_root: PathBuf,
}

impl TuiState {
    pub fn new(modules: Vec<String>, project_root: PathBuf) -> Self {
        let mut modules_list_state = ListState::default();
        modules_list_state.select(Some(0));
        let profiles_list_state = ListState::default();
        Self {
            current_view: CurrentView::Modules,
            modules,
            profiles: vec![],
            active_profiles: vec![],
            modules_list_state,
            profiles_list_state,
            command_output: vec![],
            project_root,
        }
    }

    pub fn next_item(&mut self) {
        match self.current_view {
            CurrentView::Modules => {
                let i = match self.modules_list_state.selected() {
                    Some(i) => (i + 1) % self.modules.len(),
                    None => 0,
                };
                self.modules_list_state.select(Some(i));
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
}

pub fn handle_key_event(key: KeyEvent, state: &mut TuiState) {
    match key.code {
        KeyCode::Down => state.next_item(),
        KeyCode::Up => state.previous_item(),
        KeyCode::Char('p') => match state.current_view {
            CurrentView::Modules => {
                if state.profiles.is_empty() {
                    state.profiles = maven::get_profiles(&state.project_root)
                        .unwrap_or_else(|e| vec![e.to_string()]);
                }
                state.current_view = CurrentView::Profiles;
            }
            CurrentView::Profiles => {
                state.current_view = CurrentView::Modules;
            }
        },
        KeyCode::Char('b') => {
            let args = &["-T1C", "-DskipTests", "package"];
            state.command_output =
                maven::execute_maven_command(&state.project_root, args, &state.active_profiles)
                    .unwrap_or_else(|e| vec![e.to_string()]);
        }
        KeyCode::Char('t') => {
            let args = &["test"];
            state.command_output =
                maven::execute_maven_command(&state.project_root, args, &state.active_profiles)
                    .unwrap_or_else(|e| vec![e.to_string()]);
        }
        KeyCode::Char('c') => {
            let args = &["clean"];
            state.command_output =
                maven::execute_maven_command(&state.project_root, args, &state.active_profiles)
                    .unwrap_or_else(|e| vec![e.to_string()]);
        }
        KeyCode::Char('i') => {
            let args = &["-DskipTests", "install"];
            state.command_output =
                maven::execute_maven_command(&state.project_root, args, &state.active_profiles)
                    .unwrap_or_else(|e| vec![e.to_string()]);
        }
        KeyCode::Char('d') => {
            let args = &["dependency:tree"];
            state.command_output =
                maven::execute_maven_command(&state.project_root, args, &state.active_profiles)
                    .unwrap_or_else(|e| vec![e.to_string()]);
        }
        KeyCode::Enter => {
            if state.current_view == CurrentView::Profiles {
                state.toggle_profile();
            }
        }
        _ => {}
    }
}

fn footer_spans(view: CurrentView) -> Vec<Span<'static>> {
    let mut hints: Vec<(&str, &str)> = match view {
        CurrentView::Modules => vec![("↑/↓", "Move"), ("p", "Profiles")],
        CurrentView::Profiles => vec![
            ("↑/↓", "Move"),
            ("Enter", "Toggle profile"),
            ("p", "Back to modules"),
        ],
    };

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

        // Test Modules view
        draw(&mut terminal, &mut state).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(
            rendered.contains("Modules"),
            "expected Modules pane title to render"
        );
        assert!(
            rendered.contains("Output"),
            "expected Output pane title to render"
        );
        assert!(
            rendered.contains("b  Package"),
            "expected footer to list package command"
        );

        // Test Profiles view
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
        assert!(
            rendered.contains("Profiles"),
            "expected Profiles pane title to render"
        );
        assert!(
            rendered.contains("* profile1"),
            "expected active profile to be marked"
        );
        assert!(
            rendered.contains("Enter  Toggle profile"),
            "expected footer to describe profile toggle action"
        );
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
        assert_eq!(state.command_output, vec!["-P p1 -T1C -DskipTests package"]);
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
        assert_eq!(state.command_output, vec!["-P p1 test"]);

        handle_key_event(KeyEvent::from(KeyCode::Char('c')), &mut state);
        assert_eq!(state.command_output, vec!["-P p1 clean"]);

        handle_key_event(KeyEvent::from(KeyCode::Char('i')), &mut state);
        assert_eq!(state.command_output, vec!["-P p1 -DskipTests install"]);

        handle_key_event(KeyEvent::from(KeyCode::Char('d')), &mut state);
        assert_eq!(state.command_output, vec!["-P p1 dependency:tree"]);
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
