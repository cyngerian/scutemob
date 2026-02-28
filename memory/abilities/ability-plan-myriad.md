# Ability Plan: Myriad

**Generated**: 2026-02-28
**CR**: 702.116
**Priority**: P3
**Similar abilities studied**: Annihilator (KeywordAbility::Annihilator, SelfAttacks trigger, defending_player_id tagging — `crates/engine/src/state/types.rs:277-283`, `crates/engine/src/rules/abilities.rs:1157-1220`, `crates/engine/tests/annihilator.rs`), BattleCry (SelfAttacks + ForEach — `crates/engine/src/state/builder.rs:444-459`), LivingWeapon (CreateTokenAndAttachSource — `crates/engine/src/state/builder.rs:475-500`), copy.rs Layer 1 CopyOf (`crates/engine/src/rules/copy.rs:435-456`)

## CR Rule Text

```
702.116. Myriad

702.116a Myriad is a triggered ability that may also create a delayed triggered
ability. "Myriad" means "Whenever this creature attacks, for each opponent other
than defending player, you may create a token that's a copy of this creature that's
tapped and attacking that player or a planeswalker they control. If one or more
tokens are created this way, exile the tokens at end of combat."

702.116b If a creature has multiple instances of myriad, each triggers separately.
```

### Related Rules

- **CR 603.7**: Delayed triggered abilities — created during resolution; trigger once at specified event.
- **CR 603.7b**: Delayed triggered ability triggers only once (the next time its event occurs) unless it has a stated duration.
- **CR 603.7e**: Source of the delayed trigger is the same as the source of the triggered ability that created it.
- **CR 511.2**: "Abilities that trigger 'at end of combat' trigger as the end of combat step begins."
- **CR 511.3**: "As soon as the end of combat step ends, all creatures, battles, and planeswalkers are removed from combat."
- **CR 508.5**: "Defending player" = the player the creature was attacking when it became an attacker.

## Key Edge Cases

From CR children and rulings:

1. **Tokens enter tapped and attacking** — they are NOT declared as attackers. "Whenever a creature attacks" triggers do NOT fire on the tokens (including the myriad ability itself on the tokens). Attack costs do not apply.
2. **Each token is a copy** of the printed creature (Layer 1 CopyOf). Does NOT copy tapped/untapped status, counters, Auras, Equipment, or non-copy effects that changed P/T/types/color.
3. **ETB abilities DO trigger** — "Any enters-the-battlefield abilities of the copied creature will trigger when the token enters the battlefield."
4. **All tokens enter simultaneously** — not sequentially.
5. **2-player game (1 opponent)**: If the defending player is the only opponent, no tokens are created.
6. **Multiplayer (Commander)**: For each opponent OTHER than the defending player, create one token tapped and attacking that player.
7. **Multiple instances trigger separately** (CR 702.116b) — if a creature has myriad twice, two separate triggers fire, creating two tokens per opponent.
8. **Legendary rule**: If the creature is legendary, all tokens enter (they are copies, also legendary), then the legend rule applies — owner keeps one, others go to graveyard. Dies triggers fire.
9. **Exile at end of combat**: This is a delayed triggered ability (CR 603.7). It triggers as the end of combat step begins (CR 511.2). If the tokens have already left the battlefield, the delayed trigger does nothing (CR 603.7c).
10. **"You may create"** — myriad is optional ("you may"). The player chooses whether to create tokens for each opponent.
11. **Token copies that are not creatures** (e.g., animated land) enter but are not attacking. They are still exiled at end of combat.
12. **Planeswalker attack option**: For each opponent, the controller may choose to have the token attack that player OR a planeswalker that player controls.

### Simplifications for V1

- **Always create tokens for all eligible opponents** (auto-accept "you may") — no player choice command yet. Future: add a ChooseMyriadTargets command variant.
- **Tokens always attack the player directly** (not planeswalkers) — simplifies target resolution. Future: allow planeswalker attack targets.
- **Delayed trigger implementation**: The engine's `DelayedTrigger` is still a stub (only `source: ObjectId`). Myriad needs to track which token ObjectIds to exile at end of combat. Two approaches:
  - (A) Expand `DelayedTrigger` with a `token_ids: Vec<ObjectId>` and condition/effect fields.
  - (B) Tag myriad tokens on the `GameObject` (e.g., `myriad_exile_at_eoc: bool`) and exile all tagged tokens in the end-of-combat TBA.

  **Decision: Use approach (B)** — simpler, avoids overhauling the DelayedTrigger stub, and is more robust against zone changes. Add a field `myriad_exile_at_eoc: bool` to `GameObject`. In `end_combat()` (turn_actions.rs), before clearing combat state, exile all battlefield objects where `myriad_exile_at_eoc == true && is_token == true`.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Myriad` variant after `Infect` (line ~520)
**Pattern**: Follow `KeywordAbility::Infect` at line 520

```rust
/// CR 702.116: Myriad -- "Whenever this creature attacks, for each opponent
/// other than defending player, you may create a token that's a copy of this
/// creature that's tapped and attacking that player or a planeswalker they
/// control. If one or more tokens are created this way, exile the tokens at
/// end of combat."
///
/// Triggered ability. builder.rs auto-generates a TriggeredAbilityDef from
/// this keyword at object-construction time. Multiple instances each trigger
/// separately (CR 702.116b).
Myriad,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add hash discriminant `64u8` for `KeywordAbility::Myriad` after Infect (line ~435)
**Pattern**: Follow `KeywordAbility::Infect => 63u8.hash_into(hasher),`

```rust
// Myriad (discriminant 64) -- CR 702.116
KeywordAbility::Myriad => 64u8.hash_into(hasher),
```

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add display arm for `KeywordAbility::Myriad` in the `keyword_display` function (after line ~657)
**Pattern**: Follow `KeywordAbility::Infect => "Infect".to_string(),`

```rust
KeywordAbility::Myriad => "Myriad".to_string(),
```

### Step 2: GameObject Field for Myriad Token Tracking

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `myriad_exile_at_eoc: bool` field to `GameObject` struct (after `was_unearthed`, line ~312)
**CR**: 702.116a — "exile the tokens at end of combat"

```rust
/// CR 702.116a: True for myriad token copies that must be exiled at end of combat.
/// Set when the token is created by myriad. Checked in end_combat() turn-based action.
pub myriad_exile_at_eoc: bool,
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `self.myriad_exile_at_eoc.hash_into(hasher);` to the `GameObject` `HashInto` impl (after `was_unearthed`, around line ~580)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs`
**Action**: Initialize `myriad_exile_at_eoc: false` in both `move_object_to_zone` sites (lines ~269 and ~354) and in `add_object` (wherever objects are constructed with default values)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Initialize `myriad_exile_at_eoc: false` in all `ObjectSpec` constructors and in `GameStateBuilder::build()`'s object construction (same pattern as `was_unearthed: false`)

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Initialize `myriad_exile_at_eoc: false` in `make_token()` function (line ~2272, after `was_unearthed: false`)

### Step 3: Builder Trigger Wiring

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: In the keyword-to-trigger translation loop (around line 420-600), add a block for `KeywordAbility::Myriad` that generates a `TriggeredAbilityDef` with `TriggerEvent::SelfAttacks`.
**Pattern**: Follow `KeywordAbility::Annihilator(n)` at line 424-437
**CR**: 702.116a — triggered ability fires whenever this creature attacks

```rust
// CR 702.116a: Myriad -- "Whenever this creature attacks, for each
// opponent other than defending player, you may create a token that's
// a copy of this creature that's tapped and attacking that player."
// Each keyword instance generates one TriggeredAbilityDef (CR 702.116b).
// The effect is handled specially in abilities.rs via a MyriadTrigger
// marker (not a standard Effect).
if matches!(kw, KeywordAbility::Myriad) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfAttacks,
        intervening_if: None,
        description: "Myriad (CR 702.116a): Whenever this creature attacks, \
                      for each opponent other than defending player, create a \
                      token copy tapped and attacking that player. Exile tokens \
                      at end of combat.".to_string(),
        effect: None, // Special handling in abilities.rs
    });
}
```

**Note**: The `effect: None` is intentional. Myriad's effect is complex (create N token copies, each as a copy of the source, tapped, attacking different players, tagged for EOC exile). This cannot be expressed with the existing `Effect` enum. Instead, we add a `is_myriad_trigger: bool` flag to `PendingTrigger` and handle it specially during `flush_pending_triggers`.

### Step 4: PendingTrigger Myriad Flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/stubs.rs`
**Action**: Add `is_myriad_trigger: bool` field to `PendingTrigger` (after `defending_player_id`, line ~90)
**Pattern**: Follow `is_evoke_sacrifice: bool` at line 81

```rust
/// CR 702.116a: If true, this pending trigger is a myriad trigger.
///
/// When flushed to the stack, myriad creates token copies of the source
/// creature for each opponent other than the defending player, each tapped
/// and attacking that opponent. The defending player is read from
/// `defending_player_id`. Uses the `source` field for the creature to copy.
#[serde(default)]
pub is_myriad_trigger: bool,
```

### Step 5: Trigger Collection in abilities.rs

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In the `AttackersDeclared` handler (around line 1165-1221), after the existing `SelfAttacks` trigger collection, the myriad triggers need to be tagged with `is_myriad_trigger = true`. The tagging already happens via `defending_player_id` for all `SelfAttacks` triggers.

To distinguish myriad triggers, check whether the trigger's `TriggeredAbilityDef` has `effect: None` and description contains "Myriad", OR (better) add logic in `flush_pending_triggers` to check the keyword directly.

**Better approach**: In `collect_triggers_for_event`, when a `SelfAttacks` trigger is collected and the ability description matches myriad OR the triggering ability uses effect `None`, tag `is_myriad_trigger = true` on the pending trigger. But the cleanest approach is:

In `check_triggers`, inside the `AttackersDeclared` handler, after `collect_triggers_for_event(SelfAttacks)` runs and triggers are tagged with `defending_player_id`, additionally check each new trigger: if the `TriggeredAbilityDef` at `ability_index` has `effect: None` AND description starts with "Myriad", set `is_myriad_trigger = true`.

```rust
// Tag myriad triggers (CR 702.116a)
for t in &mut triggers[pre_len..] {
    if let Some(obj) = state.objects.get(&t.source) {
        if let Some(ta) = obj.characteristics.triggered_abilities.get(t.ability_index) {
            if ta.effect.is_none() && ta.description.starts_with("Myriad") {
                t.is_myriad_trigger = true;
            }
        }
    }
}
```

### Step 6: Myriad Resolution in flush_pending_triggers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers`, add a branch for `is_myriad_trigger == true` that:

1. Determines the source creature's ObjectId and the defending player from `defending_player_id`.
2. Finds all opponents of the source's controller who are NOT the defending player and are still alive.
3. For each such opponent, creates a token copy:
   a. Create a new `GameObject` via `make_token`-like logic OR create a blank token and apply a `CopyOf` continuous effect.
   b. Set `is_token = true`, `status.tapped = true`, `myriad_exile_at_eoc = true`.
   c. Add the token to `ZoneId::Battlefield`.
   d. Register a `CopyOf` continuous effect (Layer 1) pointing at the source creature.
   e. Add the token as an attacker in `state.combat.attackers` targeting that opponent.
4. Emit `TokenCreated` + `PermanentEnteredBattlefield` events for each token.
5. Do NOT put a stack object for the myriad trigger (it resolves immediately as a special action).

**Wait** -- actually, myriad IS a triggered ability that goes on the stack. The tokens are created when the trigger resolves. Let me reconsider.

**Revised approach**: Myriad goes on the stack like any other triggered ability (via `StackObjectKind::TriggeredAbility` or a new `StackObjectKind::MyriadTrigger`). When it resolves (in `resolution.rs`), the engine creates the token copies.

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers`, add a `is_myriad_trigger` check that creates a `StackObjectKind::MyriadTrigger` variant (or uses the standard `TriggeredAbility` with a custom effect). Since the effect is complex, use a new `StackObjectKind::MyriadTrigger { source_creature: ObjectId }`.

Actually, the simplest approach following existing patterns: use `StackObjectKind::TriggeredAbility` as normal. The ability has `effect: None`. At resolution time in `resolution.rs`, when an ability with no effect resolves, check if it's a myriad trigger (via a flag on the stack object or by checking the source's keywords). If so, execute the myriad token creation.

**Cleanest approach**: Add a new `StackObjectKind::MyriadTrigger` variant similar to `EvokeSacrificeTrigger`.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs` (or wherever `StackObjectKind` is defined)
**Action**: Check where `StackObjectKind` is. Add `MyriadTrigger` variant.

Let me verify.

**Verification needed**: Search for `StackObjectKind` definition.

### Step 6 (revised): Stack Object + Resolution

#### Step 6a: StackObjectKind Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs` (search for `StackObjectKind`)
**Action**: Add `MyriadTrigger` variant

```rust
/// CR 702.116a: Myriad trigger resolution.
/// When this resolves, create token copies of the source creature for each
/// opponent other than the defending player.
MyriadTrigger,
```

#### Step 6b: flush_pending_triggers

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers`, when `is_myriad_trigger` is true, create a stack object with `StackObjectKind::MyriadTrigger` instead of `StackObjectKind::TriggeredAbility`. Store the defending_player_id as a target on the stack object (via `Target::Player(defending_player_id)`).
**Pattern**: Follow `is_evoke_sacrifice` handling in `flush_pending_triggers` (search for `EvokeSacrificeTrigger`)

#### Step 6c: Resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add a match arm for `StackObjectKind::MyriadTrigger` in the resolution logic. When this resolves:

1. Get the source creature's ObjectId from the stack object.
2. Get the defending player from the stack object's targets (`Target::Player`).
3. Get the source creature's controller.
4. Find all opponents of the controller who are NOT the defending player and are alive.
5. For each such opponent:
   a. Create a blank token `GameObject` (via `make_token`-like helper or minimal construction) with `is_token = true`, `status.tapped = true`, `myriad_exile_at_eoc = true`, `has_summoning_sickness = true`.
   b. Add it to `ZoneId::Battlefield` via `state.add_object()`.
   c. Apply a `CopyOf` continuous effect (Layer 1) from `copy::create_copy_effect()` to copy the source creature.
   d. Add it to `state.combat.attackers` as attacking that opponent (`AttackTarget::Player(opponent_id)`).
   e. Emit `TokenCreated` and `PermanentEnteredBattlefield` events.
6. Return the events.

**CR**: 702.116a — tokens enter tapped and attacking, are copies of the source

### Step 7: End-of-Combat Exile

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs`
**Action**: In `end_combat()` (line 473-476), BEFORE clearing combat state (`state.combat = None`), exile all myriad tokens:

```rust
fn end_combat(state: &mut GameState) -> Vec<GameEvent> {
    // CR 702.116a: Exile myriad tokens at end of combat.
    // CR 511.2: "at end of combat" triggers fire as the end of combat step begins.
    // We implement this as a turn-based action rather than a delayed trigger for simplicity.
    let myriad_tokens: Vec<ObjectId> = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_token && obj.myriad_exile_at_eoc)
        .map(|obj| obj.id)
        .collect();

    let mut events = Vec::new();
    for token_id in myriad_tokens {
        if let Ok(new_id) = state.move_object_to_zone(token_id, ZoneId::Exile) {
            events.push(GameEvent::ObjectExiled {
                object_id: token_id,
                new_id,
            });
        }
    }

    state.combat = None;
    events.push(GameEvent::CombatEnded);
    events
}
```

**Note**: Check if `GameEvent::ObjectExiled` exists. If not, use the existing zone-change event pattern.

### Step 8: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/myriad.rs`
**Tests to write**:

**Test 1: `test_myriad_basic_creates_token_copies_in_4_player`**
- CR 702.116a: 4-player game. P1 attacks P2 with myriad creature. Tokens are created tapped and attacking P3 and P4. After trigger resolves, verify 2 token copies on battlefield, each tapped, each registered as attackers in combat state.
- Pattern: Follow `test_annihilator_basic_sacrifice_on_attack` in `annihilator.rs`

**Test 2: `test_myriad_2_player_no_tokens_created`**
- CR 702.116a / ruling: "If the defending player is your only opponent, no tokens are put onto the battlefield." 2-player game. P1 attacks P2 with myriad. Trigger fires but creates no tokens.

**Test 3: `test_myriad_tokens_exiled_at_end_of_combat`**
- CR 702.116a: After combat (advancing to EndOfCombat step), myriad tokens are exiled. Verify tokens leave battlefield and are in exile zone.

**Test 4: `test_myriad_tokens_are_copies_of_source`**
- Verify that myriad tokens have the same characteristics (name, P/T, types) as the source creature via `calculate_characteristics`. They should have a `CopyOf` continuous effect.

**Test 5: `test_myriad_tokens_do_not_retrigger_myriad`**
- Ruling: "Abilities that trigger whenever a creature attacks won't trigger [on the tokens], including the myriad ability of the tokens." Tokens enter attacking but are NOT declared as attackers. No additional myriad triggers fire.

**Test 6: `test_myriad_multiple_instances_trigger_separately`**
- CR 702.116b: A creature with two instances of myriad creates two tokens per eligible opponent. In a 4-player game, this means 4 tokens total (2 for P3, 2 for P4).

**Test 7: `test_myriad_multiplayer_correct_opponents_targeted`**
- 4-player game: P1 attacks P3 with myriad. Tokens attack P2 and P4 (the opponents other than the defending player P3). Verify combat.attackers has the correct targets.

**Test 8: `test_myriad_token_has_myriad_exile_flag`**
- Verify that created tokens have `myriad_exile_at_eoc == true` and `is_token == true`.

**Pattern**: Follow tests in `crates/engine/tests/annihilator.rs` for combat trigger test structure.

### Step 9: Card Definition (later phase)

**Suggested card**: Warchief Giant
- `{3}{R}{R}`, Creature -- Giant Warrior, 5/3
- Haste, Myriad
- Simple, clean card to test the ability

**Alternative**: Banshee of the Dread Choir
- `{3}{B}{B}`, Creature -- Spirit, 4/4
- Myriad + "Whenever this creature deals combat damage to a player, that player discards a card."
- Tests myriad + combat damage trigger interaction

**Card lookup**: use `card-definition-author` agent

### Step 10: Game Script (later phase)

**Suggested scenario**: 4-player Commander game. P1 has Warchief Giant. P1's turn, combat phase. P1 declares Warchief Giant attacking P2. Myriad trigger fires. After resolution, tokens attack P3 and P4. Proceed to combat damage — tokens deal 5 damage each to P3 and P4, original deals 5 to P2. At end of combat, tokens are exiled.

**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

1. **Copy effects (Layer 1)**: Myriad tokens use `create_copy_effect()` from `copy.rs`. Ensure the copy captures copiable values of the source as it exists when the trigger resolves (not when it was declared as attacker).

2. **Combat state**: Tokens must be added to `state.combat.attackers` so they participate in blocking and damage steps. They are tapped and attacking but were NOT declared as attackers (no declare-attackers triggers fire).

3. **ETB triggers**: Token copies entering the battlefield DO trigger ETB abilities. This includes the myriad ability itself if the token has myriad (but the token's myriad should NOT trigger because the token was not "declared as an attacker" -- it entered attacking).

4. **Token cease-to-exist SBA (CR 704.5d)**: If a myriad token is exiled or moved to a non-battlefield zone before end of combat, it ceases to exist as an SBA. The end-of-combat exile should gracefully handle missing tokens.

5. **Summoning sickness**: Tokens have summoning sickness but are already attacking, so it does not matter. They cannot tap for abilities during combat though.

6. **Object identity (CR 400.7)**: After the source creature leaves the battlefield, the `CopyOf` continuous effect may reference a dead ObjectId. The `get_copiable_values` function must handle this gracefully (return None, token retains whatever characteristics it had).

7. **Multiplayer APNAP**: Myriad triggers go on the stack in APNAP order with other attack triggers. In a standard Commander game, only the active player attacks, so this is straightforward.

8. **GameEvent emissions**: Ensure `TokenCreated` and `PermanentEnteredBattlefield` events are emitted for each token. These trigger ETB-related abilities on other permanents.

## Files to Modify (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Myriad` |
| `crates/engine/src/state/hash.rs` | Add hash discriminant 64 |
| `crates/engine/src/state/game_object.rs` | Add `myriad_exile_at_eoc: bool` to `GameObject`; add `MyriadTrigger` to `StackObjectKind` |
| `crates/engine/src/state/stubs.rs` | Add `is_myriad_trigger: bool` to `PendingTrigger` |
| `crates/engine/src/state/mod.rs` | Initialize `myriad_exile_at_eoc: false` in zone-move helpers |
| `crates/engine/src/state/builder.rs` | Initialize field in ObjectSpec; add myriad trigger wiring |
| `crates/engine/src/effects/mod.rs` | Initialize field in `make_token()` |
| `crates/engine/src/rules/abilities.rs` | Tag myriad triggers in AttackersDeclared handler; handle `MyriadTrigger` in flush |
| `crates/engine/src/rules/resolution.rs` | Add `MyriadTrigger` resolution: create copies, register in combat |
| `crates/engine/src/rules/turn_actions.rs` | Exile myriad tokens in `end_combat()` |
| `tools/replay-viewer/src/view_model.rs` | Add display arm |
| `crates/engine/tests/myriad.rs` | 8 unit tests |

## Pre-Implementation Verification Tasks

Before coding, the runner should verify these lookups:

1. **Search for `StackObjectKind`** — confirm enum location and existing variants (e.g., `EvokeSacrificeTrigger`)
2. **Search for `GameEvent::ObjectExiled`** — confirm it exists or find the correct event for zone moves to exile
3. **Read `resolution.rs`** — find the `EvokeSacrificeTrigger` match arm to use as a pattern for `MyriadTrigger`
4. **Read `flush_pending_triggers`** — find the `is_evoke_sacrifice` handling to use as a pattern for `is_myriad_trigger`
5. **Grep for all `was_unearthed`** to find every site where `myriad_exile_at_eoc` needs to be initialized
