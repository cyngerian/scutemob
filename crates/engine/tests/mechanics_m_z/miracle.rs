//! Miracle (CR 702.94) — the "as you draw it" conjunct.
//!
//! **This file was EMPTY (one byte) and `mod miracle;`-declared before PB-DX18**, and it
//! had been since SR-9a (`aaf1b664`). Miracle's only coverage in the whole tree was one
//! golden script (`test-data/generated-scripts/stack/084_terminus_miracle_draw_cast.json`),
//! which drives the happy path. That is a large part of why `OOS-DX2-1` survived: the
//! module whose name promises coverage contained none, and a `mod` line naming an empty
//! file reads exactly like coverage. Filed as `OOS-DX18-2`, with the corpus census (the
//! other empty-but-`mod`'d module is `crates/engine/tests/rules/effects.rs`).
//!
//! ## What PB-DX18 fixed here
//!
//! CR 702.94a: *"You may reveal this card from your hand **as you draw it** if it's the
//! first card you've drawn this turn."* — **two** conjuncts.
//! `rules::miracle::handle_choose_miracle` validated the card is in hand, has the Miracle
//! keyword, and `cards_drawn_this_turn == 1`. That is the second conjunct and the hand
//! zone; nothing checked WHICH object was drawn. A miracle card already in hand — tutored,
//! drawn last turn, discarded and returned — could therefore be revealed and cast for its
//! miracle cost on any turn whose first draw had already happened.
//!
//! `PlayerState::miracle_pending` records the just-drawn object at the one site that emits
//! `GameEvent::MiracleRevealChoiceRequired`, and `handle_choose_miracle` requires the named
//! card to BE it.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameState, GameStateBuilder, GameStateError, KeywordAbility, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn miracle_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dx18-miracle".to_string()),
        name: "DX18 Miracle Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Miracle {W}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Miracle),
            AbilityDefinition::Miracle {
                cost: ManaCost {
                    white: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

fn miracle_spec(owner: PlayerId, zone: ZoneId) -> ObjectSpec {
    ObjectSpec::card(owner, "DX18 Miracle Card")
        .in_zone(zone)
        .with_card_id(CardId("dx18-miracle".to_string()))
        .with_keyword(KeywordAbility::Miracle)
}

/// A two-seat state at upkeep whose next draw-step draw is `p1`'s first of the turn, with
/// the miracle card on TOP of `p1`'s library. `held_in_hand` optionally pre-places a
/// SECOND copy in `p1`'s hand — the tutored card `OOS-DX2-1` is about.
fn build_draw_state(p1: PlayerId, p2: PlayerId, held_in_hand: bool) -> GameState {
    let registry = CardRegistry::new(vec![miracle_def()]);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::Upkeep);
    if held_in_hand {
        builder = builder.object(miracle_spec(p1, ZoneId::Hand(p1)));
    }
    // Library filler UNDER the miracle card: `Zone::top()` is the LAST element, so the
    // miracle card must be added last to be drawn first.
    for i in 0..3 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Filler {i}")).in_zone(ZoneId::Library(p1)));
    }
    builder = builder.object(miracle_spec(p1, ZoneId::Library(p1)));
    let mut state = builder.build().unwrap();
    // CR 103.8: not the first turn, so the draw step really draws.
    state.turn_mut().is_first_turn_of_game = false;
    state.turn_mut().priority_holder = Some(p1);
    state
}

/// Pass priority until the draw-step draw has happened, returning the events seen.
fn advance_to_draw(state: GameState, p1: PlayerId, p2: PlayerId) -> (GameState, Vec<GameEvent>) {
    let mut all = Vec::new();
    let mut cur = state;
    for _ in 0..4 {
        for pl in [p1, p2] {
            let (s, ev) = process_command(cur, Command::PassPriority { player: pl })
                .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
            cur = s;
            all.extend(ev);
        }
        if all
            .iter()
            .any(|e| matches!(e, GameEvent::MiracleRevealChoiceRequired { .. }))
        {
            break;
        }
    }
    (cur, all)
}

fn drawn_miracle_id(events: &[GameEvent]) -> ObjectId {
    events
        .iter()
        .find_map(|e| match e {
            GameEvent::MiracleRevealChoiceRequired { card_object_id, .. } => Some(*card_object_id),
            _ => None,
        })
        .expect("CR 702.94a: the draw must offer the miracle reveal")
}

/// Every miracle trigger this reveal produced, wherever it currently sits.
///
/// `process_command` flushes pending triggers onto the stack before returning, so counting
/// `state.pending_triggers()` alone reports **zero** on the happy path. Both homes are
/// counted so the assertion is about the trigger existing exactly once, not about which
/// side of the flush the observation happened on.
fn miracle_triggers(state: &GameState) -> usize {
    let pending = state
        .pending_triggers()
        .iter()
        .filter(|t| {
            matches!(
                t.kind,
                mtg_engine::state::stubs::PendingTriggerKind::Miracle
            )
        })
        .count();
    let on_stack = state
        .stack_objects()
        .iter()
        .filter(|so| matches!(so.kind, mtg_engine::StackObjectKind::MiracleTrigger { .. }))
        .count();
    pending + on_stack
}

// ── Control: the legal path still works ───────────────────────────────────────

#[test]
/// CR 702.94a — the card that was just drawn CAN be revealed. Control for every refusal
/// below: the gate must refuse more than HEAD did and nothing else.
fn t1_the_just_drawn_card_can_be_revealed() {
    let (p1, p2) = (p(1), p(2));
    let (state, events) = advance_to_draw(build_draw_state(p1, p2, false), p1, p2);
    let drawn = drawn_miracle_id(&events);
    assert_eq!(
        state.players().get(&p1).unwrap().miracle_pending,
        Some(drawn),
        "the draw records WHICH object was drawn (CR 702.94a's first conjunct)"
    );
    let (state, _) = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: drawn,
            reveal: true,
        },
    )
    .expect("CR 702.94a: revealing the just-drawn card is legal");
    assert_eq!(miracle_triggers(&state), 1);
}

// ── The defect (`OOS-DX2-1`) ──────────────────────────────────────────────────

#[test]
/// CR 702.94a — a miracle card ALREADY IN HAND cannot be revealed off someone else's draw.
///
/// This is the whole seed. `cards_drawn_this_turn == 1` holds (it is the first draw of the
/// turn), the held card is in `p1`'s hand, and it carries the Miracle keyword — so every
/// check the pre-PB-DX18 handler made passes. It is simply not the card that was drawn.
fn t2_a_held_miracle_card_cannot_be_revealed() {
    let (p1, p2) = (p(1), p(2));
    let (state, events) = advance_to_draw(build_draw_state(p1, p2, true), p1, p2);
    let drawn = drawn_miracle_id(&events);

    // NON-VACUITY: there really are two miracle cards in hand, and every pre-PB-DX18
    // check passes for the held one.
    let held: Vec<ObjectId> = state
        .zone(&ZoneId::Hand(p1))
        .unwrap()
        .object_ids()
        .into_iter()
        .filter(|id| *id != drawn)
        .collect();
    assert_eq!(
        held.len(),
        1,
        "exactly one HELD miracle card beside the drawn one"
    );
    let held = held[0];
    assert_eq!(state.players().get(&p1).unwrap().cards_drawn_this_turn, 1);
    assert!(state
        .object(held)
        .unwrap()
        .characteristics
        .keywords
        .contains(&KeywordAbility::Miracle));

    let err = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: held,
            reveal: true,
        },
    )
    .expect_err("CR 702.94a: 'as you draw it' — the held card was not drawn");
    match err {
        GameStateError::InvalidCommand(m) => assert!(
            m.contains("just drew") && m.contains("702.94a"),
            "got {m:?}"
        ),
        other => panic!("expected InvalidCommand, got {other:?}"),
    }
}

#[test]
/// CR 702.94a — the reveal is answered ONCE. A second `ChooseMiracle` for the same draw is
/// refused, so it cannot queue a second miracle trigger.
fn t3_the_offer_is_consumed_by_an_accept() {
    let (p1, p2) = (p(1), p(2));
    let (state, events) = advance_to_draw(build_draw_state(p1, p2, false), p1, p2);
    let drawn = drawn_miracle_id(&events);
    let (state, _) = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: drawn,
            reveal: true,
        },
    )
    .unwrap();
    assert_eq!(miracle_triggers(&state), 1);
    assert_eq!(state.players().get(&p1).unwrap().miracle_pending, None);
    let err = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: drawn,
            reveal: true,
        },
    )
    .expect_err("the offer was already answered");
    assert!(matches!(err, GameStateError::InvalidCommand(_)));
}

#[test]
/// A DECLINE consumes the offer too.
///
/// The `!reveal` arm returns early, and a guard that returns early inherits the obligation
/// of the statements it skips (PB-DP8 / PB-DX50). The obligation here is clearing the
/// record; without it a player could decline, watch the turn develop, and reveal later.
fn t4_a_decline_consumes_the_offer() {
    let (p1, p2) = (p(1), p(2));
    let (state, events) = advance_to_draw(build_draw_state(p1, p2, false), p1, p2);
    let drawn = drawn_miracle_id(&events);
    let (state, _) = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: drawn,
            reveal: false,
        },
    )
    .expect("declining is legal");
    assert_eq!(state.players().get(&p1).unwrap().miracle_pending, None);
    assert_eq!(miracle_triggers(&state), 0);
    let err = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: drawn,
            reveal: true,
        },
    )
    .expect_err("CR 702.94a: the reveal happens as you draw it, not after declining it");
    assert!(matches!(err, GameStateError::InvalidCommand(_)));
}

#[test]
/// The record does not survive the turn (`reset_turn_state`, CR 121.1's sibling clear).
///
/// `cards_drawn_this_turn` resetting is exactly what would otherwise make a stale id
/// answerable again on a later turn, so the two are cleared in the same loop.
///
/// The *same-turn* half — that a subsequent non-eligible draw also clears the record,
/// because the assignment at the draw site is UNCONDITIONAL rather than an
/// `if let Some(..)` — is pinned structurally in
/// `crates/engine/tests/core/pb_dx18_trust_boundary_roster.rs`, because
/// `perform_one_draw` is `pub(crate)` and no public channel drives a second in-turn draw
/// on this fixture without also crossing the turn boundary. Stated rather than left as a
/// silent gap.
fn t5_the_record_does_not_survive_the_turn() {
    let (p1, p2) = (p(1), p(2));
    let (state, events) = advance_to_draw(build_draw_state(p1, p2, false), p1, p2);
    let drawn = drawn_miracle_id(&events);
    assert_eq!(
        state.players().get(&p1).unwrap().miracle_pending,
        Some(drawn)
    );
    let start_turn = state.turn().turn_number;
    let mut cur = state;
    for _ in 0..60 {
        if cur.turn().turn_number != start_turn {
            break;
        }
        let holder = cur_priority(&cur);
        let (s, _) = process_command(cur, Command::PassPriority { player: holder })
            .expect("passing priority advances the turn");
        cur = s;
    }
    assert_ne!(cur.turn().turn_number, start_turn, "the turn must advance");
    assert_eq!(
        cur.players().get(&p1).unwrap().miracle_pending,
        None,
        "reset_turn_state clears the just-drawn record alongside cards_drawn_this_turn"
    );
}

fn cur_priority(state: &GameState) -> PlayerId {
    state
        .turn()
        .priority_holder
        .expect("a priority holder is required to pass")
}
