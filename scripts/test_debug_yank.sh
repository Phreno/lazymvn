#!/bin/bash

# Test script for debug yank feature

echo "=== Debug Yank Feature Test ==="
echo ""
echo "This script will:"
echo "1. Build lazymvn in release mode"
echo "2. Start it with the demo multi-module project"
echo "3. Instructions will be shown for manual testing"
echo ""

# Build
echo "Building lazymvn..."
cargo build --release 2>&1 | tail -3
echo ""

# Check if build succeeded
if [ ! -f target/release/lazymvn ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""
echo "=== Manual Test Instructions ==="
echo ""
echo "1. Launch with: ./target/release/lazymvn --debug --project demo/multi-module"
echo "2. Run a few commands (b, t, c, etc.) to generate output"
echo "3. Press Shift+Y to yank the debug report"
echo "4. You should see: '✓ Copied debug report (XXX lines) to clipboard'"
echo "5. Paste the clipboard content to verify it contains:"
echo "   - LazyMVN version information"
echo "   - System information (OS, architecture)"
echo "   - Configuration (lazymvn.toml content)"
echo "   - Output from all tabs"
echo "   - Debug and error logs (last 500 lines each)"
echo ""
echo "Expected structure:"
echo "  ========================================"
echo "  LazyMVN Debug Report"
echo "  ========================================"
echo "  Generated: [timestamp]"
echo ""
echo "  === Version Information ==="
echo "  LazyMVN Version: [version]"
echo "  ..."
echo ""
echo "Test comparison:"
echo "  - Press 'y' (lowercase): copies only current tab output"
echo "  - Press 'Y' (uppercase): copies complete debug report"
echo ""
echo "Ready to test? Press Enter to launch lazymvn, or Ctrl+C to cancel..."
read

# Launch
./target/release/lazymvn --debug --project demo/multi-module

