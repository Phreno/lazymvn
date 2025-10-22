mod config;
mod history;
mod loading;
mod logger;
mod maven;
mod project;
mod starters;
mod tui;
mod ui;
mod utils;
mod watcher;

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

    /// Force spring-boot:run for launching applications (overrides auto-detection)
    #[arg(long)]
    force_run: bool,

    /// Force exec:java for launching applications (overrides auto-detection)
    #[arg(long)]
    force_exec: bool,

    /// Generate a lazymvn.toml configuration file in the current directory
    #[arg(long)]
    setup: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Handle --setup flag
    if cli.setup {
        return setup_config();
    }

    // Initialize logger based on debug flag
    if let Err(e) = logger::init(cli.debug) {
        eprintln!("Failed to initialize logger: {}", e);
    }

    // Show log location if debug is enabled
    if cli.debug {
        if let Some(debug_log) = logger::get_debug_log_path() {
            eprintln!("📝 Debug logs: {}", debug_log.display());
        }
        if let Some(error_log) = logger::get_error_log_path() {
            eprintln!("❌ Error logs: {}", error_log.display());
        }
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
    let res = run(&mut terminal, &cli);

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
    cli: &Cli,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize loading progress with 5 steps
    let mut progress = loading::LoadingProgress::new(5);
    
    // Step 1: Loading project structure
    loading_step!(progress, terminal, 1, 5, "Searching for Maven project...");
    
    // Try to load project modules from current directory
    let (modules, project_root) = match project::get_project_modules() {
        Ok(result) => {
            log::debug!("Loaded {} modules from {:?}", result.0.len(), result.1);
            result
        }
        Err(e) => {
            log::warn!("No POM found in current directory: {}", e);

            // Try to load most recent project as fallback
            let recent_projects = config::RecentProjects::load();
            let valid_projects = recent_projects.get_projects();

            if let Some(last_project) = valid_projects.first() {
                log::info!("Attempting to load most recent project: {:?}", last_project);

                // Change to the recent project directory
                if let Err(e) = std::env::set_current_dir(last_project) {
                    return Err(format!(
                        "No POM found in current directory and failed to load recent project '{}': {}",
                        last_project.display(),
                        e
                    )
                    .into());
                }

                // Try to load modules from recent project
                match project::get_project_modules() {
                    Ok(result) => {
                        log::info!("Successfully loaded recent project: {:?}", result.1);
                        result
                    }
                    Err(e) => {
                        return Err(format!(
                            "No POM found in current directory and recent project '{}' is invalid: {}",
                            last_project.display(),
                            e
                        )
                        .into());
                    }
                }
            } else {
                return Err(
                    "No POM found in current directory and no recent projects available.\n\
                     Please run lazymvn from a Maven project directory or use --project flag.\n\
                     Example: lazymvn --project /path/to/maven/project"
                        .into(),
                );
            }
        }
    };

    // Step 2: Loading configuration
    loading_step!(progress, terminal, 2, 5, "Loading configuration...");
    
    // Add current project to recent projects
    let mut recent_projects = config::RecentProjects::load();
    recent_projects.add(project_root.clone());

    // Load config and apply CLI overrides for launch mode
    let mut config = config::load_config(&project_root);

    // CLI flags override config file
    if cli.force_run {
        log::info!("CLI override: using force-run mode");
        config.launch_mode = Some(config::LaunchMode::ForceRun);
    } else if cli.force_exec {
        log::info!("CLI override: using force-exec mode");
        config.launch_mode = Some(config::LaunchMode::ForceExec);
    }
    // If neither flag is set, use config value or default to Auto
    if config.launch_mode.is_none() {
        config.launch_mode = Some(config::LaunchMode::Auto);
    }

    // Step 3: Initializing UI state
    loading_step!(progress, terminal, 3, 5, "Initializing UI state...");
    
    let mut state = tui::TuiState::new(modules, project_root.clone(), config);

    // Step 4: Discovering Maven profiles (asynchronous)
    loading_step!(progress, terminal, 4, 5, "Starting profile discovery...");
    
    // Start loading profiles asynchronously
    state.start_loading_profiles();

    // Step 5: Ready!
    loading_step!(progress, terminal, 5, 5, "Starting LazyMVN...");
    
    // Small delay to show completion
    std::thread::sleep(std::time::Duration::from_millis(300));

    loop {
        // Poll for command updates first
        state.poll_command_updates();

        // Poll for profile loading updates
        state.poll_profiles_updates();

        // Check file watcher for auto-reload
        state.check_file_watcher();

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

                    // Load config and apply CLI overrides
                    let mut new_config = config::load_config(&new_project_root);
                    if cli.force_run {
                        new_config.launch_mode = Some(config::LaunchMode::ForceRun);
                    } else if cli.force_exec {
                        new_config.launch_mode = Some(config::LaunchMode::ForceExec);
                    }
                    if new_config.launch_mode.is_none() {
                        new_config.launch_mode = Some(config::LaunchMode::Auto);
                    }

                    // Create new state
                    state = tui::TuiState::new(new_modules, new_project_root.clone(), new_config);

                    // Load profiles asynchronously
                    state.start_loading_profiles();
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

/// Generate a lazymvn.toml configuration file in the current directory
fn setup_config() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;

    let config_path = Path::new("lazymvn.toml");

    // Check if file already exists
    if config_path.exists() {
        eprintln!("⚠️  lazymvn.toml already exists in current directory");
        eprint!("   Overwrite? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            eprintln!("❌ Setup cancelled");
            return Ok(());
        }
    }

    let config_template = r#"# LazyMVN Configuration File
# Generated with: lazymvn --setup

# Maven settings file path (optional)
# Uncomment to use a custom settings.xml
# maven_settings = "settings.xml"

# Launch mode for Spring Boot applications
# Options:
#   "auto"       - Auto-detect: use spring-boot:run if available, fallback to exec:java
#   "force-run"  - Always use spring-boot:run
#   "force-exec" - Always use exec:java
# Default: "auto"
launch_mode = "auto"

# Enable desktop notifications (optional)
# notifications_enabled = true

# File watching configuration for auto-reload
[watch]
# Enable file watching (set to true to activate)
# When enabled, LazyMVN will automatically re-run certain commands
# when source files change
enabled = false

# Commands that should trigger auto-reload when files change
# Common values: ["test", "start", "compile", "package"]
# When you run these commands, any matching file change will re-run the command
commands = ["test", "start"]

# File patterns to watch (glob syntax)
# Patterns support:
#   *.ext         - Any file with extension .ext
#   path/*.ext    - Files in specific path (not subdirs)
#   path/**/*.ext - Files in path and all subdirectories
#   **/name.ext   - Files anywhere with specific name
patterns = [
    "src/**/*.java",           # Java source files
    "src/**/*.kt",             # Kotlin source files
    "src/**/*.properties",     # Properties files
    "src/**/*.yaml",           # YAML configuration
    "src/**/*.yml",            # YAML configuration
    "src/**/*.xml",            # XML files (excluding pom.xml)
]

# Debounce delay in milliseconds
# Waits this long after the last file change before triggering reload
# Prevents multiple reloads when saving multiple files at once
# Recommended: 500-1000ms
debounce_ms = 500

# Example: Enable watch mode for TDD workflow
# [watch]
# enabled = true
# commands = ["test"]
# patterns = ["src/**/*.java", "src/**/*.properties"]
# debounce_ms = 500

# Example: Enable watch mode for Spring Boot development
# [watch]
# enabled = true
# commands = ["start"]
# patterns = ["src/**/*.java", "src/**/*.properties", "src/**/*.yaml"]
# debounce_ms = 1000

# Output buffer configuration
[output]
# Maximum number of lines to keep in output buffer (default: 10000)
# When exceeded, oldest lines are removed to prevent memory issues
max_lines = 10000

# Maximum number of updates to process per poll cycle (default: 100)
# Limits updates per event loop iteration to prevent UI freeze
# Increase if you have a very fast machine, decrease for slower systems
max_updates_per_poll = 100
"#;

    fs::write(config_path, config_template)?;

    println!("✅ Successfully created lazymvn.toml");
    println!();
    println!("📁 Location: {}", config_path.canonicalize()?.display());
    println!();
    println!("🎯 Next steps:");
    println!("   1. Edit lazymvn.toml to customize your settings");
    println!("   2. Enable watch mode if you want auto-reload");
    println!("   3. Run 'lazymvn' to start");
    println!();
    println!("📖 Configuration options:");
    println!("   • maven_settings          - Path to custom settings.xml");
    println!("   • launch_mode             - How to launch Spring Boot apps");
    println!("   • watch.enabled           - Enable file watching for auto-reload");
    println!("   • watch.commands          - Commands that trigger auto-reload");
    println!("   • watch.patterns          - File patterns to watch");
    println!("   • output.max_lines        - Max output buffer size");
    println!("   • output.max_updates_per_poll - Output processing rate limit");
    println!();
    println!("💡 Tip: Use 'lazymvn --debug' to see detailed logs");

    Ok(())
}

// Maven tests are kept separate to avoid visibility issues
#[cfg(test)]
#[path = "maven_tests.rs"]
mod maven_tests;
