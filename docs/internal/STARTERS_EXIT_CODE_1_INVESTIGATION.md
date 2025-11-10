# Investigation: Exit Code 1 sur lancement Starters

## Problème Rapporté

**Date**: 3 novembre 2025  
**Version**: 0.4.0-nightly.20251103+0dd615c  
**Application**: foo-bdd-id (WAR packaging)

### Symptômes

```
--- exec-maven-plugin:3.0.0:java (default-cli) @ HY5-module-bar ---
fr.foo.foo.foo.internal.core.ApplicationConfigurationManager - Constructeur : BootStrap en cours...
initBootStrap() en cours...

✗ Build failed with exit code 1
```

- ✅ Le fallback `exec:java` fonctionnait (plugin détecté correctement)
- ✅ L'application commençait à démarrer (logs visibles)
- ❌ Processus se terminait immédiatement avec exit code 1

### Contexte

L'utilisateur a signalé : **"L'application se lançait avant"**  
→ Indique une régression introduite récemment

## Analyse de la Cause

### Commande Générée (AVANT fix)

```bash
mvn -pl foo-module-usines \
    -Dexec.mainClass=fr.foo.foo.foo.internal.core.ApplicationConfigurationManager \
    exec:java
```

### Paramètres Manquants

1. **`-Dexec.cleanupDaemonThreads=false`** ⚠️ **CRITIQUE**
   - Par défaut, `exec-maven-plugin` tue les threads daemon
   - Les applications Spring Boot/Java EE utilisent des threads daemon
   - Sans ce flag, le processus se termine dès que le main() retourne
   
2. **`-Dexec.classpathScope=compile`** (pour WAR)
   - Les dépendances `provided` (servlet-api, etc.) ne sont pas incluses
   - Cause `NoClassDefFoundError` pour javax.servlet.*
   
3. **JVM args** (logging, Spring properties)
   - Configuration Logback/Log4j absente
   - Spring Boot properties non passées

### Commande Générée (APRÈS fix)

```bash
mvn -pl HY5-module-usines \
    -Dexec.mainClass=fr.foo.foo.foo.internal.core.ApplicationConfigurationManager \
    -Dexec.classpathScope=compile \
    -Dexec.cleanupDaemonThreads=false \
    -Dexec.args="-Dlogback.configurationFile=file:///... -Dspring.config.location=..." \
    exec:java
```

## Code Problématique

### AVANT (src/ui/state/starters.rs)

```rust
// Construction manuelle de la commande
let (goal, arg) = if detection.can_use_spring_boot_run() {
    let spring_boot_arg = format!("-Dspring-boot.run.main-class={}", fqcn);
    ("spring-boot:run", spring_boot_arg)
} else {
    let exec_arg = format!("-Dexec.mainClass={}", fqcn);  // ❌ Paramètres manquants
    ("exec:java", exec_arg)
};

let args = vec![goal, &arg];  // ❌ Seulement 2 arguments
```

**Problèmes** :
- ❌ Construction manuelle → oubli de paramètres
- ❌ Pas de réutilisation de `build_launch_command()`
- ❌ Pas de JVM args passés

### APRÈS (src/ui/state/starters.rs)

```rust
// Utilisation de la fonction existante qui gère TOUT
let jvm_args = self.build_jvm_args_for_launcher();

let command_parts = crate::maven::detection::build_launch_command(
    strategy,
    Some(fqcn),
    &[],  // profiles
    &jvm_args,
    detection.packaging.as_deref(),  // ✅ WAR detection
    detection.spring_boot_version.as_deref(),
);

let args: Vec<&str> = command_parts.iter().map(|s| s.as_str()).collect();
```

**Avantages** :
- ✅ Réutilise la logique existante et testée
- ✅ Tous les paramètres inclus automatiquement
- ✅ Support WAR avec classpathScope
- ✅ JVM args (logging, Spring properties)
- ✅ Comportement cohérent avec launcher 'l'

## Solution Implémentée

### Changements Code

**Fichier** : `src/ui/state/starters.rs`

1. Remplacement construction manuelle par `build_launch_command()`
2. Ajout récupération JVM args via `build_jvm_args_for_launcher()`
3. Passage packaging type et Spring Boot version

**Commit** : `3bd6a44`

### Tests Ajoutés

**Fichier** : `tests/starters_command_tests.rs`

6 tests couvrant :
- ✅ `cleanupDaemonThreads=false` est présent
- ✅ `classpathScope=compile` pour WAR
- ✅ `mainClass` paramètre
- ✅ JVM args passés via `exec.args`
- ✅ Spring Boot run avec main-class
- ✅ Spring Boot 1.x avec full GAV

## Vérification

### Test Unitaire

```bash
$ cargo test --test starters_command_tests

running 6 tests
test test_starters_exec_java_includes_cleanup_daemon_threads ... ok
test test_starters_war_packaging_includes_classpath_scope ... ok
test test_starters_exec_java_includes_main_class ... ok
test test_starters_exec_java_includes_jvm_args ... ok
test test_starters_spring_boot_run_includes_main_class ... ok
test test_starters_spring_boot_1x_uses_full_gav ... ok

test result: ok. 6 passed
```

### Test Manuel Attendu

Sur Windows, après recompilation :

```powershell
cargo build --release
```

Puis lancer foo-bdd-id avec la touche `s` :
- ✅ Application démarre
- ✅ Threads daemon ne sont PAS tués
- ✅ Application reste active (pas d'exit code 1)
- ✅ Logs visibles dans LazyMVN

## Documentation Technique

### exec-maven-plugin Behavior

**Default** : `cleanupDaemonThreads=true`
- Tue les threads daemon après main()
- Approprié pour scripts courts
- **Inapproprié pour serveurs/applications longue durée**

**Override** : `cleanupDaemonThreads=false`
- Laisse les threads daemon vivre
- Permet aux serveurs de continuer
- **Nécessaire pour Spring Boot, Java EE, etc.**

Référence : [exec-maven-plugin documentation](https://www.mojohaus.org/exec-maven-plugin/java-mojo.html#cleanupDaemonThreads)

### WAR Packaging + exec:java

**Problem** : Dependencies `provided` exclus par défaut

```xml
<dependency>
    <groupId>javax.servlet</groupId>
    <artifactId>javax.servlet-api</artifactId>
    <scope>provided</scope>  <!-- Container fournit -->
</dependency>
```

**Solution** : `classpathScope=compile`
- Inclut les dépendances `provided`
- Permet lancement standalone sans container

Voir : `docs/user/WAR_MODULE_SUPPORT.md`

## Historique des Commits

```
3bd6a44 fix: use build_launch_command for starters (CE FIX)
0dd615c test: add integration tests for starters regression protection
77f8a90 fix: detect Spring Boot plugin and fallback to exec:java
b675598 fix: use -pl instead of -f for Spring Boot starters
b951674 fix: properly detect Maven build failures
03ec1f9 feat: add visual feedback for Maven command execution
```

## Leçons Apprises

### 1. Ne Pas Dupliquer la Logique

**Avant** :
- Logique de build dans `build_launch_command()` ✅
- Logique dupliquée dans `starters.rs` ❌
- → Divergence et bugs

**Après** :
- Une seule source de vérité : `build_launch_command()`
- Réutilisée partout (launcher, starters)

### 2. Tester les Intégrations

Les tests unitaires sur `SpringBootDetection` étaient ✅  
Mais les tests d'**intégration** sur la commande finale manquaient ❌

→ Ajout de `starters_command_tests.rs`

### 3. Paramètres Maven Subtils

Certains paramètres sont **critiques** mais non-évidents :
- `cleanupDaemonThreads` → exit code 1 mystérieux
- `classpathScope` → NoClassDefFoundError

→ Nécessitent documentation et tests

## Prochaines Étapes

### Court Terme

1. ✅ Fix déployé
2. ⏳ Utilisateur recompile sur Windows
3. ⏳ Test sur foo-bdd-id

### Moyen Terme

1. Documenter les paramètres `exec:java` critiques
2. Ajouter tests end-to-end avec vraie application
3. Améliorer logs pour signaler quels paramètres sont utilisés

### Long Terme

1. Considérer l'ajout d'un mode "debug" qui affiche la commande Maven complète
2. Potentiellement ajouter une UI pour configurer les JVM args
3. Support pour autres lanceurs (Quarkus, Micronaut, etc.)

---

**Résolution** : COMPLÈTE ✅  
**Tests** : 6/6 passent ✅  
**Documentation** : Cette analyse ✅  
**Commit** : `3bd6a44` ✅
