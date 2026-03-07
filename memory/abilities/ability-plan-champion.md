# Ability Plan: Champion

**Generated**: 2026-03-07
**CR**: 702.72
**Priority**: P4
**Similar abilities studied**: Evoke (ETB sacrifice trigger pattern in `abilities.rs:2026-2071`, resolution in `resolution.rs:1916-1914`), Hideaway (`exiled_by_hideaway` linked-ability tracking on `GameObject`, `game_object.rs:431-442`)

## CR Rule Text

702.72. Champion

702.72a Champion represents two triggered abilities. "Champion an [object]" means "When this permanent enters, sacrifice it unless you exile another [object] you control" and "When this permanent leaves the battlefield, return the exiled card to the battlefield under its owner's control."

702.72b The two abilities represented by champion are linked. See rule 607, "Linked Abilities."

702.72c A permanent is "championed" by another permanent if the latter exiles the former as the direct result of a champion ability.

607.2k The two abilities represented by the champion keyword are linked abilities. See rule 702.72, "Champion."

607.2a If an object has an activated or triggered ability printed on it that instructs a player to exile one or more cards and an ability printed on it that refers either to "the exiled cards" or to cards "exiled with [this object]," these abilities are linked. The second ability refers only to cards in the exile zone that were put there as a result of an instruction to exile them in the first ability.

## Key Edge Cases

- **Linked abilities (CR 607.2a/607.2k)**: The LTB trigger returns ONLY the card exiled by THIS permanent's champion ETB trigger, not any other exiled card. Tracked via an ObjectId link on the champion permanent.
- **"Sacrifice unless you exile"**: This is a single trigger with two possible outcomes at resolution. The controller chooses: (a) exile another qualifying permanent they control, or (b) the champion is sacrificed. If no valid target exists, they MUST sacrifice.
- **LTB fires on ANY zone departure**: Dies (graveyard), exiled, bounced to hand, shuffled into library -- ALL trigger the return. This is NOT just `SelfDies`; it requires a broader `SelfLeavesTheBattlefield` trigger event.
- **CR 400.7 -- new identity on zone change**: The exiled card gets a new ObjectId in exile. The champion permanent must track the exile ObjectId (not the battlefield ObjectId). When the champion leaves, the exiled card's CURRENT ObjectId in exile must be looked up.
- **If exiled card is no longer in exile**: When the LTB trigger resolves, if the exiled card has left exile (e.g., processed by Riftsweeper or Rest in Peace interaction), nothing happens. The return does nothing.
- **Return under OWNER's control**: Not controller's. The exiled card returns to the battlefield under its owner's control (CR 702.72a).
- **Blinking the champion**: If the champion is blinked (exile + return), its LTB trigger fires, returning the championed card. The champion re-enters as a new object (CR 400.7), triggering its ETB champion again.
- **Two Champions championing each other**: Changeling Hero + Changeling Berserker can loop. Hero enters, exiles Berserker. Later Hero dies, Berserker returns, exiles Hero. Then Berserker dies, Hero returns, exiles Berserker. This is legal and not an infinite loop (each step requires a zone-change event to start).
- **Multiplayer -- ownership matters**: The exiled card returns under its OWNER's control, not the champion's controller's. In theft scenarios, this matters.
- **Champion filter**: "Champion a creature" = any creature you control (other than the champion itself, per "another"). "Champion a Faerie" = any Faerie you control (other than itself). Must check the permanent's subtypes/types against the filter.
- **Changeling interaction**: A creature with Changeling is every creature type. "Champion a Faerie" on Changeling Hero can exile any creature (since Changeling Hero is every type, and "another" just means another permanent, and the target must be a Faerie, which Changelings are).

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

**File**: `crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Champion` variant after `Backup(u32)` (line ~1152).
**Discriminant**: 126
**Variant shape**: `Champion` (no parameter -- the filter is in the card definition / PendingTrigger, not the keyword itself, following the Evoke/Exploit pattern where the keyword is just a marker).

```
/// CR 702.72: Champion an [object] -- "When this permanent enters, sacrifice
/// it unless you exile another [object] you control. When this permanent
/// leaves the battlefield, return the exiled card to the battlefield under
/// its owner's control."
///
/// Two linked triggered abilities (CR 607.2k). The champion filter
/// (creature, Faerie, etc.) is carried by `ChampionFilter` in the
/// `PendingTriggerKind::ChampionETB` trigger and looked up from the
/// card registry at trigger time.
///
/// Discriminant 126.
Champion,
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `KeywordAbility::Champion => 126u8.hash_into(hasher)` to the `KeywordAbility` `HashInto` impl.

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add `KeywordAbility::Champion => "Champion"` to the keyword display match arm.

### Step 2: Champion Filter Type

**File**: `crates/engine/src/state/types.rs`
**Action**: Add a `ChampionFilter` enum near the `KeywordAbility` enum.

```
/// CR 702.72a: The filter for what can be championed.
/// "Champion a creature" = `AnyCreature`
/// "Champion a [subtype]" = `Subtype(SubType)` (e.g., Faerie, Goblin)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChampionFilter {
    /// "Champion a creature" -- any creature you control (other than self).
    AnyCreature,
    /// "Champion a [subtype]" -- any permanent with this subtype you control (other than self).
    Subtype(SubType),
}
```

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `HashInto` impl for `ChampionFilter`.

**File**: `crates/engine/src/cards/helpers.rs`
**Action**: Add `ChampionFilter` to the re-exports so card definitions can use it.

### Step 3: GameObject Field -- Exile Tracking

**File**: `crates/engine/src/state/game_object.rs`
**Action**: Add `champion_exiled_card: Option<ObjectId>` field on `GameObject` (near `exiled_by_hideaway`, line ~442). This field is set on the champion permanent when it exiles a card via its ETB trigger. The LTB trigger reads this to know which card to return.

```
/// CR 702.72a / CR 607.2a: The ObjectId of the card exiled by this
/// permanent's champion ETB trigger. Used by the linked LTB trigger
/// to return the correct card to the battlefield.
///
/// Set when the ChampionETBTrigger resolves and a card is exiled.
/// Cleared on zone changes (CR 400.7 -- new object identity).
/// If `None`, no card was championed (champion was sacrificed instead
/// or the ETB trigger hasn't resolved yet).
#[serde(default)]
pub champion_exiled_card: Option<ObjectId>,
```

**Init sites** (set to `None`):
- `crates/engine/src/state/builder.rs` -- `ObjectSpec` builder (line ~951)
- `crates/engine/src/state/mod.rs` -- `move_object_to_zone` new object creation (line ~344, ~482)
- `crates/engine/src/rules/resolution.rs` -- token creation sites (search for `exiled_by_hideaway: None`)
- `crates/engine/src/effects/mod.rs` -- token creation site (search for `exiled_by_hideaway: None`)

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add `self.champion_exiled_card.hash_into(hasher)` to the `GameObject` `HashInto` impl.

### Step 4: PendingTriggerKind -- ChampionETB and ChampionLTB

**File**: `crates/engine/src/state/stubs.rs`
**Action**: Add two new `PendingTriggerKind` variants after `Backup`:

```
/// CR 702.72a: Champion ETB trigger -- "sacrifice it unless you exile
/// another [object] you control."
ChampionETB,
/// CR 702.72a: Champion LTB trigger -- "return the exiled card to the
/// battlefield under its owner's control."
ChampionLTB,
```

**Action**: Add new fields to `PendingTrigger`:

```
/// CR 702.72a: The champion filter (creature, Faerie, etc.) for the ETB trigger.
///
/// Only meaningful when `kind == PendingTriggerKind::ChampionETB`.
/// Looked up from the card registry at trigger-collection time.
#[serde(default)]
pub champion_filter: Option<ChampionFilter>,
/// CR 702.72a: The ObjectId of the card exiled by the champion ETB trigger.
///
/// Only meaningful when `kind == PendingTriggerKind::ChampionLTB`.
/// Captured from `champion_exiled_card` on the champion permanent at the
/// moment it leaves the battlefield (last-known information).
#[serde(default)]
pub champion_exiled_card: Option<ObjectId>,
```

### Step 5: StackObjectKind -- ChampionETBTrigger and ChampionLTBTrigger

**File**: `crates/engine/src/state/stack.rs`
**Action**: Add two new `StackObjectKind` variants after `BackupTrigger`:

```
/// CR 702.72a: Champion ETB trigger on the stack.
///
/// "When this permanent enters, sacrifice it unless you exile another
/// [object] you control." When this resolves, the engine auto-selects
/// the first qualifying permanent to exile (simplified -- no player choice
/// for now). If none exists, the champion is sacrificed.
///
/// Discriminant 47.
ChampionETBTrigger {
    source_object: ObjectId,
    champion_filter: ChampionFilter,
},
/// CR 702.72a: Champion LTB trigger on the stack.
///
/// "When this permanent leaves the battlefield, return the exiled card
/// to the battlefield under its owner's control." When this resolves,
/// the engine checks if the exiled card is still in exile; if so, it
/// moves it to the battlefield under its owner's control.
///
/// Discriminant 48.
ChampionLTBTrigger {
    source_object: ObjectId,
    exiled_card: ObjectId,
},
```

**File**: `tools/replay-viewer/src/view_model.rs`
**Action**: Add match arms for both new `StackObjectKind` variants in `stack_kind_info()`.

**File**: `tools/tui/src/play/panels/stack_view.rs`
**Action**: Add match arms for both new `StackObjectKind` variants.

**File**: `crates/engine/src/state/hash.rs`
**Action**: Add hash arms for both new `StackObjectKind` variants.

### Step 6: ETB Trigger Wiring

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `check_triggers()`, inside the `GameEvent::PermanentEnteredBattlefield` arm (after the Exploit block at line ~2146), add Champion ETB trigger generation.

Pattern: follows Exploit (line ~2074-2145). Check if the entering permanent has `KeywordAbility::Champion` in its keywords. Look up the `ChampionFilter` from the card definition (via `card_registry.get(card_id)` -- scan for `AbilityDefinition::Keyword(KeywordAbility::Champion)` and extract the associated `ChampionFilter` from the card definition). However, since `KeywordAbility::Champion` has no parameter, the filter must come from the card definition's abilities list.

**Design decision**: Use a new `AbilityDefinition::Champion { filter: ChampionFilter }` variant (discriminant 49) in `card_definition.rs` to carry the filter. The `enrich_spec_from_def` function in `builder.rs` will add `KeywordAbility::Champion` when it encounters this variant. At trigger time in `check_triggers`, scan the card def for the `AbilityDefinition::Champion` variant to extract the filter.

**File**: `crates/engine/src/cards/card_definition.rs`
**Action**: Add `AbilityDefinition::Champion { filter: ChampionFilter }` variant. Discriminant 49.

```
/// CR 702.72: Champion an [object]. Two linked triggered abilities (CR 607.2k):
/// 1. ETB: "sacrifice it unless you exile another [object] you control"
/// 2. LTB: "return the exiled card to the battlefield under its owner's control"
///
/// The filter specifies what can be championed (creature, Faerie, etc.).
/// `enrich_spec_from_def` adds `KeywordAbility::Champion` to the keywords.
///
/// Discriminant 49.
Champion { filter: ChampionFilter },
```

**File**: `crates/engine/src/state/builder.rs`
**Action**: In `enrich_spec_from_def()`, add a match arm for `AbilityDefinition::Champion { .. }` that adds `KeywordAbility::Champion` to the object's keywords.

**Back to**: `crates/engine/src/rules/abilities.rs`
**Trigger generation code** (inside `PermanentEnteredBattlefield` arm):

```rust
// CR 702.72a: Champion ETB trigger. "When this permanent enters,
// sacrifice it unless you exile another [object] you control."
if let Some(obj) = state.objects.get(object_id) {
    if obj.characteristics.keywords.contains(&KeywordAbility::Champion) {
        // Look up the champion filter from the card definition.
        let filter = obj.card_id.as_ref()
            .and_then(|cid| state.card_registry.get(cid.clone()))
            .and_then(|def| {
                def.abilities.iter().find_map(|a| {
                    if let AbilityDefinition::Champion { filter } = a {
                        Some(filter.clone())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(ChampionFilter::AnyCreature);

        triggers.push(PendingTrigger {
            source: *object_id,
            ability_index: 0, // unused
            controller: obj.controller,
            kind: PendingTriggerKind::ChampionETB,
            triggering_event: Some(TriggerEvent::SelfEntersBattlefield),
            entering_object_id: Some(*object_id),
            champion_filter: Some(filter),
            // ... all other fields None/default
        });
    }
}
```

### Step 7: LTB Trigger Wiring -- New Infrastructure

**Critical**: The engine currently has NO `SelfLeavesTheBattlefield` trigger event. Champion's LTB trigger fires when the champion leaves the battlefield for ANY reason (dies, exiled, bounced, etc.). This requires new infrastructure.

**Approach**: Do NOT add a generic `TriggerEvent::SelfLeavesTheBattlefield` and wire it to all zone-departure events (that would be a massive change). Instead, handle Champion LTB as a hardcoded check in `check_triggers` on ALL zone-departure events, similar to how Recover piggybacks on `CreatureDied`.

The events that represent a permanent leaving the battlefield:
1. `GameEvent::CreatureDied` -- creature goes to graveyard
2. `GameEvent::ObjectExiled` -- object is exiled
3. `GameEvent::ObjectReturnedToHand` -- object bounced to hand
4. `GameEvent::PermanentDestroyed` -- non-creature permanent goes to graveyard

For each of these events, add a Champion LTB check:

```rust
// CR 702.72a: Champion LTB trigger. When the champion permanent leaves
// the battlefield, check if it had a champion_exiled_card and fire the
// LTB trigger to return that card.
//
// CR 603.10a: LTB triggers "look back in time" -- the trigger checks
// the object's last state on the battlefield. Since move_object_to_zone
// preserves characteristics (including champion_exiled_card), we can
// read the field from the new zone object.
```

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: Add a helper function `check_champion_ltb_trigger()` that:
1. Takes the new ObjectId (in graveyard/exile/hand) after the zone move
2. Reads `champion_exiled_card` from the object (preserved by `move_object_to_zone`)
3. If `Some(exiled_id)`, creates a `PendingTrigger` with `kind: ChampionLTB` and `champion_exiled_card: Some(exiled_id)`
4. Uses the object's `controller` at death time (for `CreatureDied`, use the `controller` field from the event; for others, use the object's owner/controller)

Call this helper inside the `CreatureDied`, `ObjectExiled`, `ObjectReturnedToHand`, and `PermanentDestroyed` arms of `check_triggers`.

**IMPORTANT**: `champion_exiled_card` must be read from the object BEFORE `move_object_to_zone` resets it. However, `move_object_to_zone` creates a new object that copies characteristics. Check if `champion_exiled_card` is preserved or reset. Since it is a custom field (not in `characteristics`), it will be reset to `None` by `move_object_to_zone` (which creates a fresh `GameObject`).

**Fix**: Capture `champion_exiled_card` BEFORE the zone move at each emission site, and carry it through the event. This is the same pattern as `pre_death_counters` in `CreatureDied`.

**Alternative (simpler)**: Add `champion_exiled_card: Option<ObjectId>` to the `CreatureDied` event. For the other events (`ObjectExiled`, `ObjectReturnedToHand`, `PermanentDestroyed`), either:
(a) Add the field to each event (invasive), or
(b) Check the field on the old object ID before the move (but the old ID is dead by the time `check_triggers` runs).

**Best approach**: Add `champion_exiled_card` as a field preserved by `move_object_to_zone`. In `state/mod.rs`, the `move_object_to_zone` function creates a new `GameObject` -- add `champion_exiled_card: old_obj.champion_exiled_card` to preserve it across zone changes for LTB look-back purposes. This is the simplest approach and mirrors how `characteristics` (including `triggered_abilities`) are preserved.

Wait -- checking `move_object_to_zone`: it creates a new `GameObject` with many fields reset. Let me verify which fields survive.

Actually, looking at the code pattern for `CreatureDied` and `pre_death_counters`: the counters are captured BEFORE `move_object_to_zone` because `move_object_to_zone` resets them. The same issue applies to `champion_exiled_card`.

**Final approach**: At each site where a permanent leaves the battlefield and emits an event, capture `champion_exiled_card` from the object BEFORE the zone move. Then in `check_triggers`, when processing those events, if the captured value is `Some(exiled_id)`, emit a ChampionLTB trigger.

This requires adding `champion_exiled_card: Option<ObjectId>` to:
- `GameEvent::CreatureDied` (already has `pre_death_counters` as precedent)
- `GameEvent::PermanentDestroyed`
- `GameEvent::ObjectExiled`
- `GameEvent::ObjectReturnedToHand`

And capturing it at each emission site (SBA, effects, abilities).

**Emission sites to update** (grep for each event and add capture):

For `CreatureDied`: add `champion_exiled_card` field. Capture sites:
- `crates/engine/src/rules/sba.rs` (SBA creature death)
- `crates/engine/src/rules/abilities.rs` (sacrifice as cost)
- `crates/engine/src/rules/replacement.rs` (sacrifice effects)
- `crates/engine/src/effects/mod.rs` (DestroyTarget, etc.)
- `crates/engine/src/rules/resolution.rs` (Evoke sacrifice, etc.)

This is invasive. **Simpler alternative**: Instead of threading through events, preserve `champion_exiled_card` in `move_object_to_zone` (like characteristics are preserved), and read it from the post-move object in `check_triggers`. Since the new object is in graveyard/exile/hand, it can still be read.

Let me check if `move_object_to_zone` truly resets custom fields or preserves them.

**After code review**: `move_object_to_zone` at `state/mod.rs:344` and `state/mod.rs:482` creates a new `GameObject` with `champion_exiled_card: None` (since we're adding it with `#[serde(default)]`). BUT: if we explicitly copy it in the `move_object_to_zone` implementation, it will be preserved.

**Decision**: Preserve `champion_exiled_card` in `move_object_to_zone`. Then in `check_triggers`, for `CreatureDied { new_grave_id, .. }`, read `state.objects.get(new_grave_id).champion_exiled_card`. Same for `PermanentDestroyed`, `ObjectExiled`, and `ObjectReturnedToHand`. This is the cleanest approach and avoids modifying event structs.

### Step 8: flush_pending_triggers -- ChampionETB and ChampionLTB

**File**: `crates/engine/src/rules/abilities.rs`
**Action**: In `flush_pending_triggers()`, add match arms for `PendingTriggerKind::ChampionETB` and `PendingTriggerKind::ChampionLTB` to create the corresponding `StackObjectKind` variants.

Pattern: follows Evoke (line ~4110-4117).

```rust
PendingTriggerKind::ChampionETB => StackObjectKind::ChampionETBTrigger {
    source_object: trigger.source,
    champion_filter: trigger.champion_filter.clone().unwrap_or(ChampionFilter::AnyCreature),
},
PendingTriggerKind::ChampionLTB => StackObjectKind::ChampionLTBTrigger {
    source_object: trigger.source,
    exiled_card: trigger.champion_exiled_card.unwrap_or(trigger.source),
},
```

### Step 9: Resolution -- ChampionETBTrigger and ChampionLTBTrigger

**File**: `crates/engine/src/rules/resolution.rs`
**Action**: Add resolution handlers for both new `StackObjectKind` variants.

**ChampionETBTrigger resolution**:
1. Check if `source_object` is still on the battlefield (CR 400.7). If not, do nothing.
2. Find all qualifying permanents the controller controls (other than the champion itself) that match `champion_filter`:
   - `ChampionFilter::AnyCreature`: any creature the controller controls, other than `source_object`
   - `ChampionFilter::Subtype(st)`: any permanent the controller controls with that subtype, other than `source_object`
   - Use layer-resolved characteristics for type/subtype checking (Changeling, Humility).
3. If qualifying permanents exist: auto-select the first one (simplified -- no player choice for now). Exile it. Set `champion_exiled_card` on the champion permanent to the new exile ObjectId.
4. If no qualifying permanents exist: sacrifice the champion (same pattern as Evoke sacrifice -- `check_zone_change_replacement` then `move_object_to_zone` to graveyard).
5. Emit appropriate events (`ObjectExiled` for the exiled card, or `CreatureDied`/`PermanentDestroyed` for the sacrificed champion).

**ChampionLTBTrigger resolution**:
1. Check if `exiled_card` is still in exile (`state.objects.get(exiled_card)` exists and `zone == ZoneId::Exile`). If not, do nothing.
2. Move the exiled card to the battlefield under its OWNER's control (`move_object_to_zone(exiled_card, ZoneId::Battlefield)`).
3. Set controller to owner (should already be correct from `move_object_to_zone`).
4. Emit `PermanentEnteredBattlefield` event for the returned card (this will fire its own ETB triggers).
5. Run ETB hooks: `apply_self_etb_from_definition`, `apply_global_etb_replacements`, `register_static_continuous_effects`, `fire_when_enters_triggered_effects` (same pattern as resolution.rs permanent ETB sequence).

**Note on the counterspell arm**: Add both new SOK variants to the "abilities can't be countered" match arm at the bottom of `resolve_stack_object` (near line ~4687).

### Step 10: Unit Tests

**File**: `crates/engine/tests/champion.rs`
**Tests to write**:

- `test_champion_basic_etb_exiles_creature` -- CR 702.72a: Champion creature enters, exiles another creature you control. Verify the target creature is in exile and the champion is on the battlefield.
- `test_champion_no_target_sacrifices_self` -- CR 702.72a: Champion creature enters with no other qualifying permanent. Champion is sacrificed (goes to graveyard).
- `test_champion_ltb_returns_exiled_card` -- CR 702.72a: Champion creature dies. The exiled card returns to the battlefield under its owner's control.
- `test_champion_ltb_exiled_card_gone` -- CR 702.72a/607.2a: Champion leaves, but the exiled card has already left exile (e.g., exiled by another effect). Nothing happens.
- `test_champion_returns_under_owners_control` -- CR 702.72a: In a theft scenario, the exiled card returns under its OWNER's control, not the champion's controller.
- `test_champion_subtype_filter_faerie` -- CR 702.72a: "Champion a Faerie" only allows exiling Faeries. Non-Faerie creatures cannot be championed.
- `test_champion_changeling_matches_any_subtype` -- Changeling creature is every creature type, so it matches "Champion a Faerie".
- `test_champion_blink_returns_and_re_champions` -- Blink the champion: LTB fires (returns exiled card), then ETB fires again (must exile another or sacrifice).

**Pattern**: Follow Exploit/Evoke tests. Use `GameStateBuilder::four_player()`, `ObjectSpec::creature()` for test permanents.

### Step 11: Card Definition (later phase)

**Suggested card**: Changeling Hero (Champion a creature + Changeling + Lifelink -- good test since Changeling is already implemented)
**Card lookup**: use `card-definition-author` agent

### Step 12: Game Script (later phase)

**Suggested scenario**: Cast Changeling Hero with another creature on the battlefield. Verify the creature is exiled. Then destroy Changeling Hero and verify the exiled creature returns.
**Subsystem directory**: `test-data/generated-scripts/abilities/`

## Interactions to Watch

- **Evoke + Champion**: If a creature has both Evoke and Champion (unlikely but possible via effects), both ETB triggers go on the stack. The controller chooses the order. If Evoke sacrifice resolves first, Champion ETB trigger still resolves but the champion is already gone -- no exile happens, no sacrifice (already dead).
- **Panharmonicon + Champion**: Champion's ETB is a `SelfEntersBattlefield` trigger. `doubler_applies_to_trigger` only doubles `AnyPermanentEntersBattlefield` triggers, NOT self-ETB triggers. So Panharmonicon does NOT double Champion ETB. (Consistent with the `SelfEntersBattlefield` not being doubled -- same as Evoke, Exploit, Hideaway.)
- **Humility + Champion**: If Humility removes all abilities, the Champion keyword is removed. BUT: if the Champion ETB trigger is already on the stack, it still resolves (CR 603.3 -- triggered abilities exist independently of their source once triggered). The LTB trigger won't fire if Champion ability is removed before the champion leaves.
- **Layers and champion_exiled_card**: `champion_exiled_card` is a tracking field, not an ability. It persists even if abilities are removed (like `is_renowned`, `was_unearthed`). This is correct per CR 702.72 -- the linked ability refers to the action taken, not the current ability state.

## Discriminant Summary

| Type | Variant | Discriminant |
|------|---------|-------------|
| KeywordAbility | Champion | 126 |
| StackObjectKind | ChampionETBTrigger | 47 |
| StackObjectKind | ChampionLTBTrigger | 48 |
| AbilityDefinition | Champion { filter } | 49 |
| PendingTriggerKind | ChampionETB | (no disc -- serde skip) |
| PendingTriggerKind | ChampionLTB | (no disc -- serde skip) |

## Files to Modify (Complete List)

1. `crates/engine/src/state/types.rs` -- KeywordAbility::Champion, ChampionFilter enum
2. `crates/engine/src/state/game_object.rs` -- champion_exiled_card field
3. `crates/engine/src/state/stubs.rs` -- PendingTriggerKind::ChampionETB/ChampionLTB, PendingTrigger fields
4. `crates/engine/src/state/stack.rs` -- StackObjectKind::ChampionETBTrigger/ChampionLTBTrigger
5. `crates/engine/src/state/hash.rs` -- hash impls for all new types/fields
6. `crates/engine/src/state/mod.rs` -- preserve champion_exiled_card in move_object_to_zone; init to None in new object creation
7. `crates/engine/src/state/builder.rs` -- init champion_exiled_card to None in ObjectSpec
8. `crates/engine/src/cards/card_definition.rs` -- AbilityDefinition::Champion { filter }
9. `crates/engine/src/cards/helpers.rs` -- export ChampionFilter
10. `crates/engine/src/rules/abilities.rs` -- ETB trigger wiring, LTB trigger wiring (4 event arms), flush_pending_triggers arms
11. `crates/engine/src/rules/resolution.rs` -- ChampionETBTrigger + ChampionLTBTrigger resolution; init champion_exiled_card to None in all token creation sites
12. `crates/engine/src/effects/mod.rs` -- init champion_exiled_card to None in token creation
13. `tools/replay-viewer/src/view_model.rs` -- StackObjectKind match + KeywordAbility match
14. `tools/tui/src/play/panels/stack_view.rs` -- StackObjectKind match
15. `crates/engine/tests/champion.rs` -- new test file
