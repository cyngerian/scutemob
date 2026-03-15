//! Escalate keyword ability tests (CR 702.120).
//!
//! Escalate is a static ability on modal spells. "Escalate [cost]" means for each
//! mode chosen beyond the first, the escalate cost is paid once as an additional cost.
//!
//! Key rules verified:
//! - CR 702.120a: Escalate is an additional cost, not an alternative cost.
//! - CR 702.120a: For N extra modes, escalate cost is paid N times.
//! - CR 601.2f: The escalate cost is added to total mana cost × number of extra modes.
//! - Auto-stub: escalate_modes=0 → mode[0] only; escalate_modes=1 → modes[0]+[1]; etc.
//! - Engine validation: escalate_modes>0 on a spell without KeywordAbility::Escalate rejected.
//! - Engine validation: escalate_modes ≥ modes.len() is clamped at resolution.

use mtg_engine::cards::card_definition::EffectTarget;
use mtg_engine::state::CardType;
use mtg_engine::AdditionalCost;
use mtg_engine::Effect;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command,
    EffectAmount, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ModeSelection,
    ObjectSpec, PlayerId, PlayerTarget, Step, TypeLine, ZoneId,
};
use std::sync::Arc;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

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

/// Build a synthetic "Escalate Test Spell" card:
/// Sorcery {1}{R}
/// Choose one or more —
///   Mode 0: Controller gains 3 life.
///   Mode 1: Controller draws 2 cards.
///   Mode 2: Target player loses 2 life.
/// Escalate {1}
///
/// This self-contained card lets us test escalate with no external dependencies.
fn escalate_test_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("escalate-test-spell".to_string()),
        name: "Escalate Test Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Choose one or more — Gain 3 life; or draw 2 cards; or target player loses 2 life. Escalate {1}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Nothing,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 3,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: controller gains 3 life.
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(3),
                        },
                        // Mode 1: controller draws 2 cards.
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                        // Mode 2: deal 2 damage to the controller (measurable, no real target
                        // declaration needed in the stub approach).
                        Effect::DealDamage {
                            target: EffectTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                    ],
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Build a plain non-modal sorcery for "no escalate keyword" rejection tests.
fn plain_sorcery_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-sorcery".to_string()),
        name: "Plain Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn escalate_registry() -> Arc<CardRegistry> {
    CardRegistry::new(vec![escalate_test_spell_def()])
}

fn escalate_registry_with_plain() -> (Arc<CardRegistry>, CardDefinition) {
    let plain = plain_sorcery_def();
    let reg = CardRegistry::new(vec![escalate_test_spell_def(), plain.clone()]);
    (reg, plain)
}

/// Build a test state with the escalate spell in hand and some library cards.
fn build_escalate_state(
    p1: PlayerId,
    p2: PlayerId,
    registry: Arc<CardRegistry>,
    library_count: usize,
) -> mtg_engine::GameState {
    let spell = ObjectSpec::card(p1, "Escalate Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("escalate-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Escalate);

    let lib_cards: Vec<_> = (0..library_count)
        .map(|i| {
            ObjectSpec::card(p1, &format!("Library Card {}", i))
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Sorcery])
        })
        .collect();

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for lib in lib_cards {
        builder = builder.object(lib);
    }
    builder.build().unwrap()
}

// ── Test 1: Single mode, no extra cost ────────────────────────────────────────

/// CR 702.120a — escalate_modes=0 means only mode[0] executes and only base mana is paid.
#[test]
fn test_escalate_single_mode_no_extra_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = escalate_registry();
    let mut state = build_escalate_state(p1, p2, registry, 0);

    let initial_life = state.players[&p1].life_total;
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1; // exclude the spell itself

    // Pay base cost {1}{R} = 2 mana, no escalate.
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
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Escalate Test Spell");

    // Cast with escalate_modes = 0 (no extra modes).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
    .unwrap_or_else(|e| panic!("cast with escalate_modes=0 failed: {:?}", e));

    assert_eq!(state.stack_objects.len(), 1, "spell should be on the stack");
    assert_eq!(
        state.stack_objects[0]
            .additional_costs
            .iter()
            .find_map(|c| match c {
                AdditionalCost::EscalateModes { count } => Some(*count),
                _ => None,
            })
            .unwrap_or(0),
        0,
        "CR 702.120a: escalate_modes_paid should be 0 when no extra modes chosen"
    );

    // Resolve (both players pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Only Mode 0 (GainLife 3) should have executed. Mode 1 (DrawCards) should NOT.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.120a: Mode 0 (GainLife 3) should have executed"
    );
    let new_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        new_hand_size, initial_hand_size,
        "CR 702.120a: Mode 1 (DrawCards) should NOT execute when escalate_modes=0"
    );
}

// ── Test 2: Two modes, escalate cost paid once ─────────────────────────────────

/// CR 702.120a — escalate_modes=1: modes[0] and modes[1] execute; escalate cost paid 1×.
/// Total cost = {1}{R} + {1} = {2}{R}.
#[test]
fn test_escalate_two_modes_one_extra_cost() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = escalate_registry();
    let mut state = build_escalate_state(p1, p2, registry, 4);

    let initial_life = state.players[&p1].life_total;
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    // Pay {1}{R} + escalate {1} = {2}{R} = 3 mana total.
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

    let spell_id = find_object(&state, "Escalate Test Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            additional_costs: vec![AdditionalCost::EscalateModes { count: 1 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast with escalate_modes=1 failed: {:?}", e));

    assert_eq!(
        state.stack_objects[0]
            .additional_costs
            .iter()
            .find_map(|c| match c {
                AdditionalCost::EscalateModes { count } => Some(*count),
                _ => None,
            })
            .unwrap_or(0),
        1,
        "CR 702.120a: escalate_modes_paid should be 1"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 3) + Mode 1 (DrawCards 2) both executed.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.120a: Mode 0 (GainLife 3) should have executed"
    );
    let new_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        new_hand_size,
        initial_hand_size + 2,
        "CR 702.120a: Mode 1 (DrawCards 2) should have executed when escalate_modes=1"
    );
}

// ── Test 3: All three modes, escalate cost paid twice ─────────────────────────

/// CR 702.120a — escalate_modes=2: all 3 modes execute; escalate cost paid 2×.
/// Total cost = {1}{R} + {1} + {1} = {3}{R}.
#[test]
fn test_escalate_all_three_modes() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = escalate_registry();
    let mut state = build_escalate_state(p1, p2, registry, 4);

    let initial_life = state.players[&p1].life_total;
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    // Pay {1}{R} + escalate×2 = {3}{R} = 4 mana total.
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
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Escalate Test Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            additional_costs: vec![AdditionalCost::EscalateModes { count: 2 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast with escalate_modes=2 failed: {:?}", e));

    assert_eq!(
        state.stack_objects[0]
            .additional_costs
            .iter()
            .find_map(|c| match c {
                AdditionalCost::EscalateModes { count } => Some(*count),
                _ => None,
            })
            .unwrap_or(0),
        2,
        "CR 702.120a: escalate_modes_paid should be 2"
    );

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 3) + Mode 1 (DrawCards 2) + Mode 2 (DealDamage 2 to self).
    // Net life: +3 from Mode 0, -2 from Mode 2 = net +1.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3 - 2,
        "CR 702.120a: Mode 0 (+3 life) and Mode 2 (-2 life) should both have executed"
    );
    let new_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        new_hand_size,
        initial_hand_size + 2,
        "CR 702.120a: Mode 1 (DrawCards 2) should have executed"
    );
}

// ── Test 4: Insufficient mana for escalate cost rejected ──────────────────────

/// CR 601.2f — Attempting to pay escalate_modes=2 but only providing mana for base+1× fails.
#[test]
fn test_escalate_insufficient_mana_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = escalate_registry();

    let spell = ObjectSpec::card(p1, "Escalate Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("escalate-test-spell".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_keyword(KeywordAbility::Escalate)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only provide {1}{R} + {1} (enough for 1 extra mode), but request escalate_modes=2
    // which needs {1}{R} + {1} + {1} = {3}{R} total.
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
        .add(ManaColor::Colorless, 2); // only enough for base + 1× escalate
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Escalate Test Spell");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            additional_costs: vec![AdditionalCost::EscalateModes { count: 2 }], // requires 2× escalate cost but only 1× provided
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_err(),
        "CR 601.2f: escalate_modes=2 with insufficient mana should be rejected"
    );
}

// ── Test 5: escalate_modes>0 rejected on spell without Escalate keyword ────────

/// Engine validation — a spell without KeywordAbility::Escalate must reject
/// escalate_modes>0 with an InvalidCommand error.
#[test]
fn test_escalate_no_keyword_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let (registry, plain_def) = escalate_registry_with_plain();

    // Plain sorcery in hand (no Escalate keyword).
    let spell = ObjectSpec::card(p1, "Plain Sorcery")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(plain_def.card_id.clone())
        .with_types(vec![CardType::Sorcery]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Sorcery");

    // Attempt escalate on a spell without the keyword — should fail.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            additional_costs: vec![AdditionalCost::EscalateModes { count: 1 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_err(),
        "engine must reject escalate_modes>0 on a spell without KeywordAbility::Escalate"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("escalate"),
        "error message should mention 'escalate', got: {}",
        err_msg
    );
}

// ── Test 6: escalate_modes_paid field set correctly on StackObject ─────────────

/// CR 702.120a — Verify `escalate_modes_paid` on the StackObject reflects the
/// `escalate_modes` requested during casting.
#[test]
fn test_escalate_modes_paid_on_stack() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = escalate_registry();
    let mut state = build_escalate_state(p1, p2, registry, 0);

    // Pay for escalate_modes=1: {1}{R} + {1} = {2}{R}.
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

    let spell_id = find_object(&state, "Escalate Test Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            additional_costs: vec![AdditionalCost::EscalateModes { count: 1 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    assert_eq!(
        state.stack_objects.len(),
        1,
        "exactly one object should be on the stack"
    );
    assert_eq!(
        state.stack_objects[0]
            .additional_costs
            .iter()
            .find_map(|c| match c {
                AdditionalCost::EscalateModes { count } => Some(*count),
                _ => None,
            })
            .unwrap_or(0),
        1,
        "CR 702.120a: escalate_modes_paid must equal the escalate_modes requested during casting"
    );
    assert!(
        !state.stack_objects[0]
            .additional_costs
            .iter()
            .any(|c| matches!(c, AdditionalCost::Entwine)),
        "escalate_modes_paid and was_entwined are independent — entwine should be false"
    );
}

// ── Test 7: escalate_modes clamped at resolution if exceeds available modes ───

/// CR 702.120a — If escalate_modes_paid exceeds the number of available modes,
/// resolution clamps to the available count (modes.len()). No panic or crash.
#[test]
fn test_escalate_modes_exceed_available_clamped() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = escalate_registry();
    let mut state = build_escalate_state(p1, p2, registry, 6);

    let initial_life = state.players[&p1].life_total;
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    // Pay for 5 extra modes (more than the 3 available), but pay enough mana:
    // base {1}{R} + escalate×5 = {6}{R} = 7 mana.
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
        .add(ManaColor::Colorless, 6);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Escalate Test Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            additional_costs: vec![AdditionalCost::EscalateModes { count: 5 }], // exceeds 3 available modes
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast with escalate_modes=5 failed: {:?}", e));

    // Resolution should clamp to min(5+1, 3) = 3 modes.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0 (GainLife 3) + Mode 1 (DrawCards 2) + Mode 2 (DealDamage 2) all ran.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3 - 2,
        "all 3 modes should run when escalate_modes exceeds available count"
    );
    let new_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        new_hand_size,
        initial_hand_size + 2,
        "Mode 1 (DrawCards 2) should have run"
    );
}

// ── Test 8: Modes execute in printed order ────────────────────────────────────

/// CR 702.120a — Modes execute sequentially in printed order.
/// We verify that Mode 0 and Mode 1 both complete (state from mode 0 is visible
/// to mode 1 in the final state), and Mode 2 (damage) also applies.
#[test]
fn test_escalate_modes_execute_in_printed_order() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = escalate_registry();
    let mut state = build_escalate_state(p1, p2, registry, 6);

    let initial_life = state.players[&p1].life_total;
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count()
        - 1;

    // Pay {1}{R} + escalate×2 = {3}{R}.
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
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Escalate Test Spell");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            additional_costs: vec![AdditionalCost::EscalateModes { count: 2 }],
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("cast failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    // Mode 0: GainLife 3. Mode 1: DrawCards 2. Mode 2: DealDamage 2 to controller.
    // Net life: initial + 3 - 2 = initial + 1.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 1,
        "CR 702.120a: all 3 modes should apply in order: +3 life, draw 2, -2 life"
    );
    let new_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        new_hand_size,
        initial_hand_size + 2,
        "Mode 1 (DrawCards 2) should have run in order"
    );
}

// ── Test 9: Escalate rejected on non-modal spell ───────────────────────────────

/// CR 702.120a — "Escalate is a static ability of modal spells."
/// A card with KeywordAbility::Escalate and AbilityDefinition::Escalate but
/// no modes in its AbilityDefinition::Spell must be rejected at cast time
/// when escalate_modes > 0.
#[test]
fn test_escalate_rejected_on_non_modal_spell() {
    let p1 = p(1);
    let p2 = p(2);

    // A misconfigured card: has escalate keyword + cost but modes: None.
    let bad_def = CardDefinition {
        card_id: CardId("escalate-no-modes".to_string()),
        name: "Escalate No Modes".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [mtg_engine::state::CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card. Escalate {1}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None, // Non-modal: escalate is meaningless here
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![bad_def]);

    let spell = ObjectSpec::card(p1, "Escalate No Modes")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("escalate-no-modes".to_string()))
        .with_types(vec![mtg_engine::state::CardType::Sorcery])
        .with_keyword(KeywordAbility::Escalate);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
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
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3); // base {1} + escalate {1} extra
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Escalate No Modes");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            additional_costs: vec![AdditionalCost::EscalateModes { count: 1 }], // Requesting escalate on a non-modal spell
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.120a: escalate_modes > 0 must be rejected when the spell is not modal"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("modal"),
        "Error should mention modal requirement: {}",
        err_msg
    );
}
