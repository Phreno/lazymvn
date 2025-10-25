//! Keybinding handling for the TUI
//!
//! Processes keyboard events and updates application state accordingly.

mod types;
mod popup_keys;
mod search_keys;
mod output_keys;
mod command_keys;
mod navigation_keys;

pub use types::{CurrentView, Focus, SearchMode};

use crate::ui::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::text::{Line, Span};

struct ModuleAction {
    key_display: &'static str,
    prefix: &'static str,
    suffix: &'static str,
}

const MODULE_ACTIONS: [ModuleAction; 9] = [
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
        key_display: "s",
        prefix: "",
        suffix: "tart",
    },
    ModuleAction {
        key_display: "d",
        prefix: "",
        suffix: "eps",
    },
    ModuleAction {
        key_display: "y",
        prefix: "",
        suffix: "ank output",
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

    // Handle save favorite popup separately
    if state.show_save_favorite_popup {
        if popup_keys::handle_save_favorite_popup(key, state) {
            return;
        }
    }

    // Handle favorites popup separately
    if state.show_favorites_popup {
        if popup_keys::handle_favorites_popup(key, state) {
            return;
        }
    }

    // Handle command history popup separately
    if state.show_history_popup {
        if popup_keys::handle_history_popup(key, state) {
            return;
        }
    }

    // Handle projects popup separately
    if state.show_projects_popup {
        if popup_keys::handle_projects_popup(key, state) {
            return;
        }
    }

    if let Some(search_mod) = state.search_mod.take() {
        log::debug!(
            "In search mode: {:?}",
            match search_mod {
                SearchMode::Input => "Input",
                SearchMode::Cycling => "Cycling",
            }
        );
        let handled = match search_mod {
            SearchMode::Input => search_keys::handle_search_input(key, state),
            SearchMode::Cycling => search_keys::handle_search_cycling(key, state),
        };
        
        if handled {
            return;
        }
        // If not handled (search cycling with unrecognized key), fall through to normal handling
    }

    // Handle starter selector popup
    if state.show_starter_selector {
        if popup_keys::handle_starter_selector(key, state) {
            return;
        }
    }

    // Handle starter manager popup
    if state.show_starter_manager {
        if popup_keys::handle_starter_manager(key, state) {
            return;
        }
    }

    // Direct command execution - no menu navigation needed
    
    // Try Maven command keys first
    if command_keys::handle_maven_command(key, state) {
        return;
    }
    
    // Try tab operations (Ctrl+T/W/Left/Right)
    if navigation_keys::handle_tab_operations(key, state) {
        return;
    }
    
    // Try popup triggers (Ctrl+F/S/H/R/E)
    if navigation_keys::handle_popup_triggers(key, state) {
        return;
    }
    
    // Try view switching (0-4)
    if navigation_keys::handle_view_switching(key, state) {
        return;
    }
    
    // Try focus cycling (Left/Right arrows)
    if navigation_keys::handle_focus_cycling(key, state) {
        return;
    }
    
    // Try search operations (/, n, N)
    if navigation_keys::handle_search_operations(key, state) {
        return;
    }
    
    // Try special actions (Esc, Enter, Space)
    if navigation_keys::handle_special_actions(key, state) {
        return;
    }
    
    // Handle remaining output operations and scrolling
    match key.code {
        KeyCode::Down => output_keys::handle_scroll_down(state, state.focus),
        KeyCode::Up => output_keys::handle_scroll_up(state, state.focus),
        KeyCode::PageUp => output_keys::handle_page_up(state),
        KeyCode::PageDown => output_keys::handle_page_down(state),
        KeyCode::Home => output_keys::handle_scroll_to_start(state),
        KeyCode::End => output_keys::handle_scroll_to_end(state),
        KeyCode::Char('y') => output_keys::handle_yank_output(state),
        KeyCode::Char('Y') => output_keys::handle_yank_debug_info(state),
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

pub(crate) fn build_navigation_line() -> Vec<Line<'static>> {
    // Split into multiples logical groups on separate lines for better readability
    vec![
        // Line 1: Views
        Line::from(vec![
            Span::styled("Views: ", Theme::FOOTER_SECTION_STYLE),
            key_token("0"),
            Span::raw(" Output  "),
            key_token("1"),
            Span::raw(" Projects  "),
            key_token("2"),
            Span::raw(" Modules  "),
            key_token("3"),
            Span::raw(" Profiles  "),
            key_token("4"),
            Span::raw(" Flags"),
        ]),
        // Line 2: Focus & Navigate
        Line::from(vec![
            Span::styled("Focus: ", Theme::FOOTER_SECTION_STYLE),
            key_token("←→"),
            Span::raw("   "),
            Span::styled("Navigate: ", Theme::FOOTER_SECTION_STYLE),
            key_token("↑↓"),
            Span::raw("   "),
        ]),
        // Line 3: Tabulations
        Line::from(vec![
            Span::styled("Tabs: ", Theme::FOOTER_SECTION_STYLE),
            key_token("Ctrl+T"),
            Span::raw(" New "),
            key_token("Ctrl+W"),
            Span::raw(" Close "),
            key_token("Ctrl+←→"),
            Span::raw(" Switch "),
            key_token("Esc"),
            Span::raw(" Kill"),
        ]),
        // Actions
        Line::from(vec![
            Span::styled("Actions: ", Theme::FOOTER_SECTION_STYLE),
            key_token("Ctrl+F"),
            Span::raw(" Favs  "),
            key_token("Ctrl+H"),
            Span::raw(" History  "),
            key_token("Ctrl+R"),
            Span::raw(" Recent projects "),
            key_token("Ctrl+E"),
            Span::raw(" Edit"),
        ]),
    ]
}

pub(crate) fn simplified_footer_title(
    view: CurrentView,
    _module_name: Option<&str>,
    _active_profiles: &[String],
    _enabled_flags: &[String],
) -> Span<'static> {
    let text = "Commands".to_string();

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
    use crate::core::config::Config;
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
        assert_eq!(
            state.get_active_tab().modules_list_state.selected(),
            Some(0)
        );

        // Simulate key press event for Down arrow
        let press_event = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(press_event, &mut state);
        let after_press = state.get_active_tab().modules_list_state.selected();

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
        let after_release = state.get_active_tab().modules_list_state.selected();

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
        let after_repeat = state.get_active_tab().modules_list_state.selected();

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

        // Currently the project switching is disabled during tabs migration
        // Just verify that popup closes
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

    #[test]
    fn test_s_key_shows_starter_selector_when_no_cached() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        // Ensure no cached starters
        state.get_active_tab_mut().starters_cache.starters.clear();

        // Simulate 's' key
        let s_event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(s_event, &mut state);

        assert!(
            state.show_starter_selector,
            "'s' should show starter selector when no cached starters"
        );
    }

    #[test]
    fn test_starter_selector_navigation() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        state.starter_candidates = vec![
            "com.example.App1".to_string(),
            "com.example.App2".to_string(),
        ];
        state.show_starter_selector = true;
        state.starters_list_state.select(Some(0));

        // Test Down arrow
        let down_event = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(down_event, &mut state);
        assert_eq!(state.starters_list_state.selected(), Some(1));

        // Test Up arrow
        let up_event = KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(up_event, &mut state);
        assert_eq!(state.starters_list_state.selected(), Some(0));
    }

    #[test]
    fn test_starter_selector_filter() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        state.starter_candidates = vec![
            "com.example.Application".to_string(),
            "com.example.Main".to_string(),
        ];
        state.show_starter_selector = true;

        // Type 'A' to filter
        let char_event = KeyEvent {
            code: KeyCode::Char('A'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(char_event, &mut state);
        assert_eq!(state.starter_filter, "A");

        // Backspace to clear
        let backspace_event = KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(backspace_event, &mut state);
        assert_eq!(state.starter_filter, "");
    }

    #[test]
    fn test_starter_selector_esc_closes() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        state.show_starter_selector = true;

        let esc_event = KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(esc_event, &mut state);
        assert!(!state.show_starter_selector, "Esc should close selector");
    }

    #[test]
    fn test_ctrl_shift_s_opens_starter_manager() {
        let config = Config::default();
        let mut state = TuiState::new(vec!["module1".to_string()], PathBuf::from("."), config);

        let ctrl_shift_s_event = KeyEvent {
            code: KeyCode::Char('S'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(ctrl_shift_s_event, &mut state);
        assert!(
            state.show_starter_manager,
            "Ctrl+Shift+S should open starter manager"
        );
    }

    #[test]
    fn test_starter_manager_space_toggles_default() {
        let config = Config::default();
        let temp_dir = tempfile::tempdir().unwrap();
        let mut state = TuiState::new(
            vec!["module1".to_string()],
            temp_dir.path().to_path_buf(),
            config,
        );

        // Clear any loaded starters and add fresh ones
        let tab = state.get_active_tab_mut();
        tab.starters_cache.starters.clear();
        tab.starters_cache
            .add_starter(crate::features::starters::Starter::new(
                "com.example.App1".to_string(),
                "App1".to_string(),
                false,
            ));
        tab.starters_cache
            .add_starter(crate::features::starters::Starter::new(
                "com.example.App2".to_string(),
                "App2".to_string(),
                false,
            ));

        state.show_starter_manager = true;
        state.starters_list_state.select(Some(1));

        // Press space to toggle default
        let space_event = KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(space_event, &mut state);
        let tab = state.get_active_tab();
        assert!(tab.starters_cache.starters[1].is_default);
        assert!(!tab.starters_cache.starters[0].is_default);
    }

    #[test]
    fn test_starter_manager_delete() {
        let config = Config::default();
        let temp_dir = tempfile::tempdir().unwrap();
        let mut state = TuiState::new(
            vec!["module1".to_string()],
            temp_dir.path().to_path_buf(),
            config,
        );

        // Clear any loaded starters and add fresh one
        let tab = state.get_active_tab_mut();
        tab.starters_cache.starters.clear();
        tab.starters_cache
            .add_starter(crate::features::starters::Starter::new(
                "com.example.App1".to_string(),
                "App1".to_string(),
                false,
            ));

        state.show_starter_manager = true;
        state.starters_list_state.select(Some(0));

        // Press 'd' to delete
        let d_event = KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(d_event, &mut state);
        let tab = state.get_active_tab();
        assert_eq!(tab.starters_cache.starters.len(), 0);
    }

    #[test]
    fn test_yank_output() {
        let config = Config::default();
        let temp_dir = tempfile::tempdir().unwrap();
        let mut state = TuiState::new(
            vec!["module1".to_string()],
            temp_dir.path().to_path_buf(),
            config,
        );

        // Add some output
        state.get_active_tab_mut().command_output = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
        ];

        // Press 'y' to yank output
        let y_event = KeyEvent {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        handle_key_event(y_event, &mut state);

        // Should have added a message about copying
        // Note: actual clipboard test may fail in CI/headless environments
        // so we just check that the function was called and output updated
        assert!(state.get_active_tab().command_output.len() > 3);
    }
}
