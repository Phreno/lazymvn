# Test de l'Agent Java - Reconfiguration Périodique

## 🎯 Objectif

Tester la nouvelle version de l'agent qui **reconfigure Log4j 5 fois** (toutes les 2 secondes) pour contrer les reconfigurations tardives de `Log4jJbossLoggerFactory`.

---

## 📊 Analyse du Debug Report (28/10/2025 10:06)

### ✅ Ce qui fonctionne

L'agent démarre correctement :
```
[LazyMVN Agent] Starting Log4j reconfiguration agent...
[LazyMVN Agent] Will reconfigure Log4j with: file:///C:/Users/.../log4j-override-ec936686.properties
[LazyMVN Agent] ✓ Log4j successfully reconfigured with LazyMVN config
[LazyMVN Agent] ✓ Log format and levels from lazymvn.toml are now active
```

### ❌ Le problème identifié

**Timeline des logs :**
```
10:06:26.756 → [28/10/2025 10:06:26:756] [INFO ] ...  ← Ancien format (AVANT agent)
10:06:27.463 → [28/10/2025 10:06:27:463] [INFO ] ...  ← Ancien format (AVANT agent)
~10:06:27.5  → [LazyMVN Agent] ✓ Log4j successfully reconfigured...  ← Agent reconfigure
10:06:28.716 → [28/10/2025 10:06:28:716][INFO ] ...  ← ANCIEN FORMAT (RE-CONFIG !) ❌
10:06:28.817 → [28/10/2025 10:06:28:817][INFO ] ...  ← Ancien format persiste
```

**Cause** : `Log4jJbossLoggerFactory` se RE-INITIALISE **après** que l'agent ait reconfiguré Log4j.

**Preuve dans les logs :**
```
Log4jJbossLoggerFactory : utilise le fichier log4j.properties : app demarree sur server autre que JBoss  ← Instance 1
(agent reconfigure à t=2s)
Log4jJbossLoggerFactory : utilise le fichier log4j.properties : app demarree sur server autre que JBoss  ← Instance 2 (RE-CONFIG !)
```

---

## 🔧 Solution Implémentée

### Avant (Fix #8 original)

```java
Thread.sleep(2000); // Attendre 2 secondes
reconfigureLog4j(configUrl); // Reconfigurer UNE FOIS
```

**Problème** : Si JBoss factory se réinitialise après 2 secondes, la config LazyMVN est écrasée.

### Après (Fix #8.1 - commit c7e73b4)

```java
for (int attempt = 1; attempt <= 5; attempt++) {
    Thread.sleep(2000); // Attendre 2 secondes
    reconfigureLog4j(configUrl); // Forcer reconfiguration
    System.err.println("[LazyMVN Agent] Reconfiguration attempt " + attempt + "/5...");
}
```

**Avantages** :
- ✅ **Reconfigure toutes les 2 secondes** (5 fois au total = 10 secondes)
- ✅ **Écrase toute reconfiguration tardive** de JBoss factory
- ✅ **Dernière reconfiguration à t=10s** garantit que LazyMVN a le dernier mot
- ✅ **Pas d'impact sur performance** (thread daemon en arrière-plan)

---

## 🚀 Procédure de Test

### 1️⃣ Compiler le nouveau binaire

**Sur votre foo Windows :**

```bash
# Récupérer la dernière version
git pull origin develop

# Compiler en release
cargo build --release

# Le nouveau binaire est dans: target/release/lazymvn.exe
```

### 2️⃣ Lancer votre application

```bash
# Lancer LazyMVN
lazymvn --debug

# Appuyer sur Ctrl+R pour ouvrir recent projects
# Sélectionner application-api
# Appuyer sur 's' pour Spring Boot
```

### 3️⃣ Observer les logs de l'agent

**Logs attendus (NOUVEAU) :**

```
[LazyMVN Agent] Starting Log4j reconfiguration agent...
[LazyMVN Agent] Will reconfigure Log4j with: file:///C:/Users/.../log4j-override-ec936686.properties

(2 secondes plus tard)
[LazyMVN Agent] Reconfiguration attempt 1/5...
[LazyMVN Agent] ✓ Log4j successfully reconfigured with LazyMVN config
[LazyMVN Agent] ✓ Log format and levels from lazymvn.toml are now active

(2 secondes plus tard)
[LazyMVN Agent] Reconfiguration attempt 2/5...
[LazyMVN Agent] ✓ Log4j successfully reconfigured with LazyMVN config
[LazyMVN Agent] ✓ Log format and levels from lazymvn.toml are now active

(2 secondes plus tard)
[LazyMVN Agent] Reconfiguration attempt 3/5...
[LazyMVN Agent] ✓ Log4j successfully reconfigured with LazyMVN config
[LazyMVN Agent] ✓ Log format and levels from lazymvn.toml are now active

(2 secondes plus tard)
[LazyMVN Agent] Reconfiguration attempt 4/5...
[LazyMVN Agent] ✓ Log4j successfully reconfigured with LazyMVN config
[LazyMVN Agent] ✓ Log format and levels from lazymvn.toml are now active

(2 secondes plus tard)
[LazyMVN Agent] Reconfiguration attempt 5/5...
[LazyMVN Agent] ✓ Log4j successfully reconfigured with LazyMVN config
[LazyMVN Agent] ✓ Log format and levels from lazymvn.toml are now active
[LazyMVN Agent] Final reconfiguration completed
[LazyMVN Agent] LazyMVN log configuration should now persist
```

### 4️⃣ Vérifier le format des logs

**Après t=10 secondes, TOUS les logs devraient être au format LazyMVN :**

**✅ FORMAT ATTENDU (LazyMVN) :**
```
[INFO][fr.company.branch.assemblage] Starting ApplicationStarter...
[DEBUG][fr.company.branch.assemblage] Loaded configuration from...
[WARN][fr.company.branch.foo] Configuration warning...
[INFO][org.springframework.data.repository.config.RepositoryConfigurationDelegate] Bootstrapping Spring Data...
```

**❌ ANCIEN FORMAT (si agent échoue) :**
```
[28/10/2025 10:06:28:716][INFO ] fr.company.branch.foo...
[28/10/2025 10:06:28:817][INFO ] fr.company.branch.foo...
```

### 5️⃣ Vérifier les niveaux de log

**Selon votre `lazymvn.toml` :**

```toml
packages = [
 { name = "fr.company.branch.foo"      , level = "WARN" } ,  ← Seulement WARN et ERROR
 { name = "org.springframework"       , level = "WARN" } ,  ← Seulement WARN et ERROR
 { name = "com.couchbase"             , level = "WARN" } ,  ← Seulement WARN et ERROR
 { name = "fr.company.branch.assemblage", level = "DEBUG" },  ← Tous les logs (DEBUG+)
]
```

**Vérifications :**
- ✅ `fr.company.branch.assemblage` : Logs **DEBUG** et **INFO** visibles
- ✅ `fr.company.branch.foo` : Seulement logs **WARN** et **ERROR** (pas de DEBUG/INFO)
- ✅ `org.springframework` : Seulement logs **WARN** et **ERROR**
- ✅ `com.couchbase` : Seulement logs **WARN** et **ERROR**

---

## 📸 Copier le Debug Report

**Après avoir testé, appuyez sur `Shift+Y` pour copier le debug report.**

Cela me permettra de :
1. Vérifier que les 5 reconfigurations se sont exécutées
2. Confirmer que le format LazyMVN persiste après t=10s
3. Valider que les niveaux de log sont respectés

---

## 🎯 Résultats Attendus

### Scénario 1 : ✅ Succès (format LazyMVN après 10s)

**Logs montrent :**
```
[LazyMVN Agent] Reconfiguration attempt 1/5...
[LazyMVN Agent] Reconfiguration attempt 2/5...
[LazyMVN Agent] Reconfiguration attempt 3/5...
[LazyMVN Agent] Reconfiguration attempt 4/5...
[LazyMVN Agent] Reconfiguration attempt 5/5...
[LazyMVN Agent] Final reconfiguration completed

(puis TOUS les logs au format LazyMVN)
[INFO][fr.company.branch.assemblage] ...
[DEBUG][fr.company.branch.assemblage] ...
[WARN][fr.company.branch.foo] ...
```

**Conclusion** : ✅ **FIX RÉUSSI !** L'agent force la reconfiguration suffisamment de fois pour persister.

### Scénario 2 : ⚠️ Format mixte (ancien + nouveau)

**Logs montrent :**
```
(ancien format pendant 0-10 secondes)
[28/10/2025 10:06:28:716][INFO ] ...

(nouveau format après 10 secondes)
[INFO][fr.company.branch.assemblage] ...
```

**Conclusion** : ⚠️ **PARTIELLEMENT RÉUSSI** - Besoin d'augmenter durée ou fréquence.

### Scénario 3 : ❌ Ancien format persiste

**Logs montrent :**
```
[LazyMVN Agent] Reconfiguration attempt 1/5...
... (5 tentatives)
[LazyMVN Agent] Final reconfiguration completed

(mais logs ENCORE en ancien format)
[28/10/2025 10:06:40:000][INFO ] ...
```

**Conclusion** : ❌ **JBoss factory reconfigure EN CONTINU** → Besoin d'Option B (modifier le code).

---

## 🔧 Options de Secours

### Si Scénario 2 (format mixte)

**Augmenter la durée :**

```java
// Dans Log4jReconfigAgent.java, ligne ~45
for (int attempt = 1; attempt <= 10; attempt++) { // 10 tentatives au lieu de 5
    Thread.sleep(2000); // Total 20 secondes
    reconfigureLog4j(configUrl);
}
```

### Si Scénario 3 (échec complet)

**Passer à l'Option B** : Modifier `Log4jJbossLoggerFactory` dans votre code :

```java
// Dans Log4jJbossLoggerFactory.java (votre application)
public Log4jJbossLoggerFactory() {
    // NE PAS reconfigurer Log4j si déjà configuré
    if (!LogManager.getRootLogger().getAllAppenders().hasMoreElements()) {
        PropertyConfigurator.configure(...); // Configuration par défaut
    }
    // Sinon, garder la configuration existante (LazyMVN)
}
```

---

## 📝 Notes Techniques

### Pourquoi 5 tentatives ?

- **t=0s** : Application démarre
- **t=2s** : Tentative 1 (potentiellement trop tôt)
- **t=4s** : Tentative 2 (JBoss factory peut se réinitialiser)
- **t=6s** : Tentative 3 (Spring Boot en cours de chargement)
- **t=8s** : Tentative 4 (Beans en cours d'initialisation)
- **t=10s** : Tentative 5 (**FINALE** - application normalement stable)

### Impact Performance

**Aucun** :
- Thread daemon (ne bloque pas la JVM)
- 5 appels à `PropertyConfigurator.configure()` = ~50ms total
- Négligeable comparé au temps de démarrage Spring Boot (~30 secondes)

### Pourquoi pas reconfigurer indéfiniment ?

- ❌ **Gaspillage ressources** (boucle infinie inutile)
- ❌ **Logs pollués** (messages d'agent en continu)
- ✅ **10 secondes suffisantes** pour couvrir phase d'initialisation

---

## ✅ Checklist de Test

- [ ] Compiler `cargo build --release`
- [ ] Lancer `lazymvn --debug`
- [ ] Ouvrir projet `application-api`
- [ ] Lancer Spring Boot (`s`)
- [ ] Observer logs de l'agent (5 tentatives)
- [ ] Vérifier format logs après 10 secondes
- [ ] Vérifier niveaux de log (WARN pour foo, DEBUG pour assemblage)
- [ ] Copier debug report (`Shift+Y`)
- [ ] Partager résultats

---

## 📞 Retour Attendu

Après test, merci de partager :

1. **Debug report complet** (Shift+Y)
2. **Screenshot des logs** entre 0-15 secondes
3. **Confirmation format** : LazyMVN ou Ancien ?
4. **Niveaux de log** : Respectés ou non ?

Cela me permettra d'ajuster la stratégie si nécessaire ! 🚀
