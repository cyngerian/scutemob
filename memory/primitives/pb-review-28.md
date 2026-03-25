# Primitive Batch Review: PB-28 -- CDA / Count-Based P/T

**Date**: 2026-03-25
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 604.3, 604.3a, 613.4, 613.4a, 613.4b, 613.4c
**Engine files reviewed**: `crates/engine/src/rules/layers.rs`, `crates/engine/src/state/continuous_effect.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/cards/card_definition.rs`, `crates/engine/src/rules/replacement.rs`, `crates/engine/src/effects/mod.rs`
**Card defs reviewed**: 9 (battle_squadron, molimo_maro_sorcerer, greensleeves_maro_sorcerer, cultivator_colossus, reckless_one, psychosis_crawler, abomination_of_llanowar, jagged_scar_archers, adeline_resplendent_cathar)

## Verdict: needs-fix

The engine implementation is solid -- Layer 7a evaluation, recursion avoidance, and
`EffectAmount::Sum` are all correct. Hash discriminants are unique within their respective
enums. The main issue is one card def (Abomination of Llanowar) with a graveyard filter
that is too restrictive per oracle text, and one documented CR 604.3 limitation for
non-battlefield zones. Nine tests cover the key scenarios well.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `layers.rs:1226-1237` | **PermanentCount uses base chars instead of layer-resolved chars for filter matching.** While this avoids recursion, it means type-changing effects (Layer 4) like Arcane Adaptation granting Elf subtype won't be reflected in CDA creature counts. See Finding 1. |
| 2 | **LOW** | `replacement.rs:1770` | **CR 604.3 all-zones: CDA only active on battlefield.** `WhileSourceOnBattlefield` duration means CDA P/T returns `None` for graveyard/hand/exile objects. See Finding 2. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **MEDIUM** | `abomination_of_llanowar.rs:35-38` | **Graveyard Elf filter too restrictive.** Oracle says "Elf cards in your graveyard"; filter requires `CardType::Creature`. See Finding 3. |

### Finding Details

#### Finding 1: PermanentCount uses base characteristics for filter matching

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/layers.rs:1226-1237`
**CR Rule**: 604.3 -- CDAs function in all zones; 613.4a -- Layer 7a
**Issue**: `resolve_cda_amount` deliberately uses `obj.characteristics` (base chars) instead of `calculate_characteristics()` to avoid infinite recursion. This is the correct approach for the recursion problem, but it means CDA filters check base (printed) characteristics rather than layer-resolved characteristics. If a Layer 4 type-changing effect (e.g., Arcane Adaptation adding Elf subtype to all creatures) is active, the CDA creature count won't reflect the added subtypes because Layer 4 hasn't been applied to the base chars. This is an inherent limitation of the recursion avoidance strategy. The comment at lines 1230-1236 documents this well.
**Fix**: No code change required for PB-28. This is a known limitation. For future improvement, consider a recursion guard (e.g., thread-local or passed flag) that allows calling `calculate_characteristics` on OTHER objects while skipping the CDA object itself. Low priority since Layer 4 type changes affecting CDA counts are rare in Commander.

#### Finding 2: CDA not evaluated for non-battlefield zones (CR 604.3)

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1770`
**CR Rule**: 604.3 -- "Characteristic-defining abilities function in all zones. They also function outside the game and before the game begins."
**Issue**: The CDA continuous effect uses `EffectDuration::WhileSourceOnBattlefield`, so when the creature is in the graveyard, hand, or exile, `is_effect_active()` returns false and `calculate_characteristics()` returns `power: None`. This violates CR 604.3 -- a `*/*` creature in the graveyard should have its CDA-computed P/T (relevant for "return target creature card with power 2 or less" effects). The plan explicitly acknowledged this as an acceptable limitation for alpha.
**Fix**: No code change required for PB-28. For future improvement: in `calculate_characteristics`, after the layer loop, if `chars.power` is `None` and the source CardDefinition has a `CdaPowerToughness` ability, evaluate the CDA directly from the card definition. This would handle non-battlefield zones without needing a persistent continuous effect.

#### Finding 3: Abomination of Llanowar graveyard filter requires Creature type

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/abomination_of_llanowar.rs:35-38`
**Oracle**: "Abomination of Llanowar's power and toughness are each equal to the number of Elves you control plus the number of Elf cards in your graveyard."
**Issue**: The graveyard `CardCount` filter at lines 35-38 uses `has_card_type: Some(CardType::Creature)` combined with `has_subtype: Some(SubType("Elf"))`. The oracle text says "Elf cards" (not "Elf creature cards"), meaning any card with the Elf subtype should count, including Kindred/Tribal Elf spells (e.g., Prowess of the Fair, Gilt-Leaf Ambush). The `has_card_type: Creature` restriction incorrectly excludes these. The battlefield half (`PermanentCount`) using `has_card_type: Creature` is acceptable since "Elves you control" effectively means Elf creatures on the battlefield.
**Fix**: In the graveyard `CardCount` filter (lines 35-38), remove `has_card_type: Some(CardType::Creature)` so the filter is just `has_subtype: Some(SubType("Elf".to_string()))`:
```rust
filter: Some(TargetFilter {
    has_subtype: Some(SubType("Elf".to_string())),
    ..Default::default()
}),
```

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 604.3 | Partial | No | CDA works on battlefield; non-battlefield zones return None (Finding 2) |
| 604.3a | Yes | Yes | Unconditional (no condition field); defines P/T; printed on card |
| 613.4a | Yes | Yes | test_cda_layer_7a_before_7b, test_cda_layer_7a_before_7c |
| 613.4b | Yes | Yes | test_cda_layer_7a_before_7b (Humility overrides CDA) |
| 613.4c | Yes | Yes | test_cda_layer_7a_before_7c (+1/+1 counter stacks on CDA) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| battle_squadron | Yes | 0 | Yes | |
| molimo_maro_sorcerer | Yes | 0 | Yes | |
| greensleeves_maro_sorcerer | Yes | 1 (protection) | Yes | TODO is non-CDA ability (protection from planeswalkers/Wizards) |
| cultivator_colossus | Yes | 1 (ETB loop) | Yes | TODO is non-CDA ability (ETB land loop) |
| reckless_one | Yes | 0 | Yes | |
| psychosis_crawler | Yes | 0 | Yes | |
| abomination_of_llanowar | No | 0 | No | Finding 3: graveyard filter too restrictive |
| jagged_scar_archers | Yes | 1 (activated) | Yes | TODO is non-CDA ability (tap to deal damage) |
| adeline_resplendent_cathar | Yes | 1 (attack trigger) | Yes | TODO is non-CDA ability (per-opponent token) |
