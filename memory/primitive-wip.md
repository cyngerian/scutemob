# Primitive WIP: PB-OS2 — optional-cost sacrifice power (EF-EF1-A)

batch: OS2
task: scutemob-128
branch: feat/pb-os2-optional-cost-sacrifice-power-ef-ef1-a-maypaytheneffe
started: 2026-07-19
phase: review

Plan: `memory/primitives/pb-plan-OS2.md`. Review: `memory/primitives/pb-review-OS2.md`.

## Brief (THE PLAN IS `memory/primitives/oos-retriage-plan-2026-07-18.md` §3 PB-OS2)
CORRECTNESS / micro. `EffectAmount::PowerOfSacrificedCreature` reads
`ctx.sacrificed_creature_lki`, populated only at the **activated-cost** site
(`handle_activate_ability` → `stack_obj.sacrificed_creature_lki`) and the
`Effect::SacrificePermanents` executor (effects/mod.rs:3445-3476). The **optional-cost**
sacrifice path — `Effect::MayPayThenEffect` (effects/mod.rs:3393-3414) →
`try_pay_optional_cost` → `pay_optional_cost` (effects/mod.rs:8096-8182) → for
`Cost::Sacrifice`, `sacrifice_permanents_for_player` (effects/mod.rs:7832) — **discards**
the returned LKI vec (`let _ = ...`, explicit EF-EF1-A deferral note at ~8128-8143). So
"you may sacrifice a creature; if you do, [X] where X = its power" resolves X = 0.

Fix (mirrors the activated-cost site; the layer-resolved pre-zone-move capture ALREADY
exists inside `sacrifice_permanents_for_player`):
- `pay_optional_cost` returns `Vec<SacrificedCreatureLki>` (sacrifice branch = the returned
  vec; `Cost::Sequence` extends; every other cost = `vec![]`).
- `try_pay_optional_cost` returns the vec on success (e.g. `Option<Vec<..>>`, `None` = didn't pay).
- `Effect::MayPayThenEffect` executor: after a successful pay, set
  `ctx.sacrificed_creature_lki = returned; ctx.sacrifice_fired = !returned.is_empty();`
  **before** `execute_effect_inner(then)`. Keep `exclude_self` source threading + the
  `ctx.controller = pid` payer rebind intact.

**No new DSL type → NO PROTOCOL/HASH bump.** `SacrificedCreatureLki` already exists and is
already hashed on the stack object. If a bump is forced, STOP and re-scope.

## Roster sweep (all_cards(), pre-verified)
- **disciple_of_freyalise** (partial → **Complete**): front-face ETB "you may sacrifice
  another creature; if you do, gain X life and draw X cards, X = its power" — the sole flip.
  Back face already Complete. Wire front face as
  `MayPayThenEffect { cost: Cost::Sacrifice(TargetFilter{exclude_self:true, creature}),
  then: Sequence[ GainLife{PowerOfSacrificedCreature}, DrawCards{PowerOfSacrificedCreature} ] }`.
- **birthing_ritual**: stays partial — blocked on the top-7 DIG (OOS-EF10-1 / PB-OS8), NOT this.
- **ziatora_the_incinerator**: stays partial — blocked on optional-sacrifice-inside-a-*triggered*
  ability + reflexive "when you do" (Triggered has no `may` field), NOT this.

## Mandatory tests
- **Decoy (layer-resolution pin)**: anthem (e.g. +2/+0) in play; sacrifice a creature whose
  BASE power ≠ layer-resolved power; assert gained-life/drawn = layer-resolved power (not base),
  AND that the captured creature is the *sacrificed* one, not a wrong-creature decoy on the board.
- **Decline path**: may-pay declined (no eligible sacrifice / pay-when-able false) → `then` does
  not run, no life gained, no draw, no stale `ctx.sacrificed_creature_lki` leaking to siblings.

## Close-out
Close EF-EF1-A: replace the deferral note in `pay_optional_cost`; CLOSED banner in source
finding docs (`memory/card-authoring/w-empty-engine-findings-2026-07-17.md` line ~16 references
it; check w-pb2/pb-plan-EF10); update PB-OS2 entry in the retriage plan §3.

## Steps
- [x] 1. Plan — primitive-impl-planner → pb-plan-OS2.md (DONE)
- [x] 2. Implement engine change (pay_optional_cost / try_pay_optional_cost / MayPayThenEffect) — `crates/engine/src/effects/mod.rs`: `pay_optional_cost` (~8096) now returns `Vec<SacrificedCreatureLki>`; `try_pay_optional_cost` (~8168) now returns `Option<Vec<SacrificedCreatureLki>>`; `Effect::MayPayThenEffect` executor (~3393) sets `ctx.sacrificed_creature_lki`/`ctx.sacrifice_fired` before `execute_effect_inner(then)`; scratch probe caller (~8077) discards with `let _ =`. `cargo check -p mtg-engine` clean.
- [x] 3. Flip disciple_of_freyalise front face → Complete — `crates/card-defs/src/defs/disciple_of_freyalise.rs` wired front-face ETB as `Triggered { WhenEntersBattlefield, MayPayThenEffect { Cost::Sacrifice(exclude_self:true creature), then: Sequence[GainLife, DrawCards] both PowerOfSacrificedCreature } }`; `completeness: Completeness::Complete`; header comment updated. `cargo check -p mtg-card-defs` clean.
- [x] 4. Decoy test (anthem + wrong-creature) + decline-path test — `crates/engine/tests/primitives/pb_ef10_sacrifice_driven_amounts.rs`: `test_may_pay_sacrifice_captures_layer_resolved_power` (decoy, anthem+wrong-creature pin), `test_may_pay_sacrifice_declined_no_capture_no_leak` (decline + no-leak DECOY), `test_disciple_of_freyalise_front_face_gains_and_draws_power` (card-def integration). All 3 pass post-fix; revert-and-rerun confirmed decoy + card-integration tests FAIL against the pre-fix engine (decline test correctly passes both ways — it validates the negative path).
- [x] 5. Confirm no PROTOCOL/HASH bump — untouched; `test_pb_ef10_version_sentinels` still asserts PROTOCOL_VERSION==18, HASH_SCHEMA_VERSION==55, unchanged, passing.
- [x] 6. Review — primitive-impl-reviewer → pb-review-OS2.md; CLEAN BILL, zero HIGH/MEDIUM/LOW, 2 informational NITs (no fix). No fix phase needed.
- [x] 7. Green gates: build/test/clippy/fmt + check-defs-fmt — `cargo build --workspace` clean; `cargo test --all` all-green (0 failed, incl. `core::card_defs_fmt`); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --check` clean; `tools/check-defs-fmt.sh` clean (1798 defs). PROTOCOL/HASH sentinel tests (`core::protocol_schema`, `core::hash_schema`, 38 tests) all pass untouched — no version bump. TODO sweep on disciple_of_freyalise.rs: 0 remaining.
- [ ] 8. Close EF-EF1-A in source docs + plan; /review; Completion Sequence (deferred to close-out — not run by the implementer per this task's scope)
