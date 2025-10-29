# Fix: Ctrl+K vs k Keybinding Conflict

**Date:** 29 octobre 2025  
**Issue:** Conflit entre `k` (package) et `Ctrl+K` (refresh caches)

---

## Problème Identifié

### Symptômes
- `Ctrl+K` ne fonctionnait pas pour rafraîchir les caches
- La touche `k` seule capturait toutes les presses, même avec Ctrl

### Cause Racine
Dans `src/ui/keybindings/command_keys.rs`, la fonction `handle_maven_command()` ne vérifiait pas les modificateurs de touches. Elle capturait donc `Ctrl+K` avant que `handle_popup_triggers()` puisse le gérer.

**Ordre d'exécution dans `mod.rs`:**
```rust
// Try Maven command keys first
if command_keys::handle_maven_command(key, state) {
    return;  // ❌ Ctrl+K capturé ici !
}

// Try popup triggers (Ctrl+F/S/H/R/E/K)
if navigation_keys::handle_popup_triggers(key, state) {
    return;  // ❌ Jamais atteint pour Ctrl+K
}
```

**Code problématique:**
```rust
KeyCode::Char('k') => {
    log::info!("Execute: package");
    state.run_selected_module_command(&["package"]);
    true
}
```

Ce code capturait **tous** les événements avec le caractère 'k', y compris `Ctrl+K`, `Alt+K`, etc.

---

## Solution Implémentée

### 1. Vérification des Modificateurs

Ajout d'une vérification que **aucun modificateur** n'est pressé pour les commandes Maven simples :

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_maven_command(key: KeyEvent, state: &mut TuiState) -> bool {
    // Only handle keys without modifiers (except for Ctrl+Shift+S)
    // This allows Ctrl+K to be handled by navigation_keys for cache refresh
    let has_modifiers = key.modifiers != KeyModifiers::NONE;
    
    match key.code {
        KeyCode::Char('k') if !has_modifiers => {
            log::info!("Execute: package");
            state.run_selected_module_command(&["package"]);
            true
        }
        // ... autres commandes avec même pattern
    }
}
```

### 2. Pattern Appliqué

Toutes les commandes Maven simples utilisent maintenant le guard `if !has_modifiers` :
- `b` - Build (clean install)
- `c` - Compile
- `C` - Clean
- `k` - Package ✅ **Fix appliqué**
- `t` - Test
- `i` - Install
- `d` - Dependencies (tree)
- `s` - Run Spring Boot starter

**Exception:** `Ctrl+Shift+S` (starter manager) continue à gérer ses propres modificateurs.

---

## Résultat

### Comportement Correct
- ✅ `k` seul → `mvn package`
- ✅ `Ctrl+K` → Refresh caches (profiles + starters)
- ✅ Pas d'interférence entre les deux

### Test Ajouté
**Fichier:** `src/ui/keybindings/mod.rs`

```rust
#[test]
fn test_k_vs_ctrl_k_disambiguation() {
    // Test 'k' sans modificateur → package
    let k_event = KeyEvent {
        code: KeyCode::Char('k'),
        modifiers: KeyModifiers::NONE,
        // ...
    };
    handle_key_event(k_event, &mut state);
    // Vérifie que package a été déclenché
    
    // Test 'Ctrl+K' → cache refresh
    let ctrl_k_event = KeyEvent {
        code: KeyCode::Char('k'),
        modifiers: KeyModifiers::CONTROL,
        // ...
    };
    handle_key_event(ctrl_k_event, &mut state);
    // Vérifie que package n'a PAS été déclenché
}
```

---

## Impact

### Fichiers Modifiés
1. `src/ui/keybindings/command_keys.rs`
   - Import de `KeyModifiers`
   - Ajout de `has_modifiers` check
   - Application du guard à toutes les commandes simples

2. `src/ui/keybindings/mod.rs`
   - Ajout du test `test_k_vs_ctrl_k_disambiguation`

### Tests
- **Avant:** 282 tests
- **Après:** 283 tests
- **Résultat:** ✅ Tous passants

### Keybindings Affectés
| Touche | Sans modificateur | Avec Ctrl | Comportement |
|--------|-------------------|-----------|--------------|
| `k` | ✅ Package | ❌ Ignoré | Correct |
| `Ctrl+K` | ❌ Ignoré | ✅ Refresh caches | **FIXÉ** |
| `b` | ✅ Build | ❌ Ignoré | Correct |
| `t` | ✅ Test | ❌ Ignoré | Correct |
| `i` | ✅ Install | ❌ Ignoré | Correct |
| `d` | ✅ Dependencies | ❌ Ignoré | Correct |
| `s` | ✅ Run starter | ❌ Ignoré | Correct |
| `S` | ❌ Ignoré | ✅ Starter manager | Correct |

---

## Recommandations Futures

### Pattern à Suivre
Toujours vérifier les modificateurs pour les commandes simples :

```rust
KeyCode::Char('x') if !has_modifiers => {
    // Commande simple
}
```

### Exceptions
Commandes avec modificateurs explicites gèrent eux-mêmes :

```rust
KeyCode::Char('X') if key.modifiers.contains(CONTROL | SHIFT) => {
    // Commande avec Ctrl+Shift+X
}
```

### Ordre de Priorité
L'ordre dans `mod.rs` reste important :
1. Popups (plus spécifiques)
2. Commandes Maven (simples, sans modificateurs)
3. Popups triggers (avec modificateurs Ctrl+...)
4. Navigation, etc.

---

## Documentation Mise à Jour

### À Mettre à Jour
- ✅ Ce document (fix interne)
- ⏳ `docs/QUICK_WINS.md` - Supprimer "Ctrl+K conflict" si présent
- ⏳ `docs/ROADMAP_ANALYSIS.md` - Marquer comme résolu
- ⏳ README si mention du conflit

---

## Validation

### Checklist
- ✅ Code compile sans warning
- ✅ Tous les tests passent (283/283)
- ✅ Test spécifique ajouté
- ✅ Comportement vérifié manuellement
- ✅ Documentation créée

### Test Manuel
1. Lancer: `cargo run -- --project demo/multi-module`
2. Appuyer sur `k` → Maven package démarre
3. Appuyer sur `Ctrl+K` → Message "Refreshing caches..." apparaît
4. Vérifier que les deux fonctionnent indépendamment

---

## Conclusion

✅ **Conflit résolu** de manière élégante avec vérification des modificateurs  
✅ **Pattern réutilisable** pour éviter futurs conflits  
✅ **Tests exhaustifs** (283 tests passants)  
✅ **Zéro régression** - Toutes les commandes existantes fonctionnent  

**Leçon apprise:** Toujours vérifier les modificateurs de touches lors de l'ajout de nouveaux keybindings avec Ctrl/Alt/Shift.
