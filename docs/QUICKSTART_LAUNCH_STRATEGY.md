# Quick Start : Launch Strategy Intelligente

LazyMVN choisit désormais automatiquement entre `spring-boot:run` et `exec:java` **sans échec**.

## 🚀 Utilisation Rapide

### Option 1 : Mode Auto (recommandé)

```bash
# Lance avec détection automatique
lazymvn
```

Dans l'interface :
1. Sélectionne ton module
2. Appuie sur `s` (start)
3. LazyMVN détecte et lance automatiquement ! 🎯

### Option 2 : Forcer la stratégie

```bash
# Toujours spring-boot:run
lazymvn --force-run

# Toujours exec:java  
lazymvn --force-exec
```

## 🔧 Configuration (optionnel)

Crée `lazymvn.toml` à la racine de ton projet :

```toml
launch_mode = "auto"  # ou "force-run" ou "force-exec"
```

## 📋 Comment ça marche ?

Quand tu appuies sur `s` :

1. **Détection** → LazyMVN analyse ton `pom.xml` effectif
2. **Décision** → Choisit `spring-boot:run` ou `exec:java`
3. **Exécution** → Lance avec la bonne commande !

**Critères de décision (mode auto) :**
- ✅ Plugin Spring Boot + packaging jar/war → `spring-boot:run`
- ⚠️ Pas de plugin ou packaging pom → `exec:java`

## 📖 Logs Visibles

Dans le panneau Output, tu verras :

```
[INFO] Detecting Spring Boot capabilities for module: Some("my-app")
[DEBUG] Found spring-boot-maven-plugin
[DEBUG] Found packaging: jar
[INFO] Auto mode: Spring Boot plugin detected, using spring-boot:run
$ mvn "-Dspring-boot.run.profiles=dev" spring-boot:run
```

## 🎯 Exemples

### Projet Spring Boot standard

```bash
lazymvn
# Appuie sur 's'
# → Détecte Spring Boot plugin
# → Lance: mvn spring-boot:run
```

### Projet sans Spring Boot

```bash
lazymvn
# Appuie sur 's'
# → Pas de plugin Spring Boot
# → Lance: mvn exec:java -Dexec.mainClass=...
```

### Forcer exec:java (tests, CI/CD...)

```bash
lazymvn --force-exec
# Appuie sur 's'  
# → Force exec:java même si Spring Boot détecté
```

## ⚙️ Priorité des Configurations

1. **CLI** (`--force-run`, `--force-exec`) → **priorité maximale**
2. **Fichier** `lazymvn.toml` → si pas de flag CLI
3. **Défaut** `auto` → si rien configuré

## 🆘 Dépannage

**Erreur de détection ?**
→ Fallback automatique sur `spring-boot:run` avec warning

**Mauvaise stratégie choisie ?**
→ Utilise `--force-run` ou `--force-exec`

**Voir les détails de détection ?**
→ Lance avec `lazymvn --debug`

## 📚 Documentation Complète

- **Guide complet** : [SPRING_BOOT_LAUNCHER.md](SPRING_BOOT_LAUNCHER.md)
- **Technique** : [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)
- **Usage général** : [README.md](README.md)

---

**🎉 C'est tout ! Lance simplement `lazymvn` et appuie sur `s` !**
