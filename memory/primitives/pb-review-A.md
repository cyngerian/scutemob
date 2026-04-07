# Primitive Batch Review: PB-A -- Play from Top of Library

**Date**: 2026-04-07
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 601.1a, 601.2, 601.3, 305.1, 305.2a, 118.9, 119.4
**Engine files reviewed**: `state/stubs.rs`, `state/mod.rs`, `state/builder.rs`, `state/types.rs`, `state/hash.rs`, `cards/card_definition.rs`, `cards/helpers.rs`, `rules/casting.rs`, `rules/lands.rs`, `rules/replacement.rs`, `rules/turn_actions.rs`, `lib.rs`
**Card defs reviewed**: 10 (4 fixes + 6 new)

## Verdict: needs-fix

Two HIGH findings (on_cast_effect targets wrong ObjectId after resolution; find_play_from_top_on_cast_effect checks wrong permission), two MEDIUM findings (life payment legality check missing; missing planned tests), two LOW findings (Bolas's Citadel mandatory life payment not enforced; legal_actions.rs not updated for simulator). The core permission system is well-structured and follows the FlashGrant pattern correctly. Card definitions all match oracle text. Hash support is complete.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `casting.rs:4017` | **on_cast_effect haste targets stack ObjectId, not battlefield ObjectId.** Continuous effect uses `SingleObject(new_card_id)` where `new_card_id` is the stack ObjectId. After resolution, the permanent gets a new ObjectId on the battlefield (CR 400.7). The haste grant never applies. **Fix:** Use the Dash/Blitz pattern: store a flag on StackObject (e.g., `cast_from_top_with_bonus: bool`), then at resolution time in `resolution.rs`, directly insert `KeywordAbility::Haste` into `obj.characteristics.keywords` (like `was_dashed` does at resolution.rs:608-610). Remove the continuous effect approach. |
| 2 | **HIGH** | `casting.rs:5654` | **find_play_from_top_on_cast_effect checks ANY permission, not the specific one.** The function iterates permissions with `on_cast_effect`, but at line 5654 calls `has_play_from_top_permission()` which checks if ANY permission matches. If a player has Future Sight (All, no on_cast_effect) AND Thundermane Dragon (CreaturesWithMinPower(4), with haste on_cast_effect), casting a 2/2 creature from top would incorrectly grant haste -- Future Sight's All filter matches, so `has_play_from_top_permission` returns true, and Thundermane's `on_cast_effect` is returned. **Fix:** Inline the filter-matching logic from `has_play_from_top_permission` directly into `find_play_from_top_on_cast_effect`, checking `perm.filter` against `chars` for the specific permission being evaluated, not delegating to the global check. |
| 3 | **MEDIUM** | `casting.rs:3439` | **No life-total check before PayLifeForManaValue deduction.** CR 119.4: "If a cost or effect allows a player to pay an amount of life greater than 0, the player may do so only if their life total is greater than or equal to the amount of the payment." Currently, life is deducted unconditionally. A player at 1 life could cast a spell with mana value 5, going to -4 (SBAs would kill them, but the cast itself should be illegal). **Fix:** Before `player_state.life_total -= original_mana_value as i32`, add a check: `if player_state.life_total < original_mana_value as i32 { return Err(GameStateError::InvalidCommand("not enough life to pay life cost (CR 119.4)".into())); }`. |
| 4 | **MEDIUM** | `tests/play_from_top.rs` | **Two planned tests missing.** The plan specified `test_play_from_top_bolas_citadel_x_is_zero` (X spells cast via Bolas's Citadel must have X=0) and `test_play_from_top_haste_grant` (Thundermane Dragon creature cast from top gains haste). Neither was implemented. The X=0 enforcement is also not present in the engine code -- when `cast_with_pay_life` and `x_value > 0`, the legacy X path at casting.rs:3390 would add x_value to the (now zero) generic cost, requiring mana payment that shouldn't exist. **Fix:** (1) Add X=0 enforcement in casting.rs: when `cast_with_pay_life && x_value > 0`, return an error citing CR 107.3 / 2019-05-03 ruling. (2) Write `test_play_from_top_bolas_citadel_x_is_zero` verifying the rejection. (3) Write `test_play_from_top_haste_grant` once Finding 1 is fixed. |
| 5 | LOW | `casting.rs:550` | **Bolas's Citadel mandatory life payment not enforced.** Oracle text says "If you cast a spell this way, pay life... rather than pay its mana cost" -- the life payment is mandatory when casting from top via Citadel, not optional. Current implementation allows a player with only Bolas's Citadel to cast from top and pay mana normally by not specifying `PayLifeForManaValue`. A well-behaved client would always specify it, but the engine doesn't enforce it. **Fix:** When `casting_from_library_top` and the matching permission has `pay_life_instead: true` and no other non-pay-life permission matches, either (a) force `cast_with_pay_life = true` automatically, or (b) return an error if `alt_cost` is not `PayLifeForManaValue`. Option (a) is simpler -- if ALL matching permissions have `pay_life_instead`, auto-set it. |
| 6 | LOW | `simulator/legal_actions.rs` | **Simulator does not enumerate library-top cards as legal actions.** PlayLand only checks hand (line 191-194); CastSpell similarly only checks hand. Bots and the legal action provider won't suggest casting/playing from library top. **Fix:** After the hand-based land enumeration, also check if the top card of the library is a land and an active `play_from_top_land_permission` exists. Similarly for spells with `has_play_from_top_permission`. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| -- | -- | -- | No card definition findings. All 10 card defs match oracle text. |

### Finding Details

#### Finding 1: on_cast_effect haste targets stack ObjectId, not battlefield ObjectId

**Severity**: HIGH
**File**: `crates/engine/src/rules/casting.rs:4011-4022`
**CR Rule**: CR 400.7 -- "An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence."
**Issue**: The on_cast_effect (used by Thundermane Dragon to grant haste) creates a `ContinuousEffect` with `filter: EffectFilter::SingleObject(new_card_id)` where `new_card_id` is the ObjectId of the spell on the stack. When the spell resolves and the permanent moves to the battlefield, it gets a new ObjectId. The continuous effect still targets the old stack ObjectId, so the haste grant never applies to the battlefield permanent. Dash and Blitz avoid this by applying haste directly in `resolution.rs` after the permanent enters the battlefield.
**Fix**: Remove the continuous effect approach (lines 4001-4024). Instead, add a `was_cast_from_top_with_bonus: bool` field to StackObject. Set it to true when `find_play_from_top_on_cast_effect` returns Some. In `resolution.rs`, after the permanent enters the battlefield (near line 608 where `was_dashed` is handled), check `was_cast_from_top_with_bonus` and directly insert `KeywordAbility::Haste` + register an end-of-turn removal. Alternatively, add a UntilEndOfTurn continuous effect targeting the NEW ObjectId at resolution time.

#### Finding 2: find_play_from_top_on_cast_effect checks wrong permission

**Severity**: HIGH
**File**: `crates/engine/src/rules/casting.rs:5636-5660`
**CR Rule**: Thundermane Dragon oracle: "You may cast creature spells with power 4 or greater from the top of your library. If you cast a creature spell **this way**, it gains haste until end of turn."
**Issue**: `find_play_from_top_on_cast_effect` iterates permissions looking for one with `on_cast_effect`. When it finds one (e.g., Thundermane Dragon's), it calls `has_play_from_top_permission()` at line 5654 which checks if ANY permission matches. This means a permission with an on_cast_effect could have its bonus applied even when a DIFFERENT permission (with a broader filter) is the one actually enabling the cast. Example: player has Future Sight + Thundermane Dragon; casts a 2/2 creature from top -- Future Sight enables it, but Thundermane's haste incorrectly applies because `has_play_from_top_permission` returns true (Future Sight matches).
**Fix**: Replace line 5654 with inline filter matching that checks `perm.filter` against `chars` directly, exactly as done in `has_play_from_top_permission` but scoped to the specific `perm` being evaluated.

#### Finding 3: No life-total check before PayLifeForManaValue deduction

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/casting.rs:3439`
**CR Rule**: 119.4 -- "If a cost or effect allows a player to pay an amount of life greater than 0, the player may do so only if their life total is greater than or equal to the amount of the payment."
**Issue**: Life is deducted without checking the player can afford it. A player at 1 life casting a spell with mana value 5 would go to -4 life (technically illegal per CR 119.4 -- the cast should be rejected).
**Fix**: Add `if player_state.life_total < original_mana_value as i32 { return Err(...); }` before the deduction at line 3440.

#### Finding 4: Two planned tests missing

**Severity**: MEDIUM
**File**: `crates/engine/tests/play_from_top.rs`
**CR Rule**: CR 107.3 (X=0 when not paying mana cost); Thundermane Dragon oracle
**Issue**: The plan specified 2 tests that were not written: `test_play_from_top_bolas_citadel_x_is_zero` and `test_play_from_top_haste_grant`. Additionally, X=0 enforcement is not implemented in the engine for `PayLifeForManaValue`.
**Fix**: (1) Add explicit X=0 enforcement in casting.rs for `cast_with_pay_life`. (2) Write both missing tests.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 601.1a | Yes | Yes | All filter covers "play lands and cast spells" |
| 601.2 | Yes | Yes | test_play_from_top_cast_creature |
| 601.3 | Yes | Yes | Multiple filter tests, no-permission rejection |
| 305.1 | Yes | Yes | test_play_from_top_basic_land |
| 305.2a | Yes | Yes | test_play_from_top_land_uses_land_play |
| 118.9 | Yes | Yes | test_play_from_top_bolas_citadel_pay_life |
| 118.9a | Partial | No | X=0 enforcement missing (Finding 4) |
| 119.4 | No | No | Life total >= payment not checked (Finding 3) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Courser of Kruphix | Yes | 0 | Yes | |
| Elven Chorus | Yes | 1 (mana grant) | Yes | Mana grant is separate DSL gap |
| Thundermane Dragon | Yes | 0 | No | Haste grant broken (Finding 1) |
| Case of the Locked Hothouse | Yes | 0 | Yes | SourceIsSolved condition correct |
| Future Sight | Yes | 0 | Yes | |
| Bolas's Citadel | Yes | 1 (activated ability) | Partial | Life payment optional (Finding 5) |
| Mystic Forge | Yes | 1 (exile top ability) | Yes | |
| Oracle of Mul Daya | Yes | 0 | Yes | |
| Vizier of the Menagerie | Yes | 1 (mana spending) | Yes | Mana spending is separate gap |
| Radha, Heart of Keld | Yes | 1 (+X/+X ability) | Yes | Dynamic P/T is separate gap |
