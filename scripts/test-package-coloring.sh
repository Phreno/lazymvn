#!/bin/bash
# Test package name coloring in log output
# This script demonstrates the new package name highlighting feature

set -e

echo "🧪 Testing package name coloring in log output"
echo ""

# Create a simple test showing colored output
cat << 'EOF'
La fonctionnalité de coloration des packages est maintenant implémentée !

Exemples de logs qui seront colorisés:
  [INFO] com.example.service.UserService - User created successfully
  [DEBUG] org.springframework.boot.SpringApplication - Starting application
  [ERROR] com.myapp.database.ConnectionPool - Failed to connect
  [WARN] test.package.MyClass - Configuration missing

Avec le format de log: [%p] %c - %m%n

Les noms de packages (comme "com.example.service.UserService") seront 
affichés en CYAN pour les distinguer visuellement dans l'output Maven.

La détection est basée sur le pattern de log configuré dans lazymvn.toml,
ce qui garantit une précision maximale (pas de faux positifs comme avec 
une regex générique).

Pour activer cette fonctionnalité:
1. Le format de log doit contenir %c (logger/package)
2. Exemple dans lazymvn.toml:
   
   [logging]
   log_format = "[%p] %c - %m%n"
   
3. LazyMVN détectera automatiquement et colorisera les packages

EOF

# Run unit tests to validate
echo ""
echo "📊 Running unit tests..."
cargo test test_colorize_log_line_with --lib --quiet

echo ""
echo "📊 Running package extraction tests..."
cargo test test_extract_package --lib --quiet

echo ""
echo "✅ All tests passed! Package coloring is ready to use."
echo ""
echo "💡 Tip: Configure your log format with %c to enable package highlighting"
echo "   Example: log_format = \"[%p] %c{1} - %m%n\""
