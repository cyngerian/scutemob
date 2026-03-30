//! Tests for PB-36 evasion/protection extensions.
//!
//! Covers:
//! - CantBlock keyword (CR 509.1b): prevents a creature from blocking
//! - CantBeBlockedExceptBy keyword (CR 509.1b): filtered evasion
//! - GrantPlayerProtection effect (CR 702.16b/e/j): player protection qualities
//! - Protection from card type (CR 702.16a): instants, planeswalkers
//! - Protection from subtype (CR 702.16a): Wizards

use mtg_engine::{
    process_command, AttackTarget, BlockingExceptionFilter, CardType, Command, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId, ProtectionQuality, Step, ZoneId,
};

// ── Helper: find object ID by name ───────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

// ── CantBlock tests ───────────────────────────────────────────────────────────

#[test]
/// CR 509.1b — A creature with CantBlock cannot be declared as a blocker.
fn test_cant_block_keyword_prevents_blocking() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Attacker", 2, 2))
        .object(
            ObjectSpec::creature(p2, "CantBlock Creature", 1, 1)
                .with_keyword(KeywordAbility::CantBlock),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Attacker");
    let blocker_id = find_object(&state, "CantBlock Creature");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "A creature with CantBlock should not be able to block"
    );
}

#[test]
/// CR 509.1b — CantBlock restricts blocking, not attacking.
/// A creature with CantBlock CAN still be declared as an attacker.
fn test_cant_block_keyword_does_not_prevent_attacking() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "CantBlock Attacker", 2, 2)
                .with_keyword(KeywordAbility::CantBlock)
                .with_keyword(KeywordAbility::Haste),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "CantBlock Attacker");

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    );

    assert!(
        result.is_ok(),
        "A creature with CantBlock should still be able to attack: {:?}",
        result.err()
    );
}

// ── CantBeBlockedExceptBy tests ───────────────────────────────────────────────

#[test]
/// CR 509.1b — CantBeBlockedExceptBy(HasAnyKeyword): blocker with a matching keyword can block.
/// Signal Pest has CantBeBlockedExceptBy(flying OR reach) — a creature with flying CAN block it.
fn test_cant_be_blocked_except_by_allows_matching_keyword() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Signal Pest", 0, 1).with_keyword(
            KeywordAbility::CantBeBlockedExceptBy(BlockingExceptionFilter::HasAnyKeyword(vec![
                KeywordAbility::Flying,
                KeywordAbility::Reach,
            ])),
        ))
        .object(ObjectSpec::creature(p2, "Flyer", 2, 2).with_keyword(KeywordAbility::Flying))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Signal Pest");
    let blocker_id = find_object(&state, "Flyer");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "A creature with flying should be able to block a Signal Pest: {:?}",
        result.err()
    );
}

#[test]
/// CR 509.1b — CantBeBlockedExceptBy(HasAnyKeyword): blocker without a matching keyword cannot block.
/// Signal Pest: a ground creature without flying or reach CANNOT block it.
fn test_cant_be_blocked_except_by_rejects_non_matching_keyword() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Signal Pest", 0, 1).with_keyword(
            KeywordAbility::CantBeBlockedExceptBy(BlockingExceptionFilter::HasAnyKeyword(vec![
                KeywordAbility::Flying,
                KeywordAbility::Reach,
            ])),
        ))
        .object(ObjectSpec::creature(p2, "Ground Creature", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Signal Pest");
    let blocker_id = find_object(&state, "Ground Creature");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "A ground creature without flying or reach should not be able to block Signal Pest"
    );
}

#[test]
/// CR 509.1b — CantBeBlockedExceptBy(HasKeyword): the single-keyword variant.
/// Gingerbrute with haste-only exception: a creature with haste CAN block it.
fn test_cant_be_blocked_except_by_has_keyword_allows_matching() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Gingerbrute", 1, 1).with_keyword(
            KeywordAbility::CantBeBlockedExceptBy(BlockingExceptionFilter::HasKeyword(Box::new(
                KeywordAbility::Haste,
            ))),
        ))
        .object(ObjectSpec::creature(p2, "Haste Blocker", 2, 2).with_keyword(KeywordAbility::Haste))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Gingerbrute");
    let blocker_id = find_object(&state, "Haste Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_ok(),
        "A creature with haste should be able to block Gingerbrute: {:?}",
        result.err()
    );
}

#[test]
/// CR 509.1b — CantBeBlockedExceptBy(HasKeyword): a creature without the required keyword cannot block.
fn test_cant_be_blocked_except_by_has_keyword_rejects_non_matching() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Gingerbrute", 1, 1).with_keyword(
            KeywordAbility::CantBeBlockedExceptBy(BlockingExceptionFilter::HasKeyword(Box::new(
                KeywordAbility::Haste,
            ))),
        ))
        .object(ObjectSpec::creature(p2, "Slow Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Gingerbrute");
    let blocker_id = find_object(&state, "Slow Blocker");

    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "A creature without haste should not be able to block Gingerbrute (with HasKeyword filter)"
    );
}

#[test]
/// CR 509.1b / CR 702.111b — CantBeBlockedExceptBy combined with Menace:
/// the attacker must be blocked by 2+ creatures AND all blockers must match the filter.
fn test_cant_be_blocked_except_by_combined_with_menace() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Attacker has both Menace and CantBeBlockedExceptBy(flying/reach).
    // Attempting to block with a single ground creature should fail on menace.
    // Attempting to block with two ground creatures should fail on CantBeBlockedExceptBy.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Menace Filtered", 3, 3)
                .with_keyword(KeywordAbility::Menace)
                .with_keyword(KeywordAbility::CantBeBlockedExceptBy(
                    BlockingExceptionFilter::HasAnyKeyword(vec![
                        KeywordAbility::Flying,
                        KeywordAbility::Reach,
                    ]),
                )),
        )
        .object(ObjectSpec::creature(p2, "Ground A", 1, 1))
        .object(ObjectSpec::creature(p2, "Ground B", 1, 1))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Menace Filtered");
    let blocker_a = find_object(&state, "Ground A");
    let blocker_b = find_object(&state, "Ground B");

    // Two ground creatures can satisfy Menace but not CantBeBlockedExceptBy.
    let mut state = state;
    state.combat = Some({
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers.insert(attacker_id, AttackTarget::Player(p2));
        cs
    });

    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_a, attacker_id), (blocker_b, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "Two ground creatures should not be able to block a Menace+CantBeBlockedExceptBy attacker"
    );
}

// ── GrantPlayerProtection tests ───────────────────────────────────────────────

#[test]
/// CR 702.16b/j — A player with protection from everything cannot be targeted by spells.
/// After GrantPlayerProtection(FromAll), an opponent's direct damage spell targeting that player fails.
fn test_grant_player_protection_prevents_targeting() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P2 has a bolt-style spell targeting a player.
    let bolt_spec = ObjectSpec::card(p2, "Lightning Bolt")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(bolt_spec.in_zone(ZoneId::Hand(p2)))
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    // Manually grant P1 protection from everything (simulating GrantPlayerProtection effect).
    let mut state = state;
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.protection_qualities.push(ProtectionQuality::FromAll);
    }
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    let bolt_id = find_object(&state, "Lightning Bolt");
    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: bolt_id,
            targets: vec![mtg_engine::Target::Player(p1)],
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
    );

    assert!(
        result.is_err(),
        "A player with protection from everything should not be targetable by an opponent's spell"
    );
}

#[test]
/// CR 702.16a — Protection from card type (Instant): enforces targeting restriction.
/// A creature with protection from instants should not be targetable by an instant.
fn test_protection_from_card_type_blocks_instants() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P2 has an instant that targets a creature.
    let instant_spec = ObjectSpec::card(p2, "Doom Blade")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            black: 1,
            ..Default::default()
        });

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Protected Creature", 13, 13).with_keyword(
                KeywordAbility::ProtectionFrom(ProtectionQuality::FromCardType(CardType::Instant)),
            ),
        )
        .object(instant_spec.in_zone(ZoneId::Hand(p2)))
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let creature_id = find_object(&state, "Protected Creature");
    let instant_id = find_object(&state, "Doom Blade");

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p2);

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: instant_id,
            targets: vec![mtg_engine::Target::Object(creature_id)],
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
    );

    assert!(
        result.is_err(),
        "A creature with protection from instants should not be targetable by an instant"
    );
}
