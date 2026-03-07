# Ability Plan: Alliance

**Generated**: 2026-03-07
**CR**: 207.2c (ability word -- no individual CR entry)
**Priority**: P4
**Batch**: 12.2
**Similar abilities studied**: Impact Tremors (`cards/defs/impact_tremors.rs`), Enrage (`WhenDealtDamage` trigger condition), Exploit/Soulbond (inline `check_triggers` handlers in `abilities.rs`)

## CR Rule Text

CR 207.2c: "An ability word appears in italics at the beginning of some abilities. Ability words
are similar to keywords in that they tie together cards that have similar functionality, but they
have no special rules meaning and no individual entries in the Comprehensive Rules."

Alliance is listed among the ability words in CR 207.2c. It has no dedicated CR section.

**Pattern**: "Alliance -- Whenever another creature you control enters, [effect]"

## Key Edge Cases

- **"Another" qualifier**: The card with Alliance does NOT trigger when it itself enters
  the battlefield. Only other creatures trigger it.
- **"You control"**: Only creatures entering under the Alliance card's controller trigger it.
  Opponents' creatures do not.
- **Must be a creature**: Non-creature permanents entering do not trigger Alliance.
- **Token creatures count**: Tokens are permanents; a creature token entering under your
  control triggers Alliance.
- **Simultaneous ETB**: If multiple creatures enter simultaneously (e.g., from a mass
  reanimate), each triggers Alliance separately. Per Gala Greeters ruling (2022-04-29),
  each instance goes on the stack as a separate triggered ability.
- **Panharmonicon interaction**: Panharmonicon doubles triggers caused by artifacts or
  creatures entering. Alliance triggers from creature ETBs, so Panharmonicon applies.
  The current `doubler_applies_to_trigger` checks `AnyPermanentEntersBattlefield` events
  for artifact/creature entering -- this should naturally work.
- **Multiplayer**: Only the Alliance card's controller matters. Alliance does not care
  about teammates or opponents' creatures.

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Architecture Analysis: Existing ETB Trigger Infrastructure

### What exists

- `TriggerCondition::WheneverCreatureEntersBattlefield { filter: Option<TargetFilter> }` --
  already defined in `card_definition.rs:1092`. The `TargetFilter` has `controller: TargetController`
  (for "you control") and `has_card_type: Option<CardType>` (though creature is implicit in the
  variant name).
- `TriggerEvent::AnyPermanentEntersBattlefield` -- runtime event in `game_object.rs:135`.
- `collect_triggers_for_event` at `abilities.rs:4602` -- generic dispatcher, matches `trigger_on`.
- `PermanentEnteredBattlefield` handler in `check_triggers` at `abilities.rs:2114` -- calls
  `collect_triggers_for_event` with `AnyPermanentEntersBattlefield` for all permanents.
- Impact Tremors card def uses `WheneverCreatureEntersBattlefield` with
  `filter: Some(TargetFilter { controller: TargetController::You, ..Default::default() })`.

### What is missing (GAP)

**`WheneverCreatureEntersBattlefield` is NOT wired in `enrich_spec_from_def`.** There is no
code that converts this `TriggerCondition` to a `TriggeredAbilityDef` on the `ObjectSpec`.
This means Impact Tremors' trigger currently does NOT fire at runtime.

The reason: `enrich_spec_from_def` has explicit match arms for `WhenDies`, `WhenAttacks`,
`WhenBlocks`, `WhenDealsCombatDamageToPlayer`, `WheneverOpponentCastsSpell`, etc. -- but
NOT for `WheneverCreatureEntersBattlefield` or `WheneverPermanentEntersBattlefield`.

### Solution approach

Add a wiring arm in `enrich_spec_from_def` that:
1. Maps `WheneverCreatureEntersBattlefield { filter }` to `TriggerEvent::AnyPermanentEntersBattlefield`
2. Carries the filter information so runtime can apply it
3. Applies "creature type check" + "controller check" + "another (exclude self)" at trigger collection time

**Two design options for filter application:**

**Option A (recommended): Add `etb_filter` to `TriggeredAbilityDef`**
- Add `pub etb_filter: Option<ETBTriggerFilter>` to `TriggeredAbilityDef`
- `ETBTriggerFilter { creature_only: bool, controller_you: bool, exclude_self: bool }`
- In `collect_triggers_for_event`, when `entering_object` is `Some`, check the filter:
  skip if `creature_only && !entering_is_creature`, skip if `controller_you && entering_controller != trigger_controller`,
  skip if `exclude_self && entering_object_id == obj_id`
- This is general-purpose and can be reused for `WheneverPermanentEntersBattlefield`,
  Landfall, Constellation, etc.

**Option B: Inline in `check_triggers`**
- Add a dedicated handler block in the `PermanentEnteredBattlefield` arm of `check_triggers`
  (like Exploit/Soulbond) that reads card definitions for the condition.
- Less general but avoids touching `TriggeredAbilityDef`.

**Recommendation: Option A** -- it aligns with the existing enrichment pattern and is reusable.
The filter struct is small and optional (None for existing triggers).

## Implementation Steps

### Step 1: No Enum Variant Needed

Alliance is an ability word (CR 207.2c). It has no special rules meaning and does NOT need a
`KeywordAbility` enum variant. Card definitions use `AbilityDefinition::Triggered` with
`TriggerCondition::WheneverCreatureEntersBattlefield` directly.

**Action**: No changes to `state/types.rs`.

### Step 2: Add ETB Trigger Filter to TriggeredAbilityDef

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add an `etb_filter` field to `TriggeredAbilityDef` (at line ~244):

```rust
/// Optional filter for ETB-based triggers. When present, the trigger only fires
/// if the entering permanent matches all specified criteria.
/// Used by Alliance ("another creature you control"), Constellation
/// ("enchantment you control"), Landfall ("land"), etc.
#[serde(default)]
pub etb_filter: Option<ETBTriggerFilter>,
```

Add the `ETBTriggerFilter` struct nearby:

```rust
/// Filter applied to ETB triggers to restrict which entering permanents cause
/// the trigger to fire. All `true` fields must be satisfied (AND logic).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ETBTriggerFilter {
    /// If true, the entering permanent must be a creature.
    pub creature_only: bool,
    /// If true, the entering permanent must be controlled by the trigger source's controller.
    pub controller_you: bool,
    /// If true, the entering permanent must NOT be the trigger source itself ("another").
    pub exclude_self: bool,
}
```

**Hash**: Add `ETBTriggerFilter` hashing to `state/hash.rs` in the `TriggeredAbilityDef` hasher.
The filter has 3 bools -- hash each.

**Defaults**: All existing `TriggeredAbilityDef` constructions use `etb_filter: None` (the
`#[serde(default)]` + `Option` handles existing data). Add `etb_filter: None` to every
existing `TriggeredAbilityDef` construction site in `enrich_spec_from_def` and `abilities.rs`.

### Step 3: Wire WheneverCreatureEntersBattlefield in enrich_spec_from_def

**File**: `crates/engine/src/testing/replay_harness.rs`
**Action**: Add a new arm in `enrich_spec_from_def` (after the existing trigger wiring blocks,
around line 2128) that handles `WheneverCreatureEntersBattlefield`:

```rust
// CR 207.2c / CR 603.2: Convert "Whenever [another] creature [you control] enters"
// card-definition triggers into runtime TriggeredAbilityDef entries.
// Used by Alliance ability word and similar patterns (Impact Tremors, etc.).
// The filter is applied at trigger-collection time in collect_triggers_for_event.
for ability in &def.abilities {
    if let AbilityDefinition::Triggered {
        trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield { filter },
        effect,
        intervening_if,
    } = ability
    {
        let etb_filter = ETBTriggerFilter {
            creature_only: true,
            controller_you: filter.as_ref().map_or(false, |f| {
                matches!(f.controller, TargetController::You)
            }),
            exclude_self: true, // Alliance always says "another"
        };
        spec = spec.with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
            intervening_if: intervening_if.clone().map(/* convert if needed */),
            description: "Whenever another creature you control enters (CR 207.2c)".to_string(),
            effect: Some(effect.clone()),
            etb_filter: Some(etb_filter),
        });
    }
}
```

**Note on `exclude_self`**: The `TriggerCondition::WheneverCreatureEntersBattlefield` is used by
both Alliance cards ("another creature") and Impact Tremors ("a creature you control enters" --
no "another"). Impact Tremors IS an enchantment, not a creature, so "exclude_self" wouldn't
matter for it anyway. But for correctness, the runner should check whether the card definition's
oracle text contains "another" or if the card is a non-creature. For Alliance cards, always
set `exclude_self: true`. For Impact Tremors (enchantment), `exclude_self` is irrelevant but
harmless if true (an enchantment can't be a creature entering).

**Decision**: Set `exclude_self: true` always for `WheneverCreatureEntersBattlefield`. The
only case where it would matter is if a creature has "whenever a creature you control enters"
(without "another") -- which is a different trigger that would need `exclude_self: false`.
For now, all known uses are either "another creature" (Alliance) or non-creature sources
(Impact Tremors), so `true` is safe.

### Step 4: Apply ETB Filter in collect_triggers_for_event

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `collect_triggers_for_event` (at line ~4628), after the `trigger_on` match,
add ETB filter checking:

```rust
// Apply ETB filter if present (Alliance, Constellation, Landfall, etc.)
if let Some(ref filter) = trigger_def.etb_filter {
    if let Some(entering_id) = entering_object {
        // exclude_self: "another" qualifier
        if filter.exclude_self && obj_id == entering_id {
            continue;
        }
        if let Some(entering_obj) = state.objects.get(&entering_id) {
            // creature_only: entering permanent must be a creature
            if filter.creature_only
                && !entering_obj.characteristics.card_types.contains(&CardType::Creature)
            {
                continue;
            }
            // controller_you: entering permanent must share controller
            if filter.controller_you && entering_obj.controller != obj.controller {
                continue;
            }
        } else {
            continue; // entering object not found, skip
        }
    }
}
```

**CR justification**: CR 603.2 -- triggered abilities trigger whenever the event occurs and
the trigger condition is met. The filter conditions (creature type, controller, "another") are
part of the trigger condition text.

### Step 5: Unit Tests

**File**: `crates/engine/tests/alliance.rs`
**Tests to write**:

1. `test_alliance_fires_when_another_creature_enters` -- CR 207.2c
   - Place a creature with Alliance trigger (via ObjectSpec with TriggeredAbilityDef) on P1's
     battlefield. Enter another creature under P1's control. Assert the Alliance trigger fires
     (appears in pending_triggers or on stack after flush).
   - Pattern: Follow `tests/enrage.rs` or `tests/abilities.rs` for trigger verification.

2. `test_alliance_does_not_fire_on_self_etb` -- "another" qualifier
   - Create a creature with Alliance. When that creature itself enters the battlefield,
     verify the Alliance trigger does NOT fire. The SelfEntersBattlefield event fires, but
     the AnyPermanentEntersBattlefield handler with `exclude_self: true` should skip it.

3. `test_alliance_does_not_fire_on_opponents_creature` -- "you control" qualifier
   - Place Alliance creature under P1. Enter a creature under P2's control. Assert no
     Alliance trigger fires on P1's permanent.

4. `test_alliance_does_not_fire_on_noncreature_permanent` -- creature filter
   - Place Alliance creature under P1. Enter an artifact (non-creature) under P1's control.
     Assert no Alliance trigger fires.

5. `test_alliance_fires_on_token_creature` -- tokens are creatures
   - Place Alliance creature under P1. Create a creature token under P1's control. Assert
     the Alliance trigger fires.

**Pattern**: Follow `crates/engine/tests/enrage.rs` for trigger-based test structure.
Use `GameStateBuilder::four_player()`, place permanents with `ObjectSpec`, advance game
state, check for triggers.

### Step 6: Card Definition

**Suggested card**: Prosperous Innkeeper
- Mana cost: {1}{G}
- Type: Creature -- Halfling Citizen 1/1
- Oracle: "When this creature enters, create a Treasure token. Whenever another creature you
  control enters, you gain 1 life."
- Two abilities: (1) self-ETB creating a Treasure token, (2) Alliance-style trigger gaining life
- The Treasure ETB is expressible with existing `Effect::CreateToken { spec: treasure_token_spec(1) }`
- The Alliance trigger is expressible with `TriggerCondition::WheneverCreatureEntersBattlefield { filter }`
  + `Effect::GainLife { player: EffectTarget::Controller, amount: 1 }`

**File**: `crates/engine/src/cards/defs/prosperous_innkeeper.rs`

**Alternative simpler card** (if Treasure creates complexity): Use a test-only ObjectSpec with
the Alliance trigger directly, without needing a full card definition. The unit tests in
Step 5 can use ObjectSpec with `with_triggered_ability(TriggeredAbilityDef { ... })`.

### Step 7: Game Script

**Suggested scenario**: Prosperous Innkeeper enters, creates Treasure (self-ETB). Then another
creature enters under the same player's control, triggering the Alliance lifegain.
**Subsystem directory**: `test-data/generated-scripts/stack/` (trigger-based)
**Sequence**: Cast Prosperous Innkeeper -> ETB Treasure -> Cast another creature -> Alliance
trigger fires -> gain 1 life.

### Step 8: Coverage Update

**File**: `docs/mtg-engine-ability-coverage.md`
**Action**: Update Alliance row from `none` to `validated` with file references.

## Interactions to Watch

- **Panharmonicon**: Doubles triggers from creature/artifact ETBs. Since Alliance uses
  `AnyPermanentEntersBattlefield`, the existing `doubler_applies_to_trigger` check should
  apply naturally. Verify in testing.
- **Yarok, the Desecrated**: Similar doubling for permanents entering. Same consideration.
- **Humility**: Removes all abilities. If Alliance creature loses its abilities via
  Layer 6, the triggered ability is removed from `characteristics.triggered_abilities` and
  `collect_triggers_for_event` won't find it. This is correct behavior.
- **Clone effects**: If a creature enters as a copy of the Alliance creature, the copy
  has the Alliance trigger. The "another" check ensures the copy entering doesn't trigger
  its own Alliance (the entering creature IS the copy, not "another").

## Impact on Existing Code

- **Impact Tremors**: Currently broken (trigger never fires). This plan fixes it by wiring
  `WheneverCreatureEntersBattlefield` in `enrich_spec_from_def`. After implementation,
  Impact Tremors will correctly trigger on creature ETBs under its controller.
- **`TriggeredAbilityDef` struct change**: Adding `etb_filter: Option<ETBTriggerFilter>`
  requires updating ALL existing construction sites. The `#[serde(default)]` annotation
  handles deserialization, but Rust struct literals need the field. Search for
  `TriggeredAbilityDef {` and add `etb_filter: None` to each.
- **Hash changes**: New field in `TriggeredAbilityDef` must be hashed in `hash.rs`.
- **Replay viewer**: No new `KeywordAbility` or `StackObjectKind` variant, so no
  `view_model.rs` changes needed. The trigger goes on the stack as a normal triggered
  ability.
