//! Module preferences save/load operations

use super::project_tab::ProjectTab;
use super::types::{MavenProfile, ProfileState};
use crate::core::config::ModulePreferences;

/// Save module preferences (profiles and flags) for the currently selected module
pub fn save_module_preferences(
    tab: &mut ProjectTab,
    module: Option<&str>,
    enabled_flags: Vec<String>,
) {
    if let Some(module) = module {
        // Save only explicitly set profiles (not Default state)
        let explicit_profiles: Vec<String> = tab
            .profiles
            .iter()
            .filter_map(|p| match p.state {
                ProfileState::ExplicitlyEnabled => Some(p.name.clone()),
                ProfileState::ExplicitlyDisabled => Some(format!("!{}", p.name)),
                ProfileState::Default => None,
            })
            .collect();

        let prefs = ModulePreferences {
            active_profiles: explicit_profiles.clone(),
            enabled_flags,
        };

        log::info!(
            "Saving preferences for module '{}': profiles={:?}, flags={:?}",
            module,
            prefs.active_profiles,
            prefs.enabled_flags
        );

        tab.module_preferences
            .set_module_prefs(module.to_string(), prefs);

        if let Err(e) = tab.module_preferences.save(&tab.project_root) {
            log::error!("Failed to save module preferences: {}", e);
        }
    }
}

/// Load preferences (profiles and flags) for the specified module
pub fn load_module_preferences(tab: &mut ProjectTab, module: Option<&str>) {
    if let Some(module) = module {
        if let Some(prefs) = tab.module_preferences.get_module_prefs(module) {
            log::info!(
                "Loading preferences for module '{}': profiles={:?}, flags={:?}",
                module,
                prefs.active_profiles,
                prefs.enabled_flags
            );

            // Restore profile states
            restore_profile_states(&mut tab.profiles, prefs);

            // Restore enabled flags
            for flag in &mut tab.flags {
                flag.enabled = prefs.enabled_flags.contains(&flag.flag);
            }
        } else {
            log::debug!("No saved preferences for module '{}'", module);
            // Reset all profiles to Default state
            for profile in &mut tab.profiles {
                profile.state = ProfileState::Default;
            }
        }
    }
}

fn restore_profile_states(profiles: &mut [MavenProfile], prefs: &ModulePreferences) {
    for profile in profiles {
        // Check if profile is explicitly enabled or disabled
        let disabled_name = format!("!{}", profile.name);

        if prefs.active_profiles.contains(&profile.name) {
            profile.state = ProfileState::ExplicitlyEnabled;
            log::debug!("Restored profile '{}' as ExplicitlyEnabled", profile.name);
        } else if prefs.active_profiles.contains(&disabled_name) {
            profile.state = ProfileState::ExplicitlyDisabled;
            log::debug!("Restored profile '{}' as ExplicitlyDisabled", profile.name);
        } else {
            profile.state = ProfileState::Default;
            log::debug!("Profile '{}' in Default state", profile.name);
        }
    }
}
