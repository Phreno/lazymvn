#!/bin/bash

# LazyMVN Demo - Maven Settings Testing Script
# This script demonstrates various Maven commands using the custom settings.xml

echo "=== LazyMVN Demo - Maven Settings Testing ==="
echo ""

SETTINGS_FILE="settings.xml"
PROJECT_DIR="$(dirname "$0")"

if [ ! -f "$SETTINGS_FILE" ]; then
    echo "❌ Settings file not found: $SETTINGS_FILE"
    echo "Make sure you're running this script from the demo-project directory"
    exit 1
fi

echo "📋 Available Maven commands with custom settings:"
echo ""

echo "1. List all profiles (including settings.xml profiles):"
echo "   mvn --settings $SETTINGS_FILE help:all-profiles"
echo ""

echo "2. Build with development profile:"
echo "   mvn --settings $SETTINGS_FILE clean compile -Pdev"
echo ""

echo "3. Build with multiple profiles:"
echo "   mvn --settings $SETTINGS_FILE clean compile -Pdev,integration-tests"
echo ""

echo "4. Fast build (skip tests):"
echo "   mvn --settings $SETTINGS_FILE clean package -Pfast"
echo ""

echo "5. Release build with quality checks:"
echo "   mvn --settings $SETTINGS_FILE clean package -Prelease,quality"
echo ""

echo "6. Run with specific environment:"
echo "   mvn --settings $SETTINGS_FILE spring-boot:run -Denv=dev"
echo ""

echo "7. Database migration example:"
echo "   mvn --settings $SETTINGS_FILE flyway:migrate -Pdb-migrate"
echo ""

echo "8. Show effective POM with settings applied:"
echo "   mvn --settings $SETTINGS_FILE help:effective-pom"
echo ""

echo "9. Show effective settings:"
echo "   mvn --settings $SETTINGS_FILE help:effective-settings"
echo ""

echo "10. Run tests with coverage:"
echo "    mvn --settings $SETTINGS_FILE test -Pquality"
echo ""

echo "🚀 LazyMVN Usage:"
echo "   Start LazyMVN from this directory to automatically use the custom settings:"
echo "   ../target/debug/lazymvn"
echo ""
echo "   The profiles from both pom.xml and settings.xml should be available!"
echo ""

echo "💡 Tip: Press 'p' in LazyMVN to view and toggle profiles"
echo "💡 Tip: Use spacebar to select/deselect profiles before running commands"
echo ""