use termcolor::{Color, ColorSpec, WriteColor};
use std::io::Write;

pub fn colorize_line(line: &str) -> String {
    let mut buf = termcolor::Buffer::ansi();
    let mut color_spec = ColorSpec::new();

    if line.contains("[INFO]") {
        color_spec.set_fg(Some(Color::Green));
    } else if line.contains("[WARNING]") {
        color_spec.set_fg(Some(Color::Yellow));
    } else if line.contains("[ERROR]") || line.contains("[ERR]") {
        color_spec.set_fg(Some(Color::Red));
    } else {
        color_spec.set_fg(None);
    }

    buf.set_color(&color_spec).unwrap();
    buf.write_all(line.as_bytes()).unwrap();
    buf.reset().unwrap();

    String::from_utf8_lossy(buf.as_slice()).to_string()
}
