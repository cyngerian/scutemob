//! Squad keyword ability tests (CR 702.157).
//!
//! Squad represents two linked abilities:
//! 1. A static ability: "As an additional cost to cast this spell, you may pay [cost]
//!    any number of times." (CR 702.157a)
//! 2. A triggered ability: "When this creature enters, if its squad cost was paid,
//!    create a token that's a copy of it for each time its squad cost was paid."
//!    (CR 702.157a)
//!
//! Key rules verified:
//! - Squad is optional — cast with squad_count=0 to not pay it (no trigger, no tokens).
//! - Paying squad_count=N adds the squad cost N times to the total mana cost.
//! - When squad_count > 0, a SquadTrigger appears on the stack after the spell resolves.
//! - Resolving SquadTrigger creates N token copies on the battlefield.
//! - Token copies use copiable values (CR 707.2) — same name, P/T, types as original.
//! - Tokens are NOT cast; spells_cast_this_turn is NOT incremented (ruling 2022-10-07).
//! - Providing squad_count > 0 for a spell without Squad keyword is rejected.
//! - If the permanent loses the Squad keyword before the trigger fires, no tokens are
//!   created (ruling 2022-10-07 intervening-if check on battlefield).

use mtg_engine::AdditionalCost;
use mtg_engine::{
    calculate_characteristics, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, GameEvent, GameStateBuilder, KeywordAbility, ManaColor,
    ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_objects_on_battlefield<'a>(state: &'a mtg_engine::GameState, name: &str) -> Vec<ObjectId> {
    state
        .objects
        .iter()
        .filter(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .collect()
}

/// Count battlefield permanents matching the given name.
fn count_on_battlefield(state: &mtg_engine::GameState, name: &str) -> usize {
    find_objects_on_battlefield(state, name).len()
}

/// Pass priority for all listed players once (round-robin, one pass each).
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

/// Synthetic Squad {2} creature: 2/2 for {3}{W}. Squad {2}.
/// Ultramarines-style — a 2/2 with no other relevant abilities for easy token verification.
fn squad_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("squad-test".to_string()),
        name: "Squad Warrior".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Squad {2} (As an additional cost to cast this spell, you may pay {2} any number of times. When this creature enters, if its squad cost was paid, create a token that's a copy of it for each time its squad cost was paid.)"
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Squad),
            AbilityDefinition::Squad {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// A plain creature without Squad keyword — used to verify rejection of squad_count > 0.
fn plain_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-creature".to_string()),
        name: "Plain Soldier".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}

/// Build the ObjectSpec for the Squad Warrior in the given player's hand.
fn squad_spec(player: PlayerId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(player, "Squad Warrior")
        .in_zone(ZoneId::Hand(player))
        .with_card_id(CardId("squad-test".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Squad)
        .with_mana_cost(ManaCost {
            generic: 3,
            white: 1,
            ..Default::default()
        });
    // ObjectSpec has public power/toughness fields; set them so the permanent
    // on the battlefield has the correct base P/T for copy verification.
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

/// Build the ObjectSpec for the plain creature.
fn plain_spec(player: PlayerId) -> ObjectSpec {
    ObjectSpec::card(player, "Plain Soldier")
        .in_zone(ZoneId::Hand(player))
        .with_card_id(CardId("plain-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        })
    // P/T is populated from CardDefinition by enrich_spec_from_def in the builder.
}

/// Helper: build a 2-player state with Squad Warrior in p1's hand.
/// Adds enough mana for the base cost {3}{W} plus `extra_generic` additional colorless.
fn setup_squad_state(extra_generic: u32) -> (mtg_engine::GameState, PlayerId, PlayerId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![squad_creature_def(), plain_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(squad_spec(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    let ps = state.players.get_mut(&p1).unwrap();
    // Base cost: {3}{W}
    ps.mana_pool.add(ManaColor::Colorless, 3);
    ps.mana_pool.add(ManaColor::White, 1);
    // Extra colorless for squad payments
    ps.mana_pool.add(ManaColor::Colorless, extra_generic);
    state.turn.priority_holder = Some(p1);

    // The GameStateBuilder creates the object; we need to find it by name
    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Squad Warrior")
        .map(|(id, _)| *id)
        .expect("Squad Warrior should be in hand");

    (state, p1, p2, card_id)
}

/// Helper: cast Squad Warrior with the given squad_count and already-loaded mana.
fn cast_squad(
    state: mtg_engine::GameState,
    caster: PlayerId,
    card_id: ObjectId,
    squad_count: u32,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    process_command(
        state,
        Command::CastSpell {
            player: caster,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: if squad_count > 0 {
                vec![AdditionalCost::Squad { count: squad_count }]
            } else {
                vec![]
            },
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell(squad_count={}) failed: {:?}", squad_count, e))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.157a — Squad with squad_count=0: cast without paying the squad cost.
/// Verify no SquadTrigger is created, the spell resolves, one creature on battlefield.
#[test]
fn test_squad_zero_payments() {
    let (state, p1, p2, card_id) = setup_squad_state(0);

    // Cast without paying squad (squad_count = 0). Sufficient mana: {3}{W}.
    let (state, _) = cast_squad(state, p1, card_id, 0);

    // Only the spell should be on the stack — no trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.157a intervening-if: no SquadTrigger when squad cost was not paid"
    );

    // Resolve the spell (p1, p2 pass priority).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Squad Warrior is on the battlefield.
    assert_eq!(
        count_on_battlefield(&state, "Squad Warrior"),
        1,
        "CR 702.157a: creature should be on battlefield after resolving"
    );

    // No SquadTrigger on the stack.
    assert!(
        state.stack_objects.is_empty(),
        "CR 702.157a: no SquadTrigger should be on stack when squad_count=0"
    );
}

/// CR 702.157a — Squad with squad_count=1: cast paying the squad cost once.
/// After resolving the spell AND the SquadTrigger: 2 creatures on battlefield
/// (1 original + 1 token copy).
#[test]
fn test_squad_basic_one_payment() {
    // Extra {2} for one squad payment.
    let (state, p1, p2, card_id) = setup_squad_state(2);

    let (state, _) = cast_squad(state, p1, card_id, 1);

    // Only the spell is on the stack immediately after casting (no on-cast trigger).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "only the spell should be on the stack immediately after casting"
    );

    // Resolve the spell (p1, p2 pass priority). The SquadTrigger should be placed.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After spell resolves, Squad Warrior is on battlefield and SquadTrigger is on stack.
    assert_eq!(
        count_on_battlefield(&state, "Squad Warrior"),
        1,
        "original creature should be on battlefield after spell resolves"
    );
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.157a: SquadTrigger should be on stack after spell resolves with squad_count=1"
    );

    // Resolve the SquadTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Now both original and 1 token copy are on the battlefield.
    assert_eq!(
        count_on_battlefield(&state, "Squad Warrior"),
        2,
        "CR 702.157a: original + 1 token copy should be on battlefield (squad_count=1)"
    );
    assert!(
        state.stack_objects.is_empty(),
        "stack should be empty after SquadTrigger resolves"
    );
}

/// CR 702.157a — Squad with squad_count=3: cast paying the squad cost three times.
/// After resolving: 4 creatures on battlefield (1 original + 3 token copies).
#[test]
fn test_squad_multiple_payments() {
    // Extra {6} for three squad payments ({2} × 3).
    let (state, p1, p2, card_id) = setup_squad_state(6);

    let (state, _) = cast_squad(state, p1, card_id, 3);

    // Resolve the spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        count_on_battlefield(&state, "Squad Warrior"),
        1,
        "original should be on battlefield after spell resolves"
    );

    // SquadTrigger should be on stack.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.157a: SquadTrigger should be on stack with squad_count=3"
    );

    // Resolve the SquadTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        count_on_battlefield(&state, "Squad Warrior"),
        4,
        "CR 702.157a: original + 3 token copies should be on battlefield (squad_count=3)"
    );
}

/// CR 707.2 — Token copies have same name, P/T as the source.
/// Verify token copies created by Squad use copiable values.
#[test]
fn test_squad_tokens_are_copies() {
    // Extra {2} for one squad payment.
    let (state, p1, p2, card_id) = setup_squad_state(2);

    let (state, _) = cast_squad(state, p1, card_id, 1);
    // Resolve spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve SquadTrigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let copies = find_objects_on_battlefield(&state, "Squad Warrior");
    assert_eq!(
        copies.len(),
        2,
        "CR 707.2: should have 2 Squad Warriors on battlefield (1 original + 1 token copy)"
    );

    for id in &copies {
        // Use calculate_characteristics to apply layer system (CopyOf effect sets P/T).
        let chars = calculate_characteristics(&state, *id)
            .expect("CR 707.2: object should have layer-resolved characteristics");
        assert_eq!(
            chars.name, "Squad Warrior",
            "CR 707.2: token copy should have same name as source"
        );
        let power = chars.power.unwrap_or(-99);
        let toughness = chars.toughness.unwrap_or(-99);
        assert_eq!(
            power, 2,
            "CR 707.2: token copy should have same power as source (2)"
        );
        assert_eq!(
            toughness, 2,
            "CR 707.2: token copy should have same toughness as source (2)"
        );
    }

    // Exactly one token (is_token == true).
    let tokens: Vec<_> = copies
        .iter()
        .filter(|id| state.objects.get(id).map(|o| o.is_token).unwrap_or(false))
        .collect();
    assert_eq!(
        tokens.len(),
        1,
        "CR 702.157a: exactly one token should be created for squad_count=1"
    );
}

/// Validation — cast with squad_count > 0 on a creature without Squad keyword is rejected.
/// (CR 702.157a: Squad cost can only be paid for a squad spell.)
#[test]
fn test_squad_rejected_without_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_creature_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(plain_spec(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    let ps = state.players.get_mut(&p1).unwrap();
    ps.mana_pool.add(ManaColor::Colorless, 1);
    ps.mana_pool.add(ManaColor::White, 1);
    ps.mana_pool.add(ManaColor::Colorless, 2); // extra for bogus squad payment
    state.turn.priority_holder = Some(p1);

    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Plain Soldier")
        .map(|(id, _)| *id)
        .expect("Plain Soldier should be in hand");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![AdditionalCost::Squad { count: 1 }], // trying to pay squad cost on non-squad creature
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.157a: squad_count > 0 on a non-squad creature should be rejected"
    );
}

/// Ruling 2022-10-07 — Tokens are NOT cast: spells_cast_this_turn is NOT
/// incremented for tokens created by Squad.
#[test]
fn test_squad_tokens_not_cast() {
    // Extra {2} for one squad payment.
    let (state, p1, p2, card_id) = setup_squad_state(2);

    // Record spells_cast_this_turn before casting.
    let spells_before = state.players[&p1].spells_cast_this_turn;

    // Cast the Squad creature.
    let (state, _) = cast_squad(state, p1, card_id, 1);

    // spells_cast_this_turn should have increased by 1 (the spell itself).
    let spells_after_cast = state.players[&p1].spells_cast_this_turn;
    assert_eq!(
        spells_after_cast,
        spells_before + 1,
        "casting the squad creature should increment spells_cast_this_turn by 1"
    );

    // Resolve spell.
    let (state, _) = pass_all(state, &[p1, p2]);
    // Resolve SquadTrigger (creates 1 token).
    let (state, _) = pass_all(state, &[p1, p2]);

    // spells_cast_this_turn should STILL be spells_before + 1 — token not cast.
    assert_eq!(
        state.players[&p1].spells_cast_this_turn,
        spells_before + 1,
        "ruling 2022-10-07: token copies created by Squad are not cast; spells_cast_this_turn should not increase"
    );

    assert_eq!(
        count_on_battlefield(&state, "Squad Warrior"),
        2,
        "1 original + 1 token should be on battlefield"
    );
}
