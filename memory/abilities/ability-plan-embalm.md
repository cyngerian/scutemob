# Ability Plan: Embalm

**Generated**: 2026-03-01
**CR**: 702.128
**Priority**: P4
**Batch**: 4.4 (Alternate Casting from Graveyard)
**Similar abilities studied**: Unearth (CR 702.84) -- `rules/abilities.rs`, `rules/resolution.rs`, `rules/engine.rs`, `state/stack.rs`, `state/types.rs`, `cards/card_definition.rs`; Myriad token copy (CR 702.116) -- `rules/resolution.rs:1148-1290`

## CR Rule Text

```
702.128. Embalm

702.128a Embalm is an activated ability that functions while the card with embalm is in
a graveyard. "Embalm [cost]" means "[Cost], Exile this card from your graveyard: Create
a token that's a copy of this card, except it's white, it has no mana cost, and it's a
Zombie in addition to its other types. Activate only as a sorcery."

702.128b A token is "embalmed" if it's created by a resolving embalm ability.
```

Related copy rules (CR 707.9):
- **CR 707.9a**: Copy effects that cause the copy to gain an ability -- that ability
  becomes part of the copiable values.
- **CR 707.9b**: Copy effects that modify a characteristic -- the final set of values
  for that characteristic becomes part of the copiable values.
- **CR 707.9d**: When applying a copy effect that doesn't copy a certain characteristic
  (e.g., mana cost set to "none"), any characteristic-defining ability that defines that
  characteristic is not copied. However, the exception for "in addition to its other types"
  means subtypes/types CDAs ARE copied.

## Key Edge Cases

1. **Card is exiled as part of the cost** (CR 702.128a: "[Cost], Exile this card from your
   graveyard"). Unlike Unearth where the card stays in graveyard until resolution, Embalm
   exiles the card immediately as cost payment. Ruling 2017-07-14: "Once you've activated an
   embalm ability, the card is immediately exiled. Opponents can't try to stop the ability by
   exiling the card with an effect."

2. **Token copies printed values only** (ruling 2017-04-18): "The token copies exactly what
   was printed on the original card and nothing else. It doesn't copy any information about
   the object the card was before it was put into your graveyard." Counters, auras, equipment,
   and continuous effects on the card at the time of its death are NOT carried over.

3. **Color override**: "except it's white" -- replaces ALL original colors with White only.
   Per CR 707.9b, this modified color becomes the copiable value. A green creature embalmed
   produces a white token, not a white-green token.

4. **No mana cost**: "it has no mana cost" -- the token has no mana cost, so its mana value
   is 0 (ruling 2017-04-18). Per CR 707.9d, any CDA that defines color from mana cost is
   not copied (but color is overridden to white anyway). The token's color identity for
   Commander purposes would be just White.

5. **Zombie subtype added** (CR 707.9a/707.9d exception): "it's a Zombie in addition to its
   other types" -- adds SubType("Zombie") to the token's subtypes. Per CR 707.9d, the
   "in addition to its other types" exception means creature type CDAs (like Changeling)
   ARE still copied.

6. **Sorcery speed only**: "Activate only as a sorcery" -- same sorcery-speed validation as
   Unearth (active player, main phase, empty stack).

7. **Not a cast**: Embalm is an activated ability, not a spell cast. No "cast" triggers fire.
   No storm count increase. No prowess trigger.

8. **Token keeps printed abilities**: The token is a copy of the card as printed (oracle text),
   so it retains all the card's printed abilities (including Embalm keyword -- though the token
   can never use it since tokens in the graveyard cease to exist before the SBA that would allow
   Embalm activation).

9. **Multiplayer**: No special multiplayer interaction beyond sorcery-speed restriction (only
   during your own turn). All opponents can respond to the Embalm ability on the stack.

10. **Source card already exiled at resolution**: Since the card is exiled as cost, the ability
    does NOT need to check if the card is still in the graveyard at resolution time (unlike
    Unearth). The ability always creates the token based on the card's definition. If the ability
    is countered (e.g., by Stifle), the card stays in exile and no token is created.

## Current State (from ability-wip.md)

No existing implementation. The ability-wip.md currently tracks Retrace, not Embalm. Embalm
status in `docs/mtg-engine-ability-coverage.md` is `none`.

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring (n/a -- no triggers needed)
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant and Types

#### 1a: KeywordAbility variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Embalm` variant after `CommanderNinjutsu` (line ~761)
**Pattern**: Follow `KeywordAbility::Unearth` at line 461-469

```rust
/// CR 702.128: Embalm [cost] -- activated ability from graveyard.
/// "[Cost], Exile this card from your graveyard: Create a token that's a copy
/// of this card, except it's white, it has no mana cost, and it's a Zombie in
/// addition to its other types. Activate only as a sorcery."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The embalm cost itself is stored in `AbilityDefinition::Embalm { cost }`.
Embalm,
```

#### 1b: AbilityDefinition variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Embalm { cost: ManaCost }` after `CommanderNinjutsu { cost }` (line ~278)
**Pattern**: Follow `AbilityDefinition::Unearth { cost }` at line 228-237

```rust
/// CR 702.128: Embalm [cost]. The card's embalm ability can be activated from
/// its owner's graveyard by paying this cost plus exiling the card. When the
/// ability resolves, create a token copy of the card that is white, has no
/// mana cost, and is a Zombie in addition to its other types.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Embalm)` for quick
/// presence-checking without scanning all abilities.
Embalm { cost: ManaCost },
```

#### 1c: StackObjectKind variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::EmbalmAbility { source_card_id: Option<CardId> }` after `NinjutsuAbility`
**Pattern**: Follow `StackObjectKind::UnearthAbility { source_object }` at line 244-252

NOTE: Embalm is different from Unearth here. Because the card is exiled as part of the cost
(before the ability goes on the stack), the original ObjectId is DEAD by the time the ability
resolves. The ability needs the card's `CardId` (registry key) to look up characteristics for
the token, NOT the `ObjectId`. This is a key difference from Unearth.

```rust
/// CR 702.128a: Embalm activated ability on the stack.
///
/// When this ability resolves: create a token that's a copy of the source card,
/// except it's white, has no mana cost, and is a Zombie in addition to its
/// other types (CR 702.128a).
///
/// `source_card_id` is the CardId (registry key) of the card that was exiled as cost.
/// The original card was exiled during activation (cost payment), so no ObjectId is
/// available at resolution time (CR 400.7). The token's characteristics come from
/// the CardDefinition in the registry.
EmbalmAbility { source_card_id: Option<CardId> },
```

#### 1d: Command variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `Command::EmbalmCard { player: PlayerId, card: ObjectId }` after `UnearthCard`
**Pattern**: Follow `Command::UnearthCard { player, card }` at line 384-397

```rust
// -- Embalm (CR 702.128) ------------------------------------------------
/// Activate a card's embalm ability from the graveyard (CR 702.128a).
///
/// The card must be in the player's graveyard with `KeywordAbility::Embalm`.
/// The embalm cost is paid, the card is exiled (as part of the cost), and
/// the embalm ability is placed on the stack. When it resolves, a token copy
/// of the card is created (white, no mana cost, Zombie added to types).
///
/// Unlike Unearth, the card is exiled as part of the activation cost, NOT
/// when the ability resolves.
EmbalmCard { player: PlayerId, card: ObjectId },
```

#### 1e: Hash discriminants

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash entries for the new variants

- `KeywordAbility::Embalm` -- discriminant **89** (next after CommanderNinjutsu=88)
- `StackObjectKind::EmbalmAbility` -- discriminant **27** (next after NinjutsuAbility=26)
- `AbilityDefinition::Embalm` -- discriminant **24** (next after CommanderNinjutsu=23)

#### 1f: Match arm updates

Files that need new match arms for `StackObjectKind::EmbalmAbility`:

1. `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs` -- add:
   ```rust
   StackObjectKind::EmbalmAbility { .. } => {
       ("Embalm: ".to_string(), None) // No source_object; card was exiled
   }
   ```

2. `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs` -- add:
   ```rust
   StackObjectKind::EmbalmAbility { .. } => {
       ("embalm_ability", None) // No source_object
   }
   ```

3. `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs` -- counter_stack_object
   function (~line 2293): add `| StackObjectKind::EmbalmAbility { .. }` to the non-spell
   counter arm.

Files that need new match arms for `Command::EmbalmCard`:
- `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs` -- process_command handler

### Step 2: Rule Enforcement -- Activation Handler

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add `handle_embalm_card()` function and `get_embalm_cost()` helper
**Pattern**: Follow `handle_unearth_card()` at line 626-771 and `get_unearth_cost()` at line 777-792
**CR**: 702.128a

The handler must:

1. **Priority check** (CR 602.2): player must hold priority.
2. **Split second check** (CR 702.61a): cannot activate while split second is on stack.
3. **Zone check** (CR 702.128a): card must be in player's own graveyard.
4. **Keyword check** (CR 702.128a): card must have `KeywordAbility::Embalm`.
5. **Sorcery speed check** (CR 702.128a: "Activate only as a sorcery"):
   - Active player only
   - Main phase only (PreCombatMain or PostCombatMain)
   - Empty stack
6. **Look up embalm cost** from `AbilityDefinition::Embalm { cost }` in CardRegistry.
7. **Pay mana cost** (CR 602.2b).
8. **Exile the card from graveyard** (cost payment, CR 702.128a: "[Cost], Exile this card
   from your graveyard"). CRITICAL DIFFERENCE FROM UNEARTH: the card is exiled immediately
   as part of cost payment, not at resolution time. Use `state.move_object_to_zone(card, ZoneId::Exile)`.
9. **Capture CardId** before exiling (the ObjectId will be dead after zone move).
10. **Push the embalm ability onto the stack** as `StackObjectKind::EmbalmAbility { source_card_id }`.
    The card_id is what resolution uses to look up the token's characteristics.
11. **Reset priority** (CR 602.2e): active player gets priority.
12. **Emit events**: `AbilityActivated`, `ObjectExiled`, `PriorityGiven`.

```rust
/// Handle an EmbalmCard command: validate, pay cost, exile card, push ability onto stack.
///
/// CR 702.128a: Embalm is an activated ability from the graveyard.
/// "[Cost], Exile this card from your graveyard: Create a token that's a copy of
/// this card, except it's white, it has no mana cost, and it's a Zombie in addition
/// to its other types. Activate only as a sorcery."
pub fn handle_embalm_card(
    state: &mut GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<Vec<GameEvent>, GameStateError> {
    // Steps 1-7: identical validation pattern to handle_unearth_card
    // Step 8: Exile the card (cost payment) -- DIFFERENT from Unearth
    // Step 9: Capture card_id
    // Step 10: Push EmbalmAbility onto stack
    // Step 11-12: Reset priority, emit events
}

/// CR 702.128a: Look up the embalm cost from the card's `AbilityDefinition`.
fn get_embalm_cost(
    card_id: &Option<CardId>,
    registry: &crate::cards::CardRegistry,
) -> Option<ManaCost> {
    // Pattern: same as get_unearth_cost but matches AbilityDefinition::Embalm
}
```

### Step 3: Rule Enforcement -- Resolution Handler

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::EmbalmAbility` resolution arm
**Pattern**: Follow `StackObjectKind::MyriadTrigger` at line 1148-1290 (token copy creation)
  AND `StackObjectKind::UnearthAbility` at line 857-943 (graveyard activation resolution)
**CR**: 702.128a, 707.9b, 707.9d

The resolution must:

1. **Look up CardDefinition** from `source_card_id` in the card registry.
2. **Build token characteristics** from the card definition (CR 707.2 -- copiable values):
   - Name: same as the card
   - Power/Toughness: same as the card
   - Card types: same as the card (creature, etc.)
   - Subtypes: same as the card PLUS `SubType("Zombie")` (CR 702.128a: "in addition to its other types")
   - Colors: `{ Color::White }` only (CR 702.128a: "except it's white")
   - Keywords: same as the card (all printed keywords are copied)
   - Mana cost: None (CR 702.128a: "it has no mana cost")
   - Mana abilities: same as the card (if any)
   - Activated abilities: same as the card (if any)
3. **Create a GameObject** for the token:
   - `is_token: true`
   - `card_id: source_card_id.clone()` (so the token is linked to the card definition
     for layer system and ability resolution)
   - `controller: stack_obj.controller`
   - `owner: stack_obj.controller`
   - `zone: ZoneId::Battlefield`
   - `has_summoning_sickness: true` (CR 302.6)
   - All other flags default (not unearthed, not decayed, etc.)
4. **Add the token to the battlefield** via `state.add_object()`.
5. **Run ETB pipeline** (same as Unearth/Myriad resolution):
   - `apply_self_etb_from_definition`
   - `apply_etb_replacements`
   - `register_permanent_replacement_abilities`
   - `register_static_continuous_effects`
   - Emit `PermanentEnteredBattlefield`
   - `fire_when_enters_triggered_effects`
6. **Emit `TokenCreated`** and `AbilityResolved`.

Key token creation code pattern (adapted from Myriad at resolution.rs:1196-1237):
```rust
StackObjectKind::EmbalmAbility { source_card_id } => {
    let controller = stack_obj.controller;
    let registry = state.card_registry.clone();

    // Look up the card definition for token characteristics.
    let def_opt = source_card_id.as_ref()
        .and_then(|cid| registry.get(cid.clone()));

    if let Some(def) = def_opt {
        // Build token characteristics from card definition (CR 707.2).
        let mut subtypes = im::OrdSet::new();
        for st in &def.types.sub_types {
            subtypes.insert(st.clone());
        }
        // CR 702.128a: "Zombie in addition to its other types"
        subtypes.insert(SubType("Zombie".to_string()));

        let mut card_types = im::OrdSet::new();
        for ct in &def.types.card_types {
            card_types.insert(*ct);
        }

        let mut keywords = im::OrdSet::new();
        for ability in &def.abilities {
            if let AbilityDefinition::Keyword(kw) = ability {
                keywords.insert(kw.clone());
            }
        }

        // CR 702.128a: "except it's white"
        let mut colors = im::OrdSet::new();
        colors.insert(Color::White);

        let characteristics = Characteristics {
            name: def.name.clone(),
            power: def.power,
            toughness: def.toughness,
            card_types,
            subtypes,
            keywords,
            colors,
            // CR 702.128a: "it has no mana cost" -- no mana abilities from the cost
            mana_abilities: im::Vector::new(),
            activated_abilities: Vec::new(), // Populated from def if needed
            ..Characteristics::default()
        };

        let token_obj = GameObject {
            id: ObjectId(0), // replaced by add_object
            card_id: source_card_id.clone(),
            characteristics,
            controller,
            owner: controller,
            zone: ZoneId::Battlefield,
            status: ObjectStatus::default(),
            is_token: true,
            has_summoning_sickness: true,
            // ... all other fields default
        };

        // Add token, run ETB pipeline, emit events.
    }

    events.push(GameEvent::AbilityResolved { controller, stack_object_id: stack_obj.id });
}
```

**IMPORTANT**: The token has `card_id: source_card_id.clone()` so that `enrich_spec_from_def`
and the layer system can find the CardDefinition for continuous effects, triggered abilities,
etc. But the token's *mana cost* must be None (mana value 0) and its *color* must be White only.
The `card_id` link is needed for the token's *abilities* and *rules text*, not its cost or color.

### Step 4: Command Handler Wiring

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add `Command::EmbalmCard` handler in `process_command()` (~line 327-339)
**Pattern**: Follow `Command::UnearthCard` handler at line 327-339

```rust
Command::EmbalmCard { player, card } => {
    validate_player_active(&state, player)?;
    // CR 104.4b: embalm is a meaningful player choice; reset loop detection.
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_embalm_card(&mut state, player, card)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    let trigger_events = abilities::flush_pending_triggers(&mut state);
    events.extend(trigger_events);
    all_events.extend(events);
}
```

### Step 5: Replay Harness Action

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"embalm_card"` action type in `translate_player_action()`
**Pattern**: Follow `"unearth_card"` at line 502-508

```rust
"embalm_card" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    Some(Command::EmbalmCard {
        player,
        card: card_id,
    })
}
```

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/embalm.rs`
**Pattern**: Follow `crates/engine/tests/unearth.rs` (12 tests, same structure)

**Card definition for tests**: Sacred Cat -- 1/1 White Cat creature, Lifelink, Embalm {W}

```rust
fn sacred_cat_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("sacred-cat".to_string()),
        name: "Sacred Cat".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            sub_types: [SubType("Cat".to_string())].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        oracle_text: "Lifelink\nEmbalm {W}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Keyword(KeywordAbility::Embalm),
            AbilityDefinition::Embalm {
                cost: ManaCost { white: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
```

**Tests to write**:

1. **`test_embalm_basic_create_token`** -- CR 702.128a
   - Activate embalm on Sacred Cat in graveyard.
   - Pay cost, card is exiled immediately.
   - Both players pass, ability resolves.
   - Assert: token on battlefield named "Sacred Cat", is_token=true.
   - Assert: original card is in exile (not graveyard, not battlefield).

2. **`test_embalm_token_is_white`** -- CR 702.128a: "except it's white"
   - Embalm a green creature (Honored Hydra would work, or use Sacred Cat).
   - Assert: token's colors are `{ White }` only, not the original card's color.

3. **`test_embalm_token_has_no_mana_cost`** -- CR 702.128a: "it has no mana cost"
   - Embalm, assert: token has no `mana_cost` (mana value 0).
   - Note: this is hard to assert directly since tokens don't have ManaCost;
     the key behavior is that the token's mana value is 0 for effects that check it.

4. **`test_embalm_token_is_zombie`** -- CR 702.128a: "Zombie in addition to its other types"
   - Embalm Sacred Cat.
   - Assert: token has subtypes `{ Cat, Zombie }`.
   - Assert: token has card_types `{ Creature }`.

5. **`test_embalm_token_keeps_abilities`** -- CR 707.2 (copiable values)
   - Embalm Sacred Cat.
   - Assert: token has `KeywordAbility::Lifelink` (printed ability).

6. **`test_embalm_sorcery_speed_restriction`** -- CR 702.128a: "Activate only as a sorcery"
   - Cannot activate during opponent's turn.
   - Cannot activate during combat.
   - Cannot activate when card is not in graveyard.

7. **`test_embalm_card_exiled_as_cost`** -- CR 702.128a, ruling 2017-07-14
   - Activate embalm.
   - Before resolving (ability still on stack), verify card is already in exile.
   - This is different from Unearth where the card stays in graveyard until resolution.

8. **`test_embalm_is_not_a_cast`** -- Ruling (embalm is activated ability, not spell)
   - Activate embalm.
   - Assert: no SpellCast event.
   - Assert: `spells_cast_this_turn` unchanged.

9. **`test_embalm_countered_no_token`** -- If the embalm ability is countered (Stifle)
   - Counter the ability on the stack.
   - Assert: card remains in exile (already exiled as cost).
   - Assert: no token created.

10. **`test_embalm_requires_mana_payment`** -- CR 602.2b
    - Attempt to embalm without sufficient mana.
    - Assert: error returned.

11. **`test_embalm_multiplayer_only_active_player`** -- CR 702.128a sorcery speed
    - Non-active player attempts embalm.
    - Assert: error returned.

12. **`test_embalm_token_has_summoning_sickness`** -- CR 302.6
    - Embalm, resolve.
    - Assert: token `has_summoning_sickness == true`.

### Step 7: Card Definition (later phase)

**Suggested card**: Sacred Cat
- Simple creature: {W}, 1/1, Cat, Lifelink, Embalm {W}
- Clean test case: single color, single keyword, simple embalm cost
- Token would be: 1/1 white Zombie Cat with Lifelink, no mana cost

**Alternative showcase card**: Honored Hydra
- {5}{G}, 6/6, Snake Hydra, Trample, Embalm {3}{G}
- Tests color override (green card -> white token)
- Tests multi-type (Snake Hydra -> Zombie Snake Hydra)

**Card lookup**: use `card-definition-author` agent

### Step 8: Game Script (later phase)

**Suggested scenario**: Embalm basic flow
- P1 has Sacred Cat in graveyard with {W} mana available.
- P1 activates embalm on Sacred Cat.
- Card immediately moves to exile.
- Both players pass priority.
- Token created: 1/1 white Zombie Cat with Lifelink.
- Assert token on battlefield, original in exile, token characteristics.

**Subsystem directory**: `test-data/generated-scripts/stack/`
(Embalm uses the stack for ability resolution, similar to Unearth scripts in stack/)

## Interactions to Watch

### Embalm vs Unearth (key differences)

| Aspect | Unearth | Embalm |
|--------|---------|--------|
| Card zone at resolution | Still in graveyard | Already in exile (cost) |
| Result | Card itself enters battlefield | Token copy enters battlefield |
| Haste | Yes | No |
| Exile at end step | Yes (delayed trigger) | No |
| Replacement to exile | Yes (was_unearthed flag) | No |
| Token? | No (card itself) | Yes |
| Color change | No | White |
| Type change | No | +Zombie |
| Mana cost | Unchanged | None |

### System interactions

1. **Rest in Peace**: If RiP is on the battlefield, the card goes to exile instead of graveyard
   when it dies. Once in exile, embalm cannot be activated (graveyard only). If RiP enters
   after the card is in the graveyard, the card stays in the graveyard (RiP only replaces
   future events).

2. **Panharmonicon**: If the embalm token has an ETB triggered ability, Panharmonicon doubles
   it (it enters the battlefield, and Panharmonicon fires on "whenever a... creature enters").

3. **Doubling Season / Anointed Procession**: "Create a token" effects are doubled by these.
   The engine's current `CreateToken` effect does not handle doublers; this is a pre-existing
   gap that applies to all token creation, not specific to Embalm.

4. **Layer system**: The embalm token's `card_id` links to the CardDefinition. The layer system
   uses this to apply continuous effects from the card's abilities. The token's color (White)
   and lack of mana cost are set at creation time (copiable values, CR 707.9b), so they are
   Layer 1 characteristics that other layers can modify (e.g., Painter's Servant could add
   another color).

5. **Commander zone**: Embalm only works from graveyard. A commander in the command zone
   cannot use embalm. If a commander with embalm dies and the owner chooses to let it go to
   the graveyard (not command zone), then embalm can be activated.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| `KeywordAbility` | `Embalm` | 89 |
| `StackObjectKind` | `EmbalmAbility` | 27 |
| `AbilityDefinition` | `Embalm` | 24 |
