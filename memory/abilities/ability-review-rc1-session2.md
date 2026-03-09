# RC-1 Session 2 Review: CastSpell Sacrifice Field Consolidation

**Review Status**: REVIEWED (2026-03-09)
**Scope**: Migration of 4 sacrifice-related CastSpell fields (`bargain_sacrifice`, `emerge_sacrifice`, `casualty_sacrifice`, `devour_sacrifices`) into `AdditionalCost::Sacrifice(Vec<ObjectId>)`.
**Tests**: 1934 passing, 0 failing, clippy clean, workspace builds.

---

## Files Changed

| File | Purpose |
|------|---------|
| `crates/engine/src/state/types.rs` | Defined `AdditionalCost` enum with 12 variants |
| `crates/engine/src/rules/command.rs` | Removed 4 sacrifice fields from `CastSpell`; added `additional_costs: Vec<AdditionalCost>` |
| `crates/engine/src/state/stack.rs` | Removed `devour_sacrifices` from `StackObject`; added `additional_costs: Vec<AdditionalCost>` |
| `crates/engine/src/rules/casting.rs` | Extraction/disambiguation logic for sacrifice IDs from `additional_costs` |
| `crates/engine/src/rules/resolution.rs` | Devour resolution reads from `additional_costs` instead of `devour_sacrifices` |
| `crates/engine/src/state/hash.rs` | `HashInto for AdditionalCost` (12 variants, discriminant bytes); `additional_costs` hashed on StackObject |
| `crates/engine/src/state/builder.rs` | No `additional_costs` field in builder (not needed -- builder doesn't construct StackObjects) |
| `crates/engine/src/lib.rs` | Re-exports `AdditionalCost` |
| `crates/engine/src/testing/replay_harness.rs` | Translates bargain/emerge/casualty/devour name-based params into `AdditionalCost::Sacrifice(...)` |
| `crates/engine/src/rules/engine.rs` | Passes `additional_costs` through from Command to casting |
| `crates/engine/src/rules/copy.rs` | Copies get `additional_costs: vec![]` (correct -- sacrifice already happened) |
| `crates/simulator/src/random_bot.rs` | `additional_costs: vec![]` in all CastSpell constructions |
| `tools/tui/src/play/input.rs` | `additional_costs: vec![]` in CastSpell construction |
| ~95 test files | `additional_costs: vec![]` added to all CastSpell constructions; devour tests use `AdditionalCost::Sacrifice(...)` |

---

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| RC1-S2-01 | **MEDIUM** | `casting.rs:3415` | **Devour extraction heuristic is fragile.** The disambiguation logic uses `ids.len() > 1` as a proxy for "this is a devour sacrifice." If a future ability adds multi-target sacrifice via `AdditionalCost::Sacrifice(vec![id1, id2])` that is NOT devour (e.g., a theoretical "sacrifice two creatures" Bargain variant), it would be incorrectly extracted as devour. The `ids.len() > 1` check is not a stable discriminant. **Fix:** Check ONLY `chars.keywords.iter().any(\|kw\| matches!(kw, KeywordAbility::Devour(_)))` -- remove the `ids.len() > 1` disjunct. If the spell has Devour, take the Sacrifice vec; if not, skip it. This is safe because devour's sacrifice list is always extracted separately from bargain/casualty (which only take the first ID). | OPEN |
| RC1-S2-02 | **LOW** | `cards/helpers.rs` | **AdditionalCost not exported from helpers.rs.** `AdditionalCost` is exported from `lib.rs` but not from `crate::cards::helpers::*` which is the standard import used by card definition files (`use crate::cards::helpers::*`). Card authors who need to construct `AdditionalCost` values in card definitions won't find it in their usual prelude. Currently no card definitions need this (sacrifice costs are built in the harness/tests), but the consolidation plan specifies it should be in helpers.rs (Session 7 step 7). **Fix:** Add `pub use crate::state::types::AdditionalCost;` to `crates/engine/src/cards/helpers.rs`. | OPEN |
| RC1-S2-03 | **LOW** | `command.rs:217-225` | **Orphaned doc comment for removed `devour_sacrifices` field.** Lines 217-225 contain a `///` doc comment block describing the removed `devour_sacrifices` field. Since the field is gone (replaced by line 226 comment), these doc comments now incorrectly attach to the next field `modes_chosen`, giving it misleading documentation about devour sacrifice validation. **Fix:** Remove lines 217-225 (the `/// CR 702.82a:` through `/// - Not the card being cast (can't devour itself)` block). | OPEN |
| RC1-S2-04 | **LOW** | `types.rs:1010,1020,1039` | **Stale field references in KeywordAbility doc comments.** `Bargain` doc says "The sacrifice target is provided via `CastSpell.bargain_sacrifice`" (line 1010). `Emerge` doc references `CastSpell.emerge_sacrifice` (line 1020). `Casualty` doc references `CastSpell.casualty_sacrifice` (line 1039). These fields no longer exist. **Fix:** Update doc comments to reference `CastSpell.additional_costs` with `AdditionalCost::Sacrifice`. | OPEN |
| RC1-S2-05 | **LOW** | `legal_actions.rs:103` | **Stale reference in legal_actions.rs doc comment.** Line 103 references `bargain_sacrifice/emerge_sacrifice/etc` -- these field names no longer exist on CastSpell. **Fix:** Update to reference `additional_costs: vec![]`. | OPEN |
| RC1-S2-06 | **LOW** | `casting.rs:138-147` | **Bargain and Casualty both extract the same sacrifice ID.** `bargain_sacrifice` and `casualty_sacrifice` are both assigned `sacrifice_from_additional_costs` (the first ObjectId from the Sacrifice vec). For the common case this is correct (no card has both keywords), but if a custom card had both Bargain and Casualty, both would claim the same sacrifice and the bargain path would consume it at cast time, leaving nothing for casualty. The code comment at line 137 acknowledges this ("mutually exclusive in practice"). This is acceptable for now but should be documented as a known limitation. **Fix:** Add a defensive check: if the spell has BOTH Bargain and Casualty keywords, return an error. Or, once Session 3 is done, consider having separate `Sacrifice` entries for each ability. | OPEN |

---

## Correctness Assessment

### What's Correct

1. **AdditionalCost enum**: Properly derived with `Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`. All 12 variants defined per plan.
2. **HashInto implementation**: All 12 variants hashed with unique discriminant bytes (0-11). Vec entries length-prefixed. Consistent with existing hash patterns.
3. **CastSpell -> StackObject transfer**: `additional_costs` passed through at `casting.rs:3665`.
4. **Old field removal**: `bargain_sacrifice`, `emerge_sacrifice`, `casualty_sacrifice` removed from CastSpell struct. `devour_sacrifices` removed from both CastSpell and StackObject.
5. **Devour resolution**: Correctly reads from `stack_obj.additional_costs` (resolution.rs:1048-1058). Guards with `!devour_instances.is_empty()` so non-Devour spells skip the block.
6. **Bargain/Casualty cast-time processing**: Both correctly check keyword presence before consuming the sacrifice ID, silently skipping when the keyword is absent.
7. **Emerge disambiguation**: Correctly uses `alt_cost == Emerge` to separate emerge sacrifice from bargain/casualty.
8. **Replay harness**: Correctly translates name-based sacrifice parameters to `AdditionalCost::Sacrifice(vec![id])` for bargain, emerge, casualty, and devour.
9. **Copy system**: Copies correctly get `additional_costs: vec![]` (sacrifices don't propagate to copies).
10. **Serde**: `#[serde(default)]` on `additional_costs` fields ensures backward-compatible deserialization.

### What's Incomplete (Expected -- Session 3 scope)

The remaining ~16 CastSpell fields (assist_player, assist_amount, replicate_count, splice_cards, entwine_paid, escalate_modes, squad_count, offspring_paid, gift_opponent, mutate_target, mutate_on_top, escape_exile_cards, retrace_discard_land, jump_start_discard, collect_evidence_cards, fuse) are NOT yet migrated. This is correct per the plan -- Session 2 scope is sacrifice fields only.

---

## Summary

- **0 HIGH findings** -- no correctness issues that would produce wrong game states
- **1 MEDIUM finding** (RC1-S2-01) -- disambiguation heuristic for devour is fragile
- **5 LOW findings** -- stale doc comments, missing helpers.rs export, defensive edge case
- All 1934 tests pass; clippy clean; workspace builds
- The MEDIUM is theoretical (no current ability triggers it) but should be fixed before Session 3 to prevent propagation of the fragile pattern
