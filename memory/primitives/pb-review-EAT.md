# Primitive Batch Review: PB-EAT — ReplacementModification::EntersAsAdditionalType

**Date**: 2026-05-15
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 614.1c (entry modification), 205.3 (subtypes), 613.1d (Layer 4 distinction), 614.5 (single-application), 616.1 (multi-replacement ordering)
**Engine files reviewed**: `state/replacement_effect.rs`, `rules/replacement.rs`, `state/hash.rs`
**Card defs reviewed**: `master_biomancer.rs` (1 card)
**Tests reviewed**: `primitive_pb_eat.rs` (5 tests) + 14 PB canary files

## Verdict: PASS-WITH-NITS

Engine surface, resolver placement, hash discriminant, oracle alignment, and tests are all correct. CR 614.1c semantics are honored: subtype is pushed into `characteristics.subtypes` BEFORE every `PermanentEnteredBattlefield` emission site (verified at effects/mod.rs:4910→4932, 5147→5168, 5423→5439). Discriminant 22 does not collide (existing range is 0-21). OrdSet idempotency is structural. Two-replacement composition is correct — both modifications are commutative state writes and CR 614.5 applies per-effect, not per-source. One LOW nit on a stale interior canary message; no functional issues.

## Engine Change Findings

(None at HIGH or MEDIUM.)

## Card Definition Findings

(None.)

## Test Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `primitive_pb_xa.rs:92-95` | **Non-uniform canary message body.** The assertion was bumped to `21u8` but the message body still reads `expected 20u8`. Other 13 files got the full PB-EAT rewrite; this one only got the integer bumped. **Fix:** replace the message body with the standardized `"PB-EAT bumped HASH_SCHEMA_VERSION 20→21 ..."` text matching the other 13 files. |
| 2 | LOW | `primitive_pb_eat.rs:382-389` | **Test E does not assert subtype-count is exactly 1.** The test asserts `.contains(&SubType("Mutant"))` but `OrdSet` makes multi-entry structurally impossible, so the "idempotency" is enforced by type system not by assertion. Test still functions as a regression guard. **Fix (optional):** add `let n = initiate_obj.characteristics.subtypes.iter().filter(|s| s.0 == "Mutant").count(); assert_eq!(n, 1);` to make the idempotency assertion explicit and survive any future migration off `OrdSet`. |

### Finding Details

#### Finding 1: Stale canary message in primitive_pb_xa.rs

**File**: `crates/engine/tests/primitive_pb_xa.rs:93`
**Issue**: Message reads `"HASH_SCHEMA_VERSION sentinel: expected 20u8. If a PB bumped the version, update this sentinel to match the new value."` while the assert is `HASH_SCHEMA_VERSION, 21u8`. The other 13 PB canary files got the full standardized PB-EAT rewrite ("PB-EAT bumped HASH_SCHEMA_VERSION 20→21..."). 14/15 files uniform; one drift.
**Fix**: Rewrite the message body to match the other 13 files.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 614.1c | Yes | Yes | Test D verifies entry-time addition (functional). Test E verifies idempotent insert. |
| 205.3 (subtype additivity) | Yes | Yes | Test D verifies printed Elf/Druid preserved. |
| 613.1d (NOT Layer 4) | Yes (by construction) | Indirectly | Resolver writes to `characteristics.subtypes` not via continuous effect — would a Layer 4 regression pass? **YES (see Risk #1)**: a regression that wrote a Layer 4 type-add WOULD still cause `mystic_obj.characteristics.subtypes.contains(Mutant)` to be true after `calculate_characteristics`. However, the test reads the **stored** `obj.characteristics.subtypes` directly (not `calculate_characteristics`), which Layer 4 does not mutate — so a Layer-4-only regression would fail Test D. Acceptable. |
| 614.5 | Yes (OrdSet defense-in-depth + upstream `already_applied`) | Test E (idempotency) | |
| 616.1 | Yes | Test D (both replacements fire on same ETB) | Both modifications commutative; resolver iterates all applicable replacements. |

## Regression Discrimination Check (asked by coordinator)

- **Q: Would a regression writing via Layer 4 (continuous type-adding) pass these tests?**
  **A: No.** Test D reads `state.objects[mystic].characteristics.subtypes` directly. Layer 4 type additions live in `ContinuousEffect` entries and only manifest via `calculate_characteristics()` — the raw `obj.characteristics.subtypes` would still lack `Mutant`. Test D fails.

- **Q: Would a regression emitting `PermanentEnteredBattlefield` BEFORE pushing the subtype pass these tests?**
  **A: No.** Test D inspects post-ETB stored state; if push happened after the event, the push would still occur (modulo zone-change object identity bugs), but Test D would pass — **this is a gap**. However, the placement is structurally correct at all three call sites in `effects/mod.rs` (apply_etb_replacements is invoked, then push, then ETB event). A regression here would manifest as ETB-trigger-ordering bugs (e.g. Soul Warden gating on subtypes), not as Test D failure. Recommended (not blocking): add a test that registers a `WhenenverCreatureEntersBattlefield` trigger with a `creature_type == "Mutant"` filter against Master Biomancer and asserts the trigger fires for an Elvish Mystic ETB. Tracked as suggested follow-up; not a HIGH/MEDIUM gate.

## Risk Register Verification

- **Two replacements, same trigger filter (CR 616.1)**: `apply_etb_replacements` (replacement.rs:1411) iterates `find_applicable` results in registration order. Both modifications are commutative — counter insertion writes `obj.counters`, subtype insertion writes `obj.characteristics.subtypes`, no overlap. CR 614.5 forbids double-applying a single replacement effect, not two distinct replacements from the same source. Verified.
- **HASH discriminant 22 collision**: scanned `ReplacementModification` hash arms 0-21 (hash.rs:2008-2081). No collision. Adjacent enum impls (`ChosenColorRef`, `ReplacementManaSourceFilter`) live in separate `impl HashInto for` blocks with their own discriminant namespace.

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Master Biomancer | Yes | 0 (OOS-EWC-1 TODO removed) | Yes | Oracle verified via MCP: "...enters with a number of additional +1/+1 counters on it equal to this creature's power and as a Mutant in addition to its other types." Both halves now implemented. |

## Canary Sweep Verification

14 PB-canary files contain the standardized PB-EAT message (`"PB-EAT bumped HASH_SCHEMA_VERSION 20→21 ..."`); 1 file (`primitive_pb_xa.rs`) has the bumped integer but a stale interior message body. See Finding 1.

## Summary

PB-EAT is functionally correct, well-tested, and oracle-faithful. Discriminant assignment, hash schema bump, and serde additivity are all proper. Recommend resolving the single LOW canary drift; the test discrimination gap (PermanentEnteredBattlefield ordering) is a known minor coverage shortfall that can be addressed in a follow-up if desired.
