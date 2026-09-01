#!/usr/bin/env bash
set -e

echo "🎯 Installing GitHunter globally..."

# Check dependencies
if ! command -v git >/dev/null 2>&1; then
    echo "❌ Error: git is required to install GitHunter."
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Error: Rust & Cargo are required. Install Rust via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

TMP_DIR=$(mktemp -d)
echo "📥 Cloning GitHunter repository..."
git clone --depth 1 https://github.com/SecurityTalent/GitHunter.git "$TMP_DIR"
cd "$TMP_DIR"

echo "⚙️ Building release binary..."
cargo build --release

INSTALL_DIR="/usr/local/bin"
echo "📦 Installing binary to $INSTALL_DIR/githunter..."

if [ -w "$INSTALL_DIR" ]; then
    cp target/release/githunter "$INSTALL_DIR/githunter"
    chmod +x "$INSTALL_DIR/githunter"
else
    echo "🔑 Sudo privileges required to copy to $INSTALL_DIR:"
    sudo cp target/release/githunter "$INSTALL_DIR/githunter"
    sudo chmod +x "$INSTALL_DIR/githunter"
fi

rm -rf "$TMP_DIR"

echo ""
echo "✨ GitHunter has been installed successfully!"
echo "🚀 Try running: githunter --help"
