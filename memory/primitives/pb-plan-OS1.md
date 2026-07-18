# Primitive Batch Plan: PB-OS1 — Gain-Control Reversion (UntilEndOfTurn / UntilYourNextTurn)

**Generated**: 2026-07-18
**Primitive**: Behaviour-only fix — wire the existing `recompute_object_controller`
helper into the two end-of-turn/next-turn continuous-effect expiry passes so that a
`SetController` effect reverting at expiry actually reverts `obj.controller`. **No new DSL
type, no wire type, no schema change.**
**CR Rules**: 611.2b/611.2c (duration & fixed-set control effects), 613.7/613.7b (timestamp
order of resolution continuous effects), 514.2 (cleanup ends "until end of turn" effects)
**Cards affected**: 3 known integrity fixes (`sarkhan_vol`, `zealous_conscripts`,
`karrthus_tyrant_of_jund`), all already `Complete` — roster sweep to confirm the full set.
Zero coverage flips (these are correctness fixes, not new completions).
**Dependencies**: PB-EF9 (`scutemob-110`) — built `recompute_object_controller` and the
Step-1/2/3 expiry pattern this PB mirrors. Present and verified.
**Deferred items from prior PBs**: OOS-EF9-1 (filed by PB-EF9's planner — "the latent
never-reverts gap on the other durations"). This PB closes it.

**Wire impact: NONE.** No `Effect` / `EffectAmount` / `EffectDuration` / `EffectFilter` /
`Condition` / `Cost` / `TargetFilter` variant is added or reshaped. **No `PROTOCOL_VERSION`
bump. No `HASH_SCHEMA_VERSION` bump.** If the runner finds a schema/hash arm must change,
that contradicts the brief — **STOP and report as a re-scope signal** (per the retriage §4
wire-change expectation).

---

## Primitive Specification

This is not a new capability; it is a correctness fix to two existing functions.

`Effect::GainControl` (`effects/mod.rs:5469`) resolves by creating a Layer-2
`ContinuousEffect` (`layer: EffectLayer::Control`, `modification:
LayerModification::SetController(controller)`, `filter:
EffectFilter::SingleObject(obj_id)`) with the card's stated duration, and imperatively
setting `obj.controller`. Every `GainControl` card produces a `SingleObject` filter
(confirmed at the executor — the loop only sets controller for `ResolvedTarget::Object`).

When the effect's duration ends, two expiry passes in `rules/layers.rs` **remove the
continuous effect from `state.continuous_effects` but never revert `obj.controller`**:

- `expire_end_of_turn_effects` (`layers.rs:1583`) — filter-collect reassignment dropping
  every `EffectDuration::UntilEndOfTurn` effect (CR 514.2). Called from
  `turn_actions::cleanup_actions`.
- `expire_until_next_turn_effects` (`layers.rs:1631`) — same shape for
  `EffectDuration::UntilYourNextTurn(active_player)` (CR 611.2b). Called from
  `turn_actions::untap_active_player_permanents` at the untap step.

Neither calls `recompute_object_controller`, so the borrowed permanent stays under the
thief's control forever. `sarkhan_vol`, `zealous_conscripts`, `karrthus_tyrant_of_jund`
all ship `Complete` and are therefore **legal-but-wrong** (invariant #9 — a wrong
`Complete` def corrupts replay history).

The fix helper **already exists and is idle for these passes**:
`recompute_object_controller` (`layers.rs:1797`, currently private, called only from
`expire_while_you_control_source_effects`). It recomputes an object's controller from its
owner plus any **still-active** Layer-2 `SetController` effects in timestamp order (CR
613.7). Its doc comment states the required ordering: **the removed effect must already be
gone from `state.continuous_effects` before the call**, or it would re-apply the effect it
is meant to expire.

**The fix is: for each of the two passes, before dropping the effects, collect the
`ObjectId` of every removed `SetController`/`Control`/`SingleObject` effect matching that
pass's duration; after the reassignment, call `recompute_object_controller(state, id)` for
each. Mirror the PB-EF9 Step-2/Step-3 structure exactly.** ~10 lines of engine change.

## CR Rule Text

- **CR 514.2** — "Second, the following actions happen simultaneously: all damage marked
  on permanents (including phased-out permanents) is removed and all 'until end of turn'
  and 'this turn' effects end. This turn-based action doesn't use the stack." → drives
  `expire_end_of_turn_effects`; the control effect ending here means control reverts here.
- **CR 611.2b** — "for as long as ..." durations; if the duration ends the effect does
  nothing. → the `UntilYourNextTurn(active_player)` removal in
  `expire_until_next_turn_effects` must revert control.
- **CR 611.2c** — the set of objects a resolution control effect affects is fixed at
  creation and never changes. `recompute_object_controller` respects this: it reverts to
  owner only when no *other* still-active `SetController` effect covers the object; a
  second, still-active steal keeps its controller (stacked control) — it is not a blind
  owner-reset.
- **CR 613.7 / 613.7b** — resolution continuous effects apply in timestamp order; a
  continuous effect receives its timestamp when created. `recompute_object_controller`
  sorts the still-active `SetController` effects by `timestamp` and folds them, so the
  latest-timestamp active control effect wins.

## Engine Changes

### Change 1: revert control in `expire_end_of_turn_effects`

**File**: `crates/engine/src/rules/layers.rs` (`:1583`, the `expire_end_of_turn_effects`
continuous-effect block at `:1586-1592`)
**Action**: Before the `state.continuous_effects = keep;` reassignment, collect the
target `ObjectId`s of every effect being removed that is a control effect:

```
let reverted: Vec<ObjectId> = state
    .continuous_effects
    .iter()
    .filter(|e| e.duration == EffectDuration::UntilEndOfTurn
        && e.layer == EffectLayer::Control
        && matches!(e.modification, LayerModification::SetController(_)))
    .filter_map(|e| match e.filter {
        EffectFilter::SingleObject(id) => Some(id),
        _ => None,
    })
    .collect();
```

Then AFTER `state.continuous_effects = keep;` (so the removed effect is gone before
recompute — REQUIRED per the helper's doc comment), call:

```
for id in reverted {
    recompute_object_controller(state, id);
}
```

**Pattern**: `expire_while_you_control_source_effects` Step-1 (collect `affected`) →
Step-2 (reassign `continuous_effects`) → Step-3 (`recompute_object_controller` loop),
`layers.rs:1733-1783`.
**CR**: 514.2 (effect ends at cleanup) + 613.7 (recompute in timestamp order).
**Placement note**: this block is disjoint from the replacement-effect and flash-grant
expiry that follow in the same function; add it to the continuous-effect block only.

### Change 2: revert control in `expire_until_next_turn_effects`

**File**: `crates/engine/src/rules/layers.rs` (`:1631`, the continuous-effect block at
`:1633-1639`)
**Action**: Same shape, gated on the per-player duration. Collect before the reassignment:

```
let reverted: Vec<ObjectId> = state
    .continuous_effects
    .iter()
    .filter(|e| e.duration == EffectDuration::UntilYourNextTurn(active_player)
        && e.layer == EffectLayer::Control
        && matches!(e.modification, LayerModification::SetController(_)))
    .filter_map(|e| match e.filter {
        EffectFilter::SingleObject(id) => Some(id),
        _ => None,
    })
    .collect();
```

Then after `state.continuous_effects = keep;`, run the same
`recompute_object_controller` loop. Place it before the replacement-effect / flash-grant /
protection / ability-reset blocks that follow (order among those is irrelevant, but
recompute must follow the continuous-effect reassignment).
**CR**: 611.2b (duration end) + 613.7.

### Change 3: visibility

`recompute_object_controller` is a private `fn` in the **same module** (`rules/layers.rs`)
as both expiry functions. **No visibility change is needed** — both callers are in-module.
Confirm (do not widen to `pub(crate)` / `pub`). This is a positive assertion the runner
records.

### Change 4: exhaustive match / wire sites — NONE

No enum variant is added. `state/hash.rs`, `tools/replay-viewer/src/view_model.rs`,
`tools/tui/src/play/panels/stack_view.rs`, `rules/protocol.rs`
(`PROTOCOL_SCHEMA_FINGERPRINT`), and the sentinel hash tests are **untouched**. The runner
must verify `cargo build --workspace` stays green with **no** `PROTOCOL_VERSION` /
`HASH_SCHEMA_VERSION` edit. If any of those gates go red demanding a bump, STOP and report.

## Card Definition Fixes

**None.** All affected defs are already `Complete` and already model the ability correctly
(`Effect::GainControl { duration: UntilEndOfTurn }`). The bug is entirely in the engine
expiry path; the defs need no edit. This PB produces **0 coverage flips** — the deliverable
is the integrity correction plus the affected-card count.

### Roster sweep (mandatory deliverable — enumerate from `all_cards()`, NOT grep)

Per the SR-34/36 lesson (the `mana_abilities: vec![]` / source-grep trap), the runner MUST
enumerate the affected set from the **compiled registry**, not source text:

1. Iterate `mtg_engine::all_cards()`.
2. For each def, walk every `AbilityDefinition` / `Effect` tree for
   `Effect::GainControl { duration, .. }` (recurse through `Effect::Sequence`,
   `Effect::Conditional`, `Effect::ForEach`, modal/`Effect::MayPay*` wrappers — sarkhan_vol
   nests it inside a `LoyaltyAbility` → `Effect::Sequence`).
3. Filter to `duration ∈ { UntilEndOfTurn, UntilYourNextTurn(_) }`.
4. Report the full list + count.

**Known (preliminary grep, to be confirmed by the registry walk)**: 9 files use
`Effect::GainControl` — `sarkhan_vol`, `zealous_conscripts`, `karrthus_tyrant_of_jund`,
`thieving_skydiver`, `roil_elemental`, `olivia_voldaren`, `dragonlord_silumgar`,
`connive`, `archmages_charm`. Of these, only the ones with `UntilEndOfTurn` /
`UntilYourNextTurn` are in-scope (the retriage names the first three; the others use
`WhileYouControlSource` / `WhileSourceOnBattlefield` / permanent durations — verify each
from the registry, do not assume). The count of in-scope cards is the headline deliverable.

## New Card Definitions

None.

## Unit Tests

**File**: `crates/engine/tests/primitives/primitive_pb32.rs` (already registered as `mod
primitive_pb32;` in `tests/primitives/main.rs:41`; run via `cargo test --test primitives
primitive_pb32::`). Use the existing hand-built-`ContinuousEffect` + direct-expiry-call
style already in this file (`test_gain_control_creates_continuous_effect`,
`test_gain_control_until_eot_expires`) — that is the correct level for this fix.

### Test 1 — de-vacuous the canary `test_gain_control_until_eot_expires` (`:333`)

The current test asserts ONLY that the continuous effect is removed (`assert!(!
has_control_effect, ...)`). It passes today **with the bug live**. Add a control-reversion
assertion so it **fails pre-fix, passes post-fix**. Insert immediately after the existing
`assert!(!has_control_effect, ...)` block (the `eff` uses
`filter: EffectFilter::SingleObject(target_id)`, `p2` is the creature's owner):

```
assert_eq!(
    state.objects().get(&target_id).unwrap().controller,
    p2,
    "CR 514.2/613.7: after UntilEndOfTurn control effect expires at cleanup, \
     control reverts to the owner (p2), not stays with the thief (p1)"
);
```

(Update the test's doc comment — currently "the continuous effect is removed and controller
reverts" — the "controller reverts" clause was aspirational; the assertion now backs it.)

### Test 2 — `test_gain_control_until_eot_stacked_control_persists`

Two `SetController` effects on the SAME object. Owner `p2`; `p1` steals via `UntilEndOfTurn`
(timestamp 100); `p3` steals via `WhileSourceOnBattlefield` (later timestamp 101, source =
a battlefield permanent controlled by `p3` so `is_effect_active` returns true). Set
`obj.controller = p3` (latest active). Call `expire_end_of_turn_effects`. Assert:
- the `UntilEndOfTurn` effect is removed;
- the `WhileSourceOnBattlefield` effect remains;
- `state.objects()[target].controller == p3` — control stays with the SECOND controller,
  does **NOT** snap to owner `p2`.
**Proves** `recompute_object_controller`'s "keep the other still-active effect" path (CR
611.2c / 613.7), not a blind owner-reset. Build the `p3` source object on the battlefield so
`WhileSourceOnBattlefield`'s `is_effect_active` check (`layers.rs:503`) passes.
**CR**: 611.2c, 613.7.

### Test 3 — `test_gain_control_until_next_turn_reverts_at_untap`

Steal `UntilYourNextTurn(p1)` on p1's turn (or the classic "steal on opponent's turn"
framing). Assert the creature does **not** revert at `expire_end_of_turn_effects` (cleanup
of the current turn) but **does** revert when `expire_until_next_turn_effects(state, p1)` is
called at p1's next untap. Two calls, two assertions:
- after `expire_end_of_turn_effects` → `controller == p1` (still stolen; UntilYourNextTurn
  survives cleanup);
- after `expire_until_next_turn_effects(state, p1)` → `controller == owner`.
**Proves** the UntilEndOfTurn-vs-UntilYourNextTurn timing distinction (CR 514.2 vs 611.2b)
and that Change 2's collection is gated on the correct `active_player`.
**CR**: 514.2, 611.2b.
**Import note**: `expire_until_next_turn_effects` may need adding to the `use
mtg_engine::rules::layers::` import line at the top of the file (currently only
`expire_end_of_turn_effects` is imported).

**Negative-coverage note**: Test 2 doubles as the negative test (stacked control must NOT
revert to owner) — a naive "always set controller = owner on expiry" fix would fail it.

## Golden-Script Reconciliation

The change alters existing Threaten-style behaviour (a borrowed creature now reverts), so
any golden script that asserted the creature STAYS after a turn boundary must be reconciled
to the reverting behaviour with a CR 611.2b/613.7 (or 514.2) citation.

**How to search** (the broad `control` grep matches every `controller:` field in
`initial_state` — useless; search precisely):
1. Grep `test-data/generated-scripts/` for the roster card names
   (`sarkhan`, `zealous_conscripts`, `karrthus`) and steal keywords (`Act of Treason`,
   `Threaten`, `gain control`, `GainControl`).
2. For any hit, check whether the script crosses a turn boundary
   (`end_turn` / `pass_priority` into cleanup / a subsequent untap) AND asserts the stolen
   creature's `controller`/ownership afterward.
3. Reconcile any such assertion to the reverting value, citing CR 611.2b/613.7 in the
   script's rationale/comment.

**Expectation**: likely **zero** scripts (no steal card appears in the roster of the 271
scripts; a targeted grep should return none). If zero, record "golden-script sweep: 0
scripts assert a borrowed creature persists across a turn boundary" as a positive assertion
in the close-out. The `run_all_scripts` suite must stay green (SR-9c) — an approved script
silently encoding the old bug is exactly what the reviewer will check for.

## Verification Checklist

- [ ] `expire_end_of_turn_effects` reverts control of removed `UntilEndOfTurn`
      `SetController` targets (Change 1)
- [ ] `expire_until_next_turn_effects` reverts control of removed
      `UntilYourNextTurn(active_player)` `SetController` targets (Change 2)
- [ ] `recompute_object_controller` visibility unchanged (still private, in-module)
- [ ] Canary `test_gain_control_until_eot_expires` de-vacuoused; fails on pre-fix engine,
      passes post-fix
- [ ] Stacked-control test passes (control stays with 2nd active controller, no owner-snap)
- [ ] UntilYourNextTurn timing test passes (reverts at untap, not cleanup)
- [ ] Roster sweep from `all_cards()` complete; affected-card count reported
- [ ] Golden-script sweep complete; any persist-after-turn assertion reconciled w/ CR cite
- [ ] `cargo build --workspace` green with **NO** `PROTOCOL_VERSION` / `HASH_SCHEMA_VERSION`
      change (wire impact NONE)
- [ ] `cargo test --all` passes (incl. `core card_defs_fmt`, `run_all_scripts`)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` + `tools/check-defs-fmt.sh` clean

## Risks & Edge Cases

- **Ordering hazard (load-bearing)**: `recompute_object_controller` MUST be called AFTER
  `state.continuous_effects = keep;`. If called before, it re-reads the effect being expired
  (still `is_effect_active == true` for both durations, `layers.rs:514/518`) and re-applies
  it — a no-op revert. The plan places both loops after the reassignment; the runner must
  preserve this.
- **Non-`SingleObject` filters**: no card produces a broader-filter `GainControl` today
  (confirmed at the executor — only `ResolvedTarget::Object` sets controller). The
  `filter_map` on `SingleObject` mirrors the same documented limitation PB-EF9 recorded
  (`layers.rs:1754-1762`). If a future card builds a `WhileYouControlSource`/duration
  control effect with `AllPermanents`/`CreaturesYouControl`, this and the PB-EF9 site both
  need widening — note it, do not build it here (0 current cards).
- **Simultaneity of SBAs (CR 514.2)**: cleanup ends the effect; the recompute happens inline
  in the same pass, before the next priority. No trigger fires on control reversion (no
  "control changed" `GameEvent` exists — PB-EF9 confirmed via grep). Consistent with the
  existing `expire_while_you_control_source_effects` which also returns nothing.
- **Stacked control with two UntilEndOfTurn effects on one object**: both are removed in the
  same pass; `reverted` may list the same `ObjectId` twice → two `recompute_object_controller`
  calls, both idempotent (recompute reads the post-removal effect set). Harmless. (If the
  runner prefers, dedup the `reverted` vec — optional, not required.)
- **`UntilYourNextTurn(p)` for a different player**: Change 2 is correctly gated on
  `active_player`, so a creature stolen `UntilYourNextTurn(p3)` is not reverted when
  `expire_until_next_turn_effects(state, p1)` runs — only at p3's untap. Test 3's gating
  assertion covers the same-player case; the different-player case is covered by construction
  (the filter equality `== UntilYourNextTurn(active_player)`).

## Out-of-Scope / Follow-up (flag in close-out)

- **`WhileSourceOnBattlefield` gain-control reversion**: OOS-EF9-1 names it, but that
  duration is removed by SBA when the source leaves the battlefield (a **different removal
  path** — `sba`/`is_effect_active` returns false, not the end-of-turn passes). The same
  two-line change here does **NOT** cover it, and it would have its own golden-script blast
  radius. **Deferred** — do not fold in. Flag as a follow-up seed (candidate: extend the
  SBA-driven continuous-effect cleanup to call `recompute_object_controller` on removed
  `WhileSourceOnBattlefield` `SetController` effects). This matches the retriage §4
  "Explicitly NOT in scope" instruction.
