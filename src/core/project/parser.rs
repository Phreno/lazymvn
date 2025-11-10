use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Parse modules from POM XML content
pub fn parse_modules_from_str(content: &str) -> Vec<String> {
    let mut reader = create_xml_reader(content);
    let mut buf = Vec::new();
    let mut modules = Vec::new();
    let mut in_module = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if is_module_tag(&e) {
                    in_module = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_module {
                    add_module_text(&mut modules, &e);
                    in_module = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => (),
        }
        buf.clear();
    }

    modules
}

/// Create XML reader with trimmed text
fn create_xml_reader(content: &str) -> Reader<&[u8]> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    reader
}

/// Check if XML element is a module tag
fn is_module_tag(e: &quick_xml::events::BytesStart) -> bool {
    e.name().as_ref() == b"module"
}

/// Add module text to modules list
fn add_module_text(modules: &mut Vec<String>, e: &quick_xml::events::BytesText) {
    if let Ok(text) = e.decode() {
        modules.push(text.to_string());
    }
}

/// Compute hash of POM content for cache invalidation
pub fn compute_pom_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Normalize modules: return ["."] for empty module list (single-module project)
pub fn normalize_modules(modules: Vec<String>) -> Vec<String> {
    if modules.is_empty() {
        log::info!("No modules found, treating as single-module project");
        vec![".".to_string()]
    } else {
        modules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_modules_from_pom() {
        let content = "<project><modules><module>module1</module><module>module2</module></modules></project>";
        let modules = parse_modules_from_str(content);
        assert_eq!(modules, vec!["module1", "module2"]);
    }

    #[test]
    fn parse_modules_from_pom_without_modules() {
        let content = "<project><groupId>com.example</groupId><artifactId>simple</artifactId></project>";
        let modules = parse_modules_from_str(content);
        assert_eq!(modules, Vec::<String>::new());
    }

    #[test]
    fn normalize_modules_returns_dot_for_empty() {
        let empty_modules = vec![];
        let normalized = normalize_modules(empty_modules);
        assert_eq!(normalized, vec!["."]);
    }

    #[test]
    fn normalize_modules_preserves_non_empty() {
        let modules = vec!["module1".to_string(), "module2".to_string()];
        let normalized = normalize_modules(modules.clone());
        assert_eq!(normalized, modules);
    }

    #[test]
    fn compute_pom_hash_is_consistent() {
        let content = "<project><modules><module>test</module></modules></project>";
        let hash1 = compute_pom_hash(content);
        let hash2 = compute_pom_hash(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn compute_pom_hash_differs_for_different_content() {
        let content1 = "<project><modules><module>module1</module></modules></project>";
        let content2 = "<project><modules><module>module2</module></modules></project>";
        let hash1 = compute_pom_hash(content1);
        let hash2 = compute_pom_hash(content2);
        assert_ne!(hash1, hash2);
    }
}
