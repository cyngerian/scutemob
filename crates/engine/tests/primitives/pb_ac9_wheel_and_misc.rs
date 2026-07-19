//! Tests for PB-AC9: misc & mana (final AC-chain batch).
//!
//! Covers two genuinely-new engine primitives:
//!
//! - `Effect::WheelHand { player, disposal, draw }` (CR 701.9 / 701.24 / 121.1) —
//!   "each player discards/shuffles away their hand, then draws" family. Snapshots
//!   the hand size BEFORE disposal so `WheelDraw::ThatMany` reads the correct count
//!   (a naive DiscardCards{HandSize} + DrawCards{HandSize} sequence would read 0
//!   after the discard emptied the hand — this is the bug the old Incendiary
//!   Command / Reforge the Soul card defs worked around with `Fixed` approximations).
//! - `Effect::SetNoMaximumHandSize { player }` (CR 402.2) — a persistent "rest of
//!   the game" designation, distinct from the pre-existing battlefield-recomputed
//!   `no_max_hand_size` flag. Backed by `PlayerState.no_max_hand_size_permanent`,
//!   OR'd into the per-cleanup recompute (`turn_actions.rs`) so it survives even
//!   when no permanent grants it.
//!
//! Three of five briefed primitives (d20 results table, token-doubling replacement,
//! multi-output filter mana) were found to already exist and were dropped from
//! scope — see `memory/primitives/pb-plan-AC9.md` §0. This file also covers the
//! newly-authored card defs that consume `Effect::WheelHand`/`SetNoMaximumHandSize`
//! and the already-existing `RollDice`/`DoubleTokens` primitives via their card defs
//! (Ancient Copper/Gold/Silver Dragon, Doubling Season, Parallel Lives).
//!
//! Token-doubling completeness pass (§5): `apply_token_creation_replacement()` was
//! wired at only 2 of ~13 `GameEvent::TokenCreated` emission sites. This batch wired
//! the rest (`CreateTokenAndAttachSource`, Squad, Offspring, Myriad, Embalm,
//! Eternalize, Encore, Gift Food/Treasure). Per-class regression tests for that
//! completeness pass live in the existing keyword test files (`squad.rs`,
//! `myriad.rs`, `living_weapon.rs`, `gift.rs`), not here — see
//! `test_squad_doubling_season_doubles_token_batch`,
//! `test_myriad_doubling_season_doubles_per_opponent`,
//! `test_living_weapon_doubling_season_doubles_germ`,
//! `test_gift_treasure_doubled_when_recipient_controls_doubler` /
//! `test_gift_treasure_not_doubled_when_giver_controls_doubler`.
//!
//! Hash: `HASH_SCHEMA_VERSION` bumped 35 -> 36 (new `Effect::WheelHand` disc 91,
//! `Effect::SetNoMaximumHandSize` disc 92, new `PlayerState.no_max_hand_size_permanent`
//! field, new `WheelDisposal`/`WheelDraw` sub-enum `HashInto` impls).

use mtg_engine::cards::card_definition::{
    AbilityDefinition, Effect, PlayerTarget, WheelDisposal, WheelDraw,
};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::turn_actions::cleanup_actions;
use mtg_engine::{
    CardType, Color, ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, LayerModification, ObjectId,
    ObjectSpec, PlayerId, SubType, ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
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

fn library_count(state: &GameState, player: PlayerId) -> usize {
    state
        .zone(&ZoneId::Library(player))
        .map(|z| z.len())
        .unwrap_or(0)
}

fn exile_count(state: &GameState) -> usize {
    state.zone(&ZoneId::Exile).map(|z| z.len()).unwrap_or(0)
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

/// Build a 2-player state with `hand_n` cards in p1's hand and `library_n` cards
/// in p1's library.
fn setup_hand_and_library(hand_n: usize, library_n: usize) -> GameState {
    let mut builder = GameStateBuilder::new().add_player(p(1)).add_player(p(2));
    for i in 0..hand_n {
        builder = builder.object(
            ObjectSpec::card(p(1), &format!("Hand Card {}", i)).in_zone(ZoneId::Hand(p(1))),
        );
    }
    for i in 0..library_n {
        builder = builder.object(
            ObjectSpec::card(p(1), &format!("Library Card {}", i)).in_zone(ZoneId::Library(p(1))),
        );
    }
    builder.build().unwrap()
}

// ── HASH_SCHEMA_VERSION sentinel ──────────────────────────────────────────────

/// HASH_SCHEMA_VERSION live sentinel — fails if the schema version drifts without
/// this test being updated. PB-AC9 bumped 35 -> 36.
#[test]
fn test_pb_ac9_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 63u8,
        "PB-AC9 bumped HASH_SCHEMA_VERSION 35->36 (Effect::WheelHand disc 91, \
         Effect::SetNoMaximumHandSize disc 92, PlayerState.no_max_hand_size_permanent). \
         If you bumped again, update this test."
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Effect::WheelHand (CR 701.9 / 701.24 / 121.1)
// ═══════════════════════════════════════════════════════════════════════════

/// CR 701.9 + 121.1 — Discard/ThatMany: 3-card hand + library, hand size is
/// snapshotted BEFORE disposal, so the player draws back exactly 3 (not 0).
#[test]
fn test_wheel_hand_discard_that_many() {
    let state = setup_hand_and_library(3, 10);
    let lib_before = library_count(&state, p(1));

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::ThatMany,
        },
    );

    assert_eq!(
        hand_count(&state, p(1)),
        3,
        "CR 121.1: player should draw back exactly the pre-disposal hand size (3)"
    );
    assert_eq!(
        graveyard_count(&state, p(1)),
        3,
        "CR 701.9: the discarded 3 cards should be in the graveyard"
    );
    assert_eq!(
        library_count(&state, p(1)),
        lib_before - 3,
        "library should shrink by exactly the 3 cards drawn"
    );
}

/// Wheel of Fortune / Reforge the Soul shape — Discard/Fixed(7): a 2-card hand
/// still discards its WHOLE hand (2 cards, not a fixed 7) then draws a fixed 7.
#[test]
fn test_wheel_hand_fixed_draw() {
    let state = setup_hand_and_library(2, 10);

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::Fixed(7),
        },
    );

    assert_eq!(
        hand_count(&state, p(1)),
        7,
        "WheelDraw::Fixed(7) should draw exactly 7 cards regardless of hand size"
    );
    assert_eq!(
        graveyard_count(&state, p(1)),
        2,
        "WheelDisposal::Discard should discard the WHOLE hand (2 cards), not a fixed count"
    );
}

/// Edge case — an empty hand: Discard/ThatMany draws 0 cards, no panic.
#[test]
fn test_wheel_hand_empty_hand_noop() {
    let state = setup_hand_and_library(0, 10);
    let lib_before = library_count(&state, p(1));

    let (state, events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::ThatMany,
        },
    );

    assert_eq!(
        hand_count(&state, p(1)),
        0,
        "empty hand + ThatMany should draw 0 cards"
    );
    assert_eq!(
        library_count(&state, p(1)),
        lib_before,
        "library should be untouched when hand was empty"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "no CardDrawn events should fire for a 0-card wheel"
    );
}

/// CR 701.24 (Winds of Change) — ShuffleHandIntoLibrary/ThatMany: hand cards go
/// into the library (NOT the graveyard), then the player draws back the same
/// count. Also verifies determinism: two identical initial states produce
/// identical resulting state hashes (same timestamp_counter seed sequence).
#[test]
fn test_wheel_hand_shuffle_into_library_that_many() {
    let state1 = setup_hand_and_library(3, 10);
    let state2 = setup_hand_and_library(3, 10);

    let (state1, events1) = run_effect(
        state1,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::ShuffleHandIntoLibrary,
            draw: WheelDraw::ThatMany,
        },
    );
    let (state2, _events2) = run_effect(
        state2,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::ShuffleHandIntoLibrary,
            draw: WheelDraw::ThatMany,
        },
    );

    assert_eq!(
        hand_count(&state1, p(1)),
        3,
        "CR 701.24: player draws back the pre-disposal hand size (3)"
    );
    assert_eq!(
        graveyard_count(&state1, p(1)),
        0,
        "CR 701.24: shuffle-into-library disposal must NOT touch the graveyard"
    );
    assert_eq!(
        library_count(&state1, p(1)),
        10,
        "net library count unchanged: +3 shuffled in, -3 drawn back"
    );
    assert!(
        events1
            .iter()
            .any(|e| matches!(e, GameEvent::LibraryShuffled { .. })),
        "shuffle disposal must emit LibraryShuffled"
    );

    // Determinism: identical starting states (same timestamp_counter=0 default)
    // must produce identical resulting state hashes.
    assert_eq!(
        state1.public_state_hash(),
        state2.public_state_hash(),
        "CR: shuffle disposal is deterministic given the same game state sequence \
         (seeded from timestamp_counter, not entropy)"
    );
}

/// CR 701.24 (Echo of Eons / Timetwister) — ShuffleHandAndGraveyardIntoLibrary +
/// Fixed(7): both hand AND graveyard empty into the library, then draw a fixed 7.
#[test]
fn test_wheel_hand_shuffle_hand_and_graveyard_fixed() {
    let mut builder = GameStateBuilder::new().add_player(p(1)).add_player(p(2));
    for i in 0..2 {
        builder = builder.object(
            ObjectSpec::card(p(1), &format!("Hand Card {}", i)).in_zone(ZoneId::Hand(p(1))),
        );
    }
    for i in 0..3 {
        builder = builder.object(
            ObjectSpec::card(p(1), &format!("GY Card {}", i)).in_zone(ZoneId::Graveyard(p(1))),
        );
    }
    for i in 0..20 {
        builder = builder.object(
            ObjectSpec::card(p(1), &format!("Library Card {}", i)).in_zone(ZoneId::Library(p(1))),
        );
    }
    let state = builder.build().unwrap();

    assert_eq!(graveyard_count(&state, p(1)), 3, "test setup: 3 GY cards");
    assert_eq!(hand_count(&state, p(1)), 2, "test setup: 2 hand cards");

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::ShuffleHandAndGraveyardIntoLibrary,
            draw: WheelDraw::Fixed(7),
        },
    );

    assert_eq!(
        graveyard_count(&state, p(1)),
        0,
        "CR 701.24: graveyard must be empty after shuffling it into the library"
    );
    assert_eq!(
        hand_count(&state, p(1)),
        7,
        "WheelDraw::Fixed(7) draws exactly 7 cards"
    );
}

/// CR: APNAP order via `resolve_player_target_list`'s `PlayerTarget::EachPlayer` —
/// in a 4-player game, EachPlayer/Discard/ThatMany preserves every player's hand
/// size (draw back what was discarded) and every player's graveyard grows.
#[test]
fn test_wheel_hand_each_player_multiplayer() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4));
    for pl in [p(1), p(2), p(3), p(4)] {
        for i in 0..3 {
            builder = builder.object(
                ObjectSpec::card(pl, &format!("Hand {:?} {}", pl, i)).in_zone(ZoneId::Hand(pl)),
            );
        }
        for i in 0..10 {
            builder = builder.object(
                ObjectSpec::card(pl, &format!("Lib {:?} {}", pl, i)).in_zone(ZoneId::Library(pl)),
            );
        }
    }
    let state = builder.build().unwrap();

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::EachPlayer,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::ThatMany,
        },
    );

    for pl in [p(1), p(2), p(3), p(4)] {
        assert_eq!(
            hand_count(&state, pl),
            3,
            "player {:?} should have drawn back its pre-disposal hand size (3)",
            pl
        );
        assert_eq!(
            graveyard_count(&state, pl),
            3,
            "player {:?} should have discarded 3 cards to the graveyard",
            pl
        );
    }
}

/// CR 702.35a regression guard on reuse — `WheelHand`'s Discard disposal routes
/// through the same `discard_cards` helper as `Effect::DiscardCards`, so a card
/// with Madness in the wheeled hand is exiled (not put into the graveyard) and a
/// MadnessTrigger is pushed.
#[test]
fn test_wheel_hand_madness_routes_to_exile() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Madness Card")
                .in_zone(ZoneId::Hand(p(1)))
                .with_keyword(KeywordAbility::Madness),
        )
        .object(ObjectSpec::card(p(1), "Library Filler").in_zone(ZoneId::Library(p(1))))
        .build()
        .unwrap();

    let (state, events) = run_effect(
        state,
        p(1),
        Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::ThatMany,
        },
    );

    assert_eq!(
        graveyard_count(&state, p(1)),
        0,
        "CR 702.35a: a Madness card discarded by WheelHand must NOT go to the graveyard"
    );
    assert_eq!(
        exile_count(&state),
        1,
        "CR 702.35a: the Madness card should be in exile instead"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { .. })),
        "CR ruling: CardDiscarded still fires even though the card is exiled"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Effect::SetNoMaximumHandSize (CR 402.2)
// ═══════════════════════════════════════════════════════════════════════════

/// CR 402.2 / 514.1a — a player with the persistent flag set (no permanent on
/// the battlefield granting it) skips the cleanup discard, while a control
/// player without the flag discards down to 7 as usual.
#[test]
fn test_set_no_maximum_hand_size_survives_cleanup() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1));
    for i in 0..10 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("P1 Card {}", i)).in_zone(ZoneId::Hand(p(1))));
    }
    let mut state = builder.build().unwrap();

    // Set the persistent flag via the effect (no battlefield permanent involved).
    let mut ctx = EffectContext::new(p(1), ObjectId(0), vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::SetNoMaximumHandSize {
            player: PlayerTarget::Controller,
        },
        &mut ctx,
    );

    assert!(
        state
            .players()
            .get(&p(1))
            .unwrap()
            .no_max_hand_size_permanent,
        "SetNoMaximumHandSize must set the persistent flag"
    );

    let hand_before = hand_count(&state, p(1));
    assert!(
        hand_before > 7,
        "test setup: hand must exceed max hand size"
    );

    let _events = cleanup_actions(&mut state);

    assert_eq!(
        hand_count(&state, p(1)),
        hand_before,
        "CR 402.2: a player with no_max_hand_size_permanent must skip the cleanup discard \
         entirely, even with zero NoMaxHandSize permanents on the battlefield"
    );
}

/// Direct OR-logic verification: run the cleanup recompute and confirm the flag
/// survives even when the battlefield scan (`has_no_max`) independently returns
/// false. This is the exact bug PB-AC9 fixed — the recompute previously did
/// `ps.no_max_hand_size = has_no_max;` (clobbering), now
/// `ps.no_max_hand_size = has_no_max || ps.no_max_hand_size_permanent;`.
#[test]
fn test_set_no_max_hand_size_recompute_does_not_clobber() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .build()
        .unwrap();

    // No battlefield permanent grants NoMaxHandSize -- has_no_max will be false.
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .no_max_hand_size_permanent = true;

    let _events = cleanup_actions(&mut state);

    assert!(
        state.players().get(&p(1)).unwrap().no_max_hand_size,
        "the per-cleanup recompute must OR in no_max_hand_size_permanent, not \
         clobber it back to false when the battlefield scan finds nothing"
    );
}

/// AC8 regression + stacking: a LAYER-granted `NoMaxHandSize` (e.g. an emblem
/// proxy) still works via `has_no_max` at the same time as a persistent
/// `SetNoMaximumHandSize` designation on a DIFFERENT player -- confirms the OR
/// does not disturb the pre-existing layer-correctness fix from PB-AC8.
#[test]
fn test_set_no_max_hand_size_stacks_with_layer_granted_source() {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(2), "Emblem Proxy Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .active_player(p(2));
    for i in 0..10 {
        builder = builder
            .object(ObjectSpec::card(p(2), &format!("P2 Card {}", i)).in_zone(ZoneId::Hand(p(2))));
    }
    let mut state = builder.build().unwrap();

    // P2 gets NoMaxHandSize via a Layer 6 continuous effect (mirrors PB-AC8's regression test).
    let bear = find_by_name(&state, "Emblem Proxy Bear");
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(bear),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(bear),
        modification: LayerModification::AddKeyword(KeywordAbility::NoMaxHandSize),
        is_cda: false,
        condition: None,
    });

    // P1 (not active this turn) separately has the persistent designation set --
    // should have no effect on THIS cleanup (only the active player discards),
    // but must not be clobbered either.
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .no_max_hand_size_permanent = true;

    let hand_before = hand_count(&state, p(2));
    assert!(
        hand_before > 7,
        "test setup: hand must exceed max hand size"
    );

    let _events = cleanup_actions(&mut state);

    assert_eq!(
        hand_count(&state, p(2)),
        hand_before,
        "AC8 regression: layer-granted NoMaxHandSize must still skip the active \
         player's cleanup discard after the PB-AC9 OR change"
    );
    assert!(
        state
            .players()
            .get(&p(1))
            .unwrap()
            .no_max_hand_size_permanent,
        "the non-active player's persistent flag must remain set (untouched by \
         this cleanup, which only recomputes the active player)"
    );
}

/// Mutation-verified hash test (hazard 1): the public state hash must differ
/// between two states that differ ONLY in `no_max_hand_size_permanent`.
#[test]
fn test_no_max_hand_size_permanent_hash_mutation() {
    let state_false = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();
    let mut state_true = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();
    state_true
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .no_max_hand_size_permanent = true;

    assert_ne!(
        state_false.public_state_hash(),
        state_true.public_state_hash(),
        "flipping no_max_hand_size_permanent must change the public state hash"
    );
}

/// PB-AC9 review finding **E2** (LOW) — the mutation test above covers the new
/// `PlayerState` field, but nothing pinned the `WheelDisposal` / `WheelDraw`
/// payload hashes. Two `Effect::WheelHand` values that differ only in their
/// disposal (or only in their draw) must hash differently, or a replay could
/// silently accept a divergent script.
#[test]
fn test_hash_distinguishes_wheel_hand_payloads() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let hash_of = |e: &Effect| -> [u8; 32] {
        let mut h = Hasher::new();
        e.hash_into(&mut h);
        *h.finalize().as_bytes()
    };

    let wheel = |disposal: WheelDisposal, draw: WheelDraw| Effect::WheelHand {
        player: PlayerTarget::EachPlayer,
        disposal,
        draw,
    };

    // Disposal discriminates: Discard vs ShuffleIntoLibrary.
    assert_ne!(
        hash_of(&wheel(WheelDisposal::Discard, WheelDraw::ThatMany)),
        hash_of(&wheel(
            WheelDisposal::ShuffleHandIntoLibrary,
            WheelDraw::ThatMany
        )),
        "WheelDisposal variants must hash differently"
    );

    // Draw discriminates: ThatMany vs Fixed(7).
    assert_ne!(
        hash_of(&wheel(WheelDisposal::Discard, WheelDraw::ThatMany)),
        hash_of(&wheel(WheelDisposal::Discard, WheelDraw::Fixed(7))),
        "WheelDraw variants must hash differently"
    );

    // Fixed payload discriminates: Fixed(7) vs Fixed(3).
    assert_ne!(
        hash_of(&wheel(WheelDisposal::Discard, WheelDraw::Fixed(7))),
        hash_of(&wheel(WheelDisposal::Discard, WheelDraw::Fixed(3))),
        "WheelDraw::Fixed payload must be hashed, not just the discriminant"
    );

    // Sanity: identical effects hash identically (the assertions above are not vacuous).
    assert_eq!(
        hash_of(&wheel(WheelDisposal::Discard, WheelDraw::Fixed(7))),
        hash_of(&wheel(WheelDisposal::Discard, WheelDraw::Fixed(7))),
        "identical WheelHand effects must hash identically"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Card-integration tests (RollDice + WheelHand + SetNoMaximumHandSize card defs)
// ═══════════════════════════════════════════════════════════════════════════

/// Extract the `Effect` for a given ability index from a card's abilities vec,
/// expecting an `AbilityDefinition::Triggered`.
fn triggered_effect(card: &mtg_engine::CardDefinition, index: usize) -> Effect {
    match &card.abilities[index] {
        AbilityDefinition::Triggered { effect, .. } => effect.clone(),
        other => panic!("ability {} is not Triggered: {:?}", index, other),
    }
}

/// Incendiary Command mode 3 wheels all players (CR 701.9 / 121.1). Extracts
/// the mode-3 effect directly from the card def's modal spell and executes it.
#[test]
fn test_incendiary_command_mode3_wheels_all_players() {
    let card = mtg_engine::cards::defs::incendiary_command::card();
    let AbilityDefinition::Spell { modes, .. } = &card.abilities[0] else {
        panic!("expected AbilityDefinition::Spell");
    };
    let mode_selection = modes.as_ref().expect("modal spell should have modes");
    let mode3 = mode_selection.modes[3].clone();
    assert!(
        matches!(mode3, Effect::WheelHand { .. }),
        "Incendiary Command mode 3 should use Effect::WheelHand"
    );

    let mut builder = GameStateBuilder::new().add_player(p(1)).add_player(p(2));
    for pl in [p(1), p(2)] {
        for i in 0..2 {
            builder = builder.object(
                ObjectSpec::card(pl, &format!("Hand {:?} {}", pl, i)).in_zone(ZoneId::Hand(pl)),
            );
        }
        for i in 0..10 {
            builder = builder.object(
                ObjectSpec::card(pl, &format!("Lib {:?} {}", pl, i)).in_zone(ZoneId::Library(pl)),
            );
        }
    }
    let state = builder.build().unwrap();

    let (state, _events) = run_effect(state, p(1), mode3);

    for pl in [p(1), p(2)] {
        assert_eq!(
            hand_count(&state, pl),
            2,
            "each player should draw back its pre-disposal hand size (2)"
        );
        assert_eq!(
            graveyard_count(&state, pl),
            2,
            "each player should discard 2 cards"
        );
    }
}

/// CR 702.94a — Reforge the Soul carries BOTH miracle ability definitions, and its
/// spell body is the real wheel (not the old `Fixed(7)` discard approximation).
///
/// Regression guard for a *stale marker*: the def previously carried
/// `// TODO: Miracle {1}{R} — KeywordAbility::Miracle not yet implemented`, but
/// Miracle has long been implemented (`rules/miracle.rs`, `Command::ChooseMiracle`)
/// and is used by `terminus.rs` / `temporal_mastery.rs`. The marker, not the engine,
/// was what kept this card incomplete. See `memory/card-authoring/review-pb-ac9-backfill.md`.
#[test]
fn test_reforge_the_soul_has_miracle_and_wheel_body() {
    let card = mtg_engine::cards::defs::reforge_the_soul::card();

    assert!(
        card.abilities
            .iter()
            .any(|a| matches!(a, AbilityDefinition::Keyword(KeywordAbility::Miracle))),
        "Reforge the Soul must carry the Miracle keyword marker (CR 702.94a)"
    );

    let miracle_cost = card
        .abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::Miracle { cost } => Some(cost.clone()),
            _ => None,
        })
        .expect("Reforge the Soul must carry AbilityDefinition::Miracle { cost }");
    assert_eq!(miracle_cost.generic, 1, "miracle cost is {{1}}{{R}}");
    assert_eq!(miracle_cost.red, 1, "miracle cost is {{1}}{{R}}");

    let spell = card
        .abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::Spell { effect, .. } => Some(effect.clone()),
            _ => None,
        })
        .expect("Reforge the Soul must have a spell body");
    assert!(
        matches!(
            spell,
            Effect::WheelHand {
                player: PlayerTarget::EachPlayer,
                disposal: WheelDisposal::Discard,
                draw: WheelDraw::Fixed(7),
            }
        ),
        "oracle: 'Each player discards their hand, then draws seven cards.'"
    );

    // No stale marker may remain in the source.
    let src = include_str!("../../../card-defs/src/defs/reforge_the_soul.rs");
    assert!(
        !src.contains("TODO") && !src.contains("ENGINE-BLOCKED"),
        "Reforge the Soul is fully authored; no TODO/ENGINE-BLOCKED marker may remain"
    );
}

/// CR 706 + 614.1 — Ancient Copper Dragon: force a d20 roll and assert that many
/// Treasure tokens are created.
#[test]
fn test_ancient_copper_dragon_rolls_treasures() {
    let card = mtg_engine::cards::defs::ancient_copper_dragon::card();
    let effect = triggered_effect(&card, 1);
    assert!(
        matches!(effect, Effect::RollDice { .. }),
        "Ancient Copper Dragon's combat-damage trigger should use Effect::RollDice"
    );

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();
    // timestamp 6 -> (6 % 20) + 1 = 7
    *state.timestamp_counter_mut() = 6;

    let (state, events) = run_effect(state, p(1), effect);

    let treasure_count = state
        .objects()
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.controller == p(1)
                && o.characteristics.name == "Treasure"
        })
        .count();
    assert_eq!(
        treasure_count, 7,
        "CR 706: should create Treasure tokens equal to the d20 roll result (7)"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DiceRolled { result: 7, .. })),
        "DiceRolled event with result=7 expected"
    );
}

/// CR 706 — Ancient Gold Dragon: force a d20 roll and assert that many 1/1 blue
/// Faerie Dragon flying tokens are created.
#[test]
fn test_ancient_gold_dragon_faerie_tokens() {
    let card = mtg_engine::cards::defs::ancient_gold_dragon::card();
    let effect = triggered_effect(&card, 1);
    assert!(
        matches!(effect, Effect::RollDice { .. }),
        "Ancient Gold Dragon's combat-damage trigger should use Effect::RollDice"
    );

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();
    // timestamp 2 -> (2 % 20) + 1 = 3
    *state.timestamp_counter_mut() = 2;

    let (state, _events) = run_effect(state, p(1), effect);

    let faerie_dragons: Vec<_> = state
        .objects()
        .values()
        .filter(|o| {
            o.zone == ZoneId::Battlefield
                && o.controller == p(1)
                && o.characteristics.name == "Faerie Dragon"
        })
        .collect();
    assert_eq!(
        faerie_dragons.len(),
        3,
        "CR 706: should create 1/1 Faerie Dragon tokens equal to the d20 roll (3)"
    );
    for token in &faerie_dragons {
        assert_eq!(token.characteristics.power, Some(1));
        assert_eq!(token.characteristics.toughness, Some(1));
        assert!(token.characteristics.colors.contains(&Color::Blue));
        assert!(token
            .characteristics
            .card_types
            .contains(&CardType::Creature));
        assert!(token
            .characteristics
            .subtypes
            .contains(&SubType("Faerie".to_string())));
        assert!(token
            .characteristics
            .subtypes
            .contains(&SubType("Dragon".to_string())));
        assert!(token
            .characteristics
            .keywords
            .contains(&KeywordAbility::Flying));
    }
}

/// CR 706 + 402.2 — Ancient Silver Dragon: force a d20 roll, assert the draw
/// count AND that the persistent no_max_hand_size flag is set (both parts of
/// the same triggered ability, CR 706.3b).
#[test]
fn test_ancient_silver_dragon_draw_and_no_max() {
    let card = mtg_engine::cards::defs::ancient_silver_dragon::card();
    let effect = triggered_effect(&card, 1);
    let Effect::Sequence(steps) = &effect else {
        panic!("expected Effect::Sequence(RollDice, SetNoMaximumHandSize)");
    };
    assert!(matches!(steps[0], Effect::RollDice { .. }));
    assert!(matches!(steps[1], Effect::SetNoMaximumHandSize { .. }));

    let mut builder = GameStateBuilder::new().add_player(p(1)).add_player(p(2));
    for i in 0..20 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Lib {}", i)).in_zone(ZoneId::Library(p(1))));
    }
    let mut state = builder.build().unwrap();
    // `next_object_id()` shares the SAME counter as dice-roll seeding
    // (`timestamp_counter`) -- forcing it BELOW the post-build object count
    // would collide new draw-object ids with existing library card ids and
    // silently corrupt the library zone. Use a value well above any object id
    // allocated by the 20-card builder setup: 1009 % 20 = 9 -> roll = 10.
    *state.timestamp_counter_mut() = 1009;

    assert_eq!(
        library_count(&state, p(1)),
        20,
        "test setup: 20 library cards expected"
    );
    assert!(
        !state
            .players()
            .get(&p(1))
            .unwrap()
            .no_max_hand_size_permanent,
        "test setup: flag should start false"
    );

    let (state, events) = run_effect(state, p(1), effect);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::DiceRolled { result: 10, .. })),
        "expected DiceRolled result=10; events: {:?}",
        events
    );
    let draw_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { .. }))
        .count();
    assert_eq!(
        draw_count, 10,
        "should draw cards equal to the d20 roll result (10); events: {:?}",
        events
    );
    assert!(
        state
            .players()
            .get(&p(1))
            .unwrap()
            .no_max_hand_size_permanent,
        "CR 402.2: no_max_hand_size_permanent must be set after the combat damage trigger"
    );
}

/// CR 111.1 + 122.6 + 614.1 — Doubling Season: both the token-doubling AND
/// counter-doubling replacement abilities register and apply.
#[test]
fn test_doubling_season_doubles_tokens_and_counters() {
    let registry =
        mtg_engine::CardRegistry::new(vec![mtg_engine::cards::defs::doubling_season::card()]);

    let mut ds_spec = ObjectSpec::artifact(p(1), "Doubling Season").in_zone(ZoneId::Battlefield);
    ds_spec.card_id = Some(mtg_engine::CardId("doubling-season".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(ds_spec)
        .object(ObjectSpec::creature(p(1), "Counter Target", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let mut state = state;
    let ds_id = find_by_name(&state, "Doubling Season");
    let card_id = mtg_engine::CardId("doubling-season".to_string());
    let registry = state.card_registry().clone();
    mtg_engine::rules::replacement::register_permanent_replacement_abilities(
        &mut state,
        ds_id,
        p(1),
        Some(&card_id),
        &registry,
    );

    // Token doubling: create 1 Treasure token -> should become 2.
    let (state, _) = run_effect(
        state,
        p(1),
        Effect::CreateToken {
            spec: mtg_engine::cards::card_definition::treasure_token_spec(1),
        },
    );
    let treasure_count = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Treasure")
        .count();
    assert_eq!(
        treasure_count, 2,
        "CR 111.1: Doubling Season should double 1 Treasure token into 2"
    );

    // Counter doubling: add 1 +1/+1 counter -> should become 2.
    let target = find_by_name(&state, "Counter Target");
    let mut ctx = EffectContext::new(
        p(1),
        ObjectId(0),
        vec![mtg_engine::state::targeting::SpellTarget {
            target: mtg_engine::state::targeting::Target::Object(target),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    let mut state = state;
    let _ = execute_effect(
        &mut state,
        &Effect::AddCounter {
            target: mtg_engine::cards::card_definition::EffectTarget::DeclaredTarget { index: 0 },
            counter: mtg_engine::CounterType::PlusOnePlusOne,
            count: 1,
        },
        &mut ctx,
    );
    let counters = state
        .objects()
        .get(&target)
        .unwrap()
        .counters
        .get(&mtg_engine::CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 2,
        "CR 122.6: Doubling Season should double 1 counter placement into 2"
    );
}

/// CR 111.1 — Parallel Lives: sanity check the copied Adrix/Elspeth-style
/// token-doubling replacement registers correctly from the real card def.
#[test]
fn test_parallel_lives_doubles_tokens() {
    let registry =
        mtg_engine::CardRegistry::new(vec![mtg_engine::cards::defs::parallel_lives::card()]);

    let mut pl_spec = ObjectSpec::artifact(p(1), "Parallel Lives").in_zone(ZoneId::Battlefield);
    pl_spec.card_id = Some(mtg_engine::CardId("parallel-lives".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(pl_spec)
        .build()
        .unwrap();

    let mut state = state;
    let pl_id = find_by_name(&state, "Parallel Lives");
    let card_id = mtg_engine::CardId("parallel-lives".to_string());
    let registry = state.card_registry().clone();
    mtg_engine::rules::replacement::register_permanent_replacement_abilities(
        &mut state,
        pl_id,
        p(1),
        Some(&card_id),
        &registry,
    );

    let (state, _) = run_effect(
        state,
        p(1),
        Effect::CreateToken {
            spec: mtg_engine::cards::card_definition::treasure_token_spec(1),
        },
    );
    let treasure_count = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Treasure")
        .count();
    assert_eq!(
        treasure_count, 2,
        "CR 111.1: Parallel Lives should double 1 Treasure token into 2"
    );
}
