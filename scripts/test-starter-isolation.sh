#!/bin/bash
# Test script to verify that starters are isolated per-tab

set -e

echo "=== Testing Multi-Tab Starter Isolation ==="
echo ""
echo "This test verifies that Spring Boot starters are isolated per-tab"
echo "Each project should remember its own starter, not share with other tabs"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found"
    exit 1
fi

# Build the project
echo "Building lazymvn..."
cargo build --release

echo ""
echo "✓ Build successful"
echo ""
echo "Expected behavior:"
echo "  1. Each tab should maintain its own starters cache"
echo "  2. When pressing 's' in tab 1, it should use tab 1's starter"
echo "  3. When pressing 's' in tab 2, it should use tab 2's starter"
echo "  4. The starters should NOT be shared between tabs"
echo ""
echo "Current behavior (BUG):"
echo "  - starters_cache is global in TuiState"
echo "  - All tabs share the same starter"
echo "  - This causes wrong starter to be used in different projects"
echo ""
echo "To test after fix:"
echo "  1. Run: cargo run -- --project demo/multi-module"
echo "  2. Press 's' to select a starter for tab 1"
echo "  3. Press Ctrl+T to create tab 2 with a different project"
echo "  4. Press 's' in tab 2 - it should ask for a starter, not reuse tab 1's"
echo ""
