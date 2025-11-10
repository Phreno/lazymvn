use log::Record;

/// Format a log line with session ID and timestamp
pub fn format_log_line(record: &Record, session_id: &Option<String>) -> String {
    let timestamp = get_current_timestamp();
    let session_prefix = get_session_prefix(session_id);
    format!(
        "{}[{}] {} - {}",
        session_prefix,
        timestamp,
        record.level(),
        record.args()
    )
}

/// Get current timestamp formatted for logs
fn get_current_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

/// Get session prefix for log line
fn get_session_prefix(session_id: &Option<String>) -> String {
    session_id
        .as_ref()
        .map(|id| format!("[SESSION:{}] ", id))
        .unwrap_or_default()
}

/// Build session marker string
pub fn build_session_marker(session_id: &str) -> String {
    format!("[SESSION:{}]", session_id)
}

/// Check if line is a TRACE level log
pub fn is_trace_log_line(line: &str) -> bool {
    line.contains("] TRACE - ")
}

/// Check if line is from our session
pub fn is_line_from_session(line: &str, session_marker: &str) -> bool {
    line.contains(session_marker)
}

/// Check if line marks a different session
pub fn is_different_session_marker(line: &str, session_marker: &str) -> bool {
    line.contains("[SESSION:") && !line.contains(session_marker)
}

/// Build log header
pub fn build_log_header(session_id: &str) -> Vec<String> {
    vec![
        "=== LazyMVN Session Logs ===".to_string(),
        format!("Session ID: {}", session_id),
        format!(
            "Timestamp: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ),
        String::new(),
    ]
}

/// Get the last N lines from a vector
pub fn get_last_n_lines(all_lines: Vec<String>, max_lines: usize) -> Vec<String> {
    let start_idx = calculate_start_index(all_lines.len(), max_lines);
    all_lines[start_idx..].to_vec()
}

/// Calculate start index for last N lines
fn calculate_start_index(total_lines: usize, max_lines: usize) -> usize {
    total_lines.saturating_sub(max_lines)
}

/// Filter out TRACE logs and limit to 300 lines
pub fn filter_and_limit_logs(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| !is_trace_log_line(line))
        .take(300)
        .map(|s| s.to_string())
        .collect()
}

/// Build output with filter information
pub fn build_filtered_output(original_lines: &[&str], filtered_lines: &[String]) -> String {
    let total_lines = original_lines.len();
    let filtered_count = filtered_lines.len();
    let mut output = Vec::new();
    
    if filtered_count < total_lines {
        output.push(format!(
            "(Filtered {} lines, showing {} lines - TRACE logs excluded)",
            total_lines, filtered_count
        ));
    }
    
    output.extend(filtered_lines.iter().cloned());
    output.join("\n")
}
