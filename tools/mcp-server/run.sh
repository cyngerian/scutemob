#!/usr/bin/env bash
# Wrapper script for the MCP server.
# Rebuilds the binary if any source file is newer than the compiled output,
# then execs the server with all arguments passed through.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BINARY="$REPO_ROOT/target/debug/mtg-mcp-server"
SOURCE_DIR="$SCRIPT_DIR/src"

source "$HOME/.cargo/env" 2>/dev/null || true

needs_rebuild() {
    [[ ! -f "$BINARY" ]] && return 0
    # Rebuild if any source file is newer than the binary
    while IFS= read -r -d '' f; do
        [[ "$f" -nt "$BINARY" ]] && return 0
    done < <(find "$SOURCE_DIR" -name '*.rs' -print0)
    [[ "$SCRIPT_DIR/Cargo.toml" -nt "$BINARY" ]] && return 0
    return 1
}

if needs_rebuild; then
    cargo build -p mtg-mcp-server --manifest-path "$REPO_ROOT/Cargo.toml" >&2
fi

exec "$BINARY" "$@"
