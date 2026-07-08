# Primitive Batch Review: PB-AC2 — Optional-cost (beneficial-pay) wrapper & counter-tax

**Date**: 2026-07-07
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 118.12, 118.12a, 118.8, 119.4, 500.4, 613.1d, 701.5, 701.21a, 702.34a
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (Effect variants 1608, 1618)
- `crates/engine/src/effects/mod.rs` (exec arms 2999/3014; helpers 6927/6989/7157/7196/7249)
- `crates/engine/src/state/hash.rs` (arms 5959/5966; schema bump 213-217)
- `crates/engine/tests/optional_cost_and_counter_tax.rs` (15 tests)
**Card defs reviewed**: 0 changed this phase — card backfill is explicitly deferred to the
PB-AC2 backfill phase (primitive-wip.md). `mana_leak.rs` / `mana_tithe.rs` / etc. still carry
their pre-AC2 TODO stubs (confirmed via grep). Card-def review is therefore N/A for this
implement-phase commit.

## Verdict: RESOLVED (was needs-fix)

**Resolution (commits 50edcbcf, 456a0bd7):** MEDIUM #1 (Sequence atomicity) FIXED via
cumulative-depletion scratch-clone probe; MEDIUM #4 (real-card coverage) CLOSED by
`crates/engine/tests/pb_ac2_card_integration.rs` (5 real-CardDefinition tests); LOW #2
(multi-payer ctx scope) FIXED (payer rebind). LOW #3 note-only, left as-is. All HIGH/MEDIUM
findings addressed; 2919 tests pass.

Engine change is CR-correct on every point the brief flagged as high-risk: the hash covers
**all** fields of both new variants and bumps the schema (PB-AC1's HIGH miss is NOT repeated);
`PayLife` routes through `apply_life_loss_doubling` identically to `Effect::LoseLife` (CR 119.4);
the optional cost is paid at resolution inside `execute_effect_inner` (CR 118.12), not at
trigger-queue time; `CounterUnlessPays` delegates once to `Effect::CounterSpell` (no
double-counter, inherits flashback-exile per CR 702.34a); and the sacrifice-as-cost path is a
genuine refactor-extraction of the existing `SacrificePermanents` zone-move (dies triggers +
replacement effects preserved). No HIGH findings. Two MEDIUM (a real but non-roster-triggered
`Sequence` atomicity gap; zero real-card end-to-end coverage) and two LOW keep it at needs-fix.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM | `effects/mod.rs:7183` | **`Sequence` payability pre-check ignores cumulative depletion.** Each sub-cost is checked against the *initial* unmutated state, so a homogeneous-resource sequence over-reports payability, partial-pays, and still runs `then` — a CR 118.12 atomicity violation. **Fix:** simulate cumulative consumption (clone state, pay each sub-cost against the clone, commit only if all succeed) or document a heterogeneous-resource-only restriction. |
| 2 | LOW | `effects/mod.rs:2999` | **Multi-payer `then` is mis-scoped.** For `payer` resolving to players other than the controller, `then` runs with unchanged `ctx.controller`, so "each player may pay X; if they do, *they* draw" would benefit the controller instead of the payer. No roster card uses a non-`Controller` payer. **Fix:** rebind `then`'s implicit controller to `pid`, or document the single-payer restriction on the variant. |
| 3 | LOW | `effects/mod.rs:2999` | **Deterministic always-pay can self-harm.** "Pay when able" always pays even when strictly detrimental (crossway pays 2 life on *every* Vampire death, potentially to a losing life total). Legal + deterministic (invariant #9), but a modeling artifact until M10+ interactivity. **Fix:** none required now; note in affected defs during backfill so a reader doesn't mistake it for a bug. |

## Card Definition Findings

None this phase — no card defs were modified (see Card Def Summary).

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | MEDIUM | (test coverage) | **No real-card end-to-end coverage.** The 5 card-integration tests named in the plan (crossway_troublemakers, hazorets_monument *unconditional-draw regression*, springbloom_druid, nadir_kraken, mana_leak) are deferred to backfill; the primitive currently has only synthetic-effect tests. The hazoret's-monument "must NOT draw unconditionally" wrong-state regression is unguarded until backfill lands. **Fix:** ensure the backfill phase adds these 5 tests before PB-AC2 is closed; do not sign off the batch on the implement-phase tests alone. |

### Finding Details

#### Finding 1: Sequence atomicity ignores cumulative resource depletion
**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:7183` (`can_pay_optional_cost`, `Cost::Sequence`)
**CR Rule**: 118.12 — "The action [do something] is a cost, paid when the spell or ability
resolves." A cost is paid in full or not at all.
**Issue**: `Cost::Sequence(costs) => costs.iter().all(|c| can_pay_optional_cost(state, pid, c))`
checks every sub-cost against the same initial state. `try_pay_optional_cost` then calls
`pay_optional_cost`, which pays each sub-cost in order with **no re-check**. For heterogeneous
resources (the only roster case, Miara = `Sequence[Mana{1}, PayLife(1)]`) this is correct. But
for homogeneous sequences it breaks:
- `Sequence[PayLife(1), PayLife(1)]` at life 1 → both checks pass (1 ≥ 1), pays to life −1.
- `Sequence[Sacrifice(x), Sacrifice(x)]` with one eligible permanent → both checks pass, first
  sacrifice consumes it, second `sacrifice_permanents_for_player` finds nothing and sacrifices
  0, yet `then` still runs — **partial payment with `then` firing**, a direct CR 118.12 violation.
- `Sequence[Mana{1}, Mana{1}]` with 1 floating → second `pay_cost` runs against a depleted pool.

The doc comment claims the sequence is "atomic," which overstates the guarantee. Not currently
triggered by any shipped card, but the primitive is general and a future author can reasonably
write such a sequence.
**Fix**: make the pre-check cumulative — e.g. clone the payer-relevant state, run
`pay_optional_cost` for each sub-cost against the clone, and return true only if all sub-costs
remain payable at their turn; or reject/document homogeneous sequences explicitly.

#### Finding 4: Primitive has no real-card end-to-end coverage yet
**Severity**: MEDIUM (test gap; phase-appropriate but must be closed before batch close)
**File**: `crates/engine/tests/optional_cost_and_counter_tax.rs`
**Oracle**: hazorets_monument — "Whenever you cast a creature spell, you may discard a card. If
you do, draw a card." (currently authored as an *unconditional* draw = wrong game state, per plan)
**Issue**: All 15 tests drive `Effect::MayPayThenEffect` / `Effect::CounterUnlessPays` directly
against synthetic effects. No test exercises a real card def through cast/trigger/resolution, so
the wrong-state fix the plan calls out for hazoret's monument (remove the unconditional draw and
wrap it in `MayPayThenEffect{DiscardCard → Draw}`) has no regression guard yet.
**Fix**: land the 5 named integration tests in the backfill phase and keep them as the gate for
closing PB-AC2.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 118.12 (cost paid at resolution) | Yes | Yes | Executed in `execute_effect_inner`; pay/decline pairs for all cost kinds |
| 118.12a (counter unless pays) | Yes | Yes | `test_counter_unless_pays_counters_when_declined` |
| 118.8 / 500.4 (mana from pool only) | Yes | Yes | `_mana_requires_floating` + `_mana_empty_pool_declines` |
| 119.4 (pay life = lose life; life ≥ n) | Yes | Yes | doubling routed; `_paylife_pays`/`_paylife_insufficient` |
| 613.1d (layer-resolved sac filter) | Yes | Partial | `eligible_sacrifice_targets` uses `calculate_characteristics`; only creature-type filter tested |
| 701.21a (sacrifice ≠ destruction, dies triggers) | Yes | Yes | shared helper; `CreatureDied`+`PermanentSacrificed` asserted |
| 702.34a (flashback counter → exile) | Yes | Yes | `test_counter_unless_pays_flashback_exiles` |
| 118.12 Sequence atomicity | Partial | Partial | Heterogeneous only (Finding 1); homogeneous not covered |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| (all 13 roster cards) | N/A | unchanged | N/A | Card defs untouched this phase; deferred to backfill. `mana_leak.rs`/`mana_tithe.rs` still carry pre-AC2 TODO stubs. |

## Hash Verification (PB-AC1 HIGH-miss re-check)

- `Effect::MayPayThenEffect` (disc 88): hashes `cost` + `payer` + `then` — **all 3 fields**. PASS.
- `Effect::CounterUnlessPays` (disc 89): hashes `target` + `cost` — **all 2 fields**, including
  the execution-inert `cost`. PASS.
- `HASH_SCHEMA_VERSION` 28 → 29 with history entry (hash.rs:213-217); parity test
  `test_hash_schema_version_is_29` + `test_hash_distinguishes_new_effect_variants` present. PASS.
- No new runtime/struct fields were added, so no other `HashInto` impls needed extension. PASS.

## Notes on brief focus areas

1. **MayPayThenEffect semantics** — `then` runs only inside the `if try_pay_optional_cost(...)`
   guard; no-legal-payment ⇒ `false` ⇒ `then` skipped, no state mutation. Correct. Atomicity
   holds for heterogeneous costs; see Finding 1 for the homogeneous gap.
2. **CounterUnlessPays** — single delegation to `CounterSpell` with cloned target; no
   double-counter, correct flashback-exile inheritance (test-verified).
3. **Sacrifice helper extraction** — pure extraction; `Effect::SacrificePermanents` now calls
   `sacrifice_permanents_for_player`, which retains the full replacement-check / dies-trigger /
   `PermanentSacrificed` event path. No regression. Single definition site (grep-confirmed).
4. **hash.rs** — complete (see Hash Verification). The PB-AC1 failure mode is not repeated.
5. **PayLife** — routes through `apply_life_loss_doubling`, mirrors `Effect::LoseLife` exactly
   (doubling + `life_lost_this_turn` + `LifeLost` event). Correct.
6. **Timing** — paid at resolution in `execute_effect_inner`, not at trigger-queue. Correct.
