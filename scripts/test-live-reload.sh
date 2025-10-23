#!/bin/bash

# Script de test pour la fonctionnalité de live reload
# Ce script vérifie que les changements de configuration sont bien appliqués

set -e

echo "=== Test du Live Config Reload ==="
echo

cd /workspaces/lazymvn/demo/multi-module

# S'assurer qu'un fichier de config existe
if [ ! -f lazymvn.toml ]; then
    echo "Création du fichier de configuration de test..."
    cp ../../lazymvn.toml.example lazymvn.toml
    echo "✓ Fichier créé"
else
    echo "✓ Fichier de configuration existe déjà"
fi

echo
echo "Contenu actuel du fichier de configuration:"
echo "---"
cat lazymvn.toml | head -25
echo "---"

echo
echo "Instructions de test manuel:"
echo "1. Lancez : cargo run -- --project demo/multi-module --debug"
echo "2. Appuyez sur Ctrl+E pour éditer la configuration"
echo "3. Changez une valeur (par exemple launch_mode = \"force-exec\")"
echo "4. Sauvegardez et fermez l'éditeur"
echo "5. Vérifiez dans lazymvn-debug.log que le changement est détecté"
echo
echo "Commande pour suivre les logs:"
echo "  tail -f /workspaces/lazymvn/lazymvn-debug.log"
echo
echo "Pour nettoyer après le test:"
echo "  rm demo/multi-module/lazymvn.toml"
