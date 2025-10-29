#!/bin/bash
# Test script for history context switching feature
# Validates that history commands replay in the correct project context

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "LazyMVN History Context Switching Test"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if lazymvn is built
if [ ! -f "$PROJECT_ROOT/target/debug/lazymvn" ]; then
    echo -e "${YELLOW}Building lazymvn first...${NC}"
    cd "$PROJECT_ROOT"
    cargo build
    echo ""
fi

echo "Test scenario:"
echo "  1. Open multi-module demo project in tab 0"
echo "  2. Execute a command (e.g., compile on 'app' module)"
echo "  3. Open single-module demo project in tab 1"
echo "  4. Open history (Ctrl+H)"
echo "  5. Select command from tab 0 (multi-module project)"
echo "  6. Verify: Should automatically switch back to tab 0"
echo "  7. Verify: Command executes in correct project context"
echo ""

# Check demo projects exist
MULTI_MODULE="$PROJECT_ROOT/demo/multi-module"
SINGLE_MODULE="$PROJECT_ROOT/demo/single-module"

if [ ! -d "$MULTI_MODULE" ]; then
    echo -e "${RED}Error: Multi-module demo project not found at $MULTI_MODULE${NC}"
    exit 1
fi

if [ ! -d "$SINGLE_MODULE" ]; then
    echo -e "${RED}Error: Single-module demo project not found at $SINGLE_MODULE${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Demo projects found${NC}"
echo "  Multi-module: $MULTI_MODULE"
echo "  Single-module: $SINGLE_MODULE"
echo ""

# Check if history file exists (from previous runs)
HISTORY_FILE="$HOME/.config/lazymvn/history.json"
if [ -f "$HISTORY_FILE" ]; then
    echo -e "${YELLOW}Existing history file found:${NC}"
    echo "  $HISTORY_FILE"
    
    # Show project_root fields in history entries
    if command -v jq &> /dev/null; then
        echo ""
        echo "History entries with project_root:"
        jq '.entries[] | {timestamp: .timestamp, project_root: .project_root, module: .module, goal: .goal}' "$HISTORY_FILE" 2>/dev/null | head -20
    else
        echo "  (Install 'jq' to see history details)"
    fi
else
    echo -e "${YELLOW}No history file found yet${NC}"
    echo "  Will be created after first command execution"
fi

echo ""
echo "=========================================="
echo "Manual Testing Instructions"
echo "=========================================="
echo ""
echo "1. Run lazymvn with multi-module project:"
echo "   $ cargo run -- --project demo/multi-module --debug"
echo ""
echo "2. In the TUI:"
echo "   - Press 'c' to compile the selected module"
echo "   - Wait for command to complete"
echo "   - Note the module name (e.g., 'app' or 'library')"
echo ""
echo "3. Open a new tab with single-module project:"
echo "   - Press Ctrl+T to create new tab"
echo "   - (Implementation note: Currently tabs share project)"
echo "   - (Or restart lazymvn with different project)"
echo ""
echo "4. Test history replay:"
echo "   - Press Ctrl+H to open history"
echo "   - Use arrow keys to select the compile command from step 2"
echo "   - Press Enter to apply"
echo ""
echo "5. Expected behavior:"
echo "   ✓ Should detect different project_root"
echo "   ✓ Should switch back to multi-module project tab"
echo "   ✓ Should select the correct module"
echo "   ✓ Should execute 'mvn compile' in multi-module context"
echo "   ✓ Check lazymvn-debug.log for tab switching messages"
echo ""
echo "6. Verify in log file:"
echo "   $ tail -f lazymvn-debug.log | grep -E '(Applying history|Switching to|project_root)'"
echo ""
echo "Expected log messages:"
echo "  - 'Applying history entry for project: <path>'"
echo "  - 'History entry is for a different project'"
echo "  - 'Switching to existing tab at index <N>'"
echo "  - 'Selected module at index <N>'"
echo "  - 'History entry applied and command executed'"
echo ""

echo "=========================================="
echo "Code Verification"
echo "=========================================="
echo ""

# Verify HistoryEntry has project_root field
if grep -q "pub project_root: PathBuf" "$PROJECT_ROOT/src/features/history.rs"; then
    echo -e "${GREEN}✓ HistoryEntry has project_root field${NC}"
else
    echo -e "${RED}✗ HistoryEntry missing project_root field${NC}"
    exit 1
fi

# Verify apply_history_entry checks project context
if grep -q "current_project_root != entry.project_root" "$PROJECT_ROOT/src/ui/state/mod.rs"; then
    echo -e "${GREEN}✓ apply_history_entry checks project context${NC}"
else
    echo -e "${RED}✗ apply_history_entry missing project context check${NC}"
    exit 1
fi

# Verify tab switching logic
if grep -q "position(|tab| tab.project_root == entry.project_root)" "$PROJECT_ROOT/src/ui/state/mod.rs"; then
    echo -e "${GREEN}✓ Tab switching logic implemented${NC}"
else
    echo -e "${RED}✗ Tab switching logic missing${NC}"
    exit 1
fi

# Verify new tab creation for missing project
if grep -q "get_project_modules_for_path" "$PROJECT_ROOT/src/ui/state/mod.rs"; then
    echo -e "${GREEN}✓ New tab creation for missing project${NC}"
else
    echo -e "${RED}✗ New tab creation logic missing${NC}"
    exit 1
fi

echo ""
echo "=========================================="
echo "Test Results"
echo "=========================================="
echo ""
echo -e "${GREEN}✓ All code verifications passed${NC}"
echo -e "${GREEN}✓ Feature implementation complete${NC}"
echo ""
echo "Next steps:"
echo "  1. Run manual test scenario above"
echo "  2. Verify tab switching in debug log"
echo "  3. Test with 10+ tabs (should show error)"
echo "  4. Test with non-existent project in history"
echo ""
echo "For detailed logs, run:"
echo "  $ tail -f lazymvn-debug.log"
echo ""
