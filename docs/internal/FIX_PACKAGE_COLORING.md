# Fix: Package Name Coloring in Maven Output

## Problem
La coloration des noms de packages ne fonctionnait pas dans l'output Maven. Bien que le code pour extraire et coloriser les packages existait dans `src/utils/text.rs`, il n'était jamais utilisé lors de l'affichage de la sortie Maven dans l'interface TUI.

## Root Cause
La fonction `render_output_pane()` dans `src/ui/panes/basic_panes.rs` appelait l'ancienne fonction `colorize_log_line()` au lieu de `colorize_log_line_with_format()`. Cette ancienne fonction ne prend pas de paramètre `log_format`, donc elle ne pouvait pas extraire et coloriser les noms de packages.

## Solution Implemented

### 1. Mise à jour de `render_output_pane()`
**Fichier**: `src/ui/panes/basic_panes.rs`

Ajout d'un paramètre `log_format: Option<&str>` à la fonction `render_output_pane()` et remplacement de l'appel à `colorize_log_line()` par `colorize_log_line_with_format()`.

```rust
pub fn render_output_pane(
    // ... autres paramètres ...
    log_format: Option<&str>,  // ← Nouveau paramètre
) {
    // ...
    // Normal mode: use keyword-based coloring with log format for package extraction
    let cleaned = crate::utils::clean_log_line(line).unwrap_or_default();
    crate::utils::colorize_log_line_with_format(&cleaned, log_format)  // ← Utilise le format
}
```

### 2. Mise à jour du renderer
**Fichier**: `src/tui/renderer.rs`

Extraction du `log_format` depuis la configuration du tab actif et passage de celui-ci à `render_output_pane()`.

```rust
let tab = state.get_active_tab();
let log_format = tab.config.logging.as_ref().and_then(|l| l.log_format.as_deref());
render_output_pane(
    // ... autres arguments ...
    log_format,  // ← Passe le format de log
);
```

## Configuration
Pour activer la coloration des packages, ajoutez ceci dans `lazymvn.toml` :

```toml
[logging]
# Log format pattern (used to extract and colorize package names)
# Supported patterns: %c (logger/package), %p (level), %m (message), %n (newline)
log_format = "[%p] %c - %m%n"
```

### Exemples de log_format supportés
- `"[%p] %c - %m%n"` - Format standard avec level, package, et message
- `"[%p] %c{1} - %m%n"` - Format court (seulement la dernière partie du package)
- `"%d [%p] %c - %m%n"` - Avec timestamp

## Comment ça marche

### Flow de données
1. La configuration est chargée depuis `lazymvn.toml` dans `Config::logging::log_format`
2. Chaque `ProjectTab` conserve sa propre instance de `Config`
3. Le renderer extrait le `log_format` de la configuration du tab actif
4. Le format est passé à `render_output_pane()`
5. Pour chaque ligne de log, `colorize_log_line_with_format()` est appelée
6. La fonction `extract_package_from_log_line()` analyse le format et extrait le nom du package
7. Le nom du package est colorisé en cyan

### Extraction des packages
La fonction `extract_package_from_log_line()` :
1. Localise d'abord le niveau de log (`[INFO]`, `[DEBUG]`, etc.)
2. Cherche le texte après le niveau de log
3. Extrait le nom du package jusqu'au premier séparateur (espace, dash, etc.)
4. Valide que c'est bien un nom de package valide

## Tests
Les tests unitaires existants passent :
```bash
cargo test test_colorize_log_line_with --lib
cargo test test_extract_package --lib
```

## Exemple de résultat
Avant le fix :
```
[INFO] com.example.service.UserService - User created
```

Après le fix (avec coloration) :
```
[INFO] com.example.service.UserService - User created
       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
       (colorisé en cyan)
```

## Files Modified
- `src/ui/panes/basic_panes.rs` - Ajout du paramètre log_format et utilisation de colorize_log_line_with_format
- `src/tui/renderer.rs` - Extraction et passage du log_format
- `demo/multi-module/lazymvn.toml` - Ajout d'un exemple de configuration logging

## Related
- `src/utils/text.rs` - Contient les fonctions de coloration (déjà implémentées)
- `src/core/config/logging.rs` - Définition de LoggingConfig
- `scripts/test-package-coloring.sh` - Script de test de la fonctionnalité

## Status
✅ **Fixed** - La coloration des packages fonctionne maintenant correctement quand `log_format` est configuré dans `lazymvn.toml`.
