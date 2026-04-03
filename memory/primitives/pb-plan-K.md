# Primitive Batch Plan: PB-K -- Additional Land Drops + Case Mechanic

**Generated**: 2026-04-02
**Primitives**: (1) TriggerCondition::WheneverOpponentPlaysLand, (2) Effect::PutLandFromHandOntoBattlefield, (3) EffectFilter::LandsYouControl, (4) Designations::SOLVED + Condition::SourceIsSolved + Effect::SolveCase (Case mechanic)
**CR Rules**: 305.2, 305.3, 305.4, 305.6, 305.7, 719.3, 719.3a, 719.3b, 719.3c, 702.169
**Cards affected**: 3 new + 5 existing fixes (broken_bond, growth_spiral, spelunking, contaminant_grafter, chulane_teller_of_tales)
**Dependencies**: AdditionalLandPlays (PB-32, DONE), LandPlayed event (exists), AddSubtypes Layer 4 (exists)
**Deferred items from prior PBs**: none

## Primitive Specification

This batch adds four capabilities:

1. **TriggerCondition::WheneverOpponentPlaysLand** -- Fires when an opponent plays a land (CR 305.1). Used by Burgeoning. Note CR 305.4: "putting a land onto the battlefield" via effects does NOT trigger this. Only the special action of playing a land (CR 305.1) fires `GameEvent::LandPlayed`.

2. **Effect::PutLandFromHandOntoBattlefield** -- Moves a land card from the controller's hand onto the battlefield. Per CR 305.4, this is NOT "playing a land" and does NOT count against the land-play-per-turn limit. Used by Burgeoning's trigger resolution, and also unblocks 5 existing card defs with TODOs (Growth Spiral, Broken Bond, Spelunking, Contaminant Grafter, Chulane Teller of Tales).

3. **EffectFilter::LandsYouControl** -- Layer system filter for "Lands you control." Used by Dryad of the Ilysian Grove to add all 5 basic land types to only lands the controller owns (unlike Urborg/Yavimaya which use `AllLands`). Follows the same pattern as `CreaturesYouControl` but checks `CardType::Land` instead of `CardType::Creature`.

4. **Case Mechanic (CR 719 + 702.169)** -- Adds:
   - `Designations::SOLVED` (bit 9) on `GameObject` -- persistent designation until LTB
   - `Condition::SourceIsSolved` -- evaluates `designations.contains(Designations::SOLVED)`
   - `Effect::SolveCase` -- sets `SOLVED` designation on the source permanent
   - Card defs model "to solve" as a triggered ability (`AtBeginningOfYourEndStep` + intervening-if combining the solve condition AND `Not(SourceIsSolved)`) with effect `SolveCase`
   - "Solved" static abilities use `condition: Some(Condition::SourceIsSolved)` on `ContinuousEffectDef` or `AbilityDefinition::Triggered`

## CR Rule Text

### CR 305.2-305.4 (Land plays)
> 305.2 A player can normally play one land during their turn; however, continuous effects may increase this number.
> 305.2a To determine whether a player can play a land, compare the number of lands the player can play this turn with the number of lands they have already played this turn (including lands played as special actions and lands played during the resolution of spells and abilities). If the number of lands the player can play is greater, the play is legal.
> 305.3 A player can't play a land, for any reason, if it isn't their turn. Ignore any part of an effect that instructs a player to do so.
> 305.4 Effects may also allow players to "put" lands onto the battlefield. This isn't the same as "playing a land" and doesn't count as a land played during the current turn.

### CR 305.6-305.7 (Basic land types and mana)
> 305.6 The basic land types are Plains, Island, Swamp, Mountain, and Forest. If an object uses the words "basic land type," it's referring to one of these subtypes. An object with the land card type and a basic land type has the intrinsic ability "{T}: Add [mana symbol]," even if the text box doesn't actually contain that text or the object has no text box.
> 305.7 If an effect sets a land's subtype to one or more of the basic land types, the land no longer has its old land type. [...] If a land gains one or more land types in addition to its own, it keeps its land types and rules text, and it gains the new land types and mana abilities.

### CR 719 (Case Cards)
> 719.3 Case cards have two special keyword abilities that appear before a long dash and represent a triggered ability and an ability that may be static, triggered, or activated.
> 719.3a "To solve -- [Condition]" means "At the beginning of your end step, if [condition] and this Case is not solved, this Case becomes solved."
> 719.3b Solved is a designation a permanent can have. It has no rules meaning other than to act as a marker that spells and abilities can identify. Once a permanent becomes solved, it stays solved until it leaves the battlefield. The solved designation is neither an ability nor part of the permanent's copiable values.
> 719.3c If a Case has the solved designation, "Solved -- [Ability text]" is an ability that may affect the game if it's a static ability, it may trigger if it's a triggered ability, and it can be activated if it's an activated ability.

### CR 702.169 (Solved keyword)
> 702.169a Solved is a keyword ability found on Case cards. See rule 719.
> 702.169b For a static ability, "Solved -- [Ability text]" means "As long as this Case is solved, [ability text]."
> 702.169c For a triggered ability, "Solved -- [Ability text]" means "[Ability text]. This ability triggers only if this Case is solved."
> 702.169d For an activated ability, "Solved -- [Ability text]" means "[Ability text]. Activate only if this Case is solved."

### Burgeoning ruling (2004-10-04)
> Playing a land will trigger it, but putting a land onto the battlefield as part of an effect will not.

## Engine Changes

### Change 1: Add TriggerCondition::WheneverOpponentPlaysLand

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `TriggerCondition` enum after `WhenAnyCreatureDealsCombatDamageToOpponent` (line ~2278):
```rust
/// "Whenever an opponent plays a land" (CR 305.1).
///
/// Fires when any opponent of the trigger source's controller plays a land
/// via the special action (CR 305.1). Does NOT fire when lands are "put onto
/// the battlefield" by effects (CR 305.4). Dispatched via `GameEvent::LandPlayed`.
WheneverOpponentPlaysLand,
```
**CR**: 305.1 -- land play special action triggers this

### Change 2: Hash for WheneverOpponentPlaysLand

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add arm in `HashInto for TriggerCondition` match (after line ~4271), discriminant 40:
```rust
// CR 305.1: "Whenever an opponent plays a land" -- discriminant 40
TriggerCondition::WheneverOpponentPlaysLand => 40u8.hash_into(hasher),
```

### Change 3: Add TriggerEvent::OpponentPlaysLand

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add variant to `TriggerEvent` enum (after `AnyCreatureDealsCombatDamageToOpponent`, line ~440):
```rust
/// CR 305.1: Fires on permanents controlled by opponents of the land-playing player.
/// "Whenever an opponent plays a land" patterns use this event type.
/// The land-playing player is passed as the triggering_player parameter.
OpponentPlaysLand,
```

Also add hash arm in `HashInto for TriggerEvent` (in `state/hash.rs`, after the last TriggerEvent discriminant):
```rust
TriggerEvent::OpponentPlaysLand => <next_discriminant>u8.hash_into(hasher),
```
Check the last TriggerEvent hash discriminant and use the next value.

### Change 4: Dispatch LandPlayed in check_triggers

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add a new match arm in `check_triggers` (fn at line 2400) for `GameEvent::LandPlayed`. Place after the `PermanentEnteredBattlefield` handler block. Pattern follows `GameEvent::CardDiscarded` at line ~5322:
```rust
GameEvent::LandPlayed { player, .. } => {
    let pre_len = triggers.len();
    // CR 305.1: "Whenever an opponent plays a land"
    // Collect from all battlefield permanents controlled by opponents of the player.
    let opponent_sources: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_phased_in()
                && obj.controller != *player
        })
        .map(|obj| obj.id)
        .collect();
    for source_id in opponent_sources {
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::OpponentPlaysLand,
            Some(source_id),
            None,
        );
    }
    // Tag with triggering player for PlayerTarget resolution.
    for t in &mut triggers[pre_len..] {
        t.triggering_player = Some(*player);
    }
}
```

### Change 5: Convert TriggerCondition to TriggerEvent in enrich_spec_from_def

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add a conversion block in `enrich_spec_from_def` (after the `WheneverOpponentDiscards` block at line ~2582). This converts the `CardDefinition`'s `TriggerCondition::WheneverOpponentPlaysLand` into a runtime `TriggeredAbilityDef` with `trigger_on: TriggerEvent::OpponentPlaysLand`.

Follow the pattern of the `WheneverOpponentDiscards` block (lines 2582-2600):
```rust
// CR 305.1: Convert "Whenever an opponent plays a land" triggers.
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WheneverOpponentPlaysLand,
        effect,
        intervening_if,
        ..
    } = ability
    {
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::OpponentPlaysLand,
            intervening_if: intervening_if.as_ref().map(|c| InterveningIf {
                condition: c.clone(),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            targets: vec![],
            description: "Whenever an opponent plays a land (CR 305.1)".to_string(),
            effect: Some(effect.clone()),
        });
    }
}
```

**CRITICAL**: Without this conversion, the `TriggeredAbilityDef` is never created on the game object, and `collect_triggers_for_event` will find no matching triggers on Burgeoning. This is the #1 miss for new TriggerConditions.

### Change 6: Add Effect::PutLandFromHandOntoBattlefield

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Effect` enum after `GrantPlayerProtection` (line ~1775):
```rust
/// CR 305.4: Put a land card from the controller's hand onto the battlefield.
///
/// This is NOT "playing a land" -- it does not count against the per-turn limit.
/// Does not use the stack. The land enters as a new object (CR 400.7).
/// If no land card is in hand, the effect does nothing.
///
/// `tapped`: if true, the land enters tapped.
PutLandFromHandOntoBattlefield {
    /// If true, the land enters the battlefield tapped.
    #[serde(default)]
    tapped: bool,
},
```

### Change 7: Hash for PutLandFromHandOntoBattlefield

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add arm in `HashInto for Effect` match (after `GrantPlayerProtection`, discriminant 73), discriminant 74:
```rust
// CR 305.4: PutLandFromHandOntoBattlefield (discriminant 74) -- put land from hand
Effect::PutLandFromHandOntoBattlefield { tapped } => {
    74u8.hash_into(hasher);
    tapped.hash_into(hasher);
}
```

### Change 8: Execute Effect::PutLandFromHandOntoBattlefield

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add execution arm in the main `execute_effect` match. Logic:
1. Find a land card in the controller's hand (deterministic: pick by lowest ObjectId).
2. Call `state.move_object_to_zone(land_id, ZoneId::Battlefield)`.
3. If `tapped`, set the new object's status to tapped.
4. Run ETB hooks: `apply_self_etb_from_definition`, `apply_global_etb_replacements`, `register_static_continuous_effects`, `queue_carddef_etb_triggers` (same as `handle_play_land` in `lands.rs`, minus the `LandPlayed` event since this is NOT a land play per CR 305.4).
5. Emit `PermanentEnteredBattlefield` event (for landfall etc.) but NOT `LandPlayed` (CR 305.4).
**CR**: 305.4 -- "This isn't the same as playing a land."

### Change 9: Add EffectFilter::LandsYouControl

**File**: `crates/engine/src/state/continuous_effect.rs`
**Action**: Add variant to `EffectFilter` enum (after `OtherCreaturesYouControlWithSubtypes`, line ~201):
```rust
/// Applies to all land permanents controlled by the source's controller.
///
/// Resolved dynamically at layer-application time using `effect.source` to determine
/// the controller. Used for Dryad of the Ilysian Grove ("Lands you control are every
/// basic land type in addition to their other types.").
LandsYouControl,
```

### Change 10: Match EffectFilter::LandsYouControl in layers.rs

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Add match arm in `is_affected_by_effect` (fn at line ~500) after `OtherCreaturesYouControlWithSubtypes` (line ~871):
```rust
EffectFilter::LandsYouControl => {
    if obj_zone != ZoneId::Battlefield {
        return false;
    }
    if !chars.card_types.contains(&CardType::Land) {
        return false;
    }
    let controller = state
        .objects
        .get(&effect.source)
        .map(|obj| obj.controller);
    let obj_controller = state
        .objects
        .get(&object_id)
        .map(|obj| obj.controller);
    controller.is_some() && controller == obj_controller
}
```
**Pattern**: Follow `CreaturesYouControl` at line ~649.

### Change 11: Add Designations::SOLVED

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add flag to `Designations` bitflags (after RING_BEARER at line ~41):
```rust
/// CR 719.3b: Solved designation for Case enchantments.
/// Once a Case becomes solved, it stays solved until it leaves the battlefield.
const SOLVED         = 1 << 9;
```

### Change 12: Add Condition::SourceIsSolved

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Condition` enum (after `WasCast`, line ~2501):
```rust
/// CR 719.3b: "as long as this Case is solved" / "if this Case is solved."
///
/// True when the source permanent has the SOLVED designation.
/// Used for "Solved -- [ability]" on Case cards (CR 702.169b).
SourceIsSolved,
```

### Change 13: Hash for Condition::SourceIsSolved

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add arm in `HashInto for Condition` match (after `WasCast`, discriminant 38), discriminant 39:
```rust
// CR 719.3b: "is this Case solved?" -- discriminant 39
Condition::SourceIsSolved => 39u8.hash_into(hasher),
```

### Change 14: Evaluate Condition::SourceIsSolved

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add arm in `evaluate_condition` match:
```rust
Condition::SourceIsSolved => {
    state.objects.get(&ctx.source)
        .map(|obj| obj.designations.contains(Designations::SOLVED))
        .unwrap_or(false)
}
```

### Change 15: Add Effect::SolveCase

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Effect` enum:
```rust
/// CR 719.3a: Set the SOLVED designation on the source permanent.
///
/// Used as the effect of "to solve" triggered abilities on Case cards.
/// The intervening-if on the trigger ensures the solve condition is met
/// at both trigger and resolution time (CR 603.4).
SolveCase,
```

### Change 16: Hash for Effect::SolveCase

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add arm in `HashInto for Effect` match, discriminant 75:
```rust
// CR 719.3a: SolveCase (discriminant 75) -- set SOLVED designation
Effect::SolveCase => {
    75u8.hash_into(hasher);
}
```

### Change 17: Execute Effect::SolveCase

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add execution arm:
```rust
Effect::SolveCase => {
    if let Some(obj) = state.objects.get_mut(&ctx.source) {
        obj.designations.insert(Designations::SOLVED);
    }
}
```

### Change 18: Add Condition::And

`Condition::And` does not exist in the codebase (confirmed). It is needed for the Case's intervening-if condition which combines "7+ lands" AND "not already solved."

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add variant to `Condition` enum (after `Or`, line ~2413):
```rust
/// Logical conjunction of two conditions. True if both are true.
And(Box<Condition>, Box<Condition>),
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm in `HashInto for Condition`, discriminant 40:
```rust
Condition::And(a, b) => {
    40u8.hash_into(hasher);
    a.hash_into(hasher);
    b.hash_into(hasher);
}
```

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add evaluation arm in `evaluate_condition`:
```rust
Condition::And(a, b) => {
    evaluate_condition(a, state, ctx) && evaluate_condition(b, state, ctx)
}
```

### Change 19: Exhaustive match updates (summary)

Files requiring new match arms for new variants:

| File | Match expression | Line (approx) | Action |
|------|-----------------|------|--------|
| `cards/card_definition.rs` | `TriggerCondition` enum | L2278 | Add `WheneverOpponentPlaysLand` variant |
| `cards/card_definition.rs` | `Condition` enum | L2501 | Add `SourceIsSolved` variant |
| `cards/card_definition.rs` | `Condition` enum | L2413 | Add `And(Box, Box)` variant |
| `cards/card_definition.rs` | `Effect` enum | L1775 | Add `PutLandFromHandOntoBattlefield` + `SolveCase` variants |
| `state/game_object.rs` | `TriggerEvent` enum | L440 | Add `OpponentPlaysLand` variant |
| `state/game_object.rs` | `Designations` bitflags | L41 | Add `SOLVED = 1 << 9` |
| `state/continuous_effect.rs` | `EffectFilter` enum | L201 | Add `LandsYouControl` variant |
| `state/hash.rs` | `HashInto for TriggerCondition` | L4271 | Add discriminant 40 for `WheneverOpponentPlaysLand` |
| `state/hash.rs` | `HashInto for TriggerEvent` | check | Add discriminant for `OpponentPlaysLand` |
| `state/hash.rs` | `HashInto for Condition` | L4420 | Add discriminant 39 (`SourceIsSolved`) + 40 (`And`) |
| `state/hash.rs` | `HashInto for Effect` | L5108 | Add discriminants 74 (`PutLandFromHandOntoBattlefield`) + 75 (`SolveCase`) |
| `rules/layers.rs` | `is_affected_by_effect` | L871 | Add arm for `EffectFilter::LandsYouControl` |
| `rules/abilities.rs` | `check_triggers` | L2400 | Add `GameEvent::LandPlayed` handler |
| `testing/replay_harness.rs` | `enrich_spec_from_def` | L2582 | Add `WheneverOpponentPlaysLand` -> `OpponentPlaysLand` conversion |
| `effects/mod.rs` | `execute_effect` | various | Add `PutLandFromHandOntoBattlefield` + `SolveCase` arms |
| `effects/mod.rs` | `evaluate_condition` | various | Add `SourceIsSolved` + `And` arms |

**No new StackObjectKind**, so no changes needed in:
- `tools/replay-viewer/src/view_model.rs`
- `tools/tui/src/play/panels/stack_view.rs`

**No new KeywordAbility**, so no keyword enum changes.

**No new AbilityDefinition variant**, so no AbilityDefinition hash changes.

## Card Definition Fixes

### Existing card defs that PutLandFromHandOntoBattlefield unblocks:

### growth_spiral.rs
**Oracle text**: "Draw a card. You may put a land card from your hand onto the battlefield."
**Current state**: TODO at line 4/19 -- missing put-land effect
**Fix**: Replace the spell effect with `Effect::Sequence(vec![Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) }, Effect::PutLandFromHandOntoBattlefield { tapped: false }])`. Remove both TODO comments.

### broken_bond.rs
**Oracle text**: "Destroy target artifact or enchantment. You may put a land card from your hand onto the battlefield."
**Current state**: TODO at line 17-19 -- missing put-land effect
**Fix**: Wrap existing DestroyPermanent in Sequence with `Effect::PutLandFromHandOntoBattlefield { tapped: false }`. Remove TODO comments.

### spelunking.rs
**Oracle text**: "When this enchantment enters, draw a card, then you may put a land card from your hand onto the battlefield. If you put a Cave onto the battlefield this way, you gain 4 life. Lands you control enter untapped."
**Current state**: TODO at line 7/24 -- partial ETB (draw only), missing put-land + Cave detection
**Fix**: Add `Effect::PutLandFromHandOntoBattlefield { tapped: false }` to the ETB trigger's effect Sequence after DrawCards. The Cave-detection life gain is complex (conditional on the land being a Cave) -- mark that part with TODO (requires effect result tracking). Remove the "not expressible" TODO; add a narrow TODO for Cave conditional only.

### contaminant_grafter.rs
**Oracle text**: "...draw a card, then you may put a land card from your hand onto the battlefield."
**Current state**: TODO at line 8/47 -- missing end-step land-put effect
**Fix**: The corrupted end-step trigger's effect should be `Effect::Sequence(vec![Effect::DrawCards { ... count: 1 }, Effect::PutLandFromHandOntoBattlefield { tapped: false }])`. Remove both TODO comments.

### chulane_teller_of_tales.rs
**Oracle text**: "Whenever you cast a creature spell, draw a card, then you may put a land card from your hand onto the battlefield."
**Current state**: TODO at line 24 -- partial trigger (draw only)
**Fix**: Add `Effect::PutLandFromHandOntoBattlefield { tapped: false }` to the trigger's effect Sequence after DrawCards. Remove TODO.

## New Card Definitions

### burgeoning.rs
**Oracle text**: "Whenever an opponent plays a land, you may put a land card from your hand onto the battlefield."

```rust
CardDefinition {
    card_id: cid("burgeoning"),
    name: "Burgeoning".to_string(),
    mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
    types: types(&[CardType::Enchantment]),
    oracle_text: "Whenever an opponent plays a land, you may put a land card from your hand onto the battlefield.".to_string(),
    abilities: vec![
        AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverOpponentPlaysLand,
            effect: Effect::PutLandFromHandOntoBattlefield { tapped: false },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        },
    ],
    ..Default::default()
}
```

### dryad_of_the_ilysian_grove.rs
**Oracle text**: "You may play an additional land on each of your turns. Lands you control are every basic land type in addition to their other types."

```rust
CardDefinition {
    card_id: cid("dryad-of-the-ilysian-grove"),
    name: "Dryad of the Ilysian Grove".to_string(),
    mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
    types: full_types(&[], &[CardType::Enchantment, CardType::Creature], &["Nymph", "Dryad"]),
    oracle_text: "You may play an additional land on each of your turns.\nLands you control are every basic land type in addition to their other types.".to_string(),
    power: Some(2),
    toughness: Some(4),
    abilities: vec![
        // CR 305.2: Additional land play.
        AbilityDefinition::AdditionalLandPlays { count: 1 },
        // CR 305.7: Lands you control gain all basic land types (Layer 4).
        AbilityDefinition::Static {
            continuous_effect: ContinuousEffectDef {
                layer: EffectLayer::TypeChange,
                modification: LayerModification::AddSubtypes(
                    [
                        SubType("Plains".to_string()),
                        SubType("Island".to_string()),
                        SubType("Swamp".to_string()),
                        SubType("Mountain".to_string()),
                        SubType("Forest".to_string()),
                    ].into_iter().collect(),
                ),
                filter: EffectFilter::LandsYouControl,
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            },
        },
    ],
    ..Default::default()
}
```

### case_of_the_locked_hothouse.rs
**Oracle text**: "You may play an additional land on each of your turns. To solve -- You control seven or more lands. (If unsolved, solve at the beginning of your end step.) Solved -- You may look at the top card of your library any time, and you may play lands and cast creature and enchantment spells from the top of your library."

```rust
CardDefinition {
    card_id: cid("case-of-the-locked-hothouse"),
    name: "Case of the Locked Hothouse".to_string(),
    mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
    types: full_types(&[], &[CardType::Enchantment], &["Case"]),
    oracle_text: "You may play an additional land on each of your turns.\nTo solve \u{2014} You control seven or more lands. (If unsolved, solve at the beginning of your end step.)\nSolved \u{2014} You may look at the top card of your library any time, and you may play lands and cast creature and enchantment spells from the top of your library.".to_string(),
    abilities: vec![
        // CR 305.2: Additional land play (unsolved, always active).
        AbilityDefinition::AdditionalLandPlays { count: 1 },
        // CR 719.3a: "To solve -- You control seven or more lands."
        // Triggers at beginning of end step with intervening-if.
        AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
            effect: Effect::SolveCase,
            intervening_if: Some(Condition::And(
                Box::new(Condition::YouControlNOrMoreWithFilter {
                    count: 7,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                }),
                Box::new(Condition::Not(Box::new(Condition::SourceIsSolved))),
            )),
            targets: vec![],
            modes: None,
            trigger_zone: None,
        },
        // CR 702.169b: "Solved -- You may look at the top card of your library any time,
        // and you may play lands and cast creature and enchantment spells from the top of
        // your library."
        // TODO: Solved play-from-top ability -- requires PB-A (play from top of library).
        // The SourceIsSolved condition is correct; the effect itself needs the play-from-top
        // engine primitive which is HIGH complexity (PB-A territory).
    ],
    ..Default::default()
}
```

## Card Registry Registration

All 3 new card defs must be registered in `crates/engine/src/cards/defs/mod.rs`:
- `pub mod burgeoning;`
- `pub mod dryad_of_the_ilysian_grove;`
- `pub mod case_of_the_locked_hothouse;`

And added to the registry function.

## Unit Tests

**File**: `crates/engine/tests/pb_k_land_drops.rs`
**Tests to write**:
- `test_burgeoning_trigger_on_opponent_land_play` -- CR 305.1: place Burgeoning, have opponent play a land, verify trigger fires and a land from controller's hand is put onto battlefield
- `test_burgeoning_no_trigger_on_own_land_play` -- Verify Burgeoning does NOT trigger on the controller's own land play
- `test_burgeoning_no_trigger_on_put_land` -- CR 305.4: verify that PutLandFromHandOntoBattlefield by another effect does NOT trigger Burgeoning (it is not "playing" a land)
- `test_put_land_does_not_count_as_land_play` -- CR 305.4: PutLandFromHandOntoBattlefield does not increment land_plays_this_turn
- `test_put_land_triggers_landfall` -- PutLandFromHandOntoBattlefield emits PermanentEnteredBattlefield, triggering landfall
- `test_dryad_additional_land_play` -- CR 305.2: verify controller can play 2 lands with Dryad
- `test_dryad_lands_have_all_basic_types` -- CR 305.7: verify controller's lands gain Plains+Island+Swamp+Mountain+Forest subtypes
- `test_dryad_opponent_lands_unaffected` -- LandsYouControl filter: opponent's lands do NOT gain types
- `test_dryad_lands_keep_original_types` -- CR 305.7: "in addition to" -- verify original subtypes preserved
- `test_case_solve_at_end_step` -- CR 719.3a: Case with 7+ lands solves at end step
- `test_case_no_solve_without_condition` -- Case with <7 lands does NOT solve
- `test_case_already_solved_no_retrigger` -- Solved Case does not trigger again
- `test_solve_case_designation_persists` -- Solved stays until LTB (CR 719.3b)
- `test_condition_source_is_solved` -- SourceIsSolved evaluates correctly
- `test_condition_and_both_true` -- Condition::And returns true only when both arms true

**Pattern**: Follow tests for similar features in `tests/` (e.g., `pb_32_additional_land_plays.rs` or similar).

## Verification Checklist

- [ ] Engine primitives compile (`cargo check`)
- [ ] All 5 existing card def TODOs for this batch resolved (growth_spiral, broken_bond, spelunking, contaminant_grafter, chulane_teller_of_tales)
- [ ] 3 new card defs authored (burgeoning, dryad_of_the_ilysian_grove, case_of_the_locked_hothouse)
- [ ] Card defs registered in `defs/mod.rs`
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs (except Case solved play-from-top, explicitly marked PB-A)

## Risks & Edge Cases

- **PutLandFromHandOntoBattlefield must run ETB hooks**: The land enters the battlefield just like any other permanent -- ETB replacements (enter tapped), static continuous effects registration, and ETB trigger queueing must all fire. This is the same set of hooks as `handle_play_land` in `lands.rs` (lines ~82-385). Failing to call these would cause lands to enter without triggering landfall, without entering tapped when they should, etc. Consider extracting a shared helper from `lands.rs`.

- **Deterministic land selection**: When "putting a land from hand onto the battlefield," the engine must pick deterministically (lowest ObjectId land card in hand). This is a bot/deterministic-fallback behavior. In a real game with human players, this would require a player choice (Command::SelectCard). For now, auto-pick is correct (same approach as SearchLibrary deterministic fallback).

- **CR 305.3**: Burgeoning puts lands onto the battlefield during an opponent's turn. This is legal because it's "putting" (CR 305.4), not "playing" (CR 305.3 only restricts playing). No sorcery-speed restriction check needed.

- **Dryad + Blood Moon interaction**: Blood Moon sets nonbasic lands to Mountains (SetTypeLine in Layer 4). Dryad adds all basic types (AddSubtypes in Layer 4). The existing dependency system (Change N/A -- already handles SetTypeLine depending on AddSubtypes) ensures Dryad applies first, then Blood Moon overrides. Result: nonbasic lands controlled by Dryad's owner become Mountains only (correct per CR 613.8).

- **Case solved designation and zone changes**: CR 719.3b says "Once a permanent becomes solved, it stays solved until it leaves the battlefield." The `Designations` bitflags are reset on zone change (CR 400.7 -- new object identity), so this is automatic. No special cleanup needed.

- **enrich_spec_from_def is the #1 miss site**: The `TriggerCondition` -> `TriggerEvent` conversion in `enrich_spec_from_def` (replay_harness.rs) is **mandatory** for any new TriggerCondition. Without it, the game object will have no `TriggeredAbilityDef` matching the new `TriggerEvent`, and `collect_triggers_for_event` will silently find nothing. The runner must add the conversion block in Change 5 or Burgeoning will never trigger.

- **Case play-from-top is deferred**: The solved ability of Case of the Locked Hothouse requires play-from-top-of-library, which is PB-A (HIGH complexity). The card def will have an explicit TODO for this. The solve mechanic itself is fully functional.
