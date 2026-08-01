# Primitive WIP — PB-DX1 (the intervening-if dropped in the runtime lowering)

<!-- last_updated: 2026-08-01 -->

> Previous occupant: **PB-DP10 (decision-gate widening) — SHIPPED**, `scutemob-158`, merge
> `16ffcfd0`, PROTOCOL 31 / HASH 68 unmoved, tests **3,928** on main. **That closed the PB-DP
> suite (DP1..DP10).** Its WIP file is preserved verbatim at
> `memory/primitives/pb-wip-DP10-archive.md`.
> The seed re-rank (`scutemob-159`) retired the PB-RS queue; the authoritative queue is now
> `memory/primitives/seed-rerank-2026-07-27.md` §4, **PB-DX1..PB-DX18**.

- **PB**: PB-DX1 — rank 1 of the PB-DX queue. Seed **OOS-DP6-1** (+ riders OOS-DP6-5, OOS-DP6-9).
- **Task**: `scutemob-160`
- **Branch**: `feat/pb-dx1-the-intervening-if-dropped-in-the-runtime-lowering-oo`
- **Class**: **CORRECTNESS** — live-wrong on a `Complete`, deck-legal def; unbounded loop.
- **Phase**: plan
- **Plan**: `memory/primitives/pb-plan-DX1.md`
- **Review file**: `memory/primitives/pb-review-DX1.md`
- **Wire prediction**: **HASH** bump if fix (a) is taken (`TriggeredAbilityDef` sits inside
  `Characteristics`, `state/hash.rs:3337`). Prediction, not a licence — gate-compute.

## Premise re-verification (done before planning, on current branch head `3d73763d`)

All four legs of OOS-DP6-1 confirmed against source:

1. `crates/engine/src/testing/replay_harness.rs:2382` `build_face_ability_vectors` — **34**
   occurrences of `intervening_if: None`, hardcoded at every push site. Self-documented at
   `:2563` ("Condition and InterveningIf are separate types; conditional combat-damage triggers
   are rare and deferred").
2. Type mismatch is real: card-def field is `Option<Condition>`
   (`card-types/src/cards/card_definition.rs`), runtime field is `Option<InterveningIf>`
   (`card-types/src/state/game_object.rs:817` — a **2-variant** enum:
   `ControllerLifeAtLeast(u32)`, `SourceHadNoCounterOfType(CounterType)`).
   `TriggeredAbilityDef` is at `game_object.rs:884`, `intervening_if` at `:889`.
3. Live paths reach the lowering: `rules/face.rs:104` (transform / face change) and
   `rules/resolution.rs:864` (disturb / cast-transformed ETB). It is **also** called from
   `enrich_spec_from_def` (`replay_harness.rs:3750`), which is the universal card-def →
   runtime-object lowering used by `state/builder.rs` object specs and by
   `crates/simulator/src/legal_actions.rs` — i.e. it is not test-only.
4. `crates/card-defs/src/defs/aurelia_the_warleader.rs:32` carries
   `intervening_if: Some(Condition::IsFirstCombatPhase)` on its `WhenAttacks` ability, and the
   file has **no `completeness` field**, so it defaults to `Complete` and `validate_deck`
   admits it.

Conclusion: premise **holds**. Proceed to plan.
