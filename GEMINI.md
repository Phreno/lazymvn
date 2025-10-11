
```markdown
# GEMINI.md — Specification Technique du projet **LazyMVN**

## 📘 Résumé
**LazyMVN** est un outil **TUI multiplateforme** pour interagir efficacement avec des projets **Maven** sans quitter le terminal.  
Inspiré de *LazyGit*, il fournit une interface minimaliste pour explorer, construire et tester des modules Maven via un **binaire Rust unique**, sans dépendance externe.

---

## 🎯 Objectifs

### Fonctionnels
- Lister les modules Maven d’un projet multi-module.
- Lancer les commandes Maven usuelles (`clean`, `install`, `package`, `test`, `dependency:tree`, etc.).
- Sélectionner rapidement un module ou un profil Maven.
- Afficher les logs Maven en temps réel dans une interface claire.
- Permettre la configuration facultative via un fichier `lazymvn.toml` par projet.

### Non-fonctionnels
- Binaire standalone, sans runtime externe.
- Cross-platform (Windows, Linux, macOS).
- Exécution rapide (< 50 ms au lancement).
- Consommation mémoire minimale (< 15 Mo).
- Interface fluide sans clignotement (grâce à `ratatui` et `crossterm`).

---

## ⚙️ Stack technique

| Domaine       | Choix | Description |
|----------------|-------|--------------|
| Langage        | Rust  | compilation native, gestion mémoire sûre |
| CLI parser     | `clap` | gestion des arguments |
| TUI            | `ratatui`, `crossterm` | interface textuelle fluide |
| Fuzzy search   | `skim` ou `fuzzy-matcher` | recherche dans modules/goals |
| I/O            | `tokio` (éventuel) | streaming non bloquant des logs |
| XML parsing    | `quick-xml` | lecture des `pom.xml` |
| Config         | `toml` + `serde` | lecture du fichier `lazymvn.toml` |

---

## 📁 Arborescence projet

```

lazymvn/
├─ Cargo.toml
├─ src/
│  ├─ main.rs
│  ├─ tui.rs              # gestion interface utilisateur
│  ├─ maven.rs            # exécution des commandes Maven
│  ├─ project.rs          # détection des modules, profils
│  ├─ config.rs           # parsing du fichier lazymvn.toml
│  ├─ ui/
│  │  ├─ panes.rs         # gestion panneaux log / menu
│  │  ├─ keybindings.rs   # raccourcis clavier
│  │  └─ theme.rs         # couleurs et styles
│  └─ utils.rs
└─ README.md

````

---

## ⚒️ Commandes principales

| Action       | Raccourci  | Commande Maven exécutée           |
|---------     |------------|--------------------------         |
| Build        | `b`        | `mvn[w] -T1C -DskipTests package` |
| Test         | `t`        | `mvn[w] test`                     |
| Clean        | `c`        | `mvn[w] clean`                    |
| Install      | `i`        | `mvn[w] -DskipTests install`      |
| Dependencies | `d`        | `mvn[w] dependency:tree`          |
| Profiles     | `p`        | `mvn help:all-profiles`           |
| Custom goal  | `g`        | saisie libre (fuzzy)              |
| Quit         | `q`        | —                                 |

---

## 🧩 Détection du projet

1. Recherche récursive du premier `pom.xml` en remontant depuis le répertoire courant.  
2. Lecture des modules enfants via `<modules><module>...</module></modules>`.  
3. Vérification de la présence de `mvnw` (prioritaire sur `mvn` global).  
4. Cache des chemins des modules dans `~/.config/lazymvn/cache.json` (facultatif).

---

## ⚙️ Fichier de configuration (`lazymvn.toml`)

Exemple à la racine du projet :

```toml
default_profile = ["dev"]
skip_tests = true
concurrency = "1C"        # pour -T1C
goals = ["clean", "package", "install"]

[shortcuts]
build = "package -DskipTests"
run = "spring-boot:run -Dspring-boot.run.profiles=dev"
````

Les champs sont tous optionnels.
En l’absence de fichier, **LazyMVN** fonctionne avec des valeurs par défaut.

---

## 🖥️ Interface TUI

### Layout

```
┌────────────────────────────┬──────────────────────────────┐
│ Modules / Profils          │ Logs Maven (flux temps réel) │
│ (sélection fuzzy)          │                              │
│                            │                              │
│                            │                              │
│                            │                              │
└────────────────────────────┴──────────────────────────────┘
```

### Navigation

* `↑ / ↓` : sélectionner un module
* `Enter` : exécuter l’action sur le module sélectionné
* `b/t/c/i/d/p/g/q` : raccourcis directs
* `/` : recherche fuzzy dans modules/goals

---

## 🧪 MVP Features

| Étape | Fonctionnalité                 | Détail                           |
| ----- | ------------------------------ | -------------------------------- |
| v0.1  | CLI non interactive            | flags `--build`, `--test`, etc.  |
| v0.2  | TUI minimal                    | menu modules + logs temps réel   |
| v0.3  | Sélection profils              | `mvn help:all-profiles` + toggle |
| v0.4  | Cache local                    | mémorise derniers modules/goals  |
| v1.0  | Binaire multiplateforme stable | publication GitHub Releases      |

---

## 🧰 Commandes de build

### Linux (musl)

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

### Windows

```powershell
cargo build --release
```

### macOS (universel)

```bash
cargo build --release --target aarch64-apple-darwin
```

### Résultat

```
target/release/lazymvn(.exe)
```

---

## 🚀 Distribution

* **Release GitHub** : binaire + checksums.
* **Installation** :

  ```bash
  curl -sSL https://github.com/etienne/lazymvn/releases/latest/download/lazymvn -o /usr/local/bin/lazymvn
  chmod +x /usr/local/bin/lazymvn
  ```
* **Aucune dépendance Java requise** côté client, sauf `mvn` ou `mvnw` du projet.

---

## 🧠 Extensions futures

| Idée                     | Description                                  |
| ------------------------ | -------------------------------------------- |
| Intégration JDK détectée | Affiche version du JDK actif                 |
| Support Gradle           | détection `build.gradle`                     |
| Mode batch               | exécution silencieuse pour CI                |
| Intégration Git          | afficher branche / état du repo              |
| Mode “watch”             | relance test/build sur changement de fichier |
| Export logs              | sauvegarde dernière exécution Maven          |

---

## 🧾 Licence

MIT License — libre utilisation et modification.

---

## 👤 Auteur

**Étienne Audouin**
Développeur & Architecte Logiciel
Projet personnel open-source visant à simplifier la vie des devs Java.

