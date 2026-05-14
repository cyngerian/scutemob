# Primitive Batch Review: PB-EWC — `ReplacementModification::EntersWithCounters` count u32 → EffectAmount

**Date**: 2026-05-14
**Reviewer**: primitive-impl-reviewer (Opus 4.7 1M)
**Branch**: `feat/pb-ewc-replacementmodificationenterswithcounters-count-u32-e`
**CR Rules verified independently via MCP**: 614.1, 614.1c, 614.12 (+612.12a/b/c), 614.15, 107.3, 122.1, 122.6, 400.7, 613.1d
**Engine files reviewed**:
- `crates/engine/src/state/replacement_effect.rs` — variant migration to `count: Box<EffectAmount>` + doc comment
- `crates/engine/src/state/hash.rs` — `HASH_SCHEMA_VERSION` 17→18 + history entry + `EntersWithCounters` hash arm
- `crates/engine/src/rules/replacement.rs` — `emit_etb_modification` signature + EffectContext build/source resolution + `apply_etb_replacements` source threading + `apply_self_etb_modification` adapter + `WouldEnterBattlefield` filter rebind in `register_permanent_replacement_abilities`
- `crates/engine/src/effects/mod.rs` — `resolve_amount` visibility `fn` → `pub(crate) fn`

**Card defs reviewed** (2): `crates/engine/src/cards/defs/master_biomancer.rs`, `crates/engine/src/cards/defs/ingenious_prodigy.rs`

**Tests reviewed**: `crates/engine/tests/primitive_pb_ewc.rs` (5 tests: a, b, c, d, e); `crates/engine/tests/x_cost_spells.rs:269` (legacy Ingenious Prodigy X=4 test, verified-still-correct under new code path); `crates/engine/tests/replacement_effects.rs:62-89,2019-2078,3176-3306` (existing call-site migrations to `Box::new(EffectAmount::Fixed(N))`)

**Sentinel files reviewed**: 12 (all updated 17 → 18, matching `HASH_SCHEMA_VERSION == 18u8`)

## Verdict: **PASS-WITH-NITS**

The PB-EWC implementation correctly migrates `ReplacementModification::EntersWithCounters` from a static `u32` to a dynamic `Box<EffectAmount>`. The full chain — DSL variant (`replacement_effect.rs:140-143`) → source resolution at `apply_etb_replacements` (`replacement.rs:1416-1440`) → adapter at `apply_self_etb_modification` (`replacement.rs:1465`) → `emit_etb_modification` EffectContext build with `ctx.source = replacement_source` + `ctx.x_value = state.objects[source].x_value` (`replacement.rs:1522-1528`) → `resolve_amount` invocation with clamped u32 boundary (`replacement.rs:1529`) → counter-doubling replacement (`apply_counter_replacement`, count==0 early-exit preserved) → counter insertion + `CounterAdded` event (gated on `modified_count > 0`) — is plumbed end-to-end.

The two card defs match MCP oracle text verbatim. Master Biomancer's `CreatureControlledBy(PlayerId(0))` placeholder is correctly rebound to the actual controller at registration time via a new `WouldEnterBattlefield { filter }` clause in `register_permanent_replacement_abilities` that calls the existing `bind_object_filter` (which already handled both `ControlledBy` and `CreatureControlledBy` placeholders). The "exclude_self" argument is sound: `apply_etb_replacements` runs BEFORE `register_permanent_replacement_abilities` at every ETB call site (verified across resolution.rs lines 1606-1627 / 3031-3043 / 4188-4198 / 5870-5882 / 6078-6089 / 6289-6300 / 6515-6526 / 6938-6946 and lands.rs:114-385), so MB's replacement is not yet registered when MB itself enters. Ingenious Prodigy's X-value flow is correct: `obj.x_value = stack_obj.x_value` at resolution.rs:546 happens BEFORE `apply_self_etb_from_definition` runs at line 1606, so the entering permanent's `x_value` is populated when `emit_etb_modification` reads it.

`HASH_SCHEMA_VERSION` bump 17→18 with history entry is consistent across all 12 sentinel sites. The `Box` choice is justified (recursive `Sum(Box, Box)` variant + `CardCount { filter: TargetFilter }` make the enum large) and uses Rust's automatic Box auto-deref through `HashInto` method resolution — no new `impl HashInto for Box<T>` needed, mirroring the existing `Sum(Box, Box)` precedent. The `count == 0` early-exit in `apply_counter_replacement` correctly suppresses both the counter-doubling replacement chain AND the `CounterAdded(0)` event for the X=0 case (Ingenious Prodigy cast with `X=0` enters with no counters and no event — test e verifies this).

`cargo test --workspace --lib --tests` = 2754 passing (+5 new PB-EWC tests); `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo fmt --all -- --check` are all reported clean. The 3 OOS-EWC-N seeds in `pb-retriage-CC.md` (type-grant via `EntersAsAdditionalType`, Golgari Grave-Troll self-ETB `CardCount`, Dragonstorm Globe non-self subtype receiver) correctly bound out-of-scope follow-ups.

LOW findings concern: (1) defensive-default code structure in `apply_etb_replacements` is misleading (mentions a "race" in single-threaded code); (2) doc-comment line-number drift in both card defs and inline replacement.rs comments; (3) test (e) X=0 assertion via `unwrap_or(0)` can't distinguish "counters map missing key" from "counters map has key with value 0"; (4) `bind_object_filter` doesn't handle `OwnedByOpponentsOf(PlayerId(0))` placeholder for `WouldEnterBattlefield` (not in-scope for any current card, but latent gap symmetric to the `WouldChangeZone` case). None of these affect game state for in-scope cards.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | LOW | `crates/engine/src/rules/replacement.rs:1416-1432` | **Defensive-default + race-comment pattern is structurally awkward.** The code does `find().map(...).unwrap_or((EntersTapped, None))` then re-iterates to check `still_present` afterwards. The comment says "Defensive: effect dropped between find_applicable and lookup. Skip emitting any modification rather than fabricate one." but there is no concurrency in single-threaded `&mut state` code — `find_applicable` is called immediately above and `state.replacement_effects` cannot be mutated between the two reads. The fabricated `EntersTapped` is never reached in any execution and the `still_present` re-iteration is dead-code-equivalent. **Fix:** Restructure to a let-else early-`continue` pattern: `let Some((modification, replacement_source)) = state.replacement_effects.iter().find(\|e\| e.id == id).map(\|e\| (e.modification.clone(), e.source)) else { continue; };`. Removes the misleading defensive default and the redundant second iteration. |
| E2 | LOW | `crates/engine/src/rules/replacement.rs:1787-1797` | **`bind_object_filter` does not handle `OwnedByOpponentsOf(PlayerId(0))` placeholder for `WouldEnterBattlefield` triggers.** The `WouldEnterBattlefield { filter }` arm calls `bind_object_filter(filter, controller)`, but `bind_object_filter` only rebinds `ControlledBy(0)` and `CreatureControlledBy(0)`. A non-self `WouldEnterBattlefield` with `OwnedByOpponentsOf(PlayerId(0))` (e.g. "When a creature an opponent controls enters") would leak the placeholder through registration. The symmetric `WouldChangeZone` arm (lines 1777-1786) does handle this case via direct pattern matching. No in-scope card hits this. **Fix:** Extend `bind_object_filter` to also rebind `OwnedByOpponentsOf(PlayerId(0))` → `OwnedByOpponentsOf(controller)`. ~3 lines. Alternative: track as OOS seed alongside OOS-EWC-3 (Dragonstorm Globe subtype receiver), since both touch the same `ObjectFilter` rebind gap. |
| E3 | LOW | `crates/engine/src/state/replacement_effect.rs:138-143` (doc comment) | **Doc-comment refers to line numbers that will drift.** Comment cites "`resolution.rs:546` for self-ETB" and "the resolver builds an EffectContext pinned to the replacement source (read from `ReplacementEffect.source` for global replacements; `new_id` for self-ETB)". The semantic claim is correct; only the line number is fragile. **Fix:** Drop the explicit `resolution.rs:546` reference, leave the function-name reference ("set during permanent-spell resolution before ETB processing"). Same fix pattern as PB-LKI-Power E3. |
| E4 | LOW | `crates/engine/src/rules/replacement.rs:1509-1521` (inline comment) | **Inline comment cites `effects/mod.rs:6140` and `resolution.rs:546` line numbers that will drift.** Same pattern as E3. **Fix:** Drop the explicit line numbers, reference the function names only. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `master_biomancer.rs:14-19, 40-47` | **Card def comments cite stale-prone line numbers.** Header notes "register_permanent_replacement_abilities runs AFTER apply_etb_replacements for the same ETB (resolution.rs:1606-1627)"; inline comment cites "bind_object_filter at registration (replacement.rs:505)" and "object_matches_filter (replacement.rs:437)" and "calculate_characteristics (effects/mod.rs:6140)". The semantic argument is correct and load-bearing for the "no exclude_self needed" claim. **Fix:** Drop the explicit line numbers; keep the function names. Per `feedback_verify_cr_before_implement.md` + conventions.md "aspirationally-wrong comments are correctness hazards" — these aren't aspirationally wrong today, but will be in 1-2 PRs. |
| C2 | LOW | `ingenious_prodigy.rs:13-18` | **Card def header cites stale-prone line numbers `resolution.rs:546` and `resolution.rs:1606`.** Same pattern as C1. **Fix:** Drop the line numbers; keep function-name references ("during permanent-spell resolution before ETB processing"). |
| C3 | (none) | both cards | Both cards match MCP oracle text verbatim. No remaining TODOs in the counter half. Master Biomancer's type-grant half preserved as a clearly-labeled `TODO (OOS-EWC-1)` referencing the filed seed. Ingenious Prodigy's previous "CR 614.1c DEVIATION" comment correctly removed (the deviation no longer exists — counters are now placed by replacement, not by stack trigger). Upkeep draw-trigger preserved unchanged with its own DEVIATION comment about the "you may remove" being unconditional, which is a pre-existing PB-27 gap, NOT PB-EWC. |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| T1 | LOW | `primitive_pb_ewc.rs:408-461` (test e) | **X=0 counter-absence test cannot discriminate "key absent" from "key present with value 0".** The assertion reads via `obj.counters.get(&PlusOnePlusOne).copied().unwrap_or(0)` — if a bug ever inserted `(PlusOnePlusOne, 0)` into the OrdMap, the test would still pass (returns 0 either way). The `CounterAdded` event check IS discriminating (catches count: 0 emit), so the test catches the more-likely failure mode. **Fix:** Strengthen to `assert!(!obj.counters.contains_key(&CounterType::PlusOnePlusOne))` AND keep the existing `unwrap_or(0)` check + the CounterAdded check. ~2 lines. Non-blocking — current assertion is correct for the actual implementation. |
| T2 | (none) | test (a) through (d) | Tests are well-targeted and discriminating. Test (a) verifies live-power read AND filter rebind in a single setup (`MB has 2 power → mystic enters with 2 counters; sanity-checks that exactly 1 effect with `CreatureControlledBy(p1)` is registered, proving PlayerId(0) placeholder does NOT survive). Test (b) is the load-bearing pumped-power discriminator (`calculate_characteristics` layer-7d resolution): forces MB to 4/6 via 2 +1/+1 counters and asserts the *next* entering creature picks up 4, not the printed 2. Test (c) verifies X-value flows through to the self-ETB replacement. Test (d) is the HASH sentinel canary at `18u8`. |

### Finding Details

#### Finding E1: Defensive-default + race-comment pattern (LOW)

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1416-1432`
**Code**:
```rust
let (modification, replacement_source) = state
    .replacement_effects
    .iter()
    .find(|e| e.id == id)
    .map(|e| (e.modification.clone(), e.source))
    .unwrap_or((
        // Defensive: effect dropped between find_applicable and lookup.
        // Skip emitting any modification rather than fabricate one.
        ReplacementModification::EntersTapped,
        None,
    ));
// If the find above produced a defensive default, the effect lookup raced —
// skip this iteration without emitting a modification.
let still_present = state.replacement_effects.iter().any(|e| e.id == id);
if !still_present {
    continue;
}
```

**Issue**: There is no concurrency in the engine — `apply_etb_replacements` holds `&mut GameState` exclusively. `find_applicable` is called immediately above and the loop body is synchronous. The "race" comment is wrong (single-threaded `&mut` code cannot race), and the `EntersTapped` default sentinel is unreachable. The double-iteration (`find` + `any`) is also wasted work.

**Fix**: Restructure to a let-else early-continue:
```rust
let Some((modification, replacement_source)) = state
    .replacement_effects
    .iter()
    .find(|e| e.id == id)
    .map(|e| (e.modification.clone(), e.source))
else {
    continue;
};
```

Removes the fabricated default and the second `state.replacement_effects.iter()` scan. Functionally identical at runtime; cleaner code; removes the misleading "race" comment.

#### Finding E2: `bind_object_filter` doesn't handle `OwnedByOpponentsOf` in WouldEnterBattlefield arm (LOW)

**Severity**: LOW
**File**: `crates/engine/src/rules/replacement.rs:1793-1797` (the WouldEnterBattlefield arm calling `bind_object_filter`)

**Issue**: The new arm `ReplacementTrigger::WouldEnterBattlefield { filter } => bind_object_filter(filter, controller)` only handles `ObjectFilter::ControlledBy(0)` and `CreatureControlledBy(0)` placeholders. The symmetric `WouldChangeZone` arm at lines 1777-1786 has a dedicated pattern arm for `OwnedByOpponentsOf(_)` (Leyline of the Void pattern, MR-M8-09). If a future card uses non-self `WouldEnterBattlefield { filter: OwnedByOpponentsOf(PlayerId(0)) }`, the placeholder would leak through registration — `OwnedByOpponentsOf(PlayerId(0))` means "owned by an opponent of PlayerId(0)" which everyone except player 0 satisfies. Wrong game state.

**Engine state**: no in-scope card hits this. Master Biomancer uses `CreatureControlledBy` (handled). Ingenious Prodigy uses `Any` (no rebind needed for is_self). Documented OOS seeds (OOS-EWC-2 Golgari Grave-Troll, OOS-EWC-3 Dragonstorm Globe) don't use OwnedByOpponentsOf either.

**Fix**: Extend `bind_object_filter` (lines 505-514) to also rebind `OwnedByOpponentsOf(PlayerId(0))` → `OwnedByOpponentsOf(controller)`. ~3 lines. Alternatively, file as a sub-bullet on OOS-EWC-3 since it touches the same `ObjectFilter` rebind family.

#### Finding E3 + E4 + C1 + C2: Doc-comment line-number drift (LOW × 4)

**Severity**: LOW
**Files**:
- `crates/engine/src/state/replacement_effect.rs:138-143` (E3 — variant doc)
- `crates/engine/src/rules/replacement.rs:1509-1521` (E4 — inline comment in `emit_etb_modification`)
- `crates/engine/src/cards/defs/master_biomancer.rs:14-19, 40-47` (C1 — load-bearing exclude_self argument)
- `crates/engine/src/cards/defs/ingenious_prodigy.rs:13-18` (C2 — X resolution flow)

**Issue**: Comments reference `resolution.rs:546`, `resolution.rs:1606-1627`, `replacement.rs:505`, `replacement.rs:437`, `effects/mod.rs:6140`. These line numbers were correct at commit time but will drift on every refactor. The PB-LKI-Power review filed identical findings (E3) and the PB-TS / PB-LKI-CC reviews flagged the same anti-pattern.

**Fix**: Drop all explicit line numbers across the 4 sites. Replace with function-name + module references:
- "set during permanent-spell resolution before ETB processing" (resolution.rs:546)
- "`register_permanent_replacement_abilities` runs AFTER `apply_etb_replacements` for the same ETB" (resolution.rs:1606-1627)
- "bound by `bind_object_filter` at registration" (replacement.rs:505)
- "performed in `object_matches_filter`" (replacement.rs:437)
- "via `calculate_characteristics`" (effects/mod.rs:6140)

The load-bearing semantic claim in master_biomancer.rs C1 must be preserved — only the line numbers should be dropped.

#### Finding T1: X=0 absence test could be slightly stronger (LOW)

**Severity**: LOW
**File**: `crates/engine/tests/primitive_pb_ewc.rs:408-461` (test_ingenious_prodigy_x_zero_no_counters)

**Issue**: The assertion `o.counters.get(&PlusOnePlusOne).copied().unwrap_or(0) == 0` returns 0 for both "key absent" and "key present with value 0". The current implementation never inserts a 0-count counter (the `if modified_count > 0` guard at replacement.rs:1534 ensures this), so the test passes. But a hypothetical regression that did `counters.insert(counter, 0)` would slip past. The `CounterAdded` event check IS discriminating — catches a `CounterAdded { count: 0 }` emit. The combination is sufficient for the realistic failure modes.

**Fix** (optional, non-blocking): Add `assert!(!o.counters.contains_key(&CounterType::PlusOnePlusOne))`. The current test is correct for the implementation; this is a robustness-against-regression upgrade.

## Plumbing trace table (chain verification per `feedback_verify_full_chain.md`)

| # | Site | Path | Status |
|---|------|------|--------|
| 1 | DSL variant migration | `state/replacement_effect.rs:140-143` (`Box<EffectAmount>`) | OK + doc comment present (E3 LOW for line refs) |
| 2 | Hash arm migration | `state/hash.rs:1986-1990` (delegates to EffectAmount::hash_into via Box auto-deref) | OK |
| 3 | HASH_SCHEMA_VERSION bump | `state/hash.rs:103-113` (17→18 + history entry) | OK |
| 4 | `EffectAmount` `Box` rationale | `state/replacement_effect.rs:138` ("avoid large_enum_variant clippy warning") | OK — recursive `Sum(Box, Box)` + `CardCount { filter: TargetFilter }` make `EffectAmount` large; existing `Sum` precedent uses Box |
| 5 | `resolve_amount` visibility | `effects/mod.rs:6125` (`pub(crate) fn`) | OK — needed for cross-module call from rules::replacement |
| 6 | `apply_etb_replacements` source threading | `replacement.rs:1416-1440` | OK semantically (E1 LOW for code structure) |
| 7 | `apply_self_etb_modification` source = new_id | `replacement.rs:1453-1467` | OK (self ETB source is the entering permanent) |
| 8 | `emit_etb_modification` signature gain `replacement_source` param | `replacement.rs:1478-1484` | OK |
| 9 | EffectContext build with `source = replacement_source.unwrap_or(new_id)` | `replacement.rs:1522-1523` | OK — defensive fallback for source: None replacements (existing test at replacement_effects.rs:2032 uses source: None) |
| 10 | `ctx.x_value = state.objects[source].x_value` | `replacement.rs:1524-1528` | OK — for self-ETB (Ingenious Prodigy) reads new_id.x_value (set at resolution.rs:546). For non-self (Master Biomancer) reads MB.x_value (always 0; MB is not X-cost) |
| 11 | `resolve_amount(state, &count, &ctx)` invocation | `replacement.rs:1529` | OK — clamped via `.max(0) as u32` |
| 12 | `apply_counter_replacement` consumes raw_count | `replacement.rs:1531-1533` (Doubling Season / Pir / Hardened Scales chain) | OK — pre-existing path; count==0 early-exit preserves CR 122.6 "one or more counters" semantics |
| 13 | Counter insertion gated on `modified_count > 0` | `replacement.rs:1534-1539` | OK |
| 14 | CounterAdded event gated on `modified_count > 0` | `replacement.rs:1546-1552` | OK |
| 15 | ReplacementEffectApplied event (only for global, effect_id Some) | `replacement.rs:1540-1545` | OK |
| 16 | `WouldEnterBattlefield` filter rebind | `replacement.rs:1793-1797` | OK (E2 LOW for OwnedByOpponentsOf gap) |
| 17 | `bind_object_filter` handles `CreatureControlledBy(0)` | `replacement.rs:505-514` | OK |
| 18 | `object_matches_filter` evaluates `CreatureControlledBy` via `calculate_characteristics` (CR 613.1d) | `replacement.rs:437-447` | OK — layer-resolved creature-type check + controller equality |
| 19 | Resolution timing for MB exclude_self | resolution.rs:1606 (apply_self) → :1615 (apply_etb_replacements) → :1621 (register) | OK — verified across 8 ETB call sites in resolution.rs + lands.rs; MB's effect is not registered when MB itself enters |
| 20 | Card def 1: Master Biomancer (non-self, PowerOf(Source)) | `cards/defs/master_biomancer.rs:48-58` | OK — oracle match (counter half) |
| 21 | Card def 2: Ingenious Prodigy (self-ETB, XValue) | `cards/defs/ingenious_prodigy.rs:37-47` | OK — oracle match, DEVIATION comment removed |
| 22 | Test (a) base case (printed power 2) | `primitive_pb_ewc.rs:181-261` | OK — also verifies filter rebind via state.replacement_effects scan |
| 23 | Test (b) pumped case (4/6 MB via counters) | `:269-331` | OK — discriminating against printed-only |
| 24 | Test (c) Ingenious Prodigy X=5 (replacement, not trigger) | `:339-384` | OK |
| 25 | Test (d) HASH sentinel `== 18u8` | `:392-401` | OK |
| 26 | Test (e) X=0 zero-suppression | `:408-461` | OK (T1 LOW for stronger absence assertion) |
| 27 | Legacy test `test_x_cost_etb_counters_ingenious_prodigy` | `x_cost_spells.rs:269-342` | OK — still passes under new code path; second pass_all (line 309) is no-op now (no trigger to resolve) but harmless |
| 28 | Sentinel sweep (12 sites) | tests/*.rs grep for `HASH_SCHEMA_VERSION, 18` | OK — 12 files updated, no stale `17u8` |
| 29 | Migrated `replacement_effects.rs` call sites (3) | `tests/replacement_effects.rs:83,2037,3243` | OK — all use `Box::new(EffectAmount::Fixed(N))` |

## CR Coverage Check

| CR Rule | Verbatim from MCP? | Implemented? | Tested? | Notes |
|---------|-------------------|-------------|---------|-------|
| 614.1 (replacement effects are continuous, shield-like, not on stack) | Yes — verified | Yes (pre-existing) | Yes (test a/b/c — counters present immediately, not after pass_all twice) | Pre-existing infrastructure; PB-EWC preserves invariant |
| 614.1c (enters tapped / enters with counters / etc.) | Yes — verified | Yes | Yes (tests a-e) | Variant `EntersWithCounters` is the canonical 614.1c-with-counters case |
| 614.12 (replacement effects modifying ETB check characteristics "as it would exist on the battlefield") | Yes — verified | Yes (live `calculate_characteristics`) | Yes (test b pumped MB) | Critical: replacement source's layer-resolved power, not LKI |
| 614.15 (self-replacements first) | Yes — verified | Yes (pre-existing: apply_self_etb_from_definition before apply_etb_replacements) | Indirect via replacement_effects.rs:3186 (test_etb_self_and_global_replacement_both_apply) | Pre-existing; unchanged |
| 107.3 (X = value chosen at cast time) | Yes — verified | Yes (`ctx.x_value` from `obj.x_value`) | Yes (test c X=5, test e X=0) | Critical for Ingenious Prodigy |
| 122.6 (counters being put on an object — covers ETB) | Yes — verified | Yes (`apply_counter_replacement` called with raw_count) | Indirect — Doubling Season / Hardened Scales integration exists from PB-CD | Pre-existing PB-CD; PB-EWC preserves |
| 400.7 (zone change creates new object) | Yes — verified | N/A for PB-EWC (replacement applies BEFORE ETB completes) | N/A | The replacement source is the still-alive MB; entering object is post-zone-change new_id; both stable during evaluation |
| 613.1d (layer-resolved types for replacement applicability) | Yes — verified | Yes (`CreatureControlledBy` uses `calculate_characteristics` for type check) | Indirect via test a (Elvish Mystic IS a creature; the filter type-check fires) | Pre-existing PB-CD; PB-EWC preserves |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Master Biomancer | Yes (counter half) | 1 (type-grant half — OOS-EWC-1 filed, clearly labeled in def) | Yes for counter half | Test (a)+(b) verify live + pumped power flow; filter rebind verified via state.replacement_effects scan |
| Ingenious Prodigy | Yes | 0 (DEVIATION comment for upkeep "may" preserved — pre-existing PB-27 limitation, NOT PB-EWC) | Yes | Test (c)+(e) cover X=5 and X=0; legacy x_cost_spells.rs:269 still passes |

## Architecture invariant compliance

| Invariant | Status | Notes |
|-----------|--------|-------|
| 1. Engine is a pure library (no IO/network/async) | OK | All changes are pure state transitions |
| 2. Game state is immutable (im-rs persistent structures) | OK | `Box<EffectAmount>` is a Clone-able variant; OrdMap counters unchanged |
| 3. All player actions are Commands | OK | No new Command variants |
| 4. All state changes are Events | OK | CounterAdded + ReplacementEffectApplied unchanged (only emission gates) |
| 5. Multiplayer-first | OK | `bind_object_filter` rebind works for arbitrary PlayerId (not 1v1 special-case) |
| 6. Commander-first | OK | No commander-specific changes |
| 7. Hidden information enforced | OK | Counter placement is public information per CR |
| 8. Tests cite their rules source | OK | All 5 new tests cite specific CR rules (614.1c, 614.12, 107.3m, 122.6, 613.1g) |
| 9. Every card needs a CardDefinition | OK | Both cards are full card defs registered in `all_cards()` |

## Out-of-scope seeds verified

| Seed | Filed? | Card | Description | Yield |
|------|--------|------|-------------|-------|
| OOS-EWC-1 | Yes (pb-retriage-CC.md:687-712) | Master Biomancer (type-grant) | `EntersAsAdditionalType { subtype: SubType }` modification | 1 |
| OOS-EWC-2 | Yes (:714-730) | Golgari Grave-Troll | Self-ETB with `CardCount { Graveyard, Controller, CreatureCard }` | 1 (pure card-authoring follow-up) |
| OOS-EWC-3 | Yes (:732-750) | Dragonstorm Globe | Non-self ETB with subtype receiver filter; needs `ObjectFilter::CreatureControlledByOfSubtype` or generalization | 1 (engine work) |

3 seeds filed matching the plan's "3 OOS seeds expected" claim. Plan also mentioned `TargetFilter.exclude_self` (Éomer) — that's filed elsewhere as a separate seed and is not within PB-EWC scope.

## Summary

PB-EWC ships a clean migration from static `u32` to dynamic `Box<EffectAmount>` for the `EntersWithCounters` replacement modification. The dispatch chain is fully plumbed (29 sites verified), both card defs match oracle text, all tests are discriminating, and the HASH bump + sentinel sweep is consistent (12 sites). No HIGH or MEDIUM findings.

The 7 LOW findings break down as: 1 code-structure (E1, defensive default + race-comment), 1 latent rebind gap (E2, OwnedByOpponentsOf), 4 doc-comment line-number drift (E3 + E4 + C1 + C2, symmetric to PB-LKI-Power E3), and 1 test-robustness nit (T1, X=0 absence check could be strengthened). None affect game state for in-scope cards.

The coordinator may ship as-is (LOWs only) OR apply a brief fix-phase pass covering E1 + E3-E4 + C1-C2 (all comment hygiene + one structural cleanup, ~30 lines total). E2 (OwnedByOpponentsOf gap) is the only LOW with a future-correctness implication; recommend filing as a tracked seed on the existing OOS-EWC-3 since both touch ObjectFilter rebind.

**Verdict**: **PASS-WITH-NITS** — 0 HIGH, 0 MEDIUM, 7 LOW. Branch is correctness-clean for in-scope cards (Master Biomancer counter half + Ingenious Prodigy full ETB) and ready for signal-ready on ESM task scutemob-20 once the coordinator decides the LOW disposition.

## Resolution (2026-05-14, worker)

LOW findings dispositioned per acceptance criterion #6:

- **E1 (defensive-default + race-comment)**: **RESOLVED inline** at
  `replacement.rs:1410-1435`. Restructured to a `let-else { continue; }`
  pattern. The `EntersTapped` fabricated default and the second
  `state.replacement_effects.iter().any(...)` scan are gone. Tests pass
  (2754).
- **E2 (`OwnedByOpponentsOf` rebind gap for `WouldEnterBattlefield`)**:
  **ROUTED to OOS-EWC-3** (`memory/primitives/pb-retriage-CC.md`). The
  sub-gap is documented alongside the Dragonstorm Globe subtype-receiver
  gap since both touch the same `ObjectFilter` rebind family. No
  in-scope card is affected.
- **E3 (`replacement_effect.rs` variant doc — line numbers)**: **RESOLVED
  inline**. Dropped `resolution.rs:546` reference; kept function-name
  reference ("during permanent-spell resolution before ETB processing").
- **E4 (`replacement.rs` inline comment — line numbers)**: **RESOLVED
  inline**. Dropped `effects/mod.rs:6140` and `resolution.rs:546`;
  preserved function-name references (`resolve_amount`,
  `calculate_characteristics`). Also dropped the matching line number
  from the `apply_self_etb_modification` docstring.
- **C1 (`master_biomancer.rs` — line numbers)**: **RESOLVED inline**.
  Dropped `resolution.rs:1606-1627`, `replacement.rs:505`,
  `replacement.rs:437`, and `effects/mod.rs:6140` from the header and
  inline comments; preserved the load-bearing semantic argument and the
  function-name references.
- **C2 (`ingenious_prodigy.rs` — line numbers)**: **RESOLVED inline**.
  Dropped `resolution.rs:546` and `resolution.rs:1606` from the header
  X-resolution paragraph.
- **T1 (X=0 absence test — `unwrap_or(0)` cannot discriminate absent vs
  zero)**: **RESOLVED inline**. Added an explicit
  `assert!(!counters.contains_key(&PlusOnePlusOne))` discriminating
  check, preserved the existing `unwrap_or(0) == 0` assertion and the
  `CounterAdded` event check. Test now catches a hypothetical regression
  that did `counters.insert(PlusOnePlusOne, 0)`.

Post-fix verification:
- `cargo test --workspace --lib --tests` — 2754 passing (no regression).
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt --all -- --check` clean.
