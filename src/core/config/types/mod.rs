//! Configuration type definitions

mod main;
mod preferences;

pub use main::{
    Config, MavenConfig, CustomFlag, CustomGoal,
    WatchConfig, LaunchMode,
};
pub use preferences::{
    RecentProjects, ModulePreferences,
    ProfilesCache, ProjectPreferences,
};
