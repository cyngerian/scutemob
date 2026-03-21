//! Forage keyword action tests (CR 701.61).
//!
//! Forage is a keyword action (CR 701), not a keyword ability (CR 702). It defines a
//! composite cost with player choice between two options:
//!
//! CR 701.61a: "To forage means 'Exile three cards from your graveyard or sacrifice a Food.'"
//!
//! Key rules verified:
//! - Forage cost can be paid by sacrificing a Food artifact (CR 701.61a).
//! - Forage cost can be paid by exiling 3 cards from your graveyard (CR 701.61a).
//! - Food is an artifact subtype (not just a token type) — any artifact with SubType("Food") qualifies.
//! - Cannot forage with neither a Food you control nor 3+ cards in your graveyard (CR 701.61a).
//! - Forage cost is paid at activation time (CR 602.2), before the ability goes on the stack.
//! - Non-Food, non-graveyard-exile artifacts do NOT count as Food (subtype check required).
//! - When both options are available, deterministic fallback prefers Food sacrifice.
//! - Forage + mana cost both must be paid together (CR 602.2).

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    process_command, CardType, Command, Effect, EffectAmount, GameEvent, GameState,
    GameStateBuilder, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step,
    SubType, ZoneId,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Find an object in the game state by name (panics if not found).
fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Find an object by name in a specific zone. Returns None if not found.
fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Battlefield).is_some()
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    find_in_zone(state, name, ZoneId::Graveyard(owner)).is_some()
}

fn in_exile(state: &GameState, name: &str) -> bool {
    find_in_zone(state, name, ZoneId::Exile).is_some()
}

/// Pass priority for all listed players once (resolves 1 stack item per call when all pass).
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

/// Build a Food token `ObjectSpec` with SubType("Food") on the battlefield.
fn food_artifact(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::artifact(owner, name).with_subtypes(vec![SubType("Food".to_string())])
}

/// Build a creature with a forage-cost activated ability ({2}, Forage: Gain 2 life).
/// The effect is simple (gain life) so we can verify it resolved.
fn forager_creature(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Forager Creature", 2, 2).with_activated_ability(ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: false,
            mana_cost: Some(ManaCost {
                generic: 2,
                ..Default::default()
            }),
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: true,
            sacrifice_filter: None,
        },
        description: "{2}, Forage: You gain 2 life.".to_string(),
        effect: Some(Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(2),
        }),
        sorcery_speed: false,
            activation_condition: None,
    })
}

/// Build a creature with a forage-only ability (no mana cost, just Forage).
fn forage_only_creature(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Forage-Only Creature", 1, 1).with_activated_ability(
        ActivatedAbility {
            targets: vec![],
            cost: ActivationCost {
                requires_tap: false,
                mana_cost: None,
                sacrifice_self: false,
                discard_card: false,
                discard_self: false,
                forage: true,
                sacrifice_filter: None,
            },
            description: "Forage: You gain 1 life.".to_string(),
            effect: Some(Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            activation_condition: None,
        },
    )
}

// ── Test 1: Forage by sacrificing a Food token ────────────────────────────────

/// CR 701.61a — Forage cost paid by sacrificing a Food artifact.
/// The Food leaves the battlefield at activation time (CR 602.2). The ability
/// then resolves after all players pass priority, granting the effect.
#[test]
fn test_forage_sacrifice_food() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(forager_creature(p1).in_zone(ZoneId::Battlefield))
        .object(food_artifact(p1, "Food Token").in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {2} generic mana.
    for _ in 0..2 {
        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, 1);
    }
    state.turn.priority_holder = Some(p1);

    let creature_id = find_by_name(&state, "Forager Creature");
    let p1_life_before = state.players[&p1].life_total;

    // Activate the forage ability (ability_index 0 = first non-mana ability).
    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .expect("forage activate with Food available should succeed");

    // Food is sacrificed at activation time — not on battlefield anymore.
    assert!(
        !on_battlefield(&state, "Food Token"),
        "CR 701.61a: Food should be sacrificed (off battlefield) at activation time"
    );

    // Activation event emitted.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 701.61a: AbilityActivated event expected"
    );

    // Food goes to graveyard (it's an artifact, so PermanentDestroyed).
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentDestroyed { .. })),
        "CR 701.61a: PermanentDestroyed event expected when Food is sacrificed"
    );
    assert!(
        in_graveyard(&state, "Food Token", p1),
        "CR 701.61a: sacrificed Food should be in p1's graveyard"
    );

    // The forage ability is on the stack. Pass priority to resolve it.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After resolution, p1 should have gained 2 life.
    let p1_life_after = state.players[&p1].life_total;
    assert_eq!(
        p1_life_after,
        p1_life_before + 2,
        "CR 701.61a: forage ability resolved — p1 should have gained 2 life"
    );
}

// ── Test 2: Forage by exiling 3 cards from graveyard ────────────────────────

/// CR 701.61a — Forage cost paid by exiling 3 cards from graveyard (no Food available).
/// The 3 cards are exiled at activation time. The ability resolves after priority.
#[test]
fn test_forage_exile_three_from_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(forage_only_creature(p1).in_zone(ZoneId::Battlefield))
        // 3 cards in p1's graveyard — no Food on battlefield.
        .object(ObjectSpec::card(p1, "Dead Card A").in_zone(ZoneId::Graveyard(p1)))
        .object(ObjectSpec::card(p1, "Dead Card B").in_zone(ZoneId::Graveyard(p1)))
        .object(ObjectSpec::card(p1, "Dead Card C").in_zone(ZoneId::Graveyard(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let creature_id = find_by_name(&state, "Forage-Only Creature");
    let p1_life_before = state.players[&p1].life_total;

    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .expect("forage activate with 3 graveyard cards should succeed (no Food needed)");

    // 3 ObjectExiled events should be emitted for the 3 graveyard cards.
    let exile_events: Vec<_> = activate_events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectExiled { player, .. } if *player == p1))
        .collect();
    assert_eq!(
        exile_events.len(),
        3,
        "CR 701.61a: exactly 3 ObjectExiled events expected when foraging via graveyard"
    );

    // All 3 cards should now be in exile.
    assert!(
        in_exile(&state, "Dead Card A"),
        "CR 701.61a: Dead Card A should be in exile"
    );
    assert!(
        in_exile(&state, "Dead Card B"),
        "CR 701.61a: Dead Card B should be in exile"
    );
    assert!(
        in_exile(&state, "Dead Card C"),
        "CR 701.61a: Dead Card C should be in exile"
    );

    // Resolve the ability.
    let (state, _) = pass_all(state, &[p1, p2]);

    let p1_life_after = state.players[&p1].life_total;
    assert_eq!(
        p1_life_after,
        p1_life_before + 1,
        "CR 701.61a: forage ability resolved — p1 should have gained 1 life"
    );
}

// ── Test 3: Insufficient resources — cannot forage ───────────────────────────

/// CR 701.61a — Cannot forage with fewer than 3 graveyard cards AND no Food.
/// The engine must return an error; game state must not change.
#[test]
fn test_forage_insufficient_resources() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(forage_only_creature(p1).in_zone(ZoneId::Battlefield))
        // Only 2 graveyard cards — not enough for exile option.
        // No Food on battlefield.
        .object(ObjectSpec::card(p1, "Dead Card A").in_zone(ZoneId::Graveyard(p1)))
        .object(ObjectSpec::card(p1, "Dead Card B").in_zone(ZoneId::Graveyard(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let creature_id = find_by_name(&state, "Forage-Only Creature");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 701.61a: forage with <3 graveyard cards and no Food must fail"
    );
}

// ── Test 4: Forage + mana cost both paid ────────────────────────────────────

/// CR 602.2 — Forage ability with {2} mana cost. Both costs must be satisfied.
/// Missing mana should cause an error even if Food is available.
#[test]
fn test_forage_requires_mana_cost_too() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(forager_creature(p1).in_zone(ZoneId::Battlefield))
        .object(food_artifact(p1, "Food Token").in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // No mana in pool — forager_creature requires {2} mana + forage.
    let creature_id = find_by_name(&state, "Forager Creature");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2: forage ability without sufficient mana must fail"
    );
}

// ── Test 5: Non-token artifact with Food subtype qualifies ───────────────────

/// Ruling 2024-11-08: Any artifact with SubType("Food") qualifies for forage —
/// not just Food tokens. A non-token artifact card with the Food subtype can be
/// sacrificed to forage.
#[test]
fn test_forage_food_is_artifact_subtype_not_just_token() {
    let p1 = p(1);
    let p2 = p(2);

    // Build a non-token artifact card with Food subtype (like Heaped Harvest).
    let heaped_harvest = ObjectSpec::artifact(p1, "Heaped Harvest")
        .with_subtypes(vec![SubType("Food".to_string())])
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(forage_only_creature(p1).in_zone(ZoneId::Battlefield))
        .object(heaped_harvest.in_zone(ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let creature_id = find_by_name(&state, "Forage-Only Creature");

    // Should succeed: Heaped Harvest has Food subtype.
    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .expect("ruling 2024-11-08: non-token Food artifact should qualify for forage");

    // Heaped Harvest was sacrificed.
    assert!(
        !on_battlefield(&state, "Heaped Harvest"),
        "ruling 2024-11-08: non-token Food artifact should be sacrificed off battlefield"
    );
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentDestroyed { .. })),
        "ruling 2024-11-08: PermanentDestroyed event expected for non-token Food sacrifice"
    );
}

// ── Test 6: Non-Food artifact does NOT qualify ───────────────────────────────

/// CR 701.61a — Only artifacts with SubType("Food") count as Food for forage.
/// A plain artifact without the Food subtype cannot be sacrificed to forage.
#[test]
fn test_forage_non_food_artifact_rejected() {
    let p1 = p(1);
    let p2 = p(2);

    // Plain artifact — no Food subtype.
    let plain_artifact =
        ObjectSpec::artifact(p1, "Plain Artifact").with_types(vec![CardType::Artifact]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(forage_only_creature(p1).in_zone(ZoneId::Battlefield))
        .object(plain_artifact.in_zone(ZoneId::Battlefield))
        // No graveyard cards either.
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_by_name(&state, "Forage-Only Creature");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 701.61a: a plain artifact without Food subtype cannot be used to forage"
    );
}

// ── Test 7: Deterministic fallback prefers Food when both options available ──

/// Deterministic engine (M9.5): when both Food sacrifice AND 3+ graveyard cards
/// are available, the engine prefers sacrificing the Food artifact.
#[test]
fn test_forage_prefers_food_when_both_available() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(forage_only_creature(p1).in_zone(ZoneId::Battlefield))
        .object(food_artifact(p1, "Food Token").in_zone(ZoneId::Battlefield))
        // Also 3 graveyard cards — both options available.
        .object(ObjectSpec::card(p1, "Dead Card A").in_zone(ZoneId::Graveyard(p1)))
        .object(ObjectSpec::card(p1, "Dead Card B").in_zone(ZoneId::Graveyard(p1)))
        .object(ObjectSpec::card(p1, "Dead Card C").in_zone(ZoneId::Graveyard(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    let creature_id = find_by_name(&state, "Forage-Only Creature");

    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
        },
    )
    .expect("forage with both options available should succeed");

    // Deterministic fallback: Food should be sacrificed (not graveyard cards).
    assert!(
        !on_battlefield(&state, "Food Token"),
        "deterministic fallback: Food token should be sacrificed"
    );
    // Graveyard cards should NOT be in exile.
    assert!(
        !in_exile(&state, "Dead Card A"),
        "deterministic fallback: graveyard cards should NOT be exiled when Food is available"
    );
    assert!(
        !in_exile(&state, "Dead Card B"),
        "deterministic fallback: graveyard cards should NOT be exiled when Food is available"
    );
    assert!(
        !in_exile(&state, "Dead Card C"),
        "deterministic fallback: graveyard cards should NOT be exiled when Food is available"
    );
    // PermanentDestroyed event confirms Food sacrifice.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentDestroyed { .. })),
        "deterministic fallback: PermanentDestroyed event expected for Food sacrifice"
    );
    // No ObjectExiled events expected.
    assert!(
        !activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { .. })),
        "deterministic fallback: no ObjectExiled events expected when Food is sacrificed"
    );
}
