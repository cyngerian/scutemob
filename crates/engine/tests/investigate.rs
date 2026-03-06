//! Investigate keyword action tests (CR 701.16).
//!
//! Investigate is a keyword action (like Scry and Surveil), not a keyword ability.
//! Cards say "Investigate" as part of a spell effect or triggered/activated ability
//! effect. "Investigate" means "Create a Clue token." (CR 701.16a)
//!
//! Key rules verified:
//! - Investigate creates exactly one Clue token per investigation (CR 701.16a).
//! - Investigating N times creates N Clue tokens one at a time (ruling 2024-06-07).
//! - Investigating 0 times creates no tokens and emits no Investigated event.
//! - GameEvent::Investigated is emitted with the correct player and count.
//! - In multiplayer, the Clue token is created under the investigating player's control.
//! - Clue token can be activated ({2}, sacrifice) to draw a card (CR 111.10f).

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameStateBuilder, ManaColor, ManaCost, ObjectId, ObjectSpec,
    PlayerId, Step, SubType, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Find an object by name in the game state (panics if not found).
fn find_by_name(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Count objects with a given name on the battlefield.
fn count_clues_on_battlefield(state: &mtg_engine::GameState, controller: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == controller
                && obj.characteristics.name == "Clue"
        })
        .count()
}

/// Execute Effect::Investigate for a given player/count directly.
///
/// Returns (state, events). The source ObjectId is a placeholder (not used by Investigate).
fn run_investigate(
    mut state: mtg_engine::GameState,
    controller: PlayerId,
    count: i32,
) -> (mtg_engine::GameState, Vec<GameEvent>) {
    let effect = Effect::Investigate {
        count: EffectAmount::Fixed(count),
    };
    // Placeholder source id — Investigate does not reference the source object.
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
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

// ── Test 1: Basic investigate creates one Clue token ─────────────────────────

#[test]
/// CR 701.16a — "Investigate" means "Create a Clue token."
/// Verify the token has the Clue subtype, Artifact type, is colorless, and
/// has the correct activated ability (no tap required, {2} + sacrifice, draws 1 card).
fn test_investigate_creates_clue_token() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let initial_clues = count_clues_on_battlefield(&state, p1);
    assert_eq!(initial_clues, 0, "no Clue tokens initially");

    let (state, _events) = run_investigate(state, p1, 1);

    // One Clue token should now be on the battlefield under p1's control.
    assert_eq!(
        count_clues_on_battlefield(&state, p1),
        1,
        "CR 701.16a: Investigate should create exactly one Clue token"
    );

    // Verify token characteristics per CR 111.10f.
    let clue_obj = state
        .objects
        .values()
        .find(|o| {
            o.zone == ZoneId::Battlefield && o.controller == p1 && o.characteristics.name == "Clue"
        })
        .expect("Clue token should exist on battlefield");

    assert!(clue_obj.is_token, "Clue is a token");
    assert!(
        clue_obj.characteristics.colors.is_empty(),
        "CR 111.10f: Clue is colorless"
    );
    assert!(
        clue_obj
            .characteristics
            .card_types
            .contains(&CardType::Artifact),
        "CR 111.10f: Clue is an Artifact"
    );
    assert!(
        clue_obj
            .characteristics
            .subtypes
            .contains(&SubType("Clue".to_string())),
        "CR 111.10f: Clue has the Clue subtype"
    );
    assert_eq!(
        clue_obj.characteristics.activated_abilities.len(),
        1,
        "CR 111.10f: Clue has exactly one activated ability"
    );
    let ab = &clue_obj.characteristics.activated_abilities[0];
    assert!(
        !ab.cost.requires_tap,
        "CR 111.10f: Clue ability does NOT require {{T}} (unlike Food)"
    );
    assert!(
        ab.cost.sacrifice_self,
        "CR 111.10f: Clue ability requires sacrificing itself"
    );
    assert_eq!(
        ab.cost.mana_cost.as_ref().map(|mc| mc.generic).unwrap_or(0),
        2,
        "CR 111.10f: Clue ability requires {{2}} generic mana"
    );
}

// ── Test 2: Investigating twice creates two Clue tokens ───────────────────────

#[test]
/// Ruling 2024-06-07 — "If you're instructed to investigate multiple times, those
/// actions are sequential, meaning you'll create that many Clue tokens one at a time."
/// Effect::Investigate { count: Fixed(2) } creates two separate Clue tokens.
fn test_investigate_twice_creates_two_clues() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_investigate(state, p1, 2);

    // Two Clue tokens should be on the battlefield.
    assert_eq!(
        count_clues_on_battlefield(&state, p1),
        2,
        "Ruling 2024-06-07: Investigate twice should create 2 Clue tokens"
    );

    // Two TokenCreated events should be emitted (one per token).
    let token_created_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::TokenCreated { player, .. } if *player == p1))
        .count();
    assert_eq!(
        token_created_count, 2,
        "Two TokenCreated events should be emitted for 2 Clue tokens"
    );
}

// ── Test 3: Investigate emits GameEvent::Investigated ────────────────────────

#[test]
/// CR 701.16a — After investigating, GameEvent::Investigated { player, count } is
/// emitted. This enables "whenever you investigate" triggers (future cards).
fn test_investigate_emits_investigated_event() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (_, events) = run_investigate(state, p1, 1);

    let investigated = events
        .iter()
        .find(|e| matches!(e, GameEvent::Investigated { player, .. } if *player == p1));
    assert!(
        investigated.is_some(),
        "CR 701.16a: Investigated event must be emitted for p1"
    );

    if let Some(GameEvent::Investigated { player, count }) = investigated {
        assert_eq!(
            *player, p1,
            "Investigated event must reference the correct player"
        );
        assert_eq!(*count, 1, "Investigated event count must be 1");
    }
}

// ── Test 4: Investigating 0 times is a no-op ─────────────────────────────────

#[test]
/// Edge case: Effect::Investigate { count: Fixed(0) } creates no tokens and
/// emits no Investigated event. Analogous to CR 701.25c (Surveil 0 is a no-op).
fn test_investigate_zero_does_nothing() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_investigate(state, p1, 0);

    assert_eq!(
        count_clues_on_battlefield(&state, p1),
        0,
        "Investigate 0 should create no Clue tokens"
    );

    let investigated = events
        .iter()
        .any(|e| matches!(e, GameEvent::Investigated { .. }));
    assert!(
        !investigated,
        "Investigate 0 should NOT emit an Investigated event"
    );

    let token_created = events
        .iter()
        .any(|e| matches!(e, GameEvent::TokenCreated { .. }));
    assert!(
        !token_created,
        "Investigate 0 should NOT emit any TokenCreated events"
    );
}

// ── Test 5: Multiplayer — Clue goes under correct controller's control ────────

#[test]
/// CR 701.16a + multiplayer correctness: In a 4-player game, when player 3
/// investigates, the Clue token is created under player 3's control (not the
/// active player or any other player).
fn test_investigate_multiplayer_correct_controller() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // p3 investigates (not the active player).
    let (state, events) = run_investigate(state, p3, 1);

    // Clue should be under p3's control, not p1 (active player).
    assert_eq!(
        count_clues_on_battlefield(&state, p3),
        1,
        "The Clue token must be under the investigating player's (p3) control"
    );
    assert_eq!(
        count_clues_on_battlefield(&state, p1),
        0,
        "The active player (p1) should NOT have a Clue token"
    );

    // TokenCreated event must reference p3.
    let token_event = events
        .iter()
        .find(|e| matches!(e, GameEvent::TokenCreated { player, .. } if *player == p3));
    assert!(
        token_event.is_some(),
        "TokenCreated event must reference the investigating player p3"
    );

    // Investigated event must reference p3.
    let investigated_event = events
        .iter()
        .find(|e| matches!(e, GameEvent::Investigated { player, .. } if *player == p3));
    assert!(
        investigated_event.is_some(),
        "Investigated event must reference the investigating player p3"
    );
}

// ── Test 6: Integration — investigate then activate Clue to draw ──────────────

#[test]
/// CR 701.16a + CR 111.10f — Integration test: a sorcery with Effect::Investigate
/// creates a Clue token, then the controller activates the Clue's ability ({2},
/// sacrifice) to draw a card. Verify the full flow end-to-end.
///
/// Note: DrawCards on empty library is a no-op (gotchas-infra.md), so we place
/// a dummy card in the library.
fn test_investigate_clue_can_be_activated() {
    let p1 = p(1);
    let p2 = p(2);

    // Define a simple "Investigate" sorcery card.
    let investigate_def = CardDefinition {
        card_id: CardId("investigate-sorcery".to_string()),
        name: "Investigate Sorcery".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Investigate.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Investigate {
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![investigate_def]);

    let spell_spec = ObjectSpec::card(p1, "Investigate Sorcery")
        .with_card_id(CardId("investigate-sorcery".to_string()))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let library_card = ObjectSpec::card(p1, "Library Dummy").in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .object(spell_spec)
        .object(library_card)
        .with_registry(registry)
        .build()
        .unwrap();

    // Give p1 {1} generic mana to cast the sorcery.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let spell_id = find_by_name(&state, "Investigate Sorcery");

    // Cast the Investigate sorcery.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
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
        },
    )
    .unwrap();

    // Pass priority for both players to let the spell resolve.
    let (state, _) = pass_all(state, &[p1, p2]);

    // A Clue token should now be on the battlefield.
    assert_eq!(
        count_clues_on_battlefield(&state, p1),
        1,
        "CR 701.16a: Investigate sorcery should create exactly one Clue token"
    );

    // Give p1 {2} to pay for the Clue activation.
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let clue_id = find_by_name(&state, "Clue");
    let initial_hand_size = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();

    // Activate the Clue's ability ({2}, sacrifice this token: draw a card).
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: clue_id,
            ability_index: 0,
            targets: vec![],
        },
    )
    .unwrap();

    // Clue is sacrificed as a cost — it should no longer be on the battlefield.
    // Effect is on stack; pass priority to resolve.
    let (state_final, _) = pass_all(state, &[p1, p2]);

    // After resolution: p1 drew 1 card.
    let final_hand_size = state_final
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    assert_eq!(
        final_hand_size,
        initial_hand_size + 1,
        "CR 111.10f: Activating the Clue should draw 1 card"
    );

    // Clue token is gone (was sacrificed as a cost).
    assert_eq!(
        count_clues_on_battlefield(&state_final, p1),
        0,
        "Clue token should be gone after being sacrificed"
    );
}
