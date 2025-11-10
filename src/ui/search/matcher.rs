use super::types::SearchMatch;
use regex::Regex;

pub fn collect_search_matches(command_output: &[String], regex: &Regex) -> Vec<SearchMatch> {
    command_output
        .iter()
        .enumerate()
        .flat_map(|(line_index, line)| find_matches_in_line(line, line_index, regex))
        .collect()
}

fn find_matches_in_line(line: &str, line_index: usize, regex: &Regex) -> Vec<SearchMatch> {
    let cleaned = crate::utils::clean_log_line(line).unwrap_or_default();
    regex
        .find_iter(&cleaned)
        .map(|mat| create_search_match(line_index, mat.start(), mat.end()))
        .collect()
}

fn create_search_match(line_index: usize, start: usize, end: usize) -> SearchMatch {
    SearchMatch {
        line_index,
        start,
        end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_search_matches_basic() {
        let output = vec![
            "This is a test line".to_string(),
            "Another test here".to_string(),
            "No match on this line".to_string(),
        ];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_index, 0);
        assert_eq!(matches[1].line_index, 1);
    }

    #[test]
    fn test_collect_search_matches_multiple_per_line() {
        let output = vec!["test test test".to_string()];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_collect_search_matches_case_sensitive() {
        let output = vec!["Test test TEST".to_string()];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_collect_search_matches_case_insensitive() {
        let output = vec!["Test test TEST".to_string()];
        let regex = Regex::new("(?i)test").unwrap();
        let matches = collect_search_matches(&output, &regex);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_collect_search_matches_empty_output() {
        let output: Vec<String> = vec![];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_collect_search_matches_no_matches() {
        let output = vec!["No match here".to_string(), "Still no match".to_string()];
        let regex = Regex::new("test").unwrap();
        let matches = collect_search_matches(&output, &regex);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_create_search_match() {
        let mat = create_search_match(5, 10, 15);
        assert_eq!(mat.line_index, 5);
        assert_eq!(mat.start, 10);
        assert_eq!(mat.end, 15);
    }
}
