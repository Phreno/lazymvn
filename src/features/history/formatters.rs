/// Build command parts from goal, profiles, and flags
pub fn build_command_parts(goal: &str, profiles: &[String], flags: &[String]) -> Vec<String> {
    let mut parts = vec![goal.to_string()];

    if !profiles.is_empty() {
        parts.push(format_profiles(profiles));
    }

    parts.extend(flags.iter().cloned());
    parts
}

/// Format profiles as Maven argument
pub fn format_profiles(profiles: &[String]) -> String {
    format!("-P {}", profiles.join(","))
}

/// Format module name for display
pub fn format_module_name(module: &str) -> String {
    if module == "." {
        "(root)".to_string()
    } else {
        module.to_string()
    }
}

/// Format Unix timestamp to local time string
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::TimeZone;
    let dt = chrono::Utc.timestamp_opt(timestamp, 0).unwrap();
    let local_dt = dt.with_timezone(&chrono::Local);
    local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_module_name_root() {
        assert_eq!(format_module_name("."), "(root)");
    }

    #[test]
    fn test_format_module_name_regular() {
        assert_eq!(format_module_name("my-module"), "my-module");
    }

    #[test]
    fn test_format_profiles_single() {
        let profiles = vec!["dev".to_string()];
        assert_eq!(format_profiles(&profiles), "-P dev");
    }

    #[test]
    fn test_format_profiles_multiple() {
        let profiles = vec!["dev".to_string(), "local".to_string()];
        assert_eq!(format_profiles(&profiles), "-P dev,local");
    }

    #[test]
    fn test_build_command_parts_minimal() {
        let parts = build_command_parts("test", &[], &[]);
        assert_eq!(parts, vec!["test"]);
    }

    #[test]
    fn test_build_command_parts_with_profiles() {
        let profiles = vec!["dev".to_string()];
        let parts = build_command_parts("test", &profiles, &[]);
        assert_eq!(parts, vec!["test", "-P dev"]);
    }

    #[test]
    fn test_build_command_parts_with_flags() {
        let flags = vec!["-X".to_string(), "-U".to_string()];
        let parts = build_command_parts("test", &[], &flags);
        assert_eq!(parts, vec!["test", "-X", "-U"]);
    }

    #[test]
    fn test_build_command_parts_complete() {
        let profiles = vec!["prod".to_string()];
        let flags = vec!["-X".to_string()];
        let parts = build_command_parts("package", &profiles, &flags);
        assert_eq!(parts, vec!["package", "-P prod", "-X"]);
    }
}
