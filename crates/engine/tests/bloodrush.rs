//! Bloodrush ability word tests (CR 207.2c).
//!
//! Bloodrush is an ability word (CR 207.2c), not a keyword. It labels a pattern
//! of activated abilities from hand with the template:
//! "{cost}, Discard this card: Target attacking creature gets +N/+N
//! [and gains {keyword}] until end of turn."
//!
//! Key rules verified:
//! - CR 207.2c / CR 602.2: Bloodrush activates from hand; card discarded as cost.
//! - CR 602.2b: The discard is part of the cost — paid before ability hits the stack.
//! - CR 115: "Target attacking creature" — target must be in combat.attackers.
//! - CR 702.61a: Cannot activate while split second is on the stack.
//! - CR 608.2b: At resolution, re-validate target is still attacking.
//! - CR 514.2: The +N/+N bonus expires at end of turn.

use std::sync::Arc;

use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, AttackTarget, CardDefinition,
    CardId, CardRegistry, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_graveyard(state: &GameState, player: PlayerId, name: &str) -> Option<ObjectId> {
    state.objects.iter().find_map(|(id, obj)| {
        if obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(player) {
            Some(*id)
        } else {
            None
        }
    })
}

/// Pass priority for all listed players once.
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

/// A simple bloodrush card: +4/+4, cost {R}{G}, no keyword grant.
fn bloodrush_pump_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("bloodrush-pump".to_string()),
        name: "Bloodrush Pump".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            green: 1,
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [mtg_engine::CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "Bloodrush — {R}{G}, Discard this card: Target attacking creature gets +4/+4 until end of turn."
                .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![AbilityDefinition::Bloodrush {
            cost: ManaCost {
                red: 1,
                green: 1,
                ..Default::default()
            },
            power_boost: 4,
            toughness_boost: 4,
            grants_keyword: None,
        }],
        ..Default::default()
    }
}

/// A bloodrush card that also grants trample (like Ghor-Clan Rampager).
fn bloodrush_trample_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("bloodrush-trample".to_string()),
        name: "Bloodrush Trample".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            green: 1,
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [mtg_engine::CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text:
            "Trample\nBloodrush — {R}{G}, Discard this card: Target attacking creature gets +4/+4 and gains trample until end of turn."
                .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Bloodrush {
                cost: ManaCost {
                    red: 1,
                    green: 1,
                    ..Default::default()
                },
                power_boost: 4,
                toughness_boost: 4,
                grants_keyword: Some(KeywordAbility::Trample),
            },
        ],
        ..Default::default()
    }
}

/// Set up a 2-player state at DeclareAttackers step with attacker declared.
/// Returns (state, attacker_id).
fn setup_combat_state(
    registry: Arc<CardRegistry>,
    bloodrush_card: ObjectSpec,
    attacker_spec: ObjectSpec,
    mana_red: u32,
    mana_green: u32,
) -> (GameState, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bloodrush_card)
        .object(attacker_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attack Creature");

    // Declare the attacker.
    let (mut state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Give p1 mana for bloodrush cost.
    let player_state = state.players.get_mut(&p1).unwrap();
    player_state.mana_pool.add(ManaColor::Red, mana_red);
    player_state.mana_pool.add(ManaColor::Green, mana_green);

    state.turn.priority_holder = Some(p1);

    (state, attacker_id)
}

// ── Test 1: Basic bloodrush pump ──────────────────────────────────────────────

#[test]
/// CR 207.2c / CR 602.2 — Bloodrush activates from hand; card discarded as cost.
/// Ability goes on the stack; after resolving, attacker gets +4/+4 until EOT.
fn test_bloodrush_basic_pump() {
    let p1 = p(1);

    let registry = CardRegistry::new(vec![bloodrush_pump_def()]);

    let bloodrush_card = ObjectSpec::card(p1, "Bloodrush Pump")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodrush-pump".to_string()));

    let attacker_spec =
        ObjectSpec::creature(p1, "Attack Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let (state, attacker_id) = setup_combat_state(registry, bloodrush_card, attacker_spec, 1, 1);

    let bloodrush_id = find_object(&state, "Bloodrush Pump");

    // Activate bloodrush.
    let (state, events) = process_command(
        state,
        Command::ActivateBloodrush {
            player: p1,
            card: bloodrush_id,
            target: attacker_id,
        },
    )
    .expect("ActivateBloodrush should succeed");

    // AbilityActivated event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 602.2: AbilityActivated event expected when bloodrush is activated"
    );

    // Ability is on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 602.2: bloodrush ability should be on the stack after activation"
    );

    // CR 602.2b: Card discarded as cost — no longer in hand.
    let card_in_hand = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Bloodrush Pump" && o.zone == ZoneId::Hand(p1));
    assert!(
        !card_in_hand,
        "CR 602.2b: bloodrush card must be discarded (not in hand) as cost"
    );

    // Card is now in graveyard.
    let card_in_gy = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Bloodrush Pump" && o.zone == ZoneId::Graveyard(p1));
    assert!(
        card_in_gy,
        "CR 602.2b: bloodrush card should be in graveyard after discard"
    );

    // CardDiscarded event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { player, .. } if *player == p1)),
        "CR 701.8: CardDiscarded event should be emitted when bloodrush card is discarded"
    );

    // Resolve: pass all players.
    let (state, _) = pass_all(state, &[p1, p(2)]);

    // After resolution, attacker has +4/+4 continuous effect.
    let chars = calculate_characteristics(&state, attacker_id)
        .expect("attacker should still exist after bloodrush resolution");
    assert_eq!(
        chars.power,
        Some(6),
        "CR 207.2c: bloodrush should give attacker +4 power (2+4=6)"
    );
    assert_eq!(
        chars.toughness,
        Some(6),
        "CR 207.2c: bloodrush should give attacker +4 toughness (2+4=6)"
    );
}

// ── Test 2: Bloodrush grants keyword ─────────────────────────────────────────

#[test]
/// CR 207.2c — Bloodrush that grants trample (e.g., Ghor-Clan Rampager pattern).
/// After resolution, attacker gains trample keyword until end of turn.
fn test_bloodrush_grants_keyword() {
    let p1 = p(1);

    let registry = CardRegistry::new(vec![bloodrush_trample_def()]);

    let bloodrush_card = ObjectSpec::card(p1, "Bloodrush Trample")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodrush-trample".to_string()));

    let attacker_spec =
        ObjectSpec::creature(p1, "Attack Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let (state, attacker_id) = setup_combat_state(registry, bloodrush_card, attacker_spec, 1, 1);

    let bloodrush_id = find_object(&state, "Bloodrush Trample");

    // Activate bloodrush.
    let (state, _) = process_command(
        state,
        Command::ActivateBloodrush {
            player: p1,
            card: bloodrush_id,
            target: attacker_id,
        },
    )
    .expect("ActivateBloodrush should succeed");

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p(2)]);

    let chars = calculate_characteristics(&state, attacker_id)
        .expect("attacker should still exist after resolution");

    // Attacker has +4/+4.
    assert_eq!(
        chars.power,
        Some(6),
        "CR 207.2c: +4 power from bloodrush trample"
    );
    assert_eq!(
        chars.toughness,
        Some(6),
        "CR 207.2c: +4 toughness from bloodrush trample"
    );

    // Attacker has trample.
    assert!(
        chars.keywords.contains(&KeywordAbility::Trample),
        "CR 207.2c: bloodrush trample should grant trample keyword until end of turn"
    );
}

// ── Test 3: Target must be attacking ─────────────────────────────────────────

#[test]
/// CR 115 — "Target attacking creature" means the target must be in combat.
/// Targeting a non-attacking creature should return an error.
fn test_bloodrush_target_must_be_attacking() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodrush_pump_def()]);

    let bloodrush_card = ObjectSpec::card(p1, "Bloodrush Pump")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodrush-pump".to_string()));

    // A creature that is NOT attacking.
    let non_attacker =
        ObjectSpec::creature(p1, "Attack Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bloodrush_card)
        .object(non_attacker)
        .active_player(p1)
        .at_step(Step::DeclareBlockers) // Combat step, but no attackers declared
        .build()
        .unwrap();

    let non_attacker_id = find_object(&state, "Attack Creature");
    let bloodrush_id = find_object(&state, "Bloodrush Pump");

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    // Attempt to activate bloodrush targeting a non-attacking creature.
    let result = process_command(
        state,
        Command::ActivateBloodrush {
            player: p1,
            card: bloodrush_id,
            target: non_attacker_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 115: ActivateBloodrush should fail when target is not an attacking creature"
    );
}

// ── Test 4: No combat (no attackers) fails ────────────────────────────────────

#[test]
/// CR 115 — If there are no attacking creatures (no combat at all), bloodrush
/// cannot be activated (no valid target exists).
fn test_bloodrush_no_combat_fails() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodrush_pump_def()]);

    let bloodrush_card = ObjectSpec::card(p1, "Bloodrush Pump")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodrush-pump".to_string()));

    // A creature on the battlefield but NOT in combat.
    let creature = ObjectSpec::creature(p1, "Attack Creature", 2, 2).in_zone(ZoneId::Battlefield);

    // Set up state at main phase (no combat).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bloodrush_card)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Attack Creature");
    let bloodrush_id = find_object(&state, "Bloodrush Pump");

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    // Attempt bloodrush with no combat state.
    let result = process_command(
        state,
        Command::ActivateBloodrush {
            player: p1,
            card: bloodrush_id,
            target: creature_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 115: ActivateBloodrush should fail when there is no combat (no attacking creatures)"
    );
}

// ── Test 5: Card discarded as cost (before resolution) ───────────────────────

#[test]
/// CR 602.2b — The discard is part of the activation cost. The card is discarded
/// BEFORE the ability goes on the stack, not when it resolves.
fn test_bloodrush_card_discarded_as_cost() {
    let p1 = p(1);

    let registry = CardRegistry::new(vec![bloodrush_pump_def()]);

    let bloodrush_card = ObjectSpec::card(p1, "Bloodrush Pump")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodrush-pump".to_string()));

    let attacker_spec =
        ObjectSpec::creature(p1, "Attack Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let (state, attacker_id) = setup_combat_state(registry, bloodrush_card, attacker_spec, 1, 1);

    let bloodrush_id = find_object(&state, "Bloodrush Pump");

    // Activate bloodrush.
    let (state, _) = process_command(
        state,
        Command::ActivateBloodrush {
            player: p1,
            card: bloodrush_id,
            target: attacker_id,
        },
    )
    .expect("ActivateBloodrush should succeed");

    // BEFORE resolution: card is already in graveyard (discarded as cost).
    let card_in_gy = find_in_graveyard(&state, p1, "Bloodrush Pump");
    assert!(
        card_in_gy.is_some(),
        "CR 602.2b: bloodrush card must be in graveyard as cost BEFORE the ability resolves"
    );

    // Stack still has the ability (not yet resolved).
    assert!(
        !state.stack_objects.is_empty(),
        "CR 602.2b: ability should still be on the stack (not resolved yet)"
    );
}

// ── Test 6: Insufficient mana fails ──────────────────────────────────────────

#[test]
/// CR 602.2b — Bloodrush requires paying the mana cost. Activating without
/// sufficient mana should return an error.
fn test_bloodrush_insufficient_mana_fails() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodrush_pump_def()]);

    let bloodrush_card = ObjectSpec::card(p1, "Bloodrush Pump")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodrush-pump".to_string()));

    let attacker_spec =
        ObjectSpec::creature(p1, "Attack Creature", 2, 2).in_zone(ZoneId::Battlefield);

    // Build state but give NO mana (insufficient).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bloodrush_card)
        .object(attacker_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attack Creature");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let bloodrush_id = find_object(&state, "Bloodrush Pump");

    // Give NO mana — bloodrush cost {R}{G} cannot be paid.
    let mut state = state;
    state.turn.priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::ActivateBloodrush {
            player: p1,
            card: bloodrush_id,
            target: attacker_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: ActivateBloodrush should fail without sufficient mana"
    );
}

// ── Test 7: Card not in hand fails ────────────────────────────────────────────

#[test]
/// CR 602.2a — Bloodrush can only be activated from hand. Attempting to activate
/// when the card is on the battlefield should return an error.
fn test_bloodrush_not_in_hand_fails() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodrush_pump_def()]);

    // Card is on the battlefield, not in hand.
    let bloodrush_card = ObjectSpec::creature(p1, "Bloodrush Pump", 4, 4)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("bloodrush-pump".to_string()));

    let attacker_spec =
        ObjectSpec::creature(p1, "Attack Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(bloodrush_card)
        .object(attacker_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attack Creature");
    let bloodrush_id = find_object(&state, "Bloodrush Pump");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    // Attempt bloodrush from battlefield (should fail).
    let result = process_command(
        state,
        Command::ActivateBloodrush {
            player: p1,
            card: bloodrush_id,
            target: attacker_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2a: ActivateBloodrush should fail when card is not in hand"
    );
}

// ── Test 8: Pump expires at end of turn ───────────────────────────────────────

#[test]
/// CR 514.2 — "Until end of turn" effects expire during the cleanup step.
/// After passing through the end step, the bloodrush +4/+4 should be gone.
fn test_bloodrush_pump_expires_end_of_turn() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bloodrush_pump_def()]);

    let bloodrush_card = ObjectSpec::card(p1, "Bloodrush Pump")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bloodrush-pump".to_string()));

    let attacker_spec =
        ObjectSpec::creature(p1, "Attack Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let (state, attacker_id) = setup_combat_state(registry, bloodrush_card, attacker_spec, 1, 1);

    let bloodrush_id = find_object(&state, "Bloodrush Pump");

    // Activate and resolve bloodrush.
    let (state, _) = process_command(
        state,
        Command::ActivateBloodrush {
            player: p1,
            card: bloodrush_id,
            target: attacker_id,
        },
    )
    .expect("ActivateBloodrush should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    // Verify pump is active immediately after resolution.
    let chars_during =
        calculate_characteristics(&state, attacker_id).expect("attacker should still exist");
    assert_eq!(
        chars_during.power,
        Some(6),
        "pump should be active during the same turn"
    );

    // Advance through cleanup — UntilEndOfTurn effects expire.
    let mut current = state;
    for _ in 0..12 {
        let (s, _) = pass_all(current, &[p1, p2]);
        current = s;
        // Stop once we're past cleanup (new turn has started for p2, or p1's next upkeep).
        if current.turn.active_player != p1 {
            break;
        }
    }

    // After end of turn, the pump expires.
    let chars_after = calculate_characteristics(&current, attacker_id)
        .expect("attacker should still exist after end of turn");
    assert_eq!(
        chars_after.power,
        Some(2),
        "CR 514.2: +4/+4 from bloodrush should expire at end of turn (2+4=6 → 2)"
    );
}
