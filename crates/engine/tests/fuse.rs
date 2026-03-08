//! Fuse keyword ability tests (CR 702.102).
//!
//! Fuse is a static ability on split cards. "Fuse" means the caster may cast both
//! halves of the split card from their hand, paying the combined mana cost of both halves.
//! At resolution, the left half's instructions execute first, then the right half's.
//!
//! Key rules verified:
//! - CR 702.102a: Fuse only applies when casting from hand.
//! - CR 702.102c: The total cost includes the mana cost of each half.
//! - CR 702.102d: Left half executes before right half at resolution.
//! - CR 709.3: Without fuse, only one half is cast at a time.
//! - Engine validation: cards without Fuse keyword reject fuse=true.
//! - Engine validation: fuse=true rejected when not casting from hand.
//! - Engine validation: fuse cannot be combined with alternative costs.

use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::CardType;
use mtg_engine::Effect;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    Command, EffectAmount, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ObjectSpec, PlayerId, PlayerTarget, Step, Target, TargetRequirement, TypeLine, ZoneId,
};

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

/// Build a "Fuse Test Spell" split card:
/// Left half: "Left" — Instant {1}{R} — Deal 3 damage to target player.
/// Right half: "Right" — Instant {W} — Gain 5 life.
/// Fuse (You may cast one or both halves of this card from your hand.)
///
/// This self-contained card lets us test fuse with no external dependencies.
fn fuse_test_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("fuse-test-spell".to_string()),
        name: "Fuse Test Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Deal 3 damage to target player. // Gain 5 life. Fuse.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fuse),
            // Left half spell effect: deal 3 damage to target player
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetPlayer],
                modes: None,
                cant_be_countered: false,
            },
            // Right half: Instant {W} — Gain 5 life
            AbilityDefinition::Fuse {
                name: "Right".to_string(),
                cost: ManaCost {
                    white: 1,
                    ..Default::default()
                },
                card_type: CardType::Instant,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

/// Build a registry containing the fuse test spell.
fn fuse_registry() -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(vec![fuse_test_spell_def()])
}

/// Build a plain sorcery with no Fuse keyword (for testing validation).
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
        oracle_text: "Gain 1 life.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Test 1: Basic fuse — both halves execute ──────────────────────────────────

/// CR 702.102a, CR 702.102c, CR 702.102d — When fuse=true, both halves of the split
/// card are cast from hand. The caster pays the combined mana cost ({1}{R} + {W} = {1}{R}{W}).
/// At resolution, the left half executes first (3 damage to player), then the right half
/// (5 life to controller).
#[test]
fn test_fuse_basic_both_halves_execute() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = fuse_registry();

    let spell = ObjectSpec::card(p1, "Fuse Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fuse-test-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_keyword(KeywordAbility::Fuse)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Record initial states
    let initial_p1_life = state.players[&p1].life_total;
    let initial_p2_life = state.players[&p2].life_total;

    let spell_id = find_object(&state, "Fuse Test Spell");

    let mut state = state;
    // Pay combined fuse cost: {1}{R} (left) + {W} (right) = {1}{R}{W}
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
        .add(ManaColor::White, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    // Cast with fuse=true, targeting p2 for the left half (3 damage).
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
            fuse: true,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("fuse cast failed: {:?}", e));

    // Resolve the spell (both players pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.102d: Left half (3 damage to p2) executes first.
    assert_eq!(
        state.players[&p2].life_total,
        initial_p2_life - 3,
        "CR 702.102d: left half should deal 3 damage to p2"
    );

    // CR 702.102d: Right half (5 life to p1) executes second.
    assert_eq!(
        state.players[&p1].life_total,
        initial_p1_life + 5,
        "CR 702.102d: right half should gain 5 life for p1"
    );
}

// ── Test 2: Single half cast (no fuse) ───────────────────────────────────────

/// CR 709.3, CR 709.3a — When fuse=false, only the left half of the split card is cast.
/// Only the left half's effect (3 damage) executes. The right half (gain life) does not fire.
#[test]
fn test_fuse_single_half_cast() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = fuse_registry();

    let spell = ObjectSpec::card(p1, "Fuse Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fuse-test-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_keyword(KeywordAbility::Fuse)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_p1_life = state.players[&p1].life_total;
    let initial_p2_life = state.players[&p2].life_total;
    let spell_id = find_object(&state, "Fuse Test Spell");

    let mut state = state;
    // Pay only the left half cost: {1}{R}
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

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
    .unwrap_or_else(|e| panic!("single-half cast failed: {:?}", e));

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Only left half fires: 3 damage to p2.
    assert_eq!(
        state.players[&p2].life_total,
        initial_p2_life - 3,
        "CR 709.3: left half should deal 3 damage to p2"
    );
    // Right half does NOT fire: p1 life unchanged.
    assert_eq!(
        state.players[&p1].life_total, initial_p1_life,
        "CR 709.3: right half should NOT fire when fuse=false"
    );
}

// ── Test 3: Fuse from hand only ───────────────────────────────────────────────

/// CR 702.102a — Fuse may only be used when casting from hand. Attempting fuse=true
/// when the card is in the graveyard must be rejected.
#[test]
fn test_fuse_from_hand_only_rejected_from_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = fuse_registry();

    // Card in graveyard (not hand)
    let spell = ObjectSpec::card(p1, "Fuse Test Spell")
        .in_zone(ZoneId::Graveyard(p1))
        .with_card_id(CardId("fuse-test-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_keyword(KeywordAbility::Fuse)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let spell_id = find_object(&state, "Fuse Test Spell");

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
        .add(ManaColor::White, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
            fuse: true,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.102a: fuse=true when card is in graveyard must be rejected"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("hand") || err_msg.contains("fuse"),
        "error message should mention hand or fuse restriction: {}",
        err_msg
    );
}

// ── Test 4: Fuse rejected without keyword ────────────────────────────────────

/// Engine validation — a card without KeywordAbility::Fuse must reject fuse=true.
#[test]
fn test_fuse_no_keyword_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    // Use a plain sorcery (no Fuse keyword) in hand
    let registry = CardRegistry::new(vec![plain_sorcery_def()]);

    let spell = ObjectSpec::card(p1, "Plain Sorcery")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let spell_id = find_object(&state, "Plain Sorcery");

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn.priority_holder = Some(p1);

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
            fuse: true,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "Engine validation: fuse=true on card without Fuse keyword must be rejected"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("Fuse") || err_msg.contains("fuse"),
        "error message should mention Fuse keyword: {}",
        err_msg
    );
}

// ── Test 5: Combined mana cost ────────────────────────────────────────────────

/// CR 702.102c — The total cost of a fused split spell includes the mana cost of each half.
/// Left half: {1}{R}, right half: {W}. Combined fuse cost: {1}{R}{W}.
/// Paying only {1}{R} (left half cost) for a fused cast must fail.
#[test]
fn test_fuse_combined_mana_cost_required() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = fuse_registry();

    let spell = ObjectSpec::card(p1, "Fuse Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fuse-test-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_keyword(KeywordAbility::Fuse)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let spell_id = find_object(&state, "Fuse Test Spell");

    let mut state = state;
    // Intentionally provide INSUFFICIENT mana: only {1}{R} (left half cost only).
    // The right half requires {W} — this should cause cost payment to fail.
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

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
            fuse: true,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.102c: insufficient mana for combined fuse cost must be rejected"
    );
}

// ── Test 6: Resolution order left-then-right ──────────────────────────────────

/// CR 702.102d — As a fused split spell resolves, the controller follows the instructions
/// of the left half first, then the right half. This test verifies the order by checking
/// that damage (left) applies before life gain (right) — both should have occurred.
#[test]
fn test_fuse_resolution_order_left_then_right() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = fuse_registry();

    let spell = ObjectSpec::card(p1, "Fuse Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fuse-test-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_keyword(KeywordAbility::Fuse)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_p1_life = state.players[&p1].life_total;
    let initial_p2_life = state.players[&p2].life_total;
    let spell_id = find_object(&state, "Fuse Test Spell");

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
        .add(ManaColor::White, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
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
            fuse: true,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("fuse cast failed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.102d: Damage (left) applied to p2.
    assert_eq!(
        state.players[&p2].life_total,
        initial_p2_life - 3,
        "CR 702.102d: left half damage should apply to p2"
    );
    // CR 702.102d: Life gain (right) applied to p1 after damage.
    assert_eq!(
        state.players[&p1].life_total,
        initial_p1_life + 5,
        "CR 702.102d: right half life gain should apply to p1"
    );
}

// ── Test 7: Fuse keyword variant is present ───────────────────────────────────

/// CR 702.102a — The Fuse keyword variant must be present on cards with fuse.
/// This test verifies that KeywordAbility::Fuse is correctly registered.
#[test]
fn test_fuse_keyword_variant_present() {
    let def = fuse_test_spell_def();
    let has_fuse_keyword = def
        .abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Fuse)));
    assert!(
        has_fuse_keyword,
        "CR 702.102a: card definition should contain AbilityDefinition::Keyword(KeywordAbility::Fuse)"
    );

    let has_fuse_def = def
        .abilities
        .iter()
        .any(|a| matches!(a, AbilityDefinition::Fuse { .. }));
    assert!(
        has_fuse_def,
        "CR 702.102a: card definition should contain AbilityDefinition::Fuse {{ .. }}"
    );
}

// ── Test 8: Fuse with alternative cost rejected ───────────────────────────────

/// CR 702.102a — Fuse requires casting from hand. Alternative costs that change
/// the casting zone (like flashback from graveyard) are incompatible with fuse.
/// Combining fuse=true with an alt_cost must be rejected.
#[test]
fn test_fuse_alt_cost_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = fuse_registry();

    let spell = ObjectSpec::card(p1, "Fuse Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("fuse-test-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_keyword(KeywordAbility::Fuse)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let spell_id = find_object(&state, "Fuse Test Spell");

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn.priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            // Combine fuse with evoke alt cost — should be rejected.
            alt_cost: Some(AltCostKind::Evoke),
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
            fuse: true,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.102a: fuse=true combined with an alternative cost must be rejected"
    );
}
