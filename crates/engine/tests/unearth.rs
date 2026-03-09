//! Unearth keyword ability tests (CR 702.84).
//!
//! Unearth is an activated ability that functions while the card is in a graveyard.
//! "Unearth [cost]" means "[cost]: Return this card from your graveyard to the
//! battlefield. It gains haste. Exile it at the beginning of the next end step.
//! If it would leave the battlefield, exile it instead of putting it anywhere else.
//! Activate only as a sorcery." (CR 702.84a)
//!
//! Key rules verified:
//! - Unearth activates from graveyard; ability goes on stack; card stays in graveyard (CR 702.84a).
//! - After resolution: creature enters battlefield with haste, was_unearthed flag set (CR 702.84a).
//! - Sorcery speed restriction: active player only, main phase only, empty stack (CR 702.84a).
//! - Exile at beginning of next end step (delayed trigger, CR 702.84a).
//! - Replacement effect: if unearthed permanent would leave battlefield for non-exile, exile instead (CR 702.84a).
//! - If effect actually exiles the creature, the exile succeeds normally (ruling 2008-10-01).
//! - If card is removed from graveyard before resolution, ability does nothing (ruling, CR 400.7).
//! - Unearth is NOT a cast: no cast triggers, spells_cast_this_turn unchanged (ruling 2008-10-01).
//! - Exile effects are not abilities on the creature: persist even if creature loses all abilities (ruling).

use mtg_engine::state::types::AltCostKind;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

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

/// Pass priority for all listed players once.
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

/// Dregscape Zombie: Creature {1}{B} 2/1, Unearth {B}.
fn dregscape_zombie_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("dregscape-zombie".to_string()),
        name: "Dregscape Zombie".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(1),
        oracle_text: "Unearth {B}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Unearth),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Unearth,
                cost: ManaCost {
                    black: 1,
                    ..Default::default()
                },
                details: None,
            },
        ],
        ..Default::default()
    }
}

/// Build an ObjectSpec for Dregscape Zombie enriched with its card definition.
fn zombie_in_graveyard(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::card(owner, "Dregscape Zombie")
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(CardId("dregscape-zombie".to_string()))
        .with_keyword(KeywordAbility::Unearth)
}

// ── Test 1: Basic unearth — creature returns to battlefield ───────────────────

#[test]
/// CR 702.84a — Activate unearth on a creature in the graveyard; ability goes on
/// stack; when it resolves the creature enters the battlefield with haste.
fn test_unearth_basic_return_to_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana for unearth cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dregscape Zombie");

    // p1 activates unearth.
    let (state, activate_events) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");

    // AbilityActivated event emitted.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p1)),
        "CR 702.84a: AbilityActivated event expected when unearth is activated"
    );

    // Card is still in the graveyard (card is NOT moved as part of activation cost).
    assert!(
        in_graveyard(&state, "Dregscape Zombie", p1),
        "CR 702.84a: card stays in graveyard until ability resolves (NOT discarded as cost)"
    );

    // Both players pass priority → ability resolves.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // PermanentEnteredBattlefield event emitted.
    assert!(
        resolve_events.iter().any(
            |e| matches!(e, GameEvent::PermanentEnteredBattlefield { player, .. } if *player == p1)
        ),
        "CR 702.84a: PermanentEnteredBattlefield event expected after unearth resolves"
    );

    // Creature is now on the battlefield.
    assert!(
        on_battlefield(&state, "Dregscape Zombie"),
        "CR 702.84a: creature should be on battlefield after unearth resolves"
    );

    // Creature NOT in graveyard.
    assert!(
        !in_graveyard(&state, "Dregscape Zombie", p1),
        "CR 702.84a: creature should no longer be in graveyard after unearth"
    );

    // Creature has haste.
    let zombie_id = find_in_zone(&state, "Dregscape Zombie", ZoneId::Battlefield)
        .expect("zombie should be on battlefield");
    let obj = state.objects.get(&zombie_id).unwrap();
    assert!(
        obj.characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "CR 702.84a: unearthed creature should have haste"
    );

    // was_unearthed flag is set.
    assert!(
        obj.was_unearthed,
        "CR 702.84a: was_unearthed flag should be true after unearth resolves"
    );
}

// ── Test 2: Sorcery speed restriction ────────────────────────────────────────

#[test]
/// CR 702.84a — Unearth can only be activated during the active player's own
/// main phase with an empty stack. Attempts during opponent's turn, combat, or
/// with spells on the stack are rejected.
fn test_unearth_sorcery_speed_restriction() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    // Test 1: Cannot activate during opponent's turn (active player is p2).
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(zombie_in_graveyard(p1))
            .active_player(p2) // p2 is the active player
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Black, 1);
        state.turn.priority_holder = Some(p1);

        let card_id = find_object(&state, "Dregscape Zombie");

        let result = process_command(
            state,
            Command::UnearthCard {
                player: p1,
                card: card_id,
            },
        );
        assert!(
            result.is_err(),
            "CR 702.84a: should reject unearth during opponent's turn"
        );
    }

    // Test 2: Cannot activate during combat step.
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(zombie_in_graveyard(p1))
            .active_player(p1)
            .at_step(Step::DeclareAttackers) // Combat, not main phase
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Black, 1);
        state.turn.priority_holder = Some(p1);

        let card_id = find_object(&state, "Dregscape Zombie");

        let result = process_command(
            state,
            Command::UnearthCard {
                player: p1,
                card: card_id,
            },
        );
        assert!(
            result.is_err(),
            "CR 702.84a: should reject unearth during combat"
        );
    }

    // Test 3: Cannot activate if card is not in graveyard.
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(
                // Card in hand instead of graveyard
                ObjectSpec::card(p1, "Dregscape Zombie")
                    .in_zone(ZoneId::Hand(p1))
                    .with_card_id(CardId("dregscape-zombie".to_string()))
                    .with_keyword(KeywordAbility::Unearth),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        state
            .players
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Black, 1);
        state.turn.priority_holder = Some(p1);

        let card_id = find_object(&state, "Dregscape Zombie");

        let result = process_command(
            state,
            Command::UnearthCard {
                player: p1,
                card: card_id,
            },
        );
        assert!(
            result.is_err(),
            "CR 702.84a: should reject unearth when card is not in graveyard"
        );
    }
}

// ── Test 3: Exile at beginning of next end step ───────────────────────────────

#[test]
/// CR 702.84a — After unearth resolves, at the beginning of the next end step,
/// the creature is exiled. The delayed trigger fires when Step::End is entered.
fn test_unearth_exile_at_end_step() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    // Start at PostCombatMain so we can advance to End.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dregscape Zombie");

    // Activate unearth.
    let (state, _) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");

    // Both players pass priority → ability resolves → creature enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Dregscape Zombie"),
        "creature should be on battlefield after unearth"
    );

    // Advance through PostCombatMain → End step by passing priority until End.
    // PassPriority for both players moves us to End step (no combat declared).
    let (state, _) = pass_all(state, &[p1, p2]);

    // Now we should be at Step::End. The end_step_actions queue the unearth trigger.
    assert_eq!(
        state.turn.step,
        Step::End,
        "should have advanced to End step"
    );

    // Both players pass priority during End step → UnearthTrigger resolves → exile.
    let (state, end_events) = pass_all(state, &[p1, p2]);

    // ObjectExiled event emitted.
    assert!(
        end_events
            .iter()
            .any(|e| matches!(e, GameEvent::ObjectExiled { .. })),
        "CR 702.84a: ObjectExiled event expected when unearth delayed trigger resolves"
    );

    // Creature is now in exile, not on battlefield.
    assert!(
        !on_battlefield(&state, "Dregscape Zombie"),
        "CR 702.84a: creature should NOT be on battlefield after unearth trigger resolves"
    );
    assert!(
        in_exile(&state, "Dregscape Zombie"),
        "CR 702.84a: creature should be in exile after unearth trigger resolves"
    );
}

// ── Test 4: Replacement effect — bounce sends to exile ───────────────────────

#[test]
/// CR 702.84a — If an unearthed creature would be returned to hand (bounced),
/// it is exiled instead. Ruling 2008-10-01: "If a creature returned to the
/// battlefield with unearth would leave the battlefield for any reason,
/// it's exiled instead."
fn test_unearth_replacement_exile_on_bounce() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dregscape Zombie");

    // Activate unearth.
    let (state, _) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");

    // Resolve the ability.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Dregscape Zombie"),
        "creature should be on battlefield after unearth"
    );

    // Manually simulate a bounce (move to hand) -- the was_unearthed replacement
    // should redirect it to exile instead of hand.
    let zombie_id = find_in_zone(&state, "Dregscape Zombie", ZoneId::Battlefield)
        .expect("zombie should be on battlefield");

    // Use check_zone_change_replacement directly to verify replacement fires.
    let owner = state.objects.get(&zombie_id).unwrap().owner;
    let action = mtg_engine::rules::replacement::check_zone_change_replacement(
        &state,
        zombie_id,
        mtg_engine::state::zone::ZoneType::Battlefield,
        mtg_engine::state::zone::ZoneType::Hand,
        owner,
        &std::collections::HashSet::new(),
    );

    // Should be a Redirect to Exile (CR 702.84a replacement).
    assert!(
        matches!(
            &action,
            mtg_engine::rules::replacement::ZoneChangeAction::Redirect {
                to: ZoneId::Exile,
                ..
            }
        ),
        "CR 702.84a: bounce should be redirected to exile by unearth replacement"
    );

    // Actually perform the zone move via the replacement (to exile).
    let (new_id, _old) = state
        .move_object_to_zone(zombie_id, ZoneId::Exile)
        .expect("move to exile should succeed");
    let _ = new_id;

    assert!(
        !on_battlefield(&state, "Dregscape Zombie"),
        "creature should no longer be on battlefield"
    );
    assert!(
        in_exile(&state, "Dregscape Zombie"),
        "CR 702.84a: creature should be in exile after bounce replacement"
    );
}

// ── Test 5: Replacement effect — death sends to exile ────────────────────────

#[test]
/// CR 702.84a — If an unearthed creature would die (leave the battlefield for the
/// graveyard), the unearth replacement effect redirects it to exile instead.
/// Verified by calling check_zone_change_replacement directly for a Battlefield →
/// Graveyard transition, which is what SBAs and destroy effects produce.
fn test_unearth_replacement_exile_on_destroy() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dregscape Zombie");

    // Activate unearth.
    let (state, _) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");

    // Resolve the ability.
    let (mut state, _) = pass_all(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Dregscape Zombie"),
        "creature should be on battlefield after unearth"
    );

    let zombie_id = find_in_zone(&state, "Dregscape Zombie", ZoneId::Battlefield)
        .expect("zombie should be on battlefield");
    let owner = state.objects.get(&zombie_id).unwrap().owner;

    // Verify the replacement redirects a Battlefield→Graveyard move to Exile.
    // This is the path taken by SBAs (lethal damage, deathtouch) and destroy effects.
    let action = mtg_engine::rules::replacement::check_zone_change_replacement(
        &state,
        zombie_id,
        mtg_engine::state::zone::ZoneType::Battlefield,
        mtg_engine::state::zone::ZoneType::Graveyard,
        owner,
        &std::collections::HashSet::new(),
    );

    assert!(
        matches!(
            &action,
            mtg_engine::rules::replacement::ZoneChangeAction::Redirect {
                to: ZoneId::Exile,
                ..
            }
        ),
        "CR 702.84a: Battlefield→Graveyard should be redirected to exile by unearth replacement"
    );

    // Actually perform the redirect to exile.
    let (new_id, _old) = state
        .move_object_to_zone(zombie_id, ZoneId::Exile)
        .expect("move to exile should succeed");
    let _ = new_id;

    // NOT in graveyard.
    assert!(
        !in_graveyard(&state, "Dregscape Zombie", p1),
        "CR 702.84a: unearthed creature should NOT go to graveyard when destroyed"
    );

    // In exile.
    assert!(
        in_exile(&state, "Dregscape Zombie"),
        "CR 702.84a: unearthed creature should go to exile when destroyed"
    );
}

// ── Test 6: Actual exile succeeds (no replacement needed) ─────────────────────

#[test]
/// CR 702.84a ruling (2008-10-01) — "If the spell or ability is actually trying
/// to exile it, it succeeds at exiling it." If the destination is already exile,
/// the unearth replacement does NOT fire.
fn test_unearth_exile_does_not_replace_actual_exile() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dregscape Zombie");

    // Activate unearth and resolve.
    let (state, _) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");
    let (mut state, _) = pass_all(state, &[p1, p2]);

    let zombie_id = find_in_zone(&state, "Dregscape Zombie", ZoneId::Battlefield)
        .expect("zombie should be on battlefield");
    let owner = state.objects.get(&zombie_id).unwrap().owner;

    // Check that attempting to exile directly does NOT trigger the replacement.
    let action = mtg_engine::rules::replacement::check_zone_change_replacement(
        &state,
        zombie_id,
        mtg_engine::state::zone::ZoneType::Battlefield,
        mtg_engine::state::zone::ZoneType::Exile, // destination is already exile
        owner,
        &std::collections::HashSet::new(),
    );

    // Should Proceed (no replacement needed -- already going to exile).
    assert!(
        matches!(
            &action,
            mtg_engine::rules::replacement::ZoneChangeAction::Proceed
        ),
        "CR 702.84a ruling: exile → exile should Proceed, no replacement fires"
    );

    // Move to exile directly -- should work fine.
    let _ = state
        .move_object_to_zone(zombie_id, ZoneId::Exile)
        .expect("move to exile should succeed");

    assert!(
        in_exile(&state, "Dregscape Zombie"),
        "should be in exile after direct exile"
    );
    assert!(
        !on_battlefield(&state, "Dregscape Zombie"),
        "should not be on battlefield after exile"
    );
}

// ── Test 7: Card removed from graveyard before resolution ─────────────────────

#[test]
/// Ruling (2008-10-01) — "If you activate a card's unearth ability but that card
/// is removed from your graveyard before the ability resolves, that unearth ability
/// will resolve and do nothing." (CR 400.7)
fn test_unearth_card_removed_before_resolution() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dregscape Zombie");

    // Activate unearth -- ability goes on stack, card stays in graveyard.
    let (mut state, _) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");

    // Now exile the card from the graveyard before the ability resolves.
    // (Simulating an opponent exiling it in response.)
    let grave_card_id = find_in_zone(&state, "Dregscape Zombie", ZoneId::Graveyard(p1))
        .expect("zombie should be in graveyard");
    let _ = state
        .move_object_to_zone(grave_card_id, ZoneId::Exile)
        .expect("exile from graveyard should work");

    assert!(
        in_exile(&state, "Dregscape Zombie"),
        "zombie should be in exile after manual removal"
    );

    // Both players pass priority → UnearthAbility resolves → should do nothing.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // No PermanentEnteredBattlefield event.
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. })),
        "CR 400.7: no ETB event when source was removed from graveyard before resolution"
    );

    // Creature is NOT on battlefield (ability did nothing).
    assert!(
        !on_battlefield(&state, "Dregscape Zombie"),
        "CR 400.7: creature should NOT enter battlefield if removed from graveyard before resolution"
    );
}

// ── Test 8: Unearth is NOT a cast ─────────────────────────────────────────────

#[test]
/// Ruling (2008-10-01) — "Activating a creature card's unearth ability isn't the
/// same as casting the creature card." No SpellCast event should fire, and
/// spells_cast_this_turn should not increase.
fn test_unearth_is_not_a_cast() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let spells_before = state.players.get(&p1).unwrap().spells_cast_this_turn;

    let card_id = find_object(&state, "Dregscape Zombie");

    // Activate unearth.
    let (state, activate_events) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");

    // No SpellCast event.
    assert!(
        !activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "ruling: unearth is NOT a cast; SpellCast event should NOT fire"
    );

    // spells_cast_this_turn unchanged.
    let spells_after = state.players.get(&p1).unwrap().spells_cast_this_turn;
    assert_eq!(
        spells_before, spells_after,
        "ruling: unearth is NOT a cast; spells_cast_this_turn should not increase"
    );

    // Resolve and confirm creature enters via AbilityResolved (not SpellResolved).
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellResolved { .. })),
        "ruling: unearth is NOT a cast; SpellResolved event should NOT fire"
    );
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "ruling: unearth resolves as ability; AbilityResolved event should fire"
    );
    assert!(
        on_battlefield(&state, "Dregscape Zombie"),
        "creature should be on battlefield after unearth resolves"
    );
}

// ── Test 9: Unearthed creature has haste ─────────────────────────────────────

#[test]
/// CR 702.84a — "It gains haste." The unearthed creature gains haste and does
/// not have summoning sickness. It can attack the turn it enters.
fn test_unearth_creature_has_haste() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dregscape Zombie");

    // Activate and resolve unearth.
    let (state, _) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");
    let (state, _) = pass_all(state, &[p1, p2]);

    let zombie_id = find_in_zone(&state, "Dregscape Zombie", ZoneId::Battlefield)
        .expect("zombie should be on battlefield");
    let obj = state.objects.get(&zombie_id).unwrap();

    // Has haste keyword (CR 702.84a).
    assert!(
        obj.characteristics
            .keywords
            .contains(&KeywordAbility::Haste),
        "CR 702.84a: unearthed creature must have haste"
    );
    // was_unearthed flag set.
    assert!(
        obj.was_unearthed,
        "CR 702.84a: was_unearthed flag should be true"
    );
}

// ── Test 10: Loses-all-abilities still gets exiled ───────────────────────────

#[test]
/// Ruling (2008-10-01) — "If that creature loses all its abilities, it will
/// still be exiled at the beginning of the end step." The was_unearthed flag
/// tracks the unearth effects independently of the creature's abilities.
fn test_unearth_loses_abilities_still_exiled_by_replacement() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {B} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let card_id = find_object(&state, "Dregscape Zombie");

    // Activate and resolve unearth.
    let (state, _) = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("UnearthCard should succeed");
    let (mut state, _) = pass_all(state, &[p1, p2]);

    // Simulate "loses all abilities" by clearing keywords (like Humility would do).
    let zombie_id = find_in_zone(&state, "Dregscape Zombie", ZoneId::Battlefield)
        .expect("zombie should be on battlefield");
    state
        .objects
        .get_mut(&zombie_id)
        .unwrap()
        .characteristics
        .keywords
        .clear();

    // Verify was_unearthed is still set even after clearing keywords.
    assert!(
        state.objects.get(&zombie_id).unwrap().was_unearthed,
        "was_unearthed flag should persist even after clearing keywords"
    );

    // Check that the replacement still fires (bounce → exile).
    let owner = state.objects.get(&zombie_id).unwrap().owner;
    let action = mtg_engine::rules::replacement::check_zone_change_replacement(
        &state,
        zombie_id,
        mtg_engine::state::zone::ZoneType::Battlefield,
        mtg_engine::state::zone::ZoneType::Hand,
        owner,
        &std::collections::HashSet::new(),
    );

    assert!(
        matches!(
            &action,
            mtg_engine::rules::replacement::ZoneChangeAction::Redirect {
                to: ZoneId::Exile,
                ..
            }
        ),
        "ruling: replacement should still redirect to exile even after creature loses all abilities"
    );
}

// ── Test 11: Multiplayer — only active player can use unearth ─────────────────

#[test]
/// CR 702.84a — Unearth can only be activated as a sorcery. In multiplayer,
/// non-active players cannot activate unearth during another player's turn.
fn test_unearth_multiplayer_only_active_player() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    // Active player is p1. p2 tries to unearth during p1's turn.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(zombie_in_graveyard(p2)) // p2 has the zombie in their graveyard
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p2 {B} mana.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p2); // p2 has priority

    let card_id = find_object(&state, "Dregscape Zombie");

    // p2 tries to unearth during p1's turn — should fail (sorcery speed check).
    let result = process_command(
        state,
        Command::UnearthCard {
            player: p2,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.84a: non-active player should not be able to activate unearth (sorcery speed)"
    );
}

// ── Test 12: Cannot unearth without sufficient mana ──────────────────────────

#[test]
/// CR 602.2b — Unearth costs must be paid. Attempting to activate unearth
/// without sufficient mana in the pool should fail.
fn test_unearth_requires_mana_payment() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![dregscape_zombie_def()]);

    // p1 has no mana.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(zombie_in_graveyard(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);
    // Do NOT add any mana to p1's pool.

    let card_id = find_object(&state, "Dregscape Zombie");

    let result = process_command(
        state,
        Command::UnearthCard {
            player: p1,
            card: card_id,
        },
    );

    assert!(
        result.is_err(),
        "CR 602.2b: should reject unearth when insufficient mana is available"
    );
}
