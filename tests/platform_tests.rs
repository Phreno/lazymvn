// Platform-specific argument quoting tests
use lazymvn::maven::{extract_profiles_from_settings_xml, quote_arg_for_platform};

mod common;

#[test]
#[cfg(windows)]
fn test_quote_arg_for_platform_windows() {
    assert_eq!(
        quote_arg_for_platform("-Dfoo=bar"),
        "\"-Dfoo=bar\"",
        "Should quote -D args on Windows"
    );
    assert_eq!(
        quote_arg_for_platform("spring-boot:run"),
        "spring-boot:run",
        "Should not quote goals"
    );
}

#[test]
#[cfg(not(windows))]
fn test_quote_arg_for_platform_unix() {
    assert_eq!(
        quote_arg_for_platform("-Dfoo=bar"),
        "-Dfoo=bar",
        "Should not quote on Unix"
    );
}

#[test]
fn test_extract_profiles_from_settings_xml() {
    let settings_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
<profiles>
    <profile>
        <id>development</id>
        <properties>
            <env>dev</env>
        </properties>
    </profile>
    <profile>
        <id>production</id>
        <properties>
            <env>prod</env>
        </properties>
    </profile>
    <profile>
        <id>testing</id>
        <properties>
            <env>test</env>
        </properties>
    </profile>
</profiles>
</settings>"#;

    let profiles = extract_profiles_from_settings_xml(settings_xml).unwrap();
    assert_eq!(profiles.len(), 3, "Should find 3 profiles");
    assert!(profiles.contains(&"development".to_string()));
    assert!(profiles.contains(&"production".to_string()));
    assert!(profiles.contains(&"testing".to_string()));
}
