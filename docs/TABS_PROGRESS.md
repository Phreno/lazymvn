# Progression de l'implémentation des onglets multi-projets

## Phase 1 : Infrastructure - ProjectTab ✅ TERMINÉE

**Date** : 2025-10-23

### Tâches complétées

- [x] Création de `src/ui/state/project_tab.rs`
- [x] Structure `ProjectTab` avec tous les champs nécessaires
- [x] Méthode `new()` pour initialiser un onglet
- [x] Méthode `get_title()` pour afficher le nom
- [x] Méthode `get_short_title()` pour titre raccourci
- [x] Méthode `has_running_process()` pour vérifier processus actif
- [x] Méthode `cleanup()` pour tuer processus et sauvegarder
- [x] Implémentation de `Drop` pour cleanup automatique
- [x] Module ajouté dans `src/ui/state/mod.rs`
- [x] Compilation réussie
- [x] Clippy sans erreurs

### Fichiers modifiés

- `src/ui/state/project_tab.rs` (nouveau, 241 lignes)
- `src/ui/state/mod.rs` (ajout module)
- `TABS_PROPOSAL.md` (documentation)
- `TABS_IMPLEMENTATION.md` (plan)

### Tests

```bash
✅ cargo build      # Compilation OK
✅ cargo clippy     # Aucune erreur, 3 warnings attendus (fonctions non utilisées)
```

## Phase 2 : Adaptation de TuiState - EN COURS

### Objectifs

- [ ] Ajouter `Vec<ProjectTab>` dans `TuiState`
- [ ] Ajouter `active_tab_index` et `next_tab_id`
- [ ] Créer méthode `new_with_tabs()`
- [ ] Créer méthode `create_tab(project_root)`
- [ ] Créer méthode `close_tab(index)`
- [ ] Créer méthode `switch_to_tab(index)`
- [ ] Créer méthodes `next_tab()` et `prev_tab()`
- [ ] Créer méthodes `get_active_tab()` et `get_active_tab_mut()`
- [ ] Créer méthode `find_tab_by_project()`
- [ ] Créer méthode `cleanup_all_tabs()`
- [ ] Adapter toutes les méthodes existantes pour utiliser `get_active_tab()`

### Prochaines étapes

1. Modifier la structure `TuiState` pour ajouter les onglets
2. Créer les méthodes de gestion d'onglets
3. Adapter les méthodes existantes (accès via `get_active_tab()`)
4. Compiler et tester

## Phase 3 : Interface utilisateur (À venir)

- [ ] Créer fonction `render_tab_bar()`
- [ ] Modifier `src/tui.rs` pour ajouter ligne d'onglets
- [ ] Ajuster layout pour faire de la place à la barre
- [ ] Ajouter indicateurs visuels (●) pour processus actifs

## Phase 4 : Keybindings (À venir)

- [ ] `Ctrl+Tab` : onglet suivant
- [ ] `Ctrl+Shift+Tab` : onglet précédent
- [ ] `Ctrl+1-9` : aller à l'onglet N
- [ ] `Ctrl+W` : fermer onglet
- [ ] Modifier `Ctrl+R` pour ouvrir en nouvel onglet

## Phase 5 : Adaptation de main.rs (À venir)

- [ ] Initialiser avec `new_with_tabs()`
- [ ] Créer premier onglet au démarrage
- [ ] Supprimer code `switch_to_project`
- [ ] Appeler `cleanup_all_tabs()` à la sortie

## Phase 6 : Tests et documentation (À venir)

- [ ] Tests unitaires pour gestion d'onglets
- [ ] Tests de création/fermeture
- [ ] Tests de limite max
- [ ] Tests de prévention doublons
- [ ] Mise à jour README.md
- [ ] Mise à jour CHANGELOG.md

---

**Statut global** : Phase 1 complète ✅, Phase 2 en préparation
