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
- **Tests**: 4,048 → **4,064** (+16: 14 in the new
  `crates/engine/tests/primitives/pb_dx5_affected_set_snapshot.rs`, 1 in-source
  `#[cfg(test)]` unit test in `rules/layers.rs` for T11 — `snapshot_affected_set`/
  `effect_applies_to_object`/`candidate_ids_for_filter` are all `pub(crate)`, unreachable from an
  integration test — and 1 new roster-completeness test). Every "fails before" claim was OBSERVED
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
- **Seeds filed**: `docs/audits/decision-point-audit.md` §8.1, **OOS-DX5-1..5** + a checked
  non-finding **OOS-DX5-6** (Mirror Entity is the one Layer ≤4 mass-filter def; unaffected today
  since no roster member writes `CardType::Creature` via a Layer-4 modification).
- **All gates green**: `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo fmt --check`, `tools/check-defs-fmt.sh` (1,804 defs), `cargo test --workspace`
  (4,064 / 0).

## Docs updated

- `docs/audits/decision-point-audit.md` §8.1 — OOS-DX5-1..6 appended.
- `memory/primitives/seed-rerank-2026-07-27.md` — §2.3 table row, §4 dispatch table row, and the
  full §4 dispatch-brief entry all marked SHIPPED with the corrected 38/29/8/1 split.
- `memory/primitive-wip.md` (this file) — phase → done, result summary above.
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
