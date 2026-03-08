//! Decayed keyword ability tests (CR 702.147).
//!
//! Decayed represents a static ability and a triggered ability:
//!   "This creature can't block" (static)
//!   "When this creature attacks, sacrifice it at end of combat" (triggered)
//!
//! Key rules verified:
//! - CR 702.147a: A creature with decayed can't block.
//! - CR 702.147a ruling: Decayed does not prevent attacking.
//! - CR 702.147a: Attacking with a decayed creature tags it for EOC sacrifice.
//! - CR 702.147a: A tagged creature is sacrificed at end of combat.
//! - Ruling 2021-09-24: Sacrifice persists even if decayed is removed after attack.
//! - Ruling 2021-09-24: Decayed does not grant haste (summoning sickness still applies).
//! - Ruling 2021-09-24: Decayed does not create attacking requirements.
//! - Non-decayed creatures can block normally (baseline).

use mtg_engine::{
    process_command, AttackTarget, CardRegistry, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use mtg_engine::{zombie_decayed_token_spec, Effect};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn is_on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
}

fn is_in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .values()
        .any(|obj| obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(owner))
}

/// Advance through priority passes until the step changes (or turn number changes).
fn pass_until_advance(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let start_step = state.turn.step;
    let start_turn = state.turn.turn_number;
    let mut all_events = Vec::new();
    let mut current = state;
    loop {
        let holder = current.turn.priority_holder.expect("no priority holder");
        let (new_state, ev) = process_command(current, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        let step_changed =
            new_state.turn.step != start_step || new_state.turn.turn_number != start_turn;
        all_events.extend(ev);
        current = new_state;
        if step_changed {
            return (current, all_events);
        }
        // Safety guard: if same holder got priority again and step hasn't changed,
        // try passing for the other player once more.
        if current.turn.priority_holder == Some(holder) {
            let other = players.iter().find(|&&p| p != holder).copied();
            if let Some(other_p) = other {
                let (ns, ev) = process_command(current, Command::PassPriority { player: other_p })
                    .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", other_p, e));
                all_events.extend(ev);
                current = ns;
                if current.turn.step != start_step || current.turn.turn_number != start_turn {
                    return (current, all_events);
                }
            }
            break;
        }
    }
    (current, all_events)
}

// ── Test 1: Decayed creature cannot block ─────────────────────────────────────

#[test]
/// CR 702.147a — A creature with decayed can't block.
fn test_702_147_decayed_creature_cannot_block() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(
            ObjectSpec::creature(p2, "Decayed Creature", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Decayed Creature");

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
        "CR 702.147a: A creature with decayed should not be able to block"
    );
}

// ── Test 2: Decayed creature CAN attack ──────────────────────────────────────

#[test]
/// CR 702.147a ruling 2021-09-24 — Decayed does not prevent attacking.
fn test_702_147_decayed_creature_can_attack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Decayed Attacker", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Decayed Attacker");

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
        "CR 702.147a ruling: Decayed does not prevent attacking: {:?}",
        result.err()
    );

    let (state, _) = result.unwrap();
    // Verify the creature is registered as an attacker.
    assert!(
        state
            .combat
            .as_ref()
            .map(|c| c.attackers.contains_key(&attacker_id))
            .unwrap_or(false),
        "Decayed creature should be registered as an attacker"
    );
}

// ── Test 3: Flag is set on attack ─────────────────────────────────────────────

#[test]
/// CR 702.147a — After a decayed creature attacks, decayed_sacrifice_at_eoc flag
/// must be set to true on the creature object.
fn test_702_147_decayed_flag_set_on_attack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Decayed Attacker", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Decayed Attacker");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // The creature may have a new ObjectId after the attack (zone changes on tap).
    // Search by name since the attacker stays on the battlefield.
    let decayed_obj = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Decayed Attacker")
        .expect("Decayed Attacker should still be on the battlefield");

    assert!(
        decayed_obj.decayed_sacrifice_at_eoc,
        "CR 702.147a: decayed_sacrifice_at_eoc should be true after attack declaration"
    );
}

// ── Test 4: Decayed creature sacrificed at end of combat ─────────────────────

#[test]
/// CR 702.147a — "When this creature attacks, sacrifice it at end of combat."
/// After attacking, the decayed creature should be in the graveyard after EOC.
fn test_702_147_decayed_creature_sacrificed_at_eoc() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Decayed Attacker", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Decayed Attacker");

    // Declare the decayed creature as attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Advance to DeclareBlockers step.
    let (state, _) = pass_until_advance(state, &[p1, p2]);

    // Declare no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("DeclareBlockers (empty) should succeed");

    // Advance through CombatDamage step.
    let (state, _) = pass_until_advance(state, &[p1, p2]);

    // Advance through EndOfCombat step (where sacrifice happens).
    let (state, events) = pass_until_advance(state, &[p1, p2]);

    // Verify CreatureDied event was emitted.
    let creature_died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        creature_died,
        "CR 702.147a: CreatureDied event should be emitted when decayed creature is sacrificed at EOC"
    );

    // Verify the creature is in the graveyard, not on the battlefield.
    assert!(
        !is_on_battlefield(&state, "Decayed Attacker"),
        "CR 702.147a: Decayed attacker should not be on the battlefield after EOC"
    );
    assert!(
        is_in_graveyard(&state, "Decayed Attacker", p1),
        "CR 702.147a: Decayed attacker should be in the graveyard after EOC sacrifice"
    );
}

// ── Test 5: Sacrifice persists after losing decayed keyword ───────────────────

#[test]
/// CR 702.147a ruling 2021-09-24 — "Once a creature with decayed attacks, it will
/// be sacrificed at end of combat, even if it no longer has decayed at that time."
fn test_702_147_decayed_sacrifice_persists_after_losing_keyword() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Decayed Attacker", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Decayed Attacker");

    // Declare the decayed creature as attacker (flag is set here).
    let (mut state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Verify the flag is set.
    let obj = state
        .objects
        .values()
        .find(|obj| obj.characteristics.name == "Decayed Attacker")
        .expect("Decayed Attacker should exist");
    assert!(
        obj.decayed_sacrifice_at_eoc,
        "flag should be set after attack"
    );
    let obj_id = obj.id;

    // Simulate losing the Decayed keyword by directly removing it from the
    // object's characteristics (no layer effect — just removing from base set).
    // This models a "loses all abilities" or enchantment removal effect.
    if let Some(obj_mut) = state.objects.get_mut(&obj_id) {
        obj_mut
            .characteristics
            .keywords
            .remove(&KeywordAbility::Decayed);
    }

    // Verify keyword is gone but flag is still set.
    let obj = state.objects.get(&obj_id).unwrap();
    assert!(
        !obj.characteristics
            .keywords
            .contains(&KeywordAbility::Decayed),
        "Decayed keyword should be removed"
    );
    assert!(
        obj.decayed_sacrifice_at_eoc,
        "flag should still be set even after keyword removed"
    );

    // Advance to DeclareBlockers.
    let (state, _) = pass_until_advance(state, &[p1, p2]);

    // Declare no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("DeclareBlockers (empty) should succeed");

    // Advance through CombatDamage.
    let (state, _) = pass_until_advance(state, &[p1, p2]);

    // Advance through EndOfCombat (sacrifice happens here).
    let (state, events) = pass_until_advance(state, &[p1, p2]);

    // Creature should still be sacrificed despite losing decayed.
    let creature_died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        creature_died || is_in_graveyard(&state, "Decayed Attacker", p1),
        "Ruling 2021-09-24: Sacrifice should occur even after losing decayed keyword"
    );
    assert!(
        !is_on_battlefield(&state, "Decayed Attacker"),
        "Ruling 2021-09-24: Creature should not be on battlefield after EOC sacrifice"
    );
}

// ── Test 6: Decayed creature NOT sacrificed if it doesn't attack ──────────────

#[test]
/// CR 702.147a ruling 2021-09-24 — "Decayed does not create any attacking
/// requirements. You may choose not to attack with a creature that has decayed."
/// A decayed creature that does not attack should NOT be sacrificed at EOC.
///
/// Note: CR 508.1 / turn_structure.rs: When no attackers are declared, the step
/// transitions directly from DeclareAttackers to EndOfCombat (no DeclareBlockers,
/// no CombatDamage). We pass through EndOfCombat and verify the creature survives.
fn test_702_147_decayed_creature_not_sacrificed_if_not_attacking() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Decayed Non-Attacker", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    // Declare NO attackers (skip the decayed creature).
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers with empty list should succeed");

    // With no attackers, the engine transitions directly from DeclareAttackers
    // to EndOfCombat (skipping DeclareBlockers and CombatDamage).
    // Advance past EndOfCombat — the creature should NOT be sacrificed.
    let (state, _) = pass_until_advance(state, &[p1, p2]);

    // Creature should still be on the battlefield — it never attacked.
    assert!(
        is_on_battlefield(&state, "Decayed Non-Attacker"),
        "Ruling 2021-09-24: A decayed creature that did not attack should not be sacrificed"
    );
    assert!(
        !is_in_graveyard(&state, "Decayed Non-Attacker", p1),
        "Ruling 2021-09-24: Non-attacking decayed creature should not be in the graveyard"
    );
}

// ── Test 7: Non-decayed creature can block normally ───────────────────────────

#[test]
/// CR 702.147a — Baseline: a creature without decayed can block normally.
fn test_702_147_non_decayed_creature_can_block() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(ObjectSpec::creature(p2, "Normal Blocker", 2, 2))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Normal Blocker");

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
        "CR 702.147a baseline: A non-decayed creature should be able to block normally: {:?}",
        result.err()
    );
}

// ── Test 8: Decayed creature does not grant haste ─────────────────────────────

#[test]
/// CR 702.147a ruling 2021-09-24 — "Decayed does not grant haste."
/// A creature with decayed that entered the battlefield this turn (has summoning
/// sickness) cannot attack.
fn test_702_147_decayed_no_haste() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a state with a decayed creature that HAS summoning sickness.
    // ObjectSpec::creature() in the builder sets has_summoning_sickness: false,
    // so we build normally first then set the flag.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Decayed Sick Creature", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Decayed Sick Creature");

    // Manually set summoning sickness (simulates creature that just entered this turn).
    let mut state = state;
    if let Some(obj) = state.objects.get_mut(&attacker_id) {
        obj.has_summoning_sickness = true;
    }

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    );

    assert!(
        result.is_err(),
        "Ruling 2021-09-24: Decayed does not grant haste — creature with summoning sickness cannot attack"
    );
}

// ── Token-Specific Tests ───────────────────────────────────────────────────────
//
// These tests verify that a Decayed token created via Effect::CreateToken (using
// zombie_decayed_token_spec) or via ObjectSpec::creature().token() receives the
// Decayed keyword through make_token() and is subject to the same enforcement as
// a non-token creature with Decayed.

// ── Test 9: Token created with Decayed keyword via make_token ─────────────────

#[test]
/// CR 702.147a — A 2/2 black Zombie creature token created via Effect::CreateToken
/// with zombie_decayed_token_spec must carry the Decayed keyword on the resulting
/// GameObject. This exercises the make_token() path in effects/mod.rs.
fn test_702_147_decayed_token_created_with_keyword() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source = ObjectId(0);
    let spec = zombie_decayed_token_spec(1);
    let effect = Effect::CreateToken { spec };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    // The Zombie token should be on the battlefield with the Decayed keyword.
    let token = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Zombie" && o.zone == ZoneId::Battlefield)
        .expect("CR 702.147a: Zombie Decayed token should be on battlefield after Effect::CreateToken");

    assert!(
        token.is_token,
        "CR 702.147a: created object must be flagged as a token"
    );
    assert!(
        token
            .characteristics
            .keywords
            .contains(&KeywordAbility::Decayed),
        "CR 702.147a: make_token must propagate Decayed keyword from TokenSpec to token characteristics"
    );
    assert_eq!(
        token.characteristics.power,
        Some(2),
        "CR 702.147a: Zombie Decayed token must be a 2/2"
    );
    assert_eq!(
        token.characteristics.toughness,
        Some(2),
        "CR 702.147a: Zombie Decayed token must be a 2/2"
    );
}

// ── Test 10: Decayed token cannot block ───────────────────────────────────────

#[test]
/// CR 702.147a — A 2/2 black Zombie creature token with Decayed cannot be declared
/// as a blocker. Verifies that can't-block enforcement applies to tokens, not just
/// non-token creatures.
fn test_702_147_decayed_token_cannot_block() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Normal Attacker", 2, 2))
        .object(
            ObjectSpec::creature(p2, "Zombie Token", 2, 2)
                .with_keyword(KeywordAbility::Decayed)
                .token(),
        )
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Normal Attacker");
    let blocker_id = find_object(&state, "Zombie Token");

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
        "CR 702.147a: A Zombie Decayed token should not be able to block"
    );
}

// ── Test 11: Decayed token sacrificed at end of combat after attacking ─────────

#[test]
/// CR 702.147a — "When this creature attacks, sacrifice it at end of combat."
/// A Zombie Decayed token that attacks must be sacrificed at EOC. Tokens are then
/// cleaned up by the SBA (CR 704.5d) — they cease to exist when leaving the battlefield.
fn test_702_147_decayed_token_sacrificed_at_eoc() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Zombie Token", 2, 2)
                .with_keyword(KeywordAbility::Decayed)
                .token(),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Zombie Token");

    // Declare the Decayed token as attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed for Decayed token");

    // Advance to DeclareBlockers step.
    let (state, _) = pass_until_advance(state, &[p1, p2]);

    // Declare no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("DeclareBlockers (empty) should succeed");

    // Advance through CombatDamage step.
    let (state, _) = pass_until_advance(state, &[p1, p2]);

    // Advance through EndOfCombat step (where sacrifice happens).
    let (state, events) = pass_until_advance(state, &[p1, p2]);

    // Verify CreatureDied event was emitted.
    let creature_died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        creature_died,
        "CR 702.147a: CreatureDied event should be emitted when Decayed token is sacrificed at EOC"
    );

    // Verify the token is no longer on the battlefield (tokens cease to exist in non-bf zones).
    assert!(
        !is_on_battlefield(&state, "Zombie Token"),
        "CR 702.147a / CR 704.5d: Decayed token should not be on the battlefield after EOC sacrifice"
    );
}

// ── Test 12: Decayed token with summoning sickness cannot attack ───────────────

#[test]
/// CR 702.147a ruling 2021-09-24 — "Decayed does not grant haste."
/// A Zombie Decayed token with summoning sickness cannot attack. This validates
/// that the token variant of the no-haste ruling also holds.
fn test_702_147_decayed_token_has_summoning_sickness() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::creature(p1, "Zombie Token", 2, 2)
                .with_keyword(KeywordAbility::Decayed)
                .token(),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Zombie Token");

    // Manually set summoning sickness (simulates a token that entered this turn).
    let mut state = state;
    if let Some(obj) = state.objects.get_mut(&attacker_id) {
        obj.has_summoning_sickness = true;
    }

    let result = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    );

    assert!(
        result.is_err(),
        "Ruling 2021-09-24: Decayed token with summoning sickness cannot attack — Decayed does not grant haste"
    );
}
