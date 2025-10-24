# Debug Yank Feature

Cette fonctionnalité permet de copier toutes les informations utiles au débogage en une seule action.

## Utilisation

Appuyez sur `Shift+Y` (Y majuscule) dans l'interface pour copier un rapport de debug complet dans le presse-papier.

## Contenu du Rapport

Le rapport de debug inclut les sections suivantes :

### 1. En-tête
- Date et heure de génération du rapport
- Délimiteurs visuels pour faciliter la lecture

### 2. Informations de Version
- Version de LazyMVN (depuis `Cargo.toml`)
- Date de build (si disponible via vergen)
- Branche Git (si disponible via vergen)
- Hash du commit Git (si disponible via vergen)

### 3. Informations Système
- Système d'exploitation (Linux, Windows, macOS, etc.)
- Architecture (x86_64, aarch64, etc.)

### 4. Configuration
- Contenu complet du fichier `lazymvn.toml`
- Si le fichier n'existe pas, un message l'indique

### 5. Output de Tous les Onglets
Pour chaque onglet ouvert :
- Numéro et nom de l'onglet (nom du dossier du projet)
- Indicateur `[ACTIVE]` pour l'onglet actif
- Chemin complet du projet
- Module actuellement sélectionné
- Nombre de lignes de sortie
- Les 100 dernières lignes de sortie (ou toutes si moins de 100)

### 6. Logs LazyMVN
- 500 dernières lignes des logs de debug
- 500 dernières lignes des logs d'erreur

### 7. Pied de page
- Marqueur de fin de rapport

## Comparaison avec `y` (yank simple)

| Commande | Contenu copié | Cas d'usage |
|----------|---------------|-------------|
| `y` | Output de l'onglet actif uniquement | Partager la sortie d'une seule commande Maven |
| `Y` (Shift+Y) | Rapport de debug complet (voir ci-dessus) | Rapporter un bug, demander de l'aide, archiver l'état complet |

## Exemple de Structure du Rapport

```
================================================================================
LazyMVN Debug Report
================================================================================
Generated: 2025-10-24 15:30:45

=== Version Information ===
LazyMVN Version: 0.4.0-unstable

=== System Information ===
OS: linux
Architecture: x86_64

=== Configuration (lazymvn.toml) ===
[logging]
packages = [
  { name = "org.springframework", level = "WARN" },
  { name = "com.mycompany", level = "DEBUG" },
]

=== Output from All Tabs ===
--- Tab 1: multi-module [ACTIVE] ---
Project Root: /workspaces/lazymvn/demo/multi-module
Module: app
Output Lines: 45
(Showing last 45 lines of 45)
$ mvn clean install -pl app
[INFO] Scanning for projects...
[INFO] 
[INFO] -----------------------< com.example:app >-----------------------
...

--- Tab 2: single-module ---
Project Root: /workspaces/lazymvn/demo/single-module
Module: .
Output Lines: 0
(No output)

=== LazyMVN Logs ===
=== Debug Logs (last 500 lines) ===
[SESSION:20251024-153000-123] [2025-10-24 15:30:00.123] INFO - === LazyMVN Session Started ===
[SESSION:20251024-153000-123] [2025-10-24 15:30:00.124] INFO - Session ID: 20251024-153000-123
...

=== Error Logs (last 500 lines) ===
(No error logs)

================================================================================
End of Debug Report
================================================================================
```

## Implémentation

### Fichiers Modifiés

1. **src/logger.rs**
   - Ajout de `get_all_logs()` : récupère les 500 dernières lignes des logs debug et erreur
   - Ajout de `read_last_lines()` : fonction utilitaire pour lire les N dernières lignes d'un fichier

2. **src/ui/state/mod.rs**
   - Ajout de `yank_debug_info()` : agrège toutes les informations et les copie dans le presse-papier
   - Parcourt tous les tabs pour collecter leurs outputs
   - Utilise les mêmes mécanismes de copie que `yank_output()` (wl-copy, xclip, PowerShell, pbcopy, etc.)

3. **src/ui/keybindings/mod.rs**
   - Ajout du binding `KeyCode::Char('Y')` qui appelle `state.yank_debug_info()`

4. **README.md**
   - Documentation de la nouvelle touche `Y` dans la section "Selection & Search"

### Tests Manuels

Pour tester la fonctionnalité :

1. Compiler : `cargo build --release`
2. Lancer avec debug : `./target/release/lazymvn --debug --project demo/multi-module`
3. Exécuter quelques commandes (b, t, s, etc.)
4. Ouvrir plusieurs onglets (Ctrl+T)
5. Appuyer sur `Y` (Shift+Y)
6. Vérifier que le message de confirmation apparaît
7. Coller le contenu du presse-papier dans un éditeur
8. Vérifier que toutes les sections sont présentes

### Dépendances

Aucune nouvelle dépendance requise. La fonctionnalité utilise :
- `std::fs` pour lire le fichier de configuration
- `chrono` (déjà présent) pour les timestamps
- `env!` et `option_env!` pour les informations de version
- Mécanismes de clipboard existants (arboard, wl-copy, etc.)

## Cas d'Usage

### Rapporter un Bug
1. Reproduire le bug
2. Appuyer sur `Y`
3. Coller le rapport dans l'issue GitHub
4. Les mainteneurs ont toutes les informations nécessaires

### Demander de l'Aide
1. Rencontrer un problème
2. Appuyer sur `Y`
3. Partager le rapport sur Discord/forum
4. Les autres peuvent voir exactement votre configuration et état

### Debug Personnel
1. Avant de faire des changements, sauvegarder l'état avec `Y`
2. Coller dans un fichier texte
3. Faire les modifications
4. Si problème, comparer avec l'état sauvegardé

### Documentation
1. Créer un tutoriel ou guide
2. Appuyer sur `Y` à chaque étape importante
3. Inclure les rapports dans la documentation
4. Les lecteurs voient exactement ce qui se passe

## Notes Techniques

### Gestion de la Taille
- Logs : limités aux 500 dernières lignes (configurable dans le code)
- Output par tab : limité aux 100 dernières lignes si > 100
- Cela évite des rapports de plusieurs Mo

### Performance
- La fonction parcourt tous les tabs, mais l'opération est rapide
- Lecture des fichiers de log est bufferisée
- Pas de blocage de l'interface utilisateur

### Sécurité
- Les fichiers de configuration peuvent contenir des chemins sensibles
- Les logs peuvent contenir des informations du projet
- **Attention** : vérifier le contenu avant de partager publiquement

### Compatibilité
- Linux : utilise wl-copy (Wayland) ou xclip/xsel (X11)
- Windows : utilise PowerShell Set-Clipboard ou clip.exe
- macOS : utilise pbcopy
- Fallback : arboard (bibliothèque Rust multiplateforme)
