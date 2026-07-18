# Primitive WIP: PB-EF10 — sacrifice-driven EffectAmount / runtime max_cmc / if-you-do Condition (EF-W-MISS-7)

batch: EF10
title: Three independent sub-gaps from EF-W-MISS-7 (could be micro-PBs; ship as three cleanly-separated commits inside this one PB). (1) `EffectAmount::ToughnessOfSacrificedCreature` — mirror the existing `PowerOfSacrificedCreature` LKI machinery; read layer-resolved toughness at the sacrifice moment incl. continuous effects (anthem). (2) runtime-computed `max_cmc` on `SearchLibrary` = N + sacrificed creature's mana value (Eldritch Evolution N=2, Birthing Ritual N=1). (3) a `Condition` reporting whether a resolution-time `SacrificePermanents` actually fired ("if you do"; CR 717 interlocked / do-only-if pattern — cite exact CR via MCP). Candidates (4, discounted ~3): Momentous Fall (sub-gap 1), Eldritch Evolution (sub-gap 2, cost-sac), Birthing Ritual (sub-gaps 2+3, resolution-sac + dig — likely the hardest; may stay partial), Victimize (sub-gap 3, resolution-sac). Sacrificed-creature values MUST read LKI (CR 608.2b/i last-known-info — creature is in graveyard when amount computed). EF-EF1-A caveat: PowerOfSacrificedCreature is NOT populated in the MayPayThenEffect optional-cost path; resolution-time SacrificePermanents currently does NOT capture LKI into ctx — this PB must populate it for the resolution-sac candidates without regressing EF-EF1-A. Wire: EffectAmount variant + SearchLibrary/Effect shape + Condition variant + StackObject capture shape → PROTOCOL 14→15 (+HASH), machine-forced, justify.
task: scutemob-111
branch: feat/pb-ef10-sacrifice-driven-effectamount-runtime-maxcmc-if-you-
started: 2026-07-18
phase: fix-complete  # review: memory/primitives/pb-review-EF10.md (0H/0M/2L); both LOW fixed commit bcf9eb8a (redundant expect_characteristics dropped; MoveZone-tapped regression test added). 3453 tests. Ready for collection. plan: memory/primitives/pb-plan-EF10.md (struct SacrificedCreatureLki; max_cmc_amount on TargetFilter avoids 99-def edit; CR 608.2c/608.2h not 717; Momentous Fall/Eldritch Evolution/Victimize→Complete, Birthing Ritual→inert + OOS-EF10-1; PROTOCOL 14→15, HASH 52→53)

## Progress

- [x] COMMIT 1 — data-model migration (`SacrificedCreatureLki` struct replacing `Vec<i32>`
      powers on `EffectContext`/`StackObject`/`AdditionalCost::Sacrifice`; `EffectContext.sacrifice_fired: bool`
      added) + sub-gap 1 `EffectAmount::ToughnessOfSacrificedCreature` (hash disc 22) + full-LKI
      capture at the three capture sites (abilities.rs activated-cost, casting.rs spell-additional-cost,
      resolution.rs x2 copy-into-ctx) + hash arms (`impl HashInto for SacrificedCreatureLki`,
      `AdditionalCost::Sacrifice`, `StackObject.sacrificed_creature_lki`). ~75 mechanical
      `lki_powers`/`sacrificed_creature_powers` literal sites renamed across engine+tests
      (word-boundary sed, verified zero stragglers). TODO sweep run (see plan file, recorded
      in commit body) — surfaced 2 unlisted forced-adds (miren_the_moaning_well.rs,
      diamond_valley.rs, both flipped Complete) + 1 note-precision fix (birthing_pod.rs: needs
      an EXACT-mana-value runtime filter, not the "or less" max_cmc_amount cap — stays blocked,
      distinct from OOS-EF10-1). momentous_fall.rs authored new, Complete. 4 unit tests in
      `pb_ef10_sacrifice_driven_amounts.rs` (incl. 1 decoy, proven non-vacuous by
      revert-and-rerun). `cargo build --workspace` clean.
- [x] COMMIT 2 — sub-gap 2: `EffectAmount::ManaValueOfSacrificedCreature` (hash disc 23)
      + `TargetFilter.max_cmc_amount: Option<Box<EffectAmount>>` (Box required — direct
      `Option<EffectAmount>` created a recursive-type cycle via `EffectAmount::PermanentCount{filter:
      TargetFilter}`; serde-default so none of the 99 existing `SearchLibrary` def files changed)
      honored by the `SearchLibrary` executor (runtime cap resolved once per effect, ANDed with
      the pre-existing static `max_cmc`). eldritch_evolution.rs authored new, Complete — also adds
      an explicit `Effect::Shuffle` after the search (Harrow precedent) so "then shuffle" is fully
      modeled, not just noted as a pre-existing gap. 3 more unit tests (incl. 1 decoy pinning both
      Sum operands, proven non-vacuous by dropping the `+2` term and rerunning — 3 of 7 tests failed
      as expected, then reverted). `cargo build --workspace` clean; 7/7 pb_ef10 tests green.
- [x] COMMIT 3 — sub-gap 3: `sacrifice_permanents_for_player` returns
      `Vec<SacrificedCreatureLki>` of everything actually sacrificed (pushed only on the
      Redirect/Proceed success arms, not ChoiceRequired); `Effect::SacrificePermanents` sets
      `ctx.sacrifice_fired`/`ctx.sacrificed_creature_lki` from the collected returns (overwrite,
      not append — reflects THIS instruction, per-resolution scoping documented). New
      `Condition::SacrificeFired` (hash disc 48) + `check_condition` arm. Step 3.4 (EF-EF1-A
      optional-path capture) SKIPPED per the plan's explicit permission — threading a
      `Vec<SacrificedCreatureLki>` return through `pay_optional_cost`/`try_pay_optional_cost`
      for every `Cost` variant was judged out of proportion to the risk; `disciple_of_freyalise`
      stays partial, EF-EF1-A stays filed. victimize.rs authored new, Complete; birthing_ritual.rs
      authored new, `inert` (registers no behavior at all — the registry gate demands `inert`
      over `partial` for zero-behavior defs, a taxonomy detail the plan didn't flag but
      `card_registry_gate` enforces). **Bonus fix, found while testing Victimize (not a declared
      sub-gap)**: `Effect::MoveZone` never applied `ZoneTarget::Battlefield { tapped }` —
      `dest_tapped()` existed and was wired into the sibling `SearchLibrary` matched-cards path
      but never called from `MoveZone`, so any "return ~ tapped" effect silently entered
      untapped. Fixed by mirroring the SearchLibrary pattern.
      **Version bump**: PROTOCOL 14→15, HASH 52→53 — both fingerprints/history rows obtained
      from the failing gate output (`tests/core/protocol_schema.rs`,
      `tests/core/hash_schema.rs`), never hand-guessed. All scattered
      `assert_eq!(PROTOCOL_VERSION, 14)` / `assert_eq!(HASH_SCHEMA_VERSION, 52u8)` sentinels
      bumped (grep-verified zero stragglers).
      8 more unit tests appended (2 decoys: `test_sacrifice_fired_false_when_none_available`
      doubles as the decoy for "sacrifice_fired latched unconditionally"; 3 Victimize
      integration tests). 15/15 pb_ef10 tests green.

## Post-implementation verification (all commits complete)

- Full suite: `cargo test --all` — **3452 tests passing**, 0 failed (workspace-wide, all binaries).
- `cargo build --workspace` clean (seal gate).
- `cargo clippy --all-targets -- -D warnings` clean (one lazy-doc-continuation lint fixed:
  a doc comment line starting with `>= 1 permanent` was misread by rustdoc as a blockquote
  marker; reworded to "at least one permanent").
- `cargo fmt --all -- --check` clean.
- `tools/check-defs-fmt.sh` clean (1796 defs checked) — required a `--fix` pass after authoring
  the 4 new card defs (rustfmt's `format_strings=true` reflow, SR-35).
- `card_registry_gate::test_no_behavior_defs_are_inert_not_partial_or_known_wrong` caught
  `birthing_ritual.rs` marked `partial` with an empty `abilities` vec — fixed to `inert`.
- `python3 tools/authoring-report.py`: 1,796 files, clean 1,098 (61.1%), todo 544, empty 154.
  Delta from the PB-EF9 baseline (1093/1792, 61.0%): **+5 clean** (momentous_fall,
  eldritch_evolution, victimize new + miren_the_moaning_well/diamond_valley flipped), **+4 files**
  (the 3 new Complete defs + birthing_ritual.rs, which lands in the "empty" bucket as `inert`).
- OOS-EF10-1 filed: `memory/card-authoring/w-miss-engine-findings-2026-07-17.md` (EF-W-MISS-7
  section, now closed with a full write-up) and `memory/primitives/ef-batch-plan-2026-07-17.md`
  §12 (queue summary row updated to DONE).
- Skipped-optional: Step 3.4 (EF-EF1-A closure) — see COMMIT 3 note above.
- Deferred, not this PB's scope: `birthing_pod.rs`'s EXACT-mana-value gap (distinct from
  OOS-EF10-1; noted in its file comment and in §12).

Ready for review phase.

## Oracle text (verified via MCP 2026-07-18)
- **Momentous Fall** {2}{G}{G} Instant — "As an additional cost to cast this spell, sacrifice a creature. You draw cards equal to the sacrificed creature's power, then you gain life equal to its toughness." Ruling: last known existence checked for power AND toughness. → sub-gap 1. PowerOfSacrificedCreature already exists (see greater_good.rs, lifes_legacy.rs); this needs the toughness twin. Cost-sacrifice path (already captures powers).
- **Eldritch Evolution** {1}{G}{G} Sorcery — "As an additional cost to cast this spell, sacrifice a creature. Search your library for a creature card with mana value X or less, where X is 2 plus the sacrificed creature's mana value. Put that card onto the battlefield, then shuffle. Exile Eldritch Evolution." → sub-gap 2, N=2, cost-sacrifice. Also needs self-exile on resolve (verify Effect exists). Ruling: sacrificed creature's MV is last-known.
- **Birthing Ritual** {1}{G} Enchantment — end-step trigger, "if you control a creature, look at top seven ... Then you may sacrifice a creature. If you do, you may put a creature card with mana value X or less from among those cards onto the battlefield, where X is 1 plus the sacrificed creature's mana value. Put the rest on the bottom in a random order." → sub-gaps 2+3, N=1, RESOLUTION-sacrifice, plus a look-at-top-7 / put-one / rest-to-bottom-random dig. HARDEST — chain-verify whether the dig is expressible; if not, author truthfully partial with the real remaining blocker.
- **Victimize** {2}{B} Sorcery — "Choose two target creature cards in your graveyard. Sacrifice a creature. If you do, return the chosen cards to the battlefield tapped." → sub-gap 3, RESOLUTION-sacrifice, then conditional return. Ruling: sac isn't chosen until resolution; if both targets illegal, spell doesn't resolve → no sac; if one illegal, still sac + return the other.

## Known engine anchors (from coordinator recon)
- `EffectContext.sacrificed_creature_powers: Vec<i32>` (effects/mod.rs:134) — LKI powers, captured BEFORE move_object_to_zone. Read by `EffectAmount::PowerOfSacrificedCreature` (effects/mod.rs:7263).
- `StackObject.sacrificed_creature_powers: Vec<i32>` (card-types/state/stack.rs:464) — copied into ctx at resolution (resolution.rs:1868).
- Cost-sac LKI capture: abilities.rs:1277 (`stack_obj.sacrificed_creature_powers = sacrificed_lki_powers`) and the ActivationCost path resolution.rs:392-403.
- `EffectAmount` enum: card-types/cards/card_definition.rs:2572. Hash disc for PowerOfSacrificedCreature = 15 (hash.rs:5325).
- `TargetFilter.max_cmc: Option<u32>` (card_definition.rs:2942) — STATIC. Runtime max_cmc likely needs a new field on `Effect::SearchLibrary` (card_definition.rs:1672) resolved from an `EffectAmount` at execution, applied as an additional cap.
- `Effect::SacrificePermanents` def: card_definition.rs:1790.
- `Condition` enum: card_definition.rs:3509.
- Wire fingerprint / versions: rules/protocol.rs (PROTOCOL_VERSION), state/hash.rs (HASH_SCHEMA_VERSION, currently 52 / PROTOCOL 14 after EF9).

## Architectural question for the planner to resolve
The single `Vec<i32>` powers now must also carry toughness (sub-gap 1) and mana value (sub-gap 2). Decide: three parallel vecs vs. one `Vec<SacrificedCreatureLki { power, toughness, mana_value }>`. A struct is cleaner and mirrors once; parallel vecs are a smaller diff. Pick one and justify. Resolution-time `SacrificePermanents` must populate the SAME ctx field(s) + a "sacrifice fired" flag for sub-gap 3, threaded through the effect sequence, WITHOUT regressing EF-EF1-A (the MayPayThenEffect cost path).
