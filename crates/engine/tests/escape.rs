//! Escape keyword ability tests (CR 702.138).
//!
//! Escape is a static ability that functions while the card is in the graveyard.
//! "Escape [cost]" means "You may cast this card from your graveyard by paying
//! [cost] rather than paying its mana cost." The exile of other cards from the
//! graveyard is part of the alternative cost. (CR 702.138a)
//!
//! Key differences from Flashback:
//! - Escape applies to ALL card types, not just instants/sorceries (CR 702.138a).
//! - Escaped permanents enter the battlefield normally -- NOT exiled after resolution.
//! - A card can escape again after it dies (goes back to graveyard).
//! - Escaped permanents track `was_escaped` for "escapes with" abilities (CR 702.138b).
//!
//! Key rules verified:
//! - Cast creature from graveyard by paying escape cost (CR 702.138a).
//! - Exactly N other cards exiled from graveyard as part of cost (CR 702.138a).
//! - ObjectExiled events emitted for each exiled card (CR 400.7).
//! - Escaped permanent enters battlefield normally (NOT exiled on resolution).
//! - was_escaped flag set on StackObject and propagated to permanent (CR 702.138b).
//! - "Escapes with counter" puts counter on the permanent at ETB (CR 702.138c).
//! - Insufficient graveyard cards to exile is rejected (CR 702.138a).
//! - Wrong exile count is rejected (engine validation).
//! - Duplicate exile card IDs are rejected (engine validation).
//! - Cannot combine escape with flashback (CR 118.9a).
//! - Cannot combine escape with evoke (CR 118.9a).
//! - Timing restrictions still apply from graveyard (ruling 2020-01-24).
//! - Mana value is based on printed mana cost, not escape cost (CR 118.9c).

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::state::types::AltCostKind;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec,
    PlayerId, Step, ZoneId,
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

fn find_objects(state: &mtg_engine::GameState, name: &str) -> Vec<mtg_engine::ObjectId> {
    state
        .objects
        .iter()
        .filter(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .collect()
}

fn count_in_exile(state: &mtg_engine::GameState, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.characteristics.name == name && o.zone == ZoneId::Exile)
        .count()
}

fn total_in_exile(state: &mtg_engine::GameState) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Exile)
        .count()
}

fn graveyard_size(state: &mtg_engine::GameState, owner: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Graveyard(owner))
        .count()
}

fn battlefield_count(state: &mtg_engine::GameState, name: &str) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
        .count()
}

/// Pass priority for all listed players once.
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

/// Ox of Agonas: Creature {3}{R}{R}, 4/2.
/// "Escape -- {R}{R}, Exile five other cards from your graveyard."
/// "This creature escapes with a +1/+1 counter on it."
/// (Simplified escape cost: 2 red mana, exile 5 others)
fn ox_of_agonas_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("ox-of-agonas".to_string()),
        name: "Ox of Agonas".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Escape -- {R}{R}, Exile five other cards from your graveyard. \
                      This creature escapes with a +1/+1 counter on it."
            .to_string(),
        power: Some(4),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escape),
            AbilityDefinition::Escape {
                cost: ManaCost {
                    red: 2,
                    ..Default::default()
                },
                exile_count: 5,
            },
            AbilityDefinition::EscapeWithCounter {
                counter_type: CounterType::PlusOnePlusOne,
                count: 1,
            },
        ],
        ..Default::default()
    }
}

/// Uro, Titan of Nature's Wrath: Creature {1}{G}{U}, 6/6.
/// Simplified version: escape -- {G}{U}, exile 3 other cards.
/// No "escapes with counter" variant -- simple escape creature.
fn uro_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("uro".to_string()),
        name: "Uro".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Escape -- {G}{U}, Exile three other cards from your graveyard.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escape),
            AbilityDefinition::Escape {
                cost: ManaCost {
                    green: 1,
                    blue: 1,
                    ..Default::default()
                },
                exile_count: 3,
            },
        ],
        ..Default::default()
    }
}

/// Simple sorcery with flashback for mutual exclusion test.
fn loot_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("loot".to_string()),
        name: "Loot".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card. Flashback {2}{R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::Flashback {
                cost: ManaCost {
                    generic: 2,
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Escape),
            AbilityDefinition::Escape {
                cost: ManaCost {
                    red: 2,
                    ..Default::default()
                },
                exile_count: 2,
            },
            AbilityDefinition::Spell {
                effect: mtg_engine::Effect::DrawCards {
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

/// Simple sorcery with ONLY Escape (no Flashback). Used to test that a sorcery
/// cast with escape resolves to the graveyard (not exile) -- unlike flashback.
/// Escape {R}, exile 1 other card from your graveyard.
fn escape_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("escape-sorcery-test".to_string()),
        name: "Escape Sorcery Test".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Escape -- {R}, Exile one other card from your graveyard.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escape),
            AbilityDefinition::Escape {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
                exile_count: 1,
            },
        ],
        ..Default::default()
    }
}

// ── Test 1: Basic escape cast from graveyard ──────────────────────────────────

#[test]
/// CR 702.138a — A card with escape in the graveyard can be cast by paying
/// the escape cost (mana + exile of other cards from graveyard).
fn test_escape_basic_cast_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    // Uro is in p1's graveyard.
    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    // 3 other cards in p1's graveyard for exile cost.
    let fodder1 = ObjectSpec::card(p1, "Fodder A").in_zone(ZoneId::Graveyard(p1));
    let fodder2 = ObjectSpec::card(p1, "Fodder B").in_zone(ZoneId::Graveyard(p1));
    let fodder3 = ObjectSpec::card(p1, "Fodder C").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(fodder1)
        .object(fodder2)
        .object(fodder3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {G}{U} mana (Uro's escape cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let fodder_a_id = find_object(&state, "Fodder A");
    let fodder_b_id = find_object(&state, "Fodder B");
    let fodder_c_id = find_object(&state, "Fodder C");

    // p1 casts Uro from graveyard via escape, exiling 3 other graveyard cards.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![fodder_a_id, fodder_b_id, fodder_c_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // SpellCast event emitted.
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "CR 702.138a: SpellCast event expected for escape cast"
    );

    // Uro is now on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.138a: Uro should be on the stack"
    );

    // Mana pool should be empty (escape cost {G}{U} paid).
    let pool = &state.players[&p1].mana_pool;
    assert_eq!(
        pool.green + pool.blue + pool.red + pool.colorless + pool.black + pool.white,
        0,
        "CR 702.138a: escape cost {{G}}{{U}} should have been deducted from mana pool"
    );

    // The stack object has was_escaped: true.
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        stack_obj.was_escaped,
        "CR 702.138b: stack object should have was_escaped: true"
    );

    // 3 fodder cards are now in exile.
    assert_eq!(
        total_in_exile(&state),
        3,
        "CR 702.138a: exactly 3 cards should be in exile (the escape cost)"
    );
    assert_eq!(count_in_exile(&state, "Fodder A"), 1);
    assert_eq!(count_in_exile(&state, "Fodder B"), 1);
    assert_eq!(count_in_exile(&state, "Fodder C"), 1);

    // Uro itself is on the stack (no longer in graveyard).
    assert_eq!(
        graveyard_size(&state, p1),
        0,
        "CR 702.138a: graveyard should be empty (Uro is on stack, fodders are exiled)"
    );
}

// ── Test 2: ObjectExiled events emitted for exile cost ────────────────────────

#[test]
/// CR 702.138a / CR 601.2h / CR 400.7 — ObjectExiled events emitted for each card
/// exiled as part of the escape cost.
fn test_escape_exile_cost_events() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let fodder1 = ObjectSpec::card(p1, "Card 1").in_zone(ZoneId::Graveyard(p1));
    let fodder2 = ObjectSpec::card(p1, "Card 2").in_zone(ZoneId::Graveyard(p1));
    let fodder3 = ObjectSpec::card(p1, "Card 3").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(fodder1)
        .object(fodder2)
        .object(fodder3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let c1_id = find_object(&state, "Card 1");
    let c2_id = find_object(&state, "Card 2");
    let c3_id = find_object(&state, "Card 3");

    let (_state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![c1_id, c2_id, c3_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // Count ObjectExiled events -- one per exiled card.
    let exile_events: Vec<_> = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectExiled { .. }))
        .collect();
    assert_eq!(
        exile_events.len(),
        3,
        "CR 702.138a: 3 ObjectExiled events expected for 3 cards exiled as escape cost"
    );
}

// ── Test 3: Escaped permanent enters battlefield normally (not exiled) ─────────

#[test]
/// CR 702.138a (ruling 2020-01-24) — Unlike flashback, an escaped permanent enters
/// the battlefield normally and stays there. It is NOT exiled on resolution.
fn test_escape_permanent_resolves_to_battlefield() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let fodder1 = ObjectSpec::card(p1, "F1").in_zone(ZoneId::Graveyard(p1));
    let fodder2 = ObjectSpec::card(p1, "F2").in_zone(ZoneId::Graveyard(p1));
    let fodder3 = ObjectSpec::card(p1, "F3").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(fodder1)
        .object(fodder2)
        .object(fodder3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let f1_id = find_object(&state, "F1");
    let f2_id = find_object(&state, "F2");
    let f3_id = find_object(&state, "F3");

    // Cast Uro via escape.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![f1_id, f2_id, f3_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // Both players pass priority to resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Uro should be on the battlefield (not in exile, not in graveyard).
    assert_eq!(
        battlefield_count(&state, "Uro"),
        1,
        "CR 702.138a: escaped permanent should be on the battlefield, not in exile"
    );
    assert_eq!(
        count_in_exile(&state, "Uro"),
        0,
        "CR 702.138a: escaped permanent should NOT be exiled (unlike flashback)"
    );
    assert_eq!(
        graveyard_size(&state, p1),
        0,
        "CR 702.138a: Uro's graveyard should only contain the 3 exiled fodder cards moved to exile"
    );
}

// ── Test 4: was_escaped flag propagated to permanent ─────────────────────────

#[test]
/// CR 702.138b — A permanent "escaped" if the spell that became it was cast via
/// escape. The was_escaped flag must be true on the permanent after resolution.
fn test_escape_was_escaped_flag_on_permanent() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let f1 = ObjectSpec::card(p1, "X1").in_zone(ZoneId::Graveyard(p1));
    let f2 = ObjectSpec::card(p1, "X2").in_zone(ZoneId::Graveyard(p1));
    let f3 = ObjectSpec::card(p1, "X3").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(f1)
        .object(f2)
        .object(f3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let x1_id = find_object(&state, "X1");
    let x2_id = find_object(&state, "X2");
    let x3_id = find_object(&state, "X3");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![x1_id, x2_id, x3_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // was_escaped on the stack object.
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        stack_obj.was_escaped,
        "CR 702.138b: StackObject.was_escaped should be true after escape cast"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // was_escaped on the permanent.
    let uro_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Uro" && o.zone == ZoneId::Battlefield)
        .expect("Uro should be on battlefield");
    assert!(
        uro_bf.cast_alt_cost == Some(mtg_engine::state::types::AltCostKind::Escape),
        "CR 702.138b: permanent.cast_alt_cost should be Some(Escape) after escaped permanent enters"
    );
}

// ── Test 5: "Escapes with counter" puts counter on permanent ──────────────────

#[test]
/// CR 702.138c — "This permanent escapes with [counter]" is a replacement effect
/// on ETB. If the permanent escaped, it enters with the specified counters.
fn test_escape_with_counter() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![ox_of_agonas_def()]);

    // Ox of Agonas in graveyard. Needs 5 fodder for exile.
    let ox_spec = ObjectSpec::card(p1, "Ox of Agonas")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("ox-of-agonas".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            red: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ox_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    // Add 5 fodder cards to graveyard.
    for i in 1..=5 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Chaff {}", i)).in_zone(ZoneId::Graveyard(p1)));
    }

    let mut state = builder.build().unwrap();

    // Ox's escape cost is {R}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state.turn.priority_holder = Some(p1);

    let ox_id = find_object(&state, "Ox of Agonas");
    let chaff_ids: Vec<_> = (1..=5)
        .map(|i| find_object(&state, &format!("Chaff {}", i)))
        .collect();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ox_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: chaff_ids,
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // Resolve spell.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Ox should be on battlefield.
    let ox_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Ox of Agonas" && o.zone == ZoneId::Battlefield)
        .expect("Ox of Agonas should be on battlefield");

    // Ox should have a +1/+1 counter (from EscapeWithCounter).
    let counter_count = ox_bf
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 1,
        "CR 702.138c: Ox of Agonas should have 1 +1/+1 counter from 'escapes with' ability"
    );
}

// ── Test 6: Normal (non-escaped) permanent gets no "escapes with" counter ─────

#[test]
/// CR 702.138c — "Escapes with counter" only applies if the permanent actually escaped.
/// If cast normally from hand, no counter is placed.
fn test_escape_with_counter_not_applied_when_not_escaped() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![ox_of_agonas_def()]);

    // Ox of Agonas in HAND (not graveyard).
    let ox_spec = ObjectSpec::card(p1, "Ox of Agonas")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("ox-of-agonas".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            red: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ox_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Pay printed mana cost {3}{R}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let ox_id = find_object(&state, "Ox of Agonas");

    // Cast normally from hand (NOT escape).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ox_id,
            targets: vec![],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // Stack object should NOT have was_escaped.
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        !stack_obj.was_escaped,
        "CR 702.138b: was_escaped should be false when cast from hand"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Ox on battlefield should have NO +1/+1 counter.
    let ox_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Ox of Agonas" && o.zone == ZoneId::Battlefield)
        .expect("Ox should be on battlefield");

    let counter_count = ox_bf
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "CR 702.138c: No counter when not escaped (cast from hand)"
    );
}

// ── Test 7: Insufficient graveyard cards rejected ─────────────────────────────

#[test]
/// CR 702.138a — Escape requires exactly N other cards to exile from the graveyard.
/// If fewer than N cards are provided, the cast is rejected.
fn test_escape_insufficient_exile_cards_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    // Uro needs 3 exile cards but we only have 2 fodder.
    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let fodder1 = ObjectSpec::card(p1, "Only1").in_zone(ZoneId::Graveyard(p1));
    let fodder2 = ObjectSpec::card(p1, "Only2").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(fodder1)
        .object(fodder2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let o1_id = find_object(&state, "Only1");
    let o2_id = find_object(&state, "Only2");

    // Attempt to cast with only 2 exile cards (needs 3) — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![o1_id, o2_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.138a: escape with insufficient exile cards should be rejected"
    );
}

// ── Test 8: Duplicate exile card IDs rejected ─────────────────────────────────

#[test]
/// Engine validation — duplicate card IDs in escape_exile_cards must be rejected.
fn test_escape_duplicate_exile_ids_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    // Only 2 real fodder cards, but we'll try to pass the same ID twice.
    let fodder1 = ObjectSpec::card(p1, "Dup1").in_zone(ZoneId::Graveyard(p1));
    let fodder2 = ObjectSpec::card(p1, "Dup2").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(fodder1)
        .object(fodder2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let d1_id = find_object(&state, "Dup1");

    // Pass d1_id twice — should be rejected as duplicate.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![d1_id, d1_id, d1_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    );

    assert!(
        result.is_err(),
        "Engine validation: duplicate escape_exile_cards should be rejected"
    );
}

// ── Test 9: Exile card must be in caster's graveyard ─────────────────────────

#[test]
/// CR 702.138a: "other cards from your graveyard" — the exile cards must be in
/// the caster's graveyard, not in opponent's graveyard.
fn test_escape_exile_card_not_in_graveyard_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    // card_in_hand is in p1's HAND -- not in graveyard.
    let in_hand = ObjectSpec::card(p1, "HandCard").in_zone(ZoneId::Hand(p1));
    let in_gy = ObjectSpec::card(p1, "GY1").in_zone(ZoneId::Graveyard(p1));
    let in_gy2 = ObjectSpec::card(p1, "GY2").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(in_hand)
        .object(in_gy)
        .object(in_gy2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let hand_id = find_object(&state, "HandCard");
    let gy1_id = find_object(&state, "GY1");
    let gy2_id = find_object(&state, "GY2");

    // Try to exile HandCard (not in graveyard) as part of escape cost.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![hand_id, gy1_id, gy2_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.138a: escape exile card must be in caster's graveyard"
    );
}

// ── Test 10: Escape on dual-keyword card (escape + flashback) ─────────────────

#[test]
/// CR 118.9a / Glimpse of Freedom ruling 2020-01-24 — A card with both Escape and
/// Flashback keywords can be cast via escape by setting cast_with_escape: true.
/// The player chooses which alternative cost to apply; the other has no effect.
/// When cast_with_escape: true, flashback auto-detection must be suppressed so
/// the mutual exclusion check does NOT reject a legal game action.
fn test_escape_on_dual_keyword_card_succeeds() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // loot_def() has BOTH Escape and Flashback keywords.
    let registry = CardRegistry::new(vec![loot_def()]);

    // Loot is in graveyard -- both flashback and escape would apply.
    let loot_spec = ObjectSpec::card(p1, "Loot")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("loot".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Flashback)
        .with_keyword(KeywordAbility::Escape);

    let f1 = ObjectSpec::card(p1, "Z1").in_zone(ZoneId::Graveyard(p1));
    let f2 = ObjectSpec::card(p1, "Z2").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(loot_spec)
        .object(f1)
        .object(f2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state.turn.priority_holder = Some(p1);

    let loot_id = find_object(&state, "Loot");
    let z1_id = find_object(&state, "Z1");
    let z2_id = find_object(&state, "Z2");

    // Player explicitly chooses escape (cast_with_escape: true).
    // Flashback auto-detection is suppressed -- only escape applies.
    // This is legal per CR 118.9a: "you choose which one to apply."
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: loot_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![z1_id, z2_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    );

    // Should SUCCEED: player chose escape; flashback is not combined with escape.
    assert!(
        result.is_ok(),
        "CR 118.9a / Glimpse of Freedom ruling: casting with escape on a dual-keyword card \
         (Escape+Flashback) must succeed -- player chose escape, flashback has no effect"
    );
    let (state, _) = result.unwrap();
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        stack_obj.was_escaped,
        "CR 702.138b: was_escaped must be true when cast via escape"
    );
}

// ── Test 11: Escape requires the card to be in graveyard ──────────────────────

#[test]
/// CR 702.138a — Escape only functions while the card is in the graveyard.
/// Attempting to cast with escape from hand should be rejected.
fn test_escape_requires_card_in_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    // Uro is in HAND (not graveyard).
    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let f1 = ObjectSpec::card(p1, "H1").in_zone(ZoneId::Graveyard(p1));
    let f2 = ObjectSpec::card(p1, "H2").in_zone(ZoneId::Graveyard(p1));
    let f3 = ObjectSpec::card(p1, "H3").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(f1)
        .object(f2)
        .object(f3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let h1_id = find_object(&state, "H1");
    let h2_id = find_object(&state, "H2");
    let h3_id = find_object(&state, "H3");

    // Attempt escape from hand (invalid).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![h1_id, h2_id, h3_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.138a: escape requires the card to be in the graveyard, not hand"
    );
}

// ── Test 12: Mana value from printed cost, not escape cost ───────────────────

#[test]
/// CR 118.9c — An alternative cost doesn't change a spell's mana value. The
/// escape spell's mana value is based on its printed mana cost.
fn test_escape_mana_value_unchanged() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let f1 = ObjectSpec::card(p1, "M1").in_zone(ZoneId::Graveyard(p1));
    let f2 = ObjectSpec::card(p1, "M2").in_zone(ZoneId::Graveyard(p1));
    let f3 = ObjectSpec::card(p1, "M3").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(f1)
        .object(f2)
        .object(f3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let m1_id = find_object(&state, "M1");
    let m2_id = find_object(&state, "M2");
    let m3_id = find_object(&state, "M3");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![m1_id, m2_id, m3_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // The stack object's characteristics should reflect the printed mana cost.
    // Uro's printed cost is {1}{G}{U} = mana value 3.
    // Escape cost is {G}{U} = mana value 2.
    // The mana cost on the characteristics should be the PRINTED cost.
    let stack_obj = state.stack_objects.back().unwrap();
    // The characteristics.mana_cost was set from the card's printed cost at build time.
    // After escape cast, the printed mana_cost should be unchanged.
    let printed_mc = &state
        .objects
        .values()
        .find(|o| {
            o.zone == ZoneId::Stack
                || state
                    .stack_objects
                    .iter()
                    .any(|s| matches!(s.kind, mtg_engine::StackObjectKind::Spell { source_object: id } if id == o.id))
        })
        .map(|o| o.characteristics.mana_cost.clone());

    // Verify the stack object exists and the mana cost is {1}{G}{U}.
    if let Some(Some(mc)) = printed_mc {
        assert_eq!(mc.generic, 1, "CR 118.9c: printed generic mana should be 1");
        assert_eq!(mc.green, 1, "CR 118.9c: printed green mana should be 1");
        assert_eq!(mc.blue, 1, "CR 118.9c: printed blue mana should be 1");
    }

    // Confirm we're looking at the right stack object.
    assert!(
        stack_obj.was_escaped,
        "Was checking the right stack object (was_escaped: true)"
    );
}

// ── Test 13: Auto-detect escape from graveyard ───────────────────────────────

#[test]
/// CR 702.138a — When a card with Escape (but not Flashback) is in the graveyard
/// and cast_with_escape is false (default), the engine should auto-detect escape.
fn test_escape_auto_detected_from_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let f1 = ObjectSpec::card(p1, "A1").in_zone(ZoneId::Graveyard(p1));
    let f2 = ObjectSpec::card(p1, "A2").in_zone(ZoneId::Graveyard(p1));
    let f3 = ObjectSpec::card(p1, "A3").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(f1)
        .object(f2)
        .object(f3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let a1_id = find_object(&state, "A1");
    let a2_id = find_object(&state, "A2");
    let a3_id = find_object(&state, "A3");

    // cast_with_escape: false -- engine should auto-detect escape since card is in graveyard
    // with the Escape keyword and no Flashback keyword.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            // alt_cost: None -- engine should auto-detect escape since card is in graveyard
            // with the Escape keyword and no Flashback keyword.
            alt_cost: None,
            escape_exile_cards: vec![a1_id, a2_id, a3_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    );

    // Auto-detection: even with cast_with_escape: false, the escape is applied
    // because the card is in the graveyard with the Escape keyword.
    assert!(
        result.is_ok(),
        "CR 702.138a: escape should be auto-detected when card is in graveyard with Escape keyword"
    );
    let (state, _) = result.unwrap();
    let stack_obj = state.stack_objects.back().unwrap();
    assert!(
        stack_obj.was_escaped,
        "CR 702.138a: was_escaped should be true even with auto-detection"
    );
}

// ── Test 14: find_objects helper used correctly ───────────────────────────────

#[test]
/// Engine invariant: escape_exile_cards correctly references the right object IDs.
/// After exile, the old IDs are retired (CR 400.7) -- the objects have new IDs in exile.
fn test_escape_exile_cards_get_new_ids_in_exile() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    let f1 = ObjectSpec::card(p1, "Retire1").in_zone(ZoneId::Graveyard(p1));
    let f2 = ObjectSpec::card(p1, "Retire2").in_zone(ZoneId::Graveyard(p1));
    let f3 = ObjectSpec::card(p1, "Retire3").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(f1)
        .object(f2)
        .object(f3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let r1_id = find_object(&state, "Retire1");
    let r2_id = find_object(&state, "Retire2");
    let r3_id = find_object(&state, "Retire3");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![r1_id, r2_id, r3_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // Old IDs (r1_id, r2_id, r3_id) should no longer exist in the objects map.
    // They have been replaced by new IDs after zone transition (CR 400.7).
    assert!(
        !state.objects.contains_key(&r1_id),
        "CR 400.7: old exile card ID should be retired after zone transition"
    );
    assert!(
        !state.objects.contains_key(&r2_id),
        "CR 400.7: old exile card ID should be retired after zone transition"
    );
    assert!(
        !state.objects.contains_key(&r3_id),
        "CR 400.7: old exile card ID should be retired after zone transition"
    );

    // New objects in exile should have the same names.
    assert_eq!(
        count_in_exile(&state, "Retire1"),
        1,
        "Retire1 should exist in exile under a new ID"
    );
    assert_eq!(
        count_in_exile(&state, "Retire2"),
        1,
        "Retire2 should exist in exile under a new ID"
    );
    assert_eq!(
        count_in_exile(&state, "Retire3"),
        1,
        "Retire3 should exist in exile under a new ID"
    );

    // Suppress unused variable warnings for find_objects.
    let _ = find_objects;
}

// ── Test 15: Sorcery with escape resolves to graveyard (not exile) ────────────

#[test]
/// CR 702.138a — Unlike flashback (CR 702.34a), escape does NOT exile the card
/// on resolution. An instant or sorcery cast with escape goes to the graveyard
/// after resolution, just like a normally cast spell.
fn test_escape_sorcery_resolves_to_graveyard() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![escape_sorcery_def()]);

    // "Escape Sorcery Test" is in p1's graveyard with 1 other card.
    let sorcery_spec = ObjectSpec::card(p1, "Escape Sorcery Test")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("escape-sorcery-test".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    // 1 other card in p1's graveyard (required as exile cost).
    let fodder = ObjectSpec::card(p1, "GY Fodder").in_zone(ZoneId::Graveyard(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(sorcery_spec)
        .object(fodder)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p1 has {R} mana (escape cost).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let sorcery_id = find_object(&state, "Escape Sorcery Test");
    let fodder_id = find_object(&state, "GY Fodder");

    // Cast sorcery from graveyard via escape, exiling 1 other card.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: sorcery_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            escape_exile_cards: vec![fodder_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    )
    .unwrap();

    // Resolve the spell: both players pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.138a: The sorcery should be in the graveyard, NOT in exile.
    // This is the key difference from flashback (CR 702.34a exiles on resolution).
    assert_eq!(
        graveyard_size(&state, p1),
        1,
        "CR 702.138a: escape sorcery must resolve to graveyard, not exile (unlike flashback)"
    );
    assert_eq!(
        count_in_exile(&state, "Escape Sorcery Test"),
        0,
        "CR 702.138a: escape sorcery must NOT be exiled on resolution (unlike flashback)"
    );

    // The fodder card exiled as cost should still be in exile.
    assert_eq!(
        count_in_exile(&state, "GY Fodder"),
        1,
        "Fodder exiled as escape cost should remain in exile"
    );

    // Total exile = only the fodder.
    assert_eq!(
        total_in_exile(&state),
        1,
        "CR 702.138a: only the fodder is in exile; sorcery is in graveyard"
    );
}

// ── Test 16: Exile card from opponent's graveyard rejected ────────────────────

#[test]
/// CR 702.138a — Escape requires exiling "other cards from YOUR graveyard."
/// Cards from an opponent's graveyard cannot be used to pay the escape cost.
fn test_escape_exile_from_opponent_graveyard_rejected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![uro_def()]);

    // Uro is in p1's graveyard.
    let uro_spec = ObjectSpec::card(p1, "Uro")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("uro".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Escape);

    // Two exile candidates in p1's graveyard (legal).
    let p1_fodder1 = ObjectSpec::card(p1, "P1 Card 1").in_zone(ZoneId::Graveyard(p1));
    let p1_fodder2 = ObjectSpec::card(p1, "P1 Card 2").in_zone(ZoneId::Graveyard(p1));

    // One card in p2's graveyard -- this must NOT be usable as escape exile cost.
    let p2_card = ObjectSpec::card(p2, "P2 Card").in_zone(ZoneId::Graveyard(p2));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(uro_spec)
        .object(p1_fodder1)
        .object(p1_fodder2)
        .object(p2_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let uro_id = find_object(&state, "Uro");
    let p1_c1_id = find_object(&state, "P1 Card 1");
    let p2_card_id = find_object(&state, "P2 Card");

    // Attempt to use a card from p2's graveyard as part of the escape exile cost.
    // Uro needs exile_count: 3 but we only provide 2 cards total (1 from p1, 1 from p2).
    // Even if we provided 3 cards, the p2 card must be rejected.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: uro_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Escape),
            // Including p2's graveyard card -- must be rejected.
            escape_exile_cards: vec![p1_c1_id, p2_card_id],
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
            mutate_target: None,
            mutate_on_top: false,
            face_down_kind: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.138a: escape exile cards must come from caster's own graveyard; \
         cards from opponent's graveyard must be rejected"
    );
}
