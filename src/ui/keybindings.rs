use crate::ui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    text::Span,
};

/// Represents the current view in the TUI
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CurrentView {
    Modules,
    Profiles,
}

/// Represents which pane currently has focus
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Focus {
    Modules,
    Output,
}

/// Search mode for input vs cycling through matches
pub enum SearchMode {
    Input,
    Cycling,
}

/// Handle key events and update TUI state accordingly
pub fn handle_key_event(key: KeyEvent, state: &mut crate::ui::state::TuiState) {
    if let Some(search_mod) = state.search_mod.take() {
        match search_mod {
            SearchMode::Input => {
                match key.code {
                    KeyCode::Char(ch) => {
                        state.push_search_char(ch);
                        state.live_search(); // Trigger live search as user types
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Backspace => {
                        state.backspace_search_char();
                        state.live_search(); // Update search as user deletes
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Up => {
                        state.recall_previous_search();
                        state.live_search(); // Update search when recalling history
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Down => {
                        state.recall_next_search();
                        state.live_search(); // Update search when recalling history
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Enter => {
                        state.submit_search();
                        // If search has results, enter cycling mode; otherwise exit
                        if state.has_search_results() {
                            state.search_mod = Some(SearchMode::Cycling);
                        } else {
                            state.search_mod = None;
                        }
                    }
                    KeyCode::Esc => {
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
                        // Exit search mode and restore normal coloration
                        state.search_mod = None;
                    }
                    _ => {
                        // Exit search mode for other keys and handle them normally
                        state.search_mod = None;
                        // Re-process the key if we exited search mode
                        handle_key_event(key, state);
                    }
                }
                return;
            }
        }
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
                state.current_view = CurrentView::Profiles;
                state.focus_modules();
            }
            CurrentView::Profiles => {
                state.current_view = CurrentView::Modules;
                state.focus_modules();
            }
        },
        KeyCode::Char('b') => {
            state.run_selected_module_command(&["package"]);
        }
        KeyCode::Char('t') => {
            state.run_selected_module_command(&["test"]);
        }
        KeyCode::Char('c') => {
            state.run_selected_module_command(&["clean"]);
        }
        KeyCode::Char('i') => {
            state.run_selected_module_command(&["install"]);
        }
        KeyCode::Char('d') => {
            state.run_selected_module_command(&["dependency:tree"]);
        }
        KeyCode::Char('/') => {
            state.begin_search_input();
            state.search_mod = Some(SearchMode::Input);
        }
        KeyCode::Char('n') => {
            state.next_search_match();
        }
        KeyCode::Char('N') => {
            state.previous_search_match();
        }
        KeyCode::PageUp => {
            state.scroll_output_pages(-1);
        }
        KeyCode::PageDown => {
            state.scroll_output_pages(1);
        }
        KeyCode::Home => {
            state.scroll_output_to_start();
        }
        KeyCode::End => {
            state.scroll_output_to_end();
        }
        KeyCode::Enter => {
            state.toggle_profile();
        }
        _ => {}
    }
}

/// Generate footer spans with key hints based on current view and focus
pub fn footer_spans(view: CurrentView, focus: Focus) -> Vec<Span<'static>> {
    let mut hints: Vec<(&str, &str)> = vec![("←/→", "Focus")];

    match focus {
        Focus::Modules => {
            match view {
                CurrentView::Modules => {
                    hints.extend_from_slice(&[
                        ("↑/↓", "Select"),
                        ("p", "Profiles"),
                        ("Enter", "Build"),
                    ]);
                }
                CurrentView::Profiles => {
                    hints.extend_from_slice(&[
                        ("↑/↓", "Select"),
                        ("Enter", "Toggle profile"),
                        ("p", "Back to modules"),
                    ]);
                }
            }
        }
        Focus::Output => {
            hints.extend_from_slice(&[
                ("↑/↓", "Scroll"),
                ("PgUp/PgDn", "Page up/down"),
                ("/", "Search"),
                ("n/N", "Next/Prev match"),
            ]);
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
            Theme::KEY_HINT_STYLE,
        ));
        spans.push(Span::raw(format!(" {label} ")));
        if idx < hints.len() - 1 {
            spans.push(Span::raw(" • "));
        }
    }

    spans
}
