# Proposition : Système d'onglets multi-projets

## Problème actuel
- Travailler sur plusieurs projets nécessite plusieurs instances de lazymvn
- Ctrl+R remplace le projet actuel au lieu d'en ouvrir un nouveau
- Pas de vue d'ensemble des projets ouverts
- Perte du contexte (sélections, output) lors du changement de projet
- Gestion complexe avec plusieurs terminaux

## Solution proposée : Onglets par projet

### Architecture

#### 1. Structure `ProjectTab`
Chaque onglet représente un projet Maven complet avec son état :

```rust
pub struct ProjectTab {
    id: usize,
    title: String,                      // Nom du projet (dossier)
    project_root: PathBuf,              // Chemin du projet
    
    // Project state (tout ce qui est actuellement dans TuiState)
    modules: Vec<String>,
    profiles: Vec<MavenProfile>,
    flags: Vec<BuildFlag>,
    modules_list_state: ListState,
    profiles_list_state: ListState,
    flags_list_state: ListState,
    
    // Output and command state
    command_output: Vec<String>,
    output_offset: usize,
    is_command_running: bool,
    running_process_pid: Option<u32>,
    command_receiver: Option<mpsc::Receiver<CommandUpdate>>,
    
    // Project-specific config
    config: Config,
    module_preferences: ProjectPreferences,
    git_branch: Option<String>,
    
    // File watcher per project
    file_watcher: Option<FileWatcher>,
    last_command: Option<Vec<String>>,
    
    // Status indicator
    has_running_process: bool,
}
```

#### 2. Gestionnaire d'onglets dans `TuiState`
```rust
pub struct TuiState {
    // Tabs management
    tabs: Vec<ProjectTab>,
    active_tab_index: usize,
    next_tab_id: usize,
    
    // Global state (partagé entre tous les onglets)
    current_view: CurrentView,
    focus: Focus,
    search_state: Option<SearchState>,
    recent_projects: Vec<PathBuf>,
    clipboard: Option<arboard::Clipboard>,
    
    // Global caches (partagés)
    starters_cache: StartersCache,
    command_history: CommandHistory,
    favorites: Favorites,
    
    // UI state
    show_projects_popup: bool,
    show_starter_selector: bool,
    // ... autres popups globaux
}
```

### Interface utilisateur

#### Barre d'onglets (en haut, toujours visible)
```
┌──────────────────────────────────────────────────────────────────┐
│ [1: my-service ●] [2: auth-api] [3: frontend ●] [+] Ctrl+R     │
└──────────────────────────────────────────────────────────────────┘
│                                                                  │
│  LazyMVN - my-service (develop)                                 │
│                                                                  │
│  ┌─ Modules ──┐ ┌─ Profiles ──┐ ┌─ Flags ───┐ ┌─ Output ────┐ │
│  │ app        │ │ dev          │ │ -o        │ │              │ │
│  │ library    │ │ prod         │ │ -T 4      │ │ Building...  │ │
│  └────────────┘ └──────────────┘ └───────────┘ └──────────────┘ │
```

**Indicateurs d'onglet** :
- `●` = Processus en cours d'exécution (couleur verte clignotante)
- Pas de symbole = Projet inactif, prêt
- Nom raccourci si trop long (ex: "my-long-project..." → "my-long-p...")

**Nom d'onglet** :
- Nom du dossier du projet par défaut
- Optionnel : afficher la branche git entre parenthèses

#### Navigation entre onglets
- `Ctrl+Tab` : Onglet suivant (comme navigateur)
- `Ctrl+Shift+Tab` : Onglet précédent
- `Ctrl+1` à `Ctrl+9` : Aller directement à l'onglet N
- `Ctrl+W` : Fermer onglet actif (confirmation si processus en cours)
- `Ctrl+R` : **Ouvrir projet récent dans nouvel onglet** (au lieu de remplacer)
- Clic sur onglet : Switch vers ce projet (si support souris)

### Fonctionnalités

#### 1. Création d'onglets
- **Au démarrage** : Premier projet ouvert = premier onglet
- **Ctrl+R** : Sélectionner un projet récent l'ouvre dans un **nouvel onglet**
- **Ligne de commande** : `lazymvn --project /path/to/project` ouvre dans un nouvel onglet si déjà lancé
- Maximum 10 onglets simultanés (configurable)
- Le nouvel onglet prend automatiquement le focus

#### 2. État indépendant par projet
- Chaque onglet conserve :
  - Sa sélection de module
  - Ses profils/flags actifs
  - Son output buffer et position de scroll
  - Son processus Maven en cours
  - Ses préférences de module
  - Son file watcher (si activé)
  
#### 3. Gestion des processus
- Chaque projet peut avoir **son propre processus Maven indépendant**
- Exemple : Spring Boot sur projet 1, tests sur projet 2, build sur projet 3
- Les sorties sont complètement isolées entre onglets
- Fermer un onglet tue proprement son processus (si confirmé)

#### 4. Persistence
- Les onglets ouverts sont **sauvegardés entre sessions** (optionnel)
- Restauration automatique au prochain lancement
- Chaque onglet retrouve son état (sélections, scroll, etc.)

#### 5. Cleanup
- À la fermeture de lazymvn, tous les processus de tous les onglets sont tués
- Confirmation si des processus tournent : "3 processus en cours. Vraiment quitter ?"
- Option pour fermer tout sauf un onglet spécifique

### Cas d'usage

#### Développement microservices
```
Tab 1: [api-gateway ●]      - Spring Boot sur 8080 (projet A)
Tab 2: [auth-service ●]     - Spring Boot sur 8081 (projet B)  
Tab 3: [user-service]       - Prêt à lancer (projet C)
Tab 4: [order-service ●]    - Tests en cours (projet D)
```

Avant : 4 terminaux avec 4 instances lazymvn
Après : 1 terminal, 4 onglets dans lazymvn

#### Développement front-end + back-end
```
Tab 1: [frontend ●]         - npm dev server (projet React/Vue)
Tab 2: [backend-api ●]      - Spring Boot API (projet Java)
Tab 3: [mobile-app]         - Build Android (projet mobile)
```

#### Travail sur plusieurs branches
```
Tab 1: [my-app (main)]      - Version production
Tab 2: [my-app (feature/X) ●] - Dev nouvelle feature (tests)
Tab 3: [my-app (hotfix)]    - Hotfix urgent
```

Même projet, différents dossiers (worktrees git)

#### Développement multi-clients
```
Tab 1: [client-acme ●]      - Projet client A en cours
Tab 2: [client-contoso]     - Projet client B (standby)
Tab 3: [common-lib ●]       - Librairie partagée (build)
```

### Implémentation par phases

#### Phase 1: Infrastructure de base ✅
- [x] Structure `CommandTab`
- [x] Gestion de plusieurs `command_receiver`
- [x] Buffer de sortie par onglet

#### Phase 2: Interface utilisateur
- [ ] Rendu de la barre d'onglets
- [ ] Navigation entre onglets (Tab, Shift+Tab)
- [ ] Indicateurs visuels de statut

#### Phase 3: Gestion des commandes
- [ ] Création automatique d'onglet lors du lancement
- [ ] Fermeture d'onglet avec kill du processus
- [ ] Limitation du nombre d'onglets

#### Phase 4: Améliorations
- [ ] Raccourcis Ctrl+1-9
- [ ] Confirmation avant fermeture si processus actifs
- [ ] Statistiques par onglet (durée, lignes, etc.)
- [ ] Sauvegarde/restauration des onglets

### Configuration

```toml
[tabs]
# Maximum number of concurrent project tabs (default: 10)
max_tabs = 10

# Show tab bar even with only one tab (default: true)
# If false, tab bar only shows with 2+ tabs
always_show_tab_bar = true

# Confirm before closing tabs with running processes (default: true)
confirm_close_running = true

# Restore tabs from previous session (default: false)
# If true, reopens all projects that were open when you quit
restore_session = false

# Show git branch in tab title (default: true)
show_git_branch = true

# Tab title max length (default: 20)
# Longer names are truncated: "very-long-project-name" → "very-long-proj..."
max_title_length = 20
```

### API proposée

```rust
impl TuiState {
    // Tab management
    pub fn create_tab(&mut self, project_root: PathBuf) -> Result<usize, String>;
    pub fn close_tab(&mut self, tab_index: usize) -> Result<(), String>;
    pub fn switch_to_tab(&mut self, index: usize);
    pub fn next_tab(&mut self);
    pub fn prev_tab(&mut self);
    pub fn get_active_tab(&self) -> &ProjectTab;
    pub fn get_active_tab_mut(&mut self) -> &mut ProjectTab;
    pub fn get_tab_count(&self) -> usize;
    
    // Process control per tab
    pub fn kill_tab_process(&mut self, tab_index: usize) -> Result<(), String>;
    pub fn kill_all_tab_processes(&mut self);
    pub fn has_running_processes(&self) -> bool;
    pub fn count_running_processes(&self) -> usize;
    
    // Session management
    pub fn save_session(&self) -> Result<(), String>;
    pub fn restore_session() -> Result<Vec<PathBuf>, String>;
}

impl ProjectTab {
    pub fn new(project_root: PathBuf, config: Config) -> Result<Self, String>;
    pub fn get_title(&self) -> String;
    pub fn has_running_process(&self) -> bool;
    pub fn cleanup(&mut self);
}
```

### Avantages

✅ **Productivité** : Lancer plusieurs services depuis une seule fenêtre
✅ **Vue d'ensemble** : Statut de tous les processus en un coup d'œil
✅ **Simplicité** : Pas besoin de plusieurs terminaux/instances
✅ **Contexte** : Chaque onglet garde l'historique de sortie
✅ **Performance** : Partage des ressources (cache, config, etc.)

### Limitations connues

⚠️ **Mémoire** : Chaque onglet stocke son buffer de sortie
⚠️ **CPU** : Polling de N receivers au lieu d'un seul
⚠️ **Complexité UI** : Barre d'onglets prend de l'espace vertical

### Alternatives considérées

1. **Split horizontal/vertical** : Trop complexe, espace limité
2. **Mode liste des processus** : Moins intuitif qu'onglets
3. **Fenêtres flottantes** : Impossible en TUI standard

### Questions ouvertes

1. **Limite d'onglets atteinte** : Que faire quand on atteint max_tabs ?
   - ✅ **Recommandé** : Message d'erreur "Maximum 10 onglets atteints. Fermez-en un avec Ctrl+W"
   - Alternative : Fermer l'onglet le plus ancien automatiquement
   - Alternative : Popup pour choisir quel onglet fermer

2. **Ctrl+R avec onglets existants** :
   - ✅ **Recommandé** : Toujours ouvrir dans nouvel onglet
   - Alternative : Demander "Nouvel onglet ou remplacer actuel ?"
   - Alternative : Shift+Ctrl+R = nouvel onglet, Ctrl+R = remplacer

3. **File watcher multi-projets** :
   - ✅ **Recommandé** : Chaque onglet a son propre watcher indépendant
   - Relance automatique uniquement dans l'onglet concerné
   - Pas de synchronisation entre onglets

4. **Recherche dans output** :
   - ✅ **Recommandé** : Recherche uniquement dans l'onglet actif
   - Garder simple, pas de recherche cross-tab
   - Raccourci futur possible : Ctrl+Shift+F pour recherche globale

5. **Projet déjà ouvert** :
   - Si on essaie d'ouvrir un projet déjà dans un onglet :
     - ✅ **Recommandé** : Basculer vers l'onglet existant (pas de doublon)
     - Alternative : Permettre doublons (ex: même projet, branches différentes)

6. **Sauvegarde des processus en cours** :
   - Au restore_session, ne pas relancer les processus
   - Juste restaurer la structure (projets, sélections)
   - L'utilisateur relance manuellement ce qu'il veut

## Implémentation recommandée

Je recommande de commencer par la **Phase 1 + 2** pour valider l'UX, puis itérer selon les retours.

Voulez-vous que je commence l'implémentation ?
