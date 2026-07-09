---
pb: PB-AC7
title: Type-changing & ability-removal
phase: review
plan_file: memory/primitives/pb-plan-AC7.md
review_file: memory/primitives/pb-review-AC7.md
---

# PB-AC7 — Type-changing & ability-removal

## Scope (task scutemob-50, campaign-plan §2 PB-AC7 row)

**4 primitives:**
1. `Effect::LoseAbilities` — Layer 6 ability removal, typically "until end of
   turn" ("loses all abilities"). Timestamp-ordered against ability *grants*
   in the same layer (CR 613.1f / 613.7).
2. `Effect::SetCreatureTypes` — Layer 4 "changes to" semantics. **SETS** the new
   creature type(s), *removing* all prior creature types (CR 205.1b). Distinct
   from the additive `AddAllCreatureTypes` / `AddCreatureType` already present.
3. **One-shot Layer-4 type override with duration** — a resolution-time effect
   that registers a Layer-4 continuous effect with a duration (until end of
   turn / permanently), as opposed to a static ability's Layer-4 modification.
4. **Aura/Equipment/Vehicle subtype filter** — spell/permanent subtype filter
   usable for targeting (`TargetFilter`) and effect selection (`EffectFilter`).

## CR refs — ADVISORY ONLY, verify each via mtg-rules MCP
613.1 (layer order), 205.1b / 205.3 (subtypes; changing types), 613.1d (Layer 4),
613.1f (Layer 6), 613.6/613.7 (timestamps & dependency), 611.2c (continuous
effects from resolved spells/abilities), 708 (face-down objects).

**Do NOT grep the CR file** — it has bare `\r` line endings, so rule-number greps
silently match nothing. Use the mtg-rules MCP for all CR verification.

## Hazards (from task brief)
1. **THIS BATCH TOUCHES THE LAYER SYSTEM.** Load `memory/gotchas-rules.md`
   before planning. Type-changing is Layer 4; ability add/remove is Layer 6;
   timestamp/dependency ordering per CR 613. Study how existing until-end-of-turn
   layer modifications register duration and expiry.
   **Precedent**: `AddAllCreatureTypes` (Mirror Entity) was moved Layer 6 → Layer 4
   `TypeChange` in a PB-AC3 review fix. Follow that placement.
2. **LoseAbilities interactions to test**: granted-then-removed ordering by
   timestamp; face-down 2/2-no-text override (the face-down layer override runs
   *before* the layer loop — verify LoseAbilities composes with it, not against
   it); W3-LC discipline (all battlefield characteristic reads through
   `calculate_characteristics()`).
3. **Any new mutable/runtime fields MUST be added to `state/hash.rs` `HashInto`
   impls with mutation-verified tests.** This was a review HIGH in PB-AC1 *and*
   PB-AC5. One-shot layer effects with durations often add stack/object state.
   Also: every new enum variant needs a hash arm, and `HASH_SCHEMA_VERSION`
   (currently 33) bumps with a changelog entry + sentinel update across test files.
4. Verify the KW/AbilDef/SOK discriminant chain **from current code** before
   adding variants — do not trust remembered numbers.
5. Exhaustive matches in `tools/tui/src/play/panels/stack_view.rs`
   (`StackObjectKind`) and `tools/replay-viewer/src/view_model.rs`
   (`StackObjectKind` + `KeywordAbility`). Run `cargo build --workspace` after
   every impl phase. **`cargo build` does NOT compile test targets** — gate on
   `cargo test --all` actually running (PB-AC6 process note).
6. Harness wiring (`testing/replay_harness.rs`, `script_schema.rs`) if scripts
   need the new effects.
7. Do NOT commit phantom `.claude/skills/*` deletions in fresh worktrees.

## Roster
Discounted yield ~14 cards. The **planner** identifies the real roster from
oracle text — grep card defs for BOTH `// TODO` and `// ENGINE-BLOCKED` markers
citing lose-abilities / becomes-type / type-change / aura-equipment-vehicle-filter
patterns. PB-AC6 just ran a marker-correction sweep, so markers should be current.
Card rosters in plan docs are advisory; oracle text via MCP is authoritative
(feedback_oversight_primitive_category_not_cards).

## Close includes backfill
PB is not done until every unblocked card is re-authored and its stale markers
are deleted, and reviewed by card-batch-reviewer.

## Phases
- [x] plan (primitive-impl-planner → pb-plan-AC7.md)
- [x] implement (primitive-impl-runner) — ENGINE PHASE ONLY, see below. Card backfill
  (Kenrith's Transformation, Eaten by Piranhas, Darksteel Mutation, Sram, Leaf-Crowned
  Visionary, Final Showdown mode 0, Vraska −2) deliberately NOT done — out of scope
  per coordinator brief; deferred to the `backfill` phase.
- [x] review (primitive-impl-reviewer → pb-review-AC7.md) — 1 HIGH, 2 MEDIUM, 2 LOW found
- [x] fix (primitive-impl-runner) — H1, M1, M2 resolved; see "Fix progress" below and
  `memory/primitives/pb-review-AC7.md` "Fix pass" section
- [ ] backfill (bulk-card-author + card-batch-reviewer)
- [ ] close

## Fix progress (2026-07-09, primitive-impl-runner)

H1/M1/M2 all resolved. `cargo test --all` = 3034 passed / 0 failed (3032 baseline + 2 new
tests). `cargo build --workspace` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt
--check` all clean. No `HASH_SCHEMA_VERSION` bump (only `LazyLock` statics + a pure helper
fn added, no new field/variant). Full details in `memory/primitives/pb-review-AC7.md` "Fix
pass" section — summary:
- H1: `SetCardTypes` now applies CR 205.1a correlated-subtype removal via 6 new CR-205.3
  subtype-set statics (`state/types.rs`) + `correlated_card_types()` helper.
- M1: 3 payload-aware `depends_on` arms added in `rules/layers.rs` (not blanket — each
  justified against the CR 613.8a test individually).
- M2: 2 new tests + 1 strengthened integration test; both H1 and M1 fixes proven non-vacuous
  by temporarily reverting each and confirming the targeted tests fail, then restoring.

## Implementation progress (engine phase, this session)

**Scope decision applied exactly as briefed** — 2 of 4 brief-named primitives were
already expressible and were NOT re-added (`Effect::LoseAbilities`, one-shot Layer-4
type override with duration); only `SetCreatureTypes`/`SetCardTypes` and
`spell_subtype_filter` are net-new.

### Engine changes
- [x] `LayerModification::SetCreatureTypes(OrdSet<SubType>)` — Layer 4 (CR 205.1a).
  `crates/engine/src/state/continuous_effect.rs` (variant), `crates/engine/src/rules/layers.rs`
  `apply_layer_modification` (Reconfigure-idiom filter-and-replace-creature-subset arm),
  `crates/engine/src/state/hash.rs` (discriminant 30, verified next-free against current
  code — RemoveSuperType was 29, nothing used 30/31).
- [x] `LayerModification::SetCardTypes(OrdSet<CardType>)` — Layer 4 companion (CR 205.1a),
  same three files, discriminant 31. Adopted (not skipped) — makes both variants
  load-bearing for the CR-faithful Kenrith/Eaten-by-Piranhas/Darksteel-Mutation backfill
  (preserves Legendary supertype, which `SetTypeLine` would wipe).
- [x] `depends_on` (CR 613.8) — NO new dependency arm added for `SetCreatureTypes`/
  `SetCardTypes` vs `AddSubtypes`/`AddCardTypes`. Decision documented inline at
  `rules/layers.rs::depends_on`: both new variants only replace ONE subset of the type
  line, so a co-resident `AddSubtypes` targeting a disjoint subtype set (e.g. a land
  subtype) is order-independent — pure timestamp order is correct. Locked in by
  `test_set_creature_types_layer4_dependency_with_add_subtypes` (both orders assert the
  same union result).
- [x] `TriggerCondition::WheneverYouCastSpell.spell_subtype_filter: Option<Vec<SubType>>` —
  `cards/card_definition.rs` (field + doc), `rules/abilities.rs` (post-processing OR-match
  against `spell_subtypes`, already computed at line ~3368), `state/hash.rs` (destructure +
  hash arm). CR 205.1a.
- [x] 21 explicit `WheneverYouCastSpell {` construction sites across `cards/defs/` updated
  with `spell_subtype_filter: None` (verified via compiler errors — matched the plan's
  count exactly). 2 additional sites found in `crates/engine/tests/trigger_variants.rs`
  (not in the plan's list — plan's file inventory covered `cards/defs/` only, not
  `tests/`) — fixed. `sram_senior_edificer.rs` / `leaf_crowned_visionary.rs` /
  `tyvar_kell.rs` untouched (comment-only mentions, no construction site — backfill scope).
- [x] `HASH_SCHEMA_VERSION` 33 → 34, changelog entry added. All 26 `HASH_SCHEMA_VERSION,
  33u8` sentinel occurrences across 25 test files bulk-updated to `34u8` (verified no
  stray `33u8` sentinel assertions remain; unrelated discriminant-33 literals in hash.rs
  for other enums left untouched).

### Tests — `crates/engine/tests/pb_ac7_type_change_ability_removal.rs` (new, 14 tests, all passing)
1. `test_set_creature_types_replaces_creature_subtypes_keeps_card_types` — CR 205.1a
2. `test_set_creature_types_preserves_noncreature_subtypes` — CR 205.1a
3. `test_set_card_types_replaces_card_types_preserves_supertypes` — CR 205.1a
4. `test_darksteel_mutation_keeps_indestructible` — CR 205.1b + 613.7 (full integration:
   RemoveAllAbilities + later-timestamp AddKeyword(Indestructible) + SetCardTypes +
   SetCreatureTypes + SetPowerToughness, composed)
5. `test_granted_then_removed_ordering_by_timestamp` — CR 613.7, both orders
6. `test_lose_abilities_vs_face_down_override` — CR 708.2
7. `test_lose_abilities_one_shot_until_eot` — CR 514.2/611.2a, via real
   `Effect::ApplyContinuousEffect` + real `rules::layers::expire_end_of_turn_effects`
8. `test_set_creature_types_layer4_dependency_with_add_subtypes` — CR 613.8, both orders
9. `test_spell_subtype_filter_positive` — CR 205.1a, full CastSpell integration
   (Equipment/Vehicle/Aura spells, stack-inspection assertion)
10. `test_spell_subtype_filter_negative` — CR 205.1a, vanilla creature spell does not fire
11. `test_spell_subtype_filter_none_matches_all` — regression guard for the 21 `None` sites
12. `test_hash_schema_version_is_34` — sentinel
13. `test_hash_distinguishes_set_creature_types_payload` — discriminant 30 vs SetTypeLine's
    discriminant 2, plus payload distinctness
14. `test_hash_distinguishes_spell_subtype_filter` — None vs Some(vec![Elf]) hash distinctly

### Gates (all run, real output)
- `cargo build --workspace` — clean
- `cargo test --all` — 3023 passed (3009 baseline + 14 new), 0 failed
- `cargo clippy --all-targets -- -D warnings` — clean (2 doc-comment `doc_lazy_continuation`
  findings fixed: hash.rs changelog entry needed a `- 34:` list marker matching the
  established style; a test doc-comment's mid-paragraph `+` was read as a list item)
- `cargo fmt --check` — clean (ran `cargo fmt` once, mechanical reformatting only)

### Deviations from plan
- None in engine scope. Plan's "21 sites" count was exact; the 2 extra `trigger_variants.rs`
  test-file sites were not enumerated in the plan (file inventory scoped to `cards/defs/`)
  but were mechanically required for `cargo test --all` to compile — fixed in-scope per the
  runner brief ("only where a construction site mechanically needs the field to compile").
- Card backfill (7 cards in the plan's roster) intentionally NOT done — coordinator scope
  says a later agent handles backfill. No TODOs were deleted from card defs this session.

## Gates
- cargo build --workspace
- cargo test --all
- cargo clippy --all-targets -- -D warnings
- cargo fmt --check
- python3 tools/authoring-report.py → post clean-coverage delta as task comment

## Task reference
- ESM task: scutemob-50
- Branch: feat/pb-ac7-type-changing-ability-removal
- Commit prefix: `W6-prim:` (engine) / `W6-cards:` (backfill)
- Acceptance criteria: 4395 (primitives+tests), 4396 (review+hash), 4397 (backfill),
  4398 (gates+coverage delta)
