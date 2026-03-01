# Ability Plan: Eternalize

**Generated**: 2026-03-01
**CR**: 702.129
**Priority**: P4
**Similar abilities studied**: Unearth (CR 702.84, `validated` — `rules/abilities.rs:622-792`, `rules/resolution.rs:857-952`, `state/stack.rs:244-262`, `tests/unearth.rs`), Myriad token copy (CR 702.116, `validated` — `rules/resolution.rs:1148-1274`)
**Dependency**: Embalm (CR 702.128) should be implemented first; Eternalize shares nearly all infrastructure and differs only in token modifications (black + 4/4 override vs white + original P/T).

## CR Rule Text

### 702.129. Eternalize

> **702.129a** Eternalize is an activated ability that functions while the card with eternalize
> is in a graveyard. "Eternalize [cost]" means "[Cost], Exile this card from your graveyard:
> Create a token that's a copy of this card, except it's black, it's 4/4, it has no mana cost,
> and it's a Zombie in addition to its other types. Activate only as a sorcery."

### 707.9 — Copy with Modifications

> **707.9** Copy effects may include modifications or exceptions to the copying process.
>
> **707.9b** Some copy effects modify a characteristic as part of the copying process.
> The final set of values for that characteristic becomes part of the copiable values of the copy.
>
> **707.9d** When applying a copy effect that doesn't copy a certain characteristic,
> retains one or more original values for a certain characteristic, or provides a specific
> set of values for a certain characteristic, any characteristic-defining ability of the
> object being copied that defines that characteristic is not copied. If that characteristic
> is color, any color indicator is also not copied. This rule does not apply to copy effects
> with exceptions that state the object is a certain card type "in addition to its other types."

### 707.2 — Copiable Values

> **707.2** When copying an object, the copy acquires the copiable values of the original
> object's characteristics [...] name, mana cost, color indicator, card type, subtype,
> supertype, rules text, power, toughness, and/or loyalty [...]

## Key Edge Cases

1. **Card exiled as cost, NOT on resolution (CR 702.129a, ruling 2017-07-14)**: "Once you've
   activated an eternalize ability, the card is immediately exiled. Opponents can't try to
   stop the ability by exiling the card." The exile is part of the activation cost (paid
   immediately), not an effect on resolution. This differs from Unearth where the card stays
   in the graveyard until the ability resolves.

2. **Token copies printed card only (ruling 2017-07-14)**: "The token copies exactly what was
   printed on the original card and nothing else, except the characteristics specifically
   modified by eternalize. It doesn't copy any information about the object the card was
   before it was put into your graveyard." This means counters, damage, continuous effects
   that modified the card are NOT copied -- only the original printed characteristics.

3. **Token modifications are copiable values (ruling 2017-07-14)**: "The token is a Zombie in
   addition to its other types and is black instead of its other colors. Its base power and
   toughness are 4/4. It has no mana cost, and thus its mana value is 0. These are copiable
   values of the token that other effects may copy." A Clone copying the eternalized token
   would also be a 4/4 black Zombie with no mana cost.

4. **ETB triggers fire normally (ruling 2017-07-14)**: "If the card copied by the token had
   any 'when [this permanent] enters the battlefield' abilities, then the token also has
   those abilities and will trigger them when it's created." The eternalized token entering
   the battlefield fires its own ETB triggers (e.g., Earthshaker Khenra's ETB).

5. **Sorcery speed restriction**: "Activate only as a sorcery" -- active player only, main
   phase only, empty stack. Same pattern as Unearth.

6. **Discard-as-additional-cost eternalize cards (Sinuous Striker, Sunscourge Champion)**:
   "You can't discard the card with eternalize to pay its own cost because the card has to
   be in your graveyard to begin activating its eternalize ability." The cost includes both
   mana and exile-from-graveyard; some cards also require discarding another card.

7. **Differences from Embalm (CR 702.128)**:
   - Embalm: token is WHITE, retains original P/T, no mana cost, Zombie in addition
   - Eternalize: token is BLACK, P/T overridden to 4/4, no mana cost, Zombie in addition
   - Both exile the card as cost (immediately, not at resolution)
   - Both are sorcery-speed activated abilities from graveyard

8. **Multiplayer**: No special multiplayer considerations beyond standard priority/APNAP.
   The ability is a normal activated ability that any player can respond to after it goes
   on the stack.

## Current State (from ability-wip.md)

The ability-wip.md currently tracks Retrace, not Eternalize. No Eternalize work has been done.

- [ ] Step 1: Enum variant (KeywordAbility::Eternalize)
- [ ] Step 2: AbilityDefinition::Eternalize { cost } variant
- [ ] Step 3: Command::EternalizeCard variant
- [ ] Step 4: StackObjectKind::EternalizeAbility variant
- [ ] Step 5: Handler (handle_eternalize_card)
- [ ] Step 6: Resolution logic
- [ ] Step 7: Unit tests
- [ ] Step 8: Card definition
- [ ] Step 9: Game script

## Implementation Steps

### Step 1: Enum Variant — KeywordAbility::Eternalize

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `Eternalize` variant to `KeywordAbility` enum after `CommanderNinjutsu`
(currently the last variant at line ~761).
**Pattern**: Follow `KeywordAbility::Unearth` at line ~469.

```rust
/// CR 702.129: Eternalize [cost] -- activated ability from graveyard.
/// "[Cost], Exile this card from your graveyard: Create a token that's a
/// copy of this card, except it's black, it's 4/4, it has no mana cost,
/// and it's a Zombie in addition to its other types. Activate only as a sorcery."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The eternalize cost itself is stored in `AbilityDefinition::Eternalize { cost }`.
Eternalize,
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` in the
`HashInto for KeywordAbility` impl, after `CommanderNinjutsu` (discriminant 88):
```rust
// Eternalize (discriminant 89) -- CR 702.129
KeywordAbility::Eternalize => 89u8.hash_into(hasher),
```

**Note**: If Embalm is implemented first and takes discriminant 89, Eternalize should use
the next available (90). Check at implementation time.

### Step 2: AbilityDefinition::Eternalize { cost }

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Eternalize { cost: ManaCost }` variant to `AbilityDefinition` enum.
**Pattern**: Follow `AbilityDefinition::Unearth { cost }` at line ~237.

```rust
/// CR 702.129: Eternalize [cost]. The card's eternalize ability can be activated
/// from its owner's graveyard by paying this cost and exiling the card.
/// When the ability resolves, create a token that's a copy of the card,
/// except it's black, 4/4, has no mana cost, and is a Zombie in addition
/// to its other types.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Eternalize)` for quick
/// presence-checking without scanning all abilities.
Eternalize { cost: ManaCost },
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` in the
`HashInto for AbilityDefinition` impl, after `CommanderNinjutsu` (discriminant 23):
```rust
// Eternalize (discriminant 24) -- CR 702.129
AbilityDefinition::Eternalize { cost } => {
    24u8.hash_into(hasher);
    cost.hash_into(hasher);
}
```

**Note**: If Embalm is implemented first and takes discriminant 24, Eternalize should use
the next available (25). Check at implementation time.

### Step 3: Command::EternalizeCard

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `EternalizeCard { player: PlayerId, card: ObjectId }` variant.
**Pattern**: Follow `Command::UnearthCard` at line ~397.

```rust
// -- Eternalize (CR 702.129) -----------------------------------------------
/// Activate a card's eternalize ability from the graveyard (CR 702.129a).
///
/// The card must be in the player's graveyard with `KeywordAbility::Eternalize`.
/// The card is exiled as part of the activation cost, and the eternalize
/// ability is placed on the stack. When it resolves, a token that's a copy
/// of the card is created, except it's black, 4/4, has no mana cost, and
/// is a Zombie in addition to its other types.
///
/// "Activate only as a sorcery" -- main phase, stack empty, active player.
///
/// Unlike `CastSpell`, this is an activated ability, not a spell cast.
/// No "cast" triggers fire.
EternalizeCard { player: PlayerId, card: ObjectId },
```

### Step 4: StackObjectKind::EternalizeAbility

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `EternalizeAbility` variant to `StackObjectKind`.
**Pattern**: Follow `StackObjectKind::UnearthAbility` at line ~252.

```rust
/// CR 702.129a: Eternalize activated ability on the stack.
///
/// When this ability resolves: create a token that's a copy of the source
/// card (which was exiled as part of activation cost), except the token is
/// black, 4/4, has no mana cost, and is a Zombie in addition to its other types.
///
/// `source_card_id` is the CardId of the exiled card (needed to look up the
/// card definition for creating the copy token). The original ObjectId is dead
/// after exile (CR 400.7), so we store the CardId for definition lookup.
/// `controller` is the player who activated the ability.
EternalizeAbility {
    source_card_id: Option<crate::cards::CardId>,
    source_name: String,
},
```

**Note**: Unlike Unearth (which stores `source_object: ObjectId` because the card stays in
graveyard), Eternalize exiles the card as a cost. After exile + zone change, the original
ObjectId is dead (CR 400.7). We store the `CardId` and name so resolution can look up the
card definition and create the copy token. This is the same approach that should be used for
Embalm.

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` in the
`HashInto for StackObjectKind` impl, after `NinjutsuAbility` (discriminant 26):
```rust
// EternalizeAbility (discriminant 27) -- CR 702.129a
StackObjectKind::EternalizeAbility { source_card_id, source_name } => {
    27u8.hash_into(hasher);
    source_card_id.hash_into(hasher);
    source_name.hash_into(hasher);
}
```

**Note**: If Embalm is implemented first and takes discriminant 27, Eternalize should use
the next available (28). Check at implementation time.

**TUI stack_view.rs**: Add arm to the exhaustive match at
`/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`:
```rust
StackObjectKind::EternalizeAbility { source_name, .. } => {
    (format!("Eternalize: {}", source_name), None)
}
```

**Replay viewer view_model.rs**: Add arm to `stack_kind_info` at
`/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`:
```rust
StackObjectKind::EternalizeAbility { .. } => {
    ("eternalize_ability", None)
}
```

**Counter match**: Add to the counter_stack_object match at
`/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs` (line ~2293-2318):
```rust
| StackObjectKind::EternalizeAbility { .. }
```

### Step 5: Handler — handle_eternalize_card

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add `handle_eternalize_card` function and `get_eternalize_cost` helper.
**Pattern**: Follow `handle_unearth_card` at line ~632-770, with key differences:
1. Card is EXILED as cost (not kept in graveyard like Unearth)
2. Stack object stores `CardId` + `name` (not `ObjectId`) because the card has already
   changed zones
3. The keyword check uses `KeywordAbility::Eternalize`
4. The cost lookup uses `AbilityDefinition::Eternalize { cost }`

```rust
/// Handle an EternalizeCard command: validate, pay cost, exile card, push ability onto stack.
///
/// CR 702.129a: Eternalize is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: Create a token that's a
/// copy of this card, except it's black, it's 4/4, it has no mana cost,
/// and it's a Zombie in addition to its other types. Activate only as a sorcery."
pub fn handle_eternalize_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // 1. Priority check (CR 602.2).
    // 2. Split second check (CR 702.61a).
    // 3. Zone check: card must be in player's graveyard.
    // 4. Keyword check: card must have KeywordAbility::Eternalize.
    // 5. Sorcery speed check (active player, main phase, empty stack).
    // 6. Look up eternalize cost from CardRegistry.
    // 7. Pay mana cost (CR 602.2b).
    // 8. **Exile the card as cost** (CR 702.129a: "Exile this card from your graveyard").
    //    - Capture card_id and name BEFORE the zone move (CR 400.7).
    //    - move_object_to_zone(card, ZoneId::Exile)
    // 9. Push EternalizeAbility { source_card_id, source_name } onto stack.
    // 10. Reset priority (CR 602.2e).
    // 11. Emit AbilityActivated + PriorityGiven events.
}
```

**Engine dispatch**: Add to `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
after the `UnearthCard` handler (line ~327-339):
```rust
Command::EternalizeCard { player, card } => {
    validate_player_active(&state, player)?;
    // CR 104.4b: eternalize is a meaningful player choice; reset loop detection.
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_eternalize_card(&mut state, player, card)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

**Cost lookup helper**:
```rust
/// CR 702.129a: Look up the eternalize cost from the card's `AbilityDefinition`.
fn get_eternalize_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    card_id.as_ref().and_then(|cid| {
        registry.get(cid.clone()).and_then(|def| {
            def.abilities.iter().find_map(|a| {
                if let AbilityDefinition::Eternalize { cost } = a {
                    Some(cost.clone())
                } else {
                    None
                }
            })
        })
    })
}
```

**Replay harness**: Add `eternalize_card` action type to
`/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs` in
`translate_player_action()`:
```rust
"eternalize_card" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    Some(Command::EternalizeCard {
        player,
        card: card_id,
    })
}
```

### Step 6: Resolution Logic

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `StackObjectKind::EternalizeAbility`.
**Pattern**: Hybrid of Unearth resolution (line ~857-952) and Myriad token copy (line ~1148-1274).

The resolution must:
1. Look up the card definition from `CardRegistry` using `source_card_id`.
2. Build a token GameObject using the card definition's printed characteristics (CR 707.2).
3. Apply the "except" modifications (CR 707.9b):
   - Color: set to Black only (remove all other colors, remove color indicator)
   - P/T: set to 4/4
   - Mana cost: set to None (mana value = 0)
   - Subtypes: add Zombie in addition to existing subtypes
4. Add the token to the battlefield via `state.add_object(token_obj, ZoneId::Battlefield)`.
5. Register a Layer 1 CopyOf continuous effect? **NO** -- unlike Myriad which copies a
   live battlefield permanent, Eternalize copies the printed card. The token is created
   directly with the correct characteristics from the card definition. No CopyOf effect
   is needed because the source card is in exile and may not be a valid `ObjectId` for
   copy resolution. Instead, build the token characteristics directly from the CardDefinition.
6. Fire ETB triggers (PermanentEnteredBattlefield event).
7. Apply self ETB replacements from definition.
8. Register static continuous effects from the token.

```rust
StackObjectKind::EternalizeAbility { source_card_id, source_name } => {
    let controller = stack_obj.controller;

    // Look up the card definition to get printed characteristics.
    let def = source_card_id.as_ref().and_then(|cid| {
        state.card_registry.get(cid.clone())
    });

    if let Some(card_def) = def {
        // Build token from printed characteristics (CR 707.2)
        // then apply eternalize modifications (CR 707.9b):
        // - Black instead of original colors
        // - 4/4 instead of original P/T
        // - No mana cost
        // - Zombie in addition to original types
        let mut chars = build_characteristics_from_definition(&card_def);

        // CR 702.129a modifications:
        chars.colors = im::ordset![Color::Black];
        chars.color_indicator = None;
        chars.mana_cost = None;
        chars.power = Some(4);
        chars.toughness = Some(4);
        if !chars.subtypes.contains(&SubType::Zombie) {
            chars.subtypes.insert(SubType::Zombie);
        }

        let token_obj = GameObject {
            id: ObjectId(0), // replaced by add_object
            card_id: source_card_id.clone(),
            characteristics: chars,
            controller,
            owner: controller,
            zone: ZoneId::Battlefield,
            status: ObjectStatus::default(),
            counters: im::OrdMap::new(),
            attachments: im::Vector::new(),
            attached_to: None,
            damage_marked: 0,
            deathtouch_damage: false,
            is_token: true,
            timestamp: 0, // replaced by add_object
            has_summoning_sickness: true,
            goaded_by: im::Vector::new(),
            kicker_times_paid: 0,
            was_evoked: false,
            is_bestowed: false,
            was_escaped: false,
            is_foretold: false,
            foretold_turn: 0,
            was_unearthed: false,
            myriad_exile_at_eoc: false,
            decayed_sacrifice_at_eoc: false,
            is_suspended: false,
            exiled_by_hideaway: None,
            is_renowned: false,
        };

        if let Ok(token_id) = state.add_object(token_obj, ZoneId::Battlefield) {
            // Fire ETB events and triggers.
            events.push(GameEvent::TokenCreated {
                player: controller,
                object_id: token_id,
            });

            // Apply self ETB replacements from definition.
            // Register static continuous effects.
            // Fire PermanentEnteredBattlefield event.
            // (Same pattern as resolution.rs permanent ETB site)

            events.push(GameEvent::PermanentEnteredBattlefield {
                player: controller,
                object_id: token_id,
            });
        }
    }

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

**Important**: A helper function `build_characteristics_from_definition(def: &CardDefinition) -> Characteristics`
should be created (or an existing one reused) to build a `Characteristics` struct from a
`CardDefinition`. This helper can be shared between Embalm and Eternalize. Check if
`enrich_spec_from_def` or a similar function already does this.

**Shared infrastructure with Embalm**: If Embalm is implemented first, the resolution logic
should be factored into a shared helper:
```rust
fn resolve_graveyard_copy_token(
    state: &mut GameState,
    controller: PlayerId,
    source_card_id: &Option<CardId>,
    color_override: OrdSet<Color>,
    pt_override: Option<(i32, i32)>,  // None = keep original, Some(p,t) = override
    events: &mut Vec<GameEvent>,
) -> Result<Option<ObjectId>, GameStateError>
```
Embalm would call with `color_override = {White}, pt_override = None`.
Eternalize would call with `color_override = {Black}, pt_override = Some((4, 4))`.
Both add Zombie subtype and remove mana cost.

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/eternalize.rs`
**Pattern**: Follow `tests/unearth.rs` structure.

**Tests to write**:

1. `test_eternalize_basic_flow` -- CR 702.129a
   - Card with eternalize in graveyard; activate; ability on stack; resolves;
     token on battlefield is 4/4 black Zombie with original subtypes.
   - Assert: original card in exile, token on battlefield with correct characteristics.

2. `test_eternalize_token_is_black_4_4` -- CR 702.129a, ruling 2017-07-14
   - Verify the token is black (not original color), 4/4 (not original P/T),
     has no mana cost, and is a Zombie in addition to original types.
   - Use a non-black card (e.g., blue creature) to verify color override.

3. `test_eternalize_card_exiled_as_cost` -- CR 702.129a, ruling 2017-07-14
   - After activating eternalize, card is immediately in exile (not graveyard).
   - Even before the ability resolves, card is gone from graveyard.

4. `test_eternalize_sorcery_speed` -- CR 702.129a
   - Cannot activate during opponent's turn, during combat, or with non-empty stack.
   - Assert InvalidCommand errors.

5. `test_eternalize_not_in_graveyard` -- CR 702.129a
   - Cannot activate if card is not in graveyard (e.g., on battlefield, in hand).
   - Assert InvalidCommand error.

6. `test_eternalize_insufficient_mana` -- CR 602.2b
   - Cannot activate without enough mana to pay eternalize cost.
   - Assert InsufficientMana error.

7. `test_eternalize_token_has_etb_abilities` -- ruling 2017-07-14
   - Card with an ETB trigger + eternalize; token should trigger ETB on creation.
   - Use Earthshaker Khenra or a simplified test card with ETB + eternalize.

8. `test_eternalize_not_a_cast` -- CR 702.129a
   - Eternalize is an activated ability, not a spell cast.
   - `spells_cast_this_turn` should not increment.
   - "Whenever you cast a spell" triggers should not fire.

9. `test_eternalize_token_zombie_subtype` -- CR 702.129a
   - Token has Zombie in addition to printed subtypes (not replacing them).
   - A "Human Warrior" becomes "Zombie Human Warrior".

10. `test_eternalize_no_mana_cost` -- CR 702.129a
    - Token has no mana cost (mana_cost = None).
    - Mana value of the token is 0.

11. `test_eternalize_split_second_blocks` -- CR 702.61a
    - Cannot activate eternalize when a spell with split second is on the stack.

12. `test_eternalize_keyword_retained` -- CR 702.129a, ruling 2017-07-14
    - If the original card had keywords (e.g., Haste, Vigilance), the token copies them.
    - A Steadfast Sentinel token should still have Vigilance.

### Step 8: Card Definition (later phase)

**Suggested card**: Proven Combatant
- Simplest eternalize card: vanilla 1/1 blue Human Warrior, `Eternalize {4}{U}{U}`
- No ETB abilities, no keywords beyond Eternalize -- cleanest test case.

**Card definition file**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/proven_combatant.rs`

```rust
CardDefinition {
    id: CardId("proven_combatant".into()),
    name: "Proven Combatant".into(),
    mana_cost: Some(ManaCost { blue: 1, ..Default::default() }), // {U}
    type_line: TypeLine {
        supertypes: OrdSet::new(),
        card_types: ordset![CardType::Creature],
        subtypes: ordset![SubType::Human, SubType::Warrior],
    },
    power: Some(1),
    toughness: Some(1),
    abilities: vec![
        AbilityDefinition::Keyword(KeywordAbility::Eternalize),
        AbilityDefinition::Eternalize {
            cost: ManaCost { generic: 4, blue: 2, ..Default::default() },
        },
    ],
    ..Default::default()
}
```

**Second showcase card** (for ETB testing): Earthshaker Khenra
- 2/1 red Jackal Warrior with Haste and an ETB trigger
- Eternalize {4}{R}{R}
- Tests that the 4/4 black Zombie token still triggers the ETB and retains Haste

### Step 9: Game Script (later phase)

**Suggested scenario**: "Proven Combatant eternalize creates 4/4 black Zombie token"
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Sequence number**: TBD (next available in stack directory)

Script outline:
1. Initial state: P1 has Proven Combatant in graveyard, 6 mana available ({4}{U}{U}).
2. P1 activates eternalize on Proven Combatant.
3. Assert: Proven Combatant is now in exile (cost paid).
4. All players pass priority.
5. Ability resolves: token created on battlefield.
6. Assert: token is on battlefield, is a creature, is black, is 4/4,
   has subtypes Zombie + Human + Warrior, has no mana cost, is a token.

## Interactions to Watch

1. **Interaction with Humility**: If Humility is on the battlefield, the eternalized token
   loses all abilities (including any copied ETB triggers). The 4/4 base P/T is a copiable
   value (CR 707.9b) and should be set at token creation, not via a continuous effect -- so
   Humility's Layer 7b P/T setting (1/1) would override the 4/4 in Layer 7b. This is
   correct behavior.

2. **Interaction with Panharmonicon**: The eternalized token entering the battlefield fires
   ETB triggers. Panharmonicon should double those triggers via the existing
   `TriggerDoubling` infrastructure, if applicable.

3. **Interaction with Rest in Peace**: If Rest in Peace is on the battlefield, the card
   cannot be in the graveyard (it would already be in exile). This is a non-issue -- the
   card would never be in the graveyard to activate eternalize from.

4. **Interaction with Leyline of the Void**: Same as Rest in Peace -- the card goes directly
   to exile when it would go to graveyard, so it never has a graveyard-based eternalize
   opportunity.

5. **Interaction with Grafdigger's Cage**: Grafdigger's Cage prevents creatures from
   entering the battlefield from graveyards (CR 702.129a). However, the eternalized card
   is exiled as a cost, and the TOKEN is created on the battlefield (not the card itself).
   Grafdigger's Cage does NOT prevent the token from being created. This is a key
   distinction from Unearth.

6. **Clone copying an eternalized token**: Per CR 707.9b, the modifications from eternalize
   (black, 4/4, no mana cost, Zombie) are copiable values. A Clone copying the token would
   also be a 4/4 black Zombie with no mana cost.

7. **Shared infrastructure with Embalm**: The handler, command, stack kind, and resolution
   logic should be nearly identical. The only differences in resolution are:
   - Color: Black (Eternalize) vs White (Embalm)
   - P/T: 4/4 override (Eternalize) vs keep original (Embalm)
   Factor the shared parts into helper functions if Embalm is implemented first.

## Files to Modify (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Eternalize` |
| `crates/engine/src/state/hash.rs` | Hash for KeywordAbility (disc 89+), AbilityDefinition (disc 24+), StackObjectKind (disc 27+) |
| `crates/engine/src/cards/card_definition.rs` | Add `AbilityDefinition::Eternalize { cost }` |
| `crates/engine/src/rules/command.rs` | Add `Command::EternalizeCard { player, card }` |
| `crates/engine/src/state/stack.rs` | Add `StackObjectKind::EternalizeAbility { source_card_id, source_name }` |
| `crates/engine/src/rules/abilities.rs` | Add `handle_eternalize_card`, `get_eternalize_cost` |
| `crates/engine/src/rules/engine.rs` | Add `Command::EternalizeCard` dispatch arm |
| `crates/engine/src/rules/resolution.rs` | Add `EternalizeAbility` resolution arm + counter match arm |
| `crates/engine/src/testing/replay_harness.rs` | Add `eternalize_card` action type |
| `tools/tui/src/play/panels/stack_view.rs` | Add `EternalizeAbility` match arm |
| `tools/replay-viewer/src/view_model.rs` | Add `EternalizeAbility` match arm in `stack_kind_info` |
| `crates/engine/src/lib.rs` | Verify re-exports (Command, StackObjectKind) |
| `crates/engine/tests/eternalize.rs` | New test file (12 tests) |
| `crates/engine/src/cards/defs/proven_combatant.rs` | New card definition |
