#!/bin/bash
# Test script for development environment setup
set -e

echo "🧪 Testing LazyMVN development environment..."

# Test Rust installation
echo "🦀 Testing Rust..."
if command -v rustc &> /dev/null; then
    echo "✅ Rust: $(rustc --version)"
    echo "✅ Cargo: $(cargo --version)"
else
    echo "❌ Rust not installed"
    exit 1
fi

# Test Java installation  
echo "☕ Testing Java..."
if command -v java &> /dev/null; then
    echo "✅ Java: $(java -version 2>&1 | head -n 1)"
else
    echo "❌ Java not installed"
    exit 1
fi

# Test Maven installation
echo "📦 Testing Maven..."
if command -v mvn &> /dev/null; then
    echo "✅ Maven: $(mvn --version | head -n 1)"
else
    echo "❌ Maven not installed"
    exit 1
fi

# Test Git Flow
echo "🌊 Testing Git Flow..."
if command -v git-flow &> /dev/null; then
    echo "✅ Git Flow available"
else
    echo "⚠️  Git Flow not installed (optional)"
fi

# Test Rust tools
echo "🔧 Testing Rust tools..."
RUST_TOOLS=("cargo-watch" "cargo-edit" "cargo-audit")
for tool in "${RUST_TOOLS[@]}"; do
    if command -v "$tool" &> /dev/null; then
        echo "✅ $tool installed"
    else
        echo "⚠️  $tool not installed (optional)"
    fi
done

# Test project build
echo "🏗️  Testing project build..."
if [ -f "Cargo.toml" ]; then
    echo "Running cargo check..."
    if cargo check --quiet; then
        echo "✅ Project builds successfully"
    else
        echo "❌ Project build failed"
        exit 1
    fi
else
    echo "⚠️  No Cargo.toml found, skipping build test"
fi

# Test Maven demo projects
echo "🎯 Testing Maven demo projects..."
if [ -d "demo/multi-module" ]; then
    cd demo/multi-module
    if ./mvnw -q clean compile; then
        echo "✅ Multi-module demo builds"
    else
        echo "❌ Multi-module demo failed"
        exit 1
    fi
    cd ../..
else
    echo "⚠️  Demo projects not found"
fi

echo ""
echo "🎉 All tests passed! Development environment is ready."
echo ""
echo "Quick start commands:"
echo "  cargo build          # Build LazyMVN"
echo "  cargo test           # Run tests" 
echo "  cargo run            # Run LazyMVN"
echo "  cargo run -- --help  # Show help"
echo ""
echo "Demo projects:"
echo "  cd demo/multi-module && cargo run -- --project ."
echo "  cd demo/single-module && cargo run -- --project ."