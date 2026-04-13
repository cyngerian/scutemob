# Primitive Batch Review: PB-N — SubtypeFilteredAttack + SubtypeFilteredDeath triggers

**Date**: 2026-04-12
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 508.1m, 603.2, 603.10a, 603.4, 613.1d/f, 510.3a
**Engine files reviewed**: `cards/card_definition.rs`, `state/game_object.rs`, `state/hash.rs`, `state/builder.rs`, `rules/abilities.rs`, `rules/resolution.rs`, `testing/replay_harness.rs`
**Card defs reviewed**: kolaghan_the_storms_fury, dromoka_the_eternal, sanctum_seeker, teysa_orzhov_scion (4 newly authored), plus a sample of ~10 of the ~56 mechanical backfill files (utvara_hellkite, beastmaster_ascension, druids_repository, mardu_ascendancy, grim_haruspex, cruel_celebrant, syr_konrad_the_grim, marionette_apprentice, blood_artist, skullclamp)

## Verdict: **needs-fix**

Primitive engine surface is correct and the dispatch chain is wired through cleanly: DSL → enrich → runtime field → both dispatch sites → hash. The hash sentinel bump 3→4 is in place, the new field is hashed, and `#[serde(default)]` is applied for backwards compatibility. The 8 mandatory tests are all present (none ignored, none missing).

However, **two coordinator focus areas have material gaps**, and **two of the four newly authored cards have semantic problems** (one wrong-game-state, one missed opportunity that left a stale TODO in a non-deferred neighboring card). Specifically:

1. The "load-bearing LKI test" (test 6) does not actually exercise the LKI code path — it tests a creature whose color never changes, so the test would pass even if the death-side dispatch read post-death characteristics rather than pre-death LKI. This is the exact silent-skip pattern PB-Q4 retro warned against.
2. The `combat_damage_filter` regression test (test 9) does not actually validate the latent-bug claim — it uses `trigger_on: AnyCreatureYouControlDealsCombatDamageToPlayer`, so the trigger would be filtered out by the outer event-type match regardless of whether the filter was tightened. The test is structurally incapable of catching a regression of the tightening.
3. **Sanctum Seeker uses `Effect::DrainLife { amount: 1 }`**, which gains the controller life equal to *total* opponent life lost (3 in 4-player). Oracle says "you gain 1 life" — flat. Wrong game state in any 3+ player game.
4. **Utvara Hellkite still has its "Dragon subtype filter not yet in DSL" TODO and `filter: None`**, despite PB-N now providing exactly that primitive. The card still over-triggers on non-Dragon attackers — wrong game state — and was not in the deferred list.

The hash sentinel-bump test (test 7) only asserts the hash is non-zero, not that the sentinel value is 4 (or non-3). It would not catch a sentinel rollback. MEDIUM.

None of these are stop-and-flag events for the design itself — the engine work is sound — but two card-level wrong-game-state issues and two test-coverage gaps need to be addressed before close.

## Coordinator Focus Area Verifications

### 1. `combat_damage_filter` tightening — **PARTIAL PASS**

- Engine code: tightening is correctly applied at `crates/engine/src/rules/abilities.rs:5854-5873`. The `combat_damage_filter` consumption is now gated on `event_type == TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer`. Comment cites the latent-bug fix and CR 510.3a. PASS for the engine fix.
- CR citation: present in both code comment (line 5853) and the regression test docstring. PASS.
- Regression test (test 9, `test_pbn_combat_damage_filter_not_consulted_on_attack_events`): **FAIL — does not validate the claim**. The trigger under test has `trigger_on: AnyCreatureYouControlDealsCombatDamageToPlayer`. When `DeclareAttackers` fires `AnyCreatureYouControlAttacks`, the **outer event-type match** at `abilities.rs:5828` (`if trigger_def.trigger_on != event_type { continue; }`) would have caused the trigger to be skipped *regardless* of whether the inner `combat_damage_filter` was scoped. The test would have passed before the tightening. To actually catch the latent bug, the trigger must have `trigger_on: AnyCreatureYouControlAttacks` AND a `combat_damage_filter` set to a value that does NOT match the attacker; the post-fix expectation is that the trigger fires (because the filter is now ignored on the attack branch), but the pre-fix behavior would have been that the trigger is filtered out. This is the only configuration that exercises the regression.

### 2. Mechanical card-def shape change (~56 files) — **PARTIAL PASS**

- Sampled 10 files: utvara_hellkite, beastmaster_ascension, druids_repository, mardu_ascendancy, grim_haruspex, cruel_celebrant, syr_konrad_the_grim, marionette_apprentice, blood_artist, skullclamp. All correctly converted to the new struct shape with `filter: None`. No semantic content lost in conversion. PASS for the mechanical correctness.
- However: **utvara_hellkite (`crates/engine/src/cards/defs/utvara_hellkite.rs:18-22`) still carries its pre-PB-N TODO** (`"Dragon subtype filter not yet in DSL — over-triggers on non-Dragon attackers"`) and was left at `filter: None`. The card is now staying in a known-wrong-game-state with the primitive sitting on the shelf to fix it. This is a missed in-scope card. Per the per-card oracle gate, leaving it at `filter: None` produces wrong game state and should either ship Dragon filter or be added to the deferred list with reason. It is not in the 11 deferred cards.
- Cosmetic: a few backfilled files (grim_haruspex.rs:27-28, cruel_celebrant.rs:25-26, blood_artist.rs:19-20, marionette_apprentice.rs:21-22, syr_konrad_the_grim.rs:28-29) have misaligned `filter: None,` lines and stray closing braces — `cargo fmt` should normalize but the runner reports clean fmt, so this is an edit-side artifact in indentation that did not get touched. Cosmetic only, LOW.

### 3. Hash bump parity test — **PARTIAL PASS**

- Test 7 (`test_pbn_hash_parity_triggering_creature_filter` at lines 462-537) asserts that `hash_no_filter != hash_with_filter`, which **does** verify the new field participates in the hash. PASS for the field-parity claim.
- Test 7 also asserts `state.public_state_hash() != [0u8; 32]` to "verify sentinel is non-zero". This is **NOT** a sentinel-bump assertion — it would pass for sentinel value 0, 1, 2, 3, or 4 (any state with players hashes to a non-zero value once players are folded in). A hostile rollback from sentinel 4 to sentinel 3 would not be caught. The PB-Q H1 retro lesson asks for an actual value assertion, not a non-zero check. MEDIUM.

### 4. Oracle vs DSL parity for the 4 newly authored cards — **PARTIAL PASS**

- **Kolaghan, the Storm's Fury**: Oracle text matches DSL (Flying + Dragon-filtered attack trigger that grants `+1/+0` to creatures you control until end of turn + Dash). PASS.
- **Dromoka, the Eternal**: Oracle text matches DSL (Flying + Dragon-filtered attack trigger that bolsters 2). PASS.
- **Sanctum Seeker**: **FAIL — wrong game state**. Oracle: "Whenever a Vampire you control attacks, each opponent loses 1 life and you gain 1 life." DSL uses `Effect::DrainLife { amount: 1 }`, which (per `effects/mod.rs:494-503`) gains the controller life **equal to the total amount of life actually lost across all opponents**. In a 4-player Commander game this is 3 life gained per Vampire attack, not 1. The flat-1 gain is the printed effect; DrainLife is the wrong primitive. Either author this with sequenced `LoseLife { player: EachOpponent, 1 }` + `GainLife { player: Controller, 1 }` (if a sequenced effect is expressible) or defer the card.
- **Teysa, Orzhov Scion**: Oracle (death trigger half) matches DSL (`controller: You + exclude_self: true + colors: {Black}`, creating a 1/1 white Spirit with flying). The deferred sacrifice ability is correctly noted as TODO (lines 5-7) with cited blocker (multi-permanent sacrifice cost not in DSL). PASS for the partial-card decision.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|--------------|---------|-------|
| 508.1m  | Yes          | Yes     | Tests 1, 2, 3, 8 |
| 603.2   | Yes          | Yes     | Tests 1-6 implicitly |
| 603.10a | Yes          | Partial | Test 6 claims to test LKI but does not actually exercise the LKI distinction (no continuous effect that ends at death). |
| 603.4   | N/A          | N/A     | Filter is part of trigger condition, not intervening-if. Plan correctly notes this. |
| 613.1d/f| Yes          | No      | Filter dispatch uses `calculate_characteristics` at both sites. The optional layer-resolved subtype test (test 10) was not implemented. Acceptable per plan. |
| 510.3a  | Yes (tightened) | No  | Test 9 does not actually exercise the tightening (see Coordinator Focus 1 above). |

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | MEDIUM   | `crates/engine/tests/pbn_subtype_filtered_triggers.rs:415-454` | **Test 6 does not exercise pre-death LKI.** **Fix:** add a continuous effect that makes the dying creature non-black on the battlefield AND ends at death (or use a creature whose color comes from a now-defunct effect). Assert that pre-death color (Black) is what the death-side filter sees. As written, the test would pass even if the engine read post-death state. |
| 2 | MEDIUM   | `crates/engine/tests/pbn_subtype_filtered_triggers.rs:676-753` | **Test 9 does not validate the combat_damage_filter tightening.** The trigger uses `trigger_on: AnyCreatureYouControlDealsCombatDamageToPlayer`, which the outer event-type match drops on attack events anyway. **Fix:** change the trigger to `trigger_on: AnyCreatureYouControlAttacks` with a `combat_damage_filter` whose subtype does NOT match the attacker (e.g., filter Ninja, attacker Goblin), and assert the trigger DOES fire (because the filter is now scoped only to damage events). This is the only shape that catches the regression. |
| 3 | MEDIUM   | `crates/engine/tests/pbn_subtype_filtered_triggers.rs:525-536` | **Hash sentinel test does not verify the bump.** The assertion checks for non-zero, which is true at any sentinel value once a player exists. **Fix:** assert the public_state_hash differs between two states with otherwise-identical content but different hash impls — or precompute the expected hash bytes for an empty 2-player state and assert equality (golden hash). At minimum, assert that an empty-2-player state hash is non-equal to a known sentinel-3 fingerprint (if available), or compare against a snapshot. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **HIGH** | `crates/engine/src/cards/defs/sanctum_seeker.rs:26` | **DrainLife produces wrong game state.** Oracle: "each opponent loses 1 life and you gain 1 life" (flat 1). DSL: `Effect::DrainLife { amount: 1 }` gains controller `total_lost` (3 in 4p Commander). **Fix:** if a sequenced effect like `Effect::Sequence { effects: [LoseLife{EachOpponent,1}, GainLife{Controller,1}] }` is expressible, use it. Otherwise add Sanctum Seeker to the deferred list with reason "needs flat-gain primitive distinct from DrainLife" and revert this file to its pre-PB-N state. |
| 2 | **HIGH** | `crates/engine/src/cards/defs/utvara_hellkite.rs:18-22` | **Stale TODO + wrong game state.** PB-N exists specifically to fix this card's `"Dragon subtype filter not yet in DSL — over-triggers on non-Dragon attackers"` TODO. The trigger is still `filter: None` and the TODO comment is intact. The card was not in the deferred list. **Fix:** change `filter: None` to `filter: Some(TargetFilter { has_subtype: Some(SubType("Dragon".to_string())), ..Default::default() })`, remove the TODO, and add a PB-N citation. This is a free win that the runner missed. |
| 3 | LOW | `crates/engine/src/cards/defs/grim_haruspex.rs:27-28` and 4 other backfill files (cruel_celebrant.rs:25-26, blood_artist.rs:19-20, marionette_apprentice.rs:21-22, syr_konrad_the_grim.rs:28-29) | **Misaligned indentation** on the inserted `filter: None,` line and the closing `}`. Cosmetic. **Fix:** `cargo fmt` once should normalize. Note: the wip claims fmt is clean, so the formatter may be ignoring these blocks; verify by hand. |

### Finding Details

#### Finding 1 (HIGH): Sanctum Seeker DrainLife semantics

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/sanctum_seeker.rs:26`
**Oracle**: "Whenever a Vampire you control attacks, each opponent loses 1 life and you gain 1 life."
**Issue**: `Effect::DrainLife` (per `crates/engine/src/effects/mod.rs:465-505`) gains the controller life equal to the **total** life lost across all opponents (`total_lost`), not a flat amount. In 4-player Commander, this means 3 life gained per Vampire attack, not 1. Reading the engine's DrainLife implementation: `controller.life_total += total_lost as i32`. This contradicts Sanctum Seeker's printed "you gain 1 life" effect.
**Fix**: Replace with a sequenced effect that does (a) lose 1 life from each opponent, then (b) gain exactly 1 life on the controller, regardless of how many opponents lost life. If DSL has no sequenced primitive, defer Sanctum Seeker to a follow-up PB and revert this file. Do NOT ship as DrainLife.

#### Finding 2 (HIGH): Utvara Hellkite missed in-scope card

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/utvara_hellkite.rs:18-22`
**Oracle**: "Whenever a Dragon you control attacks, create a 6/6 red Dragon creature token with flying."
**Issue**: This is the canonical PB-N target — the file's own TODO at line 20 says `"Dragon subtype filter not yet in DSL — over-triggers on non-Dragon attackers."` PB-N adds exactly that filter. The mechanical backfill changed `WheneverCreatureYouControlAttacks` (unit) → `{ filter: None }` and walked away. The card is in the same wrong-game-state as before. It is not on the deferred list.
**Fix**: Set `filter: Some(TargetFilter { has_subtype: Some(SubType("Dragon".to_string())), ..Default::default() })`, remove the TODO comment at lines 19-20, replace with a PB-N citation. This raises the in-scope card count from 4 to 5.

#### Finding 3 (MEDIUM): Test 6 LKI not actually exercised

**Severity**: MEDIUM
**File**: `crates/engine/tests/pbn_subtype_filtered_triggers.rs:415-454`
**CR Rule**: 603.10a — "The game must check what the object would have looked like immediately before it left the battlefield."
**Issue**: The plan explicitly called for a creature whose color comes from a *continuous effect* that ends at death — so that pre-death look-back would see Black but post-death evaluation would see the printed (non-Black) color. As written, test 6 creates a creature with `with_colors(vec![Color::Black])` directly on its base characteristics, with no continuous effect at all. The test would pass under either pre-death or post-death evaluation, because the creature's printed color is Black throughout. Per PB-Q4 retro: this is the silent-skip pattern — the test name claims to validate LKI but the construction does not exercise the LKI path.
**Fix**: Either (a) make the dying creature non-Black at base and add a `LayerModification::SetColor(Black)` continuous effect with `WhileSourceOnBattlefield` duration so the effect terminates at zone change; or (b) flip the colors — have a base-Black creature with a continuous effect that turns it Red while on battlefield, and use a Red-filter trigger so post-death evaluation would see Black (no fire) but pre-death evaluation would see Red (fire). Option (b) is the cleaner discriminator.

#### Finding 4 (MEDIUM): Test 9 does not validate combat_damage_filter tightening

**Severity**: MEDIUM
**File**: `crates/engine/tests/pbn_subtype_filtered_triggers.rs:676-753`
**CR Rule**: 510.3a (combat damage event scope)
**Issue**: The test creates a trigger with `trigger_on: AnyCreatureYouControlDealsCombatDamageToPlayer` and asserts that `DeclareAttackers` does not fire it. But `collect_triggers_for_event` at `abilities.rs:5828` already filters by `trigger_on == event_type` *before* any filter consultation. The trigger would be skipped on attack events even if `combat_damage_filter` were still consulted on attacks. The test is structurally incapable of catching a regression of the tightening — it would pass against both the pre-fix and post-fix engine.
**Fix**: Use `trigger_on: AnyCreatureYouControlAttacks` so the trigger is event-matched on the attack event, and set `combat_damage_filter: Some(TargetFilter { has_subtype: Some(SubType("Ninja".into())) })` with a Goblin attacker. Pre-fix: the filter would have caused a skip (no trigger fires). Post-fix: the filter is no longer consulted on the attack branch, so the trigger fires. Assert `stack_trigger_count > 0`. Add a comment documenting the regression direction.

#### Finding 5 (MEDIUM): Hash sentinel test does not verify the bump

**Severity**: MEDIUM
**File**: `crates/engine/tests/pbn_subtype_filtered_triggers.rs:525-536`
**Issue**: The assertion `assert_ne!(state.public_state_hash(), [0u8; 32])` only verifies that the hash is non-zero. This is true at any sentinel value (0, 1, 2, 3, 4, ...) once a player is folded into the hash. A rollback of the sentinel from 4 back to 3 would not be caught by this test. The test name and docstring claim to verify the sentinel bump but the assertion is too weak.
**Fix**: One of: (a) snapshot the hash bytes for an empty-2-player state with the current schema and assert equality against a const, refreshed on every sentinel bump (golden hash); (b) construct two states with the **same content** but feed them through different schema versions (requires harness support) and assert non-equality; or (c) at minimum, write a separate `#[test]` named `test_pbn_hash_sentinel_is_4` that calls a hypothetical `pub fn schema_version() -> u8 { 4 }` accessor exported from `state/hash.rs` and asserts the value. Option (a) is the existing convention in similar repos and is the most robust.

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|--------------|-----------------|---------------------|-------|
| kolaghan_the_storms_fury | Yes | 0 (PB-N TODO removed) | Yes | Clean. |
| dromoka_the_eternal | Yes | 0 | Yes | Clean. |
| sanctum_seeker | **No** | 0 | **No** | DrainLife gains `total_lost`, oracle says flat 1. HIGH. |
| teysa_orzhov_scion | Partial | 1 (sacrifice ability, properly cited blocker) | Yes (death trigger half) | Sacrifice ability deferral is correct per partial-card policy. |
| utvara_hellkite (mechanical backfill) | **No** | **1 stale** | **No** | Should have been promoted in-scope; PB-N exists for exactly this card. HIGH. |
| 9 other sampled mechanical backfill files | Yes (no semantic change) | unchanged | Pre-existing | Cosmetic indentation issues on a handful. |

## Standard Checklist

- **Full dispatch chain** (`memory/feedback_verify_full_chain.md`): walked. DSL `card_definition.rs` (2 variants extended) → enrich at `replay_harness.rs:2434` and `replay_harness.rs:2473` (both arms forward `filter` to `triggering_creature_filter`) → runtime `TriggeredAbilityDef` field at `game_object.rs:604` → attack-side dispatch at `abilities.rs:5874-5890` → death-side dispatch at `abilities.rs:4176-4202` → hash at `hash.rs:2257` (TriggeredAbilityDef arm) and `hash.rs:4435/4495` (TriggerCondition arms). Every site honors the new field. PASS.
- **Hash exhaustiveness**: All new fields hashed. Sentinel bumped 3→4 at `hash.rs:6125`. PASS for the bump itself; MEDIUM finding for the test that fails to assert the bump.
- **`#[serde(default)]`** on the new fields: PASS at `card_definition.rs:2447, 2567` and `game_object.rs:603`.
- **CR citations** in test docstrings: All 9 tests cite at least one CR rule. PASS.
- **No silent test skips**: 8/8 mandatory tests labeled and present, none `#[ignore]`. PASS.
- **Mechanical card-def backfill**: ~56 files updated to new struct shape. Sample of 10 verified for no semantic content lost. PASS for shape; HIGH finding for utvara_hellkite missing the in-scope promotion.
- **Builder.rs / resolution.rs / replay_harness.rs `triggering_creature_filter: None` backfill**: verified at `builder.rs` (~22 sites), `resolution.rs:625`, and the 3 sentinel `basri_ket / ajani_sleeper_agent / tyvar_kell` card defs that pre-existed the new field. PASS.
- **TUI / replay-viewer exhaustive match impact**: PB-N adds fields to existing variants (no new variants in `StackObjectKind`, `KeywordAbility`, `TriggerEvent`, etc.), so no exhaustive match holes elsewhere. PASS.

## Stop-and-Flag Events

None. None of the findings rise to stop-and-flag severity:

- combat_damage_filter tightening is correct in code; only the regression test fails to validate it. (NOT stop-and-flag because the engine direction is right.)
- All 8 mandatory tests are present, none `#[ignore]`d, none missing. (NOT stop-and-flag.)
- The card-def backfill did not change semantic content of existing files. (NOT stop-and-flag.)
- Hash bump 3→4 IS present in the deserialization path, the sentinel mismatch will produce a hash diff (not a silent zero-fill). (NOT stop-and-flag — the bump itself is solid; only the test that asserts it is weak.)
- Sanctum Seeker oracle disagreement is HIGH but does not require escalation; it is a routine fix-phase item. The plan greenlit DrainLife without verifying its multiplier semantics; the runner inherited that error. The fix is to revert sanctum_seeker.rs and defer the card.
- Utvara Hellkite missed promotion is HIGH but is a routine completeness gap, not a design issue.

## Previous Findings

N/A — first review of PB-N.

---

## Re-Review (2026-04-12, post-fix commit `0e5d7cf1`)

**Reviewer**: primitive-impl-reviewer (Opus, re-review pass)
**Scope**: verify F1-F6 are resolved; check for regressions and new issues introduced by the fix commit.
**Verdict**: **ready for close** (with one LOW deferred to close commit + one note for follow-up).

### Coordinator preamble acknowledged

- **F3 status**: accepted as **resolved-by-investigation** per coordinator directive. Not re-escalated. The structural BASELINE-LKI-01 limitation (CR 400.7 ObjectId reassignment severs `EffectFilter::SingleObject(old_id)` continuous effects on zone change) is a pre-existing engine gap, not a PB-N defect. Test 6 in `0e5d7cf1` is evaluated against the narrower property "does it exercise the new `triggering_creature_filter` dispatch path on death events" — it does (PASS).
- **abilities.rs:4191-4193 aspirational comment**: known and intentionally deferred to the separate PB-N close commit. NOT flagged here.

### Per-finding verdicts

| # | Original Severity | Status | Evidence |
|---|------|--------|----------|
| F1 | HIGH | **PASS** | `sanctum_seeker.rs:22-46` — `Effect::Sequence([ForEach(EachOpponent, LoseLife 1), GainLife(Controller, 1)])` replaces DrainLife. Oracle text matches. New test `test_sanctum_seeker_flat_gain_4_player` at `pbn_subtype_filtered_triggers.rs:929-1053` asserts `p1_life_after == p1_life_before + 1` in 4-player, plus `p2/p3/p4 == start - 1`. Discriminating against the 3-life DrainLife behavior. No new engine surface added. |
| F2 | HIGH | **PASS** | `utvara_hellkite.rs:21-27` — `filter: Some(TargetFilter { has_subtype: Some(SubType("Dragon".to_string())), .. })`. TODO at lines 19-20 stripped (replaced with PB-N citation). Card-specific test `test_utvara_hellkite_dragon_filter` at `pbn_subtype_filtered_triggers.rs:807-915` covers Dragon-fires + Goblin-no-fire. Yield bumped 4 → 5 as documented. |
| F3 | MEDIUM (treated as fix-HIGH) | **ACCEPTED — RESOLVED BY INVESTIGATION** | Per coordinator directive (see preamble). Test 6 at `pbn_subtype_filtered_triggers.rs:418-468` now uses base-Vampire dying creature and exercises the new `triggering_creature_filter` consumption path on the death-side dispatch (`abilities.rs:4180-4202`). Docstring at lines 405-417 documents the BASELINE-LKI-01 limitation and ESCALATED status. Worker performed the aura-wedge experiment as directed and produced source-level diagnosis. The test validates "the new dispatch path consumes `triggering_creature_filter` correctly" (the load-bearing property that PB-N owns). It does NOT validate "pre-death-vs-post-death LKI evaluation" (structurally impossible pre-LKI-audit). NOT re-escalated. |
| F4 | MEDIUM (treated as fix-HIGH) | **PASS** | `pbn_subtype_filtered_triggers.rs:714-792` — trigger is now `trigger_on: AnyCreatureYouControlAttacks` with `combat_damage_filter: Some(TargetFilter { has_subtype: Some(SubType("Ninja".into())), .. })` and a Goblin attacker. Asserts `stack_trigger_count > 0` after `DeclareAttackers`. The test is now strictly discriminating: pre-fix engine would suppress the trigger (Goblin ≠ Ninja under the now-removed filter check on attacks), post-fix engine ignores `combat_damage_filter` on the attack branch (verified at `abilities.rs:5854-5873`). Docstring at lines 689-712 explicitly documents the regression direction. |
| F5 | MEDIUM (treated as fix-HIGH) | **PASS** | `state/hash.rs:31` — `pub const HASH_SCHEMA_VERSION: u8 = 4` declared with full history doc comment (1 → 4). `state/hash.rs:6146` — actual sentinel `hash_into` call uses `HASH_SCHEMA_VERSION` constant (not literal `4u8`). `lib.rs:30` — `pub use state::hash::HASH_SCHEMA_VERSION` exported. Test 7 at `pbn_subtype_filtered_triggers.rs:544-547` asserts `assert_eq!(HASH_SCHEMA_VERSION, 4u8)`. This is the strict-equality form: a sentinel rollback to 3 changes the constant and the test fails immediately. |
| F6 | LOW | **PARTIAL — see new finding F-N1** | `cargo fmt` was run. Files compile clean. However, the misaligned `filter: None,` insertions visible in the original review (grim_haruspex.rs:27, cruel_celebrant.rs:25, blood_artist.rs:19, etc.) are still present in the working tree. Verified by reading 3 sample files. The runner correctly observed that rustfmt does not normalize these blocks (rustfmt leaves whitespace inside macro-like struct literals alone if the file parses cleanly). This is cosmetic only and does not affect compilation, hashing, dispatch, or game state. The runner's interpretation that this is a "display artifact rustfmt accepts as-is" is mostly correct — but the misalignment is in the actual file bytes, not just a display artifact. Re-classified as a tiny cosmetic LOW that should be hand-fixed in the close commit (F-N1 below). |

### New findings introduced by the fix commit

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| F-N1 | LOW | `crates/engine/src/cards/defs/{grim_haruspex,cruel_celebrant,blood_artist,marionette_apprentice,syr_konrad_the_grim}.rs` | **Mechanical-backfill misalignment persists.** The 5 files identified in the original LOW finding 3 still have visibly mis-indented `filter: None,` lines and trailing closing braces (e.g., `cruel_celebrant.rs:25-26` shows `\t\t\t\t\tfilter: None,\n\t\t\t}`). `cargo fmt` does not normalize these. Cosmetic only — no compilation, hash, or dispatch impact. **Fix:** hand-edit each file to align `filter: None,` with the surrounding fields and close the brace at the correct indent level. Suggest folding into the separate PB-N close commit (per coordinator's standing-rule deferral pattern), NOT requiring another fix-commit cycle. May also be logged as `PB-N-L01` in `docs/mtg-engine-low-issues-remediation.md`. |

No regressions detected. Engine dispatch chain is unchanged. Hash sentinel parity restored. All F1-F6 either resolved or accepted as resolved-by-investigation per coordinator directive.

### Coordinator focus area re-verdicts

| Focus | Original | Re-review |
|-------|----------|-----------|
| 1 (combat_damage_filter tightening) | PARTIAL | **PASS** — F4 fix is now strictly discriminating; pre-fix engine would suppress the trigger, post-fix permits it. |
| 2 (mechanical card-def backfill ~56 files + utvara promotion) | PARTIAL | **PASS** (with F-N1 LOW cosmetic carryover) — utvara_hellkite promoted in-scope per F2; yield bumped 4 → 5. |
| 3 (hash bump parity test) | PARTIAL | **PASS** — F5 fix introduces `HASH_SCHEMA_VERSION` constant and strict `assert_eq!` against value 4. |
| 4 (oracle vs DSL parity, 4 cards → now 5) | PARTIAL | **PASS** — Sanctum Seeker now produces correct flat-1 game state per F1; Utvara Hellkite shipped with Dragon filter per F2. Kolaghan/Dromoka/Teysa unchanged from original review. |

### Test count and build status

- Per wip "Fix Phase Complete" section: 2648 tests passing (delta +2 from 2646 pre-fix baseline).
- Test 6 and Test 9 were REPLACED in-place (same count, more discriminating).
- `cargo fmt --check` clean per wip; verified no compilation regressions in sampled files.
- 8 pre-existing baseline clippy warnings (logged as BASELINE-CLIPPY-0N) — unchanged.

### Stop-and-flag events

NONE. F3 BASELINE-LKI-01 is a pre-existing engine gap, ESCALATED to coordinator during fix phase per documented stop-and-flag protocol; the coordinator's directive in this re-review preamble accepts F3 as resolved-by-investigation. No new stop-and-flag events.

### Final verdict

**READY FOR CLOSE.**

All HIGH and MEDIUM findings are resolved. F3 is accepted as resolved-by-investigation per coordinator directive (BASELINE-LKI-01 to be tracked separately). F-N1 (cosmetic misalignment carryover) is LOW and should be folded into the close commit alongside the abilities.rs:4191-4193 comment update. PB-N is shippable as-is; the close commit can land BASELINE-LKI-01, BASELINE-CLIPPY-0N logging, the abilities.rs comment fix, and (optionally) F-N1 alignment cleanup in a single chore commit.
