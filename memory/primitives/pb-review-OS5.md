# Primitive Batch Review: PB-OS5 (OOS-EF4-1) — Dynamic relative-count `EffectAmount`

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 205.3m, 508.1/508.1m, 109.1/603.2, 613.1d/613.3, 702.73a (Changeling), 107.3f/608.2h, 611.2a
**Engine files reviewed**: `crates/card-types/src/cards/card_definition.rs` (new variant), `crates/engine/src/effects/mod.rs` (`resolve_amount` arm), `crates/engine/src/rules/layers.rs` (`resolve_cda_amount` arm), `crates/engine/src/state/hash.rs` (discriminant + schema bump), `crates/engine/src/rules/protocol.rs` (protocol bump), + 39 sentinel test files
**Card defs reviewed**: 4 — `shared_animosity.rs`, `goblin_piledriver.rs`, `goblin_rabblemaster.rs`, `muxus_goblin_grandee.rs`

## Verdict: clean (bill)

Zero HIGH, zero MEDIUM. The single new `EffectAmount::OtherAttackersSharingCreatureType { relative_to }` variant is CR-correct: it reads **layer-resolved** subtypes for both the relative creature (`calculate_characteristics`, Option-safe) and each scanned attacker (`expect_characteristics`, safe because objects are freshly gated live via `ZoneId::Battlefield` + `combat.is_attacking`), excludes the relative creature ("other", CR 109.1/603.2), gates on creature card-type + attacking-set membership of **any** controller (CR 508.1), and returns 0 on missing combat / departed relative / typeless subject. All four card defs match oracle text exactly (MCP-verified). Wire discipline is exact (one PROTOCOL 19→20, one HASH 56→57, history rows appended not edited, all 39 sentinel files consistent, discriminant 24). The three mandatory decoy tests are genuinely non-vacuous (each pins the specific semantic against the specific regression). Two informational LOWs below; neither requires a fix.

## Engine Change Findings

_None at HIGH/MEDIUM._

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | LOW (info) | `effects/mod.rs:7600` | **No raw-ctx fallback for a departed `TriggeringCreature`.** Unlike the DealDamage arm (`mod.rs:317`), this arm does not fall back to `ctx.triggering_creature_id` when `resolve_effect_target_list` returns empty. **This is correct, not a bug**: if the relative creature left the battlefield (CR 400.7) the UEOT pump has nothing to apply to, so returning 0 is the right no-op. The doc comment (7589-7599) explains this explicitly. No fix. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW (info) | `muxus_goblin_grandee.rs:45-51` | **Redundant controller scoping.** The `PermanentCount` supplies both an inner `filter.controller: TargetController::You` and an outer `controller: PlayerTarget::Controller`. The outer param already restricts to Muxus's controller (`resolve_amount`/PermanentCount at `mod.rs:7231/7239`); the inner is belt-and-suspenders. Harmless and test-verified correct (`test_os5_muxus_you_control_scope_4player` = +2/+2). No fix. |

### Finding Details

#### E1: TriggeringCreature departed-object handling
**File**: `crates/engine/src/effects/mod.rs:7600-7639`
**CR**: 400.7 (new-object identity), 611.2a (continuous-effect subject)
**Issue**: The arm resolves `relative_to` through `resolve_effect_target_list`, which (per `mod.rs:6629-6632`) gates `EffectTarget::TriggeringCreature` on `state.objects.contains_key`. If the triggering creature already left, this yields empty → `return 0`. The Shared Animosity pump also targets that same departed creature (`EffectFilter::TriggeringCreature`, `mod.rs:3147-3150`), so it is a no-op regardless. The behavior is CR-correct.
**Fix**: None. Documented and intentional.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 205.3m (creature types) | Yes | Yes | `test_os5_shared_animosity_counts_shared_type_attackers` — shares-a-type intersection |
| 508.1 (all attackers, any controller) | Yes | Yes | `test_os5_scope_animosity_piledriver_any_controller` — p2's attacker counts toward p1 subject |
| 613.1d / 702.73a (layer-resolved / Changeling) | Yes | Yes | `test_os5_shared_animosity_layer_resolved_subtype_decoy` — Changeling counted, base-Elf non-attacker excluded; precondition asserts base subtypes lack Elf |
| 109.1 / 603.2 ("other" / exclude-self) | Yes | Yes | `test_os5_shared_animosity_excludes_triggering_creature` (relative_to key) + `test_os5_piledriver_double_multiplier_and_exclude_self` (ctx.source key) |
| 107.3f / 608.2h (count locked at resolution) | Yes | Indirect | resolution-time `resolve_amount`; non-CDA `=> 0` arm in `resolve_cda_amount` |
| Piledriver ×2 (Sum idiom) | Yes | Yes | `test_os5_piledriver_double_multiplier_and_exclude_self` — 0/+2/+4 progression |
| Muxus you-control (PermanentCount) | Yes | Yes | `test_os5_muxus_you_control_scope_4player` — +1/+1, opponent Goblin excluded |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Shared Animosity | Yes | 0 | Yes | inert→**Complete**; `WheneverCreatureYouControlAttacks` + new variant w/ `relative_to: TriggeringCreature`; any-controller scope matches ruling 2008-04-01 |
| Goblin Piledriver | Yes | 0 | Yes | new→**Complete**; protection-from-blue modeled (`ProtectionFrom(FromColor(Blue))`); `Sum(count,count)` ×2, both terms `exclude_self` via ctx.source |
| Goblin Rabblemaster | Yes | 1 (honest) | Yes (pump clause) | pump clause implemented (×1 analogue, `EachPlayer`); **stays partial** — forced-attack `GameRestriction` blocker named; note corrected to `EachPlayer` |
| Muxus, Goblin Grandee | Yes | 1 (honest) | Yes (attack half) | new→**partial**; attack half `PermanentCount` you-control + `ModifyBothDynamic`; ETB reveal/put blocker named (OOS-EF10/PB-OS8); no gated stub |

## Wire / SR-8 Check

- `PROTOCOL_VERSION 19→20`; `PROTOCOL_SCHEMA_FINGERPRINT` = `5243cffc75…21b6`; `ProtocolEpoch{version:20}` appended with matching fingerprint; v19 row unchanged. ✓
- `HASH_SCHEMA_VERSION 56→57`; `HashSchemaEpoch{version:57, decl 02fb46a2f9…, stream 31bfd0ed5d…}` appended; prior rows unedited. ✓
- Hash `HashInto for EffectAmount` arm added: discriminant `24u8` + `relative_to.hash_into`. ✓ (matches doc-comment claim in `card_definition.rs:2894`)
- Sentinel sweep: 0 remaining `HASH_SCHEMA_VERSION, 56` / `PROTOCOL_VERSION, 19`; 43 occurrences of `57`/`20` across 39 files. ✓
- `resolve_cda_amount` explicit `=> 0` arm placed **before** the `_ =>` debug-assert catch-all (`layers.rs:2123` vs `2124`). ✓

## Decoy Non-Vacuity Spot-Check

- **Layer decoy** (`test_2`): Changeling attacker's BASE subtypes asserted to lack Elf (precondition); expects +1/+0. A base-characteristics read would see an empty Changeling subtype set → count 0 → +0/+0 → test FAILS. Genuinely pins layer-read. ✓
- **Exclude-self decoy** (`test_3`): only subject attacking; expects power stays 2. Keying exclusion on `ctx.source` (the never-attacking enchantment) instead of `relative_to` would let the subject count itself → +1/+0 → power 3 → test FAILS. Genuinely pins source≠subject exclusion. ✓
- **4-player scope decoy** (`test_5`): expects +2/+2. Counting the opponent's Goblin → +3; counting attacking-only → +0. Either regression FAILS. Genuinely pins you-control + non-attacking-included + exclude-self. ✓
- **Any-controller decoy** (`test_6`): two-controller combat built directly; expects +1 from p2's attacker. A controller filter would drop it → +0 → FAILS; and non-attacking foreign Goblin correctly excluded. ✓

## Notes

- `expect_characteristics` (`layers.rs:477`) debug-panics only on an absent ObjectId; the scan iterates `state.objects.values()` filtered to live battlefield attackers, so no panic path. Release-mode fallback returns the object's own characteristics. Safe — no `.unwrap()` in library code.
- Both flipped-Complete cards rely on `CardDefinition::default().completeness == Completeness::Complete` (`card_definition.rs:268`) — confirmed, so `validate_deck` will accept them.
- Rabblemaster's pre-existing `AtBeginningOfCombat` token trigger is out of PB-OS5 scope (untouched this batch; note self-declares it "correct"); not evaluated here.
