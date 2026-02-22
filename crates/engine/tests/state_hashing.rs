//! Deterministic state hashing tests.
//!
//! Validates that `public_state_hash` and `private_state_hash` are deterministic,
//! sensitive to state changes, correctly partition public/private information,
//! and remain consistent across dual-instance command processing.

use mtg_engine::rules::engine::process_command;
use mtg_engine::{
    CardType, Color, Command, CounterType, GameStateBuilder, ObjectSpec, PlayerId, Step, ZoneId,
};

// ============================================================
// Determinism tests
// ============================================================

#[test]
/// Two identically-built states must produce identical public hashes.
fn test_hash_determinism_identical_states() {
    let state1 = GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .build().unwrap();

    assert_eq!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// Private hashes for the same player on identical states must match.
fn test_hash_determinism_private_identical_states() {
    let state1 = GameStateBuilder::four_player().build().unwrap();
    let state2 = GameStateBuilder::four_player().build().unwrap();

    for pid in 1..=4u64 {
        assert_eq!(
            state1.private_state_hash(PlayerId(pid)),
            state2.private_state_hash(PlayerId(pid)),
        );
    }
}

#[test]
/// Calling hash multiple times on the same state produces the same result.
fn test_hash_determinism_repeated_calls() {
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 2, 2))
        .build().unwrap();

    let h1 = state.public_state_hash();
    let h2 = state.public_state_hash();
    let h3 = state.public_state_hash();
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
}

// ============================================================
// Sensitivity tests — different states produce different hashes
// ============================================================

#[test]
/// Different life totals produce different public hashes.
fn test_hash_sensitivity_life_total() {
    let state1 = GameStateBuilder::four_player().build().unwrap();
    let state2 = GameStateBuilder::four_player()
        .player_life(PlayerId(1), 39)
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// Different active players produce different public hashes.
fn test_hash_sensitivity_active_player() {
    let state1 = GameStateBuilder::four_player()
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .active_player(PlayerId(2))
        .at_step(Step::PreCombatMain)
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// Different steps produce different public hashes.
fn test_hash_sensitivity_step() {
    let state1 = GameStateBuilder::four_player()
        .at_step(Step::PreCombatMain)
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .at_step(Step::Upkeep)
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// An object on the battlefield changes the public hash.
fn test_hash_sensitivity_battlefield_object() {
    let state1 = GameStateBuilder::four_player().build().unwrap();
    let state2 = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 2, 2))
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// Different creature stats produce different public hashes.
fn test_hash_sensitivity_creature_stats() {
    let state1 = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 2, 2))
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 3, 3))
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// Different poison counters produce different public hashes.
fn test_hash_sensitivity_poison_counters() {
    let state1 = GameStateBuilder::four_player().build().unwrap();
    let state2 = GameStateBuilder::four_player()
        .player_poison(PlayerId(1), 3)
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// A tapped permanent differs from an untapped one.
fn test_hash_sensitivity_tapped_status() {
    let state1 = GameStateBuilder::four_player()
        .object(ObjectSpec::land(PlayerId(1), "Forest"))
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .object(ObjectSpec::land(PlayerId(1), "Forest").tapped())
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// Different counters on objects produce different hashes.
fn test_hash_sensitivity_counters() {
    let state1 = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Hydra", 0, 0))
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(PlayerId(1), "Hydra", 0, 0)
                .with_counter(CounterType::PlusOnePlusOne, 3),
        )
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

#[test]
/// Different colors on objects produce different hashes.
fn test_hash_sensitivity_colors() {
    let state1 = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 2, 2).with_colors(vec![Color::Green]))
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .object(ObjectSpec::creature(PlayerId(1), "Bear", 2, 2).with_colors(vec![Color::Red]))
        .build().unwrap();

    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

// ============================================================
// Public/private partition tests
// ============================================================

#[test]
/// Two states identical except for library order have the same public hash
/// but different private hashes.
fn test_hash_public_excludes_library_order() {
    // State 1: cards in library in one order
    let state1 = GameStateBuilder::four_player()
        .object(
            ObjectSpec::card(PlayerId(1), "Card A")
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Library(PlayerId(1))),
        )
        .object(
            ObjectSpec::card(PlayerId(1), "Card B")
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Library(PlayerId(1))),
        )
        .build().unwrap();

    // State 2: cards in library in reverse order
    let state2 = GameStateBuilder::four_player()
        .object(
            ObjectSpec::card(PlayerId(1), "Card B")
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Library(PlayerId(1))),
        )
        .object(
            ObjectSpec::card(PlayerId(1), "Card A")
                .with_types(vec![CardType::Instant])
                .in_zone(ZoneId::Library(PlayerId(1))),
        )
        .build().unwrap();

    // Both have 2 cards in library → same public hash
    assert_eq!(state1.public_state_hash(), state2.public_state_hash());

    // But private hashes differ because library order differs
    assert_ne!(
        state1.private_state_hash(PlayerId(1)),
        state2.private_state_hash(PlayerId(1)),
    );
}

#[test]
/// Two states identical except for hand contents have the same public hash
/// but different private hashes.
fn test_hash_public_excludes_hand_contents() {
    let state1 = GameStateBuilder::four_player()
        .object(ObjectSpec::card(PlayerId(1), "Lightning Bolt").in_zone(ZoneId::Hand(PlayerId(1))))
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .object(ObjectSpec::card(PlayerId(1), "Counterspell").in_zone(ZoneId::Hand(PlayerId(1))))
        .build().unwrap();

    // Both have 1 card in hand → same public hash
    assert_eq!(state1.public_state_hash(), state2.public_state_hash());

    // Private hashes differ
    assert_ne!(
        state1.private_state_hash(PlayerId(1)),
        state2.private_state_hash(PlayerId(1)),
    );
}

#[test]
/// Private hashes are player-specific: player 1's private hash differs from player 2's.
fn test_hash_private_player_specific() {
    let state = GameStateBuilder::four_player()
        .object(ObjectSpec::card(PlayerId(1), "Card A").in_zone(ZoneId::Hand(PlayerId(1))))
        .object(ObjectSpec::card(PlayerId(2), "Card B").in_zone(ZoneId::Hand(PlayerId(2))))
        .build().unwrap();

    assert_ne!(
        state.private_state_hash(PlayerId(1)),
        state.private_state_hash(PlayerId(2)),
    );
}

#[test]
/// Different hand sizes produce different public hashes (hand size is public).
fn test_hash_public_includes_hand_size() {
    let state1 = GameStateBuilder::four_player()
        .object(ObjectSpec::card(PlayerId(1), "Card A").in_zone(ZoneId::Hand(PlayerId(1))))
        .build().unwrap();

    let state2 = GameStateBuilder::four_player()
        .object(ObjectSpec::card(PlayerId(1), "Card A").in_zone(ZoneId::Hand(PlayerId(1))))
        .object(ObjectSpec::card(PlayerId(1), "Card B").in_zone(ZoneId::Hand(PlayerId(1))))
        .build().unwrap();

    // Different hand sizes → different public hash
    assert_ne!(state1.public_state_hash(), state2.public_state_hash());
}

// ============================================================
// Dual-instance property tests
// ============================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Build two independent game states, start the game on both, then process
    /// the same PassPriority commands. Verify public hashes match after every command.
    #[test]
    fn test_dual_instance_public_hash_pass_priority() {
        // Build identical initial states independently
        let state1 = GameStateBuilder::four_player()
            .at_step(Step::Upkeep)
            .first_turn_of_game()
            .object(ObjectSpec::card(PlayerId(1), "Card A").in_zone(ZoneId::Library(PlayerId(1))))
            .object(ObjectSpec::card(PlayerId(2), "Card B").in_zone(ZoneId::Library(PlayerId(2))))
            .object(ObjectSpec::card(PlayerId(3), "Card C").in_zone(ZoneId::Library(PlayerId(3))))
            .object(ObjectSpec::card(PlayerId(4), "Card D").in_zone(ZoneId::Library(PlayerId(4))))
            .build().unwrap();

        let state2 = GameStateBuilder::four_player()
            .at_step(Step::Upkeep)
            .first_turn_of_game()
            .object(ObjectSpec::card(PlayerId(1), "Card A").in_zone(ZoneId::Library(PlayerId(1))))
            .object(ObjectSpec::card(PlayerId(2), "Card B").in_zone(ZoneId::Library(PlayerId(2))))
            .object(ObjectSpec::card(PlayerId(3), "Card C").in_zone(ZoneId::Library(PlayerId(3))))
            .object(ObjectSpec::card(PlayerId(4), "Card D").in_zone(ZoneId::Library(PlayerId(4))))
            .build().unwrap();

        // Verify initial hashes match
        assert_eq!(
            state1.public_state_hash(),
            state2.public_state_hash(),
            "initial states must have identical public hashes"
        );

        // Process PassPriority commands through multiple rounds
        // In Upkeep with 4 players: needs 4 passes to advance
        let mut s1 = state1;
        let mut s2 = state2;

        // Pass priority for all 4 players (advances to Draw step)
        for pid in 1..=4u64 {
            let cmd = Command::PassPriority {
                player: PlayerId(pid),
            };
            let (new1, _) = process_command(s1, cmd.clone()).unwrap();
            let (new2, _) = process_command(s2, cmd).unwrap();
            s1 = new1;
            s2 = new2;

            assert_eq!(
                s1.public_state_hash(),
                s2.public_state_hash(),
                "public hash mismatch after PassPriority from player {}",
                pid
            );
        }
    }

    /// Same dual-instance test but also verifying private hashes match.
    #[test]
    fn test_dual_instance_private_hash_pass_priority() {
        let state1 = GameStateBuilder::four_player()
            .at_step(Step::Upkeep)
            .first_turn_of_game()
            .object(ObjectSpec::card(PlayerId(1), "Card A").in_zone(ZoneId::Library(PlayerId(1))))
            .build().unwrap();

        let state2 = GameStateBuilder::four_player()
            .at_step(Step::Upkeep)
            .first_turn_of_game()
            .object(ObjectSpec::card(PlayerId(1), "Card A").in_zone(ZoneId::Library(PlayerId(1))))
            .build().unwrap();

        let mut s1 = state1;
        let mut s2 = state2;

        for pid in 1..=4u64 {
            let cmd = Command::PassPriority {
                player: PlayerId(pid),
            };
            let (new1, _) = process_command(s1, cmd.clone()).unwrap();
            let (new2, _) = process_command(s2, cmd).unwrap();
            s1 = new1;
            s2 = new2;

            // Check private hash for all players
            for check_pid in 1..=4u64 {
                assert_eq!(
                    s1.private_state_hash(PlayerId(check_pid)),
                    s2.private_state_hash(PlayerId(check_pid)),
                    "private hash mismatch for player {} after PassPriority from player {}",
                    check_pid,
                    pid
                );
            }
        }
    }

    proptest! {
        /// Proptest: for any number of PassPriority rounds (1-12), two independent
        /// engines processing the same commands produce identical public hashes.
        #[test]
        fn test_proptest_dual_instance_determinism(num_passes in 1..12usize) {
            let state1 = GameStateBuilder::four_player()
                .at_step(Step::Upkeep)
                .first_turn_of_game()
                .object(
                    ObjectSpec::card(PlayerId(1), "Card A")
                        .in_zone(ZoneId::Library(PlayerId(1))),
                )
                .object(
                    ObjectSpec::card(PlayerId(2), "Card B")
                        .in_zone(ZoneId::Library(PlayerId(2))),
                )
                .object(
                    ObjectSpec::card(PlayerId(3), "Card C")
                        .in_zone(ZoneId::Library(PlayerId(3))),
                )
                .object(
                    ObjectSpec::card(PlayerId(4), "Card D")
                        .in_zone(ZoneId::Library(PlayerId(4))),
                )
                .build().unwrap();

            let state2 = GameStateBuilder::four_player()
                .at_step(Step::Upkeep)
                .first_turn_of_game()
                .object(
                    ObjectSpec::card(PlayerId(1), "Card A")
                        .in_zone(ZoneId::Library(PlayerId(1))),
                )
                .object(
                    ObjectSpec::card(PlayerId(2), "Card B")
                        .in_zone(ZoneId::Library(PlayerId(2))),
                )
                .object(
                    ObjectSpec::card(PlayerId(3), "Card C")
                        .in_zone(ZoneId::Library(PlayerId(3))),
                )
                .object(
                    ObjectSpec::card(PlayerId(4), "Card D")
                        .in_zone(ZoneId::Library(PlayerId(4))),
                )
                .build().unwrap();

            let mut s1 = state1;
            let mut s2 = state2;

            for i in 0..num_passes {
                // Determine who has priority — must be the same for both
                let holder1 = s1.turn.priority_holder;
                let holder2 = s2.turn.priority_holder;
                prop_assert_eq!(holder1, holder2, "priority holders diverged at pass {}", i);

                let player = match holder1 {
                    Some(p) => p,
                    None => break, // game over or no priority
                };

                let cmd = Command::PassPriority { player };
                let r1 = process_command(s1.clone(), cmd.clone());
                let r2 = process_command(s2.clone(), cmd);

                match (r1, r2) {
                    (Ok((new1, _)), Ok((new2, _))) => {
                        prop_assert_eq!(
                            new1.public_state_hash(),
                            new2.public_state_hash(),
                            "hash diverged at pass {}",
                            i
                        );
                        s1 = new1;
                        s2 = new2;
                    }
                    (Err(e1), Err(e2)) => {
                        // Both errored — that's fine, they agree
                        prop_assert_eq!(
                            format!("{:?}", e1),
                            format!("{:?}", e2),
                            "different errors at pass {}",
                            i
                        );
                        break;
                    }
                    _ => {
                        prop_assert!(false, "one instance errored and the other didn't at pass {}", i);
                    }
                }
            }
        }
    }
}
