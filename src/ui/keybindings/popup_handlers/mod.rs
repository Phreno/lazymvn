//! Popup key event handlers

mod favorites;
mod history;
mod projects;
mod custom_goals;
mod starters;
mod packages;
mod help;

pub use favorites::{handle_save_favorite_popup, handle_favorites_popup};
pub use history::handle_history_popup;
pub use projects::handle_projects_popup;
pub use custom_goals::handle_custom_goals_popup;
pub use starters::{handle_starter_selector, handle_starter_manager};
pub use packages::handle_package_selector;
pub use help::handle_help_popup;
