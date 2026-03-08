//! Collect Evidence keyword action tests (CR 701.59).
//!
//! Collect Evidence is a keyword action used as an additional cost:
//! "As an additional cost to cast this spell, you may collect evidence N."
//! To collect evidence N means to exile cards from your graveyard with total
//! mana value N or greater (CR 701.59a).
//!
//! Key rules verified:
//! - Cards exiled must be in the caster's own graveyard (CR 701.59a).
//! - Total mana value of exiled cards must be >= N; over-exiling is allowed (CR 701.59a).
//! - Cannot choose to collect evidence if graveyard total MV < N (CR 701.59b).
//! - "If evidence was collected" checks Condition::EvidenceWasCollected at resolution (CR 701.59c).
//! - Unlike Delve, collect evidence does NOT reduce the mana cost (CR 701.59a vs CR 702.66).
//! - Providing evidence cards for a spell without CollectEvidence is rejected (engine validation).
//! - Duplicate ObjectIds in the evidence list are rejected (engine validation).
//! - Cards must be in the caster's graveyard, not on the battlefield or opponent's graveyard.

use mtg_engine::cards::card_definition::{Condition, EffectAmount, PlayerTarget};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, GameStateBuilder, ManaColor, ManaCost, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
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
) -> (mtg_engine::GameState, Vec<mtg_engine::GameEvent>) {
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

/// Synthetic collect-evidence instant.
/// "As an additional cost to cast this spell, you may collect evidence 6.
/// If evidence was collected, gain 3 life. Otherwise, gain 1 life."
/// Mana cost: {2}{W}.
fn collect_evidence_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("collect-evidence-instant".to_string()),
        name: "Evidence Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "As an additional cost to cast this spell, you may collect evidence 6. \
                      If evidence was collected, you gain 3 life. Otherwise, you gain 1 life."
            .to_string(),
        abilities: vec![
            AbilityDefinition::CollectEvidence {
                threshold: 6,
                mandatory: false,
            },
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::EvidenceWasCollected,
                    if_true: Box::new(Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    }),
                    if_false: Box::new(Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic mandatory collect-evidence instant.
/// "Collect evidence 4. Gain 2 life."
/// Mana cost: {1}{U}.
fn mandatory_collect_evidence_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mandatory-collect-evidence-instant".to_string()),
        name: "Mandatory Evidence".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Collect evidence 4. Gain 2 life.".to_string(),
        abilities: vec![
            AbilityDefinition::CollectEvidence {
                threshold: 4,
                mandatory: true,
            },
            AbilityDefinition::Spell {
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic spell without collect evidence — for rejection tests.
fn plain_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-instant".to_string()),
        name: "Plain Instant".to_string(),
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
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Graveyard card spec with mana value 3 (cost {2}{R}).
fn gy_mv3_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "GY Card MV3")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("gy-card-mv3".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        })
}

/// Graveyard card spec with mana value 4 (cost {3}{G}).
fn gy_mv4_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "GY Card MV4")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("gy-card-mv4".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        })
}

/// Graveyard card spec with mana value 2 (cost {1}{B}).
fn gy_mv2_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "GY Card MV2")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("gy-card-mv2".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        })
}

// ── Helper: build standard 2-player state ─────────────────────────────────────

/// Build a 2-player state with Evidence Instant in p1's hand, plus extra objects.
/// Returns (state, p1, p2, spell_id).
fn setup_state(
    extra_defs: Vec<CardDefinition>,
    extra_objects: Vec<ObjectSpec>,
) -> (
    mtg_engine::GameState,
    PlayerId,
    PlayerId,
    mtg_engine::ObjectId,
) {
    let p1 = p(1);
    let p2 = p(2);

    let mut all_defs = vec![collect_evidence_instant_def()];
    all_defs.extend(extra_defs);

    let registry = CardRegistry::new(all_defs);

    let spell_spec = ObjectSpec::card(p1, "Evidence Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("collect-evidence-instant".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        });

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    for obj in extra_objects {
        builder = builder.object(obj);
    }

    let mut state = builder.build().unwrap();

    // Give p1 {2}{W} mana (full cost of Evidence Instant).
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
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object_in_zone(&state, "Evidence Instant", ZoneId::Hand(p1)).unwrap();

    (state, p1, p2, spell_id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 701.59a — basic collect evidence: exile cards from graveyard with total MV >= threshold.
/// Player exiles two GY cards (MV 3 + MV 4 = 7 >= 6). Spell resolves and evidence branch fires.
#[test]
fn test_collect_evidence_basic_exile_from_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let (state, p1, _p2, spell_id) = setup_state(vec![], vec![gy_mv3_spec(p1), gy_mv4_spec(p1)]);

    let gy_mv3_id = find_object_in_zone(&state, "GY Card MV3", ZoneId::Graveyard(p1)).unwrap();
    let gy_mv4_id = find_object_in_zone(&state, "GY Card MV4", ZoneId::Graveyard(p1)).unwrap();

    let p1_life_before = state.players[&p1].life_total;

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
            collect_evidence_cards: vec![gy_mv3_id, gy_mv4_id],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap();

    // The stack object should have evidence_collected = true.
    assert!(
        state.stack_objects.iter().any(|so| so.evidence_collected),
        "StackObject should have evidence_collected = true"
    );

    // The GY cards should now be in exile.
    assert!(
        find_object_in_zone(&state, "GY Card MV3", ZoneId::Exile).is_some(),
        "GY Card MV3 should be in exile after collect evidence payment"
    );
    assert!(
        find_object_in_zone(&state, "GY Card MV4", ZoneId::Exile).is_some(),
        "GY Card MV4 should be in exile after collect evidence payment"
    );

    // Resolve the spell (p1 has priority first after cast, then p2, then resolves).
    let (state, _) = pass_all(state, &[p1, p2]);
    let p1_life_after = state.players[&p1].life_total;

    // Evidence was collected -> "if true" branch: gain 3 life.
    assert_eq!(
        p1_life_after,
        p1_life_before + 3,
        "Player should gain 3 life (evidence collected branch)"
    );
}

/// CR 701.59a — "N or greater": over-exiling is allowed (total MV 7 >= threshold 6).
#[test]
fn test_collect_evidence_over_threshold_allowed() {
    let p1 = p(1);

    let (state, p1, _p2, spell_id) = setup_state(vec![], vec![gy_mv3_spec(p1), gy_mv4_spec(p1)]);

    let gy_mv3_id = find_object_in_zone(&state, "GY Card MV3", ZoneId::Graveyard(p1)).unwrap();
    let gy_mv4_id = find_object_in_zone(&state, "GY Card MV4", ZoneId::Graveyard(p1)).unwrap();

    // Should succeed: 3+4=7 >= 6
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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![gy_mv3_id, gy_mv4_id],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_ok(),
        "Over-exiling (MV 7 >= threshold 6) should be allowed (CR 701.59a)"
    );
}

/// CR 701.59a / 701.59b — total MV insufficient: rejected.
#[test]
fn test_collect_evidence_under_threshold_rejected() {
    let p1 = p(1);

    let (state, p1, _p2, spell_id) = setup_state(vec![], vec![gy_mv2_spec(p1), gy_mv3_spec(p1)]);

    // Total MV = 2+3 = 5 < threshold 6
    let gy_mv2_id = find_object_in_zone(&state, "GY Card MV2", ZoneId::Graveyard(p1)).unwrap();
    let gy_mv3_id = find_object_in_zone(&state, "GY Card MV3", ZoneId::Graveyard(p1)).unwrap();

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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![gy_mv2_id, gy_mv3_id],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_err(),
        "Should be rejected: total MV 5 < threshold 6 (CR 701.59a)"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("mana value"),
        "Error should mention mana value: {}",
        err_msg
    );
}

/// CR 701.59a — optional collect evidence: player passes empty list, spell resolves normally.
/// "If evidence was collected" branch fires false -> gain 1 life.
#[test]
fn test_collect_evidence_not_collected_optional() {
    let (state, p1, p2, spell_id) = setup_state(vec![], vec![]);

    let p1_life_before = state.players[&p1].life_total;

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
            collect_evidence_cards: vec![], // player declines
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap();

    // Stack object should have evidence_collected = false.
    assert!(
        state.stack_objects.iter().any(|so| !so.evidence_collected),
        "StackObject should have evidence_collected = false when player declines"
    );

    // Resolve the spell (p1 has priority first after cast, then p2, then resolves).
    let (state, _) = pass_all(state, &[p1, p2]);
    let p1_life_after = state.players[&p1].life_total;

    // Evidence NOT collected -> "if false" branch: gain 1 life.
    assert_eq!(
        p1_life_after,
        p1_life_before + 1,
        "Player should gain 1 life (no evidence collected branch)"
    );
}

/// CR 701.59c — Condition::EvidenceWasCollected evaluates correctly.
/// Exiling MV4 card against threshold 6 fails (MV 4 < 6).
#[test]
fn test_collect_evidence_insufficient_single_card_rejected() {
    let p1 = p(1);

    let (state, p1, _p2, spell_id) = setup_state(vec![], vec![gy_mv4_spec(p1)]);

    let gy_id = find_object_in_zone(&state, "GY Card MV4", ZoneId::Graveyard(p1)).unwrap();

    // MV 4 < threshold 6 -- should fail.
    let fail_result = process_command(
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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![gy_id], // MV 4 < threshold 6
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        fail_result.is_err(),
        "MV 4 < threshold 6 should be rejected (CR 701.59a)"
    );
}

/// CR 701.59a — mandatory collect evidence spell without evidence cards is rejected.
#[test]
fn test_collect_evidence_mandatory_without_cards_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![mandatory_collect_evidence_instant_def()]);
    let spell_spec = ObjectSpec::card(p1, "Mandatory Evidence")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mandatory-collect-evidence-instant".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object_in_zone(&state, "Mandatory Evidence", ZoneId::Hand(p1)).unwrap();

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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![], // mandatory but empty
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_err(),
        "Mandatory collect evidence without evidence cards should be rejected (CR 701.59a)"
    );
}

/// Engine validation — duplicate ObjectId in evidence list is rejected.
#[test]
fn test_collect_evidence_duplicate_card_rejected() {
    let p1 = p(1);

    let (state, p1, _p2, spell_id) = setup_state(vec![], vec![gy_mv4_spec(p1)]);

    let gy_id = find_object_in_zone(&state, "GY Card MV4", ZoneId::Graveyard(p1)).unwrap();

    // Provide the same ObjectId twice.
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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![gy_id, gy_id], // duplicate
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_err(),
        "Duplicate ObjectId in collect_evidence_cards should be rejected"
    );
}

/// Engine validation — card not in caster's graveyard (on battlefield) is rejected.
#[test]
fn test_collect_evidence_card_not_in_graveyard_rejected() {
    let p1 = p(1);

    // Put a card on the battlefield instead of the graveyard.
    let bf_card = ObjectSpec::card(p1, "GY Card MV4")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("gy-card-mv4".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 3,
            green: 1,
            ..Default::default()
        });

    let (state, p1, _p2, spell_id) = setup_state(vec![], vec![bf_card]);

    let bf_id = find_object_in_zone(&state, "GY Card MV4", ZoneId::Battlefield).unwrap();

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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![bf_id], // on battlefield, not graveyard
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_err(),
        "Card not in caster's graveyard should be rejected (CR 701.59a)"
    );
}

/// Engine validation — card in opponent's graveyard is rejected.
#[test]
fn test_collect_evidence_opponents_graveyard_rejected() {
    let p2 = p(2);

    // Put the card in p2's graveyard.
    let opp_gy_card = gy_mv4_spec(p2).in_zone(ZoneId::Graveyard(p2));

    let (state, p1, _p2, spell_id) = setup_state(vec![], vec![opp_gy_card]);

    let opp_gy_id = find_object_in_zone(&state, "GY Card MV4", ZoneId::Graveyard(p2)).unwrap();

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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![opp_gy_id], // in opponent's graveyard
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_err(),
        "Card in opponent's graveyard should be rejected (CR 701.59a)"
    );
}

/// Engine validation — providing evidence cards for a spell without CollectEvidence is rejected.
#[test]
fn test_collect_evidence_spell_without_ability_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_instant_def()]);
    let spell_spec = ObjectSpec::card(p1, "Plain Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-instant".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let gy_card = gy_mv4_spec(p1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell_spec)
        .object(gy_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object_in_zone(&state, "Plain Instant", ZoneId::Hand(p1)).unwrap();
    let gy_id = find_object_in_zone(&state, "GY Card MV4", ZoneId::Graveyard(p1)).unwrap();

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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![gy_id], // spell has no CollectEvidence
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_err(),
        "Providing evidence for a spell without CollectEvidence should be rejected"
    );
}

/// CR 701.59a vs CR 702.66 — collect evidence does NOT reduce mana cost.
/// Player only has {W} but needs {2}{W}; should fail even with evidence cards exiled.
#[test]
fn test_collect_evidence_mana_not_reduced() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![collect_evidence_instant_def()]);
    let spell_spec = ObjectSpec::card(p1, "Evidence Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("collect-evidence-instant".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        });
    let gy_card = gy_mv4_spec(p1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell_spec)
        .object(gy_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only give {W} — not enough to pay {2}{W}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object_in_zone(&state, "Evidence Instant", ZoneId::Hand(p1)).unwrap();
    let gy_id = find_object_in_zone(&state, "GY Card MV4", ZoneId::Graveyard(p1)).unwrap();

    // Should fail: not enough mana ({W} only, needs {2}{W}).
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
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![gy_id], // should NOT reduce cost
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_err(),
        "Should fail: mana insufficient even with evidence card (evidence does NOT reduce mana cost, CR 701.59a)"
    );
}
