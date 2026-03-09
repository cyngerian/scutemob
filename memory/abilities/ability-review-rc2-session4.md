# RC-2 Session 4 Review: KeywordTrigger Consolidation

**Review Status**: REVIEWED (2026-03-09)
**Reviewer**: milestone-reviewer (Opus)

---

## Summary

Session 4 defined the `TriggerData` and `UpkeepCostKind` enums, added
`StackObjectKind::KeywordTrigger` (disc 64) and `PendingTriggerKind::KeywordTrigger`
(disc 45), and migrated 6 SOK + 6 PTK trigger variants into the unified pattern.
All 1900+ tests pass. No clippy warnings.

---

## Files Changed

| File | Change Type | Purpose |
|------|-------------|---------|
| `state/stack.rs` | Modified | Added `TriggerData`, `UpkeepCostKind`, `KeywordTrigger` SOK variant (disc 64); removed 6 SOK variants; migration comments for removed variants |
| `state/stubs.rs` | Modified | Added `KeywordTrigger` PTK variant; removed 6 PTK variants; migration comments |
| `state/hash.rs` | Modified | Added `HashInto` for `TriggerData` and `UpkeepCostKind`; PTK KeywordTrigger hash (disc 45); SOK KeywordTrigger hash (disc 64); retired disc 24-29 (PTK) and 33,37-41 (SOK) |
| `state/mod.rs` | Modified | Re-export `TriggerData`, `UpkeepCostKind` from `stack` |
| `lib.rs` | Modified | Re-export `TriggerData`, `UpkeepCostKind` |
| `rules/resolution.rs` | Modified | 6 specific KeywordTrigger match arms replacing old SOK variant arms; catch-all placeholder for future migration |
| `rules/abilities.rs` | Modified | `flush_pending_triggers`: PTK::KeywordTrigger -> SOK::KeywordTrigger passthrough |
| `rules/turn_actions.rs` | Modified | Trigger creation: 5 upkeep trigger sites now use PTK::KeywordTrigger |
| `tools/replay-viewer/src/view_model.rs` | Modified | KeywordTrigger display arm |
| `tools/tui/src/play/panels/stack_view.rs` | Modified | KeywordTrigger display arm |
| Test files (vanishing, fading, echo, cumulative_upkeep) | Modified | Updated to use new KeywordTrigger SOK patterns in assertions |

---

## Correctness Checklist

- [x] `TriggerData` and `UpkeepCostKind` have correct derives: Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize
- [x] All 6 old SOK variants fully removed (VanishingCounterTrigger, VanishingSacrificeTrigger, FadingTrigger, EchoTrigger, CumulativeUpkeepTrigger, ImpendingCounterTrigger)
- [x] All 6 old PTK variants fully removed (VanishingCounter, VanishingSacrifice, FadingUpkeep, EchoUpkeep, CumulativeUpkeep, ImpendingCounter)
- [x] No stale `StackObjectKind::` references to old variant names
- [x] No stale `PendingTriggerKind::` references to old variant names (only in comments)
- [x] Resolution dispatch handles all 6 via specific KeywordTrigger match arms (lines 2189, 2304, 2419, 2559, 2610, 3157)
- [x] Catch-all KeywordTrigger arm at line 7120 acts as fallback for future migrations
- [x] Countered/fizzled handling includes `KeywordTrigger { .. }` (line 7356)
- [x] `flush_pending_triggers` converts PTK::KeywordTrigger -> SOK::KeywordTrigger (line 6231)
- [x] Trigger creation in `turn_actions.rs` uses PTK::KeywordTrigger for all 5 trigger sites
- [x] Hash discriminant 64 for SOK KeywordTrigger is unique within SOK hash (no collision)
- [x] Hash discriminant 45 for PTK KeywordTrigger is unique within PTK hash (no collision)
- [x] `TriggerData` HashInto uses unique internal discriminants (0-3)
- [x] `UpkeepCostKind` HashInto uses unique internal discriminants (0-1)
- [x] Re-exports in `lib.rs` and `state/mod.rs` are correct
- [x] view_model.rs and stack_view.rs handle KeywordTrigger display
- [x] All tests pass (1900+)
- [x] No clippy warnings

---

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| RC2-S4-01 | **LOW** | `stubs.rs:342,348` | **Stale doc comments reference removed PTK variants.** `echo_cost` field doc says "Only meaningful when `kind == PendingTriggerKind::EchoUpkeep`" and `cumulative_upkeep_cost` says "PendingTriggerKind::CumulativeUpkeep" -- both variants no longer exist. Comments are misleading but harmless. **Fix:** Update doc comments to reference `PendingTriggerKind::KeywordTrigger { keyword: Echo, .. }` and `KeywordTrigger { keyword: CumulativeUpkeep, .. }` respectively. | OPEN |
| RC2-S4-02 | **LOW** | `stubs.rs:344-351` | **Dead fields: `echo_cost` and `cumulative_upkeep_cost` on PendingTrigger.** These fields are never set to `Some(...)` anywhere in the codebase -- the cost data is now carried inside `TriggerData::UpkeepCost` in the PTK::KeywordTrigger variant. The fields are set to `None` in ~100 PendingTrigger construction sites and hashed in hash.rs. They are pure dead weight -- removing them would eliminate ~100 `echo_cost: None` / `cumulative_upkeep_cost: None` lines and simplify PendingTrigger. **Fix:** Remove both fields from PendingTrigger, remove from hash.rs, and remove from all construction sites. This is a mechanical cleanup suitable for a later session. | OPEN |
| RC2-S4-03 | **LOW** | `state/mod.rs:138,148` | **Stale comments reference old trigger names.** Comments say "When an EchoTrigger resolves" and "When a CumulativeUpkeepTrigger resolves" -- these SOK variant names no longer exist. **Fix:** Update to "When a KeywordTrigger { keyword: Echo, .. } resolves" etc. | OPEN |
| RC2-S4-04 | **INFO** | `resolution.rs:7120` | **Catch-all KeywordTrigger placeholder is silent.** The catch-all arm at line 7120 for unmigrated-or-future KeywordTrigger combos just emits AbilityResolved without logging or warning. This is fine during migration (all 6 triggers have specific arms above), but if a new keyword+data combo is added without a corresponding resolution arm, it will silently do nothing. The comment correctly notes "Reaching this arm is a bug until migration is complete." No action needed now, but consider adding a debug_assert or log when migration is complete to catch accidental omissions. | OPEN |
| RC2-S4-05 | **INFO** | `stubs.rs:22` | **PendingTriggerKind correctly lost `Copy` derive.** The `KeywordTrigger` variant carries `ManaCost` (via `TriggerData::UpkeepCost -> UpkeepCostKind::Echo(ManaCost)`), which is `Vec<(ManaColor, u32)>` and not `Copy`. The `Clone` derive remains. Grep confirms no code was relying on `Copy` for `PendingTriggerKind` -- the `flush_pending_triggers` path uses `.clone()` on the `ref keyword` and `ref data` bindings, which is correct. | OPEN |

---

## Design Quality

The consolidation is well-structured:

1. **Extensibility**: `TriggerData` is open for extension -- new variants like `Simple` (for future keyword triggers with no data) and domain-specific data payloads allow the pattern to absorb more one-off triggers without new SOK/PTK variants.

2. **Hash stability**: Retired discriminants (24-29 for PTK, 33/37-41 for SOK) are clearly documented as "migrated to KeywordTrigger" with comments. The new discriminants (45 for PTK, 64 for SOK) are unique and non-colliding within their respective hash impl blocks. Cross-namespace collision (64u8 used in both KeywordAbility hash and SOK hash) is harmless because they are in separate `HashInto` impls with different type prefixes.

3. **Match arm ordering in resolution.rs**: The 6 specific KeywordTrigger arms precede the catch-all, leveraging Rust's first-match semantics correctly. Clippy does not warn about unreachable patterns because the catch-all covers keyword+data combos not yet migrated.

4. **PTK -> SOK passthrough in abilities.rs**: The conversion at line 6231 is a clean 1:1 mapping (`source_object: trigger.source, keyword: keyword.clone(), data: data.clone()`), preserving all data. No information loss.

---

## Recommendations

1. **Session 5 cleanup**: Remove dead fields `echo_cost` and `cumulative_upkeep_cost` from PendingTrigger as part of the next RC-2 session. This eliminates ~100 boilerplate lines.

2. **Future migration candidates**: The same pattern can absorb Recover (SOK 42), Graft (SOK 44), and other one-off upkeep/ETB triggers that carry simple data payloads.

3. **Catch-all arm**: Once RC-2 migration is complete, consider converting the catch-all to `unreachable!()` in debug builds or adding a `debug_assert!(false, "unhandled KeywordTrigger")` to catch accidental omissions.
