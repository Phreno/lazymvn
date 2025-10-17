# LazyMVN Development Container

Ce répertoire contient la configuration pour un environnement de développement GitHub Codespaces/DevContainer automatisé pour LazyMVN.

## Fonctionnalités

### Langages et Outils Installés
- **Rust** (dernière version stable)
  - `cargo`, `rustc`, `rustfmt`, `clippy`
  - Outils additionnels : `cargo-watch`, `cargo-edit`, `cargo-audit`, etc.
- **Java 21** (Microsoft OpenJDK)
- **Maven** (intégré avec Java)
- **Git** avec Git Flow
- **Zsh** avec Oh My Zsh

### Extensions VS Code
- `rust-analyzer` - Support Rust avancé
- `vscode-lldb` - Débogage Rust
- `crates` - Gestion des dépendances Cargo
- `even-better-toml` - Support TOML amélioré
- `vscode-java-pack` - Suite complète Java
- `vscode-maven` - Support Maven

### Aliases Utiles
```bash
# Rust
cb          # cargo build
ct          # cargo test  
cc          # cargo check
cf          # cargo fmt
ccl         # cargo clippy
cw          # cargo watch -x check -x test

# Maven
mvn-test    # ./mvnw test
mvn-package # ./mvnw package
mvn-clean   # ./mvnw clean

# Git
gst         # git status
glog        # git log --oneline --graph --decorate
gco         # git checkout
gcb         # git checkout -b

# Git Flow
gff         # git flow feature
gfh         # git flow hotfix
gfr         # git flow release
```

## Utilisation

### GitHub Codespaces
1. Cliquez sur "Code" → "Create codespace on main"
2. L'environnement se configure automatiquement
3. Attendez la fin du script de setup (~2-3 minutes)

### VS Code DevContainer
1. Installez l'extension "Dev Containers"
2. Ouvrez le projet dans VS Code
3. Cliquez sur "Reopen in Container" quand proposé
4. L'environnement se configure automatiquement

## Structure des Fichiers

- `devcontainer.json` - Configuration principale du container
- `setup.sh` - Script de post-installation pour outils additionnels
- `README.md` - Cette documentation

## Optimisations

- **Cache Cargo** : Les dépendances Rust sont mises en cache pour des builds plus rapides
- **Pré-compilation** : `cargo fetch` et `cargo check` exécutés au setup
- **Exclusions** : `target/` et autres dossiers exclus de la recherche VS Code
- **Ports** : 8080 et 3000 forwardés automatiquement

## Customisation

Pour modifier l'environnement :

1. Éditez `devcontainer.json` pour ajouter des features ou extensions
2. Modifiez `setup.sh` pour installer des outils supplémentaires
3. Reconstruisez le container : "Dev Containers: Rebuild Container"

## Dépannage

Si l'installation échoue :
1. Vérifiez les logs dans le terminal VS Code
2. Relancez `.devcontainer/setup.sh` manuellement
3. Reconstruisez le container depuis zéro

## Performance

Premier lancement : ~3-5 minutes (download + setup)
Lancements suivants : ~30-60 secondes (cache utilisé)