# Ability Plan: Landwalk

**Generated**: 2026-02-25
**CR**: 702.14
**Priority**: P1
**Similar abilities studied**: Flying (CR 702.9) in `rules/combat.rs:427-439`, Intimidate (CR 702.13) in `rules/combat.rs:453-473`, CantBeBlocked in `rules/combat.rs:441-451`, Menace (CR 702.110) in `rules/combat.rs:485-512`

## CR Rule Text

```
702.14. Landwalk

702.14a Landwalk is a generic term that appears within an object's rules text as
"[type]walk," where [type] is usually a land type, but it can also be the card
type land plus any combination of land types, card types, and/or supertypes.

702.14b Landwalk is an evasion ability.

702.14c A creature with landwalk can't be blocked as long as the defending player
controls at least one land with the specified land type (as in "islandwalk"), with
the specified type or supertype (as in "artifact landwalk"), without the specified
type or supertype (as in "nonbasic landwalk"), or with both the specified type or
supertype and the specified subtype (as in "snow swampwalk"). (See rule 509,
"Declare Blockers Step.")

702.14d Landwalk abilities don't "cancel" one another.

702.14e Multiple instances of the same kind of landwalk on the same creature are
redundant.
```

## Key Edge Cases

- **702.14a**: Landwalk is generic -- each variant specifies a land type or combination.
  The five basic landwalks (plainswalk, islandwalk, swampwalk, mountainwalk, forestwalk)
  check for the corresponding basic land subtype. More exotic variants like "nonbasic
  landwalk" or "artifact landwalk" check for supertypes/card types instead.
- **702.14c**: The check is against the **defending player's** controlled lands, not all
  lands on the battlefield. In multiplayer, each attacker targets a specific defending
  player -- the landwalk check must query that specific player's lands.
- **702.14c**: "at least one land" -- only one qualifying land is needed, not all lands.
- **702.14d**: Landwalk abilities don't cancel each other. If creature A has swampwalk and
  creature B has swampwalk, they don't interact. This is automatic with our implementation
  since each creature checks independently.
- **702.14e**: Multiple instances of the same landwalk are redundant -- no stacking. This
  is automatic since `OrdSet<KeywordAbility>` deduplicates.
- **Nonbasic landwalk** (e.g., Dryad Sophisticate): checks if defending player controls
  a land WITHOUT the `Basic` supertype. This is an inversion of the normal check.
- **Multiplayer**: Each attacker attacks a specific player. Landwalk check must use the
  `AttackTarget` to determine which player's lands to inspect, not a global "all defenders."
- **Dual lands / Type-changing effects**: A Breeding Pool (Forest Island) satisfies both
  forestwalk and islandwalk. Blood Moon turning nonbasic lands into Mountains satisfies
  mountainwalk. The check uses `calculate_characteristics` to get the land's current
  subtypes (post-layer), not printed types.
- **Protection does not interact with landwalk**: Landwalk is a blocking restriction, not
  targeting. Protection from the attacker's quality doesn't prevent landwalk from making
  the attacker unblockable.

## Current State (from ability-wip.md)

- [ ] Step 1: Enum variant -- `KeywordAbility::Landwalk` exists at `state/types.rs:115`
  but is a bare variant with no associated data. Needs to be changed to
  `Landwalk(LandwalkType)` to carry the land type specification.
- [ ] Step 2: Rule enforcement -- no blocking check in `rules/combat.rs`
- [ ] Step 3: Trigger wiring -- N/A (landwalk is a static evasion ability, not a trigger)
- [ ] Step 4: Unit tests -- none exist
- [ ] Step 5: Card definition
- [ ] Step 6: Game script
- [ ] Step 7: Coverage doc update

## Design Decision: LandwalkType Enum

The bare `KeywordAbility::Landwalk` variant is insufficient -- it doesn't specify which
land type is walked. We need a new enum `LandwalkType` to capture the variety of landwalk
abilities.

For P1 scope, implement the five basic landwalks plus nonbasic landwalk (6 variants).
More exotic variants (artifact landwalk, snow swampwalk) can be added later as P3/P4.

```rust
/// Specifies which kind of landwalk ability (CR 702.14a).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LandwalkType {
    /// Checks if defending player controls a land with the given basic land subtype.
    /// Covers: Plainswalk, Islandwalk, Swampwalk, Mountainwalk, Forestwalk.
    BasicType(SubType),
    /// Nonbasic landwalk: checks if defending player controls a land WITHOUT
    /// the Basic supertype.
    Nonbasic,
}
```

The `KeywordAbility` variant becomes `Landwalk(LandwalkType)`.

## Implementation Steps

### Step 1: Enum Variant Refactoring

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**:
1. Add the `LandwalkType` enum after the `ProtectionQuality` enum (around line 75).
2. Change `Landwalk,` (line 115) to `Landwalk(LandwalkType),`.
3. Add doc comment citing CR 702.14a.

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Update the `HashInto for KeywordAbility` match arm at line 275. Change from:
```rust
KeywordAbility::Landwalk => 12u8.hash_into(hasher),
```
to:
```rust
KeywordAbility::Landwalk(lw_type) => {
    12u8.hash_into(hasher);
    lw_type.hash_into(hasher);
}
```
Also add a `HashInto for LandwalkType` implementation nearby (follow the
`HashInto for ProtectionQuality` pattern):
```rust
impl HashInto for LandwalkType {
    fn hash_into(&self, hasher: &mut Hasher) {
        match self {
            LandwalkType::BasicType(st) => {
                0u8.hash_into(hasher);
                st.hash_into(hasher);
            }
            LandwalkType::Nonbasic => 1u8.hash_into(hasher),
        }
    }
}
```

**File**: `/home/airbaggie/scutemob/crates/engine/src/lib.rs`
**Action**: Export `LandwalkType` alongside `SubType` in the `pub use` block (around line 24).

**Compile check**: After this step, `cargo check` will surface any other match arms or
references to the old bare `Landwalk` variant. Fix any that appear -- likely none outside
of `hash.rs` since the variant was never actually used in rules logic.

### Step 2: Rule Enforcement in Combat

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/combat.rs`
**Action**: Add a landwalk blocking restriction check in `handle_declare_blockers`, in the
per-blocker validation loop, after the protection check (line ~482) and before the closing
brace of the per-blocker loop (line ~483).

**Imports needed** (add to existing imports at top of file, line ~17):
```rust
use crate::state::types::{CardType, KeywordAbility, SubType, SuperType};
use crate::state::types::LandwalkType;
use crate::rules::layers::calculate_characteristics;
```
Note: `calculate_characteristics` may already be imported. Check before adding.

**Logic** (CR 702.14c):
```rust
// CR 702.14c: A creature with landwalk can't be blocked as long as the
// defending player controls at least one land with the specified type.
// The `player` variable is the defending player in this context.
for kw in attacker_chars.keywords.iter() {
    if let KeywordAbility::Landwalk(lw_type) = kw {
        let defender_has_matching_land = state.objects.values().any(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == player
                && {
                    let chars = calculate_characteristics(state, obj.id)
                        .unwrap_or_default();
                    chars.card_types.contains(&CardType::Land)
                        && match lw_type {
                            LandwalkType::BasicType(st) => chars.subtypes.contains(st),
                            LandwalkType::Nonbasic => {
                                !chars.supertypes.contains(&SuperType::Basic)
                            }
                        }
                }
        });
        if defender_has_matching_land {
            return Err(GameStateError::InvalidCommand(format!(
                "Object {:?} cannot block {:?} (attacker has {:?} landwalk; \
                 defending player controls a matching land)",
                blocker_id, attacker_id, lw_type
            )));
        }
    }
}
```

**Pattern**: Follows the Intimidate check at lines 453-473 which also inspects attacker
keywords and produces an `InvalidCommand` error.

**CR**: 702.14c -- "A creature with landwalk can't be blocked as long as the defending
player controls at least one land with the specified land type."

**Important**: The check uses `calculate_characteristics` on the defender's lands to get
post-layer subtypes (handles Blood Moon, Spreading Seas, etc.). It does NOT check
`obj.characteristics.subtypes` directly -- that would miss continuous effect modifications.

**Note on `Characteristics::default()`**: `calculate_characteristics` returns `Option<Characteristics>`.
If it returns `None` for a land object (shouldn't happen for battlefield objects), we use
`unwrap_or_default()` which gives empty types/subtypes -- safe, just means no match.

### Step 3: Trigger Wiring

**N/A** -- Landwalk is a static evasion ability (CR 702.14b). It does not trigger, does
not use the stack, and requires no wiring in `builder.rs` or the trigger system.

### Step 4: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/keywords.rs`
**Tests to write** (add after the Intimidate tests, around line 930):

1. **`test_702_14_swampwalk_unblockable_when_defender_controls_swamp`**
   - CR 702.14c -- creature with swampwalk can't be blocked if defending player controls a Swamp.
   - Setup: p1 has a 2/2 creature with `Landwalk(BasicType(SubType("Swamp")))`. p2 has a 2/2
     creature and a Swamp on the battlefield. Attacker attacks p2.
   - Assert: `DeclareBlockers` with p2's creature blocking the swampwalker returns `Err`.
   - Pattern: Follow `test_702_9_flying_cannot_be_blocked_by_ground` at line 241.

2. **`test_702_14_swampwalk_blockable_when_defender_has_no_swamp`**
   - CR 702.14c negative case -- creature with swampwalk CAN be blocked if defending player
     does NOT control a Swamp.
   - Setup: p1 has a 2/2 with swampwalk. p2 has a 2/2 creature and a Plains (no Swamp).
   - Assert: `DeclareBlockers` succeeds.
   - Pattern: Follow `test_702_17_reach_can_block_flying` at line 282.

3. **`test_702_14_islandwalk_unblockable_when_defender_controls_island`**
   - Same as test 1 but for islandwalk. Confirms the parameterized SubType check works for
     a different land type.
   - Setup: p1 has a 2/2 with islandwalk. p2 has a 2/2 creature and an Island.
   - Assert: `DeclareBlockers` returns `Err`.

4. **`test_702_14_landwalk_checks_defending_player_only`**
   - CR 702.14c multiplayer edge case. A third player controls a Swamp, but the defending
     player does not. Swampwalk should NOT prevent blocking.
   - Setup: 3 players. p1 has swampwalk creature attacking p2. p2 has a 2/2 creature and
     a Plains. p3 has a Swamp.
   - Assert: `DeclareBlockers` by p2 succeeds (p3's Swamp is irrelevant).
   - Pattern: Use 3-player builder.

5. **`test_702_14_nonbasic_landwalk_unblockable_when_defender_has_nonbasic`**
   - CR 702.14c variant for nonbasic landwalk (Dryad Sophisticate pattern).
   - Setup: p1 has a 2/1 with `Landwalk(Nonbasic)`. p2 has a 2/2 creature and a land
     WITHOUT the Basic supertype (e.g., a land card typed as just `Land` with no `Basic`
     supertype).
   - Assert: `DeclareBlockers` returns `Err`.

6. **`test_702_14_nonbasic_landwalk_blockable_when_all_lands_basic`**
   - Negative case for nonbasic landwalk. All of the defending player's lands are basic.
   - Setup: p1 has `Landwalk(Nonbasic)` creature. p2 has a 2/2 creature and a Plains
     (which has `SuperType::Basic`).
   - Assert: `DeclareBlockers` succeeds.

7. **`test_702_14_landwalk_plus_flying_both_checked`**
   - Edge case: creature has BOTH flying and swampwalk. Both restrictions apply
     independently (blocker needs flying/reach AND defending player must not have Swamp).
   - Setup: p1 has a creature with Flying + Swampwalk attacking p2. p2 has a ground
     creature and a Swamp.
   - Assert: `DeclareBlockers` returns `Err` (both evasion abilities independently prevent
     blocking -- either one is sufficient).

### Step 5: Card Definition (later phase)

**Suggested card**: Bog Raiders
- Oracle: "Swampwalk (This creature can't be blocked as long as defending player controls a Swamp.)"
- Type: Creature -- Zombie
- Mana cost: {2}{B}
- P/T: 2/2
- Keywords: `[KeywordAbility::Landwalk(LandwalkType::BasicType(SubType("Swamp".into())))]`
- Simple, vanilla creature with exactly one keyword. Ideal for validation.
- Card lookup confirms: Mana Cost: {2}{B}, Type: Creature -- Zombie, P/T: 2/2, Keywords: Landwalk, Swampwalk.

**File**: `/home/airbaggie/scutemob/crates/engine/src/cards/definitions.rs`
**Action**: Add Bog Raiders definition using the `card-definition-author` agent.

### Step 6: Game Script (later phase)

**Suggested scenario**: "Swampwalk evasion -- attacker is unblockable because defender controls a Swamp"
- p1 controls Bog Raiders (2/2 swampwalk) and a Swamp.
- p2 controls a generic 2/2 creature and a Swamp.
- p1 attacks p2 with Bog Raiders.
- p2 attempts to block -- engine rejects.
- p1 passes, p2 passes, combat damage deals 2 to p2.
- Assert: p2 life = 38 (40 - 2).

**Subsystem directory**: `test-data/generated-scripts/combat/`
**Suggested filename**: `058_swampwalk_evasion.json`

## Interactions to Watch

- **Layer system**: Landwalk checks must use `calculate_characteristics` on the defender's
  lands to get post-layer subtypes. Blood Moon turning all nonbasic lands into Mountains
  means mountainwalk becomes very powerful. Spreading Seas adding Island subtype enables
  islandwalk. These interactions work automatically as long as we use `calculate_characteristics`.
- **Protection (DEBT)**: Protection prevents blocking (the B in DEBT), but landwalk is a
  separate, independent blocking restriction. A creature with both protection from red AND
  swampwalk has two independent reasons it can't be blocked (by red creatures, or at all if
  defender has a Swamp). No special interaction code needed.
- **CantBeBlocked keyword**: A creature with both `CantBeBlocked` and landwalk -- redundant
  but harmless. `CantBeBlocked` is checked first (line 441) and short-circuits.
- **Menace interaction**: If a creature has both menace and landwalk, and the defender has
  the matching land, the creature can't be blocked at all (landwalk takes priority). If
  the defender does NOT have the matching land, menace still requires 2+ blockers. No
  special code needed -- both checks run independently.
- **Multiplayer**: Each attacker's `AttackTarget` identifies the specific defending player.
  The `player` parameter in the blocker loop is the defending player calling
  `DeclareBlockers`. The landwalk check inspects that player's controlled lands, NOT all
  opponents' lands. This is correct per CR 702.14c ("the defending player").
