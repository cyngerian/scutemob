# Ability Plan: Prototype

**Generated**: 2026-03-02
**CR**: 702.160 + 718 (Prototype Cards)
**Priority**: P4
**Similar abilities studied**: Dash (alt cost + flag transfer in resolution.rs), Bestow (alt cost casting path), Devoid (CDA color-change in layers.rs layer 5), Changeling (CDA type-change in layers.rs layer 4)

## CR Rule Text

### CR 702.160 -- Prototype (keyword ability)

> 702.160a Prototype is a static ability that appears on prototype cards that have a
> secondary set of power, toughness, and mana cost characteristics. A player who casts
> a spell with prototype can choose to cast that card "prototyped." If they do, the
> alternative set of its power, toughness, and mana cost characteristics are used.
> See 718, "Prototype Cards."

### CR 718 -- Prototype Cards (full section)

> 718.1 Prototype cards have a two-part frame, with a smaller frame inset below the type
> line of the card. The inset frame contains the prototype keyword ability as well as a
> second set of power, toughness, and mana cost characteristics.

> 718.2 The mana cost, power, and toughness in the inset frame represent alternative
> characteristics that the object may have while it is a spell or while it is a permanent
> on the battlefield. The card's normal characteristics appear as usual.

> 718.2a The existence and values of these alternative characteristics are part of the
> object's copiable values.

> 718.3 As a player casts a prototype card, the player chooses whether they cast the card
> normally or cast it as a prototyped spell using the prototype keyword ability (see rule
> 702.160, "Prototype").

> 718.3a While casting a prototyped spell, use only its alternative power, toughness, and
> mana cost when evaluating those characteristics to see if it can be cast.

> 718.3b Both a prototyped spell and the permanent it becomes have only its alternative set
> of power, toughness, and mana cost characteristics. If that mana cost includes one or
> more colored mana symbols, the spell and the permanent it becomes are also that color or
> colors (see rule 105.2).

> 718.3c If a prototyped spell is copied, the copy is also a prototyped spell. It has the
> alternative power, toughness, and mana cost characteristics of the spell and not the
> normal power, toughness, and mana cost characteristics of the card that represents the
> prototyped spell. Any rule or effect that refers to a prototyped spell refers to the
> copy as well.

> 718.3d If a permanent that was a prototyped spell is copied, the copy has the alternative
> power, toughness, and mana cost characteristics of the permanent and not the normal power
> and toughness characteristics of the card that represents that permanent. Any rule or
> effect that refers to a permanent that was a prototyped spell refers to the copy as well.

> 718.4 In every zone except the stack or the battlefield, and while on the stack or the
> battlefield when not cast as a prototyped spell, a prototype card has only its normal
> characteristics.

> 718.5 A prototype card's characteristics other than its power, toughness, and mana cost
> (and other than color) remain the same whether it was cast as a prototyped spell or cast
> normally.

### CR 105.2 (color from mana cost)

> 105.2 An object can be one or more of the five colors, or it can be no color at all.
> An object is the color or colors of the mana symbols in its mana cost, regardless of
> the color of its frame.

## Key Edge Cases

1. **Prototype is NOT an alternative cost (CR 118.9).** Ruling on Blitz Automaton
   (2022-10-14): "Casting a prototyped spell isn't the same as casting it for an
   alternative cost, and an alternative cost may be applied to a spell cast this way."
   This means Prototype can COMBINE with Flashback, Escape, etc. This is the single
   most critical design consideration -- Prototype must NOT use the `AltCostKind` enum
   or the `alt_cost` field on `CastSpell`.

2. **Prototype characteristics apply ONLY on the stack and battlefield when cast as
   prototyped (CR 718.4).** In every other zone (hand, graveyard, exile, library), the
   card has its normal characteristics (e.g., Blitz Automaton is a colorless {7} MV card
   in graveyard).

3. **Prototype color is determined by prototype mana cost (CR 718.3b / CR 105.2).** If
   prototype cost is `{2}{R}`, the prototyped permanent is red. If cost is `{1}{B}{B}`,
   it is black. If cost is `{3}`, it is colorless. The normal card (non-prototyped) keeps
   its own color (or colorlessness from lack of colored mana).

4. **Mana value of prototyped spell/permanent uses prototype mana cost (ruling on Fateful
   Handoff, 2022-10-14).** "The mana value of an artifact creature that was cast as a
   prototype is based on the prototype mana cost rather than the card's usual mana cost."
   This affects `mana_cost` on `Characteristics`, not just P/T.

5. **Copies of prototyped spells/permanents retain prototype characteristics (CR 718.3c,
   718.3d).** The prototype status IS part of the copiable values (CR 718.2a). A Clone
   copying a prototyped Blitz Automaton becomes a 3/2 red creature, not a 6/4 colorless.

6. **Name, abilities, types, and subtypes are unchanged (CR 718.5).** Only mana cost,
   mana value, color, power, and toughness change.

7. **Prototype can be used from any zone the spell could be cast from (ruling
   2022-10-14).** If an effect lets you cast artifact spells from graveyard, you can
   cast a prototyped version from graveyard.

8. **Cost reduction applies to prototype cost (ruling on Esper, 2023-04-14).** "The cost
   reduction can apply to alternative costs such as prototype costs." This confirms the
   prototype cost flows through the normal cost-modification pipeline (kicker, convoke,
   delve, improvise, commander tax all apply).

9. **When the prototyped permanent leaves the battlefield, it resumes normal
   characteristics (CR 718.4, ruling 2022-10-14).** The `is_prototyped` flag is cleared
   on zone change (CR 400.7 pattern, same as other flags).

10. **Multiplayer: no special multiplayer interactions.** Prototype is a per-object
    characteristic modification, not an interaction-dependent ability.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- Prototype is static, no triggers)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: KeywordAbility Variant + AbilityDefinition + GameObject Flag

#### Step 1a: `KeywordAbility::Prototype`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Prototype` variant after `Plot` (line ~870)
**Discriminant**: 98 (next available after Plot=97)
**Pattern**: Follow `KeywordAbility::Plot` at line 861-870

```rust
/// CR 702.160: Prototype [mana cost] -- [power]/[toughness].
/// Static ability. When casting, the player may choose to cast the card
/// "prototyped" using the alternative mana cost, power, and toughness.
///
/// IMPORTANT: Prototype is NOT an alternative cost (CR 118.9). It can
/// combine with alternative costs like Flashback.
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The prototype data is stored in `AbilityDefinition::Prototype`.
Prototype,
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
Add after Plot (discriminant 97, line ~530):
```rust
// Prototype (discriminant 98) -- CR 702.160
KeywordAbility::Prototype => 98u8.hash_into(hasher),
```

#### Step 1b: `AbilityDefinition::Prototype`

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add variant after `Plot` (line ~358)
**Discriminant**: 31 (next available after Plot=30)

```rust
/// CR 702.160 / CR 718: Prototype [cost] -- [power]/[toughness].
/// The player may cast this spell with the prototype mana cost, power, and
/// toughness instead of the card's normal values.
///
/// IMPORTANT: This is NOT an alternative cost (CR 118.9). Prototype changes
/// the spell's characteristics, not just the payment. It can be combined with
/// alternative costs (Flashback, Escape, etc.).
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Prototype)` for quick
/// presence-checking without scanning all abilities.
Prototype {
    /// The prototype mana cost (paid instead of the card's normal cost when
    /// choosing to cast prototyped). Also determines the prototyped permanent's
    /// color (CR 718.3b / CR 105.2).
    cost: ManaCost,
    /// The prototype power (replaces the card's printed power).
    power: i32,
    /// The prototype toughness (replaces the card's printed toughness).
    toughness: i32,
},
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
Add after Plot AbilityDefinition hash (discriminant 30, line ~3187):
```rust
// Prototype (discriminant 31) -- CR 702.160
AbilityDefinition::Prototype { cost, power, toughness } => {
    31u8.hash_into(hasher);
    cost.hash_into(hasher);
    power.hash_into(hasher);
    toughness.hash_into(hasher);
}
```

#### Step 1c: `GameObject.is_prototyped` flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `pub is_prototyped: bool` field after `plotted_turn` (line ~502)
**CR**: 718.3b -- tracks whether this permanent was cast as a prototyped spell

```rust
/// CR 718.3b: If true, this permanent was cast as a prototyped spell.
/// The permanent uses its alternative power, toughness, mana cost, and
/// color (derived from the prototype mana cost) instead of its normal values.
///
/// Applied in the layer system:
/// - Layer 5 (ColorChange): set colors from prototype mana cost
/// - Layer 7b (PtSet): set power/toughness from prototype values
/// - Mana cost: overridden at base-characteristics level
///
/// Reset to false on zone changes (CR 400.7 / CR 718.4).
/// Part of copiable values (CR 718.2a, 718.3c, 718.3d).
#[serde(default)]
pub is_prototyped: bool,
```

**Initialize in all creation sites** (same pattern as `is_plotted`):
- `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs` (line ~909 area): `is_prototyped: false,`
- `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs` (line ~280 area, zone-move reset): `is_prototyped: false,`
- `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs` (line ~378 area, second zone-move): `is_prototyped: false,`
- `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` (line ~2455 area, token creation): `is_prototyped: false,`
- `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs` -- all 5+ `GameObject` literal sites (lines ~1425, ~2503, ~2686, ~2886): `is_prototyped: false,`

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
Add after `plotted_turn` hash (line ~694 area, in `GameObject` HashInto):
```rust
// Prototype (CR 718.3b) -- whether this permanent was cast prototyped
self.is_prototyped.hash_into(hasher);
```

#### Step 1d: `StackObject.was_prototyped` flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `pub was_prototyped: bool` field after `was_plotted` (line ~172)
**CR**: 718.3b

```rust
/// CR 718.3b: If true, this spell was cast as a prototyped spell.
/// The spell uses its alternative power, toughness, mana cost, and color.
///
/// IMPORTANT: This is NOT mutually exclusive with other alt-cost flags.
/// A spell can be both prototyped AND cast with flashback (CR 118.9 ruling).
///
/// Must always be false for copies (`is_copy: true`) -- WAIT: CR 718.3c says
/// "If a prototyped spell is copied, the copy is also a prototyped spell."
/// So copies DO inherit this flag.
#[serde(default)]
pub was_prototyped: bool,
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
Add after `was_plotted` hash in StackObject HashInto (line ~1638 area):
```rust
// Prototype (CR 718.3b) -- spell was cast prototyped
self.was_prototyped.hash_into(hasher);
```

#### Step 1e: helpers.rs prelude (if needed)

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/helpers.rs`
**Action**: Verify `KeywordAbility` and `ManaCost` are already exported. No new types should be needed since AbilityDefinition::Prototype uses ManaCost + i32 which are already available.

#### Step 1f: Match arm exhaustiveness

**Grep for all `AbilityDefinition` match expressions** and add `Prototype { .. }` arm:
- `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` (covered above)
- `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs` -- `get_*_cost` helper fns that scan abilities
- `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs` -- if any match on AbilityDefinition
- `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs` -- builder.rs keyword->trigger translation
- `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs` -- no new StackObjectKind, so no change needed

### Step 2: Casting Path (casting.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**CR**: 702.160a, 718.3, 718.3a

Prototype is NOT an `AltCostKind`. It needs a separate boolean on the `CastSpell` command.

#### Step 2a: Add `prototype: bool` to `Command::CastSpell`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add field to `CastSpell` variant (line ~149 area, before closing brace):

```rust
/// CR 702.160 / CR 718.3: If true, the spell is cast as a prototyped spell.
/// The prototype mana cost is used for payment (instead of the normal mana cost),
/// and the spell/permanent has prototype P/T and color.
///
/// IMPORTANT: This is NOT an alternative cost (CR 118.9). It can be combined
/// with `alt_cost` (e.g., prototype + flashback). The `prototype` flag is
/// orthogonal to `alt_cost`.
///
/// Validation: card must have `AbilityDefinition::Prototype` in its definition.
#[serde(default)]
prototype: bool,
```

#### Step 2b: Threading through `handle_cast_spell`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Accept the `prototype: bool` parameter and:

1. **Validate** (around line ~230 area): If `prototype == true`, verify the card has
   `AbilityDefinition::Prototype` in its card definition. Extract `(proto_cost, proto_power, proto_toughness)`.

2. **Cost selection** (around line ~971, Step 2): When `prototype == true`:
   - If another `alt_cost` is ALSO set (e.g., Flashback), the alt cost determines the
     base cost. BUT: the ruling says "Casting a prototyped spell isn't the same as casting
     it for an alternative cost, and an alternative cost may be applied to a spell cast
     this way." This means: when prototyped, the prototype mana cost REPLACES the card's
     mana cost, and then an alternative cost (if any) replaces that. In practice, the only
     meaningful combination is "prototype + no alt cost" (pay prototype mana) or
     "prototype + 'without paying its mana cost'" (free cast, still prototyped).
   - **Most common case**: `prototype == true && alt_cost == None` -> use `proto_cost` as
     the base cost.
   - **With alt cost**: the alt cost's cost replaces the base cost as usual; the prototype
     flag only affects characteristics (P/T, color), not cost. Actually, re-reading the
     ruling: "you could either cast Blitz Automaton normally, or as a prototyped spell"
     suggests prototype IS the base cost source. When combined with a "without paying
     mana cost" effect, the cost is free but characteristics are still prototyped.
   - **Implementation**: Insert a new clause BEFORE the alt-cost selection chain: if
     `prototype == true`, set `base_mana_cost` to `proto_cost`. Then the alt-cost chain
     runs on top of this modified base. If Flashback is used, the flashback cost
     OVERRIDES the prototype cost (since flashback pays a specific cost). The prototype
     flag still applies to characteristics.

   Actually, on further analysis: CR 718.3a says "use only its alternative power,
   toughness, and mana cost when evaluating those characteristics to see if it can be
   cast." The prototype mana cost BECOMES the spell's mana cost for all purposes while
   on the stack and battlefield. So if Flashback says "pay [flashback cost] instead of
   mana cost", the mana cost of a prototyped spell IS the prototype cost. The flashback
   cost overrides THAT. This is the same as normal.

   **Simplified approach**: When `prototype == true`:
   - Replace `base_mana_cost` with `proto_cost` (the card's mana cost is now the
     prototype cost for casting purposes).
   - The alt-cost chain then operates on this as the "printed mana cost" -- if no alt
     cost, pay proto_cost; if Flashback, pay flashback cost; if "without paying", free.
   - Set `was_prototyped = true` on the StackObject.

3. **Stack object creation**: Set `was_prototyped: true` on the StackObject.

4. **Characteristics on the stack object's source**: When the card is moved to the stack
   zone, its `characteristics.mana_cost` should be set to the prototype cost,
   `characteristics.power` to proto_power, `characteristics.toughness` to proto_toughness,
   and `characteristics.colors` derived from proto_cost. This ensures effects that check
   the spell's characteristics on the stack see the correct values (CR 718.3a).

#### Step 2c: Helper function `get_prototype_data`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs`
**Action**: Add helper (pattern: follow `get_flashback_cost`):

```rust
/// CR 702.160a: Extract prototype data (cost, power, toughness) from a card definition.
fn get_prototype_data(
    card_id: &Option<CardId>,
    registry: &CardRegistry,
) -> Option<(ManaCost, i32, i32)> {
    let cid = card_id.as_ref()?;
    let def = registry.get(cid)?;
    for ability in &def.abilities {
        if let AbilityDefinition::Prototype { cost, power, toughness } = ability {
            return Some((cost.clone(), *power, *toughness));
        }
    }
    None
}
```

#### Step 2d: Helper function `colors_from_mana_cost`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/casting.rs` (or a shared location)
**Action**: Add a utility to derive colors from a ManaCost (CR 105.2):

```rust
/// CR 105.2: Derive the colors of an object from its mana cost.
fn colors_from_mana_cost(cost: &ManaCost) -> im::OrdSet<Color> {
    let mut colors = im::OrdSet::new();
    if cost.white > 0 { colors.insert(Color::White); }
    if cost.blue > 0 { colors.insert(Color::Blue); }
    if cost.black > 0 { colors.insert(Color::Black); }
    if cost.red > 0 { colors.insert(Color::Red); }
    if cost.green > 0 { colors.insert(Color::Green); }
    colors
}
```

### Step 3: Resolution Path (resolution.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**CR**: 718.3b

**Action**: In the permanent-creation block (around line ~276-294), after setting `obj.kicker_times_paid` and before setting `obj.cast_alt_cost`:

```rust
// CR 718.3b: Transfer prototyped status from stack to permanent.
// The permanent uses the alternative P/T, mana cost, and color while
// on the battlefield. The layer system reads this flag.
obj.is_prototyped = stack_obj.was_prototyped;
```

Also: When the spell's characteristics are copied to the permanent at resolution, if
`was_prototyped`, the permanent's BASE characteristics should reflect the prototype
values. This is subtle -- the question is whether to modify the base characteristics
at resolution time or in the layer system.

**Decision: Modify base characteristics at resolution time AND use the layer system as backup.**

Rationale: CR 718.3b says the prototyped permanent "has only its alternative set of
power, toughness, and mana cost characteristics." This is a change to the copiable
values (CR 718.2a), not a continuous effect. Therefore, the correct implementation is:
- At resolution time, if `was_prototyped`, overwrite the permanent's
  `characteristics.power`, `characteristics.toughness`, `characteristics.mana_cost`,
  and `characteristics.colors` with the prototype values.
- The `is_prototyped` flag is set so that the copy system (CR 718.3c/3d) knows to
  propagate prototype characteristics.
- The layer system does NOT need special Prototype handling if we set base chars correctly.

This is cleaner than the layer-system approach because it follows CR 718.2a (copiable
values) and avoids adding special-case code to the performance-critical layer loop.

**Implementation at resolution.rs** (in the permanent-creation block, around line ~276):
```rust
// CR 718.3b: Apply prototype characteristics to the permanent.
if stack_obj.was_prototyped {
    obj.is_prototyped = true;
    if let Some(proto_data) = get_prototype_data(&obj.card_id, &state.card_registry) {
        let (proto_cost, proto_power, proto_toughness) = proto_data;
        obj.characteristics.power = Some(proto_power);
        obj.characteristics.toughness = Some(proto_toughness);
        obj.characteristics.colors = colors_from_mana_cost(&proto_cost);
        obj.characteristics.mana_cost = Some(proto_cost);
    }
}
```

### Step 4: Copy System (copy.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/copy.rs`
**CR**: 718.2a, 718.3c, 718.3d

**Action**: In `get_copiable_values_inner`, after retrieving base characteristics,
check if the source object `is_prototyped`. If so, the copiable values should reflect
the prototype characteristics (the characteristics already have the prototype values
from resolution time, so no modification is needed -- the base chars on the object
ARE the prototype chars). However, verify that this is correct by tracing:

1. Source is prototyped -> `obj.characteristics` has prototype P/T/mana/color (set at resolution)
2. `get_copiable_values` returns `obj.characteristics.clone()` -> correct prototype values
3. Copy effect sets the copy's characteristics to these values -> copy is prototyped

This should work automatically. The `is_prototyped` flag should also be copied to the
copy's `GameObject` so that zone changes are handled correctly. Check if `get_copiable_values`
handles the flag or if it needs explicit propagation.

**Likely no code changes needed in copy.rs** -- the base-characteristics approach at
resolution time makes this work automatically. Add a test to verify.

### Step 5: Replay Harness + Script Schema

#### Step 5a: Script Schema

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/script_schema.rs`
**Action**: Add `prototype: bool` field to `ScriptAction::PlayerAction` (after `discard_card`):

```rust
/// CR 702.160 / CR 718.3: For `cast_spell_prototype`. If true, the spell is
/// cast using its prototype cost, power, and toughness.
#[serde(default)]
prototype: bool,
```

#### Step 5b: Replay Harness

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"cast_spell_prototype"` arm to `translate_player_action()`:

```rust
"cast_spell_prototype" => {
    let card_id = find_in_hand(state, player, card_name?)?;
    let target_list = resolve_targets(targets, state, players);
    Some(Command::CastSpell {
        player,
        card: card_id,
        targets: target_list,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: None,
        escape_exile_cards: vec![],
        retrace_discard_land: None,
        jump_start_discard: None,
        prototype: true,
    })
}
```

Also update ALL existing `Command::CastSpell` constructions in the harness to include
`prototype: false` (there are ~10+ of them).

#### Step 5c: `Command::CastSpell` passthrough in engine.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Pass the `prototype` field through to `handle_cast_spell` in the
`Command::CastSpell` match arm.

### Step 6: No Trigger Wiring Needed

Prototype is a static ability that modifies casting and permanent characteristics. It has
no triggered abilities, no end-of-turn effects, and no SBA interactions beyond normal
creature toughness checks.

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/prototype.rs`
**Tests to write**:

1. **`test_prototype_basic_cast`** -- Cast a prototype card using its prototype cost.
   Verify: mana deducted = prototype cost; permanent enters with prototype P/T; permanent
   has colors from prototype mana cost.
   **CR**: 702.160a, 718.3, 718.3b

2. **`test_prototype_normal_cast`** -- Cast a prototype card for its normal cost.
   Verify: mana deducted = normal cost; permanent has normal P/T; permanent has normal
   colors (colorless for artifact creature with no colored mana).
   **CR**: 718.4

3. **`test_prototype_color_change`** -- Cast `{2}{R}` prototype of a `{7}` artifact.
   Verify: prototyped permanent is red; non-prototyped is colorless.
   **CR**: 718.3b, 105.2

4. **`test_prototype_mana_value`** -- Cast prototyped `{2}{R}` version.
   Verify: mana value is 3 (not 7).
   **CR**: 718.3b (ruling on Fateful Handoff)

5. **`test_prototype_leaves_battlefield_resumes_normal`** -- Cast prototyped, then
   bounce to hand. Verify: in hand, the card has normal characteristics (mana value 7,
   colorless, normal P/T).
   **CR**: 718.4

6. **`test_prototype_in_graveyard_normal_chars`** -- A prototype card in graveyard
   has its normal characteristics.
   **CR**: 718.4

7. **`test_prototype_retains_abilities`** -- Cast prototyped Combat Thresher.
   Verify: still has double strike and the ETB draw trigger.
   **CR**: 718.5

8. **`test_prototype_with_commander_tax`** -- Cast prototyped spell from command zone.
   Verify: commander tax applies on top of prototype cost.
   **CR**: 118.9d (additional costs apply to prototype cost)

9. **`test_prototype_negative_not_prototype_keyword`** -- Verify that a card
   without the Prototype ability cannot be cast with `prototype: true`.
   **CR**: 702.160a

10. **`test_prototype_sba_toughness_check`** -- Cast prototyped 1/1. Apply -1/-1.
    Verify: creature dies to SBA (toughness 0).
    **CR**: 704.5f

**Pattern**: Follow tests in `/home/airbaggie/scutemob/crates/engine/tests/dash.rs` or
`/home/airbaggie/scutemob/crates/engine/tests/blitz.rs` for the casting-with-flag pattern.

### Step 8: Card Definition (later phase)

**Suggested card**: Blitz Automaton (simplest prototype card -- `{7}`, Prototype `{2}{R}` -- 3/2, Haste)
**Card lookup**: use `card-definition-author` agent

Example definition structure:
```rust
CardDefinition {
    name: "Blitz Automaton".to_string(),
    mana_cost: Some(ManaCost { generic: 7, ..Default::default() }),
    card_types: vec![CardType::Artifact, CardType::Creature].into(),
    subtypes: vec![SubType("Construct".to_string())].into(),
    power: Some(6),
    toughness: Some(4),
    abilities: vec![
        AbilityDefinition::Prototype {
            cost: ManaCost { generic: 2, red: 1, ..Default::default() },
            power: 3,
            toughness: 2,
        },
        AbilityDefinition::Keyword(KeywordAbility::Prototype),
        AbilityDefinition::Keyword(KeywordAbility::Haste),
    ],
    ..Default::default()
}
```

### Step 9: Game Script (later phase)

**Suggested scenario**: Cast Blitz Automaton normally for {7} (verify 6/4 colorless),
then in a second game, cast it prototyped for {2}{R} (verify 3/2 red with haste).
**Subsystem directory**: `test-data/generated-scripts/stack/` (casting subsystem)

## Interactions to Watch

### Layer System
- Prototype characteristics are set as BASE characteristics on the permanent at resolution
  time (CR 718.2a copiable values). This means they are NOT continuous effects and do not
  interact with the layer system in the normal sense. However, effects that set P/T
  (Humility, `SetPowerToughness` in Layer 7b) will override prototype P/T, which is
  correct (CR 613.4b: P/T-setting effects in layer 7b override base P/T).
- Color-changing effects in Layer 5 (e.g., Painter's Servant) will add colors on top of
  the prototype color, which is correct.

### Copy Effects (CR 707)
- Copy effects copy the copiable values, which INCLUDE prototype characteristics
  (CR 718.2a, 718.3c, 718.3d). Since we set prototype chars as base characteristics,
  copies automatically get the right values without special handling.
- The `is_prototyped` flag should be set on copies too, but since the base characteristics
  already reflect the prototype, this is mostly for informational purposes (TUI display).

### Alternative Costs
- Prototype + Flashback: legal. The flashback cost replaces the prototype mana cost for
  payment, but the spell still has prototype P/T and color.
- Prototype + Evoke: legal. Pay evoke cost, spell is prototyped (smaller P/T).
- Prototype + "without paying mana cost": legal. Cost is {0}, but characteristics are
  prototyped.
- The `alt_cost` field on `CastSpell` is orthogonal to `prototype`. Both can be set.

### Cost Reduction
- Convoke, Improvise, Delve, Affinity, Undaunted all reduce the effective cost. When
  prototype is active, they reduce the prototype cost (which is already set as the base
  cost in the casting pipeline).

### Commander Color Identity
- A prototype card's color identity includes BOTH the normal mana cost colors AND the
  prototype mana cost colors (the prototype ability is text on the card). This should
  be handled automatically by `commander.rs:compute_color_identity` which scans all
  AbilityDefinition data -- but verify that `AbilityDefinition::Prototype { cost, .. }`
  is scanned for its colored mana.
- **Action needed**: In `commander.rs:compute_color_identity`, add a match arm for
  `AbilityDefinition::Prototype { cost, .. }` that calls `add_colors_from_mana_cost(cost)`.

### TUI (stack_view.rs)
- No new `StackObjectKind` variant, so no TUI changes needed for exhaustive match.
- The TUI may want to display "(Prototype)" annotation, but this is cosmetic and deferred.

## Files Modified (Summary)

| File | Changes |
|------|---------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Prototype` |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Prototype { cost, power, toughness }` |
| `crates/engine/src/state/game_object.rs` | Add `is_prototyped: bool` field |
| `crates/engine/src/state/stack.rs` | Add `was_prototyped: bool` field |
| `crates/engine/src/state/hash.rs` | Hash entries for all new types/fields |
| `crates/engine/src/state/builder.rs` | Initialize `is_prototyped: false` |
| `crates/engine/src/state/mod.rs` | Reset `is_prototyped: false` on zone move (2 sites) |
| `crates/engine/src/effects/mod.rs` | Initialize `is_prototyped: false` in token creation |
| `crates/engine/src/rules/command.rs` | Add `prototype: bool` to `CastSpell` |
| `crates/engine/src/rules/casting.rs` | Prototype cost selection, `get_prototype_data()`, `colors_from_mana_cost()` |
| `crates/engine/src/rules/resolution.rs` | Apply prototype chars to permanent; initialize `is_prototyped` in all `GameObject` literals |
| `crates/engine/src/rules/engine.rs` | Pass `prototype` field through to `handle_cast_spell` |
| `crates/engine/src/rules/commander.rs` | Add `Prototype { cost, .. }` arm to `compute_color_identity` |
| `crates/engine/src/testing/script_schema.rs` | Add `prototype: bool` to `ScriptAction::PlayerAction` |
| `crates/engine/src/testing/replay_harness.rs` | Add `"cast_spell_prototype"` arm; add `prototype: false` to all existing `CastSpell` constructions |
| `crates/engine/src/cards/helpers.rs` | Verify exports (probably no changes needed) |
| `crates/engine/tests/prototype.rs` | 10 unit tests |
| `docs/mtg-engine-ability-coverage.md` | Update Prototype row: CR 702.160 (not 702.157), status `validated` |
