#!/bin/bash
# Quick test guide for Yank Logs feature

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║     LazyMVN - Yank Logs Feature Test Guide            ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}📋 Test Plan${NC}"
echo ""
echo "This guide will help you test the 'Y' (Yank Logs) feature."
echo ""

# Step 1: Build
echo -e "${YELLOW}Step 1: Building LazyMVN...${NC}"
cargo build --quiet
echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Step 2: Check log directory
echo -e "${YELLOW}Step 2: Checking log directory...${NC}"
LOG_DIR=~/.local/share/lazymvn/logs
if [ -d "$LOG_DIR" ]; then
    echo -e "${GREEN}✓ Log directory exists: $LOG_DIR${NC}"
    ls -lh "$LOG_DIR"
else
    echo "ℹ  Log directory will be created on first run"
fi
echo ""

# Step 3: Manual test instructions
echo -e "${YELLOW}Step 3: Manual Testing${NC}"
echo ""
echo "Now you should manually test the feature:"
echo ""
echo "1. Start LazyMVN with debug mode:"
echo -e "   ${BLUE}cargo run -- --project demo/multi-module --debug${NC}"
echo ""
echo "2. Wait for the application to load"
echo ""
echo "3. Press 'Y' (Shift+y) to yank logs"
echo ""
echo "4. Check the output pane for a message like:"
echo "   ✓ Copied X lines of debug logs to clipboard"
echo ""
echo "5. Open a text editor and paste (Ctrl+V or Cmd+V)"
echo ""
echo "6. Verify the pasted content has this format:"
echo "   === LazyMVN Session Logs ==="
echo "   Session ID: YYYYMMDD-HHMMSS-mmm"
echo "   Timestamp: ..."
echo "   === Debug Logs ==="
echo "   [SESSION:...] ..."
echo ""

# Step 4: Automated verification
echo -e "${YELLOW}Step 4: Automated Verification${NC}"
echo ""
echo "After running the app, you can verify the logs:"
echo ""
echo "Check session ID in logs:"
echo -e "  ${BLUE}grep 'Session ID:' ~/.local/share/lazymvn/logs/debug.log | tail -1${NC}"
echo ""
echo "Count log entries for last session:"
echo -e "  ${BLUE}SESSID=\$(grep 'Session ID:' ~/.local/share/lazymvn/logs/debug.log | tail -1 | sed 's/.*Session ID: //')${NC}"
echo -e "  ${BLUE}grep \"\[SESSION:\$SESSID\]\" ~/.local/share/lazymvn/logs/debug.log | wc -l${NC}"
echo ""

# Step 5: Test clipboard tools
echo -e "${YELLOW}Step 5: Checking clipboard tools...${NC}"
echo ""
if command -v wl-copy &> /dev/null; then
    echo -e "${GREEN}✓ wl-copy is available (Wayland)${NC}"
elif command -v xclip &> /dev/null; then
    echo -e "${GREEN}✓ xclip is available (X11)${NC}"
elif command -v xsel &> /dev/null; then
    echo -e "${GREEN}✓ xsel is available (X11)${NC}"
else
    echo -e "${YELLOW}⚠  No system clipboard tool found${NC}"
    echo "   Will use arboard (Rust fallback)"
    echo ""
    echo "   To install clipboard tools:"
    echo "   - Wayland: sudo apt install wl-clipboard"
    echo "   - X11: sudo apt install xclip"
fi
echo ""

# Step 6: Expected behavior
echo -e "${YELLOW}Step 6: Expected Behavior${NC}"
echo ""
echo "✓ Each session has a unique ID"
echo "✓ Logs are tagged with [SESSION:ID]"
echo "✓ Pressing 'Y' copies only current session logs"
echo "✓ Output includes both debug and error logs"
echo "✓ Confirmation message appears in output pane"
echo "✓ Logs are formatted with headers and sections"
echo ""

# Final notes
echo "╔════════════════════════════════════════════════════════╗"
echo "║     Ready to Test!                                     ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo "Run the command above and press 'Y' in the app to test."
echo ""
echo "For more information, see:"
echo "  - YANK_LOGS.md (detailed documentation)"
echo "  - YANK_LOGS_SUMMARY.md (implementation summary)"
echo ""
