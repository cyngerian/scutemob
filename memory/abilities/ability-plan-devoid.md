# Ability Plan: Devoid

**Generated**: 2026-02-28
**CR**: 702.114
**Priority**: P4
**Similar abilities studied**: Changeling (CR 702.73, CDA in Layer 4) at `crates/engine/src/rules/layers.rs:62-72`, `crates/engine/tests/changeling.rs`

## CR Rule Text

**702.114. Devoid**

702.114a Devoid is a characteristic-defining ability. "Devoid" means "This object is colorless." This ability functions everywhere, even outside the game. See rule 604.3.

**Referenced rules:**

604.3. Some static abilities are characteristic-defining abilities. A characteristic-defining ability conveys information about an object's characteristics that would normally be found elsewhere on that object (such as in its mana cost, type line, or power/toughness box). Characteristic-defining abilities can add to or override information found elsewhere on that object. Characteristic-defining abilities function in all zones. They also function outside the game and before the game begins.

604.3a. A static ability is a characteristic-defining ability if it meets the following criteria: (1) It defines an object's colors, subtypes, power, or toughness; (2) it is printed on the card it affects, it was granted to the token it affects by the effect that created the token, or it was acquired by the object it affects as the result of a copy effect or text-changing effect; (3) it does not directly affect the characteristics of any other objects; (4) it is not an ability that an object grants to itself; and (5) it does not set the values of such characteristics only if certain conditions are met.

613.1e. Layer 5: Color-changing effects are applied.

613.3. Within layers 2-6, apply effects from characteristic-defining abilities first (see rule 604.3), then all other effects in timestamp order (see rule 613.7). Note that dependency may alter the order in which effects are applied within a layer. (See rule 613.8.)

202.2. An object is the color or colors of the mana symbols in its mana cost, regardless of the color of its frame.

## Key Edge Cases

- **Devoid functions in all zones** (CR 604.3, CR 702.114a) -- not just the battlefield. A Devoid card in hand, graveyard, library, exile, or command zone is colorless.
- **Devoid + RemoveAllAbilities (Humility/Dress Down)**: Layer 5 (color-change CDA) runs before Layer 6 (ability removal). If a creature with Devoid loses all abilities in Layer 6, it is STILL colorless because the Layer 5 CDA already applied. Ruling (Vile Aggregate 2015-08-25): "If a card loses devoid, it will still be colorless."
- **Devoid + color-adding effects**: Other effects can give a Devoid card color. A non-CDA Layer 5 effect (e.g., Painter's Servant) that adds a color runs AFTER the Devoid CDA within Layer 5 (CR 613.3). The card becomes that new color. Ruling: "Other cards and abilities can give a card with devoid color. If that happens, it's just the new color, not that color and colorless."
- **Devoid does NOT affect color identity** (CR 903.4): Color identity is determined by mana symbols in cost, oracle text, color indicator, and CDAs. However, Devoid makes the card colorless, not "no mana symbols in cost." The mana symbols still contribute to color identity. Example: Vile Aggregate has mana cost {2}{R} and color identity ["R"], despite being colorless. The engine's `compute_color_identity` in `commander.rs` derives color identity from mana cost symbols, which is already correct.
- **Protection from a color**: A creature with "protection from red" does NOT have protection from a Devoid creature with {R} in its mana cost, because the Devoid creature is colorless. The protection check in `protection.rs:161` uses `source_chars.colors.contains(c)`, which goes through `calculate_characteristics` -- this will correctly see an empty color set after Devoid applies.
- **"A card with devoid is just colorless"** (ruling 2015-08-25): It is NOT "colorless and the colors of mana in its mana cost." The colors are fully overridden, not supplemented.
- **Multiplayer**: No special multiplayer considerations. Devoid is a self-referential CDA that affects only the card it is printed on.

## Current State (from ability-wip.md)

Devoid is not the currently tracked ability in `ability-wip.md` (Horsemanship is). No prior work exists for Devoid.

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (Layer 5 CDA in `calculate_characteristics`)
- [ ] Step 3: Trigger wiring -- N/A (Devoid has no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Hash impl update
- [ ] Step 8: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Devoid` variant after the `Overload` variant (line ~599, just before the closing `}` of the enum).
**Pattern**: Follow `KeywordAbility::Changeling` at line 314-320 -- both are CDAs with the same doc structure.
**Doc comment**:
```rust
/// CR 702.114: Devoid -- "This object is colorless."
///
/// Characteristic-defining ability (CDA). Applied as a color-change in Layer 5
/// (ColorChange) before non-CDA effects (CR 613.3). Functions in all zones
/// (CR 604.3). Clears the object's colors set, making it colorless regardless
/// of its mana cost (CR 202.2).
Devoid,
```

### Step 2: Hash Update

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add a new arm to the `HashInto for KeywordAbility` match expression, after the `Overload` arm (line ~458).
**Discriminant**: 71 (next after Overload = 70).
**Pattern**: Follow `KeywordAbility::Shadow => 68u8.hash_into(hasher)` -- simple unit variant, no data.
```rust
// Devoid (discriminant 71) -- CR 702.114
KeywordAbility::Devoid => 71u8.hash_into(hasher),
```

### Step 3: Rule Enforcement (Layer 5 CDA)

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Add an inline CDA check for Devoid in the `calculate_characteristics` function, at the top of the Layer 5 (ColorChange) processing. This is the exact same pattern as the Changeling CDA check at lines 62-72, but for Layer 5 instead of Layer 4.
**Location**: Inside the `for &layer in &layers_in_order` loop, AFTER the existing Changeling CDA block (line 72), add a new block:
```rust
// CR 702.114a + CR 613.3: Devoid is a characteristic-defining ability that
// makes the object colorless in Layer 5 (ColorChange), before any non-CDA
// Layer 5 effects. CDAs apply first within each layer (CR 613.3), so this
// runs before gathering layer_effects. A subsequent SetColors/AddColors
// effect (e.g., Painter's Servant) will correctly override the Devoid
// colorlessness because it runs after the CDA within Layer 5.
// CR 604.3: Functions in all zones, not just the battlefield.
if layer == EffectLayer::ColorChange && chars.keywords.contains(&KeywordAbility::Devoid) {
    chars.colors = OrdSet::new();
}
```
**CR**: 702.114a -- "This object is colorless"; 613.3 -- CDAs apply first within each layer; 604.3 -- functions in all zones.

**Key correctness note**: The `calculate_characteristics` function already iterates over ALL objects regardless of zone -- there is no battlefield-only filter. So this naturally handles the "functions in all zones" requirement (CR 604.3). The same is true for Changeling (confirmed by `test_changeling_works_in_graveyard`).

### Step 4: Trigger Wiring

**N/A** -- Devoid is a static CDA with no triggered or activated abilities. No wiring in `builder.rs`, `abilities.rs`, or `resolution.rs` is needed.

### Step 5: Unit Tests

**File**: `crates/engine/tests/devoid.rs` (new file)
**Tests to write**:

1. **`test_devoid_creature_is_colorless`** -- CR 702.114a
   A creature with Devoid and colored mana cost (e.g., `mana_cost: ManaCost { red: 1, generic: 2, .. }`) should have empty `colors` set after `calculate_characteristics`. Use `ObjectSpec::creature(p1(), "Devoid Creature", 3, 2).with_keyword(KeywordAbility::Devoid).with_colors(vec![Color::Red])`.

2. **`test_devoid_creature_base_colors_present`** -- Negative: verify the BASE Characteristics (before layer calc) still has the mana-cost-derived colors. This confirms Devoid operates at the layer level, not at definition time. `state.objects.get(&id).unwrap().characteristics.colors` should still contain `Color::Red`.

3. **`test_devoid_lose_all_abilities_still_colorless`** -- CR 613.3 + ruling 2015-08-25
   A Devoid creature under a RemoveAllAbilities effect (Layer 6) should still be colorless. Layer 5 CDA runs before Layer 6 ability removal. Follow `test_changeling_lose_all_abilities_keeps_types` pattern.

4. **`test_devoid_color_adding_effect_overrides`** -- CR 613.3 + ruling 2015-08-25
   A non-CDA AddColors effect in Layer 5 (timestamp 10) applied to a Devoid creature should give it that color. CDA clears colors first, then the non-CDA effect adds the color.

5. **`test_devoid_works_in_graveyard`** -- CR 604.3
   A Devoid card in the graveyard should be colorless after `calculate_characteristics`. Follow `test_changeling_works_in_graveyard` pattern using `.in_zone(ZoneId::Graveyard(p1()))`.

6. **`test_devoid_works_in_hand`** -- CR 604.3
   A Devoid card in hand should be colorless after `calculate_characteristics`.

7. **`test_non_devoid_creature_retains_colors`** -- Negative test
   A creature with colored mana cost but without Devoid should retain its colors after `calculate_characteristics`.

8. **`test_devoid_protection_from_color_does_not_match`** -- CR 702.16a + CR 702.114a
   A creature with "protection from red" is NOT protected from a Devoid creature whose mana cost contains {R}, because the Devoid creature is colorless. Verify by checking `has_protection_from_source` with a Devoid source.

**Pattern**: Follow `crates/engine/tests/changeling.rs` for test structure:
- Helper functions `p1()`, `battlefield_id()`, `indef_effect()`
- Imports: `calculate_characteristics`, `Color`, `ContinuousEffect`, `EffectDuration`, `EffectFilter`, `EffectId`, `EffectLayer`, `GameStateBuilder`, `KeywordAbility`, `LayerModification`, `ObjectSpec`, `PlayerId`, `ProtectionQuality`, `ZoneId`
- Use `OrdSet` from `im` crate for color assertions
- File header with `//! Devoid ability tests (CR 702.114).`

### Step 6: Card Definition (later phase)

**Suggested card**: Forerunner of Slaughter
- Mana Cost: {B}{R}
- Type: Creature -- Eldrazi Drone
- Oracle Text: Devoid (This card has no color.) / {1}: Target colorless creature gains haste until end of turn.
- P/T: 3/2
- Color Identity: ["B", "R"]
- Keywords: Devoid
- **Why**: Simple creature with Devoid and a basic activated ability. Tests the Devoid + activated ability interaction. The activated ability targets colorless creatures, which is a natural fit since the card itself is colorless via Devoid.
- **Card lookup**: use `card-definition-author` agent

**Alternative simpler card**: Mist Intruder (Devoid + Flying + Ingest), but Ingest is not yet validated.

### Step 7: Game Script (later phase)

**Suggested scenario**: "Devoid creature is unaffected by protection from its mana cost color"
- Setup: Player 1 has a Forerunner of Slaughter (Devoid, {B}{R} mana cost) on the battlefield. Player 2 has a creature with protection from red.
- Action: Player 1 declares Forerunner as attacker, Player 2 declares their creature as blocker.
- Expected: Block IS legal -- Devoid creature is colorless, so "protection from red" does not prevent blocking.
- **Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Protection from color + Devoid**: The protection check (`protection.rs:161`) uses `source_chars.colors` from `calculate_characteristics`. Once Devoid clears colors in Layer 5, protection from any specific color will not match the Devoid source. This is correct behavior and requires no code changes -- only a test to confirm.
- **Color identity + Devoid**: The `compute_color_identity` function in `commander.rs` derives color identity from mana cost symbols and oracle text symbols (CR 903.4). It does NOT use `calculate_characteristics` or the `colors` field. So Devoid correctly has no effect on color identity. No code changes needed, but a test should confirm this.
- **Layer 5 ordering with Painter's Servant**: Painter's Servant adds a color to all objects via a non-CDA Layer 5 effect. With Devoid, the CDA clears colors first (CR 613.3), then Painter's Servant adds a color. Result: the Devoid card has only the Painter's Servant color. This is correct and requires no special handling -- just the standard CDA-before-non-CDA ordering in the layer loop.
- **Color-based targeting**: Spells that target by color (e.g., "target nonblack creature") will see Devoid creatures as colorless. The targeting infrastructure already uses `calculate_characteristics` to evaluate `TargetFilter` conditions, so this works correctly.
- **"Choose a color" effects**: Effects that interact with the chosen color of an object (e.g., Circle of Protection effects) will see Devoid creatures as colorless. No special handling needed.
