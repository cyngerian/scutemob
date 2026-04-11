# Primitive WIP: PB-X — A-42 Tier 1 Micro-PB

batch: PB-X
title: Exclusion pump filter + dynamic P/T in LayerModification + Cost::ExileSelf
cards_unblocked: 6 of 8 A-42 Tier 1 (Crippling Fear, Eyeblight Massacre, Olivia's Wrath, Balthor the Defiled, Obelisk of Urd, City on Fire). Metallic Mimic deferred (needs 4th primitive — `LayerModification::AddChosenCreatureType`). Heritage Druid out of scope (TapNCreatures cost framework).
started: 2026-04-11
phase: closed
plan_file: memory/primitives/pb-plan-X.md (DONE)
review_file: memory/primitives/pb-review-X.md (DONE)

## Review (2026-04-11)
findings: 7 (HIGH: 1, MEDIUM: 3, LOW: 3)
verdict: needs-fix
- **C1 HIGH**: FIXED 2026-04-11. `obelisk_of_urd.rs` — rewrote "As this enters, choose a creature type" from `AbilityDefinition::Triggered` to `AbilityDefinition::Replacement { trigger: WouldEnterBattlefield, modification: ChooseCreatureType(...), is_self: true }`. Mirrors Urza's Incubator / Vanquisher's Banner / Morophon pattern exactly.
- **E1 MEDIUM**: FIXED 2026-04-11. Replaced all 6 "CR 701.10" citations with "CR 118.12 + CR 406 + CR 602.2c" (abilities.rs, card_definition.rs, hash.rs ×2, game_object.rs, balthor_the_defiled.rs). Also fixed 4 citations in primitive_pb_x.rs test file.
- **C2 MEDIUM**: FIXED 2026-04-11. Added `test_balthor_activated_reanimates_black_and_red` to primitive_pb_x.rs — activates via `Command::ActivateAbility` with ExileSelf, verifies (a) Balthor in exile, (b) black+red creatures on battlefield, (c) green creature stays in graveyard.
- **C3 MEDIUM**: FIXED 2026-04-11. Added `test_obelisk_of_urd_chosen_type_pump` (anti-C1 observability window test — Humans get +2/+2 immediately, Goblins unchanged), `test_city_on_fire_triples_damage` (2→6 via apply_damage_doubling), `test_city_on_fire_does_not_triple_opponent_sources` (opponent sources unchanged).
- **E2 LOW**: FIXED 2026-04-11. Expanded `ModifyBothDynamic` doc in continuous_effect.rs to cite CR 608.2h explicitly and state substitution invariant.
- **E3 LOW**: FIXED 2026-04-11. Simplified `exile_self` block in abilities.rs:636-643 — dropped dead `pre_exile_counters`/`is_creature` captures and `let _ = (...)` discard.
- **E4 LOW**: SKIPPED (review marked optional/low priority; no real dispatch path to defend).
- **C4 LOW**: SKIPPED (review: no fix required).
- **Positives**: ModifyBothDynamic locked-at-resolution test is model full-dispatch test; ActivationCost hash field-count audit correct (9/9); schema version sentinel correctly bumped to 2; Balthor filter resolved affirmatively (plan open question 4).

## Fix Gates (2026-04-11)
- cargo test --all: ALL PASS (16 tests in primitive_pb_x.rs, no failures in workspace)
- cargo clippy --workspace -- -D warnings: CLEAN (0 warnings)
- cargo build --workspace: CLEAN (replay-viewer + TUI build)
- cargo fmt --check: CLEAN

## Scope (from a42-retriage-2026-04-10.md + 2026-04-11 reclassification)

PB-X is a **micro-PB** bundling three small independent primitives whose absence blocks A-42 Tier 1 authoring:

1. **`EffectFilter::AllCreaturesExcludingSubtype(SubType)` + `AllCreaturesExcludingChosenSubtype`** — "all non-Elf creatures", "all non-Vampire creatures", and "creatures not of the chosen type". Blocks Crippling Fear, Eyeblight Massacre, Olivia's Wrath. Discriminants 32, 33.
2. **`LayerModification::ModifyBothDynamic { amount: Box<EffectAmount>, negate: bool }`** — substituted at `Effect::ApplyContinuousEffect` execution time into a concrete `ModifyBoth(i32)` per CR 608.2h. New variant rather than migration (76 call sites of existing `ModifyBoth(i32)`). Discriminant 25. Blocks Olivia's Wrath.
3. **`Cost::ExileSelf` + `ActivationCost.exile_self: bool`** — activated-ability cost that exiles the source permanent. LKI handled by existing `embedded_effect` plumbing in `abilities.rs` (mirrors `sacrifice_self`). Blocks Balthor the Defiled.

**Explicitly NOT in scope** (deferred to other PBs):
- `TapNCreatures` cost variant (Heritage Druid) — own PB.
- `LayerModification::AddChosenCreatureType` (Metallic Mimic) — fourth primitive; spawn PB-Y micro-PB.
- Obelisk of Urd / City on Fire — verified during plan phase as **already authorable**; will be authored in the PB-X session as free wins.

## Scope Boundary (enforced)
- Three primitives only. No fourth.
- Plan phase confirmed: Metallic Mimic needs a fourth primitive. **Stopped and flagged**, did not expand.
- Followed full-chain verification discipline: feedback_verify_full_chain.md, feedback_verify_cr_before_implement.md, feedback_retriage_verification.md.

## Open Questions (resolved by plan; oversight to confirm before implement)
1. ~~`AllCreaturesExcludingSubtype(SubType)` shape vs general combinator?~~ — **Resolved**: two variants (static + chosen) with substitution at Effect::ApplyContinuousEffect time. Combinator is overkill.
2. ~~`ModifyBoth(i32,i32)` migration vs new variant?~~ — **Resolved**: new variant `ModifyBothDynamic`. 76 call sites of the existing form would all need touching for migration.
3. ~~`Cost::ExileSelf` LKI dispatch?~~ — **Resolved**: existing `embedded_effect` capture at `abilities.rs:294-313` handles it. Resolution path at `resolution.rs:1776` already falls back to embedded_effect when source is dead. Mirrors sacrifice_self plumbing exactly.
4. ~~Metallic Mimic / Obelisk / City on Fire authorability?~~ — **Resolved**: Mimic blocked (4th primitive); Obelisk + City on Fire already authorable, will be done in PB-X session.

## Open Questions for Oversight (still pending)
1. `ModifyBothDynamic` sign handling: `negate: bool` (proposed) vs. `EffectAmount::Negate` combinator.
2. Hash version bump policy — confirm constant location and bump value.
3. Metallic Mimic disposition: PB-Y micro-PB (proposed) vs. fold into PB-Q ChooseColor.
4. Balthor's `ReturnAllFromGraveyardToBattlefield` body — needs grep at impl time to confirm per-player + color filter exists. May force partial authoring of Balthor if absent.
5. Exact `Cost::ExileSelf` discriminant in HashInto for Cost — count at impl.

## Step Checklist (PLAN — DONE)
- [x] 1. Read memory/card-authoring/a42-retriage-2026-04-10.md + a42-tier4-diagnosis-2026-04-10.md for current Tier 1 status
- [x] 2. Walk full primitive chain for each of the 8 Tier 1 cards (filter → effect → layer → cost) — confirm what PB-X actually needs vs. what's already supported
- [x] 3. Read engine sources: effects/mod.rs (EffectFilter), rules/layers.rs + continuous_effect.rs (LayerModification), rules/casting.rs + abilities.rs (Cost payment), cards/helpers.rs (exports)
- [x] 4. CR lookups: 602.1, 608.2h, 118.12, 701.10
- [x] 5. Write plan file memory/primitives/pb-plan-X.md with design for all 3 primitives, test list, card def patches, explicit "do not implement this session" marker
- [x] 6. Grep call sites for LayerModification::ModifyBoth — decided new variant (76 sites)
- [x] 7. Note hash version bump policy: yes (adds enum variants → discriminant chain extends)
- [x] 8. Do NOT implement this session — end plan phase, wait for oversight to green-light implement

## Step Checklist (IMPLEMENT — COMPLETE 2026-04-11)
- [x] 1. Engine changes per plan (EffectFilter +2, LayerModification +1, Cost +1, ActivationCost +field, payment block, substitution, hash arms) — continuous_effect.rs, game_object.rs, hash.rs, layers.rs, effects/mod.rs, rules/abilities.rs, replay_harness.rs, card_definition.rs (3 token specs). All exhaustive matches covered.
- [x] 2. Card defs: Crippling Fear (crippling_fear.rs), Eyeblight Massacre (eyeblight_massacre.rs), Olivia's Wrath (olivias_wrath.rs), Balthor the Defiled (balthor_the_defiled.rs), Obelisk of Urd (obelisk_of_urd.rs), City on Fire (city_on_fire.rs) — all 6 authored and compiling.
- [x] 3. Unit tests in tests/primitive_pb_x.rs — 12 tests across all three primitives + card integrations. All pass.
- [x] 4. Build verification — cargo test --all (all pass), cargo clippy -- -D warnings (0 warnings), cargo build --workspace (clean), cargo fmt --check (clean).
- [x] 5. Hash version bump — schema version sentinel `2u8.hash_into(&mut hasher)` added at top of `public_state_hash()` in hash.rs.
- [x] 6. No new TODOs introduced; deferred Metallic Mimic noted in scope boundary. ActivationCost.exile_self hashed (PB-S H1 defense); test #11 (`test_exile_self_field_participates_in_hash`) verifies. 20+ test files bulk-fixed to add `exile_self: false` to ActivationCost struct literals.

## Hazards / Carry-forward
- PB-S-L01..L06 residuals in abilities.rs / mana_solver — opportunistic only; not in PB-X scope
- ActivationCost gains a new field (`exile_self`) — HIGH RISK for PB-S H1-style hash gap. Implementer MUST field-count `HashInto for ActivationCost` against the struct definition. Test #14 defends.
- `AllCreaturesExcludingChosenSubtype` and `ModifyBothDynamic` are DSL placeholders — must NEVER be stored in a `ContinuousEffect`. Substitution at Effect::ApplyContinuousEffect is mandatory; debug_assert arms in layers.rs catch substitution bugs.
- Full-chain verification for Balthor's body still has open question 4 (per-player color-filtered reanimate). Implementer must verify before authoring the activated body or partially author with TODO.
- replay-viewer view_model.rs + TUI stack_view.rs: exhaustive matches on KeywordAbility/StackObjectKind — PB-X does NOT touch either, but verify with `cargo build --workspace` after impl.
