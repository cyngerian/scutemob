# Primitive Batch Review: PB-20 -- Additional Combat Phases

**Date**: 2026-03-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 500.8, CR 500.10a, CR 505.1a, CR 506.1
**Engine files reviewed**: `cards/card_definition.rs`, `rules/turn_structure.rs`, `effects/mod.rs`, `state/turn.rs`, `state/hash.rs`, `state/builder.rs`, `rules/events.rs`
**Card defs reviewed**: 3 (karlach_fury_of_avernus.rs, combat_celebrant.rs, breath_of_fury.rs)

## Verdict: needs-fix

One MEDIUM finding (Combat Celebrant untaps self instead of "other creatures"), one LOW finding (stale `in_extra_combat` during extra main phase). Engine changes are correct per CR rules. Karlach trigger condition gap is pre-existing and documented.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **LOW** | `turn_structure.rs:71-73` | **Stale `in_extra_combat` during extra PostCombatMain.** When EndOfCombat pops PostCombatMain from the queue, `in_extra_combat` remains `true` from the preceding extra combat. Benign: no condition checks this flag during main phases. **Fix:** Set `turn.in_extra_combat = false;` before the `Step::PostCombatMain` return at line 73. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | **MEDIUM** | `combat_celebrant.rs` | **Untaps self instead of "all other creatures."** Oracle says "untap all other creatures you control"; def uses `ForEachTarget::EachCreatureYouControl` which includes Combat Celebrant. **Fix:** Use a ForEachTarget variant that excludes the source, or add a filter. See details. |
| 3 | **LOW** | `combat_celebrant.rs` | **`IsFirstCombatPhase` used as Exert proxy.** Oracle gates on "hasn't been exerted this turn" not "first combat phase." These are different: multiple Combat Celebrants can each exert once per turn across different combat phases. Noted as TODO in the card def. Acceptable until Exert primitive exists. |
| 4 | **LOW** | `karlach_fury_of_avernus.rs` | **`WhenAttacks` vs "Whenever you attack" DSL gap.** Oracle: "Whenever you attack" triggers when the player declares attackers, even if Karlach is not attacking (confirmed by ruling). `WhenAttacks` requires Karlach to attack. Noted as DSL gap in card def comments. Pre-existing limitation. |
| 5 | **LOW** | `breath_of_fury.rs` | **`EnchantTarget::Creature` vs "Enchant creature you control."** Oracle restricts enchant target to controller's creatures. No `EnchantTarget::CreatureYouControl` variant exists. Pre-existing DSL gap. |

### Finding Details

#### Finding 1: Stale `in_extra_combat` during extra PostCombatMain

**Severity**: LOW
**File**: `crates/engine/src/rules/turn_structure.rs:71-73`
**CR Rule**: 500.8 -- "Some effects can add phases to a turn."
**Issue**: When the EndOfCombat branch pops `Phase::PostCombatMain` from the queue, the code returns `Step::PostCombatMain` without clearing `turn.in_extra_combat`. This flag remains `true` from the preceding extra combat phase. No game logic currently checks `in_extra_combat` during main phases, so this is benign, but it's technically incorrect state.
**Fix**: Add `turn.in_extra_combat = false;` before `Step::PostCombatMain` at line 73 (the PostCombatMain case inside the EndOfCombat branch).

#### Finding 2: Combat Celebrant untaps self instead of "all other creatures"

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/combat_celebrant.rs:33-38`
**Oracle**: "untap all other creatures you control"
**Issue**: The card def uses `ForEachTarget::EachCreatureYouControl` which includes Combat Celebrant itself. The oracle says "other creatures you control" -- Combat Celebrant should remain tapped (it is exerted/attacking). In practice, since Combat Celebrant tapped to attack and isn't vigilant, it would be untapped incorrectly, allowing it to block during the opponent's next turn when it should stay tapped (exerted creatures don't untap). Since the Exert "won't untap" tracking is also TODO, the immediate game state impact is that Combat Celebrant gets an extra untap it shouldn't.
**Fix**: Either (a) add a `ForEachTarget::EachOtherCreatureYouControl` variant that excludes `ctx.source` (mirroring `EachOtherAttackingCreature`), or (b) use `ForEachTarget::EachPermanentMatching` with a filter that excludes the source. Option (a) is more reusable. If adding the variant is out of scope for a fix pass, add a `// TODO: should be EachOtherCreatureYouControl (excludes self)` comment and mark as known incorrect.

#### Finding 3: IsFirstCombatPhase used as Exert proxy (Combat Celebrant)

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/combat_celebrant.rs:30`
**Oracle**: "If this creature hasn't been exerted this turn, you may exert it as it attacks."
**Issue**: The `intervening_if: Some(Condition::IsFirstCombatPhase)` is used as a simplification for the exert check. The oracle condition is "hasn't been exerted this turn" which is per-creature state, not per-turn-phase state. With two Combat Celebrants, the oracle allows each to exert once across different combat phases; the current def prevents both from triggering after the first combat. Documented as TODO in the card def.
**Fix**: No fix needed now. Document that the Exert primitive (future PB) must replace this condition.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 500.8 (phase insertion LIFO) | Yes | Yes | test_additional_combat_lifo_ordering |
| 500.10a (controller check) | Yes | Yes | test_additional_combat_not_on_opponents_turn (indirect) |
| 505.1a (extra mains are postcombat) | Yes | Yes | test_additional_combat_phase_with_main |
| 506.1 (combat structure) | Yes | Yes | Extra combat enters via BeginningOfCombat, begin_combat() initializes CombatState |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| karlach_fury_of_avernus | Partial | 0 | Partial | WhenAttacks vs "Whenever you attack" DSL gap (pre-existing, documented) |
| combat_celebrant | Partial | 1 (Exert) | No | Untaps self (should be "other"); Exert mechanic TODO |
| breath_of_fury | Partial | 1 (Aura re-attach) | N/A | Trigger body is placeholder; Aura re-attachment DSL gap |

## Previous Findings (first review)

N/A -- this is the initial review.
