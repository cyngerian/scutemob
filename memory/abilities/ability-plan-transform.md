# Ability Plan: Transform Mini-Milestone

**Generated**: 2026-03-08
**CR**: 701.27 (Transform), 712 (Double-Faced Cards), 702.145 (Daybound/Nightbound), 702.146 (Disturb), 702.167 (Craft), 730 (Day and Night)
**Priority**: P3 (Transform) + P4 (Disturb, Daybound/Nightbound, Craft)
**Similar abilities studied**: Mutate (merged_cards on GameObject, zone-change splitting), Unearth (replacement effect for zone-change redirect), Prototype (alternate characteristics on a single CardDefinition), Bestow (state flag changing type behavior)
**Discriminant chain start**: KW 148, AbilDef 60, SOK 60

## CR Rule Text

### 701.27 — Transform

> 701.27a To transform a permanent, turn it over so that its other face is up. Only permanents represented by double-faced tokens and double-faced cards can transform.

> 701.27b Although transforming a permanent uses the same physical action as turning a permanent face up or face down, they are different game actions.

> 701.27c If a spell or ability instructs a player to transform a permanent that isn't represented by a double-faced token or a double-faced card, nothing happens.

> 701.27d If a spell or ability instructs a player to transform a permanent, and the face that permanent would transform into is an instant or sorcery face, nothing happens.

> 701.27e Some triggered abilities trigger when an object "transforms into" an object with a specified characteristic. Such an ability triggers if the object either transforms or converts and has the specified characteristic immediately after it does so.

> 701.27f If an activated or triggered ability of a permanent that isn't a delayed triggered ability of that permanent tries to transform it, the permanent does so only if it hasn't transformed or converted since the ability was put onto the stack. If a delayed triggered ability of a permanent tries to transform that permanent, the permanent does so only if it hasn't transformed or converted since that delayed triggered ability was created. In both cases, if the permanent has already transformed or converted, an instruction to do either is ignored.

> 701.27g Some spells and abilities refer to a "transformed permanent." This phrase refers to a double-faced permanent on the battlefield with its back face up. A permanent with its front face up is never considered a transformed permanent.

### 712 — Double-Faced Cards (key sections)

> 712.1 A double-faced card has a Magic card face on one side and either a Magic card face or half of an oversized card face on the other.

> 712.8a While a double-faced card is outside the game or in a zone other than the battlefield or stack, it has only the characteristics of its front face.

> 712.8c Normally, a nonmodal double-faced spell has its front face up while on the stack. However, if an effect allows a player to cast a nonmodal double-faced card "transformed" or "converted," the resulting spell will have its back face up and have only the characteristics of its back face. Its mana value is calculated using the mana cost of its front face.

> 712.8d While a double-faced permanent has its front face up, it has only the characteristics of its front face.

> 712.8e While a nonmodal double-faced permanent has its back face up, it has only the characteristics of its back face. However, its mana value is calculated using the mana cost of its front face.

> 712.9 Only permanents represented by double-faced tokens and double-faced cards that are not meld cards can transform or convert.

> 712.10 If a spell or ability instructs a player to transform or convert a permanent, and the face that permanent would transform or convert into is an instant or sorcery card face, nothing happens.

> 712.11a If a double-faced card or a copy of a double-faced card is cast as a spell "transformed" or "converted," it's put on the stack with its back face up.

> 712.13 By default, a resolving double-faced spell that becomes a permanent is put onto the battlefield with the same face up that was face up on the stack.

> 712.14 A double-faced card put onto the battlefield from a zone other than the stack enters the battlefield with its front face up by default.

> 712.14a If a spell or ability puts a double-faced card onto the battlefield "transformed" or "converted," it enters the battlefield with its back face up.

> 712.18 When a double-faced permanent transforms or converts, it doesn't become a new object. Any effects that applied to that permanent will continue to apply to it.

> 712.20 If a double-faced card would have an "As [this permanent] transforms..." ability after it transforms or converts, that ability is applied while that permanent is transforming, not afterward.

### 702.145 — Daybound and Nightbound

> 702.145a Daybound and nightbound are found on opposite faces of some double-faced cards.

> 702.145b "Daybound" means "If it is night and this permanent is represented by a double-faced card, it enters transformed," "As it becomes night, if this permanent is front face up, transform it," and "This permanent can't transform except due to its daybound ability."

> 702.145c Any time a player controls a permanent that is front face up with daybound and it's night, that player transforms that permanent. This happens immediately and isn't a state-based action.

> 702.145d Any time a player controls a permanent with daybound, if it's neither day nor night, it becomes day.

> 702.145e "Nightbound" means "As it becomes day, if this permanent is back face up, transform it" and "This permanent can't transform except due to its nightbound ability."

> 702.145f Any time a player controls a permanent that is back face up with nightbound and it's day, that player transforms that permanent. This happens immediately and isn't a state-based action.

> 702.145g Any time a player controls a permanent with nightbound, if it's neither day nor night and there are no permanents with daybound on the battlefield, it becomes night.

### 730 — Day and Night

> 730.1 Day and night are designations that the game itself can have. The game starts with neither designation. Once it has become day or night, the game will have exactly one of those designations from that point forward.

> 730.2 As the second part of the untap step, the game checks the previous turn to see if the game's day/night designation should change.

> 730.2a If it's day and the previous turn's active player didn't cast any spells during that turn, it becomes night.

> 730.2b If it's night, and previous turn's active player cast two or more spells during the previous turn, it becomes day.

> 730.2c If it's neither day nor night, this check doesn't happen and it remains neither.

### 702.146 — Disturb

> 702.146a "Disturb [cost]" means "You may cast this card transformed from your graveyard by paying [cost] rather than its mana cost."

> 702.146b A resolving double-faced spell that was cast using its disturb ability enters the battlefield with its back face up.

Ruling: "The back face of each card with disturb has an ability that instructs its controller to exile if it would be put into a graveyard from anywhere."

### 702.167 — Craft

> 702.167a "Craft with [materials] [cost]" means "[Cost], Exile this permanent, Exile [materials] from among permanents you control and/or cards in your graveyard: Return this card to the battlefield transformed under its owner's control. Activate only as a sorcery."

> 702.167b If an object in the [materials] is described using only a card type or subtype without "card," it refers to either a permanent on the battlefield or a card in a graveyard of that type.

> 702.167c An ability of a permanent may refer to the exiled cards used to craft it.

## Key Edge Cases

1. **Transform does NOT create a new object (CR 712.18)** -- continuous effects, counters, damage, attachments all persist. This is the opposite of zone changes.
2. **Mana value of back face uses front face's mana cost (CR 712.8e)** -- critical for CMC-based effects (Collected Company, cascade, etc.)
3. **DFC in non-battlefield/non-stack zones has ONLY front face characteristics (CR 712.8a)** -- searching library for a back-face name fails.
4. **Copy effects on DFCs copy the face that's currently up (CR 707.8)** -- Clone copying a transformed Delver gets Insectile Aberration's characteristics.
5. **Token copies of DFCs create double-faced tokens (CR 707.8a)** -- they can transform.
6. **Daybound/Nightbound prevents ALL other transform effects (CR 702.145b/e)** -- Moonmist can't transform Brutal Cathar.
7. **Day/night check uses PREVIOUS turn's spell count (CR 730.2a/b)** -- not current turn.
8. **Disturb back face's "exile if would go to graveyard from anywhere" (ruling)** -- includes being countered from the stack, not just dying.
9. **Craft activated ability exiles self + materials as cost (CR 702.167a)** -- returns transformed. If the card isn't a DFC, it stays in exile.
10. **701.27f transform-once guard** -- if a non-delayed triggered ability tries to transform a permanent that already transformed since the ability was put on the stack, the instruction is ignored.
11. **Multiplayer day/night** -- uses shared team turns variant for multiplayer (CR 730.2a/b). In Commander (free-for-all), each player has their own turn, so the standard rule applies: check previous turn's active player's spell count.

## Current State (from ability-wip.md)

- [ ] 1. Plan -- CR research, DFC model design, implementation plan
- [ ] 2. Implement -- DFC model, Transform action, Daybound/Nightbound, Disturb, Craft, tests
- [ ] 3. Review -- verify against CR, edge cases
- [ ] 4. Fix -- apply review findings
- [ ] 5. Cards -- card definitions for Transform cards
- [ ] 6. Scripts -- game scripts for validation
- [ ] 7. Close -- update coverage doc, CLAUDE.md, workstream state

## DFC Data Model Design

### Design Decision: Back Face on CardDefinition

The core question is: where does the back face's data live?

**Chosen approach: `back_face: Option<CardFace>` on `CardDefinition`.**

Rationale:
- A `CardDefinition` already represents a single card. DFCs are a single card with two faces.
- The front face data stays in the existing fields (name, mana_cost, types, abilities, power, toughness).
- A new `back_face: Option<CardFace>` field holds the back face's characteristics and abilities.
- This is analogous to how `Aftermath` already stores a second half's data on the same `CardDefinition`.
- It avoids creating a second `CardDefinition` with a different `CardId` (which would break the 1:1 card-to-definition mapping).

```rust
/// The back face of a double-faced card (CR 712).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardFace {
    pub name: String,
    pub mana_cost: Option<ManaCost>,
    pub types: TypeLine,
    pub oracle_text: String,
    pub abilities: Vec<AbilityDefinition>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    /// Color indicator (CR 204) -- used by back faces that have no mana cost
    /// but need a color identity (e.g., Insectile Aberration is blue via indicator).
    pub color_indicator: Option<Vec<Color>>,
}
```

### Design Decision: Transform State on GameObject

**Chosen approach: `is_transformed: bool` on `GameObject`.**

- When `false` (default), the permanent uses its front face characteristics.
- When `true`, the permanent uses its back face characteristics.
- The transform action flips this boolean. Per CR 712.18, no new object is created.
- The layer system reads `is_transformed` + `card_registry.get(card_id).back_face` to determine base characteristics.
- Reset to `false` on zone changes (CR 400.7 -- new object rule), EXCEPT: a spell cast "transformed" (Disturb) enters with `is_transformed = true`.

### Design Decision: Day/Night State on GameState

**Chosen approach: `day_night: Option<DayNight>` on `GameState`.**

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DayNight {
    Day,
    Night,
}
```

- `None` = neither day nor night (game start).
- `Some(Day)` / `Some(Night)` = current designation.
- Once set, it toggles but never returns to `None` (CR 730.1).
- Checked in the untap step (CR 730.2) and whenever a daybound/nightbound permanent enters or a player controls one.

### Design Decision: How Transform Integrates with Layer System

In `calculate_characteristics()` at the TOP (before the layer loop), when `obj.is_transformed` is `true`:
1. Look up `card_registry.get(obj.card_id)` to find the `CardDefinition`.
2. If `def.back_face` is `Some(face)`, replace `chars` with characteristics built from the back face.
3. If `def.back_face` is `None`, `is_transformed` is meaningless (CR 701.27c -- non-DFC can't transform).

This runs at the same priority as the existing merged_components Layer 1 integration (line 80 of `layers.rs`). It must run BEFORE the merged_components check, because a merged DFC permanent uses the topmost component's characteristics, which themselves may be transformed.

### Design Decision: Tracking `transforms_this_stack_cycle` for 701.27f

CR 701.27f requires tracking whether a permanent has transformed since a specific ability was put on the stack. Simple approach: add `last_transform_timestamp: u64` on `GameObject`. When transforming, set this to `state.timestamp_counter`. When a triggered/activated ability tries to transform the permanent, compare the ability's stack entry timestamp against `last_transform_timestamp`. If `last_transform_timestamp >= ability_timestamp`, the transform is ignored.

## Modification Surface (from rust-analyzer and grep)

Files and functions that need changes, mapped via similar abilities (Mutate, Prototype, Unearth):

| File | Function/Match | Line | What to add |
|------|---------------|------|-------------|
| `cards/card_definition.rs` | `CardDefinition` struct | L28 | `back_face: Option<CardFace>` field |
| `cards/card_definition.rs` | `Default for CardDefinition` | L47 | `back_face: None` |
| `cards/card_definition.rs` | `AbilityDefinition` enum | after L688 | `Disturb { cost: ManaCost }` (disc 60), `Craft { cost: ManaCost, materials: CraftMaterials }` (disc 61) |
| `state/types.rs` | `KeywordAbility` enum | after Mutate | `Transform` (disc 148), `Daybound` (disc 149), `Nightbound` (disc 150), `Disturb` (disc 151), `Craft` (disc 152) |
| `state/types.rs` | `AltCostKind` enum | after Mutate | `Disturb` variant |
| `state/game_object.rs` | `GameObject` struct | after `merged_components` | `is_transformed: bool`, `last_transform_timestamp: u64`, `was_cast_disturbed: bool` (for exile-if-would-die replacement) |
| `state/mod.rs` | `GameState` struct | after `forecast_used_this_turn` | `day_night: Option<DayNight>`, `previous_turn_spells_cast: u32` |
| `state/mod.rs` | `DayNight` enum | new type | `Day`, `Night` |
| `state/hash.rs` | `HashInto for GameObject` | near end | Hash `is_transformed`, `last_transform_timestamp`, `was_cast_disturbed` |
| `state/hash.rs` | `HashInto for GameState` | near end | Hash `day_night`, `previous_turn_spells_cast` |
| `state/hash.rs` | `HashInto for KeywordAbility` | match arms | Add 5 new KW variants |
| `state/hash.rs` | `HashInto for AbilityDefinition` | match arms | Add `Disturb`, `Craft` variants |
| `state/hash.rs` | `HashInto for StackObjectKind` | match arms | Add `TransformTrigger`, `CraftAbility`, `DayboundTransformTrigger` |
| `state/hash.rs` | `HashInto for AltCostKind` | match arms | Add `Disturb` |
| `state/builder.rs` | `GameStateBuilder::build()` | game state init | `day_night: None`, `previous_turn_spells_cast: 0` |
| `state/builder.rs` | object init blocks | all places creating objects | `is_transformed: false`, `last_transform_timestamp: 0`, `was_cast_disturbed: false` |
| `state/stack.rs` | `StackObjectKind` enum | after `MutatingCreatureSpell` | `TransformTrigger` (disc 60), `CraftAbility` (disc 61), `DayboundTransformTrigger` (disc 62) |
| `rules/layers.rs` | `calculate_characteristics()` | L37-39 | DFC face resolution: if `is_transformed` and back_face exists, swap base chars |
| `rules/layers.rs` | mana value calculation | wherever mana_value used | Back face uses front face's mana cost (CR 712.8e) |
| `rules/casting.rs` | `handle_cast_spell()` | alt cost handling | Disturb: cast from graveyard, put on stack with back face up |
| `rules/replacement.rs` | `check_zone_change_replacements()` | after unearth check ~L596 | Disturb: if `was_cast_disturbed`, exile instead of graveyard from anywhere |
| `rules/turn_actions.rs` | `untap_active_player_permanents()` | before phasing (L997) | Day/night check (CR 730.2) |
| `rules/turn_actions.rs` | `reset_turn_state()` | turn reset | Set `previous_turn_spells_cast` from current player's `spells_cast_this_turn` |
| `rules/resolution.rs` | spell resolution | permanent enters battlefield | Propagate `is_transformed` from StackObject if cast transformed |
| `rules/engine.rs` | `process_command()` | Command match | Add `Command::Transform`, `Command::ActivateCraft` |
| `rules/command.rs` | `Command` enum | new variants | `Transform { player, permanent }`, `ActivateCraft { player, source, materials, mana_paid }` |
| `rules/events.rs` | `GameEvent` enum | new variants | `PermanentTransformed`, `DayNightChanged`, `CraftActivated` |
| `rules/abilities.rs` | `check_triggers()` | trigger dispatch | `TransformTrigger` resolution, daybound/nightbound immediate transform |
| `rules/sba.rs` | SBA checks | after existing checks | Daybound/nightbound immediate transform enforcement (or inline -- CR says "not an SBA" but "happens immediately") |
| `effects/mod.rs` | `execute_effect()` | effect match | `Effect::Transform` variant execution |
| `cards/helpers.rs` | prelude exports | use crate | Export `CardFace`, `CraftMaterials`, `DayNight` |
| `testing/replay_harness.rs` | `enrich_spec_from_def()` | enrichment | Handle DFC back face enrichment when `is_transformed` |
| `testing/replay_harness.rs` | `translate_player_action()` | action types | `transform`, `activate_craft` harness actions |
| `testing/script_schema.rs` | `PlayerAction` struct | new fields | `craft_materials`, etc. |
| `tools/replay-viewer/src/view_model.rs` | `stack_kind_info()` SOK match | exhaustive match | Add `TransformTrigger`, `CraftAbility`, `DayboundTransformTrigger` |
| `tools/replay-viewer/src/view_model.rs` | KW display match | exhaustive match | Add `Transform`, `Daybound`, `Nightbound`, `Disturb`, `Craft` |
| `tools/tui/src/play/panels/stack_view.rs` | SOK match | exhaustive match | Add 3 new SOK variants |
| `lib.rs` | public exports | re-exports | Export `CardFace`, `DayNight`, `CraftMaterials` |

## Implementation Steps

### Step 1: DFC Data Model (CardDefinition + GameObject + GameState)

**Files**: `crates/engine/src/cards/card_definition.rs`, `crates/engine/src/state/game_object.rs`, `crates/engine/src/state/mod.rs`, `crates/engine/src/state/types.rs`

**Action A — CardFace struct and back_face field:**
- Add `CardFace` struct (name, mana_cost, types, oracle_text, abilities, power, toughness, color_indicator) in `card_definition.rs`
- Add `back_face: Option<CardFace>` to `CardDefinition` struct
- Add `back_face: None` to `Default for CardDefinition`
- Add `..Default::default()` note: existing card defs need no change (back_face defaults to None via serde)

**Action B — DayNight enum:**
- Add `DayNight` enum (`Day`, `Night`) in `state/types.rs` or `state/mod.rs` (prefer mod.rs since it's game-level state)
- Add `day_night: Option<DayNight>` to `GameState`
- Add `previous_turn_spells_cast: u32` to `GameState` (tracks previous turn's active player's spell count for CR 730.2)
- Initialize both in `GameStateBuilder::build()` and `Default`

**Action C — KeywordAbility variants:**
- Add `Transform` (disc 148), `Daybound` (disc 149), `Nightbound` (disc 150), `Disturb` (disc 151), `Craft` (disc 152) to `KeywordAbility` enum

**Action D — AltCostKind variant:**
- Add `Disturb` to `AltCostKind` enum

**Action E — AbilityDefinition variants:**
- Add `Disturb { cost: ManaCost }` (disc 60)
- Add `Craft { cost: ManaCost, materials: CraftMaterials }` (disc 61)
- Define `CraftMaterials` enum to describe what can be exiled (e.g., `CraftMaterials::CardType(CardType)`, `CraftMaterials::Artifact`, etc.)

**Action F — GameObject fields:**
- Add `is_transformed: bool` (default false, reset on zone changes)
- Add `last_transform_timestamp: u64` (default 0, for CR 701.27f guard)
- Add `was_cast_disturbed: bool` (default false, for exile-if-would-die replacement; reset on zone changes)
- Add these to ALL object initialization sites (builder.rs, effects/mod.rs token creation, resolution.rs)

**Action G — StackObjectKind variants:**
- Add `TransformTrigger` (disc 60) -- for card-defined transform triggers
- Add `CraftAbility` (disc 61) -- for craft activated ability on the stack
- Add `DayboundTransformTrigger` (disc 62) -- for daybound/nightbound-related triggers if needed

**Action H — Hash updates:**
- Add all new fields to `state/hash.rs` (GameObject, GameState, KeywordAbility, AbilityDefinition, StackObjectKind, AltCostKind)

**Action I — Exhaustive match updates:**
- `tools/replay-viewer/src/view_model.rs`: both SOK and KW matches
- `tools/tui/src/play/panels/stack_view.rs`: SOK match

**CR**: 712.1, 712.2, 701.27, 702.145a, 702.146a, 702.167a, 730.1

### Step 2: Layer System — DFC Face Resolution

**File**: `crates/engine/src/rules/layers.rs`

**Action**: At the top of `calculate_characteristics()` (around line 37-39, BEFORE the merged_components check at line 80):

1. If `obj.is_transformed` is true AND `obj.card_id` is Some AND the card_registry has a back_face for that card_id:
   - Build `Characteristics` from the back face data (name, mana_cost from back face, types, abilities, power, toughness, color from color_indicator or mana_cost)
   - Replace `chars` with these back-face characteristics
   - **IMPORTANT**: mana_cost on the characteristics should be the BACK face's mana_cost for color derivation, but `mana_value()` calls elsewhere must use the front face's mana cost (CR 712.8e). This can be handled by storing `front_face_mana_cost` or by always computing mana_value from the CardDefinition, not from characteristics.

2. If `obj.is_transformed` is true but the card has no back_face, ignore (CR 701.27c).

3. This must run BEFORE the merged_components check because a mutated DFC's topmost component might itself be transformed.

4. For non-battlefield zones (CR 712.8a): DFCs in hand/library/graveyard/exile always use front face characteristics. The `is_transformed` flag is reset on zone changes, so this is automatic. But verify that `calculate_characteristics` for non-battlefield objects returns front-face chars.

**CR**: 712.8a, 712.8d, 712.8e, 712.18

### Step 3: Transform Action (Command + Effect)

**File**: `crates/engine/src/rules/engine.rs`, `crates/engine/src/rules/command.rs`, `crates/engine/src/effects/mod.rs`, `crates/engine/src/rules/events.rs`

**Action A — Command::Transform:**
- New command variant: `Transform { player: PlayerId, permanent: ObjectId }`
- Handler in `engine.rs`:
  1. Validate permanent is on battlefield, controlled by player
  2. Validate permanent is a DFC (has back_face in card_registry)
  3. Check CR 701.27d: back face can't be instant/sorcery
  4. Check CR 702.145b/e: if permanent has daybound/nightbound, only daybound/nightbound can transform it (reject manual transform commands)
  5. Flip `obj.is_transformed`
  6. Set `obj.last_transform_timestamp = state.timestamp_counter`
  7. Emit `GameEvent::PermanentTransformed { object_id, to_back_face: bool }`
  8. Call `check_triggers()` + `flush_pending_triggers()`

**Action B — Effect::Transform:**
- New effect variant: `Effect::Transform { target: EffectTarget }`
- In `execute_effect()`: resolve target, call the same transform logic as the command handler
- Used by card-defined transform triggers (e.g., Delver of Secrets: "transform if top of library is instant/sorcery")

**Action C — GameEvent::PermanentTransformed:**
- New event: `PermanentTransformed { object_id: ObjectId, to_back_face: bool }`
- Used by trigger dispatch (CR 701.27e: "transforms into")

**CR**: 701.27a-f, 712.18

### Step 4: Daybound/Nightbound Enforcement

**Files**: `crates/engine/src/rules/turn_actions.rs`, `crates/engine/src/rules/engine.rs`, `crates/engine/src/rules/abilities.rs`

**Action A — Day/Night Check in Untap Step (CR 730.2):**
In `untap_active_player_permanents()`, BEFORE phasing (around line 997):
1. If `state.day_night` is `Some(Day)` and `state.previous_turn_spells_cast == 0`, set `state.day_night = Some(Night)` and emit `GameEvent::DayNightChanged { now: Night }`
2. If `state.day_night` is `Some(Night)` and `state.previous_turn_spells_cast >= 2`, set `state.day_night = Some(Day)` and emit `GameEvent::DayNightChanged { now: Day }`
3. If `state.day_night` is `None`, skip (CR 730.2c)

**Action B — Track Previous Turn's Spells:**
In `reset_turn_state()` (or wherever turns advance):
- Before resetting `spells_cast_this_turn`, save it: `state.previous_turn_spells_cast = state.players.get(&state.turn.active_player).map(|p| p.spells_cast_this_turn).unwrap_or(0)`

**Action C — Daybound/Nightbound Immediate Transform:**
Per CR 702.145c/f, this happens "immediately and isn't a state-based action." This must be checked:
1. Whenever day/night changes (after the GameEvent::DayNightChanged)
2. Whenever a daybound/nightbound permanent enters the battlefield
3. Whenever day_night is set for the first time

Implementation: a helper function `enforce_daybound_nightbound(state)` that scans all battlefield permanents:
- If it's night and a permanent has daybound (front face up), transform it
- If it's day and a permanent has nightbound (back face up), transform it
- If it's neither day nor night and a permanent has daybound, set day_night = Some(Day) (CR 702.145d)
- If it's neither day nor night and a permanent has nightbound and no daybound permanents exist, set day_night = Some(Night) (CR 702.145g)

Call this function:
- After `DayNightChanged` event in untap step
- After any permanent enters the battlefield (in resolution.rs and lands.rs ETB sites)
- After any transform effect resolves

**Action D — Daybound/Nightbound Transform Lock:**
CR 702.145b/e: permanents with daybound/nightbound can't transform except via their daybound/nightbound abilities. In the Transform command handler and Effect::Transform execution, check: if the permanent has daybound or nightbound keyword, reject the transform unless it's being done by the daybound/nightbound enforcement system. Use a flag or separate code path.

**CR**: 702.145b-g, 730.1-2

### Step 5: Disturb — Cast Transformed from Graveyard

**Files**: `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/resolution.rs`, `crates/engine/src/rules/replacement.rs`

**Action A — Casting with Disturb:**
In `handle_cast_spell()`:
1. When `alt_cost == Some(AltCostKind::Disturb)`:
   - Card must be in caster's graveyard
   - Card must have `AbilityDefinition::Disturb { cost }` -- use that cost
   - Card must have a back_face in its CardDefinition
   - Put on stack with back face characteristics (CR 712.11a, 702.146a)
   - Set `stack_object.is_cast_transformed = true` (new field on StackObject)
   - Mana value on the stack = front face's mana cost (CR 712.8c)

**Action B — Resolution with Disturb:**
In spell resolution:
- If `stack_object.is_cast_transformed`, the permanent enters with `is_transformed = true`
- Set `obj.was_cast_disturbed = true` on the resulting permanent
- This enables the replacement effect

**Action C — Disturb Exile Replacement:**
In `check_zone_change_replacements()` (replacement.rs), after the unearth check:
- If `obj.was_cast_disturbed` and the object would go to the graveyard from ANY zone:
  - Redirect to exile instead (ZoneChangeAction::Redirect { to: ZoneId::Exile })
  - This covers: dying from battlefield, being countered from stack, discarded, milled
- Pattern: identical to the `was_unearthed` check but with `was_cast_disturbed`

**Action D — StackObject field:**
- Add `is_cast_transformed: bool` to `StackObject` (default false)
- Propagated at resolution to `GameObject.is_transformed`

**CR**: 702.146a-b, 712.8c, 712.11a, 712.13

### Step 6: Craft — Activated Ability

**Files**: `crates/engine/src/rules/command.rs`, `crates/engine/src/rules/engine.rs`, `crates/engine/src/rules/resolution.rs`

**Action A — Command::ActivateCraft:**
New command: `ActivateCraft { player: PlayerId, source: ObjectId, materials: Vec<ObjectId>, mana_cost: ManaCost }`

Handler:
1. Validate source is on battlefield, controlled by player, has `AbilityDefinition::Craft`
2. Validate materials match the craft definition's requirements (CR 702.167b: permanents on battlefield OR cards in graveyard of the specified type)
3. Validate timing (sorcery speed, CR 702.167a: "Activate only as a sorcery")
4. Pay mana cost
5. Exile the source permanent (as cost)
6. Exile all material objects (as cost)
7. Return the card to the battlefield transformed (CR 702.167a)
   - The card is now in exile -- move it to battlefield with `is_transformed = true`
   - Track the exiled material ObjectIds on the permanent for abilities that reference "cards used to craft" (CR 702.167c)

**Action B — Craft Materials Tracking:**
- Add `craft_exiled_cards: Vec<ObjectId>` to `GameObject` (tracks materials for CR 702.167c)
- Reset on zone changes (CR 400.7)

**Action C — Non-DFC Craft Guard:**
Per ruling: "If a card that isn't a transforming double-faced card becomes a copy of a card with craft, it'll stay in exile if you activate the craft ability." If the card has no back_face, it stays in exile and doesn't return to the battlefield.

**CR**: 702.167a-c

### Step 7: Unit Tests

**File**: `crates/engine/tests/transform.rs` (new), `crates/engine/tests/daybound.rs` (new), `crates/engine/tests/disturb.rs` (new), `crates/engine/tests/craft.rs` (new)

**Tests to write (transform.rs):**
- `test_transform_basic_flip` -- CR 701.27a: permanent flips face, characteristics change
- `test_transform_preserves_counters` -- CR 712.18: counters survive transform
- `test_transform_preserves_continuous_effects` -- CR 712.18: CEs persist
- `test_transform_preserves_damage` -- CR 712.18: damage persists
- `test_transform_preserves_attachments` -- CR 712.18: auras/equipment stay
- `test_transform_no_new_object` -- CR 712.18: ObjectId doesn't change
- `test_transform_non_dfc_does_nothing` -- CR 701.27c
- `test_transform_instant_sorcery_back_does_nothing` -- CR 701.27d
- `test_transform_dfc_mana_value_uses_front_face` -- CR 712.8e
- `test_transform_dfc_zones_use_front_face` -- CR 712.8a: hand/library/graveyard
- `test_transform_once_guard` -- CR 701.27f: ability can't re-transform
- `test_transform_copy_uses_current_face` -- CR 707.8

**Tests to write (daybound.rs):**
- `test_daybound_sets_day_on_etb` -- CR 702.145d
- `test_nightbound_sets_night_on_etb` -- CR 702.145g (no daybound exists)
- `test_day_to_night_no_spells` -- CR 730.2a
- `test_night_to_day_two_spells` -- CR 730.2b
- `test_daybound_transforms_at_night` -- CR 702.145c
- `test_nightbound_transforms_at_day` -- CR 702.145f
- `test_daybound_blocks_other_transform` -- CR 702.145b
- `test_daybound_enters_transformed_at_night` -- CR 702.145b: "enters transformed"
- `test_day_night_multiplayer` -- 4-player day/night cycling

**Tests to write (disturb.rs):**
- `test_disturb_cast_from_graveyard` -- CR 702.146a
- `test_disturb_enters_transformed` -- CR 702.146b
- `test_disturb_exile_on_death` -- ruling: exile instead of graveyard
- `test_disturb_exile_on_counter` -- ruling: countered spell exiled
- `test_disturb_mana_value_uses_front` -- CR 712.8c

**Tests to write (craft.rs):**
- `test_craft_basic_exile_and_transform` -- CR 702.167a
- `test_craft_materials_from_battlefield_and_graveyard` -- CR 702.167b
- `test_craft_non_dfc_stays_exiled` -- ruling
- `test_craft_tracks_exiled_materials` -- CR 702.167c
- `test_craft_sorcery_speed_only` -- CR 702.167a

**Pattern**: Follow tests for Mutate in `tests/mutate.rs` (DFC model tests), Prototype in `tests/prototype.rs` (alternate characteristics), Unearth in `tests/abilities.rs` (replacement effect)

### Step 8: Card Definitions (later phase)

**Suggested cards:**
1. **Delver of Secrets // Insectile Aberration** -- classic Transform card (front: 1/1 Human Wizard, back: 3/2 flying Insect)
2. **Brutal Cathar // Moonrage Brute** -- Daybound/Nightbound with ETB exile
3. **Beloved Beggar // Generous Soul** -- Disturb (cast transformed from graveyard, exile if would die)
4. **Braided Net // Braided Quipu** -- Craft with artifact (exile + transform)

### Step 9: Game Scripts (later phase)

**Suggested scenarios:**
1. `193_transform_delver.json` -- Delver transforms when library top is instant/sorcery
2. `194_daybound_brutal_cathar.json` -- day/night cycling with Brutal Cathar
3. `195_disturb_beloved_beggar.json` -- cast from graveyard, back face enters, exile on death
4. `196_craft_braided_net.json` -- activate craft, exile materials, return transformed

**Subsystem directory**: `test-data/generated-scripts/stack/` (transform is a stack/resolution mechanic)

## Interactions to Watch

1. **Transform + Copy (CR 707.8)**: Copy of a DFC copies the currently-up face. Clone copying transformed Delver gets Insectile Aberration characteristics, NOT Delver characteristics. Token copies of DFCs are double-faced tokens that can transform.

2. **Transform + Mutate (CR 701.27g)**: A merged permanent is NEVER a "transformed permanent" even if components have back faces up. Mutated DFCs use merged_components for characteristics, not is_transformed.

3. **Transform + Layer System**: Transform changes base characteristics (Layer 0 effectively). All subsequent layers apply on top. Humility removing abilities from a transformed DFC doesn't un-transform it -- it stays transformed but loses keyword abilities.

4. **Daybound/Nightbound + ETB**: A daybound creature cast during night enters with back face up (CR 702.145b). The spell is front-face-up on the stack but enters transformed. This is an ETB replacement.

5. **Disturb + Commander**: If a commander is cast with disturb from the graveyard, it enters transformed. If it would die, the disturb replacement (exile) applies first, then the commander zone-return SBA fires from exile.

6. **Day/Night + Multiplayer**: Each player's turn is independent. The untap step checks the PREVIOUS turn's active player's spell count. In a 4-player game, if P1 casts 0 spells, it becomes night at P2's untap step.

7. **Transform + Tokens**: Tokens created as copies of DFCs can transform (CR 707.8a). Non-copy tokens cannot transform (CR 701.27c: not represented by a double-faced card/token).

8. **Craft + Zone Change**: Craft exiles the permanent as cost. The card goes to exile. Then it returns to the battlefield transformed. If the card isn't a DFC, it stays in exile. The materials are also exiled as cost and tracked via `craft_exiled_cards`.

9. **DFC + Hidden Information**: In hand/library, DFCs show only the front face (CR 712.8a). The engine must not leak back-face information for cards in hidden zones.

10. **Mana Value**: Back face of nonmodal DFC uses front face's mana cost for mana value (CR 712.8e). Copy of back face has mana value 0 (CR 202.3b). This affects cascade, collected company, and other CMC-based effects.

## Session Breakdown (Suggested for Runner)

**Session 1 — Data Model + Layer Integration (Steps 1-2)**
- CardFace struct, CardDefinition back_face, DayNight enum
- KeywordAbility variants, AltCostKind::Disturb, AbilityDefinition variants
- GameObject fields (is_transformed, last_transform_timestamp, was_cast_disturbed)
- GameState fields (day_night, previous_turn_spells_cast)
- StackObject field (is_cast_transformed)
- StackObjectKind variants
- Hash updates
- Exhaustive match updates (view_model.rs, stack_view.rs)
- Layer system DFC face resolution
- Builder init, object init sites
- Compile check: `cargo build --workspace`

**Session 2 — Transform + Daybound/Nightbound (Steps 3-4)**
- Command::Transform handler
- Effect::Transform execution
- GameEvent::PermanentTransformed
- Day/night untap step check
- previous_turn_spells_cast tracking
- enforce_daybound_nightbound helper
- Daybound/nightbound transform lock
- Tests: transform.rs + daybound.rs

**Session 3 — Disturb + Craft + Remaining Tests (Steps 5-6-7)**
- Disturb casting from graveyard (alt cost)
- Disturb resolution (enters transformed)
- Disturb exile replacement effect
- Craft command handler
- Craft materials tracking
- Tests: disturb.rs + craft.rs
- Full test run: `cargo test --all`
