# Ability Plan: DeclareAttackers / DeclareBlockers Harness Actions

**Generated**: 2026-02-26
**CR**: 508.1 (DeclareAttackers), 509.1 (DeclareBlockers)
**Priority**: P1
**Similar abilities studied**: `cycle_card`, `cast_spell_flashback`, `choose_dredge` in `replay_harness.rs:256-269` (all are harness action types that translate script JSON to engine Commands)

## CR Rule Text

### CR 508.1 (DeclareAttackers)

508.1. First, the active player declares attackers. This turn-based action doesn't use the stack. To declare attackers, the active player follows the steps below, in order. If at any point during the declaration of attackers, the active player is unable to comply with any of the steps listed below, the declaration is illegal; the game returns to the moment before the declaration (see rule 732, "Handling Illegal Actions").

508.1a The active player chooses which creatures that they control, if any, will attack. The chosen creatures must be untapped, they can't also be battles, and each one must either have haste or have been controlled by the active player continuously since the turn began.

508.1b If the defending player controls any planeswalkers, is the protector of any battles, or the game allows the active player to attack multiple other players, the active player announces which player, planeswalker, or battle each of the chosen creatures is attacking.

508.1f The active player taps the chosen creatures. Tapping a creature when it's declared as an attacker isn't a cost; attacking simply causes creatures to become tapped.

508.1k Each chosen creature still controlled by the active player becomes an attacking creature.

508.1m Any abilities that trigger on attackers being declared trigger.

### CR 509.1 (DeclareBlockers)

509.1. First, the defending player declares blockers. This turn-based action doesn't use the stack.

509.1a The defending player chooses which creatures they control, if any, will block. The chosen creatures must be untapped and they can't also be battles. For each of the chosen creatures, the defending player chooses one creature for it to block that's attacking that player, a planeswalker they control, or a battle they protect.

509.1g Each chosen creature still controlled by the defending player becomes a blocking creature.

509.1h An attacking creature with one or more creatures declared as blockers for it becomes a blocked creature; one with no creatures declared as blockers for it becomes an unblocked creature.

509.1i Any abilities that trigger on blockers being declared trigger.

## Key Edge Cases

- **Summoning sickness (CR 302.6 / 508.1a)**: Creatures without haste that haven't been controlled since the turn began cannot attack. The engine already checks this in `handle_declare_attackers` at `combat.rs:112-117`.
- **Vigilance (CR 508.1f)**: Creatures with Vigilance do not tap when attacking. Already handled at `combat.rs:101,106-108`.
- **Defender (CR 702.3a)**: Creatures with Defender cannot attack. Already checked at `combat.rs:94-99`.
- **Multiplayer attack targets (CR 508.1b)**: In Commander, the active player can attack any opponent. Each attacker needs an `AttackTarget` specifying which opponent. The harness must map player names to `AttackTarget::Player(PlayerId)`.
- **Empty attacker/blocker lists are legal**: Declaring zero attackers or zero blockers is valid (the step just passes).
- **Multiple defending players (CR 509.1)**: In multiplayer, each defending player declares blockers independently. `CombatState::defenders_declared` tracks who has declared. The harness will need to handle one `declare_blockers` action per defending player.
- **Evasion (Flying, Menace, Intimidate, Landwalk, "can't be blocked")**: All enforced in `handle_declare_blockers` at `combat.rs`. Scripts exercising evasion will rely on the engine rejecting illegal blocks.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- N/A, `Command::DeclareAttackers` and `Command::DeclareBlockers` already exist at `command.rs:86-101`
- [ ] Step 2: Rule enforcement -- N/A, `handle_declare_attackers` and `handle_declare_blockers` already exist at `combat.rs:34-180` and `combat.rs:312-380`
- [ ] Step 3: Trigger wiring -- N/A, triggers already fire for attacker/blocker declarations
- [ ] Step 4: Harness action types -- **THE GAP**: `translate_player_action` in `replay_harness.rs:190-316` does not handle `"declare_attackers"` or `"declare_blockers"` strings
- [ ] Step 5: Card definition -- use existing cards (e.g., Grizzly Bears, Llanowar Elves)
- [ ] Step 6: Game script -- rewrite combat scripts to use real commands instead of informational `turn_based_action`
- [ ] Step 7: Coverage doc update

## Implementation Steps

### Step 1: Add Schema Fields to `PlayerAction` (script_schema.rs)

**File**: `crates/engine/src/testing/script_schema.rs`
**Action**: Add two new optional fields to the `ScriptAction::PlayerAction` variant for structured attacker/blocker data.

Add after `ability_index` (line ~207):

```rust
/// For `declare_attackers`: list of (creature_name, attack_target_player) pairs.
/// Each entry declares one creature as attacking one player.
/// Example: [{"card": "Grizzly Bears", "target_player": "p2"}]
#[serde(default)]
attackers: Vec<AttackerDeclaration>,

/// For `declare_blockers`: list of (blocker_name, attacker_name) pairs.
/// Each entry declares one creature as blocking one attacker.
/// Example: [{"card": "Llanowar Elves", "blocking": "Grizzly Bears"}]
#[serde(default)]
blockers: Vec<BlockerDeclaration>,
```

Add new structs at the bottom of the file (after `ManaSource`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttackerDeclaration {
    /// Name of the attacking creature (must be on the battlefield under the player's control).
    pub card: String,
    /// Name of the player being attacked (e.g., "p2").
    /// For planeswalker attacks, use `target_planeswalker` instead.
    pub target_player: Option<String>,
    /// Name of the planeswalker being attacked (on the battlefield).
    /// Mutually exclusive with `target_player`.
    pub target_planeswalker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockerDeclaration {
    /// Name of the blocking creature (must be on the battlefield under the player's control).
    pub card: String,
    /// Name of the attacking creature being blocked.
    pub blocking: String,
}
```

**Rationale**: Using dedicated structs instead of overloading `ActionTarget` keeps the schema clean and self-documenting. The `serde(default)` on the Vec fields means all existing scripts continue to parse without changes (empty vecs for scripts that don't use these fields).

### Step 2: Update `translate_player_action` (replay_harness.rs)

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add `"declare_attackers"` and `"declare_blockers"` arms to `translate_player_action`.

**Signature change**: The function needs access to the new `attackers` and `blockers` fields from the `PlayerAction` variant. The current signature passes individual fields. Two options:

**Option A (minimal change)**: Pass the new fields as additional parameters. This adds two more parameters to the function signature.

**Option B (recommended)**: Since the function signature is already large (7 params), refactor to accept a struct or pass the fields via a new helper. However, to minimize churn, Option A is simpler.

The `script_replay.rs` call site (line ~135-152) already destructures `PlayerAction` fields. Add `attackers` and `blockers` to the destructure pattern and pass them through.

**Changes to `translate_player_action` signature** (at `replay_harness.rs:190`):

Add two new parameters:
```rust
pub fn translate_player_action(
    action: &str,
    player: PlayerId,
    card_name: Option<&str>,
    ability_index: usize,
    targets: &[ActionTarget],
    attackers_decl: &[AttackerDeclaration],  // NEW
    blockers_decl: &[BlockerDeclaration],    // NEW
    state: &GameState,
    players: &HashMap<String, PlayerId>,
) -> Option<Command> {
```

**New match arm for `"declare_attackers"`** (insert at line ~270, after `"choose_dredge"`):

```rust
// CR 508.1: Declare attackers. Resolve creature names to ObjectIds on the
// battlefield, and player names to AttackTarget::Player.
"declare_attackers" => {
    let mut atk_pairs: Vec<(crate::state::ObjectId, crate::state::combat::AttackTarget)> = Vec::new();
    for decl in attackers_decl {
        let obj_id = find_on_battlefield(state, player, &decl.card)?;
        let target = if let Some(ref pname) = decl.target_player {
            let &pid = players.get(pname)?;
            crate::state::combat::AttackTarget::Player(pid)
        } else if let Some(ref pw_name) = decl.target_planeswalker {
            let pw_id = find_on_battlefield_by_name(state, pw_name)?;
            crate::state::combat::AttackTarget::Planeswalker(pw_id)
        } else {
            // Default: attack the first non-active player (for 2-player scripts).
            let target_pid = players.values()
                .find(|&&pid| pid != player)
                .copied()?;
            crate::state::combat::AttackTarget::Player(target_pid)
        };
        atk_pairs.push((obj_id, target));
    }
    Some(Command::DeclareAttackers {
        player,
        attackers: atk_pairs,
    })
}
```

**New match arm for `"declare_blockers"`**:

```rust
// CR 509.1: Declare blockers. Resolve creature names to ObjectIds on the
// battlefield.
"declare_blockers" => {
    let mut blk_pairs: Vec<(crate::state::ObjectId, crate::state::ObjectId)> = Vec::new();
    for decl in blockers_decl {
        let blocker_id = find_on_battlefield(state, player, &decl.card)?;
        let attacker_id = find_on_battlefield_by_name(state, &decl.blocking)?;
        blk_pairs.push((blocker_id, attacker_id));
    }
    Some(Command::DeclareBlockers {
        player,
        blockers: blk_pairs,
    })
}
```

**New helper function** `find_on_battlefield_by_name` (add near line ~634, after `find_on_battlefield`):

```rust
/// Find any object on the battlefield by name, regardless of controller.
/// Used for resolving attacker names when declaring blockers (the blocker's
/// controller is the declaring player, but the attacker is controlled by
/// the opponent).
fn find_on_battlefield_by_name(
    state: &GameState,
    name: &str,
) -> Option<crate::state::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Battlefield {
            Some(id)
        } else {
            None
        }
    })
}
```

**Import**: Add `AttackerDeclaration` and `BlockerDeclaration` to the import from `script_schema` at `replay_harness.rs:20`:

```rust
use crate::testing::script_schema::{ActionTarget, AttackerDeclaration, BlockerDeclaration, InitialState};
```

**Also import**: `AttackTarget` from `crate::state::combat`. Check if it's already in scope. Currently the imports at line 22-26 include `Command` but not `AttackTarget`. Add it.

### Step 3: Update Call Site in `script_replay.rs`

**File**: `crates/engine/tests/script_replay.rs`
**Action**: Update the `ScriptAction::PlayerAction` match arm (line ~135) to destructure the new `attackers` and `blockers` fields and pass them to `translate_player_action`.

Current destructure pattern (line 135-142):
```rust
ScriptAction::PlayerAction {
    player,
    action,
    card,
    targets,
    ability_index,
    ..
} => {
```

Change to:
```rust
ScriptAction::PlayerAction {
    player,
    action,
    card,
    targets,
    ability_index,
    attackers,
    blockers,
    ..
} => {
```

And update the `translate_player_action` call (line 144-152) to pass the new fields:
```rust
let cmd = translate_player_action(
    action.as_str(),
    pid,
    card.as_deref(),
    *ability_index as usize,
    targets,
    attackers,   // NEW
    blockers,    // NEW
    &state,
    &players,
);
```

**Also update the import** at line 31: `translate_player_action` is already imported, but `AttackerDeclaration` and `BlockerDeclaration` don't need to be imported here since `script_replay.rs` only destructures the `ScriptAction` enum (serde handles the types).

### Step 4: Unit Tests

**File**: `crates/engine/tests/combat_harness.rs` (new file)
**Tests to write**:

These tests programmatically construct `GameScript` structs (like the existing harness tests in `script_replay.rs:502-828`) and run them through `replay_script()` to verify the new action types work end-to-end.

- **`test_harness_declare_attackers_basic`** -- CR 508.1: Script starts at `declare_attackers` step, p1 declares one creature attacking p2, all pass through to combat damage, assert p2 life decreased.
  - Initial state: p1 has a 2/2 creature, p2 has no creatures, phase: `declare_attackers`
  - Script: `player_action(declare_attackers, attackers: [{card: "Bear", target_player: "p2"}])` -> priority passes -> declare_blockers (empty) -> priority passes -> assert p2 life == 38
  - Pattern: Follow `test_510_unblocked_attacker_deals_damage_to_player` in `tests/combat.rs:33-102`

- **`test_harness_declare_blockers_basic`** -- CR 509.1: Script with p2 blocking p1's attacker, assert no player damage and blocker takes damage.
  - Initial state: p1 has 5/5 creature, p2 has 1/1 creature, phase: `declare_attackers`
  - Script: declare attackers -> pass -> declare blockers -> pass -> assert p2 life == 40, blocker in graveyard

- **`test_harness_declare_attackers_empty`** -- CR 508.1: Declaring zero attackers is legal. Combat phase advances normally with no damage.
  - Script: declare attackers with empty attackers list -> pass -> assert p2 life unchanged

- **`test_harness_declare_blockers_empty`** -- CR 509.1: Declaring zero blockers is legal. Unblocked attacker deals damage.
  - Script: declare attackers (1 creature) -> pass -> declare blockers (empty) -> pass -> assert p2 life decreased

- **`test_harness_declare_attackers_vigilance_not_tapped`** -- CR 508.1f / CR 702.20: Creature with Vigilance remains untapped after attacking.
  - Script: declare attacker with Vigilance -> pass -> assert permanent.Bear.tapped == false

- **`test_harness_declare_attackers_multiple_creatures`** -- Two creatures attacking same player.
  - Script: declare two attackers -> pass through to damage -> assert combined damage dealt

**Pattern**: Follow the programmatic test style in `tests/script_replay.rs:502-828`. Build a `GameScript` struct in Rust, call `replay_script()`, and check results.

**Alternatively** (preferred for integration): Write JSON script files in `test-data/generated-scripts/combat/` and rely on `run_all_scripts` to execute them. The unit tests above are a safety net; the JSON scripts are the real validation.

### Step 5: Card Definition

No new card definitions needed. Existing cards suffice:
- **Grizzly Bears** (2/2 creature) -- simple attacker
- **Llanowar Elves** (1/1 creature) -- simple blocker
- **Elvish Mystic** (1/1 creature) -- second blocker for multi-block tests
- **Darksteel Colossus** (11/11, indestructible) -- big attacker for trample scripts

Verify these exist in `crates/engine/src/cards/definitions.rs`:
```
Grep "Grizzly Bears\|Llanowar Elves\|Elvish Mystic" crates/engine/src/cards/definitions.rs
```

### Step 6: Game Script

**Suggested script**: `test-data/generated-scripts/combat/015_declare_attackers_basic.json`

**Scenario**: p1 has a Grizzly Bears (2/2) on the battlefield. p1 declares Grizzly Bears as an attacker targeting p2. p2 has no creatures and declares no blockers. After combat damage, p2 loses 2 life (40 -> 38).

JSON structure:
```json
{
  "schema_version": "1.0.0",
  "metadata": {
    "id": "script_combat_015",
    "name": "Declare attackers harness action -- unblocked 2/2 deals damage",
    "description": "p1 declares Grizzly Bears as attacker targeting p2 via the harness declare_attackers action. p2 declares no blockers. After combat damage, p2 life is 38.",
    "cr_sections_tested": ["508.1", "509.1", "510.1"],
    "tags": ["combat", "declare-attackers", "declare-blockers", "harness-action"],
    "confidence": "high",
    "review_status": "pending_review"
  },
  "initial_state": {
    "format": "commander",
    "turn_number": 5,
    "active_player": "p1",
    "phase": "declare_attackers",
    "players": {
      "p1": { "life": 40, "mana_pool": {}, "land_plays_remaining": 0 },
      "p2": { "life": 40, "mana_pool": {}, "land_plays_remaining": 0 }
    },
    "zones": {
      "battlefield": {
        "p1": [{ "card": "Grizzly Bears", "tapped": false }],
        "p2": []
      }
    }
  },
  "script": [
    {
      "step": "declare_attackers",
      "step_note": "p1 declares Grizzly Bears attacking p2",
      "actions": [
        {
          "type": "assert_state",
          "description": "Initial: Grizzly Bears untapped, p2 at 40 life",
          "assertions": {
            "zones.battlefield.p1": { "includes": [{"card": "Grizzly Bears"}] },
            "players.p2.life": 40
          }
        },
        {
          "type": "player_action",
          "player": "p1",
          "action": "declare_attackers",
          "attackers": [
            { "card": "Grizzly Bears", "target_player": "p2" }
          ],
          "cr_ref": "508.1",
          "note": "p1 declares Grizzly Bears as attacker against p2"
        },
        {
          "type": "priority_round",
          "players": ["p1", "p2"],
          "result": "all_pass",
          "note": "All pass in declare attackers step -> advance to declare blockers"
        },
        {
          "type": "player_action",
          "player": "p2",
          "action": "declare_blockers",
          "blockers": [],
          "cr_ref": "509.1",
          "note": "p2 declares no blockers"
        },
        {
          "type": "priority_round",
          "players": ["p1", "p2"],
          "result": "all_pass",
          "note": "All pass in declare blockers step -> advance to combat damage"
        },
        {
          "type": "assert_state",
          "description": "After combat damage: p2 lost 2 life, Grizzly Bears tapped",
          "assertions": {
            "players.p2.life": 38,
            "permanent.Grizzly Bears.tapped": true
          }
        }
      ]
    }
  ]
}
```

**Additional script**: `test-data/generated-scripts/combat/016_declare_blockers_creature_dies.json`
- p1 attacks with 3/3, p2 blocks with 2/2. Both take damage. 2/2 dies. p2 life unchanged.

**Subsystem directory**: `test-data/generated-scripts/combat/`

### Step 7: Update ability-wip.md and Coverage Doc

After implementation, update:
- `memory/ability-wip.md` -- check off steps
- `docs/mtg-engine-ability-coverage.md` -- change "Declare attackers action" and "Declare blockers action" from `partial` to `validated`
- Remove the "Declare attackers/blockers harness action" entry from the P1 Gaps section

## Interactions to Watch

### Phase/Step Initialization
- Scripts using `"phase": "declare_attackers"` will start with `Step::DeclareAttackers` and `combat: None`. The engine's `handle_declare_attackers` creates `CombatState` if missing. This matches what the unit tests in `tests/combat.rs` do (they use `.at_step(Step::DeclareAttackers)` which also leaves `combat: None`).

### Priority After Declaration
- After `DeclareAttackers`, the engine grants priority to the active player. Scripts need a `priority_round` (all players pass) to advance to `DeclareBlockers`.
- After `DeclareBlockers`, same pattern -- priority round to advance to `CombatDamage`.

### Summoning Sickness
- The engine checks `has_summoning_sickness` on objects. Objects created by `build_initial_state` do NOT have summoning sickness set (it's false by default). This matches the assumption that creatures in the initial state have been there since a prior turn. Set `turn_number >= 2` in scripts to be safe.

### Object Identity Across Steps
- ObjectIds remain stable across steps within a single combat phase (no zone changes happen between declare attackers and declare blockers unless a creature is killed by a trigger). The blocker resolution can safely use names to find the same objects that were declared as attackers.

### Multiplayer Blocker Declarations
- In a 4-player game, each defending player (all non-active) may declare blockers independently. The harness would need one `declare_blockers` action per defending player. For the initial implementation, 2-player scripts are sufficient. Multiplayer can be added later.

### Existing Combat Scripts
- The 14 existing combat scripts (`001`-`014`) use `turn_based_action` for all combat steps. They will NOT be broken by this change (they don't use `player_action` with `declare_attackers`). They can optionally be rewritten to use the new action types, but this is not required for the initial implementation.
- Script `011` (Audacious Thief) and `013` (Scroll Thief) have filed disputes noting the harness gap. After this implementation, new scripts can replace them with engine-executable combat.

## File Inventory

| File | Action | Lines Changed (est.) |
|------|--------|---------------------|
| `crates/engine/src/testing/script_schema.rs` | Add `AttackerDeclaration`, `BlockerDeclaration` structs; add `attackers`, `blockers` fields to `PlayerAction` | ~25 new |
| `crates/engine/src/testing/replay_harness.rs` | Add `declare_attackers`, `declare_blockers` arms to `translate_player_action`; add `find_on_battlefield_by_name` helper; update imports and signature | ~50 new |
| `crates/engine/tests/script_replay.rs` | Update `PlayerAction` destructure and `translate_player_action` call | ~5 changed |
| `crates/engine/tests/combat_harness.rs` | New file with 4-6 programmatic tests | ~200 new |
| `test-data/generated-scripts/combat/015_*.json` | New script: basic declare attackers | ~80 new |
| `test-data/generated-scripts/combat/016_*.json` | New script: declare blockers with creature death | ~90 new |
| `docs/mtg-engine-ability-coverage.md` | Update status from `partial` to `validated` | ~4 changed |
| `memory/ability-wip.md` | Check off completed steps | ~7 changed |
