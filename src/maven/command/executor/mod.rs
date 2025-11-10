mod args;
mod display;
mod env;
mod stream;

use crate::core::config::LoggingConfig;
use crate::maven::command::builder::get_maven_command;
use crate::maven::process::CommandUpdate;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub use display::build_command_display;

/// Execute Maven command synchronously
pub fn execute_maven_command(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
) -> Result<Vec<String>, std::io::Error> {
    execute_maven_command_with_options(
        project_root,
        module,
        args,
        profiles,
        settings_path,
        flags,
        false, // use_file_flag = false for backward compatibility
        None,  // No logging config for backward compatibility
    )
}

/// Execute Maven command with option to use -f instead of -pl
#[allow(clippy::too_many_arguments)]
pub fn execute_maven_command_with_options(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
    logging_config: Option<&LoggingConfig>,
) -> Result<Vec<String>, std::io::Error> {
    let maven_command = get_maven_command(project_root);
    log::debug!("execute_maven_command: Using command: {}", maven_command);
    log::debug!("  project_root: {:?}", project_root);
    log::debug!("  module: {:?}", module);
    log::debug!("  args: {:?}", args);
    log::debug!("  profiles: {:?}", profiles);
    log::debug!("  settings_path: {:?}", settings_path);
    log::debug!("  flags: {:?}", flags);
    log::debug!("  use_file_flag: {}", use_file_flag);

    let mut command = Command::new(&maven_command);
    
    // Configure environment (JAVA_TOOL_OPTIONS, etc.)
    env::configure_environment(&mut command, args, logging_config);
    
    // Add all Maven arguments
    args::add_maven_arguments(
        &mut command,
        module,
        project_root,
        profiles,
        settings_path,
        flags,
        args,
        use_file_flag,
        logging_config,
    );

    log::debug!("Final command: {:?}", command);

    // Format command display string
    let command_display = build_command_display(
        &maven_command,
        module,
        profiles,
        settings_path,
        flags,
        args,
        use_file_flag,
    );

    let output = command.current_dir(project_root).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut lines: Vec<String> = vec![command_display];
    lines.extend(stdout.lines().map(|s| s.to_string()));
    lines.extend(stderr.lines().map(|s| format!("[ERR] {}", s)));

    log::debug!("Command completed with status: {}", output.status);
    log::debug!("Output lines: {}", lines.len());

    if !output.status.success() {
        log::warn!(
            "Maven command failed with exit code: {:?}",
            output.status.code()
        );
    }

    Ok(lines)
}

/// Execute Maven command asynchronously with default options
#[allow(dead_code)]
pub fn execute_maven_command_async(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
) -> Result<mpsc::Receiver<CommandUpdate>, std::io::Error> {
    execute_maven_command_async_with_options(
        project_root,
        module,
        args,
        profiles,
        settings_path,
        flags,
        false, // use_file_flag = false for backward compatibility
        None,  // No logging config for backward compatibility
    )
}

/// Execute Maven command asynchronously with streaming output and option to use -f
#[allow(clippy::too_many_arguments)]
pub fn execute_maven_command_async_with_options(
    project_root: &Path,
    module: Option<&str>,
    args: &[&str],
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    use_file_flag: bool,
    logging_config: Option<&LoggingConfig>,
) -> Result<mpsc::Receiver<CommandUpdate>, std::io::Error> {
    let (tx, rx) = mpsc::channel();
    let maven_command = get_maven_command(project_root);
    let project_root = project_root.to_path_buf();
    let module = module.map(|s| s.to_string());
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let profiles = profiles.to_vec();
    let settings_path = settings_path.map(|s| s.to_string());
    let flags = flags.to_vec();
    let logging_config = logging_config.cloned();

    thread::spawn(move || {
        log::info!("Starting Maven command in background thread");
        log::debug!("  command: {}", maven_command);
        log::debug!("  project_root: {:?}", project_root);
        log::debug!("  module: {:?}", module);
        log::debug!("  args: {:?}", args);
        log::debug!("  profiles: {:?}", profiles);
        log::debug!("  settings_path: {:?}", settings_path);
        log::debug!("  flags: {:?}", flags);
        log::debug!("  use_file_flag: {}", use_file_flag);

        let mut command = Command::new(&maven_command);

        // Configure environment
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        env::configure_environment(&mut command, &args_refs, logging_config.as_ref());

        // Add all Maven arguments
        args::add_maven_arguments(
            &mut command,
            module.as_deref(),
            &project_root,
            &profiles,
            settings_path.as_deref(),
            &flags,
            &args_refs,
            use_file_flag,
            logging_config.as_ref(),
        );

        command
            .current_dir(&project_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        log::debug!("Spawning Maven process...");
        let mut child = match command.spawn() {
            Ok(child) => {
                log::info!("Maven process spawned successfully (PID: {:?})", child.id());
                child
            }
            Err(e) => {
                log::error!("Failed to spawn Maven process: {}", e);
                let _ = tx.send(CommandUpdate::Error(format!("Failed to start: {}", e)));
                return;
            }
        };

        let child_id = child.id();
        log::debug!("Maven child process ID: {}", child_id);

        let _ = tx.send(CommandUpdate::Started(child_id));

        let stdout = child.stdout.take().expect("Failed to get stdout");
        let stderr = child.stderr.take().expect("Failed to get stderr");

        let tx_clone = tx.clone();
        let stdout_handle = thread::spawn(move || {
            stream::read_lines_lossy(stdout, tx_clone, "STDOUT");
        });

        let tx_clone = tx.clone();
        let stderr_handle = thread::spawn(move || {
            stream::read_lines_lossy(stderr, tx_clone, "STDERR");
        });

        log::debug!("Waiting for output threads to complete...");
        let _ = stdout_handle.join();
        let _ = stderr_handle.join();
        log::debug!("Output threads completed");

        log::debug!("Waiting for Maven process to exit...");
        match child.wait() {
            Ok(status) => {
                log::info!("Maven process exited with status: {}", status);
                if status.success() {
                    let _ = tx.send(CommandUpdate::Completed);
                } else {
                    let error_msg = if let Some(code) = status.code() {
                        format!("Build failed with exit code {}", code)
                    } else {
                        "Build failed (terminated by signal)".to_string()
                    };
                    let _ = tx.send(CommandUpdate::Error(error_msg));
                }
            }
            Err(e) => {
                log::error!("Error waiting for Maven process: {}", e);
                let _ = tx.send(CommandUpdate::Error(format!("Process error: {}", e)));
            }
        }
        log::info!("Maven command thread finished");
    });

    Ok(rx)
}
