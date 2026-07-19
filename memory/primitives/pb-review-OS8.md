# Primitive Batch Review: PB-OS8 — `Effect::LookAtTopThenPlace` + `TargetFilter.min_cmc_amount`

**Date**: 2026-07-19
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 120/121 (look/draw), 202.3/608.2h (runtime mana value), 118.12 (optional cost),
601.2 (choose ≤1), 401 (library order), 400.7 (new object on zone change), 603.3/603.4
(ETB triggers + intervening-if)
**Engine files reviewed**: `crates/card-types/src/cards/card_definition.rs` (Effect variant,
`TargetFilter.min_cmc_amount`, `Cost` allow), `crates/engine/src/effects/mod.rs`
(`LookAtTopThenPlace` executor ~5047, `SearchLibrary` min-cap ~2957, `zone_move_event`),
`crates/engine/src/state/hash.rs` (Effect arm disc 96, TargetFilter field), `rules/protocol.rs`
(PROTOCOL 23)
**Card defs reviewed**: `birthing_ritual.rs` (FLIP), `growing_rites_of_itlimoc.rs` (FLIP),
`birthing_pod.rs` (doc-only, stays inert), `muxus_goblin_grandee.rs` (doc-only, stays partial)
**Tests reviewed**: `crates/engine/tests/primitives/pb_os8_look_at_top_then_place.rs` (13 tests)

## Verdict: needs-fix

The engine primitive is CR-correct on every focus axis I checked: the candidate pool is scoped
strictly to the looked-at top-N subset (`object_ids().take(n)`, filtered within `top_ids` only),
so a matching card at position N+1 is structurally unreachable; the interposed `place_cost` is
paid AFTER the look and its LKI is captured into `ctx.sacrificed_creature_lki` BEFORE the runtime
`max_cmc_amount` cap is resolved (so X = 1 + sacrificed MV feeds the filter correctly); at most one
card is placed (`min_by_key(|id| id.0)`); the remainder is bottomed deterministically
(`sort_by_key(|id| id.0)`, no `rand`); the decline path (unpayable cost) correctly skips placement
and bottoms everything. `min_cmc_amount` mirrors `max_cmc_amount` symmetrically in both executors.
The hash arm covers all 7 fields; the wire bump is a single justified PROTOCOL 22→23 / HASH 59→60
(disc 96, no discriminant shift); no new hidden-info leak (events carry only ObjectIds, same posture
as `RevealAndRoute`/`Scry`). Both card flips match oracle text exactly and are `Complete`;
birthing_pod and muxus honestly stay unfinished with accurate, correctly-seeded notes. The single
reason this is not clean: the test suite cannot discriminate the shipped top-N implementation from a
whole-library-scan implementation on the primitive's *central* axis (top-N vs whole library) — every
test library holds exactly `count` cards, so the `take(n)` truncation is never behaviorally
exercised. That is the exact distinction the plan flagged as the core focus. One MEDIUM (test gap)
plus three LOWs.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `effects/mod.rs:5069` | **Empty-library skips the interposed cost, whiff does not.** With a non-empty top-N that whiffs, `place_cost` is paid (sacrifice fires); with an empty library the executor `continue`s at the `top_ids.is_empty()` guard *before* reaching the cost, so the sacrifice never fires. Both are legal lines under a "may," but the two paths are inconsistent. **Fix:** none required (leave as-is; the empty-library path is arguably more correct than auto-sacrificing into a guaranteed whiff). Document the asymmetry in the executor comment if convenient. |
| 2 | LOW | `effects/mod.rs:5054` | **`optional` field is completely ignored (`optional: _`).** No behavioral branch reads it; both shipped defs set `true`. Documented as reserved for M10 interactive decline. It is still hashed/wired (correct, forward-looking), but currently inert. **Fix:** none required this PB; keep the doc-comment note. Flag only so a future reviewer does not mistake it for a live gate. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| — | — | — | No card-def defects. All four defs verified against oracle text below. |

## Test Findings

| # | Severity | File | Description |
|---|----------|------|-------------|
| 3 | **MEDIUM** | `pb_os8_look_at_top_then_place.rs` | **No test exercises the `count` truncation — the core look-vs-search distinction.** Every test library contains exactly `count` cards (4 cards / count 4, 3 cards / count 3, etc.), so a matching card at position N+1 is never present. The suite would pass even if the executor scanned the whole library or `count+k` cards. The plan's focus item 1(a) names this "the core SearchLibrary-vs-look distinction." **Fix:** add a decoy test with `count = N` but `N + 2` library cards where a matching creature sits at index `N` (just outside the window); assert it is neither placed (not in hand/battlefield) nor bottomed (retains its original ObjectId / no `ObjectPutOnLibrary` event for it), while a matching card inside the window is placed. This makes the top-N boundary behaviorally load-bearing. |
| 4 | LOW | `pb_os8_look_at_top_then_place.rs` | **No test for `count` resolving to 0 or an empty library.** Edge paths (`take(0)` → empty `top_ids` → `continue`; empty library → `continue` without paying `place_cost`) are unverified. **Fix:** add a small test asserting an empty library / `count` = 0 places nothing, bottoms nothing, and (with `place_cost`) does NOT pay the sacrifice. |

### Finding Details

#### Finding 1: Empty-library vs whiff cost asymmetry
**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:5069`
**CR Rule**: 118.12 (optional "you may pay"); 601.2 (may decline)
**Issue**: `if top_ids.is_empty() { continue; }` runs before the `place_cost` block, so an empty
library never pays the sacrifice, whereas a non-empty-but-no-match top-N does pay it (deterministic
"pay when able", R3). Both are legal lines; the inconsistency is cosmetic.
**Fix**: No behavioral change required. Optionally note the asymmetry in the comment so the next
reader does not read it as a bug.

#### Finding 2: `optional` is dead in the executor
**Severity**: LOW
**File**: `crates/engine/src/effects/mod.rs:5054` (`optional: _`)
**Issue**: The `optional` discriminator is never read; both card defs set `true`, and the effect
always places the best candidate when one exists. This matches every other "take when able" M7
effect and is documented as reserved for M10, so it never produces an illegal state — but the field
carries no current semantics.
**Fix**: None this PB. Retain the field (it is correctly hashed and wired for M10). Flag only.

#### Finding 3: Missing top-N truncation decoy (core distinction untested)
**Severity**: MEDIUM
**File**: `crates/engine/tests/primitives/pb_os8_look_at_top_then_place.rs`
**Issue**: The primitive's entire reason to exist (vs `SearchLibrary`) is that it only reaches the
top `count` cards. No test places a matching card *below* the top-N window, so the suite cannot
distinguish the correct `take(n)` implementation from one that scans more than `n`. Structurally the
executor is correct (I read it), but the coverage does not lock the boundary.
**Fix**: Add a decoy test — library of `count + 2` cards, a matching creature at index `count`
(outside the window) plus a matching creature inside it; assert the in-window one is placed and the
out-of-window one is untouched (still in library, original ObjectId, no move event). Per the
"tests must validate what the primitive promises" discipline, treat as a fix-phase requirement.

#### Finding 4: No zero-count / empty-library edge test
**Severity**: LOW
**File**: `crates/engine/tests/primitives/pb_os8_look_at_top_then_place.rs`
**Issue**: `count` = 0 and empty-library paths are unexercised.
**Fix**: Add a compact edge test (nothing placed, nothing bottomed, cost unpaid).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 120/601.2 (look, choose ≤1) | Yes | Yes | `test_look_place_at_most_one_even_when_two_match` (decoy vs RevealAndRoute) |
| 118.12 (interposed optional cost) | Yes | Yes | `test_look_place_cost_sacrifice_gates_and_parameterizes` + `..._declined_when_unpayable_skips_placement` |
| 202.3/608.2h (runtime MV, max) | Yes | Yes | sacrifice-driven cap = 1 + sac MV; MV-5 excluded |
| 202.3/608.2h (runtime MV, min) | Yes | Yes | `test_min_cmc_amount_caps_search_by_runtime_floor` (SearchLibrary) + `..._min_and_max_equal_exact_mv` (LookAtTopThenPlace) |
| 401 (bottom rest, deterministic) | Yes | Partial | ObjectId-ascending, no rand; the *rest-order* is not asserted (acceptable — bottom is hidden) |
| 400.7 (new object on zone change) | Yes | Yes | `test_look_place_onto_battlefield_fires_etb` (new_id + ETB event) |
| 603.3/603.4 (ETB trigger + intervening-if) | Yes | Yes | `test_birthing_ritual_end_step_flip` (real end-step path) |
| top-N truncation (look vs whole-library) | Yes (structural) | **No** | Finding 3 — no card below the window is ever present |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| `birthing_ritual` | Yes | 0 | Yes | End-step trigger + intervening-if(creature≥1) + `LookAtTopThenPlace` count 7, `place_cost` Sacrifice(creature), `max_cmc_amount` = Sum(1, ManaValueOfSacrificedCreature), dest Battlefield, rest Library Bottom, optional. `Complete`. "Random order" → deterministic (accepted M7). |
| `growing_rites_of_itlimoc` | Yes | 0 | Yes | ETB `WhenEntersBattlefield` (verified self-scoped, not any-permanent) `LookAtTopThenPlace` count 4, creature→Hand, no cost, rest Bottom; end-step `TransformSelf` w/ intervening-if(creature≥4); both back-face mana abilities (`{T}: {G}` + `{T}: {G} per creature`) + Keyword Transform preserved. `Complete`. |
| `birthing_pod` | Yes (unimplemented) | 1 (OOS-OS8-1) | N/A (inert) | Honestly `inert`; note correctly states the MV-filter blocker is CLOSED by `min_cmc_amount` but the {1}{G/P} Phyrexian-in-activated-cost blocker remains (verified: abilities.rs has 0 Phyrexian; 2-life path is casting.rs-only). Correct deferral. |
| `muxus_goblin_grandee` | Yes (attack half) | 1 (OOS-OS8-2) | Yes for authored half; ETB deferred | Honestly `partial`. Note correctly re-points the ETB ("put ALL Goblins MV≤5") to the already-shipped `Effect::RevealAndRoute` (put-multiple), NOT `LookAtTopThenPlace`. Correct not to author here. |

## Independent Verification Notes

- **Candidate-pool scoping (focus 1a)**: `top_ids` = `zone.object_ids().take(n)`; the placement
  `filter().min_by_key()` iterates `top_ids` only. A match at N+1 is not in `top_ids` → unreachable.
  Structurally correct; see Finding 3 for the test-coverage gap.
- **LKI ordering (focus 1b)**: `try_pay_optional_cost` runs, sets `ctx.sacrificed_creature_lki`,
  THEN `max_cap = resolve_amount(filter.max_cmc_amount, ctx)` — so the cap sees the sacrificed MV.
  Confirmed the cost precedes cap resolution in source order (5075→5092).
- **≤1 cardinality (focus 1c)**: single `min_by_key` winner; the decoy test proves the second match
  is bottomed, not placed.
- **Deterministic remainder (focus 1d)**: `rest_ids.sort_by_key(|id| id.0)`; no `rand`/RNG import
  in the arm. Matches RevealAndRoute/Scry/PutOnLibrary M7 precedent.
- **Hidden info (focus 3)**: the executor emits no distinct "look"/reveal event; only
  `zone_move_event` (ObjectId-only payloads — `ObjectPutOnLibrary`, `PermanentEnteredBattlefield`,
  `ObjectReturnedToHand`). Identical leak posture to `RevealAndRoute`; `private_to` genuinely does
  not exist (deferred M10). No NEW leak.
- **Wire integrity (focus 5)**: `PROTOCOL_VERSION = 23`, `HASH_SCHEMA_VERSION = 60u8`; grep for
  stale `PROTOCOL_VERSION, 22` / `HASH_SCHEMA_VERSION, 59u8` sentinels returned zero hits. Effect
  disc 96 (95 = RemoveFromCombat, no shift). Hash arm hashes player/count/filter/place_cost/
  destination/rest_to/optional (all 7). `TargetFilter` HashInto adds `min_cmc_amount` right after
  `max_cmc_amount`. The `#[allow(clippy::large_enum_variant)]` on `Cost` is justified (boxing
  `Cost::Sacrifice(TargetFilter)` would touch ~84 call sites; matches existing precedent) — it
  masks a size lint, not a correctness problem. Ratchet bump 109→110 is one NONSWALLOW predicate
  read, an exact copy of RevealAndRoute's idiom.
- **`min_cmc_amount` mirror (focus 2)**: SearchLibrary (`runtime_min_cap`, 2960) and
  LookAtTopThenPlace (`min_cap`, 5096) both apply `>= cap` symmetric with `<= cap`. Boundary
  behavior verified by `test_look_place_min_and_max_equal_exact_mv` (MV-2/MV-4 excluded, MV-3
  placed).

## Fix Directives Summary (for fix phase)

1. **MEDIUM (Finding 3)** — add a top-N truncation decoy: library of `count + 2`, a matching
   creature just outside the window must be untouched (not placed, not bottomed) while an in-window
   match is placed. Locks the primitive's core distinction from `SearchLibrary`.
2. **LOW (Finding 4)** — add a zero-count / empty-library edge test (nothing placed/bottomed,
   `place_cost` unpaid).
3. **LOW (Findings 1, 2)** — optional documentation-only clarifications; no code change required.
