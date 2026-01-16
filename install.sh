#!/bin/sh
set -e

REPO="marogosteen/asterion-cc"
BINARY="asterion"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

main() {
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        darwin) os="apple-darwin" ;;
        linux)  os="unknown-linux-gnu" ;;
        *)
            echo "Error: Unsupported OS: $os" >&2
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)  arch="aarch64" ;;
        *)
            echo "Error: Unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac

    target="${arch}-${os}"

    if [ -z "$VERSION" ]; then
        VERSION=$(curl -sI "https://github.com/${REPO}/releases/latest" | grep -i '^location:' | sed 's/.*tag\///' | tr -d '\r\n')
    fi

    if [ -z "$VERSION" ]; then
        echo "Error: Failed to get latest version" >&2
        exit 1
    fi

    url="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY}-${target}.tar.gz"

    echo "Installing ${BINARY} ${VERSION} (${target})..."

    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    curl -sL "$url" | tar xz -C "$tmpdir"

    mkdir -p "$INSTALL_DIR"
    mv "$tmpdir/$BINARY" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY"

    echo "Installed to: $INSTALL_DIR/$BINARY"

    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo ""
        echo "Add to PATH:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

main
