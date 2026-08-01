# Primitive batch WIP — PB-DX5

**Batch**: PB-DX5 — CR 611.2c: lock the affected set of a resolution-generated continuous effect
**Seed**: OOS-OS7-2 (ex-RS6; `memory/primitives/seed-rerank-2026-07-27.md` §2.3 + §4 dispatch brief)
**Task**: `scutemob-170` · **Branch**: `feat/pb-dx5-cr-6112c-lock-the-affected-set-of-a-resolution-genera`
**Phase**: plan

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
