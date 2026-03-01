# Ability Review: Partner With

**Date**: 2026-02-28
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 702.124 (especially 702.124j)
**Files reviewed**:
- `crates/engine/src/state/types.rs` (lines 197-213)
- `crates/engine/src/state/hash.rs` (lines 452-456, 1026-1076, 1345-1355, 2181-2194)
- `crates/engine/src/state/stack.rs` (lines 374-399)
- `crates/engine/src/state/stubs.rs` (lines 217-229)
- `crates/engine/src/cards/card_definition.rs` (lines 689-693)
- `crates/engine/src/effects/mod.rs` (lines 2683-2688)
- `crates/engine/src/rules/commander.rs` (lines 459-549)
- `crates/engine/src/rules/abilities.rs` (lines 984-1043, 2099-2108)
- `crates/engine/src/rules/resolution.rs` (lines 1560-1625, 1732)
- `tools/replay-viewer/src/view_model.rs` (lines 470-471, 678)
- `crates/engine/tests/partner_with.rs` (all 719 lines)

## Verdict: needs-fix

One HIGH finding: the new `has_name` field on `TargetFilter` is not included in the
`HashInto for TargetFilter` implementation, which violates the architecture invariant
that all fields must be hashed for deterministic state hashing. One MEDIUM finding:
Panharmonicon-style trigger doublers will not double Partner With ETB triggers due to
the trigger event type used, which is a CR correctness gap for the Panharmonicon
interaction. Three LOW findings for missing test coverage and event granularity.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `hash.rs:2194` | **Missing hash for `TargetFilter.has_name`.** The new `has_name: Option<String>` field is not hashed, breaking deterministic state comparison. **Fix:** Add `self.has_name.hash_into(hasher);` after line 2193. |
| 2 | MEDIUM | `abilities.rs:1013` | **Panharmonicon does not double Partner With ETB triggers.** Partner With triggers use `SelfEntersBattlefield` but the doubler checks for `AnyPermanentEntersBattlefield` only. **Fix:** Update `doubler_applies_to_trigger` to also match `SelfEntersBattlefield` when the entering object is an artifact/creature, OR document as a known interaction gap. |
| 3 | LOW | `effects/mod.rs:2684` | **No test for `has_name` in `matches_filter`.** The field works correctly but has zero test coverage. **Fix:** Add a unit test in `crates/engine/tests/` that creates a `TargetFilter` with `has_name` set and verifies `matches_filter` accepts/rejects objects by name. |
| 4 | LOW | `resolution.rs:1590-1594` | **No `CardRevealed` event emitted.** CR 702.124j says "reveal it" but the implementation silently moves the card to hand without a reveal event. No `CardRevealed` event variant exists in the engine. **Fix:** Either add a `GameEvent::CardRevealed` variant and emit it before the zone move, or document as a known event-granularity gap. |
| 5 | LOW | `partner_with.rs` (test file) | **Missing PartnerWith-specific tests for CR 702.124c/d.** No tests verify combined color identity or independent tax tracking specifically for PartnerWith commanders (as opposed to plain Partner). These behaviors are covered by existing plain Partner tests in `commander.rs`, so the risk is minimal. **Fix:** Optionally add `test_partner_with_independent_tax` and `test_partner_with_combined_color_identity` tests, or document that these are covered by existing Partner tests. |

### Finding Details

#### Finding 1: Missing hash for `TargetFilter.has_name`

**Severity**: HIGH
**File**: `/home/airbaggie/scutemob/crates/engine/src/state/hash.rs:2194`
**Architecture Invariant**: "All fields must be hashed" (CLAUDE.md hash coverage requirement)
**Issue**: The `HashInto for TargetFilter` implementation at line 2181 hashes all fields
of `TargetFilter` except the newly added `has_name: Option<String>`. The impl ends at
line 2194 with `self.has_subtype.hash_into(hasher);` and then the closing brace --
`has_name` is completely absent. This means two `TargetFilter` values that differ only
in their `has_name` field will produce identical hashes, breaking deterministic state
comparison.

The existing fields are hashed in order: `max_power`, `min_power`, `has_card_type`,
`has_keywords`, `colors`, `exclude_colors`, `non_creature`, `non_land`, `basic`,
`controller`, `has_subtype`. The new `has_name` is missing from this list.

**Fix**: Add `self.has_name.hash_into(hasher);` after `self.has_subtype.hash_into(hasher);`
at line 2193 in the `HashInto for TargetFilter` impl. The `Option<String>` impl already
exists (line 112 of hash.rs), so this is a one-line fix.

#### Finding 2: Panharmonicon does not double Partner With ETB triggers

**Severity**: MEDIUM
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:1013` (trigger generation) and `/home/airbaggie/scutemob/crates/engine/src/rules/abilities.rs:2217` (doubler filter)
**CR Rule**: 702.124j -- "When this permanent enters, target player may search..." is an
ETB triggered ability. Panharmonicon says "If an artifact or creature entering the
battlefield causes a triggered ability of a permanent you control to trigger, that
ability triggers an additional time."
**Issue**: The Partner With ETB trigger is generated in `check_triggers` as a special-case
`PendingTrigger` with `triggering_event: Some(TriggerEvent::SelfEntersBattlefield)` at
line 1013. However, `doubler_applies_to_trigger` at line 2219 only checks for
`TriggerEvent::AnyPermanentEntersBattlefield`. Since the Partner With trigger uses
`SelfEntersBattlefield`, the doubler never matches, and Panharmonicon will not double it.

This is the same pattern used by Hideaway triggers (line 976: `SelfEntersBattlefield`)
and Exploit triggers (line 914: `SelfEntersBattlefield`), so this is a systemic issue
affecting all special-case ETB keyword triggers, not just Partner With. The Panharmonicon
doubler was designed to work with generic triggered abilities (those going through
`collect_triggers_for_event`), not with the special-case keyword trigger path.

**Fix**: Either:
(a) Update `doubler_applies_to_trigger` to also match `SelfEntersBattlefield` when the
    entering object is an artifact or creature (broader fix affecting all keyword ETBs), or
(b) Document as a known interaction gap and defer to a future batch that addresses
    Panharmonicon + keyword-ETB interactions holistically.

Option (b) is recommended since this affects multiple abilities, not just Partner With.

#### Finding 3: No test for `has_name` in `matches_filter`

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/effects/mod.rs:2684`
**Issue**: The `has_name` field was added to `TargetFilter` and the `matches_filter`
function was updated to check it (line 2684), but no test exercises this path. The
Partner With resolution uses a direct object search (line 1583 of resolution.rs), so
`matches_filter` with `has_name` is never called by the current implementation. The
field exists for future use (e.g., card definitions that use `SearchLibrary` with a name
filter), but without test coverage it could regress silently.
**Fix**: Add a test that constructs a `TargetFilter` with `has_name: Some("Specific Card".to_string())`
and verifies that `matches_filter` returns `true` for an object named "Specific Card"
and `false` for an object with a different name.

#### Finding 4: No `CardRevealed` event emitted

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/src/rules/resolution.rs:1590-1594`
**CR Rule**: 702.124j -- "reveal it, put it into their hand, then shuffle."
**Issue**: The CR text explicitly says the found card must be revealed before being put
into the hand. The implementation moves the card directly to hand without emitting any
reveal event. The engine does not have a `GameEvent::CardRevealed` variant at all. This
is consistent with other search effects in the engine (e.g., `SearchLibrary` in
effects/mod.rs also doesn't emit a reveal event), so this is a systemic gap rather than
a Partner With-specific bug.
**Fix**: When the engine adds a `CardRevealed` event (likely as part of a broader
hidden-information improvement), update the Partner With resolution to emit it. For now,
document as a known event-granularity gap.

#### Finding 5: Missing PartnerWith-specific tests for CR 702.124c/d

**Severity**: LOW
**File**: `/home/airbaggie/scutemob/crates/engine/tests/partner_with.rs`
**CR Rules**: 702.124c (combined color identity), 702.124d (independent tax/damage)
**Issue**: The plan requested tests 8 and 9 for independent tax tracking and combined
color identity with PartnerWith commanders. These tests were not implemented. However,
the underlying mechanics are tested by existing plain Partner tests
(`test_partner_commanders_separate_tax_tracking` at commander.rs:971 and
`test_partner_commanders_combined_color_identity` at commander.rs:1147). Since
`validate_partner_commanders` is the only PartnerWith-specific code for these features
(and it IS tested), the risk is minimal.
**Fix**: Optionally add these tests for completeness. The existing Partner tests already
validate the mechanics; PartnerWith tests would only confirm that the validation function
correctly allows PartnerWith pairs to use the same commander infrastructure.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 702.124a (partner abilities defined) | Yes (enum variant) | N/A | Definitional; PartnerWith variant exists |
| 702.124b (100-card deck) | Yes (existing) | Yes (existing) | Existing deck validation; not PartnerWith-specific |
| 702.124c (combined color identity) | Yes (existing) | Yes (plain Partner) | No PartnerWith-specific test (Finding 5) |
| 702.124d (independent tax/damage) | Yes (existing) | Yes (plain Partner) | No PartnerWith-specific test (Finding 5) |
| 702.124e (effect refers to commander) | N/A | N/A | Interactive choice; not automatable in deterministic engine |
| 702.124f (no mixing partner variants) | Yes | Yes | `test_partner_with_cannot_combine_with_plain_partner` |
| 702.124g (multiple partner abilities) | N/A | N/A | No cards with multiple partner abilities exist; future concern |
| 702.124j (deck construction part) | Yes | Yes | `test_partner_with_deck_validation_matching_pair`, `_mismatched_names`, `_one_has_keyword_other_has_none` |
| 702.124j (ETB trigger fires) | Yes | Yes | `test_partner_with_etb_trigger_fires`, `_generated_by_check_triggers` |
| 702.124j (search finds card) | Yes | Yes | `test_partner_with_trigger_finds_partner_in_library` |
| 702.124j (search finds nothing) | Yes | Yes | `test_partner_with_trigger_partner_not_in_library`, `_fires_when_partner_already_on_battlefield` |
| 702.124j (shuffle after search) | Yes | Partial | Shuffle is executed but no test asserts library order changed |
| 702.124j (reveal found card) | No | No | No `CardRevealed` event (Finding 4) |
| 702.124j (target player) | Partial | No | Deterministic: always targets controller; no test for targeting opponent |
| 702.124j (may search) | Partial | N/A | Deterministic: always searches (the "may" is treated as "do") |
