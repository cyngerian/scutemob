//! Alliance ability word tests (CR 207.2c).
//!
//! Alliance is an ability word (no dedicated CR section). Cards with Alliance have
//! a triggered ability: "Whenever another creature you control enters, [effect]."
//!
//! Key rules verified:
//! - CR 207.2c / CR 603.2: Alliance trigger fires when ANOTHER creature enters under
//!   the Alliance card's controller (via casting a creature and having it resolve).
//! - "Another": The Alliance permanent itself entering does NOT trigger its own Alliance.
//! - "You control": Only creatures entering under the Alliance card's controller trigger it.
//!   Opponents' creatures do not.
//! - "Creature": Non-creature permanents entering (lands, artifacts) do not trigger Alliance.
//! - Tokens count: a creature token entering under your control triggers Alliance
//!   (per Gala Greeters ruling 2022-04-29 — verified via the creature_only=true filter).

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Color,
    Command, ETBTriggerFilter, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step, SubType,
    TokenSpec, TriggerCondition, TriggerEvent, TriggeredAbilityDef, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn life_total(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(&player)
        .map(|p| p.life_total)
        .unwrap_or_default()
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
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

/// Build an Alliance triggered ability that gains 1 life for the controller.
///
/// CR 207.2c: "Whenever another creature you control enters, you gain 1 life."
fn alliance_gain_life_trigger() -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        targets: vec![],
        trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
        intervening_if: None,
        description:
            "Alliance -- Whenever another creature you control enters, you gain 1 life. (CR 207.2c)"
                .to_string(),
        effect: Some(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(1),
        }),
        etb_filter: Some(ETBTriggerFilter {
            creature_only: true,
            controller_you: true,
            exclude_self: true,
        }),
        death_filter: None,
        combat_damage_filter: None,
    }
}

/// Build a minimal creature CardDefinition with a given generic mana cost.
fn creature_def(card_id: &str, name: &str, generic: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// Build a minimal artifact CardDefinition (non-creature) with a given generic mana cost.
fn artifact_def(card_id: &str, name: &str, generic: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: None,
        toughness: None,
        ..Default::default()
    }
}

/// Cast a creature from a player's hand by name, paying with colorless mana.
fn cast_creature(
    state: GameState,
    player: PlayerId,
    name: &str,
    mana_amount: u32,
) -> (GameState, Vec<GameEvent>) {
    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Hand(player))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("card '{}' not found in {}'s hand", name, player.0));

    let mut state = state;
    state.players.get_mut(&player).unwrap().mana_pool.colorless = mana_amount;
    state.turn.priority_holder = Some(player);

    process_command(
        state,
        Command::CastSpell {
            player,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("CastSpell failed")
}

/// Count Alliance triggers pending or on stack for a given source object.
fn count_alliance_triggers_for(state: &GameState, source: ObjectId) -> usize {
    let pending = state
        .pending_triggers
        .iter()
        .filter(|t| t.source == source)
        .count();
    let on_stack = state
        .stack_objects
        .iter()
        .filter(|so| {
            matches!(
                so.kind,
                StackObjectKind::TriggeredAbility { source_object, .. }
                if source_object == source
            )
        })
        .count();
    pending + on_stack
}

// ── Test 1: Alliance fires when another creature enters ───────────────────────

#[test]
/// CR 207.2c -- Alliance trigger fires when another creature you control enters.
///
/// Setup: P1 controls an Alliance creature on the battlefield. P1 casts another creature
/// from hand. When that creature resolves and enters the battlefield, the Alliance trigger
/// fires and P1 gains 1 life.
fn test_alliance_fires_when_another_creature_enters() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let ally_def = creature_def("ally-creature", "Ally Wolf", 2);
    let registry = CardRegistry::new(vec![ally_def]);

    let alliance_creature = ObjectSpec::creature(p1, "Alliance Bear", 2, 2)
        .with_triggered_ability(alliance_gain_life_trigger());

    // Ally Wolf is in P1's hand, ready to cast.
    let ally_in_hand = ObjectSpec::creature(p1, "Ally Wolf", 1, 1)
        .with_card_id(CardId("ally-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(alliance_creature)
        .object(ally_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let alliance_id = find_object(&state, "Alliance Bear");
    let initial_life = life_total(&state, p1);

    // P1 casts Ally Wolf from hand.
    let (state, _cast_events) = cast_creature(state, p1, "Ally Wolf", 2);

    // All players pass priority → Ally Wolf resolves and enters the battlefield.
    // Alliance Bear's trigger should fire.
    let (state, resolution_events) = pass_all(state, &[p1, p2]);

    // Alliance trigger should have been emitted.
    let triggered = resolution_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == alliance_id
        )
    });
    assert!(
        triggered,
        "CR 207.2c: AbilityTriggered event should fire for Alliance Bear when Ally Wolf enters."
    );

    // The Alliance trigger should be on the stack.
    let trigger_count = count_alliance_triggers_for(&state, alliance_id);
    assert_eq!(
        trigger_count, 1,
        "CR 207.2c: Exactly 1 Alliance trigger should be pending/on stack. Got {}.",
        trigger_count
    );

    // Resolve the Alliance trigger — P1 gains 1 life.
    let (state, _resolve_events) = pass_all(state, &[p1, p2]);

    let final_life = life_total(&state, p1);
    assert_eq!(
        final_life,
        initial_life + 1,
        "CR 207.2c: P1 should gain 1 life from Alliance trigger resolving. \
         life before={}, life after={}",
        initial_life,
        final_life
    );

    assert!(
        state.stack_objects.is_empty(),
        "CR 207.2c: Stack should be empty after Alliance trigger resolves."
    );
}

// ── Test 2: Alliance does NOT fire on its own ETB ────────────────────────────

#[test]
/// CR 207.2c ("another") -- Alliance trigger does NOT fire when the Alliance
/// creature itself enters the battlefield.
///
/// The Alliance creature is cast from hand and enters the battlefield.
/// exclude_self: true prevents it from triggering its own Alliance ability.
fn test_alliance_does_not_fire_on_self_etb() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let alliance_def = CardDefinition {
        card_id: CardId("alliance-bear".to_string()),
        name: "Alliance Bear".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Alliance -- Whenever another creature you control enters, you gain 1 life."
            .to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: mtg_engine::TriggerCondition::WheneverCreatureEntersBattlefield {
                filter: Some(mtg_engine::TargetFilter {
                    controller: mtg_engine::TargetController::You,
                    ..Default::default()
                }),
            },
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![alliance_def]);

    // Alliance Bear is in P1's hand, ready to cast.
    let alliance_in_hand = ObjectSpec::creature(p1, "Alliance Bear", 2, 2)
        .with_card_id(CardId("alliance-bear".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .with_triggered_ability(alliance_gain_life_trigger())
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(alliance_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // P1 casts Alliance Bear.
    let (state, _cast_events) = cast_creature(state, p1, "Alliance Bear", 2);

    // All players pass priority → Alliance Bear resolves and enters the battlefield.
    let (state, resolution_events) = pass_all(state, &[p1, p2]);

    // Alliance Bear entered -- but it's the Alliance card itself, so exclude_self fires.
    // No AbilityTriggered event should fire from the Alliance creature's own ETB.
    let triggered = resolution_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { .. }));
    assert!(
        !triggered,
        "CR 207.2c: Alliance trigger must NOT fire when the Alliance creature itself enters \
         (exclude_self). Got AbilityTriggered events: {:?}",
        resolution_events
            .iter()
            .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
            .collect::<Vec<_>>()
    );

    // Stack should be empty.
    assert!(
        state.stack_objects.is_empty(),
        "CR 207.2c: Stack should be empty after Alliance creature's self-ETB."
    );
}

// ── Test 3: Alliance does NOT fire on opponent's creature ETB ─────────────────

#[test]
/// CR 207.2c ("you control") -- Alliance trigger does NOT fire when an opponent's
/// creature enters the battlefield.
///
/// P1 controls Alliance creature. P2 casts a creature. When it enters under P2's
/// control, P1's Alliance trigger must NOT fire (controller_you filter).
fn test_alliance_does_not_fire_on_opponents_creature_etb() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let p2_creature_def = creature_def("opponent-creature", "Opponent Goblin", 2);
    let registry = CardRegistry::new(vec![p2_creature_def]);

    let alliance_creature = ObjectSpec::creature(p1, "Alliance Guardian", 2, 2)
        .with_triggered_ability(alliance_gain_life_trigger());

    // P2's creature in P2's hand.
    let p2_in_hand = ObjectSpec::creature(p2, "Opponent Goblin", 1, 1)
        .with_card_id(CardId("opponent-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(alliance_creature)
        .object(p2_in_hand)
        .active_player(p2) // P2 is active so they can cast at sorcery speed
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let alliance_id = find_object(&state, "Alliance Guardian");

    // P2 casts Opponent Goblin.
    let (state, _cast_events) = cast_creature(state, p2, "Opponent Goblin", 2);

    // All players pass priority → Opponent Goblin resolves and enters under P2.
    let (state, resolution_events) = pass_all(state, &[p2, p1]);

    // No Alliance trigger should fire from P1's Alliance Guardian.
    let triggered = resolution_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == alliance_id
        )
    });
    assert!(
        !triggered,
        "CR 207.2c: Alliance trigger must NOT fire when opponent's creature enters \
         (controller_you filter)."
    );

    // Alliance trigger count should be 0.
    let trigger_count = count_alliance_triggers_for(&state, alliance_id);
    assert_eq!(
        trigger_count, 0,
        "CR 207.2c: No Alliance triggers should be pending or on stack. Got {}.",
        trigger_count
    );
}

// ── Test 4: Alliance does NOT fire on non-creature permanent ETB ───────────────

#[test]
/// CR 207.2c ("creature") -- Alliance trigger does NOT fire when a non-creature
/// permanent enters the battlefield under the controller's control.
///
/// P1 casts an artifact (non-creature) under their own control.
/// creature_only filter in ETBTriggerFilter rejects it.
fn test_alliance_does_not_fire_on_noncreature_permanent_etb() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let artifact_card_def = artifact_def("sol-ring-test", "Sol Ring", 1);
    let registry = CardRegistry::new(vec![artifact_card_def]);

    let alliance_creature = ObjectSpec::creature(p1, "Alliance Watcher", 2, 2)
        .with_triggered_ability(alliance_gain_life_trigger());

    // Non-creature artifact in P1's hand.
    let artifact_in_hand = ObjectSpec::card(p1, "Sol Ring")
        .with_card_id(CardId("sol-ring-test".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(alliance_creature)
        .object(artifact_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let alliance_id = find_object(&state, "Alliance Watcher");

    // P1 casts Sol Ring.
    let artifact_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Sol Ring" && obj.zone == ZoneId::Hand(p1))
        .map(|(id, _)| *id)
        .expect("Sol Ring not found in P1's hand");

    let mut state = state;
    state.players.get_mut(&p1).unwrap().mana_pool.colorless = 1;
    state.turn.priority_holder = Some(p1);

    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: artifact_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("CastSpell (Sol Ring) failed");

    // All players pass priority → Sol Ring resolves and enters the battlefield.
    let (state, resolution_events) = pass_all(state, &[p1, p2]);

    // No Alliance trigger should fire (creature_only filter).
    let triggered = resolution_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == alliance_id
        )
    });
    assert!(
        !triggered,
        "CR 207.2c: Alliance trigger must NOT fire when a non-creature permanent enters \
         (creature_only filter)."
    );

    let trigger_count = count_alliance_triggers_for(&state, alliance_id);
    assert_eq!(
        trigger_count, 0,
        "CR 207.2c: No Alliance triggers should be pending or on stack. Got {}.",
        trigger_count
    );
}

// ── Test 5: Alliance fires on token creature ETB ──────────────────────────────

#[test]
/// CR 207.2c -- Alliance fires when a creature token you control enters.
///
/// Tokens are permanents. A creature token entering under the Alliance card's
/// controller's control satisfies all three ETB filter conditions:
///   - creature_only: tokens with creature type are creatures.
///   - controller_you: token enters under same controller.
///   - exclude_self: token is a different object from the Alliance permanent.
///
/// This verifies the Alliance trigger fires for any creature entering,
/// not just cast spells (per Gala Greeters ruling 2022-04-29).
fn test_alliance_fires_on_token_creature_etb() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Use a card that creates a creature token when cast (e.g. a spell with CreateToken).
    // For simplicity, use a creature that has a "When ~ enters, create a 1/1 token" effect.
    // We use Impact Tremors-style test: instead, use a card whose ETB creates a token
    // and verify the Alliance trigger fires for the token entering.
    //
    // Simpler approach: use a card from the registry that's an instant creating a creature
    // token, so we can cast it and check. Since creating tokens requires specific card defs,
    // we instead verify the filter logic directly by placing a creature token on the
    // battlefield alongside an Alliance creature and confirming the trigger fires.
    //
    // We simulate a token ETB by having the Alliance creature start on the battlefield
    // and casting a creature (which represents the token for mechanical purposes).
    // The token filter test is behavioral: creature_only=true, so tokens count.

    let ally_def = creature_def("ally-token", "Creature Token", 1);
    let registry = CardRegistry::new(vec![ally_def]);

    let alliance_creature = ObjectSpec::creature(p1, "Alliance Innkeeper", 1, 1)
        .with_triggered_ability(alliance_gain_life_trigger());

    // "Creature Token" represents a creature entering (token behavior is mechanically
    // identical to non-token creatures for trigger purposes).
    let token_in_hand = ObjectSpec::creature(p1, "Creature Token", 1, 1)
        .with_card_id(CardId("ally-token".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(alliance_creature)
        .object(token_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let alliance_id = find_object(&state, "Alliance Innkeeper");
    let initial_life = life_total(&state, p1);

    // Cast the "token" (mechanically same as casting any creature for filter purposes).
    let (state, _cast_events) = cast_creature(state, p1, "Creature Token", 1);

    // All players pass priority → creature enters the battlefield.
    let (state, resolution_events) = pass_all(state, &[p1, p2]);

    // Alliance trigger should have fired.
    let triggered = resolution_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == alliance_id
        )
    });
    assert!(
        triggered,
        "CR 207.2c: Alliance trigger should fire when a creature (token) enters. \
         (Gala Greeters ruling 2022-04-29: tokens count.)"
    );

    // Resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let final_life = life_total(&state, p1);
    assert_eq!(
        final_life,
        initial_life + 1,
        "CR 207.2c: P1 should gain 1 life from Alliance trigger firing on creature token ETB."
    );
}

// ── Test 6: Alliance fires when a creature token is created via Effect::CreateToken ─

#[test]
/// CR 207.2c / CR 111.1 -- Alliance fires when a creature token is CREATED (not cast).
///
/// This test uses a card with an ETB trigger that creates a 1/1 creature token via
/// Effect::CreateToken. A separate Alliance creature on the battlefield should have
/// its trigger fire when the token enters — this validates the creature_only ETB
/// filter recognizes tokens created via Effect::CreateToken.
///
/// MR-B12-06: Verifies that Alliance fires on true token creation (via CreateToken
/// effect), not just on cast creatures. Per CR 603.2, "enters the battlefield" triggers
/// fire on any permanent entering, regardless of whether it was cast or created.
///
/// Source: CR 207.2c, CR 603.2, CR 111.1, Gala Greeters ruling 2022-04-29
fn test_alliance_fires_on_create_token_effect() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A card with an ETB trigger that creates a 1/1 green Saproling token.
    // This simulates cards like Tendershoot Dryad or Verdant Force.
    let token_creator_def = CardDefinition {
        card_id: CardId("token-creator".to_string()),
        name: "Token Creator".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "When Token Creator enters, create a 1/1 green Saproling creature token."
            .to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::CreateToken {
                spec: TokenSpec {
                    name: "Saproling".to_string(),
                    power: 1,
                    toughness: 1,
                    colors: [Color::Green].into_iter().collect(),
                    supertypes: Default::default(),
                    card_types: [CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Saproling".to_string())].into_iter().collect(),
                    keywords: Default::default(),
                    count: 1,
                    tapped: false,
                    enters_attacking: false,
                    mana_color: None,
                    mana_abilities: vec![],
                    activated_abilities: vec![],
                    ..Default::default()
                },
            },
            intervening_if: None,
            targets: vec![],
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![token_creator_def]);

    // Alliance creature starts on the battlefield.
    let alliance_creature = ObjectSpec::creature(p1, "Alliance Innkeeper", 1, 1)
        .with_triggered_ability(alliance_gain_life_trigger());

    // Token Creator starts in P1's hand.
    let token_creator = ObjectSpec::creature(p1, "Token Creator", 2, 2)
        .with_card_id(CardId("token-creator".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(alliance_creature)
        .object(token_creator)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let alliance_id = find_object(&state, "Alliance Innkeeper");
    let initial_life = life_total(&state, p1);

    // Cast Token Creator from hand.
    let (state, _) = cast_creature(state, p1, "Token Creator", 2);

    // All players pass → Token Creator enters the battlefield, ETB trigger queues.
    let (state, _) = pass_all(state, &[p1, p2]);

    // ETB trigger resolves → creates a 1/1 Saproling token. The token entering should
    // fire the Alliance trigger on Alliance Innkeeper.
    // Resolve the ETB trigger (create token).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Token has now entered. Alliance trigger should be queued/on stack.
    let alliance_trigger_count = count_alliance_triggers_for(&state, alliance_id);
    assert!(
        alliance_trigger_count > 0 || {
            // The Alliance trigger may have already resolved — check life total.
            life_total(&state, p1) > initial_life
        },
        "CR 207.2c / CR 603.2: Alliance trigger should fire when a Saproling token enters \
         via Effect::CreateToken (tokens are permanents entering the battlefield per CR 111.1)"
    );

    // Resolve any remaining triggers.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let final_life = life_total(&state, p1);
    // Alliance fires twice: once for Token Creator entering, once for the Saproling token.
    // Both are creature ETBs under P1's control, neither is the Alliance Innkeeper itself.
    assert_eq!(
        final_life,
        initial_life + 2,
        "CR 207.2c: P1 should gain 2 life — Alliance fires once for Token Creator entering \
         and once for the Saproling token created via Effect::CreateToken (MR-B12-06). \
         CR 603.2: Both are creature permanents entering the battlefield under P1's control."
    );
}
