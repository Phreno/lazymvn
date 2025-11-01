//! Launch strategy decision logic

#![allow(dead_code)]

use crate::core::config::LaunchMode;
use super::spring_boot::SpringBootDetection;

/// Launch strategy for running applications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchStrategy {
    SpringBootRun,
    ExecJava,
    #[allow(dead_code)]
    VSCodeJava,
}

/// Decide which launch strategy to use
pub fn decide_launch_strategy(
    detection: &SpringBootDetection,
    launch_mode: LaunchMode,
) -> LaunchStrategy {
    match launch_mode {
        LaunchMode::ForceRun => LaunchStrategy::SpringBootRun,
        LaunchMode::ForceExec => LaunchStrategy::ExecJava,
        LaunchMode::Auto => {
            if detection.should_prefer_spring_boot_run() {
                log::info!(
                    "Auto mode: Spring Boot web app detected (war packaging), strongly preferring spring-boot:run"
                );
                LaunchStrategy::SpringBootRun
            } else if detection.can_use_spring_boot_run() {
                log::info!("Auto mode: Spring Boot plugin detected, using spring-boot:run");
                LaunchStrategy::SpringBootRun
            } else if detection.can_use_exec_java() {
                log::info!(
                    "Auto mode: No Spring Boot plugin or incompatible packaging, using exec:java"
                );
                LaunchStrategy::ExecJava
            } else {
                log::warn!(
                    "Auto mode: No viable launch strategy detected, defaulting to spring-boot:run"
                );
                LaunchStrategy::SpringBootRun
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decide_launch_strategy_force_run() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: Some("com.example.Main".to_string()),
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        let strategy = decide_launch_strategy(&detection, LaunchMode::ForceRun);
        assert_eq!(strategy, LaunchStrategy::SpringBootRun);
    }

    #[test]
    fn test_decide_launch_strategy_force_exec() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        let strategy = decide_launch_strategy(&detection, LaunchMode::ForceExec);
        assert_eq!(strategy, LaunchStrategy::ExecJava);
    }

    #[test]
    fn test_decide_launch_strategy_auto_spring_boot_war() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("war".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        let strategy = decide_launch_strategy(&detection, LaunchMode::Auto);
        assert_eq!(strategy, LaunchStrategy::SpringBootRun);
    }

    #[test]
    fn test_decide_launch_strategy_auto_spring_boot_jar() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: true,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: Some("2.5.0".to_string()),
        };
        let strategy = decide_launch_strategy(&detection, LaunchMode::Auto);
        assert_eq!(strategy, LaunchStrategy::SpringBootRun);
    }

    #[test]
    fn test_decide_launch_strategy_auto_exec_java() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: true,
            main_class: Some("com.example.Main".to_string()),
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        let strategy = decide_launch_strategy(&detection, LaunchMode::Auto);
        assert_eq!(strategy, LaunchStrategy::ExecJava);
    }

    #[test]
    fn test_decide_launch_strategy_auto_fallback() {
        let detection = SpringBootDetection {
            has_spring_boot_plugin: false,
            has_exec_plugin: false,
            main_class: None,
            packaging: Some("jar".to_string()),
            spring_boot_version: None,
        };
        let strategy = decide_launch_strategy(&detection, LaunchMode::Auto);
        assert_eq!(strategy, LaunchStrategy::SpringBootRun);
    }
}
