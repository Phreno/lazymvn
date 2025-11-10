/// Build command display string for UI
pub fn build_command_display(
    maven_command: &str,
    module: Option<&str>,
    profiles: &[String],
    settings_path: Option<&str>,
    flags: &[String],
    args: &[&str],
    use_file_flag: bool,
) -> String {
    let mut command_display = format!("$ {}", maven_command);
    
    if let Some(m) = module
        && m != "."
    {
        if use_file_flag {
            command_display.push_str(&format!(" -f {}/pom.xml", m));
        } else {
            command_display.push_str(&format!(" -pl {}", m));
        }
    }
    
    if !profiles.is_empty() {
        command_display.push_str(&format!(" -P {}", profiles.join(",")));
    }
    
    if let Some(s) = settings_path {
        command_display.push_str(&format!(" -s {}", s));
    }
    
    for flag in flags {
        command_display.push_str(&format!(" {}", flag));
    }
    
    for arg in args {
        command_display.push_str(&format!(" {}", arg));
    }
    
    command_display
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_display_basic() {
        let result = build_command_display(
            "mvn",
            None,
            &[],
            None,
            &[],
            &["clean", "install"],
            false,
        );
        assert_eq!(result, "$ mvn clean install");
    }

    #[test]
    fn test_build_command_display_with_module() {
        let result = build_command_display(
            "mvn",
            Some("my-module"),
            &[],
            None,
            &[],
            &["clean"],
            false,
        );
        assert_eq!(result, "$ mvn -pl my-module clean");
    }

    #[test]
    fn test_build_command_display_with_module_using_file_flag() {
        let result = build_command_display(
            "mvn",
            Some("my-module"),
            &[],
            None,
            &[],
            &["exec:java"],
            true,
        );
        assert_eq!(result, "$ mvn -f my-module/pom.xml exec:java");
    }

    #[test]
    fn test_build_command_display_with_profiles() {
        let result = build_command_display(
            "mvn",
            None,
            &["dev".to_string(), "local".to_string()],
            None,
            &[],
            &["test"],
            false,
        );
        assert_eq!(result, "$ mvn -P dev,local test");
    }

    #[test]
    fn test_build_command_display_with_settings() {
        let result = build_command_display(
            "mvn",
            None,
            &[],
            Some("settings.xml"),
            &[],
            &["verify"],
            false,
        );
        assert_eq!(result, "$ mvn -s settings.xml verify");
    }

    #[test]
    fn test_build_command_display_with_flags() {
        let result = build_command_display(
            "mvn",
            None,
            &[],
            None,
            &["-U".to_string(), "--quiet".to_string()],
            &["package"],
            false,
        );
        assert_eq!(result, "$ mvn -U --quiet package");
    }

    #[test]
    fn test_build_command_display_complete() {
        let result = build_command_display(
            "./mvnw",
            Some("backend"),
            &["production".to_string()],
            Some("/etc/maven/settings.xml"),
            &["-DskipTests".to_string(), "--batch-mode".to_string()],
            &["clean", "deploy"],
            false,
        );
        assert_eq!(
            result,
            "$ ./mvnw -pl backend -P production -s /etc/maven/settings.xml -DskipTests --batch-mode clean deploy"
        );
    }

    #[test]
    fn test_build_command_display_root_module_dot() {
        let result = build_command_display(
            "mvn",
            Some("."),
            &[],
            None,
            &[],
            &["install"],
            false,
        );
        // Root module (.) should not add -pl
        assert_eq!(result, "$ mvn install");
    }
}
