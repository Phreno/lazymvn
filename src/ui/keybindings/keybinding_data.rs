/// Represents a keyboard shortcut with its metadata
#[derive(Clone, Debug)]
pub struct Keybinding {
    pub category: &'static str,
    pub keys: &'static str,
    pub description: &'static str,
    pub action: Option<KeybindingAction>,
}

/// Action that can be executed from a keybinding
#[derive(Clone, Debug)]
pub enum KeybindingAction {
    // Navigation
    FocusPreviousPane,
    FocusNextPane,
    FocusPane(crate::ui::keybindings::Focus),
    
    // Maven Commands  
    Build,
    Compile,
    Clean,
    Package,
    Test,
    Install,
    Dependencies,
    
    // Spring Boot
    RunStarter,
    ManageStarters,
    
    // Workflow
    ShowFavorites,
    SaveFavorite,
    ShowHistory,
    ShowRecentProjects,
    EditConfig,
    RefreshCaches,
    ShowCustomGoals,
    
    // Tab Management
    NewTab,
    CloseTab,
    PreviousTab,
    NextTab,
    
    // Search & Selection
    StartSearch,
    YankOutput,
    YankDebugReport,
    
    // General
    ShowHelp,
    Quit,
    KillProcess,
}

/// Get all keybindings organized by category
pub fn get_all_keybindings() -> Vec<Keybinding> {
    vec![
        // Navigation
        Keybinding {
            category: "Navigation",
            keys: "←/→",
            description: "Cycle focus between panes",
            action: None, // Complex navigation
        },
        Keybinding {
            category: "Navigation",
            keys: "↑/↓",
            description: "Move selection / Scroll output",
            action: None,
        },
        Keybinding {
            category: "Navigation",
            keys: "PgUp/PgDn",
            description: "Scroll output by pages",
            action: None,
        },
        Keybinding {
            category: "Navigation",
            keys: "Home/End",
            description: "Jump to start/end of output",
            action: None,
        },
        Keybinding {
            category: "Navigation",
            keys: "0",
            description: "Focus Output pane",
            action: Some(KeybindingAction::FocusPane(crate::ui::keybindings::Focus::Output)),
        },
        Keybinding {
            category: "Navigation",
            keys: "1",
            description: "Focus Projects pane",
            action: Some(KeybindingAction::FocusPane(crate::ui::keybindings::Focus::Projects)),
        },
        Keybinding {
            category: "Navigation",
            keys: "2",
            description: "Focus Modules pane",
            action: Some(KeybindingAction::FocusPane(crate::ui::keybindings::Focus::Modules)),
        },
        Keybinding {
            category: "Navigation",
            keys: "3",
            description: "Focus Profiles pane",
            action: Some(KeybindingAction::FocusPane(crate::ui::keybindings::Focus::Profiles)),
        },
        Keybinding {
            category: "Navigation",
            keys: "4",
            description: "Focus Flags pane",
            action: Some(KeybindingAction::FocusPane(crate::ui::keybindings::Focus::Flags)),
        },
        Keybinding {
            category: "Navigation",
            keys: "Mouse",
            description: "Click to focus/select",
            action: None,
        },
        
        // Tab Management
        Keybinding {
            category: "Tab Management",
            keys: "Ctrl+T",
            description: "Create new tab",
            action: Some(KeybindingAction::NewTab),
        },
        Keybinding {
            category: "Tab Management",
            keys: "Ctrl+W",
            description: "Close current tab",
            action: Some(KeybindingAction::CloseTab),
        },
        Keybinding {
            category: "Tab Management",
            keys: "Ctrl+←/→",
            description: "Switch between tabs",
            action: None,
        },
        
        // Maven Commands
        Keybinding {
            category: "Maven Commands",
            keys: "b",
            description: "Build (clean install)",
            action: Some(KeybindingAction::Build),
        },
        Keybinding {
            category: "Maven Commands",
            keys: "c",
            description: "Compile",
            action: Some(KeybindingAction::Compile),
        },
        Keybinding {
            category: "Maven Commands",
            keys: "C",
            description: "Clean",
            action: Some(KeybindingAction::Clean),
        },
        Keybinding {
            category: "Maven Commands",
            keys: "k",
            description: "Package",
            action: Some(KeybindingAction::Package),
        },
        Keybinding {
            category: "Maven Commands",
            keys: "t",
            description: "Test",
            action: Some(KeybindingAction::Test),
        },
        Keybinding {
            category: "Maven Commands",
            keys: "i",
            description: "Install",
            action: Some(KeybindingAction::Install),
        },
        Keybinding {
            category: "Maven Commands",
            keys: "d",
            description: "Dependencies (tree)",
            action: Some(KeybindingAction::Dependencies),
        },
        Keybinding {
            category: "Maven Commands",
            keys: "Esc",
            description: "Kill running process",
            action: Some(KeybindingAction::KillProcess),
        },
        
        // Spring Boot
        Keybinding {
            category: "Spring Boot",
            keys: "s",
            description: "Run starter (opens selector)",
            action: Some(KeybindingAction::RunStarter),
        },
        Keybinding {
            category: "Spring Boot",
            keys: "Ctrl+Shift+S",
            description: "Open starter manager",
            action: Some(KeybindingAction::ManageStarters),
        },
        
        // Workflow
        Keybinding {
            category: "Workflow",
            keys: "Ctrl+F",
            description: "Show favorites",
            action: Some(KeybindingAction::ShowFavorites),
        },
        Keybinding {
            category: "Workflow",
            keys: "Ctrl+S",
            description: "Save current config as favorite",
            action: Some(KeybindingAction::SaveFavorite),
        },
        Keybinding {
            category: "Workflow",
            keys: "Ctrl+H",
            description: "Show command history",
            action: Some(KeybindingAction::ShowHistory),
        },
        Keybinding {
            category: "Workflow",
            keys: "Ctrl+R",
            description: "Show recent projects",
            action: Some(KeybindingAction::ShowRecentProjects),
        },
        Keybinding {
            category: "Workflow",
            keys: "Ctrl+E",
            description: "Edit configuration (lazymvn.toml)",
            action: Some(KeybindingAction::EditConfig),
        },
        Keybinding {
            category: "Workflow",
            keys: "Ctrl+K",
            description: "Refresh caches (profiles/starters)",
            action: Some(KeybindingAction::RefreshCaches),
        },
        Keybinding {
            category: "Workflow",
            keys: "Ctrl+G",
            description: "Show custom goals popup",
            action: Some(KeybindingAction::ShowCustomGoals),
        },
        Keybinding {
            category: "Workflow",
            keys: "Ctrl+P",
            description: "Select package for logging config",
            action: None,
        },
        
        // Selection & Search
        Keybinding {
            category: "Selection & Search",
            keys: "Space/Enter",
            description: "Toggle selection (profiles/flags)",
            action: None,
        },
        Keybinding {
            category: "Selection & Search",
            keys: "/",
            description: "Start search in output",
            action: Some(KeybindingAction::StartSearch),
        },
        Keybinding {
            category: "Selection & Search",
            keys: "n",
            description: "Next search match",
            action: None,
        },
        Keybinding {
            category: "Selection & Search",
            keys: "N",
            description: "Previous search match",
            action: None,
        },
        Keybinding {
            category: "Selection & Search",
            keys: "y",
            description: "Yank (copy) output to clipboard",
            action: Some(KeybindingAction::YankOutput),
        },
        Keybinding {
            category: "Selection & Search",
            keys: "Y",
            description: "Yank debug report (comprehensive)",
            action: Some(KeybindingAction::YankDebugReport),
        },
        Keybinding {
            category: "Selection & Search",
            keys: "Esc",
            description: "Exit search mode",
            action: None,
        },
        
        // General
        Keybinding {
            category: "General",
            keys: "?",
            description: "Show this help",
            action: Some(KeybindingAction::ShowHelp),
        },
        Keybinding {
            category: "General",
            keys: "q",
            description: "Quit LazyMVN",
            action: Some(KeybindingAction::Quit),
        },
    ]
}
