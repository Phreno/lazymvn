# Plan d'implémentation : Onglets multi-projets

## Vue d'ensemble

Transformation de lazymvn d'une application **mono-projet** à **multi-projets** via un système d'onglets.

## Phases d'implémentation

### Phase 1 : Refactoring de l'état (Infrastructure)
**Objectif** : Extraire l'état projet de `TuiState` dans `ProjectTab`

#### 1.1 Créer la structure `ProjectTab`
**Fichier** : `src/ui/state/project_tab.rs` (nouveau)

```rust
use std::path::PathBuf;
use std::time::Instant;
use std::sync::mpsc;
use ratatui::widgets::ListState;

pub struct ProjectTab {
    pub id: usize,
    pub project_root: PathBuf,
    
    // Project data
    pub modules: Vec<String>,
    pub profiles: Vec<crate::maven::MavenProfile>,
    pub flags: Vec<crate::ui::state::BuildFlag>,
    
    // List states
    pub modules_list_state: ListState,
    pub profiles_list_state: ListState,
    pub flags_list_state: ListState,
    
    // Command execution
    pub command_output: Vec<String>,
    pub output_offset: usize,
    pub is_command_running: bool,
    pub command_start_time: Option<Instant>,
    pub running_process_pid: Option<u32>,
    pub command_receiver: Option<mpsc::Receiver<crate::maven::CommandUpdate>>,
    
    // Module outputs cache
    module_outputs: std::collections::HashMap<String, ModuleOutput>,
    
    // Project config
    pub config: crate::config::Config,
    pub module_preferences: crate::config::ProjectPreferences,
    
    // File watcher
    pub file_watcher: Option<crate::watcher::FileWatcher>,
    pub last_command: Option<Vec<String>>,
    pub watch_enabled: bool,
    
    // Git info
    pub git_branch: Option<String>,
    
    // Metadata
    pub output_view_height: u16,
    pub output_area_width: u16,
    pub output_metrics: Option<OutputMetrics>,
}

impl ProjectTab {
    pub fn new(
        id: usize, 
        project_root: PathBuf,
        modules: Vec<String>,
        config: crate::config::Config,
    ) -> Self {
        // Initialiser tous les champs...
    }
    
    pub fn get_title(&self) -> String {
        // Retourne le nom du dossier du projet
        self.project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("???")
            .to_string()
    }
    
    pub fn cleanup(&mut self) {
        // Tue le processus en cours si existe
        if let Some(pid) = self.running_process_pid {
            let _ = crate::maven::kill_process(pid);
        }
    }
}
```

#### 1.2 Modifier `TuiState`
**Fichier** : `src/ui/state/mod.rs`

```rust
pub struct TuiState {
    // NOUVEAU : Gestion des onglets
    tabs: Vec<ProjectTab>,
    active_tab_index: usize,
    next_tab_id: usize,
    
    // État global (partagé entre onglets)
    pub current_view: CurrentView,
    pub focus: Focus,
    
    // Recherche (dans l'onglet actif)
    search_state: Option<SearchState>,
    search_input: Option<String>,
    search_history: Vec<String>,
    search_history_index: Option<usize>,
    search_error: Option<String>,
    pub search_mod: Option<SearchMode>,
    pending_center: Option<SearchMatch>,
    
    // Debouncing global
    last_nav_key_time: Option<Instant>,
    nav_debounce_duration: Duration,
    
    // Profile loading (dans l'onglet actif)
    profiles_receiver: Option<mpsc::Receiver<Result<Vec<String>, String>>>,
    pub profile_loading_status: ProfileLoadingStatus,
    profile_loading_start_time: Option<Instant>,
    profile_spinner_frame: usize,
    
    // Projets récents (global)
    pub recent_projects: Vec<PathBuf>,
    pub projects_list_state: ListState,
    pub show_projects_popup: bool,
    
    // Caches globaux (partagés)
    pub starters_cache: crate::starters::StartersCache,
    pub command_history: crate::history::CommandHistory,
    pub favorites: crate::favorites::Favorites,
    
    // Popups globaux
    pub show_starter_selector: bool,
    pub show_starter_manager: bool,
    pub show_history_popup: bool,
    pub show_favorites_popup: bool,
    pub show_save_favorite_popup: bool,
    
    // Clipboard global
    clipboard: Option<arboard::Clipboard>,
    
    // Editor command (global)
    pub editor_command: Option<(String, String)>,
}
```

#### 1.3 Ajouter méthodes de gestion d'onglets
**Fichier** : `src/ui/state/mod.rs`

```rust
impl TuiState {
    pub fn create_tab(&mut self, project_root: PathBuf) -> Result<usize, String> {
        // Vérifier si le projet est déjà ouvert
        if let Some(existing_index) = self.find_tab_by_project(&project_root) {
            self.active_tab_index = existing_index;
            return Ok(existing_index);
        }
        
        // Vérifier limite
        if self.tabs.len() >= 10 {
            return Err("Maximum 10 onglets atteints. Fermez-en un avec Ctrl+W".to_string());
        }
        
        // Charger le projet
        let modules = crate::project::get_project_modules_for_path(&project_root)?;
        let config = crate::config::load_config(&project_root);
        
        // Créer l'onglet
        let tab = ProjectTab::new(self.next_tab_id, project_root, modules, config);
        self.next_tab_id += 1;
        
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
        
        Ok(self.active_tab_index)
    }
    
    pub fn close_tab(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tabs.len() {
            return Err("Tab index out of bounds".to_string());
        }
        
        // Cleanup du tab
        self.tabs[index].cleanup();
        
        // Supprimer
        self.tabs.remove(index);
        
        // Ajuster l'index actif
        if self.active_tab_index >= self.tabs.len() && self.active_tab_index > 0 {
            self.active_tab_index -= 1;
        }
        
        Ok(())
    }
    
    pub fn switch_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab_index = index;
        }
    }
    
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
        }
    }
    
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = if self.active_tab_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_index - 1
            };
        }
    }
    
    pub fn get_active_tab(&self) -> &ProjectTab {
        &self.tabs[self.active_tab_index]
    }
    
    pub fn get_active_tab_mut(&mut self) -> &mut ProjectTab {
        &mut self.tabs[self.active_tab_index]
    }
    
    fn find_tab_by_project(&self, project_root: &PathBuf) -> Option<usize> {
        self.tabs.iter().position(|tab| &tab.project_root == project_root)
    }
}
```

### Phase 2 : Adaptation de l'UI

#### 2.1 Rendu de la barre d'onglets
**Fichier** : `src/ui/panes/mod.rs` (ou nouveau `tabs.rs`)

```rust
pub fn render_tab_bar<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    state: &TuiState,
    theme: &Theme,
) {
    // Créer la liste des titres d'onglets
    let tabs: Vec<String> = state.tabs.iter().enumerate().map(|(i, tab)| {
        let title = tab.get_title();
        let indicator = if tab.is_command_running { " ●" } else { "" };
        let prefix = format!("{}: ", i + 1);
        format!("{}{}{}", prefix, title, indicator)
    }).collect();
    
    // Créer le widget Tabs de ratatui
    let tabs_widget = Tabs::new(tabs)
        .select(state.active_tab_index)
        .style(theme.normal)
        .highlight_style(theme.selected)
        .divider("|");
    
    f.render_widget(tabs_widget, area);
}
```

#### 2.2 Ajuster le layout principal
**Fichier** : `src/tui.rs`

```rust
pub fn draw(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, state: &mut TuiState) -> io::Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),      // NOUVEAU : Barre d'onglets
                Constraint::Length(3),      // Titre
                Constraint::Min(0),         // Contenu principal
                Constraint::Length(1),      // Keybindings
            ])
            .split(f.area());
        
        // Rendu barre d'onglets
        render_tab_bar(f, chunks[0], state, &theme);
        
        // Rendu titre (avec info du projet actif)
        render_title(f, chunks[1], state.get_active_tab(), &theme);
        
        // Reste du rendu...
    })?;
    Ok(())
}
```

### Phase 3 : Adaptation des keybindings

#### 3.1 Ajouter les nouveaux raccourcis
**Fichier** : `src/ui/keybindings/mod.rs`

```rust
pub fn handle_key_event(key: KeyEvent, state: &mut TuiState) {
    // ... existing code
    
    // Gestion des onglets
    match (key.modifiers, key.code) {
        // Ctrl+Tab : onglet suivant
        (KeyModifiers::CONTROL, KeyCode::Tab) => {
            state.next_tab();
            return;
        }
        
        // Ctrl+Shift+Tab : onglet précédent
        (KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyCode::BackTab) => {
            state.prev_tab();
            return;
        }
        
        // Ctrl+W : fermer onglet
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
            let active_tab = state.get_active_tab();
            if active_tab.is_command_running {
                // TODO: Afficher popup de confirmation
                state.command_output = vec![
                    "⚠️  Processus en cours dans cet onglet.".to_string(),
                    "Appuyez à nouveau sur Ctrl+W pour forcer la fermeture.".to_string(),
                ];
            } else {
                let index = state.active_tab_index;
                if state.tabs.len() > 1 {
                    let _ = state.close_tab(index);
                } else {
                    state.command_output = vec!["Cannot close last tab".to_string()];
                }
            }
            return;
        }
        
        // Ctrl+1 à Ctrl+9 : aller à l'onglet N
        (KeyModifiers::CONTROL, KeyCode::Char(c @ '1'..='9')) => {
            let index = (c as usize) - ('1' as usize);
            if index < state.tabs.len() {
                state.switch_to_tab(index);
            }
            return;
        }
        
        _ => {}
    }
    
    // ... rest of existing code
}
```

#### 3.2 Modifier Ctrl+R pour ouvrir en nouvel onglet
**Fichier** : `src/ui/keybindings/mod.rs`

```rust
// Dans le handler de Ctrl+R
(KeyModifiers::CONTROL, KeyCode::Char('r')) => {
    if state.show_projects_popup {
        // Sélectionner projet dans la liste
        if let Some(selected) = state.projects_list_state.selected() {
            if selected < state.recent_projects.len() {
                let project = state.recent_projects[selected].clone();
                state.show_projects_popup = false;
                
                // NOUVEAU : Créer onglet au lieu de remplacer
                match state.create_tab(project) {
                    Ok(_) => {
                        log::info!("Opened project in new tab");
                    }
                    Err(e) => {
                        state.get_active_tab_mut().command_output = vec![
                            format!("❌ Failed to open project: {}", e)
                        ];
                    }
                }
            }
        }
    } else {
        state.switch_to_projects();
    }
    return;
}
```

### Phase 4 : Adaptation de main.rs

#### 4.1 Initialisation avec premier onglet
**Fichier** : `src/main.rs`

```rust
// Au lieu de créer TuiState directement
// let mut state = tui::TuiState::new(modules, project_root.clone(), config);

// NOUVEAU : Créer avec système d'onglets
let mut state = tui::TuiState::new_with_tabs();
match state.create_tab(project_root.clone()) {
    Ok(_) => {
        log::info!("Initial project loaded in first tab");
    }
    Err(e) => {
        return Err(format!("Failed to load initial project: {}", e).into());
    }
}
```

#### 4.2 Supprimer le code de switch_to_project
**Fichier** : `src/main.rs`

```rust
// SUPPRIMER ce bloc (plus nécessaire avec onglets)
/*
if let Some(new_project) = state.switch_to_project.take() {
    // Change directory
    // Reload project
    // Create new state
    // ...
}
*/

// Les projets s'ouvrent maintenant dans de nouveaux onglets via create_tab()
```

#### 4.3 Cleanup de tous les onglets
**Fichier** : `src/main.rs`

```rust
// Avant de quitter
// REMPLACER : state.cleanup();
// PAR :
state.cleanup_all_tabs();
```

### Phase 5 : Tests et validation

#### 5.1 Tests unitaires
**Fichier** : `tests/tabs_tests.rs` (nouveau)

```rust
#[cfg(test)]
mod tabs_tests {
    use super::*;
    
    #[test]
    fn test_create_multiple_tabs() {
        let mut state = TuiState::new_with_tabs();
        
        // Créer 3 onglets
        let tab1 = state.create_tab(PathBuf::from("/proj1")).unwrap();
        let tab2 = state.create_tab(PathBuf::from("/proj2")).unwrap();
        let tab3 = state.create_tab(PathBuf::from("/proj3")).unwrap();
        
        assert_eq!(state.tabs.len(), 3);
        assert_eq!(state.active_tab_index, 2); // Dernier créé
    }
    
    #[test]
    fn test_prevent_duplicate_tabs() {
        let mut state = TuiState::new_with_tabs();
        let proj = PathBuf::from("/proj1");
        
        let tab1 = state.create_tab(proj.clone()).unwrap();
        let tab2 = state.create_tab(proj.clone()).unwrap();
        
        assert_eq!(state.tabs.len(), 1); // Pas de doublon
        assert_eq!(tab1, tab2); // Même index
    }
    
    #[test]
    fn test_max_tabs_limit() {
        let mut state = TuiState::new_with_tabs();
        
        // Créer 10 onglets
        for i in 0..10 {
            state.create_tab(PathBuf::from(format!("/proj{}", i))).unwrap();
        }
        
        // Le 11e doit échouer
        let result = state.create_tab(PathBuf::from("/proj11"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_close_tab_adjusts_active_index() {
        let mut state = TuiState::new_with_tabs();
        state.create_tab(PathBuf::from("/proj1")).unwrap();
        state.create_tab(PathBuf::from("/proj2")).unwrap();
        state.create_tab(PathBuf::from("/proj3")).unwrap();
        
        state.active_tab_index = 1;
        state.close_tab(1).unwrap();
        
        assert_eq!(state.tabs.len(), 2);
        assert_eq!(state.active_tab_index, 1); // Reste valide
    }
}
```

### Phase 6 : Documentation

#### 6.1 README.md
Ajouter section sur les onglets :

```markdown
## Multi-Project Tabs

LazyMVN supports multiple projects in tabs, allowing you to work on several projects simultaneously without launching multiple instances.

### Tab Management

| Key | Action |
|-----|--------|
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |
| `Ctrl+1-9` | Switch to tab N |
| `Ctrl+W` | Close current tab |
| `Ctrl+R` | Open recent project in new tab |

### Features

- Each tab maintains independent state (selections, output, running processes)
- Visual indicator (●) shows tabs with running processes
- Maximum 10 tabs by default (configurable)
- Prevents duplicate tabs for the same project
```

## Migration guide

### Pour les contributeurs

1. **Accès à l'état projet** :
   ```rust
   // AVANT
   state.modules
   state.command_output
   
   // APRÈS
   state.get_active_tab().modules
   state.get_active_tab_mut().command_output
   ```

2. **Lancement de commande** :
   ```rust
   // Le receiver va maintenant dans l'onglet actif
   let tab = state.get_active_tab_mut();
   tab.command_receiver = Some(receiver);
   ```

3. **Cleanup** :
   ```rust
   // AVANT
   state.cleanup()
   
   // APRÈS
   state.cleanup_all_tabs()
   ```

## Estimation

- **Phase 1** : 3-4 heures (refactoring structure)
- **Phase 2** : 2-3 heures (UI barre d'onglets)
- **Phase 3** : 2 heures (keybindings)
- **Phase 4** : 1 heure (main.rs)
- **Phase 5** : 2 heures (tests)
- **Phase 6** : 1 heure (documentation)

**Total** : ~12 heures de développement

## Risques et mitigations

| Risque | Mitigation |
|--------|-----------|
| Breaking changes massifs | Refactoring incrémental, tests à chaque étape |
| Performance (N receivers) | Limiter à 10 onglets, polling optimisé |
| Mémoire (N buffers) | Config max_lines par onglet |
| Complexité UI | Barre d'onglets simple, pas de fancy UI |

Voulez-vous que je commence l'implémentation ?
