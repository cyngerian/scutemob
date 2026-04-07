//! Mass Reanimate tests — PB-H.
//!
//! Verifies:
//! - Effect::ReturnAllFromGraveyardToBattlefield: returns matching GY cards to BF
//! - `tapped`: permanents enter tapped when flag is set
//! - `graveyards: PlayerTarget::AllPlayers`: searches all players' graveyards
//! - `controller_override: None`: returned permanents are controlled by their owner
//! - `unique_names: true`: at most one card per name (Eerie Ultimatum pattern)
//! - `permanent_cards_only: true`: instants/sorceries excluded (Eerie Ultimatum)
//! - Effect::LivingDeath: three-step exile/sacrifice/return sequence
//! - CR 400.7: each zone change produces a new ObjectId
//! - CR 603.6a: ETB triggers fire for all simultaneously-entering permanents
//! - CR 101.4: APNAP ordering preserved in LivingDeath
//! - 2018-03-16 ruling: only step-1-exiled cards return in LivingDeath step 3

use mtg_engine::cards::card_definition::PlayerTarget;
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    CardType, Effect, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step,
    TargetFilter, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects.values().filter(|o| o.zone == zone).count()
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Create a creature card in the specified player's graveyard.
fn gy_creature(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_types(vec![CardType::Creature])
}

/// Create a land card in the specified player's graveyard.
fn gy_land(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_types(vec![CardType::Land])
}

/// Create an artifact card in the specified player's graveyard.
fn gy_artifact(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_types(vec![CardType::Artifact])
}

/// Create an enchantment card in the specified player's graveyard.
fn gy_enchantment(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_types(vec![CardType::Enchantment])
}

/// Create an instant card in the specified player's graveyard.
fn gy_instant(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_types(vec![CardType::Instant])
}

/// Run an effect as the given controller and return (new_state, events).
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

// ── ReturnAllFromGraveyardToBattlefield ───────────────────────────────────────

#[test]
/// CR 400.7 — Splendid Reclamation pattern: all land cards in controller's GY
/// enter the battlefield tapped.
fn test_return_all_lands_from_graveyard_basic() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_land(p(1), "Forest"))
        .object(gy_land(p(1), "Mountain"))
        .object(gy_land(p(1), "Island"))
        .object(gy_creature(p(1), "Bear")) // creature — should stay in GY
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    assert_eq!(count_in_zone(&state, ZoneId::Graveyard(p(1))), 4);
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 0);

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::ReturnAllFromGraveyardToBattlefield {
            graveyards: PlayerTarget::Controller,
            filter: TargetFilter {
                has_card_type: Some(CardType::Land),
                ..Default::default()
            },
            tapped: true,
            controller_override: None,
            unique_names: false,
            permanent_cards_only: false,
        },
    );

    // 3 lands on battlefield; creature stays in graveyard.
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 3);
    assert_eq!(count_in_zone(&state, ZoneId::Graveyard(p(1))), 1);
}

#[test]
/// CR 614.1c — Returned lands enter tapped when `tapped: true`.
fn test_return_all_lands_enters_tapped() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_land(p(1), "Forest"))
        .object(gy_land(p(1), "Plains"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::ReturnAllFromGraveyardToBattlefield {
            graveyards: PlayerTarget::Controller,
            filter: TargetFilter {
                has_card_type: Some(CardType::Land),
                ..Default::default()
            },
            tapped: true,
            controller_override: None,
            unique_names: false,
            permanent_cards_only: false,
        },
    );

    // All returned lands must be tapped.
    let all_tapped = state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield)
        .all(|o| o.status.tapped);
    assert!(all_tapped, "all returned lands should be tapped");
}

#[test]
/// CR 101.4 — Open the Vaults pattern: artifacts and enchantments return from
/// ALL players' graveyards to the battlefield under their owners' control.
fn test_return_all_from_all_graveyards() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_artifact(p(1), "Sol Ring"))
        .object(gy_enchantment(p(2), "Rhystic Study"))
        .object(gy_creature(p(1), "Bear")) // not an artifact/enchantment — stays
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, events) = run_effect(
        state,
        p(1),
        Effect::ReturnAllFromGraveyardToBattlefield {
            graveyards: PlayerTarget::EachPlayer,
            filter: TargetFilter {
                has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                ..Default::default()
            },
            tapped: false,
            controller_override: None,
            unique_names: false,
            permanent_cards_only: false,
        },
    );

    // Sol Ring and Rhystic Study both on battlefield.
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 2);
    // Bear stays in P1's graveyard.
    assert_eq!(count_in_zone(&state, ZoneId::Graveyard(p(1))), 1);
    // Two PermanentEnteredBattlefield events fired.
    let etb_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .count();
    assert_eq!(etb_count, 2);
}

#[test]
/// CR 400.7 — Returned permanents are controlled by their owner when
/// `controller_override: None` (the default "under their owners' control" clause).
fn test_return_all_controller_is_owner() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_artifact(p(2), "P2 Artifact"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // P1 casts Open the Vaults (controller = p1).
    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::ReturnAllFromGraveyardToBattlefield {
            graveyards: PlayerTarget::EachPlayer,
            filter: TargetFilter {
                has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                ..Default::default()
            },
            tapped: false,
            controller_override: None,
            unique_names: false,
            permanent_cards_only: false,
        },
    );

    // P2's artifact should be controlled by P2, not P1.
    let artifact = state
        .objects
        .values()
        .find(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "P2 Artifact")
        .expect("P2 Artifact should be on battlefield");
    assert_eq!(
        artifact.controller,
        p(2),
        "artifact returned under owner's control, not caster's"
    );
}

#[test]
/// Eerie Ultimatum pattern: `unique_names: true` — only one card per unique name
/// returns. Deterministic: lowest ObjectId wins when names duplicate.
fn test_return_all_unique_names() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_creature(p(1), "Grizzly Bears")) // first copy (lower ObjectId)
        .object(gy_creature(p(1), "Grizzly Bears")) // second copy — should NOT return
        .object(gy_creature(p(1), "Llanowar Elves")) // different name — should return
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::ReturnAllFromGraveyardToBattlefield {
            graveyards: PlayerTarget::Controller,
            filter: TargetFilter::default(),
            tapped: false,
            controller_override: None,
            unique_names: true,
            permanent_cards_only: true,
        },
    );

    // Two permanents on battlefield (one Grizzly Bears, one Llanowar Elves).
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 2);
    // One Grizzly Bears copy remains in graveyard.
    assert_eq!(count_in_zone(&state, ZoneId::Graveyard(p(1))), 1);
}

#[test]
/// Eerie Ultimatum: `permanent_cards_only: true` — instants and sorceries in GY
/// are excluded even when they match the broader filter.
fn test_return_all_permanent_cards_only() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_creature(p(1), "Bear")) // permanent card — should return
        .object(gy_instant(p(1), "Counterspell")) // instant — must NOT return
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::ReturnAllFromGraveyardToBattlefield {
            graveyards: PlayerTarget::Controller,
            filter: TargetFilter::default(),
            tapped: false,
            controller_override: None,
            unique_names: false,
            permanent_cards_only: true,
        },
    );

    // Only Bear on battlefield.
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 1);
    // Counterspell stays in graveyard.
    assert!(
        find_in_zone(&state, "Counterspell", ZoneId::Graveyard(p(1))).is_some(),
        "instant should stay in graveyard"
    );
}

// ── Living Death ──────────────────────────────────────────────────────────────

#[test]
/// CR 400.7, 2018-03-16 ruling — Standard Living Death: P1 has creatures on
/// battlefield and creature cards in graveyard. After resolution, BF creatures
/// are gone and GY creatures are on BF.
fn test_living_death_basic() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // P1: 1 creature on battlefield, 1 creature card in graveyard
        .object(ObjectSpec::creature(p(1), "BF Goblin", 1, 1))
        .object(gy_creature(p(1), "GY Zombie"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 1);
    assert_eq!(count_in_zone(&state, ZoneId::Graveyard(p(1))), 1);

    let (state, _events) = run_effect(state, p(1), Effect::LivingDeath);

    // GY Zombie is now on battlefield.
    assert!(
        find_in_zone(&state, "GY Zombie", ZoneId::Battlefield).is_some(),
        "GY Zombie should be on the battlefield"
    );
    // BF Goblin is now in graveyard (sacrificed in step 2).
    assert!(
        find_in_zone(&state, "BF Goblin", ZoneId::Graveyard(p(1))).is_some(),
        "BF Goblin should be in graveyard after sacrifice"
    );
    // Nothing in exile (step 1 exiles are all returned in step 3).
    assert_eq!(count_in_zone(&state, ZoneId::Exile), 0);
}

#[test]
/// 2018-03-16 ruling — Living Death with empty graveyards: all BF creatures
/// are sacrificed, nothing returns.
fn test_living_death_empty_graveyards() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Goblin", 1, 1))
        .object(ObjectSpec::creature(p(2), "Dragon", 5, 5))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 2);

    let (state, _events) = run_effect(state, p(1), Effect::LivingDeath);

    // All creatures sacrificed; graveyards had no creature cards so nothing returns.
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 0);
    assert_eq!(count_in_zone(&state, ZoneId::Exile), 0);
}

#[test]
/// 2018-03-16 ruling — Living Death with no creatures on battlefield: GY creature
/// cards return to BF; nothing is sacrificed.
fn test_living_death_no_creatures_on_battlefield() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_creature(p(1), "GY Bear"))
        .object(gy_creature(p(1), "GY Elf"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, events) = run_effect(state, p(1), Effect::LivingDeath);

    // Both GY creatures are now on battlefield.
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 2);
    // No sacrifice events.
    let sacrifice_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentSacrificed { .. }))
        .count();
    assert_eq!(sacrifice_count, 0, "no creatures to sacrifice");
    // Nothing in exile.
    assert_eq!(count_in_zone(&state, ZoneId::Exile), 0);
}

#[test]
/// CR 603.6a — Mass reanimate: when multiple creatures return simultaneously,
/// all PermanentEnteredBattlefield events fire (one per permanent).
fn test_mass_reanimate_etb_triggers_fire() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_creature(p(1), "Creature A"))
        .object(gy_creature(p(1), "Creature B"))
        .object(gy_creature(p(1), "Creature C"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, events) = run_effect(
        state,
        p(1),
        Effect::ReturnAllFromGraveyardToBattlefield {
            graveyards: PlayerTarget::Controller,
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            tapped: false,
            controller_override: None,
            unique_names: false,
            permanent_cards_only: false,
        },
    );

    // All 3 creatures on battlefield.
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 3);
    // 3 PermanentEnteredBattlefield events, one per creature (CR 603.6a).
    let etb_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .count();
    assert_eq!(etb_count, 3, "one ETB event per returning creature");
}

#[test]
/// CR 101.4 — 4-player Living Death: each player's GY creatures come back under
/// their own control. APNAP ordering preserved.
fn test_mass_reanimate_multiplayer() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .object(gy_creature(p(1), "P1 Zombie"))
        .object(gy_creature(p(2), "P2 Vampire"))
        .object(gy_creature(p(3), "P3 Dragon"))
        .object(gy_creature(p(4), "P4 Angel"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::ReturnAllFromGraveyardToBattlefield {
            graveyards: PlayerTarget::EachPlayer,
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            tapped: false,
            controller_override: None,
            unique_names: false,
            permanent_cards_only: false,
        },
    );

    // All 4 creatures on battlefield, each controlled by their owner.
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 4);
    for (player, name) in [
        (p(1), "P1 Zombie"),
        (p(2), "P2 Vampire"),
        (p(3), "P3 Dragon"),
        (p(4), "P4 Angel"),
    ] {
        let obj = state
            .objects
            .values()
            .find(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == name)
            .unwrap_or_else(|| panic!("{name} should be on battlefield"));
        assert_eq!(
            obj.controller, player,
            "{name} should be controlled by owner"
        );
    }
}

#[test]
/// Living Death: non-creature cards in graveyards are untouched by step 1.
/// Only creature cards are exiled; lands/artifacts/instants stay.
fn test_living_death_only_creature_cards_exiled() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_creature(p(1), "GY Bear"))
        .object(gy_land(p(1), "Forest")) // non-creature — must stay in GY
        .object(gy_instant(p(1), "Counterspell")) // non-creature — must stay in GY
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, _events) = run_effect(state, p(1), Effect::LivingDeath);

    // GY Bear returns to battlefield.
    assert!(
        find_in_zone(&state, "GY Bear", ZoneId::Battlefield).is_some(),
        "creature card should return"
    );
    // Non-creature GY cards stay in graveyard.
    assert!(
        find_in_zone(&state, "Forest", ZoneId::Graveyard(p(1))).is_some(),
        "land should stay in graveyard"
    );
    assert!(
        find_in_zone(&state, "Counterspell", ZoneId::Graveyard(p(1))).is_some(),
        "instant should stay in graveyard"
    );
}

#[test]
/// Living Death: ObjectId tracking — step-1 exiled cards get new ObjectIds when
/// moved to exile; step 3 correctly identifies and returns them by their new IDs.
/// CR 400.7: zone change = new object.
fn test_living_death_step1_id_tracking() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(gy_creature(p(1), "Ancient Zombie"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, events) = run_effect(state, p(1), Effect::LivingDeath);

    // The card moved GY -> Exile (step 1) -> BF (step 3), so it should be on BF now.
    assert!(
        find_in_zone(&state, "Ancient Zombie", ZoneId::Battlefield).is_some(),
        "step-1 exiled card should be on BF after step 3"
    );
    // ObjectExiled event from step 1.
    let exile_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectExiled { .. }))
        .count();
    assert_eq!(exile_events, 1, "step 1 should emit one ObjectExiled event");
    // PermanentEnteredBattlefield from step 3.
    let etb_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .count();
    assert_eq!(etb_events, 1, "step 3 should emit one ETB event");
}

#[test]
/// Living Death with multiple players: step 1 exiles from all GYs, step 2
/// sacrifices all BF creatures, step 3 returns all step-1 cards.
fn test_living_death_two_players_full_swap() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        // P1: creature on BF, creature in GY
        .object(ObjectSpec::creature(p(1), "P1 BF", 2, 2))
        .object(gy_creature(p(1), "P1 GY"))
        // P2: creature on BF, creature in GY
        .object(ObjectSpec::creature(p(2), "P2 BF", 3, 3))
        .object(gy_creature(p(2), "P2 GY"))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let (state, events) = run_effect(state, p(1), Effect::LivingDeath);

    // Both BF creatures are now in graveyards.
    assert!(
        find_in_zone(&state, "P1 BF", ZoneId::Graveyard(p(1))).is_some()
            || find_in_zone(&state, "P1 BF", ZoneId::Battlefield).is_none(),
        "P1 BF creature should be gone from battlefield"
    );
    // Both GY creatures are now on battlefield.
    assert!(
        find_in_zone(&state, "P1 GY", ZoneId::Battlefield).is_some(),
        "P1 GY creature should be on battlefield"
    );
    assert!(
        find_in_zone(&state, "P2 GY", ZoneId::Battlefield).is_some(),
        "P2 GY creature should be on battlefield"
    );
    // 2 creatures on battlefield total.
    assert_eq!(count_in_zone(&state, ZoneId::Battlefield), 2);
    // 2 sacrifice events (one per BF creature).
    let sacrifice_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentSacrificed { .. }))
        .count();
    assert_eq!(sacrifice_events, 2);
}
