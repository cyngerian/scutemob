#!/usr/bin/env bash
# Project-local /start hook. The start skill runs this when it exists and reports its
# output; a non-zero exit means "offer the refresh". Keep it fast and read-only.
cd "$(dirname "$0")/.." || exit 2
exec python3 tools/data-freshness.py check "$@"
