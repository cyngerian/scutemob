# Primitive Batch Plan: PB-LKI-CC — `EffectAmount::CounterCountAtLastKnownInformation` (LKI snapshot for WhenDies / WhenLeavesBattlefield)

**Generated**: 2026-04-29
**Branch**: `feat/pb-lki-cc-effectamountcountercount-lki-snapshot-for-whendies` (already checked out)
**Worktree**: `/home/skydude/projects/scutemob/.worktrees/scutemob-17/`
**Primitive**: New `EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType }` variant (discriminant 17), resolved from a counter snapshot threaded through `PendingTrigger` → `StackObject` → `EffectContext.lki_counters: OrdMap<CounterType, u32>`. Snapshot is captured at trigger-fire time (sba.rs already does this — see `pre_death_counters` at line 540) and propagated through to the resolution-time `resolve_amount` call.
**CR Rules**: 603.10a (LKI for leaves-battlefield triggers), 113.7a (LKI general), 400.7 (zone change creates new object), 122.2 (counters cease on zone change)
**Cards affected**: ≥2 confirmed (Chasm Skulker re-author + Toothy Imaginary Friend fix); broader sweep target ≥3
**Dependencies**: PB-TS (HASH 14, EffectAmount discriminant chain 0–16); existing `pre_death_counters` threading at sba.rs:540, mana.rs:171, replacement.rs:850, builder.rs:634/672/819, stack.rs:753, GameEvent::CreatureDied (events.rs:206) — all DONE.
**Deferred items from prior PBs**: Chasm Skulker death-trigger (PB-TS reverted to TODO citation OOS-TS-4); Toothy Imaginary Friend (shipped pre-PB-TS, produces 0 draws — TODO comment at line 43 INCORRECTLY claims counters survive `move_object_to_zone`).

---

## Step 0 — Verification of the OOS-TS-4 seed claims (load-bearing audit)

The PB-TS planner wrote in `pb-plan-TS.md` Risk #3:
> "relies on `move_object_to_zone` preserving the GameObject's `counters` map. The Toothy precedent at `toothy_imaginary_friend.rs:43-58` proves `EffectAmount::CounterCount { target: Source, counter: PlusOnePlusOne }` resolves correctly from a graveyard'd source."

**This claim is wrong.** Verified by reading `crates/engine/src/state/mod.rs:420`:

```rust
let mut new_object = GameObject {
    id: new_id,
    ...
    counters: OrdMap::new(),  // ← line 420: counters reset to empty
    ...
};
```

Per CR 122.2:
> "Counters on an object are not retained if that object moves from one zone to another. The counters are not 'removed'; they simply cease to exist. See rule 400.7."

And CR 400.7:
> "An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence."

**Conclusion**: `EffectAmount::CounterCount { target: EffectTarget::Source, ... }` resolves to **0** for any source that has died/left the battlefield. Toothy is shipping with broken behavior and the comment at `toothy_imaginary_friend.rs:43` is misleading documentation. The OOS-TS-4 seed in `pb-retriage-CC.md:441-472` is correct: this primitive is required.

The engine ALREADY captures the necessary LKI counter snapshot — `pre_death_counters` is computed at `sba.rs:540`, propagated through `GameEvent::CreatureDied { pre_death_counters: OrdMap<CounterType, u32> }` (events.rs:217-221), and consumed at `abilities.rs:3987-3990` for Modular's hard-coded P1P1 lookup. The gap is that this snapshot is NOT plumbed through to a generic `EffectAmount` resolver site at trigger resolution time.

---

## Step 1 — CR research (verbatim from MCP rules server)

### CR 603.10 / 603.10a — leaves-battlefield triggers look back in time

> 603.10. Normally, objects that exist immediately after an event are checked to see if the event matched any trigger conditions, and continuous effects that exist at that time are used to determine what the trigger conditions are and what the objects involved in the event look like. However, some triggered abilities are exceptions to this rule; the game "looks back in time" to determine if those abilities trigger, using the existence of those abilities and the appearance of objects immediately prior to the event.
>
> 603.10a Some zone-change triggers look back in time. These are leaves-the-battlefield abilities, abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object that all players can see is put into a hand or library.

### CR 113.7a — LKI for stack abilities resolving after source has moved

> 113.7a Once activated or triggered, an ability exists on the stack independently of its source. Destruction or removal of the source after that time won't affect the ability. […] any activated or triggered ability that references information about the source for use while announcing an activated ability or putting a triggered ability on the stack checks that information when the ability is put onto the stack. Otherwise, it will check that information when it resolves. **In both instances, if the source is no longer in the zone it's expected to be in at that time, its last known information is used.**

### CR 400.7 — zone change creates a new object

> 400.7. An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.

### CR 122.2 — counters cease on zone change

> 122.2. Counters on an object are not retained if that object moves from one zone to another. The counters are not "removed"; they simply cease to exist. See rule 400.7.

**Implication**: the snapshot must live in a separate field (the trigger payload), NOT on the new graveyard object. The graveyard `GameObject.counters` field must remain empty per CR 122.2; preserving counters there would break a core invariant and produce cascading wrong behavior at counter-check sites elsewhere (e.g. Persist/Undying intervening-if, Modular, etc., though those already use `pre_death_counters` from the event payload, not the GameObject).

---

## Step 2 — Path choice (Path A vs Path B)

### Chosen: **Path A** — new `EffectAmount` variant + LKI snapshot threaded through `PendingTrigger` → `StackObject` → `EffectContext`.

### Why Path A

1. **Honors CR 122.2 invariant**: counters on the graveyard `GameObject` stay empty. No risk of cascading bugs at the ~20 counter-check sites scattered through the engine that read `obj.counters` directly (SBAs, layer system, persist/undying intervening-if, etc.).
2. **Reuses existing snapshot infrastructure**: `pre_death_counters` is already captured at `sba.rs:540` and threaded through `GameEvent::CreatureDied`. We need a new field on `PendingTrigger` (and an `OrdMap` field on `EffectContext` and `StackObject`), but the upstream capture already exists.
3. **Aligns with `PowerOfSacrificedCreature` precedent (PB-P, HASH 6)**: that primitive snapshots LKI power on `EffectContext.sacrificed_creature_powers: Vec<i32>` at cost-payment time, threading it through the StackObject and into `EffectContext`. PB-LKI-CC follows the identical pattern but for counters and at trigger-fire time.
4. **Type-system enforcement**: a new variant means card authors can't accidentally write `CounterCount { Source, ... }` on a death trigger and silently get 0 — they explicitly write `CounterCountAtLastKnownInformation { counter: ... }`. The compiler distinguishes the two semantics.
5. **Discriminating the two arms preserves the live-counter semantic**: the existing `CounterCount` arm continues to read from `state.objects[id].counters` and is still correct for "while on battlefield" effects (e.g. activated ability on a permanent that reads its own counter count). Tests, card defs, and call sites for the live arm all keep working unchanged.

### Why NOT Path B (preserve counters on the graveyard object)

- Breaks CR 122.2 / CR 400.7 explicitly.
- Cascades into the ~20 sites that read `obj.counters` directly. Each must learn whether to treat the graveyard object's counters as "live" or "stale LKI." In particular: persist (CR 702.79a) checks if the dying creature had a -1/-1 counter at LKI; if we make `obj.counters` live in graveyard, persist's `pre_death_counters` parameter becomes redundant but the codepath still uses it, so we now have two sources of truth.
- Subsequent counter-add operations on the graveyard object (e.g. an external effect like "put a +1/+1 counter on target creature card in your graveyard" — Mortal's Resolve, Scavenge) would interact unpredictably with stale counters.
- Smaller diff but much higher risk surface.

### Why NOT a third option (read directly from the most recent `CreatureDied` event in `state.events_history`)

- `GameEvent` history is not generally in scope for effect resolution; tying `resolve_amount` to history scanning is a layering violation.
- LKI for `WhenLeavesBattlefield` (non-creature permanents) does NOT emit `CreatureDied` — only creatures do. We need a unified mechanism. Threading via `PendingTrigger` works for both creature and non-creature LBA.

---

## Step 3 — Engine architecture walk (full dispatch chain)

### Site 1 — DSL: new `EffectAmount` variant at `crates/engine/src/cards/card_definition.rs`

After line 2375 (after `PowerOfSacrificedCreature`), add a new variant. Insert before the closing `}` of `enum EffectAmount`:

```rust
/// CR 603.10a / CR 113.7a: Counter count from last-known information (LKI).
/// Used by WhenDies / WhenLeavesBattlefield triggers whose effect needs to know
/// how many counters of a given type were on the source as it last existed on
/// the battlefield. The snapshot is captured at trigger-fire time (sba.rs
/// `pre_death_counters` block) and threaded through `PendingTrigger.lki_counters`,
/// `StackObject.lki_counters`, and `EffectContext.lki_counters`.
///
/// **Versus `CounterCount`**: the existing `CounterCount { target, counter }`
/// variant reads live counters from `state.objects[id].counters`. Use that for
/// abilities that fire while the source is still on the battlefield (e.g.
/// "{T}: draw cards equal to the number of +1/+1 counters on this creature").
/// Use `CounterCountAtLastKnownInformation` for any effect on a leaves-the-
/// battlefield trigger (CR 603.10a) where `state.objects[ctx.source].counters`
/// is empty by the time the trigger resolves (CR 122.2 — counters cease on
/// zone change). The implicit target is the trigger source (LKI).
///
/// **Returns 0** if `ctx.lki_counters` is `None` (e.g. variant authored on a
/// non-LKI trigger by mistake) or if the requested counter type is absent.
/// Defensive default — the card author should pair this variant only with
/// WhenDies / WhenLeavesBattlefield triggers.
///
/// Discriminant 17 (state/hash.rs).
CounterCountAtLastKnownInformation { counter: CounterType },
```

**Note on shape**: no `target` field — implicit target is `EffectTarget::Source` (the dead permanent). The snapshot is the source's counters at LKI; there is no notion of "counters on a different LKI object" because triggers fire on a single object's zone change. Keeping the variant simple avoids the runner needing to plumb arbitrary `EffectTarget` resolution into LKI-snapshot lookup.

### Site 2 — `EffectContext` field at `crates/engine/src/effects/mod.rs` (struct ~line 47-135)

Add a new field after `sacrificed_creature_powers` (line 134). Use `im::OrdMap<CounterType, u32>` to mirror `GameObject.counters` and `PendingTrigger.pre_death_counters` infrastructure (already used by `CreatureDied` event):

```rust
/// CR 603.10a / CR 113.7a: LKI counter snapshot for leaves-battlefield triggers.
/// Populated by `flush_pending_triggers` (abilities.rs) when a `WhenDies` /
/// `WhenLeavesBattlefield` trigger is put on the stack, capturing the source's
/// counters as they existed immediately before zone change. Threaded into
/// `EffectContext` at trigger resolution time (resolution.rs:2052 / 2120).
/// Read by `EffectAmount::CounterCountAtLastKnownInformation`.
/// `None` for non-LKI trigger contexts; lookups return 0 if `None` or absent.
pub lki_counters: Option<im::OrdMap<crate::state::types::CounterType, u32>>,
```

Update both `EffectContext::new()` (~line 138) and `EffectContext::new_with_kicker()` (~line 165) to initialize `lki_counters: None`.

### Site 3 — `PendingTrigger` field at `crates/engine/src/state/stubs.rs` (struct ~line 241-403)

Add a new field after `combat_damage_amount` (line 395). Use the same `OrdMap<CounterType, u32>` shape:

```rust
/// CR 603.10a / CR 113.7a: LKI counter snapshot for WhenDies / WhenLeavesBattlefield triggers.
/// Captured at trigger queueing time (abilities.rs CreatureDied arm at line 3950+)
/// from the `GameEvent::CreatureDied.pre_death_counters` payload. Threaded
/// through `flush_pending_triggers` into `StackObject.lki_counters`, then into
/// `EffectContext.lki_counters` at resolution. Read by
/// `EffectAmount::CounterCountAtLastKnownInformation`.
/// Empty for triggers that don't fire from a leaves-battlefield event.
#[serde(default)]
pub lki_counters: im::OrdMap<crate::state::types::CounterType, u32>,
```

Update `PendingTrigger::blank()` (line 415-449) to initialize `lki_counters: im::OrdMap::new()`.

**Note**: use `OrdMap` (not `Option<OrdMap>`) on `PendingTrigger` to match the `OrdMap` field type already used for `pre_death_counters` in events. `EffectContext` keeps `Option<OrdMap>` to disambiguate "no LKI capture happened" from "LKI capture happened but counter map is empty" — both are semantically valid in different contexts.

### Site 4 — `StackObject` field at `crates/engine/src/state/stack.rs` (struct ~line 158-469)

Add a new field after `sacrificed_creature_powers` (line 457). Mirror the PendingTrigger shape:

```rust
/// CR 603.10a / CR 113.7a: LKI counter snapshot for WhenDies / WhenLeavesBattlefield triggers.
/// Set from PendingTrigger::lki_counters when the trigger is flushed to the stack
/// (abilities.rs flush_pending_triggers ~line 7387). Read at resolution time
/// (resolution.rs ~line 2052) into EffectContext.lki_counters.
/// Empty for stack objects that are not LBA triggered abilities.
#[serde(default)]
pub lki_counters: im::OrdMap<crate::state::types::CounterType, u32>,
```

Update `StackObject::trigger_default()` (line 486-534) to initialize `lki_counters: im::OrdMap::new()`.

### Site 5 — Capture LKI at trigger queueing time at `crates/engine/src/rules/abilities.rs` (CreatureDied arm)

The `GameEvent::CreatureDied` handling block at lines 3950-4042 has access to `pre_death_counters` (destructured at line 3954). Currently used at lines 3986-3990 for Modular's hard-coded P1P1 lookup. Plumb this into the `SelfDies` and `SelfLeavesBattlefield` PendingTrigger constructors:

**Edit at line 4002-4011** (SelfDies push for normal/modular triggers — `PendingTrigger { ability_index: idx, triggering_event: Some(SelfDies), data, ..PendingTrigger::blank(...) }`):

```rust
triggers.push(PendingTrigger {
    ability_index: idx,
    triggering_event: Some(TriggerEvent::SelfDies),
    data,
    // CR 603.10a: capture the dying creature's pre-death counters into LKI
    // so EffectAmount::CounterCountAtLastKnownInformation can resolve.
    lki_counters: pre_death_counters.clone(),
    ..PendingTrigger::blank(*new_grave_id, *death_controller, kind)
});
```

**Edit at line 4032-4040** (SelfLeavesBattlefield push):

```rust
triggers.push(PendingTrigger {
    ability_index: idx,
    triggering_event: Some(TriggerEvent::SelfLeavesBattlefield),
    // CR 603.10a: same LKI snapshot for leaves-battlefield triggers.
    lki_counters: pre_death_counters.clone(),
    ..PendingTrigger::blank(*new_grave_id, controller, PendingTriggerKind::Normal)
});
```

**Note**: `pre_death_counters` in this scope is the borrowed `&OrdMap<CounterType, u32>` from event destructure; clone via `.clone()` (im::OrdMap is cheap to clone — structural sharing).

**Note on AnyCreatureDies arm at line 4280-4294**: this fires "whenever a creature dies" triggers on OTHER permanents (Blood Artist, Zulaport Cutthroat). The dying creature is the trigger's *triggering object*, not its source. Should `lki_counters` be set here? **Defer**: the Blood Artist family does not need LKI counter count of the dying creature in current scope. Filed as a separate seed if a card surfaces this need later. For PB-LKI-CC: leave AnyCreatureDies untouched; the snapshot only flows for `SelfDies` and `SelfLeavesBattlefield` triggers (the LBA category).

### Site 6 — Thread LKI through `flush_pending_triggers` at `crates/engine/src/rules/abilities.rs` (~line 7387)

The flush site builds the StackObject via `StackObject::trigger_default` then sets fields:

```rust
let mut stack_obj = StackObject::trigger_default(stack_id, trigger.controller, kind);
stack_obj.targets = trigger_targets.clone();
stack_obj.damaged_player = trigger.damaged_player;
stack_obj.combat_damage_amount = trigger.combat_damage_amount;
stack_obj.triggering_creature_id = trigger.entering_object_id;
```

Add (after `triggering_creature_id` line 7394):

```rust
// CR 603.10a / CR 113.7a: Propagate LKI counter snapshot from PendingTrigger
// to StackObject so resolution.rs can build EffectContext.lki_counters.
stack_obj.lki_counters = trigger.lki_counters.clone();
```

### Site 7 — Build `EffectContext.lki_counters` at resolution time at `crates/engine/src/rules/resolution.rs`

Two sites, both inside the `StackObjectKind::TriggeredAbility` arm:

**Path 1 — CardDef-registry effect path (~line 2035-2052)**:

After existing `ctx.triggering_creature_id = stack_obj.triggering_creature_id;` (line 2051), add:

```rust
// CR 603.10a / CR 113.7a: Propagate LKI counter snapshot for
// EffectAmount::CounterCountAtLastKnownInformation.
ctx.lki_counters = if stack_obj.lki_counters.is_empty() {
    None
} else {
    Some(stack_obj.lki_counters.clone())
};
```

**Path 2 — Characteristics-based effect path (~line 2111-2120)**:

After existing `ctx.triggering_creature_id = stack_obj.triggering_creature_id;` (line 2119), add the same `ctx.lki_counters = ...` block.

**Note on the `is_empty` → `None` conversion**: the StackObject field is a flat `OrdMap` (default empty), but the EffectContext field is `Option<OrdMap>` to distinguish "no LKI was captured" from "LKI was captured but the source had no counters of any kind." This matters because a source that died with zero counters MUST resolve to 0 (not some sentinel value), and a non-LBA trigger MUST also resolve to 0 (no LKI was captured — variant misuse). Both end up returning 0 from `resolve_amount`, but the `None` discriminant is useful for future debug logging / assertion paths.

### Site 8 — `EffectAmount::CounterCountAtLastKnownInformation` resolution at `crates/engine/src/effects/mod.rs` (`resolve_amount`)

Add a new arm to `resolve_amount` (effects/mod.rs:5913). Insert after the `EffectAmount::CounterCount` arm at line 6149-6166:

```rust
// CR 603.10a / CR 113.7a: Read counter count from LKI snapshot captured at
// trigger-fire time. Returns 0 if no LKI was captured (variant misused on
// a non-LBA trigger) or if the requested counter type is absent (source died
// with zero counters of that kind). The implicit target is the trigger source.
EffectAmount::CounterCountAtLastKnownInformation { counter } => ctx
    .lki_counters
    .as_ref()
    .and_then(|map| map.get(counter).copied())
    .unwrap_or(0) as i32,
```

### Site 9 — `resolve_cda_amount` at `crates/engine/src/rules/layers.rs` (~line 1460)

`resolve_cda_amount` is the parallel resolver used by CDA evaluation in the layer system. CDAs are evaluated continuously while the source is on the battlefield; LKI snapshots are NOT relevant to CDA computation (a CDA never reads LKI — it reads live state). The new variant should return 0 in `resolve_cda_amount` (defensive default) with a comment explaining the divergence.

Add to `resolve_cda_amount` after the `CounterCount` arm (find by grepping for `EffectAmount::CounterCount` in `rules/layers.rs`):

```rust
// CR 613: CDAs cannot reference LKI — they evaluate continuously while the source
// is on the battlefield, where counters are live. Returns 0 as defensive default.
// Card authors should not pair CounterCountAtLastKnownInformation with a CDA;
// use the live `CounterCount` variant instead.
EffectAmount::CounterCountAtLastKnownInformation { .. } => 0,
```

### Site 10 — `HashInto for EffectAmount` at `crates/engine/src/state/hash.rs` (line 4402-4479)

Add a new arm after the `PlayerCounterCount` arm at line 4472-4476 (currently the last arm before the `}` at line 4477). New discriminant **17**:

```rust
// PB-LKI-CC (discriminant 17) — LKI counter snapshot for WhenDies /
// WhenLeavesBattlefield triggers. CR 603.10a / CR 113.7a / CR 122.2.
EffectAmount::CounterCountAtLastKnownInformation { counter } => {
    17u8.hash_into(hasher);
    counter.hash_into(hasher);
}
```

### Site 11 — `HashInto for PendingTrigger` and `HashInto for StackObject` at `crates/engine/src/state/hash.rs`

Both structs are hashed via existing `HashInto` impls. The new `lki_counters: OrdMap<CounterType, u32>` field must be added to BOTH hash impls.

**For PendingTrigger**: find the `impl HashInto for PendingTrigger` block (search hash.rs for `PendingTrigger` followed by `hash_into`). Add after the `combat_damage_amount` hash line:

```rust
// CR 603.10a: LKI counter snapshot — must be hashed for replay determinism.
for (ct, count) in self.lki_counters.iter() {
    ct.hash_into(hasher);
    count.hash_into(hasher);
}
```

**For StackObject**: find the `impl HashInto for StackObject` block. Add after the `sacrificed_creature_powers` hash line:

```rust
// CR 603.10a: LKI counter snapshot.
for (ct, count) in self.lki_counters.iter() {
    ct.hash_into(hasher);
    count.hash_into(hasher);
}
```

**Note**: `OrdMap<CounterType, u32>` iteration is deterministic by sorted key (im::OrdMap invariant), so the for loop produces canonical bytes without sorting.

### Site 12 — `HASH_SCHEMA_VERSION` bump at `crates/engine/src/state/hash.rs:75`

Bump `HASH_SCHEMA_VERSION: u8 = 14` → `15`. Append history entry 15 after line 74:

```rust
/// - 15: PB-LKI-CC (2026-04-29) — `EffectAmount::CounterCountAtLastKnownInformation`
///   (disc 17) reads LKI counter snapshot for WhenDies / WhenLeavesBattlefield
///   triggers (CR 603.10a, CR 113.7a, CR 122.2). New `lki_counters: OrdMap<CounterType, u32>`
///   field on `PendingTrigger` (state/stubs.rs), `StackObject` (state/stack.rs),
///   and `Option<OrdMap<CounterType, u32>>` on `EffectContext` (effects/mod.rs).
///   Snapshot is captured at trigger-fire time from `GameEvent::CreatureDied.pre_death_counters`
///   (already populated by sba.rs:540). Wire format change: PendingTrigger and
///   StackObject gain a serialized OrdMap field; pre-PB-LKI-CC replays are not
///   forward-compatible. Unblocks Chasm Skulker (re-author) and Toothy, Imaginary
///   Friend (corrects existing wrong game state).
```

### Site 13 — Sentinel-assertion test files (sweep targets)

Verified via rust-analyzer workspace_symbols (`hash_schema_version` query, results dated 2026-04-29). **6 files; 6 sentinels** — all must be renamed `..._after_pb_ts` / `..._is_14` / etc. → `..._after_pb_lki_cc` / `..._is_15` / etc., with assertion + message updated:

| File | Function | Line | Action |
|---|---|---|---|
| `crates/engine/tests/primitive_pb_cc_a.rs` | `test_hash_schema_version_after_pb_ts` | 99 | Rename to `..._after_pb_lki_cc`. Update assertion `14u8` → `15u8`. Update message: `"PB-LKI-CC bumped HASH_SCHEMA_VERSION 14→15 (EffectAmount::CounterCountAtLastKnownInformation, CR 603.10a / 113.7a)"`. |
| `crates/engine/tests/primitive_pb_cc_c_followup.rs` | `test_hash_schema_version_after_pb_ts` | 393 | Same rename + bump + message. |
| `crates/engine/tests/primitive_pb_ts.rs` | `test_pb_ts_hash_schema_version_and_token_spec_hash_determinism` | 365 | Update the inner `assert_eq!(HASH_SCHEMA_VERSION, 14u8, ...)` to `15u8`. Leave function name as-is (it asserts more than the version sentinel); update the message string to add a "(bumped to 15 by PB-LKI-CC)" suffix. |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | `test_pbt_hash_schema_version_is_14` | 403 | Rename to `..._is_15`. Update assertion + message. |
| `crates/engine/tests/pbt_up_to_n_targets.rs` | `test_pbt_hash_schema_version_sentinel_is_14_regression` | 863 | Rename to `..._is_15_regression`. Update assertion + message. |
| `crates/engine/tests/effect_sacrifice_permanents_filter.rs` | `test_sft_hash_schema_version_is_14` | 133 | Rename to `..._is_15`. Update assertion + message. |

The runner MUST rerun `mcp__rust-analyzer__rust_analyzer_workspace_symbols(query: "hash_schema_version")` after the bump to verify no additional sentinel was added between plan and impl.

### Site 14 — replay-viewer view_model + TUI stack_view (per `MEMORY.md` 50%-miss-rate gotcha)

Adding a new `EffectAmount` variant does NOT add a new variant to `StackObjectKind` or `KeywordAbility`, so the gotcha files (`tools/replay-viewer/src/view_model.rs`, `tools/tui/src/play/panels/stack_view.rs`) should not need new arms. **However**, the runner MUST run `cargo build --workspace` (not just `cargo build`) after impl to surface any indirect breakage from the new `EffectAmount` arm if any tool exhaustively matches on it.

### Site 15 — helpers.rs prelude

`crates/engine/src/cards/helpers.rs` re-exports `EffectAmount` already (line ~8). The new variant is accessible via the existing `EffectAmount::CounterCountAtLastKnownInformation { ... }` qualified path; no new export needed.

---

## Step 4 — Card scope verification (MCP oracle lookup + sweep)

### Confirmed targets (≥2)

#### 1. Chasm Skulker — re-author (file: `crates/engine/src/cards/defs/chasm_skulker.rs`)

**Oracle text** (MCP, verified 2026-04-29):
> Whenever you draw a card, put a +1/+1 counter on this creature.
> When this creature dies, create X 1/1 blue Squid creature tokens with islandwalk, where X is the number of +1/+1 counters on this creature.

**Current state**: TODO comment at lines 30-36 citing OOS-TS-4. The `WheneverYouDrawACard` trigger that adds +1/+1 counters is intact (lines 16-29).

**Fix**: Replace TODO comment with a `WhenDies` triggered ability:

```rust
// When this creature dies, create X 1/1 blue Squid creature tokens with islandwalk,
// where X is the number of +1/+1 counters on it.
// CR 603.10a: counter count read from LKI snapshot (counters cease on zone change per CR 122.2).
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenDies,
    effect: Effect::CreateToken {
        spec: TokenSpec {
            name: "Squid".to_string(),
            power: 1,
            toughness: 1,
            colors: [Color::Blue].into_iter().collect(),
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Squid".to_string())].into_iter().collect(),
            keywords: [KeywordAbility::Islandwalk].into_iter().collect(),
            count: EffectAmount::CounterCountAtLastKnownInformation {
                counter: CounterType::PlusOnePlusOne,
            },
            ..Default::default()
        },
    },
    intervening_if: None,
    targets: vec![],
    modes: None,
    trigger_zone: None,
},
```

**Verify**: The keyword name for "islandwalk" exists — check `KeywordAbility::Islandwalk` exists in `crate::state::types`. If not present, the runner must use `KeywordAbility::Landwalk(LandwalkType::Island)` or whichever exists. (Search by `mcp__rust-analyzer__rust_analyzer_workspace_symbols("Islandwalk")` during impl.) **Stop-and-flag** if neither exists — file as a separate primitive seed.

#### 2. Toothy, Imaginary Friend — fix existing TODO (file: `crates/engine/src/cards/defs/toothy_imaginary_friend.rs`)

**Oracle text** (MCP, verified 2026-04-29):
> Partner with Pir, Imaginative Rascal
> Whenever you draw a card, put a +1/+1 counter on Toothy.
> When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it.

**Current state**: WhenLeavesBattlefield trigger at lines 44-58 is authored, BUT it uses `EffectAmount::CounterCount { target: EffectTarget::Source, counter: CounterType::PlusOnePlusOne }` which resolves to 0 (the comment at line 43 is wrong — see Step 0).

**Fix**: Change line 48-51 from:

```rust
count: EffectAmount::CounterCount {
    target: EffectTarget::Source,
    counter: CounterType::PlusOnePlusOne,
},
```

to:

```rust
count: EffectAmount::CounterCountAtLastKnownInformation {
    counter: CounterType::PlusOnePlusOne,
},
```

Also update the misleading comment at line 43 from:

```rust
// Note: LKI — source is in graveyard/exile but counter count is preserved by move_object_to_zone.
```

to:

```rust
// CR 603.10a / 122.2: counter count must be read from the LKI snapshot (PendingTrigger.lki_counters)
// because move_object_to_zone resets the GameObject's counters to empty.
```

### Sweep for additional candidates (MCP-oracle-lookup roster verification)

Per the project guardrails (`feedback_oversight_primitive_category_not_cards.md`, `feedback_pb_yield_calibration.md`): MCP-verify any cards the runner finds before adding to scope. **MANDATORY** TODO sweep below.

**Search 1 — pre-existing TODO sweep for OOS-TS-4 reference**:
```bash
grep -rn "OOS-TS-4" crates/engine/src/cards/defs/
grep -rn "TODO.*counter.*LKI\|TODO.*pre.death.*counter\|TODO.*counter.*dies" crates/engine/src/cards/defs/
```

If any TODO comment names this primitive (LKI counter-count), add the card to scope as a forced add. As of plan-write time, only `chasm_skulker.rs` cites OOS-TS-4 explicitly.

**Search 2 — pattern sweep ("when ... dies" + counter count)**:
```bash
grep -rn "WhenDies\|WhenLeavesBattlefield\|WheneverCreatureDies" crates/engine/src/cards/defs/ | head -100
```
For each match, verify the effect via oracle text (MCP `lookup_card`) — does it say "where X is the number of [counter] counters on it" or similar? Most "when this dies" triggers are simple (sacrifice, deal damage, draw a card) and do not need LKI counter count.

**Search 3 — known patterns from MTG card pool**:

Cards with pattern "When [permanent] dies/leaves, [effect] X = number of [counter] counters on it":

| Card | Oracle pattern | Status (as of plan-write) |
|---|---|---|
| **Chasm Skulker** | "create X 1/1 Squid tokens, X = +1/+1 counters" | TODO'd, in-scope (forced add #1) |
| **Toothy, Imaginary Friend** | "draw cards for each +1/+1 counter" | broken-as-shipped, in-scope (forced add #2) |
| **Workhorse** | "{T}, sacrifice: add X mana, X = +1/+1 counters" — NOTE: this is an activated ability that *removes* the source via sacrifice as cost. This is a different LKI gap (cost-payment LKI). NOT in scope for PB-LKI-CC; file separate seed. | OOS — different primitive (cost-LKI) |
| **Devoted Druid** | "{T}: add {G}; pay -1/-1 counter" — not LKI death trigger | not applicable |
| **Walking Ballista** | sacrificial activated ability that deals damage based on +1/+1 counters — fires WHILE on battlefield, no LKI needed | not applicable |
| **Gyre Sage / Bow of Nylea / Hydra Broodmaster** | activated abilities on battlefield, no LKI | not applicable |
| **Sage of Hours** | exile self, take extra turn — no count needed | not applicable |
| **Camellia, the Seedmiser** | uses Forage primitive — already authored (PB-26 path) | already done |
| **Ridgescale Tusker / Cytoplast Root-Kin** | proliferate / counter-redistribution effects, fire while on battlefield | not applicable |
| **Sage Owl** (PB-26 sentinel) | "When ~ leaves the battlefield, draw a card" — fixed count (1), not counter-derived | not applicable |
| **Murasa Behemoth** | trample, no death trigger | not applicable |
| **Hydra Omnivore** | combat damage effect, no death trigger | not applicable |
| **Verdurous Gearhulk** | ETB +1/+1 counter distribution, no death trigger | not applicable |
| **Fertilid** | "{2}{G}, remove a +1/+1 counter from this creature: ..." — activated ability while alive | not applicable |
| **Champion of Lambholt** | static ability based on +1/+1 counters, fires while alive | not applicable |
| **Forgotten Ancient** | "At the beginning of each upkeep, if this creature has any +1/+1 counters on it, ..." — fires while alive | not applicable |

**Status**: Sweep yields exactly 2 confirmed (Chasm Skulker + Toothy). Workhorse is OOS (cost-LKI primitive, separate seed).

The seed says "≥2 in-scope cards re-authored / fixed (Chasm Skulker + Toothy + sweep)" — sweep target is met at 2 with no roster expansion required. The runner MUST rerun the TODO sweep grep at impl time to catch any cards added between plan and impl.

### Out-of-scope (STOP-AND-FLAG, do not widen)

Per task brief and PB-TS retriage:
1. **per-target dynamic EffectAmount in single ContinuousEffect (Phyresis Outbreak)** — separate seed, requires per-target context plumbing
2. **counter-doubling replacement at modifier time (CR 121.6 — Hardened Scales et al.)** — separate seed; PB-LKI-CC is read-side, not write-side
3. **Master Biomancer ETB-replacement counter placement** — separate seed
4. **OOS-TS-1/2/3** — separate seeds (token-count primitives, mostly shipped via PB-TS)
5. **Workhorse** (`{T}, sacrifice: add X mana, X = +1/+1 counters at time of sacrifice`) — different LKI category (cost-payment LKI for activated abilities, not trigger-fire LKI for LBA). File new seed `OOS-LKI-1` if discovered during impl.
6. **AnyCreatureDies triggers reading dying creature's counter count** (e.g. hypothetical "Whenever a creature with +1/+1 counters dies, ...") — defer to a separate primitive that threads the dying creature's LKI through the AnyCreatureDies dispatch arm. File `OOS-LKI-2` if needed.

### OOS seeds to append to `pb-retriage-CC.md`

The runner appends two new seeds at fix-phase completion (per task brief item 7):

```markdown
### OOS-LKI-1: Cost-payment LKI counter snapshot for activated abilities

**Cards**: Workhorse ({T}, sacrifice: add X mana, X = +1/+1 counters), and any
activated ability that sacrifices its source as cost and reads the source's
counter count for the effect.
**Oracle pattern**: "{cost incl. sacrifice this}: [effect] X = number of [counter] counters on this."
**Gap**: PB-P (`PowerOfSacrificedCreature`) snapshots LKI power at cost-payment
time but does not snapshot LKI counters. PB-LKI-CC (HASH 15) ships LKI
counter-snapshot for WhenDies / WhenLeavesBattlefield triggers but NOT for
activated-ability cost-payment paths. The two are different snapshot sites.
**Yield**: Workhorse + any future "sacrifice as cost, reference own counters"
card. Sweep `Cost::SacrificeSelf` activated abilities for `EffectAmount::CounterCount`
references to identify additional candidates.
**Status**: Filed by PB-LKI-CC planner 2026-04-29.

### OOS-LKI-2: AnyCreatureDies dying-creature LKI counter access

**Cards**: hypothetical "Whenever a creature with +1/+1 counters dies, ..." or
"Whenever a creature dies, draw cards equal to the +1/+1 counters that were
on it." None confirmed in current card-def universe.
**Oracle pattern**: AnyCreatureDies trigger reading the dying creature's
LKI counter count.
**Gap**: PB-LKI-CC threads LKI counters into PendingTrigger.lki_counters
ONLY for SelfDies / SelfLeavesBattlefield. AnyCreatureDies triggers fire
on OTHER permanents and would need a different snapshot field (e.g.
`triggering_creature_lki_counters: OrdMap<CounterType, u32>` on PendingTrigger).
**Yield**: 0 confirmed in current pool. File as preventive seed.
**Status**: Filed by PB-LKI-CC planner 2026-04-29.
```

---

## Step 5 — Hash strategy

**Current**: `HASH_SCHEMA_VERSION = 14` (PB-TS).
**Bumped**: `HASH_SCHEMA_VERSION = 15`.

**Rationale**: Three structs gain new serialized fields:
1. `EffectAmount` — new variant `CounterCountAtLastKnownInformation` (discriminant 17)
2. `PendingTrigger` — new field `lki_counters: OrdMap<CounterType, u32>`
3. `StackObject` — new field `lki_counters: OrdMap<CounterType, u32>`

The wire format changes accordingly (serde encodes the new field as part of struct serialization). Pre-PB-LKI-CC replays will not deserialize correctly under the new format. Per `memory/conventions.md` "Hash bump rule: bump on every change to a serialized type's field shape or variant shape. Default action: bump."

**Sentinel-assertion file sweep**: 6 files (Step 3 Site 13 table). The runner must update assertions and rename functions to reflect the new version.

**Hash arms requiring change**:
- `HashInto for EffectAmount` (lines 4402-4479): NEW arm for discriminant 17 (Site 10).
- `HashInto for PendingTrigger`: NEW field hash (Site 11).
- `HashInto for StackObject`: NEW field hash (Site 11).
- `HASH_SCHEMA_VERSION` bumped to 15; history entry 15 appended (Site 12).

---

## Step 6 — Test plan (5 mandatory tests, brief acceptance criterion 5)

**File**: `crates/engine/tests/primitive_pb_lki_cc.rs` (new file, mirrors `primitive_pb_ts.rs` and `primitive_pb_cc_a.rs`).

### Test (a) — Chasm Skulker death trigger creates X tokens (X = LKI +1/+1 counter count)

**Test name**: `test_chasm_skulker_death_trigger_creates_squid_tokens_from_lki`

**CR citation**: CR 603.10a (LKI for leaves-battlefield triggers); CR 113.7a (LKI general); CR 122.2 (counters cease on zone change); CR 400.7 (zone change creates new object).

**Setup**:
1. Build a 2-player game (`p1`, `p2`).
2. Place a Chasm Skulker on p1's battlefield.
3. Manually add 3 `+1/+1` counters to the Skulker (`obj.counters.update(CounterType::PlusOnePlusOne, 3)`).
4. Apply lethal damage (set `damage_marked >= toughness + counters`) OR call `Effect::DestroyPermanent { target: Skulker }` and process commands until SBA fires.
5. Process priority/SBA cycles until the death trigger resolves and tokens are created.
6. **Assert**: exactly **3** Squid tokens of name "Squid" appear on p1's battlefield.
7. **Assert**: each token has the `Islandwalk` keyword (or `Landwalk(Island)` per the actual DSL).
8. **Assert**: each token is a 1/1 blue Squid creature.
9. **Assert**: the Skulker's GameObject in graveyard has `counters: OrdMap::new()` (empty — confirming we're NOT preserving counters on the new graveyard object; we're using the snapshot).

**Discriminating assertion**: 3 tokens (not 0 — would catch missing LKI plumbing; not 1 — would catch a default fallback bug; not the wrong subtype — would catch token-spec construction errors).

### Test (b) — Toothy Imaginary Friend leaves-battlefield triggers draws X (X = LKI +1/+1 counter count)

**Test name**: `test_toothy_leaves_battlefield_draws_cards_from_lki_counters`

**CR citation**: CR 603.10a, CR 113.7a, CR 122.2.

**Setup**:
1. Build a 2-player game; p1 controls Toothy.
2. Add 4 `+1/+1` counters manually.
3. Confirm p1's library has at least 4 cards.
4. Bounce/exile/destroy Toothy via `Effect::ExileObject` or `Effect::DestroyPermanent` (forces `WhenLeavesBattlefield`).
5. Process priority/SBA cycles until the leaves-battlefield trigger resolves.
6. **Assert**: p1's hand size increased by exactly **4**.
7. **Assert**: p1's library shrank by exactly 4.

**Discriminating assertion**: 4 draws (not 0 — would catch the existing bug that produces 0 draws; not 1 — would catch a default fallback). This test FAILS pre-PB-LKI-CC and PASSES post-PB-LKI-CC; it is the regression sentinel.

### Test (c) — Zero-counter source baseline: creature with no counters dies → 0 tokens, no panic

**Test name**: `test_lki_counter_count_zero_counters_returns_zero_no_panic`

**CR citation**: CR 603.10a; defensive default behavior.

**Setup**:
1. Build a 2-player game; p1 controls a Chasm Skulker.
2. **Do NOT add any +1/+1 counters** (Skulker has 0 P1P1 counters at death).
3. Trigger death (lethal damage or destroy).
4. Process resolution.
5. **Assert**: exactly **0** Squid tokens are created on p1's battlefield.
6. **Assert**: no panic / no `unwrap()` failure / no `Result::Err` returned.
7. **Assert**: the `WhenDies` trigger DID resolve (check event log for `AbilityResolved` matching the Skulker's death trigger), confirming the `0` count is the result of the LKI lookup and not a skipped resolution.

**Discriminating assertion**: 0 tokens AND trigger resolved (distinguishes "correctly resolves to 0" from "trigger fizzled / never fired"). This catches a future regression where someone adds a `count == 0 → return early` short-circuit that breaks the resolved-but-zero semantics.

### Test (d) — Multi-counter mixed-type LKI returns the correct counter type

**Test name**: `test_lki_counter_count_multi_type_returns_requested_counter_type`

**CR citation**: CR 603.10a; CR 122.1 (per-counter-type tracking).

**Setup**:
1. Build a 2-player game; p1 controls a Chasm Skulker.
2. Add 5 `+1/+1` counters AND 2 `-1/-1` counters AND 1 `Loyalty` counter (intentionally a counter type Skulker shouldn't have, but the engine accepts it on `obj.counters`).
3. **Note**: SBAs apply -1/-1 vs +1/+1 cancellation (CR 704.5q) — verify behavior pre-death by checking SBAs run. After SBA cancellation, expect 3 P1P1 + 0 N1N1 (or similar; runner must verify exact post-SBA counter state). Pre-death counter snapshot is taken AFTER SBA, so the snapshot reflects post-cancellation values.
4. Trigger death.
5. Process resolution.
6. **Assert**: 3 Squid tokens (post-SBA P1P1 count, NOT 5 — would mean the snapshot was taken before SBA; NOT 8 — would mean we're summing all counter types).
7. **Assert**: counter type discrimination works — the LKI lookup correctly returns the P1P1 count, ignoring the other counter types stored in the same map.

**Discriminating assertion**: returns P1P1 count specifically, not total or wrong-type count. Validates the `counter` field of the new variant routes correctly through the OrdMap lookup.

**Note**: if the runner finds that mixing `Loyalty` counters on a creature is rejected upstream (e.g. SBAs sweep them off), substitute another non-P1P1 counter type that the engine accepts on creatures (e.g. `Charge`, `Storage`, `Time`). The discriminating semantics is "LKI lookup picks the right key from a multi-key map," not the specific counter types.

### Test (e) — Hash determinism + `HASH_SCHEMA_VERSION` sentinel + interaction with copy-doubling

**Test name**: `test_pb_lki_cc_hash_schema_version_and_lki_token_doubling_interaction`

**CR citation**: hash infrastructure (no specific CR); CR 614.1 (token-doubling replacement).

**Setup (3 sub-assertions)**:

1. **Sentinel**: `assert_eq!(HASH_SCHEMA_VERSION, 15u8, "PB-LKI-CC bumped HASH_SCHEMA_VERSION 14→15 (EffectAmount::CounterCountAtLastKnownInformation, CR 603.10a / 113.7a)")`.

2. **Determinism**: build two `EffectAmount::CounterCountAtLastKnownInformation { counter: PlusOnePlusOne }` instances; hash both via the trait. Assert hashes equal. Build a third with `counter: MinusOneMinusOne`; assert distinct hash.

3. **Interaction with copy-doubling at counter-add time** (per task brief mandatory test 5):
   - Build a 2-player game; p1 controls a Chasm Skulker AND a Hardened Scales (`+1/+1 counter put on creature you control: that many plus one`).
   - Have p1 draw a card (triggers Skulker's WheneverYouDrawACard add-counter ability).
   - **Assert**: Skulker has 2 P1P1 counters (1 base + 1 doubled by Hardened Scales) post-counter-replacement.
   - Have p1 draw a second card; Skulker now has 4 P1P1 counters (3 + 1 doubled).
   - Trigger death.
   - **Assert**: 4 Squid tokens are created (the LKI snapshot reflects the post-doubling counter count, NOT the pre-doubling count).
   - This validates that the snapshot captures *current state at death time*, including all prior continuous-effect modifications to counter placement. The doubling replacement (CR 614.1) operates at counter-add time, so the GameObject's counter map is already 2/4/etc. before death, and the snapshot inherits that. This is the correct semantic per CR 122.1 and CR 121.6: the snapshot is "what the counter map looked like at LKI," not "what would have been added if doubling was reapplied."

**Skip note for sub-3**: if Hardened Scales is not authored or its replacement-effect plumbing is incomplete in the test harness, the runner may substitute a manual counter-add path (`obj.counters.update(P1P1, current+2)`) and rely on the underlying Skulker death trigger to validate that the snapshot reflects the live count regardless of how it got there. Document the substitution in a comment.

If sub-3 cannot run cleanly due to upstream test-harness limitations: **stop-and-flag** rather than skip. Add a TODO comment in the test referencing OOS-LKI-3 (counter-doubling integration with LKI snapshot) and file the OOS seed in `pb-retriage-CC.md`.

---

## Step 7 — Implementation order (for runner, mechanical)

1. **Engine change 1** — DSL variant: add `EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType }` to `card_definition.rs:2247-2376` (after `PowerOfSacrificedCreature`).
2. **Engine change 2** — `EffectContext.lki_counters` field (effects/mod.rs:48-135 struct + 138/165 constructors).
3. **Engine change 3** — `PendingTrigger.lki_counters` field (state/stubs.rs:241-403 struct + 415-449 blank()).
4. **Engine change 4** — `StackObject.lki_counters` field (state/stack.rs:158-469 struct + 486-534 trigger_default()).
5. **Engine change 5** — Capture LKI at trigger-fire time (abilities.rs:4002-4011 SelfDies push + 4032-4040 SelfLeavesBattlefield push).
6. **Engine change 6** — Thread LKI through flush_pending_triggers (abilities.rs:7387 — after triggering_creature_id assignment).
7. **Engine change 7** — Build `ctx.lki_counters` at resolution (resolution.rs:2052 carddef path + 2120 chars path).
8. **Engine change 8** — `EffectAmount::CounterCountAtLastKnownInformation` arm in `resolve_amount` (effects/mod.rs after line 6166).
9. **Engine change 9** — `EffectAmount::CounterCountAtLastKnownInformation` arm in `resolve_cda_amount` (rules/layers.rs after the `CounterCount` arm — find via grep).
10. **Engine change 10** — HashInto for the new EffectAmount variant (hash.rs:4477 — discriminant 17).
11. **Engine change 11** — HashInto for new fields on PendingTrigger and StackObject (hash.rs — find both impls; hash the OrdMap iteration deterministically).
12. **Engine change 12** — HASH_SCHEMA_VERSION 14→15 bump + history entry 15 (hash.rs:50-75).
13. **Engine change 13** — Sentinel-assertion sweep across 6 test files (Step 3 Site 13 table).
14. **Card def 1** — `chasm_skulker.rs`: replace TODO at lines 30-36 with WhenDies triggered ability using new variant. Verify `KeywordAbility::Islandwalk` (or `Landwalk(LandwalkType::Island)`) exists; stop-and-flag if neither does.
15. **Card def 2** — `toothy_imaginary_friend.rs`: change `EffectAmount::CounterCount` → `EffectAmount::CounterCountAtLastKnownInformation` at line 48-51. Update misleading comment at line 43.
16. **OOS append** — `pb-retriage-CC.md`: append OOS-LKI-1 (cost-payment LKI for Workhorse-style cards) and OOS-LKI-2 (AnyCreatureDies dying-creature LKI) per Step 4.
17. **Tests (a-e)** — 5 tests in new file `crates/engine/tests/primitive_pb_lki_cc.rs`.
18. **Gates** — `cargo build --workspace`; `cargo test --workspace` (count must rise from 2720 to ≥ 2725); `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`. All gates green before signaling ready.
19. **Memo** — Write `memory/primitives/pb-review-LKI-CC.md` after impl + review.

---

## Step 8 — Verification checklist

- [ ] CR rule lookups complete: 603.10a, 113.7a, 400.7, 122.2 (verified via MCP)
- [ ] Engine architecture walk: 15 dispatch sites with file:line references, current behavior, required change
- [ ] Path decision documented (Path A chosen; Path B rejected with CR-grounded reasons)
- [ ] Card scope verification: 2 confirmed (Chasm Skulker + Toothy), TODO sweep + pattern sweep + known-card check all run
- [ ] Out-of-scope items explicit: 6 items + 2 OOS-LKI seeds drafted
- [ ] Hash strategy: bump 14→15, sentinel sweep across 6 test files, 1 history entry, 3 hash impl edits
- [ ] Test plan: 5 mandatory tests (a-e), CR citations, file paths
- [ ] Plan file written: `memory/primitives/pb-plan-LKI-CC.md`
- [ ] `memory/primitive-wip.md` updated: planner checklist boxes ticked, `phase: plan-complete`

## Acceptance criteria (from task brief)

1. ✅ Engine primitive landed with 5 mandatory tests (Tests a-e specified in Step 6)
2. ✅ ≥2 in-scope cards re-authored / fixed (Chasm Skulker + Toothy — Step 4)
3. ✅ HASH 14→15 + sentinel-assertion test files updated (Step 5 + Site 13 — 6 files)
4. ✅ cargo test --workspace green (count > 2725 baseline); cargo clippy --all-targets clean; cargo fmt --check clean (Step 7 step 18)
5. ✅ /review verdict PASS or PASS-WITH-NITS, 0 HIGH/MEDIUM (deferred to review phase)
6. ✅ Plan + review memos at `memory/primitives/pb-plan-LKI-CC.md` and `memory/primitives/pb-review-LKI-CC.md` (this file + review file written in Step 7 step 19)
7. ✅ Out-of-scope blockers as new OOS-LKI-N seeds in pb-retriage-CC.md (Step 4 — OOS-LKI-1 and OOS-LKI-2 drafted; runner appends at fix-phase end)

---

## Risks & edge cases

1. **AnyCreatureDies scope creep**: the planner explicitly limits the snapshot to SelfDies + SelfLeavesBattlefield trigger types. AnyCreatureDies (line 4280-4294 in abilities.rs) is a different dispatch arm where the dying creature is the *triggering object*, not the trigger source. Threading LKI counters into that arm would require a new `triggering_creature_lki_counters` field on PendingTrigger/StackObject/EffectContext — a separate primitive (filed as OOS-LKI-2). The runner MUST NOT add LKI plumbing to the AnyCreatureDies arm in PB-LKI-CC scope.

2. **Cost-payment LKI (Workhorse)**: similar shape (counters at LKI), different snapshot site (cost payment, not trigger fire). PB-P precedent for `PowerOfSacrificedCreature` snapshots LKI power at cost-payment time on `EffectContext.sacrificed_creature_powers`. PB-LKI-CC does NOT extend that vec to also carry counters — separate primitive (OOS-LKI-1). The runner MUST NOT extend `sacrificed_creature_powers` shape.

3. **CDA divergence**: `resolve_cda_amount` returns 0 for the new variant (Site 9). This is intentional — CDAs evaluate continuously while the source is on the battlefield (CR 613.4c), where counters are live. Card authors who pair `CounterCountAtLastKnownInformation` with a CDA misuse the variant; defensive 0 default avoids a panic. Document the divergence in the variant's doc-comment (Site 1).

4. **OrdMap clone cost**: `pre_death_counters.clone()` at trigger queueing time has structural-sharing cost; the OrdMap is small (typically 0-3 entries — P1P1, N1N1, occasional Loyalty). Acceptable.

5. **Replay-viewer / TUI exhaustive matches**: per `MEMORY.md` 50%-miss-rate gotcha. New EffectAmount variant should NOT add new arms to StackObjectKind/KeywordAbility, but `cargo build --workspace` is mandatory. If a tool exhaustively matches on EffectAmount somewhere, a new arm is needed; the build error will surface it.

6. **SBA timing — LKI snapshot vs SBA-induced counter changes**: counter SBAs (e.g. CR 704.5q +1/+1 vs -1/-1 cancellation) run BEFORE the death event. By the time `pre_death_counters` is captured at sba.rs:540, the counter map already reflects SBA cancellations. This is correct LKI semantics (CR 603.10a — "the appearance of objects immediately prior to the event"). Test (d) explicitly validates this for mixed counter types.

7. **Chasm Skulker oracle text precision**: oracle says "create X 1/1 blue Squid creature tokens with islandwalk." The runner must verify the keyword name in the engine — `KeywordAbility::Islandwalk` may not exist; the canonical form might be `KeywordAbility::Landwalk(LandwalkType::Island)`. Search via rust-analyzer `workspace_symbols` at impl time; **stop-and-flag** if neither variant exists.

8. **Backward-compat replays**: pre-PB-LKI-CC GameState snapshots with PendingTrigger/StackObject lacking the `lki_counters` field will fail to deserialize. This is acceptable per `memory/conventions.md` — hash version bump signals incompatibility. The `#[serde(default)]` annotation on the new fields handles missing-field cases for forward-compat WITHIN HASH 15, but cross-version replays are not expected to round-trip.

9. **Token spec construction note for Chasm Skulker**: TokenSpec is hand-constructed (no predefined helper for "Squid"). The runner must verify `..Default::default()` covers all post-PB-TS fields. Per PB-TS, `count: EffectAmount::Fixed(1)` is the new default; we override with `count: EffectAmount::CounterCountAtLastKnownInformation { ... }`. The Default impl handles `tapped`, `enters_attacking`, `mana_color`, `mana_abilities`, `activated_abilities`, `sacrifice_at_end_step`, `exile_at_end_step`.

10. **Misleading-comment cleanup**: the existing comment at `toothy_imaginary_friend.rs:43` ("counter count is preserved by move_object_to_zone") is technically false — the engine copies the GameObject in `move_object_to_zone:412-415` and then explicitly resets counters to OrdMap::new() at line 420. The comment was likely written when an earlier draft preserved counters, then never updated when the reset was added. This plan fixes the comment AND the underlying broken behavior. Future-proofing: if the runner finds OTHER card-def comments making similar claims, file as a separate doc-cleanup seed (LOW-priority, not in scope).
