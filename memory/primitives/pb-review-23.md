# Primitive Batch Review: PB-23 -- Controller-filtered creature triggers

**Date**: 2026-03-23
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 603.2, CR 603.10a, CR 508.1m, CR 510.3a
**Engine files reviewed**: `cards/card_definition.rs`, `state/game_object.rs`, `state/hash.rs`, `rules/abilities.rs`, `testing/replay_harness.rs`, `state/mod.rs`, `cards/helpers.rs`
**Card defs reviewed**: 34 (per WIP list: 23 death triggers updated, 5 attack triggers, 6 combat damage triggers)

## Verdict: needs-fix

Engine infrastructure is solid: the `DeathTriggerFilter` struct, `AnyCreatureDies` dispatch with pre-death controller, `AnyCreatureYouControlAttacks` per-attacker firing, and `AnyCreatureYouControlDealsCombatDamageToPlayer` are all correctly implemented and well-commented. Hash support is complete. The 10 unit tests cover the core positive and negative cases.

However, there are two HIGH findings (Zulaport Cutthroat wrong controller filter, Elas il-Kor missing death trigger entirely), several MEDIUM findings around `exclude_self`/`nontoken_only` not being wired from card defs, a missing type on Enduring Curiosity, stale TODO comments, and a missing test for the pre-death controller edge case.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **MEDIUM** | `replay_harness.rs:2324` | **exclude_self hardcoded false.** `enrich_spec_from_def` always sets `exclude_self: false` even for "another creature" cards. **Fix:** Add logic to detect "another" wording or add an `exclude_self: bool` field to `WheneverCreatureDies` TriggerCondition. |
| 2 | **MEDIUM** | `replay_harness.rs:2325` | **nontoken_only hardcoded false.** `enrich_spec_from_def` always sets `nontoken_only: false` even for "nontoken creature" cards (Grim Haruspex, Midnight Reaper, etc.). **Fix:** Add `nontoken_only: bool` field to `WheneverCreatureDies` TriggerCondition or use a separate mechanism. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | **HIGH** | `zulaport_cutthroat.rs` | **Wrong controller filter.** Oracle: "this creature or another creature **you control** dies." Def uses `controller: None` which fires on ALL creature deaths including opponents'. **Fix:** Change to `controller: Some(TargetController::You)`. |
| 4 | **HIGH** | `elas_il_kor_sadistic_pilgrim.rs` | **Death trigger missing entirely.** WIP claims updated but card def only has ETB trigger + stale TODO. Oracle: "Whenever another creature you control dies, each opponent loses 1 life." **Fix:** Add `AbilityDefinition::Triggered` with `WheneverCreatureDies { controller: Some(TargetController::You) }` and `Effect::ForEach { over: EachOpponent, effect: LoseLife { ... Fixed(1) } }`. |
| 5 | **MEDIUM** | `enduring_curiosity.rs:16` | **Missing Enchantment type.** Oracle: "Enchantment Creature -- Cat Glimmer." Def uses `creature_types(&["Cat", "Glimmer"])` which omits Enchantment. **Fix:** Change to `full_types(&[], &[CardType::Enchantment, CardType::Creature], &["Cat", "Glimmer"])`. |
| 6 | **MEDIUM** | `kolaghan_the_storms_fury.rs` | **Over-triggers on non-Dragons.** Oracle: "Whenever a **Dragon** you control attacks." Def uses `WheneverCreatureYouControlAttacks` with no subtype filter. TODO is documented. No fix needed now (subtype filter is a separate DSL gap), but TODO should explicitly state "over-triggers on non-Dragon attackers." |
| 7 | **MEDIUM** | `utvara_hellkite.rs` | **Over-triggers on non-Dragons.** Same as Finding 6. Oracle: "Whenever a **Dragon** you control attacks." TODO is documented. |
| 8 | **MEDIUM** | `dark_prophecy.rs:13` | **Stale TODO comment.** Says "WheneverCreatureDies is overbroad -- fires on all creature deaths, not just 'a creature you control'" but PB-23 added `controller: Some(You)`. **Fix:** Remove or update the stale TODO. |
| 9 | **MEDIUM** | `moldervine_reclamation.rs` | **Stale TODO (same as 8).** Check and update. |
| 10 | **MEDIUM** | `liliana_dreadhorde_general.rs:18` | **Stale TODO.** Says "WheneverCreatureDies overbroad" but now uses `controller: Some(You)`. **Fix:** Remove stale comment. |
| 11 | **MEDIUM** | `bastion_of_remembrance.rs:37` | **Stale TODO.** Says "WheneverCreatureDies is overbroad (all creatures, not just yours)" but now uses `controller: Some(You)`. **Fix:** Remove stale comment. |
| 12 | **MEDIUM** | `enduring_curiosity.rs:7` | **Stale file-header TODO.** Says "Per-creature combat damage trigger not in DSL" but PB-23 added exactly this. **Fix:** Remove stale header TODO. |
| 13 | **MEDIUM** | `ohran_frostfang.rs:5` | **Stale file-header TODO.** Same as Finding 12. **Fix:** Remove stale header TODO. |
| 14 | **MEDIUM** | `pitiless_plunderer.rs:3-5` | **Stale file-header TODO.** Says "WheneverCreatureDies triggers on ALL creature deaths" but PB-23 added controller filter. The `exclude_self` TODO on line 20 is still valid. **Fix:** Update header to remove stale part; keep exclude_self TODO. |
| 15 | **LOW** | Multiple (coastal_piracy, reconnaissance_mission, bident_of_thassa, toski, ohran_frostfang, enduring_curiosity) | **"To a player" vs "to an opponent" ambiguity.** Oracle for Coastal Piracy says "deals combat damage to **an opponent**" but the trigger fires on damage to any player. In practice harmless (creatures don't deal combat damage to their own controller) but technically imprecise. No fix needed. |
| 16 | **MEDIUM** | `mardu_ascendancy.rs` | **Over-triggers on token attacks.** Oracle: "Whenever a **nontoken** creature you control attacks." Def uses `WheneverCreatureYouControlAttacks` with no nontoken filter. TODO is documented but this produces wrong game state (Goblin tokens from the trigger itself would re-trigger it on future attacks, and any other token attackers trigger it incorrectly). No engine fix available yet; ensure TODO is explicit. |

### Finding Details

#### Finding 3: Zulaport Cutthroat wrong controller filter

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/zulaport_cutthroat.rs:19`
**Oracle**: "Whenever this creature or another creature you control dies, each opponent loses 1 life and you gain 1 life for each opponent that lost life."
**Issue**: The card def uses `TriggerCondition::WheneverCreatureDies { controller: None }` which fires on ANY creature dying, including opponents' creatures. The oracle restricts "another creature" to "you control." The self-death case ("this creature") is covered because when Zulaport itself dies, it was your creature, so `controller: Some(You)` would also match. The plan incorrectly classified Zulaport under "controller: None."
**Fix**: Change line 19 from `controller: None` to `controller: Some(TargetController::You)`.

#### Finding 4: Elas il-Kor missing death trigger

**Severity**: HIGH
**File**: `crates/engine/src/cards/defs/elas_il_kor_sadistic_pilgrim.rs:32-33`
**Oracle**: "Whenever another creature you control dies, each opponent loses 1 life."
**Issue**: The abilities vec contains only `Deathtouch` keyword and the ETB trigger ("Whenever another creature you control enters, you gain 1 life"). The death trigger is entirely absent -- only a stale TODO comment remains. The WIP checklist claims this card was updated.
**Fix**: Add a second `AbilityDefinition::Triggered` entry:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WheneverCreatureDies {
        controller: Some(TargetController::You),
    },
    effect: Effect::ForEach {
        over: ForEachTarget::EachOpponent,
        effect: Box::new(Effect::LoseLife {
            player: PlayerTarget::DeclaredTarget { index: 0 },
            amount: EffectAmount::Fixed(1),
        }),
    },
    intervening_if: None,
    targets: vec![],
},
```
Remove the stale TODO comment.

#### Finding 5: Enduring Curiosity missing Enchantment type

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/enduring_curiosity.rs:16`
**Oracle**: "Enchantment Creature -- Cat Glimmer"
**Issue**: Uses `creature_types(&["Cat", "Glimmer"])` which produces only `CardType::Creature`. The card is also an Enchantment.
**Fix**: Change to `full_types(&[], &[CardType::Enchantment, CardType::Creature], &["Cat", "Glimmer"])`.

## Test Findings

| # | Severity | Description |
|---|----------|-------------|
| 17 | **MEDIUM** | **Missing pre-death controller test.** Plan item 5 (`test_death_trigger_controller_uses_pre_death_controller`) was not implemented. This tests the most dangerous edge case: when a stolen creature dies, the pre-death controller (stealer) should be used for the controller_you filter, not the owner. The engine code correctly uses `death_controller` from the event, but there's no test proving it works. **Fix:** Add a test where P1 steals P2's creature (via direct controller assignment on the ObjectSpec), P1's creature dies, and verify P1's "creature you control dies" trigger fires. |
| 18 | **LOW** | **No APNAP ordering test.** Plan item 11 (`test_death_trigger_multiplayer_apnap`) was not implemented. Tests that multiple players' death triggers are ordered by APNAP. Lower risk since APNAP is tested elsewhere. |

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 603.2 | Yes | Yes | Trigger fires on matching event; per-attacker firing tested |
| CR 603.2c | Yes | Yes | `test_whenever_creature_you_control_attacks_fires_per_creature` |
| CR 603.10a | Yes | Partial | Death trigger look-back tested for basic case; pre-death controller NOT tested (Finding 17) |
| CR 508.1m | Yes | Yes | Attack trigger wired and tested (positive + negative) |
| CR 510.3a | Yes | Yes | Combat damage trigger wired and tested (positive + negative) |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| blood_artist | Yes | 0 | Approximate | DrainLife targets all opponents, oracle targets single player |
| zulaport_cutthroat | **No** | 0 | **Wrong** | controller: None fires on opponent deaths (Finding 3) |
| falkenrath_noble | Yes | 0 | Approximate | Same DrainLife approximation as Blood Artist |
| fecundity | Yes | 1 | Approximate | "that creature's controller" needs ControllerOf |
| cordial_vampire | Yes | 0 | Yes | |
| poison_tip_archer | Yes | 0 | Approximate | Missing exclude_self (Finding 1) |
| elenda_the_dusk_rose | Yes | 2 | Approximate | Missing exclude_self; death token count approximated |
| syr_konrad_the_grim | Yes | 1 | Approximate | Only 1 of 3 triggers implemented |
| vein_ripper | Yes | 1 | Approximate | Ward sacrifice cost not implemented |
| black_market | Yes | 0 | Approximate | Missing second ability (precombat main mana) |
| pitiless_plunderer | Yes | 1 | Approximate | Missing exclude_self |
| grim_haruspex | Yes | 1 | Approximate | Missing nontoken_only + exclude_self |
| midnight_reaper | Yes | 1 | Approximate | Missing nontoken_only |
| dark_prophecy | Yes | 1 (stale) | Yes | Stale TODO (Finding 8) |
| moldervine_reclamation | Yes | 1 (stale) | Yes | Stale TODO (Finding 9) |
| liliana_dreadhorde_general | Yes | 1 (stale) + 1 | Approximate | Stale TODO; missing -4/-9 |
| bastion_of_remembrance | Yes | 1 (stale) | Yes | Stale TODO (Finding 11) |
| morbid_opportunist | Yes | 1 | Approximate | once-per-turn not enforced |
| marionette_apprentice | Yes | 1 | Approximate | Doesn't cover artifact deaths |
| pawn_of_ulamog | Yes | 1 | Approximate | Missing nontoken_only |
| sifter_of_skulls | Yes | 1 | Approximate | Missing nontoken_only + exclude_self |
| skemfar_avenger | Yes | 1 | Approximate | Missing subtype filter (Elf/Berserker) |
| crossway_troublemakers | Yes | 2 | Approximate | Missing subtype filter + pay 2 life cost |
| vindictive_vampire | Yes | 0 | Approximate | Missing exclude_self |
| elas_il_kor | **No** | 1 | **Wrong** | Death trigger missing entirely (Finding 4) |
| the_meathook_massacre | Yes | 1 | Yes | X cost ETB is TODO |
| yahenni_undying_partisan | Yes | 0 | Yes | Correct controller_opponent |
| agent_venom | Yes | 1 | Approximate | Missing nontoken_only + exclude_self |
| vengeful_bloodwitch | Yes | 0 | Yes | |
| beastmaster_ascension | Yes | 1 | Approximate | Missing static +5/+5 ability |
| druids_repository | Yes | 1 | Approximate | Missing mana ability |
| mardu_ascendancy | Yes | 2 | Approximate | Missing nontoken filter + sacrifice ability (Finding 16) |
| kolaghan_the_storms_fury | Yes | 1 | Approximate | Over-triggers non-Dragons (Finding 6) |
| utvara_hellkite | Yes | 1 | Approximate | Over-triggers non-Dragons (Finding 7) |
| coastal_piracy | Yes | 0 | Yes | |
| reconnaissance_mission | Yes | 0 | Yes | |
| bident_of_thassa | Yes | 1 | Approximate | Missing forced-attack activated ability |
| toski_bearer_of_secrets | Yes | 1 | Approximate | Missing must-attack restriction |
| ohran_frostfang | Yes | 1 (stale) + 1 | Approximate | Stale TODO + missing static deathtouch grant |
| enduring_curiosity | **No** | 1 (stale) + 1 | Approximate | Missing Enchantment type (Finding 5); stale TODO |

## Previous Findings (first review)

N/A -- this is the initial review.
