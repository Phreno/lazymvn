use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, state: &mut TuiState) -> Result<(), std::io::Error> {
    terminal.draw(|f| {
        let area = f.area();
        let block = Block::default().title("Modules").borders(Borders::ALL);
        let list_items: Vec<ListItem> = state.modules.iter().map(|m| ListItem::new(m.as_str())).collect();
        let list = List::new(list_items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut state.list_state);
    })?;
    Ok(())
}

pub struct TuiState {
    pub modules: Vec<String>,
    pub list_state: ListState,
}

impl TuiState {
    pub fn new(modules: Vec<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            modules,
            list_state,
        }
    }

    pub fn next_module(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.modules.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous_module(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.modules.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

pub fn handle_key_event(key: KeyEvent, state: &mut TuiState) {
    match key.code {
        KeyCode::Down => state.next_module(),
        KeyCode::Up => state.previous_module(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Modifier, Style},
        Terminal,
    };

    #[test]
    fn test_draw_ui() {
        let backend = TestBackend::new(13, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let modules = vec!["module1".to_string(), "module2".to_string()];
        let mut state = TuiState::new(modules);
        draw(&mut terminal, &mut state).unwrap();

        let mut expected = Buffer::with_lines(vec![
            "┌Modules────┐",
            "│> module1  │",
            "│  module2  │",
            "│           │",
            "└───────────┘",
        ]);
        expected.set_style(
            Rect::new(1, 1, 11, 1),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow),
        );

        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_key_events() {
        let modules = vec!["module1".to_string(), "module2".to_string(), "module3".to_string()];
        let mut state = TuiState::new(modules);

        // Test initial state
        assert_eq!(state.list_state.selected(), Some(0));

        // Test moving down
        handle_key_event(KeyEvent::from(KeyCode::Down), &mut state);
        assert_eq!(state.list_state.selected(), Some(1));

        // Test moving down again
        handle_key_event(KeyEvent::from(KeyCode::Down), &mut state);
        assert_eq!(state.list_state.selected(), Some(2));

        // Test wrapping around
        handle_key_event(KeyEvent::from(KeyCode::Down), &mut state);
        assert_eq!(state.list_state.selected(), Some(0));

        // Test moving up
        handle_key_event(KeyEvent::from(KeyCode::Up), &mut state);
        assert_eq!(state.list_state.selected(), Some(2));

        // Test moving up again
        handle_key_event(KeyEvent::from(KeyCode::Up), &mut state);
        assert_eq!(state.list_state.selected(), Some(1));
    }
}
