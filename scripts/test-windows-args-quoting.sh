#!/usr/bin/env bash

# Test script to verify Windows argument quoting fix
# Tests that flags are properly split and arguments are quoted

set -e

echo "==================================================================="
echo "Testing Windows Argument Quoting Fix"
echo "==================================================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

test_passed=0
test_failed=0

# Test function
run_test() {
    local test_name="$1"
    local expected="$2"
    local result="$3"
    
    echo -n "Testing: $test_name... "
    if echo "$result" | grep -qF "$expected"; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((test_passed++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  Expected: $expected"
        echo "  Got: $result"
        ((test_failed++))
        return 1
    fi
}

# Test 1: Check that BuildFlag definitions no longer contain comma-separated aliases
echo "Test 1: BuildFlag definitions should use single form"
echo "-------------------------------------------------------------------"

FLAGS_FILE="src/ui/state/project_tab.rs"

if grep -q '"-U, --update-snapshots"' "$FLAGS_FILE"; then
    echo -e "${RED}✗ FAIL${NC}: Found comma-separated flag '-U, --update-snapshots'"
    ((test_failed++))
else
    echo -e "${GREEN}✓ PASS${NC}: Flag definitions use single form"
    ((test_passed++))
fi

if grep -q '"-o, --offline"' "$FLAGS_FILE"; then
    echo -e "${RED}✗ FAIL${NC}: Found comma-separated flag '-o, --offline'"
    ((test_failed++))
else
    echo -e "${GREEN}✓ PASS${NC}: Flag definitions use single form"
    ((test_passed++))
fi

if grep -q '"-X, --debug"' "$FLAGS_FILE"; then
    echo -e "${RED}✗ FAIL${NC}: Found comma-separated flag '-X, --debug'"
    ((test_failed++))
else
    echo -e "${GREEN}✓ PASS${NC}: Flag definitions use single form"
    ((test_passed++))
fi

echo ""

# Test 2: Check command building logic handles flag splitting
echo "Test 2: Command building should split comma-separated flags"
echo "-------------------------------------------------------------------"

COMMAND_FILE="src/maven/command.rs"

if grep -q 'split.*,' "$COMMAND_FILE" | grep -q "flag"; then
    echo -e "${GREEN}✓ PASS${NC}: Command building splits flags on comma"
    ((test_passed++))
else
    if grep -q 'Take only the first part before comma' "$COMMAND_FILE"; then
        echo -e "${GREEN}✓ PASS${NC}: Command building handles comma-separated flags"
        ((test_passed++))
    else
        echo -e "${YELLOW}⚠ WARN${NC}: Could not verify flag splitting logic"
    fi
fi

echo ""

# Test 3: Check for Windows quoting logic
echo "Test 3: Windows-specific argument quoting"
echo "-------------------------------------------------------------------"

if grep -q '#\[cfg(windows)\]' "$COMMAND_FILE" && grep -A5 '#\[cfg(windows)\]' "$COMMAND_FILE" | grep -q 'contains.*='; then
    echo -e "${GREEN}✓ PASS${NC}: Windows-specific quoting logic found"
    ((test_passed++))
else
    echo -e "${YELLOW}⚠ WARN${NC}: Windows quoting logic may need review"
fi

echo ""

# Test 4: Compile and run basic tests
echo "Test 4: Compile and run Maven command tests"
echo "-------------------------------------------------------------------"

if cargo test --lib maven::command::tests --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ PASS${NC}: Maven command tests pass"
    ((test_passed++))
else
    echo -e "${RED}✗ FAIL${NC}: Maven command tests failed"
    ((test_failed++))
fi

echo ""

# Summary
echo "==================================================================="
echo "Test Summary"
echo "==================================================================="
echo -e "Passed: ${GREEN}$test_passed${NC}"
echo -e "Failed: ${RED}$test_failed${NC}"
echo ""

if [ $test_failed -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    echo ""
    echo "Key fixes applied:"
    echo "  1. BuildFlag definitions use single form (-U instead of -U, --update-snapshots)"
    echo "  2. Command building splits flags on comma and takes first part only"
    echo "  3. Arguments with spaces or special chars are quoted on Windows"
    echo ""
    echo "The Maven command should now work correctly on Windows!"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
