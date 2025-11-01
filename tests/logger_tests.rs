use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temp log file with content
fn create_test_log_file(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test.log");
    let mut file = File::create(&log_path).unwrap();
    writeln!(file, "{}", content).unwrap();
    (temp_dir, log_path)
}

// Helper functions for testing (duplicated from logger.rs for testing purposes)

fn extract_session_logs_from_file(log_path: &PathBuf, session_id: &str) -> Vec<String> {
    let file = File::open(log_path).unwrap();
    let reader = BufReader::new(file);
    let session_marker = format!("[SESSION:{}]", session_id);

    let mut session_logs = Vec::new();
    let mut in_session = false;

    for line in reader.lines() {
        let line = line.unwrap();

        if line.contains(&session_marker) {
            in_session = true;
            session_logs.push(line);
        } else if in_session {
            if line.contains("[SESSION:") && !line.contains(&session_marker) {
                break;
            }
            session_logs.push(line);
        }
    }

    session_logs
}

fn read_last_lines_from_file(path: &PathBuf, max_lines: usize) -> Vec<String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let all_lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    let start_idx = if all_lines.len() > max_lines {
        all_lines.len() - max_lines
    } else {
        0
    };

    all_lines[start_idx..].to_vec()
}

fn parse_log_level_from_line(line: &str) -> Option<&str> {
    let levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
    
    for level in &levels {
        let pattern = format!("] {} - ", level);
        if line.contains(&pattern) {
            return Some(level);
        }
    }
    
    None
}

fn extract_session_id(line: &str) -> Option<&str> {
    if let Some(start) = line.find("[SESSION:")
        && let Some(end) = line[start..].find(']')
    {
        let id_start = start + "[SESSION:".len();
        let id_end = start + end;
        return Some(&line[id_start..id_end]);
    }
    None
}

fn is_trace_log(line: &str) -> bool {
    line.contains("] TRACE - ")
}

fn filter_trace_logs(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| !is_trace_log(line))
        .map(|s| s.to_string())
        .collect()
}

mod session_extraction {
    use super::*;

    #[test]
    fn test_extract_session_logs_basic() {
        let content = "\
[SESSION:20250101-120000-001] [2025-01-01 12:00:00.001] INFO - Starting
[SESSION:20250101-120000-001] [2025-01-01 12:00:01.002] DEBUG - Processing
[SESSION:20250101-120000-002] [2025-01-01 12:01:00.001] INFO - Other session";

        let (_temp, log_path) = create_test_log_file(content);
        
        // Test extraction for first session
        let logs = extract_session_logs_from_file(&log_path, "20250101-120000-001");
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("Starting"));
        assert!(logs[1].contains("Processing"));
    }

    #[test]
    fn test_extract_session_logs_no_match() {
        let content = "\
[SESSION:20250101-120000-001] [2025-01-01 12:00:00.001] INFO - Test
[SESSION:20250101-120000-002] [2025-01-01 12:01:00.001] INFO - Test2";

        let (_temp, log_path) = create_test_log_file(content);
        
        let logs = extract_session_logs_from_file(&log_path, "20250101-999999-999");
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_extract_session_logs_stops_at_next_session() {
        let content = "\
[SESSION:20250101-120000-001] [2025-01-01 12:00:00.001] INFO - Session 1 line 1
[SESSION:20250101-120000-001] [2025-01-01 12:00:01.002] INFO - Session 1 line 2
[SESSION:20250101-120000-002] [2025-01-01 12:01:00.001] INFO - Session 2 line 1
[SESSION:20250101-120000-002] [2025-01-01 12:01:01.002] INFO - Session 2 line 2";

        let (_temp, log_path) = create_test_log_file(content);
        
        let logs = extract_session_logs_from_file(&log_path, "20250101-120000-001");
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("Session 1 line 1"));
        assert!(logs[1].contains("Session 1 line 2"));
        assert!(!logs.iter().any(|l| l.contains("Session 2")));
    }

    #[test]
    fn test_extract_session_logs_empty_file() {
        let (_temp, log_path) = create_test_log_file("");
        
        let logs = extract_session_logs_from_file(&log_path, "20250101-120000-001");
        assert_eq!(logs.len(), 0);
    }
}

mod last_lines_reading {
    use super::*;

    #[test]
    fn test_read_last_lines_all() {
        let content = "line1\nline2\nline3";
        let (_temp, log_path) = create_test_log_file(content);
        
        let lines = read_last_lines_from_file(&log_path, 10);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_read_last_lines_limited() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let (_temp, log_path) = create_test_log_file(content);
        
        let lines = read_last_lines_from_file(&log_path, 2);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line4");
        assert_eq!(lines[1], "line5");
    }

    #[test]
    fn test_read_last_lines_empty() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("empty.log");
        File::create(&log_path).unwrap(); // Create empty file without writing
        
        let lines = read_last_lines_from_file(&log_path, 10);
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_read_last_lines_exact_count() {
        let content = "line1\nline2\nline3";
        let (_temp, log_path) = create_test_log_file(content);
        
        let lines = read_last_lines_from_file(&log_path, 3);
        assert_eq!(lines.len(), 3);
    }
}

mod log_level_parsing {
    use super::*;

    #[test]
    fn test_parse_log_level_from_line() {
        assert_eq!(parse_log_level_from_line("[2025-01-01 12:00:00.001] INFO - Test"), Some("INFO"));
        assert_eq!(parse_log_level_from_line("[2025-01-01 12:00:00.001] ERROR - Test"), Some("ERROR"));
        assert_eq!(parse_log_level_from_line("[2025-01-01 12:00:00.001] DEBUG - Test"), Some("DEBUG"));
        assert_eq!(parse_log_level_from_line("[2025-01-01 12:00:00.001] WARN - Test"), Some("WARN"));
        assert_eq!(parse_log_level_from_line("[2025-01-01 12:00:00.001] TRACE - Test"), Some("TRACE"));
    }

    #[test]
    fn test_parse_log_level_with_session() {
        let line = "[SESSION:123] [2025-01-01 12:00:00.001] ERROR - Test";
        assert_eq!(parse_log_level_from_line(line), Some("ERROR"));
    }

    #[test]
    fn test_parse_log_level_invalid() {
        assert_eq!(parse_log_level_from_line("Not a log line"), None);
        assert_eq!(parse_log_level_from_line(""), None);
        assert_eq!(parse_log_level_from_line("[2025-01-01 12:00:00.001] - Test"), None);
    }
}

mod session_marker_extraction {
    use super::*;

    #[test]
    fn test_extract_session_marker() {
        let line = "[SESSION:20250101-120000-001] [2025-01-01 12:00:00.001] INFO - Test";
        assert_eq!(extract_session_id(line), Some("20250101-120000-001"));
    }

    #[test]
    fn test_extract_session_marker_no_marker() {
        let line = "[2025-01-01 12:00:00.001] INFO - Test";
        assert_eq!(extract_session_id(line), None);
    }

    #[test]
    fn test_extract_session_marker_empty() {
        assert_eq!(extract_session_id(""), None);
    }

    #[test]
    fn test_extract_session_marker_malformed() {
        let line = "[SESSION:] [2025-01-01 12:00:00.001] INFO - Test";
        assert_eq!(extract_session_id(line), Some(""));
    }
}

mod trace_log_filtering {
    use super::*;

    #[test]
    fn test_is_trace_log() {
        assert!(is_trace_log("[SESSION:123] [2025-01-01 12:00:00.001] TRACE - Test"));
        assert!(is_trace_log("[2025-01-01 12:00:00.001] TRACE - Test"));
        assert!(!is_trace_log("[2025-01-01 12:00:00.001] DEBUG - Test"));
        assert!(!is_trace_log("[2025-01-01 12:00:00.001] INFO - Test"));
        assert!(!is_trace_log("Not a log line"));
    }

    #[test]
    fn test_filter_trace_logs() {
        let lines = vec![
            "[SESSION:123] [2025-01-01 12:00:00.001] INFO - Test1",
            "[SESSION:123] [2025-01-01 12:00:00.002] TRACE - Test2",
            "[SESSION:123] [2025-01-01 12:00:00.003] DEBUG - Test3",
            "[SESSION:123] [2025-01-01 12:00:00.004] TRACE - Test4",
            "[SESSION:123] [2025-01-01 12:00:00.005] ERROR - Test5",
        ];

        let filtered = filter_trace_logs(&lines);
        assert_eq!(filtered.len(), 3);
        assert!(filtered[0].contains("INFO"));
        assert!(filtered[1].contains("DEBUG"));
        assert!(filtered[2].contains("ERROR"));
    }
}

