#!/usr/bin/env bash
set -euo pipefail

REPOSITORY_URL="https://github.com/SecurityTalent/GitHunter.git"
INSTALL_DIR="${GITHUNTER_INSTALL_DIR:-${XDG_BIN_HOME:-$HOME/.local/bin}}"
TEMPORARY_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$TEMPORARY_DIR"
}
trap cleanup EXIT

require_command() {
    local command_name="$1"
    local install_hint="$2"
    if ! command -v "$command_name" >/dev/null 2>&1; then
        echo "Error: $command_name is required. $install_hint" >&2
        exit 1
    fi
}

add_path_to_profile() {
    local profile_file="$1"
    local path_line="export PATH=\"$INSTALL_DIR:\$PATH\""

    touch "$profile_file"
    if ! grep -Fqx "$path_line" "$profile_file"; then
        {
            printf '\n# GitHunter CLI\n'
            printf '%s\n' "$path_line"
        } >> "$profile_file"
    fi
}

require_command git "Install Git, then run this script again."
require_command cargo "Install Rust and Cargo from https://rustup.rs/, then run this script again."

echo "Installing GitHunter globally for the current user..."
echo "Cloning GitHunter..."
git clone --depth 1 "$REPOSITORY_URL" "$TEMPORARY_DIR/repository"

echo "Building release binary..."
cargo build --release --manifest-path "$TEMPORARY_DIR/repository/Cargo.toml"

mkdir -p "$INSTALL_DIR"
install -m 755 "$TEMPORARY_DIR/repository/target/release/githunter" "$INSTALL_DIR/githunter"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) export PATH="$INSTALL_DIR:$PATH" ;;
esac

# Login shells read .profile; typical terminal Bash/Zsh sessions read the
# shell-specific file. Update both paths that apply to the user's default shell.
add_path_to_profile "$HOME/.profile"
case "${SHELL:-}" in
    */bash) add_path_to_profile "$HOME/.bashrc" ;;
    */zsh) add_path_to_profile "$HOME/.zshrc" ;;
esac

echo
echo "GitHunter installed successfully at $INSTALL_DIR/githunter"
echo "This terminal is ready now: githunter --help"
echo "Open a new terminal to use githunter from any directory."
