//! Miracle keyword ability tests (CR 702.94).
//!
//! Miracle is a keyword that represents two linked abilities (CR 702.94a):
//! 1. A static ability: "You may reveal this card from your hand as you draw it
//!    if it's the first card you've drawn this turn."
//! 2. A triggered ability: "When you reveal this card this way, you may cast it
//!    by paying [cost] rather than its mana cost."
//!
//! Key rules verified:
//! - Drawing a miracle card as the first draw emits MiracleRevealChoiceRequired.
//! - Drawing a miracle card as the second or later draw does NOT emit it.
//! - Drawing a non-miracle card emits no miracle event.
//! - ChooseMiracle with reveal:true puts a MiracleTrigger on the stack.
//! - ChooseMiracle with reveal:false results in normal hand (no trigger).
//! - CastSpell with cast_with_miracle:true uses the miracle cost (reduced mana).
//! - A sorcery cast via miracle ignores timing restrictions (any turn, any time).
//! - MiracleTrigger resolution does nothing; card stays in hand if not cast.
//! - Miracle cannot combine with flashback (CR 118.9a: one alternative cost).
//! - Mana value is based on printed cost, not miracle cost (CR 118.9c).
//! - Miracle works on any player's turn (e.g., during opponent's turn via draw effect).

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Effect, EffectAmount, PlayerTarget,
};
use mtg_engine::rules::turn_actions::draw_card;
use mtg_engine::state::types::AltCostKind;
use mtg_engine::state::CardType;
use mtg_engine::{
    process_command, CardId, CardRegistry, Command, GameEvent, GameStateBuilder, KeywordAbility,
    ManaCost, ObjectSpec, PlayerId, StackObject, StackObjectKind, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once.
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Terminus-style sorcery: Sorcery {4}{W}{W}, "Effect. Miracle {W}"
/// Mana cost: 4 generic + 2 white = MV 6.
/// Miracle cost: 1 white = MV 1.
fn terminus_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("terminus".to_string()),
        name: "Terminus".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            white: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Put all creatures on the bottom of their owners' libraries. Miracle {W}"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Miracle),
            AbilityDefinition::Miracle {
                cost: ManaCost {
                    white: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                // Simple sorcery effect: draw a card (placeholder for "put creatures on bottom")
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(0),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// A plain instant with no special abilities (for negative tests).
fn plain_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-instant".to_string()),
        name: "Plain Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A miracle instant (for instant-speed tests).
fn miracle_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("miracle-instant".to_string()),
        name: "Miracle Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Draw a card. Miracle {U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Miracle),
            AbilityDefinition::Miracle {
                cost: ManaCost {
                    blue: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(0),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

// ── Test 1: First draw emits MiracleRevealChoiceRequired ──────────────────────

#[test]
/// CR 702.94a — Drawing a miracle card as the first card this turn emits MiracleRevealChoiceRequired.
fn test_miracle_first_draw_emits_choice_event() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_keyword(KeywordAbility::Miracle),
        )
        .build()
        .unwrap();

    // Draw the miracle card (first draw of the turn).
    let events = draw_card(&mut state, p1).unwrap();

    // MiracleRevealChoiceRequired must be emitted.
    let miracle_event = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::MiracleRevealChoiceRequired { player, .. } if *player == p1
        )
    });
    assert!(
        miracle_event,
        "CR 702.94a: MiracleRevealChoiceRequired must be emitted when miracle card is the first draw"
    );

    // CardDrawn must also be emitted (draw happens first).
    let card_drawn = events
        .iter()
        .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1));
    assert!(
        card_drawn,
        "CR 702.94a: CardDrawn must be emitted before MiracleRevealChoiceRequired"
    );

    // Card must be in hand.
    let in_hand = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Terminus" && o.zone == ZoneId::Hand(p1));
    assert!(in_hand, "Terminus must be in hand after draw");
}

// ── Test 2: Second draw does NOT emit MiracleRevealChoiceRequired ────────────

#[test]
/// CR 702.94a (negative) — Drawing a miracle card as the second draw this turn does NOT emit the choice event.
fn test_miracle_second_draw_no_choice_event() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def(), plain_instant_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        // Library: Plain Instant on top (drawn first), Terminus underneath.
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_keyword(KeywordAbility::Miracle),
        )
        .object(
            ObjectSpec::card(p1, "Plain Instant")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("plain-instant".to_string())),
        )
        .build()
        .unwrap();

    // First draw: Plain Instant (non-miracle, no choice event).
    let events_first = draw_card(&mut state, p1).unwrap();
    let first_has_miracle = events_first
        .iter()
        .any(|e| matches!(e, GameEvent::MiracleRevealChoiceRequired { .. }));
    assert!(
        !first_has_miracle,
        "CR 702.94a: non-miracle card should not emit MiracleRevealChoiceRequired"
    );

    // Second draw: Terminus (miracle, but NOT the first draw).
    let events_second = draw_card(&mut state, p1).unwrap();
    let second_has_miracle = events_second
        .iter()
        .any(|e| matches!(e, GameEvent::MiracleRevealChoiceRequired { .. }));
    assert!(
        !second_has_miracle,
        "CR 702.94a: miracle card drawn as second draw must NOT emit MiracleRevealChoiceRequired"
    );
}

// ── Test 3: Non-miracle card emits no choice event ───────────────────────────

#[test]
/// CR 702.94a (negative) — Drawing a non-miracle card emits no miracle event.
fn test_miracle_non_miracle_card_no_choice() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Forest")
                .in_zone(ZoneId::Library(p1))
                .with_types(vec![CardType::Land]),
        )
        .build()
        .unwrap();

    let events = draw_card(&mut state, p1).unwrap();

    let has_miracle = events
        .iter()
        .any(|e| matches!(e, GameEvent::MiracleRevealChoiceRequired { .. }));
    assert!(
        !has_miracle,
        "CR 702.94a (negative): non-miracle card must not emit MiracleRevealChoiceRequired"
    );
}

// ── Test 4: Reveal puts MiracleTrigger on stack ───────────────────────────────

#[test]
/// CR 702.94a — Choosing to reveal a miracle card puts a MiracleTrigger on the stack.
fn test_miracle_reveal_puts_trigger_on_stack() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_keyword(KeywordAbility::Miracle),
        )
        .build()
        .unwrap();

    // Draw the miracle card.
    draw_card(&mut state, p1).unwrap();

    // Find the drawn card.
    let card_id = find_object(&state, "Terminus");

    // Send ChooseMiracle with reveal: true.
    let (state, events) = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: card_id,
            reveal: true,
        },
    )
    .unwrap();

    // A MiracleTrigger should be on the stack.
    let has_trigger = state
        .stack_objects
        .iter()
        .any(|so| matches!(&so.kind, mtg_engine::StackObjectKind::MiracleTrigger { .. }));
    assert!(
        has_trigger,
        "CR 702.94a: ChooseMiracle(reveal:true) must put a MiracleTrigger on the stack"
    );

    // AbilityTriggered event must have been emitted.
    let triggered = events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { .. }));
    assert!(
        triggered,
        "CR 702.94a: AbilityTriggered event must fire when miracle is revealed"
    );
}

// ── Test 5: Decline reveal results in no trigger ──────────────────────────────

#[test]
/// CR 702.94a — Choosing not to reveal a miracle card results in no trigger (normal hand).
fn test_miracle_decline_reveal_no_trigger() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_keyword(KeywordAbility::Miracle),
        )
        .build()
        .unwrap();

    // Draw the miracle card.
    draw_card(&mut state, p1).unwrap();

    let card_id = find_object(&state, "Terminus");

    // Send ChooseMiracle with reveal: false.
    let (state, _events) = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: card_id,
            reveal: false,
        },
    )
    .unwrap();

    // No MiracleTrigger on the stack.
    let has_trigger = state
        .stack_objects
        .iter()
        .any(|so| matches!(&so.kind, mtg_engine::StackObjectKind::MiracleTrigger { .. }));
    assert!(
        !has_trigger,
        "CR 702.94a: ChooseMiracle(reveal:false) must NOT put a MiracleTrigger on the stack"
    );

    // Card is still in hand.
    let in_hand = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Terminus" && o.zone == ZoneId::Hand(p1));
    assert!(
        in_hand,
        "Terminus must remain in hand when miracle reveal is declined"
    );
}

// ── Test 6: Cast for miracle cost ─────────────────────────────────────────────

#[test]
/// CR 702.94a — Casting a spell with cast_with_miracle:true uses the miracle cost.
fn test_miracle_cast_for_miracle_cost() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_keyword(KeywordAbility::Miracle),
        )
        .build()
        .unwrap();

    // Give player 1 white (miracle cost = {W}).
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.add(mtg_engine::ManaColor::White, 1);
    }

    // Draw the miracle card.
    draw_card(&mut state, p1).unwrap();

    let card_id = find_object(&state, "Terminus");

    // Reveal the miracle card.
    let (mut state, _) = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: card_id,
            reveal: true,
        },
    )
    .unwrap();

    // Give priority to p1 (engine reset it after trigger went on stack).
    state.turn.priority_holder = Some(p1);

    let card_in_hand_id = find_object(&state, "Terminus");

    // Cast the spell using the miracle cost.
    let (state_after, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_in_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Miracle),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
        },
    )
    .unwrap();

    // SpellCast event must have fired.
    let cast_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { .. }));
    assert!(
        cast_event,
        "CR 702.94a: SpellCast event must fire when miracle spell is cast"
    );

    // Terminus is now on the stack.
    let on_stack = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Terminus" && o.zone == ZoneId::Stack);
    assert!(
        on_stack,
        "CR 702.94a: Terminus must be on the stack after miracle cast"
    );

    // Mana was paid for miracle cost ({W} = 1), not full mana cost ({4}{W}{W} = 6).
    // Player's mana pool should be empty after paying {W}.
    let mana_paid_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::ManaCostPaid { player, .. } if *player == p1));
    assert!(
        mana_paid_event,
        "CR 702.94a: ManaCostPaid event must fire for miracle cost"
    );
}

// ── Test 7: Sorcery miracle ignores timing restrictions ──────────────────────

#[test]
/// CR 702.94a ruling — A sorcery with miracle can be cast at instant speed (non-active player's turn).
fn test_miracle_sorcery_ignores_timing() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def()]);

    // P1 draws during P2's turn (e.g., P2 plays something that makes P1 draw).
    // P1 should be able to cast Terminus via miracle even though it's P2's turn.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p2) // P2 is the active player
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_keyword(KeywordAbility::Miracle),
        )
        .build()
        .unwrap();

    // Give p1 mana for miracle cost.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.add(mtg_engine::ManaColor::White, 1);
    }

    // P1 draws as first draw of their turn (or via an effect on P2's turn).
    draw_card(&mut state, p1).unwrap();

    let card_id = find_object(&state, "Terminus");

    // P1 reveals the miracle card.
    let (mut state, _) = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: card_id,
            reveal: true,
        },
    )
    .unwrap();

    // Give priority to p1.
    state.turn.priority_holder = Some(p1);

    let card_in_hand_id = find_object(&state, "Terminus");

    // P1 casts Terminus via miracle on P2's turn — should succeed (timing override).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_in_hand_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Miracle),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.94a ruling: sorcery with miracle should be castable at instant speed (non-active player's turn), got: {:?}",
        result.err()
    );
}

// ── Test 8: MiracleTrigger resolves without cast ──────────────────────────────

#[test]
/// CR 702.94a — If the player passes without casting, the MiracleTrigger resolves and card stays in hand.
fn test_miracle_trigger_resolves_without_cast() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_keyword(KeywordAbility::Miracle),
        )
        .build()
        .unwrap();

    draw_card(&mut state, p1).unwrap();
    let card_id = find_object(&state, "Terminus");

    let (state, _) = process_command(
        state,
        Command::ChooseMiracle {
            player: p1,
            card: card_id,
            reveal: true,
        },
    )
    .unwrap();

    // Both players pass priority — the MiracleTrigger resolves.
    let (state, _all) = pass_all(state, &[p1, p2]);

    // Stack should be empty after trigger resolves.
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.94a: MiracleTrigger should be gone from stack after all pass"
    );

    // Terminus should still be in hand (player did not cast it).
    let in_hand = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Terminus" && o.zone == ZoneId::Hand(p1));
    assert!(
        in_hand,
        "CR 702.94a: Terminus must remain in hand if miracle trigger resolves without cast"
    );
}

// ── Test 9: Cannot combine miracle with flashback ────────────────────────────

#[test]
/// CR 118.9a — Miracle cannot be combined with flashback (only one alternative cost).
///
/// A card with both Miracle and Flashback is in the graveyard. The engine must reject
/// CastSpell with cast_with_miracle: true because miracle requires the card to be in hand
/// (CR 702.94a). This exercises the defense-in-depth exclusion check: casting.rs rejects
/// the command before the cost selection step, preventing any scenario where both
/// alternative costs could be applied simultaneously.
fn test_miracle_cannot_combine_with_flashback() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build a card with both Miracle and Flashback keywords (artificial, for test).
    let hybrid_def = CardDefinition {
        card_id: CardId("hybrid-miracle".to_string()),
        name: "Hybrid Miracle".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Miracle {1}. Flashback {2}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Miracle),
            AbilityDefinition::Miracle {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Flashback),
            AbilityDefinition::Flashback {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(0),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![hybrid_def]);

    // Place the card directly in the graveyard — it was previously cast and discarded.
    // From here, a player could legitimately cast it via flashback. We test that trying
    // to ALSO use cast_with_miracle: true from graveyard is rejected (CR 702.94a requires
    // the card to be in hand; CR 118.9a forbids combining two alternative costs).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Hybrid Miracle")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("hybrid-miracle".to_string()))
                .with_keyword(KeywordAbility::Miracle)
                .with_keyword(KeywordAbility::Flashback),
        )
        .build()
        .unwrap();

    let graveyard_card_id = find_object(&state, "Hybrid Miracle");

    // Manually place a MiracleTrigger on the stack referencing the graveyard card.
    // This simulates a malicious or buggy client that sends cast_with_miracle: true
    // for a card that is not in hand (but has a MiracleTrigger on the stack from
    // some earlier state manipulation). The engine must reject this (CR 702.94a).
    let trigger_id = state.next_object_id();
    state.stack_objects.push_back(StackObject {
        id: trigger_id,
        controller: p1,
        kind: StackObjectKind::MiracleTrigger {
            source_object: graveyard_card_id,
            revealed_card: graveyard_card_id,
            miracle_cost: ManaCost {
                generic: 1,
                ..Default::default()
            },
            owner: p1,
        },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
    });

    state.turn.priority_holder = Some(p1);

    // Attempt CastSpell with cast_with_miracle: true while the card is in the graveyard.
    // CR 702.94a: miracle requires the card to be in hand; card is in graveyard here.
    // CR 118.9a: only one alternative cost can be applied — combining miracle + flashback
    // (which would apply if the card were in graveyard with the Flashback keyword) is illegal.
    // The engine must reject this command before reaching the cost selection step.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: graveyard_card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Miracle),
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.94a / CR 118.9a: cast_with_miracle from graveyard must be rejected \
         (miracle requires card in hand; graveyard card would also trigger flashback)"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("miracle: card must be in your hand"),
        "Error must cite miracle hand requirement; got: {err_msg}"
    );
}

// ── Test 10: Mana value unchanged ────────────────────────────────────────────

#[test]
/// CR 118.9c — Mana value is based on the printed mana cost, not the miracle cost.
fn test_miracle_mana_value_unchanged() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_mana_cost(ManaCost {
                    generic: 4,
                    white: 2,
                    ..Default::default()
                })
                .with_keyword(KeywordAbility::Miracle),
        )
        .build()
        .unwrap();

    let terminus_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Terminus")
        .expect("Terminus not found");

    // Mana value should be 6 ({4}{W}{W}), not 1 ({W}).
    let mv = terminus_obj
        .characteristics
        .mana_cost
        .as_ref()
        .map(|mc| mc.mana_value())
        .unwrap_or(0);
    assert_eq!(
        mv, 6,
        "CR 118.9c: mana value must be 6 (printed cost {{4}}{{W}}{{W}}), not 1 (miracle cost {{W}})"
    );
}

// ── Test 11: Miracle works on opponent's turn ────────────────────────────────

#[test]
/// CR 702.94a ruling — Miracle works on any turn if it's the first card the player drew this turn.
fn test_miracle_opponent_turn_first_draw() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let registry = CardRegistry::new(vec![terminus_def()]);

    // P2 is active, P1 draws a miracle card (via some draw effect on P2's turn).
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, "Terminus")
                .in_zone(ZoneId::Library(p1))
                .with_card_id(CardId("terminus".to_string()))
                .with_keyword(KeywordAbility::Miracle),
        )
        .build()
        .unwrap();

    // P1 draws on P2's turn (first draw for P1 this turn).
    let events = draw_card(&mut state, p1).unwrap();

    // MiracleRevealChoiceRequired must fire even though P1 is not the active player.
    let miracle_event = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::MiracleRevealChoiceRequired { player, .. } if *player == p1
        )
    });
    assert!(
        miracle_event,
        "CR 702.94a ruling: miracle must work on any turn (not just active player's turn)"
    );
}
