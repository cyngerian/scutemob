# Primitive Batch Plan: PB-22 S6 -- Emblem Creation (CR 114)

**Generated**: 2026-03-21
**Primitive**: Effect::CreateEmblem -- creates an emblem game object in the command zone with triggered/static abilities
**CR Rules**: CR 114.1-114.5, CR 113.6p
**Cards affected**: 6 (1 existing fix + 5 new)
**Dependencies**: PB-14 (planeswalker infrastructure) -- DONE
**Deferred items from prior PBs**: Emblem creation was deferred from PB-14 review (Finding 2, MEDIUM)

## Primitive Specification

Emblems are non-card, non-permanent objects that live in the command zone (CR 114.1). They have
no types, no mana cost, no color (CR 114.3), but have abilities defined by the effect that
created them (CR 114.2). Abilities of emblems function in the command zone (CR 113.6p, CR 114.4).
Emblems cannot be destroyed, exiled, or otherwise removed -- they persist for the rest of the game.

The engine needs:
1. A new `Effect::CreateEmblem` variant that creates a `GameObject` in the command zone
2. An `is_emblem: bool` field on `GameObject` to distinguish emblems from other command zone objects (commanders)
3. Exemption from the token SBA (line 476 of `sba.rs` removes tokens outside battlefield)
4. Trigger scanning extended to command zone emblem objects in `abilities.rs`
5. Static continuous effect registration for emblem static abilities

## CR Rule Text

**CR 114.1**: Some effects put emblems into the command zone. An emblem is a marker used to represent an object that has one or more abilities, but usually no other characteristics.

**CR 114.2**: An effect that creates an emblem is written "[Player] gets an emblem with [ability]." This means that [player] puts an emblem with [ability] into the command zone. The emblem is both owned and controlled by that player.

**CR 114.3**: An emblem has no characteristics other than the abilities defined by the effect that created it. In particular, an emblem has no types, no mana cost, and no color. Most emblems also have no name.

**CR 114.4**: Abilities of emblems function in the command zone.

**CR 114.5**: An emblem is neither a card nor a permanent. Emblem isn't a card type.

**CR 113.6p**: Abilities of emblems, plane cards, vanguard cards, scheme cards, and conspiracy cards function in the command zone.

## Engine Changes

### Change 1: Add `is_emblem` field to `GameObject`

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `pub is_emblem: bool` field (with `#[serde(default)]`) after `is_token` (line ~530).
**CR**: CR 114.5 -- emblems are distinct from cards and permanents; need a marker field.
**Pattern**: Follow `is_token: bool` at line 530.

### Change 2: Add `Effect::CreateEmblem` variant

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add new Effect variant after the last existing variant (currently `CreateTokenCopy`).
```rust
/// CR 114.1-114.4: Create an emblem in the command zone with the specified abilities.
/// Emblems have no types, mana cost, or color. They persist for the rest of the game.
CreateEmblem {
    /// Triggered abilities on the emblem (CR 114.2).
    triggered_abilities: Vec<EmblemAbility>,
    /// Static continuous effects on the emblem (e.g., "Ninjas you control get +1/+1").
    static_effects: Vec<ContinuousEffectDef>,
},
```
**CR**: CR 114.1-114.2

### Change 3: Add `EmblemAbility` type

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add a type alias or small struct near the Effect enum for emblem abilities.
Emblem triggered abilities use the existing `TriggeredAbilityDef` type from `game_object.rs`.
Emblem static effects use the existing `ContinuousEffectDef` type.
The `EmblemAbility` can simply be a type alias for `TriggeredAbilityDef`:
```rust
/// An ability on an emblem (CR 114.2). Uses the same structure as object triggered abilities.
pub type EmblemAbility = crate::state::game_object::TriggeredAbilityDef;
```
Or if we don't need a separate type, just use `TriggeredAbilityDef` directly in the Effect variant.

Decision: Use `TriggeredAbilityDef` directly -- no new type needed. The `CreateEmblem` variant holds `triggered_abilities: Vec<TriggeredAbilityDef>` and `static_effects: Vec<ContinuousEffectDef>`.

### Change 4: Add `ContinuousEffectDef` type (if not already present)

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Check if a serializable continuous effect definition type exists. The existing
`ContinuousEffect` in `state/continuous_effect.rs` includes runtime data (source ObjectId,
effect_id). For emblems, we need a definition that can be stored in the Effect variant and
then instantiated at creation time.

Use the existing `ApplyContinuousEffect { effect_def }` pattern -- the `effect_def` field
is a `ContinuousEffectDef` (or whatever type `ApplyContinuousEffect` uses). Check what type
that is and reuse it for emblem static effects.

**File to check**: Look at `Effect::ApplyContinuousEffect { effect_def }` to find the type.

### Change 5: Implement `Effect::CreateEmblem` dispatch

**File**: `crates/engine/src/effects/mod.rs`
**Action**: Add match arm for `Effect::CreateEmblem` after the last effect dispatch (~line 2430+).
**Pattern**: Follow `Effect::CreateToken` at line 516 for object creation pattern.

Implementation:
1. Create a new `GameObject` with:
   - `is_token: false` (emblems are not tokens -- CR 114.5)
   - `is_emblem: true`
   - `owner: ctx.controller`, `controller: ctx.controller` (CR 114.2)
   - `zone: ZoneId::Command(ctx.controller)`
   - Empty `characteristics` except for `triggered_abilities` and any static continuous effects
   - `card_id: None` (no physical card)
2. Add to `state.objects` and the command zone
3. Register static continuous effects (if any) immediately
4. Emit `GameEvent::EmblemCreated { player, object_id }`

### Change 6: Add `GameEvent::EmblemCreated`

**File**: `crates/engine/src/rules/events.rs`
**Action**: Add new variant after the last existing variant.
```rust
/// CR 114.2: A player gets an emblem with abilities.
EmblemCreated { player: PlayerId, object_id: ObjectId },
```
**CR**: CR 114.2

### Change 7: Exempt emblems from token SBA

**File**: `crates/engine/src/rules/sba.rs`
**Action**: Modify `check_token_sbas` at line 476 to exclude emblems:
```rust
.filter(|(_, obj)| obj.is_token && obj.zone != ZoneId::Battlefield && !obj.is_emblem)
```
Wait -- emblems have `is_token: false`, so they won't be caught by the token SBA filter.
Since we set `is_token: false` on emblems, NO change needed to `sba.rs`.

**IMPORTANT**: Verify that NO other SBA targets command zone objects. The existing SBAs all
filter on `zone == ZoneId::Battlefield` (confirmed by grep). No change needed.

### Change 8: Extend trigger scanning for command zone emblems

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Modify `collect_triggers_for_event` (line ~4734) to also scan emblem objects in the command zone.

Currently it only scans battlefield objects (line 4747):
```rust
.filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_phased_in())
```

Add a second pass for emblem objects:
```rust
// CR 113.6p, CR 114.4: Abilities of emblems function in the command zone.
let emblem_ids: Vec<ObjectId> = state
    .objects
    .values()
    .filter(|obj| obj.is_emblem && matches!(obj.zone, ZoneId::Command(_)))
    .map(|obj| obj.id)
    .collect();
for obj_id in emblem_ids {
    // ... same trigger matching logic as battlefield objects
}
```

The emblem scan must handle these TriggerEvents (used by the 6 known emblem abilities):
- `WheneverYouCastSpell` (Ajani -6: "whenever you cast a creature or planeswalker spell")
- `AtBeginningOfCombat` (Basri Ket -6: "at the beginning of combat on your turn")
- No trigger event needed for static +1/+1 (Kaito +1 -- uses continuous effect)
- `WheneverYouCastSpell` (Tyvar Kell -6: "whenever you cast an Elf spell")
- No trigger event (Wrenn and Realmbreaker -7 -- static "you may play" ability, complex)
- No trigger event (Wrenn and Seven -8 -- static "no maximum hand size", uses `KeywordAbility::NoMaxHandSize`)

The key insight: most emblem abilities are either triggered (scan command zone) or static
continuous effects (register at creation time). The trigger scan extension handles the
triggered case; static effects are handled in the CreateEmblem dispatch.

**HOWEVER**: The triggered ability matching logic in `check_triggers` (the main 5000+ line function) handles events case-by-case. Each event handler scans battlefield objects for matching `TriggerCondition`s. We need emblem triggers to fire for:
- `SpellCast` events -> emblem `WheneverYouCastSpell` triggers
- `StepChanged(Phase::Combat, Step::BeginningOfCombat)` -> emblem `AtBeginningOfCombat` triggers

The cleanest approach: add an emblem-scanning helper that runs AFTER the main battlefield trigger collection for relevant events. In each relevant event handler inside `check_triggers`, after scanning battlefield objects, also scan command zone emblems for matching trigger conditions.

Alternatively: factor out a shared `scan_objects_for_trigger` function that takes an iterator of objects and handles the trigger matching. This would be cleaner but a larger refactor.

**Recommended approach**: Add a helper function `collect_emblem_triggers` that:
1. Iterates all objects where `is_emblem == true`
2. For each emblem, checks if any of its `triggered_abilities` match the given event
3. Creates `PendingTrigger` entries for matches

Call this from `check_triggers` at the end of each relevant event handler (SpellCast, StepChanged for combat, etc.).

### Change 9: Register emblem static continuous effects

**File**: `crates/engine/src/effects/mod.rs` (within CreateEmblem dispatch)
**Action**: After creating the emblem object, register its static continuous effects
into `state.continuous_effects`. Use the existing `ContinuousEffect` struct with the
emblem's ObjectId as source. Duration is `EffectDuration::Permanent` (emblems persist forever).

For Kaito's "+1/+1 to Ninjas" emblem:
```rust
ContinuousEffect {
    source: emblem_id,
    effect_id: EffectId::new(),
    duration: EffectDuration::Permanent,
    layer: EffectLayer::PowerToughness,
    filter: EffectFilter::ControlledCreaturesWithSubtype("Ninja"),
    modification: LayerModification::AddPowerToughness(1, 1),
    timestamp: state.timestamp(),
}
```

For Wrenn and Seven's "no maximum hand size":
```rust
// This can be handled by setting a flag or using an existing mechanism.
// KeywordAbility::NoMaxHandSize is already a keyword -- grant it to the player.
// However, emblems affect players, not permanents. The "no maximum hand size"
// emblem modifies the player, not a permanent.
// Current engine: NoMaxHandSize is checked on objects, not players.
// This may need a player-level flag: player.no_max_hand_size = true.
// DEFER: Wrenn and Seven's emblem is the simplest case -- may need a small
// player-state addition. Document as a known gap if too complex.
```

### Change 10: Hash support for new types

**File**: `crates/engine/src/state/hash.rs`

**Effect::CreateEmblem** — discriminant 66 (next after CreateTokenCopy at 65):
```rust
Effect::CreateEmblem { triggered_abilities, static_effects } => {
    66u8.hash_into(hasher);
    (triggered_abilities.len() as u32).hash_into(hasher);
    for ta in triggered_abilities {
        ta.hash_into(hasher);
    }
    (static_effects.len() as u32).hash_into(hasher);
    for se in static_effects {
        se.hash_into(hasher);
    }
}
```

**GameEvent::EmblemCreated** — discriminant 124 (next after BecameCopyOf at 123):
```rust
GameEvent::EmblemCreated { player, object_id } => {
    124u8.hash_into(hasher);
    player.hash_into(hasher);
    object_id.hash_into(hasher);
}
```

**GameObject::is_emblem** — add `self.is_emblem.hash_into(hasher)` to the GameObject HashInto impl after `is_token`.

### Change 11: Exhaustive match updates

Files requiring new match arms for the new Effect/GameEvent variants:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/effects/mod.rs` | `match effect` | ~198 | Add `Effect::CreateEmblem` dispatch arm |
| `crates/engine/src/state/hash.rs` | `Effect` HashInto | ~4190 | Add hash arm (disc 66) |
| `crates/engine/src/state/hash.rs` | `GameEvent` HashInto | ~2633 | Add hash arm (disc 124) |
| `crates/engine/src/state/hash.rs` | `GameObject` HashInto | varies | Add `is_emblem` field hash |

**NOT required** (these don't match on Effect or GameEvent directly):
- `tools/replay-viewer/src/view_model.rs` -- does not match on `Effect` or `GameEvent` (confirmed by grep)
- `tools/tui/src/play/panels/stack_view.rs` -- does not match on `Effect` or `GameEvent` (confirmed by grep)

**GameEvent display** -- check if events.rs has a Display or fmt impl:

| File | Match expression | Action |
|------|-----------------|--------|
| `crates/engine/src/rules/events.rs` | GameEvent Display/Debug | Add arm for EmblemCreated if exhaustive |

### Change 12: Export new types in helpers.rs

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Add exports for `TriggeredAbilityDef`, `TriggerEvent`, `ContinuousEffectDef`
(or whatever types the `CreateEmblem` fields use) so card defs can reference them.
Check which of these are already exported.

## Card Definition Fixes

### 1. ajani_sleeper_agent.rs (EXISTING -- fix TODO)

**Oracle text**: "-6: You get an emblem with 'Whenever you cast a creature or planeswalker spell, target opponent gets two poison counters.'"
**Current state**: TODO at line 49 -- `Effect::Sequence(vec![])` placeholder
**Fix**: Replace the -6 loyalty ability effect with:
```rust
effect: Effect::CreateEmblem {
    triggered_abilities: vec![
        TriggeredAbilityDef {
            trigger_on: TriggerEvent::AnySpellCast,
            intervening_if: None,
            description: "Whenever you cast a creature or planeswalker spell, target opponent gets two poison counters.".to_string(),
            effect: Some(Effect::AddCounter {
                target: EffectTarget::TargetOpponent,
                counter: CounterType::Poison,
                count: EffectAmount::Fixed(2),
            }),
            etb_filter: None,
            targets: vec![TargetRequirement::Opponent],
        },
    ],
    static_effects: vec![],
},
```
Note: The trigger needs a spell-type filter (creature or planeswalker only). This may need a new filter field on the `TriggeredAbilityDef` or a `Condition` check. Check if existing `WheneverYouCastSpell` supports spell-type filtering; if not, use intervening-if or add a `spell_filter` field. The `AnySpellCast` TriggerEvent with controller check is the base; the creature/planeswalker filter is the challenge.

**Design decision for spell-type filtering**: The existing `TriggerCondition::WheneverYouCastSpell` does not filter by spell type. For Ajani's emblem ("whenever you cast a creature or planeswalker spell"), we need spell-type filtering. Options:
1. Add `spell_filter: Option<Vec<CardType>>` to `TriggeredAbilityDef`
2. Use `TriggerCondition` variants like `WheneverYouCastCreatureSpell`
3. Use an intervening-if Condition that checks the last cast spell's type

**Recommended**: Use option 1 -- add an optional `spell_type_filter: Option<Vec<CardType>>` field to `TriggeredAbilityDef`. This is the most general solution and matches how ETB filters work (via `etb_filter`). The trigger scanning code checks the spell's card types against the filter before creating a PendingTrigger.

### 2. basri_ket.rs (NEW)

**Oracle text**: "+1: Put a +1/+1 counter on up to one target creature. It gains indestructible until end of turn. / -2: Whenever one or more nontoken creatures attack this turn, create that many 1/1 white Soldier creature tokens that are tapped and attacking. / -6: You get an emblem with 'At the beginning of combat on your turn, create a 1/1 white Soldier creature token, then put a +1/+1 counter on each creature you control.'"
**Emblem ability**: Triggered -- `AtBeginningOfCombat`, creates token + distributes counters
**CardDefinition sketch**: Planeswalker, Basri subtype, {1}{W}{W}, loyalty 3, three LoyaltyAbility defs. The -6 uses `Effect::CreateEmblem` with a `TriggeredAbilityDef` for `AtBeginningOfCombat`.
**Complexity**: The -2 is a delayed trigger ("this turn"), which may need `ApplyContinuousEffect` or a turn-scoped trigger. The emblem -6 trigger creates a token AND adds counters -- use `Effect::Sequence`.
**Status**: New card def (not yet authored). The +1 and -2 have their own complexities beyond emblems. Author with emblem -6 implemented, +1 and -2 as partial/TODO if their specific mechanics aren't supported.

### 3. kaito_bane_of_nightmares.rs (NEW)

**Oracle text**: "Ninjutsu {1}{U}{B} / During your turn, as long as Kaito has one or more loyalty counters on him, he's a 3/4 Ninja creature and has hexproof. / +1: You get an emblem with 'Ninjas you control get +1/+1.' / 0: Surveil 2. Then draw a card for each opponent who lost life this turn. / -2: Tap target creature. Put two stun counters on it."
**Emblem ability**: Static -- Ninjas you control get +1/+1. This is a continuous effect, not a trigger.
**CardDefinition sketch**: Planeswalker, Kaito subtype, {2}{U}{B}, loyalty 4, Ninjutsu + 3 loyalty abilities. The +1 creates an emblem with a static P/T buff to Ninja creatures.
**Complexity**: The static "during your turn, creature + hexproof" is a continuous effect from self. The +1 emblem is a static continuous effect (Layer 7c P/T modification). Ninjutsu on a planeswalker is unusual. Multiple emblems stack (CR 113.2c -- each instance functions independently).
**Status**: New card def. Complex card -- Ninjutsu on planeswalker, self-animation, emblem with static effect. Author what's supported, TODO the rest.

### 4. tyvar_kell.rs (NEW)

**Oracle text**: "Elves you control have '{T}: Add {B}.' / +1: Put a +1/+1 counter on up to one target Elf. Untap it. It gains deathtouch until end of turn. / 0: Create a 1/1 green Elf Warrior creature token. / -6: You get an emblem with 'Whenever you cast an Elf spell, it gains haste until end of turn and you draw two cards.'"
**Emblem ability**: Triggered -- `WheneverYouCastSpell` with Elf subtype filter, grants haste + draws 2.
**Complexity**: The emblem trigger needs spell-type/subtype filtering (Elf spells). The "it gains haste" targets the spell that was just cast (needs reference to the triggering spell). This is more complex than a simple trigger -- the effect must reference the triggering object.
**Status**: New card def. The static mana ability for Elves and the emblem spell-subtype trigger are both complex. Author with emblem as best-effort.

### 5. wrenn_and_realmbreaker.rs (NEW)

**Oracle text**: "Lands you control have '{T}: Add one mana of any color.' / +1: ... / -2: ... / -7: You get an emblem with 'You may play lands and cast permanent spells from your graveyard.'"
**Emblem ability**: Static -- modifies game rules to allow playing from graveyard. This is a complex static ability that modifies `legal_actions.rs` to check graveyard for playable cards.
**Complexity**: VERY HIGH. "Play lands from graveyard" + "cast permanent spells from graveyard" are rule-modifying effects. This is similar to Yawgmoth's Will or Crucible of Worlds. The engine's `legal_actions.rs` would need to scan the graveyard for castable spells when this emblem exists.
**Status**: New card def. The emblem ability is extremely complex -- likely TODO. Author the card structure with emblem creation effect, but the actual rule-modification is a separate primitive gap.

### 6. wrenn_and_seven.rs (NEW)

**Oracle text**: "+1: Reveal top four, lands to hand, rest to graveyard. / 0: Put any number of land cards from hand onto battlefield tapped. / -3: Create Treefolk token with reach and CDA. / -8: Return all permanent cards from graveyard to hand. You get an emblem with 'You have no maximum hand size.'"
**Emblem ability**: Static -- player has no maximum hand size. `KeywordAbility::NoMaxHandSize`.
**Complexity**: The engine already has `NoMaxHandSize` as a keyword on permanents. For an emblem granting it to a player, we need to check if the player has an emblem with this effect during the cleanup step hand-size check. Alternatively, set a flag on `PlayerState`.
**Status**: New card def. The emblem is relatively simple IF we add a `no_max_hand_size: bool` to `PlayerState` (set when the emblem is created). The -8's "return all permanent cards" is `MoveZone` from graveyard to hand with a permanent-type filter.

## Implementation Simplifications

Given the complexity of some emblem abilities, the runner should implement the core infrastructure
and the simplest emblem cases first. Recommended approach:

**Must implement** (core infrastructure):
1. `is_emblem` field on `GameObject`
2. `Effect::CreateEmblem` variant and dispatch
3. `GameEvent::EmblemCreated`
4. Hash support
5. Emblem trigger scanning in `check_triggers` / `collect_triggers_for_event`

**Implement for card defs** (feasible with current DSL):
1. Ajani Sleeper Agent -6 (triggered, needs spell-type filter -- may need `spell_type_filter` field on TriggeredAbilityDef)
2. Wrenn and Seven -8 emblem (static no max hand size -- needs `no_max_hand_size` flag on PlayerState, or use emblem with NoMaxHandSize keyword and scan during cleanup)
3. Basri Ket -6 (triggered at beginning of combat -- straightforward with `AtBeginningOfCombat`)
4. Kaito +1 (static P/T buff -- continuous effect registration at emblem creation)

**May need TODOs** (complex interactions beyond emblem infrastructure):
1. Tyvar Kell -6 (spell-subtype filter + reference to triggering spell for "it gains haste")
2. Wrenn and Realmbreaker -7 ("play from graveyard" -- rule-modifying, separate primitive)

## Unit Tests

**File**: `crates/engine/tests/emblem_tests.rs` (new file)
**Tests to write**:

1. `test_emblem_creation_basic` -- CR 114.1: Activate planeswalker -6, verify emblem object exists in command zone with correct controller/owner, `is_emblem: true`, `is_token: false`. Use Ajani Sleeper Agent.

2. `test_emblem_triggered_ability_fires` -- CR 114.4, CR 113.6p: Create Ajani's emblem, then cast a creature spell. Verify the emblem's triggered ability fires (opponent gets 2 poison counters). Tests that trigger scanning finds command zone emblem objects.

3. `test_emblem_survives_board_wipe` -- CR 114.3: Create emblem, then destroy all permanents. Verify emblem still exists in command zone and its abilities still function.

4. `test_emblem_not_removed_by_token_sba` -- CR 114.5: Verify emblems are not cleaned up by the token SBA (they have `is_token: false`).

5. `test_multiple_emblems_stack` -- CR 113.2c: Create two instances of the same emblem. Verify both fire independently (e.g., two Ajani emblems = 4 poison counters per creature spell cast).

6. `test_emblem_static_effect` -- CR 114.4: Create Kaito's emblem (+1/+1 to Ninjas). Verify a Ninja creature on the battlefield gets +1/+1 from the emblem's continuous effect.

7. `test_emblem_persists_after_source_removed` -- CR 113.7a: Create emblem via planeswalker, then destroy the planeswalker. Verify emblem persists and its abilities still function.

**Pattern**: Follow tests in `crates/engine/tests/copy_effects.rs` for Effect-level testing with GameStateBuilder.

## Verification Checklist

- [ ] Engine primitive compiles (`cargo check`)
- [ ] `is_emblem` field added to GameObject with `#[serde(default)]`
- [ ] `Effect::CreateEmblem` variant added and dispatched
- [ ] `GameEvent::EmblemCreated` variant added
- [ ] Hash discriminants: Effect 66, GameEvent 124
- [ ] Emblem trigger scanning in abilities.rs
- [ ] Ajani Sleeper Agent -6 TODO resolved (emblem creation)
- [ ] New card defs authored (Basri Ket, Kaito, Tyvar Kell, Wrenn and Seven, Wrenn and Realmbreaker -- at minimum structural defs with emblem abilities)
- [ ] Unit tests pass (7+ tests)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining emblem-related TODOs in affected card defs

## Step-by-Step Implementation Order

1. **Add `is_emblem` to `GameObject`** (game_object.rs, line ~530) -- add field with `#[serde(default)]`, add to hash
2. **Add `Effect::CreateEmblem` variant** (card_definition.rs) -- use `TriggeredAbilityDef` and `ContinuousEffectDef` for fields
3. **Add `GameEvent::EmblemCreated` variant** (events.rs)
4. **Add hash support** (hash.rs) -- Effect disc 66, GameEvent disc 124, `is_emblem` in GameObject hash
5. **Implement CreateEmblem dispatch** (effects/mod.rs) -- create GameObject, add to command zone, register static CEs, emit event
6. **Extend trigger scanning** (abilities.rs) -- add emblem scanning to `collect_triggers_for_event` and relevant event handlers in `check_triggers`
7. **Export types in helpers.rs** -- ensure `TriggeredAbilityDef`, `TriggerEvent` etc. are importable by card defs
8. **Fix Ajani Sleeper Agent** (defs/ajani_sleeper_agent.rs) -- replace -6 TODO with CreateEmblem
9. **Author new card defs** -- Basri Ket, Kaito, Tyvar Kell, Wrenn and Seven, Wrenn and Realmbreaker
10. **Write unit tests** (tests/emblem_tests.rs) -- 7 tests
11. **Verify workspace** -- `cargo build --workspace`, `cargo test --all`, `cargo clippy`

## Discriminant Chain Summary

| Type | Current Max | New Value | Variant |
|------|-------------|-----------|---------|
| Effect | 65 (CreateTokenCopy) | 66 | CreateEmblem |
| GameEvent | 123 (BecameCopyOf) | 124 | EmblemCreated |
| TriggerCondition | 27 (WhenSelfBecomesTapped) | -- | no change |
| Condition | 31 (CardTypesInGraveyardAtLeast) | -- | no change |

## Risks & Edge Cases

- **Spell-type filtering for Ajani/Tyvar emblems**: The current `TriggerCondition::WheneverYouCastSpell` does not filter by spell type (creature/planeswalker/Elf). Need to either add a `spell_type_filter` to `TriggeredAbilityDef` or create new TriggerCondition variants. The former is more general-purpose.

- **Static emblem effects (Kaito +1/+1)**: Static continuous effects on emblems need to be registered into `state.continuous_effects` at creation time with `EffectDuration::Permanent`. The layer system already handles CEs from any source -- the key is that the CE's `source` ObjectId points to the emblem, which persists in the command zone.

- **"You may play from graveyard" (Wrenn and Realmbreaker)**: This is a game-rules-modifying static ability that is NOT implementable with current DSL primitives. It requires changes to `legal_actions.rs` to scan the graveyard for playable cards when this emblem exists. Document as TODO in the card def.

- **"No maximum hand size" (Wrenn and Seven)**: The engine checks `NoMaxHandSize` as a keyword on permanents. Emblems are not permanents. Either add a `no_max_hand_size: bool` to `PlayerState` or extend the cleanup hand-size check to scan command zone emblems for this keyword.

- **Token SBA race**: Since emblems have `is_token: false`, they won't be caught by `check_token_sbas`. Verified safe.

- **Emblem trigger timing**: Emblem triggers must go on the stack at the same time as other triggered abilities (APNAP order). The `check_triggers` function already collects all triggers and sorts by APNAP -- the emblem triggers just need to be added to the same collection.

- **Multiple emblems**: Each +1 activation of Kaito creates a NEW emblem. Multiple Kaito emblems stack (each adds +1/+1 independently). The continuous effect system handles this naturally since each has a unique source ObjectId.

- **Emblem owner dies in multiplayer**: When a player loses the game, all objects they own are removed (CR 800.4a). This should naturally clean up their emblems since `owner == losing_player`. Verify the player-loss cleanup scans command zone objects.
