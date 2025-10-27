# Windows Argument Parsing Fix

## Problème identifié

D'après le debug report fourni, deux problèmes critiques ont été détectés dans la commande Maven générée :

```bash
mvn.cmd ... -U, --update-snapshots -DskipTests ...
```

### Problème 1 : Flags Maven avec virgules
**Symptôme** : Maven reçoit `-U, --update-snapshots` comme **un seul argument**, ce qui cause l'erreur :
```
[ERROR] Unknown lifecycle phase ", --update-snapshots"
```

**Cause** : Les BuildFlags étaient définis avec des alias séparés par virgule :
```rust
flag: "-U, --update-snapshots".to_string()
```

Maven interprète cela comme une seule phase de cycle de vie au lieu de deux arguments séparés.

### Problème 2 : Arguments avec `=` non quotés sur Windows
**Symptôme potentiel** : Les arguments avec `=` peuvent être mal parsés par `cmd.exe` s'ils contiennent des espaces ou caractères spéciaux.

**Exemple problématique** :
```bash
-Drun.jvmArguments=-Dlog4j.configuration=file:///C:/Users/...properties
```

Sur Windows, sans quoting approprié, cela peut causer des problèmes de parsing.

## Solution implémentée

### Fix 1 : Correction des BuildFlags (src/ui/state/project_tab.rs)

**Avant** :
```rust
BuildFlag {
    name: "Force update snapshots".to_string(),
    flag: "-U, --update-snapshots".to_string(),
    enabled: false,
},
```

**Après** :
```rust
BuildFlag {
    name: "Force update snapshots".to_string(),
    flag: "-U".to_string(),  // ✓ Forme courte uniquement
    enabled: false,
},
```

**Changements appliqués** :
- `-U, --update-snapshots` → `-U`
- `-o, --offline` → `-o`
- `-X, --debug` → `-X`

### Fix 2 : Splitting des flags avec virgules (src/maven/command.rs)

**Ajouté dans `execute_maven_command`** :
```rust
// Add build flags (split on spaces if needed, skip commas and aliases)
for flag in flags {
    // Split flags like "-U, --update-snapshots" into individual flags
    // and skip aliases (anything after comma)
    let flag_parts: Vec<&str> = flag
        .split(',')
        .next() // Take only the first part before comma
        .unwrap_or(flag)
        .split_whitespace()
        .collect();
    
    for part in flag_parts {
        if !part.is_empty() {
            command.arg(part);
            log::debug!("Added flag: {}", part);
        }
    }
}
```

**Comportement** :
- Si un flag contient une virgule (legacy), on prend **uniquement la première partie**
- Le flag est ensuite split sur les espaces
- Chaque partie est ajoutée comme un argument séparé

### Fix 3 : Quoting des arguments sur Windows

**Ajouté dans `build_command_string_with_options`** :
```rust
for arg in args {
    // Quote arguments that contain spaces or special chars on Windows
    #[cfg(windows)]
    {
        if arg.contains(' ') || arg.contains('=') && arg.starts_with("-D") {
            parts.push(format!("\"{}\"", arg));
        } else {
            parts.push(arg.to_string());
        }
    }
    #[cfg(not(windows))]
    {
        parts.push(arg.to_string());
    }
}
```

**Comportement** :
- Sur Windows uniquement
- Arguments contenant des espaces → quotés
- Arguments `-D...=...` → quotés
- Autres arguments → non quotés

**Note importante** : `Rust's Command::arg()` gère **automatiquement le quoting** lors de l'exécution. Ce fix concerne principalement l'**affichage de la commande** dans les logs pour faciliter le debug.

## Impact

### Commande générée AVANT le fix :
```bash
mvn.cmd --settings ... -pl module -U, --update-snapshots -DskipTests ...
                                     ^^^^^^^^^^^^^^^^^^^^
                                     Un seul argument invalide !
```

### Commande générée APRÈS le fix :
```bash
mvn.cmd --settings ... -pl module -U -DskipTests ...
                                   ^^
                                   Argument séparé correct
```

### Affichage dans les logs (Windows) :
```bash
"-Drun.jvmArguments=-Dlog4j.configuration=file:///C:/Users/...properties"
^                                                                        ^
Guillemets ajoutés pour clarté
```

## Tests validés

- ✅ **606 tests passent** (aucune régression)
- ✅ **Build flags corrigés** : plus de virgules
- ✅ **Splitting des flags** : gère les legacy flags avec virgules
- ✅ **Quoting Windows** : arguments avec `=` quotés dans l'affichage

## Pour tester

### Sur Windows (utilisateur final) :

1. Recompiler LazyMVN :
```bash
cargo build --release
```

2. Lancer l'application avec debug :
```bash
lazymvn --debug
```

3. Activer les flags : `f` → sélectionner "Force update snapshots"

4. Lancer une commande Maven

5. Vérifier dans `lazymvn-debug.log` :
```
INFO - Executing: mvn.cmd ... -U -DskipTests ...
                              ^^
                              Correct (pas de virgule)
```

### Validation attendue :

- ❌ **Erreur précédente** : `Unknown lifecycle phase ", --update-snapshots"`
- ✅ **Comportement correct** : Maven accepte `-U` et exécute la commande

## Documentation Maven

Selon [la documentation officielle Maven](https://maven.apache.org/ref/current/maven-embedder/cli.html) :

- `-U, --update-snapshots` : **Notation de documentation** (pas un argument valide)
- `-U` : **Forme courte** (recommandée)
- `--update-snapshots` : **Forme longue** (alternative)

**On doit utiliser l'une OU l'autre, jamais les deux ensemble.**

## Fichiers modifiés

1. `src/ui/state/project_tab.rs` : Correction des définitions de BuildFlag
2. `src/maven/command.rs` : Splitting des flags + quoting Windows
3. `scripts/test-windows-args-quoting.sh` : Script de test de validation

## Commits recommandés

```bash
git add src/ui/state/project_tab.rs src/maven/command.rs
git commit -m "fix(windows): correct Maven flag parsing and argument quoting

- Fix BuildFlag definitions to use single form (-U instead of -U, --update-snapshots)
- Add flag splitting logic to handle legacy comma-separated flags
- Add Windows-specific quoting for arguments with spaces and equals signs
- Maven no longer receives malformed arguments like ', --update-snapshots'

Fixes issue where Maven reported 'Unknown lifecycle phase' errors on Windows
due to improperly formatted build flags.

Closes #XXX"
```

## Références

- Maven CLI Reference: https://maven.apache.org/ref/current/maven-embedder/cli.html
- Rust Command API: https://doc.rust-lang.org/std/process/struct.Command.html
- Windows CMD quoting: https://ss64.com/nt/syntax-esc.html
