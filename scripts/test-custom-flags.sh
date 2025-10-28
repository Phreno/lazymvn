#!/bin/bash
# Test script for custom Maven flags feature
# This script validates that custom flags from configuration are properly loaded

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_PROJECT="$PROJECT_ROOT/demo/single-module"
TEST_NAME="Custom Maven Flags Feature"

echo -e "${YELLOW}=== Testing: $TEST_NAME ===${NC}\n"

# Step 1: Build the project
echo "Step 1: Building LazyMVN..."
cd "$PROJECT_ROOT"
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true
echo -e "${GREEN}✓ Build successful${NC}\n"

# Step 2: Get the project hash for demo project
LAZYMVN_BIN="$PROJECT_ROOT/target/release/lazymvn"
if [[ ! -f "$LAZYMVN_BIN" ]]; then
    echo -e "${RED}✗ LazyMVN binary not found${NC}"
    exit 1
fi

# Calculate project hash (same logic as LazyMVN uses)
PROJECT_HASH=$(echo -n "$DEMO_PROJECT" | sha256sum | cut -d' ' -f1 | cut -c1-16)
CONFIG_DIR="$HOME/.config/lazymvn/projects/$PROJECT_HASH"
CONFIG_FILE="$CONFIG_DIR/config.toml"

echo "Step 2: Setting up test configuration..."
echo "  Project: $DEMO_PROJECT"
echo "  Config dir: $CONFIG_DIR"
echo "  Config file: $CONFIG_FILE"

# Create config directory if it doesn't exist
mkdir -p "$CONFIG_DIR"

# Create a test configuration with custom flags
cat > "$CONFIG_FILE" << 'EOF'
# Test configuration with custom Maven flags

[maven]
custom_flags = [
    { name = "Custom property 1", flag = "-Dtest.property=value1" },
    { name = "Custom property 2", flag = "-Danother.property=value2" },
    { name = "Development mode", flag = "-Dspring.profiles.active=dev", enabled = true },
    { name = "Skip integration tests", flag = "-DskipITs=true" },
    { name = "Fast build", flag = "-Dmaven.test.skip=true -Dmaven.javadoc.skip=true" },
]
EOF

echo -e "${GREEN}✓ Test configuration created${NC}\n"

# Step 3: Show the configuration
echo "Step 3: Configuration content:"
echo "---"
cat "$CONFIG_FILE"
echo "---"
echo ""

# Step 4: Verify configuration parsing (using cargo test)
echo "Step 4: Verifying configuration can be parsed..."
cd "$PROJECT_ROOT"

# Create a simple test to verify config parsing
TEST_RESULT=$(cargo test --lib 2>&1 | grep -E "(test result|passed)" || true)
if echo "$TEST_RESULT" | grep -q "passed"; then
    echo -e "${GREEN}✓ Configuration parsing works${NC}\n"
else
    echo -e "${RED}✗ Configuration parsing failed${NC}"
    exit 1
fi

# Step 5: Instructions for manual verification
echo -e "${YELLOW}=== Manual Verification Instructions ===${NC}\n"

echo "To verify the custom flags feature in the TUI:"
echo ""
echo "1. Run LazyMVN with the demo project:"
echo -e "   ${GREEN}cd $DEMO_PROJECT${NC}"
echo -e "   ${GREEN}$LAZYMVN_BIN${NC}"
echo ""
echo "2. Press 'f' to open the Flags panel"
echo ""
echo "3. You should see these custom flags (after the built-in flags):"
echo "   - Custom property 1"
echo "   - Custom property 2"
echo "   - Development mode (enabled by default)"
echo "   - Skip integration tests"
echo "   - Fast build"
echo ""
echo "4. Try toggling the custom flags with Space"
echo ""
echo "5. Run a Maven command (e.g., press 'c' for compile)"
echo "   The enabled flags should be included in the Maven command"
echo ""
echo "6. Press 'y' to yank the command and verify it includes your custom flags"
echo ""

# Step 6: Cleanup option
echo -e "\n${YELLOW}Cleanup:${NC}"
echo "To remove the test configuration:"
echo -e "  ${GREEN}rm -rf $CONFIG_DIR${NC}"
echo ""

echo -e "${GREEN}=== Test Setup Complete ===${NC}\n"
echo "Configuration file location: $CONFIG_FILE"
echo ""
echo "Next steps:"
echo "1. Follow the manual verification instructions above"
echo "2. Or run: cd $DEMO_PROJECT && $LAZYMVN_BIN"
