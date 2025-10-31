#!/usr/bin/env bash
# Test script to demonstrate the new output-priority adaptive layout logic
#
# The layout now switches based on output width rather than terminal width:
# - Two-column mode: Used when output would have >= 150 chars
# - Single-column mode: Used when output would be < 150 chars
#
# With left column taking min(30%, 40 chars), this means:
# - Terminals >= 190 cols → two-column (left=40, output=150)
# - Terminals < 190 cols → single-column (output gets full width)

set -euo pipefail

echo "🎯 Output-Priority Layout Logic Test"
echo "======================================"
echo ""
echo "New logic: Switch to single-column when output would be < 150 chars"
echo ""

# Calculate thresholds
MIN_OUTPUT=150
MAX_LEFT=40
THRESHOLD=$((MIN_OUTPUT + MAX_LEFT))

echo "📐 Key thresholds:"
echo "  • Minimum output width: ${MIN_OUTPUT} chars (typical log line)"
echo "  • Maximum left column width: ${MAX_LEFT} chars"
echo "  • Two-column threshold: >=${THRESHOLD} total width"
echo ""

echo "📊 Examples:"
echo ""

# Test cases
test_width() {
    local width=$1
    local left=$(( (width * 30) / 100 ))
    if [ $left -gt $MAX_LEFT ]; then
        left=$MAX_LEFT
    fi
    local output=$((width - left))
    
    if [ $output -ge $MIN_OUTPUT ]; then
        local mode="TWO-COLUMN"
        local emoji="✅"
    else
        local mode="SINGLE-COLUMN"
        local emoji="🔄"
    fi
    
    printf "%s Width=%3d → Left=%2d, Output=%3d → %s\n" "$emoji" "$width" "$left" "$output" "$mode"
}

test_width 60
test_width 80
test_width 100
test_width 120
test_width 150
test_width 180
test_width 189
test_width 190
test_width 200
test_width 250

echo ""
echo "🎓 Key insights:"
echo "  • 100 cols terminal → single-column (output needs 150, would only get 70)"
echo "  • 190 cols terminal → two-column (output gets exactly 150)"
echo "  • Larger terminals → two-column (output gets more space)"
echo ""
echo "✨ This ensures logs are always readable while maximizing output space!"
