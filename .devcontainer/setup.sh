#!/bin/bash
set -e

echo "🚀 Setting up LazyMVN development environment..."

# Create and fix cargo permissions
echo "🔧 Setting up Cargo directories and permissions..."
mkdir -p /home/vscode/.cargo /home/vscode/.rustup
sudo chown -R vscode:vscode /home/vscode/.cargo /home/vscode/.rustup /usr/local/cargo /usr/local/rustup || true

# Set up environment variables for the session
echo "🌐 Setting up environment variables..."
export CARGO_HOME=/home/vscode/.cargo
export RUSTUP_HOME=/home/vscode/.rustup

# Add to shell profile for persistence
echo 'export CARGO_HOME=/home/vscode/.cargo' >> ~/.bashrc
echo 'export RUSTUP_HOME=/home/vscode/.rustup' >> ~/.bashrc

# Set up Rust toolchain
echo "🦀 Setting up Rust toolchain..."
rustup default stable
rustup component add clippy rustfmt

# Pre-fetch dependencies
echo "📦 Pre-fetching project dependencies..."
cargo fetch

echo "✅ LazyMVN development environment ready!"