# W3 LOW S6 Review -- TC-21: PendingTrigger Option Field Migration

**Review Status**: REVIEWED (2026-03-20)
**Reviewer**: Claude Opus 4.6 (milestone-reviewer agent)
**Commit**: 7e474d2 (`W3: LOW S6 -- TC-21 complete: migrate 19 PendingTrigger Option fields to TriggerData`)

## Scope

Migrated 19 Option fields from `PendingTrigger` (stubs.rs) into `TriggerData` variants (stack.rs),
removed ~487 lines of `fieldname: None,` boilerplate across 7 files, added `PendingTrigger::blank()`
constructor, and added 3 new `TriggerData` variants (Madness, Miracle, Suspend).

## Files Changed

| File | Lines +/- | Purpose |
|------|-----------|---------|
| `state/stubs.rs` | -115/+60 | Remove 19 Option fields, add `data: Option<TriggerData>` + `blank()` constructor |
| `state/stack.rs` | +11 | Add TriggerData::Madness, Miracle, Suspend variants |
| `state/hash.rs` | -29/+17 | Remove stale field hashes, add Madness/Miracle/Suspend hash arms |
| `rules/abilities.rs` | -1112/+472 | Update all setters (push_back) and readers (flush_pending_triggers) |
| `rules/turn_actions.rs` | -333/+20 | Update all PendingTrigger constructors |
| `rules/resolution.rs` | -237/+20 | Update PendingTrigger constructors, reorder champion_exiled_card in GameObject |
| `rules/casting.rs` | -37/+4 | Update Madness trigger in handle_cast_spell |
| `rules/miracle.rs` | -38/+3 | Update Miracle trigger in handle_choose_miracle |
| `rules/replacement.rs` | -57/+6 | Update PendingTrigger constructors in fire_saga_chapter_triggers + queue_carddef_etb_triggers |
| `effects/mod.rs` | -84/+11 | Update Madness triggers in execute_effect_inner + discard_cards |
| `tests/partner_with.rs` | +4/-1 | Update test to read from `trigger.data` instead of removed field |

## Data Integrity Audit: Setter/Reader Pairs

For each migrated field, I verified the setter (where data is written into the trigger) and reader
(where data is extracted in `flush_pending_triggers`) both use the same `TriggerData` variant:

| Field | TriggerData | Setter Files | Reader (abilities.rs) | Status |
|-------|-------------|--------------|----------------------|--------|
| madness_exiled_card + madness_cost | Madness | abilities.rs:960-964, casting.rs:3396-3399, effects/mod.rs:3388-3391+4519-4522, turn_actions.rs:1214-1217 | 5083-5085 | OK |
| miracle_revealed_card + miracle_cost | Miracle | miracle.rs:109 | 5096-5098 | OK |
| suspend_card_id | Suspend | turn_actions.rs:73, resolution.rs:5407 | 5229-5231, 5240-5242 | OK |
| hideaway_count | ETBHideaway | abilities.rs (check_triggers) | 5254-5256 | OK |
| partner_with_name | ETBPartnerWith | abilities.rs (check_triggers) | 5269-5273 | OK |
| modular_counter_count | DeathModular | abilities.rs (death triggers) | 5160-5162 | OK |
| evolve_entering_creature | ETBEvolve | abilities.rs (check_triggers) | 5200-5202 | OK |
| graft_entering_creature | ETBGraft | abilities.rs (check_triggers) | 5421-5423 | OK |
| backup_abilities + backup_n | ETBBackup | abilities.rs (check_triggers) | 5436-5438 | OK |
| champion_filter | ETBChampion | abilities.rs (check_triggers) | 5459-5461 | OK |
| champion_exiled_card | LTBChampion | abilities.rs (death/LTB triggers) | 5471-5473 | OK |
| soulbond_pair_target | ETBSoulbond | abilities.rs (check_triggers) | 5484-5486 | OK |
| squad_count | ETBSquad | resolution.rs:1393-1395 | 5514-5516 | OK |
| gift_opponent | ETBGift | resolution.rs:1469-1473 | 5542-5544 | OK |
| encore_activator | EncoreSacrifice | turn_actions.rs:582-583 | 5369-5371 | OK |
| provoke_target_creature | CombatProvoke | abilities.rs (check_triggers) | 5318 | OK |

All 19 fields verified: setters and readers use matching TriggerData variants.

## Findings

| ID | Severity | File:Line | Description | Status |
|----|----------|-----------|-------------|--------|
| W3S6-01 | **HIGH** | `hash.rs:1741` | **Hash omission for `PendingTrigger.data` field.** The `data: Option<TriggerData>` field added to `PendingTrigger` is NOT hashed in `HashInto for PendingTrigger` (line 1703-1741). The field carries all meaningful trigger payload data (madness cost, suspend card, champion filter, etc.) but the hash function terminates at `cipher_encoded_object_id` without hashing `self.data`. The `TriggerData` enum itself has a complete `HashInto` impl (lines 1907-2093, 37 variants), but it is never called for PendingTrigger instances. This causes non-determinism in distributed verification (Architecture Invariant 7) -- two PendingTriggers with different `data` payloads would produce the same hash. **Fix:** Add `self.data.hash_into(hasher);` after `self.cipher_encoded_object_id.hash_into(hasher);` at line 1740 in `hash.rs`. | OPEN |
| W3S6-02 | **LOW** | `hash.rs:1722-1740` | **Dead legacy fields still hashed.** 13 remaining Option fields on `PendingTrigger` (`ingest_target_player`, `flanking_blocker_id`, `rampage_n`, `renown_n`, `poisonous_n`, `poisonous_target_player`, `enlist_enlisted_creature`, `recover_cost`, `recover_card`, `cipher_encoded_card_id`, `cipher_encoded_object_id`, `haunt_source_object_id`, `haunt_source_card_id`) are never set to `Some` anywhere in the codebase -- all data now flows through `trigger.data`. These fields and their hash lines add 26 lines of dead code. Hashing `None` is harmless but wasteful. **Fix:** In a follow-up session, migrate these 13 fields to TriggerData (completing the migration), remove the struct fields, remove their hash lines, and update `PendingTrigger::blank()`. | OPEN |
| W3S6-03 | **LOW** | `stubs.rs:348` | **`PendingTrigger.data` is `#[serde(skip)]`.** The `data` field uses `#[serde(skip)]` with the comment "triggers are transient within a turn." This is consistent with `kind` (also skipped). However, if PendingTrigger is ever serialized/deserialized (e.g., for save/load), all trigger payload data would be lost. This is acceptable for the current architecture (triggers are drained within the same turn step) but should be documented as a constraint. | OPEN |
| W3S6-04 | **LOW** | `resolution.rs:4338-4340` | **Inconsistent indentation in GameObject construction.** The `champion_exiled_card` field was moved from its original position to after `gift_was_given`, and the comment `// CR 702.171b: tokens are not saddled by default.` has incorrect indentation (12 spaces instead of 20). This appears in 6 identical GameObject token-creation blocks in resolution.rs. Cosmetic only; `cargo fmt` would fix. **Fix:** Run `cargo fmt`. | OPEN |

## Hash Coverage Verification

- **TriggerData enum**: 37 variants (discriminants 0-36), all covered in `HashInto for TriggerData` -- COMPLETE
- **PendingTrigger struct**: 22 fields total; 20 hashed, 1 NOT hashed (`data` -- W3S6-01), 1 correctly skipped (`kind` has its own hash) -- INCOMPLETE

## Test Status

- All 2233 tests pass (0 failures)
- `cargo clippy -- -D warnings` passes
- `cargo fmt --check` reports formatting diffs (cosmetic, W3S6-04)

## Summary

The TC-21 migration is **structurally correct** -- all 19 setter/reader pairs use matching TriggerData
variants, no data is dropped, and the new `PendingTrigger::blank()` constructor is well-designed.

However, there is **1 HIGH finding**: the `data` field is not included in PendingTrigger's hash
function, which means the hash does not reflect the actual trigger payload. This should be fixed
before any distributed verification work (M10).

The remaining 3 findings are LOW (dead code, serde skip documentation, cosmetic formatting).
