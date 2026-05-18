# Primitive Batch Review: PB-AC0 — ETBTriggerFilter subtype/nontoken fields (creature-ETB filter forwarding)

**Date**: 2026-05-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 603.2, 603.10 / 603.10a, 111.1, 205.3, 613.1d, 400.7
**Engine files reviewed**: `crates/engine/src/testing/replay_harness.rs` (~L2405),
`crates/engine/src/rules/abilities.rs` (~L6182-6211 ETB block; L4287-4376 death block — verified untouched)
**Card defs reviewed**: `ganax_astral_hunter.rs`, `lathliss_dragon_queen.rs`, `the_great_henge.rs`,
`miirym_sentinel_wyrm.rs`, `dragons_hoard.rs`, `bloomvine_regent.rs` (6 total)
**Tests reviewed**: `crates/engine/tests/etb_trigger_subtype_filter.rs` (11 tests)

## Verdict: NEEDS-FIX

The engine change is CR-correct and well-scoped, and all six card definitions match
their MCP oracle text exactly. However there is one finding that, per
`memory/conventions.md` ("Test-validity MEDIUMs are fix-phase HIGHs"), must be
resolved before close: **Change 1 (the harness `triggering_creature_filter` forwarding)
has zero test coverage** — all 11 tests bypass `enrich_spec_from_def` by wiring the
runtime trigger directly via `ObjectSpec::with_triggered_ability`, so reverting Change 1
would leave the entire suite green. The two "integration" tests do not integrate the
re-authored card definitions. Engine logic and card defs are otherwise clean.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | LOW | `abilities.rs:6193-6194` | **Scoping comment slightly imprecise.** Comment says "Scoped INSIDE the etb_filter block so death/attack defs ... are not double-evaluated here" — accurate, but the death path is in a *different function* (`apply_zone_change_triggers`-area, ~L4287), not merely "not double-evaluated." Minor wording. **Fix:** optional clarification; not blocking. |

The two substantive engine changes are **correct**:

- **Change 1** (`replay_harness.rs:2411`): `triggering_creature_filter: filter.clone()`
  forwards the full carddef `TargetFilter` on the `WheneverCreatureEntersBattlefield`
  conversion. Type-correct (`filter` is `&Option<TargetFilter>`, field is
  `Option<TargetFilter>`). Mirrors the death-trigger conversion. CR-cited.
- **Change 2** (`abilities.rs:6195-6211`): the `triggering_creature_filter` check is
  correctly placed *inside* the `if let Some(ref etb_filter)` block, after the
  `card_type_filter` check, reusing the already-computed layer-resolved `entering_chars`
  and `entering_obj`. Explicit `is_token`/`is_nontoken` guards precede `matches_filter`,
  exactly mirroring the death path (`abilities.rs:4350-4376`). Subtype matching uses
  `calculate_characteristics`-derived `entering_chars` → CR 613.1d satisfied
  (layer-resolved, not printed). CR 603.10 correctly observed: ETB is not look-back,
  live entering object is used (no LKI snapshot). Death/attack double-consumption
  verified impossible — the death path is a separate function, and attack/combat-damage
  defs carry `etb_filter: None` so they never enter this block.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | — | (all 6) | No findings. All match MCP oracle text; no TODOs remaining on the closed gap. |

All six card defs verified against MCP `lookup_card`:

- `ganax_astral_hunter.rs` — "Whenever Ganax or another Dragon you control enters,
  create a Treasure token." `has_subtype: Dragon`, `controller: You`,
  `exclude_self: false` ("Ganax OR another"). `treasure_token_spec(1)` correct.
  ENGINE-BLOCKED TODO removed. **Correct.**
- `lathliss_dragon_queen.rs` — "Whenever another nontoken Dragon you control enters,
  create a 5/5 red Dragon creature token with flying." `has_subtype: Dragon`,
  `is_nontoken: true`, `exclude_self: true`. Token spec 5/5 Red Dragon w/ Flying,
  `count: Fixed(1)`. Pump activated ability untouched and correct. TODO removed.
  **Correct.**
- `the_great_henge.rs` — "Whenever a nontoken creature you control enters, put a +1/+1
  counter on it and draw a card." `is_nontoken: true` added; `EffectTarget::Source` →
  `EffectTarget::TriggeringCreature` for the +1/+1 counter. Chain verified end-to-end:
  `PendingTrigger.entering_object_id` (`abilities.rs:6240`) → `stack_obj.triggering_creature_id`
  (`abilities.rs:7716`) → `ctx.triggering_creature_id` (`resolution.rs:2089/2179`) →
  read by `EffectTarget::TriggeringCreature`. The counter lands on the entering
  creature. Cost-reduction TODO correctly retained (separate gap). **Correct.**
- `miirym_sentinel_wyrm.rs` — "Whenever another nontoken Dragon you control enters,
  create a token that's a copy of it, except the token isn't legendary."
  `has_subtype: Dragon`, `is_nontoken: true`, `exclude_self: true`. Stale TODOs
  removed. **Correct.**
- `dragons_hoard.rs` — "Whenever a Dragon you control enters, put a gold counter on
  this artifact." `has_subtype: Dragon`, `exclude_self: false`, no nontoken word in
  oracle → no `is_nontoken`. No edit needed. **Correct as-is.**
- `bloomvine_regent.rs` — "Whenever this creature or another Dragon you control enters,
  you gain 3 life." `has_subtype: Dragon`, `exclude_self: false` ("this or another").
  MCP ruling 2025-04-04 confirms it triggers "for each of those Dragons, including
  itself" → `exclude_self: false` correct. No edit needed. **Correct as-is.**

## Test Findings

| # | Severity | Test(s) | Description |
|---|----------|---------|-------------|
| T1 | **HIGH** (fix-phase) | all 11, esp. `test_etb_ganax_treasure_integration`, `test_etb_lathliss_token_integration`, `test_etb_great_henge_counter_on_entering_creature` | **Change 1 (harness `triggering_creature_filter` forwarding) is entirely untested; "integration" tests do not integrate the card defs.** See detail below. |

### Finding Details

#### Finding T1: Change 1 has no test coverage; integration tests are mislabeled

**Severity**: HIGH (fix-phase — per `memory/conventions.md` "Test-validity MEDIUMs are
fix-phase HIGHs": a test whose name promises validation it does not deliver is a
fix-phase HIGH regardless of initial tag).
**File**: `crates/engine/tests/etb_trigger_subtype_filter.rs` (all 11 tests)
**Issue**:

PB-AC0 ships **two** engine changes:
1. `replay_harness.rs:2411` — `enrich_spec_from_def` forwards the carddef `TargetFilter`
   as `triggering_creature_filter` when converting a `WheneverCreatureEntersBattlefield`
   carddef ability into a runtime `TriggeredAbilityDef`.
2. `abilities.rs:6195-6211` — the matching loop honors `triggering_creature_filter`.

Every one of the 11 tests builds its watcher (the trigger source) via
`ObjectSpec::creature(...).with_triggered_ability(<runtime TriggeredAbilityDef>)`,
where the runtime def already has `triggering_creature_filter: Some(...)` populated
by hand. None of the registry `CardDefinition`s in any test carry a
`WheneverCreatureEntersBattlefield` ability, so `enrich_spec_from_def`'s
`WheneverCreatureEntersBattlefield` conversion arm — the site of Change 1 — is never
exercised.

Consequence: **if Change 1 were reverted** (`triggering_creature_filter: filter.clone()`
→ `triggering_creature_filter: None`), **all 11 tests would still pass.** Change 1 is
provably undiscriminated. This is the exact PB-N F3/F4 / PB-Q4 failure mode the
conventions doc calls out — a test that passes against both pre-fix and post-fix
engines.

The three tests named `test_etb_ganax_treasure_integration`,
`test_etb_lathliss_token_integration`, and `test_etb_great_henge_counter_on_entering_creature`
claim ("full card-def integration", "mirror the re-authored card definition") to
integrate the re-authored cards. They do **not**: they hand-build equivalent runtime
triggers and never register `ganax_astral_hunter` / `lathliss_dragon_queen` /
`the_great_henge`'s actual `CardDefinition`. The re-authored card defs themselves —
the deliverable of acceptance criterion 4082 — have **no end-to-end test through their
real definition path**.

This matters because `enrich_spec_from_def` is *not* a test-only convenience: it is
the canonical card-def → object conversion used by `build_initial_state`, the real
game-construction path (the replay harness is public and shared with the simulator and
replay viewer per CLAUDE.md). `WheneverCreatureEntersBattlefield` carddef triggers are
dispatched in real games **only** via the runtime `TriggeredAbilityDef` that
`enrich_spec_from_def` produces (abilities.rs has no direct match on
`WheneverCreatureEntersBattlefield`; `calculate_characteristics` only copies
`triggered_abilities`, it does not synthesize them from the def). So Change 1 is on
the production path, and leaving it untested means the actual Ganax/Lathliss/Miirym
over-trigger fix has no regression guard.

The implementer's deviation note ("`enrich_spec_from_def` only runs at
`build_initial_state` time; CastSpell resolution does not re-enrich") is true but
addresses the wrong object. It is correct for the *entering* card. But the *watcher*
(the card carrying the `WheneverCreatureEntersBattlefield` trigger) sits on the
battlefield from turn 1, placed there by the builder — so if the watcher's `ObjectSpec`
had a `card_id` pointing at a registry def *with* the `WheneverCreatureEntersBattlefield`
ability and **no** manual `with_triggered_ability` call, `enrich_spec_from_def` would
run Change 1 on it. `crates/engine/tests/alliance.rs:288-345` already demonstrates this
exact pattern (registry def with a `WheneverCreatureEntersBattlefield` ability + a
watcher object carrying that `card_id`). PB-AC0's tests do not use it.

**Fix**: Add (or convert) at least one fire-on-match **and** one no-fire-on-mismatch
test where the watcher is created from a registry `CardDefinition` that itself contains
a `WheneverCreatureEntersBattlefield` ability with a `has_subtype` filter (and one with
`is_nontoken`), with **no** manual `with_triggered_ability` on the watcher's
`ObjectSpec` — so `enrich_spec_from_def` performs the carddef→runtime conversion and
Change 1 is in the discrimination path. Mirror the `alliance.rs:288` setup. Ideally
register the actual `ganax_astral_hunter` / `lathliss_dragon_queen` card defs from the
`cards::defs` module so the "integration" tests genuinely integrate the re-authored
cards. Verify discrimination: with `triggering_creature_filter: None` restored in
`replay_harness.rs`, the new test(s) must fail.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 603.2 (trigger event matching) | Yes | Yes | tests 1,2,8 fire/no-fire on subtype |
| 205.3 (subtypes) | Yes | Yes | `matches_filter` checks `has_subtype` vs `chars.subtypes` |
| 111.1 (token vs nontoken) | Yes | Yes | tests 3,4,9 — explicit `is_token`/`is_nontoken` guards |
| 613.1d (layer-resolved subtypes) | Yes | Partial | test 7 enters a base-Dragon; does not exercise a *granted* subtype, so the layer-resolution path is only weakly distinguished from a printed-subtype path. LOW — acceptable; plan permitted a downgrade. |
| 603.10 (ETB not look-back) | Yes | Yes (by construction) | live `entering_chars` used; no LKI |
| 603.10a (death looks back — unaffected) | N/A (regression) | Yes | test 11 — death def w/ `etb_filter: None`, confirms scoping |
| 400.7 (object identity) | N/A | — | not exercised by this primitive |
| Change 1 (harness forwarding) | Yes | **No** | Finding T1 |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| ganax_astral_hunter | Yes | 0 | Yes | ENGINE-BLOCKED TODO removed; live Dragon-ETB Treasure trigger |
| lathliss_dragon_queen | Yes | 0 | Yes | TODO removed; nontoken-Dragon-ETB 5/5 token; pump ability untouched |
| the_great_henge | Yes | 1 (cost-reduction — unrelated, correctly kept) | Yes | `is_nontoken` added; `Source`→`TriggeringCreature` verified correct |
| miirym_sentinel_wyrm | Yes | 0 | Yes | `is_nontoken` added; 2 stale TODOs removed |
| dragons_hoard | Yes | 0 | Yes | No edit needed; over-trigger fixed by engine change |
| bloomvine_regent | Yes | 0 | Yes | No edit needed; `exclude_self: false` confirmed by MCP ruling 2025-04-04 |

## Hash Impact

Confirmed correct: no struct/enum field shape change. `triggering_creature_filter`
already exists on `TriggeredAbilityDef` and is already hashed. `HASH_SCHEMA_VERSION`
correctly left at 27. No parity-test change required.

## Summary of Required Action

1. **T1 (HIGH, fix-phase)** — add card-def-driven `enrich_spec_from_def` test coverage
   so Change 1 is discriminated; ideally make the three "integration" tests register
   and use the actual re-authored card defs. Verify the new test(s) fail with Change 1
   reverted.
2. **E1 (LOW)** — optional comment clarification; non-blocking.

Engine logic and all six card definitions are correct; the gate is purely the test
coverage of Change 1.
