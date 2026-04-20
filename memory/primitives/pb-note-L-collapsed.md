# PB-L (Landfall) — Partial Sweep + Minimal Engine Primitive

**Date**: 2026-04-20
**Verdict**: **PARTIAL-GAP** — DSL variant existed, battlefield dispatch did not.
Resolved with a minimal engine primitive (one filter field + one conversion block
+ one filter check) plus a stale-TODO sweep across 8 Landfall card defs.
**Task**: scutemob-4

## Step 0 Finding (revised after full dispatch-chain verification)

Landfall is an **ability word** (CR 207.2c), not a keyword ability:

> "An ability word appears in italics at the beginning of some abilities. Ability
> words are similar to keywords in that they tie together cards that have similar
> functionality, but they have no special rules meaning and no individual entries
> in the Comprehensive Rules. The ability words are ... kinship, **landfall**,
> lieutenant, ..."  — CR 207.2c

So no new `TriggerCondition::Landfall` variant is needed — the correct DSL
encoding is the general `TriggerCondition::WheneverPermanentEntersBattlefield`
with a Land + You-control filter, **and that variant already existed**
(`crates/engine/src/cards/card_definition.rs:2478`).

### First-pass verdict (WRONG)

Initial Step 0 grep found:
1. The DSL variant (`WheneverPermanentEntersBattlefield`) existed.
2. 18 Landfall card defs used the standard pattern and compiled.
3. One dispatch site in `rules/abilities.rs:6145` matched the variant.

From this I concluded *EXISTS — collapse to pure stale-TODO sweep*.

### Second-pass verification (per `feedback_verify_full_chain.md`)

Writing end-to-end tests exposed the real gap. Following the full dispatch
chain from CardDef → `enrich_spec_from_def` → runtime `TriggeredAbilityDef` →
`check_triggers` → `collect_triggers_for_event` revealed:

- `enrich_spec_from_def` had no conversion block for
  `TriggerCondition::WheneverPermanentEntersBattlefield` (it handled
  `WheneverCreatureEntersBattlefield` for Alliance, and 10+ other trigger
  conditions, but not this one).
- The one dispatch site in `rules/abilities.rs:6145` was
  `collect_graveyard_carddef_triggers` — the **graveyard** path only
  (Bloodghast-style triggers with `trigger_zone: Some(TriggerZone::Graveyard)`).
- **There was no battlefield dispatch.** The 18 "working" Landfall cards
  never actually fired from the battlefield — their CardDef entries compiled,
  but the runtime spec's `characteristics.triggered_abilities` was empty of any
  `AnyPermanentEntersBattlefield` trigger, so `collect_triggers_for_event` had
  nothing to dispatch. No existing test exercised a battlefield Landfall
  interaction, which is why this had not been caught before.

This is exactly the failure mode documented in `feedback_verify_full_chain.md`:
stopping at "the variant exists" without walking the full chain produces a
confident-but-wrong verdict. Flagging the expansion and proceeding with a
minimal engine primitive.

## Engine Primitive (minimal)

Three changes, ~50 lines total:

### 1. `state/game_object.rs` — extend `ETBTriggerFilter`
Add `card_type_filter: Option<CardType>` field. The existing `creature_only: bool`
covers the Alliance ("another creature") pattern but is a boolean; supporting
Landfall requires a general card-type filter (Land, Enchantment, etc.).

### 2. `testing/replay_harness.rs` — new conversion block in `enrich_spec_from_def`
Parallel to the existing `WheneverCreatureEntersBattlefield` (Alliance) block.
Converts `TriggerCondition::WheneverPermanentEntersBattlefield { filter }` to a
runtime `TriggeredAbilityDef` with `trigger_on: AnyPermanentEntersBattlefield`
and an `ETBTriggerFilter` derived faithfully from the source `TargetFilter`:
- `creature_only` set if `has_card_type == Some(Creature)`
- `card_type_filter` set for any other card type (Land, Enchantment, Artifact, Planeswalker)
- `controller_you` set from `TargetController::You`
- `color_filter` propagated from `TargetFilter.colors`
- `exclude_self: false` (the Landfall trigger source is never the entering land;
  no "another" qualifier applies)
- Abilities with `trigger_zone: Some(TriggerZone::Graveyard)` are skipped —
  `collect_graveyard_carddef_triggers` handles those.

### 3. `rules/abilities.rs` — extend filter check in `collect_triggers_for_event`
After the existing `creature_only` / `controller_you` / `exclude_self` /
`color_filter` checks, add a `card_type_filter` check that reads
`entering_chars.card_types` (layer-resolved characteristics).

### 4. `state/hash.rs` — bump schema + HashInto
- Bump `HASH_SCHEMA_VERSION` from 6 to 7.
- Extend `impl HashInto for ETBTriggerFilter` to hash `card_type_filter`.
- Update the three PB-N/PB-P/PB-D hash-parity tests to expect 7.

## Card def changes (stale-TODO sweep)

### SIMPLE — 3 cards authored with the standard pattern
| Card | Effect |
|------|--------|
| `khalni_heart_expedition.rs` | Add Quest counter to self. |
| `druid_class.rs` | Level 1: gain 1 life. **Also fixed a latent bug**: the ability was using `TriggerCondition::WhenEntersBattlefield` (self-ETB only) instead of `WheneverPermanentEntersBattlefield { Land + You }`. Pre-sweep, the Class gained 1 life when it entered, and never again; post-sweep, it gains 1 life on every land ETB as written. |
| `omnath_locus_of_rage.rs` | Create 5/5 red+green Elemental token. The subtype-filtered death trigger remains TODO (separate blocker). |

### COMPOUND-BLOCKED — 5 cards, TODOs rewritten to name real blocker
| Card | Real blocker (not Landfall) |
|------|------------------------------|
| `mossborn_hydra.rs` | `EffectAmount::CurrentCountersOnSource` for "double the existing +1/+1 counters" |
| `springheart_nantuko.rs` | Bestow keyword, Aura static grant, conditional copy-or-Insect fallback |
| `roil_elemental.rs` | `EffectDuration::WhileYouControlSource` for dynamic control change |
| `moraug_fury_of_akoum.rs` | Additional-combat-phase + delayed-trigger untap chaining |
| `omnath_locus_of_creation.rs` | Per-ability-per-turn resolution counter primitive |

## Tests

New file: `crates/engine/tests/pb_l_landfall.rs` (9 tests).
- `test_lotus_cobra_landfall_triggers_on_own_land` — positive, single-player
- `test_lotus_cobra_landfall_does_not_trigger_on_opponent_land` — **MANDATORY non-you-control negative (AC 3429)**
- `test_lotus_cobra_landfall_does_not_trigger_on_non_land` — filter isolation on card type
- `test_landfall_multiplayer_isolation_4p` — **MANDATORY multiplayer-isolation (AC 3429)**: p1's Cobra triggers only on p1's land, never on p2/p3/p4's
- `test_bloodghast_landfall_triggers_from_graveyard` — graveyard-zone dispatch (CR 603.3)
- `test_bloodghast_landfall_does_not_trigger_on_opponent_land` — you-control filter from graveyard
- `test_khalni_heart_expedition_landfall_triggers` — coverage for the newly-authored card
- `test_druid_class_landfall_triggers_on_land_not_on_self_etb` — regression guard for the druid_class latent bug
- `test_omnath_locus_of_rage_landfall_triggers` — coverage for the newly-authored card

## CR Citations

AC 3429 says "cites CR 614.12 or relevant subrule." CR 614.12 itself is about
replacement effects modifying how permanents enter the battlefield, which does
not apply to Landfall triggers. The relevant subrules cited in the tests are:

- **CR 207.2c** — ability words (Landfall is one); no special rules meaning, no
  individual CR entry
- **CR 603.2** — triggered abilities with "whenever" check once per event
- **CR 603.3** — triggers fire regardless of the source's zone; `trigger_zone:
  Some(TriggerZone::Graveyard)` marks abilities that watch events from the
  graveyard (Bloodghast)
- **CR 603.6** — zone-change triggers (for "enters the battlefield")

## Acceptance Criteria Mapping

| AC | Status | Notes |
|----|--------|-------|
| 3425 Step 0 verdict | **PARTIAL-GAP** | Documented here. |
| 3426 new TriggerCondition variant | **N/A** | No new variant. `WheneverPermanentEntersBattlefield` already existed; the gap was in the conversion/dispatch pipeline. `ETBTriggerFilter.card_type_filter` field added + `HASH_SCHEMA_VERSION` bumped 6 → 7. |
| 3427 dispatch sites | **DONE** | `enrich_spec_from_def` converts to runtime `TriggeredAbilityDef`; `collect_triggers_for_event` applies the filter; `collect_graveyard_carddef_triggers` already handled `trigger_zone: Some(Graveyard)`. |
| 3428 card defs updated | **DONE** | 3 simple fixes + 5 TODO-comment rewrites pointing at real blockers. Confirmed yield 3/8 SIMPLE + 5/8 STILL-BLOCKED. |
| 3429 mandatory tests | **DONE** | 9 tests in `pb_l_landfall.rs`; includes mandatory multiplayer-isolation (`test_landfall_multiplayer_isolation_4p`) and non-you-control negative (`test_lotus_cobra_landfall_does_not_trigger_on_opponent_land`). Each cites the relevant CR subrule. |
| 3430 cargo test/clippy/fmt | **DONE** | All tests pass. Clippy produces the same 8 baseline `collapsible_match` errors as HEAD~1 — no new clippy warnings introduced. Fmt clean. |
| 3431 delegation | **DONE** | `card-fix-applicator` agent applied the 3 simple fixes + 5 TODO rewrites. Engine primitive (50 lines across 3 files) applied inline due to scope/certainty; no planner/runner spawn was warranted. Stop-and-flag comment posted at verdict revision. |
