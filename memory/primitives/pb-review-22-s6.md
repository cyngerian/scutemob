# Primitive Batch Review: PB-22 S6 -- Emblem Creation (CR 114)

**Date**: 2026-03-21
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 114.1-114.5, CR 113.6p
**Engine files reviewed**: `state/game_object.rs`, `cards/card_definition.rs`, `rules/events.rs`, `state/hash.rs`, `effects/mod.rs`, `rules/abilities.rs`, `cards/helpers.rs`
**Card defs reviewed**: 6 (ajani_sleeper_agent, basri_ket, kaito_bane_of_nightmares, tyvar_kell, wrenn_and_realmbreaker, wrenn_and_seven)

## Verdict: needs-fix

The core emblem infrastructure (Effect::CreateEmblem, is_emblem field, GameEvent::EmblemCreated,
hash support, static CE registration, SBA immunity) is correctly implemented per CR 114.1-114.5.
The trigger scanning helper `collect_emblem_triggers_for_event` works for SpellCast events.
However, two card definitions have incorrect mana costs and starting loyalty values that do not
match oracle text, and the emblem trigger scanning is only wired to SpellCast events (missing
combat/upkeep/etc). Multiple card defs have incorrect trigger event types as noted workarounds.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `rules/abilities.rs:3233` | **Emblem trigger scanning only wired for SpellCast.** `collect_emblem_triggers_for_event` is only called from the SpellCast event handler. Basri Ket's emblem needs `AtBeginningOfCombat` which fires from StepChanged, but no emblem scanning call exists there. **Fix:** Add `collect_emblem_triggers_for_event` calls to StepChanged handlers for BeginningOfCombat, BeginningOfUpkeep, BeginningOfEndStep, and other step-based trigger collection points in `check_triggers`. |
| 2 | LOW | `effects/mod.rs:4007` | **EffectId derived from next_object_id.** `state.next_object_id().0` is used to generate EffectId for static CEs. This consumes an ObjectId counter value for a non-object purpose. Not a bug, but slightly unusual. No fix required. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **HIGH** | `wrenn_and_realmbreaker.rs` | **Mana cost incorrect.** Oracle: `{1}{G}{G}`, def has `generic: 2, green: 3` = `{2}{G}{G}{G}`. **Fix:** Change to `generic: 1, green: 2`. |
| 4 | **HIGH** | `wrenn_and_realmbreaker.rs` | **Starting loyalty incorrect.** Oracle: 4, def has `starting_loyalty: Some(7)`. **Fix:** Change to `Some(4)`. |
| 5 | **HIGH** | `wrenn_and_realmbreaker.rs` | **Oracle text and +1 ability wrong.** Oracle: "+1: Up to one target land you control becomes a 3/3 Elemental creature with vigilance, hexproof, and haste until your next turn. It's still a land." Def comment says "Up to two target lands" with "trample, haste, and indestructible" and has two TargetLand targets. **Fix:** Change to one `TargetLand` target; update comments to match actual oracle text (vigilance, hexproof, haste; until your next turn). |
| 6 | **HIGH** | `wrenn_and_realmbreaker.rs` | **-2 ability wrong.** Oracle: "Mill three cards. You may put a permanent card from among the milled cards into your hand." Def implements "Return target permanent card from your graveyard to your hand" with MoveZone. **Fix:** Update oracle_text string and comment to match. The MoveZone effect is an approximation, but the oracle text in the card def file MUST match the actual card. Add TODO noting mill+conditional return is the correct behavior. |
| 7 | **HIGH** | `tyvar_kell.rs` | **Starting loyalty incorrect.** Oracle: 3, def has `starting_loyalty: Some(5)`. **Fix:** Change to `Some(3)`. |
| 8 | **MEDIUM** | `basri_ket.rs:56` | **Emblem trigger event wrong.** Oracle: "At the beginning of combat on your turn". Def uses `TriggerEvent::AnySpellCast` as placeholder. This means the emblem fires on spell cast instead of combat. The NOTE comment acknowledges this, but the `trigger_on` should use the correct `TriggerEvent` value. **Fix:** Change `trigger_on` to the correct combat trigger event. If `TriggerEvent::AtBeginningOfCombat` does not exist as a TriggerEvent variant, add a TODO but keep the correct TriggerCondition name in the comment. Depends on Finding 1 being fixed first. |
| 9 | **MEDIUM** | `ajani_sleeper_agent.rs:66` | **Target too broad.** Oracle: "target opponent gets two poison counters". Def uses `TargetRequirement::TargetPlayer` which allows targeting any player including self. Should use `TargetRequirement::Opponent` if it exists, or document as TODO. **Fix:** Check if `TargetRequirement::Opponent` or similar exists; if so, use it. If not, add a `// TODO: should target opponent only, not any player` comment. |
| 10 | **MEDIUM** | `ajani_sleeper_agent.rs:57` | **Spell-type filter missing.** Oracle: "Whenever you cast a creature or planeswalker spell". Def triggers on any spell cast (AnySpellCast with no filter). The emblem will fire on instants/sorceries/etc. **Fix:** Document as TODO. This is acknowledged in the NOTE comment but the behavior is wrong for game state correctness. |
| 11 | **MEDIUM** | `tyvar_kell.rs:79` | **Spell-subtype filter missing.** Oracle: "Whenever you cast an Elf spell". Def triggers on any spell cast. **Fix:** Already documented as TODO at line 82. Acceptable as known gap. |
| 12 | **MEDIUM** | `wrenn_and_seven.rs:90` | **NoMaxHandSize applied to wrong target.** Oracle: "You have no maximum hand size" (player-level effect). Def uses `EffectFilter::CreaturesYouControl` to grant `KeywordAbility::NoMaxHandSize` to creatures. If the player controls no creatures, the emblem effect does nothing. The actual rule modifies the player, not permanents. **Fix:** Add `// TODO: NoMaxHandSize should be a player-level flag, not granted to creatures` comment if not already present. The current approach is incorrect when the player controls no creatures. |
| 13 | LOW | `basri_ket.rs:59` | **Emblem effect incomplete.** Oracle: "create a 1/1 white Soldier creature token, then put a +1/+1 counter on each creature you control." Def only creates the token (missing +1/+1 counter distribution). TODO at line 59 acknowledges this. No immediate fix required. |

### Finding Details

#### Finding 1: Emblem trigger scanning only wired for SpellCast

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:3233`
**CR Rule**: CR 114.4 -- "Abilities of emblems function in the command zone."
**Issue**: The `collect_emblem_triggers_for_event` helper is only called from the SpellCast event handler in `check_triggers`. Basri Ket's emblem has an "At the beginning of combat on your turn" trigger (TriggerCondition::AtBeginningOfCombat), which fires during the StepChanged event, not SpellCast. The current engine will never fire combat-phase emblem triggers. This affects the Basri Ket card def immediately and any future combat/upkeep/end-step emblem triggers.
**Fix**: Add `collect_emblem_triggers_for_event` calls to each relevant StepChanged handler in `check_triggers`. At minimum, add calls for AtBeginningOfCombat, AtBeginningOfYourUpkeep, and AtBeginningOfYourEndStep trigger events at the appropriate points in their respective event handlers.

#### Finding 3: Wrenn and Realmbreaker mana cost incorrect

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/wrenn_and_realmbreaker.rs:14`
**Oracle**: Wrenn and Realmbreaker costs `{1}{G}{G}` (Scryfall confirmed).
**Issue**: Def has `generic: 2, green: 3` which is `{2}{G}{G}{G}` -- a 5-mana card instead of a 3-mana card.
**Fix**: Change to `ManaCost { generic: 1, green: 2, ..Default::default() }`.

#### Finding 4: Wrenn and Realmbreaker starting loyalty incorrect

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/wrenn_and_realmbreaker.rs:72`
**Oracle**: Loyalty 4.
**Issue**: Def has `starting_loyalty: Some(7)`.
**Fix**: Change to `starting_loyalty: Some(4)`.

#### Finding 5: Wrenn and Realmbreaker +1 ability wrong

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/wrenn_and_realmbreaker.rs:29-39`
**Oracle**: "+1: Up to one target land you control becomes a 3/3 Elemental creature with vigilance, hexproof, and haste until your next turn. It's still a land."
**Issue**: The card def comment says "Up to two target lands" with "trample, haste, and indestructible" and specifies two `TargetRequirement::TargetLand` entries. The oracle text says one target with vigilance, hexproof, and haste.
**Fix**: Change targets to `vec![TargetRequirement::TargetLand]` (one target). Update comment at line 29 and the oracle_text string at line 23 to match the actual oracle text.

#### Finding 6: Wrenn and Realmbreaker -2 ability wrong

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/wrenn_and_realmbreaker.rs:41-53`
**Oracle**: "-2: Mill three cards. You may put a permanent card from among the milled cards into your hand."
**Issue**: The def implements "Return target permanent card from your graveyard to your hand" which is a completely different ability. The oracle text in the card def file is also wrong.
**Fix**: Update the oracle_text string and comment to match the actual oracle text. Change the effect to `Effect::Sequence(vec![])` with a TODO for mill + conditional return, since the current MoveZone approximation doesn't match the actual ability at all (the oracle text requires milling first, then choosing from among milled cards).

#### Finding 7: Tyvar Kell starting loyalty incorrect

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/tyvar_kell.rs:98`
**Oracle**: Loyalty 3.
**Issue**: Def has `starting_loyalty: Some(5)`.
**Fix**: Change to `starting_loyalty: Some(3)`.

#### Finding 9: Ajani Sleeper Agent target too broad

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/ajani_sleeper_agent.rs:66`
**Oracle**: "target opponent gets two poison counters"
**Issue**: Uses `TargetRequirement::TargetPlayer` which allows targeting self or allies. Should only allow targeting opponents.
**Fix**: If `TargetRequirement::Opponent` exists, use it. Otherwise add a TODO comment noting the targeting restriction gap.

#### Finding 12: Wrenn and Seven NoMaxHandSize wrong target

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/wrenn_and_seven.rs:80-92`
**Oracle**: "You have no maximum hand size."
**Issue**: The emblem grants `NoMaxHandSize` keyword to creatures the player controls via `EffectFilter::CreaturesYouControl`. This is incorrect -- it's a player-level rule modification, not a creature keyword. If the player controls no creatures, the emblem does nothing. Additionally, the engine's cleanup step likely only checks permanents for NoMaxHandSize, not the player.
**Fix**: Add a comment acknowledging this is an incorrect approximation. Consider whether `EffectFilter::AllPermanentsYouControl` would be marginally better (still wrong but less likely to fail with zero creatures).

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 114.1 | Yes | Yes | test_emblem_creation_basic, test_emblem_survives_board_wipe |
| CR 114.2 | Yes | Yes | test_emblem_creation_basic checks owner/controller |
| CR 114.3 | Yes | Partial | Emblem has empty characteristics; no test verifying no types/mana/color explicitly |
| CR 114.4 | Yes | Yes | test_emblem_triggered_ability_fires (structural), test_emblem_static_effect (functional) |
| CR 114.5 | Yes | Yes | test_emblem_not_removed_by_token_sba |
| CR 113.6p | Yes | Yes | Emblem trigger scanning in abilities.rs, but only for SpellCast events |
| CR 113.2c | Partial | Yes | test_multiple_emblems_stack (structural); functional stacking not tested end-to-end |
| CR 800.4a | No (pre-existing) | No | Player loss cleanup does not remove owned objects (pre-existing gap) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| ajani_sleeper_agent | Partial | 3 (+1, -3, Compleated) | Wrong (triggers on any spell, targets any player) | Spell-type filter + opponent targeting missing |
| basri_ket | Partial | 2 (-2 delayed trigger, emblem trigger event) | Wrong (emblem fires on spell cast not combat) | Wrong TriggerEvent::AnySpellCast placeholder |
| kaito_bane_of_nightmares | Yes | 2 (self-animation, 0-ability draw count) | Partial (emblem static +1/+1 correct, Ninjutsu and 0-ability partial) | Emblem portion correct |
| tyvar_kell | Partial | 3 (static mana, untap+deathtouch, Elf filter) | Wrong (starting loyalty 5 vs oracle 3, triggers on any spell) | Loyalty + trigger filter incorrect |
| wrenn_and_realmbreaker | **No** | 4 (static mana, +1 animate, -2 mill, emblem graveyard play) | **Wrong** (mana cost, loyalty, +1 targets, -2 ability all incorrect) | 4 HIGH findings |
| wrenn_and_seven | Partial | 3 (+1 reveal, 0 lands, -8 return permanents) | Wrong (NoMaxHandSize on creatures instead of player) | Emblem target incorrect |
