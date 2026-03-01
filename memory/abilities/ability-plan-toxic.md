# Ability Plan: Toxic

**Generated**: 2026-03-01
**CR**: 702.164 (NOT 702.156 -- the batch plan has the wrong number; 702.156 is Ravenous)
**Priority**: P4
**Similar abilities studied**: Infect (702.90) in `combat.rs:1139-1269`, `effects/mod.rs:155-197`; Lifelink in `combat.rs:1317-1343`; Poisonous (702.70) is a separate ability in the same batch

## CR Rule Text

702.164. Toxic

702.164a Toxic is a static ability. It is written "toxic N," where N is a number.

702.164b Some rules and effects refer to a creature's "total toxic value." A creature's
total toxic value is the sum of all N values of toxic abilities that creature has.

702.164c Combat damage dealt to a player by a creature with toxic causes that creature's
controller to give the player a number of poison counters equal to that creature's total
toxic value, in addition to the damage's other results. See rule 120.3.

### Supporting Rule

120.3g Combat damage dealt to a player by a creature with toxic causes that creature's
controller to give the player a number of poison counters equal to that creature's total
toxic value, in addition to the damage's other results. See rule 702.164, "Toxic."

### Poison Counter SBA (already implemented)

704.5c If a player has ten or more poison counters, that player loses the game.

104.3d If a player has ten or more poison counters, that player loses the game the next
time a player would receive priority. (This is a state-based action. See rule 704.)

## Key Edge Cases

1. **Toxic is a static ability, NOT a triggered ability (CR 702.164a).** Unlike Poisonous
   (702.70a -- triggered), Toxic applies as a damage result (CR 120.3g). Poison counters
   are given as part of the combat damage event, not via a trigger on the stack. This means
   Toxic cannot be responded to separately from damage.

2. **Multiple instances are cumulative (CR 702.164b).** If a creature has Toxic 2 and gains
   Toxic 1, its total toxic value is 3. This is unlike Infect where multiple instances are
   redundant. The implementation must sum ALL Toxic N values from the creature's keywords.

3. **"In addition to" -- normal damage still applies (ruling 2023-02-04).** A 2/2 with
   Toxic 1 that deals combat damage to a player causes the player to lose 2 life AND get
   1 poison counter. This is unlike Infect which replaces life loss.

4. **Only combat damage to players triggers Toxic (CR 702.164c, ruling 2023-02-04).**
   - Combat damage to creatures: no poison counters.
   - Combat damage to planeswalkers: no poison counters.
   - Non-combat damage to players: no poison counters.

5. **Toxic value is independent of damage amount (ruling 2023-02-04).** Even if the actual
   combat damage is modified (e.g., Gratuitous Violence doubling it, or damage prevention
   reducing it), the number of poison counters given equals the total toxic value, not the
   damage dealt. However, if ALL damage is prevented (0 damage dealt), Toxic should not
   apply (CR 120.3g says "combat damage dealt" -- 0 damage is not "dealt").

6. **Toxic + Infect interaction.** If a creature has both Toxic and Infect:
   - Infect replaces life loss with poison counters (equal to damage amount).
   - Toxic adds poison counters equal to total toxic value, "in addition to" damage results.
   - The player gets (damage amount) poison from Infect + (toxic value) poison from Toxic.
   - This is an edge case but follows from the rules text.

7. **Toxic + Lifelink.** Lifelink still applies normally (ruling 2023-02-04: "Any other
   effects of that damage, such as life gain from lifelink, still apply"). Toxic does not
   interact with lifelink -- both operate independently on the damage event.

8. **Damage must be > 0.** If combat damage is fully prevented (final_dmg == 0), no damage
   is "dealt" and Toxic does not apply. The existing `if final_dmg == 0 { continue; }`
   guard in `combat.rs:1225-1228` handles this.

9. **Multiplayer: each assignment is separate.** If a creature with Toxic attacks and deals
   combat damage to multiple players (via effects), each player gets poison counters from
   their respective damage assignment.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant
- [ ] Step 2: Rule enforcement
- [ ] Step 3: Trigger wiring (N/A -- Toxic is static, not triggered)
- [ ] Step 4: Unit tests
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Toxic(u32)` variant after `Training` (line ~700)
**Pattern**: Follow `KeywordAbility::Afflict(u32)` at line 674 -- parameterized keyword

```rust
/// CR 702.164: Toxic N -- static ability.
/// "Combat damage dealt to a player by a creature with toxic causes that
/// creature's controller to give the player a number of poison counters
/// equal to that creature's total toxic value, in addition to the damage's
/// other results."
///
/// CR 702.164b: Multiple instances are cumulative -- total toxic value is
/// the sum of all N values.
/// CR 120.3g: Only combat damage to a player; does not apply to creatures,
/// planeswalkers, or non-combat damage.
Toxic(u32),
```

**Hash**: Add to `crates/engine/src/state/hash.rs` in the `KeywordAbility` `HashInto` impl.
Next available discriminant is 83 (Training is 82).

```rust
KeywordAbility::Toxic(n) => {
    83u8.hash_into(hasher);
    n.hash_into(hasher);
}
```

**View model**: Add to `tools/replay-viewer/src/view_model.rs` in the keyword display match.

```rust
KeywordAbility::Toxic(n) => format!("Toxic {n}"),
```

**TUI stack_view.rs**: No changes needed -- Toxic is not a triggered ability, so no new
`StackObjectKind` variant is required.

**Match arms**: Grep for all exhaustive `KeywordAbility` match expressions and add the
`Toxic(n)` arm. Key locations:
- `state/hash.rs` (HashInto impl)
- `tools/replay-viewer/src/view_model.rs` (display)
- Any other exhaustive matches found by the compiler

### Step 2: Rule Enforcement

**File**: `crates/engine/src/rules/combat.rs`
**Action**: Add Toxic poison counter logic in the combat damage application loop.

This is a **damage result**, not a trigger. Per CR 120.3g and CR 702.164c, the poison
counters are given as part of the combat damage event, alongside Lifelink life gain
and Infect poison replacement. The implementation goes in `apply_combat_damage_assignments()`
(the function starting around line 1050 that applies combat damage).

#### Step 2a: Extract total toxic value in pre-extract phase

In the `app_info` collection loop (lines 1119-1171), add extraction of the creature's
total toxic value alongside deathtouch, lifelink, wither, infect.

**Pattern**: Follow the `source_infect` extraction at lines 1141-1144.

The DamageAppInfo tuple type needs a new field: `u32` for total toxic value.

```rust
// CR 702.164b: Total toxic value is the sum of all Toxic N values.
let source_toxic_total: u32 = chars
    .as_ref()
    .map(|c| {
        c.keywords
            .iter()
            .filter_map(|kw| match kw {
                KeywordAbility::Toxic(n) => Some(*n),
                _ => None,
            })
            .sum()
    })
    .unwrap_or(0);
```

Update `type DamageAppInfo` to include the toxic value (add `u32` to the tuple).

#### Step 2b: Apply Toxic poison counters in the damage application loop

In the `CombatDamageTarget::Player(player_id)` arm (lines 1258-1306), after the
existing Infect/normal damage logic and BEFORE the commander damage tracking, add
the Toxic poison counter logic.

**CR**: 120.3g -- "in addition to the damage's other results"

```rust
// CR 702.164c / CR 120.3g: Toxic -- give poison counters equal to
// total toxic value, in addition to normal damage results.
// Applies regardless of whether the source also has infect.
if *source_toxic_total > 0 {
    if let Some(player) = state.players.get_mut(player_id) {
        player.poison_counters += *source_toxic_total;
    }
    poison_events.push(GameEvent::PoisonCountersGiven {
        player: *player_id,
        amount: *source_toxic_total,
        source: assignment.source,
    });
}
```

**Key**: This is placed AFTER the Infect check and normal life loss, because Toxic
applies "in addition to" the damage's other results. Toxic and Infect are independent --
if a creature has both, the player gets Infect poison (from damage amount) + Toxic poison
(from toxic value). The order of mutation does not matter since both simply increment
`poison_counters`.

#### Step 2c: Non-combat damage -- NO changes needed

Per CR 702.164c and ruling 2023-02-04, Toxic only applies to combat damage. The
`effects/mod.rs` DealDamage handler (non-combat damage) requires NO changes for Toxic.

### Step 3: Trigger Wiring

**N/A** -- Toxic is a static ability (CR 702.164a), not a triggered ability. There is no
stack object, no trigger event, no `StackObjectKind` variant needed.

This is a critical distinction from **Poisonous** (CR 702.70a), which IS a triggered
ability ("Whenever this creature deals combat damage to a player, that player gets N
poison counters"). Poisonous uses the trigger dispatch system; Toxic modifies the damage
result inline.

### Step 4: Unit Tests

**File**: `crates/engine/tests/toxic.rs` (new file)
**Tests to write**:

1. **`test_702_164_toxic_basic_combat_damage_gives_poison`**
   CR 702.164c / CR 120.3g -- A creature with Toxic 1 deals combat damage to a player.
   The player loses life equal to the damage AND gets 1 poison counter.
   Assert: player.life_total decreased by damage amount; player.poison_counters == 1.

2. **`test_702_164_toxic_damage_to_creature_no_poison`**
   CR 702.164c / ruling 2023-02-04 -- A creature with Toxic is blocked. Damage is dealt
   to the blocking creature. No player receives poison counters.

3. **`test_702_164_toxic_multiple_instances_cumulative`**
   CR 702.164b -- A creature with Toxic 2 and Toxic 1 (total toxic value 3) deals combat
   damage to a player. The player gets 3 poison counters, not 2 or 1.

4. **`test_702_164_toxic_damage_prevented_no_poison`**
   CR 120.3g -- If all combat damage is prevented (final_dmg == 0), Toxic does not apply.
   No poison counters are given.

5. **`test_702_164_toxic_with_infect`**
   Edge case -- A creature with both Toxic 1 and Infect deals 3 combat damage to a player.
   Infect: player gets 3 poison counters (replaces life loss).
   Toxic: player gets 1 additional poison counter.
   Total: 4 poison counters, 0 life loss.

6. **`test_702_164_toxic_with_lifelink`**
   Ruling 2023-02-04 -- A creature with Toxic 1 and Lifelink deals 2 combat damage to a
   player. Player loses 2 life and gets 1 poison counter. Source controller gains 2 life.

7. **`test_702_164_toxic_kills_via_poison_sba`**
   CR 704.5c -- A player at 9 poison counters is dealt combat damage by a creature with
   Toxic 1. The player receives their 10th poison counter and loses the game via SBA.

8. **`test_702_164_toxic_multiplayer`**
   Multiplayer scenario -- A creature with Toxic 2 attacks a specific player in a 4-player
   game. Only the defending player receives the poison counters.

**Pattern**: Follow the Infect tests in `crates/engine/tests/keywords.rs` starting at
line 2611 (`test_702_90_infect_combat_damage_places_minus_counters_on_creature`). Use
the same `GameStateBuilder::four_player()`, declare attackers, advance through combat
damage step, assert on `player.poison_counters`.

**Important**: Unlike Infect tests, Toxic tests must verify that life loss ALSO occurs
(Toxic is additive, not replacement).

### Step 5: Card Definition (later phase)

**Suggested card**: Jawbone Duelist
- Name: Jawbone Duelist
- Mana cost: {1}{W}
- Type: Creature -- Phyrexian Soldier
- P/T: 1/1
- Abilities: Double Strike, Toxic 1
- Oracle: "Double strike / Toxic 1"
- This card is excellent for testing because double strike means it deals combat damage
  twice (first strike + normal), so Toxic should give 2 poison counters total (1 per
  damage step).

**Alternative simpler card**: Pestilent Syphoner
- {1}{B}, 1/1 Phyrexian Insect, Flying + Toxic 1
- Simpler to test with (no double strike interaction).

**Card lookup**: use `card-definition-author` agent

### Step 6: Game Script (later phase)

**Suggested scenario**: Pestilent Syphoner (1/1 Flying Toxic 1) attacks an opponent with
no flying blockers. Damage goes through: opponent loses 1 life and gets 1 poison counter.
Verify both life loss and poison counter placement in the assertion phase.

**Subsystem directory**: `test-data/generated-scripts/combat/`
**Suggested sequence number**: Use next available after existing combat scripts.

### Shared Infrastructure with Poisonous

Toxic and Poisonous are in the same batch (Batch 3). Key differences:

| Aspect | Toxic (702.164) | Poisonous (702.70) |
|--------|----------------|-------------------|
| Type | Static ability | Triggered ability |
| When | Part of damage result (CR 120.3g) | Goes on the stack |
| Stacking | Cumulative (sum N values) | Each instance triggers separately |
| Counter count | Total toxic value (fixed) | N per instance |
| Implementation site | `combat.rs` damage loop inline | `abilities.rs` trigger dispatch |
| Respondable | No (part of damage) | Yes (trigger uses the stack) |

**Shared infrastructure**: Both reuse:
- `player.poison_counters` (PlayerState field -- already exists)
- `GameEvent::PoisonCountersGiven` (event type -- already exists)
- `poison_events` Vec in `apply_combat_damage_assignments()` (for Toxic inline)
- 10-poison SBA in `sba.rs:225` (already exists from Infect)

**No new shared types needed.** Each uses existing infrastructure differently.

## Interactions to Watch

- **Toxic + Infect**: Both can coexist. Infect replaces life loss with poison counters
  (damage-amount worth). Toxic adds poison counters equal to toxic value independently.
  A creature with both Infect and Toxic 2 dealing 3 combat damage to a player gives
  3 (Infect) + 2 (Toxic) = 5 poison counters.
- **Toxic + Lifelink**: Independent. Lifelink causes life gain; Toxic causes poison
  counters. Both fire on the same damage event.
- **Toxic + Trample**: Trample affects how combat damage is assigned (to blocker vs.
  player). If trample damage goes through to the player, Toxic applies on that damage
  assignment. But Toxic value is fixed regardless of how much damage gets through.
- **Toxic + Double Strike**: A creature with double strike deals combat damage twice
  (first strike step + normal combat damage step). Toxic should apply each time combat
  damage is dealt to a player. Each step is a separate `apply_combat_damage_assignments()`
  call, so the extraction + application runs independently for each.
- **Toxic + damage prevention**: If ALL damage is prevented, Toxic does not apply (the
  `final_dmg == 0` guard catches this). If partial damage is prevented (some still gets
  through), Toxic applies at full toxic value (since the value is independent of damage).
- **Layer system**: Toxic(N) is a keyword on the creature's characteristics. If an effect
  removes all abilities (Humility, Dress Down), the creature loses Toxic and no poison
  counters are given. The `calculate_characteristics()` call in the pre-extract phase
  correctly reflects the creature's current keywords at the time damage is dealt.
