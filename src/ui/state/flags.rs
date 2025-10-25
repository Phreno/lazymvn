//! Build flags management
//!
//! This module handles toggling and managing Maven build flags.

use super::{TuiState, Focus};

impl TuiState {
    /// Toggle the selected build flag
    pub fn toggle_flag(&mut self) {
        if self.focus != Focus::Flags {
            return;
        }
        let tab = self.get_active_tab_mut();
        if let Some(selected) = tab.flags_list_state.selected()
            && let Some(flag) = tab.flags.get_mut(selected)
        {
            flag.enabled = !flag.enabled;
            log::info!(
                "Toggled flag '{}' ({}): {}",
                flag.name,
                flag.flag,
                flag.enabled
            );

            // Save preferences after toggling
            self.save_module_preferences();
        }
    }
}
