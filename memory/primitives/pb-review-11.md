# Primitive Batch Review: PB-11 -- Mana Spending Restrictions + ETB Player Choice

**Date**: 2026-03-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 106.1a (five colors of mana), CR 106.3 (mana production), CR 106.6 (spending restrictions), CR 400.7 (new object identity on zone change)
**Engine files reviewed**: `card_definition.rs` (ManaRestriction enum, AddManaRestricted/AddManaAnyColorRestricted/ChooseCreatureType Effect variants), `game_object.rs` (chosen_creature_type field), `player.rs` (RestrictedMana, SpellContext, ManaPool restricted methods, restriction_matches), `effects/mod.rs` (AddManaRestricted/AddManaAnyColorRestricted/ChooseCreatureType handlers, resolve_mana_restriction, add_mana_with_restriction), `casting.rs` (can_pay_cost_with_context, pay_cost_with_context), `hash.rs` (ManaRestriction, RestrictedMana, ManaPool, chosen_creature_type), `replacement.rs` (ChooseCreatureType replacement modification), `replacement_effect.rs` (ChooseCreatureType variant), `builder.rs` (chosen_creature_type init), `resolution.rs` (chosen_creature_type init on new objects), `state/mod.rs` (chosen_creature_type reset on zone change), `helpers.rs` (ManaRestriction export)
**Card defs reviewed**: 10 (cavern_of_souls, secluded_courtyard, unclaimed_territory, haven_of_the_spirit_dragon, gnarlroot_trapper, the_seedcore, voldaren_estate, etchings_of_the_chosen, three_tree_city, maelstrom_of_the_spirit_dragon)

## Verdict: needs-fix

One HIGH finding: the restricted mana infrastructure (SpellContext, can_pay_cost_with_context, pay_cost_with_context) is completely disconnected from the actual spell casting flow. The main `handle_cast_spell` path calls `can_pay_cost` and `pay_cost` (both pass `None` context), meaning restricted mana is never considered during actual gameplay -- players with only restricted mana from Cavern of Souls will be told they have insufficient mana. Three MEDIUM findings: (1) `AddManaAnyColorRestricted` adds colorless instead of a color (inherited from unrestricted `AddManaAnyColor` limitation but more impactful here since the restricted mana can only be spent via the disconnected `SpellContext` path); (2) `ChosenTypeCreaturesOnly` resolves to `SubtypeOnly` losing the "creature spell" requirement; (3) Haven, Seedcore, and Gnarlroot use `SubtypeOnly` instead of a combined creature+subtype restriction. Several card defs have expected TODOs for documented DSL gaps.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `casting.rs:3305,3308` | **Restricted mana never used in actual casting flow.** `handle_cast_spell` calls `can_pay_cost` and `pay_cost` which both pass `None` SpellContext. `SpellContext` is never constructed anywhere in the rules engine. **Fix:** At the `can_pay_cost` call site (line 3305), build a `SpellContext` from the spell being cast (check `card_types.contains(Creature)` for `is_creature`, extract `subtypes` from characteristics), then call `can_pay_cost_with_context` and `pay_cost_with_context` with it. |
| 2 | **MEDIUM** | `effects/mod.rs:1071` | **AddManaAnyColorRestricted adds colorless, not a color.** Oracle: "Add one mana of any color" means one of {W,U,B,R,G} per CR 106.1a. Handler adds `ManaColor::Colorless`. Same limitation as unrestricted `AddManaAnyColor` (line 1000), but more impactful here since the restricted mana must match spell context. **Fix:** Use the same deterministic color-choice heuristic planned for `AddManaAnyColor` when it is fixed. Document as known limitation until interactive color choice is implemented. |
| 3 | **MEDIUM** | `effects/mod.rs:3259-3267` | **ChosenTypeCreaturesOnly resolves to SubtypeOnly, losing creature requirement.** `resolve_mana_restriction` maps `ChosenTypeCreaturesOnly` to `SubtypeOnly(chosen)`. The oracle text for Cavern of Souls says "creature spell of the chosen type" -- a tribal Dragon instant would incorrectly match. **Fix:** Add a `CreatureWithSubtype(SubType)` variant to `ManaRestriction` that checks `spell.is_creature && spell.subtypes.contains(st)`. Resolve `ChosenTypeCreaturesOnly` to `CreatureWithSubtype(chosen)` instead of `SubtypeOnly(chosen)`. |
| 4 | LOW | `effects/mod.rs:1932-1955` | **ChooseCreatureType uses HashMap for type counting.** Non-deterministic iteration order. While the result is deterministic (max_by_key picks the most common), ties between equally-common types resolve non-deterministically. Not a correctness issue in practice but technically non-deterministic. **Fix:** Use `BTreeMap` or sort before selecting, or document that tie-breaking is arbitrary. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 5 | **MEDIUM** | `haven_of_the_spirit_dragon.rs:34` | **SubtypeOnly(Dragon) should be creature+subtype.** Oracle: "Spend this mana only to cast a **Dragon creature** spell." `SubtypeOnly(Dragon)` allows non-creature Dragon spells. **Fix:** Use `CreatureWithSubtype(Dragon)` (after engine variant is added per Finding 3). |
| 6 | **MEDIUM** | `the_seedcore.rs:31` | **SubtypeOnly(Phyrexian) should be creature+subtype.** Oracle: "Spend this mana only to cast **Phyrexian creature** spells." Same issue as Finding 5. **Fix:** Use `CreatureWithSubtype(Phyrexian)`. |
| 7 | **MEDIUM** | `gnarlroot_trapper.rs:24` | **SubtypeOnly(Elf) should be creature+subtype.** Oracle: "Spend this mana only to cast an **Elf creature** spell." Same issue. **Fix:** Use `CreatureWithSubtype(Elf)`. |
| 8 | **MEDIUM** | `gnarlroot_trapper.rs:20` | **Pay 1 life cost missing.** Oracle: "{T}, **Pay 1 life**: Add {G}." Def uses `Cost::Tap` only. Wrong game state: no life payment required. **Fix:** Add `Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)])` when `Cost::PayLife` variant exists. Document as known DSL gap until then. |
| 9 | **MEDIUM** | `voldaren_estate.rs:29` | **Pay 1 life cost missing.** Oracle: "{T}, **Pay 1 life**: Add one mana of any color." Same as Finding 8. **Fix:** Same approach. |
| 10 | LOW | `cavern_of_souls.rs:46-48` | **"Can't be countered" rider TODO.** Oracle: "and that spell can't be countered." Documented as future primitive. Acceptable deferral. |
| 11 | LOW | `secluded_courtyard.rs:36-37` | **Ability activation spending TODO.** Oracle includes "or activate an ability of a creature source of the chosen type." Documented as not enforced. Acceptable -- mana restrictions on ability activation require a different enforcement site. |
| 12 | LOW | `etchings_of_the_chosen.rs:25-30` | **Two TODOs: static +1/+1 and sacrifice ability.** Requires `EffectFilter::ChosenSubtype` and `Cost::SacrificeWithFilter`. Documented DSL gaps. |
| 13 | LOW | `three_tree_city.rs:35-38` | **Count-based mana scaling TODO.** Requires `EffectAmount::CreaturesOfChosenType` + color choice. Documented DSL gap. |
| 14 | LOW | `voldaren_estate.rs:37-38` | **Blood token creation with cost reduction TODO.** Requires variable cost reduction. Documented DSL gap. |
| 15 | LOW | `the_seedcore.rs:36-38` | **Corrupted activated ability TODO.** Requires conditional activation + targeting. Documented DSL gap. |
| 16 | LOW | `haven_of_the_spirit_dragon.rs:7-8,40` | **"or Ugin planeswalker card" not targetable.** Requires name+type union filter. Documented DSL gap. Carried forward from PB-10. |

### Finding Details

#### Finding 1: Restricted mana never used in actual casting flow

**Severity**: HIGH
**File**: `crates/engine/src/rules/casting.rs:3305,3308`
**CR Rule**: 106.6 -- "Some spells or abilities that produce mana restrict how that mana can be spent, have an additional effect that affects the spell or ability that mana is spent on, or create a delayed triggered ability that triggers when that mana is spent."
**Issue**: The PB-11 implementation added the full restricted mana infrastructure: `ManaRestriction` enum, `RestrictedMana` struct, `SpellContext`, `can_pay_cost_with_context`, `pay_cost_with_context`, `ManaPool.add_restricted/restricted_available/spend_restricted`. However, the actual spell casting path in `handle_cast_spell` (line 3305) calls `can_pay_cost(&player_state.mana_pool, &flat_cost)` which internally calls `can_pay_cost_with_context(pool, cost, None)` -- the `None` means restricted mana is completely ignored. Similarly, `pay_cost` at line 3308 calls `pay_cost_with_context(pool, cost, None)`. No code anywhere in the rules engine constructs a `SpellContext` during actual gameplay. The result: restricted mana is added to the pool by `AddManaRestricted`/`AddManaAnyColorRestricted` effects, but is invisible to the casting system. A player relying on Cavern of Souls restricted mana as their only source cannot cast any spell.
**Fix**: In `handle_cast_spell`, before the `can_pay_cost` call at line 3305, construct a `SpellContext` from the spell being cast:
```rust
let spell_context = SpellContext {
    is_creature: card_chars.card_types.contains(&CardType::Creature),
    subtypes: card_chars.subtypes.iter().cloned().collect(),
};
```
Then replace `can_pay_cost(&player_state.mana_pool, &flat_cost)` with `can_pay_cost_with_context(&player_state.mana_pool, &flat_cost, Some(&spell_context))` and `pay_cost(...)` with `pay_cost_with_context(...)`. Also do the same for activated ability mana payment sites, and the `mana_solver` in `crates/simulator/`.

#### Finding 3: ChosenTypeCreaturesOnly resolves to SubtypeOnly

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:3259-3267`
**CR Rule**: 106.6 -- Mana spending restrictions
**Oracle**: Cavern of Souls: "Spend this mana only to cast a **creature spell** of the chosen type"
**Issue**: `resolve_mana_restriction` converts `ChosenTypeCreaturesOnly` to `SubtypeOnly(chosen_type)`. The `SubtypeOnly` restriction in `restriction_matches` checks `spell.subtypes.iter().any(|s| s == st)` but does NOT check `spell.is_creature`. This means a non-creature spell with the chosen subtype (e.g., a tribal Dragon instant) would be payable with Cavern of Souls mana, violating the "creature spell" part of the oracle text. The same `ChosenTypeSpellsOnly` variant correctly maps to `SubtypeOnly` since it has no creature requirement.
**Fix**: Add `CreatureWithSubtype(SubType)` variant to `ManaRestriction` (plus hash support). In `restriction_matches`, check `spell.is_creature && spell.subtypes.iter().any(|s| s == st)`. Update `resolve_mana_restriction` to map `ChosenTypeCreaturesOnly` to `CreatureWithSubtype(chosen)`. Also update Haven, Seedcore, and Gnarlroot to use this variant (Findings 5-7).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 106.6 (spending restrictions) | Partial | Partial | Infrastructure exists but not wired into casting flow (Finding 1) |
| CR 106.1a (five colors) | No | No | AddManaAnyColorRestricted adds colorless, not a color (Finding 2) |
| CR 400.7 (zone change resets) | Yes | Yes | chosen_creature_type reset in move_object_to_zone |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Cavern of Souls | Partial | 1 | No | Restricted mana not usable (F1); can't-be-countered TODO (F10) |
| Secluded Courtyard | Partial | 0 (comment) | No | Restricted mana not usable (F1); ability activation restriction noted |
| Unclaimed Territory | Yes | 0 | No | Restricted mana not usable (F1); otherwise correct |
| Haven of the Spirit Dragon | Partial | 1 | No | SubtypeOnly should be creature+subtype (F5); Ugin target TODO (F16) |
| Gnarlroot Trapper | Partial | 1 | No | Missing Pay 1 life cost (F8); SubtypeOnly wrong (F7) |
| The Seedcore | Partial | 1 | No | SubtypeOnly wrong (F6); Corrupted ability TODO (F15) |
| Voldaren Estate | Partial | 1 | No | Missing Pay 1 life cost (F9); Blood token TODO (F14) |
| Etchings of the Chosen | Partial | 2 | No | Static +1/+1 and sacrifice TODO (F12) |
| Three Tree City | Partial | 1 | No | Count-based mana scaling TODO (F13) |
| Maelstrom of the Spirit Dragon | Yes | 0 | No | SubtypeOrSubtype(Dragon, Omen) correct; restricted mana not usable (F1) |

## Test Coverage

The test file `mana_restriction.rs` has 10 tests covering:
- Positive: restricted mana available for matching creature spell
- Negative: restricted mana unavailable for non-matching spell
- SubtypeOnly matching: Dragon matches, Elf rejected
- SubtypeOrSubtype matching: Dragon OR Omen match, Elf rejected
- spend_restricted: correct deduction from pool
- add_restricted: merges same-color same-restriction entries
- empty(): clears restricted mana
- ChooseCreatureType effect: picks most common creature type on battlefield
- ChooseCreatureType fallback: defaults when no creatures controlled
- AddManaRestricted: produces restricted mana in pool
- can_pay_cost_with_context: includes matching restricted mana
- pay_cost_with_context: spends restricted first

**Missing test coverage:**
- No integration test that casts an actual spell using restricted mana (would catch Finding 1)
- No test for non-creature spell matching `ChosenTypeCreaturesOnly` (would catch Finding 3)
- No test for `AddManaAnyColorRestricted` producing actual color (would catch Finding 2)
- No card integration test using Cavern of Souls end-to-end
