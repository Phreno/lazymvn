use crate::ui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Modifier,
    text::{Line, Span},
};

/// Represents the current view in the TUI
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CurrentView {
    Modules,
    Profiles,
    Flags,
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

struct ModuleAction {
    key_display: &'static str,
    prefix: &'static str,
    suffix: &'static str,
}

struct OptionsItem {
    key_display: &'static str,
    prefix: &'static str,
    suffix: &'static str,
    action: OptionsAction,
}

#[derive(Clone, Copy)]
enum OptionsAction {
    Profiles,
    Flags,
}

const MODULE_ACTIONS: [ModuleAction; 7] = [
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
];

const OPTIONS_MENU_ITEMS: [OptionsItem; 2] = [
    OptionsItem {
        key_display: "p",
        prefix: "",
        suffix: "rofiles",
        action: OptionsAction::Profiles,
    },
    OptionsItem {
        key_display: "f",
        prefix: "",
        suffix: "lags",
        action: OptionsAction::Flags,
    },
];

#[derive(Clone, Copy)]
enum ButtonState {
    Normal,
    Active,
}

/// Handle key events and update TUI state accordingly
pub fn handle_key_event(key: KeyEvent, state: &mut crate::ui::state::TuiState) {
    log::debug!("Key event: {:?}", key);
    
    if let Some(search_mod) = state.search_mod.take() {
        log::debug!("In search mode: {:?}", match search_mod {
            SearchMode::Input => "Input",
            SearchMode::Cycling => "Cycling",
        });
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
        KeyCode::Left => {
            log::debug!("Focus left -> modules");
            state.focus_modules();
        }
        KeyCode::Right => {
            log::debug!("Focus right -> output");
            state.focus_output();
        }
        KeyCode::Down => match state.focus {
            Focus::Modules => {
                log::debug!("Navigate down in modules list");
                state.next_item();
            }
            Focus::Output => {
                log::debug!("Scroll output down");
                state.scroll_output_lines(1);
            }
        },
        KeyCode::Up => match state.focus {
            Focus::Modules => {
                log::debug!("Navigate up in modules list");
                state.previous_item();
            }
            Focus::Output => {
                log::debug!("Scroll output up");
                state.scroll_output_lines(-1);
            }
        },
        KeyCode::Char('m') => {
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
        KeyCode::Char('p') => {
            log::info!("Switch to profiles view");
            state.switch_to_profiles();
        }
        KeyCode::Char('f') => {
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
            if state.current_view == CurrentView::Profiles {
                state.toggle_profile();
            } else if state.current_view == CurrentView::Flags {
                state.toggle_flag();
            }
        }
        _ => {}
    }
}

fn key_token(text: &str) -> Span<'static> {
    Span::styled(text.to_string(), Theme::KEY_HINT_STYLE)
}

fn append_bracketed_word(
    spans: &mut Vec<Span<'static>>,
    prefix: &str,
    key: &str,
    suffix: &str,
    state: ButtonState,
) {
    let (key_style, text_style) = match state {
        ButtonState::Active => (
            Theme::KEY_HINT_STYLE.add_modifier(Modifier::UNDERLINED),
            Theme::FOOTER_ACTIVE_TEXT_STYLE,
        ),
        ButtonState::Normal => (Theme::KEY_HINT_STYLE, Theme::DEFAULT_STYLE),
    };

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
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled("Navigation ", Theme::FOOTER_SECTION_STYLE));
    spans.push(key_token("←"));
    spans.push(Span::raw("  "));
    spans.push(key_token("→"));
    spans.push(Span::raw(" Focus  • "));
    spans.push(key_token("↑"));
    spans.push(Span::raw("  "));
    spans.push(key_token("↓"));
    spans.push(Span::raw(" Select"));
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
        parts.push(name.to_string());
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
        CurrentView::Modules => Theme::FOOTER_SECTION_STYLE,
        CurrentView::Profiles | CurrentView::Flags => Theme::FOOTER_SECTION_FOCUSED_STYLE,
    };

    Span::styled(text, style)
}

pub(crate) fn simplified_footer_body(view: CurrentView) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::raw(" "));

    // Module commands
    for (idx, action) in MODULE_ACTIONS.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw(" "));
        }
        append_bracketed_word(
            &mut spans,
            action.prefix,
            action.key_display,
            action.suffix,
            ButtonState::Normal,
        );
    }

    spans.push(Span::raw(" "));

    // Options commands
    for (idx, item) in OPTIONS_MENU_ITEMS.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw(" "));
        }

        let state = match item.action {
            OptionsAction::Profiles if matches!(view, CurrentView::Profiles) => ButtonState::Active,
            OptionsAction::Flags if matches!(view, CurrentView::Flags) => ButtonState::Active,
            _ => ButtonState::Normal,
        };

        append_bracketed_word(
            &mut spans,
            item.prefix,
            item.key_display,
            item.suffix,
            state,
        );
    }

    Line::from(spans)
}
