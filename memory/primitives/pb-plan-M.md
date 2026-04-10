# Primitive Batch Plan: PB-M -- Panharmonicon Trigger Doubling

**Generated**: 2026-04-09
**Primitive**: Fix trigger doubling to handle SelfEntersBattlefield + create Panharmonicon card def + fix related cards
**CR Rules**: 603.2d (trigger doubling)
**Cards affected**: 4 (1 new + 3 existing fixes)
**Dependencies**: None (infrastructure already exists from M9.4)
**Deferred items from prior PBs**: SelfEntersBattlefield triggers not doubled (documented in MEMORY.md)

## Primitive Specification

The trigger doubling infrastructure is ALREADY BUILT (M9.4). However, there are two bugs
preventing Panharmonicon from working correctly as a card, and the card definition file
does not exist.

**Bug 1 (Critical)**: `doubler_applies_to_trigger()` in `abilities.rs` line 7252 only matches
`TriggerEvent::AnyPermanentEntersBattlefield`. It does NOT match `SelfEntersBattlefield`.
Per Panharmonicon ruling (2021-03-19): "Panharmonicon affects a permanent's own
enters-the-battlefield triggered abilities as well as other triggered abilities that
trigger when that permanent enters the battlefield." The fix is to also match
`SelfEntersBattlefield` in the `ArtifactOrCreatureETB` arm.

**Bug 2 (Critical)**: `queue_carddef_etb_triggers()` in `replacement.rs` line 1182 sets
`entering_object_id: None` for CardDefETB pending triggers. The `doubler_applies_to_trigger`
function requires `entering_object_id` to be `Some(id)` so it can check whether the entering
permanent is an artifact or creature. Without this, even with Bug 1 fixed, CardDef-based
self-ETB triggers will never be doubled. The fix is to set `entering_object_id: Some(new_id)`.

**New card**: Panharmonicon card definition file.

**Existing fixes**: Drivnod (already uses `CreatureDeath` filter but has a stale TODO),
Teysa Karlov (already works but has an unrelated TODO for token abilities),
Elesh Norn (partially fixable -- ETB doubling yes, opponent suppression no).

## CR Rule Text

**CR 603.2d**: An ability may state that a triggered ability triggers additional times. In
this case, rather than simply determining that such an ability has triggered, determine how
many times it should trigger, then that ability triggers that many times. An effect that
states that an ability triggers additional times doesn't invoke itself repeatedly and doesn't
apply to other effects that affect how many times an ability triggers. An effect that states
a triggered ability of an object triggers additional times refers only to triggered abilities
that object has, not to any delayed or reflexive triggered abilities (see rule 603.7 and
rule 603.12) that may be created by abilities the object has.

## Engine Changes

### Change 1: Fix `doubler_applies_to_trigger` to match SelfEntersBattlefield

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In the `ArtifactOrCreatureETB` arm of `doubler_applies_to_trigger()` (line ~7250),
change the `is_etb` match to also accept `SelfEntersBattlefield`:

```rust
let is_etb = matches!(
    trigger.triggering_event,
    Some(TriggerEvent::AnyPermanentEntersBattlefield)
        | Some(TriggerEvent::SelfEntersBattlefield)
);
```

**CR**: 603.2d + Panharmonicon ruling: "Panharmonicon affects a permanent's own
enters-the-battlefield triggered abilities as well as other triggered abilities
that trigger when that permanent enters the battlefield."

**Pattern**: Follow the `CreatureDeath` arm at line 7284 which already matches both
`SelfDies` and `AnyCreatureDies`.

Also remove the TODO comment at lines 7227-7230 about SelfEntersBattlefield not being doubled.

### Change 2: Set `entering_object_id` in `queue_carddef_etb_triggers`

**File**: `crates/engine/src/rules/replacement.rs`
**Action**: At line 1182, change `entering_object_id: None` to `entering_object_id: Some(new_id)`.
This ensures CardDef ETB triggers carry the entering permanent's ObjectId so that
`doubler_applies_to_trigger` can check its card types.

Also fix the second occurrence at line 1220 (TributeNotPaid path) the same way.

**CR**: 603.2d -- the doubler needs to know which permanent entered to verify its types.

### Change 3: Add `AnyPermanentETB` filter variant to `TriggerDoublerFilter`

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add a new variant after `ArtifactOrCreatureETB`:

```rust
/// "If a permanent entering causes a triggered ability" -- Yarok, Elesh Norn.
///
/// Doubles ETB triggered abilities from ANY permanent entering (not just artifacts/creatures).
/// CR 603.2d.
AnyPermanentETB,
```

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add a match arm in `doubler_applies_to_trigger()` for `AnyPermanentETB`:

```rust
TriggerDoublerFilter::AnyPermanentETB => {
    matches!(
        trigger.triggering_event,
        Some(TriggerEvent::AnyPermanentEntersBattlefield)
            | Some(TriggerEvent::SelfEntersBattlefield)
    )
}
```

This variant does NOT check entering object types -- any permanent entering doubles the trigger.

### Change 4: Add `LandETB` filter variant to `TriggerDoublerFilter`

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add a new variant:

```rust
/// "If a land entering causes a triggered ability" -- Ancient Greenwarden.
///
/// Doubles ETB triggered abilities only when a land enters the battlefield.
/// CR 603.2d.
LandETB,
```

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add a match arm in `doubler_applies_to_trigger()` for `LandETB`:

```rust
TriggerDoublerFilter::LandETB => {
    let is_etb = matches!(
        trigger.triggering_event,
        Some(TriggerEvent::AnyPermanentEntersBattlefield)
            | Some(TriggerEvent::SelfEntersBattlefield)
    );
    if !is_etb {
        return false;
    }
    let entering_id = match trigger.entering_object_id {
        Some(id) => id,
        None => return false,
    };
    let entering_chars =
        crate::rules::layers::calculate_characteristics(state, entering_id).or_else(|| {
            state.objects.get(&entering_id).map(|o| o.characteristics.clone())
        });
    entering_chars
        .map(|chars| chars.card_types.contains(&CardType::Land))
        .unwrap_or(false)
}
```

### Change 5: Exhaustive match updates for new `TriggerDoublerFilter` variants

Files requiring new match arms for `AnyPermanentETB` and `LandETB`:

| File | Match expression | Line | Action |
|------|-----------------|------|--------|
| `crates/engine/src/state/hash.rs` | `HashInto for TriggerDoublerFilter` | L1396 | Add `AnyPermanentETB => 2u8`, `LandETB => 3u8` |
| `crates/engine/src/rules/abilities.rs` | `doubler_applies_to_trigger` match | L7249 | Add arms (described in Changes 3+4) |

No other files match exhaustively on `TriggerDoublerFilter` -- the enum is only matched in
these two locations (hash.rs and abilities.rs).

## Card Definition Fixes

### panharmonicon.rs (NEW)

**Oracle text**: If an artifact or creature entering causes a triggered ability of a permanent
you control to trigger, that ability triggers an additional time.
**Fix**: Create new file `crates/engine/src/cards/defs/panharmonicon.rs`

```rust
// Panharmonicon -- {4}, Artifact
// If an artifact or creature entering causes a triggered ability of a permanent
// you control to trigger, that ability triggers an additional time.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("panharmonicon"),
        name: "Panharmonicon".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "If an artifact or creature entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.".to_string(),
        abilities: vec![
            // CR 603.2d: ETB trigger doubling for artifacts and creatures.
            AbilityDefinition::TriggerDoubling {
                filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
                additional_triggers: 1,
            },
        ],
        ..Default::default()
    }
}
```

### drivnod_carnage_dominus.rs

**Oracle text**: If a creature dying causes a triggered ability of a permanent you control to
trigger, that ability triggers an additional time. {B/P}{B/P}, Exile three creature cards
from your graveyard: Put an indestructible counter on Drivnod.
**Current state**: Empty abilities vec with two TODO comments. First TODO (death trigger
doubling) is now expressible. Second TODO (activated ability with phyrexian mana + exile from
GY + indestructible counter) remains a DSL gap.
**Fix**: Add `AbilityDefinition::TriggerDoubling { filter: TriggerDoublerFilter::CreatureDeath,
additional_triggers: 1 }` to abilities vec. Update the second TODO comment to note the first
ability is now implemented.

### elesh_norn_mother_of_machines.rs

**Oracle text**: Vigilance. If a permanent entering causes a triggered ability of a permanent
you control to trigger, that ability triggers an additional time. Permanents entering don't
cause abilities of permanents your opponents control to trigger.
**Current state**: Only has Vigilance keyword. Two TODO comments.
**Fix**: Add `AbilityDefinition::TriggerDoubling { filter: TriggerDoublerFilter::AnyPermanentETB,
additional_triggers: 1 }`. Keep TODO for opponent ETB suppression (requires controller-scoped
ETBSuppressor -- separate gap, not in PB-M scope).

### ancient_greenwarden.rs

**Oracle text**: Reach. You may play lands from your graveyard. If a land entering causes a
triggered ability of a permanent you control to trigger, that ability triggers an additional time.
**Current state**: Has Reach + StaticPlayFromGraveyard. Missing land-ETB trigger doubling.
**Fix**: Add `AbilityDefinition::TriggerDoubling { filter: TriggerDoublerFilter::LandETB,
additional_triggers: 1 }`. Remove the deferred TODO comment.

## Cards NOT Fixed in This Batch (Out of Scope)

These cards have trigger doubling TODOs but require additional primitives beyond what PB-M adds:

- **Isshin, Two Heavens as One**: Needs `AttackTrigger` TriggerDoublerFilter variant + no
  `AnyCreatureAttacks` TriggerEvent exists yet. Separate PB needed.
- **Windcrag Siege**: Needs both attack trigger doubling (Mardu mode) AND ETB mode choice.
  Two separate gaps.
- **Delney, Streetwise Lookout**: Needs power-filtered trigger doubling (all triggers from
  creatures with power <= 2). Very different filter mechanism -- possibly needs a
  `PowerFilteredCreature { max_power: i32 }` variant that checks the trigger source's power.
  Also needs blocking restriction. Separate PB.

## Unit Tests

**File**: `crates/engine/tests/trigger_doubling.rs`
**Tests to write** (add to existing file which already has 4 tests):

- `test_panharmonicon_doubles_self_etb_trigger` -- CR 603.2d + ruling: a creature with its
  own "when ~ enters" ability has that trigger doubled by Panharmonicon. Tests the
  SelfEntersBattlefield fix (Bug 1).
- `test_panharmonicon_doubles_carddef_etb_trigger` -- CR 603.2d: a card definition with
  `WhenEntersBattlefield` AbilityDefinition::Triggered has its ETB doubled. Tests the
  entering_object_id fix (Bug 2). Use a card with a CardDef ETB trigger (not an ObjectSpec
  triggered_ability).
- `test_panharmonicon_does_not_double_enchantment_etb` -- CR 603.2d: Panharmonicon only
  doubles artifact/creature ETBs. An enchantment entering should NOT cause doubling.
  Negative test.
- `test_any_permanent_etb_doubler_doubles_enchantment` -- CR 603.2d: `AnyPermanentETB`
  filter (Yarok/Elesh Norn pattern) doubles enchantment ETBs that `ArtifactOrCreatureETB`
  does not.
- `test_land_etb_doubler_doubles_landfall` -- CR 603.2d: `LandETB` filter (Ancient
  Greenwarden pattern) doubles triggers when a land enters but NOT when a creature enters.
- `test_death_trigger_doubler_teysa` -- CR 603.2d: Teysa Karlov's `CreatureDeath` filter
  doubles death triggers. (May already be covered by existing tests -- check before adding.)

**Pattern**: Follow existing tests in `trigger_doubling.rs` (use `panharmonicon_def()` helper,
`pass_all_four()`, `any_etb_trigger()` helpers).

## Verification Checklist

- [ ] Engine fixes compile (`cargo check`)
- [ ] SelfEntersBattlefield matched in ArtifactOrCreatureETB arm
- [ ] entering_object_id set in queue_carddef_etb_triggers (both paths)
- [ ] AnyPermanentETB variant added + hash + match arm
- [ ] LandETB variant added + hash + match arm
- [ ] panharmonicon.rs card def created
- [ ] drivnod_carnage_dominus.rs fixed (CreatureDeath doubling added)
- [ ] elesh_norn_mother_of_machines.rs fixed (AnyPermanentETB doubling added)
- [ ] ancient_greenwarden.rs fixed (LandETB doubling added)
- [ ] Unit tests pass (`cargo test --all`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Workspace builds (`cargo build --workspace`)
- [ ] No remaining TODOs for trigger doubling in affected card defs (except out-of-scope gaps)

## Risks & Edge Cases

- **Bug 2 interaction with ETB suppression**: When `entering_object_id` is set on CardDefETB
  triggers, verify that ETB suppressors still work correctly. The suppressor checks happen
  BEFORE trigger queueing (in `queue_carddef_etb_triggers` itself), so adding
  `entering_object_id` should not affect suppression logic.

- **Panharmonicon entering simultaneously with another artifact/creature** (ruling 2021-03-19):
  "If an artifact or creature entering the battlefield at the same time as Panharmonicon
  (including Panharmonicon itself) causes a triggered ability of a permanent you control to
  trigger, that ability triggers an additional time." This works because
  `register_static_continuous_effects` runs during resolution (before triggers are flushed),
  so Panharmonicon's TriggerDoubler is registered before `flush_pending_triggers` processes
  the ETB triggers.

- **Elesh Norn partial implementation**: Adding AnyPermanentETB doubling without the opponent
  suppression means Elesh Norn will double your triggers but NOT suppress opponents'. This is
  strictly better than doing nothing (the doubling half works), and the TODO for suppression
  is preserved. The two abilities are independent.

- **CardDefETB vs ObjectSpec triggered_ability**: Both paths now set `entering_object_id`.
  The check_triggers path (ObjectSpec) sets it at line 2430; the queue_carddef_etb_triggers
  path sets it via our fix. Both converge in `flush_pending_triggers` where
  `compute_trigger_doubling` is called -- no divergence.
