# Spring Boot Launch Strategy

LazyMVN peut automatiquement choisir entre `spring-boot:run` et `exec:java` pour lancer vos applications Spring Boot, **sans attendre d'échec**.

## Trois modes de lancement

Configuration via `lazymvn.toml` :

```toml
# Mode de lancement (défaut: "auto")
launch_mode = "auto"       # Détection intelligente
# launch_mode = "force-run"  # Toujours spring-boot:run
# launch_mode = "force-exec" # Toujours exec:java
```

### Mode `auto` (recommandé)

Détecte automatiquement **avant** de lancer la commande. Utilise `spring-boot:run` si les conditions sont remplies, sinon bascule sur `exec:java`.

**Critères de détection :**

1. ✅ **Plugin Spring Boot présent**
   - Cherche `org.springframework.boot:spring-boot-maven-plugin` dans le POM effectif
   
2. ✅ **Packaging compatible**
   - `jar` ou `war` → compatible avec `spring-boot:run`
   - `pom` → incompatible, bascule sur `exec:java`

3. ✅ **Main class connue** (optionnel)
   - Propriétés Spring Boot : `spring-boot.run.mainClass`, `spring-boot.main-class`, `start-class`
   - Configuration plugin : `<mainClass>` dans la configuration du plugin
   - Si absente mais plugin présent, Spring Boot peut l'inférer

4. ✅ **Plugin exec comme fallback**
   - Si `exec-maven-plugin` configuré avec `mainClass`, prêt pour `exec:java`

### Mode `force-run`

Force l'utilisation de `spring-boot:run` même si la détection suggère le contraire.

**Utile pour :**
- Forcer l'utilisation des devtools
- Garantir le comportement Spring Boot natif

### Mode `force-exec`

Force l'utilisation de `exec:java` avec mainClass explicite.

**Utile pour :**
- Applications Java simples sans Spring Boot
- Éviter les dépendances au plugin Spring Boot
- Compatibilité maximale

## Détection pré-vol (sans échec)

Au lieu d'essayer et échouer, LazyMVN :

1. **Analyse le POM effectif** via `mvn help:effective-pom`
2. **Parse les informations** : plugins, packaging, mainClass
3. **Décide de la stratégie** selon les critères
4. **Génère la commande** appropriée

**Avantages :**
- ⚡ Pas d'échec visible pour l'utilisateur
- 🎯 Choix intelligent basé sur la configuration réelle
- 📝 Logs clairs expliquant le choix
- 🔧 Override possible via `launch_mode`

## Génération des commandes

### Avec `spring-boot:run` (mode auto détecté ou force-run)

```bash
mvn "-Dspring-boot.run.profiles=dev,debug" \
    "-Dspring-boot.run.jvmArguments=-Dfoo=bar -Xmx512m" \
    -pl war \
    --settings maven_settings.xml \
    spring-boot:run
```

**Caractéristiques :**
- Profiles via `spring-boot.run.profiles`
- JVM args via `spring-boot.run.jvmArguments`
- Devtools actif si présent
- Reload automatique des changements

### Avec `exec:java` (fallback ou force-exec)

```bash
mvn "-Dexec.mainClass=fr.laposte.app.ApplicationStarter" \
    "-Dfoo=bar" \
    -pl war \
    --settings maven_settings.xml \
    exec:java
```

**Caractéristiques :**
- Main class explicite via `exec.mainClass`
- JVM args passés directement comme `-D...`
- Pas de devtools
- Comportement Java standard

## Quotage PowerShell (Windows)

Sur Windows, les arguments `-D` sont automatiquement quotés :

```powershell
mvn "-Dspring-boot.run.profiles=dev" spring-boot:run
```

Sur Unix/Linux/macOS, pas de quotage nécessaire :

```bash
mvn -Dspring-boot.run.profiles=dev spring-boot:run
```

## Heuristiques mainClass (future)

Si aucune `mainClass` n'est fournie par le POM, LazyMVN pourrait :

1. **Scanner les sources** du module ciblé
2. **Chercher** `public static void main(String[] args)`
3. **Prioriser** noms `*Application`, `*Starter`, `*Launcher`
4. **Proposer** un choix si plusieurs candidats
5. **Mémoriser** le choix par projet

## Conversion des paramètres

| Spring Boot (`spring-boot:run`) | Exec Java (`exec:java`) |
|----------------------------------|-------------------------|
| `-Dspring-boot.run.profiles=dev` | Profils Maven `-P dev` |
| `-Dspring-boot.run.jvmArguments=-Xmx512m` | `-Xmx512m` directement |
| Inférence mainClass automatique | `-Dexec.mainClass=...` requis |
| Devtools reload | Pas de reload |

## Intégration LazyMVN

### Lancement avec arguments CLI

LazyMVN peut être lancé avec des arguments pour contrôler la stratégie de lancement :

```bash
# Mode auto (détection intelligente) - par défaut
lazymvn

# Forcer spring-boot:run
lazymvn --force-run

# Forcer exec:java
lazymvn --force-exec

# Avec un projet spécifique
lazymvn --project /path/to/project --force-run
```

**Priorité de configuration :**
1. Arguments CLI (`--force-run`, `--force-exec`) → priorité maximale
2. Fichier `lazymvn.toml` → si aucun argument CLI
3. Défaut `auto` → si aucune config

### Configuration globale

Dans `lazymvn.toml` à la racine du projet :

```toml
launch_mode = "auto"
```

### Utilisation du raccourci `s`

Une fois dans LazyMVN :

1. Sélectionnez le module à lancer (flèches ou clic)
2. Appuyez sur `s` pour démarrer
3. LazyMVN détecte automatiquement la meilleure stratégie
4. La commande est exécutée avec les profils et flags actifs

**Logs dans le panneau Output :**
```
[INFO] Detecting Spring Boot capabilities for module: Some("war")
[DEBUG] Found spring-boot-maven-plugin
[DEBUG] Found packaging: jar
[INFO] Spring Boot detection results: plugin=true, exec=false, mainClass=Some("..."), packaging=Some("jar")
[INFO] Auto mode: Spring Boot plugin detected, using spring-boot:run
[INFO] Built spring-boot:run command with 1 profile(s) and 0 JVM arg(s)
$ mvn "-Dspring-boot.run.profiles=dev" spring-boot:run
```

### Configuration par projet (future)

Possibilité de surcharger par module dans le futur.

### Logs de décision

Quand `--debug` est activé :

```
[INFO] Detecting Spring Boot capabilities for module: Some("war")
[DEBUG] Found spring-boot-maven-plugin
[DEBUG] Found packaging: jar
[DEBUG] Found mainClass 'fr.laposte.app.App' in spring-boot-maven-plugin
[INFO] Spring Boot detection results: plugin=true, exec=false, mainClass=Some("..."), packaging=Some("jar")
[INFO] Auto mode: Spring Boot plugin detected, using spring-boot:run
[INFO] Built spring-boot:run command with 1 profile(s) and 2 JVM arg(s)
```

## Exemples d'usage

### Projet Spring Boot standard

**POM :**
```xml
<build>
    <plugins>
        <plugin>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-maven-plugin</artifactId>
        </plugin>
    </plugins>
</build>
<packaging>jar</packaging>
```

**Résultat :** Mode auto → `spring-boot:run`

### Projet multi-module avec war

**POM du module war :**
```xml
<packaging>war</packaging>
<build>
    <plugins>
        <plugin>
            <groupId>org.springframework.boot</groupId>
            <artifactId>spring-boot-maven-plugin</artifactId>
            <configuration>
                <mainClass>fr.laposte.app.ApplicationStarter</mainClass>
            </configuration>
        </plugin>
    </plugins>
</build>
```

**Résultat :** Mode auto → `spring-boot:run`

### Module POM (parent)

**POM :**
```xml
<packaging>pom</packaging>
```

**Résultat :** Mode auto → `exec:java` (si exec-plugin configuré)

### Application Java simple

**POM sans Spring Boot :**
```xml
<build>
    <plugins>
        <plugin>
            <groupId>org.codehaus.mojo</groupId>
            <artifactId>exec-maven-plugin</artifactId>
            <configuration>
                <mainClass>com.example.Main</mainClass>
            </configuration>
        </plugin>
    </plugins>
</build>
```

**Résultat :** Mode auto → `exec:java`

## Points d'attention

### Devtools et reload

- **Présents avec** `spring-boot:run` → reload automatique
- **Absents avec** `exec:java` → relance manuelle nécessaire

### Paramètres spécifiques

Les paramètres `spring-boot.run.*` n'ont **aucun effet** avec `exec:java`.

LazyMVN convertit automatiquement :
- Profiles → `-P ...` (Maven natif)
- JVM args → passés directement

### Performance

La détection via `help:effective-pom` ajoute ~1-2 secondes au premier lancement.
Ensuite, la commande s'exécute normalement.

**Optimisation future :** cache de la détection par module.

## API Publique

### Types

```rust
pub enum LaunchMode {
    Auto,      // Détection intelligente
    ForceRun,  // Toujours spring-boot:run
    ForceExec, // Toujours exec:java
}

pub enum LaunchStrategy {
    SpringBootRun,
    ExecJava,
}

pub struct SpringBootDetection {
    pub has_spring_boot_plugin: bool,
    pub has_exec_plugin: bool,
    pub main_class: Option<String>,
    pub packaging: Option<String>,
}
```

### Fonctions

```rust
// Détecter les capacités Spring Boot d'un module
pub fn detect_spring_boot_capabilities(
    project_root: &Path,
    module: Option<&str>,
) -> Result<SpringBootDetection, std::io::Error>

// Décider de la stratégie de lancement
pub fn decide_launch_strategy(
    detection: &SpringBootDetection,
    launch_mode: LaunchMode,
) -> LaunchStrategy

// Construire la commande de lancement
pub fn build_launch_command(
    strategy: LaunchStrategy,
    main_class: Option<&str>,
    profiles: &[String],
    jvm_args: &[String],
) -> Vec<String>
```

## Roadmap

- [x] Détection via `help:effective-pom`
- [x] Mode `auto` avec fallback intelligent
- [x] Modes `force-run` et `force-exec`
- [x] Génération commandes avec quotage platform
- [x] Tests unitaires complets
- [ ] Intégration TUI (raccourci clavier)
- [ ] Cache de détection par module
- [ ] Heuristiques scan sources pour mainClass
- [ ] Sélecteur fuzzy si plusieurs mainClass
- [ ] Configuration par module
- [ ] Documentation utilisateur complète

## Contribution

Voir [CONTRIBUTING.md](CONTRIBUTING.md) pour les guidelines de développement.

---

**Note :** Cette fonctionnalité est conçue pour éliminer les échecs visibles et offrir une expérience fluide, que vous utilisiez Spring Boot ou Java standard.
