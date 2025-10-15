mod config;
mod logger;
mod maven;
mod project;
mod tui;
mod ui;
mod utils;

use clap::Parser;
use crossterm::event;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

#[derive(Parser)]
#[command(name = "lazymvn")]
#[command(about = "A terminal UI for Maven projects")]
struct Cli {
    /// Enable debug logging to lazymvn-debug.log
    #[arg(short, long)]
    debug: bool,

    /// Path to the Maven project (defaults to current directory)
    #[arg(short, long)]
    project: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logger based on debug flag
    if let Err(e) = logger::init(cli.debug) {
        eprintln!("Failed to initialize logger: {}", e);
    }

    log::info!("Starting lazymvn");

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
        log::error!("Application error: {:?}", err);
        println!("{:?}", err);
        return Err(err);
    }

    log::info!("Exiting lazymvn");
    Ok(())
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (modules, project_root) = project::get_project_modules()?;
    log::debug!("Loaded {} modules from {:?}", modules.len(), project_root);
    
    let config = config::load_config(&project_root);
    let mut state = tui::TuiState::new(modules, project_root.clone(), config);

    // Load available profiles
    if let Ok(profiles) = maven::get_profiles(&project_root) {
        log::debug!("Found {} Maven profiles", profiles.len());
        state.set_profiles(profiles);
    }

    loop {
        tui::draw(terminal, &mut state)?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == event::KeyCode::Char('q') {
                    log::info!("User requested quit");
                    break;
                }
                tui::handle_key_event(key, &mut state);
            }
        }
    }
    Ok(())
}
