/// Tests for coin flip (CR 705) and dice roll (CR 706) effects.
///
/// These effects use deterministic RNG seeded from the game's timestamp counter
/// for reproducible replays. Each flip/roll advances the timestamp counter by 1.
use mtg_engine::cards::card_definition::EffectTarget;
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::events::GameEvent;
use mtg_engine::state::game_object::ObjectId;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{GameStateBuilder, PlayerId};
use mtg_engine::{Effect, EffectAmount, PlayerTarget};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn setup_basic_state() -> mtg_engine::GameState {
    GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap()
}

/// Create a game state with N cards in player 1's library.
fn setup_state_with_library(n: usize) -> mtg_engine::GameState {
    let mut builder = GameStateBuilder::new().add_player(p(1)).add_player(p(2));
    for i in 0..n {
        builder = builder.object(
            mtg_engine::state::ObjectSpec::card(p(1), &format!("Card {}", i))
                .in_zone(ZoneId::Library(p(1))),
        );
    }
    builder.build().unwrap()
}

// ── Coin Flip Tests ──────────────────────────────────────────────────────────

#[test]
/// CR 705.1 — coin flip executes on_win branch when result is heads (win).
/// Deterministic: odd timestamp → heads (win).
fn test_coin_flip_win_branch() {
    let mut state = setup_basic_state();
    // Set timestamp to an odd number so result = win (odd % 2 == 1 → true).
    state.timestamp_counter = 101;
    let source = ObjectId(1);

    let effect = Effect::CoinFlip {
        on_win: Box::new(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(5),
        }),
        on_lose: Box::new(Effect::DealDamage {
            target: EffectTarget::Controller,
            amount: EffectAmount::Fixed(3),
        }),
    };

    let mut ctx = EffectContext::new(p(1), source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Should emit CoinFlipped with result=true (heads/win).
    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::CoinFlipped { player, result } if *player == p(1) && *result)
        ),
        "Expected CoinFlipped event with result=true"
    );

    // Player should have gained 5 life (win branch), not lost 3.
    let life = state.players.get(&p(1)).unwrap().life_total;
    assert_eq!(
        life, 45,
        "Player should have gained 5 life from winning the flip (40 + 5)"
    );
}

#[test]
/// CR 705.1 — coin flip executes on_lose branch when result is tails (lose).
/// Deterministic: even timestamp → tails (lose).
fn test_coin_flip_lose_branch() {
    let mut state = setup_basic_state();
    // Set timestamp to an even number so result = lose (even % 2 == 0 → false).
    state.timestamp_counter = 100;
    let source = ObjectId(1);

    let effect = Effect::CoinFlip {
        on_win: Box::new(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(5),
        }),
        on_lose: Box::new(Effect::DealDamage {
            target: EffectTarget::Controller,
            amount: EffectAmount::Fixed(3),
        }),
    };

    let mut ctx = EffectContext::new(p(1), source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Should emit CoinFlipped with result=false (tails/lose).
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::CoinFlipped { player, result } if *player == p(1) && !*result)),
        "Expected CoinFlipped event with result=false"
    );

    // Player should have lost 3 life (lose branch).
    let life = state.players.get(&p(1)).unwrap().life_total;
    assert_eq!(
        life, 37,
        "Player should have taken 3 damage from losing the flip (40 - 3)"
    );
}

#[test]
/// CR 705.1 — CoinFlipped event is always emitted, even if the branch is Nothing.
fn test_coin_flip_event_always_emitted() {
    let mut state = setup_basic_state();
    state.timestamp_counter = 101; // odd = win
    let source = ObjectId(1);

    let effect = Effect::CoinFlip {
        on_win: Box::new(Effect::Nothing),
        on_lose: Box::new(Effect::Nothing),
    };

    let mut ctx = EffectContext::new(p(1), source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CoinFlipped { .. })),
        "CoinFlipped event should be emitted even when branches are Nothing"
    );
}

#[test]
/// CR 705.1 — coin flip is deterministic: same timestamp produces same result.
/// This ensures replay fidelity.
fn test_coin_flip_deterministic_replay() {
    let source = ObjectId(1);
    let effect = Effect::CoinFlip {
        on_win: Box::new(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(1),
        }),
        on_lose: Box::new(Effect::LoseLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(1),
        }),
    };

    // Run with same initial timestamp twice — should get same result.
    let mut state1 = setup_basic_state();
    state1.timestamp_counter = 42;
    let mut ctx1 = EffectContext::new(p(1), source, vec![]);
    let events1 = execute_effect(&mut state1, &effect, &mut ctx1);

    let mut state2 = setup_basic_state();
    state2.timestamp_counter = 42;
    let mut ctx2 = EffectContext::new(p(1), source, vec![]);
    let events2 = execute_effect(&mut state2, &effect, &mut ctx2);

    let result1 = events1.iter().find_map(|e| match e {
        GameEvent::CoinFlipped { result, .. } => Some(*result),
        _ => None,
    });
    let result2 = events2.iter().find_map(|e| match e {
        GameEvent::CoinFlipped { result, .. } => Some(*result),
        _ => None,
    });

    assert_eq!(
        result1, result2,
        "Same timestamp should produce same coin flip result"
    );
    assert_eq!(
        state1.players.get(&p(1)).unwrap().life_total,
        state2.players.get(&p(1)).unwrap().life_total,
        "Deterministic replay should produce identical life totals"
    );
}

// ── Dice Roll Tests ──────────────────────────────────────────────────────────

#[test]
/// CR 706.2 — d20 roll produces a result in 1..=20 and matches the correct range.
fn test_dice_roll_d20_high_roll() {
    let mut state = setup_basic_state();
    // timestamp 19 → (19 % 20) + 1 = 20
    state.timestamp_counter = 19;
    let source = ObjectId(1);

    let effect = Effect::RollDice {
        sides: 20,
        results: vec![
            (
                1,
                9,
                Effect::DealDamage {
                    target: EffectTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
            ),
            (
                10,
                20,
                Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                },
            ),
        ],
    };

    let mut ctx = EffectContext::new(p(1), source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Result should be 20 (high roll), matching the 10-20 range.
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::DiceRolled { player, sides, result } if *player == p(1) && *sides == 20 && *result == 20)),
        "Expected DiceRolled with result=20"
    );

    let life = state.players.get(&p(1)).unwrap().life_total;
    assert_eq!(
        life, 45,
        "Player should have gained 5 life from high roll (40 + 5)"
    );
}

#[test]
/// CR 706.2 — d20 roll low result matches the low range.
fn test_dice_roll_d20_low_roll() {
    let mut state = setup_basic_state();
    // timestamp 4 → (4 % 20) + 1 = 5
    state.timestamp_counter = 4;
    let source = ObjectId(1);

    let effect = Effect::RollDice {
        sides: 20,
        results: vec![
            (
                1,
                9,
                Effect::DealDamage {
                    target: EffectTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
            ),
            (
                10,
                20,
                Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                },
            ),
        ],
    };

    let mut ctx = EffectContext::new(p(1), source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DiceRolled { result, .. } if *result == 5)),
        "Expected DiceRolled with result=5"
    );

    let life = state.players.get(&p(1)).unwrap().life_total;
    assert_eq!(
        life, 38,
        "Player should have taken 2 damage from low roll (40 - 2)"
    );
}

#[test]
/// CR 706.2 — DiceRolled event is emitted with correct sides and result.
fn test_dice_roll_event_emitted_d6() {
    let mut state = setup_basic_state();
    state.timestamp_counter = 0; // (0 % 6) + 1 = 1
    let source = ObjectId(1);

    let effect = Effect::RollDice {
        sides: 6,
        results: vec![(1, 6, Effect::Nothing)],
    };

    let mut ctx = EffectContext::new(p(1), source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        events.iter().any(|e| matches!(e, GameEvent::DiceRolled { sides, result, .. } if *sides == 6 && *result == 1)),
        "DiceRolled event should show sides=6, result=1"
    );
}

#[test]
/// CR 706.2 — dice roll is deterministic: same timestamp produces same result.
fn test_dice_roll_deterministic_replay() {
    let source = ObjectId(1);
    let effect = Effect::RollDice {
        sides: 20,
        results: vec![(1, 20, Effect::Nothing)],
    };

    let mut state1 = setup_basic_state();
    state1.timestamp_counter = 77;
    let mut ctx1 = EffectContext::new(p(1), source, vec![]);
    let events1 = execute_effect(&mut state1, &effect, &mut ctx1);

    let mut state2 = setup_basic_state();
    state2.timestamp_counter = 77;
    let mut ctx2 = EffectContext::new(p(1), source, vec![]);
    let events2 = execute_effect(&mut state2, &effect, &mut ctx2);

    let result1 = events1.iter().find_map(|e| match e {
        GameEvent::DiceRolled { result, .. } => Some(*result),
        _ => None,
    });
    let result2 = events2.iter().find_map(|e| match e {
        GameEvent::DiceRolled { result, .. } => Some(*result),
        _ => None,
    });

    assert_eq!(
        result1, result2,
        "Same timestamp should produce same dice roll result"
    );
}

// ── LastDiceRoll EffectAmount Tests ──────────────────────────────────────────

#[test]
/// CR 706.2 — EffectAmount::LastDiceRoll uses the result of the most recent roll.
/// Ancient Silver Dragon pattern: roll d20, draw cards equal to the result.
fn test_last_dice_roll_amount() {
    let mut state = setup_state_with_library(20);
    // Set timestamp so we get a known small result: (4 % 20) + 1 = 5
    state.timestamp_counter = 4;

    let source = ObjectId(1);
    let effect = Effect::RollDice {
        sides: 20,
        results: vec![(
            1,
            20,
            Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::LastDiceRoll,
            },
        )],
    };

    let mut ctx = EffectContext::new(p(1), source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    // Verify the roll result.
    let roll_result = events
        .iter()
        .find_map(|e| match e {
            GameEvent::DiceRolled { result, .. } => Some(*result),
            _ => None,
        })
        .expect("DiceRolled event should be emitted");

    // Count CardDrawn events — should equal the dice roll result.
    let draw_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { .. }))
        .count();
    assert_eq!(
        draw_count, roll_result as usize,
        "Should have drawn cards equal to dice roll result ({})",
        roll_result,
    );
}

#[test]
/// CR 705/706 — Mana Crypt card definition: upkeep trigger with coin flip.
/// Verify the card builds and has the expected structure.
fn test_mana_crypt_card_def_structure() {
    let card = mtg_engine::cards::defs::mana_crypt::card();
    assert_eq!(card.name, "Mana Crypt");

    // First ability should be a triggered ability with CoinFlip effect.
    match &card.abilities[0] {
        mtg_engine::AbilityDefinition::Triggered { effect, .. } => {
            assert!(
                matches!(effect, Effect::CoinFlip { .. }),
                "Mana Crypt upkeep trigger should use Effect::CoinFlip"
            );
        }
        _ => panic!("Expected Triggered ability as first ability"),
    }
}
