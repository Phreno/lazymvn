# Log4j JAVA_TOOL_OPTIONS Condition Bug

## Date
2025-01-27

## Symptômes
Les logs montrent que `JAVA_TOOL_OPTIONS` n'est **jamais défini** malgré le code présent dans les fonctions sync et async.

## Découverte

Après 5 tentatives de correction, on réalise que le code ne s'exécute **jamais** à cause d'une condition incorrecte :

```rust
// ❌ BUGGY CODE
if logging_config.is_some() {
    if let Some(log4j_config_url) = extract_log4j_config_url(args) {
        // ... Set JAVA_TOOL_OPTIONS
    }
}
```

### Problème

**Double condition** :
1. ✅ `logging_config.is_some()` → vérifie qu'il y a une config `[logging]` dans `lazymvn.toml`
2. ✅ `extract_log4j_config_url(args)` → vérifie que l'URL Log4j est dans les args

**MAIS** : Pour Spring Boot, LazyMVN génère **automatiquement** l'URL Log4j et l'ajoute aux args Maven. Cette URL est **toujours présente** pour Spring Boot, **indépendamment** de `logging_config`.

La condition `if logging_config.is_some()` est **trop restrictive** ! Elle empêche le code de s'exécuter même si l'URL Log4j est présente dans les args.

### Scénario Utilisateur

L'utilisateur exécute `mvn spring-boot:run` avec :

```
-Drun.jvmArguments=-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration=file:///C:/Users/.../log4j-override-ec936686.properties
```

**Mais** : Si son `lazymvn.toml` n'a **pas** de section `[logging]`, le code JAVA_TOOL_OPTIONS n'est **jamais exécuté** !

## Fix #5 : Supprimer la condition `logging_config`

```rust
// ✅ FIXED CODE
if let Some(log4j_config_url) = extract_log4j_config_url(args) {
    let opts_str = format!(
        "-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration={}",
        log4j_config_url
    );
    log::info!("Setting JAVA_TOOL_OPTIONS with Log4j configuration: {}", log4j_config_url);
    log::info!("JAVA_TOOL_OPTIONS={}", opts_str);
    command.env("JAVA_TOOL_OPTIONS", &opts_str);
} else {
    log::debug!("No Log4j configuration URL found in args");
}
```

### Changements

1. **Suppression** de `if logging_config.is_some()`
2. **Vérification directe** de l'URL Log4j dans les args
3. **Simplification** du code (plus de `Vec` intermédiaire)
4. **Log de debug** si URL non trouvée

## Philosophie

- L'URL Log4j dans les args est **la source de vérité**
- `logging_config` est **indépendant** de JAVA_TOOL_OPTIONS
- Si LazyMVN génère une URL Log4j, on **doit** la propager via JAVA_TOOL_OPTIONS

## Fichiers Modifiés

- `src/maven/command.rs` :
  - `execute_maven_command_with_options()` (ligne ~316)
  - `execute_maven_command_async_with_options()` (ligne ~597)

## Tests

- ✅ 276 tests passent
- ✅ Compilation réussie
- ✅ Aucun changement d'API publique

## Impact

**Avant** : JAVA_TOOL_OPTIONS ne fonctionne **que** si `logging_config` existe dans `lazymvn.toml`

**Après** : JAVA_TOOL_OPTIONS fonctionne **dès que** l'URL Log4j est dans les args (toujours pour Spring Boot)

## Historique des Tentatives

1. **Fix #1** : Détection Spring Boot 1.x → ✅ Fonctionne
2. **Fix #2** : `-Dlog4j.defaultInitOverride=true` → ✅ Nécessaire mais insuffisant
3. **Fix #3** : `-Dlog4j.ignoreTCL=true` → ✅ Nécessaire mais insuffisant
4. **Fix #4** : `JAVA_TOOL_OPTIONS` (sync uniquement) → ❌ Code pas dans async
5. **Fix #4.1** : `JAVA_TOOL_OPTIONS` (async aussi) → ❌ Condition `logging_config` bloque tout
6. **Fix #5** : Suppression condition `logging_config` → ✅ **DEVRAIT FONCTIONNER**

## Prochaines Étapes

1. Recompiler : `cargo build --release`
2. Copier le binaire dans PATH Windows
3. Tester avec `--debug`
4. Chercher les logs :
   - `INFO - Setting JAVA_TOOL_OPTIONS with Log4j configuration`
   - `INFO - JAVA_TOOL_OPTIONS=-Dlog4j.ignoreTCL=true ...`
   - `DEBUG - No Log4j configuration URL found in args` (si URL pas trouvée)

## Explication Technique : Pourquoi cette erreur ?

### Contexte

LazyMVN a **deux mécanismes indépendants** pour le logging :

1. **Génération automatique Log4j** (toujours actif pour Spring Boot)
   - LazyMVN crée un fichier `log4j-override-XXX.properties` temporaire
   - Ajoute `-Dlog4j.configuration=file:///...` dans les JVM args
   - **Indépendant** de `lazymvn.toml`

2. **Configuration `[logging]`** (optionnelle dans `lazymvn.toml`)
   - Permet de configurer les niveaux de log personnalisés
   - Exemple : `[logging.log_levels]`

### Erreur de Logique

Le code original **confondait** ces deux mécanismes :

```rust
if logging_config.is_some() {  // ❌ Vérifie [logging] dans TOML
    if let Some(url) = extract_log4j_config_url(args) {  // ✅ Vérifie URL dans args
```

Or :
- L'URL Log4j est dans **args**, pas dans `logging_config`
- L'URL est **générée automatiquement** par LazyMVN pour Spring Boot
- Elle est **toujours présente** (si Spring Boot détecté)

La condition `logging_config.is_some()` n'a **aucun rapport** avec la présence de l'URL dans les args !

### Correction

```rust
if let Some(url) = extract_log4j_config_url(args) {  // ✅ Vérification unique
    // ... Set JAVA_TOOL_OPTIONS
}
```

Maintenant on vérifie **uniquement** ce qui compte : l'URL dans les args.

## Leçon Apprise

**Séparer clairement les responsabilités** :
- `logging_config` → Configuration TOML optionnelle
- `extract_log4j_config_url(args)` → Détection automatique dans les args

**Ne jamais** conditionner un mécanisme automatique (génération URL) par une configuration optionnelle (TOML).
