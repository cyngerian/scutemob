# Primitive Batch Review: PB-B -- Play from GY/Exile

**Date**: 2026-04-07
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 601.2 (casting from zones), CR 601.3 (permission to cast), CR 305.1 (playing lands)
**Engine files reviewed**: `state/stubs.rs`, `state/mod.rs`, `state/builder.rs`, `state/player.rs`, `state/hash.rs`, `cards/card_definition.rs`, `cards/helpers.rs`, `effects/mod.rs`, `rules/casting.rs`, `rules/lands.rs`, `rules/turn_actions.rs`, `rules/replacement.rs`, `rules/combat.rs`
**Card defs reviewed**: 6 (ancient_greenwarden, perennial_behemoth, wrenn_and_realmbreaker, oathsworn_vampire, squee_dubious_monarch, brokkos_apex_of_forever) + 7 existing CreateEmblem defs updated

## Verdict: needs-fix

Two MEDIUM findings and two LOW findings. The primary issues are: (1) Squee's `alt_mana_cost` is never applied during cost determination -- the card is charged its normal mana cost {2}{R} instead of {3}{R} from the graveyard; (2) Squee's `ExileOtherGraveyardCards(4)` additional cost is validated for feasibility at cast time but the 4 cards are never actually exiled during cost payment (unlike Escape which performs the exile). Two LOWs are a stale TODO comment in Brokkos and a pre-existing Wrenn +1 duration bug.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `casting.rs:2455` | **CastSelfFromGraveyard alt_mana_cost never applied.** The `_alt_mana_cost` field is destructured with an underscore prefix at line 701 and never used in the base cost chain (lines 2279-2457). Squee is charged {2}{R} (normal cost) instead of {3}{R}. **Fix:** Add a branch in the base cost chain for `has_cast_self_from_graveyard` that reads the `alt_mana_cost` from the ability; if `Some`, use it; if `None`, fall through to `base_mana_cost`. |
| 2 | **MEDIUM** | `casting.rs:722-748` | **ExileOtherGraveyardCards cost never executed.** Feasibility is validated (enough cards exist) but the 4 graveyard cards are never actually exiled as part of cost payment (CR 601.2h). Escape's exile cost IS executed (line 3563-3569 via `apply_escape_exile_cost`). **Fix:** Add exile execution for `ExileOtherGraveyardCards(n)` after the cast succeeds validation, parallel to the Escape exile block. Select N other graveyard cards (deterministic: min ObjectId, matching the engine's existing pattern) and move them to exile. |
| 3 | LOW | `brokkos_apex_of_forever.rs:7-10` | **Stale TODO comment.** Lines 7-10 say "This ability is omitted" but it IS implemented at line 52. **Fix:** Remove the stale TODO comment block (lines 7-10). |
| 4 | LOW | `wrenn_and_realmbreaker.rs:42,53,65,82` | **Pre-existing: +1 uses UntilEndOfTurn instead of UntilYourNextTurn.** Oracle says "until your next turn" but all four continuous effects use `EffectDuration::UntilEndOfTurn`. `UntilYourNextTurn` exists in the engine. Not introduced by PB-B but the file was modified. **Fix:** Change all four `EffectDuration::UntilEndOfTurn` to `EffectDuration::UntilYourNextTurn(controller)` for the +1 loyalty ability effects (lines 42, 53, 65, 82). |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 5 | LOW | `brokkos_apex_of_forever.rs:41-46` | **Pre-existing: Mutate cost {3}{U/B}{G} instead of {2}{U/B}{G}{G}.** Oracle mutate cost is `{2}{U/B}{G}{G}` (generic: 2, green: 2, hybrid: U/B). Card def has `generic: 3, green: 1` -- total mana is same but color distribution is wrong (accepts 1G instead of requiring 2G). Not introduced by PB-B. **Fix:** Change `generic: 3, green: 1` to `generic: 2, green: 2` in the MutateCost ManaCost. |

### Finding Details

#### Finding 1: CastSelfFromGraveyard alt_mana_cost never applied

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:701,2279-2457`
**CR Rule**: CR 601.2f -- "The player determines the total cost of the spell. Usually this is just the mana cost. Some spells have additional or alternative costs."
**Oracle**: Squee: "by paying {3}{R} ... rather than paying its mana cost"
**Issue**: The `alt_mana_cost` field from `CastSelfFromGraveyard` is destructured with an underscore prefix (`_alt_mana_cost`) at line 701 and is never read. The base cost determination chain (lines 2279-2457) has no branch for `has_cast_self_from_graveyard`. When Squee is cast from the graveyard, the engine charges the card's normal mana cost {2}{R} instead of the ability's alt cost {3}{R}. The test at line 652-656 passes because it provides enough mana for either cost (4 total >= 3 needed for normal cost).
**Fix**: Add a branch in the base cost chain (around line 2455, before the `else { base_mana_cost }` fallback) that checks `has_cast_self_from_graveyard` and reads the `alt_mana_cost` from the `CastSelfFromGraveyard` ability. If `alt_mana_cost` is `Some(cost)`, use that as the base cost. If `None`, fall through to `base_mana_cost`. Thread `has_cast_self_from_graveyard` into the cost-determination section (it's already available). Also add a test that verifies Squee CANNOT be cast from GY with only {2}{R} in pool (expected cost is {3}{R}).

#### Finding 2: ExileOtherGraveyardCards cost never executed

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:722-748`
**CR Rule**: CR 601.2h -- "The player pays the total cost. First, they pay all costs that don't involve random elements or moving objects..."
**Oracle**: Squee: "exiling four other cards from your graveyard"
**Issue**: The validation at lines 722-748 checks that N other graveyard cards EXIST but never actually exiles them. The Escape cost IS executed at line 3563-3569 via `apply_escape_exile_cost`. For Squee, the 4 cards remain in the graveyard after casting, which means they could be exiled again for a second Squee cast (if Squee returns to GY), effectively making the cost free.
**Fix**: After the validation block, add an exile execution block parallel to the Escape pattern. When `has_cast_self_from_graveyard` is true and `additional_costs` contains `ExileOtherGraveyardCards(n)`, select N other graveyard cards (deterministic: lowest ObjectId, matching engine convention) and move them to exile zone. Emit `GameEvent::ObjectExiled` for each. Add a test that verifies the 4 filler cards are no longer in the graveyard after casting Squee.

#### Finding 3: Stale TODO in Brokkos

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/brokkos_apex_of_forever.rs:7-10`
**Issue**: Lines 7-10 contain a TODO saying "This ability is omitted" and "cast-from-zone permission system does not yet exist." The ability IS now implemented at line 52 via `CastSelfFromGraveyard`. The TODO is confusing and contradicts the actual implementation.
**Fix**: Remove lines 7-10 (the stale TODO block).

#### Finding 4: Wrenn +1 duration (pre-existing)

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/wrenn_and_realmbreaker.rs:42,53,65,82`
**Oracle**: "+1: Up to one target land you control becomes a 3/3 Elemental creature with vigilance, hexproof, and haste **until your next turn**."
**Issue**: All four continuous effects for the +1 ability use `EffectDuration::UntilEndOfTurn` instead of `EffectDuration::UntilYourNextTurn(player_id)`. This means the land reverts at end of turn instead of persisting until the controller's next turn. Pre-existing issue not introduced by PB-B.
**Fix**: Replace `EffectDuration::UntilEndOfTurn` with `EffectDuration::UntilYourNextTurn(controller)` on all four continuous effect definitions. Will need to import the player ID or use a placeholder approach consistent with other card defs using this duration.

#### Finding 5: Brokkos mutate cost mismatch (pre-existing)

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/brokkos_apex_of_forever.rs:41-46`
**Oracle**: "Mutate {2}{U/B}{G}{G}" = generic 2, hybrid U/B, green 2
**Issue**: Card def has `generic: 3, green: 1` instead of `generic: 2, green: 2`. Total mana sources identical (5) but color requirement is wrong: a player with 1G+3C+1(U or B) could cast with the card def's cost but shouldn't be able to (oracle requires 2G). Pre-existing from the Mutate batch, not introduced by PB-B.
**Fix**: Change `generic: 3` to `generic: 2` and `green: 1` to `green: 2` in the `MutateCost` ManaCost at line 42.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 601.2 (casting from zones) | Yes | Yes | Graveyard zone casting validated in casting.rs |
| CR 601.2f (cost determination) | Partial | No | alt_mana_cost not applied (Finding 1) |
| CR 601.2h (cost payment) | Partial | No | ExileOtherGraveyardCards not executed (Finding 2) |
| CR 601.3 (permission to cast) | Yes | Yes | test_play_from_graveyard_permanent_spell, Oathsworn, Squee, Brokkos tests |
| CR 305.1 (playing lands) | Yes | Yes | test_play_from_graveyard_land_basic, timing, land count tests |
| CR 118.9 (alternative costs) | Partial | No | Squee's alt cost not enforced as alternative (Finding 1) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| ancient_greenwarden | Yes (partial) | 1 (trigger doubling, deferred to PB-M) | Yes | Graveyard land play correct; trigger doubling explicitly deferred |
| perennial_behemoth | Yes | 0 | Yes | LandsOnly + Unearth both present |
| wrenn_and_realmbreaker | Partial | 2 (static mana, -2 mill) | No (Finding 4: +1 duration) | -7 emblem correct; +1 duration pre-existing bug |
| oathsworn_vampire | Yes | 0 | Yes | Condition check correct; enters-tapped present |
| squee_dubious_monarch | Yes | 0 | No (Findings 1+2: wrong cost, no exile) | Attack trigger + haste present; GY cast cost+exile not enforced |
| brokkos_apex_of_forever | Yes | 1 (stale, Finding 3) | Yes | required_alt_cost Mutate enforced; stale TODO |
