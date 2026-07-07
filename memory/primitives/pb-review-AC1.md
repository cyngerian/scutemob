# Primitive Batch Review: PB-AC1 — Counter / Untap / Once-per-turn primitives

**Date**: 2026-07-07
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit**: 19b1f364
**CR Rules**: 701.26b, 502.3, 502.4, 603.2c, 603.2e, 603.2h, 122.6, 122.7, 613.1f
**Engine files reviewed**: `effects/mod.rs`, `rules/abilities.rs`, `rules/turn_actions.rs`,
`rules/layers.rs`, `state/hash.rs`, `state/game_object.rs` (via grep), `cards/card_definition.rs`
(via grep), `tools/replay-viewer/src/view_model.rs`
**Card defs reviewed**: `morbid_opportunist.rs`, `mesmeric_orb.rs`, `goblin_sharpshooter.rs`,
`sharktocrab.rs` (4/4)
**Tests reviewed**: `crates/engine/tests/pb_ac1_untap_counter.rs` (20 tests)

## Verdict: needs-fix

The five primitives are implemented correctly against the CR at the dispatch/enforcement
level: `Effect::UntapAll` guards on `status.tapped` (CR 701.26b) and filter/controller scope;
`WheneverPermanentUntaps` fires only on real untap events and holds untap-step triggers to
upkeep (CR 502.3/502.4/603.2e); `WhenCounterPlaced` on_self/filter/counter-kind semantics and
the runner's post-filter deviation are correct; the `once_per_turn` gate in
`flush_pending_triggers` correctly skips-if-fired, forces `additional_count=0`, marks-after-push,
and resets each untap step (CR 603.2c/603.2h); and `DoesNotUntap` is layer-resolved, does not
decrement `skip_untap_steps`, and is removed by Humility (CR 502.3/613.1f). All 4 card defs
match oracle text with no residual TODO/ENGINE-BLOCKED markers. **However, two hash omissions
break the hash-schema invariant** — one of them (the mutable `triggered_abilities_fired_this_turn`
field) is a genuine state-identity collision between reachable states of the same game. These
must be fixed before the batch is clean.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `state/hash.rs:1305` | **`triggered_abilities_fired_this_turn` not hashed.** Mutable runtime field omitted from `HashInto for GameObject`. **Fix:** add `self.triggered_abilities_fired_this_turn.hash_into(hasher);` before the closing brace at L1306. |
| 2 | **MEDIUM** | `state/hash.rs:2529` | **`TriggeredAbilityDef` hash omits `once_per_turn`/`counter_filter`/`counter_on_self`.** Three new fields not hashed; reachable via `Characteristics.triggered_abilities` (L1150). **Fix:** add the three `.hash_into(hasher)` calls to the impl. |
| 3 | **MEDIUM** | `rules/abilities.rs:3530` (design) | **CR 122.6 enters-with-counters gap.** `WhenCounterPlaced` cannot fire for counters an object enters with (no `CounterAdded` emitted on ETB-with-counters). Does not affect the 4 shipped cards. **Fix:** file a tracking issue ID and reference it in the assert-current-behavior test comment. |
| 4 | LOW | `rules/abilities.rs:6696` | **once_per_turn fallback indexes card-def abilities with a runtime-triggered index.** Mismatched for cards with non-triggered abilities before triggered ones (e.g. Goblin Sharpshooter). Only reached for non-Normal PendingTriggerKind (never once_per_turn today), so currently harmless. **Fix:** document the index-space assumption or guard. |
| 5 | LOW | `rules/abilities.rs:3530` | **No `count > 0` guard** on the `CounterAdded` check_triggers arm (plan suggested one). Emission sites guard `modified_count > 0`, so harmless. **Fix:** add `if *count > 0` for defense-in-depth, optional. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 6 | LOW | `morbid_opportunist.rs` | **Self-dies-alongside-others ruling untested.** 2024-11-08 ruling (still triggers if Morbid dies simultaneously) relies on pre-existing `WheneverCreatureDies` death-path look-back; not exercised by any PB-AC1 test (`test_once_per_turn_trigger_batched_deaths` kills only opponents; source survives). Def is correct (`exclude_self:true`). **Fix:** add a test where the source dies in the same batch. |

### Finding Details

#### Finding 1: `triggered_abilities_fired_this_turn` not hashed (HIGH)

**File**: `crates/engine/src/state/hash.rs:1305` (end of `impl HashInto for GameObject`)
**CR / convention**: `memory/conventions.md` "Hash bump rule" + reviewer severity guideline
("missing hash field" = HIGH).
**Issue**: The GameObject hash impl ends at `self.skip_untap_steps.hash_into(hasher)` (L1305);
`triggered_abilities_fired_this_turn: im::OrdSet<usize>` is never hashed. Unlike the static
`once_per_turn` field, this set is **mutated during play** — inserted when a once-per-turn
ability is put on the stack (`abilities.rs:7978`) and cleared each untap step
(`layers.rs:1464`). Two reachable states of the same game that differ only in whether Morbid
Opportunist has already fired this turn (one draws on the next death, the other does not) hash
identically. That defeats state-identity/divergence detection and the loop-detection fingerprint
(CR 104.4b). The plan explicitly mandated this hash (Change 4 / Change 7 table). The HASH schema
doc comment at L210-211 even *claims* the field is hashed — an aspirationally-wrong comment
(conventions.md §"Aspirationally-wrong comments") until the line is added.
**Fix**: insert `self.triggered_abilities_fired_this_turn.hash_into(hasher);` immediately before
L1306's closing brace. (HASH_SCHEMA_VERSION already bumped to 28; no further bump needed.)

#### Finding 2: TriggeredAbilityDef hash omits three new fields (MEDIUM)

**File**: `crates/engine/src/state/hash.rs:2529-2542` (`impl HashInto for TriggeredAbilityDef`)
**CR / convention**: Hash convention + the direct precedent at L2363 where
`ActivatedAbility::once_per_turn` IS hashed ("two abilities differing only in this flag must
hash to distinct values").
**Issue**: `once_per_turn`, `counter_filter`, and `counter_on_self` were added to the runtime
`TriggeredAbilityDef` but the `HashInto` impl still stops at `triggering_creature_filter`
(L2541). This type is reached from the live hash via `Characteristics.triggered_abilities`
(L1150), so two Characteristics differing only in these fields collide. These fields are
effectively static per object (set once from the card def, never mutated), so the practical
divergence risk is low — but a `LayerModification`-granted triggered ability differing only in
`counter_filter` would collide, the plan explicitly required these (Change 7), and the
ActivatedAbility precedent makes the omission inconsistent.
**Fix**: add `self.once_per_turn.hash_into(hasher); self.counter_filter.hash_into(hasher);
self.counter_on_self.hash_into(hasher);` to the impl.

#### Finding 3: CR 122.6 enters-with-counters fidelity gap (MEDIUM)

**File**: dispatch design; test at `pb_ac1_untap_counter.rs:1067`
**CR / oracle**: CR 122.6 ("refers to ... also to an object that's given counters as it enters
the battlefield"); Sharktocrab ruling 2019-01-25 ("will trigger if that permanent somehow
enters the battlefield with those counters").
**Issue**: `WhenCounterPlaced` fires only off `GameEvent::CounterAdded`, which the ETB-with-
counters path does not emit; a permanent entering with counters therefore never fires the
trigger. This is a genuine wrong-game-state gap for a class of cards (Fathom Mage / Dusk Legion
Duelist / Simic Ascendancy entering with counters, Sharktocrab if forced to enter with a
counter). It does **not** affect the 4 shipped cards in normal play (none enter with +1/+1
counters), and the underlying cause is a pre-existing ETB emission gap, not something PB-AC1
introduced. The batch's assert-current-behavior test is acceptable practice, but the wrong
behavior is asserted as "current" with no tracking ID.
**Fix**: not a batch blocker — file a tracking issue (e.g. `WCP-ETB-01`) for ETB-with-counters
`CounterAdded` emission and cite it in the test comment at L1067 so the assertion is a tracked
gap, not silent debt. Escalate to coordinator; do not expand scope in this PB.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 701.26b (only tapped untap) | Yes (`effects/mod.rs:1923`) | Yes | test_untap_all_only_tapped, _untaps_matching |
| 502.3 (effects keep from untapping) | Yes (`turn_actions.rs:1220`) | Yes | DoesNotUntap keeps tapped; multiplayer scope |
| 502.4 (untap-step triggers held to upkeep) | Yes (pending_triggers hold) | Yes | _fires_at_untap_step_held_to_upkeep |
| 603.2e (becomes-untapped not on ETB) | Yes (event-only dispatch) | Yes | _not_on_enters_untapped |
| 122.6/122.7 (counters put on / kind) | Partial (ETB gap, Finding 3) | Yes | on_self/filter/kind tested; ETB gap asserted |
| 603.2c (once each occurrence, batch) | Yes (`flush` gate) | Yes | _batched_deaths → exactly 1 |
| 603.2h (once each turn / doubler) | Yes (`additional_count=0`) | Yes | _fires_once_across_turn; reset via expire |
| 613.1f (Humility removes DoesNotUntap) | Yes (layer-resolved) | Yes | _removed_by_humility (wedge) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| morbid_opportunist | Yes | 0 | Yes | exclude_self:true + once_per_turn:true correct; self-dies ruling untested (Finding 6) |
| mesmeric_orb | Yes | 0 | Yes | filter:None global; ControllerOf(TriggeringCreature) resolves via entering_object_id |
| goblin_sharpshooter | Yes | 0 | Yes | all 3 clauses; Keyword(DoesNotUntap), any-creature-dies untap, {T} 1 dmg any target |
| sharktocrab | Yes | 0 | Yes | Adapt + on_self +1/+1 trigger → tap opponent creature + PreventNextUntap; targeting correct |

## Residuals (acceptable, tracked)

- **`test-data/generated-scripts/baseline/105_sharktocrab_adapt.json`** demoted approved →
  pending_review: correct handling — Sharktocrab legitimately gained a new counter trigger that
  changes stack contents during Adapt. Tracked in the script's `generation_notes`. Not a
  HIGH/MEDIUM; must be regenerated (extra priority/stack_resolve steps) before it re-counts as
  covered.
- **Hash discriminants**: Effect::UntapAll=87, TriggerCondition::WheneverPermanentUntaps=42 /
  WhenCounterPlaced=43, TriggerEvent::AnyPermanentUntaps=45 / CounterPlaced=46,
  KeywordAbility::DoesNotUntap=162, HASH_SCHEMA_VERSION=28 — all verified correct and unique.
  view_model.rs DoesNotUntap display arm present ("Doesn't Untap"). No missing exhaustive arms.

## Fix list (HIGH/MEDIUM to clear before clean)

1. **HIGH** — `state/hash.rs:1305`: hash `triggered_abilities_fired_this_turn`. **RESOLVED**
   (fix session 2026-07-07) — added `self.triggered_abilities_fired_this_turn.hash_into(hasher);`
   at the end of `impl HashInto for GameObject` (now `state/hash.rs` ~L1305-1309, immediately
   after `skip_untap_steps`). HASH_SCHEMA_VERSION left at 28 (schema completion, not a bump).
2. **MEDIUM** — `state/hash.rs:2529`: hash `once_per_turn`, `counter_filter`, `counter_on_self`
   on TriggeredAbilityDef. **RESOLVED** (fix session 2026-07-07) — added the three
   `.hash_into(hasher)` calls to `impl HashInto for TriggeredAbilityDef` after
   `triggering_creature_filter`.
3. **MEDIUM** — file a tracking issue for the CR 122.6 enters-with-counters gap and cite it in
   the `test_whencounterplaced_enters_with_counters_current_behavior` comment. **RESOLVED** (fix
   session 2026-07-07) — filed `MR-AC1-01` (LOW, OPEN) in
   `docs/mtg-engine-milestone-reviews.md` (LOW table + Statistics counts updated); test comment
   at `crates/engine/tests/pb_ac1_untap_counter.rs:1060-1065` updated to cite MR-AC1-01.

**Verification**: `cargo build --workspace` clean; `cargo test -p mtg-engine` all pass (hash
parity tests still assert HASH_SCHEMA_VERSION==28); `cargo clippy --workspace --all-targets -- -D
warnings` clean; `cargo fmt --check` clean. LOW findings 4-6 left open/optional per instructions.
