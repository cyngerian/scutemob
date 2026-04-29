# PB-SFT Re-Triage Review

**Date**: 2026-04-28
**Reviewer**: primitive-impl-reviewer (Opus, RESEARCH-ONLY)
**Memo reviewed**: `memory/primitives/pb-retriage-SFT.md`
**Verdict**: **PASS**

## Summary

The memo's FIELD-ADDITION reframing of the planner's `Cost::SacrificeFilteredType`
hypothesis is fully defensible. Engine-surface claims, per-card classifications,
and stop-and-flag log all check out against source. PROCEED is justified.

## 1. Engine surface — VERIFIED

- `Cost::Sacrifice(TargetFilter)` exists at `card_definition.rs:1075`. Confirmed.
- `Effect::SacrificePermanents { player, count }` at `card_definition.rs:1503-1506`
  has **no `filter` field**. Confirmed verbatim.
- `SpellAdditionalCost` (5 sacrifice variants) at `card_definition.rs:1101-1112`. Confirmed.
- `SacrificeFilter` enum (6 variants) at `game_object.rs:196-210`. Confirmed.
- `ActivationCost.sacrifice_filter: Option<SacrificeFilter>` at `game_object.rs:242`. Confirmed.
- The 3 token-spec callsites (Food/Clue/Blood) are correctly written with
  `sacrifice_self: true` semantics, not card-def authoring sites. Confirmed.

## 2. Per-card classifications — VERIFIED (4 spot-checks)

| Card | Memo claim | File-read result |
|------|-----------|------------------|
| Fleshbag Marauder | CONFIRMED-IN-SCOPE; ETB + creature filter | TODO at line 20 reads "lacks creature-only filter — picks any permanent." Matches. |
| Grave Pact | CONFIRMED-IN-SCOPE; LtB-trigger wired, filter missing | Comment line 14: "no creature-only filter; engine picks lowest-ID permanent." Matches. |
| Blasphemous Edict | CONFIRMED-IN-SCOPE; cost-reduction is separate gap | Two distinct TODOs (cost reduction + filter). Matches memo's caveat. |
| Wight of the Reliquary | BLOCKED (CDA + "another") | Two explicit TODOs: CountInGraveyard CDA + SacrificeAnother. Matches. |

## 3. Stop-and-flag log — HONEST

Wight of the Reliquary's CDA blocker (`EffectAmount::CountInGraveyard`) is real
and explicitly documented in its source TODO. The "another" enforcement gap is
also documented in-source. Memo accurately mirrors source TODOs.

## 4. Yield calibration — REASONABLE

14 TODOs × 50–65% (filter-PB band) = 7–9 expected. Three borderline cards
(Anowon `exclude_subtypes`, Blessed Alliance `is_attacking`, Accursed Marauder
`is_nontoken`) are correctly flagged as needing one-line resolution-site work
beyond the pure field addition. Calibration is conservative and consistent with
PB-X/S/T historical yields (`feedback_pb_yield_calibration.md`).

## 5. Reframing — DEFENSIBLE

Cost surface (`Cost::Sacrifice(TargetFilter)` + `ActivationCost.sacrifice_filter`)
is fully wired with 13+ shipping cards. Effect surface
(`Effect::SacrificePermanents`) genuinely lacks a filter field. The planner's
title was misleading; the memo's correction is materially correct, not
cosmetic.

## Top issues

**None.** The memo is unusually clean: every claim checked traces to source,
the dispatch chain (`card_definition.rs` → `replay_harness.rs` → `abilities.rs`
→ `hash.rs`) was walked end-to-end per `feedback_verify_full_chain.md`, and
the verdict line, scope, tests, and risk paragraphs all line up.

## Recommendation

**PROCEED** to PB-SFT dispatch with the memo's Section 6 dispatch-ready scope:
field addition on `Effect::SacrificePermanents`, optional 3-line "another" roll-in,
5 mandatory tests, ~16 dispatch sites, ~1 implementation session.
