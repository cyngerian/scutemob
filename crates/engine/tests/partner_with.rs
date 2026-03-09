//! Partner With keyword ability tests (CR 702.124j).
//!
//! "Partner with [name]" represents two abilities:
//! (1) Deck construction: allows the named pair as co-commanders, provided each
//!     has a 'partner with [name]' ability naming the other.
//! (2) ETB trigger: "When this permanent enters, target player may search their
//!     library for a card named [name], reveal it, put it into their hand, then
//!     shuffle."
//!
//! Key rules verified:
//! - Two commanders with matching PartnerWith names pass deck validation (CR 702.124j).
//! - Mismatched PartnerWith names fail deck validation (CR 702.124j).
//! - Mixing plain Partner + PartnerWith fails deck validation (CR 702.124f).
//! - A permanent with PartnerWith(name) generates a PartnerWithTrigger on ETB (CR 702.124j).
//! - The trigger resolution moves the named card from library to hand (CR 702.124j).
//! - If the named card is not in the library, no card moves and library is shuffled (CR 702.124j).
//! - The trigger fires even when the partner is on the battlefield (search finds nothing) (CR 702.124j).
//! - Permanents without PartnerWith do NOT generate a PartnerWithTrigger (negative test).

use mtg_engine::state::{CardType, SuperType};
use mtg_engine::validate_partner_commanders;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, Command, GameEvent,
    GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec, PlayerId, StackObject,
    StackObjectKind, Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_opt(state: &mtg_engine::GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
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

/// Keep passing priority for all players until the stack is empty.
fn pass_all_until_empty(
    mut state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    for _ in 0..50 {
        if state.stack_objects.is_empty() {
            break;
        }
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// Build a minimal `StackObject` for a PartnerWithTrigger.
fn make_partner_with_trigger_stack_obj(
    id: ObjectId,
    source_object: ObjectId,
    partner_name: &str,
    target_player: PlayerId,
    controller: PlayerId,
) -> StackObject {
    StackObject {
        id,
        controller,
        kind: StackObjectKind::KeywordTrigger {
            source_object,
            keyword: KeywordAbility::PartnerWith(partner_name.to_string()),
            data: mtg_engine::state::stack::TriggerData::ETBPartnerWith {
                partner_name: partner_name.to_string(),
                target_player,
            },
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
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
    }
}

// ── Card definitions ──────────────────────────────────────────────────────────

/// Pir, Imaginative Rascal — has PartnerWith("Toothy, Imaginary Friend")
fn pir_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("pir-imaginative-rascal".to_string()),
        name: "Pir, Imaginative Rascal".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Partner with Toothy, Imaginary Friend".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::PartnerWith(
            "Toothy, Imaginary Friend".to_string(),
        ))],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// Toothy, Imaginary Friend — has PartnerWith("Pir, Imaginative Rascal")
fn toothy_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("toothy-imaginary-friend".to_string()),
        name: "Toothy, Imaginary Friend".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Partner with Pir, Imaginative Rascal".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::PartnerWith(
            "Pir, Imaginative Rascal".to_string(),
        ))],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// A creature with PartnerWith("Wrong Name") — for mismatch tests.
fn wrong_partner_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("wrong-partner".to_string()),
        name: "Wrong Partner".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Partner with Toothy, Imaginary Friend".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::PartnerWith(
            "Toothy, Imaginary Friend".to_string(),
        ))],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// A creature with plain Partner keyword.
fn plain_partner_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-partner".to_string()),
        name: "Plain Partner".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Partner".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Partner)],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// A vanilla creature — no keywords.
fn vanilla_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("vanilla-creature".to_string()),
        name: "Vanilla Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── Deck Validation Tests ─────────────────────────────────────────────────────

#[test]
/// CR 702.124j — Two commanders with matching PartnerWith names are a valid pair.
/// Pir has PartnerWith("Toothy, Imaginary Friend") and Toothy has
/// PartnerWith("Pir, Imaginative Rascal"), so they cross-reference correctly.
fn test_partner_with_deck_validation_matching_pair() {
    let pir = pir_def();
    let toothy = toothy_def();
    let result = validate_partner_commanders(&pir, &toothy);
    assert!(
        result.is_ok(),
        "CR 702.124j: Pir + Toothy should be a valid partner-with pair, got: {:?}",
        result
    );
}

#[test]
/// CR 702.124j — Two commanders with non-matching PartnerWith names fail validation.
/// Pir has PartnerWith("Toothy, Imaginary Friend") but Wrong Partner also names
/// "Toothy, Imaginary Friend" (not "Pir, Imaginative Rascal"), so they don't
/// cross-reference.
fn test_partner_with_deck_validation_mismatched_names() {
    let pir = pir_def();
    let wrong = wrong_partner_def();
    let result = validate_partner_commanders(&pir, &wrong);
    assert!(
        result.is_err(),
        "CR 702.124j: Pir + Wrong Partner should fail validation (mismatched names)"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("don't match"),
        "CR 702.124j: error should mention mismatched names, got: {err}"
    );
}

#[test]
/// CR 702.124f — A creature with PartnerWith and a creature with plain Partner
/// cannot be combined as commanders.
fn test_partner_with_cannot_combine_with_plain_partner() {
    let pir = pir_def();
    let plain = plain_partner_def();
    let result = validate_partner_commanders(&pir, &plain);
    assert!(
        result.is_err(),
        "CR 702.124f: PartnerWith + plain Partner should fail validation"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("702.124f"),
        "CR 702.124f: error should cite 702.124f, got: {err}"
    );
}

#[test]
/// CR 702.124j — A commander with PartnerWith and a commander with no partner
/// ability at all fail validation.
fn test_partner_with_one_has_keyword_other_has_none() {
    let pir = pir_def();
    let vanilla = vanilla_creature_def();
    let result = validate_partner_commanders(&pir, &vanilla);
    assert!(
        result.is_err(),
        "CR 702.124j: PartnerWith + non-partner should fail validation"
    );
}

// ── ETB Trigger Tests ─────────────────────────────────────────────────────────

#[test]
/// CR 702.124j — When a permanent with PartnerWith(name) enters the battlefield,
/// a PartnerWithTrigger is placed on the stack.
///
/// We manually push the PartnerWithTrigger onto the stack to simulate what
/// the engine does on PermanentEnteredBattlefield + flush.
fn test_partner_with_etb_trigger_fires() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![pir_def(), toothy_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Pir, Imaginative Rascal")
                .with_card_id(CardId("pir-imaginative-rascal".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::PartnerWith(
                    "Toothy, Imaginary Friend".to_string(),
                ))
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "Toothy, Imaginary Friend").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pir_id = find_object(&state, "Pir, Imaginative Rascal");

    // Manually push the PartnerWithTrigger onto the stack (simulates ETB flush).
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_partner_with_trigger_stack_obj(
            trigger_id,
            pir_id,
            "Toothy, Imaginary Friend",
            p1,
            p1,
        ));
    state.turn.priority_holder = Some(p1);

    // The trigger should be on the stack.
    let has_trigger = state
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::KeywordTrigger { keyword: KeywordAbility::PartnerWith(_), .. }));
    assert!(
        has_trigger,
        "CR 702.124j: PartnerWithTrigger should be on the stack"
    );

    // Both players pass; trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack should be empty.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.124j: stack should be empty after PartnerWithTrigger resolves"
    );
}

#[test]
/// CR 702.124j — When the PartnerWithTrigger resolves, if the named card is in
/// the target player's library, it is moved to that player's hand.
fn test_partner_with_trigger_finds_partner_in_library() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![pir_def(), toothy_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Pir, Imaginative Rascal")
                .with_card_id(CardId("pir-imaginative-rascal".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::PartnerWith(
                    "Toothy, Imaginary Friend".to_string(),
                ))
                .in_zone(ZoneId::Battlefield),
        )
        // Toothy is in P1's library.
        .object(ObjectSpec::card(p1, "Toothy, Imaginary Friend").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Filler Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Filler Card 2").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pir_id = find_object(&state, "Pir, Imaginative Rascal");

    // Verify Toothy starts in P1's library.
    let toothy_before = find_object_opt(&state, "Toothy, Imaginary Friend").unwrap();
    assert_eq!(
        state.objects.get(&toothy_before).unwrap().zone,
        ZoneId::Library(p1),
        "Toothy should start in P1's library"
    );

    // Manually push the PartnerWithTrigger.
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_partner_with_trigger_stack_obj(
            trigger_id,
            pir_id,
            "Toothy, Imaginary Friend",
            p1,
            p1,
        ));
    state.turn.priority_holder = Some(p1);

    // Resolve the trigger.
    let (state, _) = pass_all_until_empty(state, &[p1, p2]);

    // Toothy should now be in P1's hand (it gets a new ObjectId after zone move,
    // so search by name).
    let toothy_in_hand = state.objects.values().any(|obj| {
        obj.characteristics.name == "Toothy, Imaginary Friend" && obj.zone == ZoneId::Hand(p1)
    });
    assert!(
        toothy_in_hand,
        "CR 702.124j: Toothy should be in P1's hand after PartnerWithTrigger resolves"
    );

    // Library count should have decreased by 1 (Toothy moved to hand).
    let lib_count = state
        .zones
        .get(&ZoneId::Library(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        lib_count, 2,
        "CR 702.124j: library should have 2 cards left after Toothy moved to hand"
    );
}

#[test]
/// CR 702.124j — If the named card is NOT in the target player's library
/// (e.g., already in hand), the trigger still resolves (library is shuffled
/// but no card moves).
fn test_partner_with_trigger_partner_not_in_library() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![pir_def(), toothy_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Pir, Imaginative Rascal")
                .with_card_id(CardId("pir-imaginative-rascal".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::PartnerWith(
                    "Toothy, Imaginary Friend".to_string(),
                ))
                .in_zone(ZoneId::Battlefield),
        )
        // Toothy is already in P1's hand (not library).
        .object(ObjectSpec::card(p1, "Toothy, Imaginary Friend").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Filler Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Filler Card 2").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pir_id = find_object(&state, "Pir, Imaginative Rascal");

    // Verify Toothy starts in P1's hand.
    let toothy_before = find_object_opt(&state, "Toothy, Imaginary Friend").unwrap();
    assert_eq!(
        state.objects.get(&toothy_before).unwrap().zone,
        ZoneId::Hand(p1),
        "Toothy should start in P1's hand"
    );

    // Manually push the PartnerWithTrigger.
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_partner_with_trigger_stack_obj(
            trigger_id,
            pir_id,
            "Toothy, Imaginary Friend",
            p1,
            p1,
        ));
    state.turn.priority_holder = Some(p1);

    // Resolve the trigger.
    let (state, _) = pass_all_until_empty(state, &[p1, p2]);

    // Stack is empty.
    assert_eq!(state.stack_objects.len(), 0, "stack should be empty");

    // Toothy should still be in P1's hand (same card, no duplicate).
    let toothy_count = state
        .objects
        .values()
        .filter(|obj| {
            obj.characteristics.name == "Toothy, Imaginary Friend" && obj.zone == ZoneId::Hand(p1)
        })
        .count();
    assert_eq!(
        toothy_count, 1,
        "CR 702.124j: Toothy should still be in P1's hand (exactly once), not duplicated"
    );

    // Library still has 2 cards (nothing moved out of it).
    let lib_count = state
        .zones
        .get(&ZoneId::Library(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        lib_count, 2,
        "CR 702.124j: library count unchanged when partner not in library"
    );
}

#[test]
/// CR 702.124j (ruling) — The trigger fires even when the partner is already on
/// the battlefield. In that case, the search finds nothing in the library, but
/// the trigger resolves normally (library is shuffled).
fn test_partner_with_trigger_fires_when_partner_already_on_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![pir_def(), toothy_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Pir, Imaginative Rascal")
                .with_card_id(CardId("pir-imaginative-rascal".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::PartnerWith(
                    "Toothy, Imaginary Friend".to_string(),
                ))
                .in_zone(ZoneId::Battlefield),
        )
        // Toothy is also on the battlefield.
        .object(
            ObjectSpec::card(p1, "Toothy, Imaginary Friend")
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "Filler Card 1").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Filler Card 2").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pir_id = find_object(&state, "Pir, Imaginative Rascal");

    // Manually push the PartnerWithTrigger.
    let trigger_id = state.next_object_id();
    state
        .stack_objects
        .push_back(make_partner_with_trigger_stack_obj(
            trigger_id,
            pir_id,
            "Toothy, Imaginary Friend",
            p1,
            p1,
        ));
    state.turn.priority_holder = Some(p1);

    // Resolve the trigger.
    let (state, _) = pass_all_until_empty(state, &[p1, p2]);

    // Stack is empty — trigger resolved without error.
    assert_eq!(
        state.stack_objects.len(),
        0,
        "CR 702.124j: trigger should resolve even when partner is on battlefield"
    );

    // Library still has 2 cards (nothing moved out of it).
    let lib_count = state
        .zones
        .get(&ZoneId::Library(p1))
        .map(|z| z.object_ids().len())
        .unwrap_or(0);
    assert_eq!(
        lib_count, 2,
        "CR 702.124j: library unchanged when partner is on battlefield (not in library)"
    );

    // Toothy should NOT be in hand (it was on battlefield, not in library).
    let toothy_in_hand = state.objects.values().any(|obj| {
        obj.characteristics.name == "Toothy, Imaginary Friend" && obj.zone == ZoneId::Hand(p1)
    });
    assert!(
        !toothy_in_hand,
        "CR 702.124j: Toothy should NOT be in hand (it was on battlefield)"
    );
}

#[test]
/// CR 702.124j — A permanent WITHOUT PartnerWith does NOT generate a
/// PartnerWithTrigger when it enters the battlefield.
fn test_partner_with_negative_no_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![vanilla_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Vanilla Creature")
                .with_card_id(CardId("vanilla-creature".to_string()))
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::card(p1, "Some Card").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // No PartnerWithTrigger should be pending or on the stack.
    let has_trigger = state
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::KeywordTrigger { keyword: KeywordAbility::PartnerWith(_), .. }));
    assert!(
        !has_trigger,
        "CR 702.124j: no PartnerWithTrigger for creature without PartnerWith keyword"
    );

    let has_pending = state
        .pending_triggers
        .iter()
        .any(|t| t.kind == mtg_engine::state::stubs::PendingTriggerKind::PartnerWith);
    assert!(
        !has_pending,
        "CR 702.124j: no pending partner-with trigger for vanilla creature"
    );
}

#[test]
/// CR 702.124j — The engine generates a PartnerWithTrigger in pending_triggers
/// when a permanent with PartnerWith enters the battlefield via check_triggers.
/// This validates the trigger generation path by checking `pending_triggers`
/// on the state when built with a PartnerWith permanent already on the battlefield,
/// then injecting the PermanentEnteredBattlefield event via the check_triggers API.
fn test_partner_with_etb_trigger_generated_by_check_triggers() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![pir_def(), toothy_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Pir, Imaginative Rascal")
                .with_card_id(CardId("pir-imaginative-rascal".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::PartnerWith(
                    "Toothy, Imaginary Friend".to_string(),
                ))
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let pir_id = find_object(&state, "Pir, Imaginative Rascal");

    // Call check_triggers directly with a PermanentEnteredBattlefield event.
    use mtg_engine::rules::abilities::check_triggers;
    use mtg_engine::rules::GameEvent;

    let events = vec![GameEvent::PermanentEnteredBattlefield {
        object_id: pir_id,
        player: p1,
    }];
    let new_triggers = check_triggers(&state, &events);

    // There should be a PartnerWith trigger in the results.
    let has_pw_trigger = new_triggers.iter().any(|t| {
        t.kind == mtg_engine::state::stubs::PendingTriggerKind::PartnerWith
            && t.partner_with_name.as_deref() == Some("Toothy, Imaginary Friend")
    });
    assert!(
        has_pw_trigger,
        "CR 702.124j: check_triggers should generate a partner-with PendingTrigger for Pir"
    );
}
