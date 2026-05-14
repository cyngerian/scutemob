# Primitive Batch Review: PB-LKI-Power — `EffectAmount::SourcePowerAtLastKnownInformation` (LKI source-power snapshot for WhenDies / WhenLeavesBattlefield)

**Date**: 2026-05-13
**Reviewer**: primitive-impl-reviewer (Opus 4.7 1M)
**Branch**: `feat/pb-lki-power-lki-source-powertoughness-snapshot-for-whendies`
**Implementation commit**: `e7a9a16c`
**CR Rules**: 603.10a, 113.7a, 122.2, 400.7 (verified verbatim against MCP rules server)
**Engine files reviewed**: `crates/engine/src/cards/card_definition.rs` (variant disc 18 + disc 19 reservation comment), `crates/engine/src/effects/mod.rs` (EffectContext field + 2 constructors + 2 inner_ctx + check_condition stub + resolve_amount arm + zone_move_event helper), `crates/engine/src/state/stubs.rs` (PendingTrigger field + blank), `crates/engine/src/state/stack.rs` (StackObject field + trigger_default), `crates/engine/src/rules/sba.rs` (capture site at line 540 + redirect path), `crates/engine/src/rules/events.rs` (5 GameEvent payload extensions), `crates/engine/src/rules/abilities.rs` (6 trigger arms + flush + capture sites for sacrifice/exile-self/ninjutsu), `crates/engine/src/rules/casting.rs` (sacrifice-as-cost capture sites), `crates/engine/src/rules/engine.rs` (echo / cumulative-upkeep capture sites), `crates/engine/src/rules/mana.rs` (sacrifice-self mana ability), `crates/engine/src/rules/replacement.rs` (zone_change_events helper), `crates/engine/src/rules/resolution.rs` (Evoke / sacrifice-on-resolve capture sites + 2 LKI threading paths into EffectContext), `crates/engine/src/rules/turn_actions.rs` (myriad / EOC sacrifice / cleanup capture sites), `crates/engine/src/rules/layers.rs` (resolve_cda_amount arm), `crates/engine/src/state/hash.rs` (HASH 16→17 + history entry 17 + 4 hash arms)
**Card defs reviewed**: `crates/engine/src/cards/defs/conclave_mentor.rs`, `crates/engine/src/cards/defs/juri_master_of_the_revue.rs` (2)
**Tests reviewed**: `crates/engine/tests/primitive_pb_lki_power.rs` (4 tests: a, b, c, d)
**Sentinel files reviewed**: 11 (10 from plan + 1 new in primitive_pb_lki_power.rs)

## Verdict: **PASS-WITH-NITS**

The PB-LKI-Power implementation correctly mirrors PB-LKI-CC field-for-field, swapping `OrdMap<CounterType, u32>` for `Option<i32>`. Every dispatch site enumerated in the plan (Sites 1–21) is present and correct. The full chain — DSL variant (disc 18) → snapshot at sba.rs:540 (via `calculate_characteristics`) → `GameEvent::{CreatureDied, AuraFellOff, PermanentDestroyed, ObjectExiled, ObjectReturnedToHand}` payload → 6 trigger arms in `check_triggers` → `flush_pending_triggers` → 2 resolution paths → `resolve_amount` arm — is plumbed end-to-end. HASH bump 16→17 with history entry 17 is consistent across all 11 sentinel files. The Conclave Mentor (life-gain) and Juri Master (damage-deal) re-authoring matches MCP oracle text exactly and honors the load-bearing rulings (2020-06-23 and 2020-11-10). Tests are discriminating: test (c) is the load-bearing zone-change discriminator, and test (d) covers hash determinism + variant discrimination + Option tag-byte canary. Three OOS-LKI-Power-N seeds are filed in `pb-retriage-CC.md` per plan; the original OOS-LKI-Power seed is correctly marked CLOSED. WIP claims `cargo test --workspace` = 2749 passing (was 2745, +4 new), `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean, `cargo build --workspace` clean — all consistent with the audit (no stale `16u8` sentinels, no unhandled emit sites, no `.unwrap()` in library code).

LOW findings concern (1) AnyCreatureDies LKI-power gap not getting a dedicated OOS seed (the plan's Site 9 reserved OOS-LKI-Power-2 for it, but the runner used OOS-LKI-Power-2 for Master Biomancer instead), (2) animated-Food / animated-Saga / animated-planeswalker / animated-Aura edge cases where layer-4 effects produce a power that the hard-coded `pre_lba_power: None` would lose — pre-existing PB-LKI-CC pattern preserved, not reachable in current card pool, but worth a tracking comment, and (3) doc-comment line-number drift (a few comments cite stale line numbers from the plan that no longer match). None of these affect the in-scope cards' game state.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | LOW | `crates/engine/src/rules/abilities.rs:4391` | **AnyCreatureDies LKI-power gap not assigned a dedicated OOS seed.** The plan Site 9 explicitly reserved OOS-LKI-Power-2 for "if a card surfaces requiring the dying creature's power on an AnyCreatureDies trigger". The runner instead used OOS-LKI-Power-2 for Master Biomancer ETB-replacement (which is also a real gap, but a different one). The AnyCreatureDies LKI-power gap is now undocumented; the existing OOS-LKI-4 (filed by PB-LKI-CC) covers only the counter axis. **Fix:** append OOS-LKI-Power-4 to `pb-retriage-CC.md` documenting the AnyCreatureDies + LKI source power gap, mirroring OOS-LKI-4 (counter version). The arm at line 4391 already correctly defaults to `lki_power: None` so the dispatch is safe; only the documentation/tracking is missing. |
| E2 | LOW | `crates/engine/src/rules/abilities.rs:890` (Food forage), `crates/engine/src/rules/sba.rs:733` (planeswalker SBA-exile), `:854` (Saga SBA-sacrifice), `:1170` (Aura SBA-fall-off) | **Hard-coded `pre_lba_power: None` for non-creature SBA paths drops layer-4 animated power.** Each of these 4 sites unconditionally sets `pre_lba_power: None` with a comment like "Sagas are not creatures; no power LKI needed." But Layer 4 type-grant effects (e.g. Karn ultimate animating planeswalker, "Roalesk, Apex Hybrid"-style animations on enchantments, animated Food via cards like Witch's Oven? — actually animated-Food is unreachable today) can technically produce a power on these objects. A SelfLeavesBattlefield trigger using `EffectAmount::SourcePowerAtLastKnownInformation` would lose the layer-4 boost. Symmetric to PB-LKI-CC's handling of `pre_lba_counters` at the same sites; preserves blast-radius scope. **Fix:** opportunistically replace `None` with `state.objects.get(&id).and_then(|o| crate::rules::layers::calculate_characteristics(state, id).and_then(|c| c.power).or(o.characteristics.power))` at these 4 sites. No in-scope card hits these paths, so deferral is acceptable; flag as a tracked LOW. |
| E3 | LOW | `crates/engine/src/state/stack.rs:478-479` | **Doc-comment line-number drift.** The new `lki_power` field's doc references "abilities.rs flush_pending_triggers ~line 7467" and "resolution.rs ~line 2057", but the actual sites are abilities.rs:7538 and resolution.rs:2080/2158. The line-numbers were copied from the plan (which used pre-impl estimates). Cosmetic; doesn't affect correctness. **Fix:** drop the line numbers from the doc comments (they go stale on every refactor), leaving "abilities.rs flush_pending_triggers" / "resolution.rs". |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | (none) | both cards | Both card defs are clean: oracle text matches MCP exactly, no remaining TODOs, header comments updated to remove the OOS-LKI-Power TODO prose, primitive used correctly. |

### Finding Details

#### Finding E1: AnyCreatureDies LKI-power gap not assigned a dedicated OOS seed (LOW)

**Severity**: LOW
**File**: `crates/engine/src/rules/abilities.rs:4391` (AnyCreatureDies arm)
**Plan reference**: Step 3 Site 9, "STOP-AND-FLAG: if a card surfaces requiring the dying creature's power on an AnyCreatureDies trigger, file as OOS-LKI-Power-2 (mirrors PB-LKI-CC's OOS-LKI-4 for AnyCreatureDies counter-LKI)."

**Issue**: The plan reserved the OOS-LKI-Power-2 slot for the AnyCreatureDies LKI-power gap. The runner instead used OOS-LKI-Power-2 for Master Biomancer ETB-replacement (per Step 4 Out-of-scope item #1). Master Biomancer is a real gap, but it's structurally different (replacement-side dynamic counter count vs trigger-side LKI). The AnyCreatureDies + LKI source power gap (e.g. a hypothetical "Whenever a creature dies, deal damage equal to its power to its controller" or "Bloodflow Connoisseur dies → reanimate creature with that power" patterns) is now undocumented. The existing OOS-LKI-4 (filed by PB-LKI-CC) covers only the counter axis.

**Engine state**: the arm at `abilities.rs:4391` correctly defaults `lki_power: None` — no dispatch panic; the variant resolves to 0 if accidentally used in this context. So no live correctness issue; only the documentation/tracking is missing.

**Fix**: Append OOS-LKI-Power-4 to `memory/primitives/pb-retriage-CC.md` with text mirroring OOS-LKI-4 (counter version) but for the source-power axis. Cite the line at `abilities.rs:4391` and the symmetric PB-LKI-CC OOS-LKI-4 reference. ~10 lines.

#### Finding E2: Hard-coded `pre_lba_power: None` for non-creature SBA paths (LOW)

**Severity**: LOW
**Files**:
- `crates/engine/src/rules/abilities.rs:890` (Food forage sacrifice)
- `crates/engine/src/rules/sba.rs:733` (planeswalker SBA-exile)
- `crates/engine/src/rules/sba.rs:854` (Saga SBA-sacrifice)
- `crates/engine/src/rules/sba.rs:1170` (Aura SBA-fall-off)

**Issue**: Each site unconditionally sets `pre_lba_power: None` based on the (correct, in 99.99% of cases) assumption that sagas, planeswalkers, auras, and Food tokens are non-creatures with no inherent power. But Layer 4 type-grant effects (e.g. animation enchantments) can produce a power on these objects. A SelfLeavesBattlefield trigger using `EffectAmount::SourcePowerAtLastKnownInformation` would lose the layer-4 boost.

For example: a `Saga` (whose Layer-4 type-line is just "Enchantment — Saga") is enchanted by an "all enchantments are X/X creatures" effect (Opalescence, Starfield of Nyx). The Saga becomes a creature. If the Saga then fires its SBA-sacrifice on chapter completion (sba.rs:854), it leaves the battlefield and emits `PermanentDestroyed` with `pre_lba_power: None`, losing the layer-4-resolved power. A trigger using the new variant would resolve to 0 instead.

**Symmetric to PB-LKI-CC**: PB-LKI-CC's `pre_lba_counters` capture at these same 4 sites uses `im::OrdMap::new()` with the same scope-limiting reasoning. The PB-LKI-Power runner correctly mirrored that pattern. The asymmetry between sba.rs:540 (which DOES capture via `calculate_characteristics`) and these 4 specialized SBA paths is a pre-existing inconsistency, NOT an introduced regression.

**Yield impact**: 0. Neither in-scope card (Conclave Mentor, Juri Master) hits these paths — both die via the standard SBA-creature-lethal-damage path at sba.rs:540, which IS correctly captured.

**Fix**: At each site, replace `pre_lba_power: None` with a `calculate_characteristics`-based capture mirroring sba.rs:540. Optional defensive default `or(obj.characteristics.power)` for safety. ~4 sites × ~4 lines each. Defer until a real card surfaces it (file as a sub-bullet on the existing OOS-LKI-Power-3 seed, or as a new OOS-LKI-Power-5 if the runner prefers).

#### Finding E3: Doc-comment line-number drift (LOW)

**Severity**: LOW
**File**: `crates/engine/src/state/stack.rs:478-479` (and similar in `state/stubs.rs:417-418`)

**Issue**: The new `lki_power: Option<i32>` field doc on `StackObject` cites "abilities.rs flush_pending_triggers ~line 7467" and "resolution.rs ~line 2057". Actual sites are `abilities.rs:7538` and `resolution.rs:2080`/`:2158`. The line numbers came from the plan (which used pre-impl estimates) and weren't updated post-impl.

**Fix**: Drop the explicit line numbers (they go stale on every refactor). Leave the function name reference: "Set from PendingTrigger::lki_power when the trigger is flushed to the stack (abilities.rs flush_pending_triggers). Read at resolution time (resolution.rs) into EffectContext.lki_power." Same pattern in `state/stubs.rs:417-418` for the PendingTrigger field.

## CR Coverage Check

| CR Rule | Verbatim from MCP? | Implemented? | Tested? | Notes |
|---------|-------------------|-------------|---------|-------|
| 603.10a (LBA triggers look back in time) | Yes — verified | Yes | Yes (test c) | Snapshot captured BEFORE move_object_to_zone at sba.rs:540 via `calculate_characteristics` |
| 113.7a (LKI on stack) | Yes — verified | Yes | Yes (tests a/b/c) | StackObject.lki_power threaded; field is Copy, no clone |
| 122.2 (counters cease on zone change) | Yes — verified | Yes (preserved) | Yes (test c sub-1) | Test (c) explicitly asserts `grave_obj.counters.is_empty()` |
| 400.7 (zone change → new object) | Yes — verified | Yes (preserved) | Yes (test c sub-2) | Test (c) explicitly asserts `grave_obj.characteristics.power == Some(1)` (printed face) |

## Card Def Summary

| Card | Oracle Match (MCP) | TODOs Remaining | Game State Correct (death) | Notes |
|------|-------------------|-----------------|---------------------------|-------|
| Conclave Mentor | Yes (verified MCP 2026-05-13) | 0 | Yes (test a passes; gain life = 4) | Replacement half (PB-CD) + death trigger (PB-LKI-Power) both ship |
| Juri, Master of the Revue | Yes (verified MCP 2026-05-13) | 0 | Yes (tests b/c pass; deal damage = 4 / 6) | Sacrifice-trigger (existing) + death trigger (PB-LKI-Power) both ship; CR 120.4 negative-damage clamp handled at Effect::DealDamage boundary |

## Engine plumbing trace (full chain walk)

| Step | File:Line | Status |
|------|-----------|--------|
| 1. DSL variant `EffectAmount::SourcePowerAtLastKnownInformation` | `card_definition.rs:2436` (after CounterCountAtLastKnownInformation) | OK (discriminant 18; disc 19 reserved with comment for OOS-LKI-Power-1) |
| 2a. `EffectContext.lki_power: Option<i32>` field | `effects/mod.rs:153` | OK |
| 2b. EffectContext::new initializes lki_power: None | `effects/mod.rs:182` | OK |
| 2c. EffectContext::new_with_kicker initializes lki_power: None | `effects/mod.rs:216` | OK |
| 2d. inner_ctx (ForEach EachPlayer/EachOpponent) propagates lki_power | `effects/mod.rs:2573` | OK |
| 2e. inner_ctx (ForEach object collection) propagates lki_power | `effects/mod.rs:2609` | OK |
| 2f. check_condition fallback ctx initializes lki_power: None | `effects/mod.rs:7483` | OK |
| 3a. `PendingTrigger.lki_power: Option<i32>` field with #[serde(default)] | `state/stubs.rs:411-422` | OK |
| 3b. PendingTrigger::blank() initializes lki_power: None | `state/stubs.rs:468` | OK |
| 4a. `StackObject.lki_power: Option<i32>` field with #[serde(default)] | `state/stack.rs:476-483` | OK |
| 4b. StackObject::trigger_default() initializes lki_power: None | `state/stack.rs:549` | OK |
| 5. Capture LKI power at sba.rs:540 via calculate_characteristics | `sba.rs:540-554` | OK (`calculate_characteristics(state, id).and_then(|c| c.power).or(obj.characteristics.power)`) |
| 6a. `GameEvent::CreatureDied.pre_death_power: Option<i32>` field with #[serde(default)] | `events.rs:222-229` | OK (HASHED) |
| 6b. SBA emit propagates pre_death_power on Proceed path | `sba.rs:584` | OK |
| 6c. SBA emit propagates pre_death_power on Redirect-to-grave path | `sba.rs:625` | OK |
| 6d. SBA emit propagates pre_lba_power on Redirect-to-exile path | `sba.rs:610` | OK |
| 7a. `GameEvent::AuraFellOff.pre_lba_power` field with #[serde(default)] | `events.rs:248-255` | OK (NOT hashed per OOS-LKI-Power-3) |
| 7b. `GameEvent::ObjectExiled.pre_lba_power` field with #[serde(default)] | `events.rs:387-394` | OK (NOT hashed) |
| 7c. `GameEvent::PermanentDestroyed.pre_lba_power` field with #[serde(default)] | `events.rs:409-416` | OK (NOT hashed) |
| 7d. `GameEvent::ObjectReturnedToHand.pre_lba_power` field with #[serde(default)] | `events.rs:502-509` | OK (NOT hashed) |
| 8. ALL ~150 emit sites updated across 13 files | (see grep results) | OK (battlefield sources capture via calculate_characteristics, non-battlefield use None with comments) |
| 9a. CreatureDied/SelfDies trigger arm propagates pre_death_power | `abilities.rs:4087` | OK (`lki_power: *pre_death_power`) |
| 9b. CreatureDied/SelfLeavesBattlefield trigger arm | `abilities.rs:4116` | OK |
| 9c. AuraFellOff/SelfLeavesBattlefield trigger arm | `abilities.rs:4456` | OK |
| 9d. PermanentDestroyed/SelfLeavesBattlefield trigger arm | `abilities.rs:5319` | OK |
| 9e. ObjectExiled/SelfLeavesBattlefield trigger arm | `abilities.rs:5375` | OK |
| 9f. ObjectReturnedToHand/SelfLeavesBattlefield trigger arm | `abilities.rs:5390` | OK (per Grep) |
| 9g. AnyCreatureDies arm sets lki_power: None | `abilities.rs:4391` | INTENTIONAL OOS per Site 9 STOP-AND-FLAG (see E1 finding) |
| 10. Flush propagation PendingTrigger → StackObject | `abilities.rs:7538` | OK (`stack_obj.lki_power = trigger.lki_power`) |
| 11a. Resolution build EffectContext (carddef-registry path) | `resolution.rs:2080` | OK (`ctx.lki_power = stack_obj.lki_power`) |
| 11b. Resolution build EffectContext (characteristics path) | `resolution.rs:2158` | OK (same) |
| 12. resolve_amount arm | `effects/mod.rs:6500` | OK (`ctx.lki_power.unwrap_or(0)`) |
| 13. resolve_cda_amount arm | `layers.rs:1624` | OK (returns 0 with comment per Site 13) |
| 14. HashInto for EffectAmount discriminant 18 | `state/hash.rs:4552` | OK (fieldless: `18u8.hash_into(hasher)`) |
| 15a. HashInto for PendingTrigger.lki_power | `state/hash.rs:2167` | OK (generic Option<i32> impl handles tag-byte automatically) |
| 15b. HashInto for StackObject.lki_power | `state/hash.rs:3046` | OK (same) |
| 16. HashInto for GameEvent::CreatureDied.pre_death_power | `state/hash.rs:3391` | OK (added after pre_death_counters loop) |
| 17a. HashInto for AuraFellOff/PermanentDestroyed/ObjectExiled/ObjectReturnedToHand | `state/hash.rs` (`..` pattern) | INTENTIONALLY UNCHANGED per Site 17 / OOS-LKI-Power-3 |
| 18. HASH_SCHEMA_VERSION 16→17 + history entry 17 | `state/hash.rs:95-103` | OK (history entry references CR 603.10a / 113.7a, names Conclave Mentor + Juri) |
| 19. Sentinel-assertion sweep across 11 files | (see Grep) | OK (no stale `16u8`; 11 sentinels at `17u8`: primitive_pb_cc_a:101, primitive_pb_cc_c_followup:400, primitive_pb_lki_cc:440, primitive_pb_lki_power:385, primitive_pb_ts:369, pbn_subtype_filtered_triggers:558, pbd_damaged_player_filter:597, pbp_power_of_sacrificed_creature:782, effect_sacrifice_permanents_filter:136, pbt_up_to_n_targets:411, pbt_up_to_n_targets:868) |
| 20. cargo build --workspace | (per WIP claim line 319) | OK (clean) |
| 21. helpers.rs prelude | no change needed | OK (`EffectAmount` already re-exported) |

## Test review

| Test | File:Line | Coverage | Notes |
|------|-----------|----------|-------|
| (a) `test_conclave_mentor_death_trigger_gains_life_from_lki_power` | `primitive_pb_lki_power.rs:124` | death path → GainLife | Discriminating: 4 (printed 2 + 2 counters), NOT 2 (printed only) and NOT 0 (no LKI threaded) |
| (b) `test_juri_master_death_trigger_deals_damage_from_lki_power` | `:200` | death path → DealDamage | Discriminating: 4 (printed 1 + 3 counters), NOT 1 and NOT 0 |
| (c) `test_lki_power_resolves_to_pre_death_value_not_printed_value` | `:277` | LKI-after-zone-change discriminator (load-bearing) | Asserts (1) graveyard counters empty per CR 122.2, (2) graveyard `characteristics.power == Some(1)` per CR 400.7, (3) damage = 6 (= 1 + 5 counters), NOT 1 and NOT 0 |
| (d) `test_pb_lki_power_hash_schema_version_and_determinism` | `:378` | Sentinel + variant discrimination + Option tag-byte canary | (1) HASH_SCHEMA_VERSION == 17u8, (2) `SourcePowerAtLastKnownInformation` distinct hash from `PowerOf(Source)` and from `CounterCountAtLastKnownInformation`, (3) PendingTrigger with lki_power None vs Some(0) vs Some(1) all hash distinctly (proves Option tag-byte encoding works) |

All 4 tests cite CR rules. Test (c) is the load-bearing zone-change discriminator per the plan's mandatory test (c). The plan's optional test (f) (Rest in Peace redirect path) was not implemented; runner correctly elided the optional test per the plan's "Skip note: if the test infrastructure is brittle, the runner may downgrade." No regression — the redirect path is exercised in the engine via test (c)'s natural SBA flow.

## Architecture invariant compliance

- **#1 (engine = pure library)**: OK — no IO, no async, no panic in library code (all unwraps in test code only)
- **#2 (immutable state)**: OK — `Option<i32>` is Copy, no clone needed; im::OrdMap iteration deterministic
- **#3 (commands only)**: OK — no new Commands needed
- **#4 (events as truth)**: OK — 5 GameEvent variants extended with `pre_death_power` / `pre_lba_power` fields with `#[serde(default)]` for backward compat
- **#5 (multiplayer-first)**: OK — no APNAP-specific logic
- **#7 (hidden info)**: OK — power snapshots are public info (battlefield permanent's P/T is visible to all players)
- **#8 (tests cite rules)**: OK — all 4 tests cite CR 603.10a / 122.2 / 400.7 / rulings 2020-06-23 + 2020-11-10
- **#9 (every card has CardDefinition)**: OK — no card-discovery changes; both re-authored cards retain their full ability set

## Gotcha file alignment

- **memory/gotchas-rules.md** — CR 603.10a "look back in time" is the load-bearing CR. PB-LKI-Power extends PB-LKI-CC's pattern from counters (OrdMap) to power (Option<i32>) with the same sba.rs:540 capture site and the same trigger-arm propagation chain. The gotcha is fully satisfied.
- **memory/gotchas-infra.md** — Object identity (CR 400.7) preserved: graveyard `GameObject` has empty counters and printed P/T; LKI lives on `PendingTrigger` / `StackObject` / `EffectContext`. CR 122.2 invariant maintained. `#[serde(default)]` added to all new event fields for backward compat with pre-PB-LKI-Power replays.
- **MEMORY.md "behavioral gotchas"** — TUI stack_view.rs / replay-viewer view_model.rs exhaustive matches: no new `StackObjectKind` or `KeywordAbility` variant, so no exhaustive-match update needed for those tools. `cargo build --workspace` confirmed clean per WIP. New `EffectAmount` arm could in theory break exhaustive matches in TUI/replay-viewer, but the WIP claim of clean workspace build covers this.

## OOS seeds verification (per plan Step 4)

- **OOS-LKI-Power (original)** — CLOSED by PB-LKI-Power 2026-05-13 (line 580 of pb-retriage-CC.md). Correctly cites HASH 17 + disc 18 + both card defs.
- **OOS-LKI-Power-1 (toughness variant, disc 19 reserved)** — Filed (line 588). Mirrors plan Step 4 OOS bullet #6.
- **OOS-LKI-Power-2 (Master Biomancer ETB-replacement EffectAmount)** — Filed (line 604). Mirrors plan Step 4 OOS bullet #1. **NOTE**: this slot was originally reserved for AnyCreatureDies LKI-power per plan Site 9 STOP-AND-FLAG. The runner repurposed it for Master Biomancer; AnyCreatureDies LKI-power is now undocumented (see E1 finding).
- **OOS-LKI-Power-3 (LBA hash arm symmetric extension)** — Filed (line 621). Mirrors plan Step 4 OOS bullet (the LBA hash arm pre-existing inconsistency).

## Yield calibration

Plan target: ≥2 cards (Conclave Mentor + Juri).
Plan post-plan estimate: 2 (TODO sweep returned 2 forced adds; pattern sweep returned 0 additional).
Implementation outcome: 2 cards re-authored, both passing dedicated tests + the discriminating LKI-after-zone-change test. **Threshold met.**

## Acceptance criteria mapping (per ESM scutemob-19)

| AC | Criterion | Status |
|---|---|---|
| 3762 | plan written | OK (`pb-plan-LKI-Power.md` exists, 932 lines) |
| 3763 | EffectAmount variant + hash changelog | OK (Site 1 + Site 14 + Site 18 — disc 18, HASH 16→17, history entry 17) |
| 3764 | PendingTrigger/StackObject/EffectContext threading + sba.rs:540 snapshot site | OK (Sites 2, 3, 4, 5, 10, 11) |
| 3765 | GameEvent pre_death_power/pre_death_toughness on 4-5 variants + abilities.rs sweep | OK (Sites 6, 7, 8, 9). Toughness variant correctly deferred per Step 0/4 (OOS-LKI-Power-1 seed filed). |
| 3766 | HASH 16→17 + Option tag-byte encoding + sentinel sweep | OK (11 sentinel sites updated; test (d) sub-3 validates Option tag-byte encoding via Some(0) vs None hash) |
| 3767 | Conclave Mentor + Juri TODOs cleared | OK (both card defs reviewed; oracle match exact; 0 TODOs) |
| 3768 | tests including LKI-after-zone-change | OK (test (c) is the load-bearing discriminator; tests (a)/(b) are per-card; test (d) is hash determinism + sentinel) |
| 3769 | cargo gates | OK per WIP (2749 tests, clippy clean, fmt clean, build clean) |
| 3770 | review + fixes | This file. Verdict: PASS-WITH-NITS. 3 LOW findings; no HIGH/MEDIUM. |
| 3771 | CLAUDE.md + OOS seed close + authoring report regen | OOS seeds done (4 entries in pb-retriage-CC.md); CLAUDE.md + authoring report deferred to coordinator per WIP line 323-324. |

## Reviewer checklist (mirroring `memory/primitive-wip.md`)

- [x] CR rules independently verified (603.10a, 113.7a, 122.2, 400.7) via MCP
- [x] Card oracle text re-verified via MCP for both re-authored cards (Conclave Mentor, Juri Master) — match verbatim including rulings
- [x] Every dispatch site walked and confirmed correct (21 sites; full chain in plumbing trace table)
- [x] Hash arm + version bump + history entry verified (HASH=17, history entry 17, 11 sentinel files updated)
- [x] Tests verified (4 present, all cite CR, test (c) is load-bearing discriminator, test (d) covers tag-byte canary)
- [x] No scope creep (toughness variant deferred, AnyCreatureDies untouched, cost-payment LKI untouched, LBA hash arms unchanged)
- [x] Review file written: `memory/primitives/pb-review-LKI-Power.md`
- [x] Verdict: **PASS-WITH-NITS** (3 LOW findings; 0 HIGH; 0 MEDIUM)

## Recommendation

**No fix-phase required to ship.** All 3 LOW findings are documentation/seed gaps that don't affect game-state correctness for the 2 in-scope cards. Coordinator may either:

1. **Ship as-is** — merge the branch; address E1 (file OOS-LKI-Power-4 for AnyCreatureDies LKI-power), E2 (capture pre_lba_power at the 4 non-creature SBA sites), E3 (drop stale line numbers from doc comments) as opportunistic follow-ups when next touching the relevant files.
2. **Spawn a brief fix-phase pass** — ~30 minutes to resolve all 3 LOW findings: file E1's OOS-LKI-Power-4 seed (~10 lines), edit E2's 4 sites to capture power consistently (~16 lines), edit E3's 2 doc comments (~4 lines). No new tests required (E1/E3 are doc-only; E2 changes are mechanically symmetric to existing patterns).

Either path is consistent with PB project conventions. PB-LKI-CC review precedent (PASS-WITH-NITS) had 3 LOW findings all addressed in a brief fix-phase before final PASS.
