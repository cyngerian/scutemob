# Ability Plan: Persist

**Generated**: 2026-02-26
**CR**: 702.79
**Priority**: P2
**Similar abilities studied**: Ward (keyword -> triggered ability translation in `builder.rs`), Exalted/Annihilator (SelfDies/SelfAttacks trigger dispatch in `abilities.rs`), existing SelfDies trigger pipeline (abilities.rs:755-795, resolution.rs:351-405)

## CR Rule Text

```
702.79. Persist

702.79a Persist is a triggered ability. "Persist" means "When this permanent is put
into a graveyard from the battlefield, if it had no -1/-1 counters on it, return it
to the battlefield under its owner's control with a -1/-1 counter on it."
```

## Key Edge Cases

From Kitchen Finks / Murderous Redcap rulings (Scryfall 2013-06-07):

1. **Last known information**: The persist trigger checks whether the creature had -1/-1 counters just before it left the battlefield (its last known information), NOT its current state in the graveyard. Since `move_object_to_zone` resets counters to `OrdMap::new()`, the pre-death counter state must be captured BEFORE the zone move and carried through the `CreatureDied` event.

2. **Multiple persist instances**: If a permanent has multiple instances of persist, each triggers separately, but redundant instances have no effect. If one returns the card, subsequent ones do nothing (the card is no longer in the graveyard).

3. **+1/+1 and -1/-1 interaction (CR 704.5q + SBA)**: If a creature with persist has +1/+1 counters on it and receives enough -1/-1 counters to die, persist does NOT trigger. The SBA for counter annihilation (CR 704.5q) happens simultaneously with lethal-damage/zero-toughness SBAs, but the creature's last-known state still has the -1/-1 counters on it before they were annihilated. Persist checks the state "just before" it left.

4. **Token with persist**: A token creature with no -1/-1 counters and persist triggers the ability when it dies. However, the token ceases to exist in the graveyard (CR 704.5d SBA) before the triggered ability resolves. At resolution, the source object is gone, so the MoveZone effect finds nothing to move. The trigger fires but has no effect.

5. **Creature that stops being a creature**: "If a creature with persist stops being a creature, persist will still work." Persist triggers on "permanent put into graveyard from the battlefield," not just creature death.

6. **Multiplayer APNAP**: When multiple creatures with persist die simultaneously (e.g., board wipe), the active player puts all their persist triggers on the stack first, then each other player in turn order. Last triggers on stack resolve first, so non-active player's creatures return first.

7. **New object**: When a persist creature returns to the battlefield, it is a new object (CR 400.7) with no memory of its previous existence. Its only counter is the -1/-1 counter persist adds. It gets new summoning sickness.

8. **Persist + counter removal combo**: If a creature returns via persist with a -1/-1 counter, then something removes the -1/-1 counter (e.g., +1/+1 counter from another source causing annihilation via CR 704.5q), it can die and trigger persist again.

9. **Intervening-if at resolution (CR 603.4)**: Persist's "if it had no -1/-1 counters" is an intervening-if checked at trigger time AND resolution time. At resolution, the card is in the graveyard with no counters (counters reset by zone change). The resolution check examines whether the source is still in the graveyard (if the card was exiled or returned to hand before the trigger resolves, the trigger finds nothing to move).

## Current State (from ability-wip.md)

- [ ] 1. Enum variant
- [ ] 2. Rule enforcement
- [ ] 3. Trigger wiring
- [ ] 4. Unit tests
- [ ] 5. Card definition
- [ ] 6. Game script
- [ ] 7. Coverage doc update

## Implementation Steps

### Step 1: Enum Variant

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/types.rs`
**Action**: Add `KeywordAbility::Persist` variant after `Annihilator(u32)` (line ~185)
**Pattern**: Follow `KeywordAbility::Exalted` at line 350 in hash.rs (simple unit variant)
**CR**: 702.79

```rust
/// CR 702.79: Persist -- "When this permanent is put into a graveyard from
/// the battlefield, if it had no -1/-1 counters on it, return it to the
/// battlefield under its owner's control with a -1/-1 counter on it."
///
/// Translated to a TriggeredAbilityDef at object-construction time in
/// `state/builder.rs`. The trigger fires on SelfDies events; the
/// intervening-if checks pre-death counters via the CreatureDied event.
Persist,
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs`
**Action**: Add discriminant 36 for `KeywordAbility::Persist`

```rust
// Persist (discriminant 36) -- CR 702.79
KeywordAbility::Persist => 36u8.hash_into(hasher),
```

**Match arms to update**:

1. `/home/airbaggie/scutemob/tools/replay-viewer/src/view_model.rs` line ~599 in `format_keyword`:
   ```rust
   KeywordAbility::Persist => "Persist".to_string(),
   ```

### Step 2: Extend `InterveningIf` for Counter Checks

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/game_object.rs`
**Action**: Add a new `InterveningIf` variant for "had no counters of type X"

```rust
/// "if it had no [counter type] counters on it" -- checked against pre-death
/// counter state for persist/undying (CR 702.79a, CR 702.93a).
/// At trigger time: checks `CreatureDied.pre_death_counters`.
/// At resolution time: checks whether the source card is still in the graveyard
/// (if it moved, the trigger has no effect -- functionally equivalent to false).
SourceHadNoCounterOfType(CounterType),
```

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` -- add to `InterveningIf` hash impl:

```rust
InterveningIf::SourceHadNoCounterOfType(ct) => {
    1u8.hash_into(hasher);
    ct.hash_into(hasher);
}
```

### Step 3: Extend `CreatureDied` Event with Pre-Death Counters

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/events.rs`
**Action**: Add `pre_death_counters: im::OrdMap<CounterType, u32>` field to the `CreatureDied` variant

```rust
CreatureDied {
    object_id: ObjectId,
    new_grave_id: ObjectId,
    controller: PlayerId,
    /// CR 702.79a / CR 702.93a: counters on the creature just before it left
    /// the battlefield. Used by persist (checks -1/-1) and undying (checks +1/+1)
    /// to evaluate the intervening-if condition at trigger time.
    pre_death_counters: im::OrdMap<CounterType, u32>,
},
```

**All emission sites must be updated to capture `obj.counters.clone()` before `move_object_to_zone`:**

1. `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs` line 246 (sacrifice-as-cost):
   Capture `obj.counters.clone()` alongside `is_creature`, `owner`, `pre_death_controller`.

2. `/home/airbaggie/scutemob/crates/engine/src/rules/sba.rs` line 320 (SBA Proceed path):
   Capture `obj.counters.clone()` alongside `owner`, `pre_death_controller` at line 301-303.

3. `/home/airbaggie/scutemob/crates/engine/src/rules/sba.rs` line 348 (SBA Redirect path):
   Same capture; use the same pre-death counters variable.

4. `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` line 369 (DestroyPermanent Redirect):
   Capture counters alongside `card_types`, `owner`, `pre_death_controller` at line 325-336.

5. `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` line 410 (DestroyPermanent Proceed):
   Same variable from step 4.

6. `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` line 1093 (SacrificePermanents Redirect):
   Capture counters in the sacrifice loop alongside `card_types`, `owner`, `pre_sacrifice_controller`.

7. `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` line 1133 (SacrificePermanents Proceed):
   Same variable from step 6.

8. `/home/airbaggie/scutemob/crates/engine/src/rules/replacement.rs` line 1027 (`zone_change_events` helper):
   Add `pre_death_counters: im::OrdMap<CounterType, u32>` parameter to `zone_change_events`. All callers must pass it.

**Hash**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs` line 1300:
Update the `CreatureDied` match arm to hash `pre_death_counters`.

```rust
GameEvent::CreatureDied {
    object_id,
    new_grave_id,
    controller,
    pre_death_counters,
} => {
    "CreatureDied".hash_into(hasher);
    object_id.hash_into(hasher);
    new_grave_id.hash_into(hasher);
    controller.hash_into(hasher);
    // Hash counter map for determinism
    for (ct, count) in pre_death_counters.iter() {
        ct.hash_into(hasher);
        count.hash_into(hasher);
    }
}
```

### Step 4: Wire Persist Keyword to Triggered Ability (Builder)

**File**: `/home/airbaggie/scutemob/crates/engine/src/state/builder.rs`
**Action**: In the keyword-to-trigger translation loop (line ~350-436), add a persist block:

```rust
// CR 702.79a: Persist -- "When this permanent is put into a graveyard
// from the battlefield, if it had no -1/-1 counters on it, return it
// to the battlefield under its owner's control with a -1/-1 counter on it."
if matches!(kw, KeywordAbility::Persist) {
    triggered_abilities.push(TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDies,
        intervening_if: Some(InterveningIf::SourceHadNoCounterOfType(
            CounterType::MinusOneMinusOne,
        )),
        description: "Persist (CR 702.79a): When this permanent dies, \
                      if it had no -1/-1 counters on it, return it to the \
                      battlefield under its owner's control with a -1/-1 \
                      counter on it.".to_string(),
        effect: Some(Effect::Sequence(vec![
            Effect::MoveZone {
                target: EffectTarget::Source,
                to: ZoneTarget::Battlefield { tapped: false },
            },
            Effect::AddCounter {
                target: EffectTarget::Source,
                counter: CounterType::MinusOneMinusOne,
                count: 1,
            },
        ])),
    });
}
```

**Important**: The `EffectTarget::Source` in `MoveZone` will resolve to the graveyard object (which is `ctx.source` in the triggered ability's resolution). After MoveZone succeeds, `ctx.target_remaps` will NOT automatically update `ctx.source`. The `AddCounter` effect also uses `EffectTarget::Source`, which will still point at the OLD graveyard ObjectId (now retired). We need to handle the source tracking after MoveZone.

**CRITICAL ISSUE**: After `MoveZone` moves the source from graveyard to battlefield, `ctx.source` still references the old graveyard ObjectId (now removed from `state.objects`). The subsequent `AddCounter` with `EffectTarget::Source` will find nothing and silently no-op. Two solutions:

**Option A (preferred)**: After `MoveZone` processes `EffectTarget::Source`, update `ctx.source` to the new ObjectId. This requires a small addition to the MoveZone handler in `effects/mod.rs`:

```rust
// In the MoveZone handler, after move_object_to_zone:
if matches!(target, EffectTarget::Source) {
    ctx.source = new_id;
}
```

But this changes MoveZone behavior globally for all Source targets, which is fine since a source that moved zones needs its reference updated.

**Option B**: Use a dedicated `Effect::ReturnFromGraveyardWithCounter` effect primitive. This is cleaner but requires adding a new Effect variant and handler -- more work for one ability.

**Recommendation**: Option A. Minimal change, correct for all MoveZone-of-Source cases.

**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs` (MoveZone handler, line ~738)
**Action**: After `state.move_object_to_zone(id, dest)`, check if the moved id matches `ctx.source` and update it:

```rust
if let Ok((new_id, _)) = state.move_object_to_zone(id, dest) {
    if let Some(idx) = idx_opt {
        ctx.target_remaps.insert(idx, new_id);
    }
    // Update source reference if the moved object was the ability source.
    // Required for persist/undying: MoveZone moves the source from graveyard
    // to battlefield, and the subsequent AddCounter needs to find the new object.
    if id == ctx.source {
        ctx.source = new_id;
    }
    // ... existing event emission code
}
```

### Step 5: Wire Persist Keyword to Triggered Ability (Enrichment)

**File**: `/home/airbaggie/scutemob/crates/engine/src/testing/replay_harness.rs`
**Action**: In `enrich_spec_from_def`, add a block to translate `AbilityDefinition::Keyword(KeywordAbility::Persist)` into a `TriggeredAbilityDef`. This parallels the existing `WhenDies`/`WhenAttacks` enrichment blocks (~line 532-572).

However, since the builder already translates `KeywordAbility::Persist` in the keyword loop, and `enrich_spec_from_def` already propagates keywords via `spec.with_keyword(kw.clone())`, the builder's keyword loop WILL fire for Persist. No extra enrichment block is needed -- the keyword is propagated, and `builder.rs` translates it.

**Verify**: The builder's keyword-to-trigger loop at line 350 iterates `spec.keywords`, which includes keywords added by `enrich_spec_from_def`. So the pipeline is:
1. `enrich_spec_from_def` -> adds `KeywordAbility::Persist` to `spec.keywords`
2. `builder.rs` build -> iterates `spec.keywords` -> creates `TriggeredAbilityDef` for Persist

This is the same path as Ward, Prowess, Exalted, and Annihilator. No additional enrichment code needed.

### Step 6: Trigger Dispatch -- Intervening-If with Pre-Death Counters

**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs`

**Action A**: Update the `CreatureDied` match arm in `check_triggers` (line 755-795) to pass pre-death counters to the intervening-if check.

The current code calls `check_intervening_if(state, cond, *death_controller)`. For the new `SourceHadNoCounterOfType` variant, the function needs access to the pre-death counters. Two options:

**Approach**: Extend `check_intervening_if` to accept an optional `&OrdMap<CounterType, u32>` parameter for pre-death counters, OR restructure the check to handle the counter condition inline in the CreatureDied arm.

The cleanest approach: add a `pre_death_counters: Option<&im::OrdMap<CounterType, u32>>` parameter to `check_intervening_if` (defaulting to None for all other callers). Then:

```rust
InterveningIf::SourceHadNoCounterOfType(ct) => {
    pre_death_counters
        .map(|counters| !counters.contains_key(ct))
        .unwrap_or(false) // If no counter info, don't trigger
}
```

**All call sites of `check_intervening_if`** need the new parameter:
- `abilities.rs` line 775 (CreatureDied) -- pass `Some(pre_death_counters)`
- `abilities.rs` line 816 (AuraFellOff) -- pass `None`
- `abilities.rs` line 907 (generic battlefield trigger) -- pass `None`
- `resolution.rs` line 368 (resolution intervening-if) -- pass `None` (at resolution time, the card is in the graveyard with no counters; but the resolution check for persist should verify the source is still in the graveyard -- the MoveZone effect handles this naturally; if the card left the graveyard, EffectTarget::Source resolves to empty and nothing happens)

**Action B**: Update the `CreatureDied` match pattern to destructure `pre_death_counters`:

```rust
GameEvent::CreatureDied {
    new_grave_id,
    controller: death_controller,
    pre_death_counters,
    ..
} => {
    // existing code, but pass pre_death_counters to check_intervening_if
}
```

### Step 7: Unit Tests

**File**: `/home/airbaggie/scutemob/crates/engine/tests/keywords.rs`
**Tests to write**:

**7a. `test_persist_basic_returns_with_counter`**
- CR 702.79a -- creature with Persist and no -1/-1 counters dies via SBA (lethal damage), persist trigger fires, creature returns to battlefield with one -1/-1 counter.
- Setup: 2-player game; p1 has a 2/2 creature with Persist keyword and 2 damage marked.
- Pass priority -> SBA fires -> creature dies -> persist triggers -> both pass -> trigger resolves -> creature returns with -1/-1 counter (now effectively 1/1).
- Assert: creature on battlefield with one MinusOneMinusOne counter. No creature in graveyard.

**7b. `test_persist_does_not_trigger_with_minus_counter`**
- CR 702.79a (intervening-if) -- creature with Persist that already has a -1/-1 counter dies, persist does NOT trigger.
- Setup: 2-player game; p1 has a 3/3 creature with Persist keyword, one MinusOneMinusOne counter, and 3 damage marked (lethal for effective 2 toughness).
- Pass priority -> SBA fires -> creature dies -> NO persist trigger.
- Assert: creature in graveyard, NOT on battlefield.

**7c. `test_persist_second_death_no_trigger`**
- After a persist creature returns with a -1/-1 counter, if it dies again, persist does NOT trigger (it now has a -1/-1 counter).
- Setup: 2-player game; p1 has a 3/3 creature with Persist and lethal damage.
- First death: persist returns it as 2/2 (one -1/-1 counter).
- Deal enough damage to kill the 2/2. SBA fires. Persist does NOT trigger.
- Assert: creature in graveyard after second death.

**7d. `test_persist_token_trigger_but_no_return`**
- CR 702.79a + CR 704.5d -- token with persist triggers the ability but ceases to exist.
- Setup: token creature with Persist, lethal damage.
- SBA fires -> token dies -> goes to graveyard -> persist triggers (token still exists momentarily) -> CR 704.5d SBA removes token from graveyard -> trigger resolves -> MoveZone finds nothing (source gone).
- Assert: no token on battlefield or in graveyard.

**7e. `test_persist_multiplayer_apnap_ordering`**
- CR 603.3 -- multiple persist creatures die simultaneously; triggers ordered by APNAP.
- Setup: 4-player game; p1 and p3 each have a persist creature with lethal damage.
- SBA fires -> both die -> p1's persist trigger first (APNAP), p3's second -> p3's resolves first (LIFO).
- Assert: both creatures return to battlefield.

**7f. `test_persist_plus_one_cancellation`**
- CR 704.5q + 702.79a -- creature with persist returns with -1/-1 counter; later receives +1/+1 counter; SBA annihilates both; creature can persist again if it dies.
- Setup: creature with Persist returns from first death (has -1/-1 counter). Add +1/+1 counter -> SBA annihilates both -> creature has no counters -> dies again -> persist triggers.
- This test validates the full loop.

**Pattern**: Follow `test_dies_trigger_fires_on_lethal_damage_sba` at `/home/airbaggie/scutemob/crates/engine/tests/abilities.rs` line 1043.

### Step 8: Card Definition (later phase)

**Suggested card**: Kitchen Finks ({1}{G/W}{G/W}, Creature -- Ouphe, 3/2, ETB: gain 2 life, Persist)
**Alternative**: Murderous Redcap ({2}{B/R}{B/R}, Creature -- Goblin Assassin, 2/2, ETB: deal damage equal to power to any target, Persist)
**Card lookup**: Use `card-definition-author` agent.

Kitchen Finks is preferred as the first card because its ETB (gain 2 life) is simpler than Murderous Redcap's targeted ETB. It demonstrates the persist loop clearly: 3/2 -> dies -> returns as 2/1 with -1/-1 counter -> gain 2 life again.

Note: Kitchen Finks uses hybrid mana {G/W}. If hybrid mana is not yet supported in the Cost/ManaCost type, fall back to {1}{W}{G} as a simplified cost or use Glen Elendra Archmage ({3}{U}, 2/2, Flying, Persist) which uses standard mana.

### Step 9: Game Script (later phase)

**Suggested scenario**: "Kitchen Finks dies via combat damage; persist returns it; it gains 2 life on ETB; second death stays in graveyard."
**Subsystem directory**: `test-data/generated-scripts/stack/` (persist triggers use the stack)

Alternative scenario: "Board wipe kills two persist creatures from different players; APNAP ordering; both return."

## Interactions to Watch

1. **Commander SBA (CR 903.9a)**: If a commander with persist dies, the SBA gives the owner the choice to return it to the command zone. If they choose command zone, the CreatureDied event may not fire (or fires with the redirect to command zone), and persist should NOT trigger because the permanent didn't end up in the graveyard. Check: the `ZoneId::Command(_)` branch in SBA code at sba.rs:344 already skips emitting `CreatureDied`, so persist won't see the event. Correct.

2. **Rest in Peace**: If Rest in Peace is on the battlefield, the replacement effect changes "dies" to "exiled." The creature never goes to the graveyard, so `CreatureDied` is not emitted. Persist does not trigger. Correct by the existing replacement architecture.

3. **Kalitas, Traitor of Ghet**: Kalitas replaces "creature dying" with "exile + create Zombie token." Same as Rest in Peace -- the `CreatureDied` event is not emitted. Persist does not trigger.

4. **+1/+1 and -1/-1 counter annihilation (CR 704.5q)**: Already implemented in `sba.rs:850-880`. If a persist creature returns with -1/-1 and later gets +1/+1, both counters are removed as SBA. The creature then has no counters and can persist again on next death.

5. **Solemnity (prevents counters)**: If Solemnity prevents the -1/-1 counter from being placed when persist returns the creature, the creature returns with no counters. This creates an infinite persist loop. The engine's loop detection (`loop_detection.rs`) should detect this as a mandatory infinite loop and draw per CR 104.4b. Note: Solemnity is not yet in the card database -- this is a future consideration.

6. **MoveZone from graveyard to battlefield**: The `Effect::MoveZone` handler emits `PermanentEnteredBattlefield`. The ETB site in `resolution.rs` fires ETB triggers. If the persist creature has an ETB ability (like Kitchen Finks' "gain 2 life"), it fires when persist returns it. However, `MoveZone` in `effects/mod.rs` does NOT call `fire_when_enters_triggered_effects` or `register_static_continuous_effects`. This is a potential gap -- triggered ability resolution's MoveZone path may not trigger ETBs. Investigate during implementation.

7. **Source update after MoveZone (Step 4 critical issue)**: After MoveZone moves the source from graveyard to battlefield, `ctx.source` must be updated to the new ObjectId so `AddCounter` can find the permanent. See Step 4 for the fix.

## Files Modified (Summary)

| File | Change |
|------|--------|
| `crates/engine/src/state/types.rs` | Add `KeywordAbility::Persist` |
| `crates/engine/src/state/hash.rs` | Add Persist discriminant (36); update `CreatureDied` hash; add `InterveningIf::SourceHadNoCounterOfType` hash |
| `crates/engine/src/state/game_object.rs` | Add `InterveningIf::SourceHadNoCounterOfType(CounterType)` |
| `crates/engine/src/state/builder.rs` | Keyword-to-trigger translation for Persist |
| `crates/engine/src/rules/events.rs` | Add `pre_death_counters` field to `CreatureDied` |
| `crates/engine/src/rules/abilities.rs` | Update `check_triggers` to destructure `pre_death_counters`; extend `check_intervening_if` |
| `crates/engine/src/rules/sba.rs` | Capture pre-death counters at all `CreatureDied` emission sites |
| `crates/engine/src/rules/replacement.rs` | Add `pre_death_counters` param to `zone_change_events` |
| `crates/engine/src/rules/resolution.rs` | Pass `None` counters to `check_intervening_if` at resolution time |
| `crates/engine/src/effects/mod.rs` | Capture pre-death counters at `CreatureDied` emission sites; update `ctx.source` after MoveZone of Source |
| `tools/replay-viewer/src/view_model.rs` | Add `Persist` arm to `format_keyword` |
| `crates/engine/tests/keywords.rs` | 6 unit tests |

## Estimated Complexity

- **Medium-High**: The core persist logic is straightforward (trigger on death, return with counter), but the infrastructure changes are substantial:
  - `CreatureDied` event extension touches 8 emission sites across 4 files
  - `InterveningIf` extension touches hash, abilities dispatch, and resolution
  - `MoveZone` source tracking fix is required for the Sequence(MoveZone, AddCounter) pattern
- **Reusable for Undying**: All infrastructure changes (pre_death_counters on CreatureDied, InterveningIf::SourceHadNoCounterOfType, MoveZone source tracking) directly enable Undying (CR 702.93) with a near-zero-diff addition: just `KeywordAbility::Undying` + builder block using `CounterType::PlusOnePlusOne` instead of `MinusOneMinusOne`.
