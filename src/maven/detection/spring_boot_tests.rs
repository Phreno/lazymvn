//! Tests for Spring Boot detection

#[cfg(test)]
mod tests {
    use super::super::spring_boot::*;

    #[test]
    fn test_can_use_spring_boot_run_with_plugin_and_jar() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(detection.can_use_spring_boot_run());
    }

    #[test]
    fn test_can_use_spring_boot_run_with_plugin_and_war() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("war".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(detection.can_use_spring_boot_run());
    }

    #[test]
    fn test_can_use_spring_boot_run_without_plugin() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: Some("com.example.Main".to_string()),
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        assert!(!detection.can_use_spring_boot_run());
    }

    #[test]
    fn test_can_use_spring_boot_run_with_pom_packaging() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("pom".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(!detection.can_use_spring_boot_run());
    }

    #[test]
    fn test_should_prefer_spring_boot_run_war() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("war".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(detection.should_prefer_spring_boot_run());
    }

    #[test]
    fn test_should_prefer_spring_boot_run_jar() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        assert!(!detection.should_prefer_spring_boot_run());
    }

    #[test]
    fn test_can_use_exec_java_with_plugin() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        assert!(detection.can_use_exec_java());
    }

    #[test]
    fn test_can_use_exec_java_with_main_class() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: Some("com.example.App".to_string()),
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        assert!(detection.can_use_exec_java());
    }

    #[test]
    fn test_can_use_exec_java_neither() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        assert!(!detection.can_use_exec_java());
    }

    #[test]
    fn test_detect_packaging() {
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        
        detect_packaging("<packaging>war</packaging>", &mut detection);
        assert_eq!(detection.packaging, Some("war".to_string()));
    }

    #[test]
    fn test_detect_packaging_jar() {
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        
        detect_packaging("<packaging>jar</packaging>", &mut detection);
        assert_eq!(detection.packaging, Some("jar".to_string()));
    }

    #[test]
    fn test_track_plugin_state() {
        let mut in_plugin = false;
        let mut in_configuration = false;
        
        track_plugin_state("<plugin>", &mut in_plugin, &mut in_configuration);
        assert!(in_plugin);
        
        track_plugin_state("</plugin>", &mut in_plugin, &mut in_configuration);
        assert!(!in_plugin);
        assert!(!in_configuration);
    }

    #[test]
    fn test_detect_plugins_spring_boot() {
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        let mut artifact_id = String::new();
        let mut in_configuration = false;
        
        detect_plugins(
            "<artifactId>spring-boot-maven-plugin</artifactId>",
            &mut artifact_id,
            &mut in_configuration,
            &mut detection,
        );
        
        assert!(detection.has_spring_boot_plugin);
        assert_eq!(artifact_id, "spring-boot-maven-plugin");
    }

    #[test]
    fn test_detect_plugins_exec() {
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        let mut artifact_id = String::new();
        let mut in_configuration = false;
        
        detect_plugins(
            "<artifactId>exec-maven-plugin</artifactId>",
            &mut artifact_id,
            &mut in_configuration,
            &mut detection,
        );
        
        assert!(detection.has_exec_plugin);
        assert_eq!(artifact_id, "exec-maven-plugin");
    }

    #[test]
    fn test_parse_effective_pom_complete() {
        let pom = r#"
            <project>
                <packaging>jar</packaging>
                <plugins>
                    <plugin>
                        <artifactId>spring-boot-maven-plugin</artifactId>
                        <version>2.7.0</version>
                        <configuration>
                            <mainClass>com.example.Application</mainClass>
                        </configuration>
                    </plugin>
                </plugins>
            </project>
        "#;
        
        let mut detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: None,
            spring_boot_version: None,
        };
        
        parse_effective_pom(pom, &mut detection);
        
        assert!(detection.has_spring_boot_plugin);
        assert_eq!(detection.packaging, Some("jar".to_string()));
        assert_eq!(detection.spring_boot_version, Some("2.7.0".to_string()));
    }
}
