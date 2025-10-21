# LazyMVN Development Container

Configuration minimaliste pour GitHub Codespaces.

## Configuration

- **Image de base** : `mcr.microsoft.com/devcontainers/rust:1-1-bookworm`
- **Java 21** avec Maven
- **Extensions VS Code** : rust-analyzer, lldb, crates, toml

## Utilisation

1. Ouvrir le repo dans GitHub Codespaces
2. L'environnement se configure automatiquement en ~30 secondes
3. Commencer à développer avec `cargo build`, `cargo test`, etc.

Pas de configuration supplémentaire nécessaire - tout est inclus dans l'image de base Rust.
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

### Erreur de Création du Container
Si vous voyez une erreur comme :
```
Error: create /var/lib/docker/codespacemount/workspace/lazymvn/.devcontainer/cargo-cache: includes invalid characters for a local volume name
```

Cela signifie que les noms de volumes Docker doivent suivre des conventions strictes. Notre configuration utilise `lazymvn_cargo_cache` comme volume nommé au lieu d'un volume basé sur un chemin pour éviter ce problème.

### Container de Récupération
Si le container principal échoue au démarrage, GitHub Codespaces créera automatiquement un container de récupération avec Alpine Linux. Vous pouvez :
1. Corriger la configuration devcontainer.json
2. Commiter et pousser les changements
3. Reconstruire le container depuis la Palette de Commandes : "Codespaces: Rebuild Container"

### Cache des Volumes
Le cache du registre Cargo est stocké dans un volume Docker nommé `lazymvn_cargo_cache` qui persiste entre les reconstructions de container, rendant les builds suivants beaucoup plus rapides.

### Autres Problèmes
Si l'installation échoue :
1. Vérifiez les logs dans le terminal VS Code
2. Relancez `.devcontainer/setup.sh` manuellement
3. Reconstruisez le container depuis zéro

## Performance

Premier lancement : ~3-5 minutes (download + setup)
Lancements suivants : ~30-60 secondes (cache utilisé)