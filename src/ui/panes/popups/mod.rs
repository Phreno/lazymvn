//! Popup rendering modules

mod starters;
mod packages;
mod projects;
mod history;
mod favorites;
mod help;
mod custom_goals;

// Re-export all popup rendering functions
pub use starters::{render_starter_selector_popup, render_starter_manager_popup};
pub use packages::render_package_selector_popup;
pub use projects::render_projects_popup;
pub use history::render_history_popup;
pub use favorites::{render_favorites_popup, render_save_favorite_popup};
pub use help::render_help_popup;
pub use custom_goals::render_custom_goals_popup;
