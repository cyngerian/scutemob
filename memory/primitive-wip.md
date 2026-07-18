# Primitive WIP: PB-EF11 — low-yield singletons (WheelDraw greatest-discarded + spell-only single-target requirement)

batch: EF11
title: Two independent singletons bundled to amortize PB overhead; ship as two cleanly-separated commits. (1) EF-W-MISS-8: Windfall — WheelDraw::GreatestDiscarded variant: each player discards their hand, then draws cards equal to the GREATEST number any player discarded this way (CR 121 draw rules). Extend Effect::WheelHand/WheelDraw (PB-AC9). Current executor (effects/mod.rs:675) is per-player disposal+draw; GreatestDiscarded needs two-pass: dispose all affected players first (recording each discard count = pre-disposal hand size), compute max, then all affected players draw max. (2) EF-W-MISS-9: Misdirection — oracle "Change the target of target spell with a single target." Needs spell-ONLY single-target TargetRequirement (TargetSpellOrAbilityWithSingleTarget over-permissive — legalizes abilities). Reuse Effect::ChangeTargets { must_change: true } (Bolt Bend). Alt cost "exile a blue card rather than pay mana cost" IS expressible (Force of Will Pitch pattern: Cost::ExileFromHand { color: Color::Blue }). Wire: WheelDraw variant + TargetRequirement variant -> PROTOCOL 15->16 (+HASH 53->54), machine-forced, justify.
task: scutemob-112
branch: feat/pb-ef11-low-yield-singletons-wheeldraw-greatest-discarded-sp
started: 2026-07-18
phase: done  # 0H/0M/2L (pb-review-EF11.md); LOW-1 fixed (0a057cc0), LOW-2 pre-existing out-of-scope. Gates green, 3466 tests. PROTOCOL 17, HASH 55.

## Recon (worker, pre-plan)
- WheelDraw enum: crates/card-types/src/cards/card_definition.rs:2462 (ThatMany, Fixed(u32)).
- Effect::WheelHand executor: crates/engine/src/effects/mod.rs:675 — per-player loop, disposal then draw. hand_size snapshotted before disposal. Discard via discard_cards, draw via draw_one_card.
- WheelDraw hash: crates/engine/src/state/hash.rs:6720.
- TargetRequirement enum: card_definition.rs:2841; TargetSpellOrAbilityWithSingleTarget at 2879.
- Single-target validation: crates/engine/src/rules/casting.rs:6166-6193 (zone==Stack, self-target prevention, stack_obj.targets.len() == 1).
- Spell-vs-ability: target stack_obj.kind must be StackObjectKind::Spell (stack.rs:579). MutatingCreatureSpell also a spell — planner confirm whether to include.
- Effect::ChangeTargets — Bolt Bend (bolt_bend.rs) uses it with must_change:true.
- Force of Will pitch: force_of_will.rs — Cost::ExileFromHand { color: Color::Blue }.
- PROTOCOL_VERSION=15 (rules/protocol.rs:152); HASH_SCHEMA_VERSION=53 (state/hash.rs:482).
- misdirection.rs does NOT exist (never authored). windfall.rs — check.

## Progress
- [x] plan — `memory/primitives/pb-plan-EF11.md`
- [x] implement F1 (WheelDraw::GreatestDiscarded + windfall) — COMMIT 1 done. Added
  `WheelDraw::GreatestDiscarded` (card_definition.rs); two-pass executor restructure in
  `effects/mod.rs` (`Effect::WheelHand` — dispose all affected players first recording
  pre-disposal hand sizes, compute max, then all draw the max; `ThatMany`/`Fixed` byte-identical
  via an outer `match draw` wrapper, `unreachable!()` guard on the inner match's third arm);
  hash arm in `state/hash.rs` (discriminant 2); `windfall.rs` authored Complete. Tests:
  `crates/engine/tests/primitives/pb_ef11_wheel_greatest_discarded.rs` (6 tests: all-draw-max,
  non-vacuous decoy, empty-hands, hash-discriminant, windfall card-def integration, version
  sentinel), registered in `primitives/main.rs`.
- [x] implement F2 (spell-only single-target + misdirection) — COMMIT 2 done.
  `TargetRequirement::TargetSpellWithSingleTarget` added (card_definition.rs); validation
  early-return block in `validate_object_satisfies_requirement` (casting.rs) mirrors the
  `TargetSpellOrAbilityWithSingleTarget` block plus an `is_spell` kind check
  (`StackObjectKind::Spell | MutatingCreatureSpell`); 3 exhaustive-match arms added
  (hash.rs discriminant 19, casting.rs `valid` match, abilities.rs battlefield auto-target
  match) — `cargo build --workspace` confirmed no further sites missed. `misdirection.rs`
  authored Complete (AltCastAbility Pitch, no life component + Spell/ChangeTargets). Tests:
  internal precision test in casting.rs's own `#[cfg(test)] mod tests` (self-targeting +
  kind-check, direct private-fn call, mirrors the sibling variant's test) plus external
  `crates/engine/tests/primitives/pb_ef11_spell_single_target.rs` (6 tests: accepts, two
  DECOYs — count check and kind check, both verified non-vacuous by reverting each guard —
  self-prevention via live CastSpell, hash-discriminant, Misdirection retarget integration),
  registered in `primitives/main.rs`.
- [x] wire bumps (COMMIT 1) — HASH_SCHEMA_VERSION 53->54, PROTOCOL_VERSION 15->16, both
  fingerprints recomputed from the failing schema tests (never hand-authored), history rows +
  FROZEN_HISTORY_PREFIX_DIGEST re-pinned in both `state/hash.rs`/`tests/core/hash_schema.rs` and
  `rules/protocol.rs`/`tests/core/protocol_schema.rs`. All ~30 scattered
  `assert_eq!(HASH_SCHEMA_VERSION, 53u8)` / `assert_eq!(PROTOCOL_VERSION, 15)` sentinels bumped
  across the test suite (grep-verified zero stragglers).
- [x] wire bumps (COMMIT 2) — HASH_SCHEMA_VERSION 54->55, PROTOCOL_VERSION 16->17, likewise
  recomputed from the failing schema tests, history rows + FROZEN_HISTORY_PREFIX_DIGEST
  re-pinned. All ~35 scattered `HASH_SCHEMA_VERSION, 54u8` / `PROTOCOL_VERSION, 16` sentinels
  bumped across the test suite (grep-verified zero stragglers).
- [ ] review (COMMIT 2 implementation complete; not yet reviewed)
- [x] gates + docs (COMMIT 1) — `cargo build --workspace`, `cargo test --all` (all green, 0
  failed), `cargo clippy --all-targets -- -D warnings` (clean), `cargo fmt --check` +
  `tools/check-defs-fmt.sh` (clean). **Deviation from plan**: found and fixed a pre-existing
  environment-wide compile break, unrelated to this batch, blocking `cargo build --workspace`
  and `cargo test --all` on `main` itself before any PB-EF11 edits — several `.filter()`/`.find()`
  closures over `imbl::OrdMap`/`Vector` iterators called `.get(id)`/`.contains_key(id)` with a
  double-referenced key (`&&ObjectId`/`&&PlayerId`, since `Iterator::filter`/`find` take
  `&Self::Item`), which a stricter `imbl`/`equivalent` trait-bound resolution under the pinned
  Rust 1.95.0 toolchain now rejects (confirmed identical failure on a clean `main` checkout).
  Fixed minimally by dereferencing once (`.get(*id)`/`.contains_key(*id)`) at 9 call sites across
  `effects/mod.rs`, `state/mod.rs`, and 6 test files (`conditional_statics.rs`,
  `static_grants.rs`, `count_based_scaling.rs`, `graveyard_targeting.rs` x2, `offspring.rs` x2,
  `squad.rs`). Out of scope for PB-EF11 but required to get any gate green at all.
- [x] gates + docs (COMMIT 2) — `cargo build --workspace`, `cargo test --all` (all green, 0
  failed, 44 files changed), `cargo clippy --all-targets -- -D warnings` (clean), `cargo fmt
  --check` + `tools/check-defs-fmt.sh` (clean; `tools/check-defs-fmt.sh --fix` reformatted
  `misdirection.rs`'s wrapped oracle_text string, then `cargo fmt` reformatted a let-else in
  the new test file — both re-verified green after). Both PB-EF11 commits are shipped; this
  closes the batch pending review.
