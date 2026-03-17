# Primitive Batch Review: PB-7 -- Count-Based Scaling

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**Mode**: retroactive review (no plan file)
**CR Rules**: 700.5 (devotion), 702.26d (phasing exclusion), 601.2f (cost reduction)
**Engine files reviewed**: `cards/card_definition.rs` (EffectAmount enum), `effects/mod.rs` (resolve_amount dispatch), `state/hash.rs` (hash support), `rules/casting.rs` (SelfCostReduction)
**Card defs reviewed**: 29

## Verdict: needs-fix

Engine changes (PermanentCount, DevotionTo, CounterCount, AddManaScaled, SelfCostReduction)
are structurally sound with good hash support and correct dispatch. Tests cover positive and
negative cases for all three EffectAmount variants. However, DevotionTo ignores hybrid and
phyrexian mana symbols (CR 700.5 violation), Nykthos has an incorrect subtype, Faeburrow
Elder and Multani use wrong P/T representation, and Frodo's oracle text is completely wrong.
Most of the 29 cards correctly note DSL gaps as TODOs; 5 cards actively use the new
primitives and work correctly. Many remaining TODOs are genuinely blocked on missing DSL
features (dynamic P/T CDA, land animation, color choice, etc.).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `effects/mod.rs:3398-3409` | **DevotionTo ignores hybrid/phyrexian mana.** CR 700.5 counts ALL mana symbols of a color, including hybrid and phyrexian. **Fix:** see Finding 1. |
| 2 | **LOW** | `effects/mod.rs:3381` | **PermanentCount reads raw characteristics.** Uses `obj.characteristics` instead of `calculate_characteristics()`. Systemic pattern, not PB-7-specific. |
| 3 | **LOW** | `effects/mod.rs:3399` | **DevotionTo reads raw characteristics.** Same issue as #2 for mana costs. CR 700.5a says copy/control/text-changing effects should be considered. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 4 | **HIGH** | `nykthos_shrine_to_nyx.rs:16` | **Wrong subtype.** Oracle type is "Legendary Land" (no subtypes). Code adds "Shrine" subtype. **Fix:** see Finding 4. |
| 5 | **HIGH** | `faeburrow_elder.rs:26-27` | **Wrong P/T representation.** Oracle P/T is 0/0, code uses `None/None` (*/*). **Fix:** see Finding 5. |
| 6 | **HIGH** | `multani_yavimayas_avatar.rs:21` | **Wrong P/T representation.** Oracle P/T is 0/0, code uses `None/None` (*/*). **Fix:** see Finding 6. |
| 7 | **MEDIUM** | `frodo_saurons_bane.rs:38-54` | **Oracle text completely wrong.** Card def describes different abilities than actual oracle text. **Fix:** see Finding 7. |
| 8 | **LOW** | `toothy_imaginary_friend.rs:40-46` | **Explicit struct construction.** Uses explicit fields instead of `..Default::default()`. Should be migrated per PB-9.5 convention. |

### Finding Details

#### Finding 1: DevotionTo ignores hybrid and phyrexian mana symbols

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:3398-3409`
**CR Rule**: 700.5 -- "A player's devotion to [color] is equal to the number of mana symbols of that color among the mana costs of permanents that player controls."
**Oracle (Nykthos ruling)**: "Hybrid mana symbols, monocolored hybrid mana symbols, and Phyrexian mana symbols do count toward your devotion to their color(s)."
**Issue**: The DevotionTo implementation only counts the basic color fields (`mc.white`, `mc.blue`, etc.) on ManaCost. It does not iterate `mc.hybrid` or `mc.phyrexian` vectors. A permanent with mana cost `{G/W}{G/W}` (two hybrid green/white pips) would contribute 0 to green devotion and 0 to white devotion, when it should contribute 2 to each.
**Fix**: After counting the basic color fields, also iterate `mc.hybrid` and `mc.phyrexian`:
- `HybridMana::ColorColor(c1, c2)` -- add 1 if either `c1` or `c2` matches the queried color
- `HybridMana::GenericColor(c)` -- add 1 if `c` matches
- `PhyrexianMana::Single(c)` -- add 1 if `c` matches
- `PhyrexianMana::Hybrid(c1, c2)` -- add 1 if either matches

Add a test with a permanent that has hybrid mana in its cost.

#### Finding 4: Nykthos, Shrine to Nyx has incorrect Shrine subtype

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/nykthos_shrine_to_nyx.rs:16`
**Oracle**: Type line is "Legendary Land" -- no subtypes. The word "Shrine" in the card name is flavor, not a subtype.
**Current**: `full_types(&[SuperType::Legendary], &[CardType::Land], &["Shrine"])`
**Fix**: Change to `supertypes(&[SuperType::Legendary], &[CardType::Land])` (no subtypes). This is the same pattern used in the `gaeas_cradle.rs` def.

#### Finding 5: Faeburrow Elder uses None P/T instead of 0/0

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/faeburrow_elder.rs:26-27`
**Oracle**: P/T is 0/0. The ability says "gets +1/+1 for each color" -- this is a static continuous effect, NOT a characteristic-defining ability. Base P/T is 0/0.
**Current**: `power: None, toughness: None` with comment `// */* CDA`
**Issue**: Using `None` makes the engine skip SBA 704.5f (lethal toughness check). If no colored permanents are controlled, Faeburrow Elder should have 0 toughness and die to SBA. With `None`, it survives illegally.
**Fix**: Change to `power: Some(0), toughness: Some(0)`. Remove the `// */* CDA` comment.

#### Finding 6: Multani uses None P/T instead of 0/0

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/multani_yavimayas_avatar.rs:21`
**Oracle**: P/T is 0/0. The ability says "gets +1/+1 for each land" -- static continuous effect, not a CDA. Base P/T is 0/0.
**Current**: `power: None, toughness: None` with comment `// */* CDA`
**Issue**: Same as Finding 5. Without lands, Multani should die to SBA. With `None`, it survives.
**Fix**: Change to `power: Some(0), toughness: Some(0)`. Remove the CDA comment.

#### Finding 7: Frodo, Sauron's Bane oracle text is completely wrong

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/frodo_saurons_bane.rs:38-54`
**Oracle**: Actual abilities are:
1. `{W/B}{W/B}`: If Frodo is a Citizen, it becomes a Halfling Scout with base power and toughness 2/3 and lifelink.
2. `{B}{B}{B}`: If Frodo is a Scout, it becomes a Halfling Rogue with "Whenever this creature deals combat damage to a player, that player loses the game if the Ring has tempted you four or more times this game. Otherwise, the Ring tempts you."

**Current**: Oracle text string says `{B}{B}{B}` makes it a Halfling Scout with ring temptation, and `{1}{B}{B}` makes it legendary 2/3 with menace/indestructible. These are completely different abilities.
**Issue**: The oracle_text field does not match the card's actual oracle text. While both abilities are deferred as TODOs (the DSL can't express them), the oracle text string is documentation and should be accurate for future implementers. The wrong oracle text would lead to wrong implementation when the DSL gaps are filled.
**Fix**: Replace the oracle_text string with the actual oracle text from Scryfall. Update the TODO comments to accurately describe the two abilities.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 700.5 (devotion) | Partial | Yes (3 tests) | Missing hybrid/phyrexian symbol counting (Finding 1) |
| 700.5a (devotion timing) | No | No | Layer interaction (copy effects) -- systemic issue, not PB-7-specific |
| 702.26d (phasing exclusion) | Yes | No | PermanentCount and DevotionTo both check `is_phased_in()` |
| 601.2f (cost reduction) | Yes | Yes (5 tests) | SelfCostReduction variants all tested |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| blasphemous_act | Yes | 0 | Yes | Cost reduction + damage both working |
| cabal_coffers | Yes | 0 | Yes | PermanentCount with Swamp filter correct |
| cabal_stronghold | Yes | 0 | Yes | Basic Swamp filter correct |
| crypt_of_agadeem | Yes | 0 | Yes | CardCount with graveyard black creature filter |
| gaeas_cradle | Yes | 0 | Yes | PermanentCount creature filter correct |
| malakir_bloodwitch | Yes | 0 | Yes | DrainLife with Vampire count correct |
| nykthos_shrine_to_nyx | **No** | 1 | No | **Finding 4**: wrong Shrine subtype; devotion ability deferred (color choice) |
| craterhoof_behemoth | Yes | 1 | No | ETB mass pump deferred (dynamic LayerModification) |
| reckless_one | Yes | 1 | No | CDA P/T deferred (SetPowerToughness with EffectAmount) |
| faeburrow_elder | **No** | 2 | **No** | **Finding 5**: P/T should be 0/0 not None |
| multani_yavimayas_avatar | **No** | 2 | **No** | **Finding 6**: P/T should be 0/0 not None |
| frodo_saurons_bane | **No** | 2 | No | **Finding 7**: oracle text completely wrong |
| balan_wandering_knight | Yes | 2 | No | Both abilities deferred (count threshold + mass equip) |
| call_of_the_ring | Yes | 1 | Partial | Upkeep ring-tempt works; ring-bearer trigger deferred |
| crimestopper_sprite | Yes | 1 | Partial | Stun counter TODO (CounterType::Stun missing) |
| crown_of_skemfar | Yes | 2 | Partial | Reach grant works; count-based P/T boost + graveyard return deferred |
| destiny_spinner | Yes | 2 | No | Both abilities deferred |
| devilish_valet | Yes | 1 | Partial | Keywords work; Alliance power-double deferred |
| emrakul_the_promised_end | Yes | 2 | Partial | Cost reduction works; protection + cast trigger deferred |
| eomer_king_of_rohan | Yes | 2 | No | Both ETB abilities deferred |
| ghalta_primal_hunger | Yes | 0 | Yes | Cost reduction works |
| indomitable_archangel | Yes | 1 | Partial | Flying works; Metalcraft shroud grant deferred |
| jagged_scar_archers | Yes | 2 | No | CDA P/T + activated ability deferred |
| molimo_maro_sorcerer | Yes | 1 | No | CDA P/T deferred (correctly uses None for */*) |
| scion_of_draco | Yes | 1 | Partial | Cost reduction works; color-conditional grants deferred |
| scryb_ranger | Yes | 1 | Partial | Keywords work; activated ability deferred |
| the_ur_dragon | Yes | 1 | Partial | Cost modifier works; attack trigger deferred |
| three_tree_city | Yes | 1 | Partial | Type choice + colorless mana work; count mana ability deferred |
| tombstone_stairwell | Yes | 2 | Partial | CumulativeUpkeep works; token creation/destruction deferred |
| toothy_imaginary_friend | Yes | 2 | Partial | PartnerWith works; draw trigger + LTB trigger deferred |
| war_room | Yes | 1 | Partial | Colorless mana works; commander-identity-cost draw deferred |
| earthquake_dragon | Yes | 1 | Partial | Cost reduction works; graveyard return deferred |

## Test Coverage

Tests in `crates/engine/tests/count_based_scaling.rs` (7 tests):
- PermanentCount: controller scoping, opponent scoping, land filter -- all good
- DevotionTo: positive count, zero-when-no-symbols, excludes-no-mana-cost -- good
- CounterCount: positive count, zero-when-no-counters -- good

Tests in `crates/engine/tests/spell_cost_modification.rs` (5+ relevant tests):
- SelfCostReduction::PerPermanent (Blasphemous Act style) -- tested
- SelfCostReduction::TotalPowerOfCreatures (Ghalta style) -- tested
- SelfCostReduction::CardTypesInGraveyard (Emrakul style) -- tested
- SelfCostReduction::BasicLandTypes (Domain/Scion style) -- tested
- SelfCostReduction::TotalManaValue (Earthquake Dragon style) -- tested

**Missing test**: DevotionTo with hybrid/phyrexian mana symbols (blocked by Finding 1).

## Summary

- **HIGH**: 3 (Nykthos wrong subtype, Faeburrow wrong P/T, Multani wrong P/T)
- **MEDIUM**: 2 (DevotionTo hybrid/phyrexian gap, Frodo wrong oracle text)
- **LOW**: 3 (raw characteristics reads x2, explicit struct in Toothy)
