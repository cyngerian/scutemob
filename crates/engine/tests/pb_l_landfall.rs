//! Tests for PB-L: Landfall stale-TODO sweep.
//!
//! See `memory/primitives/pb-note-L-collapsed.md` for the Step 0 verdict.
//!
//! Landfall is an **ability word** (CR 207.2c) with no individual entry in the
//! Comprehensive Rules. The engine encodes it as the general triggered-ability
//! `TriggerCondition::WheneverPermanentEntersBattlefield { filter: Land + You }`,
//! dispatched through the same path as other permanent-ETB triggers.
//!
//! These tests verify:
//! 1. Positive: a canonical Landfall card (Lotus Cobra) triggers when its
//!    controller's land enters the battlefield.
//! 2. Non-you-control negative: Landfall does NOT trigger when an opponent's
//!    land enters the battlefield (TargetController::You filter).
//! 3. Non-land negative: Landfall does NOT trigger when a non-land permanent
//!    you control enters (TargetFilter::has_card_type filter).
//! 4. Multiplayer isolation: in a 4-player game, a Landfall creature owned
//!    by p1 does not trigger on lands entering under p2, p3, or p4.
//! 5. Graveyard dispatch: Bloodghast's Landfall fires while it is in its
//!    owner's graveyard (CR 603.3 — abilities with `trigger_zone`
//!    `Some(TriggerZone::Graveyard)`).
//! 6. Simple-fix coverage: Khalni Heart Expedition and Druid Class —
//!    previously stale-TODO cards, now implemented by the PB-L sweep.
//! 7. Druid Class latent-bug regression: Level 1 must trigger on a land
//!    entering, NOT on the Class itself entering (pre-sweep it used
//!    `WhenEntersBattlefield`, which fires only on the Class's own ETB —
//!    silently wrong game state).
//!
//! CR references:
//!   - CR 207.2c — "ability words ... have no special rules meaning and no
//!     individual entries in the Comprehensive Rules"
//!   - CR 603.2  — triggered abilities with "whenever" check once per event
//!   - CR 603.3  — triggers fire regardless of the source's zone; `trigger_zone`
//!     marks abilities that watch events while their source is off the battlefield
//!   - CR 603.6  — zone-change triggers (for "enters the battlefield")
//!
//! Note: AC 3429 refers to "CR 614.12 or relevant subrule." CR 614.12 itself
//! addresses replacement effects modifying permanents entering the battlefield
//! (the ETB-replacement path), which does not apply to Landfall triggers.
//! The relevant subrules for a triggered ability watching land-ETB events are
//! CR 207.2c, 603.2, 603.3, and 603.6.

use mtg_engine::rules::abilities::check_triggers;
use mtg_engine::rules::events::GameEvent;
use mtg_engine::state::game_object::TriggerEvent;
use mtg_engine::{
    all_cards, enrich_spec_from_def, CardDefinition, CardRegistry, GameState, GameStateBuilder,
    ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    let cards = all_cards();
    cards.iter().map(|d| (d.name.clone(), d.clone())).collect()
}

/// Emit a synthetic `PermanentEnteredBattlefield` event for `land_id` owned by
/// `controller`. Returns the list of triggers the engine would collect.
fn triggers_for_land_entering(
    state: &GameState,
    controller: PlayerId,
    land_id: ObjectId,
) -> Vec<mtg_engine::state::stubs::PendingTrigger> {
    let events = vec![GameEvent::PermanentEnteredBattlefield {
        object_id: land_id,
        player: controller,
    }];
    check_triggers(state, &events)
}

// ── 1. Positive: Lotus Cobra triggers on own land ────────────────────────────

/// CR 603.2 / CR 603.6 — Lotus Cobra's Landfall (an ability word, CR 207.2c)
/// triggers when a land its controller controls enters the battlefield.
#[test]
fn test_lotus_cobra_landfall_triggers_on_own_land() {
    let p1 = p(1);
    let defs = load_defs();

    let cobra_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Lotus Cobra").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p1_land = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(cobra_spec)
        .object(p1_land)
        .build()
        .unwrap();

    let cobra_id = find_object(&state, "Lotus Cobra");
    let land_id = find_object(&state, "Forest");

    let triggers = triggers_for_land_entering(&state, p1, land_id);
    let cobra_triggers: Vec<_> = triggers.iter().filter(|t| t.source == cobra_id).collect();
    assert_eq!(
        cobra_triggers.len(),
        1,
        "CR 603.2 + CR 207.2c: Lotus Cobra's Landfall must trigger exactly once \
         when a land its controller controls enters. Got {} triggers.",
        cobra_triggers.len()
    );
    assert_eq!(
        cobra_triggers[0].triggering_event,
        Some(TriggerEvent::AnyPermanentEntersBattlefield),
        "CR 603.2: Landfall dispatches through AnyPermanentEntersBattlefield"
    );
}

// ── 2. Non-you-control negative: opponent's land does NOT trigger my Landfall ──

/// CR 603.2 + TargetController::You — Landfall's "land you control" filter
/// must exclude lands entering under an opponent's control.
///
/// This is the MANDATORY non-you-control negative test from AC 3429.
#[test]
fn test_lotus_cobra_landfall_does_not_trigger_on_opponent_land() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    // p1's Lotus Cobra on the battlefield.
    let cobra_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Lotus Cobra").in_zone(ZoneId::Battlefield),
        &defs,
    );
    // p2's land enters the battlefield.
    let p2_land = ObjectSpec::land(p2, "Island").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(cobra_spec)
        .object(p2_land)
        .build()
        .unwrap();

    let cobra_id = find_object(&state, "Lotus Cobra");
    let p2_land_id = find_object(&state, "Island");

    // p2's land enters — p1's Lotus Cobra's Landfall (with you-control filter)
    // must NOT trigger.
    let triggers = triggers_for_land_entering(&state, p2, p2_land_id);
    let cobra_triggers: Vec<_> = triggers.iter().filter(|t| t.source == cobra_id).collect();
    assert!(
        cobra_triggers.is_empty(),
        "CR 603.2 + TargetController::You: Lotus Cobra's Landfall must NOT trigger \
         when an opponent's land enters the battlefield. Got {} triggers.",
        cobra_triggers.len()
    );
}

// ── 3. Non-land negative: a non-land permanent does NOT trigger Landfall ─────

/// CR 603.2 + TargetFilter::has_card_type — a creature or artifact entering
/// under your control must NOT trigger Landfall.
#[test]
fn test_lotus_cobra_landfall_does_not_trigger_on_non_land() {
    let p1 = p(1);
    let defs = load_defs();

    let cobra_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Lotus Cobra").in_zone(ZoneId::Battlefield),
        &defs,
    );
    // A non-land creature entering under p1's control. Use Jaddi Offshoot —
    // a simple creature with a Landfall trigger of its own; we look only at
    // Lotus Cobra's triggers here.
    let creature_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Jaddi Offshoot").in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(cobra_spec)
        .object(creature_spec)
        .build()
        .unwrap();

    let cobra_id = find_object(&state, "Lotus Cobra");
    let creature_id = find_object(&state, "Jaddi Offshoot");

    // Jaddi Offshoot (a creature) entering — Lotus Cobra's Landfall (land filter)
    // must NOT trigger.
    let triggers = triggers_for_land_entering(&state, p1, creature_id);
    let cobra_triggers: Vec<_> = triggers.iter().filter(|t| t.source == cobra_id).collect();
    assert!(
        cobra_triggers.is_empty(),
        "CR 603.2 + TargetFilter::has_card_type(Land): Lotus Cobra's Landfall \
         must NOT trigger when a non-land permanent enters. Got {} triggers.",
        cobra_triggers.len()
    );
}

// ── 4. Multiplayer isolation: Landfall is per-controller in a 4-player game ──

/// CR 207.2c + CR 603.2 + TargetController::You — the MANDATORY
/// multiplayer-isolation test from AC 3429.
///
/// In a 4-player game, p1's Lotus Cobra must trigger only when p1's lands
/// enter the battlefield. p2, p3, and p4 each playing a land under their own
/// control must NOT trigger p1's Landfall ability.
#[test]
fn test_landfall_multiplayer_isolation_4p() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = load_defs();

    let cobra_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Lotus Cobra").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p1_land = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);
    let p2_land = ObjectSpec::land(p2, "Island").in_zone(ZoneId::Battlefield);
    let p3_land = ObjectSpec::land(p3, "Swamp").in_zone(ZoneId::Battlefield);
    let p4_land = ObjectSpec::land(p4, "Mountain").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(cobra_spec)
        .object(p1_land)
        .object(p2_land)
        .object(p3_land)
        .object(p4_land)
        .build()
        .unwrap();

    let cobra_id = find_object(&state, "Lotus Cobra");
    let p1_land_id = find_object(&state, "Forest");
    let p2_land_id = find_object(&state, "Island");
    let p3_land_id = find_object(&state, "Swamp");
    let p4_land_id = find_object(&state, "Mountain");

    // Only p1's land must trigger p1's Lotus Cobra.
    let cases: &[(PlayerId, ObjectId, bool, &str)] = &[
        (p1, p1_land_id, true, "p1's own Forest"),
        (p2, p2_land_id, false, "p2's Island"),
        (p3, p3_land_id, false, "p3's Swamp"),
        (p4, p4_land_id, false, "p4's Mountain"),
    ];

    for (controller, land_id, should_trigger, label) in cases.iter().copied() {
        let triggers = triggers_for_land_entering(&state, controller, land_id);
        let cobra_hits: Vec<_> = triggers.iter().filter(|t| t.source == cobra_id).collect();
        if should_trigger {
            assert_eq!(
                cobra_hits.len(),
                1,
                "CR 603.2 multiplayer isolation ({}): p1's Lotus Cobra must trigger \
                 exactly once for {}. Got {} triggers.",
                label,
                label,
                cobra_hits.len()
            );
        } else {
            assert!(
                cobra_hits.is_empty(),
                "CR 207.2c + TargetController::You multiplayer isolation ({}): \
                 p1's Lotus Cobra must NOT trigger for {}. Got {} triggers.",
                label,
                label,
                cobra_hits.len()
            );
        }
    }
}

// ── 5. Graveyard dispatch: Bloodghast's Landfall fires from the graveyard ─────

/// CR 603.3 — abilities with `trigger_zone: Some(TriggerZone::Graveyard)` fire
/// while their source is in the graveyard. Bloodghast is the canonical example
/// (Landfall → return to battlefield from graveyard).
#[test]
fn test_bloodghast_landfall_triggers_from_graveyard() {
    let p1 = p(1);
    let defs = load_defs();

    // Bloodghast in p1's graveyard. The graveyard-zone Landfall dispatch in
    // collect_graveyard_carddef_triggers looks up the source object's card_id
    // in state.card_registry, so we must set both.
    let bloodghast_card_id = defs.get("Bloodghast").unwrap().card_id.clone();
    let bloodghast_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Bloodghast")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(bloodghast_card_id),
        &defs,
    );
    let p1_land = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);

    let defs_vec: Vec<CardDefinition> = defs.values().cloned().collect();
    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(defs_vec))
        .object(bloodghast_spec)
        .object(p1_land)
        .build()
        .unwrap();

    let bloodghast_id = find_object(&state, "Bloodghast");
    let p1_land_id = find_object(&state, "Forest");

    // A land entering under p1's control must trigger Bloodghast's graveyard Landfall.
    let triggers = triggers_for_land_entering(&state, p1, p1_land_id);
    let bloodghast_triggers: Vec<_> = triggers
        .iter()
        .filter(|t| t.source == bloodghast_id)
        .collect();
    assert_eq!(
        bloodghast_triggers.len(),
        1,
        "CR 603.3 / TriggerZone::Graveyard: Bloodghast's Landfall must fire from \
         the graveyard when a land its controller controls enters. Got {} triggers.",
        bloodghast_triggers.len()
    );
}

/// CR 603.3 + TargetController::You from graveyard — Bloodghast must NOT
/// trigger when an opponent's land enters the battlefield (the "you control"
/// filter applies even from graveyard dispatch).
#[test]
fn test_bloodghast_landfall_does_not_trigger_on_opponent_land() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let bloodghast_card_id = defs.get("Bloodghast").unwrap().card_id.clone();
    let bloodghast_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Bloodghast")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(bloodghast_card_id),
        &defs,
    );
    let p2_land = ObjectSpec::land(p2, "Island").in_zone(ZoneId::Battlefield);

    let defs_vec: Vec<CardDefinition> = defs.values().cloned().collect();
    let state = GameStateBuilder::four_player()
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(defs_vec))
        .object(bloodghast_spec)
        .object(p2_land)
        .build()
        .unwrap();

    let bloodghast_id = find_object(&state, "Bloodghast");
    let p2_land_id = find_object(&state, "Island");

    let triggers = triggers_for_land_entering(&state, p2, p2_land_id);
    let bloodghast_triggers: Vec<_> = triggers
        .iter()
        .filter(|t| t.source == bloodghast_id)
        .collect();
    assert!(
        bloodghast_triggers.is_empty(),
        "CR 603.3 + TargetController::You: Bloodghast's Landfall from the \
         graveyard must NOT trigger on an opponent's land. Got {} triggers.",
        bloodghast_triggers.len()
    );
}

// ── 6. Simple-fix coverage: PB-L authored Landfall abilities ──────────────────

/// CR 603.2 / CR 207.2c — Khalni Heart Expedition's Landfall was a stale-TODO
/// case (blocked on a mythical `TriggerCondition::WheneverLandEntersBattlefield`).
/// PB-L authored it using the standard pattern.
#[test]
fn test_khalni_heart_expedition_landfall_triggers() {
    let p1 = p(1);
    let defs = load_defs();

    let khalni_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Khalni Heart Expedition").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p1_land = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(khalni_spec)
        .object(p1_land)
        .build()
        .unwrap();

    let khalni_id = find_object(&state, "Khalni Heart Expedition");
    let land_id = find_object(&state, "Forest");

    let triggers = triggers_for_land_entering(&state, p1, land_id);
    let khalni_triggers: Vec<_> = triggers.iter().filter(|t| t.source == khalni_id).collect();
    assert_eq!(
        khalni_triggers.len(),
        1,
        "PB-L / CR 603.2: Khalni Heart Expedition's Landfall must trigger on \
         a land entering its controller's battlefield. Got {} triggers.",
        khalni_triggers.len()
    );
}

// ── 7. Druid Class latent-bug regression ──────────────────────────────────────

/// PB-L latent-bug fix — Druid Class Level 1 previously used
/// `TriggerCondition::WhenEntersBattlefield`, which fires only on the Class's
/// OWN entry, not on land entries. This test verifies:
///   (a) Druid Class triggers when a land enters its controller's side
///   (b) Druid Class does NOT trigger merely because Druid Class itself entered
///       (no land event present).
#[test]
fn test_druid_class_landfall_triggers_on_land_not_on_self_etb() {
    let p1 = p(1);
    let defs = load_defs();

    let class_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Druid Class").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p1_land = ObjectSpec::land(p1, "Forest").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(class_spec)
        .object(p1_land)
        .build()
        .unwrap();

    let class_id = find_object(&state, "Druid Class");
    let land_id = find_object(&state, "Forest");

    // (a) Land entering triggers Druid Class Level 1.
    let land_triggers = triggers_for_land_entering(&state, p1, land_id);
    let class_land_triggers: Vec<_> = land_triggers
        .iter()
        .filter(|t| t.source == class_id)
        .collect();
    assert_eq!(
        class_land_triggers.len(),
        1,
        "PB-L regression guard: Druid Class Level 1 must trigger on a land \
         entering its controller's battlefield (CR 603.2 + TargetFilter Land+You). \
         Got {} triggers.",
        class_land_triggers.len()
    );

    // (b) Druid Class itself entering — simulate by firing a
    //     PermanentEnteredBattlefield event for the Class itself. The Class
    //     must NOT self-trigger, because its trigger is now filtered to Land+You.
    let class_self_triggers = triggers_for_land_entering(&state, p1, class_id);
    let class_self_hits: Vec<_> = class_self_triggers
        .iter()
        .filter(|t| {
            t.source == class_id
                && t.triggering_event == Some(TriggerEvent::AnyPermanentEntersBattlefield)
        })
        .collect();
    assert!(
        class_self_hits.is_empty(),
        "PB-L latent-bug fix: Druid Class Level 1 must NOT trigger on the Class's \
         own ETB (its Landfall trigger requires a land, not an enchantment). \
         Got {} spurious triggers.",
        class_self_hits.len()
    );
}

/// PB-L simple-fix coverage — Omnath, Locus of Rage's Landfall now creates
/// a 5/5 red+green Elemental token. Verify the trigger fires.
#[test]
fn test_omnath_locus_of_rage_landfall_triggers() {
    let p1 = p(1);
    let defs = load_defs();

    let omnath_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Omnath, Locus of Rage").in_zone(ZoneId::Battlefield),
        &defs,
    );
    let p1_land = ObjectSpec::land(p1, "Mountain").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(vec![]))
        .object(omnath_spec)
        .object(p1_land)
        .build()
        .unwrap();

    let omnath_id = find_object(&state, "Omnath, Locus of Rage");
    let land_id = find_object(&state, "Mountain");

    let triggers = triggers_for_land_entering(&state, p1, land_id);
    let omnath_triggers: Vec<_> = triggers.iter().filter(|t| t.source == omnath_id).collect();
    assert_eq!(
        omnath_triggers.len(),
        1,
        "PB-L / CR 603.2: Omnath, Locus of Rage's Landfall must trigger exactly \
         once when a land its controller controls enters. Got {} triggers.",
        omnath_triggers.len()
    );
}
