# RC-1 Session 3 Review: CastSpell Additional Cost Consolidation (Remaining Fields)

**Review Status**: REVIEWED (2026-03-09)
**Reviewer**: milestone-reviewer (Opus)
**Scope**: Migration of 16 remaining CastSpell additional-cost fields into `AdditionalCost` enum, removal of 9 mirrored StackObject fields.

---

## Files Modified

| File | Change Type | Purpose |
|------|-------------|---------|
| `state/types.rs` | Modified | Added `EscapeExile`, `CollectEvidenceExile`, `Squad` variants to `AdditionalCost` |
| `rules/command.rs` | Modified | Removed 16 fields from CastSpell, trimmed doc comments |
| `state/stack.rs` | Modified | Removed 9 mirrored fields from StackObject |
| `state/hash.rs` | Modified | Added `HashInto` for all 14 `AdditionalCost` variants; removed old field hashes from StackObject |
| `rules/casting.rs` | Modified | Extraction logic: extracts each cost type from `additional_costs` Vec into local variables |
| `rules/resolution.rs` | Modified | Reads squad/offspring/gift/mutate/fuse/entwine/escalate from `additional_costs` |
| `rules/copy.rs` | Modified | Filters `additional_costs` to propagate only Entwine/Fuse/EscalateModes to copies |
| `rules/engine.rs` | Modified | CastSpell handler passes `additional_costs`; removed `CastWithMutate` command variant |
| `testing/replay_harness.rs` | Modified | All action types now construct `additional_costs: vec![...]` |
| `state/builder.rs` | No change | N/A (additional_costs defaults via serde) |
| `crates/simulator/src/random_bot.rs` | Modified | CastWithMutate uses `AdditionalCost::Mutate`; CastSpell uses `additional_costs: vec![]` |
| `~95 test files` | Modified | All CastSpell construction sites updated |

---

## Correctness Assessment

### PASS: All 16 old CastSpell fields removed
Confirmed via grep: no `pub` field declarations for any of the 16 migrated fields remain on `Command::CastSpell`.

### PASS: All 9 old StackObject mirrored fields removed
`was_entwined`, `escalate_modes_paid`, `was_fused`, `squad_count`, `offspring_paid`, `gift_was_given`, `gift_opponent`, `mutate_target`, `mutate_on_top` — all confirmed absent from `StackObject` struct.

### PASS: AdditionalCost enum covers all 14 variants
`Sacrifice`, `Discard`, `EscapeExile`, `CollectEvidenceExile`, `Assist`, `Replicate`, `Squad`, `EscalateModes`, `Splice`, `Entwine`, `Fuse`, `Offspring`, `Gift`, `Mutate` — all present with correct field types.

### PASS: Hash discriminants are unique
Discriminant bytes 0-13 used, all unique: 0=Sacrifice, 1=Discard, 2=EscapeExile, 3=Assist, 4=Replicate, 5=EscalateModes, 6=Splice, 7=Entwine, 8=Fuse, 9=Offspring, 10=Gift, 11=Mutate, 12=Squad, 13=CollectEvidenceExile.

### PASS: additional_costs properly transferred CastSpell -> StackObject
In casting.rs, `additional_costs` is passed directly to the StackObject construction at ~line 3719.

### PASS: Extraction logic correct in casting.rs
Each cost type is extracted from the Vec using `find_map` or `any`. Retrace/JumpStart share `Discard` (mutually exclusive by alt_cost check). Bargain/Casualty/Devour share `Sacrifice` (disambiguated by keyword check + alt_cost). Escape and CollectEvidence have distinct variants.

### PASS: Resolution reads from additional_costs
Fuse (line 168), Entwine (line 287), EscalateModes (line 300-303), Gift (line 334), Squad (line 497), Offspring (line 502), Mutate (line 6692) — all correctly read from `stack_obj.additional_costs`.

### PASS: Copy propagation correct
copy.rs line 239-245: only Entwine, Fuse, EscalateModes propagate. All one-shot costs filtered out. CR 707.2 compliant.

### PASS: Replay harness updated
All action types (cast_spell_escape, cast_spell_collect_evidence, cast_spell_emerge, cast_spell_bargain, cast_spell_casualty, cast_spell_replicate, cast_with_mutate, cast_spell_entwine, cast_spell_escalate, cast_spell_offspring, cast_spell_squad, cast_spell_splice, cast_spell_fuse, cast_spell_gift) correctly construct `additional_costs` Vec.

### PASS: Simulator updated
`random_bot.rs` correctly uses `additional_costs: vec![]` for basic casts and `vec![AdditionalCost::Mutate { target, on_top: true }]` for mutate casts.

### PASS: CastWithMutate command variant removed
Mutate is now handled via `CastSpell` with `alt_cost: Some(AltCostKind::Mutate)` + `AdditionalCost::Mutate`.

### PASS: 1934 tests pass, clippy clean, workspace builds

---

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| RC1S3-01 | **LOW** | `state/types.rs:1056-1057` | **Stale doc comment references `CastSpell.assist_player` and `CastSpell.assist_amount`.** These fields no longer exist on CastSpell; they are now in `AdditionalCost::Assist`. The doc for `KeywordAbility::Assist` still references the old field names. **Fix:** Update doc to reference `AdditionalCost::Assist { player, amount }` in `additional_costs`. | OPEN |
| RC1S3-02 | **LOW** | `state/types.rs:1375` | **Stale doc comment references `CastSpell.gift_opponent`.** Field removed; now in `AdditionalCost::Gift { opponent }`. **Fix:** Update doc to reference `AdditionalCost::Gift` in `additional_costs`. | OPEN |
| RC1S3-03 | **LOW** | `cards/card_definition.rs:637` | **Stale doc references `CastSpell.squad_count` and `StackObject.squad_count`.** Both fields removed; now in `AdditionalCost::Squad { count }`. Same issue at line 673 for `CastSpell.offspring_paid`. **Fix:** Update docs to reference `AdditionalCost::Squad` and `AdditionalCost::Offspring` in `additional_costs`. | OPEN |
| RC1S3-04 | **LOW** | `state/game_object.rs:640,646` | **Stale doc references `StackObject.squad_count` and `StackObject.offspring_paid`.** These fields no longer exist on StackObject; they are read from `additional_costs` at resolution. **Fix:** Update docs to say "Extracted from `AdditionalCost::Squad`/`Offspring` in `stack_obj.additional_costs` at resolution." | OPEN |
| RC1S3-05 | **LOW** | `effects/mod.rs:91` | **Stale doc references `StackObject.gift_opponent`.** Field removed; now in `AdditionalCost::Gift`. **Fix:** Update doc to reference `AdditionalCost::Gift` extraction. | OPEN |
| RC1S3-06 | **LOW** | `state/types.rs:1106` | **Stale doc references `escalate_modes` in `Command::CastSpell`.** Field removed; now `AdditionalCost::EscalateModes { count }`. **Fix:** Update doc. | OPEN |
| RC1S3-07 | **INFO** | `rules/casting.rs:194-225` | **Sacrifice disambiguation relies on mutual exclusivity.** `AdditionalCost::Sacrifice(ids)` is consumed by Bargain, Casualty, and Devour code paths (disambiguated by keyword + alt_cost checks). If a future card has both Bargain and Casualty, the same sacrifice IDs would be used for both. The code documents this as "mutually exclusive in practice" which is correct for all existing and foreseeable MTG cards. No fix needed. | CLOSED |
| RC1S3-08 | **INFO** | `state/hash.rs:2354` | **Non-sequential discriminant numbering.** `CollectEvidenceExile` uses discriminant 13 and `Squad` uses 12 (added after the original 0-11 sequence). All discriminants are unique so this is correct; noted for reference. | CLOSED |

---

## Design Assessment

**Quality: HIGH.** The refactoring is clean, mechanical, and thorough. The 142-file changeset net-deletes ~10,700 lines while preserving all 1934 tests. Key design decisions are sound:

1. **Correct split of Squad vs Replicate.** The plan originally grouped these under `Replicate`, but the implementation correctly separates them because they have fundamentally different stack/resolution behaviors (Replicate copies spells, Squad creates ETB tokens).

2. **Correct separation of EscapeExile vs CollectEvidenceExile.** Both exile cards from graveyard but have different validation rules and semantic meaning. Keeping them distinct prevents confusion.

3. **Copy propagation filter is precisely scoped.** Only mode-selection costs (Entwine, Fuse, EscalateModes) propagate to copies per CR 707.2. All one-shot costs (Sacrifice, Discard, Squad, Offspring, Gift, Mutate, etc.) are correctly filtered out.

4. **CastWithMutate removal.** Consolidating mutate into CastSpell + AdditionalCost::Mutate eliminates a separate Command variant, simplifying the engine.rs dispatch.

5. **Extraction pattern in casting.rs is pragmatic.** Rather than refactoring the entire 5800-line casting.rs to work natively with `additional_costs`, the function extracts into local variables at the top. This minimizes blast radius while achieving the type consolidation goal. Further cleanup can happen in a future pass.

---

## Summary

- **0 HIGH, 0 MEDIUM, 6 LOW, 2 INFO** findings
- **No fix phase needed** — all LOWs are stale documentation references
- CastSpell reduced from ~30 fields to 13 fields
- StackObject reduced by 9 mirrored fields
- All tests pass (1934), clippy clean, workspace builds
- LOWs can be addressed in Session 7 (Memory & Documentation Updates)
