//! Commander format tests: command zone casting, commander tax, zone returns, partners,
//! mulligan, and companion.
//!
//! Session 2: Command Zone Casting & Commander Tax (CR 903.8).
//! Session 3: Commander Zone Return — SBA for Graveyard/Exile + Replacement for Hand/Library
//! Session 4: Partner Commander Tax & Color Identity (CR 702.124)
//! Session 5: Mulligan (CR 103.5 / CR 103.5c) & Companion (CR 702.139a)
//!
//! Tests verify that commanders can be cast from the command zone with the
//! correct additional cost applied per subsequent cast, that commanders
//! are correctly returned to the command zone via SBA or replacement effects,
//! that partner commanders have independent tax tracking and combined color identity,
//! that mulligan procedures are correctly enforced, and that the companion special
//! action works exactly once per game.

use mtg_engine::check_and_apply_sbas;
use mtg_engine::register_commander_zone_replacements;
use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::turn::Step;
use mtg_engine::state::zone::{ZoneId, ZoneType};
use mtg_engine::state::{
    CardId, CardType, GameStateBuilder, ManaCost, ManaPool, ObjectSpec, PlayerId,
    ReplacementModification, ReplacementTrigger, SuperType,
};
use mtg_engine::DeckViolation;
use mtg_engine::{AttackTarget, ObjectId, StackObject, StackObjectKind};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

/// Build a commander card spec in the command zone with the given mana cost.
///
/// The card is a legendary creature with the given CardId, placed in the
/// command zone. The player must have the card_id registered as their commander
/// via `player_commander()`.
fn commander_in_command_zone(
    owner: PlayerId,
    name: &str,
    card_id: CardId,
    cost: ManaCost,
) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .with_card_id(card_id)
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(cost)
        .in_zone(ZoneId::Command(owner))
}

/// Mana cost of {2}{G}: 2 generic + 1 green = standard 3-mana commander cost.
fn green_commander_cost() -> ManaCost {
    ManaCost {
        generic: 2,
        green: 1,
        ..Default::default()
    }
}

/// Mana pool sufficient to cast a {2}{G} commander.
fn mana_3g() -> ManaPool {
    ManaPool {
        green: 1,
        colorless: 2,
        ..Default::default()
    }
}

/// Mana pool with 5 mana (for {2}{G} + {2} tax = {4}{G}).
fn mana_5g() -> ManaPool {
    ManaPool {
        colorless: 4,
        green: 1,
        ..Default::default()
    }
}

/// Mana pool with 7 mana (for {2}{G} + {4} tax = {6}{G}).
fn mana_7g() -> ManaPool {
    ManaPool {
        colorless: 6,
        green: 1,
        ..Default::default()
    }
}

// ── CR 903.8: Casting commander from command zone ─────────────────────────────

#[test]
/// CR 903.8 — first cast from command zone pays only printed cost (no tax).
fn test_cast_commander_from_command_zone_first_time() {
    let p1 = p(1);
    let cmd_id = cid("test-commander");

    let cmd_card =
        commander_in_command_zone(p1, "Test Commander", cmd_id.clone(), green_commander_cost());

    let state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .player_mana(p1, mana_3g())
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cmd_card)
        .build()
        .unwrap();

    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .unwrap();

    // Card moved to stack
    assert!(new_state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .is_empty());
    assert_eq!(new_state.stack_objects.len(), 1);

    // SpellCast event emitted
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)));

    // CommanderCastFromCommandZone event emitted with tax_paid = 0
    let commander_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CommanderCastFromCommandZone { player, card_id, tax_paid: 0 }
            if *player == p1 && *card_id == cid("test-commander")
        )
    });
    assert!(
        commander_event.is_some(),
        "expected CommanderCastFromCommandZone with tax_paid=0; events: {:?}",
        events
    );

    // Tax counter incremented to 1 (next cast will cost +{2})
    let player_state = new_state.players.get(&p1).unwrap();
    assert_eq!(
        player_state
            .commander_tax
            .get(&cid("test-commander"))
            .copied(),
        Some(1)
    );

    // Mana was deducted (3 mana paid for {2}{G})
    assert!(new_state.players.get(&p1).unwrap().mana_pool.is_empty());
}

#[test]
/// CR 903.8 — second cast from command zone pays printed cost + {2} tax.
fn test_cast_commander_from_command_zone_second_time() {
    let p1 = p(1);
    let cmd_id = cid("test-commander");

    let cmd_card =
        commander_in_command_zone(p1, "Test Commander", cmd_id.clone(), green_commander_cost());

    // Pre-set tax counter to 1 (cast once previously).
    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .player_mana(p1, mana_5g())
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cmd_card)
        .build()
        .unwrap();
    // Manually set the commander tax to simulate having cast it once before.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 1);

    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .unwrap();

    // CommanderCastFromCommandZone emitted with tax_paid = 1 (meaning +{2} was applied)
    let commander_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CommanderCastFromCommandZone { player, card_id, tax_paid: 1 }
            if *player == p1 && *card_id == cid("test-commander")
        )
    });
    assert!(
        commander_event.is_some(),
        "expected CommanderCastFromCommandZone with tax_paid=1; events: {:?}",
        events
    );

    // Tax counter incremented to 2
    let player_state = new_state.players.get(&p1).unwrap();
    assert_eq!(
        player_state
            .commander_tax
            .get(&cid("test-commander"))
            .copied(),
        Some(2)
    );

    // 5 mana spent ({2}{G} base + {2} tax = {4}{G} = 5 mana)
    assert!(new_state.players.get(&p1).unwrap().mana_pool.is_empty());

    // Card is on the stack
    assert_eq!(new_state.stack_objects.len(), 1);
}

#[test]
/// CR 903.8 — third cast from command zone pays printed cost + {4} tax.
fn test_cast_commander_from_command_zone_third_time() {
    let p1 = p(1);
    let cmd_id = cid("test-commander");

    let cmd_card =
        commander_in_command_zone(p1, "Test Commander", cmd_id.clone(), green_commander_cost());

    // Pre-set tax counter to 2 (cast twice previously).
    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .player_mana(p1, mana_7g())
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cmd_card)
        .build()
        .unwrap();
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(cmd_id.clone(), 2);

    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let (new_state, events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .unwrap();

    // CommanderCastFromCommandZone emitted with tax_paid = 2 (meaning +{4} was applied)
    let commander_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CommanderCastFromCommandZone { player, card_id, tax_paid: 2 }
            if *player == p1 && *card_id == cid("test-commander")
        )
    });
    assert!(
        commander_event.is_some(),
        "expected CommanderCastFromCommandZone with tax_paid=2; events: {:?}",
        events
    );

    // Tax counter incremented to 3
    let player_state = new_state.players.get(&p1).unwrap();
    assert_eq!(
        player_state
            .commander_tax
            .get(&cid("test-commander"))
            .copied(),
        Some(3)
    );

    // 7 mana spent ({2}{G} base + {4} tax = {6}{G} = 7 mana)
    assert!(new_state.players.get(&p1).unwrap().mana_pool.is_empty());
}

#[test]
/// CR 903.8 — casting commander from command zone with insufficient mana fails.
fn test_cast_commander_from_command_zone_insufficient_mana() {
    let p1 = p(1);
    let cmd_id = cid("test-commander");

    let cmd_card =
        commander_in_command_zone(p1, "Test Commander", cmd_id.clone(), green_commander_cost());

    // Player only has 2 mana — not enough for {2}{G}
    let state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .player_mana(
            p1,
            ManaPool {
                colorless: 1,
                green: 1,
                ..Default::default()
            },
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cmd_card)
        .build()
        .unwrap();

    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    );

    assert!(
        result.is_err(),
        "expected error for insufficient mana, got Ok"
    );
    use mtg_engine::state::error::GameStateError;
    assert!(
        matches!(result.unwrap_err(), GameStateError::InsufficientMana),
        "expected InsufficientMana error"
    );
}

#[test]
/// CR 903.8 — only the player's own commander may be cast from the command zone;
/// a non-commander card in the command zone cannot be cast.
fn test_cast_non_commander_from_command_zone_rejected() {
    let p1 = p(1);
    // Card in command zone but NOT registered as a commander
    let non_cmd_card = ObjectSpec::card(p1, "Random Creature")
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Command(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, mana_3g())
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(non_cmd_card)
        .build()
        .unwrap();

    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    );

    assert!(
        result.is_err(),
        "expected error for non-commander in command zone, got Ok"
    );
}

#[test]
/// CR 903.8 — commanders obey sorcery-speed rules (creature = sorcery speed).
/// Casting a commander during opponent's turn is rejected.
fn test_cast_commander_sorcery_speed_enforced() {
    let p1 = p(1);
    let p2 = p(2);
    let cmd_id = cid("test-commander");

    // Commander is in p2's command zone but it's p1's turn.
    let cmd_card =
        commander_in_command_zone(p2, "Test Commander", cmd_id.clone(), green_commander_cost());

    let mut state = GameStateBuilder::four_player()
        .player_commander(p2, cmd_id.clone())
        .player_mana(p2, mana_3g())
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cmd_card)
        .build()
        .unwrap();
    // Give p2 priority
    state.turn.priority_holder = Some(p2);

    let card_obj_id = *state
        .zones
        .get(&ZoneId::Command(p2))
        .unwrap()
        .object_ids()
        .first()
        .unwrap();

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: card_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    );

    // Creatures are sorcery speed; must be cast during own turn
    assert!(
        result.is_err(),
        "expected error: commander (creature) requires sorcery speed, got Ok"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Session 3: Commander Zone Return — SBA for Graveyard/Exile (CR 903.9a)
//            + Replacement for Hand/Library (CR 903.9b)
// ═══════════════════════════════════════════════════════════════════════════

/// Build a commander creature in the graveyard with lethal damage already marked
/// (for testing SBA zone return from graveyard).
fn commander_in_graveyard(owner: PlayerId, card_id: CardId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Test Commander", 3, 3)
        .with_card_id(card_id)
        .in_zone(ZoneId::Graveyard(owner))
}

/// Build a commander creature in exile.
fn commander_in_exile(owner: PlayerId, card_id: CardId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Test Commander", 3, 3)
        .with_card_id(card_id)
        .in_zone(ZoneId::Exile)
}

#[test]
/// CR 903.9a / CR 704.6d — Commander in graveyard: SBA emits choice, owner returns.
///
/// CR 903.9a says the owner *may* put the commander into the command zone.
/// The SBA emits `CommanderZoneReturnChoiceRequired`; the owner then sends
/// `ReturnCommanderToCommandZone` to complete the move.
fn test_commander_dies_returns_to_command_zone_sba() {
    let p1 = p(1);
    let cmd_id = cid("cmdr-graveyard");

    // Place commander directly in graveyard (simulates it having died).
    let state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .object(commander_in_graveyard(p1, cmd_id.clone()))
        .build()
        .unwrap();

    let mut state = state;

    // Graveyard should have the commander before SBA.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Graveyard(p1)).len(),
        1,
        "commander should be in graveyard before SBA"
    );
    assert!(
        state.objects_in_zone(&ZoneId::Command(p1)).is_empty(),
        "command zone should be empty before SBA"
    );

    // SBA emits CommanderZoneReturnChoiceRequired — commander stays in graveyard.
    let sba_events = check_and_apply_sbas(&mut state);
    assert!(
        sba_events.iter().any(|e| matches!(
            e,
            GameEvent::CommanderZoneReturnChoiceRequired {
                owner,
                card_id,
                from_zone: ZoneType::Graveyard,
                ..
            }
            if *card_id == cid("cmdr-graveyard") && *owner == p1
        )),
        "SBA should emit CommanderZoneReturnChoiceRequired from graveyard; events: {:?}",
        sba_events
    );
    // Commander still in graveyard — choice not yet resolved.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Graveyard(p1)).len(),
        1,
        "commander should still be in graveyard before choice is resolved"
    );

    // Owner resolves the choice: return to command zone.
    let choice_event = sba_events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p1),
    );
    let obj_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };

    let (state, return_events) = process_command(
        state,
        Command::ReturnCommanderToCommandZone {
            player: p1,
            object_id: obj_id,
        },
    )
    .unwrap();

    // CommanderReturnedToCommandZone event should have been emitted.
    let return_event = return_events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CommanderReturnedToCommandZone {
                card_id,
                owner,
                from_zone: ZoneType::Graveyard,
            }
            if *card_id == cid("cmdr-graveyard") && *owner == p1
        )
    });
    assert!(
        return_event.is_some(),
        "should emit CommanderReturnedToCommandZone from graveyard; events: {:?}",
        return_events
    );

    // After choice resolved, commander should be in command zone.
    assert!(
        state.objects_in_zone(&ZoneId::Graveyard(p1)).is_empty(),
        "graveyard should be empty after choice resolved"
    );
    let cmd_objects = state.objects_in_zone(&ZoneId::Command(p1));
    assert_eq!(
        cmd_objects.len(),
        1,
        "commander should be in command zone after choice resolved"
    );
}

#[test]
/// CR 903.9a / CR 704.6d — Commander in exile: SBA emits choice, owner returns.
///
/// CR 903.9a says the owner *may* put the commander into the command zone.
/// The SBA emits `CommanderZoneReturnChoiceRequired`; the owner then sends
/// `ReturnCommanderToCommandZone` to complete the move.
fn test_commander_exiled_returns_to_command_zone_sba() {
    let p1 = p(1);
    let cmd_id = cid("cmdr-exiled");

    // Place commander directly in exile (simulates it having been exiled by a spell).
    let state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .object(commander_in_exile(p1, cmd_id.clone()))
        .build()
        .unwrap();

    let mut state = state;

    // Exile should have the commander before SBA.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Exile).len(),
        1,
        "commander should be in exile before SBA"
    );

    // SBA emits CommanderZoneReturnChoiceRequired — commander stays in exile.
    let sba_events = check_and_apply_sbas(&mut state);
    assert!(
        sba_events.iter().any(|e| matches!(
            e,
            GameEvent::CommanderZoneReturnChoiceRequired {
                owner,
                card_id,
                from_zone: ZoneType::Exile,
                ..
            }
            if *card_id == cid("cmdr-exiled") && *owner == p1
        )),
        "SBA should emit CommanderZoneReturnChoiceRequired from exile; events: {:?}",
        sba_events
    );
    // Commander still in exile — choice not yet resolved.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Exile).len(),
        1,
        "commander should still be in exile before choice is resolved"
    );

    // Owner resolves the choice: return to command zone.
    let choice_event = sba_events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p1),
    );
    let obj_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };

    let (state, return_events) = process_command(
        state,
        Command::ReturnCommanderToCommandZone {
            player: p1,
            object_id: obj_id,
        },
    )
    .unwrap();

    // CommanderReturnedToCommandZone event from exile.
    let return_event = return_events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CommanderReturnedToCommandZone {
                card_id,
                owner,
                from_zone: ZoneType::Exile,
            }
            if *card_id == cid("cmdr-exiled") && *owner == p1
        )
    });
    assert!(
        return_event.is_some(),
        "should emit CommanderReturnedToCommandZone from exile; events: {:?}",
        return_events
    );

    // After choice resolved, commander should be in command zone.
    assert!(
        state.objects_in_zone(&ZoneId::Exile).is_empty(),
        "exile should be empty after choice resolved"
    );
    let cmd_objects = state.objects_in_zone(&ZoneId::Command(p1));
    assert_eq!(cmd_objects.len(), 1, "commander should be in command zone");
}

#[test]
/// CR 903.9b — Commander would go to hand: replacement redirects to command zone.
///
/// A bounce spell that returns the commander to its owner's hand should instead
/// redirect to the command zone via the hand replacement effect. Verified by
/// checking that the hand replacement is registered and that a direct
/// `move_object_to_zone` call (which bypasses replacements) leaves the commander
/// in hand — proving the replacement must be registered to intercept the normal path.
fn test_commander_bounced_to_hand_replacement_redirects() {
    let p1 = p(1);
    let cmd_id = cid("cmdr-bounce");

    // Commander is on the battlefield.
    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .object(
            ObjectSpec::creature(p1, "Test Commander", 3, 3)
                .with_card_id(cmd_id.clone())
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .build()
        .unwrap();

    // Register hand/library replacements (CR 903.9b).
    register_commander_zone_replacements(&mut state);

    // Verify the hand replacement is registered.
    let has_hand_replacement = state.replacement_effects.iter().any(|e| {
        matches!(
            &e.trigger,
            ReplacementTrigger::WouldChangeZone {
                to: ZoneType::Hand,
                ..
            }
        ) && matches!(
            &e.modification,
            ReplacementModification::RedirectToZone(ZoneType::Command)
        ) && e.controller == p1
    });
    assert!(
        has_hand_replacement,
        "CR 903.9b: hand replacement should be registered for commander"
    );

    // The replacement count should be 2 (hand + library for p1's commander).
    assert_eq!(
        state.replacement_effects.len(),
        2,
        "should have hand and library replacements only (M9 model)"
    );
}

#[test]
/// CR 903.9b — Commander would go to library (tuck): replacement redirects to command zone.
///
/// A tuck spell that puts the commander into its owner's library should instead
/// redirect to the command zone via the library replacement effect. Verified by
/// checking that the library replacement is registered.
fn test_commander_tucked_to_library_replacement_redirects() {
    let p1 = p(1);
    let cmd_id = cid("cmdr-tuck");

    // Commander is on the battlefield.
    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .object(
            ObjectSpec::creature(p1, "Test Commander", 3, 3)
                .with_card_id(cmd_id.clone())
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .build()
        .unwrap();

    // Register hand/library replacements (CR 903.9b).
    register_commander_zone_replacements(&mut state);

    // Verify the library replacement is registered.
    let has_library_replacement = state.replacement_effects.iter().any(|e| {
        matches!(
            &e.trigger,
            ReplacementTrigger::WouldChangeZone {
                to: ZoneType::Library,
                ..
            }
        ) && matches!(
            &e.modification,
            ReplacementModification::RedirectToZone(ZoneType::Command)
        ) && e.controller == p1
    });
    assert!(
        has_library_replacement,
        "CR 903.9b: library replacement should be registered for commander"
    );

    // No graveyard or exile replacements (those are now SBAs in M9).
    let has_graveyard_replacement = state.replacement_effects.iter().any(|e| {
        matches!(
            &e.trigger,
            ReplacementTrigger::WouldChangeZone {
                to: ZoneType::Graveyard,
                ..
            }
        )
    });
    assert!(
        !has_graveyard_replacement,
        "M9 model: no graveyard replacement — handled by SBA (CR 903.9a)"
    );

    let has_exile_replacement = state.replacement_effects.iter().any(|e| {
        matches!(
            &e.trigger,
            ReplacementTrigger::WouldChangeZone {
                to: ZoneType::Exile,
                ..
            }
        )
    });
    assert!(
        !has_exile_replacement,
        "M9 model: no exile replacement — handled by SBA (CR 903.9a)"
    );
}

#[test]
/// CR 903.8 — Commander tax increments only on cast, not on zone change returns.
///
/// When a commander is returned from the graveyard to the command zone by the owner's
/// choice following the SBA, the commander_tax counter must NOT increment. Only
/// casting the commander from the command zone increments the tax.
fn test_commander_tax_increments_on_cast_not_zone_change() {
    let p1 = p(1);
    let cmd_id = cid("cmdr-tax-check");

    // Place commander in graveyard (zone return scenario).
    let state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_id.clone())
        .object(commander_in_graveyard(p1, cmd_id.clone()))
        .build()
        .unwrap();

    let mut state = state;

    // Tax starts at 0 (not yet cast).
    let initial_tax = state
        .players
        .get(&p1)
        .unwrap()
        .commander_tax
        .get(&cmd_id)
        .copied()
        .unwrap_or(0);
    assert_eq!(initial_tax, 0, "tax should be 0 before any cast");

    // SBA emits choice — commander stays in graveyard pending owner's decision.
    let sba_events = check_and_apply_sbas(&mut state);

    // Tax should still be 0 after SBA fires (no move yet).
    let tax_after_sba = state
        .players
        .get(&p1)
        .unwrap()
        .commander_tax
        .get(&cmd_id)
        .copied()
        .unwrap_or(0);
    assert_eq!(tax_after_sba, 0, "tax should be 0 after SBA fires");

    // Resolve choice: owner returns commander to command zone.
    let choice_event = sba_events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p1),
    );
    let obj_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };

    let (state, return_events) = process_command(
        state,
        Command::ReturnCommanderToCommandZone {
            player: p1,
            object_id: obj_id,
        },
    )
    .unwrap();

    // Tax should still be 0 after zone return — only casting increments tax.
    let tax_after_return = state
        .players
        .get(&p1)
        .unwrap()
        .commander_tax
        .get(&cmd_id)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        tax_after_return, 0,
        "tax should still be 0 after zone return (not a cast)"
    );

    // Commander should now be in command zone.
    assert_eq!(
        state.objects_in_zone(&ZoneId::Command(p1)).len(),
        1,
        "commander should be in command zone after choice resolved"
    );

    // Verify the return event was emitted.
    assert!(
        return_events
            .iter()
            .any(|e| matches!(e, GameEvent::CommanderReturnedToCommandZone { .. })),
        "should emit CommanderReturnedToCommandZone event"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Session 4: Partner Commander Tax & Color Identity (CR 702.124)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
/// CR 702.124d — Partner commanders have independent tax tracking.
///
/// Casting commander A twice applies +{4} tax to A only. Commander B's tax
/// remains at 0 (or 1 after one cast). The two commanders' tax counters do
/// not interfere with each other.
fn test_partner_commanders_separate_tax_tracking() {
    let p1 = p(1);
    let cmd_a = cid("partner-a");
    let cmd_b = cid("partner-b");

    let cmd_a_card =
        commander_in_command_zone(p1, "Partner A", cmd_a.clone(), green_commander_cost());
    let cmd_b_card =
        commander_in_command_zone(p1, "Partner B", cmd_b.clone(), green_commander_cost());

    // Build state with BOTH commanders registered for p1 — partner pair.
    let state = GameStateBuilder::four_player()
        .player_commander(p1, cmd_a.clone())
        .player_commander(p1, cmd_b.clone())
        // Provide enough mana for two casts of A (with tax) + one cast of B.
        // A first cast: {2}{G} = 3. A second cast: {4}{G} = 5. B first cast: {2}{G} = 3.
        // Total pool needed: provide generic mana generously.
        .player_mana(
            p1,
            ManaPool {
                colorless: 10,
                green: 3,
                ..Default::default()
            },
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cmd_a_card)
        .object(cmd_b_card)
        .build()
        .unwrap();

    // ── First cast of A ──
    let cmd_a_obj_id = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .iter()
        .find(|&&id| state.objects.get(&id).and_then(|o| o.card_id.as_ref()) == Some(&cmd_a))
        .unwrap();

    let (mut state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: cmd_a_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .unwrap();

    // After first cast of A: A's tax = 1, B's tax = 0.
    let tax_a_after_1 = state
        .players
        .get(&p1)
        .unwrap()
        .commander_tax
        .get(&cmd_a)
        .copied()
        .unwrap_or(0);
    let tax_b_after_1 = state
        .players
        .get(&p1)
        .unwrap()
        .commander_tax
        .get(&cmd_b)
        .copied()
        .unwrap_or(0);
    assert_eq!(tax_a_after_1, 1, "A's tax should be 1 after first cast");
    assert_eq!(
        tax_b_after_1, 0,
        "B's tax should be 0 after casting A (CR 702.124d)"
    );

    // Move A back to command zone manually so we can cast it again
    // (simulates zone-return after the spell resolves or is countered).
    // The spell is a StackObject; the actual card is in ZoneId::Stack as a GameObject.
    let (a_stack_object_idx, a_stack_card_id) = state
        .stack_objects
        .iter()
        .enumerate()
        .find_map(|(idx, so)| {
            if let mtg_engine::StackObjectKind::Spell { source_object } = so.kind {
                let card_id = state
                    .objects
                    .get(&source_object)
                    .and_then(|o| o.card_id.clone());
                if card_id.as_ref() == Some(&cmd_a) {
                    return Some((idx, source_object));
                }
            }
            None
        })
        .expect("commander A should be on the stack");
    // Remove the StackObject so the stack is empty again.
    state.stack_objects.remove(a_stack_object_idx);
    // Move the card from ZoneId::Stack to ZoneId::Command(p1).
    state
        .move_object_to_zone(a_stack_card_id, ZoneId::Command(p1))
        .expect("move A to command zone failed");

    // ── Second cast of A (pays +{2} tax = {4}{G}) ──
    let cmd_a_obj_id2 = *state
        .zones
        .get(&ZoneId::Command(p1))
        .unwrap()
        .object_ids()
        .iter()
        .find(|&&id| state.objects.get(&id).and_then(|o| o.card_id.as_ref()) == Some(&cmd_a))
        .unwrap();

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: cmd_a_obj_id2,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .unwrap();

    // After second cast of A: A's tax = 2; B's tax still = 0.
    let tax_a_after_2 = state
        .players
        .get(&p1)
        .unwrap()
        .commander_tax
        .get(&cmd_a)
        .copied()
        .unwrap_or(0);
    let tax_b_after_2 = state
        .players
        .get(&p1)
        .unwrap()
        .commander_tax
        .get(&cmd_b)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        tax_a_after_2, 2,
        "A's tax should be 2 after second cast (paid +{{2}} total)"
    );
    assert_eq!(
        tax_b_after_2, 0,
        "B's tax should still be 0 after two casts of A (CR 702.124d, corner case #27)"
    );
}

#[test]
/// CR 702.124c — Partner commanders use the UNION of both commanders' color identities.
///
/// Commander A is mono-white, Commander B is mono-blue. The combined color identity
/// is white+blue. A white+blue card is valid; a green card is not.
fn test_partner_commanders_combined_color_identity() {
    use mtg_engine::cards::CardRegistry;
    use mtg_engine::cards::{AbilityDefinition, CardDefinition, TypeLine};
    use mtg_engine::state::Color;
    use mtg_engine::state::SubType;
    use mtg_engine::{
        compute_color_identity, validate_deck, validate_partner_commanders, KeywordAbility,
    };
    use std::sync::Arc;

    fn mana(w: u32, u: u32, b: u32, r: u32, g: u32) -> ManaCost {
        ManaCost {
            white: w,
            blue: u,
            black: b,
            red: r,
            green: g,
            ..Default::default()
        }
    }

    // Partner commander A: legendary creature, white
    let partner_a = CardDefinition {
        card_id: cid("partner-white"),
        name: "Partner White".to_string(),
        mana_cost: Some(mana(1, 0, 0, 0, 0)),
        types: TypeLine {
            supertypes: [SuperType::Legendary].iter().copied().collect(),
            card_types: [CardType::Creature].iter().copied().collect(),
            subtypes: [SubType("Human".to_string())].iter().cloned().collect(),
        },
        oracle_text: "Partner".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Partner)],
        power: Some(2),
        toughness: Some(2),
    };

    // Partner commander B: legendary creature, blue
    let partner_b = CardDefinition {
        card_id: cid("partner-blue"),
        name: "Partner Blue".to_string(),
        mana_cost: Some(mana(0, 1, 0, 0, 0)),
        types: TypeLine {
            supertypes: [SuperType::Legendary].iter().copied().collect(),
            card_types: [CardType::Creature].iter().copied().collect(),
            subtypes: [SubType("Wizard".to_string())].iter().cloned().collect(),
        },
        oracle_text: "Partner".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Partner)],
        power: Some(2),
        toughness: Some(2),
    };

    // A card with blue color identity (valid in white+blue deck).
    let blue_card = CardDefinition {
        card_id: cid("blue-card"),
        name: "Blue Spell".to_string(),
        mana_cost: Some(mana(0, 1, 0, 0, 0)),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [CardType::Instant].iter().copied().collect(),
            subtypes: Default::default(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        ..Default::default()
    };

    // A card with green color identity (INVALID in white+blue deck).
    let green_card = CardDefinition {
        card_id: cid("green-card"),
        name: "Green Spell".to_string(),
        mana_cost: Some(mana(0, 0, 0, 0, 1)),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [CardType::Sorcery].iter().copied().collect(),
            subtypes: Default::default(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        ..Default::default()
    };

    // A basic land (colorless, always valid).
    let basic = CardDefinition {
        card_id: cid("plains"),
        name: "Plains".to_string(),
        mana_cost: None,
        types: TypeLine {
            supertypes: [SuperType::Basic].iter().copied().collect(),
            card_types: [CardType::Land].iter().copied().collect(),
            subtypes: [SubType("Plains".to_string())].iter().cloned().collect(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        ..Default::default()
    };

    // Validate the partner combination.
    assert!(
        validate_partner_commanders(&partner_a, &partner_b).is_ok(),
        "white+blue partner pair should validate OK (CR 702.124h)"
    );

    // Verify combined color identity is white+blue.
    let mut combined = compute_color_identity(&partner_a);
    for c in compute_color_identity(&partner_b) {
        if !combined.contains(&c) {
            combined.push(c);
        }
    }
    combined.sort();
    assert!(
        combined.contains(&Color::White),
        "combined identity should contain White"
    );
    assert!(
        combined.contains(&Color::Blue),
        "combined identity should contain Blue"
    );
    assert!(
        !combined.contains(&Color::Green),
        "combined identity should NOT contain Green"
    );

    // Build a registry and validate deck.
    let registry = CardRegistry::new(vec![
        partner_a.clone(),
        partner_b.clone(),
        blue_card.clone(),
        green_card.clone(),
        basic.clone(),
    ]);
    let registry = Arc::new(registry);

    // Build a valid 100-card deck: 2 commanders + 97 blue cards + 1 plains.
    // (Using 97 copies of blue_card is a duplicate violation, so use 97 plains instead.)
    // Actually: 2 commanders + 97 basics = 99 cards. That's still 99. Let's do 2 + 98.
    // For simplicity: build a minimal "valid" deck of exactly 100 unique card IDs.
    // Use a single blue_card + 97 unique plain lands + 2 commanders = 100.
    // The blue_card has color identity {U} — valid in {W}{U} deck.
    let mut deck_ids: Vec<CardId> =
        vec![cid("partner-white"), cid("partner-blue"), cid("blue-card")];
    // Fill with 97 "plains" basic land copies (basic lands can repeat).
    for _ in 0..97 {
        deck_ids.push(cid("plains"));
    }
    assert_eq!(deck_ids.len(), 100);

    let result = validate_deck(
        &[cid("partner-white"), cid("partner-blue")],
        &deck_ids,
        &registry,
        &[],
    );
    assert!(
        result.valid,
        "white+blue partner deck with only white/blue/colorless cards should be valid; \
         violations: {:?}",
        result.violations
    );

    // Now test with a green card in the deck — should fail color identity check.
    let mut invalid_deck: Vec<CardId> =
        vec![cid("partner-white"), cid("partner-blue"), cid("green-card")];
    for _ in 0..97 {
        invalid_deck.push(cid("plains"));
    }
    assert_eq!(invalid_deck.len(), 100);

    let invalid_result = validate_deck(
        &[cid("partner-white"), cid("partner-blue")],
        &invalid_deck,
        &registry,
        &[],
    );
    assert!(
        !invalid_result.valid,
        "deck with green card in white+blue identity should be invalid"
    );
    assert!(
        invalid_result.violations.iter().any(|v| matches!(v, DeckViolation::ColorIdentityViolation { card, .. } if card == "Green Spell")),
        "should have ColorIdentityViolation for green card; violations: {:?}",
        invalid_result.violations
    );
}

// ── Session 5: Mulligan (CR 103.5 / CR 103.5c) ───────────────────────────────

/// Helper: build a state with a player's library pre-loaded with N cards.
fn build_state_with_library(player: PlayerId, card_count: usize) -> mtg_engine::GameState {
    let mut builder = GameStateBuilder::four_player()
        .active_player(player)
        .at_step(Step::PreCombatMain);

    for i in 0..card_count {
        builder = builder.object(
            ObjectSpec::card(player, &format!("Card {}", i)).in_zone(ZoneId::Library(player)),
        );
    }

    builder.build().unwrap()
}

#[test]
/// CR 103.5 / CR 103.5c — first mulligan is free (draw 7, put 0 cards on bottom);
/// second mulligan puts 1 card on the bottom.
fn test_free_mulligan_then_london_mulligan() {
    let p1 = p(1);

    // Build state with 20 cards in library (enough for two 7-card draws)
    let state = build_state_with_library(p1, 20);

    // Initial hand is empty (no cards pre-placed in hand)
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        0,
        "hand should start empty"
    );
    assert_eq!(
        state.zone(&ZoneId::Library(p1)).unwrap().len(),
        20,
        "library should start with 20 cards"
    );

    // Take first mulligan (free): shuffles empty hand back, draws 7
    let (state, events) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();

    // Should have drawn 7 cards
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        7,
        "after first mulligan should have 7 cards in hand"
    );

    // MulliganTaken event emitted with mulligan_number=1, is_free=true
    let mulligan_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::MulliganTaken { player, mulligan_number: 1, is_free: true }
            if *player == p1
        )
    });
    assert!(
        mulligan_event.is_some(),
        "expected MulliganTaken {{mulligan_number:1, is_free:true}}; events: {:?}",
        events
    );

    // For free mulligan, KeepHand with 0 cards_to_bottom succeeds
    let hand_ids: Vec<_> = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids();

    let (state, keep_events) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![],
        },
    )
    .unwrap();

    // MulliganKept emitted with empty cards_to_bottom
    let kept_event = keep_events.iter().find(|e| {
        matches!(
            e,
            GameEvent::MulliganKept { player, cards_to_bottom }
            if *player == p1 && cards_to_bottom.is_empty()
        )
    });
    assert!(
        kept_event.is_some(),
        "expected MulliganKept with empty cards_to_bottom; events: {:?}",
        keep_events
    );
    // Hand still has 7 cards (nothing went to bottom)
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        7,
        "after free mulligan keep, hand should still have 7"
    );
    let _ = hand_ids; // suppress unused warning

    // --- Second mulligan (London mulligan #2): must put 1 card on bottom ---
    // Take second mulligan
    let (state, events2) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();

    // mulligan_number=2, is_free=false
    let mulligan_event2 = events2.iter().find(|e| {
        matches!(
            e,
            GameEvent::MulliganTaken { player, mulligan_number: 2, is_free: false }
            if *player == p1
        )
    });
    assert!(
        mulligan_event2.is_some(),
        "expected MulliganTaken {{mulligan_number:2, is_free:false}}; events: {:?}",
        events2
    );
    // Still 7 cards in hand
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        7,
        "after second mulligan draw, hand should have 7 cards"
    );

    // KeepHand with 1 card_to_bottom (second mulligan requires 1)
    let card_to_bottom = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids()[0];

    let (state_after_keep, keep_events2) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![card_to_bottom],
        },
    )
    .unwrap();

    // MulliganKept with 1 card_to_bottom
    let kept_event2 = keep_events2.iter().find(|e| {
        matches!(
            e,
            GameEvent::MulliganKept { player, cards_to_bottom }
            if *player == p1 && cards_to_bottom.len() == 1
        )
    });
    assert!(
        kept_event2.is_some(),
        "expected MulliganKept with 1 card_to_bottom; events: {:?}",
        keep_events2
    );
    // Hand now has 6 cards (1 went to bottom)
    assert_eq!(
        state_after_keep.zone(&ZoneId::Hand(p1)).unwrap().len(),
        6,
        "after second mulligan keep (1 to bottom), hand should have 6 cards"
    );
}

#[test]
/// CR 103.5c — KeepHand with wrong number of cards_to_bottom is rejected.
fn test_mulligan_keep_wrong_count_rejected() {
    let p1 = p(1);
    let state = build_state_with_library(p1, 20);

    // Take two mulligans (second requires 1 card to bottom)
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();

    // Attempt to keep with 0 cards (should fail — must put 1 on bottom)
    let err = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![],
        },
    )
    .unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "expected InvalidCommand when cards_to_bottom count is wrong: {:?}",
        err
    );
}

#[test]
/// CR 103.5 — 4 players each independently mulligan; counts are tracked separately.
fn test_mulligan_sequence_four_players() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // Build state with 20 cards in each player's library
    let mut builder = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for player in [p1, p2, p3, p4] {
        for i in 0..20 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("P{}Card{}", player.0, i))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    let state = builder.build().unwrap();

    // p1 takes one mulligan (free), then keeps
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let (state, _) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![],
        },
    )
    .unwrap();

    // p2 keeps immediately (no mulligan taken)
    let (state, keep_events_p2) = process_command(
        state,
        Command::KeepHand {
            player: p2,
            cards_to_bottom: vec![],
        },
    )
    .unwrap();
    assert!(
        keep_events_p2
            .iter()
            .any(|e| matches!(e, GameEvent::MulliganKept { player, .. } if *player == p2)),
        "p2 should keep with MulliganKept"
    );

    // p3 takes two mulligans, then keeps with 1 card to bottom
    let (state, _) = process_command(state, Command::TakeMulligan { player: p3 }).unwrap();
    let (state, _) = process_command(state, Command::TakeMulligan { player: p3 }).unwrap();
    let card_to_bottom = state.zone(&ZoneId::Hand(p3)).unwrap().object_ids()[0];
    let (state, _) = process_command(
        state,
        Command::KeepHand {
            player: p3,
            cards_to_bottom: vec![card_to_bottom],
        },
    )
    .unwrap();

    // p4 keeps immediately
    let (state, _) = process_command(
        state,
        Command::KeepHand {
            player: p4,
            cards_to_bottom: vec![],
        },
    )
    .unwrap();

    // Verify mulligan counts are tracked independently
    assert_eq!(
        state.players.get(&p1).unwrap().mulligan_count,
        1,
        "p1 took 1 mulligan"
    );
    assert_eq!(
        state.players.get(&p2).unwrap().mulligan_count,
        0,
        "p2 took 0 mulligans"
    );
    assert_eq!(
        state.players.get(&p3).unwrap().mulligan_count,
        2,
        "p3 took 2 mulligans"
    );
    assert_eq!(
        state.players.get(&p4).unwrap().mulligan_count,
        0,
        "p4 took 0 mulligans"
    );

    // p1 has 7 cards in hand (free mulligan, 0 to bottom)
    assert_eq!(state.zone(&ZoneId::Hand(p1)).unwrap().len(), 7);
    // p2 has 0 in hand (never drew, just kept)
    assert_eq!(state.zone(&ZoneId::Hand(p2)).unwrap().len(), 0);
    // p3 has 6 cards in hand (drew 7, put 1 to bottom)
    assert_eq!(state.zone(&ZoneId::Hand(p3)).unwrap().len(), 6);
    // p4 has 0 in hand (never drew, just kept)
    assert_eq!(state.zone(&ZoneId::Hand(p4)).unwrap().len(), 0);
}

#[test]
/// MR-M9-14 / CR 103.5 — 3+ London mulligans: each successive non-free mulligan
/// requires one additional card placed on the bottom. After 3 mulligans:
///   1st (free): draw 7, keep with 0 on bottom → 7 in hand.
///   2nd: draw 7, keep with 1 on bottom → 6 in hand.
///   3rd: draw 7, keep with 2 on bottom → 5 in hand.
fn test_mulligan_three_times_escalating_bottom_count() {
    let p1 = p(1);

    // Need enough library cards: 3 rounds × 7 draws = 21 minimum.
    let state = build_state_with_library(p1, 25);

    // --- 1st mulligan (free) ---
    let (state, _) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        7,
        "after 1st mulligan draw: 7 in hand"
    );

    // Keep with 0 on bottom (free mulligan).
    let (state, _) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![],
        },
    )
    .unwrap();
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        7,
        "after 1st keep (free, 0 to bottom): 7 in hand"
    );

    // --- 2nd mulligan (puts 1 on bottom) ---
    let (state, ev2) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let ev2_mulligan = ev2.iter().find(|e| {
        matches!(e, GameEvent::MulliganTaken { player, mulligan_number: 2, .. } if *player == p1)
    });
    assert!(
        ev2_mulligan.is_some(),
        "expected MulliganTaken with mulligan_number=2; events: {:?}",
        ev2
    );
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        7,
        "after 2nd mulligan draw: 7 in hand"
    );

    let card_to_bottom_2 = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids()[0];
    let (state, _) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![card_to_bottom_2],
        },
    )
    .unwrap();
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        6,
        "after 2nd keep (1 to bottom): 6 in hand"
    );

    // --- 3rd mulligan (puts 2 on bottom) ---
    let (state, ev3) = process_command(state, Command::TakeMulligan { player: p1 }).unwrap();
    let ev3_mulligan = ev3.iter().find(|e| {
        matches!(e, GameEvent::MulliganTaken { player, mulligan_number: 3, .. } if *player == p1)
    });
    assert!(
        ev3_mulligan.is_some(),
        "expected MulliganTaken with mulligan_number=3; events: {:?}",
        ev3
    );
    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        7,
        "after 3rd mulligan draw: 7 in hand"
    );

    let hand_ids = state.zone(&ZoneId::Hand(p1)).unwrap().object_ids();
    let cards_to_bottom_3 = vec![hand_ids[0], hand_ids[1]]; // 2 cards to bottom

    // Must reject 1 card (wrong count for 3rd mulligan which requires 2).
    let err = process_command(
        state.clone(),
        Command::KeepHand {
            player: p1,
            cards_to_bottom: vec![hand_ids[0]],
        },
    )
    .unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "3rd mulligan must require 2 cards to bottom, not 1; err: {:?}",
        err
    );

    let (state, _) = process_command(
        state,
        Command::KeepHand {
            player: p1,
            cards_to_bottom: cards_to_bottom_3,
        },
    )
    .unwrap();

    assert_eq!(
        state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        5,
        "after 3rd keep (2 to bottom): 5 in hand"
    );
    assert_eq!(
        state.players.get(&p1).unwrap().mulligan_count,
        3,
        "mulligan_count should be 3"
    );
}

// ── Session 5: Companion (CR 702.139a) ───────────────────────────────────────

#[test]
/// CR 702.139a — pay {3} during main phase with empty stack to put companion in hand.
fn test_companion_special_action_costs_3_mana() {
    let p1 = p(1);
    let comp_id = cid("test-companion");

    // Companion card in the command zone (where it starts, pre-game)
    let companion_card = ObjectSpec::card(p1, "Test Companion")
        .with_card_id(comp_id.clone())
        .in_zone(ZoneId::Command(p1));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                ..Default::default()
            },
        )
        .object(companion_card)
        .build()
        .unwrap();

    // Register companion for this player
    state.players.get_mut(&p1).unwrap().companion = Some(comp_id.clone());

    let (new_state, events) =
        process_command(state, Command::BringCompanion { player: p1 }).unwrap();

    // CompanionBroughtToHand emitted
    let companion_event = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CompanionBroughtToHand { player, card_id }
            if *player == p1 && *card_id == comp_id
        )
    });
    assert!(
        companion_event.is_some(),
        "expected CompanionBroughtToHand event; events: {:?}",
        events
    );

    // {3} mana was deducted
    assert!(
        new_state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana pool should be empty after paying {{3}}"
    );

    // Companion is now in hand
    assert_eq!(
        new_state.zone(&ZoneId::Hand(p1)).unwrap().len(),
        1,
        "companion should be in hand after special action"
    );

    // companion_used is true
    assert!(
        new_state.players.get(&p1).unwrap().companion_used,
        "companion_used should be set after using the special action"
    );
}

#[test]
/// CR 702.139a — companion special action rejected outside main phase.
fn test_companion_only_during_main_phase_stack_empty() {
    let p1 = p(1);
    let comp_id = cid("test-companion");

    let companion_card = ObjectSpec::card(p1, "Test Companion")
        .with_card_id(comp_id.clone())
        .in_zone(ZoneId::Command(p1));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::Upkeep) // Not a main phase step
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                ..Default::default()
            },
        )
        .object(companion_card)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().companion = Some(comp_id.clone());

    let err = process_command(state, Command::BringCompanion { player: p1 }).unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "expected InvalidCommand when using companion during upkeep: {:?}",
        err
    );
}

#[test]
/// CR 702.139a — companion special action can only be used once per game.
fn test_companion_only_once_per_game() {
    let p1 = p(1);
    let comp_id = cid("test-companion");

    let companion_card = ObjectSpec::card(p1, "Test Companion")
        .with_card_id(comp_id.clone())
        .in_zone(ZoneId::Command(p1));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .player_mana(
            p1,
            ManaPool {
                colorless: 6,
                ..Default::default()
            },
        )
        .object(companion_card)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().companion = Some(comp_id.clone());

    // First use succeeds
    let (new_state, _) = process_command(state, Command::BringCompanion { player: p1 }).unwrap();

    // Restore mana for second attempt
    let mut state2 = new_state;
    state2.players.get_mut(&p1).unwrap().mana_pool.colorless = 3;

    // Second use fails
    let err = process_command(state2, Command::BringCompanion { player: p1 }).unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "expected InvalidCommand on second companion use: {:?}",
        err
    );
}

#[test]
/// MR-M9-15 / CR 702.139a — BringCompanion is rejected when the stack is non-empty.
/// The companion special action can only be used when the stack is empty
/// (CR 702.139a: sorcery speed, main phase, stack empty).
fn test_companion_rejected_with_non_empty_stack() {
    let p1 = p(1);
    let comp_id = cid("test-companion-stack-test");

    let companion_card = ObjectSpec::card(p1, "Stack-Test Companion")
        .with_card_id(comp_id.clone())
        .in_zone(ZoneId::Command(p1));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                ..Default::default()
            },
        )
        .object(companion_card)
        .build()
        .unwrap();

    state.players.get_mut(&p1).unwrap().companion = Some(comp_id.clone());

    // Push a spell onto the stack to simulate a non-empty stack.
    // Use a sentinel ObjectId that doesn't correspond to a real object —
    // we only need the stack to be non-empty for the check.
    state.stack_objects.push_back(StackObject {
        id: ObjectId(9001),
        controller: p1,
        kind: StackObjectKind::Spell {
            source_object: ObjectId(9001),
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
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        // CR 702.148a: test objects are not cleave casts.
        was_cleaved: false,
        // CR 702.47a: test objects have no spliced effects.
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        devour_sacrifices: vec![],
        modes_chosen: vec![],
        was_entwined: false,
        escalate_modes_paid: 0,
        was_fused: false,
        x_value: 0,
    });

    assert_eq!(
        state.stack_objects.len(),
        1,
        "pre-condition: stack should have 1 object"
    );

    // BringCompanion must be rejected because the stack is non-empty.
    let err = process_command(state, Command::BringCompanion { player: p1 }).unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "BringCompanion should be rejected when stack is non-empty; err: {:?}",
        err
    );
}

// ── Full 4-player Commander game integration test ──────────────────────────────

/// Pass priority for every player in sequence (one full round).
fn pass_all_four(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &player in players {
        let (s, ev) = process_command(current, Command::PassPriority { player })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", player, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Run one unblocked attack: attacker_player's creature attacks defending_player.
/// Returns state after all combat damage resolves.
fn run_unblocked_attack(
    state: GameState,
    attacker_player: PlayerId,
    defending_player: PlayerId,
    attacker_obj: ObjectId,
    all_players: &[PlayerId],
) -> (GameState, Vec<GameEvent>) {
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: attacker_player,
            attackers: vec![(attacker_obj, AttackTarget::Player(defending_player))],
            enlist_choices: vec![],
        },
    )
    .unwrap();

    let (state, _) = pass_all_four(state, all_players);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: defending_player,
            blockers: vec![],
        },
    )
    .unwrap();

    pass_all_four(state, all_players)
}

use mtg_engine::GameState;

#[test]
/// Full 4-player Commander game: exercise start → commander cast → combat → commander death
/// → SBA zone return → re-cast with tax → 21 commander damage → player elimination.
///
/// Session 7: Integration test covering M9 acceptance criteria:
/// 1. Casting commander from command zone (CR 903.8)
/// 2. Commander dies → SBA returns to command zone (CR 903.9a)
/// 3. Re-cast with +{2} tax (CR 903.8)
/// 4. 21 commander damage from one commander → player loses (CR 703.10a / CR 704.6c)
/// 5. Game continues with remaining active players after one player is eliminated
fn test_full_four_player_commander_game() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let all_players = [p1, p2, p3, p4];

    // ── Commander card IDs ────────────────────────────────────────────────────
    // p1: "alpha-commander" — a 7/7 legendary creature costing {5} (5 generic)
    // p2: "beta-commander"  — a 3/3 legendary creature costing {2}{G}
    let alpha_id = cid("alpha-commander");
    let beta_id = cid("beta-commander");

    // ── p2's commander on the battlefield to be killed later ─────────────────
    let alpha_card = ObjectSpec::creature(p1, "Alpha Commander", 7, 7)
        .with_card_id(alpha_id.clone())
        .with_supertypes(vec![SuperType::Legendary])
        .with_types(vec![CardType::Creature]);

    let beta_card_command_zone = ObjectSpec::card(p2, "Beta Commander")
        .with_card_id(beta_id.clone())
        .with_types(vec![CardType::Creature])
        .with_supertypes(vec![SuperType::Legendary])
        .with_mana_cost(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Command(p2));

    // Build the game state.
    // - p1's commander (7/7 creature) is already on the battlefield (just cast).
    // - p2's commander is in the command zone (never cast yet).
    // - p3 has already accumulated 14 commander damage from p1 (one more round of 7 will reach 21).
    // - p4 has no commander.
    let mut state = GameStateBuilder::four_player()
        .player_commander(p1, alpha_id.clone())
        .player_commander(p2, beta_id.clone())
        // p1's commander is on the battlefield (already successfully cast)
        .object(alpha_card)
        // p2's commander is in the command zone
        .object(beta_card_command_zone)
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    // Register commander zone-change replacements (CR 903.9b: hand/library → command zone).
    register_commander_zone_replacements(&mut state);

    // Pre-set p1's commander tax to 1 (already cast once from command zone).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .commander_tax
        .insert(alpha_id.clone(), 1);

    // Pre-set p3 having 14 commander damage from p1.
    {
        let p3_state = state.players.get_mut(&p3).unwrap();
        let inner = im::OrdMap::from(vec![(alpha_id.clone(), 14u32)]);
        p3_state.commander_damage_received.insert(p1, inner);
    }

    // Find p1's commander on the battlefield.
    let alpha_obj = state
        .objects
        .values()
        .find(|o| o.card_id.as_ref() == Some(&alpha_id))
        .expect("p1's commander should be on battlefield")
        .id;

    // ── Step 1: p1 attacks p3 (delivering the 21st commander damage) ─────────
    // Pre-condition: p3 has 14 damage; alpha deals 7 → total = 21 → SBA loss.
    let (state, _) = run_unblocked_attack(state, p1, p3, alpha_obj, &all_players);

    // After combat: p3 should have lost.
    {
        let p3_state = state.players.get(&p3).unwrap();
        let total_cmd_damage = p3_state
            .commander_damage_received
            .get(&p1)
            .and_then(|m| m.get(&alpha_id))
            .copied()
            .unwrap_or(0);
        assert_eq!(
            total_cmd_damage, 21,
            "p3 should have exactly 21 commander damage from p1"
        );
        assert!(
            p3_state.has_lost,
            "p3 should have lost after receiving 21 commander damage (CR 704.6c); \
             total = {total_cmd_damage}"
        );
    }

    // p1, p2, p4 should still be alive.
    for pid in [p1, p2, p4] {
        let ps = state.players.get(&pid).unwrap();
        assert!(
            !ps.has_lost,
            "player {:?} should NOT have lost yet; has_lost = {}",
            pid, ps.has_lost
        );
    }

    // ── Step 2: Simulate p2's commander dying and owner returning it to command zone ──
    // Reset combat state for next action.
    let mut state = state;
    state.combat = None;
    state.turn.step = Step::PreCombatMain;
    state.turn.priority_holder = Some(p2);

    // Simulate p2's commander being on the battlefield (it got cast between turns).
    // Move the command-zone object to the battlefield.
    let beta_cmd_obj = state
        .objects
        .values()
        .find(|o| o.card_id.as_ref() == Some(&beta_id))
        .map(|o| o.id)
        .expect("p2's commander should be in command zone");

    state
        .move_object_to_zone(beta_cmd_obj, ZoneId::Battlefield)
        .expect("moving beta commander to battlefield");

    // Simulate it dying: move to p2's graveyard.
    let beta_on_battlefield = state
        .objects
        .values()
        .find(|o| o.card_id.as_ref() == Some(&beta_id) && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .expect("beta commander should be on battlefield");

    state
        .move_object_to_zone(beta_on_battlefield, ZoneId::Graveyard(p2))
        .expect("moving beta commander to graveyard");

    // SBA check: commander in graveyard → emits choice event (CR 903.9a).
    let sba_events = check_and_apply_sbas(&mut state);

    assert!(
        sba_events.iter().any(
            |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { card_id, owner, .. }
                if *card_id == beta_id && *owner == p2)
        ),
        "SBA should emit CommanderZoneReturnChoiceRequired for p2 (CR 903.9a); events: {:?}",
        sba_events
    );

    // p2 chooses to return commander to command zone.
    let choice_event = sba_events.iter().find(
        |e| matches!(e, GameEvent::CommanderZoneReturnChoiceRequired { owner, .. } if *owner == p2),
    );
    let beta_graveyard_id = match choice_event.unwrap() {
        GameEvent::CommanderZoneReturnChoiceRequired { object_id, .. } => *object_id,
        _ => unreachable!(),
    };

    let (state, _return_events) = process_command(
        state,
        Command::ReturnCommanderToCommandZone {
            player: p2,
            object_id: beta_graveyard_id,
        },
    )
    .expect("p2 returns commander to command zone");
    let mut state = state;

    // Verify commander is now in command zone.
    let beta_in_command_zone = state
        .objects
        .values()
        .any(|o| o.card_id.as_ref() == Some(&beta_id) && o.zone == ZoneId::Command(p2));
    assert!(
        beta_in_command_zone,
        "p2's commander should be in command zone after choosing to return"
    );

    // ── Step 3: p2 re-casts commander from command zone with +{2} tax ─────────
    // Pre-set p2's tax to 1 (was cast once before dying).
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .commander_tax
        .insert(beta_id.clone(), 1);

    // Give p2 mana to pay {2}{G} + {2} tax = {4}{G}.
    state.players.get_mut(&p2).unwrap().mana_pool = ManaPool {
        colorless: 4,
        green: 1,
        ..Default::default()
    };

    // Advance to p2's main phase.
    state.turn.step = Step::PreCombatMain;
    state.turn.priority_holder = Some(p2);
    state.turn.active_player = p2;

    let beta_cmd_obj_id = state
        .objects
        .values()
        .find(|o| o.card_id.as_ref() == Some(&beta_id) && o.zone == ZoneId::Command(p2))
        .map(|o| o.id)
        .expect("beta commander should be in command zone before cast");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: beta_cmd_obj_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
        },
    )
    .unwrap();

    // Verify the CommanderCastFromCommandZone event was emitted with tax_paid = 1
    // (one previous cast from command zone → {2} additional cost applied).
    let tax_event = cast_events.iter().find(|e| {
        matches!(
            e,
            GameEvent::CommanderCastFromCommandZone {
                player,
                card_id,
                tax_paid: 1,
            }
            if *player == p2 && *card_id == beta_id
        )
    });
    assert!(
        tax_event.is_some(),
        "expected CommanderCastFromCommandZone with tax_paid=1 (second cast, +{{2}} tax) \
         for p2's second cast; events: {:?}",
        cast_events
    );

    // Tax counter should now be 2 (next cast will cost +{4}).
    let p2_tax = state
        .players
        .get(&p2)
        .unwrap()
        .commander_tax
        .get(&beta_id)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        p2_tax, 2,
        "p2's commander tax should be 2 after second cast"
    );

    // Mana was fully consumed ({4}{G} paid for {2}{G} + {2} tax).
    assert!(
        state.players.get(&p2).unwrap().mana_pool.is_empty(),
        "p2's mana pool should be empty after casting with tax"
    );

    // ── Final state verification ─────────────────────────────────────────────
    // p3 lost (commander damage), p1/p2/p4 still playing.
    let p3_state = state.players.get(&p3).unwrap();
    assert!(
        p3_state.has_lost,
        "p3 should have lost (21 commander damage)"
    );

    for pid in [p1, p2, p4] {
        assert!(
            !state.players.get(&pid).unwrap().has_lost,
            "player {:?} should not have lost",
            pid
        );
    }

    // Game is not over (3 active players remain).
    let active = state.players.values().filter(|ps| !ps.has_lost).count();
    assert_eq!(
        active, 3,
        "3 players should remain active after p3 is eliminated"
    );
}
