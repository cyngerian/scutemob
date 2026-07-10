# Primitive Batch Review: PB-AC9 — Misc & mana (final AC-chain batch)

**Date**: 2026-07-10
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 121.1 (draw), 402.2 (max hand size), 614.1 (replacement), 701.9 (discard),
701.24 (shuffle), 701.47a (amass), 706.2/706.3b (dice + results table)
**Engine files reviewed**: `effects/mod.rs` (WheelHand dispatch, `move_zone_all_then_shuffle`,
RollDice, Investigate/Amass token-doubling), `state/hash.rs`, `state/player.rs`,
`state/builder.rs`, `rules/turn_actions.rs` (cleanup recompute), `rules/resolution.rs`
(8 token-doubling sites), `rules/replacement.rs` (`apply_token_creation_replacement`,
placeholder binding), `cards/helpers.rs`
**Card defs reviewed**: 11 — `incendiary_command`, `shattered_perception`, `winds_of_change`,
`echo_of_eons`, `reforge_the_soul`, `ancient_silver_dragon`, `ancient_copper_dragon`,
`ancient_gold_dragon`, `parallel_lives`, `anointed_procession`, `doubling_season`
**Test file reviewed**: `tests/pb_ac9_wheel_and_misc.rs` (18 tests) + 7 completeness regression
tests in existing keyword test files (per wip note; not re-read here — see LOW E2)

## Verdict: needs-fix

The two new primitives (`Effect::WheelHand`, `Effect::SetNoMaximumHandSize`) are correct,
fully dispatch-wired, hashed, and honor draw/discard replacement effects. The token-doubling
completeness pass is genuinely complete: I independently re-derived the 13 `GameEvent::TokenCreated`
emission sites and confirmed every one is now preceded 1:1 by `apply_token_creation_replacement()`,
with Gift correctly keyed on the *recipient* and Myriad/Squad/Offspring/Embalm/Eternalize/Encore
on the token controller. All 11 card defs match oracle text exactly (MCP-verified). No HIGH
findings. One MEDIUM (a pre-existing counter-doubling gap that the Amass half of the completeness
pass left asymmetric) and three LOW findings (one of them a favorable documentation/accounting
mismatch on Reforge the Soul, which is actually *more* complete than the plan/wip claim).

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| E1 | **MEDIUM** | `effects/mod.rs:2543-2558` | **Amass counter placement bypasses counter-doubling.** AC9 wired the token-doubling half of `Effect::Amass` but the counter half still does a direct `obj.counters.insert(cur + n)`. Under Doubling Season / Corpsejack / Vorinclex, amass places N +1/+1 counters instead of 2N. Reachable (Dreadhorde Invasion + Doubling Season). **Fix:** route the amass counter placement through `apply_counter_replacement(state, controller, army_id, &PlusOnePlusOne, n)` like `Effect::AddCounter` does. |
| E2 | LOW | `tests/pb_ac9_wheel_and_misc.rs` | **No direct hash-discrimination test for `WheelDisposal`/`WheelDraw` payloads.** The mutation test covers only `PlayerState.no_max_hand_size_permanent`. **Fix:** add an assert that two `Effect::WheelHand` values differing only in disposal (Discard vs ShuffleHandIntoLibrary) or draw (ThatMany vs Fixed) hash differently. Optional. |
| E3 | LOW | `state/mod.rs:299-302` | **`next_object_id()` shares `timestamp_counter` with RNG seeding** (runner flag #1). Adjudicated as a test-authoring hazard, NOT a latent engine bug (see adjudication below). **Fix:** document in `memory/gotchas-infra.md`. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| C1 | LOW | `reforge_the_soul.rs` | **Documentation/accounting mismatch (favorable).** Plan §4.1/§8 and `primitive-wip.md` describe Reforge as remaining PARTIAL, "blocked only on KeywordAbility::Miracle," retaining 1 Miracle TODO. The committed def actually **fully implements Miracle** (`Keyword(Miracle)` + `Miracle { cost {1}{R} }`, matching the Terminus pattern) with **no TODO** — it is clean. The plan's premise that Miracle was unimplemented was a 4th stale assumption. **Fix:** correct the wip close-out note (Reforge is clean, not partial) and confirm `authoring-report.py` counted it clean (delta is likely +10 clean, not "+9 clean / +1 partial"). |

### Finding Details

#### Finding E1: Amass counters not doubled under a counter-doubler

**Severity**: MEDIUM
**File**: `effects/mod.rs:2543-2558`
**CR**: 701.47a — "Choose an Army creature you control. Put N +1/+1 counters on that
creature." + 614.1 — Doubling Season doubles counters put on a permanent you control.
**Issue**: The Amass effect now wraps its token creation with
`apply_token_creation_replacement` (correct — doubles the 0/0 Army to two tokens), but the
subsequent counter placement writes counters directly into `obj.counters` without consulting
`apply_counter_replacement`. The normal counter path (`Effect::AddCounter`,
`effects/mod.rs:2275/2310/3432`) does route through that boundary. Result under Doubling
Season: amass produces an Army with N counters instead of 2N (the extra bare 0/0 dies to SBA,
so final state is one N/N Army where CR requires 2N/2N). This asymmetry is *new-ish*: before
AC9 amass doubled neither tokens nor counters; AC9 fixed the token half only. The worker's own
deviation note #2 flagged uncertainty here but framed it as "which token gets the counters"
rather than "counters aren't doubled."
**Fix**: replace the direct `counters.insert` at 2543-2558 with a call to
`apply_counter_replacement(state, controller, army_id, &CounterType::PlusOnePlusOne, n)` and
place the returned (possibly doubled) count, extending `events` with its replacement events.

#### Finding C1: Reforge the Soul is complete, docs say partial

**Severity**: LOW
**File**: `reforge_the_soul.rs`
**Oracle**: "Each player discards their hand, then draws seven cards. Miracle {1}{R}"
**Issue**: The def implements the wheel (`WheelHand { EachPlayer, Discard, Fixed(7) }`) AND
Miracle ({1}{R}) exactly matching the established `terminus.rs` pattern. Miracle is a working
engine feature (`rules/miracle.rs`, `AltCostKind::Miracle`, `AbilityDefinition::Miracle`,
casting.rs handling). The card is oracle-accurate and clean. Only the batch documentation is
wrong. No game-state impact.
**Fix**: correct `primitive-wip.md` "Deviations" and the yield accounting; re-run/verify
`authoring-report.py` classification for Reforge.

## Runner Self-Report Adjudications

- **Flag #1 — `next_object_id()` shares `timestamp_counter`**: CONFIRMED true
  (`state/mod.rs:300`: `self.timestamp_counter += 1; ObjectId(self.timestamp_counter)`).
  Adjudication: **test-authoring hazard, not a latent engine bug.** In production
  `timestamp_counter` is monotonically increasing and is never reset below existing object IDs;
  RNG seeding (`RollDice`, `move_zone_all_then_shuffle`) reads the counter then `+= 1`, which
  only *advances* it, preserving ID uniqueness. Collision arises only when a test manually
  forces `timestamp_counter` below the post-build object count. The worker's mitigation in
  `test_ancient_silver_dragon_draw_and_no_max` (force 1009, chosen so `1009 % 20 == 9`) is
  correct. Route to `memory/gotchas-infra.md` (LOW E3).
- **Flag #2 — plan mislabeled a site "Populate"; no Populate mechanic exists**: CONFIRMED.
  Grep for `Populate|populate` across the engine returns only incidental English in
  doc-comments ("Populates PlayerState…", "Populated by flush_pending_triggers"); there is no
  `Effect::Populate` / Populate keyword. The site the plan called "Populate (SOK)" is Squad's
  ETB trigger (`resolution.rs:4620`, `KeywordAbility::Squad`). No missing token-doubling site.
- **RollDice per-roll `+= 1` (no seed collision)**: CONFIRMED. `effects/mod.rs:3649-3651`:
  `seed = timestamp_counter; timestamp_counter += 1; result = (seed % sides) + 1`. Two rolls in
  one resolution use consecutive seeds → distinct results. No `from_entropy`/`thread_rng`
  anywhere in the engine (WheelHand shuffle uses the same seeded `StdRng` pattern; determinism
  test `test_wheel_hand_shuffle_into_library_that_many` asserts hash equality across two runs).

## Dispatch-Chain Verification (per review priority #1)

**`Effect::WheelHand`** — definition (`card_definition.rs`) → `helpers.rs` export → dispatch in
`execute_effect_inner` (`effects/mod.rs:560`, the central dispatcher reachable from Sequence /
modal / Conditional / ForEach) → `move_zone_all_then_shuffle` for shuffle dispositions →
`discard_cards` for discard (Madness→exile preserved) → `draw_one_card` for draws
(honors WouldDraw replacements via `check_would_draw_replacement`; draw-from-empty sets
`has_lost` per CR 104.3b, verified in `draw_one_card:7631-7640`) → hash arm disc 91 +
`WheelDisposal`/`WheelDraw` `HashInto` impls. Snapshot-before-disposal verified (hand_size
read at `:568` before the `match disposal`). EachPlayer / APNAP via `resolve_player_target_list`
(4p test passes). Draws honor replacements — not bypassed.

**`Effect::SetNoMaximumHandSize`** — definition → dispatch (`:601`) sets
`no_max_hand_size_permanent` → OR'd into cleanup recompute (`turn_actions.rs:1510`,
`has_no_max || ps.no_max_hand_size_permanent`) → `player.rs:327` field, `builder.rs:258`
literal, `hash.rs:1450` PlayerState hash + arm disc 92 + `HASH_SCHEMA_VERSION = 36`. The AC8
layer-correctness fix (`calculate_characteristics` scan, `turn_actions.rs:1495-1505`) is
untouched — verified NOT regressed (work item 4 clean).

**Token-doubling 13-site enumeration (independently re-derived):**

| # | Emission | Apply call | Keys on | Doubles? |
|---|----------|-----------|---------|----------|
| 1 | `effects/mod.rs:647` CreateToken | `:626` | controller | YES |
| 2 | `:697` CreateTokenAndAttachSource (Living Weapon) | `:687` | controller | YES (first equipped) |
| 3 | `:777` Investigate | `:768` (per-instance) | controller | YES |
| 4 | `:2510` Amass Army token | `:2502` | controller | YES (counters: see E1) |
| 5 | `:4804` CreateTokenCopy | `:4651` | controller | YES |
| 6 | `resolution.rs:4750` Squad | `:4632` (batch) | controller | YES |
| 7 | `:5003` Offspring | `:4862` | controller | YES |
| 8 | `:5696` Myriad (per opponent) | `:5571` | controller | YES |
| 9 | `:6380` Embalm | `:6262` | controller | YES |
| 10 | `:6605` Eternalize | `:6487` | controller | YES |
| 11 | `:6845` Encore (per opponent) | `:6706` | controller | YES |
| 12 | `:7757` Gift Food | `:7751` | **recipient** | YES (correct keying) |
| 13 | `:7786` Gift Treasure | `:7780` | **recipient** | YES (correct keying) |

Count = 13, matches. Gift recipient-keying (review priority #1) is correct. No site silently
un-doubled.

**`PlayerId(0)` placeholder binding** — `register_permanent_replacement_abilities`
(`replacement.rs:1937-1941` for `WouldCreateTokens`, `:1927-1936` for `WouldPlaceCounters`)
rebinds `bind_player_filter`/`bind_object_filter` to the real controller. Doubling Season's
counter clause `receiver_filter: ControlledBy(PlayerId(0))` → `ControlledBy(controller)`;
`placer_filter: Any` passes through. So the doublers correctly scope to their controller, not
PlayerId(0). Matches the Adrix/Vorinclex precedent.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 121.1 (draw) | Yes | Yes | `test_wheel_hand_discard_that_many` |
| 402.2 (no max hand size, cleanup) | Yes | Yes | `test_set_no_maximum_hand_size_survives_cleanup`, `_recompute_does_not_clobber` |
| 701.9 (discard whole hand) | Yes | Yes | discard/that-many + fixed tests |
| 701.24 (shuffle hand[+GY] into library) | Yes | Yes | `_shuffle_into_library_that_many`, `_shuffle_hand_and_graveyard_fixed` |
| 702.35a (Madness on wheel discard) | Yes | Yes | `test_wheel_hand_madness_routes_to_exile` |
| 706.2/706.3b (d20 results table) | Yes (pre-existing) | Yes | Copper/Gold/Silver dragon integration tests |
| 701.47a (amass) | Partial (counter-doubling gap E1) | No (no amass+DS test) | pre-existing effect; token half wired |
| 614.1 (token doubling, 13 sites) | Yes | Yes (per-class regression tests per wip) | Doubling Season / Parallel Lives integration |
| 104.3b (draw from empty → loss) | Yes (via `draw_one_card`) | No (wheel-specific) | not AC9-new; safe reuse |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| incendiary_command | Yes | 0 | Yes | mode 3 → WheelHand EachPlayer/Discard/ThatMany |
| shattered_perception | Yes | 0 | Yes | Controller/Discard/ThatMany + Flashback |
| winds_of_change | Yes | 0 | Yes | ShuffleHandIntoLibrary/ThatMany |
| echo_of_eons | Yes | 0 | Yes | ShuffleHandAndGraveyardIntoLibrary/Fixed(7) + Flashback |
| reforge_the_soul | Yes | **0 (not 1)** | Yes | **Fully clean** incl. Miracle — see C1 (docs say partial) |
| ancient_silver_dragon | Yes | 0 | Yes | 8/8; Sequence[RollDice, SetNoMaximumHandSize] |
| ancient_copper_dragon | Yes | 0 | Yes | 6/5; RollDice → Treasure count; doubles via CreateToken |
| ancient_gold_dragon | Yes | 0 | Yes | 7/10; RollDice → 1/1 blue Faerie Dragon flyers |
| parallel_lives | Yes | 0 | Yes | WouldCreateTokens/DoubleTokens |
| anointed_procession | Yes | 0 | Yes | identical to Parallel Lives |
| doubling_season | Yes | 0 | Yes | token + counter clauses both register/bind correctly |

## Hash / Determinism (acceptance criterion 4424)

- `HASH_SCHEMA_VERSION = 36` (`hash.rs:312`), changelog block present.
- Effect arms disc 91 (WheelHand: player+disposal+draw) / 92 (SetNoMaximumHandSize: player)
  present; `WheelDisposal` (0/1/2) and `WheelDraw` (ThatMany=0, Fixed=1+k) `HashInto` impls present.
- `PlayerState.no_max_hand_size_permanent` hashed at `:1450`.
- `test_no_max_hand_size_permanent_hash_mutation` is a genuine mutation test (two states differ
  only in that field; asserts `public_state_hash` differs) — NOT a tautology. Confirms the field
  is in the public hash.
- Die-roll determinism and no-collision: verified (see adjudications). WheelHand shuffle uses the
  same seeded `StdRng::seed_from_u64(timestamp_counter)` + `+= 1` pattern; determinism asserted
  by `test_wheel_hand_shuffle_into_library_that_many`.

## Previous Findings

N/A — first review of PB-AC9.
