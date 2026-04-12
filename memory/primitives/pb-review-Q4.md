# Primitive Batch Review: PB-Q4 — EnchantTarget::Filtered (bundled enchant target variants)

**Date**: 2026-04-12
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 303.4a, 702.5a, 704.5m, 205.3i, 205.4a
**Engine files reviewed**:
- `crates/engine/src/state/types.rs` (EnchantFilter, EnchantControllerConstraint, EnchantTarget::Filtered)
- `crates/engine/src/state/hash.rs` (HashInto for EnchantFilter, EnchantControllerConstraint, TargetFilter)
- `crates/engine/src/cards/card_definition.rs` (TargetFilter::nonbasic)
- `crates/engine/src/rules/sba.rs` (matches_enchant_target, enchant_filter_matches, check_aura_sbas)
- `crates/engine/src/rules/casting.rs` (Aura cast-time enforcement)
- `crates/engine/src/effects/mod.rs` (matches_filter `nonbasic` arm)
- `crates/engine/src/state/mod.rs`, `cards/helpers.rs`, `lib.rs` (re-exports)

**Card defs reviewed (4)**:
- `crates/engine/src/cards/defs/awaken_the_ancient.rs`
- `crates/engine/src/cards/defs/chained_to_the_rocks.rs`
- `crates/engine/src/cards/defs/ossification.rs`
- `crates/engine/src/cards/defs/dimensional_exile.rs`

**Test file**: `crates/engine/tests/enchant.rs` (12 mandatory tests added)

## Verdict: needs-fix (LOW only — no HIGH, no MEDIUM showstoppers; 1 MEDIUM architectural duplication)

The implementation is correct against CR 303.4a / 702.5a / 704.5m for the four bundled
land-variant enchant patterns. All 12 mandatory tests are present and assert the right
things. All 4 card defs match oracle text. Hash schema parity is achieved for both
EnchantFilter and TargetFilter (with the new `nonbasic` field hashed). The full dispatch
chain (cast-time → SBA → matches_filter for `nonbasic`) is wired and consistent. The
runner's deviation from `Box<TargetFilter>` to a new `EnchantFilter` struct is technically
justified by a real circular import — but architectural duplication is a
forward-looking risk and the cleaner alternative (move TargetFilter to `state/`) was not
explored. Counted as 1 MEDIUM.

## Counts: HIGH 0 / MEDIUM 1 / LOW 3

---

## Focus 1: EnchantFilter ↔ TargetFilter field parity

**Status**: VERIFIED for current PB-Q4 scope; MEDIUM for future scope.

`TargetFilter` has 24 fields. `EnchantFilter` has 6 fields. Walking each TargetFilter
field for "is this meaningful for an Enchant restriction":

| TargetFilter field | Meaningful for Enchant? | EnchantFilter equivalent? | Notes |
|---|---|---|---|
| max_power / min_power | Yes (e.g. "Enchant creature with power 4 or greater") | NO | Future card gap |
| has_card_type | Yes | YES | |
| has_keywords | Yes ("Enchant creature with flying") | NO | Future card gap |
| colors | Yes ("Enchant red creature") | NO | Future card gap |
| exclude_colors | Yes ("Enchant nonblack creature") | NO | Future card gap |
| non_creature | No (Enchant variants don't say "noncreature permanent") | NO | OK |
| non_land | Marginal | NO | OK |
| basic | YES | YES | |
| nonbasic | YES | YES | |
| controller | YES | YES (EnchantControllerConstraint) | |
| has_subtype | YES | YES | |
| has_subtypes | YES | YES | |
| has_name | No | NO | OK |
| max_cmc / min_cmc | Marginal | NO | Future gap |
| has_card_types (Vec OR) | Marginal — flat `CreatureOrPlaneswalker` already covers main case | NO | OK for now |
| legendary | YES ("Enchant legendary creature") | NO | Future card gap |
| is_token | No | NO | OK |
| max_toughness | Marginal | NO | Future gap |
| exclude_subtypes | Marginal | NO | Future gap |
| is_attacking | No | NO | OK |
| has_chosen_subtype | Marginal | NO | Future gap |
| exclude_chosen_subtype | Marginal | NO | OK |

**For PB-Q4's stated scope (4 land-variant patterns + Forest/Plains disjunction)**, all
required fields exist. **No HIGH finding.**

**Forward-looking concern (counted as MEDIUM under Focus 4)**: when later cards need
"Enchant legendary creature", "Enchant creature with power 4 or greater", etc., either
EnchantFilter must grow toward TargetFilter's shape or developers will reach for a
parallel solution. The two structs will diverge over time. See Focus 4 for the
architectural finding.

---

## Focus 2: Hash schema completeness

**Status**: VERIFIED.

- `EnchantFilter` has 6 fields (`has_card_type`, `has_subtype`, `has_subtypes`, `basic`, `nonbasic`, `controller`).
- `HashInto for EnchantFilter` (`state/hash.rs:280-289`) hashes all 6. Parity confirmed.
- `EnchantControllerConstraint` has 3 variants (Any, You, Opponent); HashInto covers all 3 (`state/hash.rs:290-298`).
- `TargetFilter` has 24 fields. `HashInto for TargetFilter` (`state/hash.rs:4123-4150`) hashes 24 fields, including the new `nonbasic` (line 4134). Parity confirmed.
- `EnchantTarget::Filtered(f)` arm at `state/hash.rs:273-276` hashes discriminant 8u8 + filter contents. Correct.

No silent state-corruption bug. No finding.

---

## Focus 3: `nonbasic: bool` dispatch coverage

**Status**: VERIFIED.

`grep filter\.basic\b` shows exactly one site: `effects/mod.rs:6513`. The new `nonbasic`
check is added immediately after at line 6521 with the symmetric "if `nonbasic && Basic
present → false`" semantics. CR 205.4a comment present.

`enchant_filter_matches` in `sba.rs:951-996` independently checks both `basic` and
`nonbasic` against `chars.supertypes.contains(&SuperType::Basic)` (lines 974, 978).

Both dispatch sites — `matches_filter` (for `TargetFilter` use in effect/target
contexts) and `enchant_filter_matches` (for the new `EnchantFilter`) — handle
`nonbasic` correctly. No dead-code field. No finding.

---

## Focus 4: Circular dependency claim

**Status**: VERIFIED (the cycle is real); MEDIUM finding for unexplored alternative.

Verification:
- `crates/engine/src/cards/card_definition.rs:9-13` imports from `crate::state::*` (5 separate use-statements). The `cards` module depends on `state`.
- `crates/engine/src/state/types.rs` does NOT import from `cards::*` (grep returns no matches). The `state` module does NOT depend on `cards`.
- Adding `Filtered(TargetFilter)` to `EnchantTarget` in `state/types.rs` would have required `state/types.rs` to `use crate::cards::card_definition::TargetFilter`, creating the cycle. **Real, not hypothetical.**

Boxing alone (`Box<TargetFilter>`) does not break the cycle — the type is still
referenced. The runner's deviation rationale is correct.

**MEDIUM finding (M1)**: The runner did not explore moving `TargetFilter` to `state/`
or to a new lower-level module. `TargetFilter` is a pure data struct that depends on
`Color`, `CardType`, `SubType`, `KeywordAbility`, `TargetController` — all of which
already live in `state::types`. Moving `TargetFilter` (and `TargetController`,
`TargetRequirement`) into `state/` would have unblocked the plan's preferred shape
without introducing a parallel struct. Blast radius: ~50-80 import sites across
`cards/defs/*.rs` would have to update `use crate::cards::card_definition::TargetFilter`
to `use crate::state::types::TargetFilter`. Mechanical via sed.

**Why MEDIUM not HIGH**: PB-Q4's 5 cards work correctly today. The duplication risk is
forward-looking (next 5-10 enchant-target cards may force EnchantFilter to grow new
fields paralleling TargetFilter, with no compiler enforcement that the two stay
in-sync). **Fix**: Either (a) move `TargetFilter` to `state/types.rs` in a follow-up
mechanical refactor and replace `EnchantFilter` with `Filtered(Box<TargetFilter>)`,
OR (b) document explicitly in `EnchantFilter`'s doc comment that it is intentionally a
separate type and list which TargetFilter fields are NOT supported, with a TODO marker
to revisit when adding the next non-land enchant target.

---

## Standard checklist findings

### Engine Change Findings

| # | Severity | File:Line | Description |
|---|---|---|---|
| M1 | MEDIUM | `state/types.rs:286` | **EnchantFilter duplicates TargetFilter shape**. See Focus 4. **Fix**: document deliberate duplication with non-supported-fields list, or follow up by moving TargetFilter into `state/types.rs` and collapsing EnchantFilter into `Filtered(Box<TargetFilter>)`. |
| L1 | LOW | `rules/sba.rs:1067-1071` | **target_controller fallback to aura_ctrl on missing object**. If `state.objects.get(&target_id)` is None we use `aura_ctrl` as the target's controller, which makes a `controller: You` filter trivially "match" instead of failing. The surrounding code already filters out gone targets at line 1042-1047 so the branch is unreachable in practice, but a defensive `.unwrap_or(aura_ctrl)` masks future regressions. **Fix**: change to `.unwrap_or_else(\|\| { return true; })` or panic with a debug_assert; document why the fallback is safe today. |
| L2 | LOW | `tests/enchant.rs:1497` | **test_animate_land_pt_and_types_via_chained_or_awaken does not assert color**. Plan does not require it, but the test reproduces Awaken the Ancient and skips registering the SetColors layer 5 effect, so a future regression that breaks Layer 5 dispatch on `AttachedLand` would not be caught here. **Fix**: add the SetColors effect and `assert!(chars.colors.contains(&Color::Red))`. |
| L3 | LOW | `tests/enchant.rs:1634` | **test_animate_land_summoning_sickness_propagation only checks Haste keyword presence**. Does not assert that the actual `can_attack` / sickness override path returns true. The plan said "verify Haste prevents summoning sickness from blocking tap-to-attack" — the test only verifies the prerequisite (Haste is in keywords), not the conclusion (the creature can attack). **Fix**: add `let can_attack = check_can_attack(&state, mountain_id); assert!(can_attack);` or call the actual sickness predicate. |

### Card Definition Findings

None. All 4 card defs match oracle text and use the new primitive correctly. See per-card
table below.

---

## Test-by-test mandatory checklist (12 of 12)

| # | Test name | File | Line | Plan match | Asserts what plan said? |
|---|---|---|---|---|---|
| 1 | test_enchant_filtered_land_subtype_cast_time_legal | enchant.rs | 776 | YES | YES — cast-time legal Mountain target |
| 2 | test_enchant_filtered_land_subtype_cast_time_illegal | enchant.rs | 832 | YES | YES — InvalidTarget on Forest |
| 3 | test_enchant_filtered_controller_cast_time_legal | enchant.rs | 887 | YES | YES — Chained-style "you control" |
| 4 | test_enchant_filtered_controller_cast_time_illegal | enchant.rs | 944 | YES | YES — InvalidTarget on opponent's Mountain |
| 5 | test_enchant_filtered_basic_land_legal | enchant.rs | 1000 | YES | YES — basic Plains target |
| 6 | test_enchant_filtered_basic_land_illegal_nonbasic | enchant.rs | 1058 | YES | YES — non-basic dual rejected by `basic: true` |
| 7 | test_enchant_filtered_sba_control_change | enchant.rs | 1120 | YES | YES — control change → SBA falls off (CR 704.5m) |
| 8 | test_enchant_filtered_sba_land_becomes_nonland | enchant.rs | 1185 | YES | YES — Land removed → SBA falls off |
| 9 | test_enchant_filtered_disjunction_forest_or_plains | enchant.rs | 1249 | YES | YES — OR semantics validated 3 ways |
| 10 | test_enchant_filtered_nonbasic_land | enchant.rs | 1393 | YES | YES — `nonbasic: true` field exercised |
| 11 | test_animate_land_pt_and_types_via_chained_or_awaken | enchant.rs | 1497 | YES (mostly) | YES — see L2 (color assertion gap) |
| 12 | test_animate_land_summoning_sickness_propagation | enchant.rs | 1634 | YES (partial) | PARTIAL — see L3 (only checks Haste keyword presence) |

All 12 mandatory tests present, named correctly, in the expected file. Two are
under-asserting (tests 11 and 12) — flagged as LOW per the PB-Q test-skipping retro
calibration.

---

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---|---|---|---|
| 303.4a (Aura cast must target legal object per Enchant) | YES | YES | Tests 1-6, 9-10 |
| 702.5a (Enchant restricts cast and attachment) | YES | YES | Tests 1-10 |
| 704.5m (Aura SBA — illegal attachment → graveyard) | YES | YES | Tests 7-8 |
| 205.4a (Basic supertype distinction) | YES | YES | Tests 5, 6, 10 |
| 205.3i (Land subtypes; land remains land when animated) | YES | YES | Test 11 (asserts Land type preserved) |
| 613.1d (Layer-resolved characteristics for SBA) | YES | YES (indirect) | Test 8 uses base-characteristic mutation; check_aura_sbas calls `calculate_characteristics` |

---

## Card Def Summary (oracle vs DSL)

| Card | Oracle Match | TODOs | Game State Correct | Notes |
|---|---|---|---|---|
| Awaken the Ancient | YES | 0 | YES | 5-effect layered animation matches "7/7 red Giant creature with haste; still a land". Filter `Land + Mountain subtype`. Layers 4 (types), 4 (subtypes), 5 (color), 7b (P/T), 6 (keywords). |
| Chained to the Rocks | YES | 0 | YES | Filter `Land + Mountain + controller You`. ETB `ExileWithDelayedReturn` targets `creature, controller Opponent`. CR 702.5a + 610.3 cited. |
| Ossification | YES | 0 | YES | Filter `Land + basic + controller You`. ETB exiles `[Creature, Planeswalker]` via `has_card_types` Vec OR (`controller Opponent`). |
| Dimensional Exile | YES | 0 | YES | Same as Ossification but `has_card_type: Creature` only (matches oracle). |

---

## PB-Q baseline preservation

`apply_mana_production_replacements` only appears in `crates/engine/src/rules/mana.rs`.
The runner certified via `git diff 464d9e79..HEAD` that it is untouched. Trusted.
No PB-Q regression.

---

## Build/test/lint status (runner-reported, not re-run)

- `cargo build --workspace`: CLEAN
- `cargo test --all`: 2637 passed (was 2625, +12), 0 failed
- `cargo clippy --workspace -- -D warnings`: CLEAN
- `cargo fmt --check`: CLEAN
- TUI + replay-viewer compile clean (saved by `KeywordAbility::Enchant(_)` wildcard arms — runner verified)

---

## Previous Findings (re-review only)

N/A — first review of PB-Q4.
