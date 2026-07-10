---
pb: PB-AC8
title: Static restrictions, win-cons, no-max-hand
phase: fix-complete
plan_file: memory/primitives/pb-plan-AC8.md
review_file: memory/primitives/pb-review-AC8.md
---

# PB-AC8 — Static restrictions, win-cons, no-max-hand

## Fix phase (2026-07-09, HEAD `12981e69`)

Both MEDIUMs fixed; both LOWs (T1, C1) fixed; E3/E4 recorded as OOS (not fixed,
per review's own "none required now" directive).

- **E1 (MEDIUM) fixed** — `object_cant_be_sacrificed` guard added to all 4
  previously-unguarded delayed self-sacrifice sites, mirroring the existing
  Evoke guard:
  - Blitz — `rules/resolution.rs` `KeywordTrigger { keyword: Blitz, .. }` arm
    (was ~3308-3333; condition on `obj.zone == Battlefield` now also requires
    `!object_cant_be_sacrificed`).
  - Encore — `rules/resolution.rs` `KeywordTrigger { keyword: Encore, .. }` arm
    (was ~6814-6825; same pattern on `token_info`'s `.filter(...)`).
  - Mobilize (`sacrifice_at_end_step`) — guarded at the shared dispatch choke
    point `DelayedTriggerAction::SacrificeObject` in `rules/resolution.rs`
    (~7535), NOT at the `turn_actions.rs` queueing site (~1087) which only
    tags/queues the once-only delayed trigger; the actual sacrifice execution
    for Mobilize/Kiki-Jiki/The Fire Crystal all funnel through this one arm.
  - Decayed end-of-combat — `rules/turn_actions.rs` (~2144-2157): restructured
    into a two-pass scan (tag-and-clear-flag pass, matching the CR 603.7b
    fires-once semantics, THEN a `!object_cant_be_sacrificed` filter for the
    actual sacrifice) so a protected creature's flag doesn't linger and
    re-trigger at a later end of combat.
  - Docstring at `effects/mod.rs` (`object_cant_be_sacrificed`, was ~7113-7116)
    rewritten from the false "every sacrifice dispatch site... must route
    through this helper" claim to an accurate enumeration of actual callers
    (including the newly-guarded delayed sites) plus the Exploit N/A note.
- **E2 (MEDIUM) fixed** — `rules/combat.rs` goad "must attack if able" block
  (~274-313) given the same `CantAttackOwner` + `no_legal_target`
  owner-exclusion the `MustAttackEachCombat` block already had (~394-420),
  mirroring that existing pattern exactly (CR 508.1d).
- **T1 (LOW) fixed** — added `test_cant_be_sacrificed_cast_cost_emerge_cannot_pay`
  (Emerge additional-cost site, `casting.rs`).
- **C1 (LOW) fixed** — corrected stale "not in DSL" comments on
  `alexios_deimos_of_kosmos.rs`, `thassas_oracle.rs`, and
  `call_the_spirit_dragons.rs` (the latter flagged as a sibling in the review's
  Card Def Summary but not yet corrected in the prior backfill commit).
  `hellkite_tyrant.rs` / `simic_ascendancy.rs` were already accurate (fixed in
  the `12981e69` backfill commit) — verified, not re-touched. Per the explicit
  instruction, only comments were corrected; the two static restrictions were
  NOT authored onto Alexios this session (that remains a deferred backfill
  action, now correctly described by the comment instead of falsely claimed
  as an engine gap). Cards remain PARTIAL.
- **5 new regression tests** added to `crates/engine/tests/pb_ac8_restrictions_and_wingame.rs`
  (25 total in that file, was 20): `test_cant_be_sacrificed_blitz_delayed_sacrifice_skipped`,
  `test_cant_be_sacrificed_mobilize_delayed_sacrifice_skipped`,
  `test_cant_be_sacrificed_decayed_end_of_combat_sacrifice_skipped`,
  `test_cant_attack_owner_goad_yields_requirement`,
  `test_cant_be_sacrificed_cast_cost_emerge_cannot_pay`. All 5 were
  mutation-verified (temporarily reverted each corresponding guard and
  confirmed the test fails; restored and reconfirmed green) -- not vacuous.
  Encore itself was NOT given a dedicated regression test (Blitz + Mobilize +
  Decayed cover the 3 distinct code shapes: stack-resolution `KeywordTrigger`
  arm, shared `DelayedTriggerAction::SacrificeObject` dispatch, and a direct
  turn_actions.rs TBA scan); the Encore fix is structurally identical to the
  Blitz fix (same file, same pattern) and is covered by inspection + the
  existing Blitz test proving the pattern works, but is not independently
  test-covered. Documented here as an honest gap, not silently skipped.
- **Gates**: `cargo build --workspace` clean; `cargo test --all` 3062 passed / 0
  failed (3057 baseline + 5 new); `cargo clippy --all-targets -- -D warnings`
  clean; `cargo fmt --check` clean.

## OOS (skip, per review directive -- do not fix)

- **OOS-AC8-E3**: WinGame immediate-win vs SBA ordering (`effects/mod.rs:3112`
  + `engine.rs:1577`). The game-over poll runs after `resolve_top_of_stack`'s
  SBA pass; a winner at <=0 life not yet SBA-marked could be dragged into a
  draw instead of winning immediately (CR 104.1). Marginal; no roster card.
- **OOS-AC8-E4**: Must-attack owner-exclusion ignores planeswalkers
  (`combat.rs:394-403`). `no_legal_target` only scans players; a must-attack +
  CantAttackOwner creature able to attack its owner's planeswalker is wrongly
  treated as having no legal target. Consistent with pre-existing must-attack
  behavior. Marginal; no roster card.

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
- [x] Review (primitive-impl-reviewer) → `memory/primitives/pb-review-AC8.md` — 0 HIGH, 2 MEDIUM, 4 LOW
- [x] Fix all HIGH/MEDIUM findings — E1 (CantBeSacrificed half-wired: Blitz/Encore/Mobilize/decayed unguarded) + E2 (goad missing CR 508.1d owner-exclusion) both fixed; T1/C1 LOWs also fixed; E3/E4 recorded as OOS
- [x] Backfill: Nezahal + Toski fully authored (markers deleted); Curiosity Crafter fully authored after review found its ENGINE-BLOCKED marker was STALE; Hellkite/Simic/Niv-Mizzet remain PARTIAL with accurate markers
- [x] Review backfill (card-batch-reviewer) → `memory/card-authoring/review-pb-ac8-backfill.md` — 1 HIGH (stale marker), 1 MEDIUM (Simic win-con unreachable, accepted), 1 LOW (oracle text drift) — all resolved or accepted
- [x] Final gates: build --workspace / test --all (3062) / clippy -D warnings / fmt --check — all clean
- [x] authoring-report rerun — clean 970 → 973 (+3), 55.5% → 55.7%; delta posted as task comment
- [ ] `/review`, satisfy criteria 4403-4406, close

## Outcome

**Yield honesty**: PB-AC8 is a **prerequisite/infrastructure batch, not a yield
batch**. The briefed ~14 cards resolved to **3 fully unblocked** — and all three
were *mis-triaged*, blocked only by stale markers naming primitives that already
existed. The 3 genuinely-new primitives (`CantAttackOwner`, `CantBeSacrificed`,
`Effect::WinGame`) fully unblock **zero** cards on their own; every candidate
carries a second, out-of-scope co-blocking gap.

**Bugs found that were in no one's brief:**
1. Cleanup discard read `obj.characteristics.keywords` directly, so layer-granted
   `NoMaxHandSize` was invisible — `wrenn_and_seven.rs` grants exactly that, and
   its emblem proxy was silently dead. Fixed (W3-LC pattern) + regression test.
2. `handle_all_passed` never polled game-over after stack resolution, so an
   effect that ends the game set the flags but emitted no `GameOver` in the same
   command. Fixed; incidentally makes draw-from-empty-library during resolution
   emit `GameOver` same-command too.
3. `CantBeSacrificed` shipped half-wired (review E1) — the exact
   `feedback_verify_full_chain` failure mode, caught by adversarial review.
4. Curiosity Crafter's `ENGINE-BLOCKED` marker was stale (card-review HIGH);
   the token combat-damage filter was already expressible.

**Recommendation for PB-AC9 scoping**: run a stale-marker triage sweep *before*
scoping the next primitive batch. If PB-AC8's ~14 collapsed to 3 mis-triage wins,
the campaign's remaining per-batch estimates are likely counting dead TODO
markers rather than real engine gaps — a sweep would likely recover coverage
faster than the next primitive batch will.
