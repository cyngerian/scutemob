# Ability Review: Dungeon Session 4 (Card Definitions, Harness, Game Scripts)

**Date**: 2026-03-09
**Reviewer**: ability-impl-reviewer (Opus)
**CR**: 309.4c, 701.49, 725.1-2, 603.4
**Files reviewed**:
- `crates/engine/src/cards/defs/nadaar_selfless_paladin.rs` (card definition)
- `crates/engine/src/cards/defs/seasoned_dungeoneer.rs` (card definition)
- `crates/engine/src/cards/defs/acererak_the_archlich.rs` (card definition)
- `crates/engine/src/cards/card_definition.rs:1488-1492` (Condition::Not variant)
- `crates/engine/src/effects/mod.rs:3479-3481` (Condition::Not evaluation)
- `crates/engine/src/state/hash.rs:3644-3648` (Condition::Not hash)
- `crates/engine/src/state/hash.rs:1895-1903` (is_carddef_etb hash)
- `crates/engine/src/state/hash.rs:1418` (PendingTriggerKind::CardDefETB hash)
- `crates/engine/src/state/stack.rs:444-453` (is_carddef_etb field)
- `crates/engine/src/state/stubs.rs:28-33` (PendingTriggerKind::CardDefETB)
- `crates/engine/src/rules/resolution.rs:1895-2024` (CardDefETB resolution + intervening-if)
- `crates/engine/src/rules/abilities.rs:6304-6316` (PendingTriggerKind dispatch)
- `crates/engine/src/testing/replay_harness.rs:1706-1710` (venture_into_dungeon action)
- `crates/engine/tests/dungeon_cards.rs` (5 tests)
- `test-data/generated-scripts/etb-triggers/205_nadaar_ventures_on_etb.json` (game script)

## Verdict: fixed (2026-03-09)

One HIGH finding: the CardDefETB resolution path's inline intervening-if check uses a `_ => true` wildcard that silently skips `Condition::Not`, meaning Acererak's "if you haven't completed Tomb of Annihilation" condition is ALWAYS treated as true at resolution. This means Acererak will always bounce to hand even after completing the Tomb. One MEDIUM finding: the Acererak attack trigger creates Zombie tokens under the Acererak controller's control instead of under each opponent's control as the oracle text specifies.

## Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `resolution.rs:1989-1998` | **CardDefETB intervening-if wildcard skips Condition::Not.** The inline condition check only handles `OpponentHasPoisonCounters`; all other conditions (including `Not`) hit `_ => true`. Acererak's intervening-if is always treated as satisfied. **Fix:** Replace inline match with call to `check_condition()`. **FIXED** — `check_condition` made `pub(crate)` in `effects/mod.rs`; imported and called in `resolution.rs:1985-2001`. |
| 2 | **MEDIUM** | `acererak_the_archlich.rs:58-73` | **Zombie tokens created under wrong controller.** ForEach+EachOpponent keeps `controller` as Acererak's controller, so `CreateToken` gives zombies to the Acererak player instead of each opponent. **Fix:** use `Effect::CreateTokenForPlayer` or note as documented simplification. **FIXED** — documented as known simplification with TODO(M10+) comment in `acererak_the_archlich.rs`. |
| 3 | LOW | `dungeon_cards.rs` | **Missing negative test for Acererak with completed Tomb.** No test verifies Acererak stays on battlefield when player HAS completed Tomb of Annihilation. Would have caught Finding 1. |
| 4 | LOW | `seasoned_dungeoneer.rs:44-46` | **Attack trigger entirely absent from abilities vec.** The DSL gaps are documented but the triggered ability is not even present as a stub. Compare to Nadaar which has both ETB and attack triggers defined. |
| 5 | LOW | `205_nadaar_ventures_on_etb.json` | **Script review_status is pending_review.** Should be approved after validation. |

### Finding Details

#### Finding 1: CardDefETB intervening-if wildcard skips Condition::Not

**Severity**: HIGH
**File**: `crates/engine/src/rules/resolution.rs:1989-1998`
**CR Rule**: 603.4 -- "A triggered ability may read 'When/Whenever/At [trigger event], if [condition], [effect].' [...] If the ability triggers, it checks the stated condition again as it resolves. If the condition isn't true at that time, the ability is removed from the stack and does nothing."
**Issue**: The CardDefETB resolution path (lines 1984-2001) has its own inline `match cond` that only explicitly handles `Condition::OpponentHasPoisonCounters(n)`. All other conditions fall through to `_ => true`:

```rust
match cond {
    Condition::OpponentHasPoisonCounters(n) => { ... }
    // Other conditions: treat as satisfied (safe default for rare cases).
    _ => true,
}
```

Acererak's intervening-if is `Condition::Not(Box::new(Condition::CompletedSpecificDungeon(TombOfAnnihilation)))`. This hits the `_ => true` arm, so the condition is ALWAYS treated as satisfied at resolution time. After a player completes Tomb of Annihilation and casts Acererak again, the ETB should not fire (CR 603.4), but it does -- Acererak bounces to hand and ventures even though the condition is false.

The effects/mod.rs already has a correct `check_condition` function (line 3389) that handles `Condition::Not`, `CompletedSpecificDungeon`, `CompletedADungeon`, and all other variants properly. The resolution.rs CardDefETB path should use it instead of an inline match.

The existing test `test_acererak_bounces_without_tomb` passes only because the bug coincidentally gives the correct result when the condition IS true (Tomb not completed). A negative test would have caught this.

**Fix**: Replace the inline condition match at resolution.rs:1987-2000 with a call to `check_condition`. The function requires an `EffectContext`; construct one from `stack_obj.controller` and `source_object`:

```rust
let condition_holds = triggered_carddef_iif
    .as_ref()
    .map(|cond| {
        let ctx = EffectContext::new(stack_obj.controller, source_object, stack_obj.targets.clone());
        crate::effects::check_condition(state, cond, &ctx)
    })
    .unwrap_or(true);
```

If `check_condition` is not public, make it `pub(crate)`.

#### Finding 2: Zombie tokens created under wrong controller

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/acererak_the_archlich.rs:56-73`
**CR Rule**: Acererak oracle text: "for each opponent, **that player** creates a 2/2 black Zombie creature token unless that player sacrifices a creature."
**Issue**: The card definition uses `ForEach { over: ForEachTarget::EachOpponent, effect: Box::new(Effect::CreateToken { ... }) }`. When `ForEachTarget::EachOpponent` iterates, it sets each opponent as a target in `inner_ctx.targets` but keeps `inner_ctx.controller` as the Acererak controller (effects/mod.rs:1524). `CreateToken` creates tokens under `ctx.controller` (effects/mod.rs:457). This means the Zombie tokens are created under the Acererak player's control, not each opponent's. The oracle clearly states "that player creates" -- the opponents should control the zombies.

This inverts the card's design intent: Acererak's attack trigger is a punisher effect where opponents get bad tokens (must either sacrifice or get a zombie under their control), not a benefit that gives the Acererak player free creatures.

**Fix**: Either (a) document this as a known simplification in the card definition with a comment explaining the controller inversion: `// Simplification: tokens created under controller instead of each opponent (DSL gap: no CreateTokenForPlayer effect). Deferred to M10+.`; or (b) add an `Effect::CreateTokenUnderTargetControl` variant that uses the target player as the token controller instead of `ctx.controller`. Option (a) is acceptable for now but the comment MUST note the inversion, not just the "unless sacrifice" simplification.

#### Finding 3: Missing negative test for Acererak

**Severity**: LOW
**File**: `crates/engine/tests/dungeon_cards.rs`
**CR Rule**: 603.4 -- intervening-if must fail when condition no longer holds
**Issue**: There is no test for the case where a player HAS completed Tomb of Annihilation and casts Acererak. In that scenario, the intervening-if condition ("if you haven't completed Tomb of Annihilation") should be false, and Acererak should remain on the battlefield without bouncing or venturing. This negative test would have immediately caught Finding 1.
**Fix**: Add `test_acererak_stays_after_tomb_completed`. Set up state where p1's `dungeons_completed_set` contains `DungeonId::TombOfAnnihilation`, cast Acererak, resolve ETB trigger, assert Acererak is still on the battlefield and no `VenturedIntoDungeon` event was emitted.

#### Finding 4: Seasoned Dungeoneer attack trigger absent

**Severity**: LOW
**File**: `crates/engine/src/cards/defs/seasoned_dungeoneer.rs:44-46`
**CR Rule**: Seasoned Dungeoneer oracle: "Whenever you attack, target attacking Cleric, Rogue, Warrior, or Wizard gains protection from creatures until end of turn. It explores."
**Issue**: The attack trigger is documented with TODO comments but is entirely absent from the abilities vec. Nadaar, by contrast, includes its attack trigger as a fully-defined `AbilityDefinition::Triggered`. While the DSL gaps (creature-subtype targeting, protection from creatures, explore) are real, the absence of even a stub trigger is inconsistent. The comment says "WhenAttacks used as approximation" but no WhenAttacks trigger is actually present.
**Fix**: Either add a no-op WhenAttacks trigger with a TODO comment in the effect, or update the comment to say "attack trigger OMITTED (not approximated)" to avoid confusion.

#### Finding 5: Script pending_review status

**Severity**: LOW
**File**: `test-data/generated-scripts/etb-triggers/205_nadaar_ventures_on_etb.json:28`
**Issue**: `review_status` is `"pending_review"`. If the script has been validated (tests pass), it should be approved.
**Fix**: Change `review_status` to `"approved"` and fill in `reviewed_by` and `review_date`.

## CR Coverage Check

| CR Subrule | Implemented? | Tested? | Notes |
|------------|-------------|---------|-------|
| 309.4c (room abilities are triggered) | Yes | Yes | test_nadaar_enters_ventures checks RoomAbility on stack |
| 603.4 (intervening-if at trigger + resolution) | Yes (code) / **Broken (resolution)** | Partial | Trigger-time check works; resolution-time check always returns true (Finding 1) |
| 701.49a (no dungeon -- enter new) | Yes | Yes | test_nadaar_enters_ventures |
| 701.49b (mid-dungeon -- advance) | Yes | No (deferred S2 test) | |
| 701.49c (bottommost -- complete + restart) | Yes | No (deferred S2 test) | |
| 725.1 (initiative designation) | Yes | Yes | test_initiative_take_ventures_undercity checks has_initiative |
| 725.2 (take initiative -- venture Undercity) | Yes | Yes | test_initiative_take_ventures_undercity |
| Nadaar ETB venture | Yes | Yes | test_nadaar_enters_ventures |
| Nadaar attack venture | Yes | Yes | test_nadaar_attacks_ventures |
| Nadaar static +1/+1 buff | No (DSL gap) | Partial | test_nadaar_completed_dungeon_buff checks dungeons_completed but not P/T |
| Acererak ETB bounce + venture | Yes (buggy) | Positive only | test_acererak_bounces_without_tomb; no negative test (Finding 3) |
| Acererak attack zombie tokens | Yes (wrong controller) | No | Finding 2 |
| Seasoned Dungeoneer ETB initiative | Yes | Yes | test_initiative_take_ventures_undercity |
| Seasoned Dungeoneer attack trigger | No (DSL gap) | No | Finding 4 |

## Infrastructure Check

| Component | Status | Notes |
|-----------|--------|-------|
| `Condition::Not` in card_definition.rs | Correct | Properly defined with `Box<Condition>` |
| `Condition::Not` in effects/mod.rs `check_condition` | Correct | `!check_condition(state, inner, ctx)` |
| `Condition::Not` hash coverage | Correct | Discriminant 17, hashes inner |
| `PendingTriggerKind::CardDefETB` | Correct | Discriminant 46 in hash, properly wired in abilities.rs dispatch |
| `is_carddef_etb` on StackObjectKind::TriggeredAbility | Correct | Field added, hashed, serde default |
| `is_carddef_etb` in resolution.rs routing | Correct | Routing to CardDef path works; bug is in condition eval, not routing |
| `venture_into_dungeon` harness action | Correct | Maps to `Command::VentureIntoDungeon { player }` |
| Replay viewer view_model.rs | OK | Uses `..` match on TriggeredAbility, unaffected by new field |
| TUI stack_view.rs | OK | No TriggeredAbility match needed (handled elsewhere) |

## Card Definition Correctness

| Card | Oracle Match? | Simplifications Documented? | Types Correct? | Stats Correct? |
|------|--------------|---------------------------|----------------|----------------|
| Nadaar, Selfless Paladin | Partial | Yes (static buff DSL gap) | Yes (Legendary Creature -- Dragon Knight) | Yes (3/3, {3}{W}) |
| Seasoned Dungeoneer | Partial | Partially (attack trigger claimed approximated but absent) | Yes (Creature -- Human Warrior) | Yes (3/4, {3}{W}) |
| Acererak the Archlich | Partial | Yes (sacrifice choice); **No** (token controller -- Finding 2) | Yes (Legendary Creature -- Zombie Wizard) | Yes (5/5, {2}{B}) |

## Previous Findings (from S1, S2, S3 reviews)

| # | Previous ID | Session | Previous Status | Current Status | Notes |
|---|-------------|---------|----------------|----------------|-------|
| S1-F1 | HIGH | S1 | OPEN | Unknown | Tomb of Annihilation room graph (Oubliette exits). Not in scope for S4 review. |
| S1-F2 | HIGH | S1 | OPEN | Unknown | Atropal token legendary supertype. Not in scope for S4 review. |
| S2-F1 | MEDIUM | S2 | FIXED | RESOLVED | CompletedSpecificDungeon now uses dungeons_completed_set. |
| S2-F2 | MEDIUM | S2 | OPEN | RESOLVED (S3) | CR 309.6 SBA implemented in S3. |
| S3-F1 | MEDIUM | S3 | FIXED | RESOLVED | CR 725.4 active player priority. |
| S3-F2 | MEDIUM | S3 | FIXED | RESOLVED | First-strike damage initiative steal. |
