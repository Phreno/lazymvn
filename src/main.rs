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

    // Change to project directory if specified
    if let Some(ref project_path) = cli.project {
        log::info!("Changing to project directory: {}", project_path);
        std::env::set_current_dir(project_path).map_err(|e| {
            format!(
                "Failed to change to project directory '{}': {}",
                project_path, e
            )
        })?;
    }

    // Check Maven availability early
    let current_dir = std::env::current_dir()?;
    match maven::check_maven_availability(&current_dir) {
        Ok(version) => {
            log::info!("Maven available: {}", version);
        }
        Err(error) => {
            log::error!("Maven check failed: {}", error);
            eprintln!("❌ {}", error);
            eprintln!("\nPlease ensure Maven is installed and accessible from your PATH.");
            eprintln!("You can verify this by running: mvn --version");
            return Err(error.into());
        }
    }

    // setup terminal
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run(&mut terminal);

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
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

    // Add current project to recent projects
    let mut recent_projects = config::RecentProjects::load();
    recent_projects.add(project_root.clone());

    let config = config::load_config(&project_root);
    let mut state = tui::TuiState::new(modules, project_root.clone(), config);

    // Load available profiles
    if let Ok(profiles) = maven::get_profiles(&project_root) {
        log::debug!("Found {} Maven profiles", profiles.len());
        state.set_profiles(profiles);
    }

    loop {
        // Poll for command updates first
        state.poll_command_updates();

        tui::draw(terminal, &mut state)?;

        // Check if we need to switch projects
        if let Some(new_project) = state.switch_to_project.take() {
            log::info!("Switching to project: {:?}", new_project);

            // Change directory
            if let Err(e) = std::env::set_current_dir(&new_project) {
                log::error!("Failed to switch to project: {}", e);
                state.command_output = vec![format!("Error: Failed to switch to project: {}", e)];
                continue;
            }

            // Reload project
            match project::get_project_modules() {
                Ok((new_modules, new_project_root)) => {
                    log::info!(
                        "Loaded {} modules from {:?}",
                        new_modules.len(),
                        new_project_root
                    );

                    // Add to recent projects
                    recent_projects.add(new_project_root.clone());

                    // Load config
                    let new_config = config::load_config(&new_project_root);

                    // Create new state
                    state = tui::TuiState::new(new_modules, new_project_root.clone(), new_config);

                    // Load profiles
                    if let Ok(profiles) = maven::get_profiles(&new_project_root) {
                        log::debug!("Found {} Maven profiles", profiles.len());
                        state.set_profiles(profiles);
                    }
                }
                Err(e) => {
                    log::error!("Failed to load new project: {}", e);
                    state.command_output = vec![format!("Error: Failed to load project: {}", e)];
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                event::Event::Key(key) => {
                    if key.code == event::KeyCode::Char('q') && !state.show_projects_popup {
                        log::info!("User requested quit");
                        break;
                    }
                    tui::handle_key_event(key, &mut state);
                }
                event::Event::Mouse(mouse) => {
                    tui::handle_mouse_event(mouse, &mut state);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
