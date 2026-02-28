# Ability Plan: Ascend

**Generated**: 2026-02-28
**CR**: 702.131
**Priority**: P3
**Similar abilities studied**: `NoMaxHandSize` (static keyword checked at cleanup, `turn_actions.rs:229-242`); SBA functions in `sba.rs:58-123`

## CR Rule Text

702.131. Ascend

  702.131a Ascend on an instant or sorcery spell represents a spell ability. It means "If you control ten or more permanents and you don't have the city's blessing, you get the city's blessing for the rest of the game."

  702.131b Ascend on a permanent represents a static ability. It means "Any time you control ten or more permanents and you don't have the city's blessing, you get the city's blessing for the rest of the game."

  702.131c The city's blessing is a designation that has no rules meaning other than to act as a marker that other rules and effects can identify. Any number of players may have the city's blessing at the same time.

  702.131d After a player gets the city's blessing, continuous effects are reapplied before the game checks to see if the game state or preceding events have matched any trigger conditions.

## Key Edge Cases

- **Ascend requires a source with ascend**: If you control 10+ permanents but don't control a permanent or resolving spell with ascend, you do NOT get the city's blessing. (Ruling on Wayward Swordtooth: "If you control ten permanents but don't control a permanent or resolving spell with ascend, you don't get the city's blessing.")
- **City's blessing is permanent once gained**: Once a player has it, they keep it for the rest of the game even if permanents drop below 10. Cannot be removed by any effect. (CR 702.131c)
- **Ascend on permanents is NOT a triggered ability**: Does not use the stack. Players cannot respond to gaining the city's blessing once the 10th permanent is controlled. (Ruling: "Ascend on a permanent isn't a triggered ability and doesn't use the stack.")
- **Ascend on instants/sorceries is a spell ability**: Checked at resolution time, not cast time. Players may respond to the spell before it resolves. (CR 702.131a; Ruling: "If you cast a spell with ascend, you don't get the city's blessing until it resolves.")
- **10th permanent entering + leaving immediately**: "If your tenth permanent enters the battlefield and then a permanent leaves the battlefield immediately afterwards (most likely due to the Legend Rule or due to being a creature with 0 toughness), you get the city's blessing before it leaves the battlefield." This means the Ascend check happens BEFORE other SBAs in the same pass.
- **Permanents include all permanent types**: "A permanent is any object on the battlefield, including tokens and lands. Spells and emblems aren't permanents." (Ruling)
- **Multiplayer**: Any number of players may have the city's blessing simultaneously. (CR 702.131c)
- **CR 702.131d**: After gaining the blessing, continuous effects are reapplied before trigger checking. This means the layer system should recompute after the blessing is set, before the SBA loop continues.

## Current State (from ability-wip.md)

- [x] Step 1: Enum variant -- `KeywordAbility::Ascend` at `types.rs:L510`
- [x] Step 1b: Hash -- `hash.rs:L433` (discriminant 62)
- [x] Step 1c: View model -- `view_model.rs:L653`
- [x] Step 1d: PlayerState field -- `has_citys_blessing: bool` at `player.rs:L123`
- [x] Step 1e: Builder default -- `builder.rs:L260` (`has_citys_blessing: false`)
- [x] Step 1f: Hash for PlayerState -- `hash.rs:L615`
- [ ] Step 2: Rule enforcement (SBA-like check for permanents)
- [ ] Step 3: Rule enforcement (spell resolution check for instants/sorceries)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant (DONE)

All pre-edits are complete. `KeywordAbility::Ascend`, `has_citys_blessing: bool` on `PlayerState`, hash discriminant 62, view model serialization, and builder default are all in place.

### Step 2: Ascend SBA-like Check for Permanents

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/sba.rs`
**Action**: Add a new function `check_ascend` and call it from `apply_sbas_once`.

**New function** `check_ascend(state: &mut GameState, chars_map: &HashMap<ObjectId, Characteristics>) -> Vec<GameEvent>`:

1. For each player who does NOT already have `has_citys_blessing`:
   a. Check if that player controls any permanent on the battlefield with `KeywordAbility::Ascend` in its computed keywords (`chars_map`).
   b. If yes, count how many permanents that player controls on the battlefield (all types: creatures, artifacts, enchantments, lands, planeswalkers, tokens). Use `state.objects.values().filter(|o| o.zone == ZoneId::Battlefield && o.controller == player_id).count()`.
   c. If count >= 10, set `state.players.get_mut(&player_id).unwrap().has_citys_blessing = true` and emit `GameEvent::CitysBlessingGained { player }`.
2. **IMPORTANT: The Ascend check must run BEFORE other SBAs that remove permanents** (like legendary rule, creature death). This ensures the 10th-permanent-entering-then-leaving edge case works correctly. Place the call as the FIRST check in `apply_sbas_once`, before `check_player_sbas`.

**Why computed keywords**: Ascend is a static ability. If an effect removes all abilities (Humility), the permanent no longer has Ascend and should not grant the blessing. Using `chars_map` (layer-computed) is correct.

**CR**: 702.131b -- "Any time you control ten or more permanents and you don't have the city's blessing, you get the city's blessing for the rest of the game."

**Pattern**: Follow the structure of `check_player_sbas` at line 130 -- iterate players, check condition, mutate state, emit event. But simpler: no zone moves, just a boolean flip.

**Placement in `apply_sbas_once`**: Insert call at the top of the function body (after `chars_map` construction), before all other SBA checks:
```rust
events.extend(check_ascend(state, &chars_map));
events.extend(check_player_sbas(state));
// ... rest of checks ...
```

This ensures the 10th-permanent edge case works: ascend is checked before legendary rule or creature death SBAs remove permanents in the same pass.

### Step 3: Ascend Check on Instant/Sorcery Resolution

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: After the spell effect executes (line ~186) and before the card moves to its final zone, check if the resolving spell has `KeywordAbility::Ascend` and grant the city's blessing if the controller has 10+ permanents.

**Location**: Inside the `resolve_top_of_stack` function, in the `StackObjectKind::Spell { source_object }` arm, after the spell effect is executed (after `events.extend(effect_events);` at line ~186) but before the `if stack_obj.is_copy` branch at line ~195.

**Logic**:
```rust
// CR 702.131a: Ascend on instant/sorcery — check at resolution time.
// If the resolving spell has ascend, check permanent count for controller.
{
    let has_ascend = state
        .object(source_object)
        .map(|obj| obj.characteristics.keywords.contains(&KeywordAbility::Ascend))
        .unwrap_or(false);
    if has_ascend {
        if let Some(player) = state.players.get(&controller) {
            if !player.has_citys_blessing {
                let permanent_count = state
                    .objects
                    .values()
                    .filter(|o| o.zone == ZoneId::Battlefield && o.controller == controller)
                    .count();
                if permanent_count >= 10 {
                    if let Some(p) = state.players.get_mut(&controller) {
                        p.has_citys_blessing = true;
                    }
                    events.push(GameEvent::CitysBlessingGained { player: controller });
                }
            }
        }
    }
}
```

**Note**: We use `obj.characteristics.keywords` (raw) rather than layer-computed here because the spell is on the stack (not a battlefield permanent). Layer computation is only meaningful for battlefield permanents. The keywords on a stack spell come from its printed characteristics which is sufficient.

**CR**: 702.131a -- "If you control ten or more permanents and you don't have the city's blessing, you get the city's blessing for the rest of the game."

### Step 4: GameEvent Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add a new `CitysBlessingGained` variant to `GameEvent`.

**Location**: After the existing SBA events section (around line 300-310), add:
```rust
/// CR 702.131: A player gained the city's blessing designation.
///
/// Emitted when ascend (on a permanent or resolving spell) determines that
/// the player controls 10+ permanents. The city's blessing is permanent --
/// once gained, it is never removed (CR 702.131c).
CitysBlessingGained { player: PlayerId },
```

**Hash/Serialize**: `GameEvent` derives `Serialize, Deserialize` -- the new variant is auto-handled.

### Step 5: View Model Update

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add `has_citys_blessing: bool` to `PlayerView` struct and populate it from `PlayerState`.

**In `PlayerView` struct** (line ~62):
```rust
pub has_citys_blessing: bool,
```

**In `build_players_view`** where `PlayerView` is constructed (line ~265):
```rust
has_citys_blessing: player.has_citys_blessing,
```

### Step 6: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/ascend.rs`
**Tests to write**:

1. **`test_ascend_basic_permanent_grants_blessing`** -- CR 702.131b: Place 10 permanents on the battlefield, one with Ascend. Verify `has_citys_blessing` becomes true after SBA check.
   - Setup: P1 controls 9 lands + 1 creature with Ascend keyword. Total = 10.
   - Action: Pass priority (triggers SBA check).
   - Assert: `state.players.get(&p1).unwrap().has_citys_blessing == true`.
   - Assert: `CitysBlessingGained` event emitted for P1.

2. **`test_ascend_below_threshold_no_blessing`** -- CR 702.131b negative: Player has Ascend permanent but only 9 permanents total.
   - Setup: P1 controls 8 lands + 1 creature with Ascend. Total = 9.
   - Action: Pass priority.
   - Assert: `has_citys_blessing == false`.
   - Assert: No `CitysBlessingGained` event.

3. **`test_ascend_blessing_permanent_once_gained`** -- CR 702.131c: Once gained, the blessing persists even if permanents drop below 10.
   - Setup: P1 has blessing (set `has_citys_blessing = true` directly or achieve via 10 permanents).
   - Action: Remove permanents so count drops below 10. Pass priority again.
   - Assert: `has_citys_blessing` is still true.

4. **`test_ascend_no_ascend_source_no_blessing`** -- Key ruling: 10+ permanents but no permanent with Ascend -> no blessing.
   - Setup: P1 controls 10 lands (none with Ascend keyword).
   - Action: Pass priority.
   - Assert: `has_citys_blessing == false`.

5. **`test_ascend_multiple_players_independent`** -- CR 702.131c: Multiple players can have the blessing simultaneously.
   - Setup: P1 and P2 each control 10+ permanents including one with Ascend.
   - Action: Pass priority.
   - Assert: Both P1 and P2 have `has_citys_blessing == true`.

6. **`test_ascend_instant_sorcery_on_resolution`** -- CR 702.131a: Ascend on a spell grants blessing at resolution, not cast time.
   - Setup: P1 controls 10 permanents. P1 casts a sorcery with Ascend keyword.
   - Action: Cast spell, verify no blessing yet. Resolve spell, verify blessing granted.
   - Assert: After cast but before resolution: `has_citys_blessing == false`. After resolution: `has_citys_blessing == true`.
   - **Note**: This test requires a card definition for a sorcery with Ascend. Use a simple test card: "Ascend Sorcery" with `AbilityDefinition::Keyword(KeywordAbility::Ascend)` and a no-op or simple spell effect.

7. **`test_ascend_tokens_count_as_permanents`** -- Ruling: tokens are permanents and count toward the 10.
   - Setup: P1 controls 5 lands + 1 creature with Ascend + 4 token creatures. Total = 10.
   - Action: Pass priority.
   - Assert: `has_citys_blessing == true`.

**Pattern**: Follow tests in `/home/airbaggie/scutemob/crates/engine/tests/dethrone.rs` for structure (imports, `GameStateBuilder`, `process_command`, event checking). Use `ObjectSpec::card(owner, name).with_keyword(KeywordAbility::Ascend).in_zone(ZoneId::Battlefield)` for permanents with Ascend.

**Test helper**: For tests needing to check if an event was emitted:
```rust
assert!(events.iter().any(|e| matches!(e, GameEvent::CitysBlessingGained { player } if *player == p1)));
```

### Step 7: Card Definition (later phase)

**Suggested cards**:
- **Wayward Swordtooth** (Creature, simple Ascend + conditional attack/block) -- good for unit test validation
- **Golden Demise** (Sorcery, Ascend + conditional effect based on blessing) -- tests instant/sorcery path

**Card lookup**: Use `card-definition-author` agent.

### Step 8: Game Script (later phase)

**Suggested scenario**: Wayward Swordtooth enters the battlefield as the 10th permanent, granting the city's blessing. Then permanents are destroyed, but the blessing persists.
**Subsystem directory**: `test-data/generated-scripts/stack/` (for Ascend spell resolution) or `test-data/generated-scripts/baseline/` (for SBA-based permanent check)

## Interactions to Watch

1. **Ascend + Humility (Layer 6)**: Humility removes all abilities. If a permanent's Ascend is removed, it no longer counts as having Ascend. However, a player who already has the city's blessing keeps it. The SBA check uses `chars_map` (layer-computed keywords), so Humility correctly suppresses Ascend on affected permanents.

2. **Ascend timing vs. Legend Rule / SBAs**: Per the ruling about the 10th permanent entering then leaving, the Ascend check must run FIRST in the SBA loop (before legendary rule, before creature death), so the player gets the blessing before other SBAs remove permanents in the same pass.

3. **CR 702.131d**: "After a player gets the city's blessing, continuous effects are reapplied before the game checks to see if the game state or preceding events have matched any trigger conditions." This is automatically handled by the SBA loop: after each SBA pass, triggers are checked (`check_triggers` in `check_and_apply_sbas`), and on the next pass the `chars_map` is recomputed (which reapplies continuous effects). No special handling needed beyond placing the Ascend check in the SBA loop.

4. **Multiplayer**: Each player's Ascend check is independent. Player A can have the blessing while Player B does not.

5. **Tokens**: Token permanents count toward the 10-permanent threshold. The count uses `state.objects.values().filter(|o| o.zone == ZoneId::Battlefield && o.controller == pid)` which includes tokens.

## Import Requirements

**`sba.rs`**: `KeywordAbility` is already imported at line 35.
**`resolution.rs`**: Will need to add `KeywordAbility` to imports if not already present. Check existing imports.
**`events.rs`**: `PlayerId` is already imported.
