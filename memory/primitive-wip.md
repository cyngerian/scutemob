# Primitive WIP: PB-TS — TokenSpec.count u32 → EffectAmount (dynamic token-count primitive)

batch: PB-TS
title: TokenSpec.count u32 → EffectAmount (dynamic token-count primitive — CR 111.4 / 113.7)
cards_unblocked_estimated: 4 confirmed in-scope (Phyrexian Swarmlord, Chasm Skulker, Krenko Mob Boss, Izoni Thousand-Eyed). 1 OOS (Anim Pakal — non-Gnome attacker trigger filter, separate primitive seed). Per `feedback_pb_yield_calibration.md`: discount EffectAmount-PB yield 50–65%; expect 2–4 ships, AC requires ≥2.
cards_unblocked_confirmed_post_plan: 4 (Phyrexian Swarmlord, Chasm Skulker, Krenko Mob Boss, Izoni Thousand-Eyed token-half/primary mechanic)
started: 2026-04-30
phase: plan-complete
plan_file: memory/primitives/pb-plan-TS.md
review_file: memory/primitives/pb-review-TS.md
shape_chosen: A — replace `TokenSpec.count: u32` with `count: EffectAmount` directly. Default = `EffectAmount::Fixed(1)`. Predefined helpers keep `count: u32` parameter, convert internally to `EffectAmount::Fixed(count as i32)`. `Effect::CreateToken` and `Effect::CreateTokenAndAttachSource` resolve via `resolve_amount(state, &spec.count, ctx).max(0) as u32` BEFORE feeding into `apply_token_creation_replacement` (u32 boundary preserved). Rationale: type-system enforcement (per feedback_verify_full_chain.md), single source of truth, mirrors existing EffectAmount precedent for DrawCards/GainLife/Scry counts.
hash_version_pre: 13 (PB-CC-C-followup)
hash_version_post: 14 (PB-TS — TokenSpec field shape change)

## Task reference
- ESM task: scutemob-16
- Branch: feat/pb-ts-tokenspeccount-u32-effectamount-dynamic-token-count-pr
- Acceptance criteria: 3724 (engine primitive landed, 5 tests a-e), 3725 (≥2 cards re-authored, no-TODO primary mechanic), 3726 (HASH 13→14 + sentinel sweep), 3727 (cargo gates + test count > 2720), 3728 (/review PASS or PASS-WITH-NITS, 0 HIGH/MEDIUM open before signal-ready), 3729 (plan + review memos committed), 3730 (OOS blockers appended to pb-retriage-CC.md as STOP-AND-FLAG seeds)

## Context

PB-CC-A shipped `EffectAmount::PlayerCounterCount` (Vishgraz, deferred Phyrexian Swarmlord on PB-TS).
PB-CC-C-followup shipped `AbilityDefinition::CdaModifyPowerToughness` + Layer-7c live-eval (Vishgraz CDA + Fuseling CDA halves), bumping HASH 12→13.

The remaining counter-count siblings have **token creation** halves blocked on a different primitive: `TokenSpec.count` is a fixed `u32` field. The fix scope is mechanically simpler than PB-CC-C-followup — only the spell-effect path (`Effect::CreateToken` / `Effect::CreateTokenAndAttachSource`) needs to consult `EffectAmount` via the existing `resolve_amount(state, &EffectAmount, ctx)` helper at execution time; static-ability path is N/A (tokens aren't continuous effects).

### Confirmed in-scope cards (post-MCP verification)

| Card | Trigger | X formula | EffectAmount variant |
|---|---|---|---|
| **Phyrexian Swarmlord** | Upkeep | poison counters opponents have | `PlayerCounterCount{EachOpponent, Poison}` (PB-CC-A) |
| **Chasm Skulker** | Death (LKI) | +1/+1 counters on this | `CounterCount{Source, PlusOnePlusOne}` |
| **Krenko, Mob Boss** | {T} activated | Goblins you control | `PermanentCount{filter: subtypes Goblin, controller You}` |
| **Izoni, Thousand-Eyed** | ETB | creature cards in your graveyard | `CardCount{zone: Graveyard, player: You, filter: card_type Creature}` |

### Out of scope

- **Anim Pakal, Thousandth Moon** — token-count half is PB-TS, but the trigger ("with one or more non-Gnome creatures") is blocked on a non-existent trigger filter for "non-{subtype} creature attackers". File as STOP-AND-FLAG seed in pb-retriage-CC.md.
- Izoni's second activated ability ("{B}{G}, sacrifice another creature: gain 1, draw") is a separate primitive (sacrifice-other cost). The ETB token half is the *primary mechanic* (per AC 3725 phrasing).
- Replicating Ring (counter-threshold trigger gate), Phyresis Outbreak per-target poison, Vraska -9 special-case — all already flagged in pb-retriage-CC.md; do NOT widen.

## STOP-AND-FLAG triggers

1. **TokenSpec used in non-spell context (e.g. dungeon completion)** — verify dungeon.rs `spec.count = 2` site still compiles. If TokenSpec.count becomes `EffectAmount`, dungeon.rs must produce `EffectAmount::Fixed(2)`. Mechanical migration.
2. **`apply_token_creation_replacement` takes `count: u32`** — must remain `u32` (replacement-effect doubling math operates on resolved counts, not EffectAmount). Resolve `EffectAmount` to u32 BEFORE calling apply_token_creation_replacement.
3. **`make_token` per-iteration loop currently uses `0..spec.count`** — must use the resolved u32 count, not the EffectAmount.
4. **Predefined helpers (`treasure_token_spec`, `food_token_spec`, etc.)** — keep `count: u32` parameter for API ergonomics; convert internally to `EffectAmount::Fixed(count)`. Do NOT force callers to wrap `EffectAmount::Fixed(...)` for static counts.
5. **Per `feedback_pb_yield_calibration.md`**: yield expectation is ≥2; falling below 2 = hidden compound blocker → STOP and report.
6. **Per `feedback_verify_full_chain.md`**: walk every dispatch site (DSL → resolve → apply_replacement → make_token → hash → tests). Don't stop at TokenSpec field existence.
7. **Hash bump rule**: TokenSpec field shape changes — must bump HASH_SCHEMA_VERSION 13→14 (per `memory/conventions.md`). Sentinel sweep across all sentinel-assertion test files.
8. **Living Weapon / `CreateTokenAndAttachSource` missing replacement-effect call** (NEW, identified during planning): today the code does NOT apply token-doubling replacements at this dispatch site. Pre-existing bug. **Default action: stop-and-flag, file separate seed**. Runner can fix opportunistically only if existing tests confirm the expected behavior.

## Reference docs (for planner)

- `memory/primitives/pb-retriage-CC.md` PB-TS seed lines 358-361, OOS scoping 214-224, sequencing rationale 377-390
- `memory/primitives/pb-plan-CC-C-followup.md` — recent template for planning structure
- `memory/primitives/pb-review-CC-C-followup.md` (if exists) — recent reviewer template
- `feedback_pb_yield_calibration.md`, `feedback_verify_full_chain.md`, `feedback_oversight_primitive_category_not_cards.md`
- `crates/engine/src/cards/card_definition.rs:3099-3166` (TokenSpec definition + Default)
- `crates/engine/src/cards/card_definition.rs:3171-3400` (predefined token spec helpers)
- `crates/engine/src/effects/mod.rs:540-585` (Effect::CreateToken dispatch)
- `crates/engine/src/effects/mod.rs:595-660` (Effect::CreateTokenAndAttachSource dispatch)
- `crates/engine/src/rules/replacement.rs:2603-2637` (apply_token_creation_replacement)
- `crates/engine/src/state/hash.rs:4290+` (HashInto for TokenSpec)
- `crates/engine/src/state/dungeon.rs:372` (`spec.count = 2`)
- `crates/engine/tests/blood_tokens.rs:812` (`assert_eq!(spec.count, 1, ...)`)
- `crates/engine/src/cards/defs/{phyrexian_swarmlord,chasm_skulker,krenko_mob_boss,izoni_thousand_eyed,anim_pakal_thousandth_moon}.rs` — TODO citations to clear / preserve

## Planner checklist

- [x] Step 1: CR research — quote 111.1, 111.4, 614.1, 113.7, 700.2 verbatim with notes on token-count modification timing (also added 608.2h, 122.1, 122.6 — load-bearing for resolution-time + LKI semantics)
- [x] Step 2: Engine architecture walk — every dispatch site (15 sites enumerated)
- [x] Step 3: Shape decision (planner-chosen, documented with rationale) — Shape A
- [x] Step 4: Dispatch unification verdict (yield bound) — yield = 4
- [x] Step 5: Hash strategy — bump 13→14, sentinel sweep file list (5 files)
- [x] Step 6: Test plan — 5 mandatory (a-e) numbered with CR citations
- [x] Plan file written: `memory/primitives/pb-plan-TS.md`

## Implementation checklist (runner fills in)

- [ ] Engine change 1: TokenSpec field shape change implemented per plan
- [ ] Engine change 2: Effect::CreateToken / CreateTokenAndAttachSource resolve_amount integration
- [ ] Engine change 3: apply_token_creation_replacement boundary preserved (resolved count → u32)
- [ ] Engine change 4: predefined helper constructors updated
- [ ] Engine change 5: dungeon.rs `spec.count` site updated
- [ ] Engine change 6: hash arm + HASH_SCHEMA_VERSION bump 13→14 + history entry 14
- [ ] Engine change 7: sentinel-assertion test files updated
- [ ] Card def 1: phyrexian_swarmlord.rs re-authored (Upkeep token, no TODO)
- [ ] Card def 2: chasm_skulker.rs re-authored (Death LKI token, no TODO)
- [ ] Card def 3: krenko_mob_boss.rs re-authored (T-activated token, no TODO)
- [ ] Card def 4: izoni_thousand_eyed.rs re-authored (ETB token primary mechanic, no TODO)
- [ ] Anim Pakal blocker appended to memory/primitives/pb-retriage-CC.md
- [ ] Izoni sacrifice-another cost blocker appended to memory/primitives/pb-retriage-CC.md
- [ ] Tests: 5 mandatory in tests/primitive_pb_ts.rs (or matching file)
- [ ] cargo test --workspace green, count > 2720
- [ ] cargo build --workspace clean (replay-viewer + TUI exhaustive matches verified)
- [ ] cargo fmt --check clean
- [ ] cargo clippy --all-targets -- -D warnings clean

## Reviewer checklist

- [ ] CR rules independently verified
- [ ] Card oracle text verified via MCP for all re-authored cards
- [ ] Every dispatch site walked and confirmed correct
- [ ] Hash arm + version bump + history entry verified
- [ ] Test (a-e) verified
- [ ] No scope creep (Anim Pakal trigger filter out of scope)
- [ ] Review file written: `memory/primitives/pb-review-TS.md`
- [ ] Verdict: PASS or PASS-WITH-NITS (0 HIGH, 0 MEDIUM open at signal-ready)
