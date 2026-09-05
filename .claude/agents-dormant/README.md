# Dormant agents (moved aside 2026-09-05, CC-14 / `scutemob-251`)

These eight agents have not been invoked since July 2026: no milestone is running and every
keyword ability has shipped. An agent's description is injected into EVERY session's context,
so they were moved here rather than deleted (`docs/course-correction-2026-09.md` §6.1).

| Agent | Pipeline it belongs to |
|---|---|
| `rules-implementation-planner`, `session-runner`, `milestone-reviewer`, `fix-session-runner` | milestone pipeline (`/start-milestone`, the Milestone Completion Checklist) |
| `ability-impl-planner`, `ability-impl-runner`, `ability-impl-reviewer` | `/implement-ability` |
| `ability-coverage-auditor` | `/audit-abilities` |

**To restore one**: `git mv .claude/agents-dormant/<name>.md .claude/agents/<name>.md`, then
**restart the Claude Code session** — the agent registry is read at startup, so a restored
agent is not a valid `subagent_type` until then. Add its row back to the Agents table in
`CLAUDE.md` and, if `/dispatch` workers should be able to use it, to the roster in the
`/dispatch` worker prompt. Restore the matching skill from `.claude/skills-dormant/` the same way.

Before restoring, check the agent's stale references (CC-3 corrected the active agents only):
`docs/primitive-card-plan.md` / `docs/dsl-gap-closure-plan.md` are HISTORICAL,
`tools/replay-viewer/src/view_model.rs` moved to `crates/view-model/src/lib.rs`, the tool is
`Agent` not `Task`, and the clippy bar is `cargo clippy --workspace --all-targets -- -D warnings`.
