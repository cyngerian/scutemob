//! Jump-start keyword ability tests (CR 702.133).
//!
//! Jump-start is a static ability that allows instants and sorceries to be cast
//! from the graveyard by paying the card's NORMAL mana cost PLUS discarding any
//! card from hand as an ADDITIONAL cost (not alternative). When a jump-start
//! spell leaves the stack for any reason, it is exiled (same as Flashback).
//!
//! Key rules verified:
//! - Cast from graveyard by paying normal mana cost + discarding any card (CR 702.133a).
//! - Exiled on resolution (CR 702.133a).
//! - Exiled when countered via Effect::CounterSpell (CR 702.133a).
//! - Sorcery-speed timing restriction still applies from graveyard (CR 702.133a ruling 2018-10-05).
//! - Non-jump-start card cannot be cast via jump-start (negative: CR 702.133a).
//! - Normal hand cast does NOT exile the card on resolution (cast_with_jump_start flag is false).
//! - Jump-start pays NORMAL mana cost (not an alternative cost — CR 702.133a).
//! - Discard is required (no discard card → error).
//! - Discard card must be in hand, not graveyard.
//! - Any card type may be discarded (not just lands — CR 702.133a).
//! - cast_with_jump_start flag is set on stack object; cast_with_flashback is not.
//! - Insufficient mana rejected even with valid discard card (CR 601.2f-h).

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec,
    PlayerId, Step, TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
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

// ── Card definitions ───────────────────────────────────────────────────────────

/// Radical Idea: Instant {1}{U}, "Draw a card. Jump-start."
/// Mana cost: 1 generic + 1 blue = MV 2.
fn radical_idea_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("radical-idea".to_string()),
        name: "Radical Idea".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.\nJump-start".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::JumpStart),
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Generic sorcery with jump-start: Sorcery {2}{R}, "Deal 2 damage to target player. Jump-start."
fn jump_start_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("jump-start-sorcery".to_string()),
        name: "Jump Start Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Jump Start Sorcery deals 2 damage to target player. Jump-start.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::JumpStart),
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(2),
                },
                targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Lightning Bolt: Instant {R}, no jump-start (negative test target).
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
            targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Counterspell: Instant {U}{U}, "Counter target spell."
fn counterspell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("counterspell".to_string()),
        name: "Counterspell".to_string(),
        mana_cost: Some(ManaCost {
            blue: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Counter target spell.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
            },
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Grizzly Bears: Creature {1}{G}, 2/2, no jump-start. (Used as a discard card.)
fn grizzly_bears_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("grizzly-bears".to_string()),
        name: "Grizzly Bears".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        ..Default::default()
    }
}

// ── Test 1: Basic jump-start cast from graveyard ───────────────────────────────

#[test]
/// CR 702.133a — A card with jump-start in the graveyard can be cast by paying
/// its normal mana cost plus discarding any card from hand.
fn test_jump_start_basic_cast_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def()]);

    // Radical Idea is in p1's graveyard.
    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    // A creature card in p1's hand to discard.
    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {1}{U} mana (normal mana cost of Radical Idea).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");
    let discard_id = find_object(&state, "Grizzly Bears");

    // p1 casts Radical Idea from graveyard via jump-start, discarding Grizzly Bears.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap();

    // SpellCast event emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 702.133a: SpellCast event expected for jump-start cast"
    );

    // Radical Idea is now on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.133a: Radical Idea should be on the stack"
    );

    // Mana pool should be empty (normal cost {1}{U} paid).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.blue + pool.colorless + pool.red + pool.green + pool.black + pool.white,
        0,
        "CR 702.133a: normal mana cost {{1}}{{U}} should have been deducted"
    );

    // Grizzly Bears should be in p1's graveyard (discarded).
    let bears_in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Grizzly Bears" && o.zone == ZoneId::Graveyard(p1));
    assert!(
        bears_in_graveyard,
        "CR 702.133a: discarded card should be in p1's graveyard"
    );

    // CardDiscarded event should have been emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { player, .. } if *player == p1)),
        "CR 702.133a: CardDiscarded event expected for jump-start discard"
    );
}

// ── Test 2: Exile on resolution ────────────────────────────────────────────────

#[test]
/// CR 702.133a — When a jump-start spell resolves, it is exiled (not put into graveyard).
fn test_jump_start_exile_on_resolution() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def()]);

    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature]);

    // Put some cards in p1's library for the draw effect.
    let library_card = ObjectSpec::card(p1, "Library Card")
        .in_zone(ZoneId::Library(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");
    let discard_id = find_object(&state, "Grizzly Bears");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap();

    // Both players pass — Radical Idea resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Radical Idea should be in exile.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Radical Idea" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.133a: jump-start spell should go to exile on resolution, not graveyard"
    );

    // Radical Idea should NOT be in p1's graveyard.
    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Radical Idea" && o.zone == ZoneId::Graveyard(p1));
    assert!(
        !in_graveyard,
        "CR 702.133a: jump-start spell should NOT go to graveyard on resolution"
    );
}

// ── Test 3: Exile on counter ────────────────────────────────────────────────────

#[test]
/// CR 702.133a — When a jump-start spell is countered, it is exiled (not put into graveyard).
fn test_jump_start_exile_on_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def(), counterspell_def()]);

    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature]);

    let counter_card = ObjectSpec::card(p2, "Counterspell")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("counterspell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            blue: 2,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .object(counter_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");
    let discard_id = find_object(&state, "Grizzly Bears");

    // p1 casts Radical Idea via jump-start.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap();

    let counter_id = find_object(&state, "Counterspell");

    // Find Radical Idea's card object in the Stack zone (this is what Counterspell targets).
    let spell_on_stack_id = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Radical Idea" && obj.zone == ZoneId::Stack {
                Some(id)
            } else {
                None
            }
        })
        .expect("Radical Idea should be in Stack zone as a game object");

    // p1 passes priority — priority goes to p2.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // p2 casts Counterspell targeting the jump-start spell.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: counter_id,
            targets: vec![mtg_engine::Target::Object(spell_on_stack_id)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap();

    // Both players pass — Counterspell resolves (counters Radical Idea), then Counterspell itself resolves.
    let (state, _) = pass_all(state, &[p1, p2, p1, p2]);

    // Radical Idea should be in exile (not graveyard).
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Radical Idea" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 702.133a: jump-start spell should go to exile when countered, not graveyard"
    );

    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Radical Idea" && o.zone == ZoneId::Graveyard(p1));
    assert!(
        !in_graveyard,
        "CR 702.133a: jump-start spell should NOT go to graveyard when countered"
    );
}

// ── Test 4: Sorcery timing restriction ────────────────────────────────────────

#[test]
/// CR 702.133a + ruling 2018-10-05 — Sorcery-speed cards with jump-start can only be cast
/// at sorcery speed (active player's main phase, empty stack).
fn test_jump_start_sorcery_timing() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![jump_start_sorcery_def()]);

    let card = ObjectSpec::card(p1, "Jump Start Sorcery")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("jump-start-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .active_player(p2) // p2 is the active player — p1 cannot cast sorceries
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

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
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Jump Start Sorcery");
    let discard_id = find_object(&state, "Grizzly Bears");

    // p1 tries to cast jump-start sorcery during p2's turn — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![mtg_engine::Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.133a + ruling 2018-10-05: sorcery-speed jump-start cannot be cast on opponent's turn"
    );
}

// ── Test 5: Non-jump-start card cannot use jump-start ─────────────────────────

#[test]
/// CR 702.133a — A card without the JumpStart keyword cannot be cast from the graveyard
/// via jump-start even if cast_with_jump_start: true is set.
fn test_jump_start_non_jump_start_card_cannot_cast() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![lightning_bolt_def()]);

    // Lightning Bolt (no jump-start) is in p1's graveyard.
    let card = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("lightning-bolt".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });
    // No JumpStart keyword added.

    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Lightning Bolt");
    let discard_id = find_object(&state, "Grizzly Bears");

    // Try to cast Lightning Bolt from graveyard via jump-start — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![mtg_engine::Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.133a: card without JumpStart keyword cannot be cast via jump-start"
    );
}

// ── Test 6: Jump-start pays normal mana cost (not an alternative cost) ─────────

#[test]
/// CR 702.133a — Jump-start pays the card's NORMAL mana cost (not an alternative cost).
/// Radical Idea costs {1}{U}, so exactly {1}{U} must be paid via jump-start.
fn test_jump_start_pays_normal_mana_cost() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def()]);

    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature]);

    // Put a library card for the draw effect.
    let library_card = ObjectSpec::card(p1, "Library Card")
        .in_zone(ZoneId::Library(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Exactly {1}{U} — the normal mana cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");
    let discard_id = find_object(&state, "Grizzly Bears");

    // Should succeed — exact normal mana cost is available.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.133a: jump-start cast with exact normal mana cost should succeed: {:?}",
        result.err()
    );

    let (state, _) = result.unwrap();
    // Mana pool should be empty.
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.blue + pool.colorless + pool.red + pool.green + pool.black + pool.white,
        0,
        "CR 702.133a: normal mana cost {{1}}{{U}} should have been fully paid"
    );
}

// ── Test 7: Discard is required ────────────────────────────────────────────────

#[test]
/// CR 702.133a — Jump-start requires a card to discard; None should be rejected.
fn test_jump_start_discard_required() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def()]);

    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");

    // Try to cast with jump_start_discard: None — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None, // No discard card provided — should fail
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.133a: jump-start cast without discard card should fail"
    );
}

// ── Test 8: Discard card must be in hand ────────────────────────────────────────

#[test]
/// CR 601.2f-h — The card to discard as the jump-start additional cost must be in
/// the caster's hand. Providing an ObjectId for a card in graveyard should fail.
fn test_jump_start_discard_must_be_in_hand() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def()]);

    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    // Grizzly Bears is in the GRAVEYARD, not hand.
    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Graveyard(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");
    let discard_id = find_object(&state, "Grizzly Bears");

    // Try to discard a card that's in the graveyard — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 601.2f-h: jump-start discard card must be in caster's hand, not graveyard"
    );
}

// ── Test 9: Any card type may be discarded ─────────────────────────────────────

#[test]
/// CR 702.133a — Unlike Retrace (which requires a land), jump-start accepts ANY card
/// from hand. Discarding a creature card should succeed.
fn test_jump_start_discard_any_card() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def(), grizzly_bears_def()]);

    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    // Discard a CREATURE card (not a land) — this is the key difference from Retrace.
    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("grizzly-bears".to_string()))
        .with_types(vec![CardType::Creature]);

    let library_card = ObjectSpec::card(p1, "Library Card")
        .in_zone(ZoneId::Library(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");
    let discard_id = find_object(&state, "Grizzly Bears");

    // Discard a creature (not a land) — should succeed.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.133a: jump-start should accept any card type for discard, not just lands: {:?}",
        result.err()
    );
}

// ── Test 10: Normal hand cast does NOT exile ────────────────────────────────────

#[test]
/// CR 702.133a — Casting a jump-start card normally from hand (without using jump-start)
/// goes to graveyard on resolution, not exile.
fn test_jump_start_normal_hand_cast_not_exiled() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def()]);

    // Radical Idea is in hand (normal cast).
    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    let library_card = ObjectSpec::card(p1, "Library Card")
        .in_zone(ZoneId::Library(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");

    // Cast normally from hand (cast_with_jump_start: false).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None, // Normal cast — not jump-start
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap();

    // Both players pass — spell resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Radical Idea should be in graveyard (normal destination for instants).
    let in_graveyard = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Radical Idea" && o.zone == ZoneId::Graveyard(p1));
    assert!(
        in_graveyard,
        "CR 702.133a: normal cast (not jump-start) should go to graveyard on resolution"
    );

    // Should NOT be in exile.
    let in_exile = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Radical Idea" && o.zone == ZoneId::Exile);
    assert!(
        !in_exile,
        "CR 702.133a: normal cast should NOT be exiled (only jump-start casts are exiled)"
    );
}

// ── Test 11: cast_with_jump_start flag on stack object ────────────────────────

#[test]
/// CR 702.133a — The cast_with_jump_start flag should be true on the StackObject
/// for a jump-start cast; cast_with_flashback should be false.
fn test_jump_start_flag_set_on_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def()]);

    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");
    let discard_id = find_object(&state, "Grizzly Bears");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap();

    let stack_obj = state.stack_objects.back().unwrap();

    assert!(
        stack_obj.cast_with_jump_start,
        "CR 702.133a: stack object should have cast_with_jump_start: true"
    );
    assert!(
        !stack_obj.cast_with_flashback,
        "CR 702.133a: stack object should have cast_with_flashback: false (not flashback)"
    );
}

// ── Test 12: Insufficient mana rejected ────────────────────────────────────────

#[test]
/// CR 601.2f-h — With insufficient mana for the normal mana cost, jump-start should fail
/// even if a valid discard card is provided.
fn test_jump_start_insufficient_mana_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![radical_idea_def()]);

    let card = ObjectSpec::card(p1, "Radical Idea")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("radical-idea".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::JumpStart);

    let discard_card = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card)
        .object(discard_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only {U} — not enough for {1}{U}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    // Missing 1 generic mana.
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Radical Idea");
    let discard_id = find_object(&state, "Grizzly Bears");

    // Should fail — insufficient mana.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::JumpStart),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: Some(discard_id),
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 601.2f-h: jump-start with insufficient mana should fail"
    );
}
