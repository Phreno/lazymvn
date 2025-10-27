# Log4j URL Extraction Bug - split() vs splitn()

## Date
2025-10-27

## Symptômes
Après avoir supprimé la condition `logging_config.is_some()` (Fix #5), les logs montrent toujours :

```
DEBUG - No Log4j configuration URL found in args
```

Pourtant l'URL est bien présente dans les args Maven !

## Analyse

### Contexte

LazyMVN génère des args Maven comme :

```bash
-Drun.jvmArguments=-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration=file:///C:/Users/.../log4j-override.properties
```

La fonction `extract_log4j_config_url()` doit extraire l'URL depuis cet argument.

### Code Buggé (Original)

```rust
fn extract_log4j_config_url(args: &[&str]) -> Option<String> {
    for arg in args {
        if arg.starts_with("-Drun.jvmArguments=") {
            if let Some(jvm_args_str) = arg.split('=').nth(1) {  // ❌ BUG ICI
                for part in jvm_args_str.split_whitespace() {
                    if part.starts_with("-Dlog4j.configuration=") {
                        // ...
                    }
                }
            }
        }
    }
    None
}
```

### Problème : `split('=')` vs `splitn(2, '=')`

**Input** :
```
-Drun.jvmArguments=-Dlog4j.ignoreTCL=true -Dlog4j.configuration=file:///C:/test.properties
```

**Avec `split('=')`** :
```rust
arg.split('=').collect::<Vec<_>>()
// Résultat: [
//   "-Drun.jvmArguments",
//   "-Dlog4j.ignoreTCL",        // ❌ On prend ça avec nth(1)
//   "true -Dlog4j.configuration",
//   "file:///C:/test.properties"
// ]
```

On obtient **seulement** `-Dlog4j.ignoreTCL` au lieu de tout le reste !

**Avec `splitn(2, '=')`** (limite à 2 parties max) :
```rust
arg.splitn(2, '=').collect::<Vec<_>>()
// Résultat: [
//   "-Drun.jvmArguments",
//   "-Dlog4j.ignoreTCL=true -Dlog4j.configuration=file:///C:/test.properties"
//   // ✅ Tout le reste intact !
// ]
```

On obtient **tout** le contenu après le premier `=` !

### Conséquence

Avec `split('=').nth(1)` :
- On obtient : `-Dlog4j.ignoreTCL`
- `split_whitespace()` : `["-Dlog4j.ignoreTCL"]`
- Recherche de `-Dlog4j.configuration=` : **INTROUVABLE**
- Résultat : `None`
- Log : `DEBUG - No Log4j configuration URL found in args`

Avec `splitn(2, '=').nth(1)` :
- On obtient : `-Dlog4j.ignoreTCL=true -Dlog4j.configuration=file:///...`
- `split_whitespace()` : `["-Dlog4j.ignoreTCL=true", "-Dlog4j.configuration=file:///..."]`
- Recherche de `-Dlog4j.configuration=` : **TROUVÉ** ✅
- Extraction : `file:///...`
- Log : `INFO - Setting JAVA_TOOL_OPTIONS with Log4j configuration: file:///...`

## Fix #6 : Utiliser `splitn(2, '=')`

```rust
fn extract_log4j_config_url(args: &[&str]) -> Option<String> {
    for arg in args {
        if arg.starts_with("-Drun.jvmArguments=") || arg.starts_with("-Dspring-boot.run.jvmArguments=") {
            // ✅ FIXED: Use splitn(2, '=') to split ONLY at first '='
            if let Some(jvm_args_str) = arg.splitn(2, '=').nth(1) {
                for part in jvm_args_str.split_whitespace() {
                    if part.starts_with("-Dlog4j.configuration=") {
                        if let Some(config_url) = part.strip_prefix("-Dlog4j.configuration=") {
                            return Some(config_url.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
```

### Changements

1. **`split('=')`** → **`splitn(2, '=')`**
2. Ajout de 3 tests unitaires pour valider le comportement

## Tests Ajoutés

```rust
#[test]
fn test_extract_log4j_config_url_spring_boot_1x() {
    let args = vec![
        "-Drun.jvmArguments=-Dlog4j.ignoreTCL=true -Dlog4j.configuration=file:///C:/Users/test/log4j.properties"
    ];
    let result = extract_log4j_config_url(&args);
    assert_eq!(result, Some("file:///C:/Users/test/log4j.properties".to_string()));
}

#[test]
fn test_extract_log4j_config_url_spring_boot_2x() {
    let args = vec![
        "-Dspring-boot.run.jvmArguments=-Dlog4j.configuration=file:///tmp/log4j.properties"
    ];
    let result = extract_log4j_config_url(&args);
    assert_eq!(result, Some("file:///tmp/log4j.properties".to_string()));
}

#[test]
fn test_extract_log4j_config_url_not_found() {
    let args = vec!["-Dsome.other.property=value"];
    let result = extract_log4j_config_url(&args);
    assert_eq!(result, None);
}
```

## Impact

**Avant** : URL jamais extraite → JAVA_TOOL_OPTIONS jamais défini → Log4j utilise config par défaut

**Après** : URL extraite correctement → JAVA_TOOL_OPTIONS défini → Log4j utilise config LazyMVN ✅

## Historique Complet

1. **Fix #1** : Détection Spring Boot 1.x (`-Drun.jvmArguments=`) → ✅
2. **Fix #2** : `-Dlog4j.defaultInitOverride=true` → ✅ Nécessaire
3. **Fix #3** : `-Dlog4j.ignoreTCL=true` → ✅ Nécessaire
4. **Fix #4** : `JAVA_TOOL_OPTIONS` (sync uniquement) → ❌ Code pas dans async
5. **Fix #4.1** : `JAVA_TOOL_OPTIONS` (async aussi) → ❌ Condition `logging_config` bloque
6. **Fix #5** : Suppression condition `logging_config` → ❌ URL jamais extraite (`split()` bug)
7. **Fix #6** : `split('=')` → `splitn(2, '=')` → ✅ **DEVRAIT ENFIN FONCTIONNER**

## Tests

- ✅ **279 tests passent** (276 existants + 3 nouveaux)
- ✅ Compilation release réussie
- ✅ Aucune régression

## Prochaines Étapes

1. Copier binaire sur Windows : `target\release\lazymvn.exe`
2. Vérifier version : `lazymvn --version` → doit montrer nouveau commit SHA
3. Exécuter avec `--debug`
4. **ENFIN** voir dans les logs :
   - `INFO - Setting JAVA_TOOL_OPTIONS with Log4j configuration: file:///...`
   - `INFO - JAVA_TOOL_OPTIONS=-Dlog4j.ignoreTCL=true ...`
   - Logs au format LazyMVN : `[INFO][fr.laposte.disf.assemblage] ...`

## Leçon Apprise

**Toujours utiliser `splitn(n, delimiter)` quand on veut limiter le nombre de splits !**

- `split('=')` → Split à **chaque** `=` (risque de casser les valeurs)
- `splitn(2, '=')` → Split au **premier** `=` seulement (préserve les valeurs)

Cas d'usage typiques :
- Parsing d'arguments clé=valeur où la valeur peut contenir le délimiteur
- Extraction de headers HTTP : `"Content-Type: application/json; charset=utf-8"`
- Parsing de fichiers `.properties` : `key=value with = signs`

## Documentation Technique

### Comportement de `split()` vs `splitn()`

```rust
let s = "a=b=c=d";

// split() - split à chaque '='
let parts: Vec<&str> = s.split('=').collect();
// ["a", "b", "c", "d"]

// splitn(2, '=') - split au PREMIER '=' seulement
let parts: Vec<&str> = s.splitn(2, '=').collect();
// ["a", "b=c=d"]  ← Tout le reste préservé !

// splitn(3, '=') - split aux 2 PREMIERS '='
let parts: Vec<&str> = s.splitn(3, '=').collect();
// ["a", "b", "c=d"]
```

### Pourquoi `splitn(2, '=')` ?

Parce qu'on veut :
1. Séparer la clé : `-Drun.jvmArguments`
2. **Préserver** la valeur intacte : `-Dlog4j.ignoreTCL=true -Dlog4j.configuration=file:///...`

Si on utilisait `splitn(3, '=')` ou plus, on casserait encore la valeur.

## Validation

Pour valider que le fix fonctionne, cherchez ces lignes dans `lazymvn-debug.log` :

```
INFO - Setting JAVA_TOOL_OPTIONS with Log4j configuration: file:///C:/Users/.../log4j-override-ec936686.properties
INFO - JAVA_TOOL_OPTIONS=-Dlog4j.ignoreTCL=true -Dlog4j.defaultInitOverride=true -Dlog4j.configuration=file:///...
```

Et dans la sortie de l'application :

```
[INFO][fr.laposte.disf.assemblage] ...  ← Format LazyMVN appliqué ✅
[WARN][fr.laposte.disf.fwmc] ...         ← Niveau WARN respecté ✅
[DEBUG][fr.laposte.disf.assemblage] ...  ← Niveau DEBUG respecté ✅
```

**Pas** :
```
[27/10/2025 21:26:32:286] [INFO ] ...  ← Format par défaut ❌
```
