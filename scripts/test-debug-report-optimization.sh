#!/bin/bash
# Test script for debug report optimization

set -e

echo "=== Debug Report Optimization Test ==="
echo ""

# Build the project
echo "1. Building LazyMVN..."
cargo build --release --quiet
echo "✓ Build successful"
echo ""

# Determine log directory
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "darwin"* ]]; then
    LOG_DIR="$HOME/.local/share/lazymvn/logs"
else
    LOG_DIR="$LOCALAPPDATA/lazymvn/logs"
fi

echo "2. Testing debug report optimization..."
echo ""

# Create a test config with comments
TEST_DIR=$(mktemp -d)
TEST_CONFIG="$TEST_DIR/lazymvn.toml"

cat > "$TEST_CONFIG" << 'EOF'
# This is a comment that should be filtered out
# Another comment line

[maven]
# Comment about custom_goals
custom_goals = [
    # Inline comment
    { name = "Format", goal = "formatter:format" },
]

# More comments
# That should be removed

[maven.flags]
skip_tests = true

# Final comment
EOF

echo "Created test config with comments:"
wc -l "$TEST_CONFIG"
echo ""

echo "3. Verify log level filtering..."
echo ""

# Check current session ID
if [ -f "$LOG_DIR/debug.log" ]; then
    LAST_SESSION=$(grep -oE "Session ID: [0-9-]+" "$LOG_DIR/debug.log" | tail -1 | cut -d' ' -f3)
    echo "Last session ID: $LAST_SESSION"
    
    # Count log levels in last session
    if [ -n "$LAST_SESSION" ]; then
        echo ""
        echo "Log level distribution in last session:"
        grep "SESSION:$LAST_SESSION" "$LOG_DIR/debug.log" 2>/dev/null | \
            grep -oE "\[(TRACE|DEBUG|INFO|WARN|ERROR)\]" | \
            sort | uniq -c || echo "  (No logs for this session yet)"
    fi
fi

echo ""
echo "4. Expected improvements in debug reports (Shift+Y):"
echo ""
echo "   ✅ Configuration:"
echo "      - Comments removed (saves 50-70%)"
echo "      - Empty lines removed"
echo "      - Only active configuration shown"
echo ""
echo "   ✅ Logs:"
echo "      - Current session only (excludes old sessions)"
echo "      - DEBUG/INFO/WARN/ERROR only (no TRACE)"
echo "      - Limited to last 300 lines"
echo "      - Works across rotated files (.log.1, .log.2, etc.)"
echo ""
echo "   ✅ Tab Output:"
echo "      - Last 100 lines per tab"
echo "      - Shows truncation indicator when applicable"
echo ""
echo "   ✅ Expected Size Reduction:"
echo "      - Before: 5000+ lines"
echo "      - After: 500-1000 lines"
echo "      - Reduction: ~80-90%"
echo ""

echo "5. Manual testing steps:"
echo ""
echo "   a. Run LazyMVN with debug logging:"
echo "      ./target/release/lazymvn --log-level debug"
echo ""
echo "   b. Perform various operations:"
echo "      - Build a project"
echo "      - Switch tabs"
echo "      - Open popups (Ctrl+G, ?, Ctrl+H)"
echo "      - Move mouse around (should be TRACE now)"
echo ""
echo "   c. Generate debug report:"
echo "      - Press Shift+Y"
echo ""
echo "   d. Check clipboard content:"
echo "      - Should see '(Comments and empty lines removed for brevity)'"
echo "      - Should see '(Last 300 lines from current session)'"
echo "      - Should NOT see TRACE level logs"
echo "      - Should NOT see old session logs"
echo ""

echo "6. Verify TRACE vs DEBUG usage:"
echo ""
echo "   Files using TRACE (high-frequency events):"
echo "   - src/tui/mouse.rs (mouse clicks)"
echo ""
echo "   Files using DEBUG (important state changes):"
echo "   - src/core/config/types.rs (config loading)"
echo "   - src/ui/state/profiles.rs (profile operations)"
echo "   - src/features/starters.rs (starter scanning)"
echo ""

echo "7. Check log files:"
if [ -d "$LOG_DIR" ]; then
    echo ""
    echo "Current logs:"
    ls -lh "$LOG_DIR" 2>/dev/null || echo "  (No logs yet)"
else
    echo "  Log directory not created yet (will be on first run)"
fi

echo ""
echo "=== Documentation ==="
echo "See docs/internal/LOGGING_BEST_PRACTICES.md for guidelines"
echo "See docs/user/LOG_ROTATION.md for log management details"

# Cleanup
rm -rf "$TEST_DIR"
