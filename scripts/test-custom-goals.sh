#!/bin/bash
# Test script for custom goals feature

set -e

echo "=== Testing Custom Goals Feature ==="
echo ""

# Build the binary
echo "1. Building LazyMVN..."
cargo build --release --quiet
echo "✓ Build successful"
echo ""

# Test with demo project
echo "2. Configuration check..."
cd demo/multi-module
if [ -f lazymvn.toml ]; then
    echo "✓ lazymvn.toml found"
    echo ""
    echo "Custom goals configured:"
    grep -A 10 "custom_goals" lazymvn.toml || echo "  (none found)"
else
    echo "✗ No lazymvn.toml in demo/multi-module"
    exit 1
fi
echo ""

echo "3. Manual testing instructions:"
echo "   a. Run: ../../target/release/lazymvn"
echo "   b. Press Ctrl+G to open custom goals popup"
echo "   c. Use ↑↓ to navigate, Enter to execute, Esc to close"
echo "   d. Expected: Popup with 3 goals (Format Code, Checkstyle, Dependency Tree)"
echo ""

echo "4. Running LazyMVN in demo project..."
echo "   (Press 'q' to quit, or Ctrl+G to test custom goals)"
echo ""

# Run lazymvn
../../target/release/lazymvn
