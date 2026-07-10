---
pb: PB-AC8
title: Static restrictions, win-cons, no-max-hand
phase: review
plan_file: memory/primitives/pb-plan-AC8.md
review_file: memory/primitives/pb-review-AC8.md
---

# PB-AC8 — Static restrictions, win-cons, no-max-hand

Task `scutemob-51`, campaign-plan §2 PB-AC8 row (S effort — smallest of the
AC chain). Discounted yield ~14 cards.

## Scope

**5 primitives:**

1. `GameRestriction::NoMaximumHandSize` — modifies the cleanup-step discard
   check (CR 402.2 / 514.1). Find how cleanup discard is enforced today; do not
   add a parallel path.
2. `GameRestriction::MustAttack` — an attack **requirement** (CR 508.1).
   **Goad is already implemented.** Study the existing attack-requirement
   machinery in `combat.rs` and extend it. Do not invent a parallel path.
3. `GameRestriction::CantAttackOwner` — an attack **restriction** on the same
   machinery as (2). Requirements and restrictions interact per CR 508.1d/e:
   a requirement is only obeyed to the extent it doesn't violate a restriction.
4. `GameRestriction::CantBeSacrificed` — must bite at **BOTH** sites:
   - cost-payment legality (can't pay a sacrifice cost with the permanent), and
   - effect-driven sacrifice (`Effect::SacrificePermanents` and any other
     sacrifice dispatch site).
   Walk the full dispatch chain per `feedback_verify_full_chain` — do not stop
   at the first site that looks right.
5. `Effect::WinGame { condition }` — **multiplayer-correct**. In a 4-player
   Commander game "you win the game" is processed per CR 104.2 / 104.3h / 800:
   with the limited-range-of-influence option absent, a player winning causes
   each remaining opponent to lose. Verify the exact wording via MCP. Tests
   **must** cover the 4-player case, not just 1v1 (architecture invariant #5:
   multiplayer-first).

## CR refs — ADVISORY ONLY, verify each via mtg-rules MCP

104.2 (winning), 104.3h, 119 (life), 402.2 (max hand size), 508.1 (declare
attackers; requirements & restrictions), 514.1 (cleanup discard), 704 (SBAs),
800 (multiplayer).

**Do NOT grep the CR file** — it has bare `\r` line endings, so rule-number
greps silently match nothing. Use the mtg-rules MCP for all CR verification.
Per `feedback_verify_cr_before_implement`: the CR refs in this file and in the
task description are **advisory**, not authoritative. If MCP contradicts them,
MCP wins and you flag the discrepancy rather than implementing the brief.

## Hazards (from task brief)

1. **New mutable/runtime state MUST be added to `state/hash.rs` `HashInto`
   impls with mutation-verified tests.** This was a review HIGH in both PB-AC1
   and PB-AC5. Any `won`/`lost` flag, any per-turn restriction cache.
2. **Verify the KW / AbilDef / SOK discriminant chain from current source**
   before adding variants. Do not trust MEMORY.md's recorded end values.
3. **Exhaustive matches** live in `tools/tui/src/play/panels/stack_view.rs`
   (`StackObjectKind`) and `tools/replay-viewer/src/view_model.rs`
   (`StackObjectKind` **and** `KeywordAbility`). Run `cargo build --workspace`
   after every impl phase — and note `cargo build` does **not** compile test
   targets, so the real gate is `cargo test --all`.
4. **Win/lose processing must respect SBA batch discipline** — all applicable
   SBAs are checked simultaneously as a batch, then triggers go on the stack
   together in APNAP order. A win that removes players mid-batch is a bug.
5. **Do not commit phantom `.claude/skills/*` deletions** in fresh worktrees.
   (Already restored via `git checkout -- .claude/skills/` at session start.)

## Roster discovery

The planner identifies the real card roster from oracle text — the coordinator's
`~14` is a discounted estimate, not a roster (`feedback_pb_yield_calibration`:
planners overcount in-scope cards 2-3×; `feedback_oversight_primitive_category_not_cards`:
oversight names the category, the worker verifies scope from oracle text).

Grep card defs for **both** `// TODO` and `// ENGINE-BLOCKED` markers citing:
no-maximum-hand-size, must-attack / attacks each combat if able, can't-attack-you
/ can't-attack-its-owner, can't-be-sacrificed, and win-the-game patterns.

## Close includes backfill

PB-AC8 is **not done** until every card it unblocks is re-authored and its stale
TODO / ENGINE-BLOCKED markers are deleted, then reviewed by `card-batch-reviewer`.

## SCOPE CORRECTION (post-plan, 2026-07-09)

Recon + `primitive-impl-planner` independently established that **2 of the 5
briefed primitives already exist**. Following the PB-AC7 precedent
(LoseAbilities "verified already-expressible, NOT re-added"):

| Briefed primitive | Verdict |
|---|---|
| `GameRestriction::NoMaximumHandSize` | **DROP** — already `KeywordAbility::NoMaxHandSize` + `PlayerState::no_max_hand_size`, enforced `turn_actions.rs:1487-1526` |
| `GameRestriction::MustAttack` (self) | **DROP** — already `KeywordAbility::MustAttackEachCombat`, enforced `combat.rs:334-368` |
| `GameRestriction::MustAttack` (group) | **DEFER** — 0 yield (sole card PARTIAL). Seed OOS-AC8-3 |
| `GameRestriction::CantAttackOwner` | **BUILD** |
| `GameRestriction::CantBeSacrificed` | **BUILD** (full 11-site dispatch chain) |
| `Effect::WinGame` | **BUILD** |

Plus one **live bug found in scope**: the cleanup discard check reads
`obj.characteristics.keywords` directly instead of `calculate_characteristics()`,
so layer-granted `NoMaxHandSize` is invisible. `wrenn_and_seven.rs:92` grants
exactly that and is broken today. Fix + regression test.

### CR correction (MCP-verified, brief was wrong)

The brief said "with the limited-range option absent, winning makes each opponent
lose." **Inverted.** CR 104.3h / 801.14 are *gated on* the limited-range-of-influence
option (CR 801.1: Emperor / often 5+ players). **Commander does not use it.** The
correct citation is **CR 104.1** — a game ends *immediately* when a player wins
(CR 104.2b). Also **CR 104.3f**: a player who would simultaneously win and lose,
loses → WinGame must no-op if the controller is already `has_lost`.

### Design decision: no `has_won` field

Considered and **rejected**. A `has_won` flag would not reduce `active_players()`
(12 call sites across `engine.rs`, `priority.rs`, `casting.rs`, and the simulator's
`driver.rs`/`invariants.rs`/`legal_actions.rs`), so a decided game would keep
granting priority until the next game-over poll. Instead `Effect::WinGame` marks
opponents `has_lost` atomically inside its own resolution and lets the **existing**
`is_game_over` / `check_game_over` pass finalize. Consequence: **no new struct
field**, so hazard 1's new-field hash risk does not arise — only new enum arms.
Per **CR 704.5, winning-by-effect is NOT an SBA**: do *not* splice into `sba.rs`.

## Steps

- [x] Plan (primitive-impl-planner) → `memory/primitives/pb-plan-AC8.md`
- [x] Verify discriminant chain + hash.rs baseline
- [x] ~~NoMaximumHandSize~~ DROPPED (already expressible)
- [x] ~~MustAttack (self)~~ DROPPED (already expressible); group form DEFERRED
- [x] Fix layer-correctness bug in cleanup discard (`calculate_characteristics`) — `turn_actions.rs` `has_no_max` scan; regression test added
- [x] Implement: CantAttackOwner (CR 508.1c; key on **owner**, not controller) — `combat.rs` declare-attackers check + must-attack owner-exclusion (CR 508.1d)
- [x] Implement: CantBeSacrificed (full 11/12-site chain: effect + cost sites) — choke-point `object_cant_be_sacrificed()` in `effects/mod.rs`; wired into all sites (see runner report for per-site table; Exploit is N/A — engine unconditionally declines, no dispatch to guard)
- [x] Implement: Effect::WinGame (CR 104.1/104.2b/104.3f; 4p test) — dispatch in `effects/mod.rs`; added `is_game_over`/`check_game_over` poll to `engine.rs::handle_all_passed` post-resolution (needed for GameOver to fire same-command; not previously present for any resolving effect)
- [x] hash.rs: new enum arms + HASH_SCHEMA_VERSION 34→35 + mutation-verified tests — 26 other test files' hardcoded `34u8` sentinels bulk-updated to `35u8`
- [x] `cargo build --workspace` + `cargo test --all` — 3055 passed (was 3035 baseline + 20 new)
- [ ] Review (primitive-impl-reviewer) → `memory/primitives/pb-review-AC8.md`
- [ ] Fix all HIGH/MEDIUM findings
- [ ] Backfill: re-author unblocked cards, delete stale markers
- [ ] Review backfill (card-batch-reviewer)
- [ ] Final gates: build / test --all / clippy -D warnings / fmt --check
- [ ] authoring-report rerun + coverage delta posted as task comment
- [ ] `/review`, satisfy criteria 4403-4406, close
