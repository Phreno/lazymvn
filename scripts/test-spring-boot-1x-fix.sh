#!/bin/bash
# Test script for Spring Boot 1.x JVM arguments fix
# Validates that LazyMVN correctly detects Spring Boot version and uses appropriate property syntax

set -e

echo "=== Testing Spring Boot 1.x JVM Arguments Fix ==="
echo ""

# Build the project
echo "Step 1: Building LazyMVN..."
cargo build --release > /dev/null 2>&1
echo "✓ Build successful"
echo ""

# Test version detection
echo "Step 2: Testing Spring Boot version detection..."
cargo test --lib maven::detection::tests::test_build_launch_command_spring_boot_1x_uses_run_properties -- --nocapture
if [ $? -eq 0 ]; then
    echo "✓ Spring Boot 1.x test passed"
else
    echo "✗ Spring Boot 1.x test failed"
    exit 1
fi
echo ""

# Test Spring Boot 2.x compatibility
echo "Step 3: Testing Spring Boot 2.x compatibility..."
cargo test --lib maven::detection::tests::test_build_launch_command_spring_boot_with_profiles -- --nocapture
if [ $? -eq 0 ]; then
    echo "✓ Spring Boot 2.x test passed"
else
    echo "✗ Spring Boot 2.x test failed"
    exit 1
fi
echo ""

# Run all detection tests
echo "Step 4: Running all detection tests..."
cargo test --lib maven::detection -- --nocapture --quiet
if [ $? -eq 0 ]; then
    echo "✓ All detection tests passed"
else
    echo "✗ Some detection tests failed"
    exit 1
fi
echo ""

echo "=== Test Summary ==="
echo "✓ Spring Boot 1.x version detection working"
echo "✓ Spring Boot 1.x uses -Drun.* properties"
echo "✓ Spring Boot 2.x uses -Dspring-boot.run.* properties"
echo "✓ All unit tests passing"
echo ""
echo "Next: Test with real Spring Boot 1.2.2 application"
echo ""
echo "Manual test command:"
echo "  cargo run --release -- --project /path/to/spring-boot-1.2.2-app --debug"
echo ""
echo "Expected in debug log:"
echo "  - 'Found Spring Boot plugin version: 1.2.2.RELEASE'"
echo "  - Maven command contains '-Drun.jvmArguments' (not -Dspring-boot.run.jvmArguments)"
echo "  - Application logs show custom format and filtering"
