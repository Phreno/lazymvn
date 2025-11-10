use crate::maven::process::CommandUpdate;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc;

/// Helper function to read lines from a stream with UTF-8 lossy conversion
/// This ensures that non-UTF-8 characters (common in Maven output from Windows)
/// don't crash the reader thread
pub fn read_lines_lossy<R: Read>(
    reader: R,
    tx: mpsc::Sender<CommandUpdate>,
    stream_name: &str,
) {
    let mut buf_reader = BufReader::new(reader);
    let mut buffer = Vec::new();
    
    loop {
        buffer.clear();
        
        // Read until newline or EOF
        match buf_reader.read_until(b'\n', &mut buffer) {
            Ok(0) => {
                // EOF reached
                break;
            }
            Ok(_) => {
                // Convert bytes to string with lossy UTF-8 conversion
                // This replaces invalid UTF-8 sequences with �
                let line = String::from_utf8_lossy(&buffer);
                let line = line.trim_end_matches('\n').trim_end_matches('\r');
                
                log::trace!("[{}] {}", stream_name, line);
                
                if tx.send(CommandUpdate::OutputLine(line.to_string())).is_err() {
                    log::warn!("Failed to send {} line (receiver closed)", stream_name);
                    break;
                }
            }
            Err(e) => {
                log::error!("Error reading {}: {}", stream_name, e);
                break;
            }
        }
    }
    
    log::debug!("{} reader thread finished", stream_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_lines_lossy_handles_valid_utf8() {
        let (tx, rx) = mpsc::channel();
        let data = b"Line 1\nLine 2\nLine 3\n";
        let reader = Cursor::new(data);
        
        read_lines_lossy(reader, tx, "TEST");
        
        let mut lines = Vec::new();
        while let Ok(update) = rx.try_recv() {
            if let CommandUpdate::OutputLine(line) = update {
                lines.push(line);
            }
        }
        
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
    }

    #[test]
    fn read_lines_lossy_handles_invalid_utf8() {
        let (tx, rx) = mpsc::channel();
        // Include invalid UTF-8 sequence (0xFF is invalid in UTF-8)
        let data = b"Valid\nInvalid\xFF\nValid again\n";
        let reader = Cursor::new(data);
        
        read_lines_lossy(reader, tx, "TEST");
        
        let mut lines = Vec::new();
        while let Ok(update) = rx.try_recv() {
            if let CommandUpdate::OutputLine(line) = update {
                lines.push(line);
            }
        }
        
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Valid");
        // Invalid UTF-8 should be replaced with replacement character
        assert!(lines[1].contains("Invalid"));
        assert_eq!(lines[2], "Valid again");
    }

    #[test]
    fn read_lines_lossy_handles_windows_line_endings() {
        let (tx, rx) = mpsc::channel();
        let data = b"Line 1\r\nLine 2\r\nLine 3\r\n";
        let reader = Cursor::new(data);
        
        read_lines_lossy(reader, tx, "TEST");
        
        let mut lines = Vec::new();
        while let Ok(update) = rx.try_recv() {
            if let CommandUpdate::OutputLine(line) = update {
                lines.push(line);
            }
        }
        
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "Line 3");
    }
}
