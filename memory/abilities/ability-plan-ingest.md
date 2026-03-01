# Ability Plan: Ingest

**Generated**: 2026-02-28
**CR**: 702.115
**Priority**: P4
**Batch**: 1 (Low effort keyword cleanup), item 1.6
**Similar abilities studied**: Exploit (CR 702.110) -- keyword-driven triggered ability with dedicated `StackObjectKind` and `PendingTrigger` flag; Shadow (CR 702.28) -- simple keyword variant addition pattern (Batch 0); combat damage trigger infrastructure (existing `SelfDealsCombatDamageToPlayer` in `abilities.rs`)

## CR Rule Text

```
702.115. Ingest

702.115a Ingest is a triggered ability. "Ingest" means "Whenever this creature deals
combat damage to a player, that player exiles the top card of their library."

702.115b If a creature has multiple instances of ingest, each triggers separately.
```

## Key Edge Cases

From CR rules and card rulings:

1. **Exiled face up (Fathom Feeder ruling 2015-08-25)**: "The card exiled by the ingest
   ability is exiled face up." The engine's default exile behavior is face-up, so no special
   handling is needed.

2. **Empty library is a no-op (Fathom Feeder ruling 2015-08-25)**: "If the player has no
   cards in their library when the ingest ability resolves, nothing happens. That player
   won't lose the game (until they have to draw a card from an empty library)." The
   resolution must check if the library has cards before attempting to exile.

3. **Multiple instances trigger separately (CR 702.115b)**: Each instance of ingest on a
   creature generates a separate combat damage trigger. Each resolves independently,
   exiling one card each.

4. **Triggered ability uses the stack (CR 702.115a)**: Ingest is a triggered ability, not
   a static or replacement effect. The exile happens when the trigger resolves, not when
   combat damage is dealt. Opponents can respond to the trigger (e.g., counter it with
   Stifle).

5. **Creature must be on battlefield when trigger checks (CR 603.10)**: Combat damage
   triggers are NOT look-back triggers. The creature must still be on the battlefield
   after damage is dealt for the trigger to fire. The existing `collect_triggers_for_event`
   already checks `obj.zone == Battlefield`.

6. **Damage must be > 0 (CR 603.2g)**: If combat damage is fully prevented (amount == 0),
   the trigger does not fire. The existing `CombatDamageDealt` handler already skips
   assignments with `amount == 0`.

7. **Damaged player is the target**: The trigger exiles the top card of the DAMAGED
   player's library, not a random opponent. In multiplayer, a creature with ingest
   attacking player B exiles from player B's library, not player C's.

8. **Multiplayer**: In Commander (4+ players), multiple creatures with ingest can attack
   different players. Each trigger independently targets the player that was dealt damage.
   APNAP ordering applies when multiple triggers go on the stack simultaneously.

## Current State (from ability-wip.md)

ability-wip.md currently tracks Horsemanship, not Ingest. Ingest has no prior work.

- [ ] Step 1: Enum variant -- does NOT exist in `KeywordAbility`
- [ ] Step 2: Rule enforcement (trigger dispatch)
- [ ] Step 3: Trigger wiring (combat damage trigger + resolution)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Ingest` variant after `Overload` (line 599), before
the closing brace of the enum (line 600).
**Pattern**: Follow `KeywordAbility::Shadow` at line 586 (simple unit variant, no parameters)

Add this variant:

```rust
/// CR 702.115: Ingest -- triggered ability.
/// "Whenever this creature deals combat damage to a player, that player exiles
/// the top card of their library."
/// CR 702.115b: Multiple instances trigger separately.
Ingest,
```

### Step 1b: Hash discriminant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Ingest` arm to `HashInto for KeywordAbility` match block
**Pattern**: Follow `KeywordAbility::Overload` at line 458 (last entry, discriminant 70)
**Next discriminant**: 71

Add after line 458:

```rust
// Ingest (discriminant 71) -- CR 702.115
KeywordAbility::Ingest => 71u8.hash_into(hasher),
```

### Step 1c: View model format

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Ingest` arm to `format_keyword` match block
**Pattern**: Follow the last entry in the match (find via grep)

Add:

```rust
KeywordAbility::Ingest => "Ingest".to_string(),
```

### Step 1d: Match arm exhaustiveness

**Action**: Grep for all `match` expressions on `KeywordAbility` and add `Ingest` arm
where needed. The compiler will catch these as exhaustiveness errors.

```
Grep pattern="KeywordAbility::" path="crates/engine/src/" output_mode="files_with_matches"
```

Key files to check:
- `state/hash.rs` (covered in Step 1b)
- `state/builder.rs` -- keyword-to-trigger translations; check if there's a match
  that needs an arm (likely handled by `_ => {}` catch-all)
- `rules/layers.rs` -- `calculate_characteristics` keyword processing; Ingest has no
  layer interaction, handled by catch-all
- `effects/mod.rs` -- if there's a keyword-dispatch match, add as no-op
- Any other file with exhaustive match on KeywordAbility variants

### Step 2: PendingTrigger Flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add `is_ingest_trigger: bool` and `ingest_target_player: Option<PlayerId>`
fields to `PendingTrigger`
**Pattern**: Follow `is_partner_with_trigger: bool` / `partner_with_name: Option<String>`
at the end of the struct (around line 135-140)
**CR**: 702.115a -- the trigger needs to know which player was damaged so the resolution
can exile from that player's library

Add after the `partner_with_name` field:

```rust
/// CR 702.115a: If true, this pending trigger is an Ingest trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::IngestTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`.
/// The `ingest_target_player` carries the damaged player's ID so the
/// resolution knows whose library to exile from.
#[serde(default)]
pub is_ingest_trigger: bool,
/// CR 702.115a: The player dealt combat damage (whose library top card is exiled).
///
/// Only meaningful when `is_ingest_trigger` is true.
#[serde(default)]
pub ingest_target_player: Option<PlayerId>,
```

### Step 2b: PendingTrigger Hash

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `is_ingest_trigger` and `ingest_target_player` to the `PendingTrigger`
`HashInto` impl
**Pattern**: Follow `is_partner_with_trigger` / `partner_with_name` at lines 1075-1077
**Location**: After `self.partner_with_name.hash_into(hasher);` (line 1077)

Add:

```rust
// CR 702.115a: is_ingest_trigger -- ingest combat damage trigger marker
self.is_ingest_trigger.hash_into(hasher);
self.ingest_target_player.hash_into(hasher);
```

### Step 3: StackObjectKind Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add `StackObjectKind::IngestTrigger` variant
**Pattern**: Follow `StackObjectKind::PartnerWithTrigger` (line 382-399, last variant)
**CR**: 702.115a

Add after `PartnerWithTrigger`:

```rust
/// CR 702.115a: Ingest triggered ability on the stack.
///
/// "Whenever this creature deals combat damage to a player, that player
/// exiles the top card of their library."
///
/// `source_object` is the creature with ingest on the battlefield.
/// `target_player` is the player who was dealt combat damage (whose library
/// will be exiled from).
///
/// When this trigger resolves:
/// 1. Check if the target player has cards in their library.
/// 2. If yes, exile the top card face-up.
/// 3. If no, do nothing (ruling 2015-08-25).
///
/// CR 603.10: The source creature must be on the battlefield when the trigger
/// fires, but does NOT need to be on the battlefield at resolution time
/// (the trigger is already on the stack).
IngestTrigger {
    source_object: ObjectId,
    target_player: PlayerId,
},
```

### Step 3b: StackObjectKind Hash

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `StackObjectKind::IngestTrigger` arm to the `HashInto for StackObjectKind`
match block
**Pattern**: Follow `StackObjectKind::PartnerWithTrigger` at line 1347 (discriminant 17)
**Next discriminant**: 18

Add after line 1357:

```rust
// IngestTrigger (discriminant 18) -- CR 702.115a
StackObjectKind::IngestTrigger {
    source_object,
    target_player,
} => {
    18u8.hash_into(hasher);
    source_object.hash_into(hasher);
    target_player.hash_into(hasher);
}
```

### Step 3c: TUI stack_view.rs

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add `StackObjectKind::IngestTrigger` arm to the exhaustive match
**Pattern**: Follow `StackObjectKind::PartnerWithTrigger` at line 77 (last arm)
**CRITICAL**: This is an exhaustive match. Missing this arm will cause a compile error.

Add after line 79:

```rust
StackObjectKind::IngestTrigger { source_object, .. } => {
    ("Ingest: ".to_string(), Some(*source_object))
}
```

### Step 3d: Replay viewer view_model.rs

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `StackObjectKind::IngestTrigger` arm to `stack_kind_info` function
**Pattern**: Follow `StackObjectKind::PartnerWithTrigger` at line 470 (last arm)
**CRITICAL**: This is an exhaustive match. Missing this arm will cause a compile error.

Add after line 472:

```rust
StackObjectKind::IngestTrigger { source_object, .. } => {
    ("ingest_trigger", Some(*source_object))
}
```

### Step 4: Trigger Dispatch in check_triggers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add Ingest keyword detection in the `CombatDamageDealt` handler
**Location**: Inside the `GameEvent::CombatDamageDealt { assignments }` arm (line 1720),
after the existing `collect_triggers_for_event` call for `SelfDealsCombatDamageToPlayer`
(line 1731-1737), but still inside the `for assignment in assignments` loop (line 1726).
**CR**: 702.115a -- detect creatures with Ingest keyword, fire separate triggers
**Pattern**: Follow the Exploit trigger dispatch at line 861-905 (keyword detection,
counting instances from card definition, pushing PendingTrigger)

The Ingest check goes inside the existing `for assignment in assignments` loop, after
the `collect_triggers_for_event` call, but only for assignments targeting players:

```rust
// CR 702.115a: Ingest -- "Whenever this creature deals combat damage
// to a player, that player exiles the top card of their library."
// Check if the source creature has Ingest keyword.
// CR 702.115b: Multiple instances trigger separately.
if matches!(assignment.target, CombatDamageTarget::Player(_)) {
    if let Some(obj) = state.objects.get(&assignment.source) {
        if obj.zone == ZoneId::Battlefield
            && obj.characteristics.keywords.contains(&KeywordAbility::Ingest)
        {
            let damaged_player = match &assignment.target {
                CombatDamageTarget::Player(pid) => *pid,
                _ => unreachable!(),
            };

            // Count ingest instances from card definition for multiple instances.
            let ingest_count = obj
                .card_id
                .as_ref()
                .and_then(|cid| state.card_registry.get(cid.clone()))
                .map(|def| {
                    def.abilities
                        .iter()
                        .filter(|a| {
                            matches!(
                                a,
                                crate::cards::card_definition::AbilityDefinition::Keyword(
                                    KeywordAbility::Ingest
                                )
                            )
                        })
                        .count()
                })
                .unwrap_or(1)
                .max(1);

            let controller = obj.controller;
            for _ in 0..ingest_count {
                triggers.push(PendingTrigger {
                    source: assignment.source,
                    ability_index: 0, // unused for ingest triggers
                    controller,
                    triggering_event: Some(TriggerEvent::SelfDealsCombatDamageToPlayer),
                    entering_object_id: None,
                    targeting_stack_id: None,
                    triggering_player: None,
                    exalted_attacker_id: None,
                    defending_player_id: None,
                    is_evoke_sacrifice: false,
                    is_madness_trigger: false,
                    madness_exiled_card: None,
                    madness_cost: None,
                    is_miracle_trigger: false,
                    miracle_revealed_card: None,
                    miracle_cost: None,
                    is_unearth_trigger: false,
                    is_exploit_trigger: false,
                    is_modular_trigger: false,
                    modular_counter_count: None,
                    is_evolve_trigger: false,
                    evolve_entering_creature: None,
                    is_myriad_trigger: false,
                    is_suspend_counter_trigger: false,
                    is_suspend_cast_trigger: false,
                    suspend_card_id: None,
                    is_hideaway_trigger: false,
                    hideaway_count: None,
                    is_partner_with_trigger: false,
                    partner_with_name: None,
                    is_ingest_trigger: true,
                    ingest_target_player: Some(damaged_player),
                });
            }
        }
    }
}
```

**IMPORTANT**: This code block should be INSIDE the existing `for assignment in assignments`
loop (line 1726), AFTER the `collect_triggers_for_event` call (line 1731-1737), and INSIDE
the existing `if matches!(assignment.target, CombatDamageTarget::Player(_))` check
(line 1730). The Ingest trigger is separate from the card-definition-based
`SelfDealsCombatDamageToPlayer` triggers that `collect_triggers_for_event` handles.

**Note on dual-triggering**: If a card definition ALSO has a
`WhenDealsCombatDamageToPlayer` triggered ability AND the `Ingest` keyword, both will
fire. This is correct -- the keyword generates its own trigger independently from any
card-definition triggers.

### Step 5: Flush Handler for IngestTrigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: Add an `is_ingest_trigger` branch in `flush_pending_triggers` to create
`StackObjectKind::IngestTrigger`
**Location**: Inside `flush_pending_triggers` (line 1872+), in the `for trigger in sorted`
loop (line 1905), in the `let kind = if trigger.is_evoke_sacrifice { ... }` chain
(line 1953+).
**Pattern**: Follow the `is_partner_with_trigger` branch (the last branch in the chain)

Add a new branch in the `let kind = if ...` chain:

```rust
} else if trigger.is_ingest_trigger {
    StackObjectKind::IngestTrigger {
        source_object: trigger.source,
        target_player: trigger.ingest_target_player.unwrap_or(PlayerId(0)),
    }
}
```

**Note**: The `unwrap_or(PlayerId(0))` is a safety fallback; in practice
`ingest_target_player` is always `Some` when `is_ingest_trigger` is true.

### Step 6: Resolution Handler

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::IngestTrigger` resolution arm
**Location**: In the main `match stack_obj.kind` block, after the `PartnerWithTrigger`
arm (around line 1576+).
**CR**: 702.115a -- exile the top card of the target player's library
**Pattern**: Follow the Exploit resolution (line 988-1010) for structure;
follow `mill_cards` in `effects/mod.rs` (line 2627-2637) for the library-top-to-zone
move pattern, but replace `ZoneId::Graveyard` with `ZoneId::Exile`.

**Event variant**: Use `GameEvent::ObjectExiled { player, object_id, new_exile_id }`
(defined at `events.rs:387-394`). This is the existing event for exile zone moves.

Add:

```rust
// CR 702.115a: Ingest trigger resolves -- exile the top card of the
// damaged player's library.
//
// Ruling 2015-08-25: "If the player has no cards in their library when
// the ingest ability resolves, nothing happens."
//
// The exile is face-up (ruling 2015-08-25: "The card exiled by the
// ingest ability is exiled face up."). The engine's default exile
// behavior is face-up, so no special handling needed.
StackObjectKind::IngestTrigger {
    source_object: _,
    target_player,
} => {
    let controller = stack_obj.controller;
    let lib_id = ZoneId::Library(target_player);

    // Check if the target player has cards in their library.
    let top_card = state.zones.get(&lib_id).and_then(|z| z.top());

    if let Some(card_id) = top_card {
        // Exile the top card (CR 702.115a).
        if let Ok((new_exile_id, _old_obj)) =
            state.move_object_to_zone(card_id, ZoneId::Exile)
        {
            events.push(GameEvent::ObjectExiled {
                player: controller,
                object_id: card_id,
                new_exile_id,
            });
        }
    }
    // If library is empty, do nothing (ruling 2015-08-25).

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/keywords.rs`
**Pattern**: Follow the combat damage trigger tests in
`/home/airbaggie/scutemob/crates/engine/tests/combat.rs` (lines 1543-1630) for the
combat setup pattern. Follow the Exploit tests in `keywords.rs` for the keyword-specific
trigger structure.
**Insert location**: At the end of the file, after the last test section. Add a
section header comment `// ── Ingest (CR 702.115) ──`.

**Tests to write** (5 tests):

#### 7.1: `test_702_115_ingest_basic_exiles_top_card`
- **CR**: 702.115a -- basic trigger fires and resolves
- **Setup**: P1 has a 2/2 creature with `KeywordAbility::Ingest`, attacking P2.
  P2's library has at least 1 card.
- **Flow**: Declare attackers -> pass -> no blockers -> pass (enter CombatDamage step,
  damage dealt, Ingest trigger fires) -> pass to resolve trigger -> verify exile
- **Assert**: (1) P2's library has 1 fewer card. (2) The card is now in exile zone.
  (3) P2 took 2 combat damage (life = 38). (4) `AbilityTriggered` event was emitted
  for the ingest source. (5) The trigger went on the stack (stack had 1 item).
  (6) After resolution, `AbilityResolved` was emitted.

#### 7.2: `test_702_115_ingest_does_not_trigger_when_blocked`
- **CR**: 702.115a + CR 510.1c -- blocked creature deals no damage to player
- **Setup**: P1 has a 2/2 creature with Ingest attacking P2. P2 has a 3/3 blocker.
- **Flow**: Declare attackers -> declare blockers -> pass through combat damage
- **Assert**: No `AbilityTriggered` event for Ingest. Stack is empty after combat
  damage step. P2's library unchanged.

#### 7.3: `test_702_115_ingest_empty_library_is_noop`
- **CR**: 702.115a + ruling 2015-08-25 (empty library = nothing happens)
- **Setup**: P1 has a 1/1 creature with Ingest attacking P2. P2's library is empty
  (0 cards).
- **Flow**: Declare attackers -> no blockers -> pass through combat damage -> resolve
  trigger
- **Assert**: (1) No panic. (2) P2's library still has 0 cards. (3) The trigger
  resolved (`AbilityResolved` emitted). (4) P2 is still alive (not forced to draw).

#### 7.4: `test_702_115_ingest_multiple_instances_trigger_separately`
- **CR**: 702.115b -- multiple instances each trigger independently
- **Setup**: P1 has two separate 1/1 creatures with Ingest, both unblocked, both
  attacking P2. P2's library has at least 2 cards.
- **Flow**: Both attack P2 unblocked -> combat damage -> both triggers fire
- **Assert**: (1) Two `AbilityTriggered` events. (2) Stack has 2 items. (3) After
  resolving both, P2's library has 2 fewer cards. (4) 2 cards are in exile.

#### 7.5: `test_702_115_ingest_multiplayer_targets_correct_player`
- **CR**: 702.115a -- in multiplayer, each trigger targets the specific damaged player
- **Setup**: 4 players. P1 has two 1/1 creatures with Ingest. Creature A attacks P2,
  creature B attacks P3. Neither is blocked. P2, P3, P4 each have 1 card in library.
- **Flow**: Declare attackers (A -> P2, B -> P3) -> no blockers -> combat damage
- **Assert**: (1) Two triggers fire. (2) After resolving both: P2's library lost 1
  card, P3's library lost 1 card. (3) P4's library is unchanged (still has 1 card).

**Test structure** (each test follows this combat setup pattern from `combat.rs:1547`):

```rust
#[test]
/// CR 702.115a -- <description>
fn test_702_115_ingest_<scenario>() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Ingest Creature", 2, 2)
                .with_keyword(KeywordAbility::Ingest),
        )
        // Add library cards for P2 (via .object(ObjectSpec::card(p2, "Lib Card").in_zone(...)))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    // ... declare attackers, pass through blockers, pass to combat damage ...
    // ... verify trigger fires, resolve it, check exile zone ...
}
```

**Note on library setup**: `GameStateBuilder` does not currently seed libraries with
cards by default. Tests must manually add cards to P2's library zone using
`ObjectSpec::card(p2, "Library Card 1").in_zone(ZoneId::Library(p2))` or similar.
Check how existing tests add cards to libraries.

### Step 8: Card Definition (later phase)

**Suggested card**: Mist Intruder
- **Oracle text**: "Devoid (This card has no color.)\nFlying\nIngest (Whenever this
  creature deals combat damage to a player, that player exiles the top card of
  their library.)"
- **Why**: Simple 2-drop (1U) creature with Flying (already implemented) and Ingest.
  Also has Devoid (Batch 1 item 1.4), which can be represented as
  `AbilityDefinition::Keyword(KeywordAbility::Devoid)` even if Devoid enforcement
  is not yet implemented (it only affects color, not Ingest functionality).
- **Alternative**: Benthic Infiltrator ({2}{U} 1/4, Devoid + Ingest + "can't be blocked")
  -- CantBeBlocked is already implemented but Devoid is not. Still workable.
- **Dependencies**: Devoid keyword variant must exist (Batch 1 item 1.4, may not be
  implemented yet). If not, the card definition can include Devoid as a keyword but
  the color-stripping enforcement won't work. The card is still valid for testing Ingest.

**Card lookup**: use `card-definition-author` agent with "Mist Intruder"

### Step 9: Game Script (later phase)

**Suggested scenario**: "Ingest Combat Trigger"
- P1 controls a 1/2 creature with Flying + Ingest, attacking P2
- P2 has no flying/reach creatures (cannot block)
- Combat damage resolves, Ingest trigger fires and goes on stack
- Trigger resolves, top card of P2's library is exiled
- Verify: P2's library count decremented, exile zone has the card
- Edge case step: second combat with empty library -- trigger resolves as no-op

**Subsystem directory**: `test-data/generated-scripts/combat/`
**Suggested filename**: Next available number in combat scripts (likely `111_ingest_combat_trigger.json`)

## Interactions to Watch

- **Ingest + Stifle/Trickbind**: Since Ingest is a triggered ability on the stack, it
  can be countered by effects that counter triggered abilities. If countered, no card
  is exiled. The engine's existing counter-triggered-ability infrastructure handles this.

- **Ingest + Lifelink**: If a creature has both Ingest and Lifelink, both trigger/apply
  from the same combat damage event. Lifelink is processed as part of combat damage
  (not a trigger), so it resolves before the Ingest trigger. No conflict.

- **Ingest + Infect**: Infect replaces damage to players with poison counters. But
  Ingest triggers on "deals combat damage to a player" -- infect damage still counts
  as combat damage dealt to a player (it just doesn't reduce life). The Ingest trigger
  should still fire. **Verify**: Check if the `CombatDamageDealt` event still fires for
  infect damage. Currently, infect player damage emits `PoisonCountersGiven` but the
  `CombatDamageDealt` event includes the assignment with amount > 0, so the trigger
  check at line 1720-1740 should still find it. Confirm in testing.

- **Ingest + Trample**: If a creature with Ingest and Trample deals damage split between
  blockers and the defending player, the Ingest trigger fires for the player damage
  portion (assuming amount > 0). The trigger exiles one card regardless of how much
  damage was dealt to the player.

- **Ingest + Protection**: If the defending player has a prevention shield or the damage
  is fully prevented (amount == 0 after prevention), the trigger does NOT fire
  (CR 603.2g). The existing check at line 1727-1728 handles this.

- **Ingest + Double Strike**: A creature with double strike deals combat damage twice
  (first strike and regular). Each damage event could trigger Ingest separately. The
  engine handles first strike and regular combat damage as separate `CombatDamageDealt`
  events (separate calls to `execute_turn_based_actions`), so two Ingest triggers
  would fire. Verify this in testing if applicable.

- **Ingest + Panharmonicon**: Panharmonicon doubles ETB triggers, not combat damage
  triggers. The existing `doubler_applies_to_trigger` function should NOT match
  `SelfDealsCombatDamageToPlayer` events, so no double-triggering from Panharmonicon.
  Verify by checking the `TriggerDoublerFilter` enum.

- **Multiplayer APNAP**: When multiple Ingest triggers from different controllers
  fire simultaneously, they are placed on the stack in APNAP order (CR 101.4). The
  existing `flush_pending_triggers` handles APNAP ordering already.

## Estimated Effort

**Low-Medium** -- This is a straightforward keyword trigger implementation. It requires:
- New enum variant + hash (trivial)
- New PendingTrigger flag fields (2 fields, follows existing pattern)
- New StackObjectKind variant + hash + TUI + view_model arms (4 places, follows pattern)
- Trigger dispatch in check_triggers (follows Exploit pattern)
- Flush handler branch (1 line, follows pattern)
- Resolution handler (simple library-top-to-exile, follows mill pattern)
- 5 unit tests

The main complexity is threading the damaged player identity through the trigger pipeline,
which requires the PendingTrigger fields and StackObjectKind variant. The resolution itself
is trivial (take top card, move to exile).

**Files modified**: 7 source files + 1 test file
1. `crates/engine/src/state/types.rs` -- enum variant
2. `crates/engine/src/state/hash.rs` -- hash discriminants (3 additions: KeywordAbility, PendingTrigger, StackObjectKind)
3. `crates/engine/src/state/stubs.rs` -- PendingTrigger fields (2 new fields)
4. `crates/engine/src/state/stack.rs` -- StackObjectKind variant
5. `crates/engine/src/rules/abilities.rs` -- trigger dispatch + flush handler
6. `crates/engine/src/rules/resolution.rs` -- resolution handler
7. `tools/tui/src/play/panels/stack_view.rs` -- exhaustive match arm
8. `tools/replay-viewer/src/view_model.rs` -- exhaustive match arm (stack_kind_info) + keyword format arm
9. `crates/engine/tests/keywords.rs` -- 5 unit tests

**New lines of code**: ~50 (trigger dispatch) + ~30 (resolution) + ~15 (enum/hash/stack/PendingTrigger) + ~15 (TUI/view_model) + ~250 (5 tests) = ~360 total
**Risk**: Low. Well-established patterns, no new infrastructure needed.
