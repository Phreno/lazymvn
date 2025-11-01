//! XML parsing utilities for Maven POM files

#![allow(dead_code)]

/// Extract content from an XML tag
pub fn extract_tag_content(line: &str, tag_name: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag_name);
    let close_tag = format!("</{}>", tag_name);

    if let Some(start) = line.find(&open_tag)
        && let Some(end) = line.find(&close_tag)
    {
        let content = &line[start + open_tag.len()..end];
        return Some(content.trim().to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tag_content_simple() {
        let line = "<packaging>jar</packaging>";
        assert_eq!(
            extract_tag_content(line, "packaging"),
            Some("jar".to_string())
        );
    }

    #[test]
    fn test_extract_tag_content_with_whitespace() {
        let line = "<mainClass>  com.example.Main  </mainClass>";
        assert_eq!(
            extract_tag_content(line, "mainClass"),
            Some("com.example.Main".to_string())
        );
    }

    #[test]
    fn test_extract_tag_content_nested() {
        let line = "<groupId>com.example</groupId>";
        assert_eq!(
            extract_tag_content(line, "groupId"),
            Some("com.example".to_string())
        );
    }

    #[test]
    fn test_extract_tag_content_not_found() {
        let line = "<packaging>jar</packaging>";
        assert_eq!(extract_tag_content(line, "version"), None);
    }

    #[test]
    fn test_extract_tag_content_incomplete_tag() {
        let line = "<packaging>jar";
        assert_eq!(extract_tag_content(line, "packaging"), None);
    }

    #[test]
    fn test_extract_tag_content_empty() {
        let line = "<packaging></packaging>";
        assert_eq!(extract_tag_content(line, "packaging"), Some("".to_string()));
    }

    #[test]
    fn test_extract_tag_content_multiple_on_line() {
        let line = "<packaging>jar</packaging><version>1.0</version>";
        assert_eq!(
            extract_tag_content(line, "packaging"),
            Some("jar".to_string())
        );
        assert_eq!(
            extract_tag_content(line, "version"),
            Some("1.0".to_string())
        );
    }
}
