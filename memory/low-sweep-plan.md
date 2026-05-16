# LOW Sweep Campaign Plan

> Consolidated plan to clear the **entire** open-LOW backlog (42 issues) across both
> tracking docs: `docs/mtg-engine-milestone-reviews.md` (29 indexed) +
> `docs/mtg-engine-low-issues-remediation.md` SR/PB/BASELINE sections (13).
>
> **Scope decision (2026-05-15)**: user chose "also unblock everything" — blocked LOWs
> get their prerequisite engine work done, not just deferred. Execution: /dispatch one
> worker per session. Workstream: W3 (LOW Remediation).
>
> **Execution mode (revised 2026-05-15)**: STRICTLY SEQUENTIAL — one worker at a time.
> Parallel worktrees each cost ~30G `target/` and filled the disk. Order: LS-2 → LS-3 →
> LS-4 → LS-5 → LS-6 → LS-7 → LS-8. Delete each worktree `target/` immediately after
> collect. Re-launch LS-3/LS-4 workers into their existing (paused) worktrees.
>
> Created: 2026-05-15

---

## Session breakdown (8 fix sessions + residual)

Each session is a /dispatch task. Sessions in the same wave touch disjoint subsystems
and can run in parallel; later waves depend on earlier merges only where noted.

### Wave 1 — independent, low/medium risk (parallel)

**LS-1 — Commander & deck validation (6)** — `commander.rs`, `events.rs`
- MR-M9-09 — `reveals_hidden_info()`: add `MulliganTaken`, `LibraryShuffled`, `CompanionBroughtToHand` to the `true` arm
- MR-M9-10 — `validate_deck` `HashMap` → `im::OrdMap` (deterministic violation order)
- MR-M9-11 — deck-validation dedup violations by card name (or document expected)
- MR-M9-12 — mulligan `let _ = move_object_to_zone(...)` → propagate/count errors
- MR-M9-13 — `BringCompanion` returns error when companion not in command zone (no phantom event)
- MR-M9-16 — test: partner pair, one commander at 21+ dmg one not — only the 21 triggers loss

**LS-2 — Replay viewer & harness (7)** — `tools/replay-viewer/`, Svelte/JS, `replay_harness.rs`
- MR-M9.5-05 — add `FirstStrikeDamage` step to `PhaseIndicator.svelte` PHASES array
- MR-M9.5-09 — replace synthetic `"initial_state"` string with a dedicated convention/variant
- MR-M9.5-10 — hand diff: `JSON.stringify` compare contents, not `.length`
- MR-M9.5-12 — battlefield iteration by `zones[Battlefield].object_ids()` not `OrdMap` key order
- MR-M9.5-13 — `PlayerId(i as u64 + 1)` add bounds check / `try_into`
- MR-CKP-01 — `TurnBasedAction.action` dead field: wire it up or document the empty-string contract
- MR-B12-05 — `replay_harness.rs` `exclude_self: true` hardcoded → make a DSL field

**LS-3 — Combat (6)** — `combat.rs`, `turn_actions.rs`
- MR-M6-13 — test: blocker removed before damage (CR 509.1h, "was blocked, blockers gone, no trample")
- SR-TRM-01 — **real fix**: planeswalker combat damage removes loyalty counters, not damage marks (CR 120.3c)
- SR-TRM-02 — remove residual dead `is_blocked()` scan branches in `combat.rs`
- SR-FS-02 — test: creature gains first strike between the two combat damage steps (CR 702.7c)
- SR-FS-03 — test: first-strike attacker vs first-strike blocker (both in FS step only)
- MR-M2-16 — `cleanup_actions`: use `empty_all_mana_pools` return value, drop unconditional `ManaPoolsEmptied`

**LS-4 — Protection (5)** — `state/types.rs`, `protection.rs`, `tests/protection.rs`
- SR-PRO-01 — add `ProtectionQuality::{FromSuperType, FromName}` variants + enforcement + hash
- SR-PRO-02 — add `ProtectionQuality::FromPlayer` variant (CR 702.16k) + enforcement + hash
- SR-PRO-03 — test: protection vs multicolor source (shares any color)
- SR-PRO-04 — test: subtype-based protection (e.g. "protection from Goblins")
- MR-M9.4-10 — `has_protection_from_source` linear scan: micro-opt or formally re-confirm as non-bottleneck

### Wave 2 — higher risk / interdependent (serialize after Wave 1 merges)

**LS-5 — Replacement effects & triggers (6)** — `replacement.rs`, `resolution.rs`, `sba.rs`
- MR-M8-12 — self-ETB replacements route through `find_applicable`/`determine_action` (CR 614.15 ordering)
- MR-M8-16 — GC stale `WhileSourceOnBattlefield` replacement effects when source leaves
- MR-B12-03 — test: enrage prevention-reduces-to-zero path (prevention effect, no card needed)
- MR-B12-04 — **infra**: `PendingTrigger` stores embedded effect, not source `ObjectId` (fixes CR 400.7 lookup-None)
- MR-B16-07 — `ObjectId(0)` room-ability sentinel → `ObjectId::SENTINEL` constant + doc
- PB-Q4-L01 — `matches_enchant_target` defensive `.unwrap_or(aura_ctrl)` → explicit error/`debug_assert!`

**LS-6 — PB-T loyalty & DSL unblocks (3, PB-scale)** — route via `/implement-primitive` discipline
- PB-T-L01 — thread `targets: Vec<TargetRequirement>` through `handle_activate_loyalty_ability`, call `validate_targets_with_source` before stacking
- PB-T-L02 — add `Effect::MoveZone` (or extend) able to target cards in an opponent's graveyard; implement Sorin Lord of Innistrad −6 reanimate-rider
- PB-T-L03 — add `EffectDuration::UntilControllersNextUntapStep` + `Effect::PreventUntap`; implement Tamiyo Field Researcher −2 freeze-rider

**LS-7 — LKI-completeness audit (1, audit-scale)** — dedicated audit per BASELINE-LKI-01
- BASELINE-LKI-01 — filtered death/LTB triggers must match characteristics granted while on battlefield (CR 603.10a / 613.1e). Enumerate every filter with an `obj_zone == Battlefield` guard and every dispatch site reading LKI via `calculate_characteristics`; pick fix candidate (a) dispatch reads preserved chars directly, or (b) teach `calculate_characteristics` to honor preserved chars off-battlefield. Add regression tests.

**LS-8 — Spree card authoring (2)** — `/author-wave` discipline
- MR-B11-08 — author a Spree card that has a base mana cost; test mode_costs summed with base cost
- MR-B11-09 — test casting a Spree spell with zero additional modes (base-cost-only path)

### Residual — M10-gated (4) — NOT in this campaign

These genuinely require the M10 interactive-choice / targeting system, which is a
milestone, not PB-scale work. Recommend they ride M10. Documented here so the sweep
is honest about what it does not close.
- MR-M8-11 — CR 615.7 damage-prevention shield order needs interactive player choice
- MR-B16-04 — placeholder room effects need interactive targeting
- MR-B16-05 — deterministic room-effect fallbacks need interactive targeting/free-cast
- MR-B16-06 — Acererak zombie tokens need `EffectTarget::CurrentIterationPlayer` (ForEach context — could be PB-scale; re-evaluate if LS-6 capacity allows)

### Permanently deferred — re-confirmed, no action (2)
- MR-M1-18 — `Zone::Ordered` O(n) `contains`/`remove` — not a bottleneck (remediation doc: "never")
- MR-M6-14 — `blockers_for()` rebuild — ≤10 blockers, negligible

---

## Progress tracker

> **CAMPAIGN COMPLETE 2026-05-16** — all 8 sessions (LS-1..8) merged to main.
> 36 of 42 LOWs actioned; LOW-OPEN 45→6. 2860 tests passing, build/clippy/fmt clean,
> HASH 24→27. Final verification (`cargo test --all` on main) green.
> Residual 6 = 4 M10-gated (MR-M8-11, MR-B16-04/05/06) + 2 permanent perf (MR-M1-18, MR-M6-14).

| Session | Items | Status | Task ID | Branch | Notes |
|---------|-------|--------|---------|--------|-------|
| LS-1 | 6 | **DONE** | scutemob-31 | merged f492f815 | All 6 closed. Opus review 7/7. LOW-OPEN 29→23, LOW-CLOSED 119→125. |
| LS-2 | 7 | **DONE** | scutemob-32 | merged 93f6d3b5 | All 7 closed. /review 8/8. Merged main→branch first (doc-stats conflict resolved by coordinator: LOW-OPEN 29→16, LOW-CLOSED 119→132). |
| LS-3 | 6 | **DONE** | scutemob-33 | merged 6010b5c9 | All 6 closed (MR-M6-13, SR-TRM-01/02, SR-FS-02/03, MR-M2-16). /review 7/7. |
| LS-4 | 5 | **DONE** | scutemob-34 | merged b760292a | All 5 closed (SR-PRO-01..04, MR-M9.4-10). /review 6/6. Wave 1 complete — 24 LOWs closed. |
| LS-5 | 6 | **DONE** | scutemob-35 | merged e3fbd3da | All 6 closed (MR-M8-12/16, MR-B12-03/04, MR-B16-07, PB-Q4-L01). /review 7/7. |
| LS-6 | 3 | **DONE** | scutemob-36 | merged db49ddee | PB-T-L01/L02/L03 unblocked. New Effect::DestroyAndReanimate (disc 85) + Effect::PreventNextUntap (disc 86); HASH 25→26. /review 5/5, 2856 tests. |
| LS-7 | 1 | **DONE** | scutemob-37 | merged d81e107c | BASELINE-LKI-01 fixed (variant b): pre_death_characteristics snapshot on GameEvent::CreatureDied. Audit at memory/primitives/lki-completeness-*. 2858 tests. |
| LS-8 | 2 | **DONE** | scutemob-38 | merged 9b4ed0d2 | MR-B11-08/09 closed. Authored Insatiable Avarice ({B} base Spree card) + 2 tests. /review 5/5. |
| Residual | 4 | deferred → M10 | — | — | documented above |
| Perm-deferred | 2 | re-confirmed | — | — | no action |

**Total**: 42 LOWs — 36 actioned across LS-1..8, 4 deferred to M10, 2 permanently deferred.

Each completed session: worker updates `docs/mtg-engine-milestone-reviews.md` (flip
status to CLOSED with commit ref) and `docs/mtg-engine-low-issues-remediation.md` for
SR/PB/BASELINE IDs, then coordinator updates this tracker.
