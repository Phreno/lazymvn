mod config;
mod maven;
mod project;
mod tui;
mod ui;
mod utils;

use crossterm::event;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup terminal
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run(&mut terminal);

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
        return Err(err);
    }

    Ok(())
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (modules, project_root) = project::get_project_modules()?;
    let config = config::load_config(&project_root);
    let mut state = tui::TuiState::new(modules, project_root, config);

    loop {
        tui::draw(terminal, &mut state)?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == event::KeyCode::Char('q') {
                    break;
                }
                tui::handle_key_event(key, &mut state);
            }
        }
    }
    Ok(())
}
