//! Maven profile extraction and management

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

/// Get all available Maven profiles from POM and settings.xml
pub fn get_profiles(project_root: &Path) -> Result<Vec<String>, std::io::Error> {
    log::debug!(
        "get_profiles: Fetching Maven profiles from {:?}",
        project_root
    );
    // Try to load config and use settings if available
    let config = crate::config::load_config(project_root);

    // Run without -N flag to include profiles from all modules
    let output = super::command::execute_maven_command(
        project_root,
        None,
        &["help:all-profiles"],
        &[],
        config.maven_settings.as_deref(),
        &[],
    )?;

    // Use a HashSet to deduplicate profiles as they may appear multiple times
    // (once per module that inherits or defines them)
    let mut profile_set = HashSet::new();

    // Get profiles from Maven command output (POM files)
    for line in output.iter() {
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
                    log::debug!("Found profile from POM: {}", profile_name);
                    profile_set.insert(profile_name.to_string());
                }
            }
        }
    }

    // Also get profiles from settings.xml (Maven's help:all-profiles doesn't include these)
    if let Some(settings_path) = config.maven_settings.as_ref() {
        log::debug!("Checking settings.xml for profiles: {}", settings_path);
        if let Ok(settings_content) = fs::read_to_string(settings_path)
            && let Ok(profiles_from_settings) =
                extract_profiles_from_settings_xml(&settings_content)
        {
            for profile_name in profiles_from_settings {
                log::debug!("Found profile from settings.xml: {}", profile_name);
                profile_set.insert(profile_name);
            }
        }
    }

    // Convert to sorted Vec for consistent ordering
    let mut profiles: Vec<String> = profile_set.into_iter().collect();
    profiles.sort();

    log::info!("Discovered {} unique Maven profiles", profiles.len());
    Ok(profiles)
}

/// Get profiles that are currently auto-activated by Maven
/// These are profiles activated by conditions like file existence, JDK version, OS, etc.
pub fn get_active_profiles(project_root: &Path) -> Result<Vec<String>, std::io::Error> {
    log::debug!(
        "get_active_profiles: Fetching auto-activated Maven profiles from {:?}",
        project_root
    );

    let config = crate::config::load_config(project_root);
    let output = super::command::execute_maven_command(
        project_root,
        None,
        &["help:active-profiles"],
        &[],
        config.maven_settings.as_deref(),
        &[],
    )?;

    let mut active_profiles = HashSet::new();

    // Parse output looking for profile names after "- " lines
    for line in output.iter() {
        let trimmed = line.trim();
        // Lines with active profiles look like: " - dev (source: ...)"
        if let Some(stripped) = trimmed.strip_prefix("- ") {
            let parts: Vec<&str> = stripped.split_whitespace().collect();
            if let Some(profile_name) = parts.first() {
                log::debug!("Found active profile: {}", profile_name);
                active_profiles.insert(profile_name.to_string());
            }
        }
    }

    let mut profiles: Vec<String> = active_profiles.into_iter().collect();
    profiles.sort();

    log::info!("Discovered {} auto-activated profiles", profiles.len());
    Ok(profiles)
}

/// Extract the XML snippet for a specific profile from POM files
/// Returns (profile_xml, source_pom_path) or None if not found
pub fn get_profile_xml(project_root: &Path, profile_id: &str) -> Option<(String, PathBuf)> {
    log::debug!(
        "Searching for profile '{}' XML in {:?}",
        profile_id,
        project_root
    );

    let mut pom_paths = Vec::new();

    // Load config to get the maven_settings path (which may be maven_settings.xml or settings.xml)
    let config = crate::config::load_config(project_root);

    // 1. If config has maven_settings configured, use that
    if let Some(ref settings_path) = config.maven_settings {
        let settings = PathBuf::from(settings_path);
        if settings.exists() {
            log::debug!("Using configured Maven settings: {:?}", settings);
            pom_paths.push(settings);
        }
    }

    // 2. Also check user settings.xml (~/.m2/settings.xml) if not already added
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let user_settings = PathBuf::from(home).join(".m2").join("settings.xml");
        if user_settings.exists() && !pom_paths.contains(&user_settings) {
            pom_paths.push(user_settings);
        }
    }

    // 3. Check project root pom.xml
    pom_paths.push(project_root.join("pom.xml"));

    // 4. Check module POMs
    if let Ok(entries) = fs::read_dir(project_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let module_pom = path.join("pom.xml");
                if module_pom.exists() {
                    pom_paths.push(module_pom);
                }
            }
        }
    }

    // Search each file
    for pom_path in pom_paths {
        if let Ok(content) = fs::read_to_string(&pom_path)
            && let Some(xml) = extract_profile_from_xml(&content, profile_id)
        {
            log::info!("Found profile '{}' in {:?}", profile_id, pom_path);
            // Prettify the XML before returning
            let prettified = prettify_xml(&xml).unwrap_or(xml);
            return Some((prettified, pom_path));
        }
    }

    log::warn!(
        "Profile '{}' not found in any POM or settings file",
        profile_id
    );
    None
}

/// Extract profile IDs from settings.xml content
pub(crate) fn extract_profiles_from_settings_xml(xml_content: &str) -> Result<Vec<String>, String> {
    let mut profiles = Vec::new();
    let lines: Vec<&str> = xml_content.lines().collect();

    let mut in_profiles_section = false;
    let mut in_profile = false;

    for line in lines {
        let trimmed = line.trim();

        // Check if we're entering the <profiles> section
        if trimmed.starts_with("<profiles>") {
            in_profiles_section = true;
            continue;
        }

        // Check if we're leaving the <profiles> section
        if trimmed.starts_with("</profiles>") {
            in_profiles_section = false;
            continue;
        }

        if in_profiles_section {
            // Check if we're entering a <profile>
            if trimmed.starts_with("<profile>") {
                in_profile = true;
                continue;
            }

            // Check if we're leaving a <profile>
            if trimmed.starts_with("</profile>") {
                in_profile = false;
                continue;
            }

            // If we're in a profile, look for <id>
            if in_profile
                && trimmed.starts_with("<id>")
                && trimmed.contains("</id>")
                && let Some(id_start) = trimmed.find("<id>")
                && let Some(id_end) = trimmed.find("</id>")
            {
                let id = &trimmed[id_start + 4..id_end];
                profiles.push(id.to_string());
            }
        }
    }

    Ok(profiles)
}

/// Extract a single profile XML block from POM content
fn extract_profile_from_xml(xml_content: &str, profile_id: &str) -> Option<String> {
    // Find the profile block with the matching ID
    // Look for <profile> ... <id>profile_id</id> ... </profile>

    let mut in_profile = false;
    let mut in_profile_id = false;
    let mut current_profile = String::new();
    let mut depth = 0;
    let mut found_matching_id = false;

    for line in xml_content.lines() {
        let trimmed = line.trim();

        // Track when we enter a <profile> tag
        if trimmed.starts_with("<profile>") || trimmed.starts_with("<profile ") {
            in_profile = true;
            current_profile.clear();
            found_matching_id = false;
            depth = 0;
        }

        if in_profile {
            current_profile.push_str(line);
            current_profile.push('\n');

            // Track depth to handle nested tags
            if trimmed.contains("<profile>") {
                depth += 1;
            }

            // Check if we're in the <id> tag
            if trimmed.starts_with("<id>") {
                in_profile_id = true;
                if trimmed.contains(profile_id) && trimmed.contains("</id>") {
                    found_matching_id = true;
                }
            }

            if in_profile_id && trimmed.contains("</id>") {
                in_profile_id = false;
            }

            // Check if we've closed the profile tag
            if trimmed.contains("</profile>") {
                depth -= 1;
                if depth == 0 {
                    in_profile = false;

                    // If this was the matching profile, return it
                    if found_matching_id {
                        // Clean up the XML - preserve indentation
                        return Some(current_profile.trim_end().to_string());
                    }
                }
            }
        }
    }

    None
}

/// Prettify XML with proper indentation
fn prettify_xml(xml: &str) -> Option<String> {
    use std::io::Cursor;

    // Try to parse and reformat the XML
    match xmltree::Element::parse(Cursor::new(xml.as_bytes())) {
        Ok(element) => {
            let mut output = Vec::new();
            let config = xmltree::EmitterConfig::new()
                .perform_indent(true)
                .indent_string("    ");

            if element.write_with_config(&mut output, config).is_ok() {
                String::from_utf8(output).ok()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
