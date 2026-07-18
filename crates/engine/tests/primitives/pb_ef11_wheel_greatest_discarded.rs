//! Tests for PB-EF11 COMMIT 1: `WheelDraw::GreatestDiscarded` (CR 121.1).
//!
//! Windfall's oracle ("Each player discards their hand, then draws cards equal
//! to the greatest number of cards a player discarded this way") needs a draw
//! count that is a MAX over every affected player's pre-disposal hand size — a
//! value only knowable after every player has disposed. The pre-existing
//! `Effect::WheelHand` executor (PB-AC9) was a single per-player
//! disposal-then-draw loop, which cannot express a shared cross-player value.
//! This batch restructures the executor into a two-pass branch keyed on the
//! draw variant: `GreatestDiscarded` disposes every affected player first
//! (recording each player's pre-disposal hand size), computes the max, then
//! has every affected player draw that max. `ThatMany`/`Fixed` retain their
//! original single-pass behavior byte-identically (regression-guarded by
//! `pb_ac9_wheel_and_misc.rs`, untouched by this batch).
//!
//! `HASH_SCHEMA_VERSION` bumped 53 -> 54 (new `WheelDraw::GreatestDiscarded`
//! discriminant 2).

use mtg_engine::cards::card_definition::{
    AbilityDefinition, Effect, PlayerTarget, WheelDisposal, WheelDraw,
};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, ZoneId,
    HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .zone(&ZoneId::Hand(player))
        .map(|z| z.len())
        .unwrap_or(0)
}

fn graveyard_count(state: &GameState, player: PlayerId) -> usize {
    state
        .zone(&ZoneId::Graveyard(player))
        .map(|z| z.len())
        .unwrap_or(0)
}

/// Run an effect directly (bypasses casting/resolution machinery).
fn run_effect(
    mut state: GameState,
    controller: PlayerId,
    effect: Effect,
) -> (GameState, Vec<GameEvent>) {
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
}

/// Build a 3-player state where p1/p2/p3 have `hands` cards in hand
/// (respectively) and `library_n` cards each in library.
fn setup_three_players(hands: [usize; 3], library_n: usize) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3));
    for (idx, pl) in [p(1), p(2), p(3)].iter().enumerate() {
        for i in 0..hands[idx] {
            builder = builder.object(
                ObjectSpec::card(*pl, &format!("Hand {:?} {}", pl, i)).in_zone(ZoneId::Hand(*pl)),
            );
        }
        for i in 0..library_n {
            builder = builder.object(
                ObjectSpec::card(*pl, &format!("Lib {:?} {}", pl, i)).in_zone(ZoneId::Library(*pl)),
            );
        }
    }
    builder.build().unwrap()
}

// ── HASH_SCHEMA_VERSION sentinel ──────────────────────────────────────────────

/// HASH_SCHEMA_VERSION live sentinel — fails if the schema version drifts
/// without this test being updated. PB-EF11 COMMIT 1 bumped 53 -> 54
/// (WheelDraw::GreatestDiscarded); COMMIT 2 bumped 54 -> 55
/// (TargetRequirement::TargetSpellWithSingleTarget, see pb_ef11_spell_single_target.rs).
#[test]
fn test_pb_ef11_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 55u8,
        "PB-EF11 bumped HASH_SCHEMA_VERSION 53->54->55 across its two commits. If you bumped \
         again, update this test."
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// WheelDraw::GreatestDiscarded (CR 121.1)
// ═══════════════════════════════════════════════════════════════════════════

/// CR 121.1 (Windfall) — 3 players with unequal hands (5, 2, 0). After
/// `WheelHand { EachPlayer, Discard, GreatestDiscarded }`, EVERY player's hand
/// == 5 (the max any player discarded).
#[test]
fn test_greatest_discarded_all_draw_max() {
    let state = setup_three_players([5, 2, 0], 10);

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::EachPlayer,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::GreatestDiscarded,
        },
    );

    for pl in [p(1), p(2), p(3)] {
        assert_eq!(
            hand_count(&state, pl),
            5,
            "player {:?} should have drawn the GREATEST discard count (5), not its own",
            pl
        );
    }
    // Discard counts should still reflect each player's own pre-disposal hand.
    assert_eq!(graveyard_count(&state, p(1)), 5);
    assert_eq!(graveyard_count(&state, p(2)), 2);
    assert_eq!(graveyard_count(&state, p(3)), 0);
}

/// DECOY (must be non-vacuous) — p2 (discarded 2) and p3 (discarded 0) must
/// each draw the SHARED MAX (5), NOT their own discard counts. This fails if
/// the executor uses per-player counts (the `ThatMany` behavior) instead of a
/// true cross-player max — distinct from the `ThatMany` regression already
/// pinned by `pb_ac9_wheel_and_misc.rs`.
#[test]
fn test_greatest_discarded_decoy_not_per_player() {
    let state = setup_three_players([5, 2, 0], 10);

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::EachPlayer,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::GreatestDiscarded,
        },
    );

    assert_eq!(
        hand_count(&state, p(2)),
        5,
        "p2 discarded 2 cards but must draw the shared max (5), not its own count"
    );
    assert_eq!(
        hand_count(&state, p(3)),
        5,
        "p3 discarded 0 cards but must draw the shared max (5), not its own count"
    );
    assert_ne!(
        hand_count(&state, p(2)),
        2,
        "p2 must NOT have drawn back only its own discard count"
    );
    assert_ne!(
        hand_count(&state, p(3)),
        0,
        "p3 must NOT have drawn 0 (its own discard count)"
    );
}

/// Edge case — all players have empty hands: the max is 0, nobody draws, no
/// panic (`unwrap_or(0)` on an empty `counts` iterator / a max of zero-valued
/// entries).
#[test]
fn test_greatest_discarded_empty_hands() {
    let state = setup_three_players([0, 0, 0], 10);

    let (state, events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::EachPlayer,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::GreatestDiscarded,
        },
    );

    for pl in [p(1), p(2), p(3)] {
        assert_eq!(
            hand_count(&state, pl),
            0,
            "player {:?}: all-empty hands means the max is 0, nobody draws",
            pl
        );
    }
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "no CardDrawn events should fire when the shared max is 0"
    );
}

/// Hash discriminant test — `hash_of(WheelHand{...GreatestDiscarded})` differs
/// from both `ThatMany` and `Fixed(5)`; pins discriminant 2 is actually hashed
/// (mirrors `pb_ac9_wheel_and_misc.rs`'s `test_hash_distinguishes_wheel_hand_payloads`).
#[test]
fn test_greatest_discarded_hash_discriminant() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let hash_of = |e: &Effect| -> [u8; 32] {
        let mut h = Hasher::new();
        e.hash_into(&mut h);
        *h.finalize().as_bytes()
    };

    let wheel = |draw: WheelDraw| Effect::WheelHand {
        player: PlayerTarget::EachPlayer,
        disposal: WheelDisposal::Discard,
        draw,
    };

    assert_ne!(
        hash_of(&wheel(WheelDraw::GreatestDiscarded)),
        hash_of(&wheel(WheelDraw::ThatMany)),
        "WheelDraw::GreatestDiscarded must hash differently from ThatMany"
    );
    assert_ne!(
        hash_of(&wheel(WheelDraw::GreatestDiscarded)),
        hash_of(&wheel(WheelDraw::Fixed(5))),
        "WheelDraw::GreatestDiscarded must hash differently from Fixed(5)"
    );
    assert_eq!(
        hash_of(&wheel(WheelDraw::GreatestDiscarded)),
        hash_of(&wheel(WheelDraw::GreatestDiscarded)),
        "identical WheelHand[GreatestDiscarded] effects must hash identically (sanity, \
         non-vacuity check on the assertions above)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Card-integration test (Windfall)
// ═══════════════════════════════════════════════════════════════════════════

/// CR 121.1 — Windfall: build a 3-player game with unequal hands, cast/execute
/// Windfall's spell body directly from the card def, and assert every player
/// draws the greatest discard count.
#[test]
fn test_windfall_card_def() {
    let card = mtg_engine::cards::defs::windfall::card();
    let AbilityDefinition::Spell { effect, .. } = &card.abilities[0] else {
        panic!("expected AbilityDefinition::Spell");
    };
    assert!(
        matches!(
            effect,
            Effect::WheelHand {
                player: PlayerTarget::EachPlayer,
                disposal: WheelDisposal::Discard,
                draw: WheelDraw::GreatestDiscarded,
            }
        ),
        "Windfall's spell body should be WheelHand{{EachPlayer, Discard, GreatestDiscarded}}"
    );

    let state = setup_three_players([4, 1, 0], 10);
    let (state, _events) = run_effect(state, p(1), effect.clone());

    for pl in [p(1), p(2), p(3)] {
        assert_eq!(
            hand_count(&state, pl),
            4,
            "player {:?} should draw the greatest discard count (4)",
            pl
        );
    }
}
