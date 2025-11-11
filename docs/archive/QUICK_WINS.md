# Quick Wins - Améliorations Immédiates

**Date:** 29 octobre 2025  
**Objectif:** Actions à impact élevé, effort faible (< 2 semaines)

---

## 🚀 Actions Prioritaires

### 1. Auto Cache Invalidation (1 semaine) ⭐⭐⭐

**Problème actuel:**
- Utilisateur doit manuellement faire `Ctrl+K` après modification POM
- Expérience frustrante si oublié → caches périmés

**Solution:**
- File watcher sur `pom.xml` (infrastructure déjà existante pour `lazymvn.toml`)
- Invalidation automatique des caches: profiles, starters, modules
- Notification discrète: "POM changed, refreshing caches..."

**Implémentation:**
```rust
// Dans src/utils/watcher.rs (déjà existant)
patterns.push("**/pom.xml");

// Dans src/ui/state/mod.rs
fn handle_pom_change(&mut self) {
    log::info!("POM changed, invalidating caches");
    self.refresh_caches();
    self.output_buffer.push("📦 POM changed - caches refreshed".to_string());
}
```

**Impact:**
- ✅ Expérience transparente (zero configuration)
- ✅ Élimine erreurs humaines (oubli de `Ctrl+K`)
- ✅ Workflow plus fluide

**Effort:** 3-5 jours

---

### 2. Cache Cleanup (2 jours) ⭐⭐

**Problème actuel:**
- Caches s'accumulent indéfiniment dans `~/.config/lazymvn/`
- Projets supprimés/renommés laissent des caches orphelins

**Solution:**
- Cleanup au démarrage: supprimer caches > 30 jours non utilisés
- Métadonnées `last_accessed` dans les caches JSON
- Configuration optionnelle dans `lazymvn.toml`:

```toml
[cache]
cleanup_enabled = true
max_age_days = 30
```

**Implémentation:**
```rust
// src/core/config/types.rs
#[derive(Serialize, Deserialize)]
struct CacheMetadata {
    last_accessed: SystemTime,
    project_path: PathBuf,
}

// src/core/cache/cleanup.rs (NEW)
fn cleanup_old_caches(max_age_days: u64) {
    // Scan ~/.config/lazymvn/
    // Delete caches older than max_age_days
}
```

**Impact:**
- ✅ Prévient accumulation disque
- ✅ Maintenance automatique
- ✅ Config optionnelle (peut être désactivé)

**Effort:** 1-2 jours

---

### 3. Phase 7.1: Split maven_tests.rs (3 heures) ⭐⭐

**Problème actuel:**
- 1 fichier de 957 lignes
- Tests mélangés (command, profile, detection, launcher)
- Difficile à naviguer

**Solution:**
- Plan complet déjà dans `docs/internal/PHASE_7.1_PLAN.md`
- Split en 8 modules:
  ```
  tests/
  ├── common/mod.rs              (~50 lines)
  ├── maven/
  │   ├── command_tests.rs       (~150 lines)
  │   ├── profile_tests.rs       (~350 lines)
  │   ├── detection_tests.rs     (~250 lines)
  │   ├── launcher_tests.rs      (~200 lines)
  │   ├── platform_tests.rs      (~50 lines)
  │   └── module_tests.rs        (~100 lines)
  ```

**Impact:**
- ✅ Maintenabilité améliorée
- ✅ Tests plus faciles à trouver
- ✅ Pattern pour futurs tests

**Effort:** 2-3 heures (plan déjà complet)

---

### 4. Cache Statistics UI (3 jours) ⭐⭐

**Problème actuel:**
- Pas de visibilité sur l'état des caches
- Debugging difficile si problème cache

**Solution:**
- Nouveau popup: `Ctrl+Shift+C` → Cache Statistics
- Affiche:
  ```
  Cache Statistics
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Profiles Cache
    Status: ✅ Valid
    Age: 2 hours ago
    Size: 2.4 KB
    Entries: 12 profiles
  
  Starters Cache
    Status: ✅ Valid
    Age: 3 days ago
    Size: 4.1 KB
    Entries: 8 starters
  
  Modules Cache
    Status: ✅ Valid
    Age: 1 week ago
    Size: 1.2 KB
    Entries: 5 modules
  
  [Ctrl+K to refresh all] [Esc to close]
  ```

**Implémentation:**
```rust
// src/ui/state/mod.rs
pub show_cache_stats: bool,

pub fn show_cache_statistics(&mut self) {
    self.show_cache_stats = true;
}

// src/ui/panes/popups.rs
pub fn render_cache_stats_popup(f: &mut Frame, state: &TuiState) {
    // Display cache metadata
}
```

**Impact:**
- ✅ Transparence sur état des caches
- ✅ Debugging facilité
- ✅ Confiance utilisateur

**Effort:** 2-3 jours

---

### 5. Documentation Fixes (1 jour) ⭐

**Incohérences identifiées:**

1. **Debug log path incorrect**
   - **Docs disent:** `~/.local/share/lazymvn/logs/debug.log` (répertoire courant)
   - **Réalité:** `~/.local/share/lazymvn/logs/debug.log`
   - **Fichiers à corriger:**
     - `README.md` (lignes 380, 383, 508)
     - `docs/user/CUSTOM_FLAGS.md` (ligne 289)
     - Plusieurs autres

2. **Limite d'onglets floue**
   - **README dit:** "up to 10 projects simultaneously"
   - **Action:** Vérifier limite effective dans le code ou clarifier

3. **Gradle non mentionné**
   - **Problème:** Pas de mention "Maven-only"
   - **Action:** Ajouter FAQ ou disclaimer sur limitation actuelle

**Impact:**
- ✅ Documentation précise
- ✅ Moins de confusion utilisateurs
- ✅ Expectations claires

**Effort:** 4-6 heures

---

### 6. Help Popup Scrolling (2 jours) ⭐

**Problème actuel:**
- Help popup affiche tout le contenu
- Pas de scrolling si terminal trop petit
- Contenu peut être coupé

**Solution:**
- Rendre le help popup scrollable
- Afficher indicateur de scroll: "↓ More (2/5 pages)"
- Keybindings: `↑↓` pour scroller, `PgUp/PgDn` pour pages

**Implémentation:**
```rust
// src/ui/state/mod.rs
pub help_scroll_offset: usize,

// src/ui/panes/popups.rs
pub fn render_help_popup(f: &mut Frame, state: &TuiState) {
    let paragraph = Paragraph::new(help_lines)
        .scroll((state.help_scroll_offset as u16, 0));
    
    // Add scroll indicator
    if total_lines > visible_lines {
        let indicator = format!("↓ {} more lines", total_lines - visible_lines);
        // Render at bottom
    }
}
```

**Impact:**
- ✅ Help accessible sur petits terminaux
- ✅ UX améliorée
- ✅ Pas de contenu perdu

**Effort:** 1-2 jours

---

## 📊 Récapitulatif

| Action | Effort | Impact | Priorité |
|--------|--------|--------|----------|
| Auto Cache Invalidation | 1 semaine | 🔥 Très élevé | ⭐⭐⭐ |
| Cache Cleanup | 2 jours | 🔥 Élevé | ⭐⭐ |
| Phase 7.1 Split Tests | 3 heures | 🔥 Élevé | ⭐⭐ |
| Cache Statistics UI | 3 jours | 🔥 Moyen | ⭐⭐ |
| Documentation Fixes | 1 jour | 🔥 Moyen | ⭐ |
| Help Popup Scrolling | 2 jours | 🔥 Faible | ⭐ |

**Total effort:** ~2 semaines pour l'ensemble

**Impact global:**
- Expérience utilisateur grandement améliorée
- Maintenabilité code renforcée
- Documentation précise
- Confiance utilisateurs accrue

---

## 🎯 Ordre d'Exécution Recommandé

### Semaine 1
1. **Jour 1:** Documentation fixes (4-6h)
2. **Jours 1-2:** Cache cleanup (1-2 jours)
3. **Jour 2:** Phase 7.1 split tests (3 heures)
4. **Jours 3-5:** Auto cache invalidation (3 jours)

### Semaine 2
5. **Jours 1-3:** Cache statistics UI (3 jours)
6. **Jours 4-5:** Help popup scrolling (2 jours)

---

## ✅ Validation

Après chaque action:
1. ✅ Tous les tests passent (`cargo test`)
2. ✅ Zero clippy warnings (`cargo clippy -- -D warnings`)
3. ✅ Documentation mise à jour
4. ✅ Test manuel avec `demo/multi-module`
5. ✅ Commit atomique avec message clair

---

## 📈 Résultats Attendus

**Avant:**
- Caches manuels (`Ctrl+K`)
- Documentation parfois incorrecte
- Tests difficiles à naviguer
- Pas de visibilité sur caches

**Après:**
- ✅ Caches automatiques (transparent)
- ✅ Documentation précise à 100%
- ✅ Tests bien organisés
- ✅ Statistiques caches accessibles
- ✅ Help scrollable (petits terminaux)

**Impact qualitatif:**
- Moins de friction utilisateur
- Expérience plus "polie"
- Confiance accrue dans le produit
- Facilite contribution (tests organisés)

---

**Prochaine étape:** Commencer par la documentation (quick win, 1 jour)
