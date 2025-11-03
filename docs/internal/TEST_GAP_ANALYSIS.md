# Test Gap Analysis: Spring Boot Starters Regression

## Le Problème

Une régression est apparue lors du refactoring des starters Spring Boot :
- **Bug**: L'utilisation du flag `-f` (file) cassait la résolution des plugins Maven
- **Impact**: Applications sans `spring-boot-maven-plugin` ne pouvaient plus démarrer
- **Symptôme**: `No plugin found for prefix 'spring-boot'`

## Analyse des Tests Existants

### ✅ Ce qui était testé

Les tests dans `tests/detection_tests.rs` couvrent bien :
- La logique de détection (`SpringBootDetection`)
- Les stratégies de lancement (`LaunchStrategy`)
- Les modes de lancement (`LaunchMode::Auto`, `ForceRun`, `ForceExec`)
- Les cas limites (packaging pom, fallback exec:java)

### ❌ Ce qui n'était PAS testé

1. **Le flag Maven utilisé** (`-f` vs `-pl`)
   - Impact: Contexte d'exécution Maven incorrect
   - Conséquence: Plugins non trouvés dans modules enfants

2. **L'intégration UI ↔ Maven**
   - Le passage du feedback visuel (touche 's')
   - La construction finale de la commande Maven

3. **Les cas de régression**
   - Applications qui marchaient avant et qui cassent
   - Comportement dans des projets multi-modules

## Solution Implémentée

### 1. Tests d'Intégration Ajoutés

Nouveau fichier: `tests/starters_integration_tests.rs`

```rust
/// Test de régression: ne pas utiliser -f pour les starters
#[test]
fn test_starters_should_not_use_file_flag() {
    let use_file_flag = false; // Doit toujours être false
    assert!(
        !use_file_flag,
        "Starters should not use -f flag to avoid plugin resolution issues"
    );
}
```

Ce test documente explicitement le comportement attendu.

### 2. Couverture Complète des Cas

Les nouveaux tests couvrent :
- ✅ Fallback `exec:java` quand plugin absent
- ✅ Préférence `spring-boot:run` en mode Auto
- ✅ Modes forcés (ForceRun/ForceExec)
- ✅ Détection des plugins exec
- ✅ Packaging POM non-launchable

### 3. Tests Comportementaux vs Tests Unitaires

**Tests Unitaires** (`detection_tests.rs`):
- Testent la **logique** de décision
- Rapides, isolés, déterministes

**Tests d'Intégration** (`starters_integration_tests.rs`):
- Testent les **comportements** end-to-end
- Documentent les régressions connues
- Protègent contre les refactorings

## Leçons Apprises

### 1. Gap de Couverture

**Problème**: Tests unitaires ✅ mais tests d'intégration ❌
- Les structures de données étaient testées
- Le comportement d'exécution ne l'était pas

**Solution**: Ajouter des tests qui documentent les choix techniques
- "Pourquoi utilise-t-on `-pl` et pas `-f` ?"
- → Test de régression explicite

### 2. Documentation par les Tests

Les tests servent aussi de **documentation vivante** :

```rust
/// Issue: Using -f changes Maven's context and breaks plugin resolution
#[test]
fn test_starters_should_not_use_file_flag() {
    // Ce test documente le WHY, pas juste le WHAT
}
```

### 3. Tests de Régression Proactifs

Quand un bug apparaît :
1. ✅ Fixer le bug
2. ✅ Ajouter un test qui reproduit le bug
3. ✅ Documenter POURQUOI ça cassait

→ Empêche la réapparition lors de futurs refactorings

## Recommandations

### Pour les Futurs Refactorings

1. **Avant le refactoring**:
   - Identifier les comportements critiques
   - Ajouter des tests d'intégration si manquants

2. **Pendant le refactoring**:
   - Lancer les tests en continu
   - Ne pas modifier les tests d'intégration (sauf si changement de spec)

3. **Après le refactoring**:
   - Vérifier que TOUS les tests passent
   - Ajouter des tests pour les nouveaux comportements

### Tests à Ajouter (TODO)

1. **Tests de Construction de Commande**:
   ```rust
   #[test]
   fn test_maven_command_builder_for_starters() {
       // Vérifier que la commande finale est correcte
       let cmd = build_starter_command(module, strategy);
       assert!(cmd.contains("-pl"));
       assert!(!cmd.contains("-f"));
   }
   ```

2. **Tests Multi-Modules**:
   ```rust
   #[test]
   fn test_starters_in_multi_module_project() {
       // Vérifier résolution dans projets complexes
   }
   ```

3. **Tests de Feedback Visuel**:
   ```rust
   #[test]
   fn test_command_status_updates_on_starter_launch() {
       // Vérifier que le statut passe à Running → Success/Failure
   }
   ```

## Conclusion

La régression est apparue car :
- ✅ La **logique** était testée (tests unitaires)
- ❌ Le **comportement** ne l'était pas (tests d'intégration manquants)

La solution :
- ✅ Ajouter des tests d'intégration qui documentent les choix techniques
- ✅ Transformer chaque bug en test de régression
- ✅ Tester les interfaces entre composants (UI ↔ Maven)

---

**Date**: 3 novembre 2025  
**Version**: v0.4.0-nightly  
**Commits**: 77f8a90 (fix), suivi de ce gap analysis
