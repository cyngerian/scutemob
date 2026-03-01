//! Combat harness tests (CR 508.1, CR 509.1).
//!
//! Verifies that the `translate_player_action` function in the script replay harness
//! correctly handles `"declare_attackers"` and `"declare_blockers"` action strings,
//! building valid `Command::DeclareAttackers` and `Command::DeclareBlockers` from
//! structured script declarations.
//!
//! These tests use the programmatic `GameScript` struct interface (not JSON files)
//! and call `replay_script()` to verify end-to-end harness behaviour.

use std::collections::HashMap;

use mtg_engine::testing::replay_harness::build_initial_state;
use mtg_engine::testing::script_schema::{
    AttackerDeclaration, BlockerDeclaration, Confidence, GameScript, InitialState,
    PermanentInitState, PlayerInitState, ReviewStatus, ScriptAction, ScriptMetadata, ScriptStep,
    ZonesInitState,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a minimal two-player `PlayerInitState` with 40 life.
fn player_init() -> PlayerInitState {
    PlayerInitState {
        life: 40,
        mana_pool: HashMap::new(),
        land_plays_remaining: 0,
        poison_counters: 0,
        commander_damage_received: HashMap::new(),
        commander: None,
        partner_commander: None,
    }
}

/// Build a minimal `ScriptMetadata`.
fn meta(id: &str, name: &str) -> ScriptMetadata {
    ScriptMetadata {
        id: id.to_string(),
        name: name.to_string(),
        description: name.to_string(),
        cr_sections_tested: vec!["508.1".to_string(), "509.1".to_string()],
        corner_case_ref: None,
        tags: vec!["combat".to_string()],
        confidence: Confidence::High,
        review_status: ReviewStatus::Approved,
        reviewed_by: None,
        review_date: None,
        generation_notes: None,
        disputes: vec![],
    }
}

/// Build a two-player initial state at the DeclareAttackers step.
///
/// `p1_creatures`: names and (power, toughness) for creatures under p1's control.
/// `p2_creatures`: names and (power, toughness) for creatures under p2's control.
///
/// Uses real card names so `enrich_spec_from_def` can populate types/P&T.
/// Llanowar Elves (1/1) and Elvish Mystic (1/1) are used as simple creatures.
fn make_initial_state(p1_permanents: Vec<&str>, p2_permanents: Vec<&str>) -> InitialState {
    let mut players = HashMap::new();
    players.insert("p1".to_string(), player_init());
    players.insert("p2".to_string(), player_init());

    let mut battlefield = HashMap::new();
    battlefield.insert(
        "p1".to_string(),
        p1_permanents
            .into_iter()
            .map(|name| PermanentInitState {
                card: name.to_string(),
                tapped: false,
                summoning_sick: false,
                counters: HashMap::new(),
                attached: vec![],
                damage_marked: 0,
                is_commander: false,
                subtypes: None,
                is_basic: None,
            })
            .collect(),
    );
    battlefield.insert(
        "p2".to_string(),
        p2_permanents
            .into_iter()
            .map(|name| PermanentInitState {
                card: name.to_string(),
                tapped: false,
                summoning_sick: false,
                counters: HashMap::new(),
                attached: vec![],
                damage_marked: 0,
                is_commander: false,
                subtypes: None,
                is_basic: None,
            })
            .collect(),
    );

    InitialState {
        format: "commander".to_string(),
        // turn_number >= 2 ensures no summoning sickness.
        turn_number: 5,
        active_player: "p1".to_string(),
        phase: "declare_attackers".to_string(),
        step: None,
        priority: "p1".to_string(),
        players,
        zones: ZonesInitState {
            battlefield,
            hand: HashMap::new(),
            graveyard: HashMap::new(),
            exile: vec![],
            command_zone: HashMap::new(),
            library: HashMap::new(),
            stack: vec![],
        },
        continuous_effects: vec![],
    }
}

// Re-export replay_script from the script_replay integration test.
// We cannot directly call replay_script (it lives in tests/script_replay.rs) so we
// replicate the same logic inline here using the public harness API.
//
// Instead, we call the harness functions directly: build_initial_state +
// translate_player_action + process_command.

use mtg_engine::testing::replay_harness::translate_player_action;
use mtg_engine::{process_command, AttackTarget, Command, PlayerId, Step};

/// CR 508.1 — Declare attackers harness action: resolve creature name to ObjectId
/// and build Command::DeclareAttackers. Verify the attacker is tapped after.
#[test]
fn test_harness_declare_attackers_basic() {
    // CR 508.1: p1 declares Llanowar Elves (1/1) attacking p2.
    // After declaring and passing through to combat damage, p2 should lose 1 life.
    let init = make_initial_state(vec!["Llanowar Elves"], vec![]);
    let (state, players) = build_initial_state(&init);

    let p1 = players["p1"];
    let p2 = players["p2"];

    // translate_player_action should resolve the attacker name and build the command.
    let decl = vec![AttackerDeclaration {
        card: "Llanowar Elves".to_string(),
        target_player: Some("p2".to_string()),
        target_planeswalker: None,
    }];

    let cmd = translate_player_action(
        "declare_attackers",
        p1,
        None,
        0,
        &[],
        &decl,
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false,
        &[],
        None,
        None,
        &state,
        &players,
    );

    assert!(
        cmd.is_some(),
        "translate_player_action should return Some(Command) for declare_attackers"
    );
    let cmd = cmd.unwrap();

    // Verify it's the right variant and contains the correct attacker/target.
    match &cmd {
        Command::DeclareAttackers {
            player, attackers, ..
        } => {
            assert_eq!(*player, p1, "command player should be p1");
            assert_eq!(attackers.len(), 1, "one attacker declared");
            assert!(
                matches!(attackers[0].1, AttackTarget::Player(pid) if pid == p2),
                "attacker should target p2"
            );
        }
        _ => panic!("Expected DeclareAttackers, got {:?}", cmd),
    }

    // Execute the command and verify the creature is tapped (CR 508.1f).
    let (state_after, _) = process_command(state, cmd).expect("DeclareAttackers should succeed");

    // Find the Llanowar Elves object on the battlefield.
    let elf = state_after
        .objects
        .values()
        .find(|o| o.characteristics.name == "Llanowar Elves")
        .expect("Llanowar Elves should still be on battlefield");
    assert!(
        elf.status.tapped,
        "CR 508.1f: attacker should be tapped after declaring"
    );
}

/// CR 508.1: Declaring zero attackers is legal. Combat advances without damage.
#[test]
fn test_harness_declare_attackers_empty() {
    // CR 508.1: The active player may choose to attack with no creatures.
    // An empty attackers list is valid and the step should advance normally.
    let init = make_initial_state(vec!["Llanowar Elves"], vec![]);
    let (state, players) = build_initial_state(&init);

    let p1 = players["p1"];
    let p2 = players["p2"];

    let cmd = translate_player_action(
        "declare_attackers",
        p1,
        None,
        0,
        &[],
        &[], // empty attackers
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false,
        &[],
        None,
        None,
        &state,
        &players,
    );

    // Even with no attackers, translate should return a DeclareAttackers command
    // (empty attackers list).
    assert!(
        cmd.is_some(),
        "translate_player_action with empty attackers should return Some(Command)"
    );

    match cmd.unwrap() {
        Command::DeclareAttackers { attackers, .. } => {
            assert!(
                attackers.is_empty(),
                "empty attackers vec should be passed through"
            );
        }
        other => panic!("Expected DeclareAttackers, got {:?}", other),
    }

    // Execute the empty declare — p2 should remain at 40 life.
    let cmd_empty = Command::DeclareAttackers {
        player: p1,
        attackers: vec![],
        enlist_choices: vec![],
    };
    let (state_after, _) =
        process_command(state, cmd_empty).expect("Empty DeclareAttackers should succeed");

    let p2_life = state_after
        .players
        .get(&p2)
        .map(|ps| ps.life_total)
        .unwrap_or(0);
    assert_eq!(
        p2_life, 40,
        "p2 should remain at 40 life when no attackers are declared"
    );
}

/// CR 509.1: Declare blockers harness action: resolve blocker and attacker names to
/// ObjectIds and build Command::DeclareBlockers.
#[test]
fn test_harness_declare_blockers_basic() {
    // CR 509.1: p2 declares Elvish Mystic as blocking Llanowar Elves.
    // Build state: p1 has Llanowar Elves (1/1), p2 has Elvish Mystic (1/1).
    let init = make_initial_state(vec!["Llanowar Elves"], vec!["Elvish Mystic"]);
    let (state, players) = build_initial_state(&init);

    let p1 = players["p1"];
    let p2 = players["p2"];

    // First declare the attacker so the engine is in DeclareBlockers step.
    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Llanowar Elves" && o.controller == p1)
        .expect("Llanowar Elves should be on battlefield")
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Both players pass priority to advance to DeclareBlockers.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 })
        .expect("p1 pass should succeed");
    let (state, _) = process_command(state, Command::PassPriority { player: p2 })
        .expect("p2 pass should succeed");

    assert_eq!(
        state.turn.step,
        Step::DeclareBlockers,
        "should be in DeclareBlockers step"
    );

    // Now test translate_player_action for declare_blockers.
    let decl = vec![BlockerDeclaration {
        card: "Elvish Mystic".to_string(),
        blocking: "Llanowar Elves".to_string(),
    }];

    let cmd = translate_player_action(
        "declare_blockers",
        p2,
        None,
        0,
        &[],
        &[],
        &decl,
        &[],
        &[],
        &[],
        &[],
        false,
        false,
        &[],
        None,
        None,
        &state,
        &players,
    );

    assert!(
        cmd.is_some(),
        "translate_player_action should return Some(Command) for declare_blockers"
    );

    match cmd.unwrap() {
        Command::DeclareBlockers { player, blockers } => {
            assert_eq!(player, p2, "command player should be p2");
            assert_eq!(blockers.len(), 1, "one blocker declared");
        }
        other => panic!("Expected DeclareBlockers, got {:?}", other),
    }
}

/// CR 509.1: Declaring zero blockers is legal. Unblocked attacker deals damage.
#[test]
fn test_harness_declare_blockers_empty() {
    // CR 509.1: p2 may declare no blockers. An empty blockers list is valid.
    // The Llanowar Elves (1/1) goes unblocked and deals 1 damage to p2.
    let init = make_initial_state(vec!["Llanowar Elves"], vec![]);
    let (state, players) = build_initial_state(&init);

    let p1 = players["p1"];
    let p2 = players["p2"];

    // Declare attacker.
    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Llanowar Elves" && o.controller == p1)
        .expect("Llanowar Elves should be on battlefield")
        .id;

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Pass priority to advance to DeclareBlockers.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();

    assert_eq!(
        state.turn.step,
        Step::DeclareBlockers,
        "should be in DeclareBlockers"
    );

    // Test translate_player_action with empty blockers.
    let cmd = translate_player_action(
        "declare_blockers",
        p2,
        None,
        0,
        &[],
        &[],
        &[], // empty blockers
        &[],
        &[],
        &[],
        &[],
        false,
        false,
        &[],
        None,
        None,
        &state,
        &players,
    );

    assert!(
        cmd.is_some(),
        "translate_player_action with empty blockers should return Some(Command)"
    );

    let (state, _) = process_command(state, cmd.unwrap()).expect("DeclareBlockers should succeed");

    // Pass priority to advance to CombatDamage (both pass).
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _events) = process_command(state, Command::PassPriority { player: p2 }).unwrap();

    // After combat damage, p2 should have lost 1 life (Llanowar Elves is 1/1).
    let p2_life = state.players.get(&p2).map(|ps| ps.life_total).unwrap_or(40);
    assert_eq!(
        p2_life, 39,
        "p2 should have 39 life after taking 1 damage from unblocked 1/1"
    );
}

/// CR 508.1, 509.1, 510.1: Full combat cycle via harness — declare attackers,
/// declare blockers (none), advance to damage step, verify life totals.
#[test]
fn test_harness_full_combat_unblocked_damage() {
    // CR 510.1a: An unblocked attacker deals damage equal to its power to the
    // player it's attacking. Llanowar Elves (1/1) deals 1 damage to p2.
    let init = make_initial_state(vec!["Llanowar Elves"], vec![]);
    let (state, players) = build_initial_state(&init);

    let p1 = players["p1"];
    let p2 = players["p2"];

    // Step 1: Declare attacker via harness.
    let decl_atk = vec![AttackerDeclaration {
        card: "Llanowar Elves".to_string(),
        target_player: Some("p2".to_string()),
        target_planeswalker: None,
    }];
    let cmd_atk = translate_player_action(
        "declare_attackers",
        p1,
        None,
        0,
        &[],
        &decl_atk,
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false,
        &[],
        None,
        None,
        &state,
        &players,
    )
    .expect("declare_attackers should translate");

    let (state, _) = process_command(state, cmd_atk).expect("DeclareAttackers should succeed");

    // Step 2: Both players pass priority (advance to DeclareBlockers).
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // Step 3: Declare no blockers via harness.
    let cmd_blk = translate_player_action(
        "declare_blockers",
        p2,
        None,
        0,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false,
        &[],
        None,
        None,
        &state,
        &players,
    )
    .expect("declare_blockers should translate");

    let (state, _) = process_command(state, cmd_blk).expect("DeclareBlockers should succeed");

    // Step 4: Both players pass to advance through CombatDamage.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();

    // CR 510.1a: 1/1 unblocked deals 1 damage → p2 at 39 life.
    let p2_life = state.players.get(&p2).map(|ps| ps.life_total).unwrap_or(40);
    assert_eq!(
        p2_life, 39,
        "p2 should have 39 life after unblocked 1/1 deals damage"
    );
}

/// CR 508.1b: When no target_player or target_planeswalker is specified, the
/// harness defaults to attacking the first non-active player.
#[test]
fn test_harness_declare_attackers_default_target() {
    // CR 508.1b: If no target is specified, the harness should choose the opponent.
    let init = make_initial_state(vec!["Llanowar Elves"], vec![]);
    let (state, players) = build_initial_state(&init);

    let p1 = players["p1"];
    let p2 = players["p2"];

    // Declare with no target specified.
    let decl = vec![AttackerDeclaration {
        card: "Llanowar Elves".to_string(),
        target_player: None,
        target_planeswalker: None,
    }];

    let cmd = translate_player_action(
        "declare_attackers",
        p1,
        None,
        0,
        &[],
        &decl,
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false,
        &[],
        None,
        None,
        &state,
        &players,
    );

    assert!(
        cmd.is_some(),
        "translate_player_action with default target should return Some(Command)"
    );

    match cmd.unwrap() {
        Command::DeclareAttackers { attackers, .. } => {
            assert_eq!(attackers.len(), 1, "one attacker should be declared");
            // The default target should be the non-active player (p2).
            assert!(
                matches!(attackers[0].1, AttackTarget::Player(pid) if pid == p2),
                "default target should be p2 (the non-active player)"
            );
        }
        other => panic!("Expected DeclareAttackers, got {:?}", other),
    }
}
