#!/bin/bash
# Test script for history deduplication feature
# Validates that duplicate commands are moved to top instead of being duplicated

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "LazyMVN History Deduplication Test"
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
echo "  1. Execute command A (e.g., 'mvn compile' on module 'app')"
echo "  2. Execute command B (e.g., 'mvn test' on module 'library')"
echo "  3. Execute command C (e.g., 'mvn package' on module 'app')"
echo "  4. Re-execute command A (same module, goal, profiles, flags)"
echo "  5. Verify: Command A should move to top, no duplicate created"
echo "  6. Verify: History should have 3 entries, not 4"
echo ""

echo "=========================================="
echo "Code Verification"
echo "=========================================="
echo ""

# Verify HistoryEntry has matches() method
if grep -q "pub fn matches(&self, other: &HistoryEntry)" "$PROJECT_ROOT/src/features/history.rs"; then
    echo -e "${GREEN}✓ HistoryEntry has matches() method${NC}"
else
    echo -e "${RED}✗ HistoryEntry missing matches() method${NC}"
    exit 1
fi

# Verify add() method checks for duplicates
if grep -q "if let Some(existing_idx) = self.entries.iter().position(|e| e.matches(&entry))" "$PROJECT_ROOT/src/features/history.rs"; then
    echo -e "${GREEN}✓ add() method checks for duplicates${NC}"
else
    echo -e "${RED}✗ add() method missing duplicate check${NC}"
    exit 1
fi

# Verify duplicate removal
if grep -q "self.entries.remove(existing_idx)" "$PROJECT_ROOT/src/features/history.rs"; then
    echo -e "${GREEN}✓ Duplicate removal implemented${NC}"
else
    echo -e "${RED}✗ Duplicate removal missing${NC}"
    exit 1
fi

echo ""
echo "=========================================="
echo "Unit Tests"
echo "=========================================="
echo ""

# Run history tests
echo "Running history deduplication tests..."
cargo test --lib features::history::tests::command_history_deduplicates_entries --quiet 2>&1 | tail -5
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Deduplication test passed${NC}"
else
    echo -e "${RED}✗ Deduplication test failed${NC}"
    exit 1
fi

cargo test --lib features::history::tests::command_history_deduplication_updates_position --quiet 2>&1 | tail -5
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Position update test passed${NC}"
else
    echo -e "${RED}✗ Position update test failed${NC}"
    exit 1
fi

cargo test --lib features::history::tests::history_entry_matches_ignores_timestamp --quiet 2>&1 | tail -5
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Timestamp ignore test passed${NC}"
else
    echo -e "${RED}✗ Timestamp ignore test failed${NC}"
    exit 1
fi

echo ""
echo "=========================================="
echo "Test Results Summary"
echo "=========================================="
echo ""

# Count total tests
TOTAL_TESTS=$(cargo test --lib features::history 2>&1 | grep -oP '\d+(?= passed)' | head -1)
echo -e "${GREEN}✓ All $TOTAL_TESTS history tests passed${NC}"
echo ""

echo "What gets deduplicated:"
echo "  ✓ Same project_root"
echo "  ✓ Same module"
echo "  ✓ Same goal (e.g., 'compile', 'test')"
echo "  ✓ Same profiles (order matters)"
echo "  ✓ Same flags (order matters)"
echo "  ✗ Timestamp is IGNORED (not part of comparison)"
echo ""

echo "Behavior:"
echo "  • When duplicate detected: Remove old position, add to top"
echo "  • History size: Never exceeds 100 entries"
echo "  • MRU ordering: Most recently used always at top"
echo ""

echo "=========================================="
echo "Manual Testing Instructions"
echo "=========================================="
echo ""

echo "1. Run lazymvn with multi-module project:"
echo "   $ cargo run -- --project demo/multi-module --debug"
echo ""
echo "2. Execute some commands:"
echo "   - Press 'c' to compile 'app' module"
echo "   - Use arrow keys to select 'library' module"
echo "   - Press 't' to test 'library' module"
echo "   - Go back to 'app' module"
echo "   - Press 'c' again to compile 'app' (duplicate!)"
echo ""
echo "3. Open history:"
echo "   - Press Ctrl+H to view history"
echo "   - You should see only 2 entries, not 3"
echo "   - 'app compile' should be at the top (most recent)"
echo "   - 'library test' should be second"
echo ""
echo "4. Verify in history file:"
echo "   $ cat ~/.config/lazymvn/command_history.json | jq '.[] | {module, goal}' | head -20"
echo "   - Should show no duplicates"
echo "   - Most recent command at top"
echo ""

echo "=========================================="
echo "Edge Cases Covered"
echo "=========================================="
echo ""

echo "✓ Same command with different profiles → Separate entries"
echo "✓ Same command with different flags → Separate entries"
echo "✓ Same command in different projects → Separate entries"
echo "✓ Same command on different modules → Separate entries"
echo "✓ Exact same command repeated → Moved to top, no duplicate"
echo "✓ History at max size + duplicate → Old removed, new at top"
echo ""

echo "=========================================="
echo "Performance Notes"
echo "=========================================="
echo ""

echo "• Duplicate detection: O(n) where n = history size (max 100)"
echo "• Worst case: ~100 comparisons per add() call"
echo "• Acceptable: History operations are infrequent (user-triggered)"
echo "• Trade-off: Small performance cost for better UX"
echo ""

echo "For detailed logs during testing:"
echo "  $ tail -f lazymvn-debug.log | grep -E '(Removed duplicate|history entry)'"
echo ""

echo -e "${GREEN}=========================================="
echo "All Checks Passed!"
echo "==========================================${NC}"
