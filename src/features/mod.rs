//! Optional features module
//!
//! This module contains optional features that enhance the user experience:
//! - `favorites`: Save and load favorite command configurations
//! - `history`: Track command execution history
//! - `starters`: Spring Boot starter dependency management

pub mod favorites;
pub mod history;
pub mod starters;

// Re-export main types for convenience (used by UI state and other modules)
#[allow(unused_imports)]
pub use favorites::Favorites;
#[allow(unused_imports)]
pub use history::{CommandHistory, HistoryEntry};
#[allow(unused_imports)]
pub use starters::{Starter, StartersCache};
