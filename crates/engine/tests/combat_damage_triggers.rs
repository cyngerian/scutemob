//! Combat damage trigger tests (PB-30).
//!
//! Tests cover:
//! - Per-creature "you control" trigger with subtype filter
//! - "One or more" batch trigger — fires once per damaged player per combat step
//! - Equipped creature trigger — fires when attached creature deals damage
//! - Enchanted creature trigger — fires when enchanted creature deals damage
//!
//! CR Rules covered:
//! - CR 510.3a: Combat damage triggers fire after damage is dealt (NOT look-back).
//! - CR 603.2: Trigger fires once per event occurrence.
//! - CR 603.2c: "One or more" batch trigger fires once per damaged player.
//! - CR 603.2g: Fully prevented damage does not trigger.

use mtg_engine::{
    process_command, AttackTarget, CardId, CardRegistry, Command, Effect, EffectAmount, GameEvent,
    GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, SubType,
    TargetFilter, TriggerEvent, TriggeredAbilityDef, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Hand(player))
        .count()
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

fn library_card(player: PlayerId, id: &str, name: &str) -> ObjectSpec {
    ObjectSpec::creature(player, name, 1, 1)
        .in_zone(ZoneId::Library(player))
        .with_card_id(CardId(id.to_string()))
}

/// Build a per-creature "you control" combat damage trigger that draws a card.
fn per_creature_draw() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "Whenever a creature you control deals combat damage to a player, draw a card. (CR 510.3a)".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        targets: vec![],
    }
}

/// Build a per-creature "Ninja you control" combat damage trigger (subtype filter).
fn ninja_combat_damage_draw() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlDealsCombatDamageToPlayer,
        intervening_if: None,
        description:
            "Whenever a Ninja you control deals combat damage to a player, draw a card. (CR 510.3a)"
                .to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: Some(TargetFilter {
            has_subtype: Some(SubType("Ninja".to_string())),
            ..Default::default()
        }),
        targets: vec![],
    }
}

/// Build a "one or more creatures you control" batch combat damage trigger that draws.
fn batch_draw() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::AnyCreatureYouControlBatchCombatDamage,
        intervening_if: None,
        description: "Whenever one or more creatures you control deal combat damage to a player, draw a card. (CR 510.3a, CR 603.2c)".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        targets: vec![],
    }
}

/// Build an equipped creature combat damage trigger that draws a card.
fn equipped_creature_draw() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::EquippedCreatureDealsCombatDamageToPlayer,
        intervening_if: None,
        description:
            "Whenever equipped creature deals combat damage to a player, draw a card. (CR 510.3a)"
                .to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        targets: vec![],
    }
}

/// Build an enchanted creature damage trigger that draws a card.
fn enchanted_creature_draw() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        trigger_on: TriggerEvent::EnchantedCreatureDealsDamageToPlayer,
        intervening_if: None,
        description:
            "Whenever enchanted creature deals damage to a player, draw a card. (CR 510.3a)"
                .to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        targets: vec![],
    }
}

// ── Tests: AnyCreatureYouControlDealsCombatDamageToPlayer ─────────────────────

/// CR 510.3a: Per-creature combat damage trigger fires when your creature deals damage.
#[test]
fn test_per_creature_you_control_combat_damage_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let piracy = ObjectSpec::creature(p1, "Piracy", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("piracy".to_string()))
        .with_triggered_ability(per_creature_draw());
    let attacker =
        ObjectSpec::creature(p1, "Attacker", 3, 3).with_card_id(CardId("attacker".to_string()));
    let lib1 = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(piracy)
        .object(attacker)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Attacker");
    let initial_hand = hand_count(&state, p1);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Pass through blockers, damage, trigger resolution
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        hand_count(&state, p1) > initial_hand,
        "Per-creature combat damage trigger should fire; hand was {} now {}",
        initial_hand,
        hand_count(&state, p1)
    );
}

/// CR 510.3a / CR 603.2c: Per-creature trigger fires once per dealing creature.
#[test]
fn test_per_creature_triggers_fire_per_creature() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let piracy = ObjectSpec::creature(p1, "Piracy", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("piracy".to_string()))
        .with_triggered_ability(per_creature_draw());
    let a1 = ObjectSpec::creature(p1, "A1", 2, 2).with_card_id(CardId("a1".to_string()));
    let a2 = ObjectSpec::creature(p1, "A2", 2, 2).with_card_id(CardId("a2".to_string()));
    let lib1 = library_card(p1, "lib1", "LibCard1");
    let lib2 = library_card(p1, "lib2", "LibCard2");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(piracy)
        .object(a1)
        .object(a2)
        .object(lib1)
        .object(lib2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let a1_id = find_obj(&state, "A1");
    let a2_id = find_obj(&state, "A2");
    let initial_hand = hand_count(&state, p1);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (a1_id, AttackTarget::Player(p2)),
                (a2_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Need 4 passes: skip blockers, resolve first trigger, resolve second trigger, final priority
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let drawn = hand_count(&state, p1) - initial_hand;
    assert_eq!(
        drawn, 2,
        "Per-creature trigger should fire twice for 2 attackers; drew {}",
        drawn
    );
}

// ── Tests: Subtype filter on combat damage ────────────────────────────────────

/// CR 510.3a: Ninja subtype filter — fires for Ninja, NOT for non-Ninja.
#[test]
fn test_per_creature_subtype_filter_combat_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let filterer = ObjectSpec::creature(p1, "NinjaWatcher", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("ninja-watcher".to_string()))
        .with_triggered_ability(ninja_combat_damage_draw());

    // Ninja attacker — should trigger
    let ninja = ObjectSpec::creature(p1, "NinjaDealsDamage", 2, 2)
        .with_card_id(CardId("ninja".to_string()))
        .with_subtypes(vec![SubType("Ninja".to_string())]);

    // Non-Ninja attacker — should NOT trigger
    let non_ninja =
        ObjectSpec::creature(p1, "NonNinja", 2, 2).with_card_id(CardId("non-ninja".to_string()));

    let lib1 = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(filterer)
        .object(ninja)
        .object(non_ninja)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let ninja_id = find_obj(&state, "NinjaDealsDamage");
    let non_ninja_id = find_obj(&state, "NonNinja");
    let initial_hand = hand_count(&state, p1);

    // Both attack — only Ninja trigger should fire
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (ninja_id, AttackTarget::Player(p2)),
                (non_ninja_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let drawn = hand_count(&state, p1) - initial_hand;
    assert_eq!(
        drawn, 1,
        "Ninja-filter trigger should fire exactly once (for the Ninja only); drew {}",
        drawn
    );
}

// ── Tests: Batch trigger ("one or more") ──────────────────────────────────────

/// CR 510.3a / CR 603.2c: "One or more" batch trigger fires ONCE when multiple creatures deal damage.
#[test]
fn test_batch_one_or_more_trigger_fires_once() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let batch_source = ObjectSpec::creature(p1, "BatchSource", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("batch-source".to_string()))
        .with_triggered_ability(batch_draw());

    let a1 = ObjectSpec::creature(p1, "A1", 2, 2).with_card_id(CardId("a1".to_string()));
    let a2 = ObjectSpec::creature(p1, "A2", 2, 2).with_card_id(CardId("a2".to_string()));
    let a3 = ObjectSpec::creature(p1, "A3", 2, 2).with_card_id(CardId("a3".to_string()));
    let lib1 = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(batch_source)
        .object(a1)
        .object(a2)
        .object(a3)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let a1_id = find_obj(&state, "A1");
    let a2_id = find_obj(&state, "A2");
    let a3_id = find_obj(&state, "A3");
    let initial_hand = hand_count(&state, p1);

    // 3 creatures attack the same player
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (a1_id, AttackTarget::Player(p2)),
                (a2_id, AttackTarget::Player(p2)),
                (a3_id, AttackTarget::Player(p2)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let drawn = hand_count(&state, p1) - initial_hand;
    assert_eq!(
        drawn, 1,
        "Batch trigger should fire exactly ONCE even with 3 creatures attacking the same player; drew {}",
        drawn
    );
}

/// CR 603.2c: Batch trigger fires once PER damaged player (not once total).
#[test]
fn test_batch_trigger_per_damaged_player() {
    // 4-player game: P1 attacks P2 and P3 simultaneously with different creatures.
    // Batch trigger should fire TWICE (once per damaged player).
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let batch_source = ObjectSpec::creature(p1, "BatchSource", 0, 1)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("batch-source".to_string()))
        .with_triggered_ability(batch_draw());

    // Two attackers, one attacking P2, one attacking P3
    let a1 = ObjectSpec::creature(p1, "A1", 3, 3).with_card_id(CardId("a1".to_string()));
    let a2 = ObjectSpec::creature(p1, "A2", 3, 3).with_card_id(CardId("a2".to_string()));
    let lib1 = library_card(p1, "lib1", "LibCard1");
    let lib2 = library_card(p1, "lib2", "LibCard2");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(batch_source)
        .object(a1)
        .object(a2)
        .object(lib1)
        .object(lib2)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let a1_id = find_obj(&state, "A1");
    let a2_id = find_obj(&state, "A2");
    let initial_hand = hand_count(&state, p1);

    // A1 attacks P2, A2 attacks P3
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (a1_id, AttackTarget::Player(p2)),
                (a2_id, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Need 5 passes: skip blockers, damage, resolve first trigger, resolve second trigger, final
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let drawn = hand_count(&state, p1) - initial_hand;
    assert_eq!(
        drawn, 2,
        "Batch trigger should fire TWICE (once per damaged player P2 and P3); drew {}",
        drawn
    );
}

// ── Tests: Equipped creature trigger ─────────────────────────────────────────

/// CR 510.3a: Equipped creature combat damage trigger fires when attached creature attacks.
#[test]
fn test_equipped_creature_combat_damage_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Equipment with the trigger
    let equipment = ObjectSpec::artifact(p1, "TestSword")
        .with_card_id(CardId("test-sword".to_string()))
        .with_triggered_ability(equipped_creature_draw());

    // Creature to equip
    let creature =
        ObjectSpec::creature(p1, "Wielder", 3, 3).with_card_id(CardId("wielder".to_string()));

    let lib1 = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(equipment)
        .object(creature)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let equipment_id = find_obj(&state, "TestSword");
    let creature_id = find_obj(&state, "Wielder");

    // Manually attach equipment to creature
    let state = {
        let mut s = state;
        if let Some(creature_obj) = s.objects.get_mut(&creature_id) {
            creature_obj.attachments = creature_obj
                .attachments
                .clone()
                .into_iter()
                .chain(std::iter::once(equipment_id))
                .collect();
        }
        if let Some(equip_obj) = s.objects.get_mut(&equipment_id) {
            equip_obj.attached_to = Some(creature_id);
        }
        s
    };

    let initial_hand = hand_count(&state, p1);

    // Attack with the equipped creature
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(creature_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        hand_count(&state, p1) > initial_hand,
        "Equipped creature combat damage trigger should fire; hand was {} now {}",
        initial_hand,
        hand_count(&state, p1)
    );
}

/// CR 510.3a: Equipment not attached to a creature does NOT trigger.
#[test]
fn test_equipped_creature_unequipped_no_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Equipment NOT attached to any creature
    let equipment = ObjectSpec::artifact(p1, "UnequippedSword")
        .with_card_id(CardId("unequipped-sword".to_string()))
        .with_triggered_ability(equipped_creature_draw());

    // Separate creature with no equipment
    let creature = ObjectSpec::creature(p1, "BareCreature", 3, 3)
        .with_card_id(CardId("bare-creature".to_string()));

    let lib1 = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(equipment)
        .object(creature)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let creature_id = find_obj(&state, "BareCreature");
    let initial_hand = hand_count(&state, p1);

    // Attack with the unequipped creature
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(creature_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        hand_count(&state, p1),
        initial_hand,
        "Unequipped sword trigger should NOT fire; hand changed from {} to {}",
        initial_hand,
        hand_count(&state, p1)
    );
}

// ── Tests: Enchanted creature trigger ────────────────────────────────────────

/// CR 510.3a: Enchanted creature damage trigger fires when attached creature deals damage.
#[test]
fn test_enchanted_creature_damage_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Aura with the trigger (Curiosity-style)
    let aura = ObjectSpec::enchantment(p1, "TestCuriosity")
        .with_card_id(CardId("test-curiosity".to_string()))
        .with_triggered_ability(enchanted_creature_draw());

    // Creature to be enchanted
    let creature =
        ObjectSpec::creature(p1, "Wielder", 3, 3).with_card_id(CardId("wielder".to_string()));

    let lib1 = library_card(p1, "lib1", "LibCard1");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(aura)
        .object(creature)
        .object(lib1)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let aura_id = find_obj(&state, "TestCuriosity");
    let creature_id = find_obj(&state, "Wielder");

    // Manually attach aura to creature
    let state = {
        let mut s = state;
        if let Some(creature_obj) = s.objects.get_mut(&creature_id) {
            creature_obj.attachments = creature_obj
                .attachments
                .clone()
                .into_iter()
                .chain(std::iter::once(aura_id))
                .collect();
        }
        if let Some(aura_obj) = s.objects.get_mut(&aura_id) {
            aura_obj.attached_to = Some(creature_id);
        }
        s
    };

    let initial_hand = hand_count(&state, p1);

    // Attack with the enchanted creature
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(creature_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        hand_count(&state, p1) > initial_hand,
        "Enchanted creature (Curiosity) trigger should fire; hand was {} now {}",
        initial_hand,
        hand_count(&state, p1)
    );
}
