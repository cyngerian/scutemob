# Ability Plan: Morph Mini-Milestone (Morph + Megamorph + Disguise + Manifest + Cloak)

**Generated**: 2026-03-08
**CR**: 702.37 (Morph), 702.37b (Megamorph), 702.168 (Disguise), 701.40 (Manifest), 701.58 (Cloak), 708 (Face-Down Spells and Permanents)
**Priority**: P3 (Morph) + P4 (Megamorph, Disguise, Manifest, Cloak)
**Similar abilities studied**: Transform/DFC (`layers.rs:75-128`, `engine.rs:handle_transform`), Foretell (`foretell.rs` -- face-down exile), AltCostKind pattern (`command.rs:99-132`, `casting.rs`)

## CR Rule Text

### 702.37 Morph

702.37a Morph is a static ability that functions in any zone from which you could play the card it's on, and the morph effect works any time the card is face down. "Morph [cost]" means "You may cast this card as a 2/2 face-down creature with no text, no name, no subtypes, and no mana cost by paying {3} rather than paying its mana cost." (See rule 708, "Face-Down Spells and Permanents.")

702.37b Megamorph is a variant of the morph ability. "Megamorph [cost]" means "You may cast this card as a 2/2 face-down creature with no text, no name, no subtypes, and no mana cost by paying {3} rather than paying its mana cost" and "As this permanent is turned face up, put a +1/+1 counter on it if its megamorph cost was paid to turn it face up." A megamorph cost is a morph cost.

702.37c To cast a card using its morph ability, turn it face down and announce that you're using a morph ability. It becomes a 2/2 face-down creature card with no text, no name, no subtypes, and no mana cost. Any effects or prohibitions that would apply to casting a card with these characteristics (and not the face-up card's characteristics) are applied to casting this card. These values are the copiable values of that object's characteristics. Put it onto the stack (as a face-down spell with the same characteristics), and pay {3} rather than pay its mana cost. This follows the rules for paying alternative costs. You can use a morph ability to cast a card from any zone from which you could normally cast it. When the spell resolves, it enters the battlefield with the same characteristics the spell had. The morph effect applies to the face-down object wherever it is, and it ends when the permanent is turned face up.

702.37d You can't normally cast a card face down. A morph ability allows you to do so.

702.37e Any time you have priority, you may turn a face-down permanent you control with a morph ability face up. This is a special action; it doesn't use the stack (see rule 116). To do this, show all players what the permanent's morph cost would be if it were face up, pay that cost, then turn the permanent face up. (If the permanent wouldn't have a morph cost if it were face up, it can't be turned face up this way.) The morph effect on it ends, and it regains its normal characteristics. Any abilities relating to the permanent entering the battlefield don't trigger when it's turned face up and don't have any effect, because the permanent has already entered the battlefield.

702.37f If a permanent's morph cost includes X, other abilities of that permanent may also refer to X. The value of X in those abilities is equal to the value of X chosen as the morph special action was taken.

### 702.168 Disguise

702.168a Disguise is a static ability that functions in any zone from which you could play the card it's on, and the disguise effect works any time the card is face down. "Disguise [cost]" means "You may cast this card as a 2/2 face-down creature with ward {2}, no name, no subtypes, and no mana cost by paying {3} rather than paying its mana cost."

702.168b Same casting procedure as morph (see 702.37c), but the face-down spell/permanent has ward {2}.

702.168d Any time you have priority, you may turn a face-down permanent you control with a disguise ability face up. Same special action as morph, but uses the disguise cost instead.

### 701.40 Manifest

701.40a To manifest a card, turn it face down. It becomes a 2/2 face-down creature card with no text, no name, no subtypes, and no mana cost. Put that card onto the battlefield face down. That permanent is a manifested permanent for as long as it remains face down.

701.40b Any time you have priority, you may turn a manifested permanent you control face up. This is a special action that doesn't use the stack. To do this, show all players that the card representing that permanent is a creature card and what that card's mana cost is, pay that cost, then turn the permanent face up. (If the card isn't a creature card or doesn't have a mana cost, it can't be turned face up this way.)

701.40c If a card with morph is manifested, its controller may turn that card face up using either the morph procedure or the manifest procedure.

701.40d If a card with disguise is manifested, its controller may turn that card face up using either the disguise procedure or the manifest procedure.

701.40g If a manifested permanent that's represented by an instant or sorcery card would turn face up, its controller reveals it and leaves it face down. Abilities that trigger whenever a permanent is turned face up won't trigger.

### 701.58 Cloak

701.58a To cloak a card, turn it face down. It becomes a 2/2 face-down creature card with ward {2}, no name, no subtypes, and no mana cost. Put that card onto the battlefield face down. That permanent is a cloaked permanent for as long as it remains face down.

701.58b Same turn-face-up procedure as manifest (creature card + mana cost). Ward {2} while face down.

701.58c/d Morph/disguise cards that are cloaked can use either their morph/disguise cost or the cloak procedure.

701.58g Same as manifest 701.40g: instant/sorcery card stays face down.

### 708 Face-Down Spells and Permanents

708.2 Face-down spells and permanents have no characteristics other than those listed by the ability that allowed them to be face down.

708.2a Default face-down characteristics: 2/2 creature, no text, no name, no subtypes, no mana cost. These are the copiable values.

708.3 Objects put onto the battlefield face down are turned face down BEFORE they enter, so ETB abilities won't trigger.

708.4 Objects cast face down are turned face down BEFORE put on the stack. Effects that care about spell characteristics see only face-down characteristics.

708.5 You may look at your own face-down spells/permanents. You can't look at opponents'.

708.8 When turned face up, copiable values revert to normal. Effects applied while face down still apply. ETB abilities don't trigger (already entered).

708.9 If a face-down permanent leaves the battlefield for any zone, its owner must REVEAL it. If a face-down spell leaves the stack for any non-battlefield zone, reveal it.

708.10 If a face-down permanent becomes a copy, its copiable values are modified by face-down status (stays 2/2). If turned face up, uses copied values.

708.11 "As [this permanent] is turned face up..." abilities apply DURING the turn-face-up action, not afterward.

## Key Edge Cases

1. **Face-down on stack is a creature spell** with no name, types, subtypes, mana cost, abilities. It IS a creature for purposes of "counter target creature spell."
2. **Casting a morph is NOT an alternative cost in the normal AltCostKind sense** -- you pay {3} as the cost, but the card's characteristics change. However, CR 702.37c says "This follows the rules for paying alternative costs." So it IS an AltCostKind, but with special face-down handling.
3. **Turn face up is a special action** (CR 116.2b) -- does NOT use the stack. This means it cannot be responded to. The creature's morph cost is paid and it flips immediately.
4. **ETB abilities do NOT fire on turn-face-up** (CR 708.8). The permanent already entered the battlefield.
5. **"When this creature is turned face up" IS a triggered ability** -- it goes on the stack and can be responded to. This is NOT the same as the special action itself.
6. **Face-down permanent leaving battlefield must be revealed** (CR 708.9). This is critical for hidden information integrity.
7. **Manifest can put non-creature cards face-down** as 2/2 creatures. They can only be turned face up if they ARE creature cards with a mana cost (701.40b). Non-creature manifested cards are stuck face-down forever (unless they have morph/disguise).
8. **Instant/sorcery manifested cards stay face down** even if the turn-face-up procedure is attempted (701.40g).
9. **Face-down copies are also 2/2 with no characteristics** (708.10). If a face-down permanent is copied, the copy is also face-down.
10. **Disguise/Cloak add ward {2} to the face-down creature** -- this is a real ward ability that functions while face down.
11. **Megamorph adds a +1/+1 counter** when turned face up via its megamorph cost (not via manifest/cloak turn-face-up).
12. **Multiple turn-face-up methods**: A manifested card with morph can use EITHER morph cost OR its mana cost (creature cards only). Similarly for disguise+manifest, morph+cloak, disguise+cloak.
13. **Hidden information**: The engine knows the real card identity. The network layer must NOT reveal face-down card identities to opponents. Only the controller can see their own face-down cards (CR 708.5).
14. **Multiplayer**: Face-down permanents are revealed to ALL players when they leave the battlefield (CR 708.9), not just opponents. End-of-game reveal is also required.

## Current State (from ability-wip.md)

- [ ] Step 1: Plan (this document)
- [ ] Step 2: Implement
- [ ] Step 3: Review
- [ ] Step 4: Fix
- [ ] Step 5: Cards
- [ ] Step 6: Scripts
- [ ] Step 7: Close

### Pre-existing infrastructure

- `PermanentStatus.face_down: bool` exists on `GameObject` (used by Foretell, Hideaway, Plot)
- `PermanentStatus` is already hashed in `hash.rs:792-795`
- Face-down hash section exists in `hash.rs:4531-4541` (future-proofing for morphs)
- No `FaceDownKind` enum exists yet
- No morph/disguise/manifest/cloak keywords exist
- No `TurnFaceUp` command exists
- No face-down characteristic override in layers.rs

## Design Decisions

### D1: Face-Down Object Model

Add two new fields to `GameObject`:

```rust
/// What kind of face-down this is (None = not face-down, or face-down for other reasons like Foretell).
/// Determines: (a) ward {2} while face-down (Disguise, Cloak), (b) valid turn-face-up methods.
pub face_down_as: Option<FaceDownKind>,
```

Where `FaceDownKind` is a new enum in `state/types.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaceDownKind {
    /// CR 702.37: Cast face-down via Morph. Turn face up by paying morph cost.
    Morph,
    /// CR 702.37b: Cast face-down via Megamorph. Turn face up by paying megamorph cost + gets +1/+1 counter.
    Megamorph,
    /// CR 702.168: Cast face-down via Disguise. Has ward {2} while face-down. Turn face up by paying disguise cost.
    Disguise,
    /// CR 701.40: Put onto battlefield face-down via Manifest. Turn face up by paying mana cost (creature cards only).
    Manifest,
    /// CR 701.58: Put onto battlefield face-down via Cloak. Has ward {2} while face-down. Turn face up by paying mana cost (creature cards only).
    Cloak,
}
```

**Rationale**: `face_down_as` captures WHY something is face-down, which determines:
1. Whether it has ward {2} (Disguise/Cloak = yes, Morph/Megamorph/Manifest = no)
2. How it can be turned face up (morph cost, disguise cost, mana cost, or combination)
3. Whether Megamorph's +1/+1 counter applies

The existing `status.face_down: bool` remains the truth for "is this currently face-down" -- `face_down_as` is supplementary metadata.

### D2: Layer System Face-Down Override

In `layers.rs:calculate_characteristics()`, **before the layer loop** (similar to how Transform works at line 87), add a face-down characteristic override block:

```
// CR 708.2a: Face-down permanents have characteristics: 2/2 creature,
// no text, no name, no subtypes, no mana cost, colorless, no abilities.
// These ARE the copiable values (override the base characteristics).
if obj.status.face_down && obj.face_down_as.is_some() {
    chars.name = String::new();
    chars.mana_cost = None;
    chars.card_types = OrdSet::unit(CardType::Creature);
    chars.subtypes = OrdSet::new();
    chars.supertypes = OrdSet::new();
    chars.colors = OrdSet::new();
    chars.keywords = OrdSet::new();
    chars.power = Some(2);
    chars.toughness = Some(2);
    chars.triggered_abilities = vec![];
    chars.activated_abilities = vec![];
    chars.mana_abilities = vec![];

    // CR 702.168a / 701.58a: Disguise and Cloak grant ward {2} while face-down.
    if matches!(obj.face_down_as, Some(FaceDownKind::Disguise) | Some(FaceDownKind::Cloak)) {
        chars.keywords.insert(KeywordAbility::Ward(2));
    }
}
```

**Placement**: After the Transform/DFC block (line ~128) and BEFORE the merged_components block (line ~136). Face-down overrides the printed characteristics entirely -- continuous effects from the layer loop then apply on top (e.g., an Aura granting +1/+1 still works on a face-down creature).

**Critical**: This must come BEFORE the layer loop, not inside it. Face-down characteristics are the base "copiable values" per CR 708.2, not a layer effect.

### D3: Casting Face-Down (Morph/Megamorph/Disguise)

Add `Morph` to `AltCostKind`:

```rust
/// CR 702.37a: Morph -- cast face-down as 2/2 creature for {3}.
/// Also used for Megamorph (702.37b) and Disguise (702.168a).
/// The face_down_as field on the resulting GameObject records which variant was used.
Morph,
```

In `casting.rs:handle_cast_spell()`:
- When `alt_cost == Some(AltCostKind::Morph)`:
  - Override the mana cost to `{3}` (generic 3)
  - Set `status.face_down = true` on the stack object
  - Set `face_down_as` based on which keyword the card has (Morph, Megamorph, or Disguise) -- determined from the card's AbilityDefinition
  - The spell on the stack is a 2/2 creature spell with no name/abilities (layer system handles this via `face_down_as`)
  - Card is still a creature spell (can be countered by "counter target creature spell")

**New field on CastSpell command**:

```rust
/// CR 702.37c: Which face-down variant is being used (morph, megamorph, disguise).
/// Determines face_down_as on the resulting object.
#[serde(default)]
face_down_kind: Option<FaceDownKind>,
```

This is needed because a card could theoretically have both morph and disguise (unlikely but rules-legal). The player chooses which ability to use.

### D4: Turn Face Up (Special Action Command)

New `Command` variant:

```rust
/// CR 702.37e / 702.168d / 701.40b / 701.58b: Turn a face-down permanent face up.
/// Special action -- does NOT use the stack (CR 116.2b).
///
/// The engine validates:
/// 1. The permanent is face-down and controlled by the player
/// 2. The permanent can legally be turned face up (has morph/disguise/megamorph cost,
///    or is manifested/cloaked and is a creature card with a mana cost)
/// 3. The player can pay the appropriate cost
///
/// On success: pay cost, set face_down=false, clear face_down_as, regain characteristics.
/// ETB abilities do NOT fire (CR 708.8).
/// "When turned face up" triggered abilities DO fire (go on stack).
TurnFaceUp {
    player: PlayerId,
    permanent: ObjectId,
    /// Which turn-face-up method to use. Required because a manifested card
    /// with morph has two valid methods (pay morph cost vs pay mana cost).
    method: TurnFaceUpMethod,
},
```

New enum:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnFaceUpMethod {
    /// Pay the morph/megamorph cost (from CardDefinition).
    MorphCost,
    /// Pay the disguise cost (from CardDefinition).
    DisguiseCost,
    /// Pay the card's mana cost (for manifested/cloaked creatures).
    ManaCost,
}
```

Handler in `engine.rs:handle_turn_face_up()`:
1. Validate permanent exists, is face-down, controlled by player
2. Based on `method`:
   - `MorphCost`: Look up card's morph cost from CardDefinition. Validate card has Morph/Megamorph.
   - `DisguiseCost`: Look up card's disguise cost from CardDefinition. Validate card has Disguise.
   - `ManaCost`: Validate face_down_as is Manifest or Cloak. Look up card's mana cost. Validate the card is a creature card (CR 701.40b: "if the card isn't a creature card or doesn't have a mana cost, it can't be turned face up this way"). Check CR 701.40g: instant/sorcery manifested cards stay face down.
3. Pay the cost
4. Set `status.face_down = false`, `face_down_as = None`
5. If Megamorph and method is MorphCost: add +1/+1 counter (CR 702.37b)
6. Emit `PermanentTurnedFaceUp` event
7. Check triggers (for "when turned face up" abilities)
8. Do NOT fire ETB triggers (CR 708.8)
9. Reset `players_passed` (special action resets priority like any meaningful game action)

### D5: "When Turned Face Up" Triggers

New `TriggerCondition` variant:

```rust
/// CR 708.8: "When this permanent is turned face up" -- triggered ability
/// that fires when a face-down permanent is turned face up via any method.
WhenTurnedFaceUp,
```

New `TriggerEvent` variant:

```rust
/// CR 708.8: A face-down permanent was turned face up.
PermanentTurnedFaceUp { permanent: ObjectId },
```

Wire in `abilities.rs:check_triggers()`: when `PermanentTurnedFaceUp` event fires, check if the permanent has any `WhenTurnedFaceUp` triggered abilities in its CardDefinition (now face-up, so the abilities are visible). Fire them as normal triggered abilities on the stack.

### D6: Manifest and Cloak as Effects

New `Effect` variants:

```rust
/// CR 701.40a: Manifest the top card of a player's library.
Manifest { player: EffectTarget },
/// CR 701.58a: Cloak the top card of a player's library.
Cloak { player: EffectTarget },
```

Implementation in `effects/mod.rs`:
1. Get top card of the player's library
2. Turn it face down
3. Move to battlefield as a 2/2 creature
4. Set `status.face_down = true`, `face_down_as = Some(Manifest)` or `Some(Cloak)`
5. CR 708.3: ETB abilities do NOT trigger (the permanent enters face-down)

### D7: Zone-Change Reveal (CR 708.9)

In `state/mod.rs:move_object_to_zone()`, when moving a face-down permanent from battlefield to any other zone:
- Emit a `FaceDownRevealed` event containing the card identity
- Set `face_down = false` and `face_down_as = None` on the new object in the destination zone

This is primarily for network/UI layer consumption. The engine already knows the card identity.

### D8: Hidden Information

The engine stores the full card identity on face-down objects (via `card_id`). The network layer (M10+) must:
- NOT include `card_id`, `card_types`, `oracle_text`, etc. in state broadcasts for face-down permanents controlled by OTHER players
- Only send face-down characteristic data (2/2, no name, creature)
- Controller sees their own face-down cards normally (CR 708.5)

For now (pre-M10), the engine itself is correct -- hidden information enforcement is a network-layer concern. The `FaceDownRevealed` event serves as the reveal notification.

## Modification Surface

Files and functions that need changes:

| File | Function/Location | What to add |
|------|-------------------|-------------|
| `state/types.rs` | `KeywordAbility` enum | `Morph`, `Megamorph`, `Disguise`, `Manifest`, `Cloak` (KW 153-157) |
| `state/types.rs` | new enum | `FaceDownKind` enum (5 variants) |
| `state/types.rs` | new enum | `TurnFaceUpMethod` enum (3 variants) |
| `state/types.rs` | `AltCostKind` enum | `Morph` variant |
| `state/game_object.rs` | `GameObject` struct | `face_down_as: Option<FaceDownKind>` field |
| `state/hash.rs` | `GameObject` HashInto | Hash `face_down_as` |
| `state/hash.rs` | face-down section (~L4531) | Already exists, may need updates for FaceDownKind |
| `rules/command.rs` | `Command` enum | `TurnFaceUp { player, permanent, method }` variant |
| `rules/engine.rs` | `process_command` match | `Command::TurnFaceUp` arm + `handle_turn_face_up()` |
| `rules/casting.rs` | `handle_cast_spell` | `AltCostKind::Morph` handling: override cost to {3}, set face_down |
| `rules/layers.rs` | `calculate_characteristics` | Face-down override block (before layer loop, after Transform) |
| `rules/resolution.rs` | spell resolution | Preserve `face_down`/`face_down_as` when face-down spell becomes permanent |
| `rules/events.rs` | `GameEvent` enum | `PermanentTurnedFaceUp`, `FaceDownRevealed` events |
| `rules/abilities.rs` | `check_triggers` | `PermanentTurnedFaceUp` trigger dispatch |
| `effects/mod.rs` | `apply_effect` match | `Effect::Manifest`, `Effect::Cloak` |
| `state/mod.rs` | `move_object_to_zone` | Face-down reveal on zone change |
| `state/builder.rs` | object construction | Initialize `face_down_as: None` |
| `cards/card_definition.rs` | `AbilityDefinition` enum | `Morph { cost }`, `Megamorph { cost }`, `Disguise { cost }` (AbilDef 62-64) |
| `cards/helpers.rs` | exports | `FaceDownKind`, `TurnFaceUpMethod` |
| `state/stack.rs` | `StackObjectKind` | `TurnFaceUpTrigger` (SOK 63) -- for "when turned face up" |
| `tools/replay-viewer/src/view_model.rs` | KW match | Arms for Morph, Megamorph, Disguise, Manifest, Cloak |
| `tools/replay-viewer/src/view_model.rs` | SOK match | Arm for TurnFaceUpTrigger |
| `tools/tui/src/play/panels/stack_view.rs` | SOK match | Arm for TurnFaceUpTrigger |
| `crates/simulator/src/legal_actions.rs` | StubProvider | `TurnFaceUp` legal action generation |

## Implementation Steps

### Step 1: Type Definitions and Enums

**File**: `crates/engine/src/state/types.rs`

1. Add `FaceDownKind` enum (5 variants: Morph, Megamorph, Disguise, Manifest, Cloak)
2. Add `TurnFaceUpMethod` enum (3 variants: MorphCost, DisguiseCost, ManaCost)
3. Add to `AltCostKind`: `Morph` variant
4. Add to `KeywordAbility`:
   - `Morph` (discriminant 153) -- CR 702.37
   - `Megamorph` (discriminant 154) -- CR 702.37b
   - `Disguise` (discriminant 155) -- CR 702.168
   - `Manifest` (discriminant 156) -- CR 701.40 (marker: "this card's abilities include manifest-related effects")
   - `Cloak` (discriminant 157) -- CR 701.58 (marker)

**File**: `crates/engine/src/cards/card_definition.rs`

5. Add to `AbilityDefinition`:
   - `Morph { cost: ManaCost }` (discriminant 62) -- the morph turn-face-up cost
   - `Megamorph { cost: ManaCost }` (discriminant 63) -- the megamorph turn-face-up cost
   - `Disguise { cost: ManaCost }` (discriminant 64) -- the disguise turn-face-up cost

**File**: `crates/engine/src/state/game_object.rs`

6. Add `face_down_as: Option<FaceDownKind>` to `GameObject`

**File**: `crates/engine/src/state/hash.rs`

7. Hash `face_down_as` in `GameObject`'s `HashInto` impl

**File**: `crates/engine/src/state/builder.rs`

8. Initialize `face_down_as: None` in all object construction sites

**File**: `crates/engine/src/effects/mod.rs`

9. Initialize `face_down_as: None` in token creation

**File**: `crates/engine/src/rules/resolution.rs`

10. Initialize `face_down_as: None` in any object construction (unless face-down spell resolving)

**File**: `crates/engine/src/rules/command.rs`

11. Add `Command::TurnFaceUp { player, permanent, method }` variant
12. Add `face_down_kind: Option<FaceDownKind>` field to `CastSpell` (with `#[serde(default)]`)

**File**: `crates/engine/src/state/stack.rs`

13. Add `StackObjectKind::TurnFaceUpTrigger { permanent: ObjectId, source_card_id: Option<CardId> }` (discriminant 63)

**File**: `crates/engine/src/rules/events.rs`

14. Add `GameEvent::PermanentTurnedFaceUp { player: PlayerId, permanent: ObjectId }`
15. Add `GameEvent::FaceDownRevealed { player: PlayerId, permanent: ObjectId, card_name: String }`

**File**: `crates/engine/src/effects/mod.rs`

16. Add `Effect::Manifest { player: EffectTarget }` and `Effect::Cloak { player: EffectTarget }`

**File**: `crates/engine/src/cards/helpers.rs`

17. Export `FaceDownKind`, `TurnFaceUpMethod`

**File**: `tools/replay-viewer/src/view_model.rs`

18. Add match arms for all 5 new `KeywordAbility` variants
19. Add match arm for `TurnFaceUpTrigger` SOK variant

**File**: `tools/tui/src/play/panels/stack_view.rs`

20. Add match arm for `TurnFaceUpTrigger` SOK variant

### Step 2: Layer System Face-Down Override

**File**: `crates/engine/src/rules/layers.rs`
**Action**: Add face-down characteristic override block
**Location**: After the Transform/DFC block (line ~128), before the merged_components block (line ~136)
**CR**: 708.2, 708.2a

Implementation:
```
if obj.status.face_down && obj.face_down_as.is_some() {
    // CR 708.2a: Reset all characteristics to face-down defaults
    chars.name = String::new();
    chars.mana_cost = None;
    chars.card_types = OrdSet::unit(CardType::Creature);
    chars.subtypes = OrdSet::new();
    chars.supertypes = OrdSet::new();
    chars.colors = OrdSet::new();
    chars.keywords = OrdSet::new();
    chars.power = Some(2);
    chars.toughness = Some(2);
    chars.triggered_abilities = vec![];
    chars.activated_abilities = vec![];
    chars.mana_abilities = vec![];

    // CR 702.168a / 701.58a: Disguise and Cloak grant ward {2} while face-down
    if matches!(obj.face_down_as, Some(FaceDownKind::Disguise) | Some(FaceDownKind::Cloak)) {
        chars.keywords.insert(KeywordAbility::Ward(2));
    }
}
```

**Important**: This check uses `face_down_as.is_some()` to distinguish morph/manifest face-down from other face-down uses (Foretell exiles are `face_down = true` but `face_down_as = None`).

### Step 3: Casting Face-Down (Morph/Megamorph/Disguise)

**File**: `crates/engine/src/rules/casting.rs`
**Action**: Handle `AltCostKind::Morph` in `handle_cast_spell`
**CR**: 702.37c, 702.168b

When `alt_cost == Some(AltCostKind::Morph)`:
1. Override mana cost to `ManaCost { generic: 3, ..Default::default() }` (pay {3})
2. On the stack object: set `status.face_down = true`
3. Set `face_down_as = command.face_down_kind` (Morph, Megamorph, or Disguise -- from the Command)
4. The layer system will handle making it a 2/2 with no characteristics on the stack
5. When the spell resolves (resolution.rs), preserve `face_down` and `face_down_as` on the permanent

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: When resolving a face-down creature spell, preserve face-down status
**CR**: 702.37c ("enters the battlefield with the same characteristics the spell had")

In the permanent-enters-battlefield path:
- If the stack object has `status.face_down == true` and `face_down_as.is_some()`:
  - Set `face_down = true` and `face_down_as` on the new battlefield object
  - CR 708.3: Do NOT fire ETB abilities (they were suppressed because the permanent enters face-down)

### Step 4: Turn Face Up (Special Action)

**File**: `crates/engine/src/rules/engine.rs`
**Action**: Add `Command::TurnFaceUp` handler and `handle_turn_face_up()` function
**CR**: 702.37e, 702.168d, 701.40b, 701.58b, 116.2b

```
fn handle_turn_face_up(
    state: &mut GameState,
    player: PlayerId,
    permanent: ObjectId,
    method: TurnFaceUpMethod,
) -> Result<Vec<GameEvent>, GameStateError>
```

Implementation:
1. Validate permanent exists, is on battlefield, face_down == true, face_down_as.is_some(), controlled by player
2. Determine cost based on method:
   - `MorphCost`: Look up card via card_id -> card_registry. Find `AbilityDefinition::Morph { cost }` or `AbilityDefinition::Megamorph { cost }`. Error if neither exists.
   - `DisguiseCost`: Look up `AbilityDefinition::Disguise { cost }`. Error if not found.
   - `ManaCost`: Validate `face_down_as` is `Manifest` or `Cloak` (or the card has morph/disguise -- see 701.40c/d). Check that the card IS a creature card (has `CardType::Creature` in its CardDefinition, NOT its current face-down chars). Check that the card HAS a mana cost. Check CR 701.40g: if the card is an instant or sorcery, reject (stays face-down).
3. Pay the cost from mana pool
4. Set `status.face_down = false`, `face_down_as = None`
5. If `face_down_as` was `Megamorph` and method is `MorphCost`: add +1/+1 counter (CR 702.37b)
6. Emit `PermanentTurnedFaceUp { player, permanent }`
7. Check triggers for "when turned face up" abilities (now visible since card is face-up)
8. Flush pending triggers
9. Reset `players_passed` (special action)
10. Do NOT fire ETB abilities (CR 708.8)

### Step 5: Manifest and Cloak Effects

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Implement `Effect::Manifest` and `Effect::Cloak`
**CR**: 701.40a, 701.58a

Implementation:
1. Get top card of the target player's library
2. If library is empty, do nothing (CR 701.40f: "if a rule or effect prohibits the face-down object from entering the battlefield, that card isn't manifested")
3. Move the card from library to battlefield
4. Set `status.face_down = true`
5. Set `face_down_as = Some(Manifest)` or `Some(Cloak)`
6. CR 708.3: ETB abilities suppressed (enters face-down)
7. Emit appropriate event

### Step 6: Zone-Change Reveal

**File**: `crates/engine/src/state/mod.rs`
**Action**: In `move_object_to_zone()`, when source is face-down on battlefield, emit reveal
**CR**: 708.9

When moving an object that has `status.face_down == true` and `face_down_as.is_some()` from the battlefield to any other zone:
1. Look up the card's real name from `card_id` -> card_registry
2. Set `face_down = false` and `face_down_as = None` on the new object
3. The reveal event will be emitted by the calling code (engine.rs or sba.rs)

Note: The actual `FaceDownRevealed` event should be emitted from the caller (engine.rs `handle_turn_face_up` or SBA processing) rather than deep inside `move_object_to_zone`, to avoid side effects in a utility function.

### Step 7: Trigger Wiring

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add `WhenTurnedFaceUp` trigger dispatch
**CR**: "When this creature is turned face up" (e.g., Willbender, Den Protector)

In `check_triggers()`:
- When `PermanentTurnedFaceUp` event fires:
  - Look up the permanent's CardDefinition (now face-up)
  - Scan its abilities for `TriggerCondition::WhenTurnedFaceUp`
  - Fire matching triggers as `PendingTriggerKind::Normal`

Also add in `builder.rs`:
- If a CardDefinition has a `Triggered` ability with `TriggerCondition::WhenTurnedFaceUp`, register it as a `TriggeredAbilityDef` at construction time.

### Step 8: LegalActionProvider Update

**File**: `crates/simulator/src/legal_actions.rs`
**Action**: Generate `TurnFaceUp` legal actions for face-down permanents
**Pattern**: Similar to how `SaddleMount` or `ActivateBloodrush` are enumerated

For each face-down permanent the player controls with `face_down_as.is_some()`:
- Check which turn-face-up methods are valid (morph cost, disguise cost, mana cost)
- Check if the player has enough mana to pay
- Generate a `LegalAction` for each valid method

### Step 9: Unit Tests

**File**: `crates/engine/tests/morph.rs` (new file)
**Tests to write**:

1. `test_morph_cast_face_down_basic` -- Cast a creature with morph face-down for {3}. Verify it's a 2/2 on battlefield with no name, no abilities, no types except Creature. CR 702.37a, 708.2a.

2. `test_morph_turn_face_up` -- Cast morph face-down, then turn it face up by paying morph cost. Verify it regains its real characteristics (name, P/T, abilities). CR 702.37e.

3. `test_morph_face_down_no_etb` -- Cast morph face-down. Verify ETB abilities do NOT fire. Then turn face up. Verify ETB abilities still do NOT fire (CR 708.8).

4. `test_morph_when_turned_face_up_trigger` -- Use a card with "When this creature is turned face up" ability. Verify the trigger fires and goes on the stack. CR 708.8.

5. `test_morph_face_down_characteristics_layer` -- Verify that continuous effects (e.g., +1/+1 from an Aura) still apply to face-down creatures. Verify the base is 2/2 but modified by effects.

6. `test_megamorph_counter` -- Cast megamorph face-down, turn face up. Verify +1/+1 counter is added. CR 702.37b.

7. `test_disguise_ward` -- Cast disguise face-down. Verify the face-down creature has ward {2}. CR 702.168a.

8. `test_manifest_creature_turn_face_up` -- Manifest a creature card. Turn it face up by paying its mana cost. Verify it works. CR 701.40b.

9. `test_manifest_noncreature_stuck` -- Manifest a non-creature card (e.g., an instant). Verify it cannot be turned face up via ManaCost method. CR 701.40b.

10. `test_manifest_with_morph` -- Manifest a card that has morph. Verify it can be turned face up by EITHER its morph cost OR its mana cost. CR 701.40c.

11. `test_cloak_ward` -- Cloak a card. Verify it has ward {2}. CR 701.58a.

12. `test_face_down_dies_reveal` -- Kill a face-down creature. Verify a `FaceDownRevealed` event is emitted with the card's real identity. CR 708.9.

13. `test_face_down_copy_is_face_down` -- (Deferred/stretch) If a face-down creature is copied, the copy should also be a 2/2 with no characteristics. CR 708.10.

14. `test_morph_cast_face_down_is_creature_spell` -- Verify the face-down spell on the stack is a creature spell (can be targeted by "counter target creature spell"). CR 708.4.

**Pattern**: Follow tests in `crates/engine/tests/transform.rs` for similar face-transition tests.

### Step 10: Card Definitions

**Suggested cards**:

1. **Exalted Angel** (Morph, simple -- Flying + Lifelink + Morph {2}{W}{W})
   - Good test: morph a 4/5 flier, verify it starts as 2/2, verify it becomes 4/5 flying lifelink on turn-face-up

2. **Den Protector** (Megamorph -- tests +1/+1 counter on turn-face-up)
   - Has "When turned face up" trigger (return card from GY to hand)

3. **Kadena, Slinking Sorcerer** (Commander that cares about face-down creatures)
   - Deferred: complex card, optional for MVP

**Author**: Use `card-definition-author` agent after Step 9.

### Step 11: Game Scripts

**Suggested scripts**:

1. `197_morph_cast_and_flip.json` -- Cast Exalted Angel face-down for {3}, attack as 2/2, turn face up for {2}{W}{W}, verify 4/5 flying lifelink.

2. `198_manifest_and_flip.json` -- Manifest a creature, turn it face up by paying mana cost.

**Directory**: `test-data/generated-scripts/baseline/` or `test-data/generated-scripts/stack/`

## Interactions to Watch

1. **Morph + Humility**: Humility removes all abilities (Layer 6). A face-down creature already has no abilities, so Humility has no additional effect. If the face-down creature is turned face up while Humility is on the battlefield, it immediately loses all abilities again (Layer 6 applies). It keeps its P/T from being 2/2 (face-down base) to its printed P/T, then Humility sets it to 1/1 (Layer 7b).

2. **Morph + Blood Moon**: No interaction (morph is on creatures, Blood Moon affects lands).

3. **Face-down + Torpor Orb**: No ETB trigger fires when entering face-down (CR 708.3). No ETB trigger fires when turning face up (CR 708.8). "When turned face up" is NOT an ETB trigger, so Torpor Orb doesn't suppress it.

4. **Face-down + Panharmonicon**: "When turned face up" is NOT an ETB trigger, so Panharmonicon does NOT double it.

5. **Manifest + Wrath of God**: Face-down manifested creature dies. Revealed (CR 708.9). If it was a non-creature card, it goes to graveyard as a non-creature card (its face-down creature status was battlefield-only).

6. **Morph + Commander**: A face-down commander has no name, so "commander damage from [name]" tracking needs to use the underlying card identity, not the face-down name. The commander tax still applies (the morph cast is still a cast from the command zone). **Important**: commander damage should still track correctly because the engine uses ObjectId-based tracking, not name-based.

7. **Morph + Protection**: A face-down creature has no name, no colors, no card types (except Creature). Protection from [color] doesn't protect against it (it has no color). Protection from creatures DOES block it (it's a Creature).

8. **Multiplayer**: Face-down reveal (CR 708.9) reveals to ALL players, not just the controller. End-of-game reveal is also to all players.

## Discriminant Chain

| Type | Name | Discriminant |
|------|------|-------------|
| KeywordAbility | Morph | 153 |
| KeywordAbility | Megamorph | 154 |
| KeywordAbility | Disguise | 155 |
| KeywordAbility | Manifest | 156 |
| KeywordAbility | Cloak | 157 |
| AbilityDefinition | Morph { cost } | 62 |
| AbilityDefinition | Megamorph { cost } | 63 |
| AbilityDefinition | Disguise { cost } | 64 |
| StackObjectKind | TurnFaceUpTrigger | 63 |

## Harness Actions Needed

New script action types for `replay_harness.rs`:

1. `cast_spell_morph` -- CastSpell with `alt_cost: Some(AltCostKind::Morph)` and `face_down_kind: Some(FaceDownKind::X)`
2. `turn_face_up` -- `Command::TurnFaceUp { player, permanent, method }`

## Risk Assessment

- **Medium risk**: The layer system face-down override is the most critical piece. If placed incorrectly (inside the layer loop instead of before it), continuous effects will interact wrongly with face-down creatures.
- **Low risk**: The existing `status.face_down` field and face-down hash section provide good infrastructure.
- **Medium risk**: `CastSpell` field addition (`face_down_kind`) will require updating all CastSpell construction sites (same pattern as `mutate_on_top` from the Mutate session -- use `#[serde(default)]` and a Python script).
- **Low risk**: Hidden information is a network-layer concern, not an engine concern. The engine correctly stores card identity.
- **Deferred**: Face-down copies (CR 708.10) are a stretch goal. The base implementation covers morph/megamorph/disguise cast + turn-face-up, manifest, cloak, and zone-change reveal.
