# Primitive Batch Plan: PB-LKI-Power — `EffectAmount::SourcePowerAtLastKnownInformation` (LKI source-power snapshot for WhenDies / WhenLeavesBattlefield)

**Generated**: 2026-05-13
**Branch**: `feat/pb-lki-power-lki-source-powertoughness-snapshot-for-whendies` (already checked out)
**Worktree**: `/home/skydude/projects/scutemob/.worktrees/scutemob-19/`
**Primitive**: New `EffectAmount::SourcePowerAtLastKnownInformation` variant (discriminant **18**), resolved from a layer-resolved power snapshot threaded through `PendingTrigger.lki_power: Option<i32>` → `StackObject.lki_power: Option<i32>` → `EffectContext.lki_power: Option<i32>`. Snapshot is captured at trigger-fire time at `crates/engine/src/rules/sba.rs:540` (alongside the existing `pre_death_counters` block) and propagated through to the resolution-time `resolve_amount` call. Mirrors PB-LKI-CC field-for-field, swapping `OrdMap<CounterType, u32>` for `Option<i32>`.
**CR Rules**: 603.10a (LKI for leaves-battlefield triggers), 113.7a (LKI general — source moved-zone semantics), 122.2 (counters cease on zone change — drives the loss of layer-7c CDA dependencies on the graveyard'd object), 400.7 (zone change creates a new object — explains why `state.objects[graveyard_id].characteristics.power` is the printed/base value, not the on-battlefield value)
**Cards affected**: 2 confirmed in-scope (Conclave Mentor death-trigger life-gain, Juri Master of the Revue death-trigger damage). TODO sweep: 2 cards with matching comments. No additional sweep candidates found.
**Dependencies**: PB-LKI-CC (HASH 15, established the `lki_*` snapshot threading pattern through PendingTrigger → StackObject → EffectContext + the per-event-payload `pre_lba_counters` precedent on 4 GameEvent variants). PB-CD (HASH 16, established `ReplacementTrigger::WouldPlaceCounters.counter_filter` for Conclave Mentor's replacement half — already shipped). All prerequisites in place.
**Deferred items from prior PBs**: OOS-LKI-Power seed in `memory/primitives/pb-retriage-CC.md:554`. This batch closes the seed.
**Toughness variant decision**: **NOT shipped this batch.** Both in-scope cards plus the rulings (Conclave Mentor 2020-06-23, Juri 2020-11-10) explicitly cite **power**; sweep returned no SelfDies/SelfLeavesBattlefield card needing toughness LKI. Discriminant 19 is reserved for future `SourceToughnessAtLastKnownInformation` if/when a real card surfaces it. Justification: shipping unused variants is dead surface; the PB-LKI-CC OOS-LKI-3/4 precedent is to file seeds, not pre-stub variants.

---

## Step 0 — Verification of the OOS-LKI-Power seed claims (load-bearing audit)

The PB-LKI-CC reviewer wrote at `memory/primitives/pb-retriage-CC.md:554`:
> OOS-LKI-Power: WhenDies / WhenLeavesBattlefield triggers reading the source's own
> P/T at LKI. Conclave Mentor "you gain life equal to its power"; Juri Master "deals
> damage equal to its power to any target". Same shape as PB-LKI-CC's lki_counters
> threading, swapping OrdMap<CounterType,u32> for Option<i32>.

**Verified by reading the engine sources**:

1. `crates/engine/src/rules/sba.rs:540` confirms the snapshot site. The block currently captures:
   ```rust
   let (owner, pre_death_controller, pre_death_counters) = match state.objects.get(&id) {
       Some(obj) => (obj.owner, obj.controller, obj.counters.clone()),
       None => continue,
   };
   ```
   This site has live access to the on-battlefield `state.objects[id]`, where layer-resolved P/T is computable via `crate::rules::layers::calculate_characteristics(state, id)`. Snapshotting power here is mechanically identical to snapshotting counters.

2. `crates/engine/src/state/state/mod.rs:412-420` (per the PB-LKI-CC Step 0 audit) shows `move_object_to_zone` constructs a NEW `GameObject` with reset `counters: OrdMap::new()`. By the same mechanism, the new object's `Characteristics` is rebuilt from the printed face — battlefield-only continuous effects (anthems, equipment, +1/+1 counters) are gone. Per CR 122.2 and CR 400.7, this is correct and must not be changed.

3. `crates/engine/src/effects/mod.rs:6002-6025` (`EffectAmount::PowerOf` arm) reads via:
   ```rust
   if obj.zone == ZoneId::Battlefield {
       crate::rules::layers::calculate_characteristics(state, id)
           .unwrap_or_else(|| obj.characteristics.clone())
           .power
   } else {
       obj.characteristics.power
   }
   ```
   For a graveyard'd source, `obj.characteristics.power` is the printed P/T (no layer pass), so a "When this dies, you gain life equal to its power" trigger using `EffectAmount::PowerOf(EffectTarget::Source)` will resolve to the printed power, NOT the boosted-on-battlefield power. **The Conclave Mentor 2020-06-23 ruling and the Juri 2020-11-10 ruling both state explicitly: "use its power as it last existed on the battlefield."**

4. `EffectAmount::PowerOfSacrificedCreature` (PB-P, disc 15) is the CR 608.2b cost-payment-LKI sibling. It snapshots `Vec<i32>` of layer-resolved powers at sacrifice cost-payment time and stores them in `EffectContext.sacrificed_creature_powers` (effects/mod.rs:130-134). This precedent shows that layer-resolved P/T snapshotting via `calculate_characteristics` is the established pattern.

5. **Conclusion**: the gap is real, the snapshot site exists, the dispatch pattern is established. PB-LKI-Power follows PB-LKI-CC's chain swapping `OrdMap<CounterType, u32>` for `Option<i32>`.

---

## Step 1 — CR research (verbatim from MCP rules server, queried 2026-05-13)

### CR 603.10 / 603.10a — leaves-battlefield triggers look back in time

> 603.10. Normally, objects that exist immediately after an event are checked to see if the event matched any trigger conditions, and continuous effects that exist at that time are used to determine what the trigger conditions are and what the objects involved in the event look like. However, some triggered abilities are exceptions to this rule; the game "looks back in time" to determine if those abilities trigger, using the existence of those abilities and the appearance of objects immediately prior to the event.
>
> 603.10a Some zone-change triggers look back in time. These are leaves-the-battlefield abilities, abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object that all players can see is put into a hand or library.

### CR 113.7 / 113.7a — LKI for stack abilities resolving after source has moved

> 113.7. The source of an ability is the object that generated it. The source of an activated ability on the stack is the object whose ability was activated. The source of a triggered ability (other than a delayed triggered ability) on the stack, or one that has triggered and is waiting to be put on the stack, is the object whose ability triggered. To determine the source of a delayed triggered ability, see rules 603.7d–f.
>
> 113.7a Once activated or triggered, an ability exists on the stack independently of its source. Destruction or removal of the source after that time won't affect the ability. Note that some abilities cause a source to do something (for example, "This creature deals 1 damage to any target") rather than the ability doing anything directly. In these cases, any activated or triggered ability that references information about the source for use while announcing an activated ability or putting a triggered ability on the stack checks that information when the ability is put onto the stack. Otherwise, it will check that information when it resolves. **In both instances, if the source is no longer in the zone it's expected to be in at that time, its last known information is used.** The source can still perform the action even though it no longer exists.

### CR 400.7 — zone change creates a new object

> 400.7. An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.

### CR 122.2 — counters cease on zone change (driver of layer-7c loss)

> 122.2. Counters on an object are not retained if that object moves from one zone to another. The counters are not "removed"; they simply cease to exist. See rule 400.7.

**Implication for PB-LKI-Power**: a graveyard'd object's `Characteristics` reflects the printed face only — anthems are gone, +1/+1 counters are gone (per CR 122.2), equipment is detached, P/T-setting layers don't apply. By the time a SelfDies trigger resolves, `state.objects[source].characteristics.power` is the printed value. The two cards' rulings explicitly say to use "power as it last existed on the battlefield" (Conclave Mentor 2020-06-23 ruling 4; Juri Master 2020-11-10 ruling 1). Therefore the snapshot must be the **layer-resolved** power, captured BEFORE `move_object_to_zone` runs.

---

## Step 2 — Path choice (Path A vs Path B)

### Chosen: **Path A** — new `EffectAmount` variant + LKI snapshot threaded through `PendingTrigger` → `StackObject` → `EffectContext`. Mirror PB-LKI-CC's structure exactly, swapping `OrdMap<CounterType, u32>` for `Option<i32>`.

### Why Path A (same reasoning as PB-LKI-CC)

1. **Honors CR 122.2 / CR 400.7 invariant**: the graveyard `GameObject.characteristics.power` stays at printed-face. No risk of cascading bugs at the dozens of sites that read `obj.characteristics.power` directly.
2. **Reuses existing snapshot infrastructure**: `pre_death_counters` is captured at `sba.rs:540` already. Adding `pre_death_power: Option<i32>` and `pre_death_toughness: Option<i32>` (the latter behind a feature gate / not shipped this batch) at the same site is a 3-line edit.
3. **Aligns with PB-P / PB-LKI-CC precedent**: PB-P snapshots LKI power on `EffectContext.sacrificed_creature_powers: Vec<i32>` for sacrifice cost-payment LKI. PB-LKI-CC threads `EffectContext.lki_counters: Option<OrdMap<CounterType, u32>>` for trigger-fire LKI. PB-LKI-Power adds `EffectContext.lki_power: Option<i32>` for trigger-fire LKI on the source's P/T axis.
4. **Type-system enforcement**: a new variant means card authors can't accidentally write `PowerOf(Source)` on a death trigger and silently get the printed value. They explicitly write `SourcePowerAtLastKnownInformation`. The compiler distinguishes the two semantics.
5. **Discriminating the two arms preserves the live-power semantic**: the existing `EffectAmount::PowerOf(EffectTarget::Source)` arm continues to read live battlefield power and is still correct for "while on battlefield" effects (e.g. activated ability on a permanent that reads its own power, like Jagged-Scar Archers `{T}: deal damage equal to its power`). Tests, card defs, and call sites for the live arm all keep working unchanged.

### Why NOT Path B (preserve power on the graveyard object's characteristics)

- Breaks CR 122.2 / CR 400.7 explicitly (the graveyard object is a NEW object with no memory of its battlefield existence).
- Would force layer-7c re-evaluation for graveyard objects (currently scoped to battlefield), causing massive layer-system blast radius.
- Same rejection rationale as PB-LKI-CC.

### Why NOT Path C (read directly from the most recent `CreatureDied` event in `state.events_history`)

- `GameEvent` history is not in scope for effect resolution; tying `resolve_amount` to history scanning is a layering violation.
- LKI for `WhenLeavesBattlefield` (non-creature permanents) does NOT emit `CreatureDied` — only creatures do. We need a unified mechanism. Threading via `PendingTrigger` works for both creature and non-creature LBA. Same rejection rationale as PB-LKI-CC.

---

## Step 3 — Engine architecture walk (full dispatch chain — mirrors PB-LKI-CC)

### Site 1 — DSL: new `EffectAmount` variant at `crates/engine/src/cards/card_definition.rs`

After line 2398 (after `EffectAmount::CounterCountAtLastKnownInformation { counter }`), add a new variant. Insert before the closing `}` of `enum EffectAmount` at line 2399:

```rust
/// CR 603.10a / CR 113.7a: Source's layer-resolved power from last-known information
/// (LKI). Used by WhenDies / WhenLeavesBattlefield triggers whose effect needs to
/// know the source's power as it last existed on the battlefield. The snapshot is
/// captured at trigger-fire time (sba.rs `pre_death_power` block, alongside
/// `pre_death_counters`) and threaded through `PendingTrigger.lki_power`,
/// `StackObject.lki_power`, and `EffectContext.lki_power`.
///
/// **Versus `PowerOf(EffectTarget::Source)`**: the existing `PowerOf` variant reads
/// the source's live characteristics — for a battlefield permanent it returns the
/// layer-resolved value, for a non-battlefield object it returns
/// `obj.characteristics.power` (the printed/base face, since layers don't apply
/// off-battlefield). Use `PowerOf` for abilities that fire while the source is still
/// on the battlefield (e.g. "{T}: deal damage equal to this creature's power"). Use
/// `SourcePowerAtLastKnownInformation` for any effect on a leaves-the-battlefield
/// trigger (CR 603.10a) where `state.objects[ctx.source].characteristics.power` is
/// the printed value by the time the trigger resolves (CR 122.2 / CR 400.7 — counters
/// and continuous effects cease on zone change). The implicit target is the trigger
/// source (LKI).
///
/// **Returns 0** if `ctx.lki_power` is `None` (e.g. variant authored on a non-LKI
/// trigger by mistake, OR the source had `Characteristics.power = None` such as a
/// non-creature permanent with no inherent power). Defensive default — the card
/// author should pair this variant only with WhenDies / WhenLeavesBattlefield
/// triggers on creatures.
///
/// **Cards using this variant**:
/// - Conclave Mentor: "When this creature dies, you gain life equal to its power."
///   (Ruling 2020-06-23: "Use Conclave Mentor's power as it last existed on the
///   battlefield to determine how much life you gain.")
/// - Juri, Master of the Revue: "When Juri dies, it deals damage equal to its power
///   to any target." (Ruling 2020-11-10: "use its power from when it was last on
///   the battlefield to determine how much damage is dealt. If that power was 0 or
///   less, Juri deals no damage.")
///
/// Discriminant 18 (state/hash.rs). Discriminant 19 reserved for a future
/// `SourceToughnessAtLastKnownInformation` if a real card surfaces it (none in scope
/// for PB-LKI-Power).
SourcePowerAtLastKnownInformation,
```

**Note on shape**: no fields. The implicit target is `EffectTarget::Source` (the LKI source); there is no notion of "power of a different LKI object" in the SelfDies/SelfLeavesBattlefield context (the trigger fires on a single object's zone change). Keeping the variant fieldless avoids the runner needing to plumb arbitrary `EffectTarget` resolution into LKI-snapshot lookup.

**Note on Juri ruling clamp**: ruling says "If that power was 0 or less, Juri deals no damage." `Effect::DealDamage` already handles 0-or-negative damage by emitting a 0-amount damage event (clamped). The runner does NOT need a special branch in the variant — clamping happens at the effect-execution boundary.

### Site 2 — `EffectContext` field at `crates/engine/src/effects/mod.rs` (struct ~line 47-143)

Add a new field after `lki_counters` (line 142). Use `Option<i32>` to distinguish "no LKI captured" (None) from "LKI captured but source had power=None" (None) from "LKI captured with valid power" (Some(n)).

**Caveat**: both "no LKI captured" and "LKI captured but source had no inherent power" map to `None` and resolve to 0 from `resolve_amount`. This is intentional and matches PB-LKI-CC's `Option<OrdMap>` semantics.

```rust
/// CR 603.10a / CR 113.7a: LKI source-power snapshot for leaves-battlefield triggers.
/// Populated by `flush_pending_triggers` (abilities.rs) when a `WhenDies` /
/// `WhenLeavesBattlefield` trigger is put on the stack, capturing the source's
/// layer-resolved power (via `calculate_characteristics`) immediately before zone
/// change. Threaded into `EffectContext` at trigger resolution time
/// (resolution.rs).
/// Read by `EffectAmount::SourcePowerAtLastKnownInformation`.
/// `None` for non-LKI trigger contexts OR for sources whose
/// `Characteristics.power` was `None` (e.g. non-creature permanents); lookup
/// returns 0 in both cases.
pub lki_power: Option<i32>,
```

Update both `EffectContext::new()` (~line 144-172) and `EffectContext::new_with_kicker()` (~line 174-205) to initialize `lki_power: None`.

Also update the two inner-context construction sites at `effects/mod.rs:2486-2496` and `:2521-2531` (the `inner_ctx` rebuilds inside execute_effect_inner — `Sequence` and `ForEach`) to propagate `lki_power: ctx.lki_power` (mirror the existing `lki_counters: ctx.lki_counters.clone()` line — Option<i32> is Copy, no clone needed).

Also update the `check_condition` ctx construction at `effects/mod.rs:7340-7345` (the bare-bones ctx for condition evaluation) to initialize `lki_power: None` (same pattern as `lki_counters: None`).

### Site 3 — `PendingTrigger` field at `crates/engine/src/state/stubs.rs` (struct ~line 241-411)

Add a new field after `lki_counters` (line 411). Use `Option<i32>` to match `EffectContext` shape.

**Note**: PB-LKI-CC chose `OrdMap<CounterType, u32>` (flat, default-empty) on PendingTrigger and `Option<OrdMap>` on EffectContext to disambiguate "no LKI capture" from "captured but empty". For `lki_power`, BOTH PendingTrigger and EffectContext use `Option<i32>` because i32 has no natural empty/zero distinction (zero is a valid LKI power for a 0/X creature like Tarmogoyf-with-only-mountains). `None` consistently means "no LKI captured" across both structs.

```rust
/// CR 603.10a / CR 113.7a: LKI source-power snapshot for WhenDies / WhenLeavesBattlefield triggers.
/// Captured at trigger queueing time (abilities.rs CreatureDied/AuraFellOff/PermanentDestroyed/
/// ObjectExiled/ObjectReturnedToHand arms) from the corresponding GameEvent's
/// `pre_death_power` / `pre_lba_power` payload. Threaded through
/// `flush_pending_triggers` into `StackObject.lki_power`, then into
/// `EffectContext.lki_power` at resolution. Read by
/// `EffectAmount::SourcePowerAtLastKnownInformation`.
/// `None` for triggers that don't fire from a leaves-battlefield event OR for
/// sources whose layer-resolved power was `None` (non-creatures).
#[serde(default)]
pub lki_power: Option<i32>,
```

Update `PendingTrigger::blank()` (line 413-457) to initialize `lki_power: None`.

### Site 4 — `StackObject` field at `crates/engine/src/state/stack.rs` (struct ~line 158-475)

Add a new field after `lki_counters` (line 475). Mirror the `Option<i32>` shape:

```rust
/// CR 603.10a / CR 113.7a: LKI source-power snapshot for WhenDies / WhenLeavesBattlefield triggers.
/// Set from PendingTrigger::lki_power when the trigger is flushed to the stack
/// (abilities.rs flush_pending_triggers ~line 7467). Read at resolution time
/// (resolution.rs ~line 2057) into EffectContext.lki_power.
/// `None` for stack objects that are not LBA triggered abilities or whose source
/// had no inherent power.
#[serde(default)]
pub lki_power: Option<i32>,
```

Update `StackObject::trigger_default()` (line 477-541) to initialize `lki_power: None`.

### Site 5 — Capture LKI power at SBA snapshot time at `crates/engine/src/rules/sba.rs:540`

The current snapshot block:
```rust
let (owner, pre_death_controller, pre_death_counters) = match state.objects.get(&id) {
    Some(obj) => (obj.owner, obj.controller, obj.counters.clone()),
    None => continue,
};
```

Extend to capture layer-resolved power:
```rust
let (owner, pre_death_controller, pre_death_counters, pre_death_power) =
    match state.objects.get(&id) {
        Some(obj) => {
            // CR 603.10a: capture layer-resolved power BEFORE move_object_to_zone
            // (which destroys battlefield-only continuous effects). Use
            // calculate_characteristics for the boosted on-battlefield value;
            // fall back to base characteristics if layer calc returns None
            // (e.g. object newly registered without a layer pass yet).
            let lki_power = crate::rules::layers::calculate_characteristics(state, id)
                .and_then(|c| c.power)
                .or(obj.characteristics.power);
            (obj.owner, obj.controller, obj.counters.clone(), lki_power)
        }
        None => continue,
    };
```

### Site 6 — `GameEvent::CreatureDied` payload extension at `crates/engine/src/rules/events.rs:207-222`

Add a new field after `pre_death_counters` (line 221):
```rust
/// CR 603.10a / CR 113.7a: layer-resolved power of the dying creature at LKI.
/// Used by `EffectAmount::SourcePowerAtLastKnownInformation` for "when this dies,
/// gain life equal to its power" / "deal damage equal to its power" patterns
/// (Conclave Mentor, Juri Master of the Revue). Captured BEFORE move_object_to_zone
/// at sba.rs:540 alongside `pre_death_counters`.
/// `None` if the dying object had no power characteristic.
#[serde(default)]
pre_death_power: Option<i32>,
```

Update the emit site at `crates/engine/src/rules/sba.rs:565-572` (CreatureDied push):
```rust
events.push(GameEvent::CreatureDied {
    object_id: id,
    new_grave_id: new_id,
    controller: pre_death_controller,
    pre_death_counters: pre_death_counters.clone(),
    pre_death_power,
});
```

Also update the redirect-to-graveyard / redirect-to-exile branches in sba.rs (the SBA replacement-redirect block ~lines 575-600+) to propagate `pre_death_power` into the resulting `ObjectExiled`/`ObjectReturnedToHand`/etc. events. Reuse the same `pre_death_power` local computed at sba.rs:540.

### Site 7 — `GameEvent::AuraFellOff` / `PermanentDestroyed` / `ObjectExiled` / `ObjectReturnedToHand` payload extensions at `crates/engine/src/rules/events.rs`

Same pattern. Add `pre_lba_power: Option<i32>` field with `#[serde(default)]` to each of these 4 variants:
- `AuraFellOff` (events.rs:231-240, after `pre_lba_counters` at 239)
- `ObjectExiled` (events.rs:357-371, after `pre_lba_counters` at 370)
- `PermanentDestroyed` (events.rs:375-385, after `pre_lba_counters` at 384)
- `ObjectReturnedToHand` (events.rs:456-470, after `pre_lba_counters` at 469)

Field doc:
```rust
/// CR 603.10a: layer-resolved power snapshot captured before `move_object_to_zone`
/// destroys battlefield-only continuous effects. Populated when the object was on
/// the battlefield immediately before zone change; `None` for non-battlefield
/// sources OR for objects with no power characteristic.
/// Used by `SelfLeavesBattlefield` triggers to resolve
/// `EffectAmount::SourcePowerAtLastKnownInformation`.
#[serde(default)]
pre_lba_power: Option<i32>,
```

### Site 8 — Update ALL emit sites for the 5 affected GameEvent variants

Per the PB-LKI-CC fix-phase resolution, `pre_lba_counters` is emitted at ~35 sites across:
- `crates/engine/src/rules/abilities.rs` (~10 sites — Ninjutsu, Embalm, Eternalize, Encore, Scavenge, Forage, sacrifice cost-payment paths, etc.)
- `crates/engine/src/rules/casting.rs` (~5 sites — exile-self alt-cost, etc.)
- `crates/engine/src/rules/engine.rs` (~5 sites)
- `crates/engine/src/rules/resolution.rs` (~10 sites — generic effect-execution emit paths)
- `crates/engine/src/rules/turn_actions.rs` (~5 sites — end-step exile, cleanup)
- `crates/engine/src/rules/sba.rs` (the redirect-to-X paths inside the death loop)

**Implementation guidance for the runner**: grep for `pre_lba_counters:` and `pre_death_counters:` across the engine — every emit site needs a sibling `pre_lba_power: ...` / `pre_death_power: ...` field. For battlefield-source emit sites, capture `calculate_characteristics(state, id).and_then(|c| c.power).or(obj.characteristics.power)` BEFORE `move_object_to_zone`. For non-battlefield-source emit sites (graveyard→exile in Delve/Escape/Forage; hand→graveyard in discard), set `pre_lba_power: None` (no battlefield existence to snapshot from).

**Pattern to mirror**: the PB-LKI-CC E1 fix-phase that did the equivalent for `pre_lba_counters`. Look at `git log -p --grep="E1"` on the `feat/pb-lki-cc-...` branch (commit `4fde5d66` or later) for the exact set of sites updated.

### Site 9 — Update the trigger arms in abilities.rs `check_triggers` to propagate `pre_death_power` / `pre_lba_power` → `PendingTrigger.lki_power`

5 trigger arms must be updated, each currently propagates `lki_counters`:

1. **CreatureDied / SelfDies push** at `abilities.rs:4033-4045` — change `pre_death_counters` destructure (line 3985) to also bind `pre_death_power`, then add `lki_power: pre_death_power` to the `PendingTrigger { ... }` struct literal (alongside `lki_counters: pre_death_counters.clone()` at line 4043).

2. **CreatureDied / SelfLeavesBattlefield push** at `abilities.rs:4066-4076` — same change. Add `lki_power: pre_death_power`.

3. **AuraFellOff / SelfLeavesBattlefield push** at `abilities.rs:4351-4410` (struct-literal style at line 4379-4408) — destructure `pre_lba_power: aura_lki_power` from the event (line 4353-4354), and add `lki_power: aura_lki_power` to the `PendingTrigger` struct literal (currently uses explicit field-by-field listing — add the new field next to `lki_counters: aura_lki_counters.clone()` at line 4406).

4. **PermanentDestroyed / SelfLeavesBattlefield push** at `abilities.rs:5258-5269` — destructure `pre_lba_power: destroyed_lki_power` (line 5223 area), add `lki_power: destroyed_lki_power` to the `PendingTrigger` (line 5262 area, alongside `lki_counters: destroyed_lki_counters.clone()`).

5. **ObjectExiled / SelfLeavesBattlefield push** at `abilities.rs:5311-5322` — same pattern. Add `lki_power: exiled_lki_power`.

6. **ObjectReturnedToHand / SelfLeavesBattlefield push** at `abilities.rs:5364-5375` — same pattern. Add `lki_power: bounced_lki_power`.

(That's 6 actual sites; 5 GameEvent variants but CreatureDied has 2 trigger arms — SelfDies and SelfLeavesBattlefield.)

**AnyCreatureDies arm at abilities.rs:4318 (per PB-LKI-CC review)**: this fires "whenever a creature dies" triggers on OTHER permanents (Blood Artist family). The dying creature is the *triggering object*, not the trigger source. Per the PB-LKI-CC AnyCreatureDies decision (Risk #1), the dying creature's LKI is NOT propagated to OTHER permanents' triggers in this batch. **STOP-AND-FLAG**: if a card surfaces requiring the dying creature's power on an AnyCreatureDies trigger, file as OOS-LKI-Power-2 (mirrors PB-LKI-CC's OOS-LKI-4 for AnyCreatureDies counter-LKI). Do NOT widen scope.

### Site 10 — Thread LKI through `flush_pending_triggers` at `crates/engine/src/rules/abilities.rs:7467`

The flush site already sets `stack_obj.lki_counters = trigger.lki_counters.clone()`. Add immediately after:
```rust
// CR 603.10a / CR 113.7a: Propagate LKI source-power snapshot from PendingTrigger
// to StackObject so resolution.rs can build EffectContext.lki_power.
stack_obj.lki_power = trigger.lki_power;  // Option<i32> is Copy
```

### Site 11 — Build `EffectContext.lki_power` at resolution time at `crates/engine/src/rules/resolution.rs`

Two sites mirror PB-LKI-CC's `lki_counters` plumbing:

**Path 1 — CardDef-registry effect path (~line 2057-2061)**:

After existing `ctx.lki_counters = if stack_obj.lki_counters.is_empty() { None } else { Some(...) };`, add:
```rust
// CR 603.10a / CR 113.7a: Propagate LKI source-power snapshot for
// EffectAmount::SourcePowerAtLastKnownInformation.
ctx.lki_power = stack_obj.lki_power;  // Option<i32> is Copy
```

**Path 2 — Characteristics-based effect path (~line 2132-2136)**:

After the analogous `lki_counters` block at 2132-2136, add the same `ctx.lki_power = stack_obj.lki_power;` line.

### Site 12 — `EffectAmount::SourcePowerAtLastKnownInformation` resolution at `crates/engine/src/effects/mod.rs` (`resolve_amount`)

Add a new arm to `resolve_amount` (effects/mod.rs:5913). Insert after the `EffectAmount::CounterCountAtLastKnownInformation` arm at line 6360-6365:

```rust
// CR 603.10a / CR 113.7a: Read source's layer-resolved power from LKI snapshot
// captured at trigger-fire time. Returns 0 if no LKI was captured (variant misused
// on a non-LBA trigger, or source had no inherent power). The implicit target is
// the trigger source. Per Juri ruling 2020-11-10: "If that power was 0 or less,
// Juri deals no damage" — this is handled at the Effect::DealDamage boundary
// (clamping); the resolver returns the raw value (which may be negative).
EffectAmount::SourcePowerAtLastKnownInformation => ctx.lki_power.unwrap_or(0),
```

### Site 13 — `resolve_cda_amount` at `crates/engine/src/rules/layers.rs:1620`

CDAs evaluate continuously while the source is on the battlefield (CR 613.4c); LKI snapshots are NOT relevant. Add to `resolve_cda_amount` after the `CounterCountAtLastKnownInformation` arm at line 1620:

```rust
// CR 613: CDAs cannot reference LKI — they evaluate continuously while the source
// is on the battlefield, where power is live. Returns 0 as defensive default.
// Card authors should not pair SourcePowerAtLastKnownInformation with a CDA;
// use `PowerOf(EffectTarget::Source)` instead.
EffectAmount::SourcePowerAtLastKnownInformation => 0,
```

### Site 14 — `HashInto for EffectAmount` at `crates/engine/src/state/hash.rs:4529-4534`

Add a new arm after the `CounterCountAtLastKnownInformation` arm at lines 4529-4534. New discriminant **18**:

```rust
// PB-LKI-Power (discriminant 18) — LKI source-power snapshot for WhenDies /
// WhenLeavesBattlefield triggers. CR 603.10a / CR 113.7a / CR 122.2 / CR 400.7.
EffectAmount::SourcePowerAtLastKnownInformation => 18u8.hash_into(hasher),
```

**Note: discriminant 19 is reserved for `SourceToughnessAtLastKnownInformation` if a future card surfaces toughness LKI. Do NOT add the variant in PB-LKI-Power scope** — the principle from PB-LKI-CC's OOS-LKI-N seeds is to file seeds, not pre-stub variants.

### Site 15 — `HashInto for PendingTrigger` and `HashInto for StackObject` at `crates/engine/src/state/hash.rs`

Both structs are hashed via existing `HashInto` impls. The new `lki_power: Option<i32>` field must be added to BOTH hash impls. Generic `Option<T>` HashInto is already implemented at hash.rs:191-201 (tag byte 0/1, then payload), so the encoding is automatic — no manual tag-byte work.

**For PendingTrigger** at hash.rs ~line 2154-2157 (find by searching for `self.lki_counters` in the PendingTrigger impl, currently around line 2154):

After the `for (ct, count) in self.lki_counters.iter() { ... }` block, add:
```rust
// CR 603.10a: LKI source-power snapshot — must be hashed for replay determinism.
// Generic Option<i32> impl encodes as tag byte (0=None, 1=Some) + 4-byte LE i32.
self.lki_power.hash_into(hasher);
```

**For StackObject** at hash.rs ~line 3031-3034 (find by searching for `self.lki_counters` in the StackObject impl, currently around line 3031):

After the analogous `for (ct, count)` block, add:
```rust
// CR 603.10a: LKI source-power snapshot.
self.lki_power.hash_into(hasher);
```

### Site 16 — `HashInto for EffectContext`

EffectContext is NOT hashed (per PB-P precedent for `sacrificed_creature_powers` — it's runtime resolution scratch only, not part of the canonical state hash). The `lki_counters` field on EffectContext is also not hashed (only the StackObject and PendingTrigger fields are, since they're the persistent serialized form). Same applies to `lki_power` — no EffectContext hash arm needed.

**Verify by reading**: the PB-LKI-CC plan explicitly noted this: "EffectContext is runtime scratch, not hashed." The plan should NOT add a hash arm for `EffectContext.lki_power`.

### Site 17 — `HashInto` for the 5 affected GameEvent variants

**Decision**: leave GameEvent hash arms as-is, mirroring the PB-LKI-CC precedent.

The PB-LKI-CC review (and current state of `state/hash.rs:3386-3553`) shows that the 4 LBA GameEvent variants (`AuraFellOff`, `ObjectExiled`, `PermanentDestroyed`, `ObjectReturnedToHand`) hash arms use `..` and DO NOT hash `pre_lba_counters`. Only `CreatureDied` (line 3362-3377) hashes its `pre_death_counters`.

This is an existing inconsistency that PB-LKI-CC chose not to resolve. PB-LKI-Power should follow the same pattern:
- **CreatureDied hash arm (line 3362-3377)**: ADD `pre_death_power.hash_into(hasher);` after the existing `pre_death_counters` loop. This keeps CreatureDied internally consistent (it hashes ALL its LKI fields).
- **AuraFellOff / ObjectExiled / PermanentDestroyed / ObjectReturnedToHand hash arms**: leave the `..` pattern unchanged — do NOT add `pre_lba_power` to the hash. **File as OOS-LKI-Power-3 (LOW priority, deferred)**: "GameEvent LBA variants don't hash pre_lba_counters or pre_lba_power. Pre-existing inconsistency; PB-LKI-CC and PB-LKI-Power both intentionally preserve to minimize blast radius. Resolution determinism is preserved because PendingTrigger and StackObject DO hash these fields, and the events are derived state."

This is a deliberate scope-limiting choice consistent with the PB-LKI-CC precedent. **STOP-AND-FLAG**: if the runner wants to add hash arms to all 4 LBA variants symmetrically, that's a separate sentinel-bumping change — file as OOS, do NOT widen this PB.

### Site 18 — `HASH_SCHEMA_VERSION` bump at `crates/engine/src/state/hash.rs:95`

Bump `HASH_SCHEMA_VERSION: u8 = 16` → `17`. Append history entry 17 after line 94:

```rust
/// - 17: PB-LKI-Power (2026-05-13) — `EffectAmount::SourcePowerAtLastKnownInformation`
///   (disc 18) reads LKI source-power snapshot for WhenDies / WhenLeavesBattlefield
///   triggers (CR 603.10a, CR 113.7a, CR 122.2, CR 400.7). New `lki_power: Option<i32>`
///   field on `PendingTrigger` (state/stubs.rs), `StackObject` (state/stack.rs), and
///   `EffectContext` (effects/mod.rs). Snapshot is captured at trigger-fire time at
///   sba.rs:540 (creature death) via `calculate_characteristics` for the layer-resolved
///   power, alongside the existing `pre_death_counters` block; mirrored at all
///   `pre_lba_counters` emit sites for AuraFellOff/PermanentDestroyed/ObjectExiled/
///   ObjectReturnedToHand. New `pre_death_power: Option<i32>` field on
///   `GameEvent::CreatureDied` (hashed) and `pre_lba_power: Option<i32>` on the 4
///   LBA event variants (NOT hashed — preserves PB-LKI-CC precedent of leaving the
///   `..` pattern intact on LBA hash arms; PendingTrigger and StackObject hashing
///   covers determinism). Wire format change: PendingTrigger and StackObject gain
///   serialized `Option<i32>` fields encoded as 1-byte tag + 4-byte LE i32 (when
///   Some); pre-PB-LKI-Power replays are not forward-compatible (`#[serde(default)]`
///   handles missing field as None). Discriminant 19 reserved for future
///   `SourceToughnessAtLastKnownInformation` (no in-scope card needs it). Unblocks
///   Conclave Mentor and Juri Master of the Revue death triggers.
```

### Site 19 — Sentinel-assertion test files (sweep targets)

Verified via grep `HASH_SCHEMA_VERSION` in `crates/engine/tests/`. **9 files** all assert `HASH_SCHEMA_VERSION == 16u8` and must be updated to assert `17u8`. The PB-LKI-Power plan does NOT bulk-rename functions (PB-LKI-CC ran into churn); only the assertion value + the message string need to change.

| File | Line | Action |
|---|---|---|
| `crates/engine/tests/primitive_pb_cc_a.rs` | 101 | `16u8` → `17u8`; update message to cite "PB-LKI-Power bumped HASH_SCHEMA_VERSION 16→17 (EffectAmount::SourcePowerAtLastKnownInformation, CR 603.10a / 113.7a)". |
| `crates/engine/tests/primitive_pb_cc_c_followup.rs` | 400 | Same. |
| `crates/engine/tests/primitive_pb_ts.rs` | 369 | Same. |
| `crates/engine/tests/primitive_pb_lki_cc.rs` | 440 | Same. |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | 411 | Same. |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | 868 | Same. |
| `crates/engine/tests/effect_sacrifice_permanents_filter.rs` | 136 | Same. |
| `crates/engine/tests/pbn_subtype_filtered_triggers.rs` | 558 | Same. |
| `crates/engine/tests/pbp_power_of_sacrificed_creature.rs` | 782 | Same. |
| `crates/engine/tests/pbd_damaged_player_filter.rs` | 597 | Same. |

That's 10 sentinel sites across 9 files (pbt_up_to_n_targets has 2). The runner MUST rerun `grep -rn "HASH_SCHEMA_VERSION, 16" crates/engine/tests/` after impl to verify no additional sentinel was added between plan and impl.

### Site 20 — replay-viewer view_model + TUI stack_view (per `MEMORY.md` 50%-miss-rate gotcha)

Adding a new `EffectAmount` variant does NOT add a new variant to `StackObjectKind` or `KeywordAbility`, so the gotcha files (`tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`) should not need new arms. **However**, the runner MUST run `cargo build --workspace` (not just `cargo build`) after impl to surface any indirect breakage from the new `EffectAmount` arm if any tool exhaustively matches on it.

Adding new fields to GameEvent variants is a struct-literal-construction concern: any pattern-match arm using exhaustive struct destructuring (no `..`) would break. PB-LKI-CC's fix-phase added `..` to several `state/hash.rs` match arms; the same may be needed here.

### Site 21 — helpers.rs prelude

`crates/engine/src/cards/helpers.rs` re-exports `EffectAmount` already. The new variant is accessible via `EffectAmount::SourcePowerAtLastKnownInformation`; no new export needed.

---

## Step 4 — Card scope verification (MCP oracle lookup + sweep)

### Pre-existing TODO sweep (MANDATORY per planning protocol)

Ran `grep -rn "TODO.*\(LKI.*[Pp]ower\|SourcePower\|EffectAmount::SourcePower\|OOS-LKI-Power\)" crates/engine/src/cards/defs/`:

Result: **2 cards** with matching comments:
1. `crates/engine/src/cards/defs/conclave_mentor.rs:41` — TODO names "OOS-LKI-Power" and "LKI source power" explicitly.
2. `crates/engine/src/cards/defs/juri_master_of_the_revue.rs:38` — TODO names "EffectAmount::SourcePower" explicitly.

Both are **forced adds** — already in scope per the brief. No additional cards self-identify as needing this primitive.

Broader sweep `grep -rn "equal to its power" crates/engine/src/cards/defs/`:
- Other "equal to its power" hits (`swords_to_plowshares`, `jagged_scar_archers`, `bridgeworks_battle`, `warstorm_surge`, `brash_taunter`, `archdruids_charm`, `ram_through`, `eomer_king_of_rohan`, `wolverine_riders`, `infectious_bite`, `legolas_quick_reflexes`, `legolasquick_reflexes`) are NOT SelfDies/SelfLeavesBattlefield contexts — they're spells, ETB triggers on the entering creature, granted activated abilities, fight effects, or live-source activated abilities. None need LKI power.

**TODO sweep result: 2 cards. Both already in scope. Yield: 2.**

### Confirmed targets (≥2 — meets PB threshold)

#### 1. Conclave Mentor — re-author death trigger (file: `crates/engine/src/cards/defs/conclave_mentor.rs`)

**Oracle text** (MCP, verified 2026-05-13):
> If one or more +1/+1 counters would be put on a creature you control, that many plus one +1/+1 counters are put on that creature instead.
> When this creature dies, you gain life equal to its power.

**Rulings (load-bearing)**:
- 2020-06-23: "Use Conclave Mentor's power as it last existed on the battlefield to determine how much life you gain." — **explicitly mandates LKI** (not printed value).
- 2020-06-23: replacement effect doesn't apply to Conclave Mentor itself (PB-CD already handles this correctly via `is_self: false`).

**Current state**: replacement half shipped in PB-CD. Death trigger TODO at lines 41-43 citing OOS-LKI-Power.

**Fix**: Replace the TODO with a `WhenDies` triggered ability:

```rust
// When Conclave Mentor dies, you gain life equal to its power.
// CR 603.10a / Ruling 2020-06-23: power read from LKI snapshot
// (boosted-on-battlefield value, not printed 2/2).
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenDies,
    effect: Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::SourcePowerAtLastKnownInformation,
    },
    intervening_if: None,
    targets: vec![],
    modes: None,
    trigger_zone: None,
},
```

Also update the file's header comment block (lines 13-18) to remove the "TODO ... blocked on OOS-LKI-Power" prose, replacing with: "Death trigger uses `EffectAmount::SourcePowerAtLastKnownInformation` (PB-LKI-Power) to honor the 2020-06-23 ruling that gain-life amount equals power as it last existed on the battlefield."

#### 2. Juri, Master of the Revue — re-author death trigger (file: `crates/engine/src/cards/defs/juri_master_of_the_revue.rs`)

**Oracle text** (MCP, verified 2026-05-13):
> Whenever you sacrifice a permanent, put a +1/+1 counter on Juri.
> When Juri dies, it deals damage equal to its power to any target.

**Rulings (load-bearing)**:
- 2020-11-10: "For Juri's second ability, use its power from when it was last on the battlefield to determine how much damage is dealt. **If that power was 0 or less, Juri deals no damage.**" — explicitly mandates LKI; the negative-clamping is handled at the `Effect::DealDamage` boundary (existing behavior; no special variant logic needed).

**Current state**: First ability (sacrifice → +1/+1 counter) is shipped. Death trigger TODO at lines 37-38 citing "EffectAmount::SourcePower".

**Fix**: Replace the TODO with a `WhenDies` triggered ability:

```rust
// When Juri dies, it deals damage equal to its power to any target.
// CR 603.10a / Ruling 2020-11-10: power read from LKI snapshot
// (boosted by accumulated +1/+1 counters before death).
// CR 120.4: damage of 0 or less is reduced to 0 — Juri ruling explicitly notes this.
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenDies,
    effect: Effect::DealDamage {
        target: EffectTarget::DeclaredTarget { index: 0 },
        amount: EffectAmount::SourcePowerAtLastKnownInformation,
    },
    intervening_if: None,
    targets: vec![TargetRequirement::TargetAny],
    modes: None,
    trigger_zone: None,
},
```

Also update the header comment block to remove the TODO prose.

### Out-of-scope (STOP-AND-FLAG, do not widen)

Per PB-LKI-CC retriage and the brief:

1. **Master Biomancer ETB-replacement reading source's live power** (`master_biomancer.rs:7,26`) — this is a `ReplacementModification::EntersWith(EffectAmount::SourcePower)` shape. The replacement fires at ETB time, NOT at WhenDies/WhenLeavesBattlefield. The source is on the battlefield, alive — `EffectAmount::PowerOf(EffectTarget::Source)` should resolve correctly via the existing live arm. The DSL gap is in `ReplacementModification::EntersWith` accepting an `EffectAmount` — a different primitive (replacement-side dynamic counter count). **File as OOS-LKI-Power-2 seed**.
2. **Triggered abilities reading OTHER objects' P/T (not source's own LKI)** — different snapshot target, not a SelfDies LKI primitive. File as separate seed if discovered.
3. **Cost-payment LKI for activated abilities** (Workhorse-style, `{T}, sacrifice this: deal X damage where X = power`) — already filed as OOS-LKI-3 by PB-LKI-CC. Do NOT widen.
4. **AnyCreatureDies triggers reading the dying creature's LKI power** (e.g. hypothetical "Whenever a creature dies, deal damage equal to its power") — different dispatch site (line 4318 in abilities.rs threads a different snapshot). Already filed as OOS-LKI-4 by PB-LKI-CC. Do NOT widen.
5. **Toughness-as-power swaps (Layer 7b grant — Belligerent Yearling, Doran)** — out of scope. The LKI snapshot at sba.rs:540 captures the post-Layer-7b power, so cards like "When this dies, gain life equal to its power" on a Doran-affected creature would correctly use the Doran-swapped value at LKI. No separate primitive needed.
6. **`SourceToughnessAtLastKnownInformation`** — discriminant 19 reserved but NOT shipped (no in-scope card). File as OOS-LKI-Power-1 if a future card surfaces it.

### OOS seeds to append to `pb-retriage-CC.md` (runner appends at fix-phase end)

```markdown
### OOS-LKI-Power-1: SourceToughnessAtLastKnownInformation

**Cards**: hypothetical "When ~ dies, [effect] X = its toughness". None confirmed
in current card-def universe (sweep 2026-05-13 found no SelfDies/SelfLeavesBattlefield
trigger reading source toughness).
**Oracle pattern**: SelfDies/SelfLeavesBattlefield trigger reading source's own
toughness at LKI.
**Gap**: PB-LKI-Power (HASH 17) ships `EffectAmount::SourcePowerAtLastKnownInformation`
(disc 18) and reserves disc 19 for the toughness sibling. The
`pre_death_power: Option<i32>` snapshot infrastructure at sba.rs:540 +
PendingTrigger/StackObject/EffectContext threading + GameEvent payload extension
all generalize trivially: add `pre_death_toughness: Option<i32>` alongside,
add disc 19 variant, add resolve_amount arm reading `ctx.lki_toughness`.
**Yield**: 0 confirmed in current pool. File as preventive seed.
**Status**: Filed by PB-LKI-Power planner 2026-05-13.

### OOS-LKI-Power-2: ReplacementModification::EntersWith(EffectAmount) — Master Biomancer

**Cards**: Master Biomancer ("Each other creature you control enters with a number
of +1/+1 counters on it equal to Biomancer's power"), and any future card with
similar ETB-replacement wording reading the source's live power.
**Oracle pattern**: `EnterFromX` replacement that places counters where the count
is dynamic (= source's power, source's toughness, count of permanents, etc.).
**Gap**: today, `ReplacementModification::EntersWith` accepts a static u32
counter count, not an `EffectAmount`. The source is alive on the battlefield
when the replacement fires (not LKI), so `EffectAmount::PowerOf(EffectTarget::Source)`
would resolve correctly via the live arm — but the replacement DSL doesn't
plumb EffectAmount through. This is the replacement-side mirror of the PB-TS
TokenSpec.count u32→EffectAmount migration.
**Yield**: 1 confirmed (Master Biomancer); broader sweep would surface more
ETB-replacement cards using "equal to X" wording.
**Status**: Filed by PB-LKI-Power planner 2026-05-13.

### OOS-LKI-Power-3: GameEvent LBA hash arms don't hash pre_lba_counters or pre_lba_power

**Cards**: N/A (engine consistency issue, not card-blocking).
**Gap**: `GameEvent::AuraFellOff`, `GameEvent::ObjectExiled`,
`GameEvent::PermanentDestroyed`, and `GameEvent::ObjectReturnedToHand` hash
arms in `state/hash.rs:3386-3553` use `..` and do NOT hash their
`pre_lba_counters` (added by PB-LKI-CC) or `pre_lba_power` (added by
PB-LKI-Power) fields. Only `GameEvent::CreatureDied` hashes its LKI fields.
This is a pre-existing inconsistency that PB-LKI-CC and PB-LKI-Power both
intentionally preserve to minimize blast radius. Replay determinism is
preserved because PendingTrigger and StackObject DO hash these fields, and
GameEvents are derived state recomputable from commands.
**Yield**: 0 (engine-consistency cleanup, no card unblocking).
**Status**: Filed by PB-LKI-Power planner 2026-05-13. Resolution would bump
HASH_SCHEMA_VERSION; defer until a determinism issue is observed in production.
```

---

## Step 5 — Hash strategy

**Current**: `HASH_SCHEMA_VERSION = 16` (PB-CD).
**Bumped**: `HASH_SCHEMA_VERSION = 17`.

**Rationale**: Three structs gain new serialized fields:
1. `EffectAmount` — new variant `SourcePowerAtLastKnownInformation` (discriminant 18)
2. `PendingTrigger` — new field `lki_power: Option<i32>` (4 or 5 bytes wire change per instance: 1-byte tag + 4-byte i32 if Some)
3. `StackObject` — new field `lki_power: Option<i32>`
4. `GameEvent::CreatureDied` — new field `pre_death_power: Option<i32>` (hashed)
5. `GameEvent::AuraFellOff` / `ObjectExiled` / `PermanentDestroyed` / `ObjectReturnedToHand` — new field `pre_lba_power: Option<i32>` (NOT hashed per Site 17 decision; serde-default for backward compat)

The wire format changes accordingly. Pre-PB-LKI-Power replays will not deserialize correctly under the new format. `#[serde(default)]` on the new fields handles missing-field deserialization within HASH 17.

**Sentinel-assertion file sweep**: 10 sentinel sites across 9 files (Step 3 Site 19 table). Update assertion value + message; do NOT rename functions (mirror PB-LKI-CC's "no churn" approach for the sentinel sweep — PB-LKI-CC actually did rename functions and the review noted that the renames were optional; PB-LKI-Power skips them).

**Hash arms requiring change**:
- `HashInto for EffectAmount` (lines 4477+): NEW arm for discriminant 18 (Site 14).
- `HashInto for PendingTrigger`: NEW field hash via `self.lki_power.hash_into(hasher)` (Site 15) — generic Option<i32> impl handles tag-byte encoding automatically.
- `HashInto for StackObject`: same (Site 15).
- `HashInto for GameEvent::CreatureDied`: ADD `pre_death_power.hash_into(hasher)` after the existing `pre_death_counters` loop (Site 17, only this one event variant).
- `HashInto for GameEvent::AuraFellOff/ObjectExiled/PermanentDestroyed/ObjectReturnedToHand`: NO CHANGE — keep `..` pattern (Site 17 decision).
- `HASH_SCHEMA_VERSION` bumped to 17; history entry 17 appended (Site 18).

---

## Step 6 — Test plan (mandatory tests, brief acceptance criterion 5)

**File**: `crates/engine/tests/primitive_pb_lki_power.rs` (new file, mirrors `primitive_pb_lki_cc.rs`).

### Test (a) — Conclave Mentor death trigger gains life = LKI power (boosted, not printed)

**Test name**: `test_conclave_mentor_death_trigger_gains_life_from_lki_power`

**CR citation**: CR 603.10a (LKI for leaves-battlefield triggers); CR 113.7a (LKI general); CR 122.2 (counters cease on zone change); CR 400.7 (zone change creates new object). Conclave Mentor 2020-06-23 ruling.

**Setup**:
1. Build a 2-player game (`p1`, `p2`).
2. Place a Conclave Mentor on p1's battlefield (printed 2/2).
3. Manually add 2 `+1/+1` counters to the Mentor (`obj.counters.insert(CounterType::PlusOnePlusOne, 2)`).
4. Verify pre-death layer-resolved power == 4 via `calculate_characteristics` (sanity check).
5. Apply lethal damage (set `damage_marked >= toughness + counters` = 4) OR call `Effect::DestroyPermanent { target: Mentor }`.
6. Process priority/SBA cycles until the death trigger resolves.
7. Record p1's life total before death; record p1's life total after death-trigger resolves.

**Discriminating assertion**: p1's life total increased by exactly **4** (boosted LKI power, NOT 2 = printed power, NOT 0 = LKI not threaded).

```rust
assert_eq!(
    life_after - life_before, 4,
    "CR 603.10a / Conclave Mentor 2020-06-23 ruling: WhenDies must gain life = \
     LKI power (printed 2 + 2 counters = 4); got {} life. If 2, lki_power was \
     not captured at sba.rs:540 (read printed power instead). If 0, lki_power \
     was not threaded through PendingTrigger → StackObject → EffectContext.",
    life_after - life_before
);
```

### Test (b) — Juri Master death trigger deals damage = LKI power (boosted, not printed)

**Test name**: `test_juri_master_death_trigger_deals_damage_from_lki_power`

**CR citation**: CR 603.10a, CR 113.7a, CR 122.2. Juri 2020-11-10 ruling.

**Setup**:
1. Build a 2-player game; p1 controls Juri (printed 1/1).
2. Add 3 `+1/+1` counters manually (Juri is now 4/4 on battlefield).
3. Apply lethal damage (4) OR destroy Juri.
4. Process priority/SBA until the death trigger resolves; auto-target p2 (the opponent player) for the "any target" damage.
5. Record p2's life total before and after.

**Discriminating assertion**: p2's life total decreased by exactly **4** (boosted LKI power).

```rust
assert_eq!(
    life_before - life_after, 4,
    "CR 603.10a / Juri 2020-11-10 ruling: WhenDies must deal damage = LKI power \
     (printed 1 + 3 counters = 4); got {} damage. If 1, lki_power read printed \
     value. If 0, lki_power was None.",
    life_before - life_after
);
```

### Test (c) — **Discriminating LKI test**: read source's power AFTER it's in graveyard

**Test name**: `test_lki_power_resolves_to_pre_death_value_not_printed_value`

**CR citation**: CR 603.10a; CR 122.2 (counters cease on zone change); CR 400.7 (zone change → new object).

**Purpose**: this is the regression sentinel. Validates that the snapshot ACTUALLY reads pre-death (boosted) power, NOT the post-zone-change printed power.

**Setup**:
1. Build a 2-player game; p1 controls Juri (printed 1/1) with 5 `+1/+1` counters (battlefield power = 6).
2. Pre-death: assert `state.objects[juri_id].counters.get(P1P1) == Some(5)` and layer-resolved power == 6.
3. Trigger death (lethal damage = 6).
4. Process SBA + drain stack.
5. Post-death: find Juri's new graveyard ObjectId. Assert `state.objects[grave_id].counters.is_empty()` (CR 122.2 invariant) AND `state.objects[grave_id].characteristics.power == Some(1)` (printed face — `move_object_to_zone` rebuilt characteristics).
6. Assert p2 took **6** damage from the death trigger.

This test FAILS if the engine reads `state.objects[grave_id].characteristics.power` (would yield 1) or if it doesn't capture the snapshot at all (would yield 0). It only PASSES if the LKI snapshot was correctly captured at sba.rs:540 BEFORE move_object_to_zone.

```rust
// CR 122.2: graveyard object's counters are reset to empty.
let grave_obj = &state.objects[&grave_id];
assert!(
    grave_obj.counters.is_empty(),
    "CR 122.2: graveyard Juri must have NO counters (counters cease on zone change). \
     Found {:?}. If non-empty, move_object_to_zone is not resetting counters and the \
     LKI threading may be redundant.",
    grave_obj.counters
);
// CR 400.7: graveyard object's characteristics are rebuilt from printed face.
assert_eq!(
    grave_obj.characteristics.power, Some(1),
    "CR 400.7: graveyard Juri's printed power must be 1; got {:?}. If anything else, \
     move_object_to_zone is preserving battlefield characteristics — LKI snapshot \
     would still work but the engine has a separate bug.",
    grave_obj.characteristics.power
);
// The damage must reflect LKI (6), NOT printed (1) AND NOT zero.
assert_eq!(
    p2_damage_taken, 6,
    "CR 603.10a: Juri's WhenDies trigger must deal damage = LKI power (6 = 1 printed + 5 \
     counters), NOT printed (1) and NOT zero. Got {}.",
    p2_damage_taken
);
```

### Test (d) — Hash determinism + sentinel + variant-discrimination

**Test name**: `test_pb_lki_power_hash_schema_version_and_determinism`

**CR citation**: hash infrastructure (no specific CR).

**Setup (3 sub-assertions, mirrors PB-LKI-CC's E2-fixed test pattern)**:

1. **Sentinel**: `assert_eq!(HASH_SCHEMA_VERSION, 17u8, "PB-LKI-Power bumped HASH_SCHEMA_VERSION 16→17 (EffectAmount::SourcePowerAtLastKnownInformation, CR 603.10a / 113.7a)");`

2. **Variant-discriminant determinism**: hash two `EffectAmount::SourcePowerAtLastKnownInformation` instances; assert hashes equal. Hash one `EffectAmount::PowerOf(EffectTarget::Source)`; assert distinct hash. Hash `EffectAmount::CounterCountAtLastKnownInformation { counter: PlusOnePlusOne }`; assert distinct from the new variant.

```rust
use blake3::Hasher;
use mtg_engine::state::hash::HashInto;
use mtg_engine::EffectAmount;
let h = |a: &EffectAmount| {
    let mut hh = Hasher::new();
    a.hash_into(&mut hh);
    *hh.finalize().as_bytes()
};
let lki_a = EffectAmount::SourcePowerAtLastKnownInformation;
let lki_b = EffectAmount::SourcePowerAtLastKnownInformation;
let live = EffectAmount::PowerOf(EffectTarget::Source);
let cc = EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType::PlusOnePlusOne };
assert_eq!(h(&lki_a), h(&lki_b), "deterministic hash for new variant");
assert_ne!(h(&lki_a), h(&live), "discriminated from live PowerOf");
assert_ne!(h(&lki_a), h(&cc), "discriminated from CounterCountAtLastKnownInformation");
```

3. **State hash determinism on PendingTrigger.lki_power Option**:
   Construct two PendingTriggers with identical fields except one has `lki_power: None`, the other `lki_power: Some(0)`. Hash both; assert distinct hashes (proves the Option tag byte is encoded correctly). Then construct a third with `lki_power: Some(1)`; assert distinct from both.

   This is the canary for "Some(0) != None" — exactly the kind of bug the generic `Option<T>` HashInto guards against, and worth a sentinel test.

### Test (e) — Sentinel-only assertion (pulled out of (d) for clarity)

The PB-LKI-CC test file uses a separate sentinel test. PB-LKI-Power can fold this into (d) per the brief's "sentinel" requirement, OR keep it separate as `test_pb_lki_power_hash_schema_version_is_17`. **Planner choice**: fold into test (d) sub-assertion 1. Save a function definition.

### Test (f) — Optional secondary GameEvent variant test

**Test name**: `test_juri_destroyed_by_replacement_redirect_to_exile_still_snapshots_power`

**Purpose**: validates that when a creature death is redirected by a replacement effect (e.g. Rest in Peace exiles instead of graveyard), the `pre_death_power` snapshot is still propagated through the `ObjectExiled` event. This mirrors the PB-LKI-CC fix-phase regression tests for Toothy bounce/exile/destroy.

**Setup**:
1. Build a 2-player game; p1 controls Juri with 3 `+1/+1` counters (4/4 on battlefield).
2. Apply Rest in Peace replacement (or manually inject a `ZoneChangeAction::Redirect { to: Exile }`).
3. Trigger death (lethal damage).
4. Verify `GameEvent::ObjectExiled` was emitted (NOT `CreatureDied`).
5. Process the WhenDies trigger and assert p2 took 4 damage.

**Discriminating**: tests the SBA redirect path at sba.rs:582-594, which currently captures `pre_death_counters` but the runner must extend to also capture `pre_death_power` and propagate into `ObjectExiled.pre_lba_power`.

**Skip note**: if the test infrastructure for Rest in Peace or manual redirect injection is brittle, the runner may downgrade to a comment + TODO, OR substitute by destroying via `Effect::ExileObject` directly. **Stop-and-flag** if the test cannot be made deterministic — file as a regression-test gap.

---

## Step 7 — Implementation order (for runner, mechanical)

1. **Engine change 1** — DSL variant: add `EffectAmount::SourcePowerAtLastKnownInformation` to `card_definition.rs:2398` (after `CounterCountAtLastKnownInformation`).
2. **Engine change 2** — `EffectContext.lki_power` field (effects/mod.rs struct line 47-143 + 2 constructors at line 144-205 + 2 inner_ctx clones at line 2486-2531 + check_condition stub at line 7340-7345).
3. **Engine change 3** — `PendingTrigger.lki_power` field (state/stubs.rs struct line 241-411 + blank() line 413-457).
4. **Engine change 4** — `StackObject.lki_power` field (state/stack.rs struct line 158-475 + trigger_default() line 477-541).
5. **Engine change 5** — Capture LKI power at sba.rs:540 snapshot site (extend the let-bind to include `pre_death_power`).
6. **Engine change 6** — `GameEvent::CreatureDied.pre_death_power: Option<i32>` field (events.rs:207-222).
7. **Engine change 7** — `GameEvent::{AuraFellOff,PermanentDestroyed,ObjectExiled,ObjectReturnedToHand}.pre_lba_power: Option<i32>` field (events.rs).
8. **Engine change 8** — Update ALL emit sites for these 5 GameEvent variants (~35 sites across abilities.rs, casting.rs, engine.rs, resolution.rs, turn_actions.rs, sba.rs). Battlefield sources capture `calculate_characteristics(state, id).and_then(|c| c.power).or(obj.characteristics.power)` BEFORE move_object_to_zone; non-battlefield sources use `None`.
9. **Engine change 9** — Update 6 trigger arms in `check_triggers` to propagate `pre_death_power`/`pre_lba_power` → `PendingTrigger.lki_power` (abilities.rs).
10. **Engine change 10** — Thread LKI through flush_pending_triggers (abilities.rs:7467 — after stack_obj.lki_counters assignment, add stack_obj.lki_power = trigger.lki_power).
11. **Engine change 11** — Build `ctx.lki_power = stack_obj.lki_power;` at resolution (resolution.rs:2057-2061 carddef path + 2132-2136 chars path).
12. **Engine change 12** — `EffectAmount::SourcePowerAtLastKnownInformation` arm in `resolve_amount` (effects/mod.rs after line 6365).
13. **Engine change 13** — `EffectAmount::SourcePowerAtLastKnownInformation` arm in `resolve_cda_amount` (rules/layers.rs after line 1620).
14. **Engine change 14** — HashInto for the new EffectAmount variant (hash.rs:4534 — discriminant 18, fieldless: just `18u8.hash_into(hasher)`).
15. **Engine change 15** — HashInto for new fields on PendingTrigger and StackObject (hash.rs — `self.lki_power.hash_into(hasher)` after the existing `lki_counters` blocks, ~line 2154 and ~line 3031).
16. **Engine change 16** — HashInto for `GameEvent::CreatureDied.pre_death_power` (hash.rs:3362-3377 — add `pre_death_power.hash_into(hasher);` after the `pre_death_counters` for-loop). Leave the 4 LBA hash arms unchanged (Site 17 decision).
17. **Engine change 17** — HASH_SCHEMA_VERSION 16→17 bump + history entry 17 (hash.rs:50-95).
18. **Engine change 18** — Sentinel-assertion sweep across 9 test files (Step 3 Site 19 table).
19. **Card def 1** — `conclave_mentor.rs`: replace TODO at lines 41-43 with WhenDies triggered ability using new variant. Update header comment (lines 13-18).
20. **Card def 2** — `juri_master_of_the_revue.rs`: replace TODO at lines 37-38 with WhenDies triggered ability + TargetAny. Update header comment (line 3).
21. **OOS append** — `pb-retriage-CC.md`: append OOS-LKI-Power-1 (toughness variant deferred), OOS-LKI-Power-2 (Master Biomancer ETB-replacement EffectAmount), OOS-LKI-Power-3 (LBA hash arm symmetric extension) per Step 4.
22. **Tests (a-d, optionally f)** — 4-5 tests in new file `crates/engine/tests/primitive_pb_lki_power.rs`.
23. **Gates** — `cargo build --workspace`; `cargo test --workspace` (count must rise from 2745 to ≥ 2748); `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`. All gates green before signaling ready.
24. **Memo** — Write `memory/primitives/pb-review-LKI-Power.md` after impl + review.
25. **CLAUDE.md update** — Refresh Active Plan + test count + HASH version per the milestone-completion checklist (paths in CLAUDE.md "Current State" section).

---

## Step 8 — Verification checklist

- [x] CR rule lookups complete: 603.10a, 113.7a, 400.7, 122.2 (verified via MCP)
- [x] Engine architecture walk: 21 dispatch sites with file:line references, current behavior, required change
- [x] Path decision documented (Path A chosen; Paths B and C rejected with CR-grounded reasons)
- [x] Card scope verification: 2 confirmed (Conclave Mentor + Juri Master), TODO sweep returned 2 forced adds, no additional pattern sweep candidates
- [x] Out-of-scope items explicit: 6 items + 3 OOS-LKI-Power-N seeds drafted
- [x] Hash strategy: bump 16→17, sentinel sweep across 9 test files (10 sentinel sites), 1 history entry, hash impl edits enumerated
- [x] Test plan: 4-5 mandatory tests (a-d, optional f), CR citations, file path
- [x] Plan file written: `memory/primitives/pb-plan-LKI-Power.md`
- [ ] `memory/primitive-wip.md` updated: planner checklist boxes ticked, `phase: plan-complete` (runner does this)

## Acceptance criteria mapping (from ESM task scutemob-19)

| AC | Criterion | Plan coverage |
|---|---|---|
| 3762 | plan written | This file. |
| 3763 | EffectAmount variant + hash changelog | Site 1 (variant), Site 14 (hash arm disc 18), Site 18 (HASH 16→17 + history entry). |
| 3764 | PendingTrigger/StackObject/EffectContext threading + sba.rs:540 snapshot site | Sites 2, 3, 4, 5, 10, 11. |
| 3765 | GameEvent pre_death_power/pre_death_toughness on 4-5 variants + abilities.rs sweep | Sites 6, 7, 8, 9. **Toughness variant deferred per Step 0/4** (no in-scope card needs it) — covered by OOS-LKI-Power-1 seed. |
| 3766 | HASH 16→17 + Option tag-byte encoding + sentinel sweep | Sites 14, 15, 16, 17, 18, 19. Generic `Option<T>` HashInto handles tag-byte automatically (verified at hash.rs:191-201). |
| 3767 | Conclave Mentor + Juri TODOs cleared | Step 4 Card def 1 + Card def 2. |
| 3768 | tests including LKI-after-zone-change | Test (c) is the discriminating LKI-after-zone-change test; tests (a)/(b) are per-card; test (d) is hash-determinism + sentinel; optional test (f) is GameEvent-redirect-path. |
| 3769 | cargo gates | Step 7 step 23. |
| 3770 | review + fixes | Reviewer + runner phases per project protocol. |
| 3771 | CLAUDE.md + OOS seed close + authoring report regen | Step 7 step 21 (OOS seeds), step 25 (CLAUDE.md). Authoring report regen (`python3 tools/authoring-report.py`) is a post-impl step in the runner closeout. |

---

## STOP-AND-FLAG triggers (per `feedback_oversight_primitive_category_not_cards.md`)

The runner MUST halt and report (rather than expand scope) if any of the following:

1. **Toughness variant scope creep**: a card not currently identified surfaces during impl that needs `SourceToughnessAtLastKnownInformation`. Default action: file as OOS-LKI-Power-1 update (add the card to the seed), do NOT add the variant in PB-LKI-Power scope.

2. **AnyCreatureDies scope creep**: any card surfaces during impl that needs the dying creature's LKI power on an AnyCreatureDies trigger (e.g. Blood Artist family with "deal damage equal to its power"). Default action: file as OOS-LKI-Power-2 update or new OOS-LKI-Power-N seed; do NOT thread `triggering_creature_lki_power` through PendingTrigger in this PB. The PB-LKI-CC OOS-LKI-4 precedent governs.

3. **Cost-payment LKI scope creep**: any card surfaces requiring `{T}, sacrifice this: deal X damage where X = power at time of sacrifice` (Workhorse-style). Default action: file as update to existing OOS-LKI-3; do NOT extend `EffectContext.sacrificed_creature_powers` in this PB.

4. **Hash-arm symmetry creep**: do NOT add `pre_lba_power` to the AuraFellOff/PermanentDestroyed/ObjectExiled/ObjectReturnedToHand hash arms in this PB. The decision in Site 17 explicitly preserves the PB-LKI-CC `..` precedent. If runner wants symmetric hashing, file as OOS-LKI-Power-3 (already drafted) — separate sentinel-bumping change.

5. **Yield drop below 2**: if either Conclave Mentor or Juri test fails to produce the expected behavior after impl + 1 fix attempt, STOP and report. Per `feedback_pb_yield_calibration.md` the threshold is ≥2; falling below = hidden compound blocker → halt.

6. **Calculate_characteristics layer recursion**: if `calculate_characteristics(state, id)` at sba.rs:540 produces a recursion warning or panic (because layer pass references the source's own characteristics in some weird way), STOP and report. The fallback `or(obj.characteristics.power)` should be sufficient defensive default, but unexpected layer interactions are STOP-AND-FLAG.

7. **GameEvent emit-site count surprise**: PB-LKI-CC's E1 fix-phase touched ~35 emit sites. If the runner finds substantially more (say >50), STOP and report — the scope may have grown since PB-LKI-CC and a different approach (e.g. helper function) may be warranted.

8. **Per `feedback_verify_full_chain.md`**: walk every dispatch site (DSL → snapshot → emit → trigger → flush → resolution → resolve_amount → hash → tests). Don't stop at variant existence. The plan enumerates 21 sites; the runner verifies each.

---

## Risks & edge cases

1. **Negative power LKI (Juri ruling)**: Juri 2020-11-10 ruling: "If that power was 0 or less, Juri deals no damage." `Effect::DealDamage` clamps negative damage to 0 at the boundary (existing behavior). The `resolve_amount` arm returns the raw i32 (potentially negative) — clamping is downstream. Verify by running test (b) with Juri at -2/-2 (e.g. apply -1/-1 SBAs before death). **Skip note**: this edge case is informational; not required for the test plan but noted for completeness.

2. **Layer-7c CDA power at LKI**: Vishgraz the Doomhive (CDA-modified power) would correctly snapshot the boosted power via `calculate_characteristics` at sba.rs:540. The layer pass runs at snapshot time, capturing the post-CDA value. No special handling needed.

3. **OrdMap clone removed; Option<i32> is Copy**: unlike `lki_counters` which clones an OrdMap, `lki_power` is `Option<i32>` (Copy). Threading is just `=`, no clone. Marginally faster than the LKI-CC equivalent.

4. **Replay-viewer / TUI exhaustive matches**: per `MEMORY.md` 50%-miss-rate gotcha. New EffectAmount variant should NOT add new arms to StackObjectKind/KeywordAbility, but `cargo build --workspace` is mandatory. New fields on GameEvent variants COULD break struct-literal pattern matches that lack `..`; the runner must check.

5. **SBA timing — LKI snapshot vs SBA-induced power changes**: -1/-1 SBA cancellation (CR 704.5q) runs BEFORE the death event. By the time `pre_death_power` is captured at sba.rs:540, the counter map already reflects SBA cancellations and `calculate_characteristics` reflects the post-SBA layer-resolved power. This is correct LKI semantics (CR 603.10a — "the appearance of objects immediately prior to the event").

6. **Backward-compat replays**: pre-PB-LKI-Power GameState snapshots with PendingTrigger/StackObject/GameEvent variants lacking the `lki_power`/`pre_death_power`/`pre_lba_power` fields will deserialize as None via `#[serde(default)]`. Behavior degrades gracefully (LKI returns 0). Hash version bump signals the schema change.

7. **No new Card-DSL helpers**: unlike PB-TS which forced the runner to verify that new types like `Squid` SubType compile, both Conclave Mentor and Juri use existing types/abilities (Centaur Cleric, Human Shaman). No helpers.rs prelude additions.

8. **PB-CD interaction**: Conclave Mentor's replacement half (PB-CD) and PB-LKI-Power's death trigger are independent abilities on the same card. Tests must not break PB-CD behavior (replacement half still doubles +1/+1 counters on creature you control). Sentinel: re-run PB-CD tests in the test sweep (`cargo test --workspace` covers this).

9. **Token death note (Conclave Mentor edge)**: Conclave Mentor itself is not a token. The 2020-06-23 ruling applies: when Mentor dies it goes to graveyard, life gain = its LKI power. If a token version of Conclave Mentor were to die (impossible per current cards, but hypothetical), the token ceases to exist (CR 704.5d) and would still emit `CreatureDied` per current engine behavior — LKI snapshot would still work.

10. **Discriminant 19 reservation**: explicitly NOT shipped. The OOS-LKI-Power-1 seed reserves the slot. If a future PB needs `SourceToughnessAtLastKnownInformation`, it claims discriminant 19 and bumps HASH 17→18.

---

**End of plan.** Runner: proceed with Step 7 implementation order. Verify hash count + test count + cargo gates as listed.
