# Fix: Log Filtering Not Working with Spring Boot 1.x

## Problem

Le filtrage des logs ne fonctionnait pas avec Spring Boot 1.x parce que LazyMVN ne détectait pas correctement les arguments JVM passés via `-Drun.jvmArguments=` (Spring Boot 1.x).

Le code vérifiait uniquement la présence de `-Dspring-boot.run.jvmArguments=` (Spring Boot 2.x+), ce qui causait l'ajout redondant des arguments de logging directement sur la ligne de commande Maven au lieu de les passer via les JVM arguments.

### Symptômes

- Les arguments de logging (`-Dlog4j.logger.*`, `-Dlogging.level.*`) étaient ajoutés **à la ligne de commande Maven** au lieu d'être passés **aux JVM arguments**
- Les logs n'étaient pas filtrés comme prévu
- Configuration dans `lazymvn.toml` :
  ```toml
  [logging]
  packages = [
    { name = "fr.company.branch.fwmc", level = "WARN" },
    { name = "org.springframework", level = "WARN" },
  ]
  ```

### Commande générée (incorrecte)

```bash
mvn.cmd --settings settings.xml -pl module -DskipTests \
  -Dlog4j.conversionPattern=[%p][%c] %m%n \
  -Dlogging.pattern.console=[%p][%c] %m%n \
  -Dlog4j.logger.fr.company.branch.fwmc=WARN \
  -Dlogging.level.fr.company.branch.fwmc=WARN \
  -Dlog4j.logger.org.springframework=WARN \
  -Dlogging.level.org.springframework=WARN \
  -Drun.jvmArguments=-Dlog4j.configuration=file:///... \  # ← Spring Boot 1.x
  org.springframework.boot:spring-boot-maven-plugin:1.2.2.RELEASE:run
```

**Problème** : Les arguments de logging sont ajoutés **deux fois** :
1. Sur la ligne de commande Maven (`-Dlog4j.logger.*`)
2. Dans `-Drun.jvmArguments=` (via le fichier Log4j)

Les arguments de la ligne de commande Maven ne sont **pas transmis** à l'application lancée par `spring-boot:run`.

## Root Cause

Dans `src/maven/command.rs`, trois fonctions vérifiaient uniquement `-Dspring-boot.run.jvmArguments=` :

```rust
// AVANT (ligne 192, 374, 583)
let has_spring_boot_jvm_args = args
    .iter()
    .any(|arg| arg.starts_with("-Dspring-boot.run.jvmArguments="));
```

Cela ignorait le format Spring Boot 1.x : `-Drun.jvmArguments=`

## Solution

Détection des **deux formats** de Spring Boot :

```rust
// APRÈS
let has_spring_boot_jvm_args = args
    .iter()
    .any(|arg| arg.starts_with("-Dspring-boot.run.jvmArguments=") || 
              arg.starts_with("-Drun.jvmArguments="));
```

### Fichiers modifiés

1. **`src/maven/command.rs`** (3 occurrences corrigées)
   - Ligne 192 : `build_command_string_with_options()`
   - Ligne 374 : `execute_maven_command()`
   - Ligne 583 : `execute_maven_command_async()`

### Commande générée (correcte)

```bash
mvn.cmd --settings settings.xml -pl module -DskipTests \
  -Drun.jvmArguments=-Dlog4j.configuration=file:///C:/Users/.../log4j-override-ec936686.properties ... \
  org.springframework.boot:spring-boot-maven-plugin:1.2.2.RELEASE:run
```

**Résultat** : Les arguments de logging sont **uniquement** dans `-Drun.jvmArguments=`, et sont correctement transmis à l'application.

## Test Coverage

Ajout d'un nouveau test : `test_build_command_string_skips_logging_props_for_spring_boot_1x`

```rust
#[test]
fn test_build_command_string_skips_logging_props_for_spring_boot_1x() {
    let cmd = build_command_string_with_options(
        "mvn",
        Some("app"),
        &[
            "-Drun.profiles=dev",
            "-Drun.jvmArguments=-Dlog4j.configuration=file:///...",
            "org.springframework.boot:spring-boot-maven-plugin:1.2.2.RELEASE:run",
        ],
        // ...
        Some(&logging_config),
    );

    assert!(!cmd.contains("-Dlog4j.conversionPattern="));
    assert!(!cmd.contains("-Dlog4j.logger.com.example=DEBUG"));
}
```

## Validation

### Tests

```bash
cargo test --lib maven::command::tests
```

**Résultat** : 22/22 tests passent (276 tests au total)

### Vérification manuelle

1. Modifier `~/.config/lazymvn/projects/<hash>/config.toml` :
   ```toml
   [logging]
   packages = [
     { name = "fr.company.branch.fwmc", level = "WARN" },
     { name = "org.springframework", level = "WARN" },
   ]
   ```

2. Lancer l'application avec `s` (Spring Boot starter)

3. Vérifier dans les logs de débogage (Shift+Y) :
   ```
   Skipping logging overrides as Maven properties (already in JVM arguments for Spring Boot)
   ```

4. Vérifier que les logs sont bien filtrés dans la sortie

## Impact

- ✅ **Spring Boot 1.x** : Les arguments sont correctement passés via `-Drun.jvmArguments=`
- ✅ **Spring Boot 2.x+** : Les arguments sont correctement passés via `-Dspring-boot.run.jvmArguments=`
- ✅ **exec:java** : Les arguments continuent d'être ajoutés directement (comportement inchangé)
- ✅ **Autres commandes Maven** : Comportement inchangé

## Notes techniques

### Pourquoi deux formats ?

Spring Boot a changé le format des paramètres entre les versions :

| Version | Paramètre | Exemple |
|---------|-----------|---------|
| Spring Boot 1.x | `-Drun.jvmArguments=` | `-Drun.jvmArguments=-Dfoo=bar` |
| Spring Boot 2.x+ | `-Dspring-boot.run.jvmArguments=` | `-Dspring-boot.run.jvmArguments=-Dfoo=bar` |

### Mécanisme de détection

LazyMVN détecte la version de Spring Boot via `help:effective-pom` et choisit automatiquement le bon format dans `src/maven/detection.rs` :

```rust
let (profile_param, jvm_param) = if version.starts_with("1.") {
    ("run.profiles", "run.jvmArguments")  // Spring Boot 1.x
} else {
    ("spring-boot.run.profiles", "spring-boot.run.jvmArguments")  // 2.x+
};
```

La correction dans `command.rs` assure que **les deux formats sont reconnus** lors de la vérification des duplications.

## Commit

```
fix: detect Spring Boot 1.x JVM args format for log filtering

Fixes log filtering not working with Spring Boot 1.x by detecting
both `-Drun.jvmArguments=` (1.x) and `-Dspring-boot.run.jvmArguments=` (2.x+)
when checking if logging overrides should be skipped.

Previously, logging arguments were incorrectly added to Maven command line
instead of being passed through JVM arguments for Spring Boot 1.x apps.

Changes:
- Updated 3 occurrences in command.rs to check both formats
- Added test for Spring Boot 1.x argument detection
- Updated debug messages to be version-agnostic

Related: LOG_FILTERING_FIX_SPRING_BOOT_1X.md
```

## References

- [Spring Boot 1.x Maven Plugin Docs](https://docs.spring.io/spring-boot/docs/1.5.x/maven-plugin/)
- [Spring Boot 2.x Maven Plugin Docs](https://docs.spring.io/spring-boot/docs/current/maven-plugin/)
- `src/maven/detection.rs` : Version detection logic
- `docs/user/LOG_FORMATTING.md` : Logging configuration guide
