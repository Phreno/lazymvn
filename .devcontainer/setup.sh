#!/bin/bash
set -e

echo "🚀 Setting up LazyMVN development environment..."

# Install additional packages
sudo apt-get update
sudo apt-get install -y \
    git-flow \
    tree \
    htop \
    curl \
    wget \
    build-essential \
    pkg-config \
    libssl-dev

# Setup Rust environment
echo "🦀 Configuring Rust environment..."
source $HOME/.cargo/env

# Install useful Rust tools
cargo install --locked \
    cargo-watch \
    cargo-edit \
    cargo-audit \
    cargo-outdated \
    cargo-tree \
    cargo-expand

# Add Rust components
rustup component add clippy rustfmt llvm-tools-preview

# Setup Java environment
echo "☕ Configuring Java environment..."
export JAVA_HOME=/usr/lib/jvm/msopenjdk-current
export PATH="$JAVA_HOME/bin:$PATH"

# Verify installations
echo "🔍 Verifying installations..."
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Java version: $(java -version 2>&1 | head -n 1)"
echo "Maven version: $(mvn --version | head -n 1)"

# Pre-build dependencies for faster startup
echo "📦 Pre-building project dependencies..."
if [ -f "Cargo.toml" ]; then
    cargo fetch
    cargo check
fi

# Setup git flow if not already initialized
if [ -d ".git" ] && [ ! -f ".git/refs/heads/develop" ]; then
    echo "🌊 Initializing Git Flow..."
    git flow init -d || true
fi

# Create useful aliases
echo "🔧 Setting up aliases..."
cat >> ~/.bashrc << 'EOF'

# LazyMVN Development Aliases
alias ll='ls -alF'
alias la='ls -A'
alias l='ls -CF'
alias cb='cargo build'
alias ct='cargo test'
alias cc='cargo check'
alias cf='cargo fmt'
alias ccl='cargo clippy'
alias cw='cargo watch -x check -x test'
alias mvn-test='./mvnw test'
alias mvn-package='./mvnw package'
alias mvn-clean='./mvnw clean'

# Git aliases
alias gst='git status'
alias glog='git log --oneline --graph --decorate'
alias gco='git checkout'
alias gcb='git checkout -b'

# Git Flow aliases
alias gff='git flow feature'
alias gfh='git flow hotfix'
alias gfr='git flow release'

EOF

echo "✅ Development environment setup complete!"
echo ""
echo "Available commands:"
echo "  Rust: cargo build, cargo test, cargo watch"
echo "  Java: mvn test, mvn package, ./mvnw [goals]"
echo "  Tools: git-flow, tree, htop"
echo ""
echo "Happy coding! 🎉"