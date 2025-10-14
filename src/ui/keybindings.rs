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
    Flags,
}

const MODULE_ACTIONS: [ModuleAction; 7] = [
    ModuleAction {
        key: 'b',
        key_display: "b",
        prefix: "",
        suffix: "uild",
        command: &["clean", "install"],
    },
    ModuleAction {
        key: 'C',
        key_display: "C",
        prefix: "",
        suffix: "lean",
        command: &["clean"],
    },
    ModuleAction {
        key: 'c',
        key_display: "c",
        prefix: "",
        suffix: "ompile",
        command: &["compile"],
    },
    ModuleAction {
        key: 'k',
        key_display: "k",
        prefix: "pac",
        suffix: "age",
        command: &["package"],
    },
    ModuleAction {
        key: 't',
        key_display: "t",
        prefix: "",
        suffix: "est",
        command: &["test"],
    },
    ModuleAction {
        key: 'i',
        key_display: "i",
        prefix: "",
        suffix: "nstall",
        command: &["install"],
    },
    ModuleAction {
        key: 'd',
        key_display: "d",
        prefix: "",
        suffix: "eps",
        command: &["dependency:tree"],
    },
];

const OPTIONS_MENU_ITEMS: [OptionsItem; 2] = [
    OptionsItem {
        key: 'p',
        key_display: "p",
        prefix: "",
        suffix: "rofiles",
        action: OptionsAction::Profiles,
    },
    OptionsItem {
        key: 'f',
        key_display: "f",
        prefix: "",
        suffix: "lags",
        action: OptionsAction::Flags,
    },
];

pub const MODULE_ACTION_COUNT: usize = MODULE_ACTIONS.len();
pub const OPTIONS_ITEM_COUNT: usize = OPTIONS_MENU_ITEMS.len();

fn module_action(index: usize) -> &'static ModuleAction {
    &MODULE_ACTIONS[index % MODULE_ACTIONS.len()]
}

fn options_item(index: usize) -> &'static OptionsItem {
    &OPTIONS_MENU_ITEMS[index % OPTIONS_MENU_ITEMS.len()]
}

fn module_action_index_by_key(ch: char) -> Option<usize> {
    MODULE_ACTIONS.iter().position(|action| action.key == ch)
}

fn options_index_by_key(ch: char) -> Option<usize> {
    OPTIONS_MENU_ITEMS.iter().position(|item| item.key == ch)
}

#[derive(Clone, Copy)]
enum ButtonState {
    Normal,
    Active,
    Disabled,
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

    // Direct command execution - no menu navigation needed
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
        KeyCode::Char('p') => {
            state.switch_to_profiles();
        }
        KeyCode::Char('f') => {
            state.switch_to_flags();
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

fn execute_module_action(state: &mut crate::ui::state::TuiState, index: usize) {
    if MODULE_ACTIONS.is_empty() {
        return;
    }
    let action = module_action(index % MODULE_ACTION_COUNT);
    state.run_selected_module_command(action.command);
}

fn execute_options_action(state: &mut crate::ui::state::TuiState, index: usize) {
    if OPTIONS_MENU_ITEMS.is_empty() {
        return;
    }
    match options_item(index % OPTIONS_ITEM_COUNT).action {
        OptionsAction::Profiles => state.switch_to_profiles(),
        OptionsAction::Flags => state.switch_to_flags(),
    }
}

fn execute_menu_selection(state: &mut crate::ui::state::TuiState) {
    let menu = state.menu_state();
    match menu.section {
        MenuSection::Module => execute_module_action(state, menu.cycles_index),
        MenuSection::Options => execute_options_action(state, menu.options_index),
    }
}

fn handle_menu_char(state: &mut crate::ui::state::TuiState, ch: char) -> bool {
    if matches!(state.menu_state().section, MenuSection::Module) {
        if let Some(idx) = module_action_index_by_key(ch) {
            state.menu_activate(MenuSection::Module);
            state.menu_set_cycles_index(idx);
            execute_module_action(state, idx);
            return true;
        }
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
            execute_options_action(state, 0);
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
    state: ButtonState,
) {
    let (key_style, text_style) = match state {
        ButtonState::Active => (
            Theme::KEY_HINT_STYLE.add_modifier(Modifier::UNDERLINED),
            Theme::FOOTER_ACTIVE_TEXT_STYLE,
        ),
        ButtonState::Disabled => (
            Theme::FOOTER_DISABLED_KEY_STYLE,
            Theme::FOOTER_DISABLED_TEXT_STYLE,
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

pub(crate) fn module_box_body(menu_state: MenuState) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::raw("  "));
    for (idx, action) in MODULE_ACTIONS.iter().enumerate() {
        let module_focused = matches!(menu_state.section, MenuSection::Module);
        let is_active = menu_state.cycles_index % MODULE_ACTION_COUNT == idx;
        append_bracketed_word(
            &mut spans,
            action.prefix,
            action.key_display,
            action.suffix,
            if module_focused {
                if is_active {
                    ButtonState::Active
                } else {
                    ButtonState::Normal
                }
            } else {
                ButtonState::Disabled
            },
        );
        if idx < MODULE_ACTIONS.len() - 1 {
            spans.push(Span::raw("  •  "));
        }
    }
    Line::from(spans)
}

pub(crate) fn module_box_title(module_name: Option<&str>, focused: bool) -> Span<'static> {
    let title_text = module_name
        .map(|name| format!("[m]odule – {name}"))
        .unwrap_or_else(|| "[m]odule".to_string());
    if focused {
        Span::styled(title_text, Theme::FOOTER_SECTION_FOCUSED_STYLE)
    } else {
        Span::styled(title_text, Theme::FOOTER_SECTION_STYLE)
    }
}

pub(crate) fn options_box_body(view: CurrentView, menu_state: MenuState) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let options_active = matches!(menu_state.section, MenuSection::Options);
    spans.push(Span::raw("  "));

    let show_pointer = options_active || matches!(view, CurrentView::Profiles | CurrentView::Flags);
    if show_pointer {
        spans.push(Span::styled(">", Theme::FOOTER_POINTER_STYLE));
        spans.push(Span::raw("  "));
    }

    for (idx, item) in OPTIONS_MENU_ITEMS.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw(" "));
        }

        let is_active_view = match item.action {
            OptionsAction::Profiles => matches!(view, CurrentView::Profiles),
            OptionsAction::Flags => matches!(view, CurrentView::Flags),
        };

        let options_selected = is_active_view
            || (options_active && menu_state.options_index % OPTIONS_ITEM_COUNT == idx);

        append_bracketed_word(
            &mut spans,
            item.prefix,
            item.key_display,
            item.suffix,
            if options_selected {
                ButtonState::Active
            } else if options_active {
                ButtonState::Normal
            } else {
                ButtonState::Disabled
            },
        );
    }

    Line::from(spans)
}

pub(crate) fn options_box_title(
    view: CurrentView,
    active_profiles: &[String],
    enabled_flags: &[String],
    focused: bool,
) -> Span<'static> {
    let _ = view; // Not needed anymore since we always show all selections

    let mut parts = Vec::new();

    if !active_profiles.is_empty() {
        parts.push(active_profiles.join(", "));
    }

    if !enabled_flags.is_empty() {
        parts.push(enabled_flags.join(", "));
    }

    let text = if parts.is_empty() {
        "[o]ptions".to_string()
    } else {
        format!("[o]ptions – {}", parts.join(" | "))
    };

    if focused {
        Span::styled(text, Theme::FOOTER_SECTION_FOCUSED_STYLE)
    } else {
        Span::styled(text, Theme::FOOTER_SECTION_STYLE)
    }
}

pub(crate) fn simplified_footer_title(
    view: CurrentView,
    module_name: Option<&str>,
    active_profiles: &[String],
    enabled_flags: &[String],
) -> Span<'static> {
    let mut parts = Vec::new();

    if let Some(name) = module_name {
        parts.push(format!("Module: {}", name));
    }

    if !active_profiles.is_empty() {
        parts.push(format!("Profiles: {}", active_profiles.join(", ")));
    }

    if !enabled_flags.is_empty() {
        parts.push(format!("Flags: {}", enabled_flags.join(", ")));
    }

    let text = if parts.is_empty() {
        "Commands".to_string()
    } else {
        format!("Commands – {}", parts.join(" | "))
    };

    let style = match view {
        CurrentView::Modules => Theme::FOOTER_SECTION_STYLE,
        CurrentView::Profiles | CurrentView::Flags => Theme::FOOTER_SECTION_FOCUSED_STYLE,
    };

    Span::styled(text, style)
}

pub(crate) fn simplified_footer_body(view: CurrentView) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::raw("  "));

    // Module commands
    for (idx, action) in MODULE_ACTIONS.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw("  •  "));
        }
        append_bracketed_word(
            &mut spans,
            action.prefix,
            action.key_display,
            action.suffix,
            ButtonState::Normal,
        );
    }

    spans.push(Span::raw("  •  "));

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
