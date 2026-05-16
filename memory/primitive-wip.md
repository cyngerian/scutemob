# Primitive WIP: PB-EWC-D — ObjectFilter::CreatureControlledByOfSubtype + bind_object_filter OwnedByOpponentsOf rebind

batch: PB-EWC-D
title: ObjectFilter::CreatureControlledByOfSubtype { controller: PlayerId, subtype: SubType } variant + bind_object_filter OwnedByOpponentsOf rebind (sub-gap E2 from pb-review-EWC.md)
cards_unblocked: 1 confirmed in-scope — Dragonstorm Globe (Each Dragon you control enters with an additional +1/+1 counter on it.)
started: 2026-05-15
phase: review-complete
plan_file: memory/primitives/pb-plan-EWC-D.md
review_file: memory/primitives/pb-review-EWC-D.md
shape_chosen: (a) — additive new variant `ObjectFilter::CreatureControlledByOfSubtype { controller: PlayerId, subtype: SubType }`. Rationale: purely additive, no migration of existing CreatureControlledBy(PlayerId) call sites required. Mirrors PB-CD CreatureControlledBy pattern. PB-EAT precedent (`EntersAsAdditionalType { subtype: SubType }`) confirms SubType-by-value is acceptable in enum variants. See plan file Section "Design choice".
hash_version_pre: 22 (PB-XA2)
hash_version_post: 23 (PB-EWC-D — new ObjectFilter variant, discriminant 9)

## Task reference
- ESM task: scutemob-28
- Branch: feat/pb-ewc-d-objectfilter-subtype-bindobjectfilter-ownedbyoppone
- Acceptance criteria:
  - 3902: Engine surface — new ObjectFilter::CreatureControlledByOfSubtype variant (chose option (a) additive)
  - 3903: Hash arm + HASH_SCHEMA_VERSION bump 22→23 + ALL existing PB hash canary sentinels bumped to 23u8 with PB-EWC-D citation
  - 3904: Resolver — wire object_matches_filter for new subtype-aware variant (CR 614.1c receiver-filter for ETB replacements)
  - 3905: bind_object_filter — add OwnedByOpponentsOf(PlayerId(0)) → OwnedByOpponentsOf(controller) rebind (~3 lines, E2 from pb-review-EWC.md)
  - 3906: Card — author Dragonstorm Globe counter half via new variant (controller=Self placeholder, subtype=Dragon); remove inline TODO
  - 3907: Tests — new primitive_pb_ewcd.rs with HASH-23 sentinel, PartialEq discriminator, serde-default backward-compat, Dragonstorm Globe positive functional, bind_object_filter OwnedByOpponentsOf regression test
  - 3908: OOS seeds — file OOS-EWCD-N entries for any new receiver-filter gaps observed (card-type, supertype, multi-subtype)
  - 3909: cargo gates green; /review (primitive-impl-reviewer) PASS or PASS-WITH-NITS; resolve HIGH/MEDIUM inline

## Engine surface (per plan)

- `state/replacement_effect.rs` — add `ObjectFilter::CreatureControlledByOfSubtype { controller: PlayerId, subtype: SubType }` variant with doc comment citing CR 614.1c / 613.1d.
- `state/hash.rs` — add hash arm discriminant 9 (next after PB-CD's `CreatureControlledBy` = 8); bump HASH_SCHEMA_VERSION 22→23 with history entry citing PB-EWC-D.
- `rules/replacement.rs`:
  - extend `object_matches_filter` with the new variant arm — layer-resolved creature type (CR 613.1d) + controller equality + subtype membership.
  - extend `bind_object_filter` with TWO new arms:
    - (E2 fix) `OwnedByOpponentsOf(PlayerId(0))` → `OwnedByOpponentsOf(controller)` ~3 lines
    - `CreatureControlledByOfSubtype { controller: PlayerId(0), subtype }` → `{ controller, subtype }` ~3 lines
  - the non-self `WouldEnterBattlefield { filter }` arm at line 1813 already calls `bind_object_filter(filter, controller)` (PB-EWC), so picks up both new arms automatically.

## Card def

- `cards/defs/dragonstorm_globe.rs` — replace TODO with `AbilityDefinition::Replacement` carrying `trigger: WouldEnterBattlefield { filter: CreatureControlledByOfSubtype { controller: PlayerId(0), subtype: SubType("Dragon".into()) } }` + `modification: EntersWithCounters { counter: PlusOnePlusOne, count: Box::new(EffectAmount::Fixed(1)) }`. Verified oracle text via MCP lookup_card.

## Tests (per plan)

`crates/engine/tests/primitive_pb_ewcd.rs`:
1. `test_pb_ewcd_hash_schema_version_is_23` — sentinel canary.
2. `test_pb_ewcd_partial_eq_discriminates_subtype_variant` — variant equality + inequality.
3. `test_pb_ewcd_serde_default_backward_compat` — JSON roundtrip for new variant.
4. `test_dragonstorm_globe_dragon_etb_gets_extra_counter` — positive functional.
5. `test_dragonstorm_globe_non_dragon_etb_no_counter` — negative functional.
6. `test_bind_object_filter_rebinds_owned_by_opponents_of_for_wouldenterbattlefield` — E2 regression.

Sentinel bumps (16 files):
- crates/engine/tests/primitive_pb_cc_a.rs, primitive_pb_xa.rs, primitive_pb_cc_c_followup.rs, primitive_pb_xs_e.rs, primitive_pb_xs.rs, primitive_pb_lki_power.rs, primitive_pb_ts.rs, primitive_pb_eat.rs, primitive_pb_lki_cc.rs, primitive_pb_ewc.rs, primitive_pb_xa2.rs
- crates/engine/tests/pbt_up_to_n_targets.rs (×2 sites), pbn_subtype_filtered_triggers.rs, pbd_damaged_player_filter.rs, pbp_power_of_sacrificed_creature.rs, effect_sacrifice_permanents_filter.rs
- Rewrite sentinel messages to cite PB-EWC-D uniformly.

## Implementation checklist (runner fills in)

- [x] Engine change 1: `ObjectFilter::CreatureControlledByOfSubtype { controller, subtype }` variant — state/replacement_effect.rs (placed after CreatureControlledBy, doc comment citing CR 613.1d / 614.1c)
- [x] Engine change 2: hash arm discriminant 9 — state/hash.rs (after CreatureControlledBy discriminant 8; hashes controller + subtype via HashInto)
- [x] Engine change 3: HASH_SCHEMA_VERSION 22→23 + history entry 23 — state/hash.rs (entry documents new variant + OwnedByOpponentsOf bind fix)
- [x] Engine change 4: `object_matches_filter` arm for new variant — rules/replacement.rs (layer-resolved creature + subtype + controller check, CR 613.1d)
- [x] Engine change 5: `bind_object_filter` arms for new variant + OwnedByOpponentsOf (E2) — rules/replacement.rs (two new arms added)
- [x] Card def 1: dragonstorm_globe.rs — authored counter half with CreatureControlledByOfSubtype{Dragon} filter, removed TODO
- [x] Tests: primitive_pb_ewcd.rs — 6 tests (all pass)
- [x] Sentinel sweep — 17 sentinel sites bumped 22→23 with PB-EWC-D citation (pbt_up_to_n_targets.rs had ×2 sites)
- [x] OOS seeds appended to pb-retriage-CC.md (OOS-EWCD-1, OOS-EWCD-2, OOS-EWCD-3)
- [x] cargo build --workspace clean (2026-05-15)
- [x] cargo test --workspace green — 2817 tests passing (+6 new from primitive_pb_ewcd.rs)
- [x] cargo clippy --workspace --all-targets -- -D warnings clean (zero warnings)
- [x] cargo fmt --all -- --check clean (zero diffs)

## Reviewer checklist (post-implement)

- [x] CR rules independently verified (614.1c, 613.1d, 122.6) via MCP `get_rule`
- [x] Dragonstorm Globe oracle text re-verified via MCP `lookup_card` (matches verbatim; 2025-04-04 ruling noted)
- [x] Every dispatch site walked and confirmed correct (29 sites; full chain documented in pb-review-EWC-D.md plumbing trace table — variant declaration → hash arm → bind rebind (both new arms) → match resolution → card def DSL → tests)
- [x] Hash arm + version bump + history entry verified (HASH=23, history entry 23, 18 sentinel sites updated to 23u8; no stale 22u8 against HASH_SCHEMA_VERSION literal)
- [x] All 18 sentinel sites updated (17 listed in plan + 1 new in primitive_pb_ewcd.rs); no stale 22u8
- [x] Tests verified — 6 present, all cite CR. Tests 1-5 discriminating. **Test 6 IS NOT discriminating** (T1 HIGH finding: serde-only, does not invoke bind_object_filter; would pass even if the new OwnedByOpponentsOf arm at replacement.rs:532-534 were deleted)
- [x] No scope creep — 4 engine files touched (matches plan exactly); 1 card def; OOS seeds filed for card-type / supertype / multi-subtype variants
- [x] Review file written: memory/primitives/pb-review-EWC-D.md
- [x] Verdict: **PASS-WITH-NITS** — 1 HIGH (T1 test-validity for Test 6), 0 MEDIUM, 2 LOW (E1 hash-determinism test gap; E2 OOS-EWC-3 close-marker bookkeeping)

## Recommended fix-phase (per reviewer)

1. **T1 (HIGH per conventions.md test-validity rule)**: Rewrite Test 6 to actually exercise `bind_object_filter` for `OwnedByOpponentsOf(PlayerId(0))`. Recommended approach: add `#[cfg(test)] pub use bind_object_filter` (or change to `pub(crate) fn` + cfg-test re-export at crate root), then call it directly in the test and assert the result equals `OwnedByOpponentsOf(actual_controller)`. ~10-15 lines net. Alternative: synthetic CardDefinition + register through `register_permanent_replacement_abilities` + inspect `state.replacement_effects` (~40-60 lines).
2. **E1 (LOW)**: Add hash-determinism sub-test for the new variant (~10 lines).
3. **E2 (LOW, close-phase)**: Annotate OOS-EWC-3 in `pb-retriage-CC.md:732+` as CLOSED by PB-EWC-D.

## Fix-phase checklist (scutemob-28)

- [x] T1 (HIGH RESOLVED 2026-05-15): `bind_object_filter` changed to `pub fn` at `rules/replacement.rs:521` + doc-comment noting "Public for test access; not part of the engine's runtime API." Added `#[doc(hidden)] pub use rules::replacement::bind_object_filter` to `lib.rs:57`. Rewrote Test 6 to call `bind_object_filter` directly and assert: (a) `OwnedByOpponentsOf(PlayerId(0))` → `OwnedByOpponentsOf(controller)`; (b) non-placeholder `OwnedByOpponentsOf(PlayerId(3))` is unchanged; (c) `CreatureControlledByOfSubtype{PlayerId(0), Dragon}` → `{controller, Dragon}`; (d) passthrough cases (Any, AnyCreature, non-zero variants) are unchanged. Test now fails if the OwnedByOpponentsOf arm at replacement.rs:532-534 is deleted.
- [x] E1 (LOW RESOLVED 2026-05-15): Added new Test 7 `test_pb_ewcd_hash_determinism_for_creature_controlled_by_of_subtype` — asserts two equal `CreatureControlledByOfSubtype{p1, Dragon}` instances produce identical hashes; asserts different controller → different hash; different subtype → different hash; different variant `CreatureControlledBy(p1)` → different hash. Mirrors PB-LKI-Power test (d) pattern.
- [x] E2 (LOW RESOLVED 2026-05-15): OOS-EWC-3 entry in `pb-retriage-CC.md:758+` annotated with `Status (2026-05-15): CLOSED by PB-EWC-D (commit 27c1381b, fix-phase scutemob-28)`.

## Fix-phase gate results (2026-05-15)

- `cargo test --test primitive_pb_ewcd`: PASS — 7 tests passing (was 6, +1 Test 7 hash-determinism)
- All 3 findings resolved (T1 HIGH + E1 LOW + E2 LOW)
- Full workspace gates: pending below

---

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

---

# Primitive WIP: PB-LKI-CC — `EffectAmount::CounterCountAtLastKnownInformation` (LKI snapshot for WhenDies / WhenLeavesBattlefield)

batch: PB-LKI-CC
title: EffectAmount::CounterCountAtLastKnownInformation — LKI snapshot for WhenDies / WhenLeavesBattlefield (CR 603.10a)
cards_unblocked: Chasm Skulker (WhenDies token half unblocked by PB-TS OOS-TS-4), Toothy Imaginary Friend
started: 2026-04-29
phase: review-complete
plan_file: memory/primitives/pb-plan-LKI-CC.md
review_file: memory/primitives/pb-review-LKI-CC.md
hash_version_pre: 14 (PB-TS)
hash_version_post: 15 (PB-LKI-CC — LKI snapshot fields on 4 GameEvent variants)

## Task reference
- ESM task: scutemob-17
- Branch: feat/pb-lki-cc-effectamountcountercount-lki-snapshot-for-whendies

## Implementation checklist

- [x] Engine change 1: `EffectAmount::CounterCountAtLastKnownInformation { counter: CounterType }` variant added (disc 17) — `card_definition.rs`
- [x] Engine change 2: `PendingTrigger.lki_counters: OrdMap<CounterType, u32>` field added + `blank()` init — `state/stubs.rs`
- [x] Engine change 3: `StackObject.lki_counters: OrdMap<CounterType, u32>` field added + `trigger_default()` init — `state/stack.rs`
- [x] Engine change 4: `EffectContext.lki_counters: Option<OrdMap<CounterType, u32>>` field added + constructors updated — `effects/mod.rs`
- [x] Engine change 5: LKI capture + flush in `abilities.rs` — `CreatureDied` SelfDies + SelfLeavesBattlefield arms use `pre_death_counters.clone()`
- [x] Engine change 6: `resolve_amount` arm for discriminant 17 — `effects/mod.rs`
- [x] Engine change 7: `resolve_cda_amount` arm for discriminant 17 — `rules/layers.rs`
- [x] Engine change 8: `HashInto` arm for discriminant 17 + HASH_SCHEMA_VERSION 14→15 + history entry 15 — `state/hash.rs`
- [x] Engine change 9: sentinel-assertion sweep across 6 files — all updated to assert 15
- [x] Card def 1: `chasm_skulker.rs` re-authored — WhenDies trigger uses `CounterCountAtLastKnownInformation{PlusOnePlusOne}` for token count; OOS-TS-4 TODO cleared
- [x] Card def 2: `toothy_imaginary_friend.rs` re-authored — WhenLeavesBattlefield trigger uses `CounterCountAtLastKnownInformation{PlusOnePlusOne}` for draw count
- [x] Tests (a-e): 5 mandatory tests in `crates/engine/tests/primitive_pb_lki_cc.rs`

## Fix-phase checklist (post review NEEDS-FIX)

- [x] E1 (HIGH RESOLVED): `pre_lba_counters: OrdMap<CounterType, u32>` field added to `GameEvent::AuraFellOff`, `GameEvent::PermanentDestroyed`, `GameEvent::ObjectExiled`, `GameEvent::ObjectReturnedToHand` with `#[serde(default)]`. All ~35 emit sites across abilities.rs, casting.rs, engine.rs, resolution.rs, turn_actions.rs updated. Four trigger arms in `check_triggers` updated to propagate `pre_lba_counters` → `PendingTrigger.lki_counters`. `state/hash.rs` match arms updated with `..` to ignore new fields.
- [x] E2 (LOW RESOLVED): `test_pb_lki_cc_hash_determinism` added + hash-determinism sub-test in sentinel test.
- [x] E3 (LOW RESOLVED): OOS-LKI-3 (cost-payment LKI / Workhorse) and OOS-LKI-4 (AnyCreatureDies LKI) appended to `pb-retriage-CC.md`.
- [x] C1 (LOW RESOLVED): `test_toothy_bounced_to_hand_draws_lki_counter_count`, `test_toothy_destroyed_draws_lki_counter_count`, `test_toothy_exiled_draws_lki_counter_count` added and passing.

## Fix-phase gate results (2026-04-29)

- `cargo build --workspace`: PASS (clean)
- `cargo test --all`: PASS — **2734 tests** (+4 from fix-phase: 3 Toothy regression tests + 1 hash-determinism test)
- `cargo fmt --check`: PASS — zero diffs
- `cargo clippy -- -D warnings`: PASS — zero warnings

## Verdict

**PASS-WITH-NITS** → upgraded to **PASS** after fix-phase. 0 HIGH / 0 MEDIUM open. All LOWs resolved.
Review file updated with fix-phase resolution section at `memory/primitives/pb-review-LKI-CC.md`.

## Next step

Branch is PASS and ready for signal-ready on ESM task scutemob-17. Coordinator should proceed to merge to main and close PB-LKI-CC.

---


---

# Primitive WIP: PB-LKI-Power — `EffectAmount::SourcePowerAtLastKnownInformation` (LKI source-power snapshot for WhenDies / WhenLeavesBattlefield)

batch: PB-LKI-Power
title: EffectAmount::SourcePowerAtLastKnownInformation (and SourceToughnessAtLastKnownInformation if needed) — LKI source-P/T snapshot for WhenDies / WhenLeavesBattlefield (CR 603.10a / 122.2)
cards_unblocked_estimated: 2 confirmed in-scope (Conclave Mentor death-trigger life-gain, Juri Master of the Revue death-trigger damage). Sweep may add others. Per `feedback_pb_yield_calibration.md`: ≥2 yield threshold met.
cards_unblocked_post_plan: 2 (Conclave Mentor + Juri Master). TODO sweep returned 2 forced adds; both already in scope. No additional pattern-sweep candidates. Toughness variant DEFERRED (no in-scope card needs it; rulings explicitly say "power"); discriminant 19 reserved for future seed OOS-LKI-Power-1.
cards_unblocked_post_review: 2 ships (Conclave Mentor + Juri Master). Both card defs verified against MCP oracle text + rulings; both pass dedicated tests + the discriminating LKI-after-zone-change test (c). Threshold of ≥2 met.
started: 2026-05-13
phase: PASS
plan_file: memory/primitives/pb-plan-LKI-Power.md
review_file: memory/primitives/pb-review-LKI-Power.md
hash_version_pre: 16 (PB-CD)
hash_version_post: 17 (PB-LKI-Power — LKI power snapshot fields on 5 GameEvent variants + new EffectAmount variant disc 18)

## Task reference
- ESM task: scutemob-19
- Branch: feat/pb-lki-power-lki-source-powertoughness-snapshot-for-whendies
- Acceptance criteria: 3762 (plan), 3763 (EffectAmount variants + hash changelog), 3764 (PendingTrigger/StackObject/EffectContext threading + sba.rs:540 snapshot site), 3765 (GameEvent pre_death_power/pre_death_toughness on 4-5 variants + abilities.rs sweep), 3766 (HASH 16→17 + Option tag-byte encoding + sentinel sweep), 3767 (Conclave Mentor + Juri TODOs cleared), 3768 (tests including LKI-after-zone-change), 3769 (cargo gates), 3770 (review + fixes), 3771 (CLAUDE.md + OOS seed close + authoring report regen)

## Context

PB-LKI-CC (HASH 15) shipped LKI **counter** snapshots for SelfDies / SelfLeavesBattlefield triggers. Filed OOS-LKI-Power seed (pb-retriage-CC.md:554) for the symmetric **power/toughness** problem. PB-CD (HASH 16) consumed counter-doubling capability but left Conclave Mentor's death trigger TODO pending this primitive. Juri Master of the Revue has the same blocker (TODO at line 37).

This PB mirrors the PB-LKI-CC dispatch chain field-for-field, swapping `lki_counters: OrdMap<CounterType,u32>` for `lki_power: Option<i32>`.

### Confirmed in-scope cards (post-MCP verification 2026-05-13)

- **Conclave Mentor** — "When this creature dies, you gain life equal to its power" (TODO at conclave_mentor.rs:41-43, names OOS-LKI-Power explicitly). Ruling 2020-06-23: "Use Conclave Mentor's power as it last existed on the battlefield to determine how much life you gain." Confirmed POWER (not toughness).
- **Juri, Master of the Revue** — "When Juri dies, it deals damage equal to its power to any target" (TODO at juri_master_of_the_revue.rs:37-38, names EffectAmount::SourcePower). Ruling 2020-11-10: "use its power from when it was last on the battlefield to determine how much damage is dealt. If that power was 0 or less, Juri deals no damage."

### Sweep results (2026-05-13)

TODO sweep `grep "TODO.*\(LKI.*[Pp]ower\|SourcePower\|EffectAmount::SourcePower\|OOS-LKI-Power\)"`:
- Returned 2 forced adds: conclave_mentor.rs and juri_master_of_the_revue.rs. Both already in scope.

Pattern sweep `grep "equal to its power"` over `crates/engine/src/cards/defs/`:
- Hits in spell defs, ETB triggers on the entering creature, granted activated abilities, Fight/Bite, and live-source activated abilities. **None** are SelfDies/SelfLeavesBattlefield contexts. Specifically NOT in scope: swords_to_plowshares.rs (spell), jagged_scar_archers.rs (live activated), bridgeworks_battle.rs (Fight wording), warstorm_surge.rs (live ETB on ANOTHER creature — different snapshot target), brash_taunter.rs (Fight), archdruids_charm.rs (Bite spell), ram_through.rs (spell), eomer_king_of_rohan.rs (live ETB on self), wolverine_riders.rs (ETB-on-OTHER trigger reading the entering creature's toughness), infectious_bite.rs (spell), legolas/legolasquick (granted activated abilities).

Confirmed in-scope yield: **2** (Conclave Mentor, Juri).

### Out of scope (planner verifies, files seeds if discovered)

- **`SourceToughnessAtLastKnownInformation`** — discriminant 19 reserved, NOT shipped. Both in-scope cards' rulings explicitly say "power." File as OOS-LKI-Power-1.
- **Master Biomancer ETB-replacement reading source's live power** — different DSL gap (replacement-side dynamic counter count, source is alive). File as OOS-LKI-Power-2.
- **Triggered abilities reading OTHER objects' P/T (not source's own LKI)** — different snapshot target.
- **Cost-payment LKI (Workhorse)** — already filed as OOS-LKI-3.
- **AnyCreatureDies LKI** — already filed as OOS-LKI-4.
- **GameEvent LBA hash arm symmetric extension** — pre-existing PB-LKI-CC inconsistency; preserve. File as OOS-LKI-Power-3.

## STOP-AND-FLAG triggers

1. If sweep finds a card whose oracle text says "equal to its toughness" on a SelfDies/SelfLeavesBattlefield trigger → in scope; add SourceToughnessAtLastKnownInformation. Otherwise toughness variant deferred (planner decided: NOT shipped this batch — no in-scope card).
2. If any in-scope card has additional non-LKI gaps, do NOT widen scope; file OOS seed.
3. Hash bump rule: HASH_SCHEMA_VERSION 16→17 + sentinel sweep across all 9 sentinel files (10 sentinel sites).
4. Per `feedback_verify_full_chain.md`: walk every dispatch site (DSL → resolve → snapshot → emit → trigger → resolution → hash → tests). 21 sites enumerated in plan Step 3.
5. Hash arm symmetric-extension creep: do NOT add `pre_lba_power` to AuraFellOff/PermanentDestroyed/ObjectExiled/ObjectReturnedToHand hash arms. Mirror PB-LKI-CC `..` precedent. File as OOS-LKI-Power-3 if revisited.
6. AnyCreatureDies arm at abilities.rs:4318: do NOT thread `triggering_creature_lki_power` in this PB. PB-LKI-CC's OOS-LKI-4 governs.
7. Cost-payment LKI for activated abilities: do NOT extend `EffectContext.sacrificed_creature_powers` in this PB. PB-LKI-CC's OOS-LKI-3 governs.
8. Yield drop below 2: STOP and report. Per `feedback_pb_yield_calibration.md` threshold is ≥2.
9. `calculate_characteristics` recursion at sba.rs:540: STOP and report unexpected layer interactions.

## Reference docs (for planner — used)

- `memory/primitives/pb-plan-LKI-CC.md` — primary template (mirrored field-for-field, OrdMap → Option<i32>).
- `memory/primitives/pb-review-LKI-CC.md` — reviewer template + fix-phase pattern.
- `memory/primitives/pb-retriage-CC.md:554` — OOS-LKI-Power seed (the source description).
- `crates/engine/src/cards/card_definition.rs:2376-2398` — `EffectAmount::CounterCountAtLastKnownInformation` definition + doc (template for new variant disc 18).
- `crates/engine/src/rules/sba.rs:540` — `pre_death_counters` snapshot site (where pre_death_power is added).
- `crates/engine/src/rules/abilities.rs:4015-4076` (CreatureDied), `:4351-4410` (AuraFellOff), `:5258-5269` (PermanentDestroyed), `:5311-5322` (ObjectExiled), `:5364-5375` (ObjectReturnedToHand) — 6 trigger arms touching `lki_counters` propagation, all need `lki_power` siblings.
- `crates/engine/src/state/stubs.rs:411-457` — `PendingTrigger.lki_counters` (add `lki_power` next to it).
- `crates/engine/src/state/stack.rs:475-541` — `StackObject.lki_counters` (add `lki_power` next to it).
- `crates/engine/src/effects/mod.rs:142, 168-205, 6360-6365` — `EffectContext.lki_counters` field + constructors + resolve_amount arm (add `lki_power` siblings).
- `crates/engine/src/state/hash.rs:95, 4529-4534, 2154-2157, 3031-3034` — HASH_SCHEMA_VERSION (16→17), HashInto for EffectAmount disc 18, HashInto for PendingTrigger.lki_power, HashInto for StackObject.lki_power.
- `crates/engine/src/rules/events.rs:207-222` (CreatureDied), `:231-240` (AuraFellOff), `:357-371` (ObjectExiled), `:375-385` (PermanentDestroyed), `:456-470` (ObjectReturnedToHand) — 5 GameEvent variants where `pre_death_power` / `pre_lba_power: Option<i32>` field is added with `#[serde(default)]`.
- `feedback_pb_yield_calibration.md`, `feedback_verify_full_chain.md`, `feedback_oversight_primitive_category_not_cards.md`, `feedback_verify_cr_before_implement.md`.

## Planner checklist

- [x] Step 1: CR research — quoted 603.10a, 122.2, 400.7, 113.7a verbatim with notes on LKI semantics for power/toughness. Conclave Mentor 2020-06-23 ruling and Juri 2020-11-10 ruling both verbatim.
- [x] Step 2: Engine architecture walk — 21 dispatch sites enumerated (mirroring PB-LKI-CC).
- [x] Step 3: Per-card oracle-text verification via MCP `lookup_card` — confirmed Conclave Mentor (POWER, ruling 2020-06-23) and Juri Master (POWER, ruling 2020-11-10). Both rulings explicitly mandate LKI ("power as it last existed on the battlefield").
- [x] Step 4: Toughness variant scope decision — NOT shipped. No in-scope card needs toughness LKI. Discriminant 19 reserved for future seed OOS-LKI-Power-1. Justified: shipping unused variants is dead surface; PB-LKI-CC OOS-LKI-N precedent is to file seeds, not pre-stub variants.
- [x] Step 5: Hash strategy — bump 16→17, sentinel sweep file list (9 files, 10 sentinel sites), Option<i32> uses generic HashInto<Option<T>> at hash.rs:191-201 (tag byte 0/1 + payload).
- [x] Step 6: Test plan — 4-5 mandatory tests: (a) Conclave Mentor life gain == LKI power (boosted), (b) Juri damage == LKI power (boosted), (c) discriminating LKI test (read source power AFTER it's in graveyard — must equal pre-death NOT printed NOT 0), (d) hash determinism + sentinel + variant-discrimination + Option<i32> tag-byte encoding canary, (f) optional GameEvent-redirect path.
- [x] Plan file written: `memory/primitives/pb-plan-LKI-Power.md`.

## Implementation checklist (runner fills in)

- [x] Engine change 1: `EffectAmount::SourcePowerAtLastKnownInformation` variant added (disc 18) — `card_definition.rs:2398+`
- [x] Engine change 2: `EffectContext.lki_power: Option<i32>` field added + 2 constructors + 2 inner_ctx clones + check_condition stub updated — `effects/mod.rs`
- [x] Engine change 3: `PendingTrigger.lki_power: Option<i32>` field added + `blank()` init — `state/stubs.rs`
- [x] Engine change 4: `StackObject.lki_power: Option<i32>` field added + `trigger_default()` init — `state/stack.rs`
- [x] Engine change 5: LKI capture at sba.rs:540 — extended let-bind to include `pre_death_power` via `calculate_characteristics`; `calculate_characteristics(state, id).and_then(|c| c.power).or(obj.characteristics.power)`
- [x] Engine change 6: `GameEvent::CreatureDied.pre_death_power: Option<i32>` field added — `events.rs` with `#[serde(default)]`
- [x] Engine change 7: `pre_lba_power: Option<i32>` added to AuraFellOff/PermanentDestroyed/ObjectExiled/ObjectReturnedToHand — `events.rs` with `#[serde(default)]`
- [x] Engine change 8: ALL emit sites for these 5 GameEvent variants updated (~35 sites across effects/mod.rs, casting.rs, mana.rs, replacement.rs); non-battlefield sources use `None`; `zone_change_events` in replacement.rs got 8th parameter + `#[allow(clippy::too_many_arguments)]`; `zone_move_event` helper uses `pre_lba_power: None` for all arms
- [x] Engine change 9: 6 trigger arms in `check_triggers` updated to propagate `pre_death_power`/`pre_lba_power` → `PendingTrigger.lki_power` — `abilities.rs`
- [x] Engine change 10: `flush_pending_triggers` threads `stack_obj.lki_power = trigger.lki_power` — `abilities.rs`
- [x] Engine change 11: `ctx.lki_power = stack_obj.lki_power;` at resolution time (2 paths) — `resolution.rs`
- [x] Engine change 12: `resolve_amount` arm for disc 18 reads `ctx.lki_power.unwrap_or(0)` — `effects/mod.rs`
- [x] Engine change 13: `resolve_cda_amount` arm for disc 18 returns 0 — `rules/layers.rs`
- [x] Engine change 14: `HashInto for EffectAmount` disc 18 — `state/hash.rs` (fieldless: just `18u8.hash_into(hasher)`)
- [x] Engine change 15: `HashInto for PendingTrigger` and `HashInto for StackObject` add `self.lki_power.hash_into(hasher)` — `state/hash.rs`
- [x] Engine change 16: `HashInto for GameEvent::CreatureDied` add `pre_death_power` to destructure pattern + `pre_death_power.hash_into(hasher)` — `state/hash.rs`. 4 LBA hash arms left unchanged (per STOP-AND-FLAG 5: use `..`).
- [x] Engine change 17: HASH_SCHEMA_VERSION 16→17 + history entry 17 — `state/hash.rs`
- [x] Engine change 18: sentinel-assertion sweep across 9 files (10 sentinel sites) — all updated 16→17: primitive_pb_ts.rs:369, pbn_subtype_filtered_triggers.rs:558, pbd_damaged_player_filter.rs:597, effect_sacrifice_permanents_filter.rs:136, primitive_pb_cc_c_followup.rs:400, primitive_pb_lki_cc.rs:440, primitive_pb_cc_a.rs:101, pbp_power_of_sacrificed_creature.rs:782, pbt_up_to_n_targets.rs:411 and :868
- [x] Card def 1: `conclave_mentor.rs` — TODO replaced with WhenDies trigger using `EffectAmount::SourcePowerAtLastKnownInformation` + `Effect::GainLife`; header comment updated
- [x] Card def 2: `juri_master_of_the_revue.rs` — TODO replaced with WhenDies trigger using `EffectAmount::SourcePowerAtLastKnownInformation` + `Effect::DealDamage` + `TargetRequirement::TargetAny`; header comment updated
- [x] OOS seeds appended to `pb-retriage-CC.md`: OOS-LKI-Power-1 (toughness variant, disc 19 reserved), OOS-LKI-Power-2 (Master Biomancer ETB-replacement), OOS-LKI-Power-3 (LBA hash arm symmetric extension); existing OOS-LKI-Power seed marked CLOSED
- [x] Tests (a-d) in `crates/engine/tests/primitive_pb_lki_power.rs`: (a) test_conclave_mentor_death_trigger_gains_life_from_lki_power, (b) test_juri_master_death_trigger_deals_damage_from_lki_power, (c) test_lki_power_resolves_to_pre_death_value_not_printed_value, (d) test_pb_lki_power_hash_schema_version_and_determinism
- [x] Additional test file fixes: copy_redirect.rs, partner_with.rs, plot.rs, saga_class.rs, planeswalker.rs, backup.rs, commander.rs, eternalize.rs, dungeon_resolution.rs, encore.rs, ninjutsu.rs, hideaway.rs, mana_and_lands.rs, forecast.rs (missing `lki_power: None` in StackObject literals); primitive_pb37.rs (missing `lki_power: None` in EffectContext literal); delayed_triggers.rs (missing `pre_death_power: None` in GameEvent::CreatureDied literal)
- [x] cargo build --workspace clean — replay-viewer + TUI exhaustive matches verified
- [x] cargo test --workspace green — **2749 tests passing** (was 2745, +4 new in primitive_pb_lki_power.rs)
- [x] cargo fmt --check clean — zero diffs
- [x] cargo clippy --all-targets -- -D warnings clean — zero warnings
- [ ] CLAUDE.md updated: Active Plan + test count + HASH version (deferred to coordinator per dispatch brief)
- [ ] Authoring report regenerated: `python3 tools/authoring-report.py` (deferred to coordinator per dispatch brief)

## Reviewer checklist

- [x] CR rules independently verified (603.10a, 113.7a, 122.2, 400.7) via MCP
- [x] Card oracle text re-verified via MCP for both re-authored cards (Conclave Mentor, Juri Master) — match verbatim
- [x] Every dispatch site walked and confirmed correct (21 sites; full chain documented in pb-review-LKI-Power.md plumbing trace table)
- [x] Hash arm + version bump + history entry verified (HASH=17, history entry 17, 11 sentinel files updated to 17u8; no stale 16u8)
- [x] Tests verified (4 present, all cite CR, test (c) is the load-bearing LKI-after-zone-change discriminator, test (d) covers Option tag-byte canary)
- [x] No scope creep (toughness variant deferred, AnyCreatureDies untouched, cost-payment LKI untouched, LBA hash arms unchanged)
- [x] Review file written: `memory/primitives/pb-review-LKI-Power.md`
- [x] Verdict: **PASS-WITH-NITS** (3 LOW findings: E1 — AnyCreatureDies LKI-power gap not assigned a dedicated OOS seed; E2 — hard-coded `pre_lba_power: None` for non-creature SBA paths drops layer-4 animated power; E3 — doc-comment line-number drift on PendingTrigger/StackObject lki_power field docs. 0 HIGH; 0 MEDIUM. Coordinator decision: ship-as-is OR brief fix-phase pass.)

## Fix-phase checklist (2026-05-13)

- [x] E1 RESOLVED: OOS-LKI-Power-4 appended to `pb-retriage-CC.md` documenting AnyCreatureDies + LKI source-power gap (mirror of OOS-LKI-4 counter sibling). Engine code unchanged — `abilities.rs:4391` AnyCreatureDies arm correctly defaults `lki_power: None` per plan Site 9 + Risk #1.
- [x] E2 RESOLVED (deferred): OOS-LKI-Power-5 appended to `pb-retriage-CC.md` documenting non-creature SBA paths (planeswalker exile, Saga sacrifice, Aura fall-off, Food forage) hard-coding `pre_lba_power: None`. Symmetric to PB-LKI-CC's preserved pattern; no in-scope card hits these paths. Engine fix deferred per reviewer recommendation.
- [x] E3 RESOLVED: stale line-number references removed from `state/stack.rs:478-479` doc comment for `lki_power`. Now reads "abilities.rs `flush_pending_triggers`" / "resolution.rs" — function-name-only references.

## Fix-phase gate results (2026-05-13)

- `cargo build --workspace`: PASS (clean)
- `cargo fmt --check`: PASS — zero diffs
- All 3 LOW findings resolved (1 inline comment edit, 2 OOS seeds appended; engine semantics unchanged)

## Verdict (post-fix-phase)

**PASS** — 0 HIGH / 0 MEDIUM / 0 LOW open. All review findings resolved. Branch ready for signal-ready.

---

# Primitive WIP: PB-LS6 — LOW sweep: loyalty target validation, DestroyAndReanimate, PreventNextUntap / skip_untap_steps

batch: PB-LS6
title: 3 PB-T LOW issues — L01 loyalty target validation, L02 Effect::DestroyAndReanimate (Sorin -6), L03 Effect::PreventNextUntap + GameObject.skip_untap_steps (Tamiyo -2 / Hands of Binding)
cards_unblocked: Sorin Lord of Innistrad -6 (DestroyAndReanimate), Tamiyo Field Researcher -2 (PreventNextUntap), Hands of Binding (PreventNextUntap)
started: 2026-05-15
phase: implement-complete
plan_file: memory/primitives/pb-plan-LS6.md
hash_version_pre: 25 (MR-B12-04 prior LS-5 batch)
hash_version_post: 26 (PB-LS6 — Effect::DestroyAndReanimate disc 85, Effect::PreventNextUntap disc 86, GameObject.skip_untap_steps field)

## Task reference
- ESM task: scutemob-36
- Branch: feat/ls-6-low-sweep-pb-t-loyalty-dsl-unblocks-pb-scale

## Implementation checklist

- [x] L01: handle_activate_loyalty_ability (rules/engine.rs) — thread TargetRequirement validation via validate_targets_with_source before loyalty cost payment (CR 601.2c / CR 606.4)
- [x] L02: Effect::DestroyAndReanimate { targets: Vec<EffectTarget>, cant_be_regenerated: bool } variant — card_definition.rs + effects/mod.rs phase-1 (destroy pipeline) + phase-2 (reanimate non-token cards under controller)
- [x] L03: GameObject.skip_untap_steps: u32 field (#[serde(default)]) — game_object.rs; Effect::PreventNextUntap { target: EffectTarget } variant — card_definition.rs + effects/mod.rs; untap_active_player_permanents (turn_actions.rs) decrement logic
- [x] Hash: discriminant arms 85/86 + skip_untap_steps + HASH_SCHEMA_VERSION 25→26 + history entry 26 — state/hash.rs
- [x] Struct-literal sites: 15 explicit GameObject literals updated across state/mod.rs, state/builder.rs, effects/mod.rs, rules/resolution.rs + 5 in test files (zone_integrity, emblem_tests, delayed_triggers, snapshot_perf, commander_damage)
- [x] Card def 1: sorin_lord_of_innistrad.rs — -6 ability replaced with Effect::DestroyAndReanimate (3 DeclaredTarget indices)
- [x] Card def 2: tamiyo_field_researcher.rs — -2 Sequence extended with Effect::PreventNextUntap x2; CR citation corrected 613.6 → 502.3
- [x] Card def 3: hands_of_binding.rs — spell effect upgraded to Sequence([TapPermanent, PreventNextUntap]); TODO removed
- [x] Tests: loyalty_target_validation.rs (6 tests, L01 + HASH sentinel), destroy_and_reanimate.rs (6 tests, L02), prevent_next_untap.rs (5 tests, L03)
- [x] Sentinel sweep: 19 existing sentinel files bumped 25u8 → 26u8
- [x] cargo build --workspace: PASS (clean; tools/replay-viewer + tools/tui exhaustive matches verified — no new StackObjectKind/KeywordAbility variants added)
- [x] cargo test --all: PASS — **2855 tests** (was 2819, +36 net: 17 new tests across 3 new files)
- [x] cargo clippy --all-targets -- -D warnings: PASS — zero warnings
- [x] cargo fmt --all: PASS — zero diffs

## Gate results (2026-05-15)

Committed in 3 logical chunks on branch feat/ls-6-low-sweep-pb-t-loyalty-dsl-unblocks-pb-scale:
1. a2c82c68 — engine changes (9 files, +417/-4 lines)
2. 5a8b404e — card def fixes (3 files, +28/-20 lines)
3. 97423497 — tests (26 files, +1211/-38 lines; 3 new test files)

## Verdict

**IMPLEMENT-COMPLETE** — 3 issues implemented, 3 card defs fixed, 17 new tests. 2855 tests passing.
No review phase scheduled (LOW issues — single-pass implementation).
