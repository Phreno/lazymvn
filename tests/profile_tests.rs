// Maven profile management tests
use lazymvn::maven::get_profile_xml;
use std::fs;
use tempfile::tempdir;

mod common;

#[test]
fn test_get_profile_xml() {
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a POM with a profile
    let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
<modelVersion>4.0.0</modelVersion>
<groupId>com.example</groupId>
<artifactId>test-project</artifactId>
<version>1.0.0</version>

<profiles>
    <profile>
        <id>dev</id>
        <activation>
            <activeByDefault>true</activeByDefault>
        </activation>
        <properties>
            <env>development</env>
        </properties>
    </profile>
    <profile>
        <id>prod</id>
        <properties>
            <env>production</env>
        </properties>
    </profile>
</profiles>
</project>"#;

    fs::write(project_root.join("pom.xml"), pom_content).unwrap();

    // Test extracting the dev profile
    let result = get_profile_xml(project_root, "dev");
    assert!(result.is_some(), "Should find dev profile");

    let (xml, _path) = result.unwrap();
    assert!(
        xml.contains("<id>dev</id>"),
        "XML should contain profile ID"
    );
    assert!(
        xml.contains("<env>development</env>"),
        "XML should contain profile properties"
    );
    assert!(xml.contains("<profile>"), "XML should have opening tag");
    assert!(xml.contains("</profile>"), "XML should have closing tag");

    // Test extracting the prod profile
    let result = get_profile_xml(project_root, "prod");
    assert!(result.is_some(), "Should find prod profile");

    let (xml, _path) = result.unwrap();
    assert!(
        xml.contains("<id>prod</id>"),
        "XML should contain prod profile ID"
    );
    assert!(
        xml.contains("<env>production</env>"),
        "XML should contain prod properties"
    );

    // Test non-existent profile
    let result = get_profile_xml(project_root, "nonexistent");
    assert!(result.is_none(), "Should not find nonexistent profile");
}

#[test]
fn test_get_profile_xml_from_settings() {
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a settings.xml with profiles
    let settings_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
<profiles>
    <profile>
        <id>corporate-proxy</id>
        <properties>
            <http.proxyHost>proxy.corp.com</http.proxyHost>
            <http.proxyPort>8080</http.proxyPort>
        </properties>
    </profile>
    <profile>
        <id>development</id>
        <activation>
            <activeByDefault>false</activeByDefault>
        </activation>
        <properties>
            <maven.compiler.debug>true</maven.compiler.debug>
            <env>dev</env>
        </properties>
    </profile>
</profiles>
</settings>"#;

    fs::write(project_root.join("settings.xml"), settings_content).unwrap();

    // Also create a POM to ensure we search settings.xml first
    let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
<modelVersion>4.0.0</modelVersion>
<groupId>com.example</groupId>
<artifactId>test</artifactId>
<version>1.0.0</version>
</project>"#;
    fs::write(project_root.join("pom.xml"), pom_content).unwrap();

    // Test finding profile from settings.xml
    let result = get_profile_xml(project_root, "development");
    assert!(
        result.is_some(),
        "Should find development profile from settings.xml"
    );

    let (xml, path) = result.unwrap();
    assert!(
        xml.contains("<id>development</id>"),
        "XML should contain development profile"
    );
    assert!(
        xml.contains("<env>dev</env>"),
        "XML should contain settings profile properties"
    );
    assert!(
        path.ends_with("settings.xml"),
        "Should be from settings.xml"
    );

    // Test corporate proxy profile
    let result = get_profile_xml(project_root, "corporate-proxy");
    assert!(result.is_some(), "Should find corporate-proxy profile");

    let (xml, _) = result.unwrap();
    assert!(
        xml.contains("proxy.corp.com"),
        "XML should contain proxy settings"
    );
}

#[test]
fn test_get_profile_xml_with_maven_settings_xml() {
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create maven_settings.xml (note: not settings.xml)
    let maven_settings = project_root.join("maven_settings.xml");
    fs::write(
        &maven_settings,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
<profiles>
    <profile>
        <id>custom-profile</id>
        <properties>
            <custom.property>custom-value</custom.property>
        </properties>
    </profile>
</profiles>
</settings>"#,
    )
    .unwrap();

    // Create config.toml to point to maven_settings.xml (using centralized location)
    let config_content = format!("maven_settings = \"{}\"", maven_settings.to_str().unwrap());
    match lazymvn::core::config::create_project_config(project_root) {
        Ok(config_path) => {
            // Overwrite with test config that points to maven_settings.xml
            fs::write(&config_path, config_content).unwrap();
        }
        Err(e) => {
            panic!("Failed to create test config: {}", e);
        }
    }

    // Also need a pom.xml so it's a valid Maven project
    let pom = project_root.join("pom.xml");
    fs::write(&pom, "<project></project>").unwrap();

    // Test finding profile from maven_settings.xml
    let result = get_profile_xml(project_root, "custom-profile");
    assert!(
        result.is_some(),
        "Should find custom-profile from maven_settings.xml"
    );

    let (xml, path) = result.unwrap();
    assert!(
        xml.contains("<id>custom-profile</id>"),
        "XML should contain profile ID"
    );
    assert!(
        xml.contains("custom-value"),
        "XML should contain custom property"
    );
    assert_eq!(
        path, maven_settings,
        "Should return maven_settings.xml path"
    );
}
