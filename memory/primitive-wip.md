# Primitive WIP: PB-EF2 — CreateToken player-scoped recipient (EF-W-MISS-1)

batch: PB-EF2
title: `TokenSpec.recipient: PlayerTarget` (default Controller) + PlayerTarget::ControllerOfCounteredSpell / ControllerOfTriggeringObject — fix swan_song "its controller creates"
task: scutemob-102
branch: feat/pb-ef2-createtoken-player-scoped-recipient-fix-swansong-toke
started: 2026-07-18
phase: done
plan_file: memory/primitives/pb-plan-EF2.md

## Design decision (read before coding)
Recipient lives on **TokenSpec** (the `Effect::CreateToken` payload), NOT as a sibling field on
the Effect variant — this keeps all 201 existing construction sites literally unchanged
(`..Default::default()`/helpers), which is AC 4861's binding "all existing users unchanged"
constraint. Semantically justified: TokenSpec is "everything needed to create a token." Full
rationale + deviation note in the plan.

## Steps (unchecked)
- [x] Step 1 — PlayerTarget::ControllerOfCounteredSpell + ControllerOfTriggeringObject variants added to `crates/card-types/src/cards/card_definition.rs`
- [x] Step 2 — TokenSpec.recipient field (last field, `#[serde(default)]`) + `impl Default for PlayerTarget` + updated `impl Default for TokenSpec`; `mtg-card-types` and `mtg-card-defs` compile clean (all 160 sites use `..Default::default()`/helpers)
- [x] Step 3 — EffectContext.countered_spell_controller added (effects/mod.rs); set in Effect::CounterSpell arm right after `pos` resolves, before `cant_be_countered` continue; initialized `None` in `new`/`new_with_kicker` + all 5 raw struct literals (2x ForEach inner_ctx, check_condition delegate ctx, abilities.rs activation-condition ctx) — `cargo check -p mtg-engine` found and confirmed all sites
- [x] Step 4 — resolve_player_target_list arms added for ControllerOfCounteredSpell/ControllerOfTriggeringObject; also added matching arms to the two single-player match sites (Manifest, Cloak) that `cargo check` surfaced as non-exhaustive (not called out by name in the plan but required for compilation) + PlayerTarget hash match in state/hash.rs
- [x] Step 5 — CreateToken executor rewritten to loop over `resolve_player_target_list(state, &spec.recipient, ctx)`; `apply_token_creation_replacement` now keyed per-recipient; CreateTokenAndAttachSource left on ctx.controller with an explanatory comment. `cargo check --workspace` clean (no replay-viewer/TUI match arms needed — no new StackObjectKind/KeywordAbility variants added).
- [x] Step 6 — hash TokenSpec.recipient + PlayerTarget variants (discriminants 8/9); PROTOCOL 6→7, fingerprint c52ed4e2…, FROZEN_HISTORY_PREFIX_DIGEST re-pinned; HASH 44→45, decl_fingerprint 5da8e891…, stream_fingerprint ae1f49d7…, FROZEN prefix re-pinned; 30 sentinel files bulk sed'd 44→45; also raised `bare_lookup_ratchet` ceiling for effects/mod.rs 100→105 (5 new NONSWALLOW predicate reads — surfaced by `cargo test --test core`, not called out in the plan but required)
- [x] Step 7 — swan_song → Complete: `recipient: PlayerTarget::ControllerOfCounteredSpell,` added, `completeness: Completeness::known_wrong(...)` line deleted
- [x] Step 8 — authored `crates/card-defs/src/defs/an_offer_you_cant_refuse.rs` (Complete), filename confirmed against `slugify()` in `tools/authoring-report.py` (apostrophe deleted)
- [x] Step 9 — tests: `crates/engine/tests/primitives/pb_ef2_create_token_recipient.rs` (8 tests: hash sentinel, swan_song happy path, swan_song decoy, An Offer happy+decoy+mana-ability, default-recipient-unchanged, ControllerOfTriggeringObject resolves, 2x doubling-keyed-to-recipient); `mod` line added to `tests/primitives/main.rs`; also fixed a missed raw `EffectContext {}` literal in `tests/primitives/primitive_pb37.rs` (only surfaced by `cargo test`, not `cargo check -p mtg-engine`). All 8 pass; manually verified all 7 recipient-sensitive tests FAIL when `recipients` is hardcoded back to `vec![ctx.controller]` (temporary revert + restore), confirming none are vacuous.
- [x] Step 10 — bookkeeping: `memory/primitives/ef-batch-plan-2026-07-17.md` (STATUS UPDATE block,
  EF-W-MISS-1 closed), `memory/card-authoring/w-miss-roster-2026-07-17.md` and
  `w-miss-engine-findings-2026-07-17.md` (EF-W-MISS-1 marked ✅ CLOSED). Un-retired
  `test-data/generated-scripts/tokens/001_swan_song_creates_bird.json` (its assertion was
  already correct). Also found and fixed a SEPARATE pre-existing bug in an already-`approved`
  script, `test-data/generated-scripts/stack/045_swan_song_counters_damnation.json`, which
  asserted the Bird onto `zones.battlefield.p2` (the exact shape of the pre-fix bug) — not
  called out in the plan, surfaced by `cargo test --all`. `python3 tools/authoring-report.py`:
  coverage 60.0% → 60.1% (1,070 → 1,072 clean / 1,782 → 1,783 total; +2 clean).
