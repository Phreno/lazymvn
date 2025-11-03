# Fix: Yank Debug Logs (Y) était cassé

**Date**: 3 novembre 2025  
**Bug**: La fonctionnalité `Y` (Shift+y) qui copie les logs de debug était quasi vide  
**Cause**: Fonction `collect_logs()` retournait un placeholder au lieu des vrais logs  
**Fix**: Utiliser `utils::logger::get_current_session_logs()` pour récupérer les vrais logs

## Le Problème

L'utilisateur a signalé: "mes rapports de debug sont quasi vide"

Quand l'utilisateur appuyait sur `Y` dans le TUI pour yank (copier) les logs de debug, le rapport copié dans le presse-papiers contenait seulement:

```
=== Recent Logs ===
(Check ~/.local/share/lazymvn/logs/ for full logs)
```

Au lieu de contenir **les vrais logs de la session courante**.

## Investigation

### Code Cassé

Dans `src/ui/state/utilities.rs`, ligne 125-131 :

```rust
/// Collect recent log entries
fn collect_logs() -> Vec<String> {
    vec![
        "=== Recent Logs ===".to_string(),
        "(Check ~/.local/share/lazymvn/logs/ for full logs)".to_string(),
        String::new(),
    ]
}
```

Cette fonction **ne récupérait PAS les logs**, elle retournait juste un message placeholder !

### Fonctionnalité Existante Non-Utilisée

Il existe déjà une fonction dans `utils::logger` qui fait exactement ce qu'on veut:

```rust
pub fn get_current_session_logs() -> Result<String, String>
```

Cette fonction:
- ✅ Récupère le session ID courant
- ✅ Extrait les logs de debug du fichier
- ✅ Extrait les logs d'erreur du fichier
- ✅ Retourne tout formaté avec headers

## La Solution

### 1. Fix du Code

**Fichier**: `src/ui/state/utilities.rs`

```rust
/// Collect recent log entries from current session
fn collect_logs() -> Vec<String> {
    let mut info = Vec::new();
    info.push("=== Recent Logs ===".to_string());
    
    // Get logs from current session
    match crate::utils::logger::get_current_session_logs() {
        Ok(logs) => {
            // Session logs already include headers, just add them
            info.push(logs);
        }
        Err(e) => {
            info.push(format!("Error retrieving session logs: {}", e));
            info.push("(Check ~/.local/share/lazymvn/logs/ for full logs)".to_string());
        }
    }
    
    info.push(String::new());
    info
}
```

### 2. Amélioration du Flush

**Fichier**: `src/utils/logger.rs`

Ajout d'un flush explicite avant de lire les logs:

```rust
pub fn get_current_session_logs() -> Result<String, String> {
    let session_id = get_session_id().ok_or("No session ID available")?;

    // Force flush all pending logs before reading
    log::logger().flush();
    
    // Give a tiny moment for filesystem to sync
    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut all_logs = Vec::new();
    // ... reste du code
}
```

Cela garantit que tous les logs en mémoire sont écrits sur disque avant qu'on les lise.

## Tests

### Test Automatisé Créé

**Fichier**: `crates/lazymvn-test-harness/tests/logger_tests.rs`

Test `test_yank_debug_info_simulation` qui simule exactement ce qui se passe quand l'utilisateur appuie sur `Y`:

```rust
#[test]
fn test_yank_debug_info_simulation() {
    // 1. Initialize logger
    logger::init(Some("debug"));
    
    // 2. Log some activity
    log::info!("YANK_TEST_User opened project");
    log::debug!("YANK_TEST_Loading modules");
    log::info!("YANK_TEST_Running build command");
    
    // 3. Flush and collect
    log::logger().flush();
    let logs = logger::get_current_session_logs().unwrap();
    
    // 4. Verify all logs are present
    assert!(logs.contains("YANK_TEST_User opened project"));
    assert!(logs.contains("YANK_TEST_Loading modules"));
    assert!(logs.contains("YANK_TEST_Running build command"));
}
```

**Résultat**: ✅ Test passe - 1266 bytes de logs capturés

### Tests Complets

7 tests créés pour la fonctionnalité de logging:

1. ✅ `test_logger_initialization` - Logger s'initialise correctement
2. ✅ `test_get_current_session_logs` - Récupération des logs de session
3. ✅ `test_log_file_paths` - Chemins des fichiers de logs corrects
4. ✅ `test_logger_with_different_levels` - Différents niveaux de log
5. ✅ `test_debug_log_file_exists_after_init` - Fichier créé après init
6. ✅ `test_full_logging_workflow` - Workflow complet de logging
7. ✅ `test_yank_debug_info_simulation` - **TEST CRITIQUE** - Simule `Y` dans TUI

Tous les tests passent : `test result: ok. 7 passed`

## Validation Manuelle

Pour tester manuellement:

```bash
# 1. Lancer lazymvn avec debug
cargo run -- --debug -p demo/multi-module

# 2. Dans le TUI, faire quelques actions (naviguer, build, etc.)

# 3. Appuyer sur 'Y' (Shift+y)

# 4. Coller le contenu du presse-papiers dans un fichier

# 5. Vérifier que la section "=== Recent Logs ===" contient:
#    - Session ID
#    - Logs de debug de la session
#    - Logs d'erreur (si présents)
```

**Avant le fix**:
```
=== Recent Logs ===
(Check ~/.local/share/lazymvn/logs/ for full logs)
```
(~50 bytes)

**Après le fix**:
```
=== Recent Logs ===
=== LazyMVN Session Logs ===
Session ID: 20251103-163543-962
Timestamp: 2025-11-03 16:35:44

=== Debug Logs ===
[SESSION:20251103-163543-962] [2025-11-03 16:35:44.062] INFO - User opened project
[SESSION:20251103-163543-962] [2025-11-03 16:35:44.062] DEBUG - Loading modules
[SESSION:20251103-163543-962] [2025-11-03 16:35:44.062] INFO - Running build command
... (tous les logs de la session)

=== Error Logs ===
(No errors for this session)
```
(~1266+ bytes avec vrais logs)

## Impact

### Avant le Fix

❌ Yank debug info (`Y`) **inutile** - ne contenait qu'un message  
❌ Impossible de copier les logs pour débugger  
❌ Utilisateur devait manuellement aller chercher les fichiers de logs

### Après le Fix

✅ Yank debug info (`Y`) **fonctionnel** - contient tous les logs de session  
✅ Debug facile - copier/coller les logs directement  
✅ Utile pour rapporter des bugs avec contexte complet

## Fichiers Modifiés

1. ✅ `src/ui/state/utilities.rs` - Fix de `collect_logs()`
2. ✅ `src/utils/logger.rs` - Ajout flush dans `get_current_session_logs()`
3. ✅ `crates/lazymvn-test-harness/Cargo.toml` - Ajout dépendance `chrono`
4. ✅ `crates/lazymvn-test-harness/tests/logger_tests.rs` - Tests complets

## Conclusion

Le bug était simple : une fonction placeholder qui n'a jamais été implémentée.

La solution est également simple : utiliser la fonctionnalité existante `get_current_session_logs()`.

✅ **Fix validé par tests automatisés**  
✅ **Prêt pour merge**  
✅ **Prévient futures régressions grâce aux tests**
