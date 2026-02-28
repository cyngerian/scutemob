# Ability Plan: Dethrone

**Generated**: 2026-02-27
**CR**: 702.105
**Priority**: P3
**Similar abilities studied**: Battle Cry (`KeywordAbility::BattleCry`, `TriggerEvent::SelfAttacks` in `builder.rs:443`, `abilities.rs:951`), Annihilator (`KeywordAbility::Annihilator`, `defending_player_id` pattern in `abilities.rs:962-963`)

## CR Rule Text

```
702.105. Dethrone

702.105a Dethrone is a triggered ability. "Dethrone" means "Whenever this creature
attacks the player with the most life or tied for most life, put a +1/+1 counter on
this creature."

702.105b If a creature has multiple instances of dethrone, each triggers separately.
```

## Key Edge Cases

- **"Most life" is checked among ALL players in the game** (not just the attacker's
  opponents). In a 4-player Commander game (P1 40, P2 40, P3 38, P4 35), P2 and P1 are
  tied for most life. If P1's creature attacks P2, dethrone triggers because P2 is tied
  for most life.
- **Dethrone does NOT trigger when attacking planeswalkers** (ruling 2023-07-28): "Dethrone
  doesn't trigger if the creature attacks a planeswalker, even if its controller has the
  most life. The same is true if the creature attacks a battle, even if its protector has
  the most life." Our engine supports `AttackTarget::Player` and
  `AttackTarget::Planeswalker` -- only `Player` targets should trigger dethrone.
- **Trigger condition is checked at declaration time only (CR 508.2a)**: "Abilities that
  trigger on a creature attacking trigger only at the point the creature is declared as
  an attacker." Once the trigger fires, the life total comparison is irrelevant at
  resolution time (ruling 2014-05-29): "Once dethrone triggers, it doesn't matter what
  happens to the players' life totals before the ability resolves."
- **The +1/+1 counter is put on the creature before blockers are declared** (ruling
  2014-05-29). This is automatic -- the trigger goes on the stack after attackers are
  declared, and both players must pass before the blockers step.
- **Multiple instances each trigger separately (CR 702.105b)**. A creature with two
  instances of dethrone gets two +1/+1 counters (two separate triggers, each puts one
  counter).
- **The counter goes on THIS creature** (the one with dethrone), not the defending player
  or any other target. The effect uses `EffectTarget::Source`.
- **Multiplayer: "player with the most life" includes ALL active (non-eliminated) players.**
  In a 4-player game, the comparison is across all 4 players. If two players are tied at
  the highest life total, attacking either of them triggers dethrone.
- **Self-attack consideration**: If the attacking player themselves has the most life and
  attacks a different player who does NOT have the most life, dethrone does NOT trigger.
  The check is specifically about the *defending* player's life total relative to all
  players.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant + TriggerEvent variant
- [ ] Step 2: Trigger auto-generation in builder.rs
- [ ] Step 3: Trigger wiring in abilities.rs AttackersDeclared handler
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant + TriggerEvent Variant

#### 1a. KeywordAbility::Dethrone

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Dethrone` variant after `BattleCry` (around line 330).
**Doc comment**:
```rust
/// CR 702.105: Dethrone -- "Whenever this creature attacks the player with
/// the most life or tied for most life, put a +1/+1 counter on this creature."
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// Multiple instances each trigger separately (CR 702.105b).
Dethrone,
```

#### 1b. TriggerEvent::SelfAttacksPlayerWithMostLife

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add new variant to `TriggerEvent` enum (after `SelfAttacks` at line 139).

**Why a dedicated variant**: Dethrone triggers on `SelfAttacks` but with a CONDITION
(defending player has most life). This condition depends on the `AttackTarget` for each
specific attacker in the loop, which is not available inside `collect_triggers_for_event`.
Using `TriggerEvent::SelfAttacks` would collect dethrone triggers unconditionally; we
would then need to retroactively remove them. A dedicated variant lets us call
`collect_triggers_for_event` with `SelfAttacksPlayerWithMostLife` ONLY when the condition
is met, keeping the logic clean and consistent with how exalted uses
`ControllerCreatureAttacksAlone`.

```rust
/// CR 702.105a: Triggers when this creature attacks a player who has the
/// most life or is tied for the most life among all players.
/// Only fires for AttackTarget::Player (not planeswalker/battle).
/// The "most life" check is done at trigger-collection time in
/// `rules/abilities.rs` AttackersDeclared handler.
SelfAttacksPlayerWithMostLife,
```

#### 1c. Hash for KeywordAbility::Dethrone

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for KeywordAbility` impl, after `Undaunted` (discriminant 54,
line 414). Next available discriminant is **55**.
```rust
// Dethrone (discriminant 55) -- CR 702.105
KeywordAbility::Dethrone => 55u8.hash_into(hasher),
```

#### 1d. Hash for TriggerEvent::SelfAttacksPlayerWithMostLife

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Location**: In `HashInto for TriggerEvent` impl (line 1020-1046), after
`ControllerCastsSpell` (discriminant 13, line 1044). Next available discriminant is **14**.
```rust
// CR 702.105a: Dethrone "attacks player with most life" trigger — discriminant 14
TriggerEvent::SelfAttacksPlayerWithMostLife => 14u8.hash_into(hasher),
```

#### 1e. View Model format_keyword

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Location**: In `format_keyword` function (line 577-637), add after `BattleCry` (line 620):
```rust
KeywordAbility::Dethrone => "Dethrone".to_string(),
```

#### 1f. Match Arm Audit

Grep for exhaustive `match` on `KeywordAbility` and `TriggerEvent` across the codebase.
Add the new arms to every match. Known locations:
- `state/hash.rs`: `HashInto for KeywordAbility` (1c above)
- `state/hash.rs`: `HashInto for TriggerEvent` (1d above)
- `tools/replay-viewer/src/view_model.rs`: `format_keyword` (1e above)
- Any other exhaustive matches will cause a compile error -- fix them all.

### Step 2: Trigger Auto-Generation in builder.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add a dethrone block after the Battle Cry block (after line 462).
**Pattern**: Follow Battle Cry at lines 438-462.

```rust
// CR 702.105a: Dethrone -- "Whenever this creature attacks the player
// with the most life or tied for most life, put a +1/+1 counter on
// this creature."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.105b).
// The trigger uses SelfAttacksPlayerWithMostLife, a dedicated event
// that is only dispatched in abilities.rs when the defending player
// has the most life (or is tied). This avoids unconditional firing
// on all attacks.
if matches!(kw, KeywordAbility::Dethrone) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacksPlayerWithMostLife,
        intervening_if: None,
        description: "Dethrone (CR 702.105a): Whenever this creature attacks \
                      the player with the most life or tied for most life, \
                      put a +1/+1 counter on this creature.".to_string(),
        effect: Some(Effect::AddCounter {
            target: EffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
            count: 1,
        }),
    });
}
```

**CR**: 702.105a -- "put a +1/+1 counter on this creature"

**Note on imports**: `TriggerEvent` is imported at the top of builder.rs from
`game_object.rs`. Verify the import includes the path to `TriggerEvent`. If it uses a
wildcard or specific import, no change needed since the variant is added to the existing
enum.

### Step 3: Trigger Wiring in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Location**: In the `GameEvent::AttackersDeclared` handler, inside the
`for (attacker_id, attack_target) in attackers` loop (starting at line 946).

**Placement**: After the existing `defending_player_id` assignment for `SelfAttacks`
triggers (line 962-964), BEFORE the Exalted block (line 967).

**Logic**:
```rust
// CR 702.105a: Dethrone -- "Whenever this creature attacks the player
// with the most life or tied for most life, put a +1/+1 counter on
// this creature."
// Only triggers when attacking a Player (not planeswalker/battle).
// CR 508.2a: condition checked at declaration time only.
if let crate::state::combat::AttackTarget::Player(def_pid) = attack_target {
    // Find the maximum life total among all active (non-eliminated) players.
    let defending_life = state.players.get(def_pid)
        .map(|p| p.life_total)
        .unwrap_or(i32::MIN);
    let max_life = state.players.values()
        .filter(|p| !p.has_lost && !p.has_conceded)
        .map(|p| p.life_total)
        .max()
        .unwrap_or(i32::MIN);

    if defending_life >= max_life {
        let pre_len_dethrone = triggers.len();
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::SelfAttacksPlayerWithMostLife,
            Some(*attacker_id),
            None,
        );
        // Tag dethrone triggers with defending player for consistency
        // with other attack triggers (e.g., annihilator).
        for t in &mut triggers[pre_len_dethrone..] {
            t.defending_player_id = defending_player;
        }
    }
}
```

**Key design points**:
- `defending_player` variable is already in scope from the `SelfAttacks` block above
  (resolved from `AttackTarget` at line 956-961).
- The `if let AttackTarget::Player(def_pid)` naturally excludes planeswalker attacks.
- `state.players` is an `OrdMap<PlayerId, PlayerState>` -- iteration is deterministic.
- The `.filter(|p| !p.has_lost && !p.has_conceded)` ensures eliminated players are
  excluded from the "most life" calculation.
- `defending_life >= max_life` handles both "sole highest" and "tied for highest" cases.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/dethrone.rs`
**Pattern**: Follow `/home/airbaggie/scutemob/crates/engine/tests/battle_cry.rs`

**Module header**:
```rust
//! Dethrone keyword ability tests (CR 702.105).
//!
//! Dethrone is a triggered ability: "Whenever this creature attacks the player
//! with the most life or tied for most life, put a +1/+1 counter on this creature."
//!
//! Key rules verified:
//! - Trigger fires when attacking the player with the most life (CR 702.105a).
//! - Trigger fires when attacking a player TIED for most life (CR 702.105a).
//! - The +1/+1 counter goes on the dethrone creature itself (CR 702.105a).
//! - Does NOT trigger when attacking a player who does not have most life (CR 702.105a).
//! - Does NOT trigger when attacking a planeswalker (ruling 2023-07-28).
//! - Multiple instances each trigger separately (CR 702.105b).
//! - Multiplayer: most life is compared among all active players (CR 702.105a).
```

**Imports**:
```rust
use mtg_engine::{
    calculate_characteristics, process_command, AttackTarget, CardRegistry, Command,
    CounterType, GameEvent, GameState, GameStateBuilder, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};
```

**Helpers** (copy from battle_cry.rs):
```rust
fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}
```

**Tests to write** (8 tests):

1. **`test_dethrone_basic_attacks_player_with_most_life`** -- CR 702.105a
   - 2 players: P1 at 20, P2 at 25 (most life).
   - P1 has 2/2 creature with Dethrone on battlefield.
   - P1 declares attacker targeting P2.
   - Assert: `AbilityTriggered` event from dethrone source.
   - Assert: `state.stack_objects.len() == 1`.
   - Pass all to resolve trigger.
   - Assert: creature has 1 `PlusOnePlusOne` counter.
   - Assert: `calculate_characteristics` shows power=3, toughness=3.

2. **`test_dethrone_tied_for_most_life`** -- CR 702.105a ("or tied for most life")
   - 2 players: P1 at 20, P2 at 20 (tied).
   - P1 attacks P2 with dethrone creature (2/2).
   - Assert: trigger fires, counter placed after resolution.
   - Assert: creature P/T = 3/3.

3. **`test_dethrone_does_not_trigger_against_lower_life`** -- CR 702.105a (negative)
   - 2 players: P1 at 25 (most), P2 at 15.
   - P1 attacks P2.
   - Assert: NO `AbilityTriggered` from dethrone source.
   - Assert: `state.stack_objects.is_empty()`.
   - Assert: creature has 0 counters.

4. **`test_dethrone_multiplayer_four_player_most_life`** -- CR 702.105a, multiplayer
   - 4 players: P1=30, P2=40, P3=40, P4=20.
   - P1 attacks P2 (tied for most at 40).
   - Assert: trigger fires, creature gets +1/+1 counter.

5. **`test_dethrone_multiplayer_not_most_life`** -- CR 702.105a (negative, multiplayer)
   - 4 players: P1=30, P2=25, P3=40, P4=35.
   - P1 attacks P2 (25, not most -- P3 has 40).
   - Assert: no trigger fires, no counter.

6. **`test_dethrone_multiple_instances_trigger_separately`** -- CR 702.105b
   - 2 players: P1 at 20, P2 at 25.
   - P1 has creature with TWO Dethrone keywords (via `.with_keyword` twice).
   - P1 attacks P2.
   - Assert: `state.stack_objects.len() == 2`.
   - Resolve both (pass_all twice).
   - Assert: creature has 2 `PlusOnePlusOne` counters.
   - Assert: P/T = base+2/base+2.

7. **`test_dethrone_does_not_trigger_on_planeswalker_attack`** -- ruling 2023-07-28
   - 2 players: P2 at 25 (most life), P2 controls a planeswalker on battlefield.
   - P1 attacks the planeswalker (AttackTarget::Planeswalker(pw_id)).
   - Assert: no dethrone trigger fires.
   - **Note**: Create the planeswalker as `ObjectSpec::card(p2, "Test Planeswalker")`
     with card type Planeswalker, in_zone Battlefield. If engine does not yet support
     planeswalker creation in tests, defer this test with a `#[ignore]` annotation and
     a comment explaining the gap.

8. **`test_dethrone_attacker_has_most_life_attacks_lower`** -- CR 702.105a (self-life edge)
   - 2 players: P1 at 30 (most), P2 at 20.
   - P1 attacks P2 with dethrone creature.
   - Assert: NO trigger (P2 does not have most life).
   - This test confirms that the attacker having the most life does not matter -- only
     the DEFENDING player's life total relative to the global max matters.

### Step 5: Card Definition (later phase)

**Suggested card**: Marchesa's Emissary
- **Name**: Marchesa's Emissary
- **Cost**: {3}{U}
- **Type**: Creature -- Human Rogue
- **P/T**: 2/2
- **Oracle**: "Hexproof\nDethrone"
- **Keywords**: `[KeywordAbility::Hexproof, KeywordAbility::Dethrone]`
- **Color identity**: ["U"]

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`
**Action**: Use `card-definition-author` agent.

**Why this card**: Simple creature with Dethrone + Hexproof (both already validated
keywords). No complex triggered/activated abilities beyond the keyword auto-generation.
Good for validating dethrone end-to-end in a game script.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Dethrone trigger in 4-player Commander"
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Sequence number**: Use next available in the combat directory.

**Script outline**:
1. 4 players: P1 (30 life), P2 (40 life), P3 (40 life), P4 (25 life).
2. P1 controls Marchesa's Emissary (2/2, Hexproof, Dethrone) on battlefield.
3. Advance to Declare Attackers step.
4. P1 declares Marchesa's Emissary attacking P2 (tied for most life at 40).
5. Assert: stack has 1 trigger item.
6. All 4 players pass priority -- trigger resolves.
7. Assert: Marchesa's Emissary has 1 +1/+1 counter.
8. Assert: Marchesa's Emissary effective P/T is 3/3.

**File**: Use `game-script-generator` agent.

## Interactions to Watch

- **Dethrone + Counters**: The +1/+1 counter is a real persistent counter (not a
  temporary continuous effect like Battle Cry's +1/+0). It persists across turns via
  `Effect::AddCounter` which modifies `obj.counters` directly.
- **Dethrone + Humility**: Pre-existing concern for all triggered keywords (Battle Cry,
  Exalted, Afterlife, etc.). `collect_triggers_for_event` reads
  `obj.characteristics.triggered_abilities` which are the raw printed characteristics, not
  the layer-calculated result. This means Humility may not suppress triggered keyword
  abilities. Not blocking -- this is a systemic issue to address separately.
- **Dethrone + Life total changes during resolution**: Per ruling 2014-05-29, once the
  trigger fires, life totals are irrelevant at resolution. The engine handles this
  correctly because there is NO `InterveningIf` on the trigger -- the condition is checked
  purely at trigger-collection time in the `AttackersDeclared` handler.
- **Dethrone + Planeswalker attacks**: The `if let AttackTarget::Player(def_pid)` pattern
  in Step 3 naturally excludes planeswalker attacks.
- **Dethrone + Eliminated players**: The `.filter(|p| !p.has_lost && !p.has_conceded)`
  ensures eliminated players are excluded from the "most life" calculation.
- **Dethrone + Persist/Undying counter interaction**: If a creature has both Dethrone and
  Persist, the +1/+1 counter from dethrone counts as "has +1/+1 counters" for Undying
  (would prevent undying) but not for Persist (which checks -1/-1 counters). This is
  handled correctly by the existing counter infrastructure -- no special interaction needed.

## Summary of Files Modified

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Dethrone` variant |
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::SelfAttacksPlayerWithMostLife` variant |
| `crates/engine/src/state/hash.rs` | Add hash arms for both new variants (discriminants 55 and 14) |
| `crates/engine/src/state/builder.rs` | Auto-generate `TriggeredAbilityDef` for Dethrone keyword |
| `crates/engine/src/rules/abilities.rs` | Dethrone trigger collection in `AttackersDeclared` handler |
| `tools/replay-viewer/src/view_model.rs` | Add `format_keyword` arm for Dethrone |
| `crates/engine/tests/dethrone.rs` | New test file with 8 tests |
| `crates/engine/src/cards/definitions.rs` | Marchesa's Emissary card definition (Step 5) |
| `test-data/generated-scripts/combat/` | Dethrone game script (Step 6) |
