#!/usr/bin/env bash
# scutemob session-start check. Two callers:
#   tools/start-check.sh          manual: prints the data-freshness table, exit 0/1/2
#   tools/start-check.sh --hook   SessionStart hook (.claude/settings.json): same table
#                                 wrapped in guidance for the agent, ALWAYS exit 0 so a
#                                 STALE result is context, not a hook error.
# Skipped silently in dispatched-worker checkouts (.esm/worker.md) and linked worktrees:
# the Scryfall cache is gitignored and absent there, so the check would only say STALE.
cd "$(dirname "$0")/.." || exit 2
hook=0; [ "${1:-}" = "--hook" ] && { hook=1; shift; }
if [ -f .esm/worker.md ] || [ "$(git rev-parse --git-dir 2>/dev/null)" != "$(git rev-parse --git-common-dir 2>/dev/null)" ]; then
    exit 0
fi
if [ "$hook" = 0 ]; then
    exec python3 tools/data-freshness.py check "$@"
fi
out=$(python3 tools/data-freshness.py check "$@" 2>&1); rc=$?
echo "[scutemob data-freshness — SessionStart hook, read-only]"
echo "$out"
case $rc in
    0) echo "External data is CURRENT. Nothing to do." ;;
    1) echo "External data is STALE. In the /start report, say so and OFFER: python3 tools/data-freshness.py refresh — do NOT run it unprompted (it downloads and rebuilds local data). After a CR refresh, run: python3 tools/data-freshness.py cites" ;;
    *) echo "Freshness UNKNOWN (network or meta problem). Mention it in the /start report; a manual 'python3 tools/data-freshness.py check' may explain." ;;
esac
exit 0
