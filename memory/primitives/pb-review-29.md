# Primitive Batch Review: PB-29 -- Cost Reduction Statics

**Date**: 2026-03-25
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 601.2f, 602.2b
**Engine files reviewed**: `cards/card_definition.rs`, `rules/casting.rs`, `rules/abilities.rs`, `effects/mod.rs`, `state/hash.rs`, `cards/helpers.rs`
**Card defs reviewed**: 13 (archmage_of_runes, bontus_monument, hazorets_monument, oketras_monument, urzas_incubator, winged_words, cavern_hoard_dragon, boseiju_who_endures, otawara_soaring_city, eiganjo_seat_of_the_empire, takenuma_abandoned_mire, sokenzan_crucible_of_defiance, voldaren_estate)

## Verdict: needs-fix

One HIGH finding: `TargetFilter::hash_into()` does not include the new `legendary` field, which breaks state hash correctness for objects using `TargetFilter` with `legendary: true`. One MEDIUM finding: Hazoret's Monument trigger implements unconditional draw instead of optional loot. One LOW finding: `ConditionalKeyword` uses base characteristics rather than layer-resolved keywords.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `state/hash.rs:3697-3715` | **TargetFilter hash missing `legendary` field.** `hash_into()` for `TargetFilter` omits `self.legendary`, causing two filters differing only in `legendary` to produce identical hashes. **Fix:** Add `self.legendary.hash_into(hasher);` after `self.has_card_types.hash_into(hasher);` at line 3714. |
| 2 | LOW | `rules/casting.rs:5896` | **ConditionalKeyword uses base characteristics.** `obj.characteristics.keywords` reads pre-layer keywords. A creature granted Flying by a continuous effect (e.g., Levitation) would not trigger the Winged Words reduction. Consistent with existing `ConditionalPowerThreshold` pattern -- no fix needed now, but document as known approximation. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | MEDIUM | `hazorets_monument.rs` | **Triggered ability wrong game state.** Oracle: "you may discard a card. If you do, draw a card." Def implements unconditional `DrawCards`. This produces wrong game state (player draws without discarding). TODO at line 23 documents this as a DSL gap. **Fix:** No PB-29 fix -- this is a pre-existing DSL gap for optional loot. Carry forward as a known wrong-game-state card. |

### Finding Details

#### Finding 1: TargetFilter hash missing `legendary` field

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:3697-3715`
**CR Rule**: N/A -- architecture invariant: all fields contributing to object identity must be hashed
**Issue**: The `TargetFilter` struct gained a `legendary: bool` field in PB-29, but the `HashInto` implementation for `TargetFilter` was not updated to include it. `TargetFilter` is hashed as part of `AbilityDefinition` (which is on `Characteristics` on `GameObject`), so this is a state hash correctness violation. Two different `TargetFilter` values (one with `legendary: true`, one with `legendary: false`) would produce identical hashes, potentially masking game state divergence in distributed verification.

Currently, `legendary: true` is only used in `CardDefinition.activated_ability_cost_reductions` (not directly on game state), but `TargetFilter` is widely used in game-state-reachable types (`TargetRequirement`, `ETBSuppressFilter`, `SelfCostReduction::PerPermanent`, etc.). Any future use of `legendary: true` in a game-state context would silently break hash verification.

**Fix**: In `hash.rs` at line 3714, add `self.legendary.hash_into(hasher);` after the `self.has_card_types` line.

#### Finding 2: ConditionalKeyword uses base characteristics (LOW)

**Severity**: LOW
**File**: `crates/engine/src/rules/casting.rs:5896-5900`
**CR Rule**: 601.2f -- cost determination happens during casting, should use current game state
**Issue**: `ConditionalKeyword` checks `obj.characteristics.keywords` (base/printed keywords) rather than layer-resolved keywords. A creature granted Flying by a continuous effect would not be detected. This is consistent with the existing `ConditionalPowerThreshold` variant which also uses base characteristics. Known approximation, documented in the plan.
**Fix**: No fix now. Document as known LOW approximation in a comment. Will be addressed holistically when layer-resolved characteristics are used for all cost evaluation.

#### Finding 3: Hazoret's Monument wrong game state (MEDIUM)

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/hazorets_monument.rs:22-36`
**Oracle**: "Whenever you cast a creature spell, you may discard a card. If you do, draw a card."
**Issue**: The triggered ability implements unconditional `DrawCards` instead of optional loot (discard-then-draw). The TODO at line 23 documents this as a DSL gap. PB-29 correctly resolved the cost reduction TODO but the trigger produces wrong game state -- the player draws a card without the option to decline or the requirement to discard first. This is a pre-existing issue carried forward.
**Fix**: No PB-29 fix required. This is a pre-existing DSL gap (optional "may discard, if you do, draw" pattern). Carry forward to the appropriate DSL gap batch.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 601.2f (cost increases/reductions) | Yes | Yes | 11 pre-existing + 11 new tests |
| 601.2f (floor at {0}) | Yes | Yes | test_activated_ability_self_cost_reduction_floor_zero |
| 601.2f (multiple reductions, player chooses order) | Partial | No | Engine applies all reductions additively; player order choice not implemented (generic mana is commutative so this is correct for generic-only reductions) |
| 602.2b (activated abilities follow 601.2b-i) | Yes | Yes | test_activated_ability_self_cost_reduction_per_legendary |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| archmage_of_runes | Yes | 0 | Yes | Clean |
| bontus_monument | Yes | 0 | Yes | Clean |
| hazorets_monument | Partial | 1 (loot trigger) | No | Trigger draws without discard option (pre-existing) |
| oketras_monument | Yes | 0 | Yes | Clean |
| urzas_incubator | Yes | 0 | Yes | AllPlayers scope correct per oracle |
| winged_words | Yes | 0 | Yes | Clean |
| cavern_hoard_dragon | Partial | 1 (combat trigger) | Partial | Cost reduction correct; combat trigger missing (G-8 gap) |
| boseiju_who_endures | Partial | 2 (target filter) | Partial | Cost reduction correct; target filter overly broad (pre-existing) |
| otawara_soaring_city | Yes | 0 | Yes | Clean; non_land filter is acceptable approximation |
| eiganjo_seat_of_the_empire | Partial | 1 (attack/block filter) | Partial | Cost reduction correct; target filter overly broad (pre-existing) |
| takenuma_abandoned_mire | Yes | 0 | Yes | Clean |
| sokenzan_crucible_of_defiance | Partial | 1 (haste duration) | Partial | Cost reduction correct; haste is permanent not UntilEndOfTurn (pre-existing) |
| voldaren_estate | Yes | 0 | Yes | Index 1 correct (pay-life ability is activated[0], blood is activated[1]) |

## Design Notes

The runner correctly chose the alternative design (Vec on CardDefinition instead of field on AbilityDefinition::Activated). This avoided touching 400+ match sites and is a pragmatic engineering choice. The index-based coupling is documented in comments and tested. The tradeoff is acceptable given the scope of changes it avoided.

The `HasChosenCreatureSubtype` special-casing in `apply_spell_cost_modifiers()` (inline rather than in `spell_matches_cost_filter()`) is the right approach since the latter function lacks the source object context. Well-documented with an unreachable fallback arm returning `false`.

The `activated_ability_cost_reductions` index mapping is correct for all cards:
- Channel lands: mana tap goes to `mana_abilities`, channel is `activated_abilities[0]`
- Voldaren Estate: colorless tap goes to `mana_abilities`, pay-life mana is `activated_abilities[0]`, blood token is `activated_abilities[1]`
