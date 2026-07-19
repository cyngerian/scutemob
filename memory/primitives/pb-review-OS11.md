# Primitive Batch Review: PB-OS11 — RemoveCounter mana-ability lowering + batch filtered-attack trigger (FINAL PB-OS batch)

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 605.1a, 605.3b, 602.2c, 118.3, 106.12, 614.1c, 704.5f, 400.7; 508.1, 508.1m, 603.2c, 205.3, 111.1
**Engine files reviewed**:
- `crates/card-types/src/state/game_object.rs` (`ManaAbility.remove_counter`)
- `crates/card-types/src/cards/card_definition.rs` (`TriggerCondition::WheneverYouAttack` unit→struct)
- `crates/engine/src/testing/replay_harness.rs` (`ManaAbilityCost`, `mana_ability_cost_components`, no-tap guard, `mana_ability_lowering`, WheneverYouAttack conversion)
- `crates/engine/src/rules/mana.rs` (`handle_tap_for_mana` steps 5b2/6d)
- `crates/engine/src/rules/abilities.rs` (`collect_triggers_for_event` ControllerAttacks branch + dispatch L4147-4170)
- `crates/engine/src/state/hash.rs` (both HashInto arms + HASH 63)
- `crates/engine/src/rules/protocol.rs` (PROTOCOL 26 + fingerprint + epoch)
- `crates/engine/tests/core/completeness_deviation_scan.rs` (allowlist + marker floor)

**Card defs reviewed** (4 targeted + 2 backfill = 6):
`workhorse.rs` (NEW), `anim_pakal_thousandth_moon.rs`, `general_kreat_the_boltbringer.rs`, `hermes_overseer_of_elpis.rs`, `gemstone_array.rs` (backfill), `druids_repository.rs` (backfill), plus 5 migrated-only defs (`legions_landing`, `caesar_legions_emperor`, `mishra_claimed_by_gix`, `chivalric_alliance`, `seasoned_dungeoneer`).

## Verdict: needs-fix

The engine work is **correct and well-tested** on both halves. Part A (RemoveCounter mana-ability lowering) matches CR 605.1a exactly, is scoped to the self-exhausting remove-counter cost (proven by the vacuity gate test), pays before mana production, reads the source's own counters, and lets a 0/0 Workhorse die as an SBA. Part B (batch filtered-attack trigger) fires exactly once per combat, reads `state.combat.attackers`, honors the filter, and the enters-attacking Gnome tokens do not re-trigger. The 19 tests execute real commands and assert game state. Wire bumps (HASH 63, PROTOCOL 26) are both correct and gate-justified.

**However, one MEDIUM defect blocks a clean close:** the two backfill cards `gemstone_array.rs` and `druids_repository.rs` are **still marked `known_wrong` with reason strings that are now factually false**, even though the batch's own passing execution tests (`test_gemstone_array_any_color_lowered`, `test_druids_repository_any_color_lowered`) prove the documented color bug is fixed and no golden script references them. The WIP header and the dispatch brief both claim these two were "flipped known_wrong → Complete"; the shipped source shows they were **not**. Fixing this is a marker + stale-comment change only (no engine change).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| — | — | — | No engine findings. Both primitives are CR-correct, scoped, and tested. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 1 | **MEDIUM** | `gemstone_array.rs:38-39,56-62` + `druids_repository.rs:40-41,58-67` | **Backfill cards left `known_wrong` with now-false reason strings.** Batch fixed the color bug (tests pass, no golden scripts break) but markers were not flipped and notes still describe the removed engine limitation. **Fix:** flip both to `Completeness::Complete`; delete the stale `known_wrong` reasons and the stale inline "deferred to PB-37 / implemented as regular activated ability" comments. |
| 2 | LOW | `anim_pakal_thousandth_moon.rs:84-87` | **Mid-resolution LKI counter-count deviation.** `CounterCount{Source}` reads live counters; ruling 2023-11-10(a) wants last-known count if Anim Pakal leaves mid-resolution. Documented + allowlisted; accepted, non-blocking. **Fix:** none required; tracked. |

### Finding Details

#### Finding 1: gemstone_array / druids_repository left `known_wrong` with false reason strings (MEDIUM)

**Severity**: MEDIUM
**Files**: `crates/card-defs/src/defs/gemstone_array.rs:38-39,56-62`; `crates/card-defs/src/defs/druids_repository.rs:40-41,58-67`
**Oracle**: both — "Remove a charge counter from this [permanent]: Add one mana of any color."
**Issue**: The engine change in this batch (accept `Cost::RemoveCounter` in `mana_ability_cost_components`, relax the no-tap guard, pay in `handle_tap_for_mana` step 6d, produce chosen color via the existing `any_color` path) genuinely closes the documented colorless-only bug. This is **proven by execution**, not just claimed:
- `test_gemstone_array_any_color_lowered` taps Gemstone Array with `chosen_color: Some(Green)` and asserts `green == 1`, `colorless == 0`, charge `5→4`. PASSES.
- `test_druids_repository_any_color_lowered` does the same with Blue. PASSES.
- The defs are built via `all_cards()` + `enrich_spec_from_def` (the real card path), and `cargo test --all` is green (per WIP).
- No golden script activates these abilities (grep of `test-data/` finds only card-data JSON, no scripts), so the activated→mana reclassification breaks nothing.

Despite this, both defs still carry `Completeness::known_wrong(...)`, and the reason strings are **now factually false**. `druids_repository.rs:59-66` still asserts:
- "adds one COLORLESS mana" — false; the test proves chosen color.
- "mana_ability_cost_components refuses Cost::RemoveCounter (ManaAbility has no counter-cost field ... handle_tap_for_mana has no counter payment path)" — false; this batch added exactly those.
- "the requires_tap: false path is unexercised in the whole corpus" — false; Workhorse + these two now exercise it.

Consequences:
1. Two now-correct cards remain rejected by `validate_deck` (SR-2: non-`Complete` defs are rejected) — they cannot be used in decks despite working.
2. The false reason strings violate the standing convention that aspirationally/actually-wrong descriptions are correctness hazards ("never leave the wrong version standing", `memory/conventions.md`).
3. The WIP `A-Backfill` bullet ("flipped known_wrong → Complete ... chosen-color mana confirmed") is inaccurate versus shipped source; the `completeness_deviation_scan` gate comment (674→671, only 3 flips) is the accurate record and correctly does **not** count these two.

The plan (A-Backfill) directed: on execution-verify success, flip both to Complete; on failure, keep known_wrong **with an updated note reflecting the new lowered-path reason + an OOS seed**. Execution-verify succeeded, yet neither branch was carried out cleanly — the marker was kept and the note was left describing the pre-fix world.
**Fix**: Set `completeness: Completeness::Complete` on both defs. Remove the `known_wrong(...)` reason bodies. Delete the stale inline comments at `gemstone_array.rs:38-39` and `druids_repository.rs:40-41` ("Implemented as regular activated ability for this batch. Mana-ability classification deferred to PB-37"). Re-run `completeness_deviation_scan` (the `>= 662` lower-bound assert still holds; the two flips only lower the actual count further). If — contrary to the evidence here — there is a genuine surviving reason to keep them known_wrong, rewrite both reason strings to state that actual current reason and file an OOS seed; do not leave the pre-fix text standing.

#### Finding 2: Anim Pakal mid-resolution LKI counter deviation (LOW)

**Severity**: LOW
**File**: `crates/card-defs/src/defs/anim_pakal_thousandth_moon.rs:84-87` (token count = `EffectAmount::CounterCount{Source}`)
**Oracle ruling**: 2023-11-10(a) — "If Anim Pakal is no longer on the battlefield when the triggered ability resolves ... use the number of +1/+1 counters that were on it when it was last on the battlefield."
**Issue**: `CounterCount{Source}` reads live counters; if Anim Pakal leaves the battlefield in response to its own trigger, the source ObjectId is dead (CR 400.7) and the count would resolve to 0 rather than the last-known value. The engine has no non-leaves-trigger LKI counter reader today. In the overwhelming-majority case (Anim Pakal present through resolution) the count is correct.
**Fix**: None required. The deviation is documented in-def (L21-27), sanctioned by the `completeness_deviation_scan` ALLOWLIST entry (`completeness_deviation_scan.rs:150-159`), and consistent with corpus precedent for mid-resolution source removal. Non-blocking, as the plan (B-Card-1) directs.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 605.1a (mana ability: no target, could add mana, not loyalty) | Yes | Yes | `test_workhorse_lowered_as_mana_ability_not_activated` — asserts in `mana_abilities`, not `activated_abilities` |
| 605.3b (mana ability does not use the stack) | Yes | Yes | `test_workhorse_remove_counter_adds_colorless` asserts empty stack |
| 602.2c / 118.3 (cost paid on activation; ≥ n counters required) | Yes | Yes | pre-check at `mana.rs:238-250`; `test_workhorse_insufficient_counters_rejected` |
| 704.5f / 400.7 (0/0 dies as SBA; old ObjectId dead) | Yes | Yes | `test_workhorse_last_counter_removal_then_dies` (mana still produced, then SBA death) |
| 614.1c (enters with four +1/+1 counters, self-replacement) | Yes | Yes | `test_workhorse_enters_with_four_counters` (P/T 4/4 layer-resolved) |
| self-referential source (own counters only) | Yes | Yes | `test_remove_counter_mana_ability_reads_source_counters_only` (two Workhorses) |
| no-tap guard scoped to remove_counter | Yes | Yes | `pb_os11_remove_counter_lowering_gate_is_not_vacuous` (positive + `Sequence([DiscardCard, RemoveCounter])` negative) |
| 508.1/508.1m/603.2c (batch attack trigger fires once) | Yes | Yes | `test_anim_pakal_multiple_nongnome_attackers_fires_once` (3 attackers → 1 fire); `test_whenever_you_attack_fires_once_per_combat` |
| filter on declared-attacker set (exclude_subtypes / has_subtype) | Yes | Yes | Gnome-only no-fire + non-Gnome fire together prove the filter reads a populated attacker set and discriminates; Kreat/Hermes positive+negative |
| 205.3/111.1 (created attacking tokens never "declared", don't re-trigger) | Yes | Yes | `test_anim_pakal_created_gnomes_do_not_inflate_next_trigger` |
| post-increment token count scales | Yes | Yes | `test_anim_pakal_token_count_scales_with_counters` (0→1→2, cumulative 1+2=3) |
| filter:None legacy regression | Yes | Yes | `test_you_attack_filter_none_fires_on_any_attack` fires on a Gnome attacker |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `workhorse` (NEW) | Yes | 0 | Yes | Complete; fixed {C} dodges any-color bug; no gated-stub effects |
| `anim_pakal_thousandth_moon` | Yes | 0 | Yes | Complete (allowlisted LKI edge, Finding 2); `exclude_subtypes:[Gnome]` correct |
| `general_kreat_the_boltbringer` | Yes | 0 | Yes | Complete; both abilities present; `has_subtype:Goblin` batch trigger |
| `hermes_overseer_of_elpis` | Yes | 0 | Yes | Complete; both abilities present; Scry uses `count:` field (correct) |
| `gemstone_array` | Yes (now correct) | stale comment L38-39 | Yes (proven) | **MEDIUM — mislabeled `known_wrong`, false reason (Finding 1)** |
| `druids_repository` | Yes (now correct) | stale comment L40-41 | Yes (proven) | **MEDIUM — mislabeled `known_wrong`, false reason (Finding 1)** |
| 5 migrated defs (`legions_landing` etc.) | Yes | 0 | Yes (unchanged) | Mechanical `{ filter: None }` migration; regression-guarded |

## Verification Notes (adversarial checks performed)

- **Oracle text** for all four targeted cards MCP-verified verbatim; all defs match (types, subtypes, mana cost, P/T, both abilities where applicable).
- **No gated-stub effects** in any `Complete` def: Workhorse uses `AddMana` (fixed {C}); Anim Pakal `Sequence[AddCounter, CreateToken{CounterCount}]`; Kreat `CreateToken`/`ForEach DealDamage`; Hermes `CreateToken`/`Scry`. None use `Effect::Choose`, `MayPayOrElse`, `AddManaChoice`, or the `AddManaAnyColor`/`AddManaAnyOneColor` family. (Note: Gemstone/Druids DO use `AddManaAnyColor`, but that is now a correctly-lowered chosen-color mana ability — proven by the two backfill tests — not a stub; it is only their `known_wrong` marker that is stale, Finding 1.)
- **No-tap guard scoping** (`replay_harness.rs:3900`): relaxation gated on `acc.remove_counter.is_none()` alongside `exile_self_from_hand`; a no-tap `Cost::Mana`/`SacrificeSelf`-only cost still declines. Confirmed by the vacuity test's negative control.
- **Payment ordering** (`mana.rs`): 5b2 pure-validation before any mutation; 6d payment before the SR-28 snapshot and before mana production (CR 602.2c). Removing a counter does not move the source, so the snapshot boundary is preserved. No raw `.unwrap()`/`.expect()` in the new library paths — `object_mut(source)?` and `let Some(..) else` used throughout.
- **combat.attackers availability**: proven populated at collect-time by the pairing of the Gnome-only no-fire test (would also pass on an empty set) with the non-Gnome fire test (fails on an empty set) — together they show the set is both present and correctly filtered.
- **Once-per-combat dispatch** (`abilities.rs:4150-4169`): `ControllerAttacks` dispatched once per controller-source (not per attacker); the filter branch adds a skip-if-no-match, preserving fire-once. `state.combat.as_ref().map(..).unwrap_or(false)` handles absent combat gracefully; filter:None triggers skip the branch entirely and always fire.
- **Exhaustive-match completeness**: all `WheneverYouAttack` construction/match sites migrated — 5 card defs (`{ filter: None }`), 3 filtered card defs, `replay_harness.rs:3175` (destructure), `hash.rs:5791` (match arm), `abilities.rs` (ControllerAttacks branch), `trigger_variants.rs:468/485`, `completeness_deviation_scan.rs` (comment). No `tools/` (TUI/replay-viewer) construction sites (`TriggerCondition` not in their exhaustive matches — grep confirms 0 hits under `tools/`).
- **Hash**: `ManaAbility.remove_counter` hashed at `hash.rs:1825` (tuple `HashInto for (A,B)` exists at 1026, `Option` impl exists); `WheneverYouAttack{filter}` arm hashes filter at 5791-5794. HASH 62→63 with a combined `- 63:` history entry.
- **Wire**: PROTOCOL 25→26 is genuinely demanded — `Characteristics` is a `CLOSURE_MUST_CONTAIN` member (`protocol_schema.rs:98-108`), so `Characteristics.mana_abilities → ManaAbility.remove_counter` moves the wire digest. `PROTOCOL_SCHEMA_FINGERPRINT` (protocol.rs:266) matches the v26 epoch row (protocol.rs:472); values are internally consistent and, per WIP, gate-machine-computed (the `protocol_schema` recompute-from-source test would fail if fabricated; suite is green). The Part-B `TriggerCondition` change is correctly HASH-only (outside the wire closure).

## Test Rigor

19 new tests (9 Part A + 10 Part B) plus the reused `test_whenever_you_attack_fires_once_per_combat`. All execute real `Command`s and assert observable game state (mana pool, counters, tapped status, stack emptiness, zone moves, SBA death, `GameEvent` emission, life totals, `Scried` events, token counts) — none are source-traces. Decoy/negative coverage is complete for the AC matrix:
- Workhorse: wrong-permanent counters untouched (T6), insufficient counters rejected (T5), lowered-not-stacked (T3), last-counter death (T4).
- Anim Pakal: Gnome-only no-fire, multi-attacker fires once, token no-inflation, count scales.
- Kreat/Hermes: positive + negative each.
- Backfill: chosen-color-not-colorless + counter removed, both cards.

## Previous Findings (re-review only)

N/A — first review of PB-OS11.
