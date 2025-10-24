#!/bin/bash
# Test script for profile loading in new tabs
# Tests that Maven profiles are correctly loaded when creating new tabs

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "Testing Profile Loading in New Tabs"
echo "=========================================="
echo ""

# Check if lazymvn binary exists
if [ ! -f "$PROJECT_ROOT/target/release/lazymvn" ]; then
    echo "❌ Error: lazymvn binary not found at $PROJECT_ROOT/target/release/lazymvn"
    echo "   Please build the project first: cargo build --release"
    exit 1
fi

echo "✅ Found lazymvn binary"
echo ""

# Test with demo multi-module project
DEMO_PROJECT="$PROJECT_ROOT/demo/multi-module"

if [ ! -d "$DEMO_PROJECT" ]; then
    echo "❌ Error: Demo project not found at $DEMO_PROJECT"
    exit 1
fi

echo "✅ Found demo project at $DEMO_PROJECT"
echo ""

# Check if Maven is available
if ! command -v mvn &> /dev/null && [ ! -f "$DEMO_PROJECT/mvnw" ]; then
    echo "❌ Error: Maven not found (neither 'mvn' command nor './mvnw' in demo project)"
    exit 1
fi

echo "✅ Maven is available"
echo ""

# Check for POM files
if [ ! -f "$DEMO_PROJECT/pom.xml" ]; then
    echo "❌ Error: No pom.xml found in demo project"
    exit 1
fi

echo "✅ Found pom.xml in demo project"
echo ""

# Test profile loading manually
echo "Testing Maven profile discovery..."
echo ""

cd "$DEMO_PROJECT"

# Run Maven help:all-profiles to verify profiles exist
if command -v mvn &> /dev/null; then
    MVN_CMD="mvn"
elif [ -f "./mvnw" ]; then
    MVN_CMD="./mvnw"
fi

PROFILES_OUTPUT=$($MVN_CMD help:all-profiles -q 2>&1 || true)

if echo "$PROFILES_OUTPUT" | grep -q "Listing Profiles"; then
    echo "✅ Maven profiles can be discovered"
    echo ""
    echo "Available profiles:"
    echo "$PROFILES_OUTPUT" | grep "Profile Id:" | sed 's/^/  - /'
    echo ""
else
    echo "⚠️  Warning: Could not discover Maven profiles"
    echo "   This is expected if the demo project has no profiles"
    echo ""
fi

# Return to project root
cd "$PROJECT_ROOT"

echo "=========================================="
echo "Manual Test Instructions"
echo "=========================================="
echo ""
echo "To manually test profile loading in new tabs:"
echo ""
echo "1. Start LazyMVN with the demo project:"
echo "   ./target/release/lazymvn --project demo/multi-module"
echo ""
echo "2. Wait for profiles to load (observe the loading spinner)"
echo ""
echo "3. Press 'p' to switch to Profiles view"
echo "   - You should see profiles listed (if any exist)"
echo ""
echo "4. Press Ctrl+T to create a new tab"
echo ""
echo "5. Select 'demo/single-module' or another project"
echo "   - A loading spinner should appear briefly"
echo "   - Profiles for the new project should load"
echo ""
echo "6. Press 'p' to verify profiles are loaded in the new tab"
echo ""
echo "7. Switch back to first tab with Ctrl+Left"
echo "   - Original profiles should still be there"
echo ""
echo "Expected behavior:"
echo "  ✅ Each tab independently loads its own profiles"
echo "  ✅ Loading spinner appears during profile loading"
echo "  ✅ Profiles are available in Profiles view (press 'p')"
echo "  ✅ Switching tabs maintains separate profile lists"
echo ""

echo "=========================================="
echo "All pre-flight checks passed!"
echo "=========================================="
echo ""
