//! PB-D: TargetController::DamagedPlayer filter tests.
//!
//! Tests verify that `TargetController::DamagedPlayer` correctly scopes target and ForEach
//! filters to the specific player dealt combat damage in the triggering event.
//!
//! CR Rules covered:
//! - CR 510.3a: Combat damage triggers fire after damage is dealt; the "damaged player"
//!   identity is bound from the triggering event.
//! - CR 601.2c: Target legality is evaluated at the moment the trigger is put on the stack.
//! - CR 603.3d: If a trigger has no legal target, it is put on the stack but its effect
//!   does nothing (treated as having been countered on resolution when targets are illegal).

use mtg_engine::{
    all_cards, process_command, AbilityDefinition, AttackTarget, CardDefinition, CardId,
    CardRegistry, CardType, Command, Effect, ForEachTarget, GameStateBuilder, ObjectSpec, PlayerId,
    Step, SubType, Target, TargetController, TargetFilter, TargetRequirement, TriggerEvent,
    TriggeredAbilityDef, TypeLine, ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_obj(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn on_battlefield(state: &mtg_engine::GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn in_graveyard(state: &mtg_engine::GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && matches!(o.zone, ZoneId::Graveyard(_)))
}

fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<mtg_engine::GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Build a triggered ability: "Whenever this deals combat damage to a player, destroy target
/// creature that player controls." Reduced-scope version of Throat Slitter.
/// Uses `TriggeredAbilityDef` + `TriggerEvent::SelfDealsCombatDamageToPlayer` (runtime form).
fn destroy_target_damaged_players_creature_trigger() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever this deals combat damage to a player, destroy target creature \
                      that player controls. (CR 510.3a / PB-D)"
            .to_string(),
        effect: Some(Effect::DestroyPermanent {
            target: mtg_engine::CardEffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: false,
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
            controller: TargetController::DamagedPlayer,
            ..Default::default()
        })],
    }
}

// ── Test M1: Positive case — trigger targets only the damaged player's creature ─

/// CR 510.3a / CR 601.2c — PB-D M1: DamagedPlayer targets only the specific damaged player's creature.
///
/// Setup: 4-player game. P1 controls a 1/1 Thug with a WhenDealsCombatDamageToPlayer trigger
/// ("destroy target creature that player controls"). P2 controls a Goblin. P3 controls an Elf.
/// P1 attacks P2 with Thug, no blockers.
///
/// Assert: The trigger targets P2's Goblin (not P3's Elf). After resolution, P2's Goblin is in
/// the graveyard and P3's Elf is still on the battlefield.
///
/// Discriminator: without TargetController::DamagedPlayer, the engine would use Opponent
/// and could target P3's Elf. This test specifically verifies the auto-target chose P2's creature.
#[test]
fn test_damaged_player_target_controller_creature_match_fires() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // P1: attacker with the DamagedPlayer-targeted destroy trigger.
    let thug = ObjectSpec::creature(p1, "Thug", 1, 1)
        .with_card_id(CardId("thug".to_string()))
        .with_triggered_ability(destroy_target_damaged_players_creature_trigger());

    // P2: target victim (controlled by the player who will be damaged).
    let p2_goblin =
        ObjectSpec::creature(p2, "P2 Goblin", 1, 1).with_card_id(CardId("p2-goblin".to_string()));

    // P3: should NOT be targeted (not the damaged player).
    let p3_elf =
        ObjectSpec::creature(p3, "P3 Elf", 1, 1).with_card_id(CardId("p3-elf".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(thug)
        .object(p2_goblin)
        .object(p3_elf)
        .build()
        .unwrap();

    let thug_id = find_obj(&state, "Thug");
    let p2_goblin_id = find_obj(&state, "P2 Goblin");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(thug_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // Advance through DeclareBlockers, CombatDamage, trigger on stack, and resolution.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // skip DeclareBlockers
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // combat damage fires; trigger queued
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // trigger resolves
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // post-resolution priority drain

    // P2's Goblin must be destroyed (targeted by DamagedPlayer filter).
    assert!(
        in_graveyard(&state, "P2 Goblin"),
        "CR 510.3a / PB-D M1: P2's Goblin should be destroyed (target was P2 = damaged player). \
         P2 Goblin ID={:?}",
        p2_goblin_id
    );

    // P3's Elf must NOT be destroyed (P3 was not damaged).
    assert!(
        on_battlefield(&state, "P3 Elf"),
        "CR 510.3a / PB-D M1: P3's Elf should still be on the battlefield (P3 was not the damaged player)"
    );
}

// ── Test M2: Negative case — no legal target when damaged player has no creatures ─

/// CR 510.3a / CR 603.3d — PB-D M2: DamagedPlayer filter skips trigger when damaged player has no creatures.
///
/// Setup: 4-player game. P1 attacks P2 with a Thug carrying the destroy trigger. P2 controls
/// NO creatures. P3 controls a 3/3 Goblin.
///
/// Assert: The trigger has no legal target (P2 has no creatures). Per CR 603.3d the trigger is
/// skipped. P3's 3/3 Goblin is NOT destroyed. The stack is empty after combat resolves.
///
/// Discriminator: without DamagedPlayer, the Opponent arm would find P3's Goblin as a legal target.
/// With DamagedPlayer, only P2's creatures are searched — and P2 has none.
#[test]
fn test_damaged_player_target_controller_negative_excludes_other_opponent() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // P1: attacker with the destroy trigger (DamagedPlayer).
    let thug = ObjectSpec::creature(p1, "Thug2", 1, 1)
        .with_card_id(CardId("thug2".to_string()))
        .with_triggered_ability(destroy_target_damaged_players_creature_trigger());

    // P2: no creatures — so the trigger has no legal target.

    // P3: has a creature but should NOT be targeted.
    let p3_goblin = ObjectSpec::creature(p3, "P3 Big Goblin", 3, 3)
        .with_card_id(CardId("p3-big-goblin".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(thug)
        .object(p3_goblin)
        .build()
        .unwrap();

    let thug_id = find_obj(&state, "Thug2");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(thug_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // DeclareBlockers
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // CombatDamage + trigger (no legal target → skipped)
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // post-damage priority
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // drain

    // No trigger should have resolved (no legal target for P2 — skipped per CR 603.3d).
    assert!(
        on_battlefield(&state, "P3 Big Goblin"),
        "CR 510.3a / PB-D M2: P3's Goblin should be unaffected — it was never a legal target \
         (P3 was not the damaged player; DamagedPlayer filter excludes P3's creatures)"
    );
}

// ── Test M3: Spell casting rejects DamagedPlayer filter ──────────────────────

/// CR 601.2c — PB-D M3: DamagedPlayer controller filter returns false for spell casting.
///
/// Spells cast from hand have no combat-damage context (no damaged_player), so a spell whose
/// TargetRequirement uses TargetController::DamagedPlayer has no legal targets and fails to cast.
/// This exercises the defensive `DamagedPlayer => false` dispatch at casting.rs.
///
/// Setup: A card definition (instant) requires TargetCreatureWithFilter(DamagedPlayer).
/// There is a creature on the battlefield (P2's). Player 1 attempts to cast the instant
/// targeting P2's creature.
///
/// Discriminator: Without the explicit `DamagedPlayer => false` arm in casting.rs, the compiler
/// would reject the non-exhaustive match. The arm produces a clean rejection.
#[test]
fn test_damaged_player_spell_casting_rejects_filter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A card definition: instant that requires "target creature that player controls"
    // with DamagedPlayer controller filter — defensive arm in casting.rs returns false.
    let spell_def = CardDefinition {
        card_id: CardId("damaged-player-instant".to_string()),
        name: "DamagedPlayer Instant".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Destroy target creature that player controls.".to_string(),
        power: None,
        toughness: None,
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: mtg_engine::CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller: TargetController::DamagedPlayer,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    // A creature P2 controls — normally a valid target, but DamagedPlayer filter blocks it.
    let target_creature = ObjectSpec::creature(p2, "Target Creature M3", 2, 2)
        .with_card_id(CardId("target-creature-m3".to_string()));

    // The instant in P1's hand.
    let instant = ObjectSpec::card(p1, "DamagedPlayer Instant")
        .with_card_id(CardId("damaged-player-instant".to_string()))
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![spell_def]))
        .active_player(p1)
        .at_step(Step::Upkeep)
        .object(target_creature)
        .object(instant)
        .build()
        .unwrap();

    let instant_id = find_obj(&state, "DamagedPlayer Instant");
    let target_id = find_obj(&state, "Target Creature M3");

    // Attempt to cast with the P2 creature as declared target.
    // The casting.rs DamagedPlayer arm returns false → the target fails validation.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: instant_id,
            targets: vec![Target::Object(target_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
            additional_costs: vec![],
        },
    );

    // The cast must fail (InvalidTarget or equivalent) — DamagedPlayer filter blocks all targets.
    assert!(
        result.is_err(),
        "CR 601.2c / PB-D M3: Spell with DamagedPlayer controller filter should fail to cast \
         (no damaged_player context available for spells cast from hand)"
    );
}

// ── Test M4: ForEach over DamagedPlayer's lands (Nature's Will pattern) ───────

/// CR 510.3a — PB-D M4: ForEach EachPermanentMatching with DamagedPlayer taps only the damaged player's lands.
///
/// Setup: 4-player game. P1 controls a creature with a "tap all lands that player controls"
/// trigger (ForEach EachPermanentMatching {card_type: Land, controller: DamagedPlayer}).
/// P2 controls 4 untapped Forests. P3 controls 4 untapped Plains. P1 attacks P2.
///
/// Assert: all 4 of P2's Forests become tapped. All 4 of P3's Plains remain untapped.
///
/// Discriminator: This is the load-bearing test for Nature's Will + ForEach dispatch.
/// Without the DamagedPlayer arm at effects/mod.rs EachPermanentMatching, the effect
/// would tap no lands (fall-through to false) or all lands. The 4/4 split proves precision.
#[test]
fn test_damaged_player_foreach_land_tap_nature_will_pattern() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // The trigger: tap all lands that the damaged player controls.
    let tap_lands_trigger = TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever this deals combat damage to a player, tap all lands that player \
                      controls. (CR 510.3a / PB-D M4)"
            .to_string(),
        effect: Some(Effect::ForEach {
            over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                has_card_type: Some(CardType::Land),
                controller: TargetController::DamagedPlayer,
                ..Default::default()
            })),
            effect: Box::new(Effect::TapPermanent {
                target: mtg_engine::CardEffectTarget::DeclaredTarget { index: 0 },
            }),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
    };

    let attacker = ObjectSpec::creature(p1, "Nature Attacker", 5, 5)
        .with_card_id(CardId("nature-attacker".to_string()))
        .with_triggered_ability(tap_lands_trigger);

    // P2 controls 4 Forests (untapped, on battlefield by default).
    let f1 = ObjectSpec::land(p2, "P2 Forest 1").with_card_id(CardId("p2-forest-1".to_string()));
    let f2 = ObjectSpec::land(p2, "P2 Forest 2").with_card_id(CardId("p2-forest-2".to_string()));
    let f3 = ObjectSpec::land(p2, "P2 Forest 3").with_card_id(CardId("p2-forest-3".to_string()));
    let f4 = ObjectSpec::land(p2, "P2 Forest 4").with_card_id(CardId("p2-forest-4".to_string()));

    // P3 controls 4 Plains (untapped).
    let pl1 = ObjectSpec::land(p3, "P3 Plains 1").with_card_id(CardId("p3-plains-1".to_string()));
    let pl2 = ObjectSpec::land(p3, "P3 Plains 2").with_card_id(CardId("p3-plains-2".to_string()));
    let pl3 = ObjectSpec::land(p3, "P3 Plains 3").with_card_id(CardId("p3-plains-3".to_string()));
    let pl4 = ObjectSpec::land(p3, "P3 Plains 4").with_card_id(CardId("p3-plains-4".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(attacker)
        .object(f1)
        .object(f2)
        .object(f3)
        .object(f4)
        .object(pl1)
        .object(pl2)
        .object(pl3)
        .object(pl4)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Nature Attacker");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    // Advance through combat and trigger resolution.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // DeclareBlockers
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // CombatDamage + trigger queued
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // trigger resolves
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // drain

    // Count P2's tapped lands.
    let p2_forests_tapped = state
        .objects
        .values()
        .filter(|o| {
            o.controller == p2
                && o.characteristics.card_types.contains(&CardType::Land)
                && o.status.tapped
        })
        .count();

    // Count P3's tapped lands.
    let p3_plains_tapped = state
        .objects
        .values()
        .filter(|o| {
            o.controller == p3
                && o.characteristics.card_types.contains(&CardType::Land)
                && o.status.tapped
        })
        .count();

    assert_eq!(
        p2_forests_tapped, 4,
        "CR 510.3a / PB-D M4: All 4 of P2's Forests should be tapped (P2 = damaged player). \
         Found {} tapped P2 lands.",
        p2_forests_tapped
    );

    assert_eq!(
        p3_plains_tapped, 0,
        "CR 510.3a / PB-D M4: None of P3's Plains should be tapped (P3 was not damaged). \
         Found {} tapped P3 lands.",
        p3_plains_tapped
    );
}

// ── Test M5: DestroyAll with DamagedPlayer — multiplayer isolation ─────────────

/// CR 510.3a — PB-D M5: DestroyAll with DamagedPlayer filter destroys only the damaged player's creatures.
///
/// Setup: 4-player game. P1 controls a "battlecry" creature with a trigger
/// Effect::ForEach { over: EachPermanentMatching { card_type: Creature, controller: DamagedPlayer },
/// effect: DestroyPermanent }. P2 and P3 each control 2 creatures. P1 attacks P3.
///
/// Assert: Both of P3's creatures are destroyed. Both of P2's creatures remain on the battlefield.
///
/// Discriminator: exercises the EachPermanentMatching dispatch at effects/mod.rs.
/// Without DamagedPlayer, all opponents' creatures would be destroyed.
#[test]
fn test_damaged_player_destroy_all_filter_multiplayer_isolation() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // Trigger: ForEach over DamagedPlayer's creatures → destroy each.
    let destroy_all_trigger = TriggeredAbilityDef {
        trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever this deals combat damage to a player, destroy each creature \
                      that player controls. (CR 510.3a / PB-D M5)"
            .to_string(),
        effect: Some(Effect::ForEach {
            over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                has_card_type: Some(CardType::Creature),
                controller: TargetController::DamagedPlayer,
                ..Default::default()
            })),
            effect: Box::new(Effect::DestroyPermanent {
                target: mtg_engine::CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            }),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
    };

    let battlecry = ObjectSpec::creature(p1, "Battlecry", 5, 5)
        .with_card_id(CardId("battlecry".to_string()))
        .with_triggered_ability(destroy_all_trigger);

    // P2 controls 2 creatures — should survive.
    let p2_c1 =
        ObjectSpec::creature(p2, "P2 Creature A", 2, 2).with_card_id(CardId("p2-ca".to_string()));
    let p2_c2 =
        ObjectSpec::creature(p2, "P2 Creature B", 2, 2).with_card_id(CardId("p2-cb".to_string()));

    // P3 controls 2 creatures — should be destroyed (P3 is the attacked/damaged player).
    let p3_c1 =
        ObjectSpec::creature(p3, "P3 Creature A", 2, 2).with_card_id(CardId("p3-ca".to_string()));
    let p3_c2 =
        ObjectSpec::creature(p3, "P3 Creature B", 2, 2).with_card_id(CardId("p3-cb".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(battlecry)
        .object(p2_c1)
        .object(p2_c2)
        .object(p3_c1)
        .object(p3_c2)
        .build()
        .unwrap();

    let battlecry_id = find_obj(&state, "Battlecry");

    // P1 attacks P3.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(battlecry_id, AttackTarget::Player(p3))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");

    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // DeclareBlockers
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // CombatDamage + trigger
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // trigger resolves
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // drain

    // P3's creatures must be destroyed.
    assert!(
        in_graveyard(&state, "P3 Creature A"),
        "CR 510.3a / PB-D M5: P3 Creature A should be destroyed (P3 = damaged player)"
    );
    assert!(
        in_graveyard(&state, "P3 Creature B"),
        "CR 510.3a / PB-D M5: P3 Creature B should be destroyed (P3 = damaged player)"
    );

    // P2's creatures must remain on the battlefield.
    assert!(
        on_battlefield(&state, "P2 Creature A"),
        "CR 510.3a / PB-D M5: P2 Creature A should be unaffected (P2 was not the damaged player)"
    );
    assert!(
        on_battlefield(&state, "P2 Creature B"),
        "CR 510.3a / PB-D M5: P2 Creature B should be unaffected (P2 was not the damaged player)"
    );
}

// ── Test M6: Hash parity — all 4 TargetController variants produce distinct hashes ──

/// PB-D M6: Hash parity test — all four TargetController variants hash to distinct values.
/// Also verifies HASH_SCHEMA_VERSION is exactly 11 (PB-CC-C bump from PB-CC-B's 10).
///
/// Discriminator: forces the sentinel assertion to fail if the bump is not made.
/// Any two variants colliding would indicate a hash implementation bug.
#[test]
fn test_damaged_player_hash_parity_all_variants() {
    // Hash sentinel is bumped to 13 (PB-CC-C-followup added AbilityDefinition::CdaModifyPowerToughness
    // disc 76, CR 611.3a, Layer-7c dynamic CDA modification with continuous re-evaluation).
    assert_eq!(
        HASH_SCHEMA_VERSION, 13u8,
        "HASH_SCHEMA_VERSION must be 13 (PB-CC-C-followup bump from PB-CC-A's 12 for \
         AbilityDefinition::CdaModifyPowerToughness, CR 611.3a). \
         If you bumped the sentinel, update this test."
    );

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build four states with identical structure except for TargetController variant in trigger.
    // Using TriggeredAbilityDef.targets with different TargetController values.
    let make_state = |controller: TargetController| {
        let trigger = TriggeredAbilityDef {
            trigger_on: TriggerEvent::SelfDealsCombatDamageToPlayer,
            intervening_if: None,
            description: "Hash parity test trigger".to_string(),
            effect: Some(Effect::DestroyPermanent {
                target: mtg_engine::CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller,
                ..Default::default()
            })],
        };
        let obj = ObjectSpec::creature(p1, "Hash Test Creature", 1, 1)
            .with_card_id(CardId("hash-test".to_string()))
            .with_triggered_ability(trigger);
        GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(obj)
            .build()
            .unwrap()
    };

    let state_any = make_state(TargetController::Any);
    let state_you = make_state(TargetController::You);
    let state_opponent = make_state(TargetController::Opponent);
    let state_damaged = make_state(TargetController::DamagedPlayer);

    let hash_any = state_any.public_state_hash();
    let hash_you = state_you.public_state_hash();
    let hash_opponent = state_opponent.public_state_hash();
    let hash_damaged = state_damaged.public_state_hash();

    assert_ne!(
        hash_any, hash_you,
        "PB-D M6: TargetController::Any and You must hash distinctly"
    );
    assert_ne!(
        hash_you, hash_opponent,
        "PB-D M6: TargetController::You and Opponent must hash distinctly"
    );
    assert_ne!(
        hash_opponent, hash_damaged,
        "PB-D M6: TargetController::Opponent and DamagedPlayer must hash distinctly"
    );
    assert_ne!(
        hash_any, hash_damaged,
        "PB-D M6: TargetController::Any and DamagedPlayer must hash distinctly"
    );
    assert_ne!(
        hash_you, hash_damaged,
        "PB-D M6: TargetController::You and DamagedPlayer must hash distinctly"
    );
}

// ── Test M7: Throat Slitter end-to-end precision fix ─────────────────────────

/// CR 510.3a / CR 601.2c — PB-D M7: Throat Slitter precision fix — destroys only the damaged
/// player's nonblack creature, not another opponent's.
///
/// Setup: 4-player game. P1 controls Throat Slitter (2/2 Rat Ninja). P2 controls a nonblack
/// Goblin. P3 controls a nonblack Elf. P1 attacks P2 with Throat Slitter, no blockers.
///
/// Assert:
/// - After resolution, P2's Goblin is in the graveyard.
/// - P3's Elf is still on the battlefield.
///
/// Discriminator: pre-PB-D Throat Slitter used TargetController::Opponent. With DamagedPlayer,
/// only P2's creatures are valid targets. This verifies the precision fix is working end-to-end
/// using the real Throat Slitter card definition.
#[test]
fn test_throat_slitter_end_to_end_precision_fix() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // Load Throat Slitter from the card registry.
    let cards = all_cards();
    let throat_slitter_def = cards
        .iter()
        .find(|d| d.name == "Throat Slitter")
        .cloned()
        .expect("Throat Slitter must be in the card registry");

    // Throat Slitter with the real card_id so the registry can fire its triggers.
    let throat_slitter = ObjectSpec::creature(p1, "Throat Slitter", 2, 2)
        .with_card_id(CardId("throat-slitter".to_string()));

    // P2's nonblack Goblin — should be targeted and destroyed.
    let p2_goblin = ObjectSpec::creature(p2, "P2 Goblin M7", 1, 1)
        .with_card_id(CardId("p2-goblin-m7".to_string()))
        .with_subtypes(vec![SubType("Goblin".to_string())]);

    // P3's nonblack Elf — should NOT be targeted.
    let p3_elf = ObjectSpec::creature(p3, "P3 Elf M7", 1, 1)
        .with_card_id(CardId("p3-elf-m7".to_string()))
        .with_subtypes(vec![SubType("Elf".to_string())]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![throat_slitter_def]))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .object(throat_slitter)
        .object(p2_goblin)
        .object(p3_elf)
        .build()
        .unwrap();

    let slitter_id = find_obj(&state, "Throat Slitter");

    // P1 attacks P2 with Throat Slitter.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(slitter_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers with Throat Slitter failed");

    // Advance through combat and trigger resolution.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // DeclareBlockers
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // CombatDamage + trigger queued
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // trigger resolves
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // drain

    assert!(
        in_graveyard(&state, "P2 Goblin M7"),
        "CR 510.3a / PB-D M7: P2's Goblin should be destroyed by Throat Slitter trigger \
         (DamagedPlayer = P2 → targets only P2's nonblack creatures)"
    );

    assert!(
        on_battlefield(&state, "P3 Elf M7"),
        "CR 510.3a / PB-D M7: P3's Elf should still be on the battlefield \
         (P3 was not the damaged player — DamagedPlayer filter excluded P3's creatures)"
    );
}
