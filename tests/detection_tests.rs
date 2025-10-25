// Spring Boot detection and launch strategy tests
use lazymvn::core::config::LaunchMode;
use lazymvn::maven::{
    decide_launch_strategy, extract_tag_content, LaunchStrategy, SpringBootDetection,
};

mod common;

#[test]
fn test_spring_boot_detection_with_plugin() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: false,
        main_class: None,
        packaging: Some("jar".to_string()),
    };

    assert!(
        detection.can_use_spring_boot_run(),
        "Should be able to use spring-boot:run with plugin and jar packaging"
    );
}

#[test]
fn test_spring_boot_detection_with_war_packaging() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: false,
        main_class: None,
        packaging: Some("war".to_string()),
    };

    assert!(
        detection.can_use_spring_boot_run(),
        "Should be able to use spring-boot:run with plugin and war packaging"
    );
}

#[test]
fn test_spring_boot_detection_with_pom_packaging() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: false,
        main_class: None,
        packaging: Some("pom".to_string()),
    };

    assert!(
        !detection.can_use_spring_boot_run(),
        "Should not be able to use spring-boot:run with pom packaging"
    );
}

#[test]
fn test_spring_boot_detection_fallback_to_exec() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: true,
        main_class: Some("com.example.App".to_string()),
        packaging: Some("jar".to_string()),
    };

    assert!(
        !detection.can_use_spring_boot_run(),
        "Should not use spring-boot:run without plugin"
    );
    assert!(
        detection.can_use_exec_java(),
        "Should be able to use exec:java with exec plugin"
    );
}

#[test]
fn test_launch_strategy_auto_prefers_spring_boot() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: true,
        main_class: Some("com.example.App".to_string()),
        packaging: Some("jar".to_string()),
    };

    let strategy = decide_launch_strategy(&detection, LaunchMode::Auto);
    assert_eq!(
        strategy,
        LaunchStrategy::SpringBootRun,
        "Auto mode should prefer spring-boot:run when available"
    );
}

#[test]
fn test_launch_strategy_auto_falls_back_to_exec() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: true,
        main_class: Some("com.example.App".to_string()),
        packaging: Some("jar".to_string()),
    };

    let strategy = decide_launch_strategy(&detection, LaunchMode::Auto);
    assert_eq!(
        strategy,
        LaunchStrategy::ExecJava,
        "Auto mode should fall back to exec:java when spring-boot:run not available"
    );
}

#[test]
fn test_launch_strategy_force_run() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: false,
        has_exec_plugin: true,
        main_class: Some("com.example.App".to_string()),
        packaging: Some("jar".to_string()),
    };

    let strategy = decide_launch_strategy(&detection, LaunchMode::ForceRun);
    assert_eq!(
        strategy,
        LaunchStrategy::SpringBootRun,
        "ForceRun should always use spring-boot:run"
    );
}

#[test]
fn test_launch_strategy_force_exec() {
    let detection = SpringBootDetection {
        has_spring_boot_plugin: true,
        has_exec_plugin: false,
        main_class: None,
        packaging: Some("jar".to_string()),
    };

    let strategy = decide_launch_strategy(&detection, LaunchMode::ForceExec);
    assert_eq!(
        strategy,
        LaunchStrategy::ExecJava,
        "ForceExec should always use exec:java"
    );
}

#[test]
fn test_extract_tag_content() {
    let line = "<mainClass>com.example.Application</mainClass>";
    let content = extract_tag_content(line, "mainClass");
    assert_eq!(content, Some("com.example.Application".to_string()));

    let line_with_spaces = "  <packaging>jar</packaging>  ";
    let content = extract_tag_content(line_with_spaces, "packaging");
    assert_eq!(content, Some("jar".to_string()));

    let invalid_line = "<mainClass>incomplete";
    let content = extract_tag_content(invalid_line, "mainClass");
    assert_eq!(content, None);
}
