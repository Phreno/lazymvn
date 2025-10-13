use crate::ui::state::{MenuSection, MenuState};
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

struct CycleAction {
    key: char,
    key_display: &'static str,
    prefix: &'static str,
    suffix: &'static str,
    command: &'static [&'static str],
}

struct OptionsItem {
    key: char,
    key_display: &'static str,
    prefix: &'static str,
    suffix: &'static str,
    action: OptionsAction,
}

#[derive(Clone, Copy)]
enum OptionsAction {
    Profiles,
}

const CYCLE_ACTIONS: [CycleAction; 7] = [
    CycleAction {
        key: 'b',
        key_display: "b",
        prefix: "",
        suffix: "uild",
        command: &["clean", "install"],
    },
    CycleAction {
        key: 'C',
        key_display: "C",
        prefix: "",
        suffix: "lean",
        command: &["clean"],
    },
    CycleAction {
        key: 'c',
        key_display: "c",
        prefix: "",
        suffix: "ompile",
        command: &["compile"],
    },
    CycleAction {
        key: 'k',
        key_display: "k",
        prefix: "pac",
        suffix: "age",
        command: &["package"],
    },
    CycleAction {
        key: 't',
        key_display: "t",
        prefix: "",
        suffix: "est",
        command: &["test"],
    },
    CycleAction {
        key: 'i',
        key_display: "i",
        prefix: "",
        suffix: "nstall",
        command: &["install"],
    },
    CycleAction {
        key: 'd',
        key_display: "d",
        prefix: "",
        suffix: "eps",
        command: &["dependency:tree"],
    },
];

const OPTIONS_MENU_ITEMS: [OptionsItem; 1] = [OptionsItem {
    key: 'p',
    key_display: "p",
    prefix: "",
    suffix: "rofiles",
    action: OptionsAction::Profiles,
}];

pub const CYCLE_ACTION_COUNT: usize = CYCLE_ACTIONS.len();
pub const OPTIONS_ITEM_COUNT: usize = OPTIONS_MENU_ITEMS.len();

fn cycle_action(index: usize) -> &'static CycleAction {
    &CYCLE_ACTIONS[index % CYCLE_ACTIONS.len()]
}

fn options_item(index: usize) -> &'static OptionsItem {
    &OPTIONS_MENU_ITEMS[index % OPTIONS_MENU_ITEMS.len()]
}

fn cycle_action_index_by_key(ch: char) -> Option<usize> {
    CYCLE_ACTIONS.iter().position(|action| action.key == ch)
}

fn options_index_by_key(ch: char) -> Option<usize> {
    OPTIONS_MENU_ITEMS.iter().position(|item| item.key == ch)
}

/// Handle key events and update TUI state accordingly
pub fn handle_key_event(key: KeyEvent, state: &mut crate::ui::state::TuiState) {
    if let Some(search_mod) = state.search_mod.take() {
        match search_mod {
            SearchMode::Input => {
                match key.code {
                    KeyCode::Char(ch) => {
                        state.push_search_char(ch);
                        state.live_search();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Backspace => {
                        state.backspace_search_char();
                        state.live_search();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Up => {
                        state.recall_previous_search();
                        state.live_search();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Down => {
                        state.recall_next_search();
                        state.live_search();
                        state.search_mod = Some(SearchMode::Input);
                    }
                    KeyCode::Enter => {
                        state.submit_search();
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

    if state.menu_state().active {
        match key.code {
            KeyCode::Left => {
                state.menu_prev_section();
                return;
            }
            KeyCode::Right => {
                state.menu_next_section();
                return;
            }
            KeyCode::Up => {
                state.menu_prev_item();
                return;
            }
            KeyCode::Down => {
                state.menu_next_item();
                return;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                execute_menu_selection(state);
                return;
            }
            KeyCode::Esc => {
                state.menu_deactivate();
                return;
            }
            KeyCode::Char(ch) => {
                if handle_menu_char(state, ch) {
                    return;
                }
            }
            _ => {}
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
        KeyCode::Char('o') => {
            state.menu_activate(MenuSection::Options);
            state.menu_set_options_index(0);
        }
        KeyCode::Char('m') => {
            state.switch_to_modules();
        }
        KeyCode::Char('b') => {
            state.run_selected_module_command(&["clean", "install"]);
        }
        KeyCode::Char('C') => {
            state.run_selected_module_command(&["clean"]);
        }
        KeyCode::Char('c') => {
            state.run_selected_module_command(&["compile"]);
        }
        KeyCode::Char('k') => {
            state.run_selected_module_command(&["package"]);
        }
        KeyCode::Char('t') => {
            state.run_selected_module_command(&["test"]);
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
            if state.current_view == CurrentView::Profiles {
                state.toggle_profile();
            }
        }
        KeyCode::Esc => {
            state.menu_deactivate();
        }
        _ => {}
    }
}

fn execute_cycle_action(state: &mut crate::ui::state::TuiState, index: usize) {
    if CYCLE_ACTIONS.is_empty() {
        return;
    }
    let action = cycle_action(index % CYCLE_ACTION_COUNT);
    state.run_selected_module_command(action.command);
}

fn execute_options_action(state: &mut crate::ui::state::TuiState, index: usize) {
    if OPTIONS_MENU_ITEMS.is_empty() {
        return;
    }
    match options_item(index % OPTIONS_ITEM_COUNT).action {
        OptionsAction::Profiles => state.switch_to_profiles(),
    }
}

fn execute_menu_selection(state: &mut crate::ui::state::TuiState) {
    let menu = state.menu_state();
    match menu.section {
        MenuSection::Cycles => execute_cycle_action(state, menu.cycles_index),
        MenuSection::Options => execute_options_action(state, menu.options_index),
        MenuSection::Modules => state.switch_to_modules(),
    }
}

fn handle_menu_char(state: &mut crate::ui::state::TuiState, ch: char) -> bool {
    if let Some(idx) = cycle_action_index_by_key(ch) {
        state.menu_activate(MenuSection::Cycles);
        state.menu_set_cycles_index(idx);
        execute_cycle_action(state, idx);
        return true;
    }
    if let Some(idx) = options_index_by_key(ch) {
        state.menu_activate(MenuSection::Options);
        state.menu_set_options_index(idx);
        execute_options_action(state, idx);
        return true;
    }
    match ch {
        'o' => {
            state.menu_activate(MenuSection::Options);
            state.menu_set_options_index(0);
            true
        }
        'm' => {
            state.switch_to_modules();
            true
        }
        _ => false,
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
    active: bool,
) {
    let text_style = if active {
        Theme::FOOTER_ACTIVE_TEXT_STYLE
    } else {
        Theme::DEFAULT_STYLE
    };

    if !prefix.is_empty() {
        spans.push(Span::styled(prefix.to_string(), text_style));
    }

    let mut key_style = Theme::KEY_HINT_STYLE;
    if active {
        key_style = key_style.add_modifier(Modifier::UNDERLINED);
    }
    spans.push(Span::styled("[", text_style));
    spans.push(Span::styled(key.to_string(), key_style));
    spans.push(Span::styled("]", text_style));

    if !suffix.is_empty() {
        spans.push(Span::styled(suffix.to_string(), text_style));
    }
}

fn blank_line() -> Line<'static> {
    Line::raw("")
}

fn build_navigation_line() -> Line<'static> {
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

fn build_cycles_line(menu_state: MenuState) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled("Cycles   ", Theme::FOOTER_SECTION_STYLE));
    for (idx, action) in CYCLE_ACTIONS.iter().enumerate() {
        let is_active = menu_state.active
            && matches!(menu_state.section, MenuSection::Cycles)
            && menu_state.cycles_index % CYCLE_ACTION_COUNT == idx;
        append_bracketed_word(
            &mut spans,
            action.prefix,
            action.key_display,
            action.suffix,
            is_active,
        );
        if idx < CYCLE_ACTIONS.len() - 1 {
            spans.push(Span::raw("  •  "));
        }
    }
    Line::from(spans)
}

fn build_options_line(view: CurrentView, menu_state: MenuState) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled("Options  ", Theme::FOOTER_SECTION_STYLE));
    let options_active = menu_state.active && matches!(menu_state.section, MenuSection::Options);
    append_bracketed_word(&mut spans, "", "o", "ptions", options_active);

    spans.push(Span::raw("  "));

    let show_pointer = options_active || matches!(view, CurrentView::Profiles);
    if show_pointer {
        spans.push(Span::styled(">", Theme::FOOTER_POINTER_STYLE));
        spans.push(Span::raw("  "));
    }

    if !OPTIONS_MENU_ITEMS.is_empty() {
        let item = options_item(0);
        let options_selected = (options_active
            && menu_state.options_index % OPTIONS_ITEM_COUNT == 0)
            || matches!(view, CurrentView::Profiles);
        append_bracketed_word(
            &mut spans,
            item.prefix,
            item.key_display,
            item.suffix,
            options_selected,
        );
    }

    Line::from(spans)
}

fn build_modules_line(view: CurrentView, menu_state: MenuState) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled("Modules  ", Theme::FOOTER_SECTION_STYLE));
    if menu_state.active && matches!(menu_state.section, MenuSection::Modules) {
        spans.push(Span::styled(">", Theme::FOOTER_POINTER_STYLE));
        spans.push(Span::raw("  "));
    }
    let modules_selected = matches!(view, CurrentView::Modules)
        && (!menu_state.active || matches!(menu_state.section, MenuSection::Modules));
    append_bracketed_word(&mut spans, "", "m", "odules", modules_selected);
    Line::from(spans)
}

/// Generate navigation footer lines with key hints based on current view and focus
pub fn footer_lines(view: CurrentView, _focus: Focus, menu_state: MenuState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(build_navigation_line());
    lines.push(blank_line());
    lines.push(build_cycles_line(menu_state));
    lines.push(blank_line());
    lines.push(build_options_line(view, menu_state));
    lines.push(blank_line());
    lines.push(build_modules_line(view, menu_state));
    lines
}
