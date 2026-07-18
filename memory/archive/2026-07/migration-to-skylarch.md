# Migration Plan: scutemob dingersuite → skylarch

**Created**: 2026-04-19 by oversight session (dingersuite-side)
**Status**: Phase 1 (push) COMPLETE — `ff69e249` on origin/main, skylarch-verified
**Purpose**: Move active scutemob development off the Debian VM to the Arch workstation.

---

## Facts

| Thing | Dingersuite (source) | Skylarch (destination) |
|---|---|---|
| User | `airbaggie` | **`skydude`** |
| Home | `/home/airbaggie` | `/home/skydude` |
| Project path | `/home/airbaggie/scutemob` | `/home/skydude/projects/scutemob` |
| Claude project-memory dir | `~/.claude/projects/-home-airbaggie-scutemob/` | `~/.claude/projects/-home-skydude-projects-scutemob/` |
| Rust | 1.93.1 stable | 1.94.0 stable (newer — OK) |
| Node | v24.11.0 | present |
| npm | 11.6 | 11.12.1 |
| pnpm | 10.30.3 | **NOT INSTALLED** |
| GitHub auth | (N/A — pushes done) | `gh` CLI as `cyngerian`, repo scope |
| SSH to github | yes | no |
| Free disk | plenty | **58G free of 396G (85% used)** |

## Global sed rule

Everywhere a path rewrite is mentioned:

```
s|/home/airbaggie/scutemob|/home/skydude/projects/scutemob|g
```

## Invariants

1. **Dingersuite stays untouched** until skylarch verification passes. It's the rollback.
2. **All rewrites happen on skylarch** (after clone), never on the source. Keeps dingersuite replayable.
3. **`~/.claude/projects/<hash>/`** is keyed off the project's absolute path. The rsync renames simultaneously.
4. Archival memory (`memory/abilities/`, `memory/cleanup/`, `memory/archive/`) keeps old paths — those are historical docs, not live config.

---

## Phase 0: Prechecks on skylarch (5 min)

```bash
# Identity
whoami                    # must be skydude
echo $HOME                # /home/skydude

# Target path creation
mkdir -p /home/skydude/projects

# Toolchain
rustc --version           # ≥ 1.93.1 (1.94.0 confirmed)
node --version            # ≥ v24
npm --version             # ≥ 11

# pnpm decision — pick ONE:
#   Option A: install pnpm (recommended — matches lockfile)
npm install -g pnpm
#   Option B: use npm with existing package-lock.json (fallback if pnpm install fails)

# Disk headroom
df -h /home/skydude       # need ≥ 30G free after cold build
# If tight, plan `cargo clean` after Phase 6, and set CARGO_INCREMENTAL=0 for first build

# gh auth (already confirmed, re-verify)
gh auth status
gh api repos/cyngerian/scutemob --jq .default_branch  # prints "main"
```

## Phase 1: Push — DONE ✅

Commits pushed 2026-04-19. `origin/main` at `ff69e249`. Confirmed skylarch-side via `gh` that HEAD matches.

## Phase 2: Clone (2 min)

```bash
cd /home/skydude/projects
gh repo clone cyngerian/scutemob   # clones to ./scutemob
cd scutemob
git log --oneline -5               # top line must be ff69e249

# Optional: wire gh auth into git for future push/fetch
gh auth setup-git
```

## Phase 3: Gitignored state transfer (10 min)

Rsync from dingersuite. Note: `.mcp.json` is **written fresh**, not rsync'd (absolute paths inside).

```bash
# On skylarch
cd /home/skydude/projects/scutemob

# 3a. Small files (Claude settings + dev notes)
rsync -av airbaggie@dingersuite:/home/airbaggie/scutemob/.claude/CLAUDE.local.md      .claude/
rsync -av airbaggie@dingersuite:/home/airbaggie/scutemob/.claude/settings.local.json  .claude/

# 3b. Large rebuildable data (rsync is cheaper than regen)
rsync -av --progress airbaggie@dingersuite:/home/airbaggie/scutemob/.scryfall-cache/  .scryfall-cache/
rsync -av --progress airbaggie@dingersuite:/home/airbaggie/scutemob/cards.sqlite      .

# 3c. Claude Code project memory dir — rsync + rename via destination path
rsync -av --progress airbaggie@dingersuite:/home/airbaggie/.claude/projects/-home-airbaggie-scutemob/ \
      /home/skydude/.claude/projects/-home-skydude-projects-scutemob/

# 3d. Sed the path references inside the rsynced memory dir
sed -i 's|/home/airbaggie/scutemob|/home/skydude/projects/scutemob|g' \
    /home/skydude/.claude/projects/-home-skydude-projects-scutemob/MEMORY.md

# 3e. Write fresh .mcp.json (do NOT rsync — absolute paths)
cat > /home/skydude/projects/scutemob/.mcp.json <<'EOF'
{
  "mcpServers": {
    "mtg-rules": {
      "command": "/home/skydude/projects/scutemob/tools/mcp-server/run.sh",
      "args": [
        "--db", "/home/skydude/projects/scutemob/cards.sqlite",
        "--rules", "/home/skydude/projects/scutemob/.scryfall-cache/MagicCompRules.txt"
      ]
    },
    "rust-analyzer": {
      "command": "/home/skydude/projects/scutemob/tools/rust-analyzer-mcp/run.sh",
      "args": ["/home/skydude/projects/scutemob"]
    }
  }
}
EOF
```

**Optional**: if `.claude/CLAUDE.local.md` or `.claude/settings.local.json` contains `/home/airbaggie/scutemob`, sed them too:
```bash
sed -i 's|/home/airbaggie/scutemob|/home/skydude/projects/scutemob|g' \
    .claude/CLAUDE.local.md .claude/settings.local.json 2>/dev/null || true
```

## Phase 4: In-repo hardcoded-path rewrite (5 min, committed)

**Scoped to live config only**. Archival memory files keep old paths.

```bash
cd /home/skydude/projects/scutemob

# Rewrite live agent prompts, skill defs, live policy docs
find .claude/agents .claude/skills -type f -name "*.md" \
    -exec sed -i 's|/home/airbaggie/scutemob|/home/skydude/projects/scutemob|g' {} +
sed -i 's|/home/airbaggie/scutemob|/home/skydude/projects/scutemob|g' \
    docs/cleanup-retention-policy.md

# Verify no remaining `/home/airbaggie` in live config
grep -rn "/home/airbaggie" .claude/agents .claude/skills docs/cleanup-retention-policy.md \
    && echo "UNEXPECTED — investigate" \
    || echo "clean"

# Sanity: memory/abilities archival refs should still be there (expected, not broken)
grep -c "/home/airbaggie" memory/abilities/*.md | head -3

# Commit
git add -u
git commit -m "chore: rewrite hardcoded paths for skylarch migration

Scoped to live agent prompts (.claude/agents/), skill definitions
(.claude/skills/), and docs/cleanup-retention-policy.md. Archival
planning docs in memory/abilities/, memory/cleanup/, and
memory/archive/ keep original paths as historical context.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"

git push origin main
```

## Phase 5: Rebuild native artifacts (30-60 min, disk-aware)

**Disk strategy** (58G free, 85% used): skip `--release` on first pass. Release bins grow `target/` by 2-3x.

```bash
cd /home/skydude/projects/scutemob

# 5a. Main workspace — debug only
~/.cargo/bin/cargo build --workspace       # expect ~15-25G in target/ cold

# 5b. Tools — these need release for performance (MCP server, rust-analyzer)
cd tools/mcp-server         && cargo build --release && cd ../..
cd tools/rust-analyzer-mcp  && cargo build --release && cd ../..

# 5c. Replay viewer
cd tools/replay-viewer
# If pnpm installed:
pnpm install && pnpm run build
# If using npm fallback:
# npm install && npm run build
cargo build
cd ../..

# 5d. Tauri app (first time on desktop — bonus, not blocking)
# ls the tauri dir, try cargo build if present. Skip on error.

# 5e. Disk check after builds
df -h /home/skydude
du -sh target/ tools/*/target/ 2>/dev/null
```

**If disk gets tight during Phase 5**: `cargo clean -p <crate>` on non-essential crates, or set `CARGO_TARGET_DIR=/some/other/volume` and rebuild.

## Phase 6: Verify (10 min — MANDATORY gate)

```bash
cd /home/skydude/projects/scutemob

# 6a. Test suite baseline
~/.cargo/bin/cargo test --all 2>&1 | tail -5
# Expect: "test result: ok. 2648 passed"  (exact match to dingersuite)

# 6b. Format
~/.cargo/bin/cargo fmt --check

# 6c. Clippy baseline — expect BASELINE-CLIPPY-01..06 pre-existing errors
#     per memory/workstream-state.md handoff. Do NOT claim "clippy clean".
~/.cargo/bin/cargo clippy --all-targets -- -D warnings 2>&1 | tail -30
# Compare error count to dingersuite baseline — same or fewer is acceptable.

# 6d. MCP binaries respond
./tools/mcp-server/run.sh --help        2>&1 | head -5
./tools/rust-analyzer-mcp/run.sh --help 2>&1 | head -5 || echo "interactive — OK if starts"

# 6e. Claude Code session end-to-end
# Open Claude Code in /home/skydude/projects/scutemob
# Run: /start-session
# Expected:
#   - git log shows ff69e249 (and the Phase 4 migration commit)
#   - workstream-state: W6 ACTIVE, PB-D plan-complete
#   - MCP tools (mtg-rules, rust-analyzer) connect
#   - auto-memory loads (MEMORY.md should have NEW paths)
```

**If any step fails**: diagnose before cut-over. Do not proceed to Phase 7 with a broken Phase 6.

## Phase 7: Cut-over + dingersuite decommission

**Only after Phase 6 passes cleanly.**

```bash
# On dingersuite: kill dev processes
ssh airbaggie@dingersuite
tmux kill-session -t scutemob    # may be >1 session — kill each
pkill -f mtg-mcp-server          # pid 3840 at survey time
ps -u airbaggie | grep -E "scutemob|mcp" && echo "leftovers"
```

**7-day cold rollback window**: leave `/home/airbaggie/scutemob/` on dingersuite in place. Reclaim the 93G target/:
```bash
# On dingersuite (optional, frees 93G immediately)
cd /home/airbaggie/scutemob && rm -rf target/ tools/*/target/
# Repo stays at ~400M as a read-only rollback
```

**After 7 days of healthy skylarch use**:
```bash
# On dingersuite
rm -rf /home/airbaggie/scutemob
rm -rf /home/airbaggie/.claude/projects/-home-airbaggie-scutemob  # if not needed
```

## Rollback

Any Phase 5 or 6 failure → resume work on dingersuite. Nothing on dingersuite has been touched by this migration (by design). To reset skylarch and retry:

```bash
# On skylarch
rm -rf /home/skydude/projects/scutemob
rm -rf /home/skydude/.claude/projects/-home-skydude-projects-scutemob
# Restart from Phase 2
```

If the Phase 4 sed commit landed on origin/main and you want to unwind it: `git revert <migration-commit-hash>` and push. Don't force-push.

## Risks / gotchas

- **Rust 1.94 on skylarch vs 1.93.1 on dingersuite**: forward-compat expected, but watch for `-D warnings` lints that appeared in 1.94. If clippy count grows by new-lint count, that's expected, not a regression.
- **pnpm vs npm**: `tools/replay-viewer` likely has `pnpm-lock.yaml`. If you fall back to npm, regenerate the lockfile locally — don't commit the npm lockfile unless you're also removing pnpm-lock.yaml intentionally.
- **Disk pressure**: monitor during Phase 5. A runaway `cargo check` with many crates open can spike `target/` to 40G+ on a bad day. If `df` drops below 20G, `cargo clean` and restart with `CARGO_INCREMENTAL=0`.
- **MCP auto-start silent failure**: Claude Code launches MCP servers at session start. If `tools/mcp-server/` or `tools/rust-analyzer-mcp/` binaries aren't built, MCP fails silently and tool calls return "not available". **Always complete Phase 5b before first Claude session**.
- **MEMORY.md staleness**: after the rsync+sed in Phase 3d, verify with `grep /home/airbaggie /home/skydude/.claude/projects/-home-skydude-projects-scutemob/MEMORY.md` — should be empty. If not, repeat sed.
- **jsonl transcripts**: the 240M of session history references old paths in content. They remain searchable via `/sessions` but any file-path link inside a transcript points to dingersuite. Cosmetic, not load-bearing.
- **Tauri app first build**: finally possible on desktop Arch. If it fails for missing `webkit2gtk-4.1` or similar, `pacman -S webkit2gtk gtk3` (exact list in the Tauri v2 prereqs).

## Post-migration cleanup (optional, opportunistic)

- Add `CLAUDE.md` update: Environment section currently says "Debian 12 (bookworm) VM on Unraid home server". Update to Arch workstation stats when convenient. **Not blocking** — memory state is canonical, CLAUDE.md is documentation.
- Archive `memory/migration-to-skylarch.md` to `memory/archive/2026-04/` after 7-day confidence window per `docs/cleanup-retention-policy.md`.
