//! Tests for the effect execution engine (CR 608.2, M7).
//!
//! Verifies that `execute_effect` correctly applies Effects to game state and
//! emits the right GameEvents. Uses CardRegistry to wire up CardDefinitions
//! so that spell resolution exercises the full pipeline.

use mtg_engine::state::{CardType, ObjectId};
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition,
    CardEffectTarget, CardId, CardRegistry, CombatDamageTarget, Command, Cost, Effect,
    EffectAmount, GameEvent, GameStateBuilder, ManaColor, ManaCost, ObjectSpec, PlayerId,
    PlayerTarget, Step, Target, TriggerEvent, TriggeredAbilityDef, TypeLine, ZoneId,
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
            cant_be_countered: false,
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
            cant_be_countered: false,
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
            cant_be_countered: false,
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
        .build()
        .unwrap();

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
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
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
        .build()
        .unwrap();

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
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
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
        .build()
        .unwrap();

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
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
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
    let state = builder.build().unwrap();

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
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
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
    let state_builder = GameStateBuilder::new().add_player(p1).build().unwrap();
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
        .build()
        .unwrap();
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
    let mut state = GameStateBuilder::new().add_player(p1).build().unwrap();
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

// ── Session 3 fix tests ───────────────────────────────────────────────────────

#[test]
/// MR-M7-01 (CR 400): MoveZone to graveyard emits ObjectPutInGraveyard, not ObjectExiled.
fn test_move_zone_to_graveyard_emits_correct_event() {
    use mtg_engine::cards::card_definition::ZoneTarget;
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::state::targeting::SpellTarget;
    use mtg_engine::state::targeting::Target;

    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Goblin", 1, 1).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let goblin_id = find_object(&state, "Goblin");
    let source = ObjectId(0);

    let effect = Effect::MoveZone {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        to: ZoneTarget::Graveyard {
            owner: PlayerTarget::Controller,
        },
    };

    let mut ctx = EffectContext::new(
        p1,
        source,
        vec![SpellTarget {
            target: Target::Object(goblin_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    let events = execute_effect(&mut state, &effect, &mut ctx);

    let has_put_in_graveyard = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::ObjectPutInGraveyard {
                object_id,
                ..
            } if *object_id == goblin_id
        )
    });
    assert!(
        has_put_in_graveyard,
        "MoveZone to Graveyard should emit ObjectPutInGraveyard, not ObjectExiled"
    );
    let has_exiled = events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectExiled { .. }));
    assert!(
        !has_exiled,
        "MoveZone to Graveyard must NOT emit ObjectExiled"
    );
}

#[test]
/// MR-M7-01 (CR 400): MoveZone to hand emits ObjectReturnedToHand, not ObjectExiled.
fn test_move_zone_to_hand_emits_correct_event() {
    use mtg_engine::cards::card_definition::ZoneTarget;
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::state::targeting::SpellTarget;
    use mtg_engine::state::targeting::Target;

    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p2, "Dragon", 5, 5).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let dragon_id = find_object(&state, "Dragon");
    let source = ObjectId(0);

    // "Return target creature to its controller's hand."
    let effect = Effect::MoveZone {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        to: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
    };

    let mut ctx = EffectContext::new(
        p1,
        source,
        vec![SpellTarget {
            target: Target::Object(dragon_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    let events = execute_effect(&mut state, &effect, &mut ctx);

    let has_returned = events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectReturnedToHand { object_id, .. } if *object_id == dragon_id));
    assert!(
        has_returned,
        "MoveZone to Hand should emit ObjectReturnedToHand"
    );
    let has_exiled = events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectExiled { .. }));
    assert!(!has_exiled, "MoveZone to Hand must NOT emit ObjectExiled");
}

#[test]
/// MR-M7-04 (CR 400): resolve_zone_target uses owner PlayerTarget, not always controller.
/// Verify that Hand { owner: EachOpponent } puts the card in the opponent's hand.
fn test_move_zone_uses_owner_player_target() {
    use mtg_engine::cards::card_definition::ZoneTarget;
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::state::targeting::SpellTarget;
    use mtg_engine::state::targeting::Target;

    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::card(p1, "Token").in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let token_id = find_object(&state, "Token");
    let source = ObjectId(0);

    // Move to opponent's hand (owner: EachOpponent — first opponent is p2).
    let effect = Effect::MoveZone {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        to: ZoneTarget::Hand {
            owner: PlayerTarget::EachOpponent,
        },
    };

    let mut ctx = EffectContext::new(
        p1, // controller is p1
        source,
        vec![SpellTarget {
            target: Target::Object(token_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    execute_effect(&mut state, &effect, &mut ctx);

    // The object should now be in p2's hand, not p1's hand.
    let in_p2_hand = state
        .objects
        .values()
        .any(|obj| obj.zone == ZoneId::Hand(p2));
    assert!(in_p2_hand, "Card should be in opponent (p2)'s hand");

    let in_p1_hand = state
        .objects
        .values()
        .any(|obj| obj.zone == ZoneId::Hand(p1));
    assert!(!in_p1_hand, "Card must NOT be in controller (p1)'s hand");
}

#[test]
/// MR-M7-05: Negative EffectAmount::Fixed doesn't wrap to huge u32 — damage is 0.
fn test_deal_damage_negative_amount_clamped_to_zero() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .build()
        .unwrap();

    let initial_life = state.players[&p2].life_total;
    let source = ObjectId(0);

    // Deal -3 damage (should clamp to 0, no life loss or wrapping).
    let effect = Effect::DealDamage {
        target: CardEffectTarget::Controller, // p1 controls source → player target
        amount: EffectAmount::Fixed(-3),
    };

    // Use p2 as controller so DealDamage { target: Controller } targets p2.
    let mut ctx2 = EffectContext::new(p2, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx2);

    assert_eq!(
        state.players[&p2].life_total, initial_life,
        "Negative damage amount must be clamped to 0, causing no life loss"
    );
}

#[test]
/// MR-M7-06 (CR 608.2): ForEach EachOpponent applies the effect to each opponent.
fn test_foreach_each_opponent_applies_to_all_opponents() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .build()
        .unwrap();

    let p2_initial = state.players[&p2].life_total;
    let p3_initial = state.players[&p3].life_total;
    let p1_initial = state.players[&p1].life_total;
    let source = ObjectId(0);

    // "Each opponent loses 2 life." — inner effect targets DeclaredTarget(0) as player.
    use mtg_engine::cards::card_definition::ForEachTarget;
    let effect = Effect::ForEach {
        over: ForEachTarget::EachOpponent,
        effect: Box::new(Effect::LoseLife {
            player: PlayerTarget::DeclaredTarget { index: 0 },
            amount: EffectAmount::Fixed(2),
        }),
    };

    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.players[&p2].life_total,
        p2_initial - 2,
        "Opponent p2 should lose 2 life"
    );
    assert_eq!(
        state.players[&p3].life_total,
        p3_initial - 2,
        "Opponent p3 should lose 2 life"
    );
    assert_eq!(
        state.players[&p1].life_total, p1_initial,
        "Controller p1 should NOT lose life"
    );
}

#[test]
/// MR-M7-10 (CR 400): SearchLibrary with Battlefield { tapped: true } causes
/// the permanent to enter the battlefield tapped.
fn test_search_library_enters_battlefield_tapped() {
    use mtg_engine::cards::card_definition::{TargetFilter, ZoneTarget};
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::creature(p1, "Forest Bear", 2, 2).in_zone(ZoneId::Library(p1)))
        .build()
        .unwrap();

    let source = ObjectId(0);

    // SearchLibrary → put onto battlefield tapped.
    let effect = Effect::SearchLibrary {
        player: PlayerTarget::Controller,
        filter: TargetFilter::default(),
        reveal: false,
        destination: ZoneTarget::Battlefield { tapped: true },
    };

    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let bear = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Forest Bear" && obj.zone == ZoneId::Battlefield)
        .expect("Forest Bear should be on the battlefield");
    assert!(
        bear.status.tapped,
        "Permanent should enter the battlefield tapped when ZoneTarget::Battlefield {{ tapped: true }}"
    );
}

#[test]
/// Effect::Conditional executes if_false when condition is false.
fn test_effect_conditional_false() {
    use mtg_engine::cards::card_definition::Condition;
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);
    let mut state = GameStateBuilder::new().add_player(p1).build().unwrap();
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

// ── CreateToken effect ─────────────────────────────────────────────────────────

#[test]
/// CR 701.6 — CreateToken puts a token on the battlefield and emits TokenCreated.
///
/// MR-M7-15: unit test for CreateToken effect.
fn test_effect_create_token_enters_battlefield() {
    use mtg_engine::cards::card_definition::TokenSpec;
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::state::{CardType, Color, SubType};

    let p1 = p(1);
    let mut state = GameStateBuilder::new().add_player(p1).build().unwrap();
    let source = ObjectId(0);

    // 3/3 green Beast token.
    let spec = TokenSpec {
        name: "Beast".to_string(),
        power: 3,
        toughness: 3,
        colors: [Color::Green].into_iter().collect(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [SubType("Beast".to_string())].into_iter().collect(),
        count: 1,
        tapped: false,
        ..Default::default()
    };

    let effect = Effect::CreateToken { spec };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Token appears on battlefield.
    let on_bf: Vec<_> = state
        .objects
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Battlefield && obj.characteristics.name == "Beast")
        .collect();
    assert_eq!(on_bf.len(), 1, "one Beast token should be on battlefield");
    let token_id = *on_bf[0].0;
    assert!(
        on_bf[0].1.is_token,
        "created object should be flagged as a token"
    );

    // TokenCreated event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TokenCreated { player, object_id } if *player == p1 && *object_id == token_id)),
        "expected TokenCreated event"
    );
}

// ── CounterSpell effect ────────────────────────────────────────────────────────

#[test]
/// CR 701.5 — CounterSpell removes the spell from the stack and moves its card to
/// the graveyard, emitting SpellCountered.
///
/// MR-M7-15: unit test for CounterSpell effect.
fn test_effect_counter_spell_removes_from_stack() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::state::stack::{StackObject, StackObjectKind};
    use mtg_engine::state::targeting::SpellTarget;
    use mtg_engine::{Target, ZoneId};

    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .build()
        .unwrap();

    // Place a card in the Stack zone (as if it was just cast).
    let spell_card = ObjectSpec::card(p2, "Lightning Bolt").in_zone(ZoneId::Stack);
    state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(spell_card)
        .build()
        .unwrap();

    // Grab the card's ObjectId in state.objects (zone == Stack).
    let spell_obj_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.zone == ZoneId::Stack)
        .map(|(id, _)| *id)
        .expect("spell card should be in Stack zone");

    // Push a StackObject representing the spell.
    let stack_entry_id = state.next_object_id();
    state.stack_objects.push_back(StackObject {
        id: stack_entry_id,
        controller: p2,
        kind: StackObjectKind::Spell {
            source_object: spell_obj_id,
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
    });

    // Fire CounterSpell targeting the spell's source object.
    let source = ObjectId(0);
    let effect = Effect::CounterSpell {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
    };
    let mut ctx = EffectContext::new(
        p1,
        source,
        vec![SpellTarget {
            target: Target::Object(spell_obj_id),
            zone_at_cast: Some(ZoneId::Stack),
        }],
    );
    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Stack object removed.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after CounterSpell"
    );

    // Spell card moved to graveyard.
    let in_graveyard = state.objects.iter().any(|(_, obj)| {
        obj.zone == ZoneId::Graveyard(p2) && obj.characteristics.name == "Lightning Bolt"
    });
    assert!(
        in_graveyard,
        "countered spell card should go to owner's graveyard"
    );

    // SpellCountered event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { player, .. } if *player == p2)),
        "expected SpellCountered event"
    );
}

// ── Rhystic Study — MayPayOrElse triggered ability ────────────────────────────

#[test]
/// CR 603.1, CR 603.3 — Rhystic Study: whenever an opponent casts a spell the
/// controller may draw a card unless that player pays {1}.
///
/// In M7 the payment choice is not interactive — the `or_else` branch always fires
/// (the opponent never pays). Verifies the full pipeline: SpellCast event → trigger
/// queued via OpponentCastsSpell → trigger flushed to stack → trigger resolves →
/// controller draws a card.
///
/// Uses TriggerEvent::OpponentCastsSpell (CR 603.2 / CR 102.2). The opponent check
/// is enforced at trigger-collection time in check_triggers (abilities.rs): only
/// permanents whose controller != caster are eligible.
fn test_rhystic_study_draws_card_when_opponent_casts() {
    let p1 = p(1); // Rhystic Study controller
    let p2 = p(2); // opponent who casts a spell

    // Rhystic Study on the battlefield as p1's permanent.
    // TriggeredAbilityDef: fires on OpponentCastsSpell; effect = MayPayOrElse { pay {1} or draw }.
    let rhystic_study = ObjectSpec::card(p1, "Rhystic Study")
        .with_types(vec![CardType::Enchantment])
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::OpponentCastsSpell,
            intervening_if: None,
            description: "Whenever an opponent casts a spell, you may draw a card unless that player pays {1}.".into(),
            effect: Some(Effect::MayPayOrElse {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                payer: PlayerTarget::DeclaredTarget { index: 0 },
                or_else: Box::new(Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                }),
            }),
        })
        .in_zone(ZoneId::Battlefield);

    // p2 has a spell to cast (instant, no targets).
    let filler = ObjectSpec::card(p2, "Shock")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    // p1 needs library cards to draw from.
    let library_card = ObjectSpec::card(p1, "Plains").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(rhystic_study)
        .object(filler)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    // Count p1's initial hand size.
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // p2 casts Shock — this emits SpellCast, which check_triggers picks up as OpponentCastsSpell.
    let shock_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shock" && obj.zone == ZoneId::Hand(p2))
        .map(|(id, _)| *id)
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: shock_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // After CastSpell, the Rhystic Study trigger should be pending.
    // Flush triggers: both p2 and p1 pass → trigger goes on stack → both pass again →
    // trigger resolves → p1 draws a card.
    //
    // Pass p2 (priority after CastSpell resets to active player p2):
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    // Pass p1:
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    // Now the Shock resolves. Rhystic Study trigger may have gone on the stack before this.
    // After the pass cycle, triggers are flushed to the stack by the engine before granting priority.

    // Keep passing until the stack is empty (trigger resolves, then Shock resolves).
    let mut current = state;
    for _ in 0..6 {
        if current.stack_objects.is_empty() {
            break;
        }
        let (ns, _) =
            process_command(current.clone(), Command::PassPriority { player: p2 }).unwrap();
        let (ns, _) = process_command(ns, Command::PassPriority { player: p1 }).unwrap();
        current = ns;
    }

    // p1 should have drawn exactly one card from the Rhystic Study trigger.
    let final_hand = current
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    assert_eq!(
        final_hand,
        initial_hand + 1,
        "Rhystic Study should draw 1 card for p1 when opponent casts a spell (M7: payment always skipped)"
    );
}

// ── Opponent-casts trigger: additional correctness tests ──────────────────────

#[test]
/// CR 603.2 / CR 102.2 — OpponentCastsSpell must NOT fire when the permanent's
/// controller casts the spell. Rhystic Study controller casts their own spell;
/// the trigger must not fire and p1's hand must be unchanged.
fn test_opponent_casts_trigger_does_not_fire_on_own_spell() {
    let p1 = p(1); // Rhystic Study controller — also the caster
    let p2 = p(2);

    let rhystic_study = ObjectSpec::card(p1, "Rhystic Study")
        .with_types(vec![CardType::Enchantment])
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::OpponentCastsSpell,
            intervening_if: None,
            description: "Whenever an opponent casts a spell (CR 603.2)".into(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
        })
        .in_zone(ZoneId::Battlefield);

    // p1 (study controller) casts a spell.
    let own_spell = ObjectSpec::card(p1, "Shock")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    // p1 needs library cards so a draw would be observable if it incorrectly fires.
    let library_card = ObjectSpec::card(p1, "Plains").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(rhystic_study)
        .object(own_spell)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let spell_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shock" && obj.zone == ZoneId::Hand(p1))
        .map(|(id, _)| *id)
        .unwrap();

    // p1 casts their own spell — opponent-casts trigger must NOT fire.
    let (state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // No AbilityTriggered event should be present right after casting.
    let triggered = events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { .. }));
    assert!(
        !triggered,
        "CR 102.2: OpponentCastsSpell must not fire when the permanent's controller casts"
    );

    // The library card should remain in the library (i.e. no draw occurred).
    // Fully resolve the stack (Shock resolves, nothing else happens).
    let mut current = state;
    for _ in 0..6 {
        if current.stack_objects.is_empty() {
            break;
        }
        let (ns, _) =
            process_command(current.clone(), Command::PassPriority { player: p1 }).unwrap();
        let (ns, _) = process_command(ns, Command::PassPriority { player: p2 }).unwrap();
        current = ns;
    }

    // Plains should still be in the library — it was never drawn because the trigger didn't fire.
    let plains_still_in_library = current
        .objects
        .values()
        .any(|o| o.characteristics.name == "Plains" && o.zone == ZoneId::Library(p1));
    assert!(
        plains_still_in_library,
        "CR 102.2: no card should be drawn when the Rhystic-Study controller casts their own spell"
    );
}

#[test]
/// CR 603.2 / CR 102.2 / CR 903.2 — In a 4-player Commander game (FFA), all
/// other players are opponents. When p3 casts a spell and p1 controls a
/// Rhystic-Study-like permanent, exactly p1's trigger fires (not from p2 or
/// p4, who control no such permanents).
fn test_opponent_casts_trigger_multiplayer_fires_for_correct_player() {
    let p1 = p(1); // controls Rhystic-Study-like permanent
    let p2 = p(2); // no relevant permanent
    let p3 = p(3); // the caster (p1's opponent)
    let p4 = p(4); // no relevant permanent

    let rhystic_study = ObjectSpec::card(p1, "Rhystic Study")
        .with_types(vec![CardType::Enchantment])
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::OpponentCastsSpell,
            intervening_if: None,
            description: "Whenever an opponent casts a spell (CR 603.2)".into(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
        })
        .in_zone(ZoneId::Battlefield);

    // p3 casts a spell.
    let spell = ObjectSpec::card(p3, "Shock")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p3));

    // p1 needs library cards to draw from.
    let library_card = ObjectSpec::card(p1, "Plains").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::four_player()
        .object(rhystic_study)
        .object(spell)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p3)
        .build()
        .unwrap();

    let initial_hand_p1 = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let spell_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shock" && obj.zone == ZoneId::Hand(p3))
        .map(|(id, _)| *id)
        .unwrap();

    // p3 casts Shock.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p3,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // Exactly 1 AbilityTriggered event: p1's Rhystic Study.
    let triggered_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 1,
        "CR 603.2: exactly 1 OpponentCastsSpell trigger should fire (p1's study only)"
    );

    // The triggered ability's controller must be p1.
    let trigger_controller = cast_events
        .iter()
        .find_map(|e| {
            if let GameEvent::AbilityTriggered { controller, .. } = e {
                Some(*controller)
            } else {
                None
            }
        })
        .expect("AbilityTriggered event must be present");
    assert_eq!(
        trigger_controller, p1,
        "CR 603.3a: triggered ability controller must be p1 (the study controller)"
    );

    // Resolve the stack — p1 draws 1 card.
    let mut current = state;
    for _ in 0..10 {
        if current.stack_objects.is_empty() {
            break;
        }
        let (ns, _) =
            process_command(current.clone(), Command::PassPriority { player: p3 }).unwrap();
        let (ns, _) = process_command(ns, Command::PassPriority { player: p4 }).unwrap();
        let (ns, _) = process_command(ns, Command::PassPriority { player: p1 }).unwrap();
        let (ns, _) = process_command(ns, Command::PassPriority { player: p2 }).unwrap();
        current = ns;
    }

    let final_hand_p1 = current
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    assert_eq!(
        final_hand_p1,
        initial_hand_p1 + 1,
        "CR 603.2: p1 should draw exactly 1 card (Rhystic Study trigger resolved)"
    );
}

#[test]
/// CR 603.2c — If multiple permanents each have OpponentCastsSpell, each triggers
/// independently. Two Rhystic-Study-like enchantments controlled by p1 both fire
/// when p2 casts a spell, and p1 draws 2 cards total.
fn test_opponent_casts_trigger_multiple_studies_each_trigger_independently() {
    let p1 = p(1); // controls 2 enchantments
    let p2 = p(2); // the caster

    let make_study = |name: &str| {
        ObjectSpec::card(p1, name)
            .with_types(vec![CardType::Enchantment])
            .with_triggered_ability(TriggeredAbilityDef {
                trigger_on: TriggerEvent::OpponentCastsSpell,
                intervening_if: None,
                description: "Whenever an opponent casts a spell (CR 603.2)".into(),
                effect: Some(Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                }),
            })
            .in_zone(ZoneId::Battlefield)
    };

    let study1 = make_study("Study Alpha");
    let study2 = make_study("Study Beta");

    // p2 casts a spell.
    let spell = ObjectSpec::card(p2, "Shock")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    // p1 needs 2 library cards to draw from.
    let lib1 = ObjectSpec::card(p1, "Plains A").in_zone(ZoneId::Library(p1));
    let lib2 = ObjectSpec::card(p1, "Plains B").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(study1)
        .object(study2)
        .object(spell)
        .object(lib1)
        .object(lib2)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    let spell_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shock" && obj.zone == ZoneId::Hand(p2))
        .map(|(id, _)| *id)
        .unwrap();

    // p2 casts Shock — both studies should trigger.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    let triggered_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 603.2c: both studies must trigger independently (2 AbilityTriggered events)"
    );

    // Resolve all stack items — p1 draws 2 cards.
    let mut current = state;
    for _ in 0..10 {
        if current.stack_objects.is_empty() {
            break;
        }
        let (ns, _) =
            process_command(current.clone(), Command::PassPriority { player: p2 }).unwrap();
        let (ns, _) = process_command(ns, Command::PassPriority { player: p1 }).unwrap();
        current = ns;
    }

    let final_hand = current
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    assert_eq!(
        final_hand,
        initial_hand + 2,
        "CR 603.2c: p1 should draw 2 cards (one per triggered study)"
    );
}

#[test]
/// CR 603.2 / CR 102.2 — The triggered ability stack entry for OpponentCastsSpell
/// carries the casting player as Target::Player at index 0, so that
/// DeclaredTarget { index: 0 } at effect resolution time correctly identifies
/// the specific opponent who cast the spell.
fn test_opponent_casts_trigger_carries_casting_player_as_target() {
    let p1 = p(1); // Rhystic Study controller
    let p2 = p(2); // opponent who casts

    let rhystic_study = ObjectSpec::card(p1, "Rhystic Study")
        .with_types(vec![CardType::Enchantment])
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::OpponentCastsSpell,
            intervening_if: None,
            description: "Whenever an opponent casts a spell (CR 603.2)".into(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
        })
        .in_zone(ZoneId::Battlefield);

    let spell = ObjectSpec::card(p2, "Shock")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    let library_card = ObjectSpec::card(p1, "Plains").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(rhystic_study)
        .object(spell)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let spell_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shock" && obj.zone == ZoneId::Hand(p2))
        .map(|(id, _)| *id)
        .unwrap();

    // p2 casts Shock — this should queue the OpponentCastsSpell trigger.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // Pass p2 priority so pending triggers flush to the stack.
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();

    // The triggered ability stack entry must have p2 as Target::Player at index 0.
    // The stack should have: Shock (bottom) + Rhystic Study trigger (top).
    let triggered_stack_obj = state
        .stack_objects
        .iter()
        .find(|o| matches!(o.kind, mtg_engine::StackObjectKind::TriggeredAbility { .. }))
        .expect("triggered ability must be on the stack after trigger flush");

    assert_eq!(
        triggered_stack_obj.targets.len(),
        1,
        "CR 603.2: triggered ability must have exactly 1 target (the casting player)"
    );
    assert_eq!(
        triggered_stack_obj.targets[0].target,
        Target::Player(p2),
        "CR 102.2: target[0] must be Target::Player(p2) — the opponent who cast the spell"
    );
}

// ── Enrich-path test: trigger fires when built via enrich_spec_from_def ───────

#[test]
/// CR 603.2 / CR 603.3 — Rhystic Study's OpponentCastsSpell trigger must fire
/// when the card is built via `enrich_spec_from_def` (the replay-harness path),
/// not just when `with_triggered_ability` is called directly.
///
/// This test catches the regression where the enrich path was missing the
/// WheneverOpponentCastsSpell → TriggerEvent::OpponentCastsSpell mapping.
/// When p2 casts a spell, p1's Rhystic Study (enriched from CardDefinition)
/// must queue a triggered ability and ultimately draw p1 a card.
fn test_rhystic_study_enrich_path_trigger_fires() {
    let p1 = p(1); // Rhystic Study controller
    let p2 = p(2); // opponent who casts a spell

    // Build a CardDefinition map from the actual card database.
    let cards = all_cards();
    let defs: std::collections::HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();

    // Enrich Rhystic Study from its CardDefinition — this is the same path as
    // the replay harness uses in `build_initial_state` for battlefield permanents.
    let rhystic_study = enrich_spec_from_def(
        ObjectSpec::card(p1, "Rhystic Study")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(mtg_engine::card_name_to_id("Rhystic Study")),
        &defs,
    );

    // p2's spell: a simple instant with no mana cost and no targets so it can be
    // cast freely. Using a bare `ObjectSpec::card` (no enrich) so the mana_cost
    // field is None — the engine casts it for free.
    let opp_spell = ObjectSpec::card(p2, "Shock")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    // p1 needs a library card so the draw effect can succeed.
    let library_card = ObjectSpec::card(p1, "Plains").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(rhystic_study)
        .object(opp_spell)
        .object(library_card)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    // Count p1's initial hand size.
    let initial_hand = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Verify that Rhystic Study on the battlefield has a triggered ability
    // with TriggerEvent::OpponentCastsSpell (enrich path correctness check).
    let rhystic_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Rhystic Study" && o.zone == ZoneId::Battlefield)
        .expect("Rhystic Study must be on the battlefield");
    assert!(
        rhystic_obj
            .characteristics
            .triggered_abilities
            .iter()
            .any(|t| t.trigger_on == TriggerEvent::OpponentCastsSpell),
        "enrich_spec_from_def must populate OpponentCastsSpell trigger on Rhystic Study"
    );

    // p2 casts Shock — SpellCast event is emitted, check_triggers dispatches
    // OpponentCastsSpell trigger for Rhystic Study.
    let shock_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Shock" && obj.zone == ZoneId::Hand(p2))
        .map(|(id, _)| *id)
        .unwrap();

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: shock_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
        },
    )
    .unwrap();

    // At least one AbilityTriggered event must be present immediately after the cast.
    let triggered = cast_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { .. }));
    assert!(
        triggered,
        "CR 603.2: enrich-path Rhystic Study must emit AbilityTriggered when opponent casts; \
         got events: {:?}",
        cast_events
    );

    // Keep passing priority until the stack is empty — the trigger resolves and
    // p1 draws a card (MayPayOrElse always applies or_else in non-interactive mode).
    let mut current = state;
    for _ in 0..20 {
        if current.stack_objects.is_empty() {
            break;
        }
        let (ns, _) =
            process_command(current.clone(), Command::PassPriority { player: p2 }).unwrap();
        let (ns, _) = process_command(ns, Command::PassPriority { player: p1 }).unwrap();
        current = ns;
    }

    let final_hand = current
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    assert_eq!(
        final_hand,
        initial_hand + 1,
        "CR 603.2 / enrich path: p1 should draw 1 card from Rhystic Study trigger \
         when opponent casts via enrich_spec_from_def"
    );
}
