# Primitive Batch Review: PB-AC0 — ETBTriggerFilter subtype/nontoken fields (creature-ETB filter forwarding)

**Date**: 2026-05-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 603.2, 603.10 / 603.10a, 111.1, 205.3, 613.1d, 400.7
**Engine files reviewed**: `crates/engine/src/testing/replay_harness.rs` (~L2405),
`crates/engine/src/rules/abilities.rs` (~L6182-6211 ETB block; L4287-4376 death block — verified untouched)
**Card defs reviewed**: `ganax_astral_hunter.rs`, `lathliss_dragon_queen.rs`, `the_great_henge.rs`,
`miirym_sentinel_wyrm.rs`, `dragons_hoard.rs`, `bloomvine_regent.rs` (6 total)
**Tests reviewed**: `crates/engine/tests/etb_trigger_subtype_filter.rs` (11 tests → 13 after fix phase)

## Verdict: NEEDS-FIX (original) → **PASS** (re-review 2026-05-18, fix commit `a7ebac79`)

The engine change is CR-correct and well-scoped, and all six card definitions match
their MCP oracle text exactly. The original review raised one finding that, per
`memory/conventions.md` ("Test-validity MEDIUMs are fix-phase HIGHs"), had to be
resolved before close: **Change 1 (the harness `triggering_creature_filter` forwarding)
had zero test coverage** — all 11 tests bypassed `enrich_spec_from_def` by wiring the
runtime trigger directly via `ObjectSpec::with_triggered_ability`. The fix phase added
2 card-def-driven tests that genuinely exercise `replay_harness.rs:2411`; T1 is now
**RESOLVED**. E1's comment fix introduced a minor factual inaccuracy (a non-existent
function name) — non-blocking LOW, noted below. **Final verdict: PASS.**

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | LOW | `abilities.rs:6193-6196` | **Scoping comment names a non-existent function.** Fix-phase comment now reads "Death defs are handled in apply_zone_change_triggers (~L4287), a separate function." There is no function `apply_zone_change_triggers` in the codebase (grep: only this comment matches). The death block at L4287 is inside `check_triggers` (`abilities.rs:2586-6027`); the Change-2 ETB block at L6197 is inside `collect_triggers_for_event` (`abilities.rs:6027-6362`). The "separate function" claim is *true*, but the named function is wrong. **Fix (optional, non-blocking):** change `apply_zone_change_triggers` → `check_triggers`. |

The two substantive engine changes are **correct** (unchanged from original review):

- **Change 1** (`replay_harness.rs:2411`): `triggering_creature_filter: filter.clone()`
  forwards the full carddef `TargetFilter` on the `WheneverCreatureEntersBattlefield`
  conversion. Type-correct (`filter` is `&Option<TargetFilter>`, field is
  `Option<TargetFilter>`). Mirrors the death-trigger conversion. CR-cited.
- **Change 2** (`abilities.rs:6197-6213`): the `triggering_creature_filter` check is
  correctly placed *inside* the `if let Some(ref etb_filter)` block, after the
  `card_type_filter` check, reusing the already-computed layer-resolved `entering_chars`
  and `entering_obj`. Explicit `is_token`/`is_nontoken` guards precede `matches_filter`,
  exactly mirroring the death path. Subtype matching uses `calculate_characteristics`-derived
  `entering_chars` → CR 613.1d satisfied (layer-resolved, not printed). CR 603.10
  correctly observed: ETB is not look-back, live entering object is used (no LKI
  snapshot). When `triggering_creature_filter` is `None` the entire `if let Some(...)`
  block is skipped — confirmed: this is exactly what makes reverting Change 1 over-fire.

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | — | (all 6) | No findings. All match MCP oracle text; no TODOs remaining on the closed gap. |

(Card def detail unchanged from original review — see "Card Def Summary" below.)

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
  `EffectTarget::TriggeringCreature` for the +1/+1 counter. Chain verified end-to-end.
  Cost-reduction TODO correctly retained (separate gap). **Correct.**
- `miirym_sentinel_wyrm.rs` — `has_subtype: Dragon`, `is_nontoken: true`,
  `exclude_self: true`. Stale TODOs removed. **Correct.**
- `dragons_hoard.rs` — `has_subtype: Dragon`, `exclude_self: false`, no `is_nontoken`.
  No edit needed. **Correct as-is.**
- `bloomvine_regent.rs` — `has_subtype: Dragon`, `exclude_self: false` ("this or
  another"). No edit needed. **Correct as-is.**

## Test Findings

| # | Severity | Test(s) | Description |
|---|----------|---------|-------------|
| T1 | **HIGH** (fix-phase) | originally all 11; **RESOLVED** by tests 12 + 13 | Change 1 (harness `triggering_creature_filter` forwarding) had zero discriminating coverage. See "Re-Review" below. |

### Finding Details

#### Finding T1: Change 1 had no test coverage; integration tests were mislabeled — **RESOLVED**

**Severity**: HIGH (fix-phase). **Status**: RESOLVED — see re-review section.

(Original finding text retained for the record:)

PB-AC0 ships **two** engine changes: (1) `replay_harness.rs:2411` forwards the carddef
`TargetFilter` as `triggering_creature_filter`; (2) `abilities.rs` honors it. Every one
of the original 11 tests built its watcher via `ObjectSpec::with_triggered_ability(<runtime
def>)`, so `enrich_spec_from_def`'s `WheneverCreatureEntersBattlefield` conversion arm —
the site of Change 1 — was never exercised. Reverting Change 1 would leave all 11 tests
green. The three "integration"-named tests hand-built equivalent runtime triggers and
never registered the actual re-authored `CardDefinition`s.

**Fix directive given**: add a fire-on-match and a no-fire-on-mismatch test where the
watcher is built from a registry `CardDefinition` (with `has_subtype` and `is_nontoken`
filters) via `enrich_spec_from_def` and **no** manual `with_triggered_ability`. Verify
discrimination by reverting Change 1 → `None`.

## Re-Review (fix commit `a7ebac79`, 2026-05-18)

### T1 — RESOLVED ✓

The fix phase added two tests (12 + 13) to `crates/engine/tests/etb_trigger_subtype_filter.rs`.
Verified independently:

- **`test_etb_ganax_carddef_integration_via_enrich`** (`etb_trigger_subtype_filter.rs:1383`).
  The watcher is built `enrich_spec_from_def(ObjectSpec::card(p1, "Ganax, Astral Hunter")
  .in_zone(ZoneId::Battlefield), &defs)` where `defs = load_defs()` =
  `all_cards()` keyed by name — i.e. **the real registered `ganax_astral_hunter`
  `CardDefinition`**. There is **no** `with_triggered_ability` call on the watcher spec.
  Confirmed: `ganax_astral_hunter.rs:22-36` carries
  `AbilityDefinition::Triggered { trigger_condition: WheneverCreatureEntersBattlefield {
  filter: Some(TargetFilter { has_subtype: Dragon, controller: You, .. }), exclude_self:
  false }, .. }`. This is exactly the pattern matched by the `enrich_spec_from_def`
  conversion arm at `replay_harness.rs:2360-2414`, so Change 1's
  `triggering_creature_filter: filter.clone()` at **L2411 genuinely runs** on this
  watcher. Coverage: Goblin enters → 0 Treasures (no-fire-on-mismatch, subtype filter);
  Dragon enters → +1 Treasure (fire-on-match, subtype filter). **Both directions, real
  carddef path. ✓**

- **`test_etb_lathliss_carddef_integration_via_enrich`** (`etb_trigger_subtype_filter.rs:1503`).
  Watcher built the same way from the real `lathliss_dragon_queen` `CardDefinition`
  (`lathliss_dragon_queen.rs:25-58`: `WheneverCreatureEntersBattlefield { filter:
  Some(TargetFilter { has_subtype: Dragon, controller: You, is_nontoken: true, .. }),
  exclude_self: true }`). No `with_triggered_ability`. Coverage: nontoken Dragon enters
  → +1 Dragon token (fire-on-match, subtype + nontoken filter); a separate creature's
  ETB creates a *token* Dragon → `trigger_count_for(lathliss_id) == 0` (no-fire-on-mismatch,
  nontoken filter). **Both directions, real carddef path, exercises the `is_nontoken`
  field specifically. ✓**

**Discrimination verified (logic-traced):** With Change 1 reverted
(`replay_harness.rs:2411` → `triggering_creature_filter: None`), the conversion arm
produces a runtime `TriggeredAbilityDef` with `triggering_creature_filter: None`. In
`collect_triggers_for_event` the `if let Some(ref creature_filter) =
trigger_def.triggering_creature_filter` block at `abilities.rs:6197` is then entirely
skipped — no subtype check, no nontoken check. The trigger fires for *any* creature
that passes the earlier `etb_filter` checks (`creature_only` / `controller_you` /
`exclude_self`). Consequently:
  - Ganax test: Goblin enters → Ganax over-fires → a Treasure is created →
    `assert_eq!(count_treasures, initial_treasures)` **fails**.
  - Lathliss test: token Dragon enters → Lathliss over-fires → a Lathliss trigger is
    pending/on-stack → `assert_eq!(trigger_count_for(lathliss_id), 0)` **fails**.
The runner's reported discrimination check (both new tests FAIL with Change 1 reverted;
restoring `filter.clone()` makes them pass) is consistent with this trace. The runner
also reports the original 11 tests stay green under the revert — expected, since they
hand-wire `triggering_creature_filter` and never touch the conversion arm. This
confirms Change 1 was genuinely unexercised before the fix and is now discriminated.

**Scope check:** the fix touched only `etb_trigger_subtype_filter.rs` (test count
11 → 13 — verified by enumerating `fn test_*`; the original 11 are untouched) and the
comment block at `abilities.rs:6193-6196`. No engine logic, no card defs, no other
files. No scope creep. Gates per runner: 2873 tests pass, build/clippy/fmt clean.

### E1 — Fixed, but with a residual LOW

The comment at `abilities.rs:6193-6196` was tightened. It now correctly states the
death path is in a *separate function* and that attack/combat-damage defs carry
`etb_filter: None`. However it names that function `apply_zone_change_triggers`, which
**does not exist** — the death block at L4287 is inside `check_triggers`. The original
E1 (a wording imprecision) is addressed; the fix substituted a wrong function name.
This is a non-blocking LOW (a misleading comment, no behavioral impact). Recommend a
trivial follow-up: `apply_zone_change_triggers` → `check_triggers`. Not gating.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 603.2 (trigger event matching) | Yes | Yes | tests 1,2,8; now also 12,13 via real carddef path |
| 205.3 (subtypes) | Yes | Yes | `matches_filter` checks `has_subtype` vs `chars.subtypes`; test 12 fire/no-fire |
| 111.1 (token vs nontoken) | Yes | Yes | tests 3,4,9; now also test 13 via real carddef path |
| 613.1d (layer-resolved subtypes) | Yes | Partial | test 7 enters base-Dragon; granted-subtype path only weakly distinguished. LOW — plan permitted a downgrade. |
| 603.10 (ETB not look-back) | Yes | Yes (by construction) | live `entering_chars` used; no LKI |
| 603.10a (death looks back — unaffected) | N/A (regression) | Yes | test 11 — death def w/ `etb_filter: None` |
| 400.7 (object identity) | N/A | — | not exercised by this primitive |
| Change 1 (harness forwarding) | Yes | **Yes** | tests 12 + 13 — RESOLVED |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| ganax_astral_hunter | Yes | 0 | Yes | ENGINE-BLOCKED TODO removed; now covered end-to-end by test 12 |
| lathliss_dragon_queen | Yes | 0 | Yes | TODO removed; now covered end-to-end by test 13 |
| the_great_henge | Yes | 1 (cost-reduction — unrelated, correctly kept) | Yes | `is_nontoken` added; `Source`→`TriggeringCreature` verified |
| miirym_sentinel_wyrm | Yes | 0 | Yes | `is_nontoken` added; 2 stale TODOs removed |
| dragons_hoard | Yes | 0 | Yes | No edit needed; over-trigger fixed by engine change |
| bloomvine_regent | Yes | 0 | Yes | No edit needed; `exclude_self: false` confirmed by MCP ruling 2025-04-04 |

## Hash Impact

Confirmed correct: no struct/enum field shape change. `triggering_creature_filter`
already exists on `TriggeredAbilityDef` and is already hashed. `HASH_SCHEMA_VERSION`
correctly left at 27. No parity-test change required.

## Previous Findings

| # | Previous Status | Current Status | Notes |
|---|----------------|----------------|-------|
| T1 | OPEN (HIGH, fix-phase) | **RESOLVED** | Tests 12 + 13 build the watcher from the real Ganax/Lathliss `CardDefinition` via `enrich_spec_from_def` (no `with_triggered_ability`), genuinely exercising `replay_harness.rs:2411`. Discrimination verified by trace: reverting Change 1 → `None` skips the `abilities.rs:6197` filter block → over-fire → both new tests' no-fire assertions fail. Fire-on-match + no-fire-on-mismatch covered for both subtype and nontoken filters. |
| E1 | OPEN (LOW) | **PARTIALLY FIXED** (residual LOW) | Comment tightened and now correctly states the death path is a separate function — but names a non-existent function `apply_zone_change_triggers`; the actual containing function is `check_triggers`. Non-blocking; trivial follow-up recommended. |

## Final Verdict: PASS

T1 (the sole blocking finding) is resolved correctly and verifiably. The two new tests
genuinely exercise Change 1 through the production card-def → object conversion path,
cover both fire-on-match and no-fire-on-mismatch for both the subtype and the nontoken
filter, and are discriminating (reverting Change 1 breaks them). No regressions, no
scope creep. The only residual issue is E1's comment naming a non-existent function —
a cosmetic LOW with zero behavioral impact, not gating. PB-AC0 is cleared to close.
