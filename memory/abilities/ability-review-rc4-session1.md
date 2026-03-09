# RC-4 Session 1 Review: Designations Bitfield Migration

**Review Status**: REVIEWED (2026-03-09)
**Reviewer**: milestone-reviewer (Opus)
**Scope**: Migration of 8 boolean designation fields on `GameObject` into a `Designations` bitflags type

---

## Files Modified

| File | Purpose |
|------|---------|
| `crates/engine/Cargo.toml` | Added `bitflags = { version = "2", features = ["serde"] }` dependency |
| `crates/engine/src/state/game_object.rs` | Defined `Designations` bitflags type (u16), added `designations` field, removed 8 bools |
| `crates/engine/src/state/hash.rs` | Hash `designations.bits() as u32` instead of 8 individual bools |
| `crates/engine/src/state/builder.rs` | Use `Designations::default()` in object construction |
| `crates/engine/src/state/mod.rs` | Use `Designations::default()` in all 3 zone-change sites |
| `crates/engine/src/effects/mod.rs` | Use `Designations::default()` in token creation; `.insert()`/`.remove()`/`.contains()` at read/write sites |
| `crates/engine/src/rules/casting.rs` | `.contains(Designations::FORETOLD)` |
| `crates/engine/src/rules/foretell.rs` | `.insert(Designations::FORETOLD)` |
| `crates/engine/src/rules/engine.rs` | `.remove(Designations::ECHO_PENDING)` |
| `crates/engine/src/rules/turn_actions.rs` | `.contains()`/`.remove()` for SUSPENDED, ECHO_PENDING, SADDLED |
| `crates/engine/src/rules/combat.rs` | `.contains(Designations::SUSPECTED)` for can't-block check |
| `crates/engine/src/rules/abilities.rs` | `.contains(Designations::RENOWNED)` |
| `crates/engine/src/rules/resolution.rs` | `.insert()` for BESTOWED, ECHO_PENDING, SADDLED, RENOWNED; `default()` for token creation |
| `crates/engine/src/rules/lands.rs` | `.insert(Designations::ECHO_PENDING)` |
| `crates/engine/src/rules/suspend.rs` | `.insert(Designations::SUSPENDED)` |
| `crates/engine/src/rules/sba.rs` | `.contains()`/`.remove()` for BESTOWED, RECONFIGURED |
| `crates/engine/src/rules/layers.rs` | `.contains(Designations::SUSPECTED)` and `.contains(Designations::RECONFIGURED)` |
| `crates/engine/src/testing/replay_harness.rs` | `.insert(Designations::SUSPENDED)` and `.contains(Designations::FORETOLD)` |
| `crates/engine/src/lib.rs` | Export `Designations` |
| `crates/engine/tests/bestow.rs` | All reads/writes migrated |
| `crates/engine/tests/combat.rs` | All reads/writes migrated |
| `crates/engine/tests/commander_damage.rs` | Object construction uses `Designations::default()` |
| `crates/engine/tests/echo.rs` | All reads/writes migrated |
| `crates/engine/tests/foretell.rs` | All reads/writes migrated |
| `crates/engine/tests/ninjutsu.rs` | All reads/writes migrated |
| `crates/engine/tests/protection.rs` | All reads/writes migrated |
| `crates/engine/tests/reconfigure.rs` | All reads/writes migrated |
| `crates/engine/tests/renown.rs` | All reads/writes migrated |
| `crates/engine/tests/saddle.rs` | All reads/writes migrated |
| `crates/engine/tests/snapshot_perf.rs` | Object construction uses `Designations::default()` |
| `crates/engine/tests/suspect.rs` | All reads/writes migrated |
| `crates/engine/tests/suspend.rs` | All reads/writes migrated |
| `crates/engine/tests/zone_integrity.rs` | Object construction uses `Designations::default()` |

---

## Correctness Checklist

- [x] `Designations` bitflags type has correct derive macros (Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)
- [x] All 8 flags have unique, non-overlapping bit values (1<<0 through 1<<7)
- [x] Every read site correctly uses `.contains(Designations::FLAG)`
- [x] Every write site correctly uses `.insert(Designations::FLAG)` or `.remove(Designations::FLAG)`
- [x] hash.rs hashes the `designations` field (as u32 via `.bits()`)
- [x] builder.rs defaults to `Designations::default()` (all zero / all false)
- [x] All 8 old boolean fields fully removed from GameObject struct
- [x] Token creation in effects/mod.rs uses `Designations::default()`
- [x] All 3 move_object_to_zone sites use `Designations::default()` (CR 400.7 reset)
- [x] All test files updated
- [x] `Designations::default()` value is `0` (all flags clear)
- [x] Consistent `.contains()`/`.insert()`/`.remove()` pattern used everywhere
- [x] `Designations` exported from `lib.rs`
- [x] `bitflags` dependency has `serde` feature enabled

---

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| RC4-S1-01 | **LOW** | `layers.rs:69` | **Stale comment references old field name.** Comment says `obj.is_suspected` but should reference `obj.designations.contains(Designations::SUSPECTED)`. The actual code on line 71 is correct. Not a correctness issue, but could mislead future developers. **Fix:** Update comment on line 69 to match the new API. | OPEN |
| RC4-S1-02 | **LOW** | `cards/helpers.rs` | **`Designations` not exported from helpers.rs prelude.** The type consolidation plan (Session 7) and conventions.md both state that `Designations` should be exported from helpers.rs for card definition files. Currently only exported from `lib.rs`. No card definitions currently need it, so no compilation failure, but future card defs that manipulate designations would need a manual import. **Fix:** Add `pub use crate::state::game_object::Designations;` to helpers.rs. | OPEN |

---

## Verification Results

- **1934 tests passing** (all pass)
- **clippy clean** (no warnings with `-D warnings`)
- **No stale references** to old boolean field names in code (only one stale comment in layers.rs)
- **Zone-change behavior preserved**: All 3 zone-change constructors reset designations to default (CR 400.7)
- **Hash correctness**: Single `u32` hash of `bits()` replaces 8 individual `bool` hashes. The hash value WILL differ from pre-migration hashes (different byte layout), but this is acceptable since the hash contract requires consistency within a single game session, not across engine versions.

---

## Summary

The RC-4 Designations bitfield migration is **clean and correct**. The refactoring is purely mechanical: 8 boolean fields replaced with a single `u16` bitflags type, all read/write sites consistently updated to use `.contains()`/`.insert()`/`.remove()`. Zone-change resets (CR 400.7) are properly handled via `Designations::default()`. Only 2 LOW findings (stale comment, missing helpers.rs export) -- no fix phase required. These can be addressed opportunistically or during Session 7 (Memory & Documentation Updates).
