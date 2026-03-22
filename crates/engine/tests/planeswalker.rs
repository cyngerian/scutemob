//! Planeswalker framework tests: loyalty abilities, ETB counters, SBA, combat targeting.
//!
//! CR 306 (Planeswalkers), CR 606 (Loyalty Abilities), CR 704.5i (0-loyalty SBA).

use mtg_engine::{check_and_apply_sbas, *};

fn test_pw_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-pw".to_string()),
        name: "Test Planeswalker".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            supertypes: im::OrdSet::new(),
            card_types: im::ordset![CardType::Planeswalker],
            subtypes: im::OrdSet::new(),
        },
        oracle_text: String::new(),
        abilities: vec![
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                },
                targets: vec![],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                targets: vec![],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![],
            },
        ],
        starting_loyalty: Some(4),
        ..Default::default()
    }
}

fn build_pw_state() -> (GameState, ObjectId, PlayerId) {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![test_pw_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Planeswalker")
                .with_card_id(CardId("test-pw".to_string()))
                .with_types(vec![CardType::Planeswalker])
                .with_counter(CounterType::Loyalty, 4)
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pw_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Test Planeswalker")
        .unwrap()
        .id;

    (state, pw_id, p1)
}

/// Helper: pass priority for 2 players to resolve the stack.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<rules::GameEvent>) {
    let mut s = state;
    let mut events = Vec::new();
    for &p in players {
        let (ns, e) =
            rules::process_command(s, rules::Command::PassPriority { player: p }).unwrap();
        s = ns;
        events.extend(e);
    }
    (s, events)
}

/// CR 704.5i: Planeswalker with 0 loyalty counters → owner's graveyard (SBA).
#[test]
fn test_planeswalker_zero_loyalty_sba_cr704_5i() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::planeswalker(p1, "Zero Loyalty PW", 0)
                .with_counter(CounterType::Loyalty, 0)
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Run SBAs directly.
    let _sba_events = check_and_apply_sbas(&mut state);

    let in_gy = state.objects.values().any(|o| {
        o.characteristics.name == "Zero Loyalty PW" && matches!(o.zone, ZoneId::Graveyard(_))
    });
    assert!(
        in_gy,
        "Planeswalker with 0 loyalty should be in graveyard (CR 704.5i)"
    );
}

/// CR 306.8: Damage dealt to a planeswalker removes loyalty counters.
#[test]
fn test_planeswalker_damage_removes_loyalty_cr306_8() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::planeswalker(p1, "Damaged PW", 5)
                .with_counter(CounterType::Loyalty, 5)
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pw_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Damaged PW")
        .unwrap()
        .id;

    let mut state = state;
    let mut ctx = effects::EffectContext::new(p1, pw_id, vec![]);
    let effect = Effect::DealDamage {
        target: CardEffectTarget::Source,
        amount: EffectAmount::Fixed(3),
    };
    let _events = effects::execute_effect(&mut state, &effect, &mut ctx);

    let pw = state.objects.get(&pw_id).unwrap();
    assert_eq!(
        pw.counters.get(&CounterType::Loyalty).copied().unwrap_or(0),
        2,
        "Planeswalker should have 2 loyalty after taking 3 damage (CR 306.8)"
    );
}

/// CR 606.4: +N loyalty ability adds N loyalty counters as cost.
#[test]
fn test_loyalty_plus_cost_adds_counters_cr606_4() {
    let (state, pw_id, p1) = build_pw_state();

    let (state2, _) = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 0, // +1
            targets: vec![],
            x_value: None,
        },
    )
    .unwrap();

    let pw = state2.objects.get(&pw_id).unwrap();
    assert_eq!(
        pw.counters.get(&CounterType::Loyalty).copied().unwrap_or(0),
        5, // 4 + 1
        "Loyalty should be 5 after +1 activation (CR 606.4)"
    );
}

/// CR 606.4: -N loyalty ability removes N loyalty counters as cost.
#[test]
fn test_loyalty_minus_cost_removes_counters_cr606_4() {
    let (state, pw_id, p1) = build_pw_state();

    let (state2, _) = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 1, // -3
            targets: vec![],
            x_value: None,
        },
    )
    .unwrap();

    let pw = state2.objects.get(&pw_id).unwrap();
    assert_eq!(
        pw.counters.get(&CounterType::Loyalty).copied().unwrap_or(0),
        1, // 4 - 3
        "Loyalty should be 1 after -3 activation (CR 606.4)"
    );
}

/// CR 606.3: Only one loyalty ability per permanent per turn.
#[test]
fn test_loyalty_once_per_turn_cr606_3() {
    let (state, pw_id, p1) = build_pw_state();

    // Activate +1
    let (state2, _) = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    )
    .unwrap();

    // Second activation should fail
    let result = rules::process_command(
        state2,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 2, // 0-cost
            targets: vec![],
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "Second loyalty ability activation should fail (CR 606.3)"
    );
}

/// CR 606.3: Must be main phase.
#[test]
fn test_loyalty_sorcery_speed_cr606_3() {
    let (mut state, pw_id, p1) = build_pw_state();
    state.turn.step = Step::BeginningOfCombat;

    let result = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "Loyalty ability should fail outside main phase (CR 606.3)"
    );
}

/// CR 606.6: Can't activate -N if not enough loyalty.
#[test]
fn test_loyalty_insufficient_counters_cr606_6() {
    let (state, pw_id, p1) = build_pw_state(); // 4 loyalty

    // -3 costs 3 which is fine, but try ability_index 1 with only 4 loyalty
    // and it should work since 4 >= 3. Let me test with a custom setup instead.
    let pw_def = CardDefinition {
        card_id: CardId("big-minus-pw".to_string()),
        name: "Big Minus PW".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            supertypes: im::OrdSet::new(),
            card_types: im::ordset![CardType::Planeswalker],
            subtypes: im::OrdSet::new(),
        },
        oracle_text: String::new(),
        abilities: vec![AbilityDefinition::LoyaltyAbility {
            cost: LoyaltyCost::Minus(10),
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(5),
            },
            targets: vec![],
        }],
        starting_loyalty: Some(3),
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![pw_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Big Minus PW")
                .with_card_id(CardId("big-minus-pw".to_string()))
                .with_types(vec![CardType::Planeswalker])
                .with_counter(CounterType::Loyalty, 3)
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pw2_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Big Minus PW")
        .unwrap()
        .id;

    let result = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw2_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "Should fail: only 3 loyalty but -10 needed (CR 606.6)"
    );
}

/// CR 606.3: Stack must be empty.
#[test]
fn test_loyalty_needs_empty_stack_cr606_3() {
    let (mut state, pw_id, p1) = build_pw_state();

    // Put a dummy object on the stack.
    let dummy_id = state.next_object_id();
    state.stack_objects.push_back(state::stack::StackObject {
        id: dummy_id,
        controller: p1,
        kind: state::stack::StackObjectKind::ActivatedAbility {
            source_object: pw_id,
            ability_index: 0,
            embedded_effect: None,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        // CR 715.3d: test objects are not adventure casts.
        was_cast_as_adventure: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
    });

    let result = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "Loyalty ability should fail with non-empty stack (CR 606.3)"
    );
}

/// LoyaltyCost::Zero doesn't change counters.
#[test]
fn test_loyalty_zero_cost() {
    let (state, pw_id, p1) = build_pw_state();

    let (state2, _) = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 2, // Zero cost
            targets: vec![],
            x_value: None,
        },
    )
    .unwrap();

    let pw = state2.objects.get(&pw_id).unwrap();
    assert_eq!(
        pw.counters.get(&CounterType::Loyalty).copied().unwrap_or(0),
        4,
        "Zero cost should not change loyalty counters"
    );
}

/// CR 606.3 + turn boundary: loyalty_ability_activated_this_turn resets each turn.
#[test]
fn test_loyalty_resets_on_turn_boundary() {
    let (state, pw_id, p1) = build_pw_state();

    let (state2, _) = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    )
    .unwrap();

    assert!(
        state2
            .objects
            .get(&pw_id)
            .unwrap()
            .loyalty_ability_activated_this_turn
    );

    let mut state3 = state2;
    rules::turn_actions::reset_turn_state(&mut state3, p1);

    assert!(
        !state3
            .objects
            .get(&pw_id)
            .unwrap()
            .loyalty_ability_activated_this_turn,
        "Flag should be cleared after turn reset (CR 606.3)"
    );
}

/// Loyalty ability resolves and executes its effect.
#[test]
fn test_loyalty_ability_resolves_effect() {
    let (state, pw_id, p1) = build_pw_state();
    let p2 = PlayerId(2);

    let initial_life = state.players.get(&p1).unwrap().life_total;

    // Activate +1 (gains 5 life on resolve)
    let (state2, _) = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    )
    .unwrap();

    // Resolve by passing priority
    let (state3, _) = pass_all(state2, &[p1, p2]);

    let final_life = state3.players.get(&p1).unwrap().life_total;
    assert_eq!(
        final_life,
        initial_life + 5,
        "Loyalty ability should resolve and gain 5 life"
    );
}

/// CR 306.8: Combat damage dealt to a planeswalker removes loyalty counters, not marked damage.
///
/// A 3/3 attacker with no blockers attacks a planeswalker with 5 loyalty. After the combat
/// damage step, the planeswalker should have 2 loyalty remaining (5 - 3 = 2), not 5 loyalty
/// with 3 damage_marked. This test catches the bug where damage_marked was set instead.
#[test]
fn test_combat_damage_to_planeswalker_removes_loyalty_cr306_8() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Use explicit CounterType::Loyalty counter so the loyalty system tracks via counters,
    // matching how actual planeswalker cards work (via starting_loyalty ETB replacement).
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 3, 3))
        .object(
            ObjectSpec::planeswalker(p2, "Combat Target PW", 5)
                .with_counter(CounterType::Loyalty, 5),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Attacker")
        .unwrap()
        .id;
    let pw_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Combat Target PW")
        .unwrap()
        .id;

    // Declare attacker targeting the planeswalker.
    let (state, _) = rules::process_command(
        state,
        rules::Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Planeswalker(pw_id))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Both players pass through DeclareAttackers → DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // p2 declares no blockers.
    let (state, _) = rules::process_command(
        state,
        rules::Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Both pass → CombatDamage step — damage is applied automatically.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 306.8: Planeswalker should have 5 - 3 = 2 loyalty counters remaining.
    let pw = state.objects.get(&pw_id).unwrap();
    let loyalty = pw.counters.get(&CounterType::Loyalty).copied().unwrap_or(0);
    assert_eq!(
        loyalty, 2,
        "Combat damage should remove loyalty counters (CR 306.8): expected 2, got {}",
        loyalty
    );
    // Sanity check: damage_marked should NOT be used for planeswalkers.
    assert_eq!(
        pw.damage_marked, 0,
        "Planeswalkers should not use damage_marked (CR 306.8); damage_marked = {}",
        pw.damage_marked
    );
}

/// Can't activate loyalty ability on opponent's planeswalker.
#[test]
fn test_loyalty_only_own_planeswalker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![test_pw_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p2, "Test Planeswalker") // P2 controls it
                .with_card_id(CardId("test-pw".to_string()))
                .with_types(vec![CardType::Planeswalker])
                .with_counter(CounterType::Loyalty, 4)
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pw_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Test Planeswalker")
        .unwrap()
        .id;

    let result = rules::process_command(
        state,
        rules::Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "Can't activate loyalty ability on opponent's planeswalker"
    );
}
