# LazyMVN Roadmap - Analyse et Recommandations

**Date:** 29 octobre 2025  
**Analyse basée sur:** Documentation complète (README, CHANGELOG, CONTRIBUTING, docs/user, docs/internal, docs/ideas)

---

## 📊 État Actuel du Projet

### ✅ Fonctionnalités Principales Implémentées

1. **Interface TUI** (LazyGit-style)
   - Navigation par panneaux (Projects, Modules, Profiles, Flags, Output)
   - Support souris et clavier
   - Layout adaptatif (terminal étroit/court)
   - Thèmes et colorisation

2. **Support Multi-Projets**
   - 10 onglets simultanés max
   - État indépendant par onglet
   - Gestion des projets récents (20 max)
   - Isolation des processus Maven

3. **Intégration Maven**
   - Commandes principales (build, test, package, install, clean)
   - Support multi-modules (`-pl` automatique)
   - Profils Maven interactifs
   - Flags personnalisables (built-in + custom via config)
   - Détection Maven wrapper (`mvnw`)

4. **Spring Boot**
   - Détection intelligente des starters
   - Sélection fuzzy search
   - Manager de starters (`Ctrl+Shift+S`)
   - Stratégie de lancement auto/force (`spring-boot:run` vs `exec:java`)
   - Support WAR modules (classpath `provided`)

5. **Performance & Caching**
   - Cache des modules (POM hash)
   - Cache des profils Maven (par projet)
   - Cache des starters (par projet)
   - Refresh manuel (`Ctrl+K`)

6. **Configuration**
   - `lazymvn.toml` par projet
   - Live reload (`Ctrl+E`)
   - Flags Maven personnalisés
   - Configuration logging (Log4j, Logback, SLF4J)
   - Overrides Spring Boot properties
   - Watch mode (auto-rebuild)

7. **Workflow**
   - Historique des commandes (`Ctrl+H`)
   - Favoris (`Ctrl+F`, `Ctrl+S`)
   - Recherche dans l'output (`/`, `n`, `N`)
   - Yank output (`y`)
   - Debug report (`Y` - Shift+Y)
   - Help popup (`?`)

8. **Qualité & Maintenabilité**
   - 282 tests unitaires passants
   - Architecture modulaire (22 modules créés en 6 phases)
   - Documentation exhaustive (user + internal)
   - Scripts de test

---

## 🎯 Axes d'Amélioration Identifiés

### 🔴 Priorité HAUTE - Fonctionnalités Manquantes Critiques

#### 1. **Gradle Support** ⭐⭐⭐
**Problème:** LazyMVN est exclusivement Maven, limitant son adoption

**Solution proposée:**
- Détection automatique (`build.gradle`, `settings.gradle`)
- Commandes Gradle équivalentes (`build`, `test`, `bootRun`, etc.)
- Support Gradle wrapper (`gradlew`)
- Cache des tâches Gradle (`gradle tasks --all`)
- Architecture: Ajouter `src/gradle/` parallèle à `src/maven/`

**Impact utilisateur:** Ouvre LazyMVN à ~50% des projets JVM (Gradle vs Maven)

**Effort estimé:** 3-4 semaines (architecture modulaire facilite l'ajout)

---

#### 2. **Automatic Cache Invalidation** ⭐⭐⭐
**Problème actuel:** Utilisateur doit manuellement `Ctrl+K` après changements POM

**Documentation identifiée:**
- `docs/internal/CACHING_IMPLEMENTATION.md` ligne 245: "Auto-refresh on POM change"
- `docs/internal/CACHING_IMPLEMENTATION.md` ligne 253: "No auto-invalidation: User must press Ctrl+K"

**Solution proposée:**
- File watcher sur `pom.xml` (déjà existant pour `lazymvn.toml`)
- Invalidation automatique des caches (profiles, starters, modules)
- Notification discrète: "POM changed, caches refreshed"

**Impact utilisateur:** Expérience transparente, zéro intervention manuelle

**Effort estimé:** 1 semaine (réutiliser `src/utils/watcher.rs`)

---

#### 3. **Legacy Insights Dashboard** ⭐⭐⭐
**Idée documentée:** `docs/ideas/LEGACY_INSIGHTS.md`

**Objectif:** 
- Donner visibilité sur les modules "risqués" (legacy debt)
- Metrics: complexité, taille POM, taux d'échec builds, dépendances
- Badges: "High fan-in", "Flaky", "Large POM"
- Cache: `~/.config/lazymvn/insights/<hash>.json`

**Cas d'usage:**
- Onboarding: nouveaux devs identifient modules problématiques
- Refactoring: priorisation data-driven
- Planification: estimation de l'effort de cleanup

**Impact utilisateur:** Améliore maintenabilité des gros projets legacy

**Effort estimé:** 3-4 semaines

**Architecture suggérée:**
```
src/
├── analysis/              # NEW module
│   ├── mod.rs
│   ├── complexity.rs      # Metrics calculation
│   ├── insights.rs        # LegacyInsight struct
│   └── cache.rs           # Insights caching
├── ui/
│   ├── panes/
│   │   └── insights.rs    # NEW pane (Insights tab)
```

---

#### 4. **Error Telemetry & Package Noisiness** ⭐⭐
**Idée documentée:** `docs/ideas/LEGACY_INSIGHTS.md` ligne 57 (Future Enhancements)

**Objectif:**
- Parser l'output en temps réel
- Compter erreurs/warnings par package
- Afficher "noisiest packages" breakdown

**Cas d'usage:**
- Debugging: identifier rapidement quel layer/package échoue
- Qualité: metrics sur les packages "bruyants"

**Impact utilisateur:** Debugging plus rapide

**Effort estimé:** 2 semaines

---

### 🟡 Priorité MOYENNE - Amélioration UX/Performance

#### 5. **Help Popup Enhancements** ⭐⭐
**Documentation:** `docs/internal/HELP_POPUP_IMPLEMENTATION.md` ligne 120

**Améliorations suggérées:**
- Scrolling si contenu > viewport
- Recherche interne dans le help (`/`)
- Tips contextuels basés sur la vue active
- Indicateurs visuels pour modificateurs (Ctrl, Shift)

**Impact utilisateur:** Meilleure découvrabilité

**Effort estimé:** 1 semaine

---

#### 6. **Cache Statistics & Debugging** ⭐⭐
**Documentation:** `docs/internal/CACHING_IMPLEMENTATION.md` ligne 247, 294

**Améliorations suggérées:**
- Afficher l'âge des caches dans le status bar
- Commande pour voir les stats (`Ctrl+Shift+C` ?)
- TTL pour caches périmés (24h configurable)
- Cleanup automatique des vieux caches projet

**Impact utilisateur:** Transparence et contrôle

**Effort estimé:** 1-2 semaines

---

#### 7. **Background Cache Refresh** ⭐⭐
**Documentation:** `docs/internal/CACHING_IMPLEMENTATION.md` ligne 248

**Objectif:**
- Rafraîchir caches en arrière-plan (sans bloquer UI)
- Notification discrète quand refresh terminé

**Impact utilisateur:** Moins d'attente

**Effort estimé:** 1 semaine (async déjà utilisé pour profiles)

---

#### 8. **Export Capabilities** ⭐
**Documentation:** `docs/ideas/LEGACY_INSIGHTS.md` ligne 52

**Fonctionnalités:**
- Export insights en Markdown/JSON
- Export historique des commandes
- Export configuration active (pour partage)

**Cas d'usage:**
- Rétrospectives d'équipe
- Documentation projet
- Partage de config entre devs

**Impact utilisateur:** Collaboration améliorée

**Effort estimé:** 1 semaine

---

#### 9. **Test File Support** ⭐
**Problème identifié:** Pas de raccourci pour exécuter un seul test

**Solution proposée:**
- Détection de fichier test actuel (IDE integration ?)
- Raccourci pour `mvn test -Dtest=ClassName`
- Support fuzzy search pour sélection de tests

**Impact utilisateur:** TDD workflow plus rapide

**Effort estimé:** 2 semaines

---

### 🟢 Priorité BASSE - Nice-to-Have

#### 10. **Custom Badge Rules** ⭐
**Documentation:** `docs/ideas/LEGACY_INSIGHTS.md` ligne 55

**Objectif:**
- Règles personnalisables dans `lazymvn.toml`
- Définir ce qui constitue "legacy" dans le contexte équipe

**Effort estimé:** 1 semaine

---

#### 11. **Trend Graphs** ⭐
**Documentation:** `docs/ideas/LEGACY_INSIGHTS.md` ligne 54

**Objectif:**
- Graphes ASCII dans le terminal
- Évolution de la santé des modules release-to-release

**Effort estimé:** 2-3 semaines

---

#### 12. **IDE Integration** ⭐
**Idée:** 
- Plugin VSCode/IntelliJ pour lancer LazyMVN sur fichier actuel
- Commandes contextuelles (Run test, Run module)

**Effort estimé:** 4-6 semaines (nécessite expertise plugins)

---

## 🔧 Tâches Techniques Identifiées

### Phase 7.1: Split `maven_tests.rs` ✅ PLANIFIÉ
**Documentation:** `docs/internal/PHASE_7.1_PLAN.md`

**Objectif:** 957 lignes → 8 modules
- `tests/maven/command_tests.rs`
- `tests/maven/profile_tests.rs`
- `tests/maven/detection_tests.rs`
- `tests/maven/launcher_tests.rs`
- `tests/maven/platform_tests.rs`
- `tests/maven/module_tests.rs`

**Statut:** Plan complet, prêt à exécuter

**Effort estimé:** 2-3 heures

---

### Test Coverage Improvements ✅ EN COURS
**Documentation:** `docs/internal/TEST_COVERAGE_ANALYSIS.md`, `TEST_COVERAGE_PROGRESS.md`

**Priorités:**
1. `src/maven/command.rs` (571 lignes, 0% testé)
2. `src/core/config.rs` (773 lignes, partiellement testé)
3. `src/ui/state/mod.rs` (1,694 lignes, tests state management seulement)

**Effort estimé:** 2-3 semaines (sprint dédié)

---

## 📋 Incohérences Détectées

### 1. **Documentation `--debug` Flag** ⚠️
**Problème:** Plusieurs docs mentionnent "~/.local/share/lazymvn/logs/debug.log" dans le répertoire courant

**Réalité:** Logs sont maintenant dans `~/.local/share/lazymvn/logs/debug.log`

**Fichiers à corriger:**
- `README.md` lignes 380, 383, 508
- `docs/user/CUSTOM_FLAGS.md` ligne 289
- Plusieurs autres fichiers

**Action:** Mise à jour globale de la documentation

---

### 2. **Limite d'Onglets Non Appliquée** ⚠️
**Documentation:** `docs/internal/TABS_PROPOSAL.md` ligne 197: "Limitation du nombre d'onglets"

**Réalité:** README dit "up to 10 projects simultaneously" mais pas de check dans le code ?

**Action:** Vérifier implémentation ou documenter limite actuelle

---

### 3. **Cache Cleanup Non Implémenté** ⚠️
**Documentation:** `docs/internal/CACHING_IMPLEMENTATION.md` ligne 255

**Problème:** "Old project caches accumulate over time"

**Impact:** `~/.config/lazymvn/` peut grossir indéfiniment

**Action:** Implémenter cleanup basé sur dernier accès (ex: supprimer caches > 30 jours non utilisés)

---

### 4. **Gradle Mentionné Nulle Part** ⚠️
**Problème:** LazyMVN est présenté comme "Maven TUI" sans mention de limitation

**Suggestion:** 
- README devrait mentionner "Maven-only (Gradle support planned)"
- Ou ajouter FAQ "Does it work with Gradle?"

---

## 🗺️ Roadmap Proposée

### Q1 2025 - Focus Stabilité & Performance

**Objectifs:**
- ✅ Help popup (FAIT)
- ✅ Cache profiles/starters (FAIT)
- ✅ Custom flags (FAIT)
- 🔄 Phase 7.1: Split maven_tests.rs (EN COURS)
- 🔄 Auto cache invalidation (POM watch)
- 🔄 Cache cleanup & statistics
- 🔄 Corrections incohérences documentation

**Livrable:** v0.4.0 - "Polish Release"

---

### Q2 2025 - Legacy Insights & Quality

**Objectifs:**
- Legacy Insights Dashboard (MVP)
- Error Telemetry & Package Noisiness
- Test Coverage Sprint (+50% coverage)
- Export capabilities (Markdown/JSON)

**Livrable:** v0.5.0 - "Analytics Release"

---

### Q3 2025 - Gradle Support

**Objectifs:**
- Gradle detection & commands
- Gradle wrapper support
- Gradle task caching
- Documentation Gradle-specific

**Livrable:** v0.6.0 - "Multi-Build Tool Release"

---

### Q4 2025 - IDE Integration & Advanced Features

**Objectifs:**
- VSCode plugin (MVP)
- Test file support (single test execution)
- Background cache refresh
- Help popup enhancements (scrolling, search)

**Livrable:** v0.7.0 - "Integration Release"

---

### 2026+ - Enterprise Features

**Objectifs:**
- IntelliJ IDEA plugin
- Trend graphs (health evolution)
- Custom badge rules
- CI/CD integration (exports for pipelines)
- Distributed cache (team sharing)

**Livrable:** v1.0.0 - "Enterprise Ready"

---

## 📊 Priorisation par Impact/Effort

### Quick Wins (High Impact, Low Effort)
1. **Auto cache invalidation** - 1 semaine, impactant
2. **Cache statistics** - 1 semaine, visible
3. **Help popup enhancements** - 1 semaine, UX
4. **Phase 7.1 split tests** - 3 heures, maintenabilité

### Strategic (High Impact, Medium Effort)
1. **Legacy Insights** - 4 semaines, différenciateur majeur
2. **Error Telemetry** - 2 semaines, debugging
3. **Test file support** - 2 semaines, TDD workflow

### Major Initiatives (High Impact, High Effort)
1. **Gradle support** - 4 semaines, double la base utilisateurs
2. **IDE plugins** - 6+ semaines, adoption professionnelle

### Nice-to-Have (Low Priority)
1. Trend graphs
2. Custom badge rules
3. Export en JSON (sauf si demandé)

---

## 🎬 Recommandations Immédiates

### Court Terme (1-2 semaines)

1. **Corriger incohérences documentation** (1 jour)
   - Paths de debug log
   - Limite d'onglets
   - Mentions Gradle

2. **Phase 7.1: Split maven_tests.rs** (3 heures)
   - Plan déjà complet
   - Améliore maintenabilité immédiatement

3. **Auto cache invalidation** (1 semaine)
   - Réutilise file watcher existant
   - Élimine `Ctrl+K` manuel
   - UX majeur improvement

4. **Cache cleanup** (2 jours)
   - Prévient accumulation
   - Basé sur dernier accès (30 jours)

### Moyen Terme (1-2 mois)

5. **Legacy Insights Dashboard MVP** (4 semaines)
   - Fonctionnalité différenciatrice
   - Cas d'usage legacy évident
   - Plan déjà documenté

6. **Cache statistics** (1 semaine)
   - Visibilité sur perf
   - Debugging facilité

7. **Error Telemetry** (2 semaines)
   - Complète Legacy Insights
   - Debugging plus rapide

### Long Terme (3-6 mois)

8. **Gradle Support** (4 semaines)
   - Expansion base utilisateurs
   - Architecture modulaire prête

9. **Test Coverage Sprint** (3 semaines)
   - Cible 70% coverage
   - Focus: maven/command.rs, core/config.rs

10. **VSCode Plugin MVP** (6 semaines)
    - Adoption pro
    - Nécessite expertise plugins

---

## 🔍 Métriques de Succès Proposées

### Performance
- Temps de startup: < 1s (avec cache) ✅
- Temps de refresh cache: < 5s ✅
- Réactivité UI: < 50ms ✅

### Qualité
- Test coverage: 70% (actuel: ~40%)
- Clippy warnings: 0 ✅
- Documentation: 100% fonctionnalités documentées ✅

### Adoption
- GitHub stars: 100+ (actuel: ?)
- Issues/mois: < 10 bugs, > 5 features requests
- Communauté: 5+ contributeurs actifs

### Features
- Gradle support: v0.6.0 (Q3 2025)
- Legacy Insights: v0.5.0 (Q2 2025)
- IDE integration: v0.7.0 (Q4 2025)

---

## 💡 Conclusion

### Points Forts Actuels
✅ Architecture modulaire solide (22 modules)  
✅ Couverture fonctionnelle Maven exhaustive  
✅ Performance excellente (caching)  
✅ Documentation exemplaire (user + internal)  
✅ UX polie (keybindings, help, workflow)

### Opportunités Majeures
🎯 **Gradle support** - doubler la base utilisateurs  
🎯 **Legacy Insights** - différenciateur unique  
🎯 **IDE integration** - adoption professionnelle  
🎯 **Auto-invalidation** - UX transparent  

### Risques à Mitiger
⚠️ Accumulation technique (cache cleanup)  
⚠️ Documentation désynchronisée (debug log paths)  
⚠️ Test coverage insuffisant (40% vs 70% target)  

### Recommandation Globale

**Prioriser dans l'ordre:**
1. **Stabilité** (fixes documentation, cache cleanup) - 1 semaine
2. **UX Polish** (auto-invalidation, cache stats) - 2 semaines
3. **Différenciation** (Legacy Insights) - 1 mois
4. **Expansion** (Gradle support) - 1 mois
5. **Professionnalisation** (IDE plugin) - 2 mois

**Timeline réaliste:** 6 mois pour passer de v0.4 à v0.7 avec impact majeur.

---

**Auteur:** Analyse automatisée de la documentation LazyMVN  
**Date:** 29 octobre 2025  
**Fichiers analysés:** 142 fichiers .md, 282 tests, architecture complète
