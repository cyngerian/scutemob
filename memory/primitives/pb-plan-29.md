# Primitive Batch Plan: PB-29 -- Cost Reduction Statics

**Generated**: 2026-03-25
**Primitive**: Extend spell cost reduction filters (compound filters, chosen-subtype, conditional self-reduction) and add activated ability cost reduction infrastructure
**CR Rules**: 601.2f (spell total cost determination), 602.2b (activated abilities follow same cost process as spells)
**Cards affected**: ~13 existing fixes + 0 new (new card authoring paused)
**Dependencies**: PB-8 (basic spell cost reduction infrastructure -- DONE)
**Deferred items from prior PBs**: nighthawk_scavenger needs DistinctCardTypesInGraveyards variant (from PB-28 handoff -- not related to PB-29, carry forward)

## Primitive Specification

PB-29 closes gap G-7 from `docs/dsl-gap-closure-plan.md`. The engine already has two cost reduction mechanisms from PB-8:

1. **`SpellCostModifier`** on `CardDefinition` -- permanents that reduce/increase spell costs for a player (Goblin Warchief, Thalia, medallions). Uses `SpellCostFilter` enum and `CostModifierScope`. Applied in `apply_spell_cost_modifiers()` in casting.rs.

2. **`SelfCostReduction`** on `CardDefinition` -- spells that reduce their own cost based on game state (Blasphemous Act, Ghalta). Applied in `apply_self_cost_reduction()` in casting.rs.

**What's missing** (3 sub-features):

**A. New `SpellCostFilter` variants** -- The existing filter enum lacks compound filters needed by ~6 cards:
- `ColorAndCreature(Color)` -- "Red creature spells cost {1} less" (Monuments)
- `HasChosenCreatureSubtype` -- "Creature spells of the chosen type" referencing `chosen_creature_type` on the source permanent (Urza's Incubator, Herald's Horn)
- `AllSpells` -- "Spells you cast cost {1} less" (generic, no filter)

**B. New `SelfCostReduction` variants** -- 2 cards need new self-reduction patterns:
- `ConditionalKeyword { keyword, reduction }` -- "costs {1} less if you control a creature with [keyword]" (Winged Words)
- `MaxOpponentPermanents { filter, per }` -- "costs {X} less where X is the greatest number of [permanents] an opponent controls" (Cavern-Hoard Dragon)

**C. Activated ability cost reduction** -- Entirely new mechanism. ~10 cards have activated abilities that cost less based on game state. Two patterns:
- **Self-reduction on a specific ability**: "This ability costs {1} less to activate for each legendary creature you control" (5 channel lands, Voldaren Estate). This is analogous to `SelfCostReduction` but on an `AbilityDefinition::Activated`.
- **Global reduction from another permanent**: "Activated abilities of creatures cost {1} less to activate" (Heartstone, Training Grounds). This is analogous to `SpellCostModifier` but for activated abilities.

**Scope decision -- global activated ability cost reduction is DEFERRED.** The global activated ability cost reduction (Heartstone, Training Grounds, Silver-Fur Master) requires a new scan-all-permanents mechanism in `handle_activate_ability()`, a new filter type (what abilities qualify), and special "can't reduce below 1 mana" handling. This is a significant feature (G-26 territory) that would bloat PB-29 beyond its scope. The self-reduction pattern for channel lands is simpler and IS included.

**In scope for PB-29:**
- Sub-feature A: 3 new `SpellCostFilter` variants (4 card fixes: monuments + incubator)
- Sub-feature B: 2 new `SelfCostReduction` variants (2 card fixes: Winged Words, Cavern-Hoard Dragon)
- Sub-feature C (partial): `SelfActivatedCostReduction` on `AbilityDefinition::Activated` for channel lands (6 card fixes)
- 1 card fix that needs no engine changes (Archmage of Runes -- existing filter works)
- 1 new `TargetFilter` field: `legendary: bool` (needed for channel land filters)
- Total: ~13 card fixes

**Deferred to PB-37 (complex activated abilities):**
- Heartstone (global activated ability cost reduction, all players)
- Training Grounds (global activated ability cost reduction, controller only)
- Silver-Fur Master (ninjutsu ability cost reduction)
- Puresteel Paladin (metalcraft conditional equip cost = {0})
- Morophon (colored mana reduction -- entirely different mechanism)

## CR Rule Text

**CR 601.2f**: The player determines the total cost of the spell. Usually this is just the mana cost. Some spells have additional or alternative costs. Some effects may increase or reduce the cost to pay, or may provide other alternative costs. Costs may include paying mana, tapping permanents, sacrificing permanents, discarding cards, and so on. The total cost is the mana cost or alternative cost (as determined in rule 601.2b), plus all additional costs and cost increases, and minus all cost reductions. If multiple cost reductions apply, the player may apply them in any order. If the mana component of the total cost is reduced to nothing by cost reduction effects, it is considered to be {0}. It can't be reduced to less than {0}. Once the total cost is determined, any effects that directly affect the total cost are applied. Then the resulting total cost becomes "locked in." If effects would change the total cost after this time, they have no effect.

**CR 602.2b**: The remainder of the process for activating an ability is identical to the process for casting a spell listed in rules 601.2b-i. Those rules apply to activating an ability just as they apply to casting a spell. An activated ability's analog to a spell's mana cost (as referenced in rule 601.2f) is its activation cost.

## Engine Changes

### Change 1: Add `legendary` field to `TargetFilter`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `legendary: bool` field to `TargetFilter` struct (near `basic: bool`, around line ~1746)
**CR**: General -- needed to express "legendary creature" filter for channel land cost reduction

```rust
/// Must have the Legendary supertype. Default: false (no restriction).
#[serde(default)]
pub legendary: bool,
```

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add check in `matches_filter()` (after the `basic` check, around line ~5034)

```rust
if filter.legendary
    && !chars
        .supertypes
        .contains(&crate::state::types::SuperType::Legendary)
{
    return false;
}
```

### Change 2: Add `SpellCostFilter` variants

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add 3 new variants to `SpellCostFilter` enum (after line ~2577)
**Pattern**: Follow existing `HasColor(Color)` variant

```rust
/// Creature spells of a specific color (Bontu's Monument: "Black creature spells").
/// Compound filter: must be BOTH a creature AND the specified color.
ColorAndCreature(Color),
/// Creature spells of the chosen creature type (Urza's Incubator, Herald's Horn).
/// References `chosen_creature_type` on the source permanent at cast time.
HasChosenCreatureSubtype,
/// All spells (no filter). Used for generic cost changes.
AllSpells,
```

### Change 3: Add `SelfCostReduction` variants

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add 2 new variants to `SelfCostReduction` enum (after line ~2614)
**Pattern**: Follow existing `ConditionalPowerThreshold` variant

```rust
/// "Costs {N} less if you control a creature with [keyword]" (Winged Words).
ConditionalKeyword {
    keyword: KeywordAbility,
    reduction: u32,
},
/// "Costs {X} less where X is the greatest number of [permanents matching filter] an
/// opponent controls" (Cavern-Hoard Dragon).
MaxOpponentPermanents {
    filter: TargetFilter,
    per: i32,
},
```

### Change 4: Add `self_cost_reduction` field to `AbilityDefinition::Activated`

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `self_cost_reduction: Option<SelfActivatedCostReduction>` to the `AbilityDefinition::Activated` variant (after `activation_condition` field, around line ~180)

```rust
AbilityDefinition::Activated {
    cost: Cost,
    effect: Effect,
    timing_restriction: Option<TimingRestriction>,
    targets: Vec<TargetRequirement>,
    activation_condition: Option<Condition>,
    /// CR 602.2b + 601.2f: Self-cost-reduction evaluated at activation time.
    /// "This ability costs {1} less for each legendary creature you control."
    #[serde(default)]
    self_cost_reduction: Option<SelfActivatedCostReduction>,
},
```

### Change 5: Add `SelfActivatedCostReduction` enum

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new enum near `SelfCostReduction` (after line ~2615)

```rust
/// Self-cost-reduction for activated abilities. Analogous to `SelfCostReduction` for spells.
/// CR 602.2b: activated ability costs follow the same reduction rules as spell costs (CR 601.2f).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelfActivatedCostReduction {
    /// "{1} less to activate for each <permanent matching filter> you control"
    /// (Channel lands: "{1} less for each legendary creature you control").
    PerPermanent {
        per: i32,
        filter: TargetFilter,
        controller: PlayerTarget,
    },
}
```

### Change 6: Update `spell_matches_cost_filter` dispatch

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add match arms for the 3 new `SpellCostFilter` variants in `spell_matches_cost_filter()` (after line ~5703)
**CR**: 601.2f

```rust
SpellCostFilter::ColorAndCreature(color) => {
    chars.card_types.contains(&CardType::Creature) && chars.colors.contains(color)
}
SpellCostFilter::HasChosenCreatureSubtype => {
    // Matched dynamically in apply_spell_cost_modifiers -- needs source object context.
    // This arm should not be reached directly; see apply_spell_cost_modifiers.
    false
}
SpellCostFilter::AllSpells => true,
```

### Change 7: Update `apply_spell_cost_modifiers` for `HasChosenCreatureSubtype`

**File**: `crates/engine/src/rules/casting.rs`
**Action**: In `apply_spell_cost_modifiers()` (around line ~5668), add special handling for `HasChosenCreatureSubtype` that reads `chosen_creature_type` from the source object and checks if the spell has that subtype AND is a creature.
**CR**: 601.2f

The `spell_matches_cost_filter` function doesn't have access to the source object, so the `HasChosenCreatureSubtype` check must happen inline in the loop body. Replace the single filter check at line 5669 with:

```rust
// Filter check: does the spell match?
let matches = match &modifier.filter {
    SpellCostFilter::HasChosenCreatureSubtype => {
        // Must be a creature spell with the source's chosen creature subtype.
        spell_chars.card_types.contains(&CardType::Creature)
            && obj.chosen_creature_type.as_ref()
                .map(|ct| spell_chars.subtypes.contains(ct))
                .unwrap_or(false)
    }
    other => spell_matches_cost_filter(spell_chars, other),
};
if !matches {
    continue;
}
```

### Change 8: Update `evaluate_self_cost_reduction` dispatch

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Add match arms for the 2 new `SelfCostReduction` variants in `evaluate_self_cost_reduction()` (after `ConditionalPowerThreshold` arm, around line ~5847)
**CR**: 601.2f

```rust
SelfCostReduction::ConditionalKeyword { keyword, reduction } => {
    // "Costs {N} less if you control a creature with [keyword]"
    let has_creature_with_keyword = state.objects.values().any(|obj| {
        obj.zone == ZoneId::Battlefield
            && obj.controller == caster
            && obj.characteristics.card_types.contains(&CardType::Creature)
            && obj.characteristics.keywords.contains(keyword)
    });
    if has_creature_with_keyword { *reduction as i32 } else { 0 }
}
SelfCostReduction::MaxOpponentPermanents { filter, per } => {
    // "Costs {X} less where X is the greatest number of [filter] an opponent controls"
    let max_count = state.players.keys()
        .filter(|&&pid| pid != caster)
        .map(|&pid| {
            state.objects.values()
                .filter(|obj| {
                    obj.zone == ZoneId::Battlefield
                        && obj.controller == pid
                        && crate::effects::matches_filter(&obj.characteristics, filter)
                })
                .count() as i32
        })
        .max()
        .unwrap_or(0);
    max_count * per
}
```

### Change 9: Apply `SelfActivatedCostReduction` in `handle_activate_ability`

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: After resolving the mana cost (line ~468) but before payment (line ~476), apply `self_cost_reduction` from the source card definition's activated ability.
**CR**: 602.2b + 601.2f

The `AbilityDefinition::Activated` variant needs to be read from the card definition (not the resolved `ActivatedAbility` on the game object, which doesn't carry `self_cost_reduction`). Look up the `CardDefinition` for the source, find the matching activated ability by index, and apply the reduction.

Insert after line ~475 (after x_value is resolved into `resolved_cost`):

```rust
// CR 602.2b + 601.2f: Apply self-activated-cost-reduction.
if let Some(card_id) = state.object(source).ok().and_then(|o| o.card_id.clone()) {
    if let Some(card_def) = state.card_registry.get(card_id) {
        if let Some(reduction) = get_self_activated_cost_reduction(&card_def, ability_index) {
            let amount = evaluate_self_activated_cost_reduction(state, player, &reduction);
            if amount > 0 {
                resolved_cost.generic = resolved_cost.generic.saturating_sub(amount);
            }
        }
    }
}
```

Add helper functions at end of abilities.rs:

```rust
/// Extract self_cost_reduction from a CardDefinition's activated ability at the given index.
fn get_self_activated_cost_reduction(
    card_def: &crate::cards::card_definition::CardDefinition,
    ability_index: usize,
) -> Option<crate::cards::card_definition::SelfActivatedCostReduction> {
    use crate::cards::card_definition::AbilityDefinition;
    // Walk abilities, counting only Activated variants to find the right index.
    let mut activated_idx = 0;
    for ab in &card_def.abilities {
        if let AbilityDefinition::Activated { self_cost_reduction, .. } = ab {
            if activated_idx == ability_index {
                return self_cost_reduction.clone();
            }
            activated_idx += 1;
        }
    }
    None
}

/// Evaluate a SelfActivatedCostReduction against current game state.
/// Returns the number of generic mana to subtract (always >= 0).
fn evaluate_self_activated_cost_reduction(
    state: &crate::state::GameState,
    controller: crate::state::player::PlayerId,
    reduction: &crate::cards::card_definition::SelfActivatedCostReduction,
) -> u32 {
    use crate::cards::card_definition::SelfActivatedCostReduction;
    match reduction {
        SelfActivatedCostReduction::PerPermanent { per, filter, controller: player_target } => {
            use crate::cards::card_definition::PlayerTarget;
            let target_player = match player_target {
                PlayerTarget::Controller => controller,
                PlayerTarget::ActivePlayer => state.turn.active_player,
                _ => controller,
            };
            let count = state.objects.values()
                .filter(|obj| {
                    obj.zone == crate::state::zone::ZoneId::Battlefield
                        && obj.controller == target_player
                        && crate::effects::matches_filter(&obj.characteristics, filter)
                })
                .count();
            (count as i32 * per).max(0) as u32
        }
    }
}
```

### Change 10: Exhaustive match updates for `AbilityDefinition::Activated`

The `AbilityDefinition::Activated` variant is destructured in many files. Adding `self_cost_reduction` requires updating every match arm.

**CRITICAL -- the runner MUST perform these greps before implementing:**

```
Grep pattern="AbilityDefinition::Activated" path="crates/engine/src" output_mode="content" -n=true
Grep pattern="AbilityDefinition::Activated" path="tools/" output_mode="content" -n=true
```

Every destructuring pattern that explicitly names fields (not using `..`) must add `self_cost_reduction`. Every construction site must add `self_cost_reduction: None` (unless it has a reduction).

Known files requiring updates:

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/cards/card_definition.rs` | `expand_cost_into_activated` / all Activated constructors (~lines 2282, 2326, 2373) | Add `self_cost_reduction: None` |
| `crates/engine/src/state/hash.rs` | HashInto for AbilityDefinition (search for `Activated` in hash.rs) | Add hash of `self_cost_reduction` field |
| `crates/engine/src/state/builder.rs` | Activated ability construction | Add `self_cost_reduction: None` |
| `crates/engine/src/rules/abilities.rs` | Activated destructuring in `handle_activate_ability` area | Add `self_cost_reduction` to pattern |
| `crates/engine/src/rules/casting.rs` | Activated matching (if any) | Add field |
| `crates/engine/src/rules/resolution.rs` | Activated matching | Add field |
| `crates/engine/src/effects/mod.rs` | Activated matching | Add field |
| `crates/engine/src/testing/replay_harness.rs` | Activated construction | Add `self_cost_reduction: None` |
| `tools/replay-viewer/src/view_model.rs` | Activated matching (if any) | Add field |

**All card def files with `AbilityDefinition::Activated { ... }`** also need the field. These are numerous (~100+ files). The easiest approach: use `..` rest syntax or add `self_cost_reduction: None` to each.

### Change 11: Export new types from helpers.rs

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Add `SelfActivatedCostReduction` to the `use crate::cards::card_definition::` import (line ~8-10)

### Change 12: Export from cards/mod.rs and lib.rs

**File**: `crates/engine/src/cards/mod.rs` (line ~17-19)
**File**: `crates/engine/src/lib.rs` (line ~10-13)
**Action**: Add `SelfActivatedCostReduction` to re-exports

### Change 13: Hash support for new types

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `HashInto` impl for `SelfActivatedCostReduction` enum. The `SpellCostFilter` and `SelfCostReduction` variants on `CardDefinition` are NOT hashed (CardDefinition is not part of GameState), but `SelfActivatedCostReduction` needs hashing IF it appears on any GameState-reachable struct. Check: `AbilityDefinition` is on `CardDefinition` (not GameState) and on `ActivatedAbility` (on `GameObject.characteristics`). The `ActivatedAbility` struct does NOT carry `self_cost_reduction` -- it's only on `AbilityDefinition::Activated`. Since `AbilityDefinition` IS hashed (it's in `CardDefinition.abilities` but also stored on resolved characteristics), the new variant needs hashing.

Search for existing `AbilityDefinition` hash handling:
```
Grep pattern="AbilityDefinition" path="crates/engine/src/state/hash.rs" output_mode="content" -n=true
```

Add `HashInto` for `SelfActivatedCostReduction` near the other card_definition type hashes.

## Card Definition Fixes

### archmage_of_runes.rs
**Oracle text**: Instant and sorcery spells you cast cost {1} less to cast. Whenever you cast an instant or sorcery spell, draw a card.
**Current state**: TODO -- says "SelfCostReduction lacks spell-type filter" but `SpellCostFilter::InstantOrSorcery` already exists
**Fix**: Add `spell_cost_modifiers: vec![SpellCostModifier { change: -1, filter: SpellCostFilter::InstantOrSorcery, scope: CostModifierScope::Controller, eminence: false, exclude_self: false }]`, remove TODO. This is identical to baral_chief_of_compliance.rs pattern.

### bontus_monument.rs
**Oracle text**: Black creature spells you cast cost {1} less to cast. Whenever you cast a creature spell, each opponent loses 1 life and you gain 1 life.
**Current state**: TODO -- "color+type cost reduction not in DSL"
**Fix**: Add `spell_cost_modifiers: vec![SpellCostModifier { change: -1, filter: SpellCostFilter::ColorAndCreature(Color::Black), scope: CostModifierScope::Controller, eminence: false, exclude_self: false }]`, remove TODO.

### hazorets_monument.rs
**Oracle text**: Red creature spells you cast cost {1} less to cast. Whenever you cast a creature spell, you may discard a card. If you do, draw a card.
**Current state**: TODO -- "color+type cost reduction not in DSL"
**Fix**: Add `spell_cost_modifiers: vec![SpellCostModifier { change: -1, filter: SpellCostFilter::ColorAndCreature(Color::Red), scope: CostModifierScope::Controller, eminence: false, exclude_self: false }]`, remove TODO. Note: the triggered "may discard, if you do draw" remains a separate TODO (optional discard DSL gap -- not PB-29 scope).

### oketras_monument.rs
**Oracle text**: White creature spells you cast cost {1} less to cast. Whenever you cast a creature spell, create a 1/1 white Warrior creature token with vigilance.
**Current state**: TODO -- "color+type cost reduction not in DSL"
**Fix**: Add `spell_cost_modifiers: vec![SpellCostModifier { change: -1, filter: SpellCostFilter::ColorAndCreature(Color::White), scope: CostModifierScope::Controller, eminence: false, exclude_self: false }]`, remove TODO.

### urzas_incubator.rs
**Oracle text**: As this artifact enters, choose a creature type. Creature spells of the chosen type cost {2} less to cast.
**Current state**: TODO -- "SpellCostFilter::HasChosenSubtype -- no variant exists"
**Fix**: Add `spell_cost_modifiers: vec![SpellCostModifier { change: -2, filter: SpellCostFilter::HasChosenCreatureSubtype, scope: CostModifierScope::AllPlayers, eminence: false, exclude_self: false }]`, remove TODO. Note: scope is AllPlayers per oracle text ("cost {2} less" with no "you cast" qualifier).

### winged_words.rs
**Oracle text**: This spell costs {1} less to cast if you control a creature with flying. Draw two cards.
**Current state**: TODO -- "Conditional cost reduction not expressible"
**Fix**: Add `self_cost_reduction: Some(SelfCostReduction::ConditionalKeyword { keyword: KeywordAbility::Flying, reduction: 1 })`, remove TODO.

### cavern_hoard_dragon.rs
**Oracle text**: This spell costs {X} less to cast, where X is the greatest number of artifacts an opponent controls. Flying, trample, haste. Whenever this creature deals combat damage to a player, you create a Treasure token for each artifact that player controls.
**Current state**: TODO -- "dynamic cost reduction based on opponent artifact count not expressible"
**Fix**: Add `self_cost_reduction: Some(SelfCostReduction::MaxOpponentPermanents { filter: TargetFilter { has_card_type: Some(CardType::Artifact), ..Default::default() }, per: 1 })`, remove the cost reduction TODO. The combat damage trigger TODO remains (separate DSL gap -- G-8/PB-30).

### boseiju_who_endures.rs
**Oracle text**: {T}: Add {G}. Channel -- {1}{G}, Discard this card: Destroy target [...]. This ability costs {1} less to activate for each legendary creature you control.
**Current state**: TODO -- "Cost reduction per legendary creature"
**Fix**: Add `self_cost_reduction: Some(SelfActivatedCostReduction::PerPermanent { per: 1, filter: TargetFilter { legendary: true, has_card_type: Some(CardType::Creature), ..Default::default() }, controller: PlayerTarget::Controller })` to the channel Activated ability. Remove cost reduction TODO. (Other TODOs for target filter remain.)

### otawara_soaring_city.rs
**Oracle text**: {T}: Add {U}. Channel -- {3}{U}, Discard this card: Return target [...]. This ability costs {1} less to activate for each legendary creature you control.
**Current state**: TODO -- "Cost reduction per legendary creature"
**Fix**: Same pattern as boseiju_who_endures. Add `self_cost_reduction` to the channel Activated ability.

### eiganjo_seat_of_the_empire.rs
**Oracle text**: {T}: Add {W}. Channel -- {2}{W}, Discard this card: 4 damage to target [...]. This ability costs {1} less to activate for each legendary creature you control.
**Current state**: TODO -- "Cost reduction per legendary creature"
**Fix**: Same pattern as boseiju_who_endures.

### takenuma_abandoned_mire.rs
**Oracle text**: {T}: Add {B}. Channel -- {3}{B}, Discard this card: Mill 3, return creature/PW from GY. This ability costs {1} less to activate for each legendary creature you control.
**Current state**: TODO -- "Cost reduction per legendary creature"
**Fix**: Same pattern as boseiju_who_endures.

### sokenzan_crucible_of_defiance.rs
**Oracle text**: {T}: Add {R}. Channel -- {3}{R}, Discard this card: Create two 1/1 Spirit tokens with haste. This ability costs {1} less to activate for each legendary creature you control.
**Current state**: TODO -- "Cost reduction per legendary creature"
**Fix**: Same pattern as boseiju_who_endures.

### voldaren_estate.rs
**Oracle text**: {5}, {T}: Create a Blood token. This ability costs {1} less to activate for each Vampire you control.
**Current state**: TODO -- "Cost reduction per Vampire not expressible"
**Fix**: Add `self_cost_reduction: Some(SelfActivatedCostReduction::PerPermanent { per: 1, filter: TargetFilter { has_subtype: Some(SubType("Vampire".to_string())), ..Default::default() }, controller: PlayerTarget::Controller })` to the Blood token Activated ability. Remove TODO.

## New Card Definitions

None. Card authoring is paused at A-28. Cards like Herald's Horn, Cloud Key, Kefnet's Monument, and Rhonas's Monument are not yet authored and will be picked up during post-gap authoring.

## Unit Tests

**File**: `crates/engine/tests/cost_reduction.rs` (extend existing file if it exists, otherwise create new)

Check for existing test file first:
```
Glob pattern="crates/engine/tests/*cost*"
```

If no existing file, also check for tests in `crates/engine/tests/card_def_fixes.rs` or similar that test spell_cost_modifiers.

**Tests to write**:

- `test_spell_cost_filter_color_and_creature_reduces_matching` -- CR 601.2f: Black creature spell costs {1} less with Bontu's Monument on battlefield. Verify non-creature black spells are NOT reduced.
- `test_spell_cost_filter_color_and_creature_no_match_noncreature` -- CR 601.2f: Verify a black instant is not reduced by ColorAndCreature(Black).
- `test_spell_cost_filter_chosen_creature_subtype` -- CR 601.2f: Urza's Incubator with chosen_creature_type "Goblin" reduces Goblin creature spells by {2}. Non-Goblin creatures not reduced.
- `test_spell_cost_filter_chosen_subtype_all_players` -- Verify Urza's Incubator reduces for opponents too (AllPlayers scope).
- `test_self_cost_reduction_conditional_keyword` -- CR 601.2f: Winged Words costs {1} less when controller has creature with flying on battlefield.
- `test_self_cost_reduction_conditional_keyword_no_match` -- Winged Words full cost when no creature with flying present.
- `test_self_cost_reduction_max_opponent_permanents` -- CR 601.2f: Cavern-Hoard Dragon costs less by greatest artifact count among opponents. 1v1 with opponent having 3 artifacts = costs {3} less.
- `test_self_cost_reduction_max_opponent_permanents_multiplayer` -- 4-player: opponents have 1, 3, 5 artifacts; verify max (5) is used.
- `test_activated_ability_self_cost_reduction_per_legendary` -- CR 602.2b: Channel ability costs {1} less per legendary creature controller has. 2 legendary creatures = {2} less.
- `test_activated_ability_self_cost_reduction_floor_zero` -- CR 601.2f: Reduction cannot bring generic cost below {0}. 10 legendary creatures but base cost is only {3} generic = cost is {0} generic (colored mana still required).
- `test_activated_ability_self_cost_reduction_voldaren_estate` -- Vampires reduce Blood token creation cost. 3 vampires = {5} - {3} = {2} generic.

**Pattern**: Follow existing tests in `crates/engine/tests/`. Use `GameStateBuilder` to set up battlefield with the source permanent and relevant creatures/artifacts, then attempt casting/activation and verify mana pool consumption.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved (13 card files)
- [ ] New card defs authored (if any) -- N/A
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining cost-reduction TODOs in affected card defs (except deferred: Heartstone, Training Grounds, Silver-Fur Master, Puresteel Paladin, Morophon)

## Risks & Edge Cases

- **`AbilityDefinition::Activated` field addition is high-risk for compile errors.** This is an enum variant with named fields -- every match arm and construction site must be updated. The runner MUST grep for ALL `AbilityDefinition::Activated` occurrences across the entire workspace (including tools/) and update each one. This is the #1 risk. Estimate: 50-100+ sites. Consider whether a simpler approach (e.g., putting self_cost_reduction on CardDefinition keyed by ability index, similar to spell_cost_modifiers) would avoid the enum variant change entirely.
- **Alternative design to avoid enum variant change**: Instead of adding a field to `AbilityDefinition::Activated`, add a new `CardDefinition` field `activated_ability_cost_reductions: Vec<(usize, SelfActivatedCostReduction)>` keyed by ability index. This avoids touching every `AbilityDefinition::Activated` match arm. The tradeoff: less discoverable, index-based coupling. **Recommended if the runner finds >80 match sites to update.**
- **`HasChosenCreatureSubtype` requires source-object context in filter matching.** The standard `spell_matches_cost_filter()` doesn't have the source object. The inline special-case in `apply_spell_cost_modifiers()` is necessary. If refactored later, the source context must be threaded through.
- **Channel ability index mapping.** `get_self_activated_cost_reduction` must correctly count only `Activated` variants to map `ability_index` to the right card definition ability. The activated ability index on the game object corresponds to the Nth `Activated` ability in the card definition's abilities vec. Verify this mapping is correct for cards with mixed ability types (e.g., channel lands have both a mana tap Activated and a channel Activated -- but mana abilities are NOT in `activated_abilities`, they're in `mana_abilities`). The mana tap ability goes through `enrich_spec_from_def` into `mana_abilities`, NOT `activated_abilities`. So the channel ability should be ability_index=0 for channel lands (it's the only non-mana activated ability).
- **`matches_filter` visibility.** `crate::effects::matches_filter` is already `pub` (line 4991 of effects/mod.rs). No visibility change needed.
- **SelfCostReduction::ConditionalKeyword should ideally use layer-resolved keywords.** The creature with flying check should ideally use `calculate_characteristics()` to handle cases where flying is granted by a continuous effect. However, the existing SelfCostReduction variants use `obj.characteristics.keywords` (base characteristics), so follow that pattern for consistency. Note this as a known approximation (LOW priority).
- **Floor at zero.** CR 601.2f: "It can't be reduced to less than {0}." The existing `apply_self_cost_reduction` handles this with `.max(0)`. The activated ability reduction must use `.saturating_sub()` to ensure the same floor. Only generic mana is reduced -- colored mana requirements are never reduced by these effects (except Morophon, which is deferred).
- **TargetFilter `legendary` field.** New field with `#[serde(default)]` -- defaults to `false`, so all existing TargetFilter constructions using `..Default::default()` are unaffected. Only explicit constructions without `..Default::default()` need updating. Grep for `TargetFilter {` constructions without `..Default::default()` to check.
