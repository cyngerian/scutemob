# Ability Plan: Afflict

**Generated**: 2026-03-01
**CR**: 702.130
**Priority**: P4
**Similar abilities studied**: Annihilator (CR 702.86) — `types.rs:291-297`, `builder.rs:419-437`, `abilities.rs:1331-1350`, `hash.rs:363-367`, `tests/annihilator.rs`

## CR Rule Text

702.130. Afflict

702.130a Afflict is a triggered ability. "Afflict N" means "Whenever this creature becomes blocked, defending player loses N life."

702.130b If a creature has multiple instances of afflict, each triggers separately.

### Related CR: "Becomes blocked" triggers (CR 509.3c)

509.3c An ability that reads "Whenever [a creature] becomes blocked, . . ." generally triggers only once each combat for that creature, even if it's blocked by multiple creatures. It will trigger if that creature becomes blocked by at least one creature declared as a blocker. It will also trigger if that creature becomes blocked by an effect or by a creature that's put onto the battlefield as a blocker, but only if the attacking creature was an unblocked creature at that time. (See rule 509.1h.)

### Related CR: Defending player identity (CR 509.1h, CR 508.5)

509.1h An attacking creature with one or more creatures declared as blockers for it becomes a blocked creature; one with no creatures declared as blockers for it becomes an unblocked creature.

If a creature is attacking a planeswalker, that planeswalker's controller is the defending player. If a creature is attacking a battle, that battle's protector is the defending player.

## Key Edge Cases

1. **Multiple blockers, single trigger (CR 509.3c)**: If 3 creatures block an attacker with afflict, afflict triggers only once. The trigger fires per "becomes blocked" event, not per blocker.
2. **Life loss, not damage (rulings 2017-07-14)**: Afflict causes the defending player to lose life; it is not damage or combat damage. This means:
   - It cannot be prevented by damage prevention effects.
   - It does not count as combat damage for commander damage tracking.
   - It does not trigger lifelink.
   - It DOES count as life loss for effects like Neheb, the Eternal.
3. **Resolves before combat damage (rulings 2017-07-14)**: Afflict triggers go on the stack during the declare blockers step. They resolve before combat damage is dealt. If the life loss kills a player, they lose immediately (a blocker's lifelink combat damage won't save them).
4. **Multiple instances trigger separately (CR 702.130b)**: A creature with Afflict 2 and Afflict 3 generates two separate triggers, causing the defending player to lose 2 + 3 = 5 life total.
5. **Planeswalker attack (rulings 2023-07-28)**: If a creature with afflict attacks a planeswalker, the defending player is the planeswalker's controller.
6. **Multiplayer**: Each afflict trigger must target the correct defending player for that specific attacker. In a 4-player game, P1 might attack P2 and P3 with different creatures, each having afflict -- each trigger must target the right defender.
7. **Unblocked creature**: If no creatures block the afflict creature, afflict does NOT trigger (it only triggers on "becomes blocked").

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement (trigger generation in builder.rs)
- [ ] Step 3: Trigger wiring (BlockersDeclared handler in abilities.rs)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Afflict(u32)` variant after `Ingest` (line ~638).
**Pattern**: Follow `KeywordAbility::Annihilator(u32)` at line 297 — same parameterized keyword pattern.

```rust
/// CR 702.130: Afflict N -- triggered ability.
/// "Whenever this creature becomes blocked, defending player loses N life."
/// CR 702.130b: Multiple instances trigger separately.
///
/// Implemented as a triggered ability. builder.rs auto-generates a
/// TriggeredAbilityDef from this keyword at object-construction time.
/// The trigger fires on SelfBecomesBlocked events; the defending player
/// is resolved at flush time via PendingTrigger.defending_player_id.
Afflict(u32),
```

**Discriminant 80** (per batch context: Flanking=76, Bushido=77, Rampage=78, Provoke=79, Afflict=80).

### Step 1a: TriggerEvent variant

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `TriggerEvent::SelfBecomesBlocked` variant after `ControllerProliferates` (line ~198).
**CR**: 509.3c — "Whenever [a creature] becomes blocked" trigger condition.

```rust
/// CR 509.3c / CR 702.130a: Triggers when this attacking creature becomes
/// a blocked creature (at least one blocker is declared for it).
/// Fires only once per combat per creature, even if multiple creatures block it.
/// Used by the Afflict keyword.
SelfBecomesBlocked,
```

### Step 1b: Hash — KeywordAbility

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `KeywordAbility::Afflict(n)` after Ingest (line ~468).
**Pattern**: Follow `KeywordAbility::Annihilator(n)` at line 363-367.

```rust
// Afflict (discriminant 80) -- CR 702.130
KeywordAbility::Afflict(n) => {
    80u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

### Step 1c: Hash — TriggerEvent

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arm for `TriggerEvent::SelfBecomesBlocked` in the TriggerEvent match.
**Note**: Find the existing TriggerEvent hash match and add a new discriminant. Check the current last discriminant number for TriggerEvent.

```rust
TriggerEvent::SelfBecomesBlocked => <next_discriminant>u8.hash_into(hasher),
```

### Step 1d: View Model — keyword display

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add display arm for `KeywordAbility::Afflict(n)` after Ingest (line ~687).
**Pattern**: Follow `KeywordAbility::Annihilator(n)` display format.

```rust
KeywordAbility::Afflict(n) => format!("Afflict {n}"),
```

### Step 1e: Match arm exhaustiveness

**Action**: Grep for exhaustive `KeywordAbility` match arms and add the new `Afflict(n)` arm.
Files to check:
- `crates/engine/src/state/hash.rs` (KeywordAbility match) -- covered in Step 1b
- `tools/replay-viewer/src/view_model.rs` (keyword display) -- covered in Step 1d
- Any other files with exhaustive matches on KeywordAbility

### Step 2: Trigger Generation in builder.rs

**File**: `crates/engine/src/state/builder.rs`
**Action**: Add triggered ability generation for `KeywordAbility::Afflict(n)` after the Annihilator block (line ~437).
**Pattern**: Follow `KeywordAbility::Annihilator(n)` at line 424-437.
**CR**: 702.130a — "Afflict N means 'Whenever this creature becomes blocked, defending player loses N life.'"

```rust
// CR 702.130a: Afflict N -- "Whenever this creature becomes blocked,
// defending player loses N life."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.130b).
// The effect targets the defending player (DeclaredTarget { index: 0 }),
// which is resolved at flush time via PendingTrigger.defending_player_id.
if let KeywordAbility::Afflict(n) = kw {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfBecomesBlocked,
        intervening_if: None,
        description: format!(
            "Afflict {n} (CR 702.130a): Whenever this creature becomes blocked, \
             defending player loses {n} life."
        ),
        effect: Some(Effect::LoseLife {
            player: PlayerTarget::DeclaredTarget { index: 0 },
            amount: EffectAmount::Fixed(*n as i32),
        }),
    });
}
```

### Step 3: Trigger Wiring — BlockersDeclared handler

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Extend the `GameEvent::BlockersDeclared` handler (line ~1444) to also collect `SelfBecomesBlocked` triggers for each attacker that has at least one blocker declared for it.

**CR**: 509.3c — triggers once per "becomes blocked" event per creature (deduplicate by attacker ID). 509.1h — an attacking creature with one or more creatures declared as blockers for it becomes a blocked creature.

**Pattern**: Similar to how the `AttackersDeclared` handler collects `SelfAttacks` triggers and tags `defending_player_id`.

```rust
GameEvent::BlockersDeclared {
    blockers,
    defending_player,
} => {
    // SelfBlocks: fires on each creature that is blocking (CR 603.5).
    for (blocker_id, _) in blockers {
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::SelfBlocks,
            Some(*blocker_id),
            None,
        );
    }

    // CR 509.3c / CR 702.130a: SelfBecomesBlocked triggers fire once per
    // attacker that becomes blocked. Deduplicate attacker IDs so that an
    // attacker blocked by multiple creatures only triggers once.
    let mut blocked_attackers: Vec<ObjectId> = blockers
        .iter()
        .map(|(_, attacker_id)| *attacker_id)
        .collect();
    blocked_attackers.sort();
    blocked_attackers.dedup();

    for attacker_id in blocked_attackers {
        let pre_len = triggers.len();
        collect_triggers_for_event(
            state,
            &mut triggers,
            TriggerEvent::SelfBecomesBlocked,
            Some(attacker_id),
            None,
        );
        // Tag each new trigger with the defending player ID so that
        // flush_pending_triggers sets Target::Player at index 0,
        // resolving PlayerTarget::DeclaredTarget { index: 0 } to the
        // correct defending player for the LoseLife effect.
        for t in &mut triggers[pre_len..] {
            t.defending_player_id = Some(*defending_player);
        }
    }
}
```

**Key implementation detail**: The `defending_player_id` flow is already used by Annihilator. When `flush_pending_triggers` processes a trigger with `defending_player_id = Some(dp)`, it sets `Target::Player(dp)` at index 0 (line ~2031-2038 in abilities.rs). The `Effect::LoseLife { player: PlayerTarget::DeclaredTarget { index: 0 }, ... }` then resolves to the correct defending player. No changes needed to `flush_pending_triggers`.

**No custom StackObjectKind needed**: Afflict uses the standard `TriggeredAbility` stack kind with the `defending_player_id` target mechanism. The `LoseLife` effect is already fully implemented in `effects/mod.rs:319`. This is the same approach used by Annihilator.

### Step 4: Unit Tests

**File**: `crates/engine/tests/afflict.rs`

**Tests to write**:

1. `test_702_130a_afflict_basic_life_loss` -- Creature with Afflict 2 becomes blocked. Defending player loses 2 life. Verify: trigger fires (AbilityTriggered event), life total decreases by N after resolution.
   **CR**: 702.130a

2. `test_702_130a_afflict_not_blocked_no_trigger` -- Creature with Afflict attacks, no blockers declared. Afflict does NOT trigger. Stack is empty after blockers step.
   **CR**: 702.130a, 509.3c

3. `test_509_3c_afflict_multiple_blockers_single_trigger` -- Creature with Afflict 3 is blocked by 2 creatures. Afflict triggers only once (defending player loses 3 life, not 6).
   **CR**: 509.3c, rulings 2017-07-14

4. `test_702_130b_afflict_multiple_instances_trigger_separately` -- Creature with Afflict 2 AND Afflict 1. When blocked, two separate triggers fire. After both resolve, defending player loses 3 life total.
   **CR**: 702.130b

5. `test_afflict_multiplayer_correct_defending_player` -- 4-player game. P1 attacks P2 and P3 with two different afflict creatures. Only the correct defending player for each attacker loses life.
   **CR**: 508.5, 702.130a

6. `test_afflict_life_loss_not_damage` -- Afflict creature becomes blocked. Verify life loss is NOT damage (does not trigger lifelink-like effects, does not count as combat damage). Verify by checking that the life total decreases but no DamageDealt event is emitted.

**Pattern**: Follow `crates/engine/tests/annihilator.rs` structure:
- `find_object` helper
- `pass_all` helper (pass priority for all players)
- Start at `Step::DeclareAttackers`, declare attackers, then declare blockers to trigger afflict
- Verify trigger on stack, then resolve and check life totals

**Test flow** (differs from Annihilator because Afflict fires at declare blockers, not declare attackers):
1. Build state at `Step::DeclareAttackers`
2. `DeclareAttackers` command (creature attacks P2)
3. Pass priority to advance to `DeclareBlockers` step
4. `DeclareBlockers` command (P2 declares blockers) -- afflict trigger fires here
5. Pass priority to resolve the afflict trigger
6. Assert defending player's life decreased by N

### Step 5: Card Definition (later phase)

**Suggested card**: Khenra Eternal
- Name: Khenra Eternal
- Cost: {1}{B}
- Type: Creature -- Zombie Jackal Warrior
- P/T: 2/2
- Abilities: Afflict 1
- Simple, clean creature with only Afflict -- ideal for testing
- **Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Khenra Eternal attacks, gets blocked, afflict trigger fires and resolves, defending player loses 1 life before combat damage.
**Subsystem directory**: `test-data/generated-scripts/combat/`
**Script name pattern**: `114_khenra_eternal_afflict_blocked.json`

## Interactions to Watch

1. **Defending player resolution in multiplayer**: The `defending_player` field on `BlockersDeclared` identifies which player is blocking. This must be propagated to `PendingTrigger.defending_player_id` so `flush_pending_triggers` sets the correct target. The existing infrastructure (used by Annihilator/Myriad) handles this via the `defending_player_id` -> `Target::Player` pipeline.

2. **Trigger timing**: Afflict triggers go on the stack at declare-blockers time (CR 509.2a). They are checked and queued inside `handle_declare_blockers` (line ~647 in combat.rs), which calls `abilities::check_triggers(state, &events)` and pushes results to `state.pending_triggers`. These are flushed when priority is next granted.

3. **No interaction with combat damage**: Afflict's life loss is independent of combat damage. The creature might deal 0 combat damage (all absorbed by blockers), but afflict still causes the defending player to lose N life.

4. **LoseLife effect**: Already fully implemented at `effects/mod.rs:319`. Takes `PlayerTarget` and `EffectAmount`, clamps negative amounts, emits `LifeChanged` event. No new effect code needed.

5. **Protection**: Protection from a quality prevents blocking (DEBT: B = Blocking). If a creature has protection and cannot be blocked, afflict never triggers. This is already enforced by declare-blockers validation in `combat.rs`.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Afflict(u32)` |
| `crates/engine/src/state/game_object.rs` | Add `TriggerEvent::SelfBecomesBlocked` |
| `crates/engine/src/state/hash.rs` | Add hash arms for both new variants |
| `crates/engine/src/state/builder.rs` | Generate TriggeredAbilityDef for Afflict |
| `crates/engine/src/rules/abilities.rs` | Extend BlockersDeclared handler for SelfBecomesBlocked |
| `tools/replay-viewer/src/view_model.rs` | Add display for Afflict(n) |
| `crates/engine/tests/afflict.rs` | New test file: 6 tests |

**No new StackObjectKind needed.** Afflict reuses the standard TriggeredAbility path.
**No new Effect needed.** `Effect::LoseLife` already exists.
**No changes to `flush_pending_triggers` needed.** The `defending_player_id` -> `Target::Player` pipeline already exists (used by Annihilator).
