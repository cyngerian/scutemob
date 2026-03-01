# Ability Plan: Enlist

**Generated**: 2026-03-01
**CR**: 702.154 (batch plan incorrectly lists 702.155 -- gotcha #36)
**Priority**: P4
**Similar abilities studied**: Provoke (optional attacker-declared trigger, custom `StackObjectKind::ProvokeTrigger`, `forced_blocks` state in `combat.rs`), Bushido (triggered +N/+N until end of turn via `ApplyContinuousEffect` with `CEDuration::UntilEndOfTurn` in `builder.rs:711-745`), Flanking (custom `StackObjectKind::FlankingTrigger` with `ContinuousEffect` registration at resolution in `resolution.rs:1688-1726`), Training (dedicated `TriggerEvent::SelfAttacksWithGreaterPowerAlly` in the `AttackersDeclared` handler in `abilities.rs:1508-1550`), Myriad (`is_myriad_trigger` flag + post-processing after `collect_triggers_for_event` in `abilities.rs:1401-1419`)

## CR Rule Text

```
702.154. Enlist

702.154a Enlist represents a static ability and a triggered ability. Enlist means
"As this creature attacks, you may tap up to one untapped creature you control that
you didn't choose to attack with and that either has haste or has been under your
control continuously since this turn began. When you do, this creature gets +X/+0
until end of turn, where X is the tapped creature's power."

702.154b Enlist's static ability represents an optional cost to attack (see rule
508.1g). Its triggered ability is linked to that static ability (see rule 607.2h).

702.154c A creature "enlists" another creature when you pay the cost of the
creature's enlist ability by tapping the other creature. Note that it isn't possible
for a creature to enlist itself.

702.154d Multiple instances of enlist on a single creature function independently.
The triggered ability represented by each instance of enlist triggers only once and
only for the cost associated with that enlist ability.
```

### Related Rules

```
508.1g If there are any optional costs to attack with the chosen creatures
(expressed as costs a player may pay "as" a creature attacks), the active player
chooses which, if any, they will pay.

508.1h If any of the chosen creatures require paying costs to attack, or if any
optional costs to attack were chosen, the active player determines the total cost
to attack.

508.1j Once the player has enough mana in their mana pool, they pay all costs in
any order. Partial payments are not allowed.

607.2h If an object has both a static ability and one or more triggered abilities
printed on it in the same paragraph, each of those triggered abilities is linked to
the static ability. Each triggered ability refers only to actions taken as a result
of the static ability. See rule 603.11.
```

## Key Edge Cases

1. **The tapped creature must not be attacking** (CR 702.154a). "you didn't choose to
   attack with" is the condition. Even if a creature has vigilance and was declared as an
   attacker (thus remaining untapped), it cannot be enlisted because it was still "chosen
   to attack with." The creature must be entirely outside the attacker list.

2. **Summoning sickness matters for the enlisted creature** (CR 702.154a). The creature
   being tapped must "either have haste or have been under your control continuously since
   this turn began." This is the same summoning sickness check used for attackers (CR 302.6).
   The engine tracks this via `has_summoning_sickness` on `GameObject`. A creature that
   entered this turn without haste cannot be enlisted.

3. **Cannot enlist itself** (CR 702.154c). The attacker with Enlist is attacking (and
   tapped), so it naturally fails the "not attacking" condition. The CR explicitly calls
   this out as a separate rule. Validation rejects `attacker_id == enlisted_id`.

4. **One creature per enlist instance** (ruling 2022-09-09). "You may tap only one creature
   for an enlist ability of an attacking creature, and a single creature can't be tapped
   for more than one enlist ability." This means:
   - Each Enlist keyword instance on an attacker allows tapping at most one creature.
   - A given non-attacking creature can be enlisted by at most one Enlist ability total
     across all attackers.

5. **Multiple instances of Enlist on one creature** (CR 702.154d). Each instance functions
   independently. A creature with two Enlist keywords can tap two different creatures, and
   each generates its own trigger. But the same creature cannot be used for both instances.

6. **The +X/+0 is a triggered ability that uses the stack** (ruling 2022-09-09). "This is
   a triggered ability that goes on the stack immediately after attackers have been declared
   in the declare attackers step." Opponents can respond to it. If the enlisting creature
   is removed before the trigger resolves, the trigger does nothing (CR 400.7).

7. **Power is read at resolution time** (standard MTG trigger evaluation). The trigger
   says "where X is the tapped creature's power." Since this is a triggered ability on
   the stack, X is evaluated when the trigger resolves, not when the creature is tapped.
   If the enlisted creature's power changes between tap and resolution (e.g., via an
   instant-speed spell), the new power is used. If the enlisted creature has left the
   battlefield, use `calculate_characteristics` with last-known information (which falls
   back to raw object characteristics or 0 if the object no longer exists).

8. **The choice happens during declare-attackers step** (ruling 2022-09-09). "The attacking
   player chooses whether to tap a creature for an enlist ability immediately after they
   tap the creatures that they have chosen to attack with. You can't choose to enlist a
   creature later." This means the enlist choices must be part of the `DeclareAttackers`
   command, not a separate subsequent command.

9. **Enlisted creature's power can be 0 or negative** (edge case). If the enlisted
   creature has 0 or negative power at resolution time, the bonus is effectively 0 or
   negative. A +0/+0 or negative bonus is still a valid resolution (it just does nothing
   beneficial). The trigger still resolves; it just registers no continuous effect (or a
   trivial one).

10. **Multiplayer**: No special multiplayer considerations beyond standard Commander attack
    rules. The enlisted creature just needs to be a non-attacking creature you control.
    Multiple Enlist creatures attacking different opponents can each enlist a different
    creature.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Command extension + validation in combat.rs
- [ ] Step 3: Trigger wiring (builder.rs + abilities.rs + resolution.rs)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant + Supporting Types

#### 1a. KeywordAbility::Enlist

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Enlist` variant after `Training` (currently at line ~692).
**Doc comment**:
```rust
/// CR 702.154: Enlist -- "As this creature attacks, you may tap up to one
/// untapped creature you control that you didn't choose to attack with and
/// that either has haste or has been under your control continuously since
/// this turn began. When you do, this creature gets +X/+0 until end of turn,
/// where X is the tapped creature's power."
///
/// Static ability: optional cost to attack (CR 508.1g). Expressed as an
/// enlist_choices field on the DeclareAttackers command.
/// Triggered ability: linked to the static ability (CR 607.2h). Goes on
/// the stack after attackers are declared. Resolves to +X/+0.
/// Multiple instances function independently (CR 702.154d).
Enlist,
```

#### 1b. Hash for KeywordAbility::Enlist

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for KeywordAbility` impl, after `Training => 82u8` (line 494).
Next available discriminant is **83**.
```rust
// Enlist (discriminant 83) -- CR 702.154
KeywordAbility::Enlist => 83u8.hash_into(hasher),
```

#### 1c. StackObjectKind::EnlistTrigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stack.rs`
**Action**: Add new variant after `RenownTrigger` (currently ends at line ~513).
```rust
/// CR 702.154a: Enlist triggered ability on the stack.
///
/// "When you [tap a creature for enlist], this creature gets +X/+0 until
/// end of turn, where X is the tapped creature's power."
///
/// `source_object` is the attacking creature with Enlist.
/// `enlisted_creature` is the creature that was tapped to pay the
/// enlist cost.
///
/// When this trigger resolves:
/// 1. Check if the source (enlisting) creature is still on the battlefield.
/// 2. Read the enlisted creature's power via calculate_characteristics
///    (if still on battlefield) or raw characteristics (if departed).
/// 3. If the source creature is alive and power > 0, register a
///    ContinuousEffect with ModifyPower(X) in Layer 7c (PtModify)
///    targeting SingleObject(source_object) with UntilEndOfTurn duration.
/// 4. If the source left the battlefield, do nothing (CR 400.7).
///
/// CR 702.154d: Multiple instances each create their own EnlistTrigger
/// with different `enlisted_creature` values.
EnlistTrigger {
    source_object: ObjectId,
    enlisted_creature: ObjectId,
},
```

#### 1d. Hash for StackObjectKind::EnlistTrigger

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for StackObjectKind` impl, after `RenownTrigger` (discriminant 22, line ~1458).
Next available discriminant is **23**.
```rust
// EnlistTrigger (discriminant 23) -- CR 702.154a
StackObjectKind::EnlistTrigger {
    source_object,
    enlisted_creature,
} => {
    23u8.hash_into(hasher);
    source_object.hash_into(hasher);
    enlisted_creature.hash_into(hasher);
}
```

#### 1e. TUI stack_view.rs

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arm after `RenownTrigger` (line ~93):
```rust
StackObjectKind::EnlistTrigger { source_object, .. } => {
    ("Enlist: ".to_string(), Some(*source_object))
}
```

#### 1f. Replay Viewer view_model.rs -- format_keyword

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Location**: In `format_keyword` function, after `Training` arm (line ~706):
```rust
KeywordAbility::Enlist => "Enlist".to_string(),
```

#### 1g. Replay Viewer view_model.rs -- format_stack_kind

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Find the match on `StackObjectKind` for stack kind formatting (around line ~485-487) and add:
```rust
StackObjectKind::EnlistTrigger { source_object, .. } => {
    ("enlist_trigger", Some(*source_object))
}
```

#### 1h. Match Arm Audit

Grep for exhaustive `match` on `StackObjectKind` across the codebase. Known locations:
- `tools/tui/src/play/panels/stack_view.rs` (1e above)
- `tools/replay-viewer/src/view_model.rs` (1g above)
- `crates/engine/src/rules/resolution.rs` -- resolve arm (Step 3e) + counter arm (Step 3f)
- `crates/engine/src/state/hash.rs` (1d above)

The compiler will catch any missed exhaustive matches. Run `cargo build --all` after Step 1.

---

### Step 2: Command Extension + Validation (combat.rs)

#### 2a. Extend DeclareAttackers command

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/command.rs`
**Action**: Add an `enlist_choices` field to the `DeclareAttackers` variant (after
`attackers`, around line 193):

```rust
DeclareAttackers {
    player: PlayerId,
    /// (attacker ObjectId, attack target) pairs.
    attackers: Vec<(ObjectId, AttackTarget)>,
    /// CR 702.154a / CR 508.1g: Optional enlist cost payments.
    ///
    /// Each entry is (enlisting_attacker_id, enlisted_creature_id).
    /// The enlisted creature will be tapped as a cost during the
    /// declare-attackers step. The attacker must have Enlist; the
    /// enlisted creature must be untapped, non-attacking, controlled
    /// by the player, a creature, and not have summoning sickness
    /// (or have haste).
    ///
    /// Empty vec for no enlist choices. At most one entry per Enlist
    /// keyword instance on a given attacker. A creature can only be
    /// enlisted once across all attackers (ruling 2022-09-09).
    ///
    /// Validated in handle_declare_attackers.
    #[serde(default)]
    enlist_choices: Vec<(ObjectId, ObjectId)>,
},
```

**NOTE**: The `#[serde(default)]` ensures backward compatibility with all existing
scripts and commands that do not include the field. Existing scripts will deserialize
with an empty vec.

#### 2b. Update engine.rs command dispatch

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/engine.rs`
**Action**: Update the `DeclareAttackers` destructuring (line 156) to include
the new field and pass it to combat::handle_declare_attackers:
```rust
Command::DeclareAttackers {
    player,
    attackers,
    enlist_choices,
} => {
    validate_player_active(&state, player)?;
    loop_detection::reset_loop_detection(&mut state);
    let events =
        combat::handle_declare_attackers(&mut state, player, attackers, enlist_choices)?;
    all_events.extend(events);
}
```

#### 2c. Update handle_declare_attackers signature and logic

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Action**: Extend `handle_declare_attackers` to accept and validate enlist choices.

**Signature change** (line ~34):
```rust
pub fn handle_declare_attackers(
    state: &mut GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
    enlist_choices: Vec<(ObjectId, ObjectId)>,
) -> Result<Vec<GameEvent>, GameStateError> {
```

**Validation logic** -- insert after the goaded creature validation block (after line ~260,
before tapping attackers at line ~264):

```rust
// ---- CR 702.154a / CR 508.1g: Validate enlist choices ----
//
// Each (enlisting_attacker_id, enlisted_creature_id) must satisfy:
//  1. The attacker is in the declared_attacker_ids set.
//  2. The attacker has the Enlist keyword (layer-aware check).
//  3. The enlisted creature is on the battlefield, controlled by the player.
//  4. The enlisted creature is NOT in the declared_attacker_ids set.
//  5. The enlisted creature is untapped.
//  6. The enlisted creature is a creature (layer-aware check).
//  7. The enlisted creature does not have summoning sickness (or has haste).
//  8. Each enlisted creature appears at most once across ALL enlist choices
//     (ruling 2022-09-09: "a single creature can't be tapped for more than
//     one enlist ability").
//  9. For a given attacker, the number of enlist choices must not exceed
//     the number of Enlist keyword instances on that attacker (CR 702.154d).
// 10. The enlisted creature is not the same as the attacker (CR 702.154c).
{
    let mut enlisted_ids_used: Vec<ObjectId> = Vec::new();
    let mut enlist_used_per_attacker: im::OrdMap<ObjectId, u32> = im::OrdMap::new();

    for (attacker_id, enlisted_id) in &enlist_choices {
        // Check 10: cannot enlist itself (CR 702.154c).
        if attacker_id == enlisted_id {
            return Err(GameStateError::InvalidCommand(format!(
                "Enlist: creature {:?} cannot enlist itself (CR 702.154c)",
                attacker_id
            )));
        }

        // Check 1: attacker is declared.
        if !declared_attacker_ids.contains(attacker_id) {
            return Err(GameStateError::InvalidCommand(format!(
                "Enlist: creature {:?} is not a declared attacker",
                attacker_id
            )));
        }

        // Check 2: attacker has Enlist keyword + check 9: instance count.
        let attacker_chars = calculate_characteristics(state, *attacker_id)
            .ok_or(GameStateError::ObjectNotFound(*attacker_id))?;
        let enlist_count = attacker_chars
            .keywords
            .iter()
            .filter(|kw| matches!(kw, KeywordAbility::Enlist))
            .count() as u32;
        if enlist_count == 0 {
            return Err(GameStateError::InvalidCommand(format!(
                "Enlist: attacker {:?} does not have the Enlist keyword",
                attacker_id
            )));
        }
        let used = enlist_used_per_attacker
            .entry(*attacker_id)
            .or_insert(0);
        *used += 1;
        if *used > enlist_count {
            return Err(GameStateError::InvalidCommand(format!(
                "Enlist: attacker {:?} has {} Enlist instance(s) but {} choices were made",
                attacker_id, enlist_count, *used
            )));
        }

        // Check 4: enlisted creature is not attacking.
        if declared_attacker_ids.contains(enlisted_id) {
            return Err(GameStateError::InvalidCommand(format!(
                "Enlist: creature {:?} is an attacker and cannot be enlisted",
                enlisted_id
            )));
        }

        // Check 3: on battlefield, controlled by player.
        let enlisted_obj = state.object(*enlisted_id)?;
        if enlisted_obj.zone != ZoneId::Battlefield {
            return Err(GameStateError::ObjectNotOnBattlefield(*enlisted_id));
        }
        if enlisted_obj.controller != player {
            return Err(GameStateError::NotController {
                player,
                object_id: *enlisted_id,
            });
        }

        // Check 5: untapped.
        if enlisted_obj.status.tapped {
            return Err(GameStateError::PermanentAlreadyTapped(*enlisted_id));
        }

        // Check 6: is a creature.
        let enlisted_chars = calculate_characteristics(state, *enlisted_id)
            .ok_or(GameStateError::ObjectNotFound(*enlisted_id))?;
        if !enlisted_chars.card_types.contains(&CardType::Creature) {
            return Err(GameStateError::InvalidCommand(format!(
                "Enlist: object {:?} is not a creature",
                enlisted_id
            )));
        }

        // Check 7: no summoning sickness (or has haste).
        let has_haste = enlisted_chars.keywords.contains(&KeywordAbility::Haste);
        if enlisted_obj.has_summoning_sickness && !has_haste {
            return Err(GameStateError::InvalidCommand(format!(
                "Enlist: creature {:?} has summoning sickness and no haste (CR 702.154a)",
                enlisted_id
            )));
        }

        // Check 8: not already enlisted by another attacker.
        if enlisted_ids_used.contains(enlisted_id) {
            return Err(GameStateError::InvalidCommand(format!(
                "Enlist: creature {:?} is already enlisted by another attacker \
                 (ruling 2022-09-09)",
                enlisted_id
            )));
        }
        enlisted_ids_used.push(*enlisted_id);
    }
}
```

**Tapping enlisted creatures** -- insert after tapping attackers (after the loop at line ~276,
before recording attackers in combat state at line ~279):

```rust
// CR 702.154a / CR 508.1j: Tap enlisted creatures as part of the
// attack cost payment.
for (_, enlisted_id) in &enlist_choices {
    if let Some(obj) = state.objects.get_mut(enlisted_id) {
        obj.status.tapped = true;
    }
    events.push(GameEvent::PermanentTapped {
        player,
        object_id: *enlisted_id,
    });
}
```

**Store enlist pairings** -- after recording attackers in combat state (after line ~283):

```rust
// CR 702.154a: Store enlist pairings for trigger collection in abilities.rs.
if let Some(combat) = state.combat.as_mut() {
    combat.enlist_pairings = enlist_choices.clone();
}
```

#### 2d. Add enlist_pairings to CombatState

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/combat.rs`
**Action**: Add a field to `CombatState` struct (after `forced_blocks`, line ~64):
```rust
/// CR 702.154a: Enlist pairings made during declare-attackers.
///
/// Each entry is (enlisting_attacker_id, enlisted_creature_id).
/// Used by abilities.rs to fire EnlistTrigger for each pairing.
/// Cleared naturally when CombatState is dropped at end of combat.
pub enlist_pairings: Vec<(ObjectId, ObjectId)>,
```

**Initialize**: In `CombatState::new()` (line ~70), add to the struct literal:
```rust
enlist_pairings: Vec::new(),
```

**Hash**: In `HashInto for CombatState` in `hash.rs` (line ~1533), add after `forced_blocks`:
```rust
// CR 702.154a: enlist_pairings
self.enlist_pairings.hash_into(hasher);
```

#### 2e. Update replay harness for declare_attackers

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: Update the `"declare_attackers"` arm in `translate_player_action()` to
pass `enlist_choices`.

The existing `"declare_attackers"` arm constructs `Command::DeclareAttackers { player, attackers }`. After the new field is added, this becomes `Command::DeclareAttackers { player, attackers, enlist_choices }`.

For basic scripts without enlist, pass `enlist_choices: Vec::new()`.

For enlist scripts, add an optional `enlist` array to the action JSON:
```json
{
    "action": "declare_attackers",
    "attackers": [
        { "card": "Coalition Skyknight", "target_player": "P2" }
    ],
    "enlist": [
        { "attacker": "Coalition Skyknight", "enlisted": "Llanowar Elves" }
    ]
}
```

This requires:
1. Adding an `EnlistDecl` struct (or using a tuple) to parse the JSON `enlist` array.
2. Resolving card names to ObjectIds using `find_on_battlefield`.
3. Constructing the `enlist_choices: Vec<(ObjectId, ObjectId)>`.

**Detailed approach**: In the `PlayerAction` deserialization struct (or wherever attackers
are parsed), add an optional `enlist` field:
```rust
#[serde(default)]
enlist: Vec<EnlistDecl>,
```
Where `EnlistDecl` is:
```rust
#[derive(Debug, Clone, Deserialize)]
struct EnlistDecl {
    attacker: String,
    enlisted: String,
}
```

In the `"declare_attackers"` handler, after building `atk_pairs`, resolve enlist:
```rust
let mut enlist_choices: Vec<(ObjectId, ObjectId)> = Vec::new();
for edecl in &action.enlist {
    let attacker_id = find_on_battlefield(state, player, &edecl.attacker)?;
    let enlisted_id = find_on_battlefield(state, player, &edecl.enlisted)?;
    enlist_choices.push((attacker_id, enlisted_id));
}
```

Then construct: `Command::DeclareAttackers { player, attackers: atk_pairs, enlist_choices }`

**Pattern**: Follow how `crew_vehicle` parses `crew_creatures` from script JSON --
`CrewVehicle` has a dedicated action type. For Enlist, we're extending an existing action
type with an optional field, which is slightly different but follows the same
name-resolution pattern.

---

### Step 3: Trigger Wiring (builder.rs + abilities.rs + resolution.rs)

#### 3a. Builder auto-generation (builder.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Location**: After the Training block (line ~511).

Enlist's trigger is linked to a specific enlisted creature (CR 607.2h). The trigger's
effect depends on which creature was tapped. Standard `TriggeredAbilityDef` with
`TriggerEvent::SelfAttacks` would fire for every attack without knowing WHICH creature
was enlisted.

**Strategy**: Generate a placeholder `TriggeredAbilityDef` with `TriggerEvent::SelfAttacks`
and `effect: None`. In the `AttackersDeclared` handler in abilities.rs, these triggers
are collected by `collect_triggers_for_event`, then post-processed:
- If an enlist pairing exists for this attacker, tag the trigger with
  `is_enlist_trigger = true` and `enlist_enlisted_creature = Some(enlisted_id)`.
- If NO enlist pairing exists, REMOVE the trigger (it should not fire).

This mirrors the Myriad pattern (collect via SelfAttacks, then tag), with the difference
that unmatched Enlist triggers are removed rather than kept.

```rust
// CR 702.154a: Enlist -- "As this creature attacks, you may tap up to
// one untapped creature [...]. When you do, this creature gets +X/+0
// until end of turn, where X is the tapped creature's power."
// The static ability (optional cost) is handled in combat.rs via the
// enlist_choices field on DeclareAttackers. The triggered ability is
// handled by creating an EnlistTrigger StackObjectKind at trigger-
// collection time in abilities.rs.
// builder.rs generates a placeholder TriggeredAbilityDef so
// ability_index is valid. The effect is None because resolution is
// custom (EnlistTrigger reads the enlisted creature's power).
// CR 702.154d: Each Enlist instance generates one placeholder.
if matches!(kw, KeywordAbility::Enlist) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: "Enlist (CR 702.154a): As this creature attacks, you may \
                      tap an untapped non-attacking creature you control. When \
                      you do, this creature gets +X/+0 until end of turn, where \
                      X is the tapped creature's power.".to_string(),
        effect: None, // Custom resolution via EnlistTrigger
    });
}
```

#### 3b. PendingTrigger fields

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add two new fields after `renown_n` (line ~299, before the closing `}`
of `PendingTrigger`):
```rust
/// CR 702.154a: If true, this pending trigger is an Enlist trigger.
///
/// When flushed to the stack, creates a `StackObjectKind::EnlistTrigger`
/// instead of the normal `StackObjectKind::TriggeredAbility`. The
/// `enlist_enlisted_creature` carries the ObjectId of the creature that
/// was tapped as the enlist cost.
#[serde(default)]
pub is_enlist_trigger: bool,
/// CR 702.154a: The ObjectId of the creature tapped for the enlist cost.
///
/// Only meaningful when `is_enlist_trigger` is true. Used at resolution
/// time to read the enlisted creature's power for the +X/+0 bonus.
#[serde(default)]
pub enlist_enlisted_creature: Option<ObjectId>,
```

**Hash**: Add both fields to `HashInto for PendingTrigger` in `hash.rs`.

#### 3c. Trigger collection in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Location**: In the `GameEvent::AttackersDeclared` handler, inside the per-attacker
loop, after the Provoke trigger post-processing block (line ~1468).

The existing flow:
1. `collect_triggers_for_event(state, ..., SelfAttacks, Some(*attacker_id), ...)` collects
   ALL SelfAttacks triggers for this attacker (line ~1383).
2. Myriad triggers are tagged (lines ~1409-1419).
3. Provoke triggers are tagged (lines ~1421-1468).

Add Enlist trigger post-processing after Provoke:

```rust
// CR 702.154a: Enlist trigger post-processing.
// Each enlist pairing from combat.enlist_pairings for this attacker
// should match one "Enlist"-prefixed placeholder TriggeredAbilityDef.
// - If a pairing exists, tag the trigger with is_enlist_trigger=true
//   and the enlisted creature's ObjectId.
// - If no pairing exists for a given Enlist placeholder trigger,
//   REMOVE it (the player chose not to use that Enlist instance).
{
    let enlist_pairings_for_attacker: Vec<ObjectId> = state
        .combat
        .as_ref()
        .map(|c| {
            c.enlist_pairings
                .iter()
                .filter(|(aid, _)| aid == attacker_id)
                .map(|(_, eid)| *eid)
                .collect()
        })
        .unwrap_or_default();

    // Collect indices of Enlist placeholder triggers from this batch.
    let mut enlist_trigger_indices: Vec<usize> = Vec::new();
    for (i, t) in triggers[pre_len..].iter().enumerate() {
        if let Some(obj) = state.objects.get(&t.source) {
            if let Some(ta) =
                obj.characteristics.triggered_abilities.get(t.ability_index)
            {
                if ta.description.starts_with("Enlist") {
                    enlist_trigger_indices.push(pre_len + i);
                }
            }
        }
    }

    // Match pairings to placeholder triggers.
    // Tag matched triggers; mark unmatched for removal.
    let mut indices_to_remove: Vec<usize> = Vec::new();
    let mut pairing_iter = enlist_pairings_for_attacker.iter();
    for &idx in &enlist_trigger_indices {
        if let Some(&enlisted_id) = pairing_iter.next() {
            triggers[idx].is_enlist_trigger = true;
            triggers[idx].enlist_enlisted_creature = Some(enlisted_id);
        } else {
            // No pairing for this Enlist instance -- mark for removal.
            indices_to_remove.push(idx);
        }
    }

    // Remove unmatched Enlist placeholder triggers (reverse order to
    // preserve indices).
    for &idx in indices_to_remove.iter().rev() {
        triggers.remove(idx);
    }
}
```

**IMPORTANT**: The `triggers.remove(idx)` operates on a `Vec`, so removing in reverse
order preserves indices. Verify that `triggers` is indeed a `Vec<PendingTrigger>` (it is --
see `check_triggers` return type).

#### 3d. Flush pending triggers (abilities.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Location**: In `flush_pending_triggers`, in the chain of `else if trigger.is_X_trigger`
checks, after the `is_renown_trigger` block (line ~2717):

```rust
} else if trigger.is_enlist_trigger {
    // CR 702.154a: Enlist trigger -- "this creature gets +X/+0 until
    // end of turn, where X is the tapped creature's power."
    // `enlist_enlisted_creature` carries the tapped creature's ObjectId.
    StackObjectKind::EnlistTrigger {
        source_object: trigger.source,
        enlisted_creature: trigger
            .enlist_enlisted_creature
            .unwrap_or(trigger.source),
    }
```

#### 3e. Resolution (resolution.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add a resolution arm for `StackObjectKind::EnlistTrigger`. Insert it
after the `RenownTrigger` resolution block (which ends around line ~1970).

**Pattern**: Follow `FlankingTrigger` resolution (lines 1688-1726) which registers
a `ContinuousEffect` with `UntilEndOfTurn` duration.

```rust
// CR 702.154a: Enlist trigger resolves -- the enlisting creature gets
// +X/+0 until end of turn, where X is the tapped creature's power.
//
// The +X/+0 is a continuous effect in Layer 7c (PtModify) with
// UntilEndOfTurn duration. If the source (enlisting) creature has
// left the battlefield by resolution time (CR 400.7), the trigger
// does nothing.
//
// Power of the enlisted creature: use calculate_characteristics if
// the creature is still on the battlefield or in any zone. If the
// object no longer exists at all, use 0.
StackObjectKind::EnlistTrigger {
    source_object,
    enlisted_creature,
} => {
    let controller = stack_obj.controller;

    // Check if the source (enlisting) creature is still on the battlefield.
    let source_alive = state
        .objects
        .get(&source_object)
        .map(|obj| obj.zone == ZoneId::Battlefield)
        .unwrap_or(false);

    if source_alive {
        // Read the enlisted creature's power (layer-aware).
        // calculate_characteristics works regardless of zone.
        let enlisted_power =
            crate::rules::layers::calculate_characteristics(state, enlisted_creature)
                .and_then(|c| c.power)
                .unwrap_or(0);

        if enlisted_power > 0 {
            // Register the +X/+0 continuous effect.
            let eff_id = state.next_object_id().0;
            let ts = state.timestamp_counter;
            state.timestamp_counter += 1;
            let effect = crate::state::continuous_effect::ContinuousEffect {
                id: crate::state::continuous_effect::EffectId(eff_id),
                source: None, // trigger-based effect, not from a permanent
                timestamp: ts,
                layer: crate::state::continuous_effect::EffectLayer::PtModify,
                duration: crate::state::continuous_effect::EffectDuration::UntilEndOfTurn,
                filter: crate::state::continuous_effect::EffectFilter::SingleObject(
                    source_object,
                ),
                modification:
                    crate::state::continuous_effect::LayerModification::ModifyPower(
                        enlisted_power,
                    ),
                is_cda: false,
            };
            state.continuous_effects.push_back(effect);
        }
        // If enlisted_power <= 0, still resolve successfully (no buff applied).
    }
    // If source left the battlefield, do nothing (CR 400.7).

    events.push(GameEvent::AbilityResolved {
        controller,
        stack_object_id: stack_obj.id,
    });
}
```

#### 3f. Counter-on-stack arm (resolution.rs)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add `StackObjectKind::EnlistTrigger { .. }` to the counter-on-stack
match arm (the `|` chain around lines ~1994-1997) so that if an Enlist trigger is
countered (e.g., by Stifle), it is simply removed from the stack.

```rust
| StackObjectKind::EnlistTrigger { .. }
```

Insert after `StackObjectKind::RenownTrigger { .. }` in the chain.

---

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/enlist.rs`
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/training.rs`
and `/home/airbaggie/scutemob/crates/engine/tests/provoke.rs`

**Module header**:
```rust
//! Enlist keyword ability tests (CR 702.154).
//!
//! Enlist is a static ability (optional attack cost) + triggered ability.
//! "As this creature attacks, you may tap an untapped non-attacking creature
//! you control without summoning sickness. When you do, this creature gets
//! +X/+0 until end of turn, where X is the tapped creature's power."
//!
//! Key rules verified:
//! - Tapping an eligible creature adds its power as +X/+0 (CR 702.154a).
//! - Trigger uses the stack (ruling 2022-09-09).
//! - Enlisted creature must not be attacking (CR 702.154a).
//! - Enlisted creature must not have summoning sickness without haste (CR 702.154a).
//! - Cannot enlist self (CR 702.154c).
//! - A creature can only be enlisted once (ruling 2022-09-09).
//! - Multiple Enlist instances each tap a different creature (CR 702.154d).
//! - No enlist = no trigger (negative case).
//! - Multiplayer works correctly.
```

**Imports**:
```rust
use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};
```

**Helpers** (copy from training.rs):
```rust
fn find_object(state: &GameState, name: &str) -> ObjectId { ... }
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) { ... }
```

**Tests to write** (8 tests):

1. **`test_702_154a_enlist_basic_power_addition`** -- CR 702.154a
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Enlist on battlefield.
   - P1 has 3/3 vanilla creature on battlefield (no summoning sickness).
   - P1 declares the Enlist creature as attacker targeting P2, with
     `enlist_choices: [(enlist_creature_id, vanilla_creature_id)]`.
   - Assert: vanilla creature is tapped (PermanentTapped event).
   - Assert: `state.stack_objects.len() >= 1` (Enlist trigger on stack).
   - Pass all to resolve trigger.
   - Assert: `calculate_characteristics` on enlist creature shows power = 5
     (2 base + 3 from enlisted creature's power).
   - Assert: toughness unchanged at 2.

2. **`test_702_154a_enlist_no_choice_no_trigger`** -- CR 702.154a (negative)
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Enlist on battlefield.
   - P1 has 3/3 vanilla creature on battlefield.
   - P1 declares the Enlist creature as attacker with `enlist_choices: vec![]`.
   - Assert: no Enlist trigger on stack (stack may have other triggers, but no
     EnlistTrigger variant).
   - Assert: vanilla creature is NOT tapped.
   - Assert: Enlist creature power is still 2.

3. **`test_702_154a_enlist_enlisted_must_not_be_attacking`** -- CR 702.154a
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Enlist and 3/3 vanilla creature.
   - P1 declares BOTH as attackers, with `enlist_choices` trying to enlist
     the 3/3 (which is also attacking).
   - Assert: command returns `Err(InvalidCommand)` with message about the
     enlisted creature being an attacker.

4. **`test_702_154a_enlist_summoning_sickness_rejected`** -- CR 702.154a
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Enlist (no sickness) and 3/3 creature WITH
     summoning sickness, no haste.
   - To set summoning sickness: after building the state, get a mutable ref
     to the 3/3 creature's `GameObject` and set `has_summoning_sickness = true`.
   - P1 declares Enlist creature as attacker, tries to enlist the sick creature.
   - Assert: command returns `Err(InvalidCommand)` mentioning summoning sickness.

5. **`test_702_154a_enlist_summoning_sickness_with_haste_allowed`** -- CR 702.154a
   - 2 players: P1, P2.
   - P1 has 2/2 creature with Enlist and 3/3 creature WITH summoning sickness
     but also has Haste keyword.
   - Set `has_summoning_sickness = true` on the 3/3, add `KeywordAbility::Haste`.
   - P1 declares Enlist creature as attacker, enlists the hasty sick creature.
   - Assert: command succeeds. Trigger on stack. Resolves to +3/+0.
   - Assert: Enlist creature's power = 5 after resolution.

6. **`test_702_154c_enlist_cannot_enlist_self`** -- CR 702.154c
   - 2 players: P1, P2.
   - P1 has 3/3 creature with Enlist.
   - P1 declares it as attacker, tries to enlist itself:
     `enlist_choices: [(creature_id, creature_id)]`.
   - Assert: command returns `Err(InvalidCommand)` mentioning CR 702.154c.

7. **`test_702_154_enlist_creature_used_once_only`** -- ruling 2022-09-09
   - 2 players: P1, P2.
   - P1 has two 2/2 creatures with Enlist and one 4/4 vanilla creature.
   - P1 declares both Enlist creatures as attackers, tries to enlist the 4/4
     for BOTH: `enlist_choices: [(a1, big), (a2, big)]`.
   - Assert: command returns `Err(InvalidCommand)` mentioning the creature
     being already enlisted.

8. **`test_702_154a_enlist_multiplayer_four_player`** -- CR 702.154a + multiplayer
   - 4 players: P1, P2, P3, P4.
   - P1 has 1/1 creature with Enlist and 5/5 vanilla creature.
   - P1 declares Enlist creature attacking P2, enlists the 5/5.
   - All 4 players pass to resolve trigger.
   - Assert: Enlist creature power = 6 (1 + 5).
   - Assert: 5/5 creature is tapped.

---

### Step 5: Card Definition (later phase)

**Suggested card**: Coalition Skyknight
- **Name**: Coalition Skyknight
- **Cost**: {3}{W}
- **Type**: Creature -- Human Knight
- **P/T**: 2/2
- **Oracle**: "Flying\nEnlist (As this creature attacks, you may tap a nonattacking creature you control without summoning sickness. When you do, add its power to this creature's until end of turn.)"
- **Keywords**: `[KeywordAbility::Flying, KeywordAbility::Enlist]`
- **Color identity**: ["W"]

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/defs/coalition_skyknight.rs`
**Action**: Use `card-definition-author` agent.

**Why this card**: Simple creature with Flying (already validated) + Enlist (the
new keyword). No extra triggered/activated abilities. Good for testing Enlist
end-to-end in a game script with evasion + power bonus.

**Alternative card**: Guardian of New Benalia ({1}{W}, 2/2, Enlist + Scry-on-enlist +
discard-for-indestructible). More complex but better for testing the linked trigger
("Whenever this creature enlists a creature, scry 2" -- but this second ability is
a separate triggered ability, not part of the Enlist keyword itself). Use Coalition
Skyknight for the basic case.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Enlist power bonus in 4-player Commander"
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Sequence number**: Check next available after existing scripts (currently 120 is last).
Use 121.

**Script outline**:
1. 4 players: P1 (40 life), P2 (40 life), P3 (40 life), P4 (40 life).
2. P1 controls Coalition Skyknight (2/2, Flying, Enlist) and a 4/4 vanilla creature
   on battlefield.
3. Advance to Declare Attackers step.
4. P1 declares Coalition Skyknight attacking P2, enlisting the 4/4.
5. Assert: stack has 1 trigger item (Enlist).
6. All 4 players pass priority -- trigger resolves.
7. Assert: Coalition Skyknight effective P/T is 6/2 (2 + 4 from enlisted).
8. Assert: the 4/4 creature is tapped.

**File**: Use `game-script-generator` agent.

---

## Interactions to Watch

1. **Enlist + Vigilance on the attacker**: A creature with both Enlist and Vigilance
   attacks without tapping (Vigilance). The enlisted creature is a DIFFERENT creature
   that gets tapped. The attacker's Vigilance status is irrelevant to the enlist
   mechanic. However, a creature with Vigilance that is declared as an attacker CANNOT
   be enlisted by another attacker -- it was "chosen to attack with" per CR 702.154a,
   regardless of whether it tapped.

2. **Enlist + Convoke on same turn**: Convoke taps creatures for mana cost; Enlist
   taps a creature for attack cost. Both are separate tap costs. The same creature
   cannot be used for both (it would already be tapped after convoke). No conflict
   in implementation.

3. **Enlist + UntilEndOfTurn expiry**: The +X/+0 continuous effect expires at the
   next cleanup step (CR 514.2), same as Bushido, Battle Cry, etc. Uses the existing
   `CEDuration::UntilEndOfTurn` cleanup in `turn_actions.rs`.

4. **Enlist + Power-changing instants**: Because the trigger uses the stack, an
   opponent can respond with an instant that changes the enlisted creature's power.
   This is correctly handled because power is read at resolution time, not at
   trigger/declaration time.

5. **Enlist + Humility**: If Humility removes the Enlist keyword, the creature cannot
   use Enlist (the command validation checks `calculate_characteristics` which reflects
   Humility's Layer 6 ability removal). If an Enlist trigger is already on the stack
   when Humility enters, the trigger still resolves normally (it's already on the
   stack, CR 603.6).

6. **Enlist + Goad**: A goaded creature must attack if able. A goaded creature that
   is attacking cannot be enlisted. A goaded creature that is NOT attacking could
   theoretically be enlisted if it meets requirements, but in practice it will fail
   the enlist requirements (it can't attack means it's tapped, has sickness, or has
   defender). No special handling needed.

7. **Multiple Enlist instances (CR 702.154d)**: Two Enlist keywords on one creature =
   two separate optional costs. Each can tap a different creature. Each generates its
   own trigger. The `enlist_choices` vector can contain two entries for the same
   attacker, each with a different enlisted creature. Validation ensures the count
   does not exceed the Enlist keyword count.

8. **Enlist + creature with 0 or negative power**: If the enlisted creature has 0
   power at resolution, the continuous effect applies +0/+0 (effectively nothing).
   The resolution code only registers the effect if `enlisted_power > 0`. If negative,
   the `calculate_characteristics` returns a negative `power` value, and the
   resolution should NOT apply a negative modifier unless we want to faithfully
   implement that (a -1 power creature enlisted would give -1/+0). Decision: apply
   ModifyPower(X) for any non-zero X, even negative. Only skip if X == 0.

---

## Summary of Files Modified

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Enlist` variant |
| `crates/engine/src/state/hash.rs` | Add hash arm for `KeywordAbility::Enlist` (disc 83), `StackObjectKind::EnlistTrigger` (disc 23), `CombatState.enlist_pairings`, `PendingTrigger` new fields |
| `crates/engine/src/state/stack.rs` | Add `StackObjectKind::EnlistTrigger` variant |
| `crates/engine/src/state/stubs.rs` | Add `is_enlist_trigger` + `enlist_enlisted_creature` fields to `PendingTrigger` |
| `crates/engine/src/state/combat.rs` | Add `enlist_pairings: Vec<(ObjectId, ObjectId)>` to `CombatState` + initialize in `new()` |
| `crates/engine/src/state/builder.rs` | Add placeholder `TriggeredAbilityDef` for Enlist keyword |
| `crates/engine/src/rules/command.rs` | Add `enlist_choices` field to `DeclareAttackers` variant |
| `crates/engine/src/rules/engine.rs` | Update `DeclareAttackers` destructuring to pass `enlist_choices` |
| `crates/engine/src/rules/combat.rs` | Validate enlist choices (10 checks), tap enlisted creatures, store pairings |
| `crates/engine/src/rules/abilities.rs` | Post-process Enlist triggers in AttackersDeclared handler + flush arm |
| `crates/engine/src/rules/resolution.rs` | Resolve `EnlistTrigger` (+X/+0 continuous effect) + counter arm |
| `tools/tui/src/play/panels/stack_view.rs` | Add `EnlistTrigger` match arm |
| `tools/replay-viewer/src/view_model.rs` | Add `format_keyword` and `format_stack_kind` arms |
| `crates/engine/src/testing/replay_harness.rs` | Add `enlist` field parsing for `declare_attackers` + `EnlistDecl` struct |
| `crates/engine/tests/enlist.rs` | New test file with 8 tests |
| `crates/engine/src/cards/defs/coalition_skyknight.rs` | Coalition Skyknight card def (Step 5) |
| `test-data/generated-scripts/combat/` | Enlist game script 121 (Step 6) |

## Complexity Assessment

**Effort**: Medium-High. This is more complex than most keyword abilities because:
1. It requires a new field on the `DeclareAttackers` command (infrastructure change to
   a fundamental combat command).
2. Validation logic in `combat.rs` is substantial (10 checks).
3. The trigger is custom (`EnlistTrigger` StackObjectKind) because it needs to reference
   the specific enlisted creature at resolution time.
4. Resolution involves dynamic power reading at resolve time.
5. The replay harness needs schema extension for the new `enlist` action field.
6. The trigger post-processing in `abilities.rs` must selectively remove unmatched
   Enlist placeholder triggers (new pattern not used by any existing ability).

The closest analog is Provoke (also modifies declare-attackers, has custom
StackObjectKind, and requires validation). Flanking is the closest for the resolution
pattern (ContinuousEffect registration with UntilEndOfTurn).

## Risk: Edge Case with enlisted_power <= 0

The resolution code should handle the case where the enlisted creature's power is
0 or negative at resolution time. The current plan applies `ModifyPower(X)` only
when `X > 0`. However, per strict CR reading, even a 0-power creature can be
enlisted (the CR says "you may tap... When you do, this creature gets +X/+0 where
X is the tapped creature's power"). The trigger resolves regardless of X. If X is 0,
the effect does nothing useful but is still technically applied. If X is negative
(e.g., a creature with -1 power due to a debuff), the effect should apply -1/+0.

**Recommendation**: Apply `ModifyPower(X)` for any `X != 0`. Skip only when `X == 0`
(optimization -- a +0/+0 effect is a no-op). This correctly handles negative power.
