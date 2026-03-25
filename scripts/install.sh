#!/bin/sh
# Install shrug — static CLI for Atlassian Cloud
# Usage: curl -fsSL https://github.com/mfassaie/shrug/releases/latest/download/install.sh | sh

set -e

REPO="mfassaie/shrug"
INSTALL_DIR="${SHRUG_INSTALL_DIR:-/usr/local/bin}"

detect_target() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64|amd64) echo "x86_64-unknown-linux-musl" ;;
                *) echo "Error: unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64|amd64) echo "x86_64-apple-darwin" ;;
                arm64|aarch64) echo "aarch64-apple-darwin" ;;
                *) echo "Error: unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        *)
            echo "Error: unsupported OS: $os (use install.ps1 for Windows)" >&2
            exit 1
            ;;
    esac
}

get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" | rev | cut -d'/' -f1 | rev
    elif command -v wget >/dev/null 2>&1; then
        wget -qO /dev/null --server-response "https://github.com/$REPO/releases/latest" 2>&1 | grep -i location | tail -1 | rev | cut -d'/' -f1 | rev
    else
        echo "Error: curl or wget required" >&2
        exit 1
    fi
}

download() {
    url="$1"
    output="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$output" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$output" "$url"
    fi
}

main() {
    target="$(detect_target)"
    version="$(get_latest_version)"

    if [ -z "$version" ]; then
        echo "Error: could not determine latest version" >&2
        exit 1
    fi

    echo "Installing shrug $version ($target)..."

    url="https://github.com/$REPO/releases/download/$version/shrug-$target.tar.gz"
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    download "$url" "$tmpdir/shrug.tar.gz"
    tar xzf "$tmpdir/shrug.tar.gz" -C "$tmpdir"

    if [ -w "$INSTALL_DIR" ]; then
        mv "$tmpdir/shrug" "$INSTALL_DIR/shrug"
    else
        echo "Installing to $INSTALL_DIR (requires sudo)..."
        sudo mv "$tmpdir/shrug" "$INSTALL_DIR/shrug"
    fi

    chmod +x "$INSTALL_DIR/shrug"

    echo "Installed shrug $version to $INSTALL_DIR/shrug"

    if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
        echo ""
        echo "Note: $INSTALL_DIR is not in your PATH."
        echo "Add it with: export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

main
