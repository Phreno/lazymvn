#!/bin/bash
# Test script to simulate user interaction with lazymvn

echo "Testing lazymvn interactively..."
cd /workspaces/lazymvn

# Send 'b' key (build command) then wait, then send 'q' to quit
(sleep 2; echo "b"; sleep 5; echo "q") | timeout 10 cargo run -- -p demo/multi-module 2>&1 | tee /tmp/lazymvn-test.log

echo ""
echo "=== Output captured in /tmp/lazymvn-test.log ==="
cat /tmp/lazymvn-test.log
