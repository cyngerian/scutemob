# Primitive Batch Review: PB-30 -- Combat Damage Triggers

**Date**: 2026-03-25
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 510.3a, CR 603.2, CR 603.2c, CR 603.2g, CR 603.10
**Engine files reviewed**: `cards/card_definition.rs`, `state/game_object.rs`, `state/stubs.rs`, `state/stack.rs`, `state/hash.rs`, `effects/mod.rs`, `rules/abilities.rs`, `rules/resolution.rs`, `testing/replay_harness.rs`, `state/builder.rs`
**Card defs reviewed**: 26 (old_gnawbone, the_indomitable, professional_face_breaker, contaminant_grafter, ingenious_infiltrator, prosperous_thief, rakish_heir, stensia_masquerade, alela_cunning_conqueror, curiosity, ophidian_eye, mask_of_memory, sword_of_fire_and_ice, sword_of_body_and_mind, sword_of_sinew_and_steel, sword_of_truth_and_justice, the_reaver_cleaver, lathril_blade_of_the_elves, balefire_dragon, marisi_breaker_of_the_coil, sword_of_feast_and_famine, sword_of_light_and_shadow, sword_of_war_and_peace, grim_hireling, natures_will, sigil_of_sleep) + 6 existing defs updated to struct form (ohran_frostfang, enduring_curiosity, toski_bearer_of_secrets, bident_of_thassa, coastal_piracy, reconnaissance_mission) + edric_spymaster_of_trest (NOT updated)

## Verdict: needs-fix

Five HIGH findings: (1) TriggeredAbilityDef hash missing `combat_damage_filter`; (2) PendingTrigger hash missing `damaged_player` and `combat_damage_amount`; (3) `SelfDealsCombatDamageToPlayer` triggers not populated with combat data, breaking Lathril; (4) Batch trigger dispatch does not check `combat_damage_filter`, causing Prosperous Thief and Alela to trigger on non-matching creatures; (5) Edric card def not updated to use new `WhenAnyCreatureDealsCombatDamageToOpponent`. Three MEDIUM findings.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `hash.rs:1907-1916` | **TriggeredAbilityDef hash missing `combat_damage_filter`.** Field not hashed. **Fix:** Add `self.combat_damage_filter.hash_into(hasher);` after line 1915. |
| 2 | **HIGH** | `hash.rs:1674-1714` | **PendingTrigger hash missing `damaged_player` and `combat_damage_amount`.** Fields not hashed. **Fix:** Add `self.damaged_player.hash_into(hasher);` and `self.combat_damage_amount.hash_into(hasher);` after line 1713 (before closing brace). |
| 3 | **HIGH** | `abilities.rs:4261-4267` | **SelfDealsCombatDamageToPlayer triggers not populated with combat data.** `collect_triggers_for_event` is called but resulting triggers do not get `damaged_player` or `combat_damage_amount` set. Breaks Lathril (`CombatDamageDealt` resolves to 0). **Fix:** Capture `pre_len` before calling `collect_triggers_for_event`, extract `damaged_pid` from `assignment.target`, then set `t.damaged_player = Some(damaged_pid)` and `t.combat_damage_amount = assignment.amount` for all `triggers[pre_len..]`. |
| 4 | **HIGH** | `abilities.rs:4590-4636` | **Batch trigger dispatch does not check `combat_damage_filter`.** PendingTriggers are created without checking if any dealing creature matches the filter. Prosperous Thief (Ninja/Rogue) and Alela (Faerie) trigger on ANY creatures dealing damage. **Fix:** After line 4605, check `trigger_def.combat_damage_filter` against the dealing creatures for this (controller, damaged_pid) pair. Iterate the original `assignments` to find creatures controlled by `controller` that dealt damage to `damaged_pid`, then check each against the filter (using `matches_filter` + `is_token` check). If no creature matches, `continue` (skip this trigger). |
| 5 | MEDIUM | `abilities.rs:4641-4694` | **`combat_only` field on `WhenEnchantedCreatureDealsDamageToPlayer` not enforced.** The field is defined on the TriggerCondition and hashed, but the dispatch always fires from `CombatDamageDealt`. Cards with `combat_only: false` (Curiosity, Ophidian Eye, Sigil of Sleep) should also fire on noncombat damage via `GameEvent::DamageDealt`, but this dispatch path does not exist. **Fix:** In the `GameEvent::DamageDealt` handler (line 4965), add a dispatch for `EnchantedCreatureDealsDamageToPlayer` when the damage target is a player and the source creature has aura attachments. Alternatively, document as known limitation and defer to PB-37, since noncombat creature-to-player damage triggers are a broader gap. |
| 6 | MEDIUM | `effects/mod.rs:5049-5152` | **`matches_filter` does not check `is_token` field.** The `is_token` check on TargetFilter is only done in the `combat_damage_filter` code in abilities.rs:5491, not in the shared `matches_filter` function. If `is_token` is ever used in other filter contexts (ETB, death, effect targets), it will be silently ignored. **Fix:** Add `if filter.is_token { /* check obj.is_token */ }` to `matches_filter`, or document that `is_token` is only valid for `combat_damage_filter` contexts. The former is safer. Note: `matches_filter` takes `Characteristics` not `GameObject`, so `is_token` (a GameObject field) cannot be checked there. The current approach (checking separately) is architecturally correct but should be documented. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 7 | **HIGH** | `edric_spymaster_of_trest.rs` | **Card def not updated to use new trigger.** Oracle: "Whenever a creature deals combat damage to one of your opponents, its controller may draw a card." PB-30 added `WhenAnyCreatureDealsCombatDamageToOpponent` specifically for this card, but the def still has only a TODO comment and empty abilities. **Fix:** Add triggered ability using `TriggerCondition::WhenAnyCreatureDealsCombatDamageToOpponent` with `Effect::DrawCards { player: PlayerTarget::ControllerOf(Box::new(EffectTarget::TriggeringCreature)), count: EffectAmount::Fixed(1) }`. Note: "its controller may draw a card" needs the creature's controller, not Edric's controller, so `PlayerTarget::ControllerOf(TriggeringCreature)` is needed. If that variant doesn't exist, use `PlayerTarget::Controller` as approximation and document. |
| 8 | MEDIUM | `curiosity.rs`, `ophidian_eye.rs` | **Oracle says "opponent" not "player".** Oracle: "deals damage to an opponent" but `WhenEnchantedCreatureDealsDamageToPlayer` fires on damage to ANY player including self. In multiplayer Commander, this matters if damage is redirected. **Fix:** Either add an `opponent_only: bool` field to `WhenEnchantedCreatureDealsDamageToPlayer` (and check in dispatch), or document as known approximation. LOW priority since self-damage-to-player is rare. |
| 9 | MEDIUM | `alela_cunning_conqueror.rs:60-73` | **Goad effect uses `DeclaredTarget { index: 0 }` but `targets` is empty.** The triggered ability has `targets: vec![]` but the effect references `DeclaredTarget { index: 0 }`, which will fail at resolution. **Fix:** Either add `targets: vec![TargetRequirement::TargetCreature]` or use `EffectTarget::Source` with proper ForEach over DamagedPlayer's creatures. Since ForEach with DamagedPlayer is deferred to PB-37, leave the TODO but do NOT use `DeclaredTarget { index: 0 }` without a matching target requirement -- change the effect to a placeholder like `Effect::Sequence(vec![])` or remove the triggered ability until PB-37. |

### Finding Details

#### Finding 1: TriggeredAbilityDef hash missing combat_damage_filter

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:1907-1916`
**CR Rule**: Architecture invariant: all game-state-affecting fields must be hashed
**Issue**: The `TriggeredAbilityDef` hash implementation hashes `trigger_on`, `intervening_if`, `description`, `effect`, `etb_filter`, `death_filter`, `targets` but does NOT hash `combat_damage_filter`. Two TriggeredAbilityDefs that differ only in their combat_damage_filter (e.g., one filtering for Ninjas, one for any creature) will produce identical hashes.
**Fix**: Add `self.combat_damage_filter.hash_into(hasher);` after line 1915 (`self.targets.hash_into(hasher);`).

#### Finding 2: PendingTrigger hash missing combat damage fields

**Severity**: HIGH
**File**: `crates/engine/src/state/hash.rs:1674-1714`
**CR Rule**: Architecture invariant: all game-state-affecting fields must be hashed
**Issue**: PendingTrigger's `damaged_player` and `combat_damage_amount` fields are not included in the hash. Two PendingTriggers in the pending_triggers queue that differ only in which player was damaged (or damage amount) will hash identically. This can cause incorrect loop detection or state comparison failures.
**Fix**: Add `self.damaged_player.hash_into(hasher);` and `self.combat_damage_amount.hash_into(hasher);` after line 1713 (`self.data.hash_into(hasher);`).

#### Finding 3: SelfDealsCombatDamageToPlayer data propagation missing

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:4261-4267`
**CR Rule**: CR 510.3a -- combat damage triggers need damage amount/player data
**Oracle**: Lathril: "Whenever Lathril deals combat damage to a player, create that many 1/1 green Elf Warrior creature tokens."
**Issue**: When `collect_triggers_for_event` is called for `SelfDealsCombatDamageToPlayer` (line 4264), the resulting PendingTriggers are not populated with `damaged_player` or `combat_damage_amount`. This means Lathril's `EffectAmount::CombatDamageDealt` resolves to 0, creating 0 tokens. The plan (Change 13a) prescribed capturing `pre_len` and populating, but this was not implemented.
**Fix**: Before line 4261, add `let pre_len = triggers.len();`. After line 4267, add:
```rust
let CombatDamageTarget::Player(damaged_pid) = &assignment.target else { unreachable!() };
for t in &mut triggers[pre_len..] {
    t.damaged_player = Some(*damaged_pid);
    t.combat_damage_amount = assignment.amount;
    t.entering_object_id = Some(assignment.source);
}
```
The `entering_object_id` is also needed for `EffectTarget::TriggeringCreature` to work on self-damage triggers.

#### Finding 4: Batch trigger filter not checked

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:4590-4636`
**CR Rule**: CR 603.2 -- trigger fires only if event matches trigger condition
**Oracle**: Prosperous Thief: "Whenever one or more **Ninja or Rogue** creatures you control deal combat damage to a player"
**Issue**: The batch trigger dispatch at lines 4590-4636 constructs PendingTriggers by iterating battlefield objects and checking `trigger_def.trigger_on == AnyCreatureYouControlBatchCombatDamage`, but never checks `trigger_def.combat_damage_filter`. Prosperous Thief (filter: Ninja/Rogue) and Alela (filter: Faerie) will trigger even when only non-matching creatures deal combat damage.
**Fix**: After the intervening-if check (line 4604), add a filter check: iterate the original `assignments` to find if at least one creature controlled by `controller` that dealt damage to `damaged_pid` matches `trigger_def.combat_damage_filter`. If the filter is `Some(f)` and no matching creature exists, `continue`. Use `calculate_characteristics` + `matches_filter` + the `is_token` check from the per-creature path.

#### Finding 5: combat_only not enforced for noncombat damage

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/abilities.rs:4641-4694`
**CR Rule**: CR 510.3a, general damage trigger rules
**Oracle**: Curiosity: "Whenever enchanted creature deals damage to an opponent" (includes noncombat)
**Issue**: `WhenEnchantedCreatureDealsDamageToPlayer { combat_only: false }` should fire on both combat AND noncombat damage. Currently, `EnchantedCreatureDealsDamageToPlayer` is only dispatched from `GameEvent::CombatDamageDealt`. The `GameEvent::DamageDealt` handler does not dispatch this event. Cards with `combat_only: false` (Curiosity, Ophidian Eye, Sigil of Sleep) will miss noncombat damage triggers.
**Fix**: Defer to PB-37 and document. The noncombat creature-damage-to-player trigger path requires plumbing through the `DamageDealt` event handler to identify which creature dealt damage and check its attachments. This is a broader infrastructure gap.

#### Finding 6: matches_filter does not check is_token

**Severity**: MEDIUM
**File**: `crates/engine/src/effects/mod.rs:5049-5152`
**Issue**: `is_token` is a `GameObject` field, not a `Characteristics` field, so `matches_filter(&Characteristics, &TargetFilter)` cannot check it. The check is done separately in the `combat_damage_filter` path (abilities.rs:5491). This is architecturally correct but means `is_token` on TargetFilter is silently ignored in all non-combat-damage contexts.
**Fix**: Add a doc comment on the `is_token` field noting it is only checked in the `combat_damage_filter` path in abilities.rs, not in `matches_filter`. Consider renaming or adding a runtime assert in future if `is_token` is used elsewhere.

#### Finding 7: Edric card def not updated

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/edric_spymaster_of_trest.rs`
**Oracle**: "Whenever a creature deals combat damage to one of your opponents, its controller may draw a card."
**Issue**: PB-30 added `TriggerCondition::WhenAnyCreatureDealsCombatDamageToOpponent` and `TriggerEvent::AnyCreatureDealsCombatDamageToOpponent` specifically for Edric. The dispatch code is in abilities.rs:4696-4736. But the card def still has an empty abilities list with a TODO comment. The engine primitive was built but never wired to the card.
**Fix**: Replace the TODO with a triggered ability:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WhenAnyCreatureDealsCombatDamageToOpponent,
    effect: Effect::DrawCards {
        player: PlayerTarget::Controller,  // approximation -- should be creature's controller
        count: EffectAmount::Fixed(1),
    },
    intervening_if: None,
    targets: vec![],
},
```
Note: Edric says "its controller may draw a card" where "its" = the creature, not Edric. In a multiplayer game where an opponent's creature deals damage to another opponent, the creature's controller draws. This needs `PlayerTarget::ControllerOf(TriggeringCreature)`. If that doesn't exist, document as approximation.

#### Finding 8: Curiosity/Ophidian Eye "opponent" vs "player"

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/curiosity.rs:19-21`, `ophidian_eye.rs:20-22`
**Oracle**: "Whenever enchanted creature deals damage to an **opponent**, you may draw a card."
**Issue**: `WhenEnchantedCreatureDealsDamageToPlayer` triggers when the enchanted creature deals damage to ANY player, including the controller themselves (if damage is redirected). In multiplayer Commander this matters. The trigger condition has no `opponent_only` field.
**Fix**: Document as known approximation in the card defs. Add `opponent_only: bool` to `WhenEnchantedCreatureDealsDamageToPlayer` in a future PB (PB-37 or later) and filter in the dispatch. LOW urgency.

#### Finding 9: Alela DeclaredTarget without targets

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/alela_cunning_conqueror.rs:60-73`
**Oracle**: "Whenever one or more Faeries you control deal combat damage to a player, goad target creature that player controls."
**Issue**: The effect uses `EffectTarget::DeclaredTarget { index: 0 }` but `targets: vec![]` is empty. At resolution, this will attempt to read target index 0 from an empty list, producing no valid target and silently failing (or panicking depending on error handling).
**Fix**: Either add `targets: vec![TargetRequirement::TargetCreature]` (which is an approximation since the oracle requires targeting a creature "that player" controls, needing DamagedPlayer-scoped targeting), or replace the effect with `Effect::Sequence(vec![])` as a no-op placeholder with a TODO comment until PB-37 adds DamagedPlayer ForEach support. The current code is actively wrong, not just incomplete.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 510.3a | Yes | Partial | Per-creature, batch, equipped, enchanted all tested; missing CombatDamageDealt amount test, DamagedPlayer test |
| CR 603.2 | Yes | Yes | Per-creature trigger fires when event matches |
| CR 603.2c | Yes | Yes | Batch fires once per damaged player, tested in test_batch_one_or_more_trigger_fires_once and test_batch_trigger_per_damaged_player |
| CR 603.2g | Yes | No | No test for 0-damage-doesn't-trigger |
| CR 603.10 | Yes | Partial | Creature must be on battlefield; checked in dispatch code but no dedicated test |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| old_gnawbone | Yes | 0 | Yes | Per-creature + CombatDamageDealt works |
| the_indomitable | Yes | 1 (cast from GY) | Yes | Trigger correct |
| professional_face_breaker | Yes | 1 (impulse draw) | Yes | Batch trigger correct |
| contaminant_grafter | Yes | 1 (land from hand) | Yes | Batch trigger correct |
| ingenious_infiltrator | Yes | 0 | Yes | Ninja subtype filter |
| prosperous_thief | Yes | 0 | **No** | F4: batch filter not checked |
| rakish_heir | Yes | 0 | Yes | TriggeringCreature correct |
| stensia_masquerade | Yes | 1 (Madness) | Yes | Vampire filter + TriggeringCreature |
| alela_cunning_conqueror | Partial | 2 | **No** | F4: filter not checked; F9: empty targets |
| curiosity | Partial | 0 | Partial | F5: noncombat gap; F8: opponent vs player |
| ophidian_eye | Partial | 0 | Partial | F5: noncombat gap; F8: opponent vs player |
| mask_of_memory | Yes | 0 | Yes | Equipment trigger correct |
| sword_of_fire_and_ice | Yes | 0 | Yes | Equipment trigger + targets |
| sword_of_body_and_mind | Partial | 1 (protection) | Yes | DamagedPlayer + mill |
| sword_of_sinew_and_steel | Yes | 0 | Yes | Equipment + dual targets |
| sword_of_truth_and_justice | Yes | 0 | Yes | Equipment + counter + proliferate |
| the_reaver_cleaver | Partial | 0 | Yes | Missing planeswalker damage trigger |
| lathril_blade_of_the_elves | Yes | 1 (tap Elves) | **No** | F3: self-trigger data not populated |
| balefire_dragon | Yes | 1 (ForEach DamagedPlayer) | N/A | Deferred to PB-37 |
| marisi_breaker_of_the_coil | Yes | 2 (CantCast + goad) | N/A | Deferred to PB-37 |
| sword_of_feast_and_famine | Yes | 0 | Yes | DamagedPlayer discard works |
| sword_of_light_and_shadow | Yes | 0 | Yes | Equipment + GY return |
| sword_of_war_and_peace | Partial | 0 | Partial | Uses TargetPlayer not DamagedPlayer |
| grim_hireling | Yes | 1 (X-cost ability) | Yes | Batch trigger correct |
| natures_will | Partial | 1 (tap DamagedPlayer lands) | Partial | Only untap half implemented |
| sigil_of_sleep | Partial | 0 | Partial | F5: noncombat gap |
| edric_spymaster_of_trest | **No** | 1 | **No** | F7: not updated to use new trigger |

## Test Gaps

| Gap | Severity | Description |
|-----|----------|-------------|
| CombatDamageDealt amount | MEDIUM | No test verifies `EffectAmount::CombatDamageDealt` resolves to the actual damage amount (e.g., Lathril pattern) |
| DamagedPlayer resolution | MEDIUM | No test verifies `PlayerTarget::DamagedPlayer` resolves to the correct player |
| TriggeringCreature resolution | LOW | No test verifies `EffectTarget::TriggeringCreature` resolves to the dealing creature |
| Batch filter | MEDIUM | No test for batch trigger with subtype filter (Prosperous Thief pattern) |
| 0-damage negative | LOW | No test for CR 603.2g (fully prevented damage does not trigger) |
| Opponent filter | LOW | No test for `WhenAnyCreatureDealsCombatDamageToOpponent` (Edric pattern) |
