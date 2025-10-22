// Maven module tests
use crate::config::LaunchMode;
use crate::maven::command::{
    execute_maven_command, execute_maven_command_with_options, get_maven_command,
};
use crate::maven::detection::{SpringBootDetection, extract_tag_content, quote_arg_for_platform};
use crate::maven::profiles::extract_profiles_from_settings_xml;
use crate::maven::*;
use crate::utils;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tempfile::tempdir;

fn write_script(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }
    // On Windows, batch files (.bat, .cmd) are executable by default
    // For tests, we create both the script and a .bat version
    #[cfg(windows)]
    {
        // Create a .bat file for Windows
        let bat_path = path.with_extension("bat");
        // Convert basic shell echo to batch echo
        let bat_content = content
            .replace("#!/bin/sh\n", "")
            .replace("echo $@", "echo %*");
        fs::write(&bat_path, bat_content).unwrap();
    }
}

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn get_maven_command_returns_mvnw_if_present() {
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Test with mvnw present
    #[cfg(unix)]
    {
        let mvnw_path = project_root.join("mvnw");
        fs::File::create(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "./mvnw");
        std::fs::remove_file(&mvnw_path).unwrap();
    }

    #[cfg(windows)]
    {
        let mvnw_path = project_root.join("mvnw.bat");
        fs::File::create(&mvnw_path).unwrap();
        assert_eq!(get_maven_command(project_root), "mvnw.bat");
        std::fs::remove_file(&mvnw_path).unwrap();
    }

    // Test without mvnw present
    #[cfg(windows)]
    {
        assert_eq!(get_maven_command(project_root), "mvn.cmd");
    }
    #[cfg(not(windows))]
    {
        assert_eq!(get_maven_command(project_root), "mvn");
    }
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_captures_output() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a mock mvnw script
    let mvnw_path = project_root.join("mvnw");
    write_script(&mvnw_path, "#!/bin/sh\necho 'line 1'\necho 'line 2'\n");

    let output: Vec<String> = execute_maven_command(project_root, None, &["test"], &[], None, &[])
        .unwrap()
        .iter()
        .filter_map(|line| utils::clean_log_line(line))
        .collect();

    // Output now includes command line at the start
    // Skip the command line to check actual Maven output
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();
    assert_eq!(maven_output, vec!["line 1", "line 2"]);
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_captures_stderr() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    write_script(
        &mvnw_path,
        "#!/bin/sh\necho 'line 1'\n>&2 echo 'warn message'\n",
    );

    let output: Vec<String> = execute_maven_command(project_root, None, &["test"], &[], None, &[])
        .unwrap()
        .iter()
        .filter_map(|line| utils::clean_log_line(line))
        .collect();

    // Skip command line header
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();
    assert!(
        maven_output.contains(&"line 1".to_string()),
        "stdout line should be present"
    );
    assert!(
        maven_output.contains(&"[ERR] warn message".to_string()),
        "stderr line should be tagged"
    );
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_with_profiles() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a mock mvnw script
    let mvnw_path = project_root.join("mvnw");
    write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let profiles = vec!["p1".to_string(), "p2".to_string()];
    let output: Vec<String> =
        execute_maven_command(project_root, None, &["test"], &profiles, None, &[])
            .unwrap()
            .iter()
            .filter_map(|line| utils::clean_log_line(line))
            .collect();

    // Skip command line header and check actual Maven output
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();
    assert_eq!(maven_output, vec!["-P p1,p2 test"]);
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn test_get_profiles() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a mock mvnw script that simulates Maven's help:all-profiles output
    let mvnw_path = project_root.join("mvnw");
    write_script(
        &mvnw_path,
        "#!/bin/sh\necho '  Profile Id: profile-1 (Active: false, Source: pom)'\necho '  Profile Id: profile-2 (Active: true, Source: pom)'\n",
    );

    let profiles = get_profiles(project_root).unwrap();
    assert_eq!(profiles, vec!["profile-1", "profile-2"]);
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn test_get_profiles_deduplicates_and_sorts() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    // Create a mock mvnw script that simulates Maven's help:all-profiles output
    // with duplicates (as would happen in multi-module projects without -N)
    let mvnw_path = project_root.join("mvnw");
    write_script(
        &mvnw_path,
        "#!/bin/sh\necho '  Profile Id: profile-2 (Active: false, Source: pom)'\necho '  Profile Id: profile-1 (Active: false, Source: pom)'\necho '  Profile Id: profile-2 (Active: false, Source: pom)'\necho '  Profile Id: child-profile (Active: false, Source: pom)'\n",
    );

    let profiles = get_profiles(project_root).unwrap();
    // Should be deduplicated and sorted
    assert_eq!(profiles, vec!["child-profile", "profile-1", "profile-2"]);
}

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

    // Create lazymvn.toml to point to maven_settings.xml
    let config_file = project_root.join("lazymvn.toml");
    fs::write(
        &config_file,
        format!("maven_settings = \"{}\"", maven_settings.to_str().unwrap()),
    )
    .unwrap();

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

#[test]
fn test_build_launch_command_spring_boot_run() {
    let profiles = vec!["dev".to_string(), "debug".to_string()];
    let jvm_args = vec!["-Dfoo=bar".to_string(), "-Xmx512m".to_string()];

    let command = build_launch_command(
        LaunchStrategy::SpringBootRun,
        None,
        &profiles,
        &jvm_args,
        None,
    );

    // Should contain profiles argument
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("spring-boot.run.profiles=dev,debug")),
        "Should set profiles: {:?}",
        command
    );

    // Should contain JVM arguments
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("spring-boot.run.jvmArguments")),
        "Should set jvmArguments: {:?}",
        command
    );

    // Should end with the goal
    assert_eq!(command.last(), Some(&"spring-boot:run".to_string()));
}

#[test]
fn test_build_launch_command_exec_java() {
    let jvm_args = vec!["-Dfoo=bar".to_string()];

    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("com.example.Application"),
        &[],
        &jvm_args,
        None,
    );

    // Should contain mainClass argument
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.mainClass=com.example.Application")),
        "Should set mainClass: {:?}",
        command
    );

    // Should contain JVM args
    assert!(
        command.contains(&quote_arg_for_platform("-Dfoo=bar")),
        "Should include JVM args: {:?}",
        command
    );

    // Should contain cleanup daemon threads flag
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.cleanupDaemonThreads=false")),
        "Should include cleanupDaemonThreads flag: {:?}",
        command
    );

    // Should end with the goal
    assert_eq!(command.last(), Some(&"exec:java".to_string()));
}

#[test]
fn test_build_launch_command_exec_java_without_main_class() {
    let command = build_launch_command(LaunchStrategy::ExecJava, None, &[], &[], None);

    // Should not contain mainClass if not provided
    assert!(
        !command.iter().any(|arg| arg.contains("exec.mainClass")),
        "Should not set mainClass if none provided: {:?}",
        command
    );

    // Should still have the goal
    assert_eq!(command.last(), Some(&"exec:java".to_string()));
}

#[test]
fn test_build_launch_command_exec_java_war_packaging() {
    // Test that WAR packaging adds classpathScope=compile
    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("com.example.WarApplication"),
        &[],
        &[],
        Some("war"),
    );

    // Should contain mainClass argument
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.mainClass=com.example.WarApplication")),
        "Should set mainClass: {:?}",
        command
    );

    // Should contain classpathScope=compile for WAR packaging
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.classpathScope=compile")),
        "Should include classpathScope=compile for WAR packaging: {:?}",
        command
    );

    // Should contain cleanup daemon threads flag
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.cleanupDaemonThreads=false")),
        "Should include cleanupDaemonThreads flag: {:?}",
        command
    );

    // Should end with the goal
    assert_eq!(command.last(), Some(&"exec:java".to_string()));
}

#[test]
fn test_build_launch_command_exec_java_jar_packaging() {
    // Test that JAR packaging does NOT add classpathScope=compile
    let command = build_launch_command(
        LaunchStrategy::ExecJava,
        Some("com.example.JarApplication"),
        &[],
        &[],
        Some("jar"),
    );

    // Should contain mainClass argument
    assert!(
        command
            .iter()
            .any(|arg| arg.contains("exec.mainClass=com.example.JarApplication")),
        "Should set mainClass: {:?}",
        command
    );

    // Should NOT contain classpathScope for JAR packaging
    assert!(
        !command
            .iter()
            .any(|arg| arg.contains("exec.classpathScope")),
        "Should NOT include classpathScope for JAR packaging: {:?}",
        command
    );

    // Should end with the goal
    assert_eq!(command.last(), Some(&"exec:java".to_string()));
}

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

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_scopes_to_module() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let output: Vec<String> =
        execute_maven_command(project_root, Some("module-a"), &["test"], &[], None, &[])
            .unwrap()
            .iter()
            .filter_map(|line| utils::clean_log_line(line))
            .collect();

    // Skip command line header
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();
    assert_eq!(maven_output, vec!["-pl module-a test"]);
}

#[test]
#[cfg(unix)] // Shell script execution not supported on Windows
fn execute_maven_command_without_pl_for_root_module() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let output: Vec<String> =
        execute_maven_command(project_root, Some("."), &["test"], &[], None, &[])
            .unwrap()
            .iter()
            .filter_map(|line| utils::clean_log_line(line))
            .collect();

    // Skip command line header
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();
    assert_eq!(maven_output, vec!["test"]);
}

#[test]
#[cfg(unix)]
fn test_exec_java_with_file_flag_adds_also_make() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let output: Vec<String> = execute_maven_command_with_options(
        project_root,
        Some("my-module"),
        &["exec:java"],
        &[],
        None,
        &[],
        true, // use_file_flag = true
        None, // no logging config
    )
    .unwrap()
    .iter()
    .filter_map(|line| utils::clean_log_line(line))
    .collect();

    // Skip command line header
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();

    // Should contain -f flag, --also-make, and exec:java
    let command_output = maven_output.join(" ");
    assert!(command_output.contains("-f"));
    assert!(command_output.contains("--also-make"));
    assert!(command_output.contains("exec:java"));
}

#[test]
#[cfg(unix)]
fn test_exec_java_with_file_flag_preserves_existing_also_make() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    write_script(&mvnw_path, "#!/bin/sh\necho $@\n");

    let flags = vec!["--also-make-dependents".to_string()];
    let output: Vec<String> = execute_maven_command_with_options(
        project_root,
        Some("my-module"),
        &["exec:java"],
        &[],
        None,
        &flags,
        true, // use_file_flag = true
        None, // no logging config
    )
    .unwrap()
    .iter()
    .filter_map(|line| utils::clean_log_line(line))
    .collect();

    // Skip command line header
    let maven_output: Vec<String> = output
        .iter()
        .skip_while(|line| line.starts_with("$ "))
        .cloned()
        .collect();

    let command_output = maven_output.join(" ");
    // Should contain existing flag but not auto-add --also-make
    assert!(command_output.contains("--also-make-dependents"));
    // Should have only one occurrence of "also-make" (from the existing flag)
    assert_eq!(command_output.matches("also-make").count(), 1);
}

#[test]
#[cfg(unix)]
fn test_command_display_in_output() {
    let _guard = test_lock().lock().unwrap();
    let dir = tempdir().unwrap();
    let project_root = dir.path();

    let mvnw_path = project_root.join("mvnw");
    write_script(&mvnw_path, "#!/bin/sh\necho 'test output'\n");

    let profiles = vec!["dev".to_string()];
    let flags = vec!["--offline".to_string()];
    let output = execute_maven_command(
        project_root,
        Some("my-module"),
        &["clean", "install"],
        &profiles,
        None,
        &flags,
    )
    .unwrap();

    // First line should be the command
    assert!(
        output[0].starts_with("$ ./mvnw"),
        "First line should be the command: {}",
        output[0]
    );
    assert!(
        output[0].contains("-P dev"),
        "Command should include profiles: {}",
        output[0]
    );
    assert!(
        output[0].contains("-pl my-module"),
        "Command should include module: {}",
        output[0]
    );
    assert!(
        output[0].contains("--offline"),
        "Command should include flags: {}",
        output[0]
    );
    assert!(
        output[0].contains("clean install"),
        "Command should include goals: {}",
        output[0]
    );
}
