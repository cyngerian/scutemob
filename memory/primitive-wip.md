# Primitive WIP: PB-TS — TokenSpec.count u32 → EffectAmount (dynamic token-count primitive)

batch: PB-TS
title: TokenSpec.count u32 → EffectAmount (dynamic token-count primitive — CR 111.4 / 113.7)
cards_unblocked_estimated: 4 confirmed in-scope (Phyrexian Swarmlord, Chasm Skulker, Krenko Mob Boss, Izoni Thousand-Eyed). 1 OOS (Anim Pakal — non-Gnome attacker trigger filter, separate primitive seed). Per `feedback_pb_yield_calibration.md`: discount EffectAmount-PB yield 50–65%; expect 2–4 ships, AC requires ≥2.
cards_unblocked_confirmed_post_plan: 3 (Phyrexian Swarmlord, Krenko Mob Boss, Izoni Thousand-Eyed token-half/primary mechanic)
cards_unblocked_confirmed_post_review: 3 valid (Phyrexian Swarmlord, Krenko after E1 fix, Izoni); Chasm Skulker has wrong game state per C1 HIGH finding (death-trigger CounterCount{Source} from graveyard reads empty counters → 0 tokens). Threshold of 2 still met after fix-phase clears C1 + E1.
cards_unblocked_confirmed_post_rereview: 3 ships (Phyrexian Swarmlord, Krenko, Izoni primary). Chasm Skulker correctly reverted to TODO citing OOS-TS-4; first ability (counter-on-draw) still ships but does not count toward AC 3725 token-create primary mechanic. Threshold of ≥2 met.
started: 2026-04-30
phase: review-complete
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

- [x] Engine change 1: TokenSpec field shape change implemented — `count: u32` → `count: EffectAmount`, Default `EffectAmount::Fixed(1)` (card_definition.rs ~3114, ~3156)
- [x] Engine change 2: Effect::CreateToken / CreateTokenAndAttachSource resolve_amount integration — `resolve_amount(state, &spec.count, ctx).max(0) as u32` added before replacement boundary (effects/mod.rs ~543, ~603)
- [x] Engine change 3: apply_token_creation_replacement boundary preserved — u32 signature unchanged, resolves BEFORE calling replacement
- [x] Engine change 4: predefined helper constructors updated — treasure/food/clue/blood/army/zombie_decayed all use `EffectAmount::Fixed(count as i32)` internally; also builder.rs (afterlife, living weapon germ), replacement.rs (fabricate servo)
- [x] Engine change 5: dungeon.rs `spec.count` sites updated — 5 helpers `count: 1` → `EffectAmount::Fixed(1)`, Muiral's graveyard `spec.count = 2` → `EffectAmount::Fixed(2)`
- [x] Engine change 6: hash arm + HASH_SCHEMA_VERSION bump 13→14 + history entry 14 — state/hash.rs; HashInto for TokenSpec dispatches through EffectAmount::hash_into (no new arm needed)
- [x] Engine change 7: sentinel-assertion test files updated — primitive_pb_cc_a.rs, primitive_pb_cc_c_followup.rs, pbt_up_to_n_targets.rs (×2), effect_sacrifice_permanents_filter.rs, pbn_subtype_filtered_triggers.rs, pbd_damaged_player_filter.rs, pbp_power_of_sacrificed_creature.rs (8 total)
- [x] Card def 1: phyrexian_swarmlord.rs re-authored — `PlayerCounterCount{EachOpponent, Poison}` token count, no TODO
- [x] Card def 2: chasm_skulker.rs re-authored — `CounterCount{Source, PlusOnePlusOne}` token count (resolves at execution time), no TODO **— REVIEWED: produces wrong game state per C1 HIGH; counter snapshot primitive missing — REVERTED to TODO in fix-phase, blocked on OOS-TS-4**
- [x] Card def 3: krenko_mob_boss.rs re-authored — `PermanentCount{Goblin creature, You}` token count, no TODO **— REVIEWED: timing_restriction wrongly set to SorcerySpeed per E1 HIGH — FIXED to None in fix-phase**
- [x] Card def 4: izoni_thousand_eyed.rs re-authored — `CardCount{Graveyard, You, Creature}` token count primary mechanic; secondary ability left TODO (OOS seed OOS-TS-2 appended)
- [x] Anim Pakal blocker appended to memory/primitives/pb-retriage-CC.md (OOS-TS-1)
- [x] Izoni sacrifice-another cost blocker appended to memory/primitives/pb-retriage-CC.md (OOS-TS-2)
- [x] CreateTokenAndAttachSource missing replacement-effect call appended as OOS-TS-3
- [x] Tests: 5 mandatory in crates/engine/tests/primitive_pb_ts.rs — (a) Fixed(N) creates N tokens, (b) Fixed(0) clamp, (c) PermanentCount Krenko-style, (d) CounterCount from live source, (e) hash determinism + HASH_SCHEMA_VERSION=14 sentinel
- [x] cargo test --workspace green — 2725 tests passing (was 2720, +5 new)
- [x] cargo build --workspace clean — replay-viewer + TUI exhaustive matches verified
- [x] cargo fmt --check clean — zero diffs
- [x] cargo clippy --all-targets -- -D warnings clean — zero warnings

## Reviewer checklist

- [x] CR rules independently verified (111.1, 111.4, 614.1, 614.1c, 113.7/113.7a, 608.2h, 122.1, 122.6, 603.10a, 400.7, 602.5d)
- [x] Card oracle text verified via MCP for all re-authored cards (Phyrexian Swarmlord, Chasm Skulker, Krenko Mob Boss, Izoni Thousand-Eyed)
- [x] Every dispatch site walked and confirmed correct (21 sites; full chain in review file)
- [x] Hash arm + version bump + history entry verified (HASH=14, history entry 14, 8 sentinel files updated)
- [x] Test (a-e) verified (all 5 present, all cite CR, all discriminating; test (d) deviation acknowledged)
- [x] No scope creep (Anim Pakal trigger filter out of scope; OOS-TS-1 filed)
- [x] Review file written: `memory/primitives/pb-review-TS.md`
- [x] Verdict: NEEDS-FIX — 2 HIGH (E1 Krenko sorcery-speed; C1 Chasm Skulker 0 tokens), 0 MEDIUM, 4 LOW (L1/L2/L3 stale sentinel comments; L4 missing OOS-TS-4 seed)

## Fix-phase prerequisites (for runner second pass)

1. **E1 (HIGH)**: `crates/engine/src/cards/defs/krenko_mob_boss.rs:48` — change `timing_restriction: Some(TimingRestriction::SorcerySpeed)` → `timing_restriction: None`. Krenko's tap ability is instant-speed per oracle text.
   - [x] FIXED 2026-04-30: `timing_restriction: None` set at krenko_mob_boss.rs:48.
2. **C1 (HIGH)**: `crates/engine/src/cards/defs/chasm_skulker.rs` — either revert second ability to TODO, OR rewrite the misleading "preserved through move_object_to_zone" comment to a TODO citation referencing OOS-TS-4. Reviewer recommends revert.
   - [x] FIXED 2026-04-30: Second ability reverted (option a). Replaced entire WhenDies AbilityDefinition::Triggered block with a TODO comment citing OOS-TS-4.
3. **L4 (LOW)**: Append OOS-TS-4 seed to `memory/primitives/pb-retriage-CC.md` documenting the pre-death counter snapshot primitive (CR 603.10a "looks back in time" semantics for WhenDies / WhenLeavesBattlefield + EffectAmount::CounterCount{Source}).
   - [x] FIXED 2026-04-30: OOS-TS-4 appended to pb-retriage-CC.md with full description including 2 engine paths, yield, and references.
4. **L1, L2, L3 (LOW polish — optional but recommended)**: Update stale prose comments in `pbd_damaged_player_filter.rs:588,594-595`, `pbp_power_of_sacrificed_creature.rs:765-768,779-780`, `pbn_subtype_filtered_triggers.rs:540-553` to reference PB-TS bump 13→14.
   - [x] FIXED 2026-04-30: All three files updated — header/inline comments now cite "PB-TS bump 13→14 (TokenSpec.count: u32 → EffectAmount)".

## Fix-phase gate results (2026-04-30)

- `cargo build --workspace`: PASS (clean, 8.35s)
- `cargo test --workspace`: PASS — 2725 tests, 0 failed
- `cargo fmt --check`: PASS — zero diffs
- `cargo clippy --all-targets -- -D warnings`: PASS — zero warnings

## Re-review checklist (post fix-phase commit `4fde5d66`)

- [x] Read fix-phase commit `4fde5d66` and verified diff scope (krenko_mob_boss.rs one-line, chasm_skulker.rs revert + comment, pb-retriage-CC.md OOS-TS-4 append, three sentinel test files prose updates)
- [x] E1 verified RESOLVED — krenko_mob_boss.rs:48 reads `timing_restriction: None`; oracle re-verified via MCP confirms no sorcery restriction
- [x] C1 verified RESOLVED — chasm_skulker.rs second ability fully reverted to TODO comment block citing OOS-TS-4 + state/mod.rs:420 + CR 603.10a; aspirationally-wrong "Toothy precedent" wording removed; first ability (counter-on-draw) intact
- [x] L4 verified RESOLVED — OOS-TS-4 entry in pb-retriage-CC.md (lines 441-472) cites CR 603.10a / 113.7a / 400.7 / 122.2, references state/mod.rs:420 and effects/mod.rs:6011-6012, names two engine paths, identifies Toothy as also-affected
- [x] L1 verified RESOLVED — pbd_damaged_player_filter.rs prose now cites "PB-TS bumped HASH_SCHEMA_VERSION 13→14"
- [x] L2 verified RESOLVED — pbp_power_of_sacrificed_creature.rs header now says "sentinel 14"; inline comment cites PB-TS bump
- [x] L3 verified RESOLVED — pbn_subtype_filtered_triggers.rs history block now contains the PB-TS bump line
- [x] No regressions introduced by the fix commit — diff is purely card-def revert, constant change, comment edits, OOS seed append; no engine code paths altered
- [x] No new findings opened
- [x] Review file updated with re-review section: `memory/primitives/pb-review-TS.md` (lines 291+)
- [x] Verdict: PASS (0 HIGH / 0 MEDIUM open; all LOWs resolved including load-bearing L4)

## Next step

Branch is **PASS** and ready for /review + signal-ready on ESM task scutemob-16. Coordinator should
proceed to merge to main and close PB-TS WIP.
