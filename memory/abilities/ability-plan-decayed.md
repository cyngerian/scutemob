# Ability Plan: Decayed

**Generated**: 2026-02-28
**CR**: 702.147
**Priority**: P4
**Similar abilities studied**:
- Shadow (CR 702.28) -- blocking restriction pattern: `crates/engine/src/rules/combat.rs:491-501`, `crates/engine/tests/shadow.rs`
- Myriad (CR 702.116) -- end-of-combat TBA pattern: `crates/engine/src/rules/turn_actions.rs:556-609`, `crates/engine/src/state/game_object.rs:395-398` (`myriad_exile_at_eoc` flag), `crates/engine/tests/myriad.rs`
- Evoke (CR 702.74) -- sacrifice-at-resolution pattern: `crates/engine/src/rules/resolution.rs:683-760`

## CR Rule Text

```
702.147. Decayed

702.147a Decayed represents a static ability and a triggered ability.
         "Decayed" means "This creature can't block" and "When this
         creature attacks, sacrifice it at end of combat."
```

Also relevant:
- CR 122.1b: Decayed can be a keyword counter (on permanents or cards in zones
  other than the battlefield, causing the object to gain the keyword).

## Key Edge Cases

- **"Sacrifice at end of combat" persists even after losing decayed (Wilhelt ruling 2021-09-24)**:
  "Once a creature with decayed attacks, it will be sacrificed at end of combat,
  even if it no longer has decayed at that time." This means the sacrifice effect is
  locked in at attack declaration, not re-checked at end of combat. Implementation:
  tag the object with `decayed_sacrifice_at_eoc = true` when it attacks, and check
  the tag (not the keyword) in `end_combat()`.

- **No attacking requirement (ruling 2021-09-24)**: "Decayed does not create any
  attacking requirements. You may choose not to attack with a creature that has
  decayed." The creature is not forced to attack. The sacrifice only happens
  when/if it attacks.

- **No haste (ruling 2021-09-24)**: "Decayed does not grant haste. Creatures with
  decayed that enter the battlefield during your turn may not attack until your
  next turn." Summoning sickness applies normally.

- **"Can't block" is absolute**: A creature with decayed cannot block under any
  circumstances. This is a static ability (not conditional on attacking). Even if
  the creature never attacks, it still cannot block.

- **Token-centric in practice**: Most decayed creatures are 2/2 black Zombie tokens
  created by other abilities (e.g., Wilhelt, Ghoulcaller's Harvest). The keyword
  itself is not token-specific -- any creature can have it -- but cards always
  create tokens with decayed rather than giving it to existing creatures.

- **Sacrifice is not destruction (CR 701.17a)**: Indestructible does not prevent
  sacrifice. The `end_combat()` sacrifice bypasses indestructible naturally because
  sacrifice uses `move_object_to_zone` to graveyard, not the "destroy" path.

- **Keyword counters (CR 122.1b)**: Decayed can exist as a keyword counter. The
  engine does not yet implement keyword counters as a separate system -- they are
  modeled as keywords on the object. No special handling needed for V1.

- **Multiple instances are redundant**: "This creature can't block" doesn't stack.
  "When this creature attacks, sacrifice it at end of combat" triggers for each
  instance but the result is the same (creature is already being sacrificed). No
  special multi-instance handling needed.

- **Multiplayer**: No special multiplayer considerations beyond the standard combat
  framework. The creature attacks one player, the sacrifice happens at EOC
  regardless of which player was attacked.

## Current State (from ability-wip.md)

No steps done. Horsemanship is currently in WIP phase (plan). Decayed is a separate
ability in the same batch (Batch 1, position 1.5).

- [ ] 1. Enum variant -- does not exist anywhere in the codebase
- [ ] 2. Rule enforcement (blocking restriction in combat.rs)
- [ ] 3. Rule enforcement (EOC sacrifice in turn_actions.rs)
- [ ] 4. Trigger wiring (tag object on SelfAttacks)
- [ ] 5. Unit tests
- [ ] 6. Card definition
- [ ] 7. Game script
- [ ] 8. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Decayed` variant after `Overload` (line ~599), before
the closing `}` of the enum.
**Pattern**: Follow `KeywordAbility::Shadow` at lines 586-590.

Add:
```rust
    /// CR 702.147: Decayed -- static ability + triggered ability.
    /// "This creature can't block" (static) and "When this creature attacks,
    /// sacrifice it at end of combat" (triggered).
    ///
    /// The blocking restriction is enforced in `rules/combat.rs`.
    /// The EOC sacrifice uses a tag-on-object pattern (like Myriad):
    /// `decayed_sacrifice_at_eoc` is set on the creature when it attacks,
    /// and `end_combat()` in `turn_actions.rs` sacrifices all tagged creatures.
    ///
    /// Ruling 2021-09-24: "Once a creature with decayed attacks, it will be
    /// sacrificed at end of combat, even if it no longer has decayed at that time."
    /// CR 702.147a: Multiple instances are redundant.
    Decayed,
```

### Step 1b: Hash Discriminant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add discriminant for `KeywordAbility::Decayed` after the last keyword
discriminant (70 = Overload). Use discriminant 71 (or 72 if Horsemanship takes 71
first -- check which is implemented first).

Add (in the `KeywordAbility` match in `hash.rs`, after Overload discriminant 70):
```rust
            // Decayed (discriminant 72) -- CR 702.147
            KeywordAbility::Decayed => 72u8.hash_into(hasher),
```

Note: If Horsemanship (planned as discriminant 71) is implemented first, use 72.
If Decayed is implemented first, use 71 and Horsemanship will use 72.

### Step 1c: GameObject Flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add `decayed_sacrifice_at_eoc: bool` field after `myriad_exile_at_eoc`
(line ~398).

Add:
```rust
    /// CR 702.147a: If true, this creature attacked with decayed and must be
    /// sacrificed at end of combat. Set when the SelfAttacks trigger fires for
    /// a creature with the Decayed keyword.
    ///
    /// Ruling 2021-09-24: "Once a creature with decayed attacks, it will be
    /// sacrificed at end of combat, even if it no longer has decayed at that time."
    /// This flag ensures the sacrifice happens even if decayed is removed after
    /// the attack declaration.
    #[serde(default)]
    pub decayed_sacrifice_at_eoc: bool,
```

### Step 1d: Flag in Zone-Change Reset

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/mod.rs`
**Action**: Add `decayed_sacrifice_at_eoc: false` in both zone-change locations where
`myriad_exile_at_eoc: false` appears (lines ~291 and ~378).

Add (after each `myriad_exile_at_eoc: false` line):
```rust
            // CR 400.7: decayed sacrifice flag is not preserved across zone changes.
            decayed_sacrifice_at_eoc: false,
```

### Step 1e: Flag in Builder

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: Add `decayed_sacrifice_at_eoc: false` in the `GameObject` construction
(after `myriad_exile_at_eoc: false` at line ~745).

Add:
```rust
                decayed_sacrifice_at_eoc: false,
```

### Step 1f: Flag in Effects Token Creation

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs`
**Action**: Add `decayed_sacrifice_at_eoc: false` in the token `GameObject`
construction (after `myriad_exile_at_eoc: false` at line ~2460).

Add:
```rust
        decayed_sacrifice_at_eoc: false,
```

### Step 1g: Flag in Myriad Token Creation

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs`
**Action**: Add `decayed_sacrifice_at_eoc: false` in the myriad token `GameObject`
construction (after `myriad_exile_at_eoc: true` at line ~1232).

Add:
```rust
                    decayed_sacrifice_at_eoc: false,
```

### Step 1h: Hash for Flag

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add `self.decayed_sacrifice_at_eoc.hash_into(hasher)` in the `GameObject`
hash impl, after `myriad_exile_at_eoc` (line ~617).

Add:
```rust
        // Decayed (CR 702.147a) -- creature must be sacrificed at end of combat
        self.decayed_sacrifice_at_eoc.hash_into(hasher);
```

### Step 1i: View Model (Replay Viewer)

**File**: `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs`
**Action**: Add arm to the exhaustive match on `KeywordAbility` (after the Overload
arm, around line ~679).

Add:
```rust
        KeywordAbility::Decayed => "Decayed".to_string(),
```

### Step 1j: TUI Stack View

**File**: `/home/airbaggie/scutemob/tools/tui/src/play/panels/stack_view.rs`
**Action**: No `StackObjectKind` variant is being added for Decayed (the sacrifice
is handled as a TBA, not a stack object), so no TUI change is needed for
StackObjectKind.

However, check whether there is an exhaustive match on `KeywordAbility` in the TUI.
Per the Horsemanship plan: "The TUI does not match on KeywordAbility." No TUI
change needed.

### Step 2: Blocking Restriction (Static Ability)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Action**: Add decayed blocking restriction check in `handle_declare_blockers`,
after the creature-type validation (line ~377) and before the duplicate-blocker
check (line ~379). This is the earliest possible point -- a creature with decayed
cannot block at all, so we reject it before any attacker-specific checks.

**CR**: 702.147a -- "This creature can't block."
**Pattern**: Similar to the `Defender` check in `handle_declare_attackers` (line ~93-98),
but for blocking. A creature-level restriction, not an attacker-level restriction.

Add (after `blocker_chars` is computed and the creature-type check at line ~376,
before the duplicate-blocker check at line ~379):
```rust
        // CR 702.147a: A creature with decayed can't block.
        if blocker_chars
            .keywords
            .contains(&KeywordAbility::Decayed)
        {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} has decayed and cannot block (CR 702.147a)",
                blocker_id
            )));
        }
```

### Step 3: EOC Sacrifice (Turn-Based Action)

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/turn_actions.rs`
**Action**: Add decayed sacrifice logic in `end_combat()`, after the myriad token
exile block (lines 575-604) and before `state.combat = None` (line 606).

**CR**: 702.147a -- "When this creature attacks, sacrifice it at end of combat."
**Pattern**: Follow the Myriad exile pattern exactly, but:
- Filter by `decayed_sacrifice_at_eoc` instead of `myriad_exile_at_eoc`
- Do NOT filter by `is_token` -- any creature (token or non-token) with the flag
  should be sacrificed
- Move to graveyard (sacrifice) instead of exile
- Use `check_zone_change_replacement` for proper replacement effect handling
  (e.g., commander zone redirect SBA, Rest in Peace exile replacement)
- Emit `CreatureDied` event for die-trigger support

Add (after the myriad exile loop, before `state.combat = None`):
```rust
    // CR 702.147a: Sacrifice all creatures that attacked with decayed.
    // Creatures are tagged with `decayed_sacrifice_at_eoc = true` at attack
    // declaration time (in check_triggers for SelfAttacks events).
    //
    // Ruling 2021-09-24: "Once a creature with decayed attacks, it will be
    // sacrificed at end of combat, even if it no longer has decayed at that time."
    //
    // CR 701.17a: Sacrifice is NOT destruction -- indestructible does not prevent it.
    //
    // TODO(M10+): Per CR 702.147a / CR 603.7, the EOC sacrifice is technically a
    // delayed triggered ability ("sacrifice it at end of combat"). The current TBA
    // implementation sacrifices with no interaction window (can't Stifle the
    // sacrifice). Same caveat as Myriad's EOC exile. Refactor when delayed trigger
    // infrastructure is expanded.
    let decayed_sacrifice_ids: Vec<crate::state::game_object::ObjectId> = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == crate::state::zone::ZoneId::Battlefield
                && obj.decayed_sacrifice_at_eoc
        })
        .map(|obj| obj.id)
        .collect();

    for obj_id in decayed_sacrifice_ids {
        let (owner, controller, pre_death_counters) = match state.objects.get(&obj_id) {
            Some(obj) => (obj.owner, obj.controller, obj.counters.clone()),
            None => continue,
        };

        // CR 614: Check replacement effects before moving to graveyard.
        let action = crate::rules::replacement::check_zone_change_replacement(
            state,
            obj_id,
            crate::state::zone::ZoneType::Battlefield,
            crate::state::zone::ZoneType::Graveyard,
            owner,
            &std::collections::HashSet::new(),
        );

        match action {
            crate::rules::replacement::ZoneChangeAction::Redirect {
                destination,
                ..
            } => {
                if let Ok((new_id, _old)) = state.move_object_to_zone(obj_id, destination) {
                    events.push(GameEvent::CreatureDied {
                        player: controller,
                        object_id: obj_id,
                        new_graveyard_id: new_id,
                        pre_death_counters,
                    });
                }
            }
            crate::rules::replacement::ZoneChangeAction::Proceed => {
                if let Ok((new_id, _old)) =
                    state.move_object_to_zone(obj_id, crate::state::zone::ZoneId::Graveyard(owner))
                {
                    events.push(GameEvent::CreatureDied {
                        player: controller,
                        object_id: obj_id,
                        new_graveyard_id: new_id,
                        pre_death_counters,
                    });
                }
            }
            crate::rules::replacement::ZoneChangeAction::PreventCompletely => {
                // Replacement prevented the zone change entirely (rare).
            }
        }
    }
```

**Important**: The `CreatureDied` event may not be the correct event for sacrifice
(sacrifice is not "dying" in all MTG contexts -- "dies" means "put into a graveyard
from the battlefield", which includes sacrifice). Verify that `CreatureDied` is the
standard event for this in the engine. If there's a separate `PermanentSacrificed`
event, use that instead.

Check `GameEvent` variants for the appropriate sacrifice event. The EvokeSacrificeTrigger
resolution at `resolution.rs:683-760` uses `CreatureDied`, confirming that sacrifice
uses the same event in this engine.

### Step 4: Attack Declaration Tag

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`
**Action**: In the `AttackersDeclared` event handler within `check_triggers`, after
the `SelfAttacks` trigger collection and myriad tagging (around line ~1358), add
logic to tag creatures with decayed.

**CR**: 702.147a -- "When this creature attacks, sacrifice it at end of combat."
**Pattern**: Similar to the myriad trigger tagging at lines 1341-1358, but simpler:
just set the flag on the object, no need for a stack trigger.

Add (after the myriad trigger tagging block, around line ~1359):
```rust
                    // CR 702.147a: Tag decayed creatures for EOC sacrifice.
                    // "When this creature attacks, sacrifice it at end of combat."
                    // This tag is set regardless of whether the creature still has
                    // decayed at resolution time (ruling 2021-09-24).
                    if let Some(obj) = state.objects.get(attacker_id) {
                        let chars = super::layers::calculate_characteristics(state, *attacker_id);
                        if let Some(c) = chars {
                            if c.keywords.contains(&KeywordAbility::Decayed) {
                                if let Some(mut obj_mut) = state.objects.get_mut(attacker_id) {
                                    obj_mut.decayed_sacrifice_at_eoc = true;
                                }
                            }
                        }
                    }
```

**WAIT**: The `state` in `check_triggers` is `&GameState`, not `&mut GameState`.
The function signature is `pub fn check_triggers(state: &GameState, ...) -> Vec<PendingTrigger>`.
It cannot mutate state.

**Revised approach**: The tagging must happen at the site where `AttackersDeclared`
is processed and state is mutable. Look at where `DeclareAttackers` command is
handled in `engine.rs` or `combat.rs`.

Let me trace the call chain:
1. `Command::DeclareAttackers` is handled in `engine.rs` which calls `combat::handle_declare_attackers`
2. `handle_declare_attackers` mutates state (setting combat attackers, tapping creatures)
3. After that, `check_triggers` is called and triggers are flushed

The tagging should happen in `handle_declare_attackers` in `combat.rs`, right after
the attackers are registered and before triggers are checked. This is the same place
where the creature is marked as attacking.

**Revised file**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Revised action**: At the end of `handle_declare_attackers`, after all attackers
are registered in `combat.attackers` and tapped, iterate over the declared attackers
and tag any with decayed.

Add (at the end of `handle_declare_attackers`, before the function returns):
```rust
    // CR 702.147a: Tag creatures with decayed for EOC sacrifice.
    // "When this creature attacks, sacrifice it at end of combat."
    // Must be tagged here (when state is mutable) rather than in check_triggers
    // (which receives &GameState). The tag persists even if decayed is removed
    // later (ruling 2021-09-24).
    for (attacker_id, _) in &attackers {
        let has_decayed = super::layers::calculate_characteristics(state, *attacker_id)
            .map(|c| c.keywords.contains(&KeywordAbility::Decayed))
            .unwrap_or(false);
        if has_decayed {
            if let Some(obj) = state.objects.get_mut(attacker_id) {
                obj.decayed_sacrifice_at_eoc = true;
            }
        }
    }
```

**Placement**: This must go after the attackers loop that taps creatures and before
the events are returned. Check the exact end of `handle_declare_attackers` for the
right insertion point.

### Step 5: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/decayed.rs`
**Pattern**: Combine patterns from `crates/engine/tests/shadow.rs` (blocking restriction)
and `crates/engine/tests/myriad.rs` (EOC behavior).

**Imports**:
```rust
use mtg_engine::{
    process_command, AttackTarget, CardRegistry, Command, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
```

**Helpers**: Same `find_object`, `pass_all`, `pass_until_advance` as in `myriad.rs`.

**Tests to write**:

1. `test_702_147_decayed_creature_cannot_block`
   - CR 702.147a: A creature with decayed can't block.
   - Setup: P2 has a creature with Decayed on the battlefield. P1 has an attacker.
     State at DeclareBlockers step with P1's creature as a declared attacker.
   - Assert: `DeclareBlockers` with the decayed creature as blocker returns `Err`.
   - Pattern: Follow `test_702_28_shadow_creature_cannot_be_blocked_by_non_shadow`
     but the restriction is on the BLOCKER (decayed creature can't block anything),
     not on the attacker-blocker pairing.

2. `test_702_147_decayed_creature_can_attack`
   - CR 702.147a ruling: Decayed does not prevent attacking.
   - Setup: P1 has a creature with Decayed. State at DeclareAttackers.
   - Assert: `DeclareAttackers` with the decayed creature succeeds.
   - Verify the creature is registered in combat.attackers.

3. `test_702_147_decayed_creature_sacrificed_at_eoc`
   - CR 702.147a: "When this creature attacks, sacrifice it at end of combat."
   - Setup: P1 attacks P2 with a decayed creature. Advance through declare blockers
     (no blockers), combat damage, to end of combat.
   - Assert: After EOC step, the creature is in the graveyard, not on the battlefield.
   - Assert: `CreatureDied` event was emitted.
   - Pattern: Follow `test_myriad_tokens_exiled_at_end_of_combat` but check for
     graveyard instead of exile.

4. `test_702_147_decayed_sacrifice_persists_after_losing_keyword`
   - Ruling 2021-09-24: "Once a creature with decayed attacks, it will be sacrificed
     at end of combat, even if it no longer has decayed at that time."
   - Setup: P1 attacks with a decayed creature. After attack declaration, verify
     `decayed_sacrifice_at_eoc` is set. Manually remove the Decayed keyword from
     the creature (simulating a removal effect). Advance to EOC.
   - Assert: Creature is still sacrificed despite losing the keyword.
   - This test validates the flag-based approach (tag at attack, not re-check at EOC).

5. `test_702_147_decayed_creature_not_sacrificed_if_not_attacking`
   - Ruling 2021-09-24: "Decayed does not create any attacking requirements."
   - Setup: P1 has a decayed creature but does NOT declare it as an attacker.
     Advance through the combat phase to EOC.
   - Assert: The creature is still on the battlefield (not sacrificed).

6. `test_702_147_decayed_flag_set_on_attack`
   - Implementation detail: Verify that `decayed_sacrifice_at_eoc` is set to `true`
     on the creature object after `DeclareAttackers`.
   - Setup: P1 declares the decayed creature as attacker.
   - Assert: `state.objects.get(&attacker_id).unwrap().decayed_sacrifice_at_eoc == true`.

7. `test_702_147_non_decayed_creature_can_block`
   - Baseline: A creature without decayed can block normally.
   - Setup: Normal blocking scenario, neither creature has decayed.
   - Assert: `DeclareBlockers` succeeds.

8. `test_702_147_decayed_no_haste`
   - Ruling 2021-09-24: "Decayed does not grant haste."
   - Setup: P1 has a decayed creature that just entered the battlefield this turn
     (has summoning sickness, no haste).
   - Assert: `DeclareAttackers` with this creature fails due to summoning sickness.

### Step 6: Card Definition (later phase)

**Suggested card**: Hobbling Zombie
- Mana cost: {2}{B}
- Type: Creature -- Zombie
- Oracle text: "Deathtouch. When this creature dies, create a 2/2 black Zombie
  creature token with decayed."
- P/T: 2/2
- Keywords: [Deathtouch]
- Creates a decayed token on death -- tests both the keyword on a token and
  die-trigger interaction.
- **Card lookup**: use `card-definition-author` agent with "Hobbling Zombie"

Alternative simpler card for keyword-only testing: No vanilla creature with just
Decayed exists in printed MTG. All cards that produce decayed creatures do so via
token creation. For unit tests, use `ObjectSpec::creature().with_keyword(Decayed)`.

### Step 7: Game Script (later phase)

**Suggested scenario**: Decayed token attacks, deals combat damage, then is
sacrificed at end of combat. Tests the full lifecycle:
1. A creature creates a 2/2 Zombie token with decayed (via ETB trigger)
2. Token survives to next turn (summoning sickness)
3. Token attacks an opponent
4. Token deals combat damage
5. Token is sacrificed at end of combat
6. Token goes to graveyard (briefly) then ceases to exist (SBA for tokens)

**Subsystem directory**: `test-data/generated-scripts/combat/`

## Interactions to Watch

- **Replacement effects on sacrifice (CR 614)**: When a decayed creature is
  sacrificed, replacement effects like Rest in Peace (exile instead of graveyard)
  or commander zone redirect SBA apply. The `end_combat()` sacrifice must use
  `check_zone_change_replacement` just like the Evoke sacrifice does in
  `resolution.rs`. This is handled in Step 3 above.

- **Tokens ceasing to exist (CR 704.5d)**: If the decayed creature is a token,
  it briefly enters the graveyard (triggering "when this dies" abilities) then
  ceases to exist as an SBA. This is standard token behavior, already implemented.
  No special handling needed.

- **Indestructible**: Sacrifice is not destruction (CR 701.17a). Indestructible
  does not prevent sacrifice. The `end_combat()` sacrifice moves directly to
  graveyard (via `move_object_to_zone`), bypassing destruction checks entirely.

- **Myriad + Decayed**: A creature with both Myriad and Decayed would: (a) create
  token copies on attack (myriad), (b) be tagged for EOC sacrifice (decayed).
  The myriad tokens would NOT have decayed unless the source also has decayed
  (they copy the source's characteristics). At EOC: myriad tokens are exiled,
  decayed creature is sacrificed. Order doesn't matter since both happen in the
  same TBA. No interaction issue.

- **Multiplayer**: Standard multiplayer combat. No special considerations.

- **Layer system**: Decayed is a keyword ability. It can be granted or removed by
  continuous effects in Layer 6. `calculate_characteristics` handles this
  generically. The blocking restriction and attack tagging both check the keyword
  through `calculate_characteristics`, which correctly reflects layer-applied
  modifications.

- **Die triggers**: The `CreatureDied` event emitted in `end_combat()` will fire
  any "when this creature dies" triggers on the sacrificed creature. This is correct
  per CR ("sacrifice" = "put into graveyard from the battlefield" = "dies").

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Decayed` |
| `crates/engine/src/state/game_object.rs` | Add `decayed_sacrifice_at_eoc: bool` |
| `crates/engine/src/state/mod.rs` | Add flag reset in 2 zone-change sites |
| `crates/engine/src/state/builder.rs` | Add flag init in object construction |
| `crates/engine/src/state/hash.rs` | Add discriminant + flag hash |
| `crates/engine/src/effects/mod.rs` | Add flag init in token construction |
| `crates/engine/src/rules/resolution.rs` | Add flag init in myriad token construction |
| `crates/engine/src/rules/combat.rs` | Add blocking restriction + attack tagging |
| `crates/engine/src/rules/turn_actions.rs` | Add EOC sacrifice in `end_combat()` |
| `tools/replay-viewer/src/view_model.rs` | Add keyword display arm |
| `crates/engine/tests/decayed.rs` | New test file with 8 tests |
