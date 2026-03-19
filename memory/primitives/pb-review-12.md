# Primitive Batch Review: PB-12 -- Complex Replacement Effects

**Date**: 2026-03-18
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: 614.1, 614.5, 614.16, 615, 616.1, 122.6, 111.1, 701.19, 701.34, 603.2d
**Engine files reviewed**: `state/replacement_effect.rs`, `rules/replacement.rs`, `state/hash.rs`, `state/stubs.rs`, `cards/card_definition.rs`, `rules/abilities.rs`, `effects/mod.rs`, `rules/combat.rs`
**Card defs reviewed**: 8 (adrix_and_nev_twincasters, bloodletter_of_aclazotz, vorinclex_monstrous_raider, aven_mindcensor, pir_imaginative_rascal, tekuthal_inquiry_dominus, teysa_karlov, twinflame_tyrant)

## Verdict: needs-fix

Two HIGH findings: life-loss doubling is dead code (defined but never called from any effect execution path), and damage doubling is not applied to combat damage. Two cards (Bloodletter of Aclazotz and Twinflame Tyrant) have replacement effects that are registered but never fire in their intended game scenarios. Additionally, Vorinclex's counter replacement only handles permanents, not players (oracle says "permanent or player"), and two card defs have remaining TODOs for incomplete abilities.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `rules/replacement.rs:2654` | **Life-loss doubling is dead code.** `apply_life_loss_doubling` is defined but never called from `Effect::LoseLife`, `Effect::DrainLife`, or any damage-to-life-loss path. Bloodletter's replacement effect is registered but never applied. **Fix:** Call `apply_life_loss_doubling` in `Effect::LoseLife` (effects/mod.rs:370) and `Effect::DrainLife` (effects/mod.rs:396) before subtracting from life_total. |
| 2 | **HIGH** | `rules/combat.rs:1510` | **Combat damage bypasses damage doubling.** `apply_combat_damage` calls `apply_damage_prevention` but not `apply_damage_doubling`. Twinflame Tyrant's DoubleDamage replacement only works for `Effect::DealDamage`, not combat damage. **Fix:** Insert `apply_damage_doubling` call for each assignment's amount before `apply_damage_prevention` in `apply_combat_damage`. |
| 3 | **MEDIUM** | `rules/replacement.rs:2471` | **Counter replacement ignores player receivers.** `apply_counter_replacement` takes `receiver_id: ObjectId` only. Vorinclex's oracle says "permanent **or player**" -- counters placed on players (poison, experience, energy) are not doubled/halved. **Fix:** Add a parallel `apply_counter_replacement_player(state, placer, player, counter, count)` function and call it from poison/experience counter paths. |
| 4 | **MEDIUM** | `rules/abilities.rs:7277` | **Teysa CreatureDeath doubling only matches SelfDies.** The `TriggerDoublerFilter::CreatureDeath` arm only matches `TriggerEvent::SelfDies` but not `WheneverCreatureDies` CardDef triggers from other permanents. The code comment acknowledges this gap. **Fix:** Propagate triggering event through the CardDef trigger collection path so `WheneverCreatureDies` triggers on other permanents are also doubled. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 5 | **HIGH** | `bloodletter_of_aclazotz.rs` | **Replacement effect never fires.** The card registers a WouldLoseLife replacement but `apply_life_loss_doubling` is never called (Finding 1). Even if it were called, the "during your turn" condition is missing. **Fix:** Wire `apply_life_loss_doubling` calls (Finding 1), then add turn-condition checking. |
| 6 | **HIGH** | `twinflame_tyrant.rs` | **Damage doubling incomplete for combat damage.** The card registers a DamageWouldBeDealt replacement but combat damage bypasses `apply_damage_doubling` (Finding 2). Also missing opponent-target filter (oracle: "to an opponent or a permanent an opponent controls"). **Fix:** Wire combat damage doubling (Finding 2), then add opponent-target filtering to DamageTargetFilter. |
| 7 | MEDIUM | `tekuthal_inquiry_dominus.rs:33` | **TODO remaining: activated ability.** Oracle has "{1}{U/P}{U/P}, Remove three counters from among other artifacts, creatures, and planeswalkers you control: Put an indestructible counter on Tekuthal." This is not implemented. **Fix:** Implement when remove-counters-from-others cost becomes expressible in the DSL. Track as known gap. |
| 8 | MEDIUM | `teysa_karlov.rs:27` | **TODO remaining: static token ability.** Oracle says "Creature tokens you control have vigilance and lifelink." Not implemented -- needs EffectFilter::TokenCreatures. **Fix:** Implement when token-only EffectFilter becomes available. Track as known gap. |
| 9 | MEDIUM | `bloodletter_of_aclazotz.rs:10` | **TODO remaining: "during your turn" condition.** Oracle specifies "during your turn" but the replacement fires unconditionally. Documented with TODO. **Fix:** Add a `Condition` or turn-check field to `ReplacementEffect` and check `state.turn.active_player == controller` at application time. |
| 10 | MEDIUM | `twinflame_tyrant.rs:9` | **TODO remaining: opponent-target filter.** Oracle specifies "to an opponent or a permanent an opponent controls" but the implementation doubles ALL damage from controller's sources regardless of target. Documented with TODO. **Fix:** Add `DamageTargetFilter::ToOpponentOrTheirPermanent(PlayerId)` and use compound filter on the replacement trigger. |

### Finding Details

#### Finding 1: Life-loss doubling is dead code

**Severity**: HIGH
**File**: `crates/engine/src/rules/replacement.rs:2654`
**CR Rule**: 614.1 -- replacement effects modify events as they happen
**Oracle**: Bloodletter of Aclazotz: "If an opponent would lose life during your turn, they lose twice that much life instead."
**Issue**: The function `apply_life_loss_doubling` is defined (lines 2654-2688) and tested in isolation (token_damage_search_replacement.rs:314-351), but it is never called from any effect execution path. `Effect::LoseLife` (effects/mod.rs:370-391) directly subtracts from `life_total` without checking for life-loss replacements. `Effect::DrainLife` (effects/mod.rs:396-430) does the same. Combat damage life loss (combat.rs:1608) also does not call this function. The registered replacement effect has no runtime impact.
**Fix**: In `effects/mod.rs`, add calls to `apply_life_loss_doubling` in both `Effect::LoseLife` and `Effect::DrainLife` handlers before subtracting from `life_total`. For combat damage, evaluate whether life-loss doubling should also apply there (per Bloodletter ruling: "doesn't change the amount of damage dealt" -- damage doubling and life-loss doubling are distinct; life-loss doubling should apply to the life loss caused by damage, not the damage itself).

#### Finding 2: Combat damage bypasses damage doubling

**Severity**: HIGH
**File**: `crates/engine/src/rules/combat.rs:1510`
**CR Rule**: 614.1 -- replacement effects apply to all instances of the replaced event
**Oracle**: Twinflame Tyrant: "If a source you control would deal damage to an opponent or a permanent an opponent controls, it deals double that damage instead."
**Issue**: `apply_combat_damage` (combat.rs:1220) builds damage assignments, then applies only `apply_damage_prevention` (line 1510) before marking damage. It never calls `apply_damage_doubling`. Combat damage from sources Twinflame Tyrant's controller controls is not doubled. The DealDamage effect path correctly calls `apply_damage_doubling` (effects/mod.rs:185), but combat damage is a separate code path.
**Fix**: Before the `apply_damage_prevention` loop in `apply_combat_damage`, add a pre-processing step that calls `apply_damage_doubling(state, assignment.source, assignment.amount)` for each assignment and updates the amount. This must happen before prevention per CR 614.1 (doublers modify the event before preventers).

#### Finding 3: Counter replacement ignores player receivers

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/replacement.rs:2471`
**CR Rule**: 122.6 -- counters can be placed on both permanents and players
**Oracle**: Vorinclex: "If you would put one or more counters on a permanent **or player**, put twice that many..."
**Issue**: `apply_counter_replacement` only takes `receiver_id: ObjectId`. When poison counters (infect damage to players), experience counters, or energy counters are placed on players, Vorinclex's replacement does not apply because there is no parallel function for player receivers. The `WouldPlaceCounters` trigger has `receiver_filter: ObjectFilter` which has no player-matching variant.
**Fix**: Add an `apply_counter_replacement_player` function (or extend the existing function with an enum receiver) and call it from the infect/poison path (effects/mod.rs:215, combat.rs:1630+). Add `ObjectFilter::Player(PlayerId)` or use a separate `PlayerFilter` for receiver matching.

#### Finding 4: Teysa death trigger doubling limited to SelfDies

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:7277`
**CR Rule**: 603.2d -- trigger doubling
**Oracle**: Teysa Karlov: "If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time."
**Ruling**: (2019-01-25) "Teysa affects a creature's own 'when this creature dies' triggered abilities as well as other triggered abilities that trigger when that creature dies."
**Issue**: The `CreatureDeath` filter only matches `TriggerEvent::SelfDies`. Other permanents with "whenever a creature dies" triggers (e.g., Blood Artist, Zulaport Cutthroat) use `PendingTriggerKind::Normal` without a `triggering_event` field, so they are not doubled. The code comment at line 7280 acknowledges this.
**Fix**: Propagate the triggering event through the CardDef trigger collection path in `check_events_for_triggers` so that `WheneverCreatureDies`-sourced `PendingTrigger` entries carry `triggering_event: Some(TriggerEvent::SelfDies)` or a new `TriggerEvent::AnyCreatureDies` variant.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 614.1 (replacement effects) | Partial | Partial | Token doubling, counter doubling, search restriction: wired + tested. Life-loss doubling: dead code (HIGH). Damage doubling: missing combat path (HIGH). |
| 614.5 (one application per event) | Partial | No | `find_applicable` used for counters/tokens/search/life-loss. Damage doubling iterates directly (functionally correct for single doublers). |
| 614.16 (token/counter replacement scope) | Yes | Yes | Token creation replacement applies to resolved spell effects per 614.16. |
| 616.1 (multiple replacement ordering) | Partial | Yes | Deterministic ordering (pre-M10). Counter stacking test (Vorinclex+Pir). Player choice deferred. |
| 122.6 (counters on permanents/players) | Partial | Yes (permanents only) | Missing player receiver path (MEDIUM). |
| 111.1 (token creation) | Yes | Yes | `apply_token_creation_replacement` wired in CreateToken. |
| 701.19 (library search) | Yes | Yes | `apply_search_library_replacement` wired in SearchLibrary. |
| 701.34 (proliferate) | Yes | Yes | `apply_proliferate_replacement` wired in Proliferate effect. |
| 603.2d (trigger doubling) | Partial | Yes | Panharmonicon (ETB) fully wired + 3 tests. Teysa (CreatureDeath) only matches SelfDies (MEDIUM). |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| adrix_and_nev_twincasters | Yes | 0 | Yes | Token doubling correctly wired |
| bloodletter_of_aclazotz | No | 1 | **No** | Life-loss doubling never fires (HIGH F1/F5); missing "during your turn" (MEDIUM F9) |
| vorinclex_monstrous_raider | Partial | 0 | Partial | Counter doubling works for permanents; missing player receivers (MEDIUM F3) |
| aven_mindcensor | Yes | 0 | Yes | Search restriction correctly wired |
| pir_imaginative_rascal | Yes | 0 | Yes | AddExtraCounter correctly scoped to controlled permanents |
| tekuthal_inquiry_dominus | Partial | 1 | Partial | Proliferate doubling works; activated ability not implemented (MEDIUM F7) |
| teysa_karlov | Partial | 1 | Partial | SelfDies doubling works; static token grant missing (MEDIUM F8); WheneverCreatureDies doubling incomplete (MEDIUM F4) |
| twinflame_tyrant | No | 1 | **No** | Damage doubling missing combat path (HIGH F2/F6); opponent-target filter missing (MEDIUM F10) |

## Test Coverage Summary

- `counter_replacement.rs`: 8 tests covering DoubleCounters, HalveCounters, AddExtraCounter, stacking (Vorinclex+Pir), zero-counter edge case, negative cases
- `token_damage_search_replacement.rs`: 6 tests covering DoubleTokens, RestrictSearchTopN, DoubleDamage, DoubleLifeLoss, negative cases
- `trigger_doubling.rs`: 4 tests covering Panharmonicon single/double/removal/registration-via-resolution
- **Gap**: No integration test for life-loss doubling through the actual Effect::LoseLife path (because it's dead code)
- **Gap**: No integration test for damage doubling through combat damage path
- **Gap**: No test for counter replacement on player receivers (poison/experience)
- **Gap**: No test for Teysa doubling "whenever a creature dies" triggers from other permanents
