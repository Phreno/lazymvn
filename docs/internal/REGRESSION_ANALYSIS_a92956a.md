# Analyse des Régressions depuis a92956a

**Date**: 3 novembre 2025  
**Commit Stable**: `a92956a`  
**Commit Actuel**: `64f5e81`  
**Nombre de commits**: 48

## 🔴 DIAGNOSTIC: Refactoring Majeur Cassé

### Commit Problématique Identifié

**Commit**: `8ce63a8` - "Phase 4: Extract maven-command-builder"  
**Date**: 1er novembre 2025  
**Impact**: ⚠️ **BREAKING CHANGE** - Refactoring structurel majeur

### Ce Qui a Été Fait

Le commit `8ce63a8` a **extrait** le code Maven en **3 nouvelles bibliothèques** :

1. **`maven-command-builder`** - Construction/exécution commandes Maven
2. **`maven-log-analyzer`** - Parsing et analyse logs  
3. **`maven-log-colorizer`** - Colorisation output

**Problème**: Le refactoring a cassé la fonctionnalité existante.

### Fichiers Éclatés

**AVANT** (`a92956a`):
```
src/maven/command.rs     (1 fichier, tout marche)
src/maven/detection.rs   (1 fichier, tout marche)
```

**APRÈS** (`8ce63a8`):
```
crates/maven-command-builder/
  ├── src/builder.rs       (469 lignes)
  ├── src/executor.rs      (118 lignes)
  └── src/lib.rs           (132 lignes)

src/maven/command/
  ├── mod.rs
  ├── builder.rs           (nouveau)
  ├── executor.rs          (nouveau)
  ├── helpers.rs           (nouveau)
  └── log4j_config.rs      (nouveau)

src/maven/detection/
  ├── mod.rs
  ├── strategy.rs          (nouveau)
  ├── xml_parser.rs        (nouveau)
  ├── spring_boot.rs       (nouveau)
  └── command_builder.rs   (nouveau)
```

## Problèmes Signalés par l'Utilisateur

1. ❌ **Logs perdus** - "j'ai perdu des logs"
2. ❌ **Build fonctionne mal** - "mes build fonctionnent mal"
3. ❌ **Start ne fonctionne pas** - "mes starts fonctionnent mal voire pas du tout"

## Causes Racines Probables

### 1. Logs Perdus

**Cause**: Le nouveau `maven-command-builder/executor.rs` a probablement:
- ❌ Cassé la capture de stdout/stderr
- ❌ Perdu le code de streaming des logs
- ❌ Cassé le `BufReader` qui lisait ligne par ligne

**Ancien code** (`a92956a` - command.rs):
```rust
let mut child = command
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().unwrap();
let stderr = child.stderr.take().unwrap();

// Spawn threads to read stdout/stderr
// ... code qui marche ...
```

**Nouveau code** (`executor.rs`):
```rust
// Probablement manque le code de streaming
// ou a un bug dans la capture
```

### 2. Build Fonctionne Mal

**Cause**: Séparation `builder.rs` / `executor.rs` a probablement:
- ❌ Perdu les flags Maven (`-pl`, `--also-make`)
- ❌ Cassé la construction de la commande
- ❌ Perdu les profils

**Indices**:
- Fonction `run_selected_module_command_with_options` marquée `dead_code`
- Nouvelle hiérarchie de fonctions mal intégrée

### 3. Start Ne Fonctionne Pas

**Cause**: Code Spring Boot detection déplacé dans `detection/spring_boot.rs`:
- ❌ Import cassé
- ❌ Logique de fallback exec:java perdue
- ❌ JVM args mal passés

## Solution: REVERT du Refactoring

### Option 1: Revert Complet (RECOMMANDÉ)

```bash
# Revenir au dernier état stable
git revert 8ce63a8..HEAD --no-commit
git commit -m "revert: roll back Phase 4 refactoring (broken)"

# OU simplement
git reset --hard a92956a
git push origin develop --force  # ATTENTION: force push!
```

### Option 2: Cherry-pick des Bons Commits

```bash
# Partir de a92956a
git checkout a92956a -b fix/regression-recovery

# Cherry-pick SEULEMENT les commits utiles (pas le refactoring)
git cherry-pick 03ec1f9  # feat: visual feedback
git cherry-pick b951674  # fix: detect build failures
git cherry-pick 0dd615c  # test: integration tests
git cherry-pick 3bd6a44  # fix: build_launch_command
git cherry-pick 64f5e81  # feat: purge command

# Test
cargo test
cargo run -- -p demo/multi-module

# Si ça marche, push
git push origin fix/regression-recovery
```

### Option 3: Fix Incrémental (LONG)

Réparer chaque problème un par un:

1. ✅ Restaurer l'ancien `command.rs`
2. ✅ Garder seulement les bons fixes (visual feedback, purge)
3. ✅ Supprimer les crates `maven-*`
4. ✅ Tests complets

**Estimation**: 2-3 heures

## Recommandations
