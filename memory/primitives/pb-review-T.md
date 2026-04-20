# Primitive Batch Review: PB-T — TargetRequirement::UpToN (optional target slots)

**Date**: 2026-04-20
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 601.2c (variable target count announcement), 115.1/115.1a (target keyword triggers targeting), 608.2b (partial fizzle), 400.7 (object identity across zones)
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (DSL schema)
- `crates/engine/src/state/hash.rs` (hash arm + sentinel bump 7→8)
- `crates/engine/src/rules/casting.rs` (validator + target_count_range helper)
- `crates/engine/src/rules/abilities.rs` (auto-target enumeration arm for UpToN)
- `crates/engine/src/rules/engine.rs` (loyalty ability activate — no change; pre-existing gap)
- `crates/engine/src/effects/mod.rs` (resolve_effect_target_list — unchanged, confirmed UpToN-safe)
- `crates/engine/src/testing/replay_harness.rs` (no TargetRequirement references — confirmed passthrough)
- `crates/simulator/src/legal_actions.rs` (no TargetRequirement references — confirmed)

**Card defs reviewed (14)**:
elder_deep_fiend, force_of_vigor, marang_river_regent, sorin_lord_of_innistrad, basri_ket, tamiyo_field_researcher, teferi_temporal_archmage, tyvar_jubilant_brawler, tyvar_kell, teferi_time_raveler, kogla_the_titan_ape, moonsnare_specialist, skemfar_elderhall, sword_of_sinew_and_steel

**Test file reviewed**: `crates/engine/tests/pbt_up_to_n_targets.rs` (10 tests: M1-M8 + O1-O2)

**Docs reviewed**: `docs/mtg-engine-low-issues-remediation.md` (PB-T-L01/L02/L03 entries)

## Verdict: needs-fix

The PB-T implementation ships a workable UpToN primitive and 14 card-def fixes with correct happy-path semantics, but the review found **1 HIGH** correctness issue (greedy-consume validator rejects CR-legal target declarations when the player declares targets out of slot order), **3 MEDIUM** issues (test-validity HIGH per conventions.md, PB-T-L01 description is factually wrong and understates severity, a pre-existing loyalty-activate gap now demonstrably affects PB-T roster), and multiple LOW items (stale docstrings/comments, CR citation accuracy, missing integration tests, auto-target enumeration nested-UpToN arm, two-parallel-UpToN slot-mapping semantics). The test suite's M1/M2/M3/M6/M7/M8 are solid, and M4 genuinely exercises the hash. Hash bump 7→8 and 3-file sentinel update are correctly done. Fix the HIGH + MEDIUMs before signaling ready.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **HIGH** | `crates/engine/src/rules/casting.rs:5360-5411` | **Greedy-consume validator rejects CR-legal target declarations when declared targets are out of slot order.** For `[UpToN{Planeswalker}, UpToN{Artifact}]` with declared `[artifact_id, planeswalker_id]` (reverse of slot order), slot 0 rejects artifact, slot 1 matches artifact, slot 1's `count=1` is exhausted, planeswalker_id has no slot → rejected. Per CR 601.2c, the player chooses targets; CR does NOT require declaration in slot order. **Fix:** change the mapping algorithm to do a best-fit assignment (e.g., two passes: first assign each target to the *most-restrictive* matching UpToN slot whose capacity isn't exhausted; or backtrack when a slot rejects and a later slot would accept). Simplest correct fix: for each target, find the first slot whose inner accepts it AND has remaining capacity (scanning all slots, not just advancing). Add a test to M7 or new M9 covering reverse-order declaration. |
| E2 | MEDIUM | `crates/engine/src/rules/abilities.rs:6395-6611` | **Auto-target enumeration outer `match req` sends UpToN{inner=TargetPlayer} down the battlefield branch.** Line 6395-6457 routes `TargetPlayer` / `TargetCreatureOrPlayer` / `TargetAny` / `TargetPlayerOrPlaneswalker` to a player-picker branch; everything else (including `UpToN`) falls through to the battlefield scan at line 6458. If a triggered ability with `UpToN { inner: Box::new(TargetPlayer) }` ever ships, auto-target will never find a player target. No PB-T card uses this shape (all are permanent-target UpToNs), so this is latent. **Fix:** route UpToN to the player-picker branch when its inner is a player-targeting requirement; recursively delegate. Minimal patch: add `TargetRequirement::UpToN { inner, .. } if matches!(inner.as_ref(), TargetPlayer \| TargetCreatureOrPlayer \| TargetAny \| TargetPlayerOrPlaneswalker) => { <recurse on inner> }` or mirror the inner dispatch in an `UpToN` arm. |
| E3 | MEDIUM | `crates/engine/src/rules/abilities.rs:6566-6611` | **Nested UpToN inside UpToN falls through to `_ => false`** at the inner match (line 6609). If a card def ever authors `UpToN { inner: Box::new(UpToN { ... }) }` (nonsense per CR, but the DSL permits it), auto-target silently rejects all candidates. **Fix:** either document in `TargetRequirement::UpToN` doc comment that nested UpToN is an author error, or add a debug_assert in validator/hash that inner is not itself UpToN. Preferred: doc-comment convention (nested UpToN is meaningless per CR 601.2c — "up to N target [something]" never nests). |
| E4 | MEDIUM | `crates/engine/src/rules/casting.rs:5275-5278` | **Aspirationally-wrong doc comment** on `validate_targets`: "requirements is indexed in parallel with targets (requirements[i] applies to targets[i])". Post-PB-T, the mapping is via greedy-consume, not parallel indexing. Per `memory/conventions.md` "Aspirationally-wrong code comments are correctness hazards." **Fix:** update the comment to describe the actual mapping (UpToN contributes 0..=count targets; greedy consume; map built before validation). |
| E5 | LOW | `crates/engine/src/cards/card_definition.rs:2306, 2315` | **CR citation inaccuracy** on UpToN variant doc comment: cites "CR 601.2c / 115.1b" but CR 115.1b is about Aura targeting (not "up to N" semantics). The authoritative CR is 601.2c alone (variable number of targets). **Fix:** drop the 115.1b citation from the doc comment and all `PB-T:` hash/validator comments. Keep 601.2c. Alternatively cite 115.1 (general targeting) rather than 115.1b. Same wrong citation appears at `state/hash.rs:42`, `state/hash.rs:4241`, `casting.rs:5540`, `casting.rs:5693-5695`, and most card def comments. Sweep all of them. |
| E6 | LOW | `crates/engine/src/rules/engine.rs:2198-2374` | **`handle_activate_loyalty_ability` does no target validation** — not just for UpToN, but for ANY TargetRequirement. This is a pre-existing engine gap, but PB-T materially exposes it: Sorin −6 now has `UpToN{3, Creature/Planeswalker}`, so activating with non-creature/non-planeswalker targets (e.g., a Land) will succeed and destroy the Land in violation of oracle text. Pre-PB-T, Sorin's −6 was `Effect::Nothing`, so the gap had no effect. **Fix (out of PB-T scope; tracking as PB-T-L01)**: call `validate_targets_with_source(...)` in `handle_activate_loyalty_ability` against the ability's `targets: Vec<TargetRequirement>`. Requires the ability's target list to be threaded through to the validator (currently only the `effect` is captured). **For PB-T**: correct the PB-T-L01 description in `docs/mtg-engine-low-issues-remediation.md` — see finding D1. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `sorin_lord_of_innistrad.rs:64-70` | **Missing "other planeswalkers" exclusion.** Oracle says "creatures and/or **other** planeswalkers" — Sorin should not be targetable as a planeswalker (only when he's a creature via outside effect, per 2011 ruling). Current filter accepts Sorin as a planeswalker target. Filter lacks `exclude_self`. **Fix:** this is a known DSL gap (TargetFilter has no `exclude_self`). Add a TODO annotation on the filter linking back to the Marang River Regent LOW (same DSL gap). Do NOT scope-creep into fixing the filter. |
| C2 | LOW | `marang_river_regent.rs:18-22` | **Missing "other" (exclude_self) filter** on nonland permanent requirement. Oracle says "up to two **other** target nonland permanents." Current filter allows Regent to target itself (cosmetically wrong — Regent is a creature so doesn't satisfy `non_land: true` filter at resolution time, but at cast time Regent IS on battlefield and could match). The card def acknowledges this in a comment. **Fix:** acknowledged as LOW; leave the TODO comment on filter referring to DSL gap. |
| C3 | LOW | `tyvar_kell.rs:51-57` | **Filter uses `TargetCreatureWithFilter` with `has_subtype: Elf`.** Oracle is "up to one target Elf" (no "creature" qualifier). Strict reading: a non-creature Elf permanent (rare but possible via some effects) would qualify as a "target Elf" per oracle. Current impl restricts to creatures. **Fix:** accept as LOW — this matches standard MTG convention where "target Elf" implicitly means "target Elf creature" given Elf is a creature subtype (CR 205.3g). No change required; log for future DSL tightening if an audit surface emerges. |
| C4 | MEDIUM | `sword_of_sinew_and_steel.rs:59-68` | **Two-parallel-UpToN slot semantics** — effect sequence uses `DeclaredTarget{index:0}` for the planeswalker-destroy and `DeclaredTarget{index:1}` for the artifact-destroy, but the validator's greedy-consume maps a lone artifact target to slot 1 (storing mapping[0]=slot1) while the effect still reads `ctx.targets[0]` as the "planeswalker slot target." For symmetric destroy/destroy effects this is benign (destroy is destroy). For any future heterogeneous two-parallel-UpToN card (e.g., "exile up to one target planeswalker and tap up to one target artifact"), the slot-mapping mismatch would apply the wrong effect semantics to targets declared to the later slot. Compound with E1 (reverse-order rejection). **Fix:** not PB-T-blocking for this card (both slots destroy), but flag in the TargetRequirement::UpToN doc comment that `DeclaredTarget{index:N}` is a *positional* reference into `ctx.targets`, NOT a slot-index reference. Document the gotcha so authors of future UpToN cards use effect sequences that match this semantic. Consider a follow-up primitive (UpToN slot-addressed targets) if heterogeneous two-parallel-UpToN cards land in scope. |
| C5 | LOW | `basri_ket.rs:27-43` | Oracle: "Put a +1/+1 counter on up to one target creature. **It** gains indestructible until end of turn." Correctly implemented: effect sequence adds counter to DT{0} then grants Indestructible to DT{0}. When 0 targets declared, both no-op (ctx.targets[0] is None). No issue; noted for completeness. |
| C6 | LOW | All 14 cards | **No integration tests.** All 10 tests in `pbt_up_to_n_targets.rs` use synthetic card defs built inline. A card def could silently be wrong (wrong filter, wrong counter type, wrong ZoneTarget owner) and the test suite wouldn't catch it. PB conventions call for at least one card-integration test per batch. **Fix:** add at least one integration test that uses a real PB-T card def (e.g., Tyvar Jubilant Brawler +1 loyalty ability, since it's a single-target UpToN with a simple Effect::UntapPermanent). Could even be an optional O3 test. |

### Finding Details

#### Finding E1: Greedy-consume validator rejects CR-legal target declarations out of slot order

**Severity**: HIGH
**File**: `crates/engine/src/rules/casting.rs:5360-5411`
**CR Rule**: CR 601.2c — "The player announces their choice of an appropriate object or player for each target the spell requires." — the rule does not prescribe that targets be chosen in slot order.
**Issue**: The greedy-consume algorithm walks requirements left-to-right. For each UpToN slot, it tries to match `targets[target_idx]` against the inner; on failure it breaks and moves to the next slot. For a spell `[UpToN{PW, 1}, UpToN{Artifact, 1}]` declared with `[artifact_id, planeswalker_id]` (reverse order), slot 0 (Planeswalker) rejects `artifact_id`, break; slot 1 (Artifact) accepts `artifact_id`, mapping[0]=slot1, target_idx=1, consumed=1, exhausted; outer loop exits. `mapping.len()=1 < targets.len()=2` → **rejected with "declared 2 targets but only 1 could be matched."** The player's declaration was CR-legal: one artifact (slot 1) and one planeswalker (slot 0). The validator misattributes and rejects. This bug is latent for PB-T cards because Sword of Sinew's effect sequence is symmetric and test O2b only checks single-target casts, but it's a correctness violation of CR 601.2c.
**Fix**:
```rust
// Replace the greedy left-to-right peek with a best-fit assignment pass.
// For each target in declaration order, scan ALL UpToN slots that still have
// capacity; pick the first whose inner accepts it. For mandatory slots, assign
// the Nth un-assigned target to the Nth mandatory slot.
```
A minimal correct algorithm:
1. First pass: assign mandatory-slot targets in declaration order (1 per slot).
2. Second pass: for each remaining target, find the first UpToN slot whose inner accepts it AND consumed<count; assign.
3. If any target is unassigned after both passes, reject.

Add a test (new M9 or extending M7/O2):
```rust
// Test: [UpToN{PW,1}, UpToN{Artifact,1}] with declared [artifact, planeswalker]
// in reverse slot order. Must succeed per CR 601.2c.
```

#### Finding E2: Auto-target enumeration ignores UpToN{inner=TargetPlayer}

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:6395-6458`
**Issue**: The outer match at `let candidate: Option<SpellTarget> = match req` routes player-target requirements (TargetPlayer, TargetCreatureOrPlayer, TargetAny, TargetPlayerOrPlaneswalker) to the player-picker branch; everything else falls into `_ => {battlefield scan}`. For `UpToN { inner: Box::new(TargetPlayer) }`, auto-target will never find a player. No PB-T card uses this shape today.
**Fix**: add a pre-check at line 6395:
```rust
let candidate: Option<SpellTarget> = match req {
    // ... existing arms ...
    TargetRequirement::UpToN { inner, .. } => {
        // Delegate recursively on inner by re-matching.
        // (Or: extract inner.as_ref() and match on it.)
        let inner_req = inner.as_ref();
        match inner_req {
            TargetRequirement::TargetPlayer | ... => { /* player-picker */ },
            _ => { /* battlefield scan */ }
        }
    }
    _ => { /* battlefield scan */ }
}
```

#### Finding E3: Nested UpToN { inner: UpToN { ... } } is silently rejected

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:6609`
**Issue**: The inner match at 6576 handles UpToN's inner, falling through to `_ => false` for nested UpToN. Hash arm in `state/hash.rs:4242-4246` recursively hashes inner, so nested UpToN IS representable in-memory and persists across serialization. Validator in casting.rs:5696 also recurses — validation would work correctly. Only auto-target is broken.
**Fix**: add a doc comment on `TargetRequirement::UpToN` explicitly stating that `inner` must not itself be an `UpToN` variant. Optional: add a debug_assert in `target_count_range` or the hash arm.

#### Finding E4: Aspirationally-wrong doc comment on validate_targets

**Severity**: MEDIUM (per conventions.md: aspirationally-wrong comments are hazards)
**File**: `crates/engine/src/rules/casting.rs:5275-5278`
**Issue**: The doc says "`requirements` is indexed in parallel with `targets` (requirements[i] applies to targets[i])". After PB-T, this is no longer true — UpToN contributes 0..=count to the target-consumption count, and the mapping is computed via a greedy algorithm.
**Fix**: rewrite the doc comment to describe actual behavior:
```
/// Requirements are mapped to targets via a greedy-consume algorithm:
/// mandatory slots consume exactly 1 target; UpToN{count, inner} slots consume
/// 0..=count consecutive targets that match `inner`. See `target_count_range`
/// for range computation and `validate_targets_inner` for the algorithm.
```

#### Finding E5: CR 115.1b citation is incorrect

**Severity**: LOW (but pervasive — 20+ sites)
**Files**: multiple (see table above)
**Issue**: CR 115.1b is about Aura spells being targeted via the Enchant keyword ability — nothing to do with "up to N target" semantics. The authoritative rule for variable-target counts is CR 601.2c alone ("If the spell has a variable number of targets, the player announces how many targets they will choose"). Independently verified via MCP `get_rule 115.1` with children.
**Fix**: grep for `115.1b` across the codebase and drop the citation everywhere it appears in PB-T code and card defs. Keep CR 601.2c as the sole authority. Suggested sweep:
```
crates/engine/src/cards/card_definition.rs:2306, 2315
crates/engine/src/state/hash.rs:42, 4241
crates/engine/src/rules/casting.rs:5313, 5540, 5693-5695
crates/engine/src/rules/abilities.rs:6565
crates/engine/src/cards/defs/{basri_ket,elder_deep_fiend,force_of_vigor,marang_river_regent,sorin_lord_of_innistrad,tamiyo_field_researcher,teferi_temporal_archmage,tyvar_jubilant_brawler,tyvar_kell,teferi_time_raveler,kogla_the_titan_ape,moonsnare_specialist,skemfar_elderhall,sword_of_sinew_and_steel}.rs
crates/engine/tests/pbt_up_to_n_targets.rs (line 17, 400, 408, 412, 867)
```

#### Finding E6: Pre-existing loyalty-activate target-validation gap materially affects PB-T roster

**Severity**: LOW (acknowledged in PB-T-L01, but description needs correction — see D1)
**File**: `crates/engine/src/rules/engine.rs:2198-2374`
**Issue**: `handle_activate_loyalty_ability` does NOT call `validate_targets`/`validate_targets_with_source`/`validate_object_satisfies_requirement` at any point — it only converts `Vec<Target>` to `Vec<SpellTarget>` without type/filter checks. Pre-existing. **PB-T exposes it** because Sorin −6 now has `UpToN{3, Creature/Planeswalker}` (was `Effect::Nothing`), Basri +1 has `UpToN{1, Creature}`, Tyvar Kell has `UpToN{1, Elf}`, etc. A user can activate any of these loyalty abilities with non-matching targets and the effects will run against whatever they passed. Example: Tyvar Kell's +1 with a non-Elf target → non-Elf gets +1/+1 counter, untaps, gains deathtouch. Wrong game state.
**Fix (out of scope for PB-T)**: thread ability targets into the validator — add `ability.targets` to the `AbilityDefinition::LoyaltyAbility` destructuring at line 2266, then call `validate_targets_with_source(state, &targets, &ability_targets, player, source_chars, source)` before pushing to the stack. Tracked as PB-T-L01.

## Card Definition Details

| Card | Oracle Match | TODOs | Game State Correct | Notes |
|------|:---:|:---:|:---:|-------|
| elder_deep_fiend | Yes | 0 | Yes | UpToN{4, Permanent} + 4 TapPermanent effects. Clean. |
| force_of_vigor | Partial | 1 (pitch cost) | Yes for destroy half | UpToN{2, artifact/enchantment} fixed. Pitch-alt-cost TODO retained (correctly out of PB-T scope). Oracle match for destroy half is verbatim. |
| marang_river_regent | Partial | implicit (exclude_self) | Yes for bounce | UpToN{2, nonland}. Lacks "other" filter — see C2. ETB trigger authored correctly. |
| sorin_lord_of_innistrad | Partial | 1 (reanimate rider, PB-T-L02) | Partial (see E6) | Destroy half correctly authored with UpToN{3, creature/planeswalker}. Reanimate rider deferred. Loyalty activate gap means non-creature targets won't be rejected (E6). Also lacks "other" on planeswalker clause (C1). |
| basri_ket | Yes | 0 (the UpToN ability) | Yes | UpToN{1, Creature} + AddCounter(+1/+1) + grant Indestructible. Per 2011-ruling-style behavior, 0 targets → no-op is correct. Oracle match. |
| tamiyo_field_researcher | Partial | 1 (PB-T-L03, freeze rider) | Partial (tap works; freeze missing) | UpToN{2, nonland} + 2 TapPermanent. Freeze rider correctly deferred. Oracle match for tap half. |
| teferi_temporal_archmage | Yes | 0 (−1 ability) | Yes | UpToN{4, Permanent} + 4 UntapPermanent. Clean. |
| tyvar_jubilant_brawler | Yes | 0 (+1 ability) | Yes | UpToN{1, Creature} + UntapPermanent(DT{0}). Clean. |
| tyvar_kell | Yes | 0 (+1 ability) | Yes | UpToN{1, TargetCreatureWithFilter{subtype:Elf}} + counter + untap + deathtouch. Clean (modulo C3 LOW). |
| teferi_time_raveler | Yes | 0 (−3 ability) | Yes | UpToN{1, artifact/creature/enchantment} + MoveZone + DrawCard. Draw fires regardless of target count (correct per oracle and CR 601.2c). |
| kogla_the_titan_ape | Yes | 0 (ETB ability) | Yes | UpToN{1, creature you don't control} + Fight. Clean. |
| moonsnare_specialist | Yes | 0 | Yes | UpToN{1, Creature} + MoveZone. Clean. |
| skemfar_elderhall | Yes | 0 | Yes | UpToN{1, creature you don't control} + ModifyBoth(-2) + CreateToken. Clean. Token half always fires. |
| sword_of_sinew_and_steel | Yes (see C4) | 0 | Yes for this card; latent for heterogeneous two-parallel-UpToN | Two UpToN slots (PW+Artifact) + 2 DestroyPermanent. Symmetric effects → correct. Heterogeneous slot mapping is a latent gotcha — see C4. |

## Test Correctness Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| T1 | MEDIUM (test-validity HIGH per conventions.md) | `test_pbt_up_to_n_partial_fizzle_on_zone_change` (M5) | Test name says "partial fizzle on zone change" but the setup has **no zone change**. The test declares 1 of up-to-2 targets (like M2) and verifies the 1 declared target is tapped. This is a partial-target-resolution test, not a partial-fizzle test. Per `memory/conventions.md`: test-validity MEDIUMs are fix-phase HIGHs. The test as written does not discriminate "CR 608.2b partial-fizzle on zone change" vs "partial declaration at cast time." **Fix:** rewrite the test to actually cause a zone change mid-resolution — e.g., declare 2 targets, then bounce/destroy one before the spell resolves, then assert only the surviving target is tapped. Or rename to `test_pbt_up_to_n_partial_target_declaration` to match what it actually tests. |
| T2 | MEDIUM | No integration test uses a PB-T card def | All 10 tests use synthetic card defs. A regression in any of the 14 card defs would not be caught by this suite. Per PB pattern (PB-N, PB-L), a card-integration test is expected. **Fix:** add one test (could be M9 or O3) that uses Tyvar Jubilant Brawler (or similar simple card) and confirms end-to-end behavior. |
| T3 | LOW | `test_pbt_hash_schema_version_is_8` (M4) | The test asserts `HASH_SCHEMA_VERSION == 8` and three distinct UpToN-variant hashes, meeting the requirement. One minor gap: does not assert a UpToN hash differs when the `inner` is wrapped in an equivalent shape (e.g., `UpToN{1, TargetCreatureWithFilter(empty filter)}` vs `UpToN{1, TargetCreature}`). Non-blocking. |
| T4 | LOW | `test_pbt_two_parallel_up_to_n_slots` (O2) | O2b asserts success for "1 artifact target against `[UpToN{PW}, UpToN{Artifact}]`" but does NOT assert which slot the target was mapped to, or whether the resulting effect sequence behaves correctly. For Sword of Sinew (destroy/destroy) this is benign, but the test doesn't gate the C4 semantic concern. **Fix:** acceptable as-is since PB-T card is symmetric; no gate required for PB-T scope. Noted. |

## Documentation Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| D1 | MEDIUM | `docs/mtg-engine-low-issues-remediation.md:417` (PB-T-L01) | **Description is factually wrong.** Claims: "it hard-checks `targets.len() != ability.targets.len()` which fails for UpToN slots." No such check exists — `handle_activate_loyalty_ability` does NOT validate targets against requirements at all. Also claims "No loyalty ability currently uses UpToN (Sorin -6 uses it but is an ETB trigger, not a loyalty activate)" — Sorin's −6 IS a `AbilityDefinition::LoyaltyAbility` (`sorin_lord_of_innistrad.rs:57`), not an ETB trigger. So multiple PB-T loyalty abilities (Sorin −6, Basri +1, Tamiyo −2, Teferi Temporal Archmage −1, Tyvar Jubilant Brawler +1, Tyvar Kell +1) are affected by this gap. **Fix:** rewrite the PB-T-L01 entry to accurately describe the gap (zero target validation in loyalty activation path) and list the affected cards. The fix description ("apply `target_count_range` range-gate at loyalty-ability validation site") should be changed to "call `validate_targets_with_source` with the ability's target requirements." |
| D2 | LOW | `crates/engine/tests/pbp_power_of_sacrificed_creature.rs:767-768` | Docstring says "asserts HASH_SCHEMA_VERSION == 6" but the assertion is now for 8. Aspirationally-wrong. **Fix:** update to "asserts HASH_SCHEMA_VERSION == 8 (PB-T sentinel)". |
| D3 | LOW | `crates/engine/tests/pbd_damaged_player_filter.rs:588` | Docstring says "HASH_SCHEMA_VERSION is exactly 5 (the PB-D bump from 4)" but assertion is now for 8. **Fix:** update to reflect current sentinel. |
| D4 | LOW | `crates/engine/src/cards/defs/skullsnatcher.rs:31-32` | TODO comments are aspirationally wrong: "TargetController::DamagedPlayer not in DSL" (PB-D shipped it) and "up to two → no UpToN target variant" (PB-T ships it). Card remains DEFERRED due to compound blockers, but the specific claims in the comments are now false. **Fix:** rewrite the TODO comments to accurately describe the remaining gap (per plan Step 0: "graveyard-side dispatch of DamagedPlayer not wired; UpToN now available but card still blocked on the combined filter"). |
| D5 | LOW | `crates/engine/src/cards/defs/bridgeworks_battle.rs:17-18` | TODO says "'up to one' optional targeting is not yet supported in the DSL" — now false. **Fix:** this card was DEFERRED (optional, rejected during planning). Rewrite the comment to reflect that UpToN exists and the card is authored in mandatory-target form by choice (per plan Risk section). Or author it properly; the plan said deferral was "rejected during planning" on yield-risk grounds but UpToN now exists. |
| D6 | LOW | `crates/engine/src/cards/defs/glissa_sunslayer.rs:7-11` | Partial aspirationally-wrong: "no 'up to N' count modifier" is still correct (Glissa's gap is about *counter* count, not target count); "no any-type or multi-type counter removal effect" is correct; but the phrasing could be read as implying target-count UpToN is missing too. **Fix:** minor rewording to clarify counter-count UpTo vs target-count UpToN. Non-blocking. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|:---:|:---:|-------|
| 601.2c (variable target count announcement) | Yes | M1, M2, M3, M6, M7, O2 | Main authoritative rule. Zero-target, partial, and full-count covered; mandatory regression covered (M6). |
| 115.1 (general targeting) | Yes (inherits) | Implicit | UpToN delegates to inner requirement, which carries the targeting semantics (hexproof, protection, shroud). |
| 115.1a (target keyword triggers targeting) | Not PB-T scope | — | Non-targeted "up to N" effects excluded per plan. |
| 115.1b (Aura spells targeting) | MISCITED | — | Plan and code cite 115.1b as authority for "up to N" — INCORRECT. 115.1b is about Auras. See finding E5. |
| 608.2b (partial fizzle) | Yes (inherits from pre-existing legal_targets filter) | M5 (partial — see T1) | Existing behavior; no PB-T changes. M5 is test-invalid for what it claims to test (see T1). |
| 400.7 (object identity on zone change) | Yes (inherits) | — | No dedicated test for UpToN object identity; same behavior as pre-PB-T. |

## Dispatch Chain Walk (independent verification per feedback_verify_full_chain.md)

| Site | File | Claim in plan | Independent verdict |
|------|------|---------------|---------------------|
| 1. DSL schema | `card_definition.rs:2321-2324` | UpToN variant added | CONFIRMED — clean addition, no enum renumbering. |
| 2. Validator | `casting.rs:5331-5411` | greedy-consume + length-range check + post-mapping length check | CONFIRMED (with E1 bug) |
| 3. Target declaration | `casting.rs:3380-3397` | no change required | CONFIRMED via grep — handle_cast_spell calls validate_targets_with_source. |
| 4. Resolve effect targets | `effects/mod.rs:5323-5363` | no change required | CONFIRMED — `ctx.targets.get(idx)` returns None for out-of-bounds; all call sites handle None as empty list. |
| 5. Hash | `state/hash.rs:4242-4246` | arm + schema bump 7→8 | CONFIRMED. Exhaustive match (no `_ =>`). Three test files updated to 8u8. |
| 6. Replay harness | `testing/replay_harness.rs` | no pre-validation | CONFIRMED via grep (zero TargetRequirement references; passthrough). |
| 7. LegalActionProvider | `simulator/src/legal_actions.rs` | no change | CONFIRMED via grep (zero TargetRequirement references). |
| 8. TargetProvider | test helpers | no change | CONFIRMED. |
| 9. Loyalty ability | `engine.rs:2198-2374` | no change; pre-existing LOW | CONFIRMED, but E6/D1 reveals the "pre-existing LOW" description is inaccurate and the gap materially affects PB-T cards. |
| 10. Resolution legal_targets | `resolution.rs` | no change | CONFIRMED via plan; `stack_obj.targets` filter is type-agnostic re: UpToN. |
| **Bonus**: auto-target (abilities.rs) | `abilities.rs:6395-6611` | arm added | CONFIRMED for permanent-inner UpToN. Latent issues E2 (player-inner UpToN) and E3 (nested UpToN). |

## Previous Findings (re-review only)

N/A — this is first review.

## Severity Summary

- **HIGH**: 1 (E1 — greedy validator rejects CR-legal reverse-order declarations)
- **MEDIUM**: 5 (E2, E3, E4, T1, T2, D1 — 6 total actually but E4 and D1 paired; 5 distinct concerns to fix)
- **LOW**: ~11 (E5 pervasive CR citation, E6 pre-existing, C1, C2, C3, C5, C6, D2, D3, D4, D5, D6, T3, T4)

## Actions Required Before Signaling Ready

1. **E1 (HIGH)**: rewrite `validate_targets_inner` mapping to handle out-of-slot-order target declarations. Add a regression test.
2. **T1 (MEDIUM, per conventions.md = HIGH-equivalent)**: rewrite or rename `test_pbt_up_to_n_partial_fizzle_on_zone_change`. Either do a real zone-change mid-resolution test, or rename to match what it actually tests.
3. **D1 (MEDIUM)**: rewrite the PB-T-L01 entry in `docs/mtg-engine-low-issues-remediation.md` to accurately describe the loyalty-activate validation gap and list the affected PB-T cards.
4. **E2/E3 (MEDIUM)**: add auto-target UpToN{inner=TargetPlayer} routing, or explicitly document that nested/player-inner UpToN is not supported. Preferred: minimal patch to route UpToN based on inner.
5. **E4 (MEDIUM)**: rewrite the `validate_targets` doc comment to describe actual post-PB-T behavior (not parallel indexing).
6. **T2 (MEDIUM)**: add one card-integration test using a real PB-T card def (Tyvar Jubilant Brawler recommended).

LOW items can be addressed opportunistically (E5 citation sweep is good hygiene; stale docstrings/comments can be batched).

## Reviewer notes on the shape decision

Shape A (enum wrapper) is the right call — dispatch impact is minimal, and the existing `ctx.targets.get(idx)` robustness does heavy lifting. The one real issue with Shape A is the implicit assumption that target declaration order matches slot order (finding E1) — which isn't inherent to Shape A but is an artifact of the greedy-consume mapping. A more robust mapping algorithm fixes this without changing the shape. Shapes B/C would have required more invasive refactoring for no clear benefit.

---

## Re-review 2026-04-20

**Reviewer**: primitive-impl-reviewer (Opus)
**Fix commits reviewed**: `fd1f2f0c`, `a9b4df6e`, `83b187b9`
**Files re-read**:
- `crates/engine/src/rules/casting.rs` (validate_targets, validate_targets_inner, target_count_range, validate_object_satisfies_requirement UpToN arm)
- `crates/engine/src/rules/abilities.rs` (auto-target UpToN outer match arm)
- `crates/engine/src/cards/card_definition.rs` (TargetRequirement::UpToN doc comment)
- `crates/engine/tests/pbt_up_to_n_targets.rs` (M5 renamed, M9 new, M10 new, O3 new)
- `docs/mtg-engine-low-issues-remediation.md` (PB-T-L01 rewrite)

### Previous Findings Status

| # | Severity | Previous Status | Current Status | Notes |
|---|----------|----------------|----------------|-------|
| E1 | HIGH | OPEN | **RESOLVED** | Two-pass best-fit algorithm replaces greedy-consume. Pass 1 (casting.rs:5394-5409) assigns mandatory slots; Pass 2 (5411-5428) assigns remaining targets to UpToN slots with capacity. M10 regression test (`test_pbt_up_to_n_reverse_order_declaration_succeeds`) exercises `[artifact, PW]` against `[UpToN{PW}, UpToN{Artifact}]` and asserts success. Analytically traced: artifact → si=0 rejects → si=1 accepts (cap→1); PW → si=0 accepts (cap→1). No unassigned. Correct. M6 mandatory regression preserved (count-range check still runs first at 5351-5359, then Pass 1 assigns single target to the mandatory slot). |
| E2 | MEDIUM | OPEN | **RESOLVED** | `abilities.rs:6463-6499` now routes `TargetRequirement::UpToN { inner, .. }` with player-inner to the player-picker; permanent-inner returns None (skip optional slot — matches author intent of "auto-target triggers contribute 0 to optional slots"). Code inspection confirms correct dispatch for all four player-target variants. |
| E3 | MEDIUM | OPEN | **RESOLVED (with caveat)** | `card_definition.rs:2319-2322` adds author invariant: "`inner` MUST NOT itself be `UpToN`". Runner chose doc-invariant-only over debug_assert. **Caveat**: `memory/conventions.md` documents debug_assert as the stronger pattern, but (a) no real MTG card would nest UpToN (CR 601.2c phrasing "up to N target X" never nests), (b) auto-target is the only broken site — validator and hash both recurse correctly, (c) precedent from other TargetRequirement variants is doc-comment-only (e.g. no debug_assert for nonsensical filter combinations elsewhere). **Accept invariant-only**: acceptable for an author-facing constraint with no real-card trigger. Not downgrading to NEW LOW. |
| E4 | MEDIUM | OPEN | **RESOLVED** | `casting.rs:5269-5288` doc comment rewritten. New wording describes two-pass best-fit algorithm, cites CR 601.2c, explicitly notes "declaration order is NOT required to match slot order". No mention of "parallel indexing". Pass 1 and Pass 2 semantics documented. |
| T1 | MEDIUM (test-validity HIGH) | OPEN | **RESOLVED** | Original M5 renamed to `test_pbt_up_to_n_partial_target_declaration_resolves` (tests/pbt_up_to_n_targets.rs:482); doc comment now honestly describes partial-declaration semantics. New M9 `test_pbt_up_to_n_partial_fizzle_on_zone_change` (tests/pbt_up_to_n_targets.rs:1025-1134) genuinely exercises CR 608.2b zone-change fizzle: P1 casts Tap Up To Two targeting A+B → P2 casts Destroy Creature on A (with_types Instant so castable at instant speed) → Destroy resolves first, A→graveyard → Tap resolves, A illegal (skipped), B tapped. **Verified assertions**: line 1106 asserts A in graveyard (post-Destroy), line 1110 asserts B still on battlefield (post-Destroy), line 1121 asserts A STILL in graveyard after Tap resolves (wasn't untapped/resurrected), line 1130 asserts B.status.tapped=true. The assertions genuinely discriminate the bug where UpToN would either fail to tap B (no-op on partial fizzle) or try to tap A (phantom-tap in graveyard). Solid. |
| T2 | MEDIUM | OPEN | **RESOLVED** | New O3 `test_pbt_force_of_vigor_card_integration` (tests/pbt_up_to_n_targets.rs:1254-1312) uses `mtg_engine::cards::defs::force_of_vigor::card()` registered via CardRegistry::new. Mana pool `colorless: 2, green: 2` matches `ManaCost { generic: 2, green: 2 }`. Casts with 1 of 2 UpToN targets; asserts Artifact Alpha→graveyard, Artifact Beta→battlefield. Exercises full path: registry lookup → enrich_spec_from_def → validate_targets (UpToN arm) → resolve (Effect::Sequence with DeclaredTarget{0,1}, index 1 falls through as None and no-ops). |
| D1 | MEDIUM | OPEN | **RESOLVED** | `docs/mtg-engine-low-issues-remediation.md:417` completely rewritten. New entry: (a) names `handle_activate_loyalty_ability` as the gap location, (b) accurately describes zero target validation (no call to validate_targets/validate_targets_with_source/validate_object_satisfies_requirement), (c) lists all 6 affected PB-T cards (Sorin −6, Basri +1, Tamiyo −2, Teferi Temporal Archmage −1, Tyvar Jubilant Brawler +1, Tyvar Kell +1), (d) points to fix approach (thread `ability.targets` through and call `validate_targets_with_source`), (e) cites file:line `engine.rs:2198-2374`. The prior factually-wrong claims (hard-check of `targets.len() != ability.targets.len()`, Sorin is ETB trigger) are both removed. |

### LOW Findings Status (informational — not fix-blocking)

Per standing rules, LOW findings are not fix-required. Runner noted in WIP (line 198) that LOWs E5, E6, C1-C4, D2-D6, T3-T4 remain open for opportunistic cleanup. This re-review confirms those are still present and tracks-them-correctly in the remediation doc. No action required.

### Gates verification

I was unable to execute `~/.cargo/bin/cargo test --all`, `cargo fmt --check`, `cargo build --workspace`, or `cargo clippy --all-targets` from this reviewer session (no Bash tool access). Gate verification is by code inspection only:

- **cargo test**: reviewer note — runner reported 0 failures / 269 suites. The 4 new/renamed test functions (M5 renamed, M9, M10, O3) are well-formed Rust code with matching imports (CardDefinition, CardType, TypeLine, Effect, EffectTarget, TargetRequirement, AbilityDefinition, ManaCost all imported at top of file). M9 uses `with_types(vec![CardType::Instant])` which matches ObjectSpec::with_types signature (grep confirmed use elsewhere in test file). Force of Vigor card def has `mana_cost: Some(ManaCost { generic: 2, green: 2, .. })` matching O3's mana pool `colorless: 2, green: 2` (generic cost is paid with colorless). Code inspection finds no obvious compile blockers.
- **cargo fmt**: the append to `crates/engine/tests/pbt_up_to_n_targets.rs` follows rustfmt conventions (consistent indentation, trailing commas in multi-line vecs, ManaCost/ManaPool struct literal formatting matches existing code in the file).
- **cargo build --workspace**: the abilities.rs outer match still ends with `_ =>` catch-all at line 6501 — UpToN arm is added above it. The inner match in validate_object_satisfies_requirement still has the UpToN arm at line 5731 with explicit `return`. Both are syntactically valid.
- **cargo clippy**: runner notes "0 new lints introduced in modified files" (WIP line 199). Pre-existing `#[allow(clippy::collapsible_match)]` attributes from implement phase remain valid.

**Recommendation for coordinator**: run the full gate suite before collection. Code-inspection confidence is high, but uncertainty exists around rare compile/clippy regressions that only surface at build time.

### Full dispatch-chain verification (per feedback_verify_full_chain.md)

I walked all callers of `validate_targets` / `validate_targets_with_source` / `validate_targets_inner`:

- `casting.rs:3397` — `handle_cast_spell` calls `validate_targets_with_source`. Now receives two-pass best-fit mapping. Pre-existing callers pass mandatory-only requirements → Pass 1 assigns all, Pass 2 is no-op. Identical to pre-fix behavior for non-UpToN spells.
- `abilities.rs:327` — `handle_activate_ability` calls `validate_targets`. Same reasoning: mandatory-only requirements still work identically.

Both callsites continue to work correctly with the new two-pass algorithm. The algorithm is strictly more permissive than greedy-consume (accepts everything greedy did, plus reverse-order UpToN declarations).

### New LOW findings from re-review

None. The runner addressed all HIGH + MEDIUM findings correctly; introduced no new issues.

### Final verdict

**PASS** — All HIGH + MEDIUM findings resolved. Engine changes correct; tests genuinely exercise what they claim; doc fix for D1 is accurate and complete; E3 invariant-only resolution is acceptable for an author-facing constraint with no real-card trigger.

Ready for coordinator collection subject to gate verification (`cargo test --all`, `cargo fmt --check`, `cargo build --workspace`, `cargo clippy --all-targets`). Code-inspection confidence: high.

LOW findings (~11) remain tracked in `docs/mtg-engine-low-issues-remediation.md` for opportunistic cleanup.
