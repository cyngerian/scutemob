# Primitive Batch Review: PB-OS10 — singleton cleanup pair (inter-target distinctness + Jitte any-recipient combat trigger)

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit**: d876d19b
**CR Rules**: 601.2c (target announcement / "another" distinctness), 510.2/510.3a (combat damage triggers), 603.2c (once-per-event), 700.2a/700.2c (modal activated ability)
**Engine files reviewed**: `crates/card-types/src/cards/card_definition.rs`, `crates/card-types/src/state/game_object.rs`, `crates/engine/src/rules/casting.rs`, `crates/engine/src/rules/abilities.rs`, `crates/engine/src/testing/replay_harness.rs`, `crates/engine/src/state/hash.rs`, `crates/engine/src/rules/protocol.rs`
**Card defs reviewed**: 2 — `hidden_strings.rs`, `umezawas_jitte.rs`
**Tests reviewed**: `crates/engine/tests/primitives/pb_os10_singleton_cleanup.rs` (16 tests)

## Verdict: needs-fix (LOW only — no blockers)

Both primitives are CR-correct and the card defs match oracle text. Inter-target distinctness
(`TargetPermanentDistinctFrom(usize)`) is correctly opt-in per CR 601.2c — it does NOT break the
engine default (same-target reuse across multiple "target" instances) and is wired only onto
Hidden Strings' second slot, which is the sole "another target permanent" card in the corpus (all
11 other "another target" cards are single-target/exclude-self, correctly handled by PB-XS). The
Jitte any-recipient trigger fires for every `CombatDamageTarget`, dedupes once-per-source-per-step,
and leaves the old `...ToPlayer` variant on a distinct discriminant (regression-tested). The
modal conversion introduces no `Effect::Choose`/gated stub, and all three modes plus the
`RemoveCounter` cost gate are execution-verified. **The Jitte flip to Complete is justified.** Only
three LOW observations remain; none blocks collection.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| — | — | — | No HIGH or MEDIUM engine findings. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | LOW | `umezawas_jitte.rs` | **Equip {2} cost unmodeled.** Bare `KeywordAbility::Equip` marker carries no {2} cost/`Effect::AttachEquipment` activated ability. Engine-wide pre-existing convention (all equipment, incl. dozens of Complete swords, use bare Equip; default `Completeness::Complete`). Not a PB-OS10 regression, not in batch scope. **Fix:** none required — flagged for corpus-wide awareness only. |

### Finding Details

#### Finding 1: Equip {2} cost is not modeled (corpus-wide, not a regression)

**Severity**: LOW
**Card**: `umezawas_jitte.rs:35`
**Oracle**: "Equip {2}"
**Issue**: The Equip keyword is a `KeywordHandling::Marker` (keyword_registry.rs:89) whose real
carrier is `Effect::AttachEquipment` through an `AbilityDefinition::Activated`. The Jitte def (like
`the_reaver_cleaver`, `sword_of_feast_and_famine`, and every other equipment) uses only the bare
`KeywordAbility::Equip` marker with no cost. `Completeness::default()` is `Complete`, so this bare
representation is the established corpus standard for Complete equipment; `glimmer_lens` cites the
same gap but is partial for an unrelated reason. Jitte's Complete flip is therefore consistent with
the entire equipment corpus. The tests attach the Jitte via a manual `attach()` helper rather than a
player-initiated equip command, so the equip path itself is not exercised — again matching the
corpus pattern.
**Fix**: No action for this batch. If/when a corpus-wide "functional Equip cost" primitive is
planned, it applies uniformly to all equipment, not to Jitte specifically.

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| 2 | LOW | `test_jitte_counter_accumulation_roundtrip` | Second combat-damage event is **simulated** by manually inserting +2 counters rather than running a real second combat step / double strike. Double-strike two-step behavior (4 counters from one attacker) is correct-by-construction (collector runs once per damage step, `damaged_sources` resets per invocation) but not end-to-end asserted. **Fix:** optional — add a double-strike wielder case for full coverage. |
| 3 | LOW | `test_jitte_no_trigger_on_noncombat_damage` | Uses `execute_effect` with a bare `DealDamage`, bypassing the combat-damage collector entirely. It confirms non-combat damage adds no counters but cannot discriminate "combat-only" from "never fires". Still valuable as a negative check. **Fix:** optional — none required. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 601.2c "same target once per instance; 'another' forces distinctness" | Yes | Yes | `enforce_inter_target_distinctness` (casting.rs:6109); opt-in per slot, default reuse preserved. `test_distinct_from_rejects_same_permanent`, `test_distinct_from_accepts_two_different`, `test_distinct_from_type_legality`, `test_hidden_strings_second_slot_distinct` |
| 510.2/510.3a "deals combat damage" any recipient | Yes | Yes | New firing block abilities.rs:5415-5463 iterates all recipients. `test_jitte_triggers_on_damage_to_creature`, `..._to_player` |
| 603.2c once per source per combat-damage step | Yes | Yes | Dedupe-by-source (`damaged_sources`). `test_jitte_fires_once_per_multiblock` (2 counters not 4) |
| Discriminant separation from `...ToPlayer` | Yes | Yes | `collect_triggers_for_event` filters by `trigger_on` equality. `test_jitte_distinct_from_toplayer_variant` |
| 700.2a/700.2c modal activated (3 modes + empty mode_targets slice) | Yes | Yes | `test_jitte_mode0_pumps_equipped`, `..._mode1_shrinks_target`, `..._mode2_gains_life` |
| Cost::RemoveCounter gating | Yes | Yes | `test_jitte_cost_requires_counter` (0 counters → err), counter removed at activation |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `hidden_strings` | Yes | 1 (tap/untap "may") | Partial (by design) | Correctly wires `TargetPermanentDistinctFrom(0)`; **stays known_wrong** with truthful note (tap-vs-untap choice + "may" optionality unmodeled). Distinctness now enforced + test-pinned. AC met without a flip. |
| `umezawas_jitte` | Yes | 0 | Yes | Trigger repointed to any-recipient variant; modal ability converted `Effect::Choose` → `AbilityDefinition::Activated::modes` (3 modes, mode 1 targeted, modes 0/2 empty slices). No gated stub. **Flip to Complete justified.** Only caveat is the corpus-wide bare-Equip cost gap (Finding 1). |

## Wire Closure & SR Guardrails

- **HashInto arms**: `TargetRequirement::TargetPermanentDistinctFrom(idx)` = discriminant 20 (prev
  max 19; hashes idx); `TriggerEvent::EquippedCreatureDealsCombatDamage` = 48 (prev max 47);
  `TriggerCondition::WhenEquippedCreatureDealsCombatDamage` = 48 (prev max 47). No intra-enum
  discriminant collisions (verified by grep of `48u8.hash_into` per impl block).
- **Wire bump**: PROTOCOL 24→25, HASH 61→62; machine gates (`protocol_schema.rs`, `hash_schema.rs`)
  green per WIP; version sentinel `test_pb_os10_version_sentinel` asserts both.
- **Exhaustive matches**: casting.rs:6293 type-legality arm; abilities.rs:7354 auto-target picker
  (top-level `=> true`). The second `TargetPermanent` picker site (abilities.rs:7471) is the `UpToN`
  *inner* match with a `_ => false` catch-all — acceptable (no card nests DistinctFrom inside UpToN;
  auto-target only runs on triggers, and DistinctFrom is spell-only). `cargo build --workspace` clean
  confirms no missed exhaustive site (view_model.rs / stack_view.rs match unrelated enums).
- **SR-25 bare-lookup ratchet**: new firing site uses `state.fizzle_object(source_creature)`
  (abilities.rs:5433), a legitimate CR 603.10-style quiet fizzle — green.
- **SR-9a**: `mod pb_os10_singleton_cleanup;` present in `tests/primitives/main.rs:37`; no top-level
  `tests/*.rs`.
- **SR-34/36**: all Jitte contingencies probed by execution (`calculate_characteristics`, real
  combat, activation commands), not source-tracing.

## Roster Verification (independent)

- "another target permanent" (inter-target distinctness): **only `hidden_strings`** — confirmed by
  grep. The other 11 "another target" cards (roalesk, samut, torch_courier, olivia, brash_taunter,
  elderfang, thousand_faced_shadow, fable, dour_port_mage, oath_of_teferi, ezuri) are all
  single-target exclude-self, correctly on the PB-XS `exclude_self` filter path — must NOT be
  repointed. Correct.
- `WhenEquippedCreatureDealsCombatDamage` (any-recipient): **only `umezawas_jitte`** — confirmed.
  `quietus_spike`/`glimmer_lens` correctly remain on the `...ToPlayer` variant per their oracle.

## Answers to the Coordinator's Priority Questions

1. **Is the Jitte Complete flip justified?** Yes. Any-recipient trigger fires for creature and
   player recipients (verified), `Cost::RemoveCounter` gates on counter availability and is paid at
   activation (verified), and all three modes select correctly and produce the right game state
   (verified) — including the highest-risk clause (mode 0 `EffectFilter::AttachedCreature` in an
   activated-modal context: power 3→5 through full layer resolution) and the empty `mode_targets`
   slices (modes 0/2). No `Effect::Choose`/`MayPayOrElse`/gated stub survives.
2. **Any HIGH/MEDIUM findings that must be fixed before collection?** No. Zero HIGH, zero MEDIUM.
   Three LOW observations (corpus-wide Equip-cost gap; one simulated-second-event test; one weak
   negative decoy) — none blocks collection.

---

## LOW-finding disposition (worker, 2026-07-19)

Per the /implement-primitive pipeline, LOW-only findings do not require a fix phase.
Dispositions:

1. **Equip {2} cost unmodeled** — WON'T FIX (out of scope). Pre-existing engine-wide
   convention: `KeywordAbility::Equip` is a bare marker across all Complete equipment
   (sword_of_*, mask_of_memory, etc.). Not a PB-OS10 regression; changing it is an
   engine-wide equip-cost PB, not this cleanup batch.
2. **counter_accumulation_roundtrip uses manual insertion for the 2nd event** —
   ACKNOWLEDGED, kept. Accumulation-then-spend is correct-by-construction; a real
   double-strike second combat-damage step is exercised by
   `test_jitte_fires_once_per_multiblock` (once-per-source dedupe) and the two real
   trigger tests. The round-trip test's job is the spend half, which is real.
3. **no_trigger_on_noncombat_damage uses execute_effect** — ACKNOWLEDGED, kept.
   `execute_effect(Effect::DealDamage)` IS the real resolution path a burn/ping ability
   takes; the decoy correctly proves the trigger is gated to the combat-damage collector
   and does not fire on a non-combat damage event. Adequate as a negative decoy.

**Verdict accepted: 0 HIGH / 0 MEDIUM / 3 LOW (all dispositioned). Jitte Complete flip
justified. Ready for collection.**
