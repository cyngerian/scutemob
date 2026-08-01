# Primitive batch WIP — PB-DX5

**Batch**: PB-DX5 — CR 611.2c: lock the affected set of a resolution-generated continuous effect
**Seed**: OOS-OS7-2 (ex-RS6; `memory/primitives/seed-rerank-2026-07-27.md` §2.3 + §4 dispatch brief)
**Task**: `scutemob-170` · **Branch**: `feat/pb-dx5-cr-6112c-lock-the-affected-set-of-a-resolution-genera`
**Phase**: done

## Result summary (2026-08-01)

**SHIPPED.** `ContinuousEffect` gains `affected_set: Option<OrdSet<ObjectId>>` (CR 611.2c),
populated only at `Effect::ApplyContinuousEffect` (`rules::layers::snapshot_affected_set`, called
before the effect is pushed) and read as pure membership by `effect_applies_to`. `None` means a
static ability's effect (CR 611.3a — genuinely not locked in, `filter` stays live).

- **Roster, final measured**: 38 mass-filter defs — 29 `Complete`, 8 `partial`, 1 `known_wrong`.
  Corrects this file's own earlier "37 (28/8/1)" — the table this file built already listed 38
  rows summing to 29, an uncaught arithmetic slip. New test
  `pb_dx5_mass_filter_roster_by_completeness` (`crates/engine/tests/core/pb_dx5_continuous_effect_roster.rs`)
  re-measures this every run rather than pinning an exact count.
- **Engine changes**: `crates/card-types/src/state/continuous_effect.rs` (field),
  `crates/engine/src/state/hash.rs` (hash feed + HASH bump), `crates/engine/src/rules/layers.rs`
  (`snapshot_affected_set` + `candidate_ids_for_filter`, exhaustive no-`_`-arm match; the
  `effect_applies_to` membership read; doc updates to `is_effect_active`),
  `crates/engine/src/effects/mod.rs` (the `ApplyContinuousEffect` creation site + the
  `CreateEmblem` static-effects comment), `crates/engine/src/rules/replacement.rs`
  (`register_static_continuous_effects` comment). Mechanical backfill of `affected_set: None` at
  all 180 pre-existing `ContinuousEffect` construction sites (49 files), zero manual judgement
  calls (every site is either a static registration or a `SingleObject` effect).
- **Fingerprints**: `HASH_SCHEMA_VERSION` 69 → **70** (mandatory, confirmed by the SR-19 gate).
  `PROTOCOL_VERSION` **confirmed unmoved at 32** by running `--test core protocol_schema` — not
  assumed. 42 files / 43 sentinel assertions re-pinned 69→70 by symbol grep; two more (multi-line
  `assert_eq!` shape the grep's single-line pattern couldn't see) caught only by running the full
  workspace suite with `--no-fail-fast`.
- **Tests**: 4,048 → **4,065** (+17, corrected by the fix cycle — Finding 6 — from this file's own
  originally-recorded "+16 → 4,064", an arithmetic slip: `crates/engine/tests/core/
  pb_dx5_continuous_effect_roster.rs` carries TWO `#[test]`s, not one —
  `pb_dx5_resolution_continuous_effect_roster` (written during the premise-verification phase, on
  this branch, i.e. AFTER the 4,048 merge-base measurement) and
  `pb_dx5_mass_filter_roster_by_completeness`. Decomposition: 14 in the new
  `crates/engine/tests/primitives/pb_dx5_affected_set_snapshot.rs`, 1 in-source
  `#[cfg(test)]` unit test in `rules/layers.rs` for T11 — `snapshot_affected_set`/
  `effect_applies_to_object`/`candidate_ids_for_filter` are all `pub(crate)`, unreachable from an
  integration test — and 2 roster tests. 14 + 1 + 2 = 17, and 4,048 + 17 = 4,065, which is the
  figure a full-workspace run actually produces. Every "fails before" claim was OBSERVED
  (read-site membership check reverted, actual value recorded, restored), not reasoned to — this
  caught the runner's own first draft of the T3 control-change test using the buffed creature as
  its own effect source, masking the divergence it claimed to test.
- **Existing-test repair**: `pb_ac3_dynamic_pt_counts.rs`'s
  `test_set_both_dynamic_locked_at_resolution` was asserting the CR 611.2c bug this batch fixes
  and passing; inverted with a CR cite, renamed to
  `test_611_2c_new_creature_after_resolution_does_not_get_the_locked_value`. Golden corpus
  unaffected (`stack/173_spree_final_showdown.json` exercises Final Showdown's mode 2, DestroyAll
  — not the `AllCreatures|Ability` mode 0 the roster found, which is a documented DSL-gap
  omission in the script itself).
- **Yield**: 0 completeness flips, exactly as predicted (`python3 tools/authoring-report.py`:
  1,137/1,804 = 63.0%, byte-identical body, only the regenerated-date header moved).
- **Benchmarks**: `full_turn_4p`/`priority_cycle_4p`/`sba_check`/`board_wipe_4p` all within ~1% of
  the merge base (`d568615b`, throwaway worktree); `board_wipe_4p` (flagged as most likely to
  move) measured slightly faster on the branch.
- **Seeds filed (as originally shipped, before the fix cycle corrected two of them)**:
  `docs/audits/decision-point-audit.md` §8.1, OOS-DX5-1..5 + a checked non-finding OOS-DX5-6
  (Mirror Entity is the one Layer ≤4 mass-filter def; unaffected today since no roster member
  writes `CardType::Creature` via a Layer-4 modification). **Both corrected by the fix cycle —
  see below.**
- **All gates green (as originally shipped)**: `cargo build --workspace`, `cargo clippy
  --workspace --all-targets -- -D warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh`
  (1,804 defs), `cargo test --workspace` (4,064 / 0 — corrected to 4,065 by fix-cycle Finding 6,
  and to **4,066** after the fix cycle's own +1 new test, T15; see below).

## Fix cycle (2026-08-01, same day)

Review `memory/primitives/pb-review-DX5.md`: **0 HIGH, 6 MEDIUM, 6 LOW, all 12 applied.** None
change observable engine behaviour — every one of the twelve is a repair to the RECORD (doc
comments, seed rows, historical docs, test-arithmetic, one weak assertion, one narrower-than-
planned gate). This is stated as a summary claim, not assumed: every finding below is marked
record-only or behaviour-adjacent-but-not-behaviour-changing, and the full test suite (4,066/0),
clippy, fmt, `check-defs-fmt.sh` and both fingerprint gates were all re-run clean after every edit.

1. **Finding 1 (MEDIUM, record-only) — APPLIED.** The batch's own review-cited mechanism (a
   source-relative filter going dead once the spell's card object retires to the graveyard,
   CR 400.7, applying a mass pump to NOBODY once the spell resolved — not merely a newcomer leak)
   was verified EMPIRICALLY before writing anything: reverted the read-site membership block in
   `rules/layers.rs::effect_applies_to`, re-ran T12
   (`pb_dx5_affected_set_snapshot.rs::test_611_2c_snapshot_survives_the_pb_dp9_abort_and_replay`),
   observed BOTH board creatures collapse to `power == Some(1)` (their own base power, no pump on
   either) instead of the intended `Some(3)`, then restored. Recorded in: the module's top-level
   doc comment (new paragraph), T12's own doc comment (new "Observed pre-fix" block with the
   `Some(1)`/`Some(1)` numbers), a new seed **OOS-DX5-7** in `docs/audits/decision-point-audit.md`
   §8.1 (filed CLOSED, since PB-DX5 fixed it as a side effect), and this file. Did NOT change
   `effect_applies_to` or any other production behaviour — the fix was already correct; only the
   record was silent about its own second effect.
2. **Finding 2 (MEDIUM, record-only + new test) — APPLIED.** The implement phase's "verified: none
   exist in the roster" claim for the Layer≤4 divergence was FALSE — checked the wrong population
   (mass-filter defs writing `CardType::Creature`, not ANY effect writing it) and mis-stated the
   mechanism ("later-timestamped" vs. "the live gather sees zero Layer-4 modifications at all").
   Rewrote `snapshot_affected_set`'s doc block in `rules/layers.rs` to state the corrected
   predicate and name `inkmoth_nexus`/Mirror Entity as the reachable counterparty. Added **T15**
   (`test_611_2c_snapshot_uses_full_resolution_a_layer_le4_mass_filter_reaches_a_layer_le4_counterparty`)
   to `pb_dx5_affected_set_snapshot.rs`: animate Inkmoth Nexus, activate Mirror Entity's
   `AddAllCreatureTypes`, assert Nexus receives every creature type. **Empirically confirmed both
   ways**: passes post-fix; reverting the membership block makes it fail (Nexus's `subtypes` does
   NOT contain `SubType("Human")` — the exact mechanism). Rewrote seed **OOS-DX5-6** in
   `docs/audits/decision-point-audit.md` §8.1 from "not filed — checked non-finding" to an open,
   corrected finding with the real mechanism, the real corpus check, and a pointer at T15.
   Behaviour is UNCHANGED (T15 passes against the already-shipped code) — this closes a
   documentation falsehood about a real, CR-correct behaviour change the batch already shipped.
3. **Finding 3 (MEDIUM, record-only) — APPLIED.** `pb_os7_defending_player_continuous_filter.rs`'s
   module doc declared OOS-OS7-2 ("NOT fixed here") as a live limitation; that seed IS this batch's
   own subject and is closed. Rewrote the paragraph to a CLOSED note citing PB-DX5/`scutemob-170`
   and explaining why none of that file's 11 tests observe the difference (all static boards).
   Added closure banners to the two dated planning docs the review named:
   `memory/primitives/oos-retriage-plan-2026-07-18.md` (a blockquote under the original OOS-OS7-2
   filing) and `memory/primitives/rider-seed-triage-2026-07-19.md` (the §1a summary row, inline —
   the §3 R6 rank-table row already said "RE-RANKED UP → PB-DX5" and needed no change).
4. **Finding 4 (MEDIUM, record-only) — APPLIED.** OOS-DX5-1's "13 keyword-trigger grants, all
   verified `SingleObject`" was false for the 13th site — `StackObjectKind::ClassLevelAbility`
   forwards an arbitrary card-def `Static` filter, correct because it is a STATIC registration
   (CR 611.3a), not because it happens to be `SingleObject`. Corrected the phrasing in both
   `snapshot_affected_set`'s doc block and `docs/audits/decision-point-audit.md`'s OOS-DX5-1 row
   (folded into the Finding-5 rewrite below, since both touch the same row). Cited by symbol
   (`StackObjectKind::ClassLevelAbility`), not line.
5. **Finding 5 (MEDIUM, record-only, in-source notes added) — APPLIED.** Widened OOS-DX5-1 to name
   the three READ sites that ignore `affected_set` entirely: `rules/copy.rs::copy_effect_applies_to`,
   `rules/layers.rs::recompute_object_controller`, `rules/layers.rs::expire_while_you_control_source_effects`.
   Added an explanatory in-source `NOTE` at each of the three (measured-zero-exposure evidence
   inline, not just in the seed doc) plus updated the pre-existing PB-EF9 note at
   `expire_while_you_control_source_effects` to point at `affected_set` as the ready-made answer
   (this is also Finding 12 — same edit closes both). No behaviour change: all three sites are
   measured zero-exposure today (`EffectLayer::Copy` and `LayerModification::SetController` both
   have zero corpus occurrences outside a TODO comment).
6. **Finding 6 (MEDIUM, record-only) — APPLIED.** Corrected "+16 → 4,064" to "+17 → 4,065" (this
   file, and `memory/primitives/seed-rerank-2026-07-27.md`'s R6 row / dispatch-brief entry) —
   `pb_dx5_continuous_effect_roster.rs` carries TWO `#[test]`s, not one. See the corrected "Tests"
   bullet above. The fix cycle's own +1 (T15) makes the FINAL count **4,066**, confirmed by a full
   `cargo test --workspace` run, not computed.
7. **Finding 7 (LOW, record-only) — APPLIED.** Split the captured doc block between
   `effect_applies_to_object` (now: "delegates to `effect_applies_to`, see its doc comment") and
   `effect_applies_to` (now carries the full CR 611.2c/611.3a contract, previously duplicated).
   No behaviour change.
8. **Finding 8 (LOW, hardened assertion) — APPLIED.** Replaced the vacuous
   `debug_assert!(eff.affected_set.is_some())` (immediately after the assignment that guarantees
   it) with an assertion on the value actually PUSHED
   (`state.continuous_effects.back().and_then(|e| e.affected_set.as_ref()).is_some()`), which would
   catch a future refactor that rebuilds/clears `eff` between snapshot and push. Debug-only, no
   release behaviour change.
9. **Finding 9 (LOW, gate strengthened) — APPLIED (enriched the board, not just recorded the
   deviation).** T11's `multi_zone_board()` gained a phased-out battlefield creature (so the CR
   702.26e guard is part of the shortcut-vs-brute-force agreement being checked, not just zone
   scoping), an Equipment genuinely attached to a creature (a real, non-trivial `AttachedCreature`
   match, alongside the original empty-attachment edge case), and a `AllCreaturesWithSubtype`
   filter (the `chars`-reading predicate shape). Caught and fixed a SIDE EFFECT of this change: the
   new post-build mutation (`state.objects.get_mut`) tripped the SR-25 `bare_lookup_ratchet` gate
   (54 → 56 bare lookups in `layers.rs`); switched to `state.expect_object_mut` instead, which does
   not count against the ratchet and is the correct diagnostics-vocabulary choice for a lookup that
   must succeed. Test re-verified green after both changes.
10. **Finding 10 (LOW, record-only) — APPLIED.** `pb_dx5_continuous_effect_roster.rs`'s doc comment
    now dates its "38/29/8/1" figures explicitly ("as measured 2026-08-01") and says plainly to
    re-read the `eprintln!` output for the current numbers, not the comment.
11. **Finding 11 (LOW, gate added) — APPLIED.** Plan §3 Q4's "almost certainly no corpus member
    [uses a non-fixed-window duration]" was a spot check, not a measurement. Added
    `collect_filter_and_duration` to `pb_dx5_continuous_effect_roster.rs` and a new block in
    `pb_dx5_mass_filter_roster_by_completeness` that enumerates every (filter, duration) pair for
    the mass-filter roster, prints it, and ASSERTS the non-fixed-window bucket (`Indefinite`,
    `WhileSourceOnBattlefield`, `WhileYouControlSource`) is empty — a standing, re-measured check
    rather than a one-time sentence. **Measured**: 10 filter shapes use `UntilEndOfTurn`, 1
    (`CreaturesYouControl`) uses `UntilYourNextTurn`; zero non-fixed-window members. The spot check
    was right. Added a closure note to `memory/primitives/pb-plan-DX5.md` §3 Q4.
12. **Finding 12 (LOW, record-only) — APPLIED.** Folded into Finding 5's edit (same site,
    `expire_while_you_control_source_effects`'s PB-EF9 note): now points at `affected_set` as the
    ready-made answer for the resolution-generated case, with the static-effect residual named
    explicitly.

**Empirical standard held throughout**: both new "fails before" claims in this fix cycle
(Finding 1's T12 numbers, Finding 2's T15) were produced by actually reverting the read-site
membership block, running the test, reading the real `left` value, and restoring — never reasoned
to. Neither weakens an existing assertion; T15 is a wholly new probe and T12's assertions and
expected values are unchanged, only its doc comment gained the observed pre-fix numbers.

**Fingerprints re-confirmed, not assumed, after the fix cycle**: `cargo test -p mtg-engine --test
core hash_schema::` (19/19 green, including `hash_schema_version_sentinel`) and `cargo test -p
mtg-engine --test core protocol_schema::` (17/17 green, including
`protocol_schema_fingerprint_is_pinned`) both re-run clean — HASH stays **70**, PROTOCOL stays
**32**. No field, type, `Command` or `GameEvent` variant was touched by the fix cycle.

`git diff --stat -- tools/play-server` is empty throughout (non-negotiable #1 held).

## Docs updated

- `docs/audits/decision-point-audit.md` §8.1 — OOS-DX5-1..6 appended (implement phase); OOS-DX5-1
  widened (Findings 4+5), OOS-DX5-6 corrected from a checked non-finding to an open corrected
  finding (Finding 2), OOS-DX5-7 added (Finding 1) by the fix cycle.
- `memory/primitives/seed-rerank-2026-07-27.md` — §2.3 table row, §4 dispatch table row, and the
  full §4 dispatch-brief entry all marked SHIPPED with the corrected 38/29/8/1 split (implement
  phase); test-count arithmetic corrected 4,064→4,065 (Finding 6, fix cycle).
- `memory/primitives/oos-retriage-plan-2026-07-18.md` — closure banner added under the original
  OOS-OS7-2 filing (Finding 3, fix cycle).
- `memory/primitives/rider-seed-triage-2026-07-19.md` — §1a OOS-OS7-2 row marked SHIPPED
  (Finding 3, fix cycle).
- `memory/primitives/pb-plan-DX5.md` — §3 Q4 closure note added with the measured answer
  (Finding 11, fix cycle).
- `memory/primitive-wip.md` (this file) — phase → done, result summary above, Fix cycle section
  added.
- `CLAUDE.md` Current State + Last Updated — updated by the calling agent/coordinator at collect
  time per house convention (not edited by this worker session directly, per instructions not to
  touch files outside the engine/tests/docs/memory scope this task owns; see final report).

## Premise re-verification (done first, on this branch, before planning)

All four premise claims in the dispatch brief hold, and the roster does not.

| claim | verified | where |
|---|---|---|
| `ContinuousEffect` has a `filter` and no affected-object set | **holds** | `crates/card-types/src/state/continuous_effect.rs:531-562` — 10 fields, none of them a set |
| the filter is re-evaluated live on every characteristics calculation | **holds** | `crates/engine/src/rules/layers.rs:591` `effect_applies_to` matches on `effect.filter` against current state; entered from `:582` `effect_applies_to_object` and from `is_effect_active` at `:501` |
| there is exactly one resolution-time creation site | **holds** | `crates/engine/src/effects/mod.rs:3828` `Effect::ApplyContinuousEffect` — the only place a `ContinuousEffect` is built from an `effect_def` and pushed to `state.continuous_effects` |
| CR 611.2c applies to every one of these | **holds, and is total** | every `LayerModification` variant (`continuous_effect.rs:289-465`) either modifies characteristics or is `SetController(PlayerId)`. There is no "rule-modifying" variant that CR 611.2c would leave unlocked, so the rule covers the whole enum with no exception carve-out needed |

### The roster is 4× the brief's, and the brief's own list contains a phantom

The brief's "**9** defs, **7** `Complete`" came from a per-file grep conjunction — file mentions
`ApplyContinuousEffect` **and** file mentions `EffectFilter::All*`. Enumerated instead from
`all_cards()` with a recursive walk that reads each `ApplyContinuousEffect`'s **own**
`effect_def.filter` (`crates/engine/tests/core/pb_dx5_continuous_effect_roster.rs`):

- **116** defs generate a continuous effect at resolution at all.
- **37** of those use a **mass** (multi-object) filter — the class CR 611.2c is visible on.
- Of the 37: **28 `Complete`** (26 of them `Complete` only by the `#[default]` derive), 8
  `partial`, 1 `known_wrong`. Not 7.

Two independent errors in the grep roster:

1. **It missed the entire `CreaturesYouControl*` family** — 27 defs, because the filter name
   does not start with `All`. That family contains the most-played members of the class:
   `craterhoof_behemoth`, `purphoros_god_of_the_forge`, `mirror_entity`,
   `triumph_of_the_hordes`, `unbreakable_formation`, `goblin_bushwhacker`,
   `ezuri_renegade_leader`. Craterhoof is the sharpest instance in the corpus: a creature
   entering after it resolves wrongly gets +X/+X **and trample**.
2. **`elvish_dreadlord` is a phantom.** Its only occurrence of `ApplyContinuousEffect` is inside
   a *blocker-note string* (`elvish_dreadlord.rs:33-37`) describing a rewire that has not been
   done. It generates no continuous effect at all.

That makes this the fourth consecutive suite batch whose published roster was wrong (PB-DP6
3-vs-14, PB-DP8 84-vs-77, PB-DP9 74/16/8-vs-69/16/7, PB-DX4 97-with-two-class-D). The
`#[default] Completeness::Complete` finding from PB-DX3b/DX4 recurs here too: 26 of the 28
live-wrong `Complete` defs never declare a marker.

### Mass-filter roster (37), by filter variant

| filter | n | defs |
|---|---|---|
| `AllCreatures` | 3 | Final Showdown*, Golgari Charm, The Meathook Massacre |
| `AllCreaturesExcludingChosenSubtype` | 1 | Crippling Fear |
| `AllCreaturesExcludingSubtype` | 2 | Eyeblight Massacre, Olivia's Wrath |
| `AllCreaturesWithSubtype` | 2 | Bladewing the Risen, Goblin Lookout |
| `CreaturesOpponentsControl` | 1 | Massacre Wurm* |
| `CreaturesControlledByDefendingPlayer` | 1 | Silumgar, the Drifting Death |
| `CreaturesYouControl` | 22 | Binding the Old Gods, Castle Embereth, Crashing Drawbridge, Craterhoof Behemoth, Elspeth Storm Slayer, Felidar Retreat, Finale of Devastation*, Goblin Bushwhacker, Goblin Surprise, Goblin War Party, Goldnight Commander, Goro-Goro*, Kolaghan the Storm's Fury, Mardu Ascendancy*, Mirror Entity, Purphoros, Sarkhan Vol, Triumph of the Hordes, Unbreakable Formation*, Vault of the Archangel, Vito† , You See a Pair of Goblins |
| `CreaturesYouControlExcludingSubtype` | 1 | Return of the Wildspeaker* |
| `CreaturesYouControlWithSubtype` | 4 | Battle Cry Goblin*, Elvish Warmaster, Ezuri Renegade Leader, Lathliss Dragon Queen |
| `AttachedCreature` | 1 | Umezawa's Jitte |

`*` = `partial` (8) · `†` = `known_wrong` (1) · everything unmarked = `Complete` (28).

The remaining 79 defs use `DeclaredTarget` (54), `Source` (21) or `TriggeringCreature` (6). All
three resolve to `EffectFilter::SingleObject` at execution (`effects/mod.rs:3831-3853`), so a
snapshot of `{that id}` is behaviourally identical to the live filter — no change expected, and
that is a *prediction to falsify*, not an assumption to bank.

## Next

Plan phase: `primitive-impl-planner` → `memory/primitives/pb-plan-DX5.md`.
