# W3 LOW S1 Review: Doc Comments, Dead Code, Rename, Helper Exports

**Review Status**: REVIEWED (2026-03-20)
**Reviewer**: milestone-reviewer (Opus)
**Scope**: Uncommitted changes on branch `w3-low-s1-doc-cleanup` vs `main`

## Summary

21 files changed, 46 insertions, 175 deletions (net -129 lines). Purely mechanical cleanup:
stale doc comments, dead field removal, one enum variant rename, two helper re-exports, one
clippy fix (mentioned in description but not present in diff -- may have been done separately).

**Overall assessment**: Clean mechanical work. No behavioral changes. Two minor rename
oversights in comments. Zero risk to correctness.

## Files Changed

| File | +/- | Purpose |
|------|-----|---------|
| `cards/card_definition.rs` | 4/4 | Doc: `CastSpell.squad_count` / `offspring_paid` -> `AdditionalCost::*` |
| `cards/helpers.rs` | 4/4 | Re-export `Designations` and `AdditionalCost` |
| `effects/mod.rs` | 6/10 | Doc: `gift_opponent` source; remove `echo_cost`/`cumulative_upkeep_cost` from 2 PendingTrigger sites |
| `rules/abilities.rs` | 0/66 | Remove `echo_cost`/`cumulative_upkeep_cost` from ~33 PendingTrigger construction sites |
| `rules/casting.rs` | 0/2 | Remove dead fields from 1 PendingTrigger site |
| `rules/events.rs` | 2/2 | Doc: `HideawayTrigger` -> `KeywordTrigger` (Hideaway) |
| `rules/layers.rs` | 2/2 | Doc: `obj.is_suspected` -> `obj.designations.contains(Designations::SUSPECTED)` |
| `rules/miracle.rs` | 0/2 | Remove dead fields from 1 PendingTrigger site |
| `rules/replacement.rs` | 0/6 | Remove dead fields from 3 PendingTrigger sites |
| `rules/resolution.rs` | 2/16 | Remove dead fields from 6 sites; rename `EvolveTrigger` -> `ETBEvolve` in match arm |
| `rules/turn_actions.rs` | 0/26 | Remove dead fields from 13 PendingTrigger sites |
| `state/builder.rs` | 10/10 | Doc: old SOK names -> `KeywordTrigger (X)` in 7 comment sites |
| `state/combat.rs` | 2/2 | Doc: `ProvokeTrigger` -> `KeywordTrigger` (Provoke) |
| `state/game_object.rs` | 8/8 | Doc: old SOK/field names -> new names in 4 sites |
| `state/hash.rs` | 2/6 | Remove 4 dead hash lines; rename `EvolveTrigger` -> `ETBEvolve` in match |
| `state/mod.rs` | 4/4 | Doc: `EchoTrigger` / `CumulativeUpkeepTrigger` -> `KeywordTrigger (X)` |
| `state/stack.rs` | 2/2 | Rename `EvolveTrigger` -> `ETBEvolve` in enum definition |
| `state/stubs.rs` | 4/14 | Remove `echo_cost` / `cumulative_upkeep_cost` fields; add tombstone comments |
| `state/types.rs` | 11/12 | Doc: 9 stale CastSpell field refs -> `AdditionalCost::*`; 3 old SOK refs -> `KeywordTrigger` |
| `tests/evolve.rs` | 4/4 | Rename `EvolveTrigger` -> `ETBEvolve` in 2 assert messages |
| `memory/workstream-state.md` | 2/2 | Claim W3 workstream |

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| W3S1-01 | **LOW** | `tests/evolve.rs:1061` | **Missed EvolveTrigger rename in assert message.** The rename from `EvolveTrigger` to `ETBEvolve` was applied to 2 of 3 assert message strings in `evolve.rs`. The third at line 1061 still reads `"CR 603.4: stack entry should be EvolveTrigger"`. Not a correctness issue (string literal only). **Fix:** Change to `"CR 603.4: stack entry should be ETBEvolve"`. | OPEN |
| W3S1-02 | **LOW** | `rules/abilities.rs:3955,4534` | **Stale SOK names in source comments.** Two code comments in `abilities.rs` still reference `MyriadTrigger` (line 3955) and `ModularTrigger` (line 4534) instead of `KeywordTrigger (Myriad)` / `KeywordTrigger (Modular)`. These are in-scope for this cleanup pass but were missed. **Fix:** Update both comments to use `KeywordTrigger (Myriad)` and `KeywordTrigger (Modular)`. | OPEN |
| W3S1-03 | **INFO** | `tests/*.rs` | **Stale SOK names in test file comments.** Approximately 40+ references to old SOK variant names (`HideawayTrigger`, `EchoTrigger`, `CumulativeUpkeepTrigger`, `ModularTrigger`, `ProvokeTrigger`) remain in test file comments and string literals across `echo.rs`, `hideaway.rs`, `provoke.rs`, `modular.rs`, `cumulative_upkeep.rs`. These are documentation-only and do not affect correctness, but are inconsistent with the source-code cleanup. Consider a follow-up pass over test files. | OPEN |
| W3S1-04 | **INFO** | `cards/card_definition.rs:692` | **Overlord clippy fix not in diff.** The description mentions removing a redundant `..Default::default()` in `overlord_of_the_hauntwoods.rs`, but this change is not present in the working tree diff. Either it was already committed separately or was not yet applied. No action needed -- just noting the discrepancy. | OPEN |

## Correctness Verification

| Check | Result |
|-------|--------|
| `cargo check --workspace` | PASS (clean) |
| `cargo clippy -- -D warnings` | PASS (clean) |
| Dead field removal complete (no reads of `echo_cost`/`cumulative_upkeep_cost` remain) | PASS |
| Rename consistent in all code paths (`EvolveTrigger` -> `ETBEvolve`) | PASS (enum def, hash, match arms, flush_pending_triggers, resolution.rs) |
| Hash discriminant preserved (32u8 for ETBEvolve) | PASS -- no hash-breaking change |
| Helper re-exports point to correct modules | PASS (`Designations` from `state::game_object`, `AdditionalCost` from `state::types`) |
| No behavioral changes introduced | PASS -- all changes are doc/dead-code/rename only |

## Notes

- This is a clean, low-risk cleanup session. All changes are mechanical and non-behavioral.
- The dead field removal (`echo_cost`, `cumulative_upkeep_cost`) is well-executed: fields removed from struct definition, hash impl, and all ~58 construction sites. Tombstone comments left in `stubs.rs` explaining why they were removed.
- The `EvolveTrigger` -> `ETBEvolve` rename follows the project's `ETB` prefix convention for ETB-related trigger data (consistent with other `TriggerData` variants).
- No fix phase needed -- only 2 LOWs and 2 INFOs. Address in the same session or opportunistically.
- Total findings: 0 HIGH, 0 MEDIUM, 2 LOW, 2 INFO.
