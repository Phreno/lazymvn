#!/bin/bash

# Script de test pour vérifier que les processus Maven sont tués à la fermeture

set -e

echo "=== Test du cleanup des processus Maven ==="
echo

cd /workspaces/lazymvn/demo/multi-module

echo "1. Vérification des processus Maven avant le test"
echo "   Processus Maven actuels:"
ps aux | grep -E "[m]vn|[j]ava.*maven" || echo "   Aucun processus Maven en cours"
echo

echo "2. Instructions pour le test manuel:"
echo "   a) Lancez: cargo run -- --project demo/multi-module --debug"
echo "   b) Sélectionnez un module (app ou library)"
echo "   c) Appuyez sur 's' pour démarrer l'application Spring Boot"
echo "   d) Attendez que le processus démarre (vous verrez les logs)"
echo "   e) Testez les 2 scénarios de fermeture:"
echo
echo "   SCÉNARIO 1: Fermeture normale avec 'q'"
echo "   - Appuyez sur 'q'"
echo "   - Vérifiez que le processus Java est tué"
echo
echo "   SCÉNARIO 2: Fermeture brutale avec Ctrl+C"
echo "   - Relancez l'app et démarrez le module"
echo "   - Appuyez sur Ctrl+C"
echo "   - Vérifiez que le processus Java est tué"
echo

echo "3. Vérification après fermeture:"
echo "   Exécutez cette commande pour voir les processus restants:"
echo "   ps aux | grep -E '[m]vn|[j]ava.*maven'"
echo
echo "   Si le cleanup fonctionne, aucun processus ne devrait rester."
echo

echo "4. Logs de debug:"
echo "   tail -f /workspaces/lazymvn/lazymvn-debug.log"
echo "   Cherchez les lignes:"
echo "   - 'Killing running Maven process with PID: ...'"
echo "   - 'Successfully killed Maven process ...'"
echo

echo "=== Vérification des processus Maven actuels ==="
ps aux | grep -E "[m]vn|[j]ava.*maven" | grep -v grep || echo "✓ Aucun processus Maven actif"
