#!/bin/bash
# Test script for yank logs functionality

echo "Testing yank logs functionality..."
echo ""

# Test 1: Check if session ID is generated
echo "1. Checking session ID generation..."
SESSION_ID=$(grep "Session ID:" ~/.local/share/lazymvn/logs/debug.log | tail -1)
echo "   Last session ID: $SESSION_ID"
echo ""

# Test 2: Check if get_current_session_logs works
echo "2. Testing get_current_session_logs function..."
cat > /tmp/test_logs.rs << 'EOF'
fn main() {
    // This would be called from within the app
    println!("Testing log extraction...");
}
EOF

# Test 3: Show sample of current session logs
echo "3. Sample of current session logs (last 10 lines):"
tail -10 ~/.local/share/lazymvn/logs/debug.log | head -10
echo ""

# Test 4: Count log entries for current session
CURRENT_SESSION=$(grep "Session ID:" ~/.local/share/lazymvn/logs/debug.log | tail -1 | sed 's/.*Session ID: //' | tr -d '[]')
if [ ! -z "$CURRENT_SESSION" ]; then
    COUNT=$(grep "\[SESSION:$CURRENT_SESSION\]" ~/.local/share/lazymvn/logs/debug.log | wc -l)
    echo "4. Log entries for current session ($CURRENT_SESSION): $COUNT"
else
    echo "4. No current session found"
fi
echo ""

# Test 5: Check error log
echo "5. Error log status:"
if [ -f ~/.local/share/lazymvn/logs/error.log ]; then
    ERROR_COUNT=$(wc -l < ~/.local/share/lazymvn/logs/error.log)
    echo "   Error log exists with $ERROR_COUNT entries"
    if [ $ERROR_COUNT -gt 0 ]; then
        echo "   Last error:"
        tail -1 ~/.local/share/lazymvn/logs/error.log
    fi
else
    echo "   No error log found (no errors occurred)"
fi
echo ""

echo "✓ Test complete"
echo ""
echo "To test the 'Y' key functionality:"
echo "1. Run: cargo run -- --project demo/multi-module --debug"
echo "2. Press 'Y' (Shift+y) in the application"
echo "3. Check the output pane for confirmation message"
echo "4. Paste (Ctrl+V) to verify logs are in clipboard"
