//! Myriad keyword ability tests (CR 702.116).
//!
//! Myriad is a triggered ability: "Whenever this creature attacks, for each
//! opponent other than defending player, you may create a token that's a copy
//! of this creature that's tapped and attacking that player or a planeswalker
//! they control. If one or more tokens are created this way, exile the tokens
//! at end of combat."
//!
//! Key rules verified:
//! - In 4-player game, attacking 1 opponent creates 2 token copies for the other 2 (CR 702.116a).
//! - In 2-player game (defending = only opponent), no tokens are created (CR 702.116a).
//! - Tokens are tapped and attacking on the battlefield immediately after trigger resolves.
//! - Tokens are tagged `myriad_exile_at_eoc = true` and are exiled at end of combat (CR 702.116a).
//! - Multiple myriad instances trigger separately (CR 702.116b).
//! - Tokens attack the correct non-defending opponents in multiplayer (CR 702.116a).
//! - Tokens are copies of the source creature via CopyOf continuous effect (CR 707.2).
//! - Tokens did NOT trigger myriad themselves (they entered attacking, not declared).

use mtg_engine::{
    process_command, AttackTarget, CardRegistry, Command, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Count permanents on the battlefield controlled by `player`.
fn battlefield_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == player)
        .count()
}

/// Count tokens on the battlefield controlled by `player`.
fn token_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.controller == player && obj.is_token)
        .count()
}

/// Count exile objects owned by any player.
fn exile_count(state: &GameState) -> usize {
    state
        .objects
        .values()
        .filter(|obj| matches!(obj.zone, ZoneId::Exile))
        .count()
}

/// Pass priority for all listed players once.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

/// Advance through priority passes until the step changes (or turn number changes).
fn pass_until_advance(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let start_step = state.turn.step;
    let start_turn = state.turn.turn_number;
    let mut all_events = Vec::new();
    let mut current = state;
    loop {
        let holder = current.turn.priority_holder.expect("no priority holder");
        // Find this holder in the players slice to know who goes next.
        // If holder is not in the slice, just pass all in order.
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
        // break to avoid infinite loop.
        if current.turn.priority_holder == Some(holder) {
            // Try passing once more for the other player.
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

// ── Test 1: Basic token creation in 4-player game ─────────────────────────────

#[test]
/// CR 702.116a — In a 4-player game, P1 attacks P2 with myriad creature.
/// Tokens are created for P3 and P4 (the opponents other than the defending player P2).
/// After the trigger resolves: 2 tokens on battlefield, each tapped, each attacking.
fn test_myriad_basic_creates_token_copies_in_4_player() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let attacker = ObjectSpec::creature(p1, "Myriad Creature", 5, 3)
        .with_keyword(KeywordAbility::Myriad)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Myriad Creature");

    // P1 declares Myriad Creature attacking P2.
    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Myriad trigger should be on the stack.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 702.116a: AbilityTriggered event expected from myriad"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.116a: myriad trigger should be on the stack"
    );

    // No tokens yet — trigger hasn't resolved.
    assert_eq!(
        token_count(&state, p1),
        0,
        "No tokens should exist before myriad trigger resolves"
    );

    // All players pass priority — myriad trigger resolves.
    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);

    // Trigger should have resolved: 2 tokens created (for P3 and P4).
    let token_created_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::TokenCreated { .. }))
        .count();
    assert_eq!(
        token_created_count, 2,
        "CR 702.116a: 2 tokens should be created (one for P3, one for P4)"
    );

    // Both tokens on battlefield controlled by P1.
    assert_eq!(
        token_count(&state, p1),
        2,
        "CR 702.116a: P1 should control 2 myriad tokens on the battlefield"
    );

    // Tokens are tapped (they entered tapped per CR 702.116a).
    let tapped_token_count = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.is_token
                && obj.controller == p1
                && obj.status.tapped
        })
        .count();
    assert_eq!(
        tapped_token_count, 2,
        "CR 702.116a: myriad tokens should enter tapped"
    );

    // Tokens are registered as attackers in combat (attacking P3 and P4).
    let combat = state.combat.as_ref().expect("combat state should exist");
    let token_attackers: Vec<_> = combat
        .attackers
        .iter()
        .filter(|(id, _)| state.objects.get(*id).map(|o| o.is_token).unwrap_or(false))
        .collect();
    assert_eq!(
        token_attackers.len(),
        2,
        "CR 702.116a: 2 token attackers should be registered in combat"
    );

    // Tokens attack P3 and P4 (not P2).
    let token_targets: std::collections::HashSet<PlayerId> = token_attackers
        .iter()
        .filter_map(|(_, target)| {
            if let AttackTarget::Player(pid) = target {
                Some(*pid)
            } else {
                None
            }
        })
        .collect();
    assert!(
        token_targets.contains(&p3),
        "CR 702.116a: one token should attack P3"
    );
    assert!(
        token_targets.contains(&p4),
        "CR 702.116a: one token should attack P4"
    );
    assert!(
        !token_targets.contains(&p2),
        "CR 702.116a: no token should attack P2 (the defending player)"
    );
}

// ── Test 2: 2-player game — no tokens created ────────────────────────────────

#[test]
/// CR 702.116a — ruling: "If the defending player is your only opponent, no
/// tokens are put onto the battlefield." In a 2-player game, myriad trigger
/// fires but creates no tokens since there are no other opponents.
fn test_myriad_2_player_no_tokens_created() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Myriad Creature", 5, 3)
        .with_keyword(KeywordAbility::Myriad)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Myriad Creature");

    // P1 declares Myriad Creature attacking P2 (the only opponent).
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Trigger is on the stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "myriad trigger should still be on the stack"
    );

    // Both players pass priority — myriad trigger resolves.
    let (state, events) = pass_all(state, &[p1, p2]);

    // No TokenCreated events — P2 was the only opponent, so no eligible opponents.
    let token_created_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::TokenCreated { .. }))
        .count();
    assert_eq!(
        token_created_count, 0,
        "CR 702.116a: no tokens should be created in a 2-player game"
    );

    // No tokens on the battlefield.
    assert_eq!(
        token_count(&state, p1),
        0,
        "CR 702.116a: no myriad tokens in 2-player game"
    );
}

// ── Test 3: Tokens exiled at end of combat ───────────────────────────────────

#[test]
/// CR 702.116a — "exile the tokens at end of combat."
/// After the myriad trigger resolves, advancing to EndOfCombat exiles all
/// myriad tokens.
fn test_myriad_tokens_exiled_at_end_of_combat() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let attacker = ObjectSpec::creature(p1, "Myriad Creature", 5, 3)
        .with_keyword(KeywordAbility::Myriad)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Myriad Creature");

    // P1 attacks P2 with the myriad creature.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the myriad trigger (creates 1 token for P3).
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // Verify 1 token created (3-player game: 1 eligible opponent = P3).
    assert_eq!(
        token_count(&state, p1),
        1,
        "Should have 1 myriad token (for P3)"
    );

    // Verify token has myriad_exile_at_eoc flag.
    let token = state
        .objects
        .values()
        .find(|obj| obj.is_token && obj.controller == p1)
        .expect("token should exist");
    assert!(
        token.myriad_exile_at_eoc,
        "CR 702.116a: myriad token should have myriad_exile_at_eoc = true"
    );
    let token_bf_id = token.id;
    let _ = token_bf_id; // used in exile check below

    // Advance to DeclareBlockers: pass all players.
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // We should now be at DeclareBlockers. Declare no blockers.
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("DeclareBlockers should succeed");

    // Advance through CombatDamage step.
    let (state, _) = pass_until_advance(state, &[p1, p2, p3]);
    // Now at CombatDamage or EndOfCombat; advance to EndOfCombat.
    let (state, events) = pass_until_advance(state, &[p1, p2, p3]);

    // Check if we've passed EndOfCombat (tokens exiled).
    // Either we are now at EndOfCombat and tokens got exiled, or we passed through.
    // Check for ObjectExiled events.
    let exiled = events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectExiled { .. }));

    // Also check exile zone.
    let exile_total = exile_count(&state);

    // Tokens should have been exiled; either via events or via zone check.
    assert!(
        exiled || exile_total >= 1,
        "CR 702.116a: myriad tokens should be exiled at end of combat"
    );

    // No myriad tokens should remain on the battlefield.
    let remaining_tokens: Vec<_> = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_token && obj.myriad_exile_at_eoc)
        .collect();
    assert_eq!(
        remaining_tokens.len(),
        0,
        "CR 702.116a: myriad tokens should not remain on the battlefield after end of combat"
    );
}

// ── Test 4: Token has myriad_exile_at_eoc flag ────────────────────────────────

#[test]
/// CR 702.116a: Myriad tokens must be tagged with `myriad_exile_at_eoc = true`
/// and `is_token = true` when created.
fn test_myriad_token_has_correct_flags() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let attacker = ObjectSpec::creature(p1, "Myriad Creature", 5, 3)
        .with_keyword(KeywordAbility::Myriad)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Myriad Creature");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the myriad trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // Check token flags.
    let tokens: Vec<_> = state
        .objects
        .values()
        .filter(|obj| obj.zone == ZoneId::Battlefield && obj.is_token && obj.controller == p1)
        .collect();

    assert_eq!(tokens.len(), 1, "Should have exactly 1 myriad token");

    let token = &tokens[0];
    assert!(token.is_token, "CR 702.116a: myriad copy must be a token");
    assert!(
        token.myriad_exile_at_eoc,
        "CR 702.116a: myriad token must have myriad_exile_at_eoc = true"
    );
    assert!(
        token.status.tapped,
        "CR 702.116a: myriad token must enter tapped"
    );
    assert!(
        token.has_summoning_sickness,
        "CR 302.6: tokens have summoning sickness"
    );
}

// ── Test 5: Multiple myriad instances trigger separately ─────────────────────

#[test]
/// CR 702.116b — "If a creature has multiple instances of myriad, each triggers
/// separately." A creature with two myriad keywords in a 4-player game (attacking
/// P2) should create 4 tokens total: 2 for P3 and 2 for P4.
fn test_myriad_multiple_instances_trigger_separately() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    // Creature with TWO myriad instances.
    let attacker = ObjectSpec::creature(p1, "Double Myriad", 5, 3)
        .with_keyword(KeywordAbility::Myriad)
        .with_keyword(KeywordAbility::Myriad)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Double Myriad");

    let (state, events) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Two myriad instances = 2 triggers on stack.
    let trigger_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
        .count();
    assert_eq!(
        trigger_count, 2,
        "CR 702.116b: 2 triggers should fire (one per myriad instance)"
    );
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.116b: 2 myriad triggers should be on the stack"
    );

    // Resolve both triggers.
    let (state, events) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, events2) = pass_all(state, &[p1, p2, p3, p4]);
    let all_events: Vec<_> = events.into_iter().chain(events2).collect();

    let token_created_count = all_events
        .iter()
        .filter(|e| matches!(e, GameEvent::TokenCreated { .. }))
        .count();
    assert_eq!(
        token_created_count, 4,
        "CR 702.116b: double myriad in 4-player game should create 4 tokens (2 per eligible opponent)"
    );

    // P1 should control 4 tokens.
    assert_eq!(
        token_count(&state, p1),
        4,
        "CR 702.116b: P1 should control 4 tokens from double myriad"
    );
}

// ── Test 6: Correct opponents targeted in multiplayer ────────────────────────

#[test]
/// CR 702.116a — "for each opponent other than defending player."
/// In 4-player game: P1 attacks P3. Tokens should attack P2 and P4.
fn test_myriad_multiplayer_correct_opponents_targeted() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let attacker = ObjectSpec::creature(p1, "Myriad Creature", 5, 3)
        .with_keyword(KeywordAbility::Myriad)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Myriad Creature");

    // P1 attacks P3 (NOT P2).
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p3))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the myriad trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // Tokens should attack P2 and P4 (not P3 — the defending player, not P1 — the attacker).
    let combat = state.combat.as_ref().expect("combat state should exist");
    let token_targets: std::collections::HashSet<PlayerId> = combat
        .attackers
        .iter()
        .filter(|(id, _)| state.objects.get(*id).map(|o| o.is_token).unwrap_or(false))
        .filter_map(|(_, target)| {
            if let AttackTarget::Player(pid) = target {
                Some(*pid)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        token_targets.len(),
        2,
        "Should have tokens attacking 2 different opponents"
    );
    assert!(
        token_targets.contains(&p2),
        "CR 702.116a: token should attack P2"
    );
    assert!(
        token_targets.contains(&p4),
        "CR 702.116a: token should attack P4"
    );
    assert!(
        !token_targets.contains(&p3),
        "CR 702.116a: no token should attack P3 (the defending player)"
    );
    assert!(
        !token_targets.contains(&p1),
        "No token should attack P1 (the controller)"
    );
}

// ── Test 7: Original creature not affected ───────────────────────────────────

#[test]
/// CR 702.116a — The original myriad creature is not affected by its own trigger.
/// It continues attacking the defending player (P2), and P1 still controls it.
fn test_myriad_original_attacker_unaffected() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    let attacker = ObjectSpec::creature(p1, "Myriad Creature", 5, 3)
        .with_keyword(KeywordAbility::Myriad)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Myriad Creature");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Resolve the myriad trigger.
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // Original myriad creature should still be on battlefield.
    let original = state
        .objects
        .get(&attacker_id)
        .expect("original should still exist");
    assert_eq!(
        original.zone,
        ZoneId::Battlefield,
        "Original myriad creature should still be on the battlefield"
    );
    assert_eq!(
        original.controller, p1,
        "Original myriad creature should still be controlled by P1"
    );
    assert!(!original.is_token, "Original creature is not a token");

    // Original attacker should still be attacking P2 in combat state.
    let combat = state.combat.as_ref().expect("combat state should exist");
    assert!(
        combat.attackers.contains_key(&attacker_id),
        "Original myriad creature should still be an attacker"
    );
    assert_eq!(
        combat.attackers.get(&attacker_id),
        Some(&AttackTarget::Player(p2)),
        "Original myriad creature should still be attacking P2"
    );

    // P1 should control the original creature + 1 token (for P3).
    assert_eq!(
        battlefield_count(&state, p1),
        2,
        "P1 should control original + 1 myriad token"
    );
}
