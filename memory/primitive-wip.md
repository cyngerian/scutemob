# Primitive WIP: PB-AC3 — Dynamic P/T & count amounts (CDA residual)

batch: PB-AC3
title: Dynamic P/T & count amounts (CDA residual)
cards_affected: ~14 (discounted; real roster to be identified from oracle text)
started: 2026-07-08
phase: review
plan_file: memory/primitives/pb-plan-AC3.md

## Coordinator decisions on plan (2026-07-08)
- **HandSize**: ADD it (criterion 4225 names it literally) as a thin alias
  `EffectAmount::HandSize { player }` delegating to the same counting logic as
  `CardCount{Hand}` in BOTH resolve_amount and resolve_cda_amount; doc-comment flags
  it as a convenience alias. Discriminant 21.
- **Hash collision at LayerModification disc 26** (RemoveSuperType vs ModifyPowerDynamic):
  FIX in-batch — reassign RemoveSuperType → next free disc; schema is already bumping.

## Implementation progress (2026-07-08)
- [x] `EffectAmount::AttackingCreatureCount` (disc 19), `TappedCreatureCount` (disc 20) —
      `crates/engine/src/cards/card_definition.rs` (enum def), `resolve_amount`
      (`effects/mod.rs`) and `resolve_cda_amount` (`rules/layers.rs`) both wired (lockstep).
- [x] `EffectAmount::HandSize { player }` (disc 21) — thin alias delegating to
      `CardCount{Hand}` in both resolve_amount and resolve_cda_amount, per coordinator decision.
- [x] `LayerModification::SetBothDynamic { amount: Box<EffectAmount> }` (disc 28, Layer 7b) —
      `state/continuous_effect.rs` enum def; substitution arm in `effects/mod.rs`
      `ApplyContinuousEffect`; apply arm in `rules/layers.rs` `apply_layer_modification`.
- [x] `CombatState::is_attacking` helper — `state/combat.rs`; wired into both count arms
      (not left dead).
- [x] hash.rs: 3 new `EffectAmount` HashInto arms (19/20/21) + 1 `LayerModification` arm
      (28) + `HASH_SCHEMA_VERSION` 29 -> 30 with changelog block.
- [x] Hash collision verdict: CONFIRMED REAL — `RemoveSuperType` and `ModifyPowerDynamic`
      both hashed prefix `26u8`. Fixed: `RemoveSuperType` reassigned to discriminant 29.
      Updated all 21 test files asserting `HASH_SCHEMA_VERSION, 29u8` -> `30u8`.
- [x] Card fixes: keep_watch (AttackingCreatureCount), throne_of_the_god_pharaoh
      (Triggered end-step + TappedCreatureCount), mirror_entity (SetBothDynamic +
      AddAllCreatureTypes), krenko_tin_street_kingpin (PowerOf(Source) token count),
      ulvenwald_hydra (CdaPowerToughness lands), wight_of_the_reliquary
      (CdaModifyPowerToughness graveyard-creature-count; SacrificeAnother TODO
      intentionally left — separate pre-existing DSL gap, out of PB-AC3 scope),
      storm_kiln_artist (CdaModifyPowerToughness artifact count).
- [x] PARTIAL TODOs updated (galadhrim_ambush, mishra_claimed_by_gix,
      ashaya_soul_of_the_wild, multani_yavimayas_avatar) — functional code untouched
      per coordinator instruction, comments updated to name remaining gap.
      OOS cards (grand_warlord_radha, harvest_season, commissar_severina_raine,
      dawnstrike_vanguard, opposition, springleaf_drum, mothdust_changeling,
      glare_of_subdual, iroas_god_of_victory, dolmen_gate, reconnaissance) left alone.
- [x] Unit tests file `pb_ac3_dynamic_pt_counts.rs` — 19 tests, all passing.
- [x] Gates — ALL GREEN:
      `cargo build --workspace` clean; `cargo test --all` 2938 passed / 0 failed
      (2919 baseline + 19 new); `cargo clippy --all-targets -- -D warnings` clean;
      `cargo fmt --check` clean (after one `cargo fmt` auto-fix pass).
      `python3 tools/authoring-report.py`: clean 946->951 (54.1%->54.4%), todo
      621->616, total_todos 1122->1106.
      **Finding for coordinator**: expected +6 clean (6 CLEAN-roster cards fixed)
      but delta is +5. Root cause: `tools/authoring-report.py`'s `classify_file`
      empty-abilities regex `abilities:\s*vec!\[\s*\]\s*,` has NO word boundary, so
      it false-positive-matches the substring in `activated_abilities: vec![],` /
      `mana_abilities: vec![],` (present in Krenko's TokenSpec literal, pre-existing,
      unrelated to this batch's edit). Krenko is misclassified "empty" both before
      and after this batch's fix — its card def is functionally correct and covered
      by `test_krenko_tokens_equal_power` (loads the actual shipped CardDefinition).
      This is a pre-existing tooling defect, not a PB-AC3 regression; left unfixed
      (out of declared PB-AC3 scope — tooling change, not engine/card/test).

## Task reference
- ESM task: scutemob-45
- Branch: feat/pb-ac3-dynamic-pt-count-amounts-cda-residual
- Acceptance criteria:
  - 4225: Engine primitives implemented — `LayerModification::ModifyBoth` accepting
    `EffectAmount`; `EffectAmount::{AttackingCreatureCount, TappedCreatureCount,
    HandSize}`; power-based token count — each with tests citing CR sections (layer
    interactions covered)
  - 4226: Review pass complete; primitive-impl-reviewer findings written; all
    HIGH/MEDIUM fixed
  - 4227: Backfill complete; all cards unblocked by PB-AC3 re-authored, stale
    TODO/ENGINE-BLOCKED markers removed, reviewed by card-batch-reviewer
  - 4228: All gates green; authoring-report rerun and coverage delta posted as
    task comment

## Scope (from campaign-plan-2026-05-16.md §2, PB-AC3 row)
Primitives to add:
- `LayerModification::ModifyBoth` accepting `EffectAmount` — dynamic P/T set/modify
  (CDA residual). Layer 7a (CDA), dependency-ordered per CR 613.4.
- `EffectAmount::{AttackingCreatureCount, TappedCreatureCount, HandSize}` — dynamic
  count amounts.
- Power-based token count (token count = a creature's power / some dynamic value).

CR refs (613 layers, 107.3) are ADVISORY — verify against the CR via the mtg-rules
MCP. Card rosters in the plan are advisory; identify the real roster from oracle
text (feedback_oversight_primitive_category_not_cards). Grep card defs for BOTH
`// TODO` and `// ENGINE-BLOCKED` markers citing dynamic-P/T / CDA / count-amount
patterns.

## Hazards (from task description)
1. **LAYER SYSTEM batch** — load `memory/gotchas-rules.md` before planning. CDA P/T
   is Layer 7a; characteristic-defining abilities apply in the CDA sublayer,
   dependency-ordered (CR 613.4). All battlefield characteristic reads must go
   through `calculate_characteristics()` (W3-LC discipline).
2. Card DSL gotcha: `*/*` CDA creatures use `power: None, toughness: None`
   (NOT `Some(0)`).
3. Verify KW/AbilDef/SOK discriminant chain from current code before adding variants.
4. New struct fields / mutable runtime fields MUST be added to `state/hash.rs`
   HashInto impls (PB-AC1 review HIGH was exactly this).
5. Exhaustive matches in `tools/tui/src/play/panels/stack_view.rs` AND
   `tools/replay-viewer/src/view_model.rs` need arms for every new enum variant —
   verify with `cargo build --workspace`.
6. Do NOT commit phantom `.claude/skills/*/SKILL.md` deletions in the worktree.

## Gates
- cargo build --workspace
- cargo test --all
- cargo clippy --all-targets -- -D warnings
- cargo fmt --check
- python3 tools/authoring-report.py → post clean-coverage delta as task comment

## Commit prefix
W6-prim:
