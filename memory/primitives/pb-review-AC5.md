# Primitive Batch Review: PB-AC5 — Alt-Costs & Timing Keywords

**Date**: 2026-07-08
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit**: `32bd607d`
**CR Rules**: 702.185 (Warp), 702.53 (Transmute), 701.43 (Exert), 118.9/118.9a/c (alt costs),
508.1g, 607.2h, 608.3g, 400.7, 502.3, 701.5
**Engine files reviewed**: `rules/casting.rs`, `rules/resolution.rs`, `rules/turn_actions.rs`,
`rules/abilities.rs`, `rules/combat.rs`, `effects/mod.rs`, `state/hash.rs`, `state/stack.rs`,
`state/game_object.rs`, `state/combat.rs`, `state/types.rs`, `cards/card_definition.rs`,
`tools/replay-viewer/src/view_model.rs`, `crates/simulator/src/legal_actions.rs`
**Card defs reviewed**: 9 (timeline_culler, dimir_infiltrator, combat_celebrant, arena_of_glory,
force_of_will, force_of_vigor, force_of_negation, starfield_shepherd, force_of_despair) + 21
counter-spell defs touched for `Effect::CounterSpell.exile_instead`
**Test file**: `crates/engine/tests/pb_ac5_alt_costs.rs` (25 tests)

## Verdict: needs-fix

The primitive is largely correct — Warp exile/recast, Transmute, Exert (both shapes), and Pitch
all implement their CR rules faithfully, object-identity (CR 400.7) is handled correctly via
`move_object_to_zone` resetting `designations`/`warped_turn`, the KeywordTrigger reuse does NOT
conflate Warp with Dash/Blitz (arms discriminate by keyword), the mutual-exclusion is complete,
and all 9 card defs match oracle text (2 correctly kept BLOCKED, arena_of_glory correctly partial).
**However, two new mutable serialized fields — `StackObject.was_warped` and `ActivationCost.exert`
— were NOT added to their `HashInto` impls**, despite the hash changelog explicitly claiming they
were. These are HIGH replay/rewind-corruption defects of the exact PB-S H1 class the codebase
already warns about, and the changelog is aspirationally-wrong (a conventions violation). Plus
minor test-strength LOWs. The two runner deviations (KeywordTrigger reuse; no `legal_actions.rs`
arms) are **correct and justified**.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `state/hash.rs:3165-3219` | **`StackObject.was_warped` not hashed.** The `impl HashInto for StackObject` hashes `was_dashed`/`was_blitzed`/`was_impended`/… but omits `was_warped`. Two stack states differing only in whether a spell was warp-cast (→ end-step exile vs. permanent stays) hash identically. **Fix:** add `self.was_warped.hash_into(hasher);` alongside `self.was_dashed` (~3201). |
| 2 | **HIGH** | `state/hash.rs:2392-2406` | **`ActivationCost.exert` not hashed.** The `impl HashInto for ActivationCost` hashes `exile_self`/`discard_self`/`forage`/… but omits `exert` (the field exists — `game_object.rs:280`, set true by `flatten_cost_into` at `replay_harness.rs:3548`). This is the exact failure mode the adjacent `exile_self` comment warns about ("PB-S H1 failure mode"). **Fix:** add `self.exert.hash_into(hasher);` after `self.exile_self.hash_into(hasher);`. |
| 3 | **LOW** | `state/hash.rs:243-247` | **Changelog claims coverage it doesn't deliver.** The schema-32 doc-comment lists "`StackObject.was_warped: bool`, `ActivationCost.exert: bool`" among covered new fields, but neither is hashed (Findings 1-2). Aspirationally-wrong comment (conventions.md "aspirationally-wrong comments are correctness hazards"). **Fix:** true after Findings 1-2 are applied; leave the changelog only once the hash arms exist. |
| 4 | **LOW** | `rules/turn_actions.rs:1245-1265` | **Exert + `skip_untap_steps` on the same permanent expire in different steps.** If a permanent is both EXERTED and has `skip_untap_steps > 0`, the `EXERTED` branch consumes this untap step and clears the flag, but `skip_untap_steps` is NOT decremented (it's in an `else if`), so that effect carries to the *following* untap step. CR 502.3 / 701.43b treat all "doesn't untap next untap step" effects as expiring during the same step. Extremely rare (no roster card), so LOW. **Fix (opt.):** decrement `skip_untap_steps` in the EXERTED branch too, or document as an accepted deviation with a tracking LOW. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | — | all 9 | No card-def defects. Oracle text, types, P/T, costs, and primitive usage all correct. arena_of_glory correctly BLOCKED on the mana-spend-haste rider; starfield_shepherd + force_of_despair correctly BLOCKED with honest comments. `has_card_types` verified OR-semantics (FoV Artifact/Enchantment). Dimir shuffle setting matches established convention (see Observations). |

## Test Findings

| # | Severity | Test | Description |
|---|----------|------|-------------|
| 5 | **LOW** | `test_pitch_cannot_pitch_self` | Weak: passes even if the explicit `pitch_card_id == card` guard (casting.rs:4178) were removed, because the later zone check (4186, card now on stack not in hand) would also reject. Does not isolate the named guard. **Fix:** assert the error message mentions "own pitch"/"itself", or construct a case only the self-guard catches. |
| 6 | **LOW** | `test_exert_combat_celebrant_untaps_and_extra_combat` | Does not guard against a double-fire of the linked trigger — `additional_phases.contains(&Combat)` is true whether 1 or 2 phases were scheduled. (No double-fire actually occurs — verified `WhenExertedAsAttacks` has no runtime `TriggerEvent` mapping — but the test wouldn't catch one.) **Fix (opt.):** assert exactly one added combat phase. |

## Verify-Hard Points (per dispatch brief)

1. **CR 702.185a countered/left-before-end-step.** CORRECT. `resolution.rs:3437-3466` filters `obj.zone == Battlefield` before exiling; countered/dead warp spells are never exiled (`test_warp_countered_spell_not_exiled` validates, and the end-step collector `turn_actions.rs:717-724` only picks up battlefield objects with `cast_alt_cost == Warp`). Recast gate `warped_turn >= turn_number` (casting.rs:365) is strictly-later-turn; `turn_number` is monotonic so rollover is safe.
2. **CR 702.185b WARPED distinguishability.** CORRECT. `Designations::WARPED` set on the *new* exile object (resolution.rs:3455) and cleared naturally on any subsequent zone change (`move_object_to_zone` sets `designations: Designations::default()`, `warped_turn: 0`). Recast reads the flag pre-move; no stale leak (CR 400.7).
3. **CR 701.43b multiple exerts / vs DoesNotUntap.** CORRECT for the roster. Boolean `EXERTED` collapses N exerts to one skipped step (turn_actions.rs:1251). A permanent with both EXERTED and DoesNotUntap still does not untap (EXERTED branch takes it, no untap). Caveat: exert + `skip_untap_steps` is Finding 4 (LOW).
4. **CR 701.43c off-battlefield.** CORRECT. Enforced in both cost paths: `combat.rs:506-509` (attack-cost shape) and `abilities.rs:716-721` (activation-cost shape). `test_exert_cannot_exert_off_battlefield` validates the activation path with a message check.
5. **CR 118.9a single alt cost.** CORRECT and COMPLETE. All 20+ param-driven alt costs are mutually exclusive by construction (single `alt_cost: Option<AltCostKind>`); the only zone/keyword auto-detected *alternative* costs (flashback, escape, madness) are explicitly guarded for both Warp (casting.rs:408-424) and Pitch (446-462). Retrace/jump-start are additional costs or require explicit params — no 118.9a conflict.
6. **CR 118.9c mana value unchanged.** CORRECT. Pitch pays `{0}` without overwriting `characteristics.mana_cost`; `test_pitch_mana_value_unchanged` confirms FoW MV stays 5. Warp mirrors (payment cost fetched separately, printed cost untouched). Transmute's "same MV" is a faithful fixed hardcode for the fixed-MV roster card.
7. **CR 702.53a/b Transmute.** CORRECT. Hand-only (DiscardSelf gate), sorcery-timing, equal-MV search (min=max=2). 702.53b "has an activated ability in other zones" is not broken — the `Activated` ability + `Transmute` marker live on the CardDefinition in every zone.
8. **hash.rs.** DEFECTIVE — Findings 1-2 (was_warped, exert unhashed). All OTHER new fields ARE covered: `warped_turn` (1239), `exerted_attackers` (3289), Designations bits (bitfield), `KeywordAbility` 163/164/165 (981-985), `AltCostKind` 30/31 (3331-3332), `AltCastDetails::Warp/Pitch` (6196-6217), `AdditionalCost::ExileFromHand` (3411), `Cost::ExileFromHand`/`Exert` (5277-5282), `TriggerCondition::WhenExertedAsAttacks` (5056), `CounterSpell.exile_instead` (5388). Schema bump 31→32 present; parity test `test_hash_schema_version_is_32` present.
9. **CR 400.7 object identity.** CORRECT. Exile-then-recast produces new objects each hop; `move_object_to_zone` resets designations + warped_turn; no stale ObjectId carried.

## Runner Deviations — Adjudicated

- **No `PendingTriggerKind::WarpExile` (reuse `KeywordTrigger`/`DelayedZoneChange`)**: **CORRECT.**
  Resolution arms discriminate on `keyword: KeywordAbility::Warp` (resolution.rs:3431) vs `::Dash`
  (3257) vs `::Blitz` (3302); Warp exiles, Dash returns-to-hand, Blitz sacrifices — no conflation.
  Consistent with the RC-2 consolidation the codebase mandates.
- **No `legal_actions.rs` arms for Warp/Pitch/Exert**: **CORRECT.** Verified `StubProvider`
  (`legal_actions.rs:106-147`) wires ZERO alt-cost casting for any existing alt cost (Foretell,
  Escape, Flashback, Dash, Blitz, Adventure, Enlist, Impending) — the bot always uses
  `alt_cost: None`; the file's own doc-comment says so. Adding Warp/Pitch/Exert would be
  inconsistent. Deferred-to-W2 is the established, documented pattern. NOT a finding.

## Test-Vacuity Audit (per brief)

Audited all 25 tests. The warp/pitch/exert/transmute tests correctly discriminate on the behavior
they name: `test_warp_recast_from_exile_after_turn` (same-turn err vs later-turn ok isolates the
`warped_turn` gate), `test_pitch_force_of_vigor_opponents_turn_only` (only active-player differs
between the err and ok halves), `test_pitch_wrong_color_rejected` (message-checked),
`test_exert_cannot_exert_off_battlefield` (message-checked), `test_exert_twice_expires_same_step`
(counts untap steps). Two weak tests found — Findings 5 and 6 (LOW). No test passes for a
wrong reason on broken behavior.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 702.185a (warp cost/exile/recast) | Yes | Yes | cast_from_hand, exiled_at_end_step, recast_after_turn, not_exiled_if_not_warp, timeline_culler_from_graveyard |
| 702.185a (countered never exiled) | Yes | Yes | warp_countered_spell_not_exiled |
| 702.185b (WARPED flag) | Yes | Yes | exiled_at_next_end_step asserts WARPED + warped_turn |
| 702.53a (transmute) | Yes | Yes | searches_equal_mana_value, only_from_hand, sorcery_timing, discards_self |
| 701.43a/b (exert untap expiry) | Yes | Yes | does_not_untap_next_untap_step, twice_expires_same_step |
| 701.43c (off-battlefield) | Yes | Yes | cannot_exert_off_battlefield |
| 701.43d/607.2h (linked trigger) | Yes | Yes | combat_celebrant_untaps_and_extra_combat, offer_requires_not_already_exerted |
| 118.9a (single alt cost) | Yes | Yes | warp_mutual_exclusion, pitch_mutual_exclusion |
| 118.9c (MV unchanged) | Yes | Yes | pitch_mana_value_unchanged |
| 701.5/FoN (exile-instead counter) | Yes | Yes | force_of_negation_counters_and_exiles |
| Hash schema (was_warped) | **No** | No | **Finding 1 (HIGH)** |
| Hash schema (exert) | **No** | No | **Finding 2 (HIGH)** |

## Card Def Summary

| Card | Oracle Match | TODOs | Game State Correct | Notes |
|------|-------------|-------|-------------------|-------|
| timeline_culler | Yes | 0 | Yes | Warp—{B},Pay2life; from_graveyard:true; Haste; 2/2 |
| dimir_infiltrator | Yes | 0 | Yes | CantBeBlocked + Transmute marker + Mana/DiscardSelf activated, MV2 search, sorcery |
| combat_celebrant | Yes | 0 | Yes | Exert marker + WhenExertedAsAttacks linked trigger (untap others + extra combat) |
| arena_of_glory | Partial (correct) | 0 | Yes (2/3 abilities) | 3rd ability correctly BLOCKED on mana-spend-haste rider; documented |
| force_of_will | Yes | 0 | Yes | Pitch [PayLife1, Exile Blue]; CounterSpell |
| force_of_vigor | Yes | 0 | Yes | Pitch [Exile Green] opp-turn-only; UpToN destroy Artifact/Enchant (OR filter verified) |
| force_of_negation | Yes | 0 | Yes | Pitch [Exile Blue] opp-turn-only; CounterSpell noncreature exile_instead:true |
| starfield_shepherd | Correctly BLOCKED | comment | N/A | Flying only; ETB disjunctive-search gap documented |
| force_of_despair | Correctly BLOCKED | comment | N/A | entered-this-turn DestroyAll gap documented |

## Observations (not findings)

- **Dimir transmute `shuffle_before_placing: false`**: The `SearchLibrary` effect only shuffles
  when `shuffle_before_placing == true`. Dimir uses `false` with `destination: Hand`, so the
  library is not shuffled after the search despite oracle "then shuffle." This is **out of scope**:
  it matches the established convention for search-to-hand tutors (verified `diabolic_tutor.rs:21`
  uses the identical pattern). If a shuffle bug exists it is a pre-existing engine issue affecting
  all such tutors, not a PB-AC5 regression.
- **Exert linked-trigger runtime path**: `WhenExertedAsAttacks` has no runtime `TriggerEvent`
  mapping, so `enrich_spec_from_def`'s runtime triggered abilities cannot fire it — only the
  card-registry scan (abilities.rs:3755-3792), gated on `combat.exerted_attackers`, fires it. No
  double-fire risk. Confirmed the trigger fires ONLY when the player chose to exert.
