# Primitive Batch Review: PB-6 -- Static Grant with Controller Filter

**Date**: 2026-03-16
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 604.2 (static abilities), CR 613.1f (Layer 6 ability-granting), CR 613.1g (Layer 7 P/T)
**Engine files reviewed**: `state/continuous_effect.rs` (EffectFilter enum), `rules/layers.rs` (filter matching), `state/hash.rs` (hash support)
**Card defs reviewed**: 30 card def files (all from DSL gap audit `static_grant_conditional` list, plus fervor.rs)

## Verdict: needs-fix

The engine changes are correct and well-tested. The three new `EffectFilter` variants
(`CreaturesYouControl`, `OtherCreaturesYouControl`, `OtherCreaturesYouControlWithSubtype`)
are properly implemented in layers.rs with correct source-controller resolution, zone checks,
creature-type checks, and self-exclusion logic. Hash support is present. Six unit tests cover
positive, negative, cross-controller, no-source, and subtype-filtering cases.

However, only 8 of the 30 cards in the batch were actually fixed to use the new primitives.
The remaining 22 cards still have empty abilities or stale TODOs claiming the filter doesn't
exist. Of those 22, at least 5 cards have abilities that ARE now implementable with the
existing PB-6 filters but were never updated. Additionally, goblin_warchief.rs uses the
wrong filter variant (`OtherCreaturesYouControlWithSubtype` instead of a self-inclusive
variant matching oracle text "Goblins you control have haste").

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `layers.rs:619` | **Comment typo.** Line starts with `/ CR 604.2` (missing leading `/` for doc comment). Same on lines 634 and 652. **Fix:** Change `/ CR` to `// CR` on lines 619, 634, and 652. |

### Finding Details

#### Finding 1: Comment typo in layers.rs

**Severity**: LOW
**File**: `crates/engine/src/rules/layers.rs:619`
**CR Rule**: N/A (style)
**Issue**: Three comment lines at 619, 634, and 652 start with `/ CR` instead of `// CR`. These appear to be accidental truncation of the `//` prefix.
**Fix**: Change `/ CR` to `// CR` on all three lines.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | **HIGH** | `goblin_warchief.rs` | **Wrong filter: oracle says "Goblins you control" (includes self), def uses OtherCreaturesYouControlWithSubtype (excludes self).** Goblin Warchief is a Goblin and should grant itself haste. **Fix:** Change filter to a self-inclusive variant. Since no `CreaturesYouControlWithSubtype` variant exists, add one to EffectFilter OR use two abilities: `Keyword(Haste)` on self + `OtherCreaturesYouControlWithSubtype` for others (functionally equivalent). |
| 3 | MEDIUM | `archetype_of_endurance.rs` | **Stale TODO; grant half now implementable.** Oracle: "Creatures you control have hexproof." `CreaturesYouControl` + `AddKeyword(Hexproof)` is available. The removal half ("opponents' creatures lose hexproof") needs a new filter, but the grant half should be implemented now. **Fix:** Add `AbilityDefinition::Static` with `CreaturesYouControl` + `AddKeyword(Hexproof)`. Keep TODO for removal half only. |
| 4 | MEDIUM | `archetype_of_imagination.rs` | **Stale TODO; grant half now implementable.** Same as Archetype of Endurance. Oracle: "Creatures you control have flying." **Fix:** Add `AbilityDefinition::Static` with `CreaturesYouControl` + `AddKeyword(Flying)`. Keep TODO for removal half only. |
| 5 | MEDIUM | `iroas_god_of_victory.rs` | **Stale TODO; menace grant now implementable.** Oracle: "Creatures you control have menace." `CreaturesYouControl` + `AddKeyword(Menace)` is available. **Fix:** Add `AbilityDefinition::Static` with `CreaturesYouControl` + `AddKeyword(Menace)`. Keep TODOs for devotion and damage prevention. |
| 6 | MEDIUM | `vito_thorn_of_the_dusk_rose.rs` | **Stale TODO; activated lifelink grant now implementable.** Oracle: "{3}{B}{B}: Creatures you control gain lifelink until end of turn." `AbilityDefinition::Activated` with `Cost::Mana` + `Effect::ApplyContinuousEffect` using `CreaturesYouControl` + `AddKeyword(Lifelink)` + `UntilEndOfTurn` is available (same pattern as crashing_drawbridge.rs). **Fix:** Add the activated ability. Keep TODO for the triggered ability (life-loss trigger). |
| 7 | MEDIUM | `vault_of_the_archangel.rs` | **Stale TODO; activated deathtouch+lifelink grant now implementable.** Oracle: "{2}{W}{B},{T}: Creatures you control gain deathtouch and lifelink until end of turn." Can use two `ApplyContinuousEffect` in a `Sequence` or implement as two stacked effects. Same Crashing Drawbridge pattern. **Fix:** Add activated ability with `Cost::Sequence([Mana({2}{W}{B}), Tap])` and `Effect::Sequence` applying two continuous effects (Deathtouch + Lifelink) with `CreaturesYouControl` + `UntilEndOfTurn`. |
| 8 | LOW | `archetype_of_endurance.rs:4-6` | **Stale TODO text.** Comment says "no ApplyContinuousEffect with EffectFilter::CreaturesYouControl granting Hexproof is currently supported" but `CreaturesYouControl` now exists. **Fix:** Update comment to note only the removal half is blocked. |
| 9 | LOW | `archetype_of_imagination.rs:6-7` | **Stale TODO text.** Same as finding 8. Comment claims `CreaturesYouControl` is "not available" when it is. **Fix:** Update comment. |
| 10 | LOW | `iroas_god_of_victory.rs:13-14` | **Stale TODO text.** Claims "no EffectFilter for all creatures you control in a static continuous ability" when `CreaturesYouControl` now exists. **Fix:** Update comment. |
| 11 | LOW | `vito_thorn_of_the_dusk_rose.rs:10-12` | **Stale TODO text.** Claims "ApplyContinuousEffect to EffectTarget::AllCreaturesYouControl is not currently supported" when it is. **Fix:** Update comment. |
| 12 | LOW | `vault_of_the_archangel.rs:22` | **Stale TODO text.** Claims "no ApplyContinuousEffect with AllCreaturesYouControl" when the pattern exists (crashing_drawbridge.rs proves it). **Fix:** Update comment. |

### Finding Details

#### Finding 2: Goblin Warchief uses wrong filter (excludes self)

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/goblin_warchief.rs:21`
**Oracle**: "Goblins you control have haste."
**Issue**: The oracle text says "Goblins you control have haste" which includes Goblin Warchief itself (it is a Goblin Warrior). The def uses `EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Goblin"))` which explicitly excludes the source object. This means Goblin Warchief does not grant itself haste, producing wrong game state. In practice the card also has `Keyword(KeywordAbility::Haste)` absent from its abilities, so it has no haste at all from the static.
**Fix**: Either (a) add a new `EffectFilter::CreaturesYouControlWithSubtype(SubType)` variant that includes the source, or (b) use the workaround: keep the `OtherCreaturesYouControlWithSubtype("Goblin")` filter AND add `AbilityDefinition::Keyword(KeywordAbility::Haste)` as an intrinsic keyword. Option (b) is simpler but note: the oracle doesn't say Goblin Warchief has haste intrinsically -- it gets haste from its own static. Option (b) is functionally equivalent and acceptable as a workaround.

#### Finding 3: Archetype of Endurance -- grant half implementable

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/archetype_of_endurance.rs:22`
**Oracle**: "Creatures you control have hexproof. / Creatures your opponents control lose hexproof and can't have or gain hexproof."
**Issue**: The card has `abilities: vec![]` with a TODO claiming the filter doesn't exist. `EffectFilter::CreaturesYouControl` now exists and can implement the first ability. The second ability (removal from opponents) requires a new `CreaturesOpponentsControl` filter and is legitimately blocked, but the grant half should not be blocked.
**Fix**: Add `AbilityDefinition::Static { continuous_effect: ContinuousEffectDef { layer: EffectLayer::Ability, modification: LayerModification::AddKeyword(KeywordAbility::Hexproof), filter: EffectFilter::CreaturesYouControl, duration: EffectDuration::WhileSourceOnBattlefield } }`. Update TODO to note only the removal half is blocked.

#### Finding 4: Archetype of Imagination -- grant half implementable

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/archetype_of_imagination.rs:23`
**Oracle**: "Creatures you control have flying. / Creatures your opponents control lose flying and can't have or gain flying."
**Issue**: Same as Finding 3. Grant half is now implementable.
**Fix**: Same pattern as Finding 3 but with `KeywordAbility::Flying`.

#### Finding 5: Iroas, God of Victory -- menace grant implementable

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/iroas_god_of_victory.rs:34`
**Oracle**: "Creatures you control have menace."
**Issue**: Only `Keyword(Indestructible)` is implemented. The menace grant is noted as a DSL gap, but `CreaturesYouControl` now supports it. Devotion and damage prevention are legitimately blocked.
**Fix**: Add `AbilityDefinition::Static { continuous_effect: ContinuousEffectDef { layer: EffectLayer::Ability, modification: LayerModification::AddKeyword(KeywordAbility::Menace), filter: EffectFilter::CreaturesYouControl, duration: EffectDuration::WhileSourceOnBattlefield } }`.

#### Finding 6: Vito -- activated lifelink grant implementable

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/vito_thorn_of_the_dusk_rose.rs:28`
**Oracle**: "{3}{B}{B}: Creatures you control gain lifelink until end of turn."
**Issue**: Empty abilities. The activated ability pattern is identical to Crashing Drawbridge (which is already implemented). `AbilityDefinition::Activated` + `Cost::Mana` + `Effect::ApplyContinuousEffect` + `CreaturesYouControl` + `UntilEndOfTurn`.
**Fix**: Add `AbilityDefinition::Activated { cost: Cost::Mana(ManaCost { generic: 3, black: 2, ..Default::default() }), effect: Effect::ApplyContinuousEffect { effect_def: Box::new(ContinuousEffectDef { layer: EffectLayer::Ability, modification: LayerModification::AddKeyword(KeywordAbility::Lifelink), filter: EffectFilter::CreaturesYouControl, duration: EffectDuration::UntilEndOfTurn }) }, timing_restriction: None, targets: vec![] }`.

#### Finding 7: Vault of the Archangel -- activated deathtouch+lifelink grant implementable

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/vault_of_the_archangel.rs:21`
**Oracle**: "{2}{W}{B}, {T}: Creatures you control gain deathtouch and lifelink until end of turn."
**Issue**: TODO claims the pattern isn't supported. It is -- Crashing Drawbridge uses the same pattern. Need two continuous effects in a Sequence (one for Deathtouch, one for Lifelink).
**Fix**: Add activated ability: `Cost::Sequence(vec![Cost::Mana(ManaCost { generic: 2, white: 1, black: 1, ..Default::default() }), Cost::Tap])`, effect: `Effect::Sequence(vec![Effect::ApplyContinuousEffect { ... Deathtouch ... }, Effect::ApplyContinuousEffect { ... Lifelink ... }])`.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 604.2 (static abilities) | Yes | Yes | 6 tests in static_grants.rs |
| CR 613.1f (Layer 6 ability grant) | Yes | Yes | Covered by keyword grant tests |
| CR 613.1g (Layer 7 P/T) | Yes | Yes | Subtype P/T test covers this |
| CR 604.7 (no LKI for statics) | Yes | Implicit | Source controller resolved live, not cached |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| fervor | Yes | 0 | Yes | Correct |
| mass_hysteria | Yes | 0 | Yes | Correct |
| dragonlord_kolaghan | Yes | 1 (trigger) | Partial | Static grant correct; trigger legitimately blocked |
| goblin_war_drums | Yes | 0 | Yes | Correct |
| brave_the_sands | Yes | 1 (additional blocker) | Partial | Vigilance grant correct; blocker assignment blocked |
| goblin_warchief | **No** | 0 | **No** | **Finding 2**: Wrong filter excludes self; oracle includes self |
| markov_baron | Yes | 0 | Yes | Correct |
| karrthus_tyrant_of_jund | Yes | 1 (ETB) | Partial | Static grant correct; ETB control change blocked |
| crashing_drawbridge | Yes | 0 | Yes | Correct |
| ultramarines_honour_guard | Yes | 0 | Yes | Correct |
| camellia_the_seedmiser | Partial | 1 (food trigger) | Partial | Static+Forage correct; food sacrifice trigger blocked |
| archetype_of_endurance | **No** | 2 (stale) | **No** | **Finding 3**: Grant half implementable but missing |
| archetype_of_imagination | **No** | 2 (stale) | **No** | **Finding 4**: Grant half implementable but missing |
| iroas_god_of_victory | **No** | 3 (2 stale) | **No** | **Finding 5**: Menace grant implementable but missing |
| rhythm_of_the_wild | No | 2 | No | Both abilities legitimately blocked (nontoken filter + can't-be-countered) |
| vito_thorn_of_the_dusk_rose | **No** | 2 (1 stale) | **No** | **Finding 6**: Activated lifelink grant implementable |
| vault_of_the_archangel | **No** | 1 (stale) | **No** | **Finding 7**: Activated grant implementable |
| dragon_tempest | No | 2 | No | Legitimately blocked (keyword ETB filter + count-based damage) |
| scourge_of_valkas | No | 2 | No | Legitimately blocked (subtype ETB trigger + count-based damage) |
| ainok_bond_kin | No | 1 | No | Legitimately blocked (counter-presence filter on static) |
| akromas_will | No | 1 | No | Legitimately blocked (conditional modal + multi-keyword mass grant) |
| blade_historian | No | 1 | No | Legitimately blocked (attacking-creatures filter) |
| bloodmark_mentor | No | 1 | No | Legitimately blocked (color filter for creatures) |
| castle_embereth | Partial | 1 | Partial | ETB-tapped + mana correct; activated pump blocked |
| cryptic_coat | Partial | 3 | Partial | ETB cloak correct; static grant to equipped creature blocked |
| etchings_of_the_chosen | Partial | 2 | Partial | Choose-type implemented; dynamic subtype grant blocked |
| final_showdown | Partial | 2 | Partial | Spree mode 2 (destroy all) correct; modes 0+1 blocked |
| great_oak_guardian | No | 1 | No | Legitimately blocked (targeted trigger + mass pump) |
| greymond_avacyns_stalwart | No | 2 | No | Legitimately blocked (choose-ability + count threshold) |
| indomitable_archangel | No | 1 | No | Legitimately blocked (metalcraft condition + artifact filter) |
| legolass_quick_reflexes | No | 1 | No | Legitimately blocked (multi-effect spell) |
| overwhelming_stampede | No | 1 | No | Legitimately blocked (dynamic X based on max power) |
| tatyova_steward_of_tides | No | 2 | No | Legitimately blocked (land-creature filter + animate-land) |
| teysa_karlov | Partial | 1 | Partial | Trigger doubling correct; token vigilance+lifelink blocked |
| thousand_year_elixir | Partial | 1 | Partial | Activated untap correct; haste-bypass static blocked |
| throatseeker | No | 1 | No | Legitimately blocked (combat-state + subtype filter) |
| reckless_bushwhacker | No | 1 | No | Legitimately blocked (surge-condition ETB trigger) |
| jagged_scar_archers | No | 2 | No | Legitimately blocked (CDA + activated with self-power) |
| nadaar_selfless_paladin | Partial | 1 | Partial | Triggers correct; conditional +1/+1 blocked (dungeon condition) |

## Test Coverage

Tests in `crates/engine/tests/static_grants.rs` provide good coverage:

| Test | What it covers |
|------|---------------|
| `test_creatures_you_control_grants_keyword_to_own_creatures_only` | Positive: own creature gets keyword; Negative: opponent's creature doesn't |
| `test_creatures_you_control_excludes_non_creatures` | Negative: land doesn't get keyword |
| `test_other_creatures_you_control_excludes_source` | Positive: other own creature gets keyword; Negative: source excluded; Negative: opponent excluded |
| `test_other_creatures_with_subtype_filters_correctly` | Positive: matching subtype gets bonus; Negative: source excluded; Negative: wrong subtype excluded; Negative: opponent excluded |
| `test_creatures_you_control_no_source_matches_nothing` | Edge case: no source = no matches |
| `test_multiple_controllers_grant_independently` | Multi-player: each controller's enchantment grants only to their own creatures |

**Missing test**: No integration test using an actual CardDefinition (e.g., Fervor) cast through the full engine pipeline. All tests manually construct ContinuousEffect objects. This is LOW severity -- the unit tests are thorough and the CardDef static registration is tested implicitly by the card def compilation.
