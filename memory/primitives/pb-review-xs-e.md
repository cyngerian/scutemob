# Primitive Batch Review: PB-XS-E — Trigger-side `exclude_self` for ETB Triggers

**Date**: 2026-05-15
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 109.1 (object identity), CR 603.2 (triggered abilities), CR 207.2c (ability words: Alliance/Landfall), CR 400.7 (zone-change identity for graveyard dispatch path).
**Engine files reviewed**:
- `crates/engine/src/cards/card_definition.rs` (enum surface lines 2723–2766)
- `crates/engine/src/state/hash.rs` (HASH_SCHEMA_VERSION bump 19→20; arms at 4670–4687)
- `crates/engine/src/testing/replay_harness.rs` (runtime conversion lines 2338–2505)
- `crates/engine/src/rules/abilities.rs` (battlefield enforcement 6119–6168; graveyard dispatch 6338–6377)
- `crates/engine/src/state/game_object.rs` (ETBTriggerFilter 548–566 — unchanged)
**Tests reviewed**:
- `crates/engine/tests/primitive_pb_xs_e.rs` (11 new tests A–H)
- `crates/engine/tests/alliance.rs` (line 302 migration)
- 13 PB hash-canary sites uniformly bumped to `20u8` with PB-XS-E sentinel string.
**Card defs reviewed**: 12 of 60 spot-checked end-to-end against MCP-verified oracle text — Shadow Alley Denizen, Metastatic Evangel, Forerunner of the Legion, Marwyn the Nurturer, Cathars' Crusade, Risen Reef, Lotus Cobra, Ayara First of Locthwain, Bloomvine Regent, Satoru the Infiltrator, Witty Roastmaster, Impact Tremors, Warstorm Surge, Puresteel Paladin, Omnath Locus of Rage, Aesi Tyrant of Gyre Strait, Aura Shards, Prosperous Innkeeper, Tireless Tracker, General Kreat, Foundry Street Denizen, Horn of Greed, Molten Gatekeeper, Tatyova Benthic Druid, Evolution Sage, Purphoros, Champion of Lambholt, Elvish Warmaster, Goldnight Commander.

## Verdict: needs-fix (LOW-only — clean to merge in spirit)

End-to-end implementation is correct. The enum surface, hash arms, schema bump, runtime conversion, and per-card values all align with CR 109.1 / 603.2 / 207.2c. The trigger-collection enforcement at `rules/abilities.rs:6123` was already in place (pre-existing `ETBTriggerFilter.exclude_self` from PB-N plumbing) and now correctly receives the per-card value rather than the hardcoded `true`/`false` previously baked in by replay_harness conversion. Every spot-checked card def matches its MCP-verified oracle text. The hash bump is uniform across all 13 canary sites with a single sentinel message string. The new test file covers HASH-20 sentinel, PartialEq distinctness per variant, serde-default round-trip per variant, both creature and permanent ETB positive/negative discriminators, the inclusive default path (the silent latent bug for Risen Reef-style cards is now provably fixed), and a subtype-filtered exclude-self regression. OOS-XS-5 is marked SHIPPED; OOS-XS-E-1 and OOS-XS-E-2 are filed with concrete card rosters and follow-up scopes.

All findings below are LOW (stale comments contradicting the now-shipped implementation, and one cosmetic function-name mismatch). Zero HIGH, zero MEDIUM. The PR is functionally ready to merge; the LOW items are tech-debt that can either land in this PR or as a follow-up sweep.

---

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| — | — | — | No engine-side findings. Enum, hash, runtime conversion, and enforcement chain are correct. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `metastatic_evangel.rs:18-21` | **Stale inline comment claims `exclude_self` "is also unavailable on this trigger variant. Both are minor inaccuracies."** The implementation directly below now sets `exclude_self: true` (line 28), making the comment contradict the code. **Fix:** delete lines 18–21 (replace with a one-line CR 603.2 / "another" annotation, or remove entirely; the `nontoken` half of the comment is still accurate as a separate DSL gap and can be retained on its own line). |
| C2 | LOW | `foundry_street_denizen.rs:5-8, 23-24` | **Stale top-of-file and inline comments both assert "`exclude_self` is not supported on this trigger; the Denizen will also trigger on its own ETB. Minor inaccuracy."** No longer true — `exclude_self: true` is set on line 33 and is enforced by the engine. **Fix:** delete the top-of-file note block (lines 4–8) and replace inline lines 22–24 with a CR 603.2 / 207.2c citation. |
| C3 | LOW | `prosperous_innkeeper.rs:1-5, 27` | **Stale top-of-file header and inline comment both say "the engine's trigger collector sets `exclude_self: true` automatically for all `WheneverCreatureEntersBattlefield` triggers."** Pre-PB-XS-E this was a runtime-hardcode; PB-XS-E now reads the per-card field, and the card correctly sets `exclude_self: true`. **Fix:** rewrite lines 2–5 to "Alliance ability word (CR 207.2c) — `exclude_self: true` per oracle 'another'"; remove the parenthetical on line 27. |
| C4 | LOW | `aura_shards.rs:16-18` | **Stale comment "The engine applies `exclude_self=true` automatically for `WheneverCreatureEntersBattlefield` triggers (Aura Shards is an enchantment, so exclude_self would not matter here anyway)."** First half is no longer true; second half is correct (the source is an enchantment, can never be the entering creature). The card correctly sets `exclude_self: false`. **Fix:** rewrite to "Source is an enchantment; `exclude_self` value is moot — `false` matches oracle wording 'a creature you control'." |

## Test Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| T1 | LOW | `primitive_pb_xs.rs:65` | **Function name `test_pbxs_hash_schema_version_is_19` no longer matches the asserted value (`20u8`).** The assertion was correctly bumped, but the test name still says `is_19`. Cosmetic; does not affect pass/fail. **Fix:** rename to `test_pbxs_hash_schema_version_is_20` (or to a version-agnostic name like `test_pbxs_hash_schema_version_sentinel`). |

---

### Finding Details

#### C1: Metastatic Evangel — stale inline DSL-gap comment
**Severity**: LOW
**File**: `crates/engine/src/cards/defs/metastatic_evangel.rs:18-21`
**Oracle**: "Whenever another nontoken creature you control enters, proliferate."
**Issue**: The comment block predates PB-XS-E and asserts that `exclude_self` is unavailable on this trigger variant. The struct literal immediately below (line 28) now sets `exclude_self: true`, contradicting the comment. The `is_token`/`nontoken_only` half of the comment is still a real DSL gap (ETBTriggerFilter doesn't propagate token-status filtering for the Creature variant path), but the `exclude_self` half is resolved.
**Fix**: Replace lines 18–21 with a single annotation citing CR 603.2 + "another". Keep a separate one-line note about the `nontoken` filter being unsupported if you want to preserve that DSL-gap signal for a future audit.

#### C2: Foundry Street Denizen — stale top-of-file + inline DSL-gap comments
**Severity**: LOW
**File**: `crates/engine/src/cards/defs/foundry_street_denizen.rs:5-8, 22-24`
**Oracle**: "Whenever another red creature you control enters, this creature gets +1/+0 until end of turn."
**Issue**: Both the file-header note (lines 4–8) and the inline comment (lines 22–24) claim the Denizen will erroneously trigger on its own ETB because `exclude_self` is not supported. PB-XS-E directly fixes this: line 33 sets `exclude_self: true`, which is enforced at `rules/abilities.rs:6123`.
**Fix**: Delete the file-header DEVIATION block. Replace the inline comment with a CR 603.2 citation noting "another" semantics enforced via `exclude_self: true`.

#### C3: Prosperous Innkeeper — stale top-of-file header + inline aside
**Severity**: LOW
**File**: `crates/engine/src/cards/defs/prosperous_innkeeper.rs:1-5, 27`
**Oracle**: "Whenever another creature you control enters, you gain 1 life." (Alliance ability word.)
**Issue**: Both the file header and the inline parenthetical state the engine applies `exclude_self: true` automatically. That was the pre-PB-XS-E runtime hardcode; PB-XS-E removes the hardcode and reads from the per-card field. The card def correctly sets `exclude_self: true` on line 34.
**Fix**: Rewrite the header to reference CR 207.2c (Alliance) and the per-card `exclude_self: true` instead of the obsolete runtime behavior.

#### C4: Aura Shards — stale comment about runtime hardcode
**Severity**: LOW
**File**: `crates/engine/src/cards/defs/aura_shards.rs:16-18`
**Oracle**: "Whenever a creature you control enters, you may destroy target artifact or enchantment."
**Issue**: Comment asserts the engine auto-sets `exclude_self: true` — no longer true. The card correctly sets `exclude_self: false` (the source is an enchantment, so the value is moot but matches the oracle wording).
**Fix**: Rewrite to "Source is an enchantment; `exclude_self: false` matches oracle ('a creature') and is harmless either way."

#### T1: PB-XS canary test — function name out of sync with assertion
**Severity**: LOW
**File**: `crates/engine/tests/primitive_pb_xs.rs:65`
**Issue**: After uniform PB-XS-E sentinel migration, the function `test_pbxs_hash_schema_version_is_19` asserts `HASH_SCHEMA_VERSION == 20u8`. The name lies; behavior is correct.
**Fix**: Rename to `test_pbxs_hash_schema_version_is_20` (or version-agnostic). Same renaming opportunity exists at other older PB canary tests (e.g. `primitive_pb_ts.rs:test_*_is_15`, `primitive_pb_lki_cc.rs:test_*_is_15`); a future cleanup pass can normalize all of them to version-agnostic names.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 109.1 — object identity (source vs. entering object are distinct objects) | Yes | Yes | Tests C-1, F-1, H-1 (source-enters → SKIPPED). |
| 603.2 — triggered abilities fire on event | Yes | Yes | Tests D-1, G-1, E-1 (another enters → FIRES; inclusive default → fires on self). |
| 207.2c — ability words (Alliance, Landfall) | Yes | Yes | `alliance.rs` migration; Lotus Cobra/Evolution Sage/Tatyova/Aesi/Tireless Tracker all carry the correct Landfall `exclude_self: false` default. |
| 400.7 — zone-change identity (graveyard-dispatch path) | Yes | Implicitly | `rules/abilities.rs:6351` adds `entering_id == obj_id` gate even though the source lives in graveyard (kept for symmetry; comment correctly notes it's moot by CR 400.7). No direct test, but the existing Bloodghast graveyard-trigger tests cover the path; this gate cannot regress the path because the equality is structurally impossible. |

---

## Card Def Sample Summary

| Card | Oracle Match | exclude_self | Notes |
|------|-------------|--------------|-------|
| Shadow Alley Denizen | Yes | true | "another black creature" — correct |
| Metastatic Evangel | Yes | true | "another nontoken creature" — exclude_self correct; nontoken still a separate DSL gap; comment is stale (C1) |
| Forerunner of the Legion | Yes | true | "another Vampire" — correct |
| Marwyn, the Nurturer | Yes | true | "another Elf" — correct (note: has_subtype filter is a pre-existing PB-N gap unrelated to this PB) |
| Cathars' Crusade | Yes | false | "a creature you control" (no "another"), source is enchantment — correct |
| Risen Reef | Yes | false | "this creature or another Elemental" — correct (self-inclusive) |
| Lotus Cobra | Yes | false | Landfall default — correct |
| Ayara, First of Locthwain | Yes | false | "Ayara or another black creature" — correct (self-inclusive) |
| Bloomvine Regent | Yes | false | "this creature or another Dragon" — correct (self-inclusive) |
| Satoru, the Infiltrator | Yes | false | "Satoru and/or one or more other nontoken creatures" — correct (self-inclusive) |
| Witty Roastmaster | Yes | true | "another creature you control" (per current oracle; Alliance keyword) — correct |
| Impact Tremors | Yes | false | "a creature you control" — correct; enchantment source, exclude_self moot |
| Warstorm Surge | Yes | false | Enchantment source — exclude_self moot |
| Puresteel Paladin | Yes | false | Equipment filter — source is non-Equipment creature, exclude_self moot |
| Omnath, Locus of Rage | Yes | false | Landfall (Land filter), source is non-land — correct |
| Aesi, Tyrant of Gyre Strait | Yes | false | Landfall — correct |
| Aura Shards | Yes | false | Enchantment source, "a creature" — correct; comment is stale (C4) |
| Prosperous Innkeeper | Yes | true | Alliance — correct; header/inline comments stale (C3) |
| Tireless Tracker | Yes | false | Landfall — correct |
| General Kreat | Yes | true | "another creature" — correct |
| Foundry Street Denizen | Yes | true | "another red creature" — correct; comments stale (C2) |
| Horn of Greed | Yes | false | Land filter, source is artifact — exclude_self moot |
| Molten Gatekeeper | Yes | true | "another creature" — correct |
| Tatyova, Benthic Druid | Yes | false | Landfall — correct |
| Evolution Sage | Yes | false | Landfall — correct |
| Purphoros, God of the Forge | Yes | true | "another creature" — correct |
| Champion of Lambholt | Yes | true | "another creature" — correct |
| Elvish Warmaster | Yes | true | "one or more other Elves" — correct |
| Goldnight Commander | Yes | true | "another creature" — correct |

No card def is wrong. All 17 `true` cards have `another` in oracle text; all `false` cards either have non-`another` oracle text or have a non-matchable source (non-creature for creature filter; non-land for land filter), making the value moot. The default-`false` flip from the prior runtime hardcode is the actual behavioral correctness fix — pre-PB-XS-E, Risen Reef/Ayara/Bloomvine/Satoru silently never fired on their own ETBs despite oracle including the source.

---

## Notes on Worker.md Acceptance Criterion #4

The acceptance criterion seed roster included Boggart Shenanigans, Athreos God of Passage, and Meren of Clan Nel Toth. MCP oracle lookup confirms these are **DIES** triggers (CR 603.10a), not ETB triggers — they are out of scope for PB-XS-E and were correctly left untouched in this PR. The worker has filed OOS-XS-E-1 to track a follow-up audit confirming their existing `WheneverCreatureDies.exclude_self` (PB-23) usage. This is the correct disposition.

The acceptance criterion text mentioning "Witty Roastmaster: Whenever a creature enters under your control" reflects an older or paraphrased oracle. MCP-verified current oracle is "Alliance — Whenever **another** creature you control enters" — so `exclude_self: true` is the correct setting, and the card def matches.

## OOS Seed Filing

| Seed | Filed | Status | Notes |
|------|-------|--------|-------|
| OOS-XS-5 | Previously | SHIPPED | Marked SHIPPED 2026-05-15 by this PB with shipped-impl reference. |
| OOS-XS-E-1 | New | Open | Dies-side card-def audit for Boggart Shenanigans, Athreos, Meren. No engine gap. |
| OOS-XS-E-2 | New | Open | Regression sweep for self-inclusive ETB cards (Risen Reef, Ayara, Bloomvine, Satoru). Generated-script audit pending. |

All three are well-scoped with card rosters, oracle patterns, gap statements, yield estimates, and CR references. Filing is complete and discoverable.

---

## Summary

**Zero HIGH, zero MEDIUM, five LOW** (four stale-comment fixes, one cosmetic test-name rename). The PR is functionally clean and ready to merge. The LOW items are tech-debt that can be applied inline in a follow-up commit or deferred — none of them affect game correctness, build status, test pass rate, or wire-format hash determinism.
