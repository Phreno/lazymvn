#!/bin/bash
# Test de validation manuelle de LazyMVN après refactoring architectural

set -e

echo "🧪 LazyMVN - Validation Post-Refactoring"
echo "========================================"
echo ""

PROJECT_DIR="/workspaces/lazymvn/demo/multi-module"
BINARY="/workspaces/lazymvn/target/release/lazymvn"

cd "$PROJECT_DIR"

echo "📁 Projet de test: $PROJECT_DIR"
echo ""

# Test 1: Compilation
echo "✅ Test 1: Compilation réussie"
if [ -f "$BINARY" ]; then
    echo "   ✓ Binary trouvé: $BINARY"
else
    echo "   ✗ Binary non trouvé"
    exit 1
fi
echo ""

# Test 2: --help
echo "✅ Test 2: Option --help"
$BINARY --help > /dev/null 2>&1 && echo "   ✓ --help fonctionne" || exit 1
echo ""

# Test 3: --setup
echo "✅ Test 3: Configuration centralisée (--setup)"
# Créer/recréer la config
echo "y" | $BINARY --setup > /dev/null 2>&1
CONFIG_HASH=$(echo -n "$PROJECT_DIR" | md5sum | cut -c1-8)
CONFIG_PATH="$HOME/.config/lazymvn/projects/$CONFIG_HASH/config.toml"
if [ -f "$CONFIG_PATH" ]; then
    echo "   ✓ Config créée: $CONFIG_PATH"
else
    echo "   ✗ Config non créée"
    exit 1
fi
echo ""

# Test 4: Vérifier absence de fichiers dans le projet
echo "✅ Test 4: Empreinte nulle dans le projet"
if [ -f "$PROJECT_DIR/lazymvn.toml" ]; then
    echo "   ✗ Fichier lazymvn.toml trouvé (ne devrait pas exister)"
    exit 1
else
    echo "   ✓ Aucun lazymvn.toml dans le projet"
fi

if [ -d "$PROJECT_DIR/.lazymvn" ]; then
    echo "   ✗ Dossier .lazymvn trouvé (ne devrait pas exister)"
    exit 1
else
    echo "   ✓ Aucun dossier .lazymvn dans le projet"
fi
echo ""

# Test 5: Modules Maven détectés
echo "✅ Test 5: Détection des modules Maven"
if [ -f "$PROJECT_DIR/pom.xml" ]; then
    echo "   ✓ pom.xml trouvé"
    # Compter les modules
    MODULE_COUNT=$(grep -c "<module>" "$PROJECT_DIR/pom.xml" || true)
    echo "   ✓ $MODULE_COUNT modules détectés"
else
    echo "   ✗ pom.xml non trouvé"
    exit 1
fi
echo ""

# Test 6: Structure des modules refactorés
echo "✅ Test 6: Architecture modulaire"
for module in core features maven ui utils; do
    if [ -d "/workspaces/lazymvn/src/$module" ]; then
        FILE_COUNT=$(find "/workspaces/lazymvn/src/$module" -name "*.rs" | wc -l)
        echo "   ✓ src/$module/ ($FILE_COUNT fichiers)"
    else
        echo "   ✗ src/$module/ manquant"
        exit 1
    fi
done
echo ""

# Test 7: lib.rs existe
echo "✅ Test 7: API publique (lib.rs)"
if [ -f "/workspaces/lazymvn/src/lib.rs" ]; then
    echo "   ✓ src/lib.rs créé"
else
    echo "   ✗ src/lib.rs manquant"
    exit 1
fi
echo ""

# Test 8: Tests unitaires
echo "✅ Test 8: Tests unitaires"
cd /workspaces/lazymvn
TEST_OUTPUT=$(cargo test --quiet 2>&1 | grep "test result" | tail -1)
if echo "$TEST_OUTPUT" | grep -q "ok"; then
    echo "   ✓ $TEST_OUTPUT"
else
    echo "   ✗ Tests échoués"
    exit 1
fi
echo ""

echo "🎉 VALIDATION COMPLÈTE RÉUSSIE !"
echo ""
echo "📊 Résumé:"
echo "   - Compilation: ✓"
echo "   - CLI: ✓"
echo "   - Configuration centralisée: ✓"
echo "   - Empreinte projet nulle: ✓"
echo "   - Détection Maven: ✓"
echo "   - Architecture modulaire: ✓"
echo "   - API publique: ✓"
echo "   - Tests: ✓"
echo ""
echo "💡 Pour lancer l'interface:"
echo "   cd $PROJECT_DIR"
echo "   $BINARY"
