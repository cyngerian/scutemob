# Ability Plan: The Ring Tempts You

**Generated**: 2026-03-09
**CR**: 701.54
**Priority**: P4 (W1-B16, part 2 of Dungeon batch)
**Similar abilities studied**: Dungeon/Venture (CR 701.49) — `state/dungeon.rs`, `rules/engine.rs:2136-2220`, `effects/mod.rs:2047`; Suspect designation (CR 701.60) — `effects/mod.rs:1743`, `rules/layers.rs:64-73`, `rules/combat.rs:553-564`; Skulk blocking restriction — `rules/combat.rs:707-721`

## CR Rule Text

```
701.54. The Ring Tempts You

701.54a Certain spells and abilities have the text "the Ring tempts you." Each time
the Ring tempts you, choose a creature you control. That creature becomes your
Ring-bearer until another creature becomes your Ring-bearer or another player gains
control of it.

701.54b Ring-bearer is a designation a permanent can have. Being a Ring-bearer is
not a copiable value.

701.54c If a player doesn't have an emblem named The Ring at the time the Ring
tempts them, they get an emblem named The Ring before choosing a creature to be
their Ring-bearer. The Ring has "Your Ring-bearer is legendary and can't be blocked
by creatures with greater power." As long as the Ring has tempted that player two
or more times, it has "Whenever your Ring-bearer attacks, draw a card, then discard
a card." As long as the Ring has tempted that player three or more times, it has
"Whenever your Ring-bearer becomes blocked by a creature, the blocking creature's
controller sacrifices it at end of combat." As long as the Ring has tempted that
player four or more times, it has "Whenever your Ring-bearer deals combat damage to
a player, each opponent loses 3 life."

701.54d Some abilities trigger "Whenever the Ring tempts you." The Ring tempts a
player whenever they complete the actions in 701.54a, even if some or all of those
actions were impossible.

701.54e Some abilities check to see if a creature "is your Ring-bearer." For the
purposes of those abilities, that condition is true if that creature is on the
battlefield under your control and has the Ring-bearer designation.
```

## Key Edge Cases

- **No creatures**: The ring can tempt you even if you control no creatures. You still get the emblem/level advancement, and "whenever the Ring tempts you" triggers still fire (ruling from Call of the Ring 2023-06-16).
- **Re-choosing the same creature**: Choosing a creature that is already your ring-bearer still counts as choosing (triggers "whenever you choose a creature as your Ring-bearer"). CR 701.54a doesn't require a different creature.
- **Control change**: If another player gains control of your ring-bearer, it stops being your ring-bearer (CR 701.54a). The creature does NOT become the new controller's ring-bearer automatically.
- **Ring abilities are cumulative**: The emblem gains abilities permanently in order. Level 2 has abilities 1+2, level 3 has 1+2+3, etc. Once gained, abilities persist for the rest of the game (ruling 2023-06-16).
- **Ring-bearer is a designation, not a copiable value** (CR 701.54b). Copy effects do not make the copy a ring-bearer.
- **Must choose a creature if you control one** (ruling 2023-06-16). Not optional.
- **Each player has at most one Ring emblem and one ring-bearer** (ruling 2023-06-16).
- **Multiplayer**: Each player independently tracks their own ring level and ring-bearer. All 4+ players can have their own Ring emblem at different levels.
- **Level 1 blocking restriction**: "can't be blocked by creatures with greater power" is identical to Skulk (CR 702.118b). Reuse the same combat.rs validation pattern.
- **Level 2 attack trigger**: "Whenever your Ring-bearer attacks, draw a card, then discard a card." — Looting trigger on attack declaration.
- **Level 3 block trigger**: "Whenever your Ring-bearer becomes blocked by a creature, that creature's controller sacrifices it at end of combat." — Uses end-of-combat sacrifice pattern (like Decayed).
- **Level 4 combat damage trigger**: "Whenever your Ring-bearer deals combat damage to a player, each opponent loses 3 life." — Uses existing `WhenDealsCombatDamageToPlayer` trigger infrastructure.

## Current State (from ability-wip.md)

No existing ring infrastructure. This is a fresh implementation.

- [ ] Step 1: Data model (PlayerState fields, Designations bit)
- [ ] Step 2: Effect::TheRingTemptsYou + handle_ring_tempts_you
- [ ] Step 3: Layer system integration (legendary + blocking restriction)
- [ ] Step 4: Ring trigger dispatch (levels 2-4)
- [ ] Step 5: Unit tests
- [ ] Step 6: Card definitions (later phase)
- [ ] Step 7: Game scripts (later phase)

## Modification Surface

Files and functions that need changes, mapped via dungeon/suspect pattern analysis:

| File | Function/Match | What to add |
|------|---------------|-------------|
| `state/player.rs` | `PlayerState` | `ring_level: u8` (0 = no ring, 1-4 = levels), `ring_bearer_id: Option<ObjectId>` |
| `state/game_object.rs` | `Designations` bitflags | `RING_BEARER = 1 << 8` |
| `state/hash.rs` | `PlayerState` HashInto | Hash `ring_level`, `ring_bearer_id` |
| `state/hash.rs` | `Designations` | Already hashed as u16, no change needed |
| `state/hash.rs` | `Effect` HashInto | Add discriminant for `Effect::TheRingTemptsYou` |
| `state/hash.rs` | `GameEvent` HashInto | Add discriminants for new events |
| `state/hash.rs` | `Condition` HashInto | Add discriminant for `RingHasTemptedYou(u8)` |
| `cards/card_definition.rs` | `Effect` enum | Add `TheRingTemptsYou` variant |
| `cards/card_definition.rs` | `Condition` enum | Add `RingHasTemptedYou(u8)` (for Frodo-type cards) |
| `cards/card_definition.rs` | `TriggerCondition` enum | Add `WheneverRingTemptsYou` |
| `rules/events.rs` | `GameEvent` enum | Add `RingTempted { player, new_level }`, `RingBearerChosen { player, creature }` |
| `rules/engine.rs` | `process_command` | Add `Command::TheRingTemptsYou` handler (or handle inline in effect) |
| `effects/mod.rs` | `execute_effect` | Add `Effect::TheRingTemptsYou` execution |
| `effects/mod.rs` | `evaluate_condition` | Add `Condition::RingHasTemptedYou(n)` arm |
| `rules/layers.rs` | Pre-layer-loop section | Add ring-bearer legendary supertype (Layer 4) |
| `rules/combat.rs` | `validate_blocker` | Add ring-bearer + ring level >= 1 blocking restriction (like Skulk) |
| `rules/combat.rs` | Provoke impossibility check | Add ring-bearer blocking restriction check |
| `rules/abilities.rs` | `check_triggers` | Add ring level 2 attack trigger, level 3 block trigger, level 4 combat damage trigger |
| `rules/turn_actions.rs` | `end_combat()` | Add ring level 3 sacrifice-at-end-of-combat handling |
| `rules/sba.rs` | SBA checks | Add ring-bearer control change check (clear designation when controller changes) |
| `testing/replay_harness.rs` | `translate_player_action` | Add `ring_tempts_you` harness action (optional) |
| `tools/replay-viewer/src/view_model.rs` | `keyword_display` | No new KW variant needed (Ring is not a keyword ability) |
| `tools/tui/src/play/panels/stack_view.rs` | SOK match | Add arm if any new SOK variant (unlikely — ring triggers use `KeywordTrigger`) |

## Implementation Steps

### Step 1: Data Model

**Files**: `crates/engine/src/state/player.rs`, `crates/engine/src/state/game_object.rs`, `crates/engine/src/state/hash.rs`

**Action — PlayerState fields**:
Add two fields to `PlayerState`:

```rust
/// CR 701.54c: Number of times the Ring has tempted this player (0-4, capped).
/// Determines which ring abilities are active. Once incremented, never decreases.
#[serde(default)]
pub ring_level: u8,

/// CR 701.54a: ObjectId of this player's current ring-bearer creature.
/// `None` if the player has no ring-bearer (never tempted, or no creatures when tempted,
/// or ring-bearer left the battlefield / changed control).
#[serde(default)]
pub ring_bearer_id: Option<ObjectId>,
```

**Pattern**: Follow `dungeons_completed` / `dungeons_completed_set` at `player.rs:152-166`.

**Action — Designations bitfield**:
Add `RING_BEARER = 1 << 8` to the `Designations` bitflags in `game_object.rs`.

```rust
/// CR 701.54b: Ring-bearer designation. Not a copiable value.
const RING_BEARER    = 1 << 8;
```

**Pattern**: Follow `RECONFIGURED = 1 << 7` at `game_object.rs:37`.

**Action — Hash**:
In `hash.rs` `PlayerState` HashInto impl, add:
```rust
self.ring_level.hash_into(hasher);
self.ring_bearer_id.hash_into(hasher);
```

**Pattern**: Follow `self.dungeons_completed.hash_into(hasher)` at `hash.rs:995`.

**Note**: `ring_level` is a `u8` (0 = not yet tempted; 1-4 = ring level; capped at 4). NOT a separate `RingLevel` enum. The CR text says "two or more times", "three or more times", "four or more times" — a simple counter is the natural representation.

**Note**: `ring_bearer_id` is an `Option<ObjectId>` on `PlayerState` (not on `GameState`). Per-player, one ring-bearer at a time. Stored as `ObjectId` rather than `CardId` because the ring-bearer designation is lost on zone change (CR 400.7 — new object), unlike commander identity which persists.

### Step 2: Events

**File**: `crates/engine/src/rules/events.rs`

**Action**: Add two new `GameEvent` variants:

```rust
/// CR 701.54a: The Ring tempted a player. Emitted after the ring level advances.
///
/// Discriminant: 117.
RingTempted {
    /// The player who was tempted.
    player: PlayerId,
    /// The new ring level (1-4) after this temptation.
    new_level: u8,
},

/// CR 701.54a: A creature was chosen as a player's ring-bearer.
///
/// Discriminant: 118.
RingBearerChosen {
    /// The player who chose the ring-bearer.
    player: PlayerId,
    /// The creature that became the ring-bearer.
    creature: ObjectId,
},
```

**Hash**: Add discriminants 117 and 118 to `GameEvent` HashInto in `hash.rs`.

**Pattern**: Follow `DungeonCompleted` at `events.rs:1233` (discriminant 115) and `InitiativeTaken` at `events.rs:1246` (discriminant 116).

### Step 3: Effect + Condition + TriggerCondition

**File**: `crates/engine/src/cards/card_definition.rs`

**Action — Effect**:
Add `TheRingTemptsYou` to the `Effect` enum:

```rust
/// CR 701.54a: "The ring tempts you."
///
/// Advances the controller's ring level (cap 4), then the controller chooses
/// a creature they control as their ring-bearer. Deterministic fallback: choose
/// the creature with the lowest ObjectId.
TheRingTemptsYou,
```

**Pattern**: Follow `Effect::VentureIntoDungeon` at `card_definition.rs:1148` and `Effect::TakeTheInitiative` at `card_definition.rs:1155`.

**Hash**: Add Effect discriminant 51 in `hash.rs` (after `TakeTheInitiative` at discriminant 50).

**Action — Condition**:
Add to `Condition` enum:

```rust
/// CR 701.54c: "if the Ring has tempted you N or more times this game."
/// True when the controller's `ring_level >= n`.
/// Used by Frodo, Sauron's Bane (checks ring_level >= 4).
RingHasTemptedYou(u8),
```

**Hash**: Add Condition discriminant 17 (after `CompletedSpecificDungeon` at 16).

**Action — TriggerCondition**:
Add to `TriggerCondition` enum:

```rust
/// CR 701.54d: "Whenever the Ring tempts you."
WheneverRingTemptsYou,
```

This will be matched against `GameEvent::RingTempted` in `check_triggers`.

### Step 4: Effect Execution — handle_ring_tempts_you

**File**: `crates/engine/src/effects/mod.rs`

**Action**: Add `Effect::TheRingTemptsYou` arm to `execute_effect`:

```rust
Effect::TheRingTemptsYou => {
    let controller = ctx.controller;
    if let Ok(ring_events) = handle_ring_tempts_you(state, controller) {
        events.extend(ring_events);
    }
}
```

**New function** `handle_ring_tempts_you` (in `effects/mod.rs` or as a standalone helper in `rules/engine.rs` — follow the dungeon pattern which puts `handle_venture_into_dungeon` in `rules/engine.rs`):

```rust
/// CR 701.54a-c: Process "the Ring tempts you" for a player.
///
/// Steps:
/// 1. Advance ring_level (cap at 4). Emit RingTempted event.
/// 2. Choose a creature the player controls as ring-bearer (deterministic: lowest ObjectId).
/// 3. Clear RING_BEARER from previous ring-bearer (if different).
/// 4. Set RING_BEARER on new ring-bearer. Update player.ring_bearer_id.
/// 5. Emit RingBearerChosen event.
/// 6. Return events (RingTempted always; RingBearerChosen only if a creature was chosen).
pub fn handle_ring_tempts_you(
    state: &mut GameState,
    player: PlayerId,
) -> Result<Vec<GameEvent>, GameStateError>
```

**Key logic**:
- Ring level increments from 0 to min(current+1, 4). Even if no creature available, level still advances.
- Creature selection: find all creatures on battlefield controlled by `player`. If none, skip ring-bearer selection but still emit `RingTempted`. Deterministic fallback: choose creature with lowest ObjectId.
- If the chosen creature is already the ring-bearer, still emit `RingBearerChosen` (ruling 2023-06-16).
- Clear old ring-bearer's `RING_BEARER` designation if it's a different creature.
- Set new ring-bearer's `RING_BEARER` designation and update `player.ring_bearer_id`.

**Pattern**: Follow `handle_venture_into_dungeon` at `rules/engine.rs:2136-2220`.

**Also wire into Command handler** in `rules/engine.rs` (optional — if we want a `Command::TheRingTemptsYou`):

```rust
Command::TheRingTemptsYou { player } => {
    let events = handle_ring_tempts_you(&mut state, player)?;
    let new_triggers = abilities::check_triggers(&state, &events);
    for t in new_triggers {
        state.pending_triggers.push_back(t);
    }
    flush_pending_triggers(&mut state);
    all_events.extend(events);
}
```

However, given that "the ring tempts you" typically comes from spell/ability resolution (like VentureIntoDungeon), the `Effect` path is primary. A `Command` variant is optional — only needed if a harness action wants to directly invoke it. For now, **skip the Command variant** and rely on `Effect::TheRingTemptsYou` being executed during resolution.

**Condition evaluation** — add to `evaluate_condition` in `effects/mod.rs`:

```rust
Condition::RingHasTemptedYou(n) => state
    .players
    .get(&ctx.controller)
    .map(|ps| ps.ring_level >= *n)
    .unwrap_or(false),
```

**Pattern**: Follow `Condition::CompletedADungeon` at `effects/mod.rs:3471-3475`.

### Step 5: Layer System — Ring-Bearer Gets Legendary

**File**: `crates/engine/src/rules/layers.rs`

**Action**: In the pre-layer-loop section (where Suspected grants Menace), add ring-bearer legendary supertype grant:

```rust
// CR 701.54c (ring level >= 1): Ring-bearer is legendary.
// Applied pre-layer-loop (Layer 4) like Suspected's menace grant.
// This adds the Legendary supertype — does NOT need a continuous effect
// registration because it's tied to the designation, not a specific source.
if obj.designations.contains(Designations::RING_BEARER) && obj.zone == ZoneId::Battlefield {
    // Check if the owner's ring level >= 1 (always true if they have a ring-bearer,
    // since level advances before choosing a ring-bearer).
    chars.supertypes.insert(SuperType::Legendary);
}
```

**Pattern**: Follow Suspected Menace grant at `layers.rs:64-73`.

**Note**: The legendary supertype is part of ring level 1 which is always active once the ring has tempted you at least once. Since `ring_level` advances before the ring-bearer is chosen, any creature with `RING_BEARER` always has at least ring level 1.

### Step 6: Combat — Blocking Restriction (Ring Level >= 1)

**File**: `crates/engine/src/rules/combat.rs`

**Action**: In `validate_blocker` (after Skulk check at line ~711), add ring-bearer blocking restriction:

```rust
// CR 701.54c (ring level >= 1): Ring-bearer can't be blocked by creatures
// with greater power. Identical to Skulk (CR 702.118b) but triggered by
// the ring-bearer designation rather than a keyword.
if attacker_obj.designations.contains(Designations::RING_BEARER) {
    // Verify the attacker's controller has ring level >= 1
    if let Some(ps) = state.players.get(&attacker_obj.controller) {
        if ps.ring_level >= 1 {
            let attacker_power = attacker_chars.power.unwrap_or(0);
            let blocker_power = blocker_chars.power.unwrap_or(0);
            if blocker_power > attacker_power {
                return Err(GameStateError::InvalidCommand(format!(
                    "Object {:?} cannot block ring-bearer {:?} \
                     (blocker power {} > attacker power {}, CR 701.54c)",
                    blocker_id, attacker_id, blocker_power, attacker_power
                )));
            }
        }
    }
}
```

**Also** add the same check in the provoke impossibility check section (~line 914) to skip impossible provoke requirements.

**Pattern**: Follow Skulk blocking check at `combat.rs:707-721` and `combat.rs:914-921`.

### Step 7: Ring Trigger Dispatch (Levels 2-4)

**File**: `crates/engine/src/rules/abilities.rs`

**Action**: Add trigger dispatch for ring-bearer-related events.

**Level 2 (ring level >= 2): "Whenever your Ring-bearer attacks, draw a card, then discard a card."**

In `check_triggers`, match `GameEvent::AttackersDeclared`:
- For each attacker, check if the attacking creature has `RING_BEARER` designation.
- Check if the controller's `ring_level >= 2`.
- If so, push a `PendingTrigger` with `PendingTriggerKind::Normal` that executes `Sequence([DrawCards(1), DiscardCards(1)])` (looting).
- Use `StackObjectKind::KeywordTrigger` with a new `TriggerData` variant (see Step 7b below).

**Level 3 (ring level >= 3): "Whenever your Ring-bearer becomes blocked by a creature, that creature's controller sacrifices it at end of combat."**

In `check_triggers`, match `GameEvent::BlockersDeclared` (or the relevant blocking event):
- For each blocker assigned to the ring-bearer, check `ring_level >= 3`.
- If so, mark the blocking creature for end-of-combat sacrifice (similar to Decayed pattern).
- This requires a per-creature flag or a collection on `GameState`. Use `ring_sacrifice_at_eoc: bool` on `GameObject` following the Decayed `decayed_sacrifice_at_eoc` pattern, OR use a Vec on `GameState`. The flag approach is simpler.

**BUT WAIT** — the "sacrifice at end of combat" wording means the sacrifice is deferred, not immediate. This is distinct from the blocking creature dying immediately. Use the EOC flag pattern from `gotchas-infra.md`:
1. Add `ring_sacrifice_at_eoc: bool` on `GameObject`.
2. Set it in the blocker-declared trigger handler.
3. Check it in `end_combat()` in `turn_actions.rs`.
4. Reset in `move_object_to_zone()`.
5. Initialize to `false` in builder.rs, effects/mod.rs (token), resolution.rs.
6. Hash in `hash.rs`.

**Level 4 (ring level >= 4): "Whenever your Ring-bearer deals combat damage to a player, each opponent loses 3 life."**

In `check_triggers`, match `GameEvent::DamageDealt` (or `CombatDamageDealt`):
- Check if the damage source has `RING_BEARER` designation and dealt combat damage to a player.
- Check if the controller's `ring_level >= 4`.
- If so, push a trigger that executes `LoseLife { player: EachOpponent, amount: Fixed(3) }`.

**TriggerData variants**: Add to the `TriggerData` enum:

```rust
/// CR 701.54c: Ring-bearer loot trigger (level 2+).
RingBearerLoot,
/// CR 701.54c: Ring-bearer blocked sacrifice trigger (level 3+).
RingBearerBlockedSacrifice { blocking_creature: ObjectId },
/// CR 701.54c: Ring-bearer combat damage trigger (level 4+).
RingBearerCombatDamage,
```

**Resolution**: In `resolution.rs` `KeywordTrigger` match block, add arms for these three `TriggerData` variants.

**"Whenever the Ring tempts you" trigger**: In `check_triggers`, match `GameEvent::RingTempted`:
- Scan battlefield for permanents with `TriggerCondition::WheneverRingTemptsYou` triggered abilities controlled by the tempted player.
- Fire a `PendingTriggerKind::Normal` trigger for each match.

**Pattern**: Follow existing trigger dispatch in `abilities.rs` for `AttackersDeclared` events and `DamageDealt` events.

### Step 7b: TriggerData + SOK + Resolution

**File**: `crates/engine/src/state/hash.rs` (TriggerData hash), `crates/engine/src/rules/resolution.rs` (dispatch)

**TriggerData** — Add 3 new variants to the `TriggerData` enum. Find the enum location:

```
Grep pattern="enum TriggerData" path="crates/engine/src/"
```

**StackObjectKind** — Ring triggers use `KeywordTrigger` with a synthetic keyword. However, "The Ring Tempts You" is NOT a keyword ability. Options:
1. Use `KeywordTrigger { keyword: KeywordAbility::TheRing, data: TriggerData::... }` — but this requires adding a KW variant for a non-keyword.
2. Use a new SOK variant `RingTrigger { source_object, data: TriggerData }` — adds to SOK (goes against consolidation).
3. Use `PendingTriggerKind::Normal` with the effect embedded directly — simplest, no new SOK/TriggerData needed.

**Decision: Use `PendingTriggerKind::Normal`** for ring triggers. This is the same pattern used for CardDef-defined triggers. The effect to execute is known at trigger creation time:
- Level 2: `Sequence([DrawCards(1), DiscardCards(1)])`
- Level 3: Sacrifice the blocking creature (use `Effect::SacrificeTarget` or similar)
- Level 4: `LoseLife { player: EachOpponent, amount: Fixed(3) }`

This avoids new SOK or TriggerData variants entirely. The triggers are synthetic (created by the engine based on ring state, not from CardDef abilities), but `PendingTriggerKind::Normal` with an embedded `AbilityDefinition::Triggered` works fine.

**Implementation detail**: In `check_triggers` (abilities.rs), when detecting ring-bearer events:
- Create a `PendingTrigger` with `kind: PendingTriggerKind::Normal` and fill in the ability index pointing to a synthetically constructed `AbilityDefinition::Triggered` in the trigger's embedded data.

**Alternative simpler approach**: Create the triggers directly in the ring event handlers and push `StackObjectKind::TriggeredAbility` onto the stack with the effect embedded. This is what dungeon room abilities do (`StackObjectKind::RoomAbility`).

**Final decision**: Use a **new SOK variant `RingAbility`** following the `RoomAbility` pattern. This is cleaner than trying to fit ring triggers into an existing SOK pattern:

```rust
/// CR 701.54c: Ring-bearer triggered ability (levels 2-4).
///
/// Discriminant: next after current max.
RingAbility {
    /// The ring-bearer creature that caused this trigger.
    source_object: ObjectId,
    /// The effect to execute when this resolves.
    effect: Box<Effect>,
    /// The player who controls the ring.
    controller: PlayerId,
},
```

**Hash**: Add SOK discriminant. Check current max SOK discriminant.

**Resolution**: In `resolution.rs`, add a match arm for `StackObjectKind::RingAbility`:
```rust
StackObjectKind::RingAbility { effect, controller, .. } => {
    let ctx = EffectContext { controller, source: source_object, .. };
    execute_effect(state, &effect, &ctx)?;
}
```

### Step 8: Ring-Bearer Control Change SBA

**File**: `crates/engine/src/rules/sba.rs`

**Action**: Add an SBA check that clears the ring-bearer designation when the ring-bearer's controller changes:

```rust
/// CR 701.54a: Ring-bearer loses designation when another player gains control.
fn check_ring_bearer_control_change(state: &mut GameState) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for (player_id, ps) in state.players.iter() {
        if let Some(bearer_id) = ps.ring_bearer_id {
            if let Some(obj) = state.objects.get(&bearer_id) {
                // Check if the ring-bearer is still on the battlefield under this player's control
                if obj.controller != *player_id || !matches!(obj.zone, ZoneId::Battlefield) {
                    // Clear designation
                    // (Must be done via mutable access below)
                }
            } else {
                // Object no longer exists (left battlefield — CR 400.7)
                // Clear ring_bearer_id
            }
        }
    }
    // Apply mutations...
    events
}
```

**Note**: Because `im-rs` requires building new maps for mutation, this SBA collects player IDs that need clearing, then applies the mutations in a second pass.

**Pattern**: Follow `check_dungeon_completion_sba` at `sba.rs:1317`.

**Wire into `check_state_based_actions`**: Add call to `check_ring_bearer_control_change` alongside other SBA checks.

### Step 9: Unit Tests

**File**: `crates/engine/tests/ring_tempts_you.rs`

**Tests to write**:

1. `test_ring_tempts_you_basic_level_1` — CR 701.54a/c: Ring tempts player, ring level goes to 1, ring-bearer chosen (lowest ObjectId), creature gets RING_BEARER designation and Legendary supertype.

2. `test_ring_tempts_you_level_progression` — CR 701.54c: Ring tempts 4 times, level advances from 1 to 4, capped at 4 on 5th temptation.

3. `test_ring_tempts_you_no_creatures` — CR 701.54a + ruling: Ring tempts with no creatures controlled, level advances but no ring-bearer chosen. ring_bearer_id remains None.

4. `test_ring_tempts_you_re_choose_same_creature` — Ruling 2023-06-16: Choosing the same creature still counts; RingBearerChosen event still emitted.

5. `test_ring_bearer_blocking_restriction` — CR 701.54c level 1: Ring-bearer can't be blocked by creatures with greater power. Identical to Skulk check.

6. `test_ring_bearer_blocking_equal_power_allowed` — Complement: equal power CAN block (strictly greater, not >=).

7. `test_ring_bearer_legendary` — CR 701.54c level 1: Ring-bearer has Legendary supertype via layer system.

8. `test_ring_bearer_control_change_clears_designation` — CR 701.54a: When another player gains control, ring-bearer loses designation. SBA check.

9. `test_ring_bearer_leaves_battlefield_clears` — CR 400.7: Ring-bearer dies/exiled, ring_bearer_id cleared. New object in graveyard does NOT have RING_BEARER.

10. `test_ring_level_2_loot_trigger` — CR 701.54c: Ring-bearer attacks, draw-then-discard trigger fires (ring level >= 2).

11. `test_ring_level_3_sacrifice_at_eoc` — CR 701.54c: Ring-bearer becomes blocked, blocking creature sacrificed at end of combat (ring level >= 3). Deferred — complex combat setup.

12. `test_ring_level_4_combat_damage_trigger` — CR 701.54c: Ring-bearer deals combat damage to player, each opponent loses 3 life (ring level >= 4). Deferred — complex combat setup.

13. `test_ring_tempts_you_multiplayer_independence` — Each player tracks their own ring level and ring-bearer independently.

14. `test_whenever_ring_tempts_you_trigger` — CR 701.54d: "Whenever the Ring tempts you" triggered ability fires when Ring tempts event happens.

**Pattern**: Follow dungeon tests in `tests/dungeon_venture.rs` — use `GameStateBuilder::four_player()`, directly call `handle_ring_tempts_you`, assert on state and events.

### Step 10: Card Definitions (later phase)

**Suggested cards**:

1. **Call of the Ring** ({1}{B} Enchantment) — "At the beginning of your upkeep, the Ring tempts you. Whenever you choose a creature as your Ring-bearer, you may pay 2 life. If you do, draw a card."
   - Uses: `TriggerCondition::AtBeginningOfYourUpkeep`, `Effect::TheRingTemptsYou`, `TriggerCondition::WheneverRingTemptsYou` (second ability is a "whenever you choose" trigger — may need a separate trigger condition)

2. **Gollum, Patient Plotter** ({1}{B} Legendary Creature 3/1) — "When Gollum leaves the battlefield, the Ring tempts you."
   - Uses: `TriggerCondition::WhenLeavesBattlefield`, `Effect::TheRingTemptsYou`

**Card lookup**: use `card-definition-author` agent.

### Step 11: Game Scripts (later phase)

**Suggested scenario**: "Ring tempts you twice, ring-bearer attacks with looting trigger"
- Setup: Player has a creature. Two effects that tempt the ring. Ring reaches level 2. Ring-bearer attacks, looting trigger resolves.
- **Subsystem directory**: `test-data/generated-scripts/commander/` (or a new `ring/` subdirectory)

## Interactions to Watch

1. **Legendary rule interaction**: When ring-bearer becomes Legendary via ring level 1, the legend rule SBA applies. If you already control another legendary permanent with the same name, one must be sacrificed. This is a real edge case for tokens that become ring-bearers.

2. **Humility interaction**: Humility removes abilities in Layer 6, but the ring-bearer's Legendary supertype is applied in Layer 4 (supertypes). Since Layer 4 < Layer 6, the Legendary supertype persists even under Humility. However, the blocking restriction is a ring-designated property (not a keyword), so it should also persist. The loot/sacrifice/damage triggers are engine-generated from the ring state, not from the creature's abilities, so they also persist under Humility.

3. **Copy effects**: CR 701.54b says ring-bearer is NOT a copiable value. A clone of a ring-bearer is NOT a ring-bearer. The Designations bitfield is NOT copied — this is already correct because `Designations` are reset on zone change (CR 400.7).

4. **Mutate**: If a creature with ring-bearer mutates (over or under), the resulting merged creature keeps the ring-bearer designation because the bottom creature's ObjectId is preserved.

5. **Multiple ring temptations in one resolution**: If a spell says "the Ring tempts you" twice, both temptations happen sequentially, advancing the level twice. Each temptation individually triggers "whenever the Ring tempts you" abilities.

6. **End-of-combat sacrifice (level 3)**: Need the EOC flag pattern. The blocking creature is sacrificed, not destroyed — relevant for indestructible.

## Discriminant Chain

Based on current state (CLAUDE.md says KW 157, AbilDef 55, SOK ~20):

- **KeywordAbility**: No new KW variant needed. The Ring is a keyword action, not a keyword ability.
- **Effect**: Next discriminant = 51 (after TakeTheInitiative at 50) for `TheRingTemptsYou`.
- **GameEvent**: Next discriminants = 117, 118 (after InitiativeTaken at 116).
- **Condition**: Next discriminant = 17 (after CompletedSpecificDungeon at 16).
- **StackObjectKind**: Next discriminant = 60 (after MutatingCreatureSpell at 59) for `RingAbility`. Check current max before implementing.
- **Designations**: Bit 8 (after RECONFIGURED at bit 7) for RING_BEARER.
- **TriggerCondition**: Verify current max before adding `WheneverRingTemptsYou`.

**IMPORTANT**: Always verify discriminant chains from actual code before implementing. The planner gets these wrong frequently (see gotchas-infra.md).
