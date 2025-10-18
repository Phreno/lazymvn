use crate::ui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::text::{Line, Span};

/// Represents the current view in the TUI
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CurrentView {
    Projects,
    Modules,
    Profiles,
    Flags,
}

/// Represents which pane currently has focus
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Focus {
    Projects,
    Modules,
    Profiles,
    Flags,
    Output,
}

impl Focus {
    /// Get the next focus in the cycle (right arrow)
    pub fn next(self) -> Self {
        match self {
            Focus::Projects => Focus::Modules,
            Focus::Modules => Focus::Profiles,
            Focus::Profiles => Focus::Flags,
            Focus::Flags => Focus::Output,
            Focus::Output => Focus::Projects,
        }
    }

    /// Get the previous focus in the cycle (left arrow)
    pub fn previous(self) -> Self {
        match self {
            Focus::Projects => Focus::Output,
            Focus::Modules => Focus::Projects,
            Focus::Profiles => Focus::Modules,
            Focus::Flags => Focus::Profiles,
            Focus::Output => Focus::Flags,
        }
    }
}

/// Search mode for input vs cycling through matches
pub enum SearchMode {
    Input,
    Cycling,
}

struct ModuleAction {
    key_display: &'static str,
    prefix: &'static str,
    suffix: &'static str,
}

const MODULE_ACTIONS: [ModuleAction; 8] = [
    ModuleAction {
        key_display: "b",
        prefix: "",
        suffix: "uild",
    },
    ModuleAction {
        key_display: "C",
        prefix: "",
        suffix: "lean",
    },
    ModuleAction {
        key_display: "c",
        prefix: "",
        suffix: "ompile",
    },
    ModuleAction {
        key_display: "k",
        prefix: "pac",
        suffix: "age",
    },
    ModuleAction {
        key_display: "t",
        prefix: "",
        suffix: "est",
    },
    ModuleAction {
        key_display: "i",
        prefix: "",
        suffix: "nstall",
    },
    ModuleAction {
        key_display: "d",
        prefix: "",
        suffix: "eps",
    },
    ModuleAction {
        key_display: "x",
        prefix: "",
        suffix: "kill",
    },
];

/// Handle key events and update TUI state accordingly
pub fn handle_key_event(key: KeyEvent, state: &mut crate::ui::state::TuiState) {
    // Only process key press events, ignore release and repeat events
    // This prevents duplicate actions on Windows and some terminals
    if key.kind != KeyEventKind::Press {
        log::debug!("Ignoring non-press key event: {:?}", key.kind);
        return;
    }

    log::debug!("Key event: {:?}", key);

    // Handle projects popup separately
    if state.show_projects_popup {
        match key.code {
            KeyCode::Down => {
                log::debug!("Navigate down in projects list");
                state.next_project();
            }
            KeyCode::Up => {
                log::debug!("Navigate up in projects list");
                state.previous_project();
            }
            KeyCode::Enter => {
                log::info!("Select project from recent list");
                state.select_current_project();
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                log::info!("Cancel project selection");
                state.hide_recent_projects();
            }
            _ => {}
        }
        return;
    }

    if let Some(search_mod) = state.search_mod.take() {
        log::debug!(
            "In search mode: {:?}",
            match search_mod {
                SearchMode::Input => "Input",
                SearchMode::Cycling => "Cycling",
            }
        );
        match search_mod {
            SearchMode::Input => {
                match key.code {
                    KeyCode::Char(ch) => {
                        log::debug!("Search input: '{}'", ch);
                        state.push_search_char(ch);
                        state.live_search();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Backspace => {
                        log::debug!("Search backspace");
                        state.backspace_search_char();
                        state.live_search();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Up => {
                        log::debug!("Search recall previous");
                        state.recall_previous_search();
                        state.live_search();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Down => {
                        log::debug!("Search recall next");
                        state.recall_next_search();
                        state.live_search();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Enter => {
                        log::debug!("Search submit");
                        state.submit_search();
                        if state.has_search_results() {
                            log::debug!("Search has results, entering cycling mode");
                            state.search_mod = Some(SearchMode::Cycling);
                        } else {
                            log::debug!("No search results, exiting search");
                            state.search_mod = None;
                        }
                    }
                    KeyCode::Esc => {
                        log::debug!("Search cancelled");
                        state.cancel_search_input();
                        state.search_mod = None;
                    }
                    _ => {
                        state.search_mod = Some(SearchMode::Input);
                    }
                }
                return;
            }
            SearchMode::Cycling => {
                match key.code {
                    KeyCode::Char('n') => {
                        state.next_search_match();
                        state.search_mod = Some(SearchMode::Cycling);
                    }
                    KeyCode::Char('N') => {
                        state.previous_search_match();
                        state.search_mod = Some(SearchMode::Cycling);
                    }
                    KeyCode::Char('/') => {
                        state.begin_search_input();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Enter | KeyCode::Esc => {
                        state.search_mod = None;
                    }
                    _ => {
                        state.search_mod = None;
                        handle_key_event(key, state);
                    }
                }
                return;
            }
        }
    }

    // Direct command execution - no menu navigation needed
    match key.code {
        KeyCode::Char('r')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            log::info!("Show recent projects");
            state.show_recent_projects();
        }
        KeyCode::Left => {
            log::debug!("Cycle focus left");
            state.cycle_focus_left();
        }
        KeyCode::Right => {
            log::debug!("Cycle focus right");
            state.cycle_focus_right();
        }
        KeyCode::Down => match state.focus {
            Focus::Output => {
                log::debug!("Scroll output down");
                state.scroll_output_lines(1);
            }
            _ => {
                log::debug!("Navigate down in list");
                state.next_item();
            }
        },
        KeyCode::Up => match state.focus {
            Focus::Output => {
                log::debug!("Scroll output up");
                state.scroll_output_lines(-1);
            }
            _ => {
                log::debug!("Navigate up in list");
                state.previous_item();
            }
        },
        KeyCode::Char('0') => {
            log::info!("Focus output pane");
            state.focus_output();
        }
        KeyCode::Char('1') => {
            log::info!("Switch to projects view");
            state.switch_to_projects();
        }
        KeyCode::Char('2') => {
            log::info!("Switch to modules view");
            state.switch_to_modules();
        }
        KeyCode::Char('b') => {
            log::info!("Execute: clean install");
            state.run_selected_module_command(&["clean", "install"]);
        }
        KeyCode::Char('C') => {
            log::info!("Execute: clean");
            state.run_selected_module_command(&["clean"]);
        }
        KeyCode::Char('c') => {
            log::info!("Execute: compile");
            state.run_selected_module_command(&["compile"]);
        }
        KeyCode::Char('k') => {
            log::info!("Execute: package");
            state.run_selected_module_command(&["package"]);
        }
        KeyCode::Char('t') => {
            log::info!("Execute: test");
            state.run_selected_module_command(&["test"]);
        }
        KeyCode::Char('i') => {
            log::info!("Execute: install");
            state.run_selected_module_command(&["install"]);
        }
        KeyCode::Char('d') => {
            log::info!("Execute: dependency:tree");
            state.run_selected_module_command(&["dependency:tree"]);
        }
        KeyCode::Char('x') => {
            log::info!("Kill running process");
            state.kill_running_process();
        }
        KeyCode::Char('3') => {
            log::info!("Switch to profiles view");
            state.switch_to_profiles();
        }
        KeyCode::Char('4') => {
            log::info!("Switch to flags view");
            state.switch_to_flags();
        }
        KeyCode::Char('/') => {
            log::info!("Begin search input");
            state.begin_search_input();
            state.search_mod = Some(SearchMode::Input);
        }
        KeyCode::Char('n') => {
            log::debug!("Next search match");
            state.next_search_match();
        }
        KeyCode::Char('N') => {
            log::debug!("Previous search match");
            state.previous_search_match();
        }
        KeyCode::PageUp => {
            log::debug!("Page up");
            state.scroll_output_pages(-1);
        }
        KeyCode::PageDown => {
            log::debug!("Page down");
            state.scroll_output_pages(1);
        }
        KeyCode::Home => {
            log::debug!("Scroll to start");
            state.scroll_output_to_start();
        }
        KeyCode::End => {
            log::debug!("Scroll to end");
            state.scroll_output_to_end();
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if state.focus == Focus::Profiles {
                state.toggle_profile();
            } else if state.focus == Focus::Flags {
                state.toggle_flag();
            }
        }
        _ => {}
    }
}

fn key_token(text: &str) -> Span<'static> {
    Span::styled(text.to_string(), Theme::KEY_HINT_STYLE)
}

fn append_bracketed_word(spans: &mut Vec<Span<'static>>, prefix: &str, key: &str, suffix: &str) {
    let key_style = Theme::KEY_HINT_STYLE;
    let text_style = Theme::DEFAULT_STYLE;

    if !prefix.is_empty() {
        spans.push(Span::styled(prefix.to_string(), text_style));
    }

    spans.push(Span::styled("[", text_style));
    spans.push(Span::styled(key.to_string(), key_style));
    spans.push(Span::styled("]", text_style));

    if !suffix.is_empty() {
        spans.push(Span::styled(suffix.to_string(), text_style));
    }
}

pub(crate) fn blank_line() -> Line<'static> {
    Line::raw("")
}

pub(crate) fn build_navigation_line() -> Line<'static> {
    let spans: Vec<Span<'static>> = vec![
        Span::styled("Views ", Theme::FOOTER_SECTION_STYLE),
        key_token("0"),
        Span::raw(" Output • "),
        key_token("1"),
        Span::raw(" Projects • "),
        key_token("2"),
        Span::raw(" Modules • "),
        key_token("3"),
        Span::raw(" Profiles • "),
        key_token("4"),
        Span::raw(" Flags  •  "),
        Span::styled("Focus ", Theme::FOOTER_SECTION_STYLE),
        key_token("←"),
        Span::raw("  "),
        key_token("→"),
        Span::raw("  •  "),
        Span::styled("Navigate ", Theme::FOOTER_SECTION_STYLE),
        key_token("↑"),
        Span::raw("  "),
        key_token("↓"),
    ];
    Line::from(spans)
}

pub(crate) fn simplified_footer_title(
    view: CurrentView,
    module_name: Option<&str>,
    active_profiles: &[String],
    enabled_flags: &[String],
) -> Span<'static> {
    let mut parts = Vec::new();

    if let Some(name) = module_name {
        let display_name = if name == "." { "(root project)" } else { name };
        parts.push(display_name.to_string());
    }

    if !active_profiles.is_empty() {
        parts.push(active_profiles.join(", "));
    }

    if !enabled_flags.is_empty() {
        parts.push(enabled_flags.join(", "));
    }

    let text = if parts.is_empty() {
        "Commands".to_string()
    } else {
        format!("Commands: {}", parts.join(" • "))
    };

    let style = match view {
        CurrentView::Projects | CurrentView::Modules => Theme::FOOTER_SECTION_STYLE,
        CurrentView::Profiles | CurrentView::Flags => Theme::FOOTER_SECTION_FOCUSED_STYLE,
    };

    Span::styled(text, style)
}

pub(crate) fn simplified_footer_body(_view: CurrentView) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::raw(" "));

    // Module commands
    for (idx, action) in MODULE_ACTIONS.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw(" "));
        }
        append_bracketed_word(&mut spans, action.prefix, action.key_display, action.suffix);
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ui::state::TuiState;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use std::path::PathBuf;

    #[test]
    fn test_key_event_only_processes_press_events() {
        let config = Config::default();
        let mut state = TuiState::new(
            vec!["module1".to_string(), "module2".to_string()],
            PathBuf::from("."),
            config,
        );

        // Initial state - first module selected
        assert_eq!(state.modules_list_state.selected(), Some(0));

        // Simulate key press event for Down arrow
        let press_event = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(press_event, &mut state);
        let after_press = state.modules_list_state.selected();

        // Selection should have moved to next module
        assert_eq!(after_press, Some(1));

        // Simulate key release event
        let release_event = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(release_event, &mut state);
        let after_release = state.modules_list_state.selected();

        // Selection should NOT change on release
        assert_eq!(after_release, Some(1));

        // Simulate repeat event
        let repeat_event = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Repeat,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(repeat_event, &mut state);
        let after_repeat = state.modules_list_state.selected();

        // Selection should NOT change on repeat (since we filter them out)
        assert_eq!(after_repeat, Some(1));
    }

    #[test]
    fn test_ctrl_r_shows_recent_projects_popup() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        assert!(
            !state.show_projects_popup,
            "Popup should be hidden initially"
        );

        // Simulate Ctrl+R key press
        let ctrl_r_event = KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(ctrl_r_event, &mut state);

        assert!(
            state.show_projects_popup,
            "Ctrl+R should show the projects popup"
        );
        assert_eq!(state.focus, Focus::Projects, "Focus should be on projects");
    }

    #[test]
    fn test_popup_navigation_up_down() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        state.recent_projects = vec![
            PathBuf::from("/tmp/project1"),
            PathBuf::from("/tmp/project2"),
            PathBuf::from("/tmp/project3"),
        ];
        state.projects_list_state.select(Some(0));
        state.show_projects_popup = true;

        // Simulate Down arrow in popup
        let down_event = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(down_event, &mut state);
        assert_eq!(state.projects_list_state.selected(), Some(1));

        // Simulate Up arrow in popup
        let up_event = KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(up_event, &mut state);
        assert_eq!(state.projects_list_state.selected(), Some(0));
    }

    #[test]
    fn test_popup_escape_closes_popup() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        state.show_projects_popup = true;

        // Simulate Esc key
        let esc_event = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(esc_event, &mut state);

        assert!(!state.show_projects_popup, "Esc should close the popup");
    }

    #[test]
    fn test_popup_enter_selects_project() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        state.recent_projects = vec![
            PathBuf::from("/tmp/project1"),
            PathBuf::from("/tmp/project2"),
        ];
        state.projects_list_state.select(Some(1));
        state.show_projects_popup = true;

        // Simulate Enter key
        let enter_event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(enter_event, &mut state);

        assert_eq!(
            state.switch_to_project,
            Some(PathBuf::from("/tmp/project2")),
            "Enter should select the project"
        );
        assert!(
            !state.show_projects_popup,
            "Popup should close after selection"
        );
    }

    #[test]
    fn test_popup_q_closes_without_quitting_app() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        state.show_projects_popup = true;

        // Simulate 'q' key in popup
        let q_event = KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(q_event, &mut state);

        assert!(!state.show_projects_popup, "'q' should close popup");
        // Note: In actual app, main loop checks !state.show_projects_popup before quitting
    }
}
