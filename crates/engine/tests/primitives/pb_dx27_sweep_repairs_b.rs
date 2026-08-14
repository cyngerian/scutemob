//! PB-DX27 (scutemob-209), sweep-repairs batch B: five card defs whose blocker notes were
//! adjudicated FALSE by a read-only pass and re-verified independently here before authoring.
//! `memory/primitives/seed-rerank-2026-08-02.md` §4 rank 11 dispatch. Every claim below was
//! checked against `card_definition.rs` / `effects/mod.rs` at HEAD before the def edit, and
//! every authored ability is proven by an EXECUTED revert (see the coordinator's revert
//! matrix in the task report) -- this file is the discriminating half of that proof.
//!
//! - `marisi_breaker_of_the_coil` (`inert` -> `partial`): CR 510.3a/701.15a goad clause.
//!   T1-T2.
//! - `encroaching_dragonstorm` (`partial`, second trigger authored): CR 603.2 "when a Dragon
//!   you control enters, return this to owner's hand." T3-T5.
//! - `ruthless_technomancer` (`inert` -> `partial`): CR 118.12 ETB optional sacrifice -> Treasure
//!   count = sacrificed creature's power. T6-T7.
//! - `vampire_gourmand` (`inert` -> `partial`): CR 118.12 attack-trigger optional sacrifice ->
//!   draw + temporary CantBeBlocked. T8-T9.
//! - `kaito_shizuki` (`partial`, -2 authored, -7 deliberately NOT authored): CR 701.x-style
//!   token grant with a printed "can't be blocked" keyword clause. T10.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, AttackTarget,
    CardDefinition, CardId, CardRegistry, CardType, Command, CounterType, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step,
    SubType, TypeLine, ZoneId,
};
use std::collections::HashMap;

// ── Shared helpers ───────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn in_hand(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Hand(owner))
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn count_named_on_battlefield(state: &GameState, name: &str, controller: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| {
            o.characteristics.name == name
                && o.zone == ZoneId::Battlefield
                && o.controller == controller
        })
        .count()
}

fn hand_size(state: &GameState, owner: PlayerId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(owner))
        .count()
}

fn place_on_battlefield(owner: PlayerId, def: &CardDefinition) -> ObjectSpec {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    enrich_spec_from_def(
        ObjectSpec::card(owner, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &m,
    )
}

fn place_in_hand(owner: PlayerId, def: &CardDefinition) -> ObjectSpec {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    enrich_spec_from_def(
        ObjectSpec::card(owner, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Hand(owner)),
        &m,
    )
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

fn drain_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(
            guard < 50,
            "drain_stack: stack did not empty after 50 rounds"
        );
        state = pass_all(state, players).0;
    }
    state
}

fn declare_attackers(
    state: GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
) -> GameState {
    process_command(
        state,
        Command::DeclareAttackers {
            player,
            attackers,
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("DeclareAttackers should succeed")
    .0
}

fn empty_cast_spell_data(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
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
        additional_costs: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }))
}

fn add_mana(state: &mut GameState, player: PlayerId, color: ManaColor, amount: u32) {
    state
        .players_mut()
        .get_mut(&player)
        .unwrap()
        .mana_pool
        .add(color, amount);
}

/// A minimal test creature card, used to drive real `WhenEntersBattlefield` triggers on
/// Encroaching Dragonstorm via the actual cast/resolve pipeline rather than a bypassed
/// direct-state construction. `subtypes` distinguishes the Dragon case from the control case.
fn minimal_creature_def(card_id: &str, name: &str, subtypes: Vec<SubType>) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: subtypes.into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        ..Default::default()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Marisi, Breaker of the Coil — goad clause (CR 510.3a / CR 701.15a)
// ═══════════════════════════════════════════════════════════════════════════

// ── T1: goads every creature the damaged player controls ───────────────────

/// CR 510.3a / CR 701.15a: Marisi attacks P2 unblocked, dealing combat damage. Both of P2's
/// creatures must be goaded. Discriminator: pre-fix `abilities` was empty (the file's own
/// stale TODOs), so this is vacuous-to-live -- revert removes the Triggered ability entirely.
#[test]
fn test_dx27_marisi_goads_damaged_players_creatures() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = load_defs();

    let marisi = place_on_battlefield(p1, defs.get("Marisi, Breaker of the Coil").unwrap());
    let p2_c1 = ObjectSpec::creature(p2, "P2 Creature A", 1, 1)
        .with_card_id(CardId("p2-creature-a".to_string()));
    let p2_c2 = ObjectSpec::creature(p2, "P2 Creature B", 1, 1)
        .with_card_id(CardId("p2-creature-b".to_string()));
    let p3_c1 = ObjectSpec::creature(p3, "P3 Creature", 1, 1)
        .with_card_id(CardId("p3-creature".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(all_cards()))
        .object(marisi)
        .object(p2_c1)
        .object(p2_c2)
        .object(p3_c1)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let marisi_id = find_object(&state, "Marisi, Breaker of the Coil");
    let state = declare_attackers(state, p1, vec![(marisi_id, AttackTarget::Player(p2))]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // DeclareBlockers
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]); // CombatDamage + trigger queued
    let state = drain_stack(state, &[p1, p2, p3, p4]); // trigger resolves

    let p2_c1_id = find_object(&state, "P2 Creature A");
    let p2_c2_id = find_object(&state, "P2 Creature B");
    let p2_c1_obj = state.objects().get(&p2_c1_id).unwrap();
    let p2_c2_obj = state.objects().get(&p2_c2_id).unwrap();

    assert!(
        p2_c1_obj.goaded_by.contains(&p1),
        "CR 701.15a: P2 Creature A should be goaded by Marisi's controller"
    );
    assert!(
        p2_c2_obj.goaded_by.contains(&p1),
        "CR 701.15a: P2 Creature B should be goaded by Marisi's controller"
    );
}

// ── T2: a non-damaged player's creature is NOT goaded (multiplayer isolation) ──

/// CR 510.3a: P3 is not the damaged player -- P3's creature must not be goaded, even
/// though it shares the battlefield. Proves the `DamagedPlayer` scope, not a bare
/// "goad everything" bug.
#[test]
fn test_dx27_marisi_does_not_goad_other_players_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = load_defs();

    let marisi = place_on_battlefield(p1, defs.get("Marisi, Breaker of the Coil").unwrap());
    let p2_c1 = ObjectSpec::creature(p2, "P2 Only Creature", 1, 1)
        .with_card_id(CardId("p2-only-creature".to_string()));
    let p3_c1 = ObjectSpec::creature(p3, "P3 Untouched Creature", 1, 1)
        .with_card_id(CardId("p3-untouched-creature".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(all_cards()))
        .object(marisi)
        .object(p2_c1)
        .object(p3_c1)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let marisi_id = find_object(&state, "Marisi, Breaker of the Coil");
    let state = declare_attackers(state, p1, vec![(marisi_id, AttackTarget::Player(p2))]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let state = drain_stack(state, &[p1, p2, p3, p4]);

    let p3_c1_id = find_object(&state, "P3 Untouched Creature");
    let p3_c1_obj = state.objects().get(&p3_c1_id).unwrap();
    assert!(
        p3_c1_obj.goaded_by.is_empty(),
        "CR 510.3a: P3 was not the damaged player -- its creature must not be goaded"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Encroaching Dragonstorm — second trigger (CR 603.2 return-to-hand)
// ═══════════════════════════════════════════════════════════════════════════

// ── T3: a Dragon entering under your control returns Dragonstorm to hand ───

/// CR 603.2 / CR 400: casting and resolving a Dragon while Dragonstorm is on the
/// battlefield fires the trigger and moves Dragonstorm to its owner's hand.
/// Discriminator: pre-fix `abilities` had only the ETB search trigger -- reverting the
/// second `Triggered` ability leaves Dragonstorm on the battlefield forever.
#[test]
fn test_dx27_dragonstorm_returns_to_hand_when_dragon_enters() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let dragonstorm = place_on_battlefield(p1, defs.get("Encroaching Dragonstorm").unwrap());
    let dragon_def = minimal_creature_def(
        "dx27-test-dragon",
        "DX27 Test Dragon",
        vec![SubType("Dragon".to_string())],
    );

    let mut registry_cards = all_cards();
    registry_cards.push(dragon_def.clone());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(registry_cards))
        .object(dragonstorm)
        .object(place_in_hand(p1, &dragon_def))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    add_mana(&mut state, p1, ManaColor::Colorless, 1);

    let dragon_spell_id = find_object(&state, "DX27 Test Dragon");
    let (state, _) = process_command(state, empty_cast_spell_data(p1, dragon_spell_id))
        .expect("casting the test Dragon should succeed");
    let (state, _) = pass_all(state, &[p1, p2]); // dragon resolves, ETB fires
    let state = drain_stack(state, &[p1, p2]); // Dragonstorm's return trigger resolves

    assert!(
        in_hand(&state, "Encroaching Dragonstorm", p1),
        "CR 603.2: Encroaching Dragonstorm should have returned to its owner's hand \
         after a Dragon entered under its controller's control"
    );
    assert!(
        !on_battlefield(&state, "Encroaching Dragonstorm"),
        "Encroaching Dragonstorm must have left the battlefield (CR 400.7 new object on move)"
    );
}

// ── T4: a non-Dragon creature entering does NOT return Dragonstorm ─────────

/// CR 603.2: the trigger condition is "a Dragon you control enters" -- a same-controller
/// non-Dragon creature must not fire it.
#[test]
fn test_dx27_dragonstorm_stays_when_non_dragon_creature_enters() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let dragonstorm = place_on_battlefield(p1, defs.get("Encroaching Dragonstorm").unwrap());
    let bear_def = minimal_creature_def("dx27-test-bear", "DX27 Test Bear", vec![]);

    let mut registry_cards = all_cards();
    registry_cards.push(bear_def.clone());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(registry_cards))
        .object(dragonstorm)
        .object(place_in_hand(p1, &bear_def))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    add_mana(&mut state, p1, ManaColor::Colorless, 1);

    let bear_spell_id = find_object(&state, "DX27 Test Bear");
    let (state, _) = process_command(state, empty_cast_spell_data(p1, bear_spell_id))
        .expect("casting the test Bear should succeed");
    let (state, _) = pass_all(state, &[p1, p2]);
    let state = drain_stack(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Encroaching Dragonstorm"),
        "CR 603.2: a non-Dragon creature entering must not return Dragonstorm to hand"
    );
}

// ── T5: an OPPONENT's Dragon entering does NOT return your Dragonstorm ─────

/// CR 603.2: "a Dragon YOU control" -- P2's Dragon entering under P2's control must not
/// fire P1's Dragonstorm.
#[test]
fn test_dx27_dragonstorm_stays_when_opponents_dragon_enters() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let dragonstorm = place_on_battlefield(p1, defs.get("Encroaching Dragonstorm").unwrap());
    let dragon_def = minimal_creature_def(
        "dx27-test-dragon-2",
        "DX27 Opponent Dragon",
        vec![SubType("Dragon".to_string())],
    );

    let mut registry_cards = all_cards();
    registry_cards.push(dragon_def.clone());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(registry_cards))
        .object(dragonstorm)
        .object(place_in_hand(p2, &dragon_def))
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p2);
    add_mana(&mut state, p2, ManaColor::Colorless, 1);

    let dragon_spell_id = find_object(&state, "DX27 Opponent Dragon");
    let (state, _) = process_command(state, empty_cast_spell_data(p2, dragon_spell_id))
        .expect("casting the opponent's test Dragon should succeed");
    let (state, _) = pass_all(state, &[p2, p1]);
    let state = drain_stack(state, &[p2, p1]);

    assert!(
        on_battlefield(&state, "Encroaching Dragonstorm"),
        "CR 603.2: an opponent's Dragon entering must not return your Dragonstorm to hand"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Ruthless Technomancer — ETB optional sacrifice (CR 118.12)
// ═══════════════════════════════════════════════════════════════════════════

// ── T6: sacrifices another creature, creates Treasures = its power ─────────

/// CR 118.12 / CR 109.1: with an eligible "another creature" on the battlefield when
/// Ruthless Technomancer enters, the engine pays (MayPayThenEffect is pay-when-able) and
/// creates Treasure tokens equal to the sacrificed creature's power. Discriminator: pre-fix
/// `abilities` was empty (the file's own stale ENGINE-BLOCKED note) -- reverting the
/// Triggered ability leaves 0 Treasures and the Bear alive.
#[test]
fn test_dx27_ruthless_technomancer_sacrifices_and_creates_treasures() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let bear = ObjectSpec::creature(p1, "Sac Fodder Bear", 3, 3)
        .with_card_id(CardId("sac-fodder-bear".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(bear)
        .object(place_in_hand(
            p1,
            defs.get("Ruthless Technomancer").unwrap(),
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    add_mana(&mut state, p1, ManaColor::Colorless, 3);
    add_mana(&mut state, p1, ManaColor::Black, 1);

    let technomancer_id = find_object(&state, "Ruthless Technomancer");
    let (state, _) = process_command(state, empty_cast_spell_data(p1, technomancer_id))
        .expect("casting Ruthless Technomancer should succeed");
    let (state, _) = pass_all(state, &[p1, p2]); // Technomancer resolves, ETB fires
    let state = drain_stack(state, &[p1, p2]); // ETB trigger resolves

    assert!(
        in_graveyard(&state, "Sac Fodder Bear", p1),
        "CR 118.12: the Bear should have been sacrificed"
    );
    assert_eq!(
        count_named_on_battlefield(&state, "Treasure", p1),
        3,
        "CR 118.12: Treasure count should equal the sacrificed creature's power (3)"
    );
    assert!(
        on_battlefield(&state, "Ruthless Technomancer"),
        "Ruthless Technomancer itself must never be sacrificed (exclude_self, CR 109.1)"
    );
}

// ── T7: alone (no eligible sacrifice), no Treasures and Technomancer survives ──

/// CR 118.12: with no OTHER creature to sacrifice, the optional cost is unpayable
/// (`can_pay_optional_cost` returns false) -- no Treasures, and Technomancer itself
/// cannot satisfy `exclude_self`.
#[test]
fn test_dx27_ruthless_technomancer_alone_creates_no_treasures() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(place_in_hand(
            p1,
            defs.get("Ruthless Technomancer").unwrap(),
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    add_mana(&mut state, p1, ManaColor::Colorless, 3);
    add_mana(&mut state, p1, ManaColor::Black, 1);

    let technomancer_id = find_object(&state, "Ruthless Technomancer");
    let (state, _) = process_command(state, empty_cast_spell_data(p1, technomancer_id))
        .expect("casting Ruthless Technomancer should succeed");
    let (state, _) = pass_all(state, &[p1, p2]);
    let state = drain_stack(state, &[p1, p2]);

    assert_eq!(
        count_named_on_battlefield(&state, "Treasure", p1),
        0,
        "CR 118.12: with no eligible 'another creature', no Treasure should be created"
    );
    assert!(
        on_battlefield(&state, "Ruthless Technomancer"),
        "Ruthless Technomancer should remain on the battlefield"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Vampire Gourmand — attack-trigger optional sacrifice (CR 118.12 / CR 509.1)
// ═══════════════════════════════════════════════════════════════════════════

// ── T8: sacrifices another creature -> draws a card and gains CantBeBlocked ──

/// CR 118.12 / CR 509.1: with an eligible "another creature", attacking Vampire Gourmand
/// sacrifices it, draws a card, and gains `KeywordAbility::CantBeBlocked` until end of turn.
/// Discriminator: pre-fix `abilities` was empty -- reverting the Triggered ability leaves
/// hand size unchanged and no CantBeBlocked keyword.
#[test]
fn test_dx27_vampire_gourmand_sacrifices_draws_and_becomes_unblockable() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let bear = ObjectSpec::creature(p1, "Gourmand Fodder", 1, 1)
        .with_card_id(CardId("gourmand-fodder".to_string()));
    let gourmand = place_on_battlefield(p1, defs.get("Vampire Gourmand").unwrap());
    let library_card = ObjectSpec::card(p1, "Library Filler").in_zone(ZoneId::Library(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(bear)
        .object(gourmand)
        .object(library_card)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let before_hand = hand_size(&state, p1);
    let gourmand_id = find_object(&state, "Vampire Gourmand");
    let state = declare_attackers(state, p1, vec![(gourmand_id, AttackTarget::Player(p2))]);
    let state = drain_stack(state, &[p1, p2]); // attack trigger resolves

    assert!(
        in_graveyard(&state, "Gourmand Fodder", p1),
        "CR 118.12: the other creature should have been sacrificed"
    );
    assert_eq!(
        hand_size(&state, p1),
        before_hand + 1,
        "CR 118.12: a card should have been drawn"
    );
    let chars = calculate_characteristics(&state, gourmand_id)
        .expect("Vampire Gourmand should still resolve characteristics");
    assert!(
        chars.keywords.contains(&KeywordAbility::CantBeBlocked),
        "CR 509.1: Vampire Gourmand should have gained CantBeBlocked this turn"
    );
}

// ── T9: attacking alone grants no draw and no evasion (unpayable cost) ─────

/// CR 118.12: with no eligible "another creature" (Vampire Gourmand itself is excluded via
/// `exclude_self`), the optional cost cannot be paid -- no draw, no CantBeBlocked.
#[test]
fn test_dx27_vampire_gourmand_alone_gains_no_evasion() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let gourmand = place_on_battlefield(p1, defs.get("Vampire Gourmand").unwrap());

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(gourmand)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let before_hand = hand_size(&state, p1);
    let gourmand_id = find_object(&state, "Vampire Gourmand");
    let state = declare_attackers(state, p1, vec![(gourmand_id, AttackTarget::Player(p2))]);
    let state = drain_stack(state, &[p1, p2]);

    assert_eq!(
        hand_size(&state, p1),
        before_hand,
        "CR 118.12: with no eligible sacrifice, no card should be drawn"
    );
    let chars = calculate_characteristics(&state, gourmand_id)
        .expect("Vampire Gourmand should still resolve characteristics");
    assert!(
        !chars.keywords.contains(&KeywordAbility::CantBeBlocked),
        "CR 509.1: with the cost unpaid, Vampire Gourmand must not gain CantBeBlocked"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Kaito Shizuki — the -2 loyalty ability only (NOT the -7 emblem)
// ═══════════════════════════════════════════════════════════════════════════

// ── T10: -2 creates a 1/1 blue Ninja token that can't be blocked ───────────

/// The printed -2: "Create a 1/1 blue Ninja creature token with 'This token can't be
/// blocked.'" Discriminator: pre-fix -2's `abilities` slot did not exist at all (only the
/// +1 ability was authored) -- reverting the LoyaltyAbility entry makes `ability_index: 1`
/// out of range and the activation is rejected outright.
#[test]
fn test_dx27_kaito_minus_two_creates_unblockable_ninja_token() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let kaito = ObjectSpec::card(p1, "Kaito Shizuki")
        .with_card_id(defs.get("Kaito Shizuki").unwrap().card_id.clone())
        .with_types(vec![CardType::Planeswalker])
        .with_counter(CounterType::Loyalty, 3)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(kaito)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let kaito_id = find_object(&state, "Kaito Shizuki");
    let (state, _) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: kaito_id,
            ability_index: 1,
            targets: vec![],
            x_value: None,
        },
    )
    .expect("Kaito's -2 loyalty ability should be activatable");
    let state = drain_stack(state, &[p1, p2]); // CR 606.1 -> 602: loyalty ability resolves off the stack

    assert_eq!(
        count_named_on_battlefield(&state, "Ninja", p1),
        1,
        "exactly one Ninja token should have been created"
    );
    let ninja_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Ninja" && o.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
        .expect("Ninja token should be on the battlefield");
    let chars = calculate_characteristics(&state, ninja_id)
        .expect("Ninja token should resolve characteristics");
    assert!(
        chars.keywords.contains(&KeywordAbility::CantBeBlocked),
        "the printed Ninja token must carry \"This token can't be blocked.\""
    );
    assert_eq!(chars.power, Some(1), "Ninja token should be 1/1");
    assert_eq!(chars.toughness, Some(1), "Ninja token should be 1/1");

    let kaito_obj = state.objects().get(&kaito_id).unwrap();
    let loyalty = kaito_obj
        .counters
        .get(&CounterType::Loyalty)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        loyalty, 1,
        "loyalty should have dropped from 3 to 1 after paying Minus(2)"
    );
}
