#!/usr/bin/env bash
DIR="$(cd "$(dirname "$0")" && pwd)"
exec "$DIR/target/release/rust-analyzer-mcp" "$@"
