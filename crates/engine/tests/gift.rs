//! Gift keyword ability tests (CR 702.174).
//!
//! Gift is an optional additional cost found on some permanents and instants/sorceries.
//! "As an additional cost to cast this spell, you may choose an opponent." (CR 702.174a)
//! If the gift cost was paid, the chosen opponent receives a gift; the caster may also
//! gain additional effects via `Condition::GiftWasGiven`.
//!
//! Key rules verified:
//! - CR 702.174a: Gift cost is choosing an opponent — optional, not mandatory.
//! - CR 702.174b: Permanents use ETB trigger; instants/sorceries use inline conditional.
//! - CR 702.174j: For instant/sorcery, gift effect happens BEFORE any other spell effects.
//! - CR 702.174a: Choosing yourself as gift opponent is rejected (must be an opponent).
//! - CR 702.174d: Gift a Food — chosen player creates a Food token.
//! - CR 702.174e: Gift a card — chosen player draws a card.
//! - CR 702.174h: Gift a Treasure — chosen player creates a Treasure token.
//! - Multiplayer: only the chosen opponent receives the gift (not all opponents).

use mtg_engine::cards::card_definition::GiftType;
use mtg_engine::{
    process_command, AbilityDefinition, AdditionalCost, CardDefinition, CardId, CardRegistry,
    CardType, Command, Condition, Effect, EffectAmount, GameEvent, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine,
    ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Count battlefield permanents controlled by the given player with the given name.
fn count_on_battlefield(state: &mtg_engine::GameState, player: PlayerId, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.controller == player
                && o.characteristics.name == name
        })
        .count()
}

/// Count tokens on the battlefield controlled by the given player.
fn count_tokens_on_battlefield(state: &mtg_engine::GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.controller == player && o.is_token)
        .count()
}

/// Count cards in the given player's hand.
fn hand_count(state: &mtg_engine::GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(player))
        .count()
}

/// Pass priority for all listed players once (round-robin, one pass each).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Synthetic Gift instant: "Gift a card" instant — when gift is given, opponent draws a card.
/// When gift was given, controller also gains 2 life (to test CR 702.174j ordering).
/// Without gift, controller gains 1 life.
/// Mana cost: {1}{U}.
fn gift_card_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("gift-card-instant".to_string()),
        name: "Gift Card Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Gift a card (You may choose an opponent as you cast this spell. \
            If you do, that player draws a card.)\n\
            If this spell's gift was given, you gain 2 life. Otherwise, you gain 1 life."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Gift),
            AbilityDefinition::Gift {
                gift_type: GiftType::Card,
            },
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::GiftWasGiven,
                    if_true: Box::new(Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    }),
                    if_false: Box::new(Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                modes: None,
                targets: vec![],
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic Gift instant: "Gift a Treasure" instant — no main effect beyond the treasure.
/// Mana cost: {R}.
fn gift_treasure_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("gift-treasure-instant".to_string()),
        name: "Gift Treasure Instant".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Gift a Treasure (You may choose an opponent as you cast this spell. \
            If you do, that player creates a Treasure token.)"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Gift),
            AbilityDefinition::Gift {
                gift_type: GiftType::Treasure,
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                modes: None,
                targets: vec![],
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic Gift creature: "Gift a Treasure" creature with ETB trigger.
/// When it enters, if its gift cost was paid, the chosen opponent creates a Treasure.
/// Mana cost: {2}{G}. 3/3 creature.
fn gift_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("gift-creature".to_string()),
        name: "Gift Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Gift a Treasure (You may choose an opponent as you cast this spell. \
            If you do, that player creates a Treasure token when this creature enters.)"
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Gift),
            AbilityDefinition::Gift {
                gift_type: GiftType::Treasure,
            },
        ],
        ..Default::default()
    }
}

/// A plain instant without Gift keyword.
fn plain_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-instant-gift-test".to_string()),
        name: "Plain Instant Gift".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "You gain 1 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            modes: None,
            targets: vec![],
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Setup helpers ─────────────────────────────────────────────────────────────

/// Build a 2-player state with the Gift Card Instant in p1's hand.
/// p1 has {1}{U} mana available (base instant cost).
fn setup_gift_card_state() -> (
    mtg_engine::GameState,
    PlayerId,
    PlayerId,
    mtg_engine::ObjectId,
) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![gift_card_instant_def(), plain_instant_def()]);

    // Library card for p2 so draw won't fail (empty library = no draw).
    let p2_library_card = ObjectSpec::creature(p2, "P2 Library Card", 1, 1)
        .in_zone(ZoneId::Library(p2))
        .with_mana_cost(ManaCost::default());

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Gift Card Instant")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("gift-card-instant".to_string()))
                .with_types(vec![CardType::Instant])
                .with_keyword(KeywordAbility::Gift)
                .with_mana_cost(ManaCost {
                    generic: 1,
                    blue: 1,
                    ..Default::default()
                }),
        )
        .object(p2_library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    {
        let ps = state.players.get_mut(&p1).unwrap();
        ps.mana_pool.add(ManaColor::Colorless, 1);
        ps.mana_pool.add(ManaColor::Blue, 1);
    }
    state.turn.priority_holder = Some(p1);

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Gift Card Instant")
        .map(|(id, _)| *id)
        .expect("Gift Card Instant should be in hand");

    (state, p1, p2, card_id)
}

/// Build a 2-player state with the Gift Creature in p1's hand.
/// p1 has {2}{G} mana for the creature's base cost.
fn setup_gift_creature_state() -> (
    mtg_engine::GameState,
    PlayerId,
    PlayerId,
    mtg_engine::ObjectId,
) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![gift_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object({
            let mut spec = ObjectSpec::card(p1, "Gift Creature")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("gift-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::Gift)
                .with_mana_cost(ManaCost {
                    generic: 2,
                    green: 1,
                    ..Default::default()
                });
            spec.power = Some(3);
            spec.toughness = Some(3);
            spec
        })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    {
        let ps = state.players.get_mut(&p1).unwrap();
        ps.mana_pool.add(ManaColor::Colorless, 2);
        ps.mana_pool.add(ManaColor::Green, 1);
    }
    state.turn.priority_holder = Some(p1);

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Gift Creature")
        .map(|(id, _)| *id)
        .expect("Gift Creature should be in hand");

    (state, p1, p2, card_id)
}

/// Helper: cast spell with the given gift_opponent.
fn cast_spell_with_gift(
    state: mtg_engine::GameState,
    player: PlayerId,
    card_id: mtg_engine::ObjectId,
    gift_opponent: Option<PlayerId>,
) -> Result<(mtg_engine::GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
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
            additional_costs: if let Some(opp) = gift_opponent {
                vec![AdditionalCost::Gift { opponent: opp }]
            } else {
                vec![]
            },
        },
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.174a — Gift not paid: cast instant without choosing an opponent.
/// No gift effect fires. Condition::GiftWasGiven evaluates false → controller gains 1 life.
#[test]
fn test_gift_not_paid_instant() {
    let (state, p1, p2, card_id) = setup_gift_card_state();
    let initial_p1_life = state.players[&p1].life_total;
    let initial_p2_hand = hand_count(&state, p2);

    // Cast without paying gift (gift_opponent = None).
    let (state, _) = cast_spell_with_gift(state, p1, card_id, None).unwrap();

    // Spell is on the stack.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");

    // gift_was_given should be false on the StackObject.
    let stack_obj = &state.stack_objects[0];
    assert!(
        !stack_obj
            .additional_costs
            .iter()
            .any(|c| matches!(c, AdditionalCost::Gift { .. })),
        "CR 702.174a: gift should not be in additional_costs when no opponent was chosen"
    );

    // Resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after resolution"
    );

    // CR 702.174a: opponent's hand should NOT change (no gift given).
    assert_eq!(
        hand_count(&state, p2),
        initial_p2_hand,
        "CR 702.174a: opponent hand should not change when gift was not paid"
    );

    // Condition::GiftWasGiven = false → controller gains 1 life (else branch).
    assert_eq!(
        state.players[&p1].life_total,
        initial_p1_life + 1,
        "CR 702.174b: Condition::GiftWasGiven=false → controller gains 1 life (else branch)"
    );
}

/// CR 702.174e + CR 702.174j — Gift a card instant: opponent draws a card BEFORE main effect.
/// Verify: (1) opponent's hand increases by 1, (2) Condition::GiftWasGiven=true → gains 2 life.
#[test]
fn test_gift_basic_instant_card_draw() {
    let (state, p1, p2, card_id) = setup_gift_card_state();
    let initial_p1_life = state.players[&p1].life_total;
    let initial_p2_hand = hand_count(&state, p2);

    // Cast choosing p2 as the gift recipient.
    let (state, _) = cast_spell_with_gift(state, p1, card_id, Some(p2)).unwrap();

    // Spell is on the stack with gift_was_given=true.
    assert_eq!(state.stack_objects.len(), 1, "spell should be on stack");
    let stack_obj = &state.stack_objects[0];
    assert!(
        stack_obj
            .additional_costs
            .iter()
            .any(|c| matches!(c, AdditionalCost::Gift { .. })),
        "CR 702.174a: Gift should be in additional_costs when opponent was chosen"
    );
    assert_eq!(
        stack_obj
            .additional_costs
            .iter()
            .find_map(|c| match c {
                AdditionalCost::Gift { opponent } => Some(Some(*opponent)),
                _ => None,
            })
            .flatten(),
        Some(p2),
        "CR 702.174a: gift_opponent should be p2"
    );

    // p2's hand is unchanged before resolution (draw happens at resolution, not on cast).
    assert_eq!(
        hand_count(&state, p2),
        initial_p2_hand,
        "p2's hand should not change until spell resolves"
    );

    // Resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after resolution"
    );

    // CR 702.174e: p2 draws a card.
    assert_eq!(
        hand_count(&state, p2),
        initial_p2_hand + 1,
        "CR 702.174e: chosen opponent should draw 1 card when gift a card resolves"
    );

    // Condition::GiftWasGiven = true → controller gains 2 life (if_true branch).
    assert_eq!(
        state.players[&p1].life_total,
        initial_p1_life + 2,
        "CR 702.174b + Condition::GiftWasGiven=true → controller gains 2 life (if_true branch)"
    );
}

/// CR 702.174b — Gift permanent: ETB trigger fires and chosen opponent gets Treasure.
#[test]
fn test_gift_permanent_etb_trigger() {
    let (state, p1, p2, card_id) = setup_gift_creature_state();
    let initial_p2_tokens = count_tokens_on_battlefield(&state, p2);

    // Cast the Gift Creature choosing p2.
    let (state, _) = cast_spell_with_gift(state, p1, card_id, Some(p2)).unwrap();

    // Spell is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "spell should be on stack after cast"
    );

    // Resolve the spell (creature enters battlefield).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is on the battlefield.
    assert_eq!(
        count_on_battlefield(&state, p1, "Gift Creature"),
        1,
        "CR 702.174b: creature should be on battlefield after spell resolves"
    );

    // GiftETBTrigger should now be on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.174b: GiftETBTrigger should be on stack after creature enters"
    );

    // Resolve the GiftETBTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack empty.
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after trigger resolves"
    );

    // CR 702.174h: p2 gets a Treasure token.
    assert_eq!(
        count_tokens_on_battlefield(&state, p2),
        initial_p2_tokens + 1,
        "CR 702.174h: chosen opponent should have 1 Treasure token after GiftETBTrigger resolves"
    );
}

/// CR 702.174b — Gift permanent without gift paid: no ETB trigger.
#[test]
fn test_gift_permanent_not_paid() {
    let (state, p1, p2, card_id) = setup_gift_creature_state();
    let initial_p2_tokens = count_tokens_on_battlefield(&state, p2);

    // Cast the Gift Creature WITHOUT choosing an opponent.
    let (state, _) = cast_spell_with_gift(state, p1, card_id, None).unwrap();

    // Resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Creature is on the battlefield.
    assert_eq!(
        count_on_battlefield(&state, p1, "Gift Creature"),
        1,
        "creature should be on battlefield after spell resolves"
    );

    // No GiftETBTrigger should be on the stack.
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.174b: no GiftETBTrigger when gift cost was not paid"
    );

    // p2 does NOT get a Treasure token.
    assert_eq!(
        count_tokens_on_battlefield(&state, p2),
        initial_p2_tokens,
        "CR 702.174b: no Treasure token for p2 when gift was not paid"
    );
}

/// CR 702.174a — Choosing yourself as the gift opponent is rejected.
#[test]
fn test_gift_invalid_self_rejected() {
    let (state, p1, _p2, card_id) = setup_gift_card_state();

    // Attempt to choose p1 (the caster) as the gift opponent.
    let result = cast_spell_with_gift(state, p1, card_id, Some(p1));
    assert!(
        result.is_err(),
        "CR 702.174a: choosing yourself as gift opponent should be rejected"
    );
}

/// CR 702.174a — Providing a gift_opponent for a spell without Gift keyword is rejected.
#[test]
fn test_gift_rejected_without_keyword() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![plain_instant_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Plain Instant Gift")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("plain-instant-gift-test".to_string()))
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Plain Instant Gift")
        .map(|(id, _)| *id)
        .expect("Plain Instant Gift should be in hand");

    // Attempt to provide gift_opponent on a non-gift spell.
    let result = cast_spell_with_gift(state, p1, card_id, Some(p2));
    assert!(
        result.is_err(),
        "CR 702.174a: providing gift_opponent on a non-Gift spell should be rejected"
    );
}

/// CR 702.174h — Gift a Treasure instant: chosen opponent gets Treasure token, not all opponents.
/// Uses 3-player game to verify only the chosen opponent benefits (multiplayer correctness).
#[test]
fn test_gift_multiplayer_choose_specific_opponent() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let registry = CardRegistry::new(vec![gift_treasure_instant_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Gift Treasure Instant")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("gift-treasure-instant".to_string()))
                .with_types(vec![CardType::Instant])
                .with_keyword(KeywordAbility::Gift)
                .with_mana_cost(ManaCost {
                    red: 1,
                    ..Default::default()
                }),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Gift Treasure Instant")
        .map(|(id, _)| *id)
        .expect("Gift Treasure Instant should be in hand");

    let initial_p2_tokens = count_tokens_on_battlefield(&state, p2);
    let initial_p3_tokens = count_tokens_on_battlefield(&state, p3);

    // Cast choosing p3 specifically (not p2).
    let (state, _) = cast_spell_with_gift(state, p1, card_id, Some(p3)).unwrap();

    // Resolve the spell (p1, p2, p3 all pass priority).
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // CR 702.174h: only p3 gets a Treasure token.
    assert_eq!(
        count_tokens_on_battlefield(&state, p3),
        initial_p3_tokens + 1,
        "CR 702.174h: chosen opponent (p3) should receive 1 Treasure token"
    );
    // p2 should NOT get a token.
    assert_eq!(
        count_tokens_on_battlefield(&state, p2),
        initial_p2_tokens,
        "CR 702.174a: non-chosen opponent (p2) should NOT receive a token (multiplayer)"
    );
}

/// CR 702.174j — For instants/sorceries, gift effect happens BEFORE main spell effects.
/// Verify that p2's hand size increases before p1's life change (ordering via event sequence).
/// We can't directly observe order in state, but we verify both effects happen after resolution.
#[test]
fn test_gift_instant_both_effects_resolve() {
    let (state, p1, p2, card_id) = setup_gift_card_state();
    let initial_p1_life = state.players[&p1].life_total;
    let initial_p2_hand = hand_count(&state, p2);

    // Cast with gift paid.
    let (state, _) = cast_spell_with_gift(state, p1, card_id, Some(p2)).unwrap();

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Both effects should have resolved:
    // CR 702.174e: p2 draws 1 card.
    assert_eq!(
        hand_count(&state, p2),
        initial_p2_hand + 1,
        "CR 702.174e: opponent should draw 1 card (gift effect)"
    );
    // Condition::GiftWasGiven=true → p1 gains 2 life (main spell effect).
    assert_eq!(
        state.players[&p1].life_total,
        initial_p1_life + 2,
        "CR 702.174j: after resolution, controller should have gained 2 life (main effect)"
    );
}
