# Ability Review: Suspect

**Date**: 2026-03-07
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 701.60
**Files reviewed**:
- `crates/engine/src/state/game_object.rs:477-486`
- `crates/engine/src/cards/card_definition.rs:880-888`
- `crates/engine/src/rules/events.rs:791-811`
- `crates/engine/src/state/hash.rs:841-842` (GameObject), `2984-2998` (GameEvent), `3626-3635` (Effect)
- `crates/engine/src/state/builder.rs:974-975`
- `crates/engine/src/state/mod.rs:347-349, 504-506`
- `crates/engine/src/effects/mod.rs:1715-1754, 2870-2871`
- `crates/engine/src/rules/resolution.rs:3631-3633, 4754-4756, 4950-4952, 5163-5165`
- `crates/engine/src/rules/layers.rs:62-73`
- `crates/engine/src/rules/combat.rs:553-564, 854-857`
- `crates/engine/tests/suspect.rs` (full file, 455 lines)

## Verdict: clean

The implementation correctly models Suspect as a keyword action (not a keyword ability),
with proper designation tracking on GameObject, Menace grant in the pre-layer-loop block
of calculate_characteristics, can't-block enforcement in combat.rs, idempotency, zone-change
clearing, and hash coverage. All four CR 701.60 subrules are addressed. The one inaccuracy
(can't-block persisting under Humility ability-removal) is documented with a TODO and
accepted in the plan as a known limitation. No HIGH or MEDIUM findings.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `tests/suspect.rs:366-410` | **Weak not-copiable test.** Does not actually create a copy. |
| 2 | LOW | `combat.rs:553-564` | **Humility can't-block inaccuracy.** Known, documented, deferred. |

### Finding Details

#### Finding 1: Weak not-copiable test

**Severity**: LOW
**File**: `crates/engine/tests/suspect.rs:366-410`
**CR Rule**: 701.60b -- "Suspected is neither an ability nor part of the permanent's copiable values."
**Issue**: The test (`test_suspect_not_copiable`) checks that `is_suspected` lives on
`GameObject` rather than `Characteristics`, but it never actually creates a copy of the
suspected creature and verifies the copy's `is_suspected` is false. It is a structural
assertion rather than a behavioral test. The implementation IS correct by construction
(copy.rs operates on `Characteristics` only, and all object-creation sites initialize
`is_suspected: false`), so this is not a correctness gap -- just a test quality gap.
**Fix**: Optionally enhance the test to create a token copy (via Effect::CreateToken or
direct state manipulation) of the suspected creature and assert `copy.is_suspected == false`.
Not urgent since correctness is guaranteed by architecture.

#### Finding 2: Humility can't-block inaccuracy

**Severity**: LOW
**File**: `crates/engine/src/rules/combat.rs:553-564`
**CR Rule**: 701.60c -- "A suspected permanent has menace and 'This creature can't block' for as long as it's suspected."
**Issue**: The can't-block check uses `obj.is_suspected` directly on the raw GameObject,
bypassing the layer system. Under Humility (Layer 6 "lose all abilities"), the Menace
grant IS correctly stripped (it's added pre-layer and removed by Humility in Layer 6),
but the can't-block restriction persists because it checks the raw designation flag
instead of a layer-resolved ability. Per the 2024-02-02 ruling, both menace and
can't-block should be removed under ability-removal effects. This is documented with a
TODO in the code and explicitly accepted in the plan. The same pattern is used for
Decayed (keyword check on raw object).
**Fix**: Deferred. When Humility is fully implemented, model can't-block as a
layer-resolvable restriction rather than a raw flag check. Track under existing TODO.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 701.60a (suspect action) | Yes | Yes | test_suspect_basic_gains_menace, test_suspect_zone_change_clears |
| 701.60a (unsuspect) | Yes | Yes | test_unsuspect_removes_menace_and_blocking_restriction |
| 701.60b (designation, not copiable) | Yes | Weak | test_suspect_not_copiable checks structure, not actual copy behavior |
| 701.60c (menace grant) | Yes | Yes | test_suspect_basic_gains_menace, test_suspect_menace_evasion |
| 701.60c (can't block) | Yes | Yes | test_suspect_basic_cant_block, test_suspect_baseline_non_suspected_can_block |
| 701.60d (idempotent) | Yes | Yes | test_suspect_idempotent |
| Zone change clears (CR 400.7) | Yes | Yes | test_suspect_zone_change_clears |
| Can still attack | Yes | Yes | test_suspect_negative_can_attack |
| Hash: is_suspected field | Yes | n/a | hash.rs:841-842 |
| Hash: GameEvent variants | Yes | n/a | hash.rs:2984-2998 |
| Hash: Effect variants | Yes | n/a | hash.rs:3626-3635 |
| Init: builder.rs | Yes | n/a | builder.rs:974-975 |
| Init: mod.rs (2 sites) | Yes | n/a | mod.rs:349, 506 |
| Init: effects/mod.rs (1 site) | Yes | n/a | effects/mod.rs:2871 |
| Init: resolution.rs (4 sites) | Yes | n/a | resolution.rs:3633, 4756, 4952, 5165 |
| Forced-block exclusion | Yes | No | combat.rs:854-857 -- suspected skipped in provoke/goad forced-block |
