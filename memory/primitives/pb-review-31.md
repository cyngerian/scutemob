# Primitive Batch Review: PB-31 -- Cost Primitives (RemoveCounter, SpellAdditionalCost)

**Date**: 2026-03-25
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 118.3, CR 118.8, CR 602.2, CR 602.2b, CR 601.2h
**Engine files reviewed**: `cards/card_definition.rs` (Cost::RemoveCounter, SpellAdditionalCost enum, spell_additional_costs field), `state/game_object.rs` (ActivationCost.remove_counter_cost), `state/hash.rs` (Cost discriminant 9, ActivationCost hash, (A,B) tuple impl), `rules/abilities.rs` (counter removal payment), `rules/casting.rs` (spell sacrifice validation + execution), `testing/replay_harness.rs` (flatten_cost_into, cast_spell sacrifice wiring), `cards/mod.rs` + `lib.rs` + `cards/helpers.rs` (exports)
**Card defs reviewed**: 18 (dragons_hoard, spawning_pit, ominous_seas, gemstone_array, golgari_grave_troll, ghave_guru_of_spores, spike_weaver, ramos_dragon_engine, druids_repository, umezawas_jitte, village_rites, deadly_dispute, goblin_grenade, altar_of_bone, crop_rotation, corrupted_conviction, lifes_legacy, abjure)

## Verdict: needs-fix

One MEDIUM oracle text mismatch on Umezawa's Jitte (trigger fires only on player damage, but oracle says any combat damage). One MEDIUM on Life's Legacy producing clearly wrong game state (draws 0 cards). Five LOWs for documented deferrals and pre-existing patterns.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `casting.rs:189-196` | **Shared sacrifice_from_additional_costs is fragile.** All of spell_sac, bargain, and casualty share the same `sacrifice_from_additional_costs` Option. Currently safe due to keyword guard checks, but a card with both `spell_additional_costs` and Bargain keyword would double-sacrifice. **Fix:** Add a comment noting the mutual-exclusion assumption, or skip bargain/casualty when spell_sac_id is Some. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 2 | **MEDIUM** | `umezawas_jitte.rs:29` | **Trigger mismatch.** Oracle says "Whenever equipped creature deals combat damage" (any target). Def uses `WhenEquippedCreatureDealsCombatDamageToPlayer` which only fires on player damage. Misses combat damage to creatures. **Fix:** Need a `WhenEquippedCreatureDealsCombatDamage` variant (no "ToPlayer" qualifier). If variant doesn't exist yet, add TODO citing oracle text. |
| 3 | **MEDIUM** | `lifes_legacy.rs:23` | **Empty abilities vec produces wrong game state.** Oracle: "Draw cards equal to the sacrificed creature's power." Def has `abilities: vec![]` so the spell does nothing on resolution -- caster pays sacrifice + mana for zero effect. **Fix:** Add a placeholder `AbilityDefinition::Spell { effect: Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) }, .. }` as a minimum approximation (draw 1 card) with a TODO for PB-37 to use `EffectAmount::SacrificedCreaturePower`. Drawing 1 is closer to correct than drawing 0. |
| 4 | LOW | `spike_weaver.rs:48` | **Second ability omitted.** "{1}, Remove a +1/+1 counter: Prevent all combat damage this turn" blocked on G-19 (PB-32). Documented. |
| 5 | LOW | `ramos_dragon_engine.rs:26` | **"Whenever you cast a spell" trigger missing.** Deferred to PB-37. Once-per-turn restriction also deferred. Both documented. |
| 6 | LOW | `ghave_guru_of_spores.rs:33-34` | **First ability removes from self only.** Oracle says "from a creature you control." Documented deferral to PB-37. |
| 7 | LOW | `gemstone_array.rs:27`, `druids_repository.rs:27`, `ramos_dragon_engine.rs:29` | **Mana-producing abilities should be mana abilities (CR 605.1).** Implemented as regular activated abilities that use the stack. Pre-existing architectural limitation, documented deferral to PB-37. |

### Finding Details

#### Finding 2: Umezawa's Jitte Trigger Mismatch

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/umezawas_jitte.rs:29`
**Oracle**: "Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte."
**Issue**: The card def uses `TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer`, which only fires when the equipped creature deals combat damage to a player. The oracle text says "deals combat damage" without qualification -- this includes combat damage to creatures (blocking/blocked creatures). In Commander, blocking is common; Jitte should gain counters whenever the equipped creature deals ANY combat damage, including to other creatures.
**Fix**: If a `TriggerCondition::WhenEquippedCreatureDealsCombatDamage` variant exists (no "ToPlayer"), use it. If not, change the trigger to one that fires on any combat damage from the equipped creature, or add a TODO citing the oracle text discrepancy and leave for PB-37.

#### Finding 3: Life's Legacy Empty Abilities

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/lifes_legacy.rs:23`
**Oracle**: "As an additional cost to cast this spell, sacrifice a creature. Draw cards equal to the sacrificed creature's power."
**Issue**: The card def has `abilities: vec![]`, meaning when the spell resolves it does nothing. The caster sacrifices a creature and pays {1}{G} for zero cards drawn. This is clearly wrong game state. While the exact draw count (based on sacrificed creature's power) requires `EffectAmount::SacrificedCreaturePower` (deferred to PB-37), drawing 0 cards is worse than any reasonable approximation.
**Fix**: Add a `AbilityDefinition::Spell` with `Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) }` as a minimum placeholder. Retain the PB-37 TODO for the correct power-based count. This ensures the card at least does something on resolution.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 118.3 (must have resources) | Yes | Yes | test_remove_counter_cost_insufficient, test_remove_counter_cost_exact_zero |
| CR 118.8 (additional costs on spells) | Yes | Yes | test_spell_sacrifice_cost_creature, _missing, _wrong_type; 5 card-def-checks |
| CR 602.2 (activating activated abilities) | Yes | Yes | test_remove_counter_cost_basic |
| CR 601.2h (pay costs simultaneously) | Yes | Yes | test_remove_counter_cost_in_sequence |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| Dragon's Hoard | Yes | 0 | Yes | |
| Spawning Pit | Yes | 0 | Yes | |
| Ominous Seas | Yes | 0 | Yes | |
| Gemstone Array | Yes | 0 | Yes | Mana ability timing gap (LOW) |
| Golgari Grave-Troll | Partial | 0 | Partial | ETB counter count deferred (pre-existing) |
| Ghave, Guru of Spores | Partial | 0 | Partial | Ability 1 removes from self only (LOW) |
| Spike Weaver | Partial | 1 (PB-32) | Partial | Ability 2 omitted |
| Ramos, Dragon Engine | Partial | 1 (PB-37) | Partial | Cast trigger + once-per-turn missing |
| Druids' Repository | Yes | 0 | Yes | Mana ability timing gap (LOW) |
| Umezawa's Jitte | No | 0 | No | Trigger wrong -- MEDIUM |
| Village Rites | Yes | 0 | Yes | |
| Deadly Dispute | Yes | 0 | Yes | |
| Goblin Grenade | Yes | 0 | Yes | |
| Altar of Bone | Yes | 0 | Yes | |
| Crop Rotation | Yes | 0 | Yes | |
| Corrupted Conviction | Yes | 0 | Yes | |
| Life's Legacy | Partial | 2 (PB-37) | No | Empty abilities -- MEDIUM |
| Abjure | Yes | 0 | Yes | |
