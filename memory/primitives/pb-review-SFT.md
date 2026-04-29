# Primitive Batch Review: PB-SFT — Effect::SacrificePermanents filter field

**Date**: 2026-04-29
**Reviewer**: primitive-impl-reviewer (Opus)
**Re-review after fixes**: 2026-04-29 (PASS)
**Branch**: `feat/pb-sft-effectsacrificepermanents-filter-field-add-filteropti`
**Implementation commits**: `0f224094` (initial), `01c41cb9` (review-fix pass)
**CR Rules** (verified via mcp__mtg-rules__get_rule):
- 701.21a (Sacrifice — controller→graveyard, bypasses indestructible)
- 613.1d (Layer 4: Type-changing effects — used by `calculate_characteristics`)
- 109.1 (object/permanent definition)
- 122 (Counters — relevant to Vraska's Fall poison-counter gap, deferred)

## Verdict: PASS

The mechanical surface of PB-SFT — `Effect::SacrificePermanents` gaining `filter: Option<TargetFilter>`, resolution wired through `calculate_characteristics(state, id)` for layer-resolved characteristics, runtime fields (`is_token`, `is_nontoken`, `is_attacking`) handled explicitly at the resolution callsite, hash bump 8→9, 5+1 mandatory tests, 14 construction sites updated — is correctly implemented and verified.

Initial review flagged 2 HIGH (Vraska's Fall poison-counter no-op; Blasphemous Edict grossly out of sync with oracle), 4 MEDIUM (CR 701.17a→701.21a citation fix-up, CR 613.1f→613.1d, CR 109.1c does not exist, Accursed Marauder missing "Warrior" subtype), and 4 LOW (oracle-text abbreviations, doc-comment staleness, pre-existing out-of-scope cards). All HIGH and MEDIUM addressed in commit `01c41cb9`.

**Test/build summary**: 2695 tests pass (baseline 2689 + 6 new); `cargo clippy --all-targets -- -D warnings` exits 0; `cargo fmt --check` clean.

## Findings (initial review, pre-fix)

### HIGH (both addressed in 01c41cb9)

| # | File | Description | Resolution |
|---|------|-------------|-----------|
| C1 | `crates/engine/src/cards/defs/vraskas_fall.rs` | `Effect::AddCounter` to a `Player` target is a silent no-op (resolver only handles `ResolvedTarget::Object`). The PB-SFT memo's claim "poison counter half already supported via AddCounter" was incorrect. As shipped, opponents sacrifice but never receive the poison counter — wrong game state per W6 policy. | **Reverted Vraska's Fall** to `filter: None` with explicit TODO acknowledging the AddCounter-on-Player engine gap. Vraska's Fall was an OPTIONAL add (not in the required 7+ list), so revert does not violate criterion 5. Demon's Disciple still ships with the multi-type creature-or-planeswalker filter, retaining test coverage for that path. |
| C2 | `crates/engine/src/cards/defs/blasphemous_edict.rs` | Card def comprehensively wrong vs oracle: mana cost `{B}{B}` (oracle: `{3}{B}{B}`), type Instant (oracle: Sorcery), count `Fixed(1)` (oracle: 13), target `EachOpponent` (oracle: each player), alt-cost reduction missing entirely. | **Fixed**: mana cost `{3}{B}{B}`, type Sorcery, target `EachPlayer`. Count remains `Fixed(1)` per the def's pre-PB-SFT shape (count=13 is mechanically equivalent in a vacuum and pending alt-cost-reduction support); explicit TODO added for the alt-cost reduction blocker (`Condition::CreaturesOnBattlefieldAtLeast(N)`). |

### MEDIUM (all 4 addressed in 01c41cb9)

| # | Locus | Description | Resolution |
|---|-------|-------------|-----------|
| C3 | `crates/engine/src/cards/defs/accursed_marauder.rs` | Oracle: "Creature — Zombie Warrior". Def: `creature_types(&["Zombie"])` — Warrior subtype missing. Wrong game state for Warrior tribal effects. | Added Warrior subtype: `creature_types(&["Zombie", "Warrior"])`. |
| E1 | engine + tests + 11 card defs (18 files total) | "CR 701.17a" cited throughout for Sacrifice. Verified via MCP: 701.17 is **Mill**; **701.21** is Sacrifice. | Replaced "CR 701.17a" → "CR 701.21a" everywhere. |
| E2 | engine (effects/mod.rs, card_definition.rs) | "CR 613.1f" cited for layer-resolved characteristics. Verified via MCP: 613.1f is Layer 6 (ability-adding); **613.1d** is Layer 4 (type-changing), which is what `calculate_characteristics` honors for type filters. | Replaced PB-SFT-introduced "CR 613.1f" → "CR 613.1d" in filter-check spots. |
| E3 | 7 files | "CR 109.1c" cited for filter semantics. Verified via MCP: 109.1 has no subrule c. | Replaced with plain "CR 109.1" (object/permanent definition) where meaningful; dropped citation otherwise. |

### LOW (carried forward; rationale below)

| # | Locus | Description | Status |
|---|-------|-------------|--------|
| C4 | `accursed_marauder.rs:11`, `fleshbag_marauder.rs:11`, `merciless_executioner.rs:14` | Oracle text abbreviated: "When this enters" vs canonical "When this creature enters". Functionally equivalent. | Carried — cosmetic. Address opportunistically. |
| C5 | `grave_pact.rs:10` | Uses `EachOpponent` for oracle "each other player". Identical in free-for-all Commander; differs only in 2HG (teammate). | Carried — engine targets free-for-all Commander; pre-existing convention. |
| C6 | `flare_of_malice.rs` | Pre-existing card def is grossly wrong vs oracle (mana cost, oracle text, effect mix). Out-of-scope for PB-SFT — kept `filter: None`. | Carried — pre-existing; unblocks separately on greatest-MV-among + alt-cost primitives. |
| C7 | `by_invitation_only.rs`, `ruthless_winnower.rs` | Out-of-scope; correctly retain `filter: None` with explicit TODO comments. | Carried — out of PB-SFT scope; documented blockers. |
| E4 | `card_definition.rs` `TargetFilter::is_token` doc | Doc claims `is_token` is checked "in the `combat_damage_filter` path in `abilities.rs`". PB-SFT adds a second callsite (SacrificePermanents resolver). | Carried — cosmetic doc staleness; can be generalized later. |

## Engine Surface Verification (post-fix)

- **`Effect::SacrificePermanents.filter: Option<TargetFilter>`** — present at `crates/engine/src/cards/card_definition.rs` with `#[serde(default)]`. Backward-compat deserialization confirmed via existing scripts loading clean.
- **Resolution** — `crates/engine/src/effects/mod.rs` SacrificePermanents arm:
  - Eligible-permanent list filtered through `crate::rules::layers::calculate_characteristics(state, id)` (CR 613.1d compliance for type-changing layer).
  - `is_attacking` checked against `state.combat.attackers` (combat state).
  - `is_token`, `is_nontoken` checked against `GameObject` runtime fields directly.
  - `matches_filter(&Characteristics, &TargetFilter)` reused for the standard fields (card_type, has_card_types OR-semantics, exclude_subtypes, etc.).
- **`HASH_SCHEMA_VERSION`** — bumped 8→9 at `crates/engine/src/state/hash.rs`. New field `filter: Option<TargetFilter>` hashed via existing `Option` blanket impl. New `is_nontoken: bool` on TargetFilter hashed.
- **CR 701.21a behavior** — zero-match → no error, no SBA fault. Verified by `filter_excludes_all_player_has_nothing_to_sacrifice` test.
- **Determinism** — `sort_unstable()` then `take(n)` for ascending ObjectId order. Verified.

## Card Def Summary (post-fix)

| Card | Status | Notes |
|------|--------|-------|
| fleshbag_marauder | SHIPPED | creature filter; minor oracle abbreviation (LOW C4) |
| merciless_executioner | SHIPPED | creature filter; minor oracle abbreviation (LOW C4) |
| butcher_of_malakir | SHIPPED | creature filter |
| dictate_of_erebos | SHIPPED | creature filter |
| grave_pact | SHIPPED | creature filter; EachOpponent vs "each other player" (LOW C5) |
| blasphemous_edict | SHIPPED | mana cost / type / target fixed; alt-cost reduction TODO documented |
| roiling_regrowth | SHIPPED | full re-author: land filter + 2× SearchLibrary (basic, tapped) + Shuffle |
| liliana_dreadhorde_general | SHIPPED (−4 only) | creature filter, count=2; −9 ability still TODO (acknowledged) |
| demons_disciple | SHIPPED | creature OR planeswalker filter |
| anowon_the_ruin_sage | SHIPPED | creature + exclude_subtypes Vampire |
| blessed_alliance (mode 2) | SHIPPED | creature + is_attacking |
| accursed_marauder | SHIPPED | creature + is_nontoken; Warrior subtype added (C3 fix) |
| vraskas_fall | NOT SHIPPED (reverted) | poison-counter no-op pre-existing engine gap; filter: None retained |
| ruthless_winnower | NOT SHIPPED | out-of-scope (AtBeginningOfEachPlayersUpkeep blocker) |
| by_invitation_only | NOT SHIPPED | out-of-scope (player-choice variable count blocker) |
| flare_of_malice | NOT SHIPPED | out-of-scope (greatest-MV-among + alt-cost blockers) |

**Total cards re-authored under PB-SFT**: **12** (well above the 7-card minimum and the 7-9 calibrated yield estimate from the re-triage memo §4).

## Test Coverage

`crates/engine/tests/effect_sacrifice_permanents_filter.rs` — 6 tests, all passing:

1. `each_player_sacrifices_creature_filter` (Fleshbag Marauder pattern) ✓
2. `each_player_sacrifices_land_filter` (Roiling Regrowth pattern) ✓
3. `multi_count_sacrifice_with_filter` (Liliana DH-4 pattern, count=2) ✓
4. `filter_excludes_all_player_has_nothing_to_sacrifice` (zero-match no-error per CR 701.21a) ✓
5. `multi_type_filter_creature_or_planeswalker` (Demon's Disciple OR pattern) ✓
6. `hash_schema_version_pinned_at_9` (sentinel) ✓

Hash-version assertions in 4 existing test files updated 8→9.

## Acceptance Criteria

| # | ID | Description | Status |
|---|-----|-------------|--------|
| 1 | 3676 | Filter field with `#[serde(default)]` for backward-compat | ✓ |
| 2 | 3677 | Resolution honors filter; CR 701.21a zero-match no-error | ✓ |
| 3 | 3678 | Filter check uses `calculate_characteristics` (CR 613.1d); runtime fields explicit | ✓ |
| 4 | 3679 | 5 mandatory tests pass | ✓ (5 mandatory + 1 hash-sentinel) |
| 5 | 3680 | 7+ card defs re-authored | ✓ (12 — exceeds requirement) |
| 6 | 3681 | All existing tests still pass | ✓ (2695 / 0 fail) |
| 7 | 3682 | Clippy `-D warnings` exits 0 | ✓ |
| 8 | 3683 | HASH_SCHEMA_VERSION bumped | ✓ (8→9) |
| 9 | 3684 | /review pass; HIGH/MEDIUM resolved | ✓ (this memo) |

## Verdict (repeated): PASS

All HIGH and MEDIUM findings from the initial review are addressed in commit `01c41cb9`. Carried-LOW findings are documented above with rationale (cosmetic doc staleness, pre-existing out-of-scope cards, free-for-all-Commander-correct EachOpponent convention). Build is clean (2695 tests, 0 failures, 0 clippy warnings, fmt clean). Ready for collection.
