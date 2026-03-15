//! Toxic keyword ability enforcement tests (CR 702.164).
//!
//! Toxic is a STATIC ability, not a triggered ability (CR 702.164a).
//! "Combat damage dealt to a player by a creature with toxic causes that
//! creature's controller to give the player a number of poison counters
//! equal to that creature's total toxic value, in addition to the damage's
//! other results."
//!
//! Key differences from Poisonous (CR 702.70):
//! - Toxic applies inline as part of the combat damage event (no stack object).
//! - Toxic is additive with normal damage: life IS lost AND poison IS given.
//! - Multiple Toxic instances are cumulative: total toxic value = sum of all N.
//! - Poison counter count is independent of damage dealt (CR 702.164b).
//!
//! Tests:
//! - Basic: combat damage to player causes life loss AND toxic-value poison counters (CR 702.164c)
//! - Damage to creature does NOT give poison counters (CR 702.164c)
//! - Multiple Toxic instances are cumulative (CR 702.164b)
//! - All-damage-prevented case: no poison counters (CR 120.3g "combat damage dealt")
//! - Toxic + Infect: both apply independently (Infect gives damage-count, Toxic adds toxic-value)
//! - Toxic + Lifelink: both apply independently (CR ruling 2023-02-04)
//! - 10+ poison counters via Toxic triggers SBA loss (CR 704.5c)
//! - Multiplayer: only the attacked player receives poison counters (CR 702.164c)

use mtg_engine::{
    process_command, AbilityDefinition, AttackTarget, CardDefinition, CardId, CardRegistry,
    CardType, Command, GameEvent, GameStateBuilder, KeywordAbility, LossReason, ObjectSpec,
    PlayerId, Step, TypeLine,
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

// ── Helper: pass all priority ─────────────────────────────────────────────────

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

// ── Helper: poison counter count for a player ────────────────────────────────

fn poison_counters(state: &mtg_engine::GameState, player: PlayerId) -> u32 {
    state
        .players
        .get(&player)
        .map(|p| p.poison_counters)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
}

// ── CR 702.164: Toxic ─────────────────────────────────────────────────────────

#[test]
/// CR 702.164c / CR 120.3g — Combat damage dealt to a player by a creature with
/// Toxic causes the player to receive poison counters equal to the total toxic
/// value, IN ADDITION to the normal life loss from combat damage.
///
/// A 2/2 with Toxic 1 attacks unblocked. The defending player loses 2 life
/// AND gets 1 poison counter (not a replacement — both occur).
fn test_702_164_toxic_basic_combat_damage_gives_poison() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Toxic Attacker", 2, 2).with_keyword(KeywordAbility::Toxic(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Toxic Attacker");
    let initial_life = state.players[&p2].life_total;

    assert_eq!(
        poison_counters(&state, p2),
        0,
        "setup: p2 starts with 0 poison counters"
    );

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — inline Toxic applies.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // CR 120.3a: normal life loss occurs (2 damage from the 2/2).
    assert_eq!(
        state.players[&p2].life_total,
        initial_life - 2,
        "CR 120.3a: p2 should have lost 2 life from combat damage"
    );

    // CR 702.164c: p2 gets 1 poison counter equal to total toxic value.
    assert_eq!(
        poison_counters(&state, p2),
        1,
        "CR 702.164c: p2 should have 1 poison counter equal to Toxic 1 value"
    );

    // PoisonCountersGiven event must be emitted during the damage phase (not after trigger).
    let poison_given = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::PoisonCountersGiven { player, amount: 1, .. }
            if *player == p2
        )
    });
    assert!(
        poison_given,
        "CR 702.164c: PoisonCountersGiven event must be emitted inline during damage (not via trigger)"
    );

    // Toxic is static — no AbilityTriggered event for toxic, no stack object after damage.
    let toxic_triggered = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::AbilityTriggered { source_object_id, .. }
            if *source_object_id == attacker_id
        )
    });
    assert!(
        !toxic_triggered,
        "CR 702.164a: Toxic is a static ability — AbilityTriggered must NOT fire for Toxic"
    );

    // Stack must be empty immediately after combat damage (no trigger on stack).
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.164a: Toxic is static — stack must be empty after combat damage step"
    );

    // P1 is unaffected.
    assert_eq!(
        poison_counters(&state, p1),
        0,
        "p1 (attacker controller) should have 0 poison counters"
    );
}

#[test]
/// CR 702.164c / ruling 2023-02-04 — Toxic only applies when combat damage is
/// dealt to a player. A creature with Toxic that deals combat damage to a
/// blocking creature gives the blocking creature no poison counters; no player
/// receives poison counters.
fn test_702_164_toxic_damage_to_creature_no_poison() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Toxic Attacker", 2, 2).with_keyword(KeywordAbility::Toxic(1)),
        )
        .object(ObjectSpec::creature(p2, "Strong Blocker", 3, 3))
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Toxic Attacker");
    let blocker_id = find_object(&state, "Strong Blocker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 blocks with the 3/3.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — all damage goes to blocker (creature).
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // CR 702.164c: Toxic does NOT apply to combat damage dealt to creatures.
    assert_eq!(
        poison_counters(&state, p1),
        0,
        "CR 702.164c: p1 should have 0 poison counters (Toxic does not apply to creature damage)"
    );
    assert_eq!(
        poison_counters(&state, p2),
        0,
        "CR 702.164c: p2 should have 0 poison counters (Toxic does not apply when blocked)"
    );

    // PoisonCountersGiven must NOT fire.
    let poison_given = damage_events
        .iter()
        .any(|e| matches!(e, GameEvent::PoisonCountersGiven { .. }));
    assert!(
        !poison_given,
        "CR 702.164c: PoisonCountersGiven must NOT fire when Toxic creature's damage goes to a creature"
    );

    // P2's life total is unchanged (fully blocked).
    assert_eq!(
        state.players[&p2].life_total, 40,
        "p2's life should be unchanged (attack was fully blocked)"
    );
}

#[test]
/// CR 702.164b — A creature's total toxic value is the sum of ALL N values
/// of Toxic abilities on that creature. Multiple Toxic instances are cumulative.
///
/// A creature with Toxic 2 and Toxic 1 (total toxic value 3) deals combat
/// damage to a player. The player receives 3 poison counters, not 2 or 1.
fn test_702_164_toxic_multiple_instances_cumulative() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a card with two Toxic instances via CardDefinition.
    let double_toxic_def = CardDefinition {
        card_id: CardId("double-toxic-creature".to_string()),
        name: "Multi-Toxic Creature".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Toxic 2\nToxic 1".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Toxic(2)),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
        ],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    };

    let registry = CardRegistry::new(vec![double_toxic_def]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::creature(p1, "Multi-Toxic Creature", 1, 1)
                .with_keyword(KeywordAbility::Toxic(2))
                .with_keyword(KeywordAbility::Toxic(1))
                .with_card_id(CardId("double-toxic-creature".to_string())),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Multi-Toxic Creature");
    let initial_life = state.players[&p2].life_total;

    assert_eq!(
        poison_counters(&state, p2),
        0,
        "setup: p2 starts with 0 poison counters"
    );

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // CR 702.164b: total toxic value = Toxic 2 + Toxic 1 = 3 poison counters.
    assert_eq!(
        poison_counters(&state, p2),
        3,
        "CR 702.164b: p2 should have 3 poison counters (Toxic 2 + Toxic 1, cumulative total)"
    );

    // Life loss also occurs (Toxic is additive, not a replacement).
    assert_eq!(
        state.players[&p2].life_total,
        initial_life - 1,
        "CR 702.164c: p2 should have lost 1 life from the 1/1 combat damage"
    );

    // PoisonCountersGiven event with amount=3 must fire.
    let poison_given = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::PoisonCountersGiven { player, amount: 3, .. }
            if *player == p2
        )
    });
    assert!(
        poison_given,
        "CR 702.164b: PoisonCountersGiven should fire with amount=3 for cumulative Toxic 2+1"
    );

    // Stack is empty — Toxic is static, no trigger.
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.164a: Toxic is static — stack must be empty after combat damage"
    );
}

#[test]
/// CR 120.3g — Toxic applies only when combat damage IS dealt ("combat damage
/// dealt to a player"). If all combat damage is prevented (final_dmg == 0),
/// no damage is "dealt" and Toxic does not give poison counters.
///
/// We test this by using a 0/1 creature: power=0 means it assigns 0 combat
/// damage, so the creature deals 0 damage and Toxic never fires.
fn test_702_164_toxic_zero_damage_no_poison() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // A 0/1 creature with Toxic 2 — power is 0, so 0 combat damage is dealt.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Zero Power Toxic", 0, 1)
                .with_keyword(KeywordAbility::Toxic(2)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Zero Power Toxic");
    let initial_life = state.players[&p2].life_total;

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — 0 damage dealt.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // CR 120.3g: Toxic does not apply when 0 combat damage is "dealt".
    assert_eq!(
        poison_counters(&state, p2),
        0,
        "CR 120.3g: p2 should have 0 poison counters when 0 combat damage is dealt"
    );

    // Life total unchanged.
    assert_eq!(
        state.players[&p2].life_total, initial_life,
        "p2's life should be unchanged (0 damage from 0-power creature)"
    );

    // PoisonCountersGiven must NOT fire.
    let poison_given = damage_events
        .iter()
        .any(|e| matches!(e, GameEvent::PoisonCountersGiven { .. }));
    assert!(
        !poison_given,
        "CR 120.3g: PoisonCountersGiven must NOT fire when 0 combat damage is dealt"
    );
}

#[test]
/// CR 702.164c + CR 702.90b — A creature with BOTH Infect and Toxic deals
/// combat damage to a player. The effects are independent and cumulative:
/// - Infect gives poison counters equal to the damage amount (replaces life loss).
/// - Toxic gives additional poison counters equal to the total toxic value.
///
/// A 3/3 with Infect and Toxic 1 deals 3 combat damage to a player:
/// - Infect: player gets 3 poison counters (replaces 3 life loss).
/// - Toxic 1: player gets 1 more poison counter.
/// - Total: 4 poison counters, 0 life loss.
fn test_702_164_toxic_with_infect() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Infect-Toxic Attacker", 3, 3)
                .with_keyword(KeywordAbility::Infect)
                .with_keyword(KeywordAbility::Toxic(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Infect-Toxic Attacker");
    let initial_life = state.players[&p2].life_total;

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step.
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    // CR 702.90b: Infect replaces life loss — p2's life total must be UNCHANGED.
    assert_eq!(
        state.players[&p2].life_total, initial_life,
        "CR 702.90b: Infect replaces life loss — p2 life should be unchanged"
    );

    // Infect gives 3 poison (from 3 damage) + Toxic 1 gives 1 more = 4 total.
    assert_eq!(
        poison_counters(&state, p2),
        4,
        "CR 702.164c + CR 702.90b: Infect gives 3 + Toxic 1 gives 1 = 4 poison counters total"
    );

    // PoisonCountersGiven events: one for Infect (amount=3) and one for Toxic (amount=1).
    let infect_poison = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::PoisonCountersGiven { player, amount: 3, .. }
            if *player == p2
        )
    });
    assert!(
        infect_poison,
        "CR 702.90b: PoisonCountersGiven amount=3 must fire for Infect poison"
    );

    let toxic_poison = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::PoisonCountersGiven { player, amount: 1, .. }
            if *player == p2
        )
    });
    assert!(
        toxic_poison,
        "CR 702.164c: PoisonCountersGiven amount=1 must fire for Toxic 1 poison (in addition to Infect)"
    );

    // Stack is empty (Toxic is static; Infect is also inline, not a trigger).
    assert!(
        state.stack_objects.is_empty(),
        "stack must be empty after combat damage (both Infect and Toxic are static)"
    );
}

#[test]
/// CR 702.164c + CR 702.15a + ruling 2023-02-04 — Toxic and Lifelink are
/// independent. A creature with Toxic 1 and Lifelink deals 2 combat damage
/// to a player:
/// - The player loses 2 life (normal damage, CR 120.3a).
/// - The player gets 1 poison counter (Toxic 1, CR 702.164c).
/// - The attacker's controller gains 2 life (Lifelink, CR 702.15a).
fn test_702_164_toxic_with_lifelink() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let initial_p1_life = 40i32;
    let initial_p2_life = 40i32;

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Lifelink-Toxic Attacker", 2, 2)
                .with_keyword(KeywordAbility::Lifelink)
                .with_keyword(KeywordAbility::Toxic(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Lifelink-Toxic Attacker");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.15a: Lifelink — p1 gains 2 life (40 + 2 = 42).
    assert_eq!(
        state.players[&p1].life_total,
        initial_p1_life + 2,
        "CR 702.15a: p1 should have gained 2 life from Lifelink"
    );

    // CR 120.3a: normal life loss — p2 loses 2 life (40 - 2 = 38).
    assert_eq!(
        state.players[&p2].life_total,
        initial_p2_life - 2,
        "CR 120.3a: p2 should have lost 2 life from combat damage"
    );

    // CR 702.164c: Toxic 1 gives 1 poison counter.
    assert_eq!(
        poison_counters(&state, p2),
        1,
        "CR 702.164c: p2 should have 1 poison counter from Toxic 1"
    );

    // Lifelink controller should NOT receive poison counters.
    assert_eq!(
        poison_counters(&state, p1),
        0,
        "ruling 2023-02-04: p1 (lifelink controller) should have 0 poison counters"
    );
}

#[test]
/// CR 702.164c + CR 704.5c — A player at 9 poison counters is dealt combat
/// damage by a creature with Toxic 1. The player gets their 10th poison
/// counter and the SBA check marks them as having lost the game.
fn test_702_164_toxic_kills_via_poison_sba() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // P2 starts with 9 poison counters.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_poison(p2, 9)
        .object(
            ObjectSpec::creature(p1, "Lethal Toxic", 1, 1).with_keyword(KeywordAbility::Toxic(1)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Lethal Toxic");

    assert_eq!(
        poison_counters(&state, p2),
        9,
        "setup: p2 starts with 9 poison counters"
    );
    assert!(
        !state.players.get(&p2).unwrap().has_lost,
        "setup: p2 has not yet lost"
    );

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers.
    let (state, _) = pass_all(state, &[p1, p2]);

    // P2 declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("declare blockers failed");

    // Advance through CombatDamage step — Toxic 1 gives the 10th poison counter.
    let (state, events) = pass_all(state, &[p1, p2]);

    // P2 now has 10 poison counters.
    assert_eq!(
        poison_counters(&state, p2),
        10,
        "CR 702.164c: p2 should have 10 poison counters after Toxic 1 fires"
    );

    // CR 704.5c: SBA check must have marked P2 as having lost.
    let p2_lost = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::PlayerLost { player, reason: LossReason::PoisonCounters }
            if *player == p2
        )
    });
    assert!(
        p2_lost,
        "CR 704.5c: p2 must lose the game (PlayerLost event) when poison counters reach 10 via Toxic"
    );

    assert!(
        state.players.get(&p2).unwrap().has_lost,
        "CR 704.5c: p2.has_lost must be true after 10 poison counters via Toxic"
    );
}

#[test]
/// CR 702.164c — In a multiplayer game, only the player who receives combat
/// damage from the Toxic creature gets poison counters. Other players are
/// unaffected.
///
/// P1 controls a Toxic 2 creature and attacks P3. P2 and P4 are uninvolved.
/// After combat damage, only P3 has 2 poison counters; P2 and P4 have 0.
fn test_702_164_toxic_multiplayer_correct_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .object(
            ObjectSpec::creature(p1, "Multiplayer Toxic", 2, 2)
                .with_keyword(KeywordAbility::Toxic(2)),
        )
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Multiplayer Toxic");

    assert_eq!(poison_counters(&state, p2), 0, "setup: p2 has 0 poison");
    assert_eq!(poison_counters(&state, p3), 0, "setup: p3 has 0 poison");
    assert_eq!(poison_counters(&state, p4), 0, "setup: p4 has 0 poison");

    // P1 attacks P3 specifically.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p3))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers failed");

    // Advance to DeclareBlockers (all 4 players pass).
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // P3 is the defending player — declares no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p3,
            blockers: vec![],
        },
    )
    .expect("p3 declare blockers failed");

    // Advance through CombatDamage step.
    let (state, damage_events) = pass_all(state, &[p1, p2, p3, p4]);

    // Only P3 should have received poison counters.
    assert_eq!(
        poison_counters(&state, p2),
        0,
        "CR 702.164c: p2 should have 0 poison counters (was not attacked)"
    );
    assert_eq!(
        poison_counters(&state, p3),
        2,
        "CR 702.164c: p3 should have 2 poison counters (Toxic 2, attacked player)"
    );
    assert_eq!(
        poison_counters(&state, p4),
        0,
        "CR 702.164c: p4 should have 0 poison counters (was not attacked)"
    );

    // P3's life also decreased (Toxic is additive, not a replacement).
    assert_eq!(
        state.players[&p3].life_total, 38,
        "CR 702.164c: p3 should have lost 2 life from 2/2 combat damage (Toxic is additive)"
    );

    // PoisonCountersGiven for P3 with amount=2.
    let poison_given = damage_events.iter().any(|e| {
        matches!(
            e,
            GameEvent::PoisonCountersGiven { player, amount: 2, .. }
            if *player == p3
        )
    });
    assert!(
        poison_given,
        "CR 702.164c: PoisonCountersGiven should fire with player=p3, amount=2"
    );

    // Stack is empty (Toxic is static).
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.164a: Toxic is static — stack must be empty after combat damage"
    );
}
