# Primitive Batch Plan: PB-26 -- Trigger Variants (all remaining)

**Generated**: 2026-03-24
**Primitive**: 8 new/extended TriggerCondition + TriggerEvent variants, with dispatch wiring in `check_triggers`
**CR Rules**: CR 603.2 (trigger matching), CR 603.10a (LTB look-back), CR 701.9 (discard), CR 701.21 (sacrifice), CR 508.1 (declare attackers)
**Cards affected**: ~72 (all existing fixes, no new card defs)
**Dependencies**: PB-23 (controller-filtered creature triggers -- DONE), PB-24 (conditional statics -- DONE), PB-25 (continuous effect grants -- DONE)
**Deferred items from prior PBs**: None specific to PB-26

---

## Primitive Specification

This batch consolidates 8 gap types (G-4, G-9, G-10, G-11, G-12, G-13, G-14, G-15) from the DSL gap closure plan. Each gap adds or extends a TriggerCondition variant, adds corresponding TriggerEvent variants (where needed), and wires dispatch in `check_triggers()` to process the relevant GameEvent. The gaps are:

1. **G-4**: Add `spell_type_filter` to `WheneverYouCastSpell` and `WheneverOpponentCastsSpell` (~19 cards)
2. **G-9**: Add `WheneverYouDiscard` and `WheneverOpponentDiscards` TriggerConditions (~9 cards)
3. **G-10**: Add `WheneverYouSacrifice` TriggerCondition + new `GameEvent::PermanentSacrificed` (~6+ cards)
4. **G-11**: Add `WheneverYouAttack` TriggerCondition (fires once when controller declares attackers) (~8 cards)
5. **G-12**: Add `WhenLeavesBattlefield` TriggerCondition + LTB dispatch (~6 cards)
6. **G-13**: Add `player_filter` to `WheneverPlayerDrawsCard` + wire `GameEvent::CardDrawn` dispatch (~16 cards)
7. **G-14**: Wire `GameEvent::LifeGained` dispatch in check_triggers for existing `WheneverYouGainLife` (~3 cards)
8. **G-15**: Add `WhenYouCastThisSpell` TriggerCondition (fires from stack before resolution) (~5 cards)

**Critical finding**: `WheneverYouDrawACard`, `WheneverYouGainLife`, and `WheneverPlayerDrawsCard` already exist as TriggerCondition variants but are **NOT dispatched** in `check_triggers`. Cards like Niv-Mizzet, Parun and The Locust God use them but the triggers never fire. This batch wires them.

---

## CR Rule Text

### CR 603.2 -- Trigger matching
> Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers.

### CR 603.2c -- One trigger per event
> An ability triggers only once each time its trigger event occurs.

### CR 603.2g -- Prevented/replaced events don't trigger
> An ability triggers only if its trigger event actually occurs. An event that's prevented or replaced won't trigger anything.

### CR 603.10a -- LTB look-back
> Some zone-change triggers look back in time. These are leaves-the-battlefield abilities, abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object that all players can see is put into a hand or library.

### CR 701.9a -- Discard
> To discard a card, move it from its owner's hand to that player's graveyard.

### CR 701.21a -- Sacrifice
> To sacrifice a permanent, its controller moves it from the battlefield directly to its owner's graveyard. A player can't sacrifice something that isn't a permanent, or something that's a permanent they don't control. Sacrificing a permanent doesn't destroy it.

### CR 508.1 -- Declare attackers
> First, the active player declares attackers. This turn-based action doesn't use the stack.

---

## Engine Changes

### Change 1: Extend `WheneverYouCastSpell` with spell_type_filter (G-4)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add optional `spell_type_filter` field to `WheneverYouCastSpell` variant:
```rust
WheneverYouCastSpell {
    during_opponent_turn: bool,
    /// Optional card type filter (e.g., creature, instant/sorcery, noncreature).
    /// None = any spell. Some(types) = spell must have at least one of these types.
    /// For "noncreature", use `noncreature_only: bool` field instead.
    #[serde(default)]
    spell_type_filter: Option<Vec<CardType>>,
    /// If true, only fires on noncreature spells.
    #[serde(default)]
    noncreature_only: bool,
},
```
**Pattern**: Follows the `controller` field pattern on `WheneverCreatureDies` (PB-23)
**Line**: ~1803 (current `WheneverYouCastSpell` definition)

Also extend `WheneverOpponentCastsSpell` with the same fields:
```rust
WheneverOpponentCastsSpell {
    #[serde(default)]
    spell_type_filter: Option<Vec<CardType>>,
    #[serde(default)]
    noncreature_only: bool,
},
```
**Line**: ~1764 (current `WheneverOpponentCastsSpell` -- currently a unit variant)

### Change 2: Add WheneverYouDiscard / WheneverOpponentDiscards (G-9)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add two new TriggerCondition variants:
```rust
/// "Whenever you discard a card" (CR 701.9a).
WheneverYouDiscard,
/// "Whenever an opponent discards a card" (CR 701.9a).
WheneverOpponentDiscards,
```
**Hash discriminants**: 30, 31 (next available in TriggerCondition hash)

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add two new TriggerEvent variants:
```rust
/// CR 701.9a: Fires on permanents controlled by the discarding player.
ControllerDiscards,
/// CR 701.9a: Fires on permanents controlled by opponents of the discarding player.
OpponentDiscards,
```
**Hash discriminants**: 31, 32 (next available in TriggerEvent hash)

### Change 3: Add WheneverYouSacrifice (G-10)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new TriggerCondition variant:
```rust
/// "Whenever you sacrifice a permanent" (CR 701.21a).
/// Optional filter restricts to specific permanent types (e.g., creature, Food, Treasure).
WheneverYouSacrifice {
    #[serde(default)]
    filter: Option<TargetFilter>,
},
```
**Hash discriminant**: 32

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add new TriggerEvent variant:
```rust
/// CR 701.21a: Fires on permanents controlled by the sacrificing player.
ControllerSacrifices,
```
**Hash discriminant**: 33

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add new GameEvent variant:
```rust
/// A permanent was sacrificed by its controller (CR 701.21a).
/// Distinct from CreatureDied/PermanentDestroyed -- sacrifice is NOT destruction.
/// Emitted IN ADDITION TO the existing CreatureDied/PermanentDestroyed events
/// when the zone move is caused by a sacrifice action.
PermanentSacrificed {
    /// The player who sacrificed.
    player: PlayerId,
    /// ObjectId on the battlefield (now retired).
    object_id: ObjectId,
    /// New ObjectId in graveyard (or exile if replaced).
    new_id: ObjectId,
},
```

**Files emitting sacrifice events** (must add `PermanentSacrificed` event alongside existing events):
- `crates/engine/src/effects/mod.rs` -- `Effect::SacrificePermanents` handler (~L2120)
- `crates/engine/src/rules/abilities.rs` -- `sacrifice_self` cost payment (~L514)
- `crates/engine/src/rules/abilities.rs` -- `sacrifice_filter` cost payment (search for `sacrifice_filter`)
- Any other sacrifice cost paths (bargain, emerge, casualty, devour) -- grep for all `sacrifice` paths

### Change 4: Add WheneverYouAttack (G-11)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new TriggerCondition variant:
```rust
/// "Whenever you attack" -- fires once when controller declares one or more attackers.
/// CR 508.1: fires at declare-attackers step, once per combat (not per creature).
WheneverYouAttack,
```
**Hash discriminant**: 33

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add new TriggerEvent variant:
```rust
/// CR 508.1: Fires on permanents controlled by the attacking player when they
/// declare one or more attackers. Fires once per combat, not per creature.
ControllerAttacks,
```
**Hash discriminant**: 34

### Change 5: Add WhenLeavesBattlefield (G-12)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new TriggerCondition variant:
```rust
/// "When ~ leaves the battlefield" -- fires when this permanent leaves for any reason
/// (dies, exiled, bounced, sacrificed). CR 603.10a: looks back in time.
WhenLeavesBattlefield,
```
**Hash discriminant**: 34

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add new TriggerEvent variant:
```rust
/// CR 603.10a: Fires on the object that left the battlefield (look-back trigger).
/// Covers all zone transitions FROM battlefield: death, exile, bounce, sacrifice.
SelfLeavesBattlefield,
```
**Hash discriminant**: 35

### Change 6: Extend WheneverPlayerDrawsCard with player_filter (G-13) + Wire CardDrawn dispatch

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Extend `WheneverPlayerDrawsCard` from unit variant to struct:
```rust
/// "Whenever a player draws a card" / "Whenever an opponent draws a card"
WheneverPlayerDrawsCard {
    /// None = any player. Some(Opponent) = opponents only. Some(You) = you only.
    #[serde(default)]
    player_filter: Option<TargetController>,
},
```
**Line**: ~1766

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add new TriggerEvent variants:
```rust
/// CR 603.2: Fires when the controller of this permanent draws a card.
ControllerDrawsCard,
/// CR 603.2: Fires when any player draws a card (used with player_filter).
AnyPlayerDrawsCard,
/// CR 603.2: Fires when an opponent of the controller draws a card.
OpponentDrawsCard,
```
**Hash discriminants**: 36, 37, 38

### Change 7: Wire LifeGained dispatch (G-14)

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add new TriggerEvent variant:
```rust
/// CR 603.2: Fires when the controller of this permanent gains life.
ControllerGainsLife,
```
**Hash discriminant**: 39

No new TriggerCondition needed -- `WheneverYouGainLife` already exists (discriminant 15).

### Change 8: Add WhenYouCastThisSpell (G-15)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new TriggerCondition variant:
```rust
/// "When you cast this spell" -- fires when the spell is put on the stack (CR 603.2).
/// Unlike WhenEntersBattlefield, this fires from the stack BEFORE resolution.
/// The triggered ability goes above the spell on the stack and resolves first.
WhenYouCastThisSpell,
```
**Hash discriminant**: 35

### Change 9: Dispatch wiring in check_triggers

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add match arms in `check_triggers()` (before the `_ => {}` catch-all at ~L4592) for:

#### 9a: GameEvent::CardDrawn dispatch (G-13 + G-14 existing WheneverYouDrawACard)
```rust
GameEvent::CardDrawn { player, .. } => {
    // ControllerDrawsCard: fire on permanents controlled by the drawing player.
    let controller_sources: Vec<ObjectId> = state.objects.values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.controller == *player)
        .map(|obj| obj.id)
        .collect();
    for obj_id in controller_sources {
        collect_triggers_for_event(state, &mut triggers, TriggerEvent::ControllerDrawsCard, Some(obj_id), None);
    }
    // OpponentDrawsCard: fire on permanents controlled by opponents of drawing player.
    let opponent_sources: Vec<ObjectId> = state.objects.values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.controller != *player)
        .map(|obj| obj.id)
        .collect();
    for obj_id in opponent_sources {
        collect_triggers_for_event(state, &mut triggers, TriggerEvent::OpponentDrawsCard, Some(obj_id), None);
    }
    // AnyPlayerDrawsCard: fire on all permanents.
    collect_triggers_for_event(state, &mut triggers, TriggerEvent::AnyPlayerDrawsCard, None, None);
}
```

#### 9b: GameEvent::LifeGained dispatch (G-14)
```rust
GameEvent::LifeGained { player, .. } => {
    // ControllerGainsLife: fire on permanents controlled by the gaining player.
    let controller_sources: Vec<ObjectId> = state.objects.values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.controller == *player)
        .map(|obj| obj.id)
        .collect();
    for obj_id in controller_sources {
        collect_triggers_for_event(state, &mut triggers, TriggerEvent::ControllerGainsLife, Some(obj_id), None);
    }
}
```

#### 9c: GameEvent::CardDiscarded dispatch (G-9)
```rust
GameEvent::CardDiscarded { player, .. } => {
    // ControllerDiscards: fire on permanents controlled by the discarding player.
    let controller_sources: Vec<ObjectId> = state.objects.values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.controller == *player)
        .map(|obj| obj.id)
        .collect();
    for obj_id in controller_sources {
        collect_triggers_for_event(state, &mut triggers, TriggerEvent::ControllerDiscards, Some(obj_id), None);
    }
    // OpponentDiscards: fire on permanents controlled by opponents.
    let opponent_sources: Vec<ObjectId> = state.objects.values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.controller != *player)
        .map(|obj| obj.id)
        .collect();
    let pre_len = triggers.len();
    for obj_id in opponent_sources {
        collect_triggers_for_event(state, &mut triggers, TriggerEvent::OpponentDiscards, Some(obj_id), None);
    }
    // Tag with triggering player so effect can reference "that player".
    for t in &mut triggers[pre_len..] {
        t.triggering_player = Some(*player);
    }
}
```

#### 9d: GameEvent::PermanentSacrificed dispatch (G-10)
```rust
GameEvent::PermanentSacrificed { player, object_id, new_id } => {
    // ControllerSacrifices: fire on permanents controlled by the sacrificing player.
    let controller_sources: Vec<ObjectId> = state.objects.values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.controller == *player)
        .map(|obj| obj.id)
        .collect();
    for obj_id in controller_sources {
        collect_triggers_for_event(state, &mut triggers, TriggerEvent::ControllerSacrifices, Some(obj_id), None);
    }
}
```
**Note**: The sacrifice filter (creature, Food, Treasure, etc.) must be checked when matching -- see Change 11 below.

#### 9e: GameEvent::AttackersDeclared -- add WheneverYouAttack dispatch (G-11)
**Action**: In the existing `GameEvent::AttackersDeclared` arm (~L3041), add AFTER the existing creature-attack trigger collection:
```rust
// WheneverYouAttack: fires once on each permanent controlled by the attacking player
// when that player declares one or more attackers.
if !attackers.is_empty() {
    let controller_sources: Vec<ObjectId> = state.objects.values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in() && obj.controller == *attacking_player)
        .map(|obj| obj.id)
        .collect();
    for obj_id in controller_sources {
        collect_triggers_for_event(state, &mut triggers, TriggerEvent::ControllerAttacks, Some(obj_id), None);
    }
}
```

#### 9f: LTB triggers (G-12)
**Action**: Wire `SelfLeavesBattlefield` dispatch on the following existing events that represent leaving the battlefield:
- `GameEvent::CreatureDied` -- already handled for SelfDies; ADD SelfLeavesBattlefield dispatch
- `GameEvent::PermanentDestroyed` -- ADD SelfLeavesBattlefield dispatch
- `GameEvent::ObjectExiled` -- ADD SelfLeavesBattlefield dispatch (check that it came from battlefield)
- `GameEvent::ObjectReturnedToHand` -- ADD SelfLeavesBattlefield dispatch (check it came from battlefield)
- `GameEvent::AuraFellOff` -- ADD SelfLeavesBattlefield dispatch

**CR 603.10a**: LTB triggers look back in time. The trigger must check the object's abilities BEFORE the zone change. Since `check_triggers` runs on events emitted by zone moves, and the object has already moved, we need LKI (last known information). The existing `CreatureDied` handler already uses the object_id (pre-death) for SelfDies -- follow the same pattern. The `collect_triggers_for_event` with `only_object: Some(object_id)` will fail because the object is no longer on the battlefield. Instead, for LTB triggers, we must scan the card registry for the source card's triggered abilities and check if any have `WhenLeavesBattlefield`, then push a PendingTrigger manually (same pattern as the existing `SelfDies` handler in the CreatureDied arm).

#### 9g: SpellCast -- extend for spell_type_filter (G-4)
**Action**: In the existing `GameEvent::SpellCast` arm (~L2912), the `ControllerCastsSpell` and `OpponentCastsSpell` triggers already fire. Now we need to ALSO pass the spell's card types through to `collect_triggers_for_event` so that the filter can be checked. However, `collect_triggers_for_event` currently has no parameter for spell types.

**Approach**: Instead of modifying `collect_triggers_for_event`, handle filtering at CardDef trigger lookup time. When the SpellCast event fires, capture the spell's card types from the `source_object_id`. Then, when checking CardDef triggered abilities with `WheneverYouCastSpell { spell_type_filter, noncreature_only, .. }`, verify the spell matches the filter. This happens in the `enrich_spec_from_def` → `collect_triggers_for_event` path -- we need to add the spell type info to the trigger-checking logic.

**Implementation**: After existing ControllerCastsSpell collection, iterate collected triggers and remove those whose CardDef has a spell_type_filter that doesn't match the cast spell. Or better: extend the CardDef-based trigger scanning (in the SpellCast arm) to look up each permanent's CardDef triggers with `WheneverYouCastSpell` and check the filter against the spell's card types before adding the trigger.

#### 9h: SpellCast -- cast triggers (G-15)
**Action**: In the existing `GameEvent::SpellCast` arm, add scanning for the CAST SPELL's own CardDef triggered abilities with `WhenYouCastThisSpell`. The spell is on the stack (source_object_id), so look up its card_id, get the CardDef, and check for `WhenYouCastThisSpell` triggers. Push a PendingTrigger with the stack object as the source.

### Change 10: TriggerCondition → TriggerEvent mapping in enrich_spec_from_def

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add conversion arms for all new TriggerCondition variants:

| TriggerCondition | TriggerEvent | Notes |
|---|---|---|
| `WheneverYouDiscard` | `ControllerDiscards` | New |
| `WheneverOpponentDiscards` | `OpponentDiscards` | New |
| `WheneverYouSacrifice { .. }` | `ControllerSacrifices` | New |
| `WheneverYouAttack` | `ControllerAttacks` | New |
| `WhenLeavesBattlefield` | `SelfLeavesBattlefield` | New (look-back) |
| `WheneverYouDrawACard` | `ControllerDrawsCard` | **Existing TC, new TE** |
| `WheneverYouGainLife` | `ControllerGainsLife` | **Existing TC, new TE** |
| `WheneverPlayerDrawsCard { player_filter }` | `AnyPlayerDrawsCard` / `OpponentDrawsCard` / `ControllerDrawsCard` | Extended |
| `WhenYouCastThisSpell` | N/A (handled via CardDef scan in SpellCast arm) | Special |

For `WheneverYouCastSpell` and `WheneverOpponentCastsSpell`, the existing mappings to `ControllerCastsSpell` / `OpponentCastsSpell` remain -- the spell_type_filter is checked at trigger-collection time, not via TriggerEvent matching.

**Location**: ~L2000-2400 in `replay_harness.rs` (the triggered ability enrichment section)

### Change 11: Sacrifice filter matching

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `GameEvent::PermanentSacrificed` handler, after collecting triggers for `ControllerSacrifices`, filter out triggers whose CardDef has a `WheneverYouSacrifice { filter }` that doesn't match the sacrificed permanent. This requires:
1. Looking up the sacrificed permanent's last-known characteristics (it's already in the graveyard by now -- use `state.objects.get(new_id)` or look up the card_id from before the zone change).
2. For each collected trigger, check its CardDef's `WheneverYouSacrifice.filter` against the sacrificed object's types.

**Alternative**: Pass the sacrificed object's card_types through `entering_object_id` field of PendingTrigger (repurposed), and check the filter in `collect_triggers_for_event`. Follow the pattern used by `AnyCreatureDies` which passes the dying creature's info.

### Change 12: Exhaustive match updates

Files requiring new match arms for new variants:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `TriggerCondition::*` | ~L3896-3958 | Add hash arms for new TriggerCondition variants (discriminants 30-35) |
| `crates/engine/src/state/hash.rs` | `TriggerEvent::*` | ~L1750-1803 | Add hash arms for new TriggerEvent variants (discriminants 31-39) |
| `crates/engine/src/state/hash.rs` | `GameEvent::*` | somewhere | Add hash arm for `PermanentSacrificed` |
| `crates/engine/src/testing/replay_harness.rs` | TriggerCondition→TriggerEvent mapping | ~L2000-2400 | Add conversion arms for all new TC variants |
| `crates/engine/src/rules/abilities.rs` | `check_triggers` match on GameEvent | ~L2292-4592 | Add arms for CardDrawn, LifeGained, CardDiscarded, PermanentSacrificed |
| `crates/engine/src/cards/helpers.rs` | exports | top | No changes needed (no new types for card defs) |
| `crates/engine/src/rules/events.rs` | `GameEvent` enum | ~L53 | Add `PermanentSacrificed` variant |
| `crates/engine/src/rules/events.rs` | `is_private()` match | ~L1233 | Add `PermanentSacrificed => true/false` arm |
| `crates/engine/src/rules/events.rs` | `private_to()` match | check | Add arm if exists |

**Note on WheneverYouCastSpell/WheneverOpponentCastsSpell**: Changing these from simpler variants to struct variants with new fields will break ALL existing match arms referencing them. Every card def and every match site needs updating. The runner must grep for all occurrences and add `..` to destructuring patterns or add the new fields.

### Change 13: Sacrifice event emission

**Files** that perform sacrifice and must emit `PermanentSacrificed`:

| File | Location | Sacrifice type |
|------|----------|---------------|
| `crates/engine/src/effects/mod.rs` | `Effect::SacrificePermanents` (~L2120) | Effect-driven sacrifice |
| `crates/engine/src/rules/abilities.rs` | `sacrifice_self` cost (~L514) | Activated ability self-sacrifice |
| `crates/engine/src/rules/abilities.rs` | `sacrifice_filter` cost (search) | Activated ability other-sacrifice |
| `crates/engine/src/rules/abilities.rs` | Bargain/Emerge/Casualty additional costs | CastSpell sacrifice costs |
| `crates/engine/src/rules/abilities.rs` | Devour ETB sacrifice | Devour keyword |
| `crates/engine/src/rules/abilities.rs` | Champion ETB sacrifice | Champion keyword |
| `crates/engine/src/rules/resolution.rs` | Various sacrifice-as-effect paths | Resolution-time sacrifice |

Each site must emit `GameEvent::PermanentSacrificed` IN ADDITION TO the existing `CreatureDied`/`PermanentDestroyed` event. The sacrifice event is supplementary -- it carries the additional semantic that a sacrifice occurred (rather than destruction, SBA, etc.).

---

## Card Definition Fixes

### G-4: Spell-type filter on triggers (~19 cards)

These cards currently use `WheneverYouCastSpell { during_opponent_turn: false }` without a filter, or have TODOs. After adding the filter field, fix each:

#### talrand_sky_summoner.rs
**Oracle text**: "Whenever you cast an instant or sorcery spell, create a 2/2 blue Drake creature token with flying."
**Current state**: Uses unfiltered approximation
**Fix**: Add `spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]), noncreature_only: false`

#### guttersnipe.rs
**Oracle text**: "Whenever you cast an instant or sorcery spell, Guttersnipe deals 2 damage to each opponent."
**Current state**: Uses unfiltered approximation
**Fix**: Add `spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery])`

#### murmuring_mystic.rs
**Oracle text**: "Whenever you cast an instant or sorcery spell, create a 1/1 blue Bird Illusion creature token with flying."
**Current state**: TODO
**Fix**: Add `spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery])`

#### archmage_emeritus.rs
**Oracle text**: "Magecraft -- Whenever you cast or copy an instant or sorcery spell, draw a card."
**Current state**: TODO
**Fix**: Add `spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery])`

#### beast_whisperer.rs
**Oracle text**: "Whenever you cast a creature spell, draw a card."
**Current state**: TODO
**Fix**: Add `spell_type_filter: Some(vec![CardType::Creature])`

#### monastery_mentor.rs
**Oracle text**: "Whenever you cast a noncreature spell, create a 1/1 white Monk creature token with prowess."
**Current state**: TODO
**Fix**: Add `noncreature_only: true`

#### whispering_wizard.rs
**Oracle text**: "Whenever you cast a noncreature spell, create a 1/1 blue Wizard creature token."
**Current state**: TODO
**Fix**: Add `noncreature_only: true`

#### lys_alana_huntmaster.rs
**Oracle text**: "Whenever you cast an Elf spell, create a 1/1 green Elf Warrior creature token."
**Current state**: TODO
**Fix**: Add `spell_type_filter: Some(vec![CardType::Creature])` and note subtype filter gap (Elf subtype filtering on spells is beyond current TargetFilter scope -- mark as approximation)

#### sram_senior_edificer.rs
**Oracle text**: "Whenever you cast an Aura, Equipment, or Vehicle spell, draw a card."
**Current state**: TODO
**Fix**: This needs subtype filtering on spells which is beyond `spell_type_filter`. Mark as partial -- filter by artifact/enchantment types as approximation, note remaining gap.

#### bontus_monument.rs
**Oracle text**: "Whenever you cast a creature spell, each opponent loses 1 life and you gain 1 life."
**Current state**: TODO
**Fix**: Add `spell_type_filter: Some(vec![CardType::Creature])`

#### nezahal_primal_tide.rs
**Oracle text**: "Whenever an opponent casts a noncreature spell, draw a card."
**Current state**: TODO
**Fix**: Change to `WheneverOpponentCastsSpell { noncreature_only: true, spell_type_filter: None }`

#### mystic_remora.rs
**Oracle text**: "Whenever an opponent casts a noncreature spell, you may draw a card unless that player pays {4}."
**Current state**: TODO (using unfiltered trigger)
**Fix**: Change to `WheneverOpponentCastsSpell { noncreature_only: true, .. }` (MayPayOrElse still a gap)

#### archmage_of_runes.rs
**Oracle text**: "Whenever you cast an instant or sorcery spell, scry 1."
**Current state**: TODO
**Fix**: Add `spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery])`

#### beast_whisperer.rs (already listed above)

#### hazorets_monument.rs
**Oracle text**: "Whenever you cast a creature spell, you may discard a card. If you do, draw a card."
**Current state**: TODO (optional loot gap)
**Fix**: Add `spell_type_filter: Some(vec![CardType::Creature])`. Loot effect still needs MayDiscard pattern -- partial fix.

#### slickshot_show_off.rs
**Oracle text**: "Whenever you cast a noncreature spell, Slickshot Show-Off gets +2/+0 until end of turn."
**Current state**: TODO
**Fix**: Add `noncreature_only: true`

#### hermes_overseer_of_elpis.rs
**Oracle text**: "Whenever you cast a noncreature spell, create a 2/2 blue Bird creature token with flying."
**Current state**: TODO
**Fix**: Add `noncreature_only: true`

#### leaf_crowned_visionary.rs
**Oracle text**: "Whenever you cast an Elf spell, you may pay {G}. If you do, draw a card."
**Current state**: Ability removed to avoid wrong game state
**Fix**: Add `spell_type_filter: Some(vec![CardType::Creature])` as Elf approximation (subtype gap). MayPay still a gap.

#### storm_kiln_artist.rs
**Oracle text**: "Magecraft -- Whenever you cast or copy an instant or sorcery spell, create a Treasure token."
**Current state**: Check if it has a TODO
**Fix**: Add `spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery])`

#### chulane_teller_of_tales.rs
**Oracle text**: "Whenever you cast a creature spell, draw a card, then you may put a land card from your hand onto the battlefield."
**Current state**: TODO (spell filter + complex effect)
**Fix**: Add `spell_type_filter: Some(vec![CardType::Creature])`. Land-from-hand still a gap -- partial fix.

#### ajani_sleeper_agent.rs (emblem)
**Oracle text**: Emblem: "Whenever you cast a creature or planeswalker spell, gain 2 life and draw a card."
**Current state**: TODO (emblem lacks spell_type_filter)
**Fix**: Emblem trigger dispatch uses TriggeredAbilityDef. Add spell_type_filter field to TriggeredAbilityDef or handle at dispatch time.

### G-9: Discard triggers (~9 cards)

#### waste_not.rs
**Oracle text**: "Whenever an opponent discards a creature card, create a 2/2 black Zombie creature token. Whenever an opponent discards a land card, add {B}{B}. Whenever an opponent discards a noncreature, nonland card, draw a card."
**Current state**: All three abilities TODO
**Fix**: Use `WheneverOpponentDiscards` for all three. Note: per-card-type filtering on discarded card is a sub-gap -- implement the trigger, mark card-type filter as approximation.

#### lilianas_caress.rs
**Oracle text**: "Whenever an opponent discards a card, that player loses 2 life."
**Current state**: TODO
**Fix**: Use `WheneverOpponentDiscards` with `Effect::LoseLife { player: PlayerTarget::TriggeringPlayer, amount: Fixed(2) }`

#### megrim.rs
**Oracle text**: "Whenever an opponent discards a card, Megrim deals 2 damage to that player."
**Current state**: TODO
**Fix**: Use `WheneverOpponentDiscards` with `Effect::DealDamage { target: TriggeringPlayer, amount: Fixed(2) }`

#### raiders_wake.rs
**Oracle text**: "Whenever an opponent discards a card, that player loses 2 life."
**Current state**: TODO (first ability)
**Fix**: Use `WheneverOpponentDiscards`

#### fell_specter.rs
**Oracle text**: "Whenever an opponent discards a card, that player loses 2 life."
**Current state**: TODO
**Fix**: Use `WheneverOpponentDiscards`

#### glint_horn_buccaneer.rs
**Oracle text**: "Whenever you discard a card, Glint-Horn Buccaneer deals 1 damage to each opponent."
**Current state**: TODO
**Fix**: Use `WheneverYouDiscard`

#### brallin_skyshark_rider.rs
**Oracle text**: "Whenever you discard a card, Brallin deals 1 damage to each opponent."
**Current state**: TODO
**Fix**: Use `WheneverYouDiscard`

#### teferi_master_of_time.rs
**Oracle text**: Various loyalty abilities including loot (draw then discard)
**Current state**: TODO (WheneverYouDiscard gap noted)
**Fix**: Check if the TODO is specifically about a discard trigger or just the loot effect.

### G-10: Sacrifice triggers (~6+ cards)

#### korvold_fae_cursed_king.rs
**Oracle text**: "Whenever you sacrifice a permanent, put a +1/+1 counter on Korvold and draw a card."
**Current state**: TODO
**Fix**: Use `WheneverYouSacrifice { filter: None }`

#### camellia_the_seedmiser.rs
**Oracle text**: "Whenever you sacrifice one or more Foods, create a 1/1 green Squirrel creature token."
**Current state**: TODO
**Fix**: Use `WheneverYouSacrifice { filter: Some(TargetFilter { has_subtype: Some(SubType::Food), ..Default::default() }) }`

#### smothering_abomination.rs
**Oracle text**: "Whenever you sacrifice a creature, draw a card."
**Current state**: TODO
**Fix**: Use `WheneverYouSacrifice { filter: Some(TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() }) }`

#### captain_lannery_storm.rs
**Oracle text**: "Whenever Captain Lannery Storm attacks, create a Treasure token. Whenever you sacrifice a Treasure, Captain Lannery Storm gets +1/+0 until end of turn."
**Current state**: TODO (sacrifice trigger)
**Fix**: Use `WheneverYouSacrifice { filter: Some(TargetFilter { has_subtype: Some(SubType::Treasure), ..Default::default() }) }`

#### tireless_tracker.rs
**Oracle text**: "Whenever you sacrifice a Clue, put a +1/+1 counter on Tireless Tracker."
**Current state**: TODO (sacrifice trigger)
**Fix**: Use `WheneverYouSacrifice { filter: Some(TargetFilter { has_subtype: Some(SubType::Clue), ..Default::default() }) }`

#### juri_master_of_the_revue.rs
**Oracle text**: "Whenever you sacrifice a permanent, put a +1/+1 counter on Juri."
**Current state**: TODO
**Fix**: Use `WheneverYouSacrifice { filter: None }`

#### mirkwood_bats.rs
**Oracle text**: "Whenever you create or sacrifice a token, each opponent loses 1 life."
**Current state**: TODO (create half works, sacrifice half TODO)
**Fix**: Add `WheneverYouSacrifice` trigger for the sacrifice half. Token filter: `is_token` field may be needed on TargetFilter -- if not available, use unfiltered as approximation.

#### carmen_cruel_skymarcher.rs
**Oracle text**: "Whenever a player sacrifices a permanent, put a +1/+1 counter on Carmen."
**Current state**: TODO
**Fix**: Needs "any player sacrifices" -- could be `WheneverAnyPlayerSacrifices` or use `WheneverYouSacrifice` with any-player semantics. Since the original gap spec says "WheneverYouSacrifice", this card may need a broader variant. Evaluate: add a `player_filter` field to `WheneverYouSacrifice` (similar to G-13 draw trigger).

#### mayhem_devil.rs
**Oracle text**: "Whenever a player sacrifices a permanent, Mayhem Devil deals 1 damage to any target."
**Current state**: TODO
**Fix**: Same as Carmen -- needs any-player sacrifice trigger.

### G-11: WheneverYouAttack (~8 cards)

#### caesar_legions_emperor.rs
**Oracle text**: "Whenever you attack, create two 1/1 red and white Soldier creature tokens that are tapped and attacking."
**Current state**: TODO
**Fix**: Use `WheneverYouAttack` trigger

#### clavileno_first_of_the_blessed.rs
**Oracle text**: "Whenever you attack, up to one target nontoken Vampire you control gains flying and becomes a Demon in addition to its other types until end of turn."
**Current state**: TODO
**Fix**: Use `WheneverYouAttack` trigger. Complex effect (type change + flying grant) -- may be partial.

#### seasoned_dungeoneer.rs
**Oracle text**: "Whenever you attack, target creature can't be blocked this turn. Venture into the dungeon."
**Current state**: TODO
**Fix**: Use `WheneverYouAttack` trigger

#### chivalric_alliance.rs
**Oracle text**: "Whenever you attack with two or more creatures, draw a card."
**Current state**: TODO
**Fix**: Use `WheneverYouAttack` with a condition on attacker count (may need an intervening-if or a count field).

#### mishra_claimed_by_gix.rs
**Oracle text**: "Whenever you attack, each opponent loses X life and you gain X life, where X is the number of attacking creatures."
**Current state**: TODO
**Fix**: Use `WheneverYouAttack` trigger. Effect needs `EffectAmount::AttackingCreatureCount` or similar.

#### anim_pakal_thousandth_moon.rs
**Oracle text**: "Whenever you attack with one or more non-Gnome creatures, create a tapped and attacking 1/1 Gnome artifact creature token for each non-Gnome creature you control that's attacking."
**Current state**: TODO
**Fix**: Use `WheneverYouAttack`. Complex effect -- partial at best.

#### ainok_strike_leader.rs
**Oracle text**: Bolster-related attack trigger
**Current state**: TODO
**Fix**: Evaluate -- may be WhenAttacks (self) rather than WheneverYouAttack

#### hermes_overseer_of_elpis.rs
**Oracle text**: "Whenever you attack with one or more Birds, draw a card."
**Current state**: TODO
**Fix**: Use `WheneverYouAttack` with creature-type condition. Subtype filter on attackers is a sub-gap.

### G-12: LTB triggers (~6 cards)

#### aven_riftwatcher.rs
**Oracle text**: "When Aven Riftwatcher leaves the battlefield, you gain 2 life."
**Current state**: TODO
**Fix**: Use `WhenLeavesBattlefield` trigger

#### toothy_imaginary_friend.rs
**Oracle text**: "When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it."
**Current state**: TODO
**Fix**: Use `WhenLeavesBattlefield` trigger with `Effect::DrawCards { amount: EffectAmount::CountersOnSource(PlusOnePlusOne) }`

#### sengir_autocrat.rs
**Oracle text**: "When Sengir Autocrat leaves the battlefield, exile all Serf tokens."
**Current state**: TODO
**Fix**: Use `WhenLeavesBattlefield` trigger

#### nadiers_nightblade.rs
**Oracle text**: "Whenever a token you control leaves the battlefield, each opponent loses 1 life."
**Current state**: TODO (broader LTB trigger -- "a token you control leaves" not "self leaves")
**Fix**: This is NOT a self-LTB trigger. It's "whenever a token you control leaves." This needs a different variant (`WheneverTokenYouControlLeavesBattlefield`) -- beyond the scope of `WhenLeavesBattlefield`. Mark as partial/deferred.

#### tombstone_stairwell.rs
**Oracle text**: Complex enchantment with LTB and end-step triggers
**Current state**: TODO
**Fix**: Complex card -- evaluate what's fixable with `WhenLeavesBattlefield`.

### G-13: Draw-card trigger filtering (~16 cards)

These cards use draw triggers that need player_filter or need the existing CardDrawn dispatch wired.

#### Cards already using WheneverYouDrawACard (need dispatch wiring, no filter change):
- `niv_mizzet_parun.rs` -- "Whenever you draw a card, deal 1 damage to any target"
- `the_locust_god.rs` -- "Whenever you draw a card, create a 1/1 Insect"
- `chasm_skulker.rs` -- "Whenever you draw a card, put a +1/+1 counter"
- `psychosis_crawler.rs` -- "Whenever you draw a card, each opponent loses 1 life"
- `nadir_kraken.rs` -- "Whenever you draw a card, put a counter and create a Tentacle"
- `ominous_seas.rs` -- "Whenever you draw a card, put a foreshadow counter"
- `teferi_temporal_pilgrim.rs` -- "Whenever you draw a card, put a loyalty counter"

#### Cards needing WheneverYouDrawACard (currently TODO):
- `toothy_imaginary_friend.rs` -- "Whenever you draw a card, put a +1/+1 counter"
- `niv_mizzet_the_firemind.rs` -- "Whenever you draw a card, deal 1 damage to any target"

#### Cards needing opponent-draw filter (WheneverPlayerDrawsCard with player_filter):
- `smothering_tithe.rs` -- "Whenever an opponent draws a card" (currently uses unfiltered WheneverPlayerDrawsCard)
- `consecrated_sphinx.rs` -- "Whenever an opponent draws a card, you may draw two cards"
- `scrawling_crawler.rs` -- "Whenever an opponent draws a card, that player loses 1 life"
- `razorkin_needlehead.rs` -- "Whenever an opponent draws a card, deal 1 damage to them"
- `orcish_bowmasters.rs` -- "Whenever an opponent draws a card except the first one" (needs Nth-draw counter -- beyond scope, partial)
- `rhystic_study.rs` -- Check if already implemented or TODO

### G-14: Lifegain trigger wiring (~3 cards)

#### marauding_blight_priest.rs
**Oracle text**: "Whenever you gain life, each opponent loses 1 life."
**Current state**: Has `WheneverYouGainLife` trigger but it never fires (no dispatch wiring)
**Fix**: Dispatch wiring (Change 7/9b) fixes this automatically

#### vito_thorn_of_the_dusk_rose.rs
**Oracle text**: "Whenever you gain life, target opponent loses that much life."
**Current state**: TODO (needs "that much" amount reference)
**Fix**: Add `WheneverYouGainLife` trigger. The "that much life" needs `EffectAmount::TriggeringAmount` or similar -- may be a sub-gap.

#### elendas_hierophant.rs
**Oracle text**: "Whenever you gain life, put a +1/+1 counter on Elenda's Hierophant."
**Current state**: TODO
**Fix**: Use `WheneverYouGainLife` trigger

### G-15: Cast triggers (~5 cards)

#### prossh_skyraider_of_kher.rs
**Oracle text**: "When you cast this spell, create X 0/1 red Kobold creature tokens."
**Current state**: TODO
**Fix**: Use `WhenYouCastThisSpell` trigger. X = mana spent needs `EffectAmount::ManaSpentToCast` -- sub-gap.

#### elder_deep_fiend.rs
**Oracle text**: "When you cast this spell, tap up to four target permanents."
**Current state**: TODO
**Fix**: Use `WhenYouCastThisSpell` trigger

#### emrakul_the_promised_end.rs
**Oracle text**: "When you cast this spell, you gain control of target opponent during that player's next turn."
**Current state**: TODO
**Fix**: Complex -- control-change is beyond current DSL. Use `WhenYouCastThisSpell` as trigger, mark effect as TODO.

---

## Unit Tests

**File**: `crates/engine/tests/trigger_variants.rs` (new file)
**Tests to write**:

- `test_whenever_you_cast_creature_spell_filter` -- WheneverYouCastSpell with spell_type_filter only fires on creature spells (CR 603.2)
- `test_whenever_you_cast_noncreature_spell_filter` -- noncreature_only=true doesn't fire on creature spells
- `test_whenever_opponent_casts_noncreature_filter` -- WheneverOpponentCastsSpell with noncreature_only (CR 603.2)
- `test_whenever_you_discard_trigger` -- WheneverYouDiscard fires on CardDiscarded event (CR 701.9a)
- `test_whenever_opponent_discards_trigger` -- WheneverOpponentDiscards fires for opponents (CR 701.9a)
- `test_whenever_you_sacrifice_trigger` -- WheneverYouSacrifice fires on sacrifice (CR 701.21a)
- `test_whenever_you_sacrifice_with_filter` -- filter restricts to creature-only sacrifice
- `test_whenever_you_attack_trigger` -- WheneverYouAttack fires once per combat, not per creature (CR 508.1)
- `test_when_leaves_battlefield_trigger` -- WhenLeavesBattlefield fires on death (CR 603.10a)
- `test_when_leaves_battlefield_exile` -- WhenLeavesBattlefield fires on exile
- `test_when_leaves_battlefield_bounce` -- WhenLeavesBattlefield fires on bounce to hand
- `test_whenever_you_draw_card_trigger` -- WheneverYouDrawACard fires on CardDrawn (CR 603.2)
- `test_whenever_opponent_draws_card_trigger` -- WheneverPlayerDrawsCard with opponent filter
- `test_whenever_you_gain_life_trigger` -- WheneverYouGainLife fires on LifeGained (CR 603.2)
- `test_when_you_cast_this_spell_trigger` -- WhenYouCastThisSpell fires from stack before resolution
- `test_sacrifice_trigger_not_on_destruction` -- WheneverYouSacrifice does NOT fire on destruction/SBA death

**Pattern**: Follow tests for similar trigger features in `tests/trigger_variants.rs` or `tests/card_def_fixes.rs`

---

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] All existing card def TODOs for this batch resolved
- [ ] New card defs authored (if any)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs in affected card defs for G-4/G-9/G-10/G-11/G-12/G-13/G-14/G-15

---

## Risks & Edge Cases

1. **WheneverYouCastSpell struct variant change**: Changing from `WheneverYouCastSpell { during_opponent_turn: bool }` to a struct with 3 fields will break ~20+ existing card def files and match sites. Every `WheneverYouCastSpell { during_opponent_turn: false }` needs updating to include the new fields (or use `..`). Similarly, `WheneverOpponentCastsSpell` changing from unit to struct breaks existing uses. Grep and fix ALL sites.

2. **WheneverPlayerDrawsCard struct variant change**: Same issue -- changing from unit variant to struct with `player_filter` field breaks existing uses (smothering_tithe.rs, hash.rs).

3. **LTB look-back (CR 603.10a)**: The `WhenLeavesBattlefield` trigger must use last-known information. By the time `check_triggers` runs, the object has already moved zones. The trigger source's abilities must be looked up from the CardDef registry, not from the object's current characteristics. Follow the `SelfDies` pattern which already handles this.

4. **Sacrifice event emission scope**: Many code paths perform sacrifice (activated ability costs, additional cast costs, effects). ALL must emit `PermanentSacrificed`. Missing any path = silent trigger failure. Comprehensive grep needed.

5. **"Any player sacrifices" pattern**: Korvold's trigger is "whenever you sacrifice", but Mayhem Devil and Carmen are "whenever A PLAYER sacrifices." The initial `WheneverYouSacrifice` only covers controller. Consider adding `player_filter: Option<TargetController>` to `WheneverYouSacrifice` (default None = you only, Some(Any) = any player) to handle Carmen/Mayhem Devil.

6. **Spell subtype filtering**: Cards like Lys Alana Huntmaster ("Elf spell") and Sram ("Aura, Equipment, or Vehicle spell") need subtype filtering on SPELLS (on the stack), not permanents. The existing `TargetFilter` struct doesn't apply to stack objects. The `spell_type_filter` field handles card types (Creature, Instant, Sorcery) but not subtypes. These cards remain approximate.

7. **WhenYouCastThisSpell resolution timing**: The cast trigger goes on the stack ABOVE the spell. It resolves BEFORE the spell itself. The engine's `check_triggers` for `SpellCast` already fires before the spell resolves, so the trigger will naturally be placed above the spell on the stack. Verify this ordering in tests.

8. **Draw trigger batching**: Multiple CardDrawn events in a batch (e.g., "draw 3 cards") each fire the trigger separately (CR 603.2c: "an ability triggers only once each time its trigger event occurs" -- but drawing 3 cards is 3 separate draw events). Verify each CardDrawn fires independently.

9. **EffectAmount::TriggeringAmount**: Vito ("that much life") needs the amount from the triggering event. This may need a new `EffectAmount` variant or passing the amount through the PendingTrigger. Defer if complex.

10. **Sacrifice filter for tokens**: Mirkwood Bats ("whenever you create or sacrifice a token") needs to filter sacrifice triggers to token-only. TargetFilter may need an `is_token: bool` field. If not available, mark as approximation.

---

## Implementation Priority

Given the large scope (8 gaps), recommend implementing in this order within the batch:

1. **G-13 + G-14**: Wire CardDrawn and LifeGained dispatch (no new types, just match arms) -- unblocks ~19 cards immediately
2. **G-4**: Extend WheneverYouCastSpell with spell_type_filter -- many card fixes
3. **G-9**: Add discard triggers -- self-contained
4. **G-11**: Add WheneverYouAttack -- self-contained
5. **G-12**: Add WhenLeavesBattlefield -- needs LKI care
6. **G-10**: Add sacrifice triggers + PermanentSacrificed event -- most complex (many emission sites)
7. **G-15**: Add WhenYouCastThisSpell -- small, self-contained

Total estimated sessions: 3-4 (engine changes + card fixes + tests)
