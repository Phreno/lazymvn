#!/bin/bash

set -e

echo "===  Building lazymvn-test-harness ==="
cd /workspaces/lazymvn
cargo build --package lazymvn-test-harness --quiet

echo ""
echo "=== Running integration tests ==="
cargo test --package lazymvn-test-harness --test integration_tests test_maven_output_captured -- --exact --nocapture

echo ""
echo "=== Test completed ==="
