# Fix: Log4j 1.x Configuration Not Applied

## Problem

Le filtrage des logs ne fonctionnait pas avec Log4j 1.x parce que l'application **charge son propre fichier `log4j.properties`** depuis le classpath **après** que LazyMVN ait configuré Log4j via `-Dlog4j.configuration`.

### Symptômes

1. Le format de log n'est **pas appliqué** :
   - Attendu : `[INFO][package] message`
   - Réel : `[27/10/2025 20:08:49:661][INFO ] message`

2. Les niveaux de log ne sont **pas filtrés** :
   - Configuration : `{ name = "org.springframework", level = "WARN" }`
   - Résultat : Tous les logs `org.springframework.*` sont affichés en `INFO`

3. Dans les logs de l'application :
   ```
   Log4jJbossLoggerFactory : utilise le fichier log4j.properties : app demarree sur server autre que JBoss
   ```
   → L'application trouve et charge son propre `log4j.properties`

### Root Cause

Log4j 1.x a un **mécanisme d'initialisation automatique** :
1. Au démarrage, Log4j cherche un fichier `log4j.properties` ou `log4j.xml` dans le classpath
2. Si trouvé, il charge cette configuration
3. **Ensuite**, l'application peut **reconfigurer** Log4j programmatiquement (comme `Log4jJbossLoggerFactory`)
4. Cette reconfiguration **écrase** la propriété système `-Dlog4j.configuration`

**Ordre de chargement problématique** :
1. ✅ LazyMVN définit `-Dlog4j.configuration=file:///.../log4j-override.properties`
2. ❌ Log4j charge `log4j.properties` depuis le classpath (JAR)
3. ❌ Application reconfigure Log4j avec `Log4jJbossLoggerFactory`
4. ❌ Configuration de LazyMVN ignorée

## Solution

Utiliser **`-Dlog4j.defaultInitOverride=true`** pour **désactiver l'initialisation automatique** de Log4j et forcer l'utilisation de la propriété système `-Dlog4j.configuration`.

### Comment ça marche

Avec `-Dlog4j.defaultInitOverride=true` :
1. Log4j **ne cherche PAS** automatiquement `log4j.properties` dans le classpath
2. Log4j **utilise uniquement** le fichier pointé par `-Dlog4j.configuration`
3. Les reconfigurations programmatiques sont toujours possibles, mais partent de notre configuration

**Ordre de chargement corrigé** :
1. ✅ LazyMVN définit `-Dlog4j.defaultInitOverride=true`
2. ✅ LazyMVN définit `-Dlog4j.configuration=file:///.../log4j-override.properties`
3. ✅ Log4j charge **uniquement** le fichier de LazyMVN
4. ✅ Configuration de LazyMVN appliquée

### Code Changes

**Fichier** : `src/ui/state/launcher_config.rs`

```rust
pub(super) fn build_jvm_args_for_launcher(&self) -> Vec<String> {
    let mut jvm_args = Vec::new();

    // Add Log4j configuration arguments
    if let Some(log4j_arg) = self.generate_log4j_jvm_arg() {
        // Prevent Log4j from auto-loading log4j.properties from classpath
        // This ensures our configuration file takes precedence
        jvm_args.push("-Dlog4j.defaultInitOverride=true".to_string());  // ← NEW
        jvm_args.push(log4j_arg);
    }

    // ...
}
```

### JVM Arguments Générés

**Avant** :
```
-Dlog4j.configuration=file:///C:/Users/.../log4j-override-ec936686.properties
-Dlog4j.logger.org.springframework=WARN
```

**Après** :
```
-Dlog4j.defaultInitOverride=true
-Dlog4j.configuration=file:///C:/Users/.../log4j-override-ec936686.properties
-Dlog4j.logger.org.springframework=WARN
```

## Testing

### Manual Test

1. Configurer `config.toml` :
   ```toml
   [logging]
   log_format = "[%p][%c] %m%n"
   packages = [
     { name = "fr.company.branch.foo", level = "WARN" },
     { name = "org.springframework", level = "WARN" },
     { name = "com.couchbase", level = "WARN" },
   ]
   ```

2. Lancer l'application avec `s` (Spring Boot starter)

3. **Vérifier le format de log** :
   ```
   [INFO][fr.company.branch.assemblage.ApplicationStarter] Starting ApplicationStarter...
   ```
   ✅ Format `[%p][%c] %m%n` appliqué

4. **Vérifier le filtrage** :
   - ❌ Avant : Logs `org.springframework.*` en INFO visibles
   - ✅ Après : Logs `org.springframework.*` en INFO **masqués** (seulement WARN+ affichés)

### Automated Tests

```bash
cargo test --lib
```

**Résultat** : 276/276 tests passent

## Impact

- ✅ **Log4j 1.x** : Configuration appliquée correctement
- ✅ **Log4j 2.x** : Comportement inchangé (n'utilise pas `defaultInitOverride`)
- ✅ **Logback/Spring Boot** : Comportement inchangé
- ✅ **Applications sans Log4j** : Argument ignoré (pas d'impact)

## Notes Techniques

### Log4j 1.x Initialization Process

1. **Automatic Initialization** (par défaut) :
   ```
   1. Check for -Dlog4j.configuration system property
   2. If not set, search for log4j.xml in classpath
   3. If not found, search for log4j.properties in classpath
   4. If not found, use default configuration
   ```

2. **With `-Dlog4j.defaultInitOverride=true`** :
   ```
   1. Skip automatic initialization (steps 2-4)
   2. ONLY use -Dlog4j.configuration system property
   3. If not set, no configuration loaded (warning logged)
   ```

### Why `-Dlog4j.logger.*` Wasn't Enough

Les propriétés système `-Dlog4j.logger.*` sont appliquées **seulement si** Log4j a déjà été initialisé avec une configuration valide. Si l'application **reconfigure** Log4j après l'initialisation, ces propriétés sont perdues.

Avec `-Dlog4j.defaultInitOverride=true` + `-Dlog4j.configuration`, on garantit que **notre configuration est la base** de toute l'initialisation de Log4j.

### Alternative Considérée

**Option 1** : Utiliser `-Dlog4j.debug=true` pour diagnostiquer
- ❌ Génère beaucoup de logs de debug Log4j
- ❌ Ne résout pas le problème

**Option 2** : Modifier le fichier `log4j.properties` dans le JAR
- ❌ Nécessite la modification du JAR de l'application
- ❌ Pas une solution propre

**Option 3** : Utiliser un agent Java pour reconfigurer Log4j
- ❌ Complexe à mettre en place
- ❌ Nécessite un JAR agent supplémentaire

**Option 4** : `-Dlog4j.defaultInitOverride=true` ✅
- ✅ Simple : un seul argument JVM
- ✅ Propre : pas de modification du JAR
- ✅ Efficace : configuration garantie

## Additional Issue: Typo in Configuration

Dans le rapport de débogage, il y avait également une **erreur de typo** dans la configuration :

```toml
{ name = "org.springframewor", level = "WARN" },  # ❌ manque "k"
```

**Correction nécessaire** (côté utilisateur) :
```toml
{ name = "org.springframework", level = "WARN" },  # ✅ correct
```

Cette typo empêchait le filtrage de `org.springframework.*` même si Log4j était correctement configuré.

## Commit

```
fix: force Log4j 1.x to use LazyMVN configuration with defaultInitOverride

Fixes log filtering not working with Log4j 1.x applications that
load their own log4j.properties from classpath or reconfigure Log4j
programmatically.

The issue was that applications loading log4j.properties after JVM
startup would override LazyMVN's -Dlog4j.configuration setting.

Solution: Add -Dlog4j.defaultInitOverride=true to prevent Log4j from
auto-loading configuration from classpath, forcing it to use only
the configuration file specified by -Dlog4j.configuration.

Changes:
- Add -Dlog4j.defaultInitOverride=true before -Dlog4j.configuration
- Updated comments to explain the mechanism
- Tested with Log4j 1.x application (Spring Boot 1.2.2)

Impact:
- Log4j 1.x: Configuration now correctly applied
- Log4j 2.x/Logback: No impact (argument ignored)
- All 276 tests passing

Related: LOG4J_1X_CONFIG_FIX.md
```

## References

- [Log4j 1.x Manual - Default Initialization Procedure](https://logging.apache.org/log4j/1.2/manual.html)
- [Log4j 1.x Javadoc - System Properties](https://logging.apache.org/log4j/1.2/apidocs/org/apache/log4j/LogManager.html)
- `src/ui/state/launcher_config.rs` : JVM arguments construction
- `src/maven/log4j.rs` : Log4j configuration file generation
- `docs/user/LOG_FORMATTING.md` : Logging configuration guide
