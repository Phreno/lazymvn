//! Testable helper functions for Maven command construction

/// Parse Maven output lines to extract profile IDs
/// Expects lines in the format: "Profile Id: profileName (source: ...)"
pub fn parse_profile_id_from_line(line: &str) -> Option<String> {
    if line.contains("Profile Id:") {
        let parts: Vec<&str> = line.split("Profile Id:").collect();
        if parts.len() > 1 {
            // Extract just the profile name, stop at first space or parenthesis
            let profile_part = parts[1].trim();
            let profile_name = profile_part
                .split_whitespace()
                .next()
                .unwrap_or("")
                .split('(')
                .next()
                .unwrap_or("")
                .trim();
            if !profile_name.is_empty() {
                return Some(profile_name.to_string());
            }
        }
    }
    None
}

/// Parse Maven output lines to extract active profile names
/// Expects lines in the format: " - profileName (source: ...)"
pub fn parse_active_profile_from_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if let Some(stripped) = trimmed.strip_prefix("- ") {
        let parts: Vec<&str> = stripped.split_whitespace().collect();
        if let Some(profile_name) = parts.first() {
            return Some(profile_name.to_string());
        }
    }
    None
}

/// Filter out incompatible flags for spring-boot:run
/// --also-make and --also-make-dependents cause spring-boot:run to execute on ALL modules
pub fn filter_spring_boot_incompatible_flags(flags: &[String]) -> Vec<String> {
    flags
        .iter()
        .filter(|flag| {
            let flag_lower = flag.to_lowercase();
            !flag_lower.contains("also-make")
        })
        .cloned()
        .collect()
}

/// Check if the args contain a spring-boot:run goal
pub fn is_spring_boot_run_command(args: &[&str]) -> bool {
    args.iter().any(|arg| {
        arg.contains("spring-boot:run")
            || (arg.contains("spring-boot-maven-plugin") && arg.contains(":run"))
    })
}

/// Parse a flag string and extract individual flags
/// Handles comma-separated aliases like "-U, --update-snapshots"
/// Returns only the first part before comma
pub fn parse_flag_parts(flag: &str) -> Vec<String> {
    flag.split(',')
        .next()
        .unwrap_or(flag)
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_profile_id_from_line_valid() {
        let line = "Profile Id: dev (source: pom.xml)";
        assert_eq!(
            parse_profile_id_from_line(line),
            Some("dev".to_string())
        );
    }

    #[test]
    fn test_parse_profile_id_from_line_with_spaces() {
        let line = "Profile Id:   production   (source: pom.xml)";
        assert_eq!(
            parse_profile_id_from_line(line),
            Some("production".to_string())
        );
    }

    #[test]
    fn test_parse_profile_id_from_line_no_match() {
        let line = "Some other line without profile info";
        assert_eq!(parse_profile_id_from_line(line), None);
    }

    #[test]
    fn test_parse_profile_id_from_line_empty() {
        let line = "Profile Id:   ";
        assert_eq!(parse_profile_id_from_line(line), None);
    }

    #[test]
    fn test_parse_active_profile_from_line_valid() {
        let line = " - dev (source: settings.xml)";
        assert_eq!(
            parse_active_profile_from_line(line),
            Some("dev".to_string())
        );
    }

    #[test]
    fn test_parse_active_profile_from_line_no_prefix() {
        let line = "dev (source: settings.xml)";
        assert_eq!(parse_active_profile_from_line(line), None);
    }

    #[test]
    fn test_parse_active_profile_from_line_empty() {
        let line = " - ";
        assert_eq!(parse_active_profile_from_line(line), None);
    }

    #[test]
    fn test_filter_spring_boot_incompatible_flags_removes_also_make() {
        let flags = vec![
            "-DskipTests".to_string(),
            "--also-make".to_string(),
            "-X".to_string(),
            "--also-make-dependents".to_string(),
        ];
        let filtered = filter_spring_boot_incompatible_flags(&flags);
        assert_eq!(filtered, vec!["-DskipTests", "-X"]);
    }

    #[test]
    fn test_filter_spring_boot_incompatible_flags_preserves_others() {
        let flags = vec!["-DskipTests".to_string(), "-X".to_string()];
        let filtered = filter_spring_boot_incompatible_flags(&flags);
        assert_eq!(filtered, flags);
    }

    #[test]
    fn test_filter_spring_boot_incompatible_flags_empty() {
        let flags: Vec<String> = vec![];
        let filtered = filter_spring_boot_incompatible_flags(&flags);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_is_spring_boot_run_command_direct() {
        let args = vec!["spring-boot:run"];
        assert!(is_spring_boot_run_command(&args));
    }

    #[test]
    fn test_is_spring_boot_run_command_with_plugin() {
        let args = vec!["org.springframework.boot:spring-boot-maven-plugin:run"];
        assert!(is_spring_boot_run_command(&args));
    }

    #[test]
    fn test_is_spring_boot_run_command_other_goal() {
        let args = vec!["clean", "install"];
        assert!(!is_spring_boot_run_command(&args));
    }

    #[test]
    fn test_is_spring_boot_run_command_build_image() {
        let args = vec!["spring-boot:build-image"];
        assert!(!is_spring_boot_run_command(&args));
    }

    #[test]
    fn test_parse_flag_parts_simple() {
        let flag = "-X";
        assert_eq!(parse_flag_parts(flag), vec!["-X"]);
    }

    #[test]
    fn test_parse_flag_parts_with_alias() {
        let flag = "-U, --update-snapshots";
        assert_eq!(parse_flag_parts(flag), vec!["-U"]);
    }

    #[test]
    fn test_parse_flag_parts_multiple_words() {
        let flag = "-D property=value";
        assert_eq!(parse_flag_parts(flag), vec!["-D", "property=value"]);
    }

    #[test]
    fn test_parse_flag_parts_empty() {
        let flag = "";
        let parts = parse_flag_parts(flag);
        assert!(parts.is_empty());
    }
}
