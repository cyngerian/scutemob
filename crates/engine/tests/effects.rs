//! Tests for the effect execution engine (CR 608.2, M7).
//!
//! Verifies that `execute_effect` correctly applies Effects to game state and
//! emits the right GameEvents. Uses CardRegistry to wire up CardDefinitions
//! so that spell resolution exercises the full pipeline.

use mtg_engine::state::{CardType, ObjectId};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    CombatDamageTarget, Command, Effect, EffectAmount, GameEvent, GameStateBuilder, ManaColor,
    ManaCost, ObjectSpec, PlayerId, PlayerTarget, Step, Target, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all players once (advances the step or resolves stack).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &p in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: p })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", p, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Build a simple Lightning Bolt card definition.
///
/// Effect: Deal 3 damage to any target (player, creature, or planeswalker).
fn lightning_bolt_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("lightning-bolt".to_string()),
        name: "Lightning Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![mtg_engine::TargetRequirement::TargetAny],
            modes: None,
        }],
        ..Default::default()
    }
}

/// Build a simple "Exile target creature; its controller gains life equal to its power."
fn swords_to_plowshares_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("swords-to-plowshares".to_string()),
        name: "Swords to Plowshares".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Exile target creature. Its controller gains life equal to its power."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::ExileObject {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                },
                Effect::GainLife {
                    player: mtg_engine::PlayerTarget::ControllerOf(Box::new(
                        CardEffectTarget::DeclaredTarget { index: 0 },
                    )),
                    amount: EffectAmount::PowerOf(CardEffectTarget::DeclaredTarget { index: 0 }),
                },
            ]),
            targets: vec![mtg_engine::TargetRequirement::TargetCreature],
            modes: None,
        }],
        ..Default::default()
    }
}

/// Build a simple draw-2 card definition ("Draw 2 cards.").
fn divination_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("divination".to_string()),
        name: "Divination".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw 2 cards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
        }],
        ..Default::default()
    }
}

// ── DealDamage effect ─────────────────────────────────────────────────────────

#[test]
/// CR 608.2: Lightning Bolt deals 3 damage to a player target. Player life decreases.
fn test_effect_deal_damage_to_player() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_id = CardId("lightning-bolt".to_string());
    let registry = CardRegistry::new(vec![lightning_bolt_def()]);

    // Build: p1 has Lightning Bolt in hand, p2 is target.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Lightning Bolt")
                .with_card_id(bolt_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    red: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build();

    let initial_p2_life = state.players[&p2].life_total;
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);

    let bolt_card_id = find_object(&state, "Lightning Bolt");

    // Cast Lightning Bolt targeting p2.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: bolt_card_id,
            targets: vec![Target::Player(p2)],
        },
    )
    .unwrap();

    // Both players pass → spell resolves.
    let (state, events) = pass_all(state, &[p1, p2]);

    // P2's life should be down by 3.
    assert_eq!(
        state.players[&p2].life_total,
        initial_p2_life - 3,
        "Lightning Bolt should deal 3 damage to p2"
    );

    // DamageDealt event should be present.
    let damage_event = events.iter().any(|e| {
        matches!(e, GameEvent::DamageDealt { target, amount, .. }
            if matches!(target, CombatDamageTarget::Player(p) if *p == p2)
            && *amount == 3)
    });
    assert!(
        damage_event,
        "DamageDealt event expected; events: {:?}",
        events
    );
}

#[test]
/// CR 608.2: Lightning Bolt deals 3 damage to a creature, marking damage.
fn test_effect_deal_damage_to_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_id = CardId("lightning-bolt".to_string());
    let registry = CardRegistry::new(vec![lightning_bolt_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Lightning Bolt")
                .with_card_id(bolt_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    red: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(ObjectSpec::creature(p2, "Grizzly Bears", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);

    let bolt_card_id = find_object(&state, "Lightning Bolt");
    let bear_id = find_object(&state, "Grizzly Bears");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: bolt_card_id,
            targets: vec![Target::Object(bear_id)],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // Grizzly Bears should be dead (2/2 takes 3 damage → lethal → CreatureDied via SBA).
    let creature_died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        creature_died,
        "Grizzly Bears should die from 3 damage; events: {:?}",
        events
    );
    // Bear should not be on battlefield.
    assert!(
        !state
            .objects
            .values()
            .any(|o| o.characteristics.name == "Grizzly Bears" && o.zone == ZoneId::Battlefield),
        "Grizzly Bears should not be on battlefield"
    );
    let _ = state;
}

// ── ExileObject effect ────────────────────────────────────────────────────────

#[test]
/// CR 608.2 + Swords to Plowshares: exile creature, controller gains life = power.
fn test_effect_exile_and_gain_life() {
    let p1 = p(1);
    let p2 = p(2);

    let stp_id = CardId("swords-to-plowshares".to_string());
    let registry = CardRegistry::new(vec![swords_to_plowshares_def()]);

    // p1 casts STP targeting p2's 3/3 creature.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Swords to Plowshares")
                .with_card_id(stp_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    white: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(ObjectSpec::creature(p2, "Serra Angel", 4, 4))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build();

    let initial_p2_life = state.players[&p2].life_total;
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);

    let stp_id_card = find_object(&state, "Swords to Plowshares");
    let angel_id = find_object(&state, "Serra Angel");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: stp_id_card,
            targets: vec![Target::Object(angel_id)],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // Serra Angel (4/4) should be exiled.
    let angel_exiled = events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectExiled { .. }));
    assert!(
        angel_exiled,
        "Serra Angel should be exiled; events: {:?}",
        events
    );

    // Angel should not be on battlefield.
    assert!(
        !state
            .objects
            .values()
            .any(|o| o.characteristics.name == "Serra Angel" && o.zone == ZoneId::Battlefield),
        "Serra Angel should not be on battlefield"
    );

    // P2 (angel's controller) should have gained 4 life (power = 4).
    // After exile, P2's life should be initial + 4.
    assert_eq!(
        state.players[&p2].life_total,
        initial_p2_life + 4,
        "P2 should gain 4 life from STP (angel power = 4)"
    );
}

// ── DrawCards effect ──────────────────────────────────────────────────────────

#[test]
/// CR 121: Divination draws 2 cards for the controller.
fn test_effect_draw_cards() {
    let p1 = p(1);
    let p2 = p(2);

    let div_id = CardId("divination".to_string());
    let registry = CardRegistry::new(vec![divination_def()]);

    // Give p1 a library with 5 cards.
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Divination")
                .with_card_id(div_id.clone())
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    blue: 1,
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    for i in 0..5 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }
    let state = builder.build();

    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    // Pay 3 mana (any colors for generic — give blue + 2 generic by just adding 3 blue).
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 3);

    let div_card_id = find_object(&state, "Divination");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: div_card_id,
            targets: vec![],
        },
    )
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // p1 should have drawn 2 cards.
    let cards_drawn = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        cards_drawn, 2,
        "Divination should draw 2 cards; events: {:?}",
        events
    );

    let final_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    // Net change: -1 (Divination cast) + 2 (drawn) = +1 net.
    assert_eq!(
        final_hand_size,
        initial_hand_size - 1 + 2,
        "Hand should have grown by 1 net (cast 1, drew 2)"
    );
}

// ── Nothing effect ────────────────────────────────────────────────────────────

#[test]
/// Effect::Nothing produces no events and doesn't change life totals.
fn test_effect_nothing() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let state_builder = GameStateBuilder::new().add_player(p1).build();
    let mut state = state_builder;
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let events = execute_effect(&mut state, &Effect::Nothing, &mut ctx);
    assert!(events.is_empty(), "Effect::Nothing should emit no events");
}

// ── Sequence effect ───────────────────────────────────────────────────────────

#[test]
/// Effect::Sequence executes all sub-effects in order.
fn test_effect_sequence() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .build();
    let source = ObjectId(0);

    // Sequence: gain 5 life then lose 2 life → net +3.
    let effect = Effect::Sequence(vec![
        Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(5),
        },
        Effect::LoseLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(2),
        },
    ]);

    let initial_life = state.players[&p1].life_total;
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "Sequence(GainLife 5, LoseLife 2) should net +3 life"
    );
    let gained = events.iter().any(
        |e| matches!(e, GameEvent::LifeGained { player, amount } if *player == p1 && *amount == 5),
    );
    let lost = events.iter().any(
        |e| matches!(e, GameEvent::LifeLost { player, amount } if *player == p1 && *amount == 2),
    );
    assert!(gained && lost, "Expected LifeGained and LifeLost events");
}

// ── Conditional effect ────────────────────────────────────────────────────────

#[test]
/// Effect::Conditional executes if_true when condition holds.
fn test_effect_conditional_true() {
    use mtg_engine::cards::card_definition::Condition;
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let mut state = GameStateBuilder::new().add_player(p1).build();
    let source = ObjectId(0);

    // Condition: controller has 40+ life (true for Commander). Gain 3 life.
    let effect = Effect::Conditional {
        condition: Condition::ControllerLifeAtLeast(40),
        if_true: Box::new(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(3),
        }),
        if_false: Box::new(Effect::Nothing),
    };

    let initial_life = state.players[&p1].life_total;
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "Conditional should execute if_true when condition holds"
    );
}

#[test]
/// Effect::Conditional executes if_false when condition is false.
fn test_effect_conditional_false() {
    use mtg_engine::cards::card_definition::Condition;
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let mut state = GameStateBuilder::new().add_player(p1).build();
    // Set life below threshold.
    state.players.get_mut(&p1).unwrap().life_total = 20;
    let source = ObjectId(0);

    // Condition: 40+ life (false now). Lose 3 life if false.
    let effect = Effect::Conditional {
        condition: Condition::ControllerLifeAtLeast(40),
        if_true: Box::new(Effect::Nothing),
        if_false: Box::new(Effect::LoseLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(3),
        }),
    };

    let initial_life = state.players[&p1].life_total;
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players[&p1].life_total,
        initial_life - 3,
        "Conditional should execute if_false when condition is false"
    );
}
