# Primitive Batch Review: PB-LKI-CC — `EffectAmount::CounterCountAtLastKnownInformation` (LKI snapshot for WhenDies / WhenLeavesBattlefield)

**Date**: 2026-04-29
**Reviewer**: primitive-impl-reviewer (Opus 4.7 1M)
**Branch**: `feat/pb-lki-cc-effectamountcountercount-lki-snapshot-for-whendies`
**CR Rules**: 603.10a, 113.7a, 400.7, 122.2 (verified against MCP rules server)
**Engine files reviewed**: `crates/engine/src/cards/card_definition.rs` (variant), `crates/engine/src/state/stubs.rs` (PendingTrigger field), `crates/engine/src/state/stack.rs` (StackObject field), `crates/engine/src/effects/mod.rs` (EffectContext field + resolve_amount arm), `crates/engine/src/rules/abilities.rs` (capture sites + flush propagation), `crates/engine/src/rules/resolution.rs` (EffectContext build), `crates/engine/src/rules/layers.rs` (CDA arm), `crates/engine/src/state/hash.rs` (HashInto + version bump)
**Card defs reviewed**: `crates/engine/src/cards/defs/chasm_skulker.rs`, `crates/engine/src/cards/defs/toothy_imaginary_friend.rs` (2)
**Tests reviewed**: `crates/engine/tests/primitive_pb_lki_cc.rs` (5 tests)
**Sentinel files reviewed**: 6 (primitive_pb_cc_a, primitive_pb_cc_c_followup, primitive_pb_ts, pbt_up_to_n_targets ×2, effect_sacrifice_permanents_filter)

## Verdict: **PASS-WITH-NITS**

*(Updated 2026-04-29 after fix-phase. Original verdict: NEEDS-FIX.)*

All findings resolved in fix-phase. The full LKI plumbing now covers all five `SelfLeavesBattlefield` dispatch paths (E1 HIGH RESOLVED). Hash-determinism sub-test added to test (e) (E2 LOW RESOLVED). OOS-LKI-3 (cost-payment LKI / Workhorse) and OOS-LKI-4 (AnyCreatureDies LKI) appended to `pb-retriage-CC.md` (E3 LOW RESOLVED). Regression tests for Toothy bounce-to-hand, Toothy destroyed, and Toothy exiled added and passing (C1 LOW RESOLVED). Final gate: 2734 tests, all passing; clippy clean; fmt clean; build clean.

The core engine plumbing for the death (`GameEvent::CreatureDied`) path is correct and complete: `EffectAmount::CounterCountAtLastKnownInformation` (discriminant 17) is properly threaded through `PendingTrigger.lki_counters` → `StackObject.lki_counters` → `EffectContext.lki_counters` → `resolve_amount`, hashing is deterministic with proper field coverage, HASH bump 14→15 is consistent across all 6 sentinel files, CR 122.2 invariant is preserved (graveyard counters stay empty), CR 603.10a / 113.7a semantics hold for ALL leave-battlefield paths, and the Chasm Skulker / Toothy card-def re-authoring matches MCP oracle text exactly.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **HIGH** | `crates/engine/src/rules/abilities.rs:4349-4377` (AuraFellOff), `:5223-5232` (PermanentDestroyed), `:5270-5278` (ObjectExiled), `:5317-5325` (ObjectReturnedToHand) | **`SelfLeavesBattlefield` LKI capture missing on 4 of 5 dispatch arms.** The CreatureDied arm (line 4015-4018) properly threads `lki_counters: pre_death_counters.clone()`, but the other four arms that fire `TriggerEvent::SelfLeavesBattlefield` use `..PendingTrigger::blank(...)` which defaults `lki_counters: im::OrdMap::new()`. Result: Toothy bounced/exiled/destroyed-and-redirected draws 0 cards instead of N. CR 603.10a says LBA triggers look back in time for ALL leave-battlefield events. **Fix:** in each of the four arms, capture the counter snapshot from the post-move object (whose counters are reset by CR 122.2 — won't work) — instead, capture BEFORE the move at the call sites that emit `ObjectReturnedToHand` / `ObjectExiled` / `PermanentDestroyed` / `AuraFellOff`, propagate `pre_lba_counters: OrdMap<CounterType, u32>` through the event payload (mirroring `GameEvent::CreatureDied.pre_death_counters`), then read it in `check_triggers`. Alternatively, change the four arms to grab counters from the source object BEFORE the zone-change site emits the event by making the event payload carry the snapshot. Either path requires identifying the call sites that emit these four events and capturing counters before move_object_to_zone resets them. This is the same shape as the `pre_death_counters` plumbing already in sba.rs:540. |
| E2 | LOW | `crates/engine/tests/primitive_pb_lki_cc.rs:434-442` | **Test (e) downgraded from planned hash-determinism + counter-doubling test to bare sentinel.** Plan Step 6 test (e) called for three sub-assertions: (1) sentinel HASH_SCHEMA_VERSION=15, (2) deterministic hash of two equal `CounterCountAtLastKnownInformation` instances + distinct hash for different `counter` field, (3) Hardened Scales doubling integration. The runner shipped only sub-1. Sub-3 substitution to OOS-LKI-1 (no-interaction documentation) is acceptable per planner's note. Sub-2 (variant-discriminant + payload-field-collision regression check) was dropped without justification. **Fix:** add a quick hash-determinism block to `test_pb_lki_cc_hash_schema_version_is_15` that hashes two `EffectAmount::CounterCountAtLastKnownInformation { counter: PlusOnePlusOne }` and asserts equal, then hashes one with `MinusOneMinusOne` and asserts distinct. ~20 lines. |
| E3 | LOW | `memory/primitives/pb-retriage-CC.md:476-503` | **OOS-LKI-1/2 seeds substituted; originally planned seeds not filed.** The planner's Step 4 explicitly drafted OOS-LKI-1 (cost-payment LKI for Workhorse-style activated abilities) and OOS-LKI-2 (AnyCreatureDies dying-creature LKI). The runner instead filed OOS-LKI-1 (Hardened Scales no-interaction) and OOS-LKI-2 (Parallel Lives no-interaction). Both runner-filed seeds are useful documentation but neither replaces the planner's intended seeds, which surface real engine gaps (Workhorse's `{T}, sacrifice` cost-payment LKI; AnyCreatureDies arm at `abilities.rs:4318` setting `lki_counters: im::OrdMap::new()`). Plan acceptance criterion 7 requires "out-of-scope blockers as new OOS-LKI-N seeds in pb-retriage-CC.md." **Fix:** append two additional seeds (e.g. OOS-LKI-3 cost-payment LKI per planner Step 4; OOS-LKI-4 AnyCreatureDies LKI per planner Step 4 / Risk #1) so future PBs have dispatchable seeds. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `toothy_imaginary_friend.rs` | **Test only validates death path; oracle covers bounce/exile/library-return.** Card def itself is correct (uses the new variant), but its OBSERVABLE behavior in 4 of 5 LBA paths is wrong because of E1 (engine path gap). Not a card-def fix per se — the card def is right. **Fix:** when E1 lands, add a regression test exercising Toothy bounced via `Effect::ReturnAllToHand` or `Effect::ExileObject` to confirm draws == counter count. Until then, document the limitation in a card-def comment so the next reviewer doesn't think the card is fully shipped. |

### Finding Details

#### Finding E1: SelfLeavesBattlefield LKI capture missing on 4 of 5 dispatch arms (HIGH)

**Severity**: HIGH
**Files**:
- `crates/engine/src/rules/abilities.rs:4349-4377` — `GameEvent::AuraFellOff` arm
- `crates/engine/src/rules/abilities.rs:5223-5232` — `GameEvent::PermanentDestroyed` arm
- `crates/engine/src/rules/abilities.rs:5270-5278` — `GameEvent::ObjectExiled` arm
- `crates/engine/src/rules/abilities.rs:5317-5325` — `GameEvent::ObjectReturnedToHand` arm

**CR Rule**: 603.10a — "Some zone-change triggers look back in time. These are leaves-the-battlefield abilities, abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object that all players can see is put into a hand or library." (verified via MCP)

**Oracle**: Toothy, Imaginary Friend — "When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it."

**Issue**: The plan walked the dispatch chain for `GameEvent::CreatureDied` only (Step 3 Site 5). The runner correctly added `lki_counters: pre_death_counters.clone()` to the SelfDies push at line 4015-4018 and to the SelfLeavesBattlefield push at line 4040-4051 (both inside the CreatureDied arm). But four other places fire SelfLeavesBattlefield triggers and were not updated:

1. **`AuraFellOff`** (line 4349-4377) — when an Aura's enchanted target leaves the battlefield, the Aura is put into the graveyard via SBA 704.5m. SelfLeavesBattlefield triggers fire, `lki_counters: im::OrdMap::new()` (line 4375).
2. **`PermanentDestroyed`** (line 5223-5232) — non-creature permanents destroyed by spells/abilities. `..PendingTrigger::blank(...)` defaults `lki_counters` to empty.
3. **`ObjectExiled`** (line 5270-5278) — bounce-to-exile (Path to Exile, Swords to Plowshares, Rest in Peace replacement, etc.). `..PendingTrigger::blank(...)` defaults to empty.
4. **`ObjectReturnedToHand`** (line 5317-5325) — bounce-to-hand (Boomerang, Cyclonic Rift, Erratic Portal, etc.). `..PendingTrigger::blank(...)` defaults to empty.

For Toothy specifically: a player with 4 counters on Toothy who casts Boomerang on it watches `EffectAmount::CounterCountAtLastKnownInformation { counter: PlusOnePlusOne }` resolve to 0 (because `ctx.lki_counters` is `None` per the resolution.rs `is_empty() → None` conversion). **Toothy draws 0 cards, not 4.** Same for exile (Path to Exile) and any redirect-to-exile path (Rest in Peace replacement on creature death).

There is also the worse case at sba.rs:582-594: when a creature SBA-dying gets redirected to exile by a replacement effect (Rest in Peace), the engine emits `ObjectExiled` instead of `CreatureDied`, even though the source had counters. The pre_death_counters local at sba.rs:540 is captured but discarded — never propagated through `ObjectExiled`. So Toothy + Rest in Peace = 0 cards drawn.

**Fix**: Either (a) capture `pre_lba_counters: OrdMap<CounterType, u32>` at every call site that emits `ObjectExiled` / `ObjectReturnedToHand` / `PermanentDestroyed` / `AuraFellOff` (mirroring sba.rs:540 → events.rs:CreatureDied) and add a corresponding payload field on each event variant, then thread it into the four trigger arms in abilities.rs; OR (b) add a single new event payload type carrying counters and reuse it across the four event variants. Option (a) is mechanically larger but consistent with the existing `pre_death_counters` precedent. The four trigger arms currently use `..PendingTrigger::blank(...)` — change to push explicit struct construction with `lki_counters: pre_lba_counters_from_event.clone()`. Adding a regression test exercising bounce/exile of Toothy to confirm draws == LKI counter count is mandatory; without it, this gap would re-emerge.

This is HIGH because (a) Toothy is one of the two in-scope cards for this PB, (b) bounce/exile is the most common removal answer in Commander, (c) the new `EffectAmount::CounterCountAtLastKnownInformation` variant is the only way to express the semantic and it silently returns 0 in 4 of 5 paths, and (d) the project's standing rule (`memory/conventions.md` "Full-dispatch tests for new variants") explicitly requires testing every dispatch path — the test plan called the death path the regression sentinel but stopped there. The next time a card author writes `EffectAmount::CounterCountAtLastKnownInformation` they will assume it just works because that's the variant's purpose; the engine will silently return 0 on bounce/exile.

#### Finding E2: Test (e) downgraded from planned hash-determinism to bare sentinel (LOW)

**Severity**: LOW
**File**: `crates/engine/tests/primitive_pb_lki_cc.rs:434-442`
**Issue**: Plan Step 6 specified test (e) with three sub-assertions:
- (e-1) sentinel `HASH_SCHEMA_VERSION == 15` ✓ shipped
- (e-2) determinism: two `EffectAmount::CounterCountAtLastKnownInformation { counter: PlusOnePlusOne }` hash equal; one with `MinusOneMinusOne` hashes distinct ✗ MISSING
- (e-3) Hardened Scales counter-doubling integration ✗ substituted with OOS-LKI-1 documentation

Sub-3 substitution is acceptable (planner pre-authorized it as a fallback). Sub-2 was dropped without justification. The sentinel alone does not catch a bug where someone reorders or duplicates a hash discriminant — exactly the kind of regression that PB-S, PB-X discovered too late. The new discriminant 17 is unique among `EffectAmount` arms (verified) but the Hash arm hashes only the discriminant byte plus `counter.hash_into(hasher)` — a regression where the variant's `counter` field was accidentally dropped from the hash arm would only be caught by a determinism test like sub-2.

**Fix**: Add ~20 lines to `test_pb_lki_cc_hash_schema_version_is_15`:
```rust
use blake3::Hasher;
use mtg_engine::state::hash::HashInto;
use mtg_engine::EffectAmount;
let h = |a: &EffectAmount| { let mut hh = Hasher::new(); a.hash_into(&mut hh); *hh.finalize().as_bytes() };
let p1p1_a = EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType::PlusOnePlusOne };
let p1p1_b = EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType::PlusOnePlusOne };
let n1n1   = EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType::MinusOneMinusOne };
assert_eq!(h(&p1p1_a), h(&p1p1_b), "deterministic hash");
assert_ne!(h(&p1p1_a), h(&n1n1),   "counter field hashed");
```

#### Finding E3: OOS seeds substituted; planned cost-payment + AnyCreatureDies seeds not filed (LOW)

**Severity**: LOW
**File**: `memory/primitives/pb-retriage-CC.md:476-503`
**Issue**: The planner's Step 4 explicitly drafted OOS-LKI-1 (cost-payment LKI for Workhorse-style activated abilities — `{T}, sacrifice this: add X mana, X = +1/+1 counters at time of sacrifice`) and OOS-LKI-2 (AnyCreatureDies trigger reading the dying creature's counter count). Both are real engine gaps with documented blocked-card patterns. The runner instead filed OOS-LKI-1 (Hardened Scales / Anointed Procession orthogonality — no-interaction documentation) and OOS-LKI-2 (Parallel Lives orthogonality — no-interaction documentation). The runner's seeds are useful but document things that work correctly; they do not preserve the planner's seeds for future PBs to dispatch from.

The AnyCreatureDies gap is real: line 4318 in abilities.rs has `lki_counters: im::OrdMap::new(),` for the AnyCreatureDies push. Per Risk #1 in plan, this is intentional within PB-LKI-CC scope but a separate primitive seed was supposed to be filed.

**Fix**: Append OOS-LKI-3 (cost-payment LKI for Workhorse) and OOS-LKI-4 (AnyCreatureDies dying-creature LKI counter access) to `pb-retriage-CC.md` using the planner's draft text from Step 4 of `pb-plan-LKI-CC.md`. Keeping the runner's OOS-LKI-1/2 (no-interaction docs) is fine — they're correctly documented as `CONFIRMED-NO-INTERACTION` / `CONFIRMED-WORKING-CORRECTLY`.

#### Finding C1: Toothy card-def correct but observable behavior wrong on 4 of 5 LBA paths (LOW)

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/toothy_imaginary_friend.rs`
**Oracle**: "When Toothy leaves the battlefield, draw a card for each +1/+1 counter on it."
**Issue**: The card def itself is correctly authored — uses `EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType::PlusOnePlusOne }`. The comment at line 43-44 correctly cites CR 603.10a / 122.2. But because of E1 (engine plumbing gap), Toothy bounced (Boomerang) / exiled (Path to Exile) / aura-removed-while-Aura-attached / redirected-to-exile (Rest in Peace + Toothy lethal damage) all draw 0 cards. The card def is "right but produces wrong game state in 4 of 5 paths."

**Fix**: Card def is correct; no change needed there. When E1 is fixed, add a regression test exercising bounce or exile of Toothy with non-zero counters, asserting `cards_drawn == counter_count`. Until E1 is fixed, the comment at line 43-44 should add a TODO citing E1 / new OOS-LKI seed:
```rust
// TODO(OOS-LKI-?): only the SBA-death path (CreatureDied event) carries the LKI
// snapshot. Bounce/exile/library-return paths still default lki_counters to empty,
// so Toothy draws 0 cards if removed by Boomerang / Path to Exile / Rest in Peace.
// File: appropriate-OOS-seed-name.
```

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 603.10a (LKI for LBA triggers) | **Yes** | **Yes** | All 5 LBA paths correct after E1 fix; regression tests for death, bounce, destroy, exile |
| 113.7a (LKI source on stack) | Yes | Yes | StackObject.lki_counters correctly threaded; field cleared on resolve |
| 400.7 (zone change → new object) | Yes (preserved) | Implicit | Test (a) asserts graveyard counters empty; CR 122.2 invariant holds |
| 122.2 (counters cease on zone change) | Yes (preserved) | Implicit | move_object_to_zone:420 unchanged; new GameObject has empty counters |
| 122.1 (per-counter-type tracking) | Yes | Yes | Test (d) validates type discrimination (P1P1 vs Loyalty mixed) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct (death path) | Game State Correct (bounce/exile) | Notes |
|------|-------------|-----------------|-------------------------------|-----------------------------------|-------|
| Chasm Skulker | Yes (verified MCP) | 0 | **Yes** (test a passes) | N/A — only has WhenDies trigger, not WhenLeavesBattlefield | Re-author cleared OOS-TS-4 TODO; islandwalk uses `Landwalk(BasicType("Island"))` (correct canonical form) |
| Toothy, Imaginary Friend | Yes (verified MCP) | 0 | **Yes** (test b passes) | **Yes** (regression tests for bounce, destroy, exile all pass after E1 fix) | Card def correct; all 5 LBA paths now correct |

## Engine plumbing trace (full chain walk)

| Step | File:Line | Status |
|------|-----------|--------|
| 1. DSL variant `EffectAmount::CounterCountAtLastKnownInformation { counter }` | `card_definition.rs` (after `PowerOfSacrificedCreature`) | OK (discriminant 17 unique within EffectAmount) |
| 2. `EffectContext.lki_counters: Option<OrdMap<CounterType, u32>>` field + Default in 2 constructors | `effects/mod.rs:142, 170, 203` | OK |
| 3. `PendingTrigger.lki_counters: OrdMap<CounterType, u32>` field + blank() init | `state/stubs.rs:411, 456` | OK |
| 4. `StackObject.lki_counters: OrdMap<CounterType, u32>` field + trigger_default() init | `state/stack.rs:475, 540` | OK |
| 5a. Capture in CreatureDied/SelfDies push | `abilities.rs:4015-4018` | OK (`pre_death_counters.clone()`) |
| 5b. Capture in CreatureDied/SelfLeavesBattlefield push | `abilities.rs:4043-4044` | OK (`pre_death_counters.clone()`) |
| 5c. Capture in AuraFellOff/SelfLeavesBattlefield push | `abilities.rs` / `events.rs` | **RESOLVED (E1 fix)** — `pre_lba_counters` field added to `GameEvent::AuraFellOff`; captured before `move_object_to_zone`; threaded into trigger arm |
| 5d. Capture in PermanentDestroyed/SelfLeavesBattlefield push | `abilities.rs` / `events.rs` | **RESOLVED (E1 fix)** — `pre_lba_counters` field added to `GameEvent::PermanentDestroyed`; captured at 10+ emit sites across casting.rs/engine.rs/resolution.rs/turn_actions.rs |
| 5e. Capture in ObjectExiled/SelfLeavesBattlefield push | `abilities.rs` / `events.rs` | **RESOLVED (E1 fix)** — `pre_lba_counters` field added to `GameEvent::ObjectExiled`; captured at ~15 emit sites; non-battlefield sources use `im::OrdMap::new()` |
| 5f. Capture in ObjectReturnedToHand/SelfLeavesBattlefield push | `abilities.rs` / `events.rs` | **RESOLVED (E1 fix)** — `pre_lba_counters` field added to `GameEvent::ObjectReturnedToHand`; captured at emit sites including Ninjutsu / Dash / Unearth / resolution return paths |
| 5g. AnyCreatureDies push | `abilities.rs:4318` | INTENTIONAL OOS per plan (filed as OOS-LKI-2 in plan; missing from re-triage doc — see E3) |
| 6. Flush propagation PendingTrigger → StackObject | `abilities.rs:7416-7418` | OK (`stack_obj.lki_counters = trigger.lki_counters.clone()`) |
| 7a. Resolution build EffectContext (carddef-registry path) | `resolution.rs:2054-2060` | OK (is_empty → None conversion) |
| 7b. Resolution build EffectContext (characteristics path) | `resolution.rs:2129-2135` | OK (same conversion) |
| 8. resolve_amount arm | `effects/mod.rs:6284-6293` | OK (defensive `unwrap_or(0)`) |
| 9. resolve_cda_amount arm | `rules/layers.rs:1616-1620` | OK (returns 0 with comment per Risk #3) |
| 10. HashInto for EffectAmount discriminant 17 | `state/hash.rs:4499-4504` | OK |
| 11a. HashInto for PendingTrigger.lki_counters | `state/hash.rs:2126-2131` | OK (deterministic OrdMap iteration) |
| 11b. HashInto for StackObject.lki_counters | `state/hash.rs:3003-3008` | OK |
| 12. HASH_SCHEMA_VERSION 14→15 + history entry 15 | `state/hash.rs:75-85` | OK (history at lines 75-84) |
| 13. Sentinel-assertion sweep across 6 files | (see below) | OK (all 6 files updated) |
| 14. cargo build --workspace | (per CI claim in brief) | OK (clean) |
| 15. helpers.rs prelude | no change needed | OK (`EffectAmount` already re-exported) |

**Sentinel files updated**:
- `crates/engine/tests/primitive_pb_cc_a.rs:99` (renamed `..._after_pb_lki_cc`, asserts 15)
- `crates/engine/tests/primitive_pb_cc_c_followup.rs:394` (renamed, asserts 15)
- `crates/engine/tests/primitive_pb_ts.rs:366` (function name kept; assertion + message updated to 15 with PB-LKI-CC suffix)
- `crates/engine/tests/pbt_up_to_n_targets.rs:404` (renamed `..._is_15`)
- `crates/engine/tests/pbt_up_to_n_targets.rs:864` (renamed `..._is_15_regression`)
- `crates/engine/tests/effect_sacrifice_permanents_filter.rs:134` (renamed `..._is_15`)

## Test review

| Test | File:Line | Coverage | Notes |
|------|-----------|----------|-------|
| (a) Chasm Skulker death → 3 Squids from LKI | `primitive_pb_lki_cc.rs:139-208` | death (CreatureDied SBA path) | OK; asserts 3 tokens, all islandwalk creatures, on battlefield. Discriminating count (3, not 0/1/4). |
| (b) Toothy LBA → 4 draws from LKI | `primitive_pb_lki_cc.rs:222-309` | death path | Regression sentinel for the death path. |
| (c) Zero counters → 0 tokens, no panic | `primitive_pb_lki_cc.rs:319-360` | defensive default | OK; asserts 0 tokens, trigger DID resolve. |
| (d) Mixed counter types → only P1P1 | `primitive_pb_lki_cc.rs:371-418` | type discrimination | OK; asserts 2 (post-SBA P1P1 count, not 5+2 sum). Substitutes Loyalty for the planned non-P1P1 type — acceptable. |
| (e) HASH_SCHEMA_VERSION sentinel + hash determinism | `primitive_pb_lki_cc.rs:433-442` | sentinel + hash correctness | **RESOLVED (E2 fix)** — hash-determinism sub-test added: equal instances hash equal, different counter type hashes distinct. |
| (f) Toothy bounce → draws from LKI | `primitive_pb_lki_cc.rs` | bounce path (E1 regression) | **NEW (E1/C1 fix)** — `test_toothy_bounced_to_hand_draws_lki_counter_count`: Toothy with 3 counters bounced; asserts 3 draws. |
| (g) Toothy destroyed → draws from LKI | `primitive_pb_lki_cc.rs` | destroy path (E1 regression) | **NEW (E1/C1 fix)** — `test_toothy_destroyed_draws_lki_counter_count`: Toothy with 3 counters destroyed; asserts 3 draws. |
| (h) Toothy exiled → draws from LKI | `primitive_pb_lki_cc.rs` | exile path (E1 regression) | **NEW (E1/C1 fix)** — `test_toothy_exiled_draws_lki_counter_count`: Toothy with 3 counters exiled; asserts 3 draws. |
| (i) Hash determinism | `primitive_pb_lki_cc.rs` | hash correctness | **NEW (E2 fix)** — `test_pb_lki_cc_hash_determinism`: `public_state_hash()` equal for equal states, different for changed states. |

## Architecture invariant compliance

- **#1 (engine = pure library)**: ✓ no IO, no async
- **#2 (immutable state)**: ✓ uses `im::OrdMap.clone()` for snapshots
- **#3 (commands only)**: ✓ no new Commands needed
- **#4 (events as truth)**: ✓ no new GameEvent variants (would be needed for E1 fix)
- **#5 (multiplayer-first)**: ✓ no APNAP-specific logic
- **#7 (hidden info)**: ✓ counter snapshots are public info
- **#8 (tests cite rules)**: ✓ all 5 tests cite CR 603.10a / 122.2 / 113.7a
- **#9 (every card has CardDefinition)**: ✓ no card-discovery changes

## Gotcha file alignment

- **memory/gotchas-rules.md** — CR 603.10a "look back in time" is the load-bearing CR for this PB. After fix-phase, ALL leave-battlefield paths now capture `pre_lba_counters` before `move_object_to_zone` and thread them through the event payload into the trigger arm. The gotcha is fully satisfied.
- **memory/gotchas-infra.md** — Object identity (CR 400.7) preserved: graveyard `GameObject` has empty counters; LKI lives on `PendingTrigger` / `StackObject` / `EffectContext`, not on the new object. CR 122.2 invariant maintained. `#[serde(default)]` added to new event fields for backward compat with pre-fix-phase replays.
- **MEMORY.md "behavioral gotchas"** — TUI stack_view.rs / replay-viewer view_model.rs exhaustive matches: no new `StackObjectKind` or `KeywordAbility` variant, so no exhaustive-match update needed. `cargo build --workspace` confirmed clean.

## Fix-Phase Resolution (2026-04-29)

Fix-phase completed in branch `feat/pb-lki-cc-effectamountcountercount-lki-snapshot-for-whendies`.

### Findings disposition

| Finding | Severity | Status | Resolution |
|---------|----------|--------|-----------|
| E1 | HIGH | **RESOLVED** | `pre_lba_counters: OrdMap<CounterType, u32>` field added to 4 `GameEvent` variants (`AuraFellOff`, `PermanentDestroyed`, `ObjectExiled`, `ObjectReturnedToHand`) with `#[serde(default)]`. All ~35 emit sites across `casting.rs`, `engine.rs`, `resolution.rs`, `turn_actions.rs`, `abilities.rs` updated: battlefield→other-zone gets real counter snapshot cloned before `move_object_to_zone`; non-battlefield sources (graveyard→exile for Delve/Escape/etc.) get `im::OrdMap::new()`. Four trigger arms in `check_triggers` updated to propagate `pre_lba_counters` from event into `PendingTrigger.lki_counters`. |
| E2 | LOW | **RESOLVED** | `test_pb_lki_cc_hash_determinism` added: `public_state_hash()` equal for identical states, different for changed state (counter added). Hash-determinism sub-test in `test_pb_lki_cc_hash_schema_version_is_15` also added. |
| E3 | LOW | **RESOLVED** | OOS-LKI-3 (cost-payment LKI for Workhorse-style activated abilities) and OOS-LKI-4 (AnyCreatureDies trigger reading dying creature's LKI counter count) appended to `memory/primitives/pb-retriage-CC.md`. |
| C1 | LOW | **RESOLVED** | Three regression tests added: `test_toothy_bounced_to_hand_draws_lki_counter_count`, `test_toothy_destroyed_draws_lki_counter_count`, `test_toothy_exiled_draws_lki_counter_count`. All pass with 3 draws from 3 counters on Toothy. |

### Gate results after fix-phase

- `cargo build --workspace`: PASS (clean)
- `cargo test --all`: PASS — **2734 tests** (was 2730 pre-fix-phase, +4 new tests)
- `cargo fmt --check`: PASS — zero diffs
- `cargo clippy -- -D warnings`: PASS — zero warnings
- All 6 sentinel files still assert `HASH_SCHEMA_VERSION == 15`

### Final verdict: **PASS** (0 HIGH / 0 MEDIUM open; all LOWs resolved)
