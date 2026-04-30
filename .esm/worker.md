# Worker Agent — scutemob-16

You are a worker agent operating in a git worktree. Your job is to implement
the task described below, satisfy all acceptance criteria, and signal completion.

## Task
**ID**: scutemob-16
**Title**: PB-TS: TokenSpec.count u32 → EffectAmount (dynamic token-count primitive)
**Branch**: feat/pb-ts-tokenspeccount-u32-effectamount-dynamic-token-count-pr
**ESM Project**: scutemob
**ESM Server**: http://tower:8765

## Acceptance Criteria
1. (ID: 3724) PB-TS engine primitive landed: token count accepts EffectAmount, integration with resolve_effect_amount (or equivalent), 5 mandatory tests a-e analogous to PB-CC-C-followup pattern
2. (ID: 3725) ≥2 in-scope cards re-authored from MCP oracle text (yield-calibrated target; PB-Q4 shipped 36%, PB-P 23%); each authored card has a clean (no TODO) primary-mechanic implementation
3. (ID: 3726) HASH bumped 13→14 and sentinel-assertion test files updated to match
4. (ID: 3727) cargo test --workspace green (count must increase from 2720 baseline); cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean
5. (ID: 3728) /review verdict PASS or PASS-WITH-NITS — 0 HIGH and 0 MEDIUM open before signal-ready (apply fix-phase if needed)
6. (ID: 3729) Plan memo at memory/primitives/pb-plan-TS.md and review memo at memory/primitives/pb-review-TS.md committed to feature branch
7. (ID: 3730) Out-of-scope blockers encountered (Anim Pakal non-Gnome filter, threshold gates, etc.) appended as new STOP-AND-FLAG seeds to memory/primitives/pb-retriage-CC.md

## Getting Started
1. Run `/home/skydude/.local/bin/esm task get scutemob-16` to check the current state
2. Run `/home/skydude/.local/bin/esm session start --project scutemob --agent worker` to register your work session
3. Begin implementation

## Rules
1. Only commit to this branch. Do not touch main.
2. Commit frequently with descriptive messages prefixed by the task ID.
3. Do not run /task, /done, /start, /end, /spawn, or /collect — those are coordinator skills.
4. Write tests for your implementation with coverage for happy path and edge cases.
5. Mark criteria satisfied as you complete them (see CLI cheat sheet below).
6. Post brief status updates via task comments for significant progress.
7. Run `/review` before signaling completion.

## Completion Sequence
When all criteria are satisfied, committed, and reviewed:
1. Run `/home/skydude/.local/bin/esm task signal-ready scutemob-16 --agent worker` — releases coordinator's lock,
   transitions to `in_review`, publishes `task_ready_for_collection` event.
2. Run `/home/skydude/.local/bin/esm session end <session_id> --summary "..."`.
3. Exit the session.

## ESM CLI Cheat Sheet
```bash
# Check task state
/home/skydude/.local/bin/esm task get scutemob-16

# Mark a criterion as satisfied (get criterion IDs from task get output)
/home/skydude/.local/bin/esm task satisfy scutemob-16 <criterion_id> --by worker --note "description of what was done"

# Post a status comment
/home/skydude/.local/bin/esm task comment scutemob-16 --agent worker "Completed: X. Next: Y."

# Signal ready for collection (do this LAST, after /review passes)
/home/skydude/.local/bin/esm task signal-ready scutemob-16 --agent worker

# Session management
/home/skydude/.local/bin/esm session start --project scutemob --agent worker
/home/skydude/.local/bin/esm session end <session_id> --summary "what was accomplished"
```
