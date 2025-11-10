use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use super::formatters;
use super::file_ops;

/// Extract logs for the current session from a log file
pub fn extract_session_logs(
    log_path: &PathBuf,
    session_id: &str,
) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let session_marker = formatters::build_session_marker(session_id);

    collect_session_lines(reader, &session_marker)
}

/// Collect lines belonging to a session
fn collect_session_lines<R: std::io::BufRead>(
    reader: R,
    session_marker: &str,
) -> Result<Vec<String>, std::io::Error> {
    let mut session_logs = Vec::new();
    let mut in_session = false;

    for line in reader.lines() {
        let line = line?;

        if formatters::is_line_from_session(&line, session_marker) {
            in_session = true;
            session_logs.push(line);
        } else if in_session {
            if formatters::is_different_session_marker(&line, session_marker) {
                break;
            }
            session_logs.push(line);
        }
    }

    Ok(session_logs)
}

/// Read the last N lines from a file (tail-like functionality)
pub fn read_last_lines(path: &Path, max_lines: usize) -> Result<Vec<String>, std::io::Error> {
    use std::io::BufRead;

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    Ok(formatters::get_last_n_lines(all_lines, max_lines))
}

/// Flush logs and wait for filesystem sync
pub fn flush_and_wait_for_sync() {
    log::logger().flush();
    std::thread::sleep(std::time::Duration::from_millis(10));
}

/// Get concatenated logs from the current session (debug + error logs)
pub fn get_current_session_logs(session_id: &str) -> Result<String, String> {
    flush_and_wait_for_sync();
    
    let mut all_logs = formatters::build_log_header(session_id);
    add_debug_logs(&mut all_logs, session_id);
    add_error_logs(&mut all_logs, session_id);

    Ok(all_logs.join("\n"))
}

/// Add debug logs to output
fn add_debug_logs(all_logs: &mut Vec<String>, session_id: &str) {
    if let Some(debug_path) = file_ops::get_debug_log_path()
        && debug_path.exists()
    {
        all_logs.push("=== Debug Logs ===".to_string());
        add_session_logs_from_file(all_logs, &debug_path, session_id, "debug");
        all_logs.push(String::new());
    }
}

/// Add error logs to output
fn add_error_logs(all_logs: &mut Vec<String>, session_id: &str) {
    if let Some(error_path) = file_ops::get_error_log_path()
        && error_path.exists()
    {
        all_logs.push("=== Error Logs ===".to_string());
        add_session_logs_from_file(all_logs, &error_path, session_id, "errors");
        all_logs.push(String::new());
    }
}

/// Add session logs from a file to output
fn add_session_logs_from_file(
    all_logs: &mut Vec<String>,
    path: &PathBuf,
    session_id: &str,
    log_type: &str,
) {
    match extract_session_logs(path, session_id) {
        Ok(logs) => {
            if logs.is_empty() {
                all_logs.push(format!("(No {} for this session)", log_type));
            } else {
                all_logs.extend(logs);
            }
        }
        Err(e) => {
            all_logs.push(format!("Error reading {} logs: {}", log_type, e));
        }
    }
}

/// Get all available logs (last 500 lines from debug and error logs)
pub fn get_all_logs() -> String {
    let mut output = Vec::new();
    add_debug_log_tail(&mut output);
    add_error_log_tail(&mut output);
    output.join("\n")
}

/// Add debug log tail to output
fn add_debug_log_tail(output: &mut Vec<String>) {
    if let Some(debug_path) = file_ops::get_debug_log_path()
        && debug_path.exists()
    {
        output.push("=== Debug Logs (last 500 lines) ===".to_string());
        add_log_tail(output, &debug_path, "debug");
        output.push(String::new());
    }
}

/// Add error log tail to output
fn add_error_log_tail(output: &mut Vec<String>) {
    if let Some(error_path) = file_ops::get_error_log_path()
        && error_path.exists()
    {
        output.push("=== Error Logs (last 500 lines) ===".to_string());
        add_log_tail(output, &error_path, "error");
    }
}

/// Add log tail to output
fn add_log_tail(output: &mut Vec<String>, path: &Path, log_type: &str) {
    match read_last_lines(path, 500) {
        Ok(lines) => {
            if lines.is_empty() {
                output.push(format!("(No {} logs)", log_type));
            } else {
                output.extend(lines);
            }
        }
        Err(e) => {
            output.push(format!("Error reading {} logs: {}", log_type, e));
        }
    }
}

/// Get logs for debug report (optimized for size)
/// - Only includes current session logs (across all rotated files)
/// - Filters out TRACE level logs (keeps DEBUG, INFO, WARN, ERROR)
/// - Limits to last 300 lines to keep report manageable
pub fn get_logs_for_debug_report(session_id: &str) -> String {
    match get_current_session_logs(session_id) {
        Ok(session_logs) => format_debug_report(&session_logs),
        Err(e) => format!("Error getting session logs: {}", e),
    }
}

/// Format session logs for debug report
fn format_debug_report(session_logs: &str) -> String {
    let lines: Vec<&str> = session_logs.lines().collect();
    let filtered = formatters::filter_and_limit_logs(&lines);
    
    if filtered.is_empty() {
        "(No logs for current session)".to_string()
    } else {
        formatters::build_filtered_output(&lines, &filtered)
    }
}
