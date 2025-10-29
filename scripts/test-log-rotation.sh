#!/bin/bash
# Test script to demonstrate log rotation system

set -e

echo "=== Log Rotation Test ==="
echo ""

# Determine log directory based on OS
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "darwin"* ]]; then
    LOG_DIR="$HOME/.local/share/lazymvn/logs"
else
    # Windows
    LOG_DIR="$LOCALAPPDATA/lazymvn/logs"
fi

echo "Log directory: $LOG_DIR"
echo ""

# Check if logs exist
if [ -d "$LOG_DIR" ]; then
    echo "Current log files:"
    ls -lh "$LOG_DIR" 2>/dev/null || echo "  (directory exists but is empty)"
    echo ""
    
    # Show file sizes
    if [ -f "$LOG_DIR/debug.log" ]; then
        DEBUG_SIZE=$(du -h "$LOG_DIR/debug.log" | cut -f1)
        echo "debug.log size: $DEBUG_SIZE"
    fi
    
    if [ -f "$LOG_DIR/error.log" ]; then
        ERROR_SIZE=$(du -h "$LOG_DIR/error.log" | cut -f1)
        echo "error.log size: $ERROR_SIZE"
    fi
    
    # Count rotated files
    ROTATED_COUNT=$(ls "$LOG_DIR"/*.log.[1-5] 2>/dev/null | wc -l)
    echo "Rotated backup files: $ROTATED_COUNT"
    
    if [ "$ROTATED_COUNT" -gt 0 ]; then
        echo ""
        echo "Rotated log files:"
        ls -lh "$LOG_DIR"/*.log.[1-5] 2>/dev/null || true
    fi
else
    echo "Log directory doesn't exist yet (will be created on first run)"
fi

echo ""
echo "=== Rotation Settings ==="
echo "  Maximum file size: 5 MB"
echo "  Rotated backups kept: 5 (per file)"
echo "  Cleanup retention: 30 days"
echo "  Maximum total size: ~60 MB"
echo ""

echo "=== Testing Log Rotation ==="
echo ""
echo "To test rotation:"
echo "  1. Run LazyMVN with debug logging:"
echo "     cargo run -- --log-level debug"
echo ""
echo "  2. Use LazyMVN normally (build, test, etc.)"
echo ""
echo "  3. When debug.log exceeds 5 MB, restart LazyMVN"
echo "     - debug.log will be rotated to debug.log.1"
echo "     - A new empty debug.log will be created"
echo ""
echo "  4. Check rotation happened:"
echo "     ls -lh $LOG_DIR"
echo ""

echo "=== Manual Rotation Test ==="
echo ""
read -p "Create a fake large log file to test rotation? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    mkdir -p "$LOG_DIR"
    
    # Create a 6 MB file
    echo "Creating fake 6 MB debug.log..."
    dd if=/dev/zero of="$LOG_DIR/debug.log" bs=1M count=6 2>/dev/null
    
    echo "File created:"
    ls -lh "$LOG_DIR/debug.log"
    echo ""
    
    echo "Now running LazyMVN - it should rotate this log on startup..."
    echo ""
    
    cargo build --release --quiet
    
    # Run LazyMVN briefly
    timeout 2 ./target/release/lazymvn --log-level debug 2>/dev/null || true
    
    echo ""
    echo "After rotation:"
    ls -lh "$LOG_DIR"
    echo ""
    
    if [ -f "$LOG_DIR/debug.log.1" ]; then
        echo "✅ SUCCESS - Log was rotated!"
        echo "   - debug.log.1 (6 MB) = old log"
        echo "   - debug.log (small) = new log"
    else
        echo "⚠️  Rotation may not have triggered (file might be under 5 MB after restart)"
    fi
fi

echo ""
echo "=== Documentation ==="
echo "See docs/user/LOG_ROTATION.md for complete details"
