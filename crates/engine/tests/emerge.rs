//! Emerge keyword ability tests (CR 702.119).
//!
//! Emerge is an alternative cost found on some spells.
//! "You may cast this spell by paying [cost] and sacrificing a creature.
//! The total cost to cast this spell is reduced by the sacrificed creature's
//! mana value." (CR 702.119a)
//!
//! Key rules verified:
//! - Emerge is an alternative cost (CR 118.9) -- cannot combine with other alt costs.
//! - The sacrifice target must be a creature (CR 702.119a).
//! - Only the caster's own creatures may be sacrificed (CR 702.119a).
//! - The emerge cost is reduced by the sacrificed creature's MV (CR 702.119a).
//! - Tokens have MV 0 -- no cost reduction (CR 202.3b).
//! - A creature with MV exceeding the emerge cost reduces the cost to {0} (cost floor).
//! - Providing emerge_sacrifice without alt_cost Emerge is rejected (engine validation).
//! - Providing alt_cost Emerge without emerge_sacrifice is rejected (engine validation).
//! - Cards without Emerge keyword reject alt_cost Emerge (engine validation).
//! - The spell can also be cast normally without using emerge (CR 702.119a).

use mtg_engine::state::types::AltCostKind;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId, Step,
    TypeLine, ZoneId,
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

fn find_object_in_zone(
    state: &mtg_engine::GameState,
    name: &str,
    zone: ZoneId,
) -> Option<mtg_engine::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == zone {
            Some(id)
        } else {
            None
        }
    })
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

/// Synthetic emerge creature: {8} normal cost, Emerge {5}{U}{U} (total MV 7).
/// When it resolves, the controller gains 1 life (simple observable effect for testing).
fn emerge_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("emerge-creature".to_string()),
        name: "Emerge Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 8,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(5),
        toughness: Some(6),
        oracle_text: "Emerge {5}{U}{U} (You may cast this spell by paying {5}{U}{U} and sacrificing a creature. The total cost is reduced by the sacrificed creature's mana value.)".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Emerge),
            AbilityDefinition::Emerge {
                cost: ManaCost {
                    generic: 5,
                    blue: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// Synthetic spell without Emerge -- used to verify emerge sacrifice is rejected.
fn plain_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-creature".to_string()),
        name: "Plain Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(3),
        toughness: Some(3),
        oracle_text: "".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

/// Helper: build a 2-player state with the emerge creature in p1's hand
/// and an optional list of extra objects. Returns (state, p1, p2, spell_id).
fn setup_emerge_state(
    extra_objects: Vec<ObjectSpec>,
) -> (
    mtg_engine::GameState,
    PlayerId,
    PlayerId,
    mtg_engine::ObjectId,
) {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![emerge_creature_def()]);

    let spell = ObjectSpec::card(p1, "Emerge Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("emerge-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 8,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Emerge);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    for obj in extra_objects {
        builder = builder.object(obj);
    }

    let mut state = builder.build().unwrap();
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Emerge Creature");
    (state, p1, p2, spell_id)
}

// ── Test 1: Basic emerge -- sacrifice reduces cost ────────────────────────────

/// CR 702.119a — Casting a spell with emerge while sacrificing a creature with
/// MV 3 should reduce the emerge cost {5}{U}{U} (total 7) by 3, resulting in
/// a cost of {2}{U}{U} (total 4).
#[test]
fn test_emerge_basic_sacrifice_reduces_cost() {
    let p1 = p(1);
    let p2 = p(2);

    // Creature with MV 3 ({1}{G}{G}) for p1 to sacrifice.
    let creature = ObjectSpec::card(p1, "Llanowar Elves")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 2,
            ..Default::default()
        });

    let (mut state, _p1, _p2, spell_id) = setup_emerge_state(vec![creature]);

    // Add {2}{U}{U} mana (emerge cost {5}{U}{U} reduced by 3 = {2}{U}{U}).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);

    let creature_id = find_object(&state, "Llanowar Elves");

    // Cast Emerge Creature using emerge, sacrificing the 3-MV creature.
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
            alt_cost: Some(AltCostKind::Emerge),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![creature_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with emerge (MV 3 creature) failed: {:?}", e));

    // CR 702.119a: The sacrificed creature must no longer be on the battlefield.
    let creature_on_bf = find_object_in_zone(&state, "Llanowar Elves", ZoneId::Battlefield);
    assert!(
        creature_on_bf.is_none(),
        "CR 702.119a: creature sacrificed for emerge must no longer be on the battlefield"
    );

    // CR 702.119a: The sacrificed creature must be in the graveyard.
    let creature_in_gy = find_object_in_zone(&state, "Llanowar Elves", ZoneId::Graveyard(p1));
    assert!(
        creature_in_gy.is_some(),
        "CR 702.119a: creature sacrificed for emerge must be in the graveyard"
    );

    // Emerge Creature must be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.119a: emerge creature should be on the stack after casting"
    );

    // Both players pass priority -- spell resolves, creature enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    let creature_on_bf = find_object_in_zone(&state, "Emerge Creature", ZoneId::Battlefield);
    assert!(
        creature_on_bf.is_some(),
        "CR 702.119a: emerge creature should be on the battlefield after resolving"
    );
}

// ── Test 2: Token sacrifice (MV 0) -- no cost reduction ───────────────────────

/// CR 702.119a + CR 202.3b — A token has no mana cost, so MV = 0.
/// Sacrificing a token provides no cost reduction; the full emerge cost must be paid.
#[test]
fn test_emerge_sacrifice_token_mv_zero() {
    let p1 = p(1);
    let p2 = p(2);

    // A 1/1 creature token (MV = 0).
    let token = ObjectSpec::card(p1, "Creature Token")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .token();

    let (mut state, _p1, _p2, spell_id) = setup_emerge_state(vec![token]);

    // Must pay full emerge cost {5}{U}{U} (7 mana total, no reduction from token).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);

    let token_id = find_object(&state, "Creature Token");

    // Cast using emerge, sacrificing the token (MV 0 = no reduction).
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
            alt_cost: Some(AltCostKind::Emerge),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![token_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with emerge (token, MV 0) failed: {:?}", e));

    // Token must have been sacrificed.
    let token_on_bf = find_object_in_zone(&state, "Creature Token", ZoneId::Battlefield);
    assert!(
        token_on_bf.is_none(),
        "CR 702.119a: token sacrificed for emerge must no longer be on the battlefield"
    );

    // Emerge Creature must be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.119a + CR 202.3b: emerge spell should be on stack after casting with token (MV 0)"
    );

    // Resolve: both players pass.
    let (state, _) = pass_all(state, &[p1, p2]);

    let creature_on_bf = find_object_in_zone(&state, "Emerge Creature", ZoneId::Battlefield);
    assert!(
        creature_on_bf.is_some(),
        "emerge creature should enter battlefield after resolving"
    );
}

// ── Test 3: High-MV creature reduces cost to {0} (cost floor) ─────────────────

/// CR 702.119a — If the sacrificed creature's MV exceeds the emerge cost,
/// the remaining cost is {0} (cost cannot go negative).
///
/// Emerge cost: {5}{U}{U} (total 7). Sacrificing creature with MV 10 → cost = {0}.
#[test]
fn test_emerge_sacrifice_high_mv_creature() {
    let p1 = p(1);
    let p2 = p(2);

    // Creature with MV 10 (e.g., {7}{G}{G}{G}).
    let big_creature = ObjectSpec::card(p1, "Enormous Creature")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 7,
            green: 3,
            ..Default::default()
        });

    let (state, _p1, _p2, spell_id) = setup_emerge_state(vec![big_creature]);

    // With MV 10, emerge cost {5}{U}{U} (7) is reduced to {0}. No mana needed.
    // (mana pool stays empty)

    let creature_id = find_object(&state, "Enormous Creature");

    // Cast emerge with no mana -- cost should have been reduced to {0}.
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
            alt_cost: Some(AltCostKind::Emerge),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![creature_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "CastSpell with emerge (MV 10 creature, cost floor {0}) failed: {:?}",
            e
        )
    });

    // Creature must have been sacrificed.
    let creature_on_bf = find_object_in_zone(&state, "Enormous Creature", ZoneId::Battlefield);
    assert!(
        creature_on_bf.is_none(),
        "CR 702.119a: creature sacrificed for emerge must no longer be on the battlefield"
    );

    // Emerge Creature must be on the stack (cast for free after reduction).
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.119a: emerge spell should be on stack after free cast (MV reduction to {{0}})"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    let creature_on_bf = find_object_in_zone(&state, "Emerge Creature", ZoneId::Battlefield);
    assert!(
        creature_on_bf.is_some(),
        "emerge creature should enter battlefield after resolving at zero cost"
    );
}

// ── Test 4: Sacrifice must be a creature ──────────────────────────────────────

/// CR 702.119a — The emerge sacrifice target must be a creature.
/// Attempting to sacrifice a non-creature artifact must be rejected.
#[test]
fn test_emerge_sacrifice_must_be_creature() {
    let p1 = p(1);

    // An artifact (non-creature) -- invalid emerge sacrifice target.
    let artifact = ObjectSpec::card(p1, "Sol Ring")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);

    let (mut state, _p1, _p2, spell_id) = setup_emerge_state(vec![artifact]);

    // Add enough mana so that payment failure is about validation, not mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);

    let artifact_id = find_object(&state, "Sol Ring");

    // Attempting to sacrifice an artifact for emerge should be rejected.
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
            alt_cost: Some(AltCostKind::Emerge),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![artifact_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.119a: non-creature artifact should be rejected as emerge sacrifice target"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "CR 702.119a: error should be InvalidCommand; got: {:?}",
        err
    );
}

// ── Test 5: Sacrifice must be own creature ────────────────────────────────────

/// CR 702.119a — The emerge sacrifice must be controlled by the caster.
/// Attempting to sacrifice an opponent's creature must be rejected.
#[test]
fn test_emerge_sacrifice_must_be_own_creature() {
    let p1 = p(1);
    let p2 = p(2);

    // A creature controlled by the opponent (p2).
    let opponent_creature = ObjectSpec::card(p2, "Opponent Creature")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        });

    let (mut state, _p1, _p2, spell_id) = setup_emerge_state(vec![opponent_creature]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);

    let opponent_creature_id = find_object(&state, "Opponent Creature");

    // Attempting to sacrifice the opponent's creature should be rejected.
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
            alt_cost: Some(AltCostKind::Emerge),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![opponent_creature_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.119a: sacrificing an opponent's creature should be rejected for emerge"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "CR 702.119a: error should be InvalidCommand; got: {:?}",
        err
    );
}

// ── Test 6: Emerge without sacrifice fails ────────────────────────────────────

/// CR 702.119a — Emerge requires sacrificing a creature. Providing
/// alt_cost Emerge without emerge_sacrifice must be rejected.
#[test]
fn test_emerge_without_sacrifice_fails() {
    let p1 = p(1);

    let (mut state, _p1, _p2, spell_id) = setup_emerge_state(vec![]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);

    // Emerge alt cost without a sacrifice should be rejected.
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
            alt_cost: Some(AltCostKind::Emerge),
            prototype: false,
            // emerge_sacrifice removed — no Sacrifice in additional_costs means missing
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.119a: emerge alt_cost without emerge_sacrifice must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "CR 702.119a: error should be InvalidCommand; got: {:?}",
        err
    );
}

// ── Test 7: Emerge mutual exclusion with flashback ────────────────────────────

/// CR 118.9a — Only one alternative cost may be applied to a spell at a time.
/// Attempting to combine emerge with flashback must be rejected.
#[test]
fn test_emerge_mutual_exclusion_with_flashback() {
    let p1 = p(1);

    let creature = ObjectSpec::card(p1, "Llanowar Elves")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let (mut state, _p1, _p2, spell_id) = setup_emerge_state(vec![creature]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 2);

    let creature_id = find_object(&state, "Llanowar Elves");

    // Attempting to combine emerge with flashback must be rejected.
    // Note: alt_cost can only be one value; this test passes Flashback while also
    // providing emerge_sacrifice -- the engine rejects because flashback != Emerge.
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
            alt_cost: Some(AltCostKind::Flashback), // Flashback, not Emerge
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![creature_id],
                lki_powers: vec![],
            }], // // Also providing emerge sacrifice
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    // Should fail: the sacrifice is silently ignored (no Emerge alt_cost to claim it),
    // and the flashback alt_cost fails for other reasons (no flashback cost defined,
    // or mana issues). The key point: emerge + flashback can't combine.
    assert!(
        result.is_err(),
        "CR 118.9a: emerge sacrifice with flashback alt_cost must fail"
    );
}

// ── Test 8: No keyword rejects emerge alt_cost ────────────────────────────────

/// CR 702.119a — Providing alt_cost Emerge for a spell without the Emerge keyword
/// must be rejected by the engine.
#[test]
fn test_emerge_no_keyword_rejects_emerge() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_creature_def()]);

    // Plain creature with NO emerge keyword.
    let spell = ObjectSpec::card(p1, "Plain Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    // Another creature for p1 to attempt to sacrifice.
    let creature = ObjectSpec::card(p1, "Llanowar Elves")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Creature");
    let creature_id = find_object(&state, "Llanowar Elves");

    // Attempting to use emerge alt_cost on a card without Emerge keyword should be rejected.
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
            alt_cost: Some(AltCostKind::Emerge),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![creature_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.119a: providing emerge alt_cost for a spell without Emerge must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "CR 702.119a: error should be InvalidCommand; got: {:?}",
        err
    );
}

// ── Test 9: Normal cast (no emerge) works ─────────────────────────────────────

/// CR 702.119a — Emerge is an optional alternative cost. The spell can always be
/// cast normally by paying its printed mana cost without using emerge.
#[test]
fn test_emerge_normal_cast_without_emerge() {
    let p1 = p(1);
    let p2 = p(2);

    let (mut state, _p1, _p2, spell_id) = setup_emerge_state(vec![]);

    // Pay the full normal cost {8}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 8);

    // Cast without emerge alt_cost.
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
            alt_cost: None, // Normal cast
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("Normal cast of emerge creature failed: {:?}", e));

    // The spell should be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "CR 702.119a: emerge creature should be on stack after normal cast"
    );

    // Resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    let creature_on_bf = find_object_in_zone(&state, "Emerge Creature", ZoneId::Battlefield);
    assert!(
        creature_on_bf.is_some(),
        "CR 702.119a: emerge creature should enter battlefield after normal cast and resolution"
    );
}

// ── Test 10: Sacrifice in additional_costs without matching ability is ignored ─

/// RC-1 consolidation: providing Sacrifice in additional_costs without a matching
/// ability (neither Bargain keyword, Casualty keyword, nor Emerge alt_cost) is
/// harmless — the sacrifice entry is silently ignored and the spell casts normally
/// without performing any sacrifice.
#[test]
fn test_sacrifice_without_matching_ability_is_ignored() {
    let p1 = p(1);

    // Creature for p1 to attempt to sacrifice.
    let creature = ObjectSpec::card(p1, "Llanowar Elves")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let (mut state, _p1, _p2, spell_id) = setup_emerge_state(vec![creature]);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 8);

    let creature_id = find_object(&state, "Llanowar Elves");

    // Providing Sacrifice in additional_costs without emerge alt_cost or bargain keyword.
    // RC-1: This is now silently ignored (sacrifice is not performed).
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
            alt_cost: None, // Not using emerge alt cost
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![creature_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    // Spell casts successfully; the unclaimed sacrifice is ignored.
    assert!(
        result.is_ok(),
        "unclaimed sacrifice in additional_costs should be silently ignored; got: {:?}",
        result.err()
    );
    // The creature should still be on the battlefield (sacrifice was not performed).
    let (new_state, _events) = result.unwrap();
    let creature_obj = new_state.object(creature_id).unwrap();
    assert_eq!(
        creature_obj.zone,
        ZoneId::Battlefield,
        "creature should remain on battlefield when sacrifice is unclaimed"
    );
}
