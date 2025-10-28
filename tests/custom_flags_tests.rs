use std::fs;
use tempfile::tempdir;

use lazymvn::core::config::{Config, CustomFlag, MavenConfig};

#[test]
fn test_load_config_with_custom_flags() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("lazymvn.toml");

    // Create a config with custom flags
    let config_content = r#"
[maven]
custom_flags = [
    { name = "Custom property", flag = "-Dtest.property=value" },
    { name = "Enabled by default", flag = "-Denabled.flag=true", enabled = true },
]
"#;

    fs::write(&config_path, config_content).unwrap();

    // Load the config directly from file
    let content = fs::read_to_string(&config_path).unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    // Verify the custom flags were loaded
    assert!(config.maven.is_some());
    let maven_config = config.maven.unwrap();
    assert_eq!(maven_config.custom_flags.len(), 2);

    // Check first flag
    assert_eq!(maven_config.custom_flags[0].name, "Custom property");
    assert_eq!(maven_config.custom_flags[0].flag, "-Dtest.property=value");
    assert!(!maven_config.custom_flags[0].enabled);

    // Check second flag (enabled by default)
    assert_eq!(maven_config.custom_flags[1].name, "Enabled by default");
    assert_eq!(maven_config.custom_flags[1].flag, "-Denabled.flag=true");
    assert!(maven_config.custom_flags[1].enabled);
}

#[test]
fn test_config_without_custom_flags() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("lazymvn.toml");

    // Create a config without maven section
    let config_content = r#"
notifications_enabled = true
"#;

    fs::write(&config_path, config_content).unwrap();

    // Load the config directly from file
    let content = fs::read_to_string(&config_path).unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    // Verify maven config is None
    assert!(config.maven.is_none());
}

#[test]
fn test_empty_custom_flags() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("lazymvn.toml");

    // Create a config with empty custom_flags array
    let config_content = r#"
[maven]
custom_flags = []
"#;

    fs::write(&config_path, config_content).unwrap();

    // Load the config directly from file
    let content = fs::read_to_string(&config_path).unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    // Verify the custom flags array is empty
    assert!(config.maven.is_some());
    let maven_config = config.maven.unwrap();
    assert_eq!(maven_config.custom_flags.len(), 0);
}

#[test]
fn test_multiple_properties_in_single_flag() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("lazymvn.toml");

    // Create a config with multiple properties in one flag
    let config_content = r#"
[maven]
custom_flags = [
    { name = "Fast build", flag = "-DskipTests -Dmaven.javadoc.skip=true" },
]
"#;

    fs::write(&config_path, config_content).unwrap();

    // Load the config directly from file
    let content = fs::read_to_string(&config_path).unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    // Verify the flag contains multiple properties
    assert!(config.maven.is_some());
    let maven_config = config.maven.unwrap();
    assert_eq!(maven_config.custom_flags.len(), 1);
    assert_eq!(maven_config.custom_flags[0].name, "Fast build");
    assert_eq!(
        maven_config.custom_flags[0].flag,
        "-DskipTests -Dmaven.javadoc.skip=true"
    );
}

#[test]
fn test_custom_flag_defaults() {
    // Create a CustomFlag with default enabled value
    let flag = CustomFlag {
        name: "Test flag".to_string(),
        flag: "-Dtest=true".to_string(),
        enabled: false, // This is the default
    };

    assert!(!flag.enabled);

    // Create a MavenConfig with default values
    let maven_config = MavenConfig::default();
    assert_eq!(maven_config.custom_flags.len(), 0);
}
