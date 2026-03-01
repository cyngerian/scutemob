# Ability Plan: Encore

**Generated**: 2026-03-01
**CR**: 702.141
**Priority**: P4
**Similar abilities studied**: Unearth (graveyard activation, end-step sacrifice/exile), Myriad (per-opponent token creation), Decayed (EOC sacrifice flag pattern)

## CR Rule Text

702.141. Encore

702.141a Encore is an activated ability that functions while the card with encore is in a graveyard. "Encore [cost]" means "[Cost], Exile this card from your graveyard: For each opponent, create a token that's a copy of this card that attacks that opponent this turn if able. The tokens gain haste. Sacrifice them at the beginning of the next end step. Activate only as a sorcery."

## Key Edge Cases

From rulings (Araumi of the Dead Tide, Briarblade Adept, Impulsivity, Rakshasa Debaser):

1. **Exiling the card is a cost** (CR 602.2): Once you announce activation, no player can respond to remove the card from your graveyard. The exile happens before the ability goes on the stack.
2. **Opponents who have left the game aren't counted** when determining how many tokens to create. Only active opponents (not eliminated/conceded) get a token.
3. **Each token must attack the appropriate player if able.** This is a mandatory attack requirement, not a "whenever attacks" trigger. Enforced at declare-attackers time.
4. **If a token can't attack** for any reason (tapped, Propaganda, etc.), it doesn't attack. If there's a cost associated with having it attack, the controller isn't forced to pay.
5. **If an effect stops a token from attacking a specific player**, that token can attack any player, planeswalker, or battle, or not attack at all.
6. **The tokens copy only what's on the original card.** Effects that modified the creature when it was previously on the battlefield won't be copied (use card registry for copiable values, not last-known battlefield state).
7. **If one of the tokens is under another player's control** when the delayed triggered ability resolves, you can't sacrifice that token. It remains on the battlefield indefinitely.
8. **"Activate only as a sorcery"**: main phase, stack empty, active player only (CR 307.1).
9. **Multiplayer (Commander)**: In a 4-player game, when P1 encores, 3 tokens are created (one for P2, P3, P4 -- each attacks that opponent). If P3 has been eliminated, only 2 tokens.
10. **Encore vs Unearth differences**: Unearth returns the card itself to battlefield with a replacement effect (exile if would leave). Encore exiles the card as cost, then creates TOKENS -- no replacement effect. The tokens are sacrificed at end step (not exiled). The card itself is already in exile.

## Current State (from ability-wip.md)

ability-wip.md currently tracks Retrace, not Encore. No Encore work exists.

- [ ] Step 1: Enum variant (`KeywordAbility::Encore`)
- [ ] Step 2: `AbilityDefinition::Encore { cost }` variant
- [ ] Step 3: `Command::EncoreCard` handler
- [ ] Step 4: `StackObjectKind::EncoreAbility` + `EncoreSacrificeTrigger`
- [ ] Step 5: Rule enforcement -- activation, resolution, token creation
- [ ] Step 6: End-step sacrifice trigger wiring
- [ ] Step 7: Unit tests
- [ ] Step 8: Card definition
- [ ] Step 9: Game script

## Implementation Steps

### Step 1: Enum Variant — `KeywordAbility::Encore`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Encore` variant after `CommanderNinjutsu` (line ~700 area, at the end of the enum).
**Pattern**: Follow `KeywordAbility::Unearth` at line 461-469.
**Doc comment**:
```
/// CR 702.141: Encore [cost] -- activated ability from graveyard.
/// "[Cost], Exile this card from your graveyard: For each opponent, create
/// a token that's a copy of this card that attacks that opponent this turn
/// if able. The tokens gain haste. Sacrifice them at the beginning of the
/// next end step. Activate only as a sorcery."
///
/// Marker for quick presence-checking (`keywords.contains`).
/// The encore cost itself is stored in `AbilityDefinition::Encore { cost }`.
Encore,
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
- KeywordAbility discriminant 89 (next after CommanderNinjutsu=88)
- `KeywordAbility::Encore => 89u8.hash_into(hasher),`

### Step 2: AbilityDefinition Variant — `AbilityDefinition::Encore`

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/card_definition.rs`
**Action**: Add `Encore { cost: ManaCost }` variant after `CommanderNinjutsu` (line ~278).
**Pattern**: Follow `AbilityDefinition::Unearth { cost }` at line 228-237.
**Doc comment**:
```
/// CR 702.141: Encore [cost]. The card's encore ability can be activated
/// from its owner's graveyard by paying this cost and exiling the card.
/// When the ability resolves, for each opponent, create a token copy of
/// this card that attacks that opponent this turn if able. Tokens gain
/// haste. Sacrifice them at the beginning of the next end step.
///
/// Cards with this ability should also include
/// `AbilityDefinition::Keyword(KeywordAbility::Encore)` for quick
/// presence-checking without scanning all abilities.
Encore { cost: ManaCost },
```

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
- AbilityDefinition discriminant 24 (next after CommanderNinjutsu=23)
- ```
  AbilityDefinition::Encore { cost } => {
      24u8.hash_into(hasher);
      cost.hash_into(hasher);
  }
  ```

### Step 3: GameObject Flag — `encore_sacrifice_at_end_step`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `pub encore_sacrifice_at_end_step: bool` field after `decayed_sacrifice_at_eoc` (line ~421).
**Pattern**: Follow `decayed_sacrifice_at_eoc` at line 412-421, and `was_unearthed` at line 392-404.
**Doc comment**:
```
/// CR 702.141a: If true, this token was created by an encore ability and
/// must be sacrificed at the beginning of the next end step.
///
/// Unlike Unearth (which exiles and uses a replacement effect), encore
/// tokens are simply sacrificed -- no replacement effect is involved.
///
/// Set when the EncoreAbility resolves and creates the token. Checked in
/// `end_step_actions()` in turn_actions.rs. Reset on zone changes (CR 400.7).
///
/// Ruling 2020-11-10: "If one of the tokens is under another player's
/// control as the delayed triggered ability resolves, you can't sacrifice
/// that token." -- sacrifice only if controller == encore activator.
#[serde(default)]
pub encore_sacrifice_at_end_step: bool,
```

**Initialize to `false` in ALL 6 sites**:
1. `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs` ~line 916 (after `decayed_sacrifice_at_eoc: false`)
2. `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` ~line 2477 (after `decayed_sacrifice_at_eoc: false`)
3. `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs` ~line 293 (first `move_object_to_zone` site)
4. `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs` ~line 384 (second `move_object_to_zone` site)
5. `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs` ~line 1234 (Myriad token creation)
6. Encore's own token creation site (Step 5 below)

**Hash**: Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` in the `GameObject` hasher (after `decayed_sacrifice_at_eoc` line ~673):
```
// Encore (CR 702.141a) -- token must be sacrificed at beginning of next end step
self.encore_sacrifice_at_end_step.hash_into(hasher);
```

### Step 4: Command Variant — `Command::EncoreCard`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add `EncoreCard { player: PlayerId, card: ObjectId }` variant.
**Pattern**: Follow `Command::UnearthCard` at line 384-397.
**Doc comment**:
```
// -- Encore (CR 702.141) -----------------------------------------------
/// Activate a card's encore ability from the graveyard (CR 702.141a).
///
/// The card must be in the player's graveyard with `KeywordAbility::Encore`.
/// The card is exiled as a cost, the encore cost is paid, and the encore
/// ability is placed on the stack. When it resolves, for each opponent,
/// a token copy of the exiled card is created, tapped, attacking that
/// opponent, with haste, and tagged for sacrifice at the next end step.
///
/// "Activate only as a sorcery" -- main phase, stack empty, active player.
///
/// Unlike `UnearthCard`, the card is exiled as part of the cost (before
/// the ability goes on the stack), not moved to the battlefield.
EncoreCard { player: PlayerId, card: ObjectId },
```

### Step 5: StackObjectKind Variants — `EncoreAbility` + `EncoreSacrificeTrigger`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add two new variants after `NinjutsuAbility`.

```rust
/// CR 702.141a: Encore activated ability on the stack.
///
/// When this ability resolves: for each active opponent, create a token
/// copy of the exiled card, tapped and attacking that opponent, with haste,
/// tagged `encore_sacrifice_at_end_step = true`.
///
/// `source_card_id` is the CardId of the exiled creature card (needed
/// to look up the card definition for copying, since the GameObject was
/// exiled as a cost and may have a new ObjectId in exile).
/// `activator` is the player who activated the encore ability.
EncoreAbility {
    source_card_id: Option<crate::cards::CardId>,
    activator: crate::state::player::PlayerId,
},
/// CR 702.141a: Encore delayed triggered ability on the stack.
///
/// "Sacrifice them at the beginning of the next end step."
/// This is a delayed triggered ability created when the encore tokens
/// are created. Each token gets its own sacrifice trigger.
///
/// When this trigger resolves:
/// 1. Check if the token is still on the battlefield (CR 400.7).
/// 2. Check if the token is still controlled by the encore activator
///    (ruling 2020-11-10: can't sacrifice if under another player's control).
/// 3. If both checks pass, sacrifice the token (move to graveyard/exile
///    via replacement effects).
///
/// If countered (e.g., by Stifle), the token stays on the battlefield.
EncoreSacrificeTrigger {
    source_object: ObjectId,
    activator: crate::state::player::PlayerId,
},
```

**Hash discriminants**:
- `EncoreAbility` = discriminant 27 (next after NinjutsuAbility=26)
- `EncoreSacrificeTrigger` = discriminant 28

Add to `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`:
```rust
// EncoreAbility (discriminant 27) -- CR 702.141a
StackObjectKind::EncoreAbility { source_card_id, activator } => {
    27u8.hash_into(hasher);
    source_card_id.hash_into(hasher);
    activator.hash_into(hasher);
}
// EncoreSacrificeTrigger (discriminant 28) -- CR 702.141a
StackObjectKind::EncoreSacrificeTrigger { source_object, activator } => {
    28u8.hash_into(hasher);
    source_object.hash_into(hasher);
    activator.hash_into(hasher);
}
```

**PendingTrigger fields** (in `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`):
Add after the Enlist fields:
```rust
/// CR 702.141a: If true, this pending trigger is an Encore sacrifice trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::EncoreSacrificeTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The
/// `encore_activator` field carries the player who activated encore (to verify
/// control at resolution). The `ability_index` field is unused when this is true.
#[serde(default)]
pub is_encore_sacrifice_trigger: bool,
/// CR 702.141a: The player who activated the encore ability.
///
/// Only meaningful when `is_encore_sacrifice_trigger` is true. Used at
/// resolution time to verify the token is still under this player's control
/// before sacrificing.
#[serde(default)]
pub encore_activator: Option<PlayerId>,
```

**Note**: Update ALL `PendingTrigger` construction sites to include
`is_encore_sacrifice_trigger: false, encore_activator: None`. These sites are:
- `turn_actions.rs` (all `PendingTrigger { ... }` constructions)
- `abilities.rs` (all `PendingTrigger { ... }` constructions)
- `replacement.rs` (if any)
- `resolution.rs` (if any)
- `effects/mod.rs` (if any)

### Step 6: Engine Handler — `Command::EncoreCard`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Add handler for `Command::EncoreCard` after the `UnearthCard` handler (~line 327).
**Pattern**: Follow `Command::UnearthCard` at line 327-340.

```rust
Command::EncoreCard { player, card } => {
    validate_player_active(&state, player)?;
    // CR 104.4b: encore is a meaningful player choice; reset loop detection.
    loop_detection::reset_loop_detection(&mut state);
    let mut events = abilities::handle_encore_card(&mut state, player, card)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    state.pending_triggers.extend(new_triggers);
    abilities::flush_pending_triggers(&mut state, &mut events);
    all_events.extend(events);
}
```

### Step 7: Activation Handler — `handle_encore_card`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add `pub fn handle_encore_card(...)` function after the Unearth section.
**Pattern**: Follow `handle_unearth_card` at line 626-771, with these differences:

Key differences from Unearth:
1. **Exiling the card is COST, not resolution**: The card is exiled from the graveyard before the ability goes on the stack (CR 602.2). Unearth keeps the card in the graveyard until resolution.
2. **EncoreAbility carries CardId, not ObjectId**: Since the card is exiled as cost and gets a new ObjectId, the stack object stores the `CardId` so resolution can look up the `CardDefinition` for copying.
3. **No replacement effect**: Unearth has "if it would leave the battlefield, exile instead." Encore has no such effect -- tokens are simply sacrificed.

Implementation:
```
pub fn handle_encore_card(state, player, card) -> Result<Vec<GameEvent>>:
    1. Validate player has priority (implicit from engine.rs validate_player_active)
    2. Zone check: card must be in player's graveyard (ZoneId::Graveyard(player))
    3. Keyword check: card must have KeywordAbility::Encore in characteristics.keywords
    4. Sorcery-speed check: active player, main phase, empty stack
    5. Look up encore cost from AbilityDefinition::Encore { cost } in card registry
    6. Capture card_id (Option<CardId>) from the object BEFORE exiling
    7. COST: Exile the card from graveyard (move_object_to_zone to ZoneId::Exile)
       - This is the cost, not the effect. Emit ObjectExiled event.
    8. COST: Pay mana cost (pay_cost with the encore ManaCost)
    9. Push EncoreAbility { source_card_id: card_id, activator: player } onto the stack
    10. Reset players_passed (priority reset after activation)
```

Also add helper `get_encore_cost(card_id, registry) -> Option<ManaCost>`:
**Pattern**: Follow `get_unearth_cost` at line 773-789.

### Step 8: Resolution — `StackObjectKind::EncoreAbility`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `EncoreAbility` and `EncoreSacrificeTrigger`.
**Pattern**: Combine Myriad token creation (line 1148-1281) + Unearth delayed trigger resolution (line 945-979).

#### EncoreAbility resolution:

```
StackObjectKind::EncoreAbility { source_card_id, activator }:
    1. Find all active opponents of the activator:
       state.players.values()
           .filter(|p| !p.has_lost && !p.has_conceded && p.id != activator)
       CR 702.141a: "For each opponent" + ruling "opponents who have left the game
       aren't counted."

    2. Look up the card definition from source_card_id via state.card_registry.
       If no definition found, skip token creation (ability fizzles).

    3. For each opponent_id:
       a. Build a token object:
          - characteristics: from card definition (NOT from any previous battlefield state)
            Ruling: "tokens copy only what's on the original card"
          - controller: activator
          - owner: activator
          - zone: Battlefield
          - status.tapped: true (attacking tokens enter tapped)
          - is_token: true
          - has_summoning_sickness: true (irrelevant -- already attacking with haste)
          - encore_sacrifice_at_end_step: true  <<< NEW FLAG
          - Add KeywordAbility::Haste to keywords (CR 702.141a: "tokens gain haste")
          - All other flags: false/default

       b. Add token to battlefield via state.add_object(token_obj, ZoneId::Battlefield)

       c. Apply Layer 1 CopyOf continuous effect:
          copy::create_copy_effect(state, token_id, source_card_id lookup, activator)
          NOTE: Encore copies from card definition, not a battlefield object.
          Use the same pattern as Myriad but source is card_id-based.
          IMPORTANT: Since the source card is in exile (not on battlefield), we need
          to build characteristics from the card definition directly, similar to how
          Unearth reads from registry. The copy effect may need to reference the
          exile-zone object or card definition directly.

       d. Register token in combat state as attacking the opponent:
          if let Some(combat) = state.combat.as_mut() {
              combat.attackers.insert(token_id, AttackTarget::Player(opponent_id));
          }
          NOTE: This requires that encore is activated during combat (which it CAN'T be
          -- encore is sorcery speed!). The tokens are created with "attacks that opponent
          this turn if able" -- this is NOT "enters attacking." The tokens must be
          declared as attackers during the next declare-attackers step.

          CORRECTION: Re-reading the CR: "create a token that's a copy of this card that
          attacks that opponent this turn if able." This means the tokens have a mandatory
          attack requirement, but they DON'T enter attacking (unlike Myriad/Ninjutsu).
          Since encore is sorcery-speed (pre-combat main), the tokens enter the battlefield
          untapped and must attack the designated opponent during the next combat phase.

          Implementation: Store the mandatory attack target on each token. This can be done
          with a new field or by using the goad-like infrastructure.

          Revised approach: Add `encore_must_attack: Option<PlayerId>` to GameObject.
          During declare-attackers validation, if a creature has `encore_must_attack = Some(pid)`,
          it must attack that player if able. This is simpler than the Myriad approach
          (which creates tokens already attacking).

    4. Emit TokenCreated + PermanentEnteredBattlefield events for each token.

    5. Emit AbilityResolved.
```

**IMPORTANT DESIGN DECISION**: Encore tokens do NOT enter tapped and attacking. They are
sorcery-speed, so they enter the battlefield before combat. They have a mandatory attack
requirement for the designated opponent. The tokens are NOT tapped on entry (they need
to attack later). They gain haste (so summoning sickness doesn't prevent attacking).

Revised token creation:
- `status.tapped: false` (they need to be untapped to attack)
- `has_summoning_sickness: true` (but has haste, so can attack)
- `encore_must_attack: Some(opponent_id)` (mandatory attack target)
- `encore_sacrifice_at_end_step: true`

### Step 8b: New GameObject Field — `encore_must_attack`

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `pub encore_must_attack: Option<PlayerId>` field.
**Doc comment**:
```
/// CR 702.141a: If set, this token was created by encore and must attack
/// the specified player this turn if able.
///
/// Enforced during declare-attackers validation in `combat.rs`. The token
/// must attack this player if able; if it can't attack that player (tapped,
/// Propaganda cost not paid, etc.), it can attack any player or not attack.
///
/// Cleared at end of turn (the obligation is "this turn" only). Also
/// cleared on zone changes (CR 400.7).
#[serde(default)]
pub encore_must_attack: Option<PlayerId>,
```

**Initialize to `None` in all sites** (same 5 sites as encore_sacrifice_at_end_step, plus builder.rs).

**Hash**: Add to hash.rs in GameObject hasher:
```
// Encore (CR 702.141a) -- mandatory attack target for this turn
self.encore_must_attack.hash_into(hasher);
```

### Step 9: End-Step Sacrifice Wiring

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs`
**Action**: Modify `end_step_actions()` to also queue encore sacrifice triggers.
**Pattern**: Follow Unearth's end-step exile trigger pattern at line 120-189.

Add after the Unearth trigger queueing:
```rust
// CR 702.141a: Queue sacrifice triggers for all encore tokens on the battlefield.
// "Sacrifice them at the beginning of the next end step."
let encore_tokens: Vec<(ObjectId, PlayerId, PlayerId)> = state
    .objects
    .values()
    .filter(|obj| obj.zone == ZoneId::Battlefield && obj.encore_sacrifice_at_end_step)
    .map(|obj| (obj.id, obj.controller, obj.owner))
    .collect();

for (obj_id, controller, _owner) in encore_tokens {
    state.pending_triggers.push_back(PendingTrigger {
        source: obj_id,
        ability_index: 0, // unused for encore sacrifice triggers
        controller,
        // ... all other fields false/None ...
        is_encore_sacrifice_trigger: true,
        encore_activator: Some(controller), // activator == current controller at creation
        // ... rest of fields ...
    });
}
```

### Step 10: EncoreSacrificeTrigger Resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add resolution arm for `EncoreSacrificeTrigger`.
**Pattern**: Follow `UnearthTrigger` resolution at line 945-979, but sacrifice (graveyard) instead of exile.

```rust
StackObjectKind::EncoreSacrificeTrigger { source_object, activator } => {
    let controller = stack_obj.controller;

    // Check if the token is still on the battlefield (CR 400.7).
    let token_info = state
        .objects
        .get(&source_object)
        .filter(|obj| obj.zone == ZoneId::Battlefield)
        .map(|obj| (obj.owner, obj.controller));

    if let Some((owner, current_controller)) = token_info {
        // Ruling 2020-11-10: "If one of the tokens is under another player's
        // control as the delayed triggered ability resolves, you can't sacrifice
        // that token."
        if current_controller == activator {
            // Sacrifice: move to graveyard (check replacement effects first).
            // Use the same pattern as Decayed sacrifice in end_combat().
            let pre_death_counters = state.objects.get(&source_object)
                .map(|o| o.counters.clone())
                .unwrap_or_default();

            let action = replacement::check_zone_change_replacement(
                state, source_object,
                ZoneType::Battlefield, ZoneType::Graveyard,
                owner, &HashSet::new(),
            );

            // Handle Proceed / Redirect / ChoiceRequired (same as Decayed)
            // ... move to graveyard or redirected zone, emit events ...
        }
        // else: token under another player's control -- do nothing (stays on battlefield)
    }
    // If not on battlefield, do nothing (already gone).

    events.push(GameEvent::AbilityResolved { controller, stack_object_id: stack_obj.id });
}
```

### Step 11: flush_pending_triggers Integration

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers()`, add an arm for `is_encore_sacrifice_trigger`.
**Pattern**: Follow the `is_unearth_trigger` arm at line ~3025-3030.

```rust
} else if trigger.is_encore_sacrifice_trigger {
    // CR 702.141a: Encore delayed sacrifice trigger -- "Sacrifice them
    // at the beginning of the next end step."
    StackObjectKind::EncoreSacrificeTrigger {
        source_object: trigger.source,
        activator: trigger.encore_activator.unwrap_or(trigger.controller),
    }
}
```

### Step 12: Replay Harness Action — `encore_card`

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"encore_card"` action type.
**Pattern**: Follow `"unearth_card"` at line 500-509.

```rust
"encore_card" => {
    let card_id = find_in_graveyard(state, player, card_name?)?;
    Some(Command::EncoreCard {
        player,
        card: card_id,
    })
}
```

### Step 13: TUI stack_view.rs

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add arms for `EncoreAbility` and `EncoreSacrificeTrigger` in the exhaustive match.
**Pattern**: Follow `NinjutsuAbility` at line 104-105.

```rust
StackObjectKind::EncoreAbility { .. } => {
    ("Encore: ".to_string(), None)  // source_card_id is CardId, not ObjectId
}
StackObjectKind::EncoreSacrificeTrigger { source_object, .. } => {
    ("Encore sacrifice: ".to_string(), Some(*source_object))
}
```

### Step 14: Replay Viewer view_model.rs

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add arms for `EncoreAbility` and `EncoreSacrificeTrigger`.
**Pattern**: Follow `NinjutsuAbility` at line 497-498.

```rust
StackObjectKind::EncoreAbility { .. } => {
    ("encore_ability", None)
}
StackObjectKind::EncoreSacrificeTrigger { source_object, .. } => {
    ("encore_sacrifice_trigger", Some(*source_object))
}
```

### Step 15: Resolution Skip-List

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::EncoreAbility { .. }` and `StackObjectKind::EncoreSacrificeTrigger { .. }` to the exhaustive match at line ~2295-2310 (the "non-spell resolution, skip card zone move" pattern).
**Pattern**: Follow `StackObjectKind::UnearthAbility { .. }` and `StackObjectKind::NinjutsuAbility { .. }` in that list.

### Step 16: Combat Enforcement — `encore_must_attack`

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Action**: During declare-attackers validation, check `encore_must_attack` on each creature the activator controls. If the creature has this field set and the designated opponent is a valid attack target, the creature must be declared as attacking that opponent.

This is a "soft" enforcement for V1: the engine accepts any valid attack declaration but could log a warning if an encore token does not attack its designated opponent. Full enforcement (rejecting invalid declarations) can be added later.

**V1 simplification**: Since the engine currently auto-resolves combat in scripts, and unit tests manually construct attackers, defer full combat enforcement to a later batch. The tokens will have haste and `encore_must_attack` set, but the player/script is responsible for declaring them as attackers.

**End-of-turn cleanup**: Clear `encore_must_attack` in the cleanup step (or end-of-turn) so the obligation doesn't persist. Since tokens are sacrificed at end step, this is academic -- but clear it in `end_step_actions` or `cleanup_step` for correctness.

### Step 17: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/encore.rs`
**Tests to write**:

1. **`test_encore_basic_activation_4p`** -- CR 702.141a basic case
   - 4-player game. P1 has a creature card with Encore in graveyard.
   - P1 activates encore (sorcery speed, main phase, empty stack).
   - Card is exiled as cost. EncoreAbility goes on stack.
   - All pass priority. EncoreAbility resolves.
   - 3 tokens created (one per opponent: P2, P3, P4).
   - Each token has haste, `encore_sacrifice_at_end_step = true`, `encore_must_attack` set.
   - Verify: 3 tokens on battlefield, source card in exile.

2. **`test_encore_token_sacrifice_at_end_step`** -- CR 702.141a delayed sacrifice
   - After encore resolves, advance to end step.
   - `end_step_actions` queues EncoreSacrificeTrigger for each token.
   - Each trigger resolves: tokens are sacrificed (move to graveyard, then cease to exist as SBA since they're tokens).
   - Verify: no encore tokens remain on battlefield after end step.

3. **`test_encore_tokens_gain_haste`** -- CR 702.141a "tokens gain haste"
   - Verify each created token has `KeywordAbility::Haste` in keywords.

4. **`test_encore_eliminated_opponent_no_token`** -- Ruling: "opponents who have left the game aren't counted"
   - 4-player game. P3 has been eliminated (has_lost = true).
   - P1 encores. Only 2 tokens created (for P2 and P4).

5. **`test_encore_sorcery_speed_restriction`** -- CR 702.141a "activate only as a sorcery"
   - Attempt to activate encore during opponent's turn: error.
   - Attempt to activate during combat: error.
   - Attempt to activate with non-empty stack: error.

6. **`test_encore_card_exiled_as_cost`** -- CR 602.2
   - Verify card moves from graveyard to exile BEFORE the ability is placed on stack.
   - After activation, card is in exile (not graveyard, not stack).

7. **`test_encore_tokens_copy_original_card`** -- Ruling: "tokens copy only what's on the original card"
   - Verify tokens have the characteristics from the card definition, not from any modified battlefield state.

8. **`test_encore_2_player_game`** -- Edge case: 1v1
   - P1 encores. Only 1 opponent (P2). 1 token created.

9. **`test_encore_no_keyword_fails`** -- Negative test
   - Attempt to encore a creature without the Encore keyword: error.

10. **`test_encore_not_in_graveyard_fails`** -- Negative test
    - Attempt to encore a card that's on the battlefield or in hand: error.

**Pattern**: Follow Unearth tests in `crates/engine/tests/unearth.rs` and Myriad tests in `crates/engine/tests/myriad.rs`.

### Step 18: Card Definition (later phase)

**Suggested card**: Briarblade Adept
- Name: Briarblade Adept
- Mana cost: {4}{B}
- Type: Creature -- Elf Assassin 3/4
- Oracle: "Whenever this creature attacks, target creature an opponent controls gets -1/-1 until end of turn. Encore {3}{B}"
- Abilities:
  ```rust
  vec![
      AbilityDefinition::Keyword(KeywordAbility::Encore),
      AbilityDefinition::Encore {
          cost: ManaCost { generic: 3, black: 1, ..Default::default() },
      },
      AbilityDefinition::Triggered(TriggeredAbilityDef {
          trigger_on: TriggerEvent::SelfAttacks,
          intervening_if: None,
          description: "Whenever this creature attacks, target creature an opponent controls gets -1/-1 until end of turn.".to_string(),
          effect: Some(Effect::ApplyContinuousEffect {
              target: EffectTarget::TargetCreature,
              effect: ContinuousEffectType::ModifyBoth(-1),
              duration: Duration::UntilEndOfTurn,
              layer: Layer::PtModify,
          }),
      }),
  ]
  ```

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/briarblade_adept.rs`

### Step 19: Game Script (later phase)

**Suggested scenario**: "Encore creates tokens per opponent in 4-player Commander"
**Subsystem directory**: `test-data/generated-scripts/stack/`
**Description**: P1 has Briarblade Adept in graveyard. P1 activates encore ({3}{B}, exile). 3 tokens created. Advance to combat, tokens attack each opponent. Advance to end step, tokens are sacrificed.

## Interactions to Watch

1. **Encore + Panharmonicon**: The encore tokens enter the battlefield, which fires ETB triggers. Panharmonicon doubles artifact/creature ETB triggers. Each encore token's ETB (e.g., Briarblade Adept's "when this creature attacks" is an attack trigger, not ETB -- but if the creature had an ETB, Panharmonicon would double it for each token).

2. **Encore + Rest in Peace**: When encore tokens are sacrificed at end step, Rest in Peace exiles them instead of putting them in the graveyard. Since they're tokens, they cease to exist in exile. The token owner's graveyard is unaffected.

3. **Encore + Doubling Season / Anointed Procession**: These double token creation. Each "create a token" instruction would create 2 tokens per opponent instead of 1. All tokens would still have the encore_sacrifice_at_end_step flag.

4. **Encore + Stifle on sacrifice trigger**: If the EncoreSacrificeTrigger is countered (Stifle, Disallow), the token remains on the battlefield permanently. It keeps haste.

5. **Encore + control-change effects (Threaten)**: Ruling 2020-11-10: If a token is under another player's control when the sacrifice trigger resolves, the encore activator can't sacrifice it. It stays forever.

6. **Encore + copies (Spark Double, Sakashima)**: If a player copies an encore token, the copy does NOT have `encore_sacrifice_at_end_step` (that's a flag on the object, not a copiable characteristic). The copy persists.

7. **Multiplayer timing**: Tokens for all opponents are created simultaneously as part of one resolution. They all enter the battlefield at the same time.

8. **Token death triggers**: When encore tokens are sacrificed at end step, "when a creature dies" triggers fire for each one. These are real creatures dying, not being exiled.

9. **Encore from command zone**: Encore specifically says "exile this card from your graveyard." It does NOT function from other zones (unlike Commander Ninjutsu). No commander-zone special case.

10. **Object identity (CR 400.7)**: The card exiled as cost gets a new ObjectId in the exile zone. The EncoreAbility on the stack stores `source_card_id: Option<CardId>` (the card definition ID), not the old ObjectId, so it can look up the definition even after the zone change.
