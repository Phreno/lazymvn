use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn colorize_xml_line(text: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // Flush any accumulated text as normal
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }

            // Start of tag
            current.push(ch);

            // Check if it's a closing tag or comment
            let is_closing = chars.peek() == Some(&'/');
            let is_comment = chars.peek() == Some(&'!');

            // Consume the tag
            let mut in_quotes = false;
            let mut quote_char = ' ';

            while let Some(&next_ch) = chars.peek() {
                chars.next();
                current.push(next_ch);

                if next_ch == '"' || next_ch == '\'' {
                    if in_quotes && quote_char == next_ch {
                        in_quotes = false;
                    } else if !in_quotes {
                        in_quotes = true;
                        quote_char = next_ch;
                    }
                }

                if next_ch == '>' && !in_quotes {
                    break;
                }
            }

            // Colorize the tag
            if is_comment {
                // Comments in dark gray (more subtle)
                spans.push(Span::styled(
                    current.clone(),
                    Style::default().fg(Color::DarkGray),
                ));
            } else {
                // Parse tag name and attributes
                colorize_xml_tag(&current, &mut spans, is_closing);
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    // Flush remaining text
    if !current.is_empty() {
        let trimmed = current.trim();
        if !trimmed.is_empty() {
            // Text content in white
            spans.push(Span::raw(current));
        } else {
            // Preserve whitespace
            spans.push(Span::raw(current));
        }
    }

    if spans.is_empty() {
        spans.push(Span::raw(text.to_string()));
    }

    Line::from(spans)
}

/// Helper to colorize XML tag components
fn colorize_xml_tag(tag: &str, spans: &mut Vec<Span<'static>>, is_closing: bool) {
    // Tag format: <tagname attr="value">
    let content = tag.trim_start_matches('<').trim_end_matches('>');

    if content.starts_with('?') {
        // XML declaration: <?xml ... ?> in light purple
        spans.push(Span::styled(
            tag.to_string(),
            Style::default().fg(Color::LightMagenta),
        ));
        return;
    }

    let mut parts = content.split_whitespace();

    // Tag name
    if let Some(tag_name) = parts.next() {
        // Opening bracket in dark gray
        spans.push(Span::styled(
            "<".to_string(),
            Style::default().fg(Color::DarkGray),
        ));

        // Tag name: light blue for opening, light red for closing
        let tag_color = if is_closing {
            Color::LightRed
        } else {
            Color::LightBlue
        };
        spans.push(Span::styled(
            tag_name.to_string(),
            Style::default().fg(tag_color),
        ));

        // Attributes
        let remainder: Vec<&str> = parts.collect();
        if !remainder.is_empty() {
            let attrs = remainder.join(" ");
            colorize_xml_attributes(&attrs, spans);
        }

        // Closing bracket in dark gray
        spans.push(Span::styled(
            ">".to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        // Fallback
        spans.push(Span::raw(tag.to_string()));
    }
}

/// Helper to colorize XML attributes
fn colorize_xml_attributes(attrs: &str, spans: &mut Vec<Span<'static>>) {
    let chars = attrs.chars();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut in_value = false;

    for ch in chars {
        if ch == '=' && !in_quotes {
            // Attribute name in light yellow
            if !current.trim().is_empty() {
                spans.push(Span::raw(" ".to_string()));
                spans.push(Span::styled(
                    current.trim().to_string(),
                    Style::default().fg(Color::LightYellow),
                ));
                current.clear();
            }
            spans.push(Span::styled(
                "=".to_string(),
                Style::default().fg(Color::DarkGray),
            ));
            in_value = true;
        } else if ch == '"' || ch == '\'' {
            if !in_quotes {
                // Start of value
                in_quotes = true;
                current.push(ch);
            } else {
                // End of value - in light green
                current.push(ch);
                spans.push(Span::styled(
                    current.clone(),
                    Style::default().fg(Color::LightGreen),
                ));
                current.clear();
                in_quotes = false;
                in_value = false;
            }
        } else if ch.is_whitespace() && !in_quotes && !in_value {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        spans.push(Span::raw(current));
    }
}

