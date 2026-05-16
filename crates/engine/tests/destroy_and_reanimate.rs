//! DestroyAndReanimate tests — PB-LS6, Issue L02.
//!
//! Verifies:
//! - Effect::DestroyAndReanimate: destroys each declared target via the standard
//!   destroy pipeline, then reanimates any resulting graveyard cards under the
//!   activating player's control (Sorin, Lord of Innistrad -6).
//! - Indestructible targets survive and are not reanimated (CR 702.12b).
//! - Token targets are destroyed but NOT reanimated (CR 704.5d — tokens cease to
//!   exist when they leave the battlefield; the is_token flag gates phase 2).
//! - Replacement effects that redirect to exile prevent reanimation (CR 614.1a).
//! - ETB triggered abilities from a reanimated permanent's CardDefinition are queued
//!   (CR 603.6a — queue_carddef_etb_triggers is called on the reanimate path).
//! - The activating player's controller identity overrides the original controller.
//!
//! CR refs:
//!   CR 614.1a — replacement effect, redirect to exile
//!   CR 702.12b — indestructible
//!   CR 704.5d — tokens cease to exist when leaving battlefield
//!   CR 603.6a — ETB triggers fire on zone entry

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::replacement_effect::{
    ObjectFilter, ReplacementEffect, ReplacementModification, ReplacementTrigger,
};
use mtg_engine::{
    AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry, CardType, Effect,
    EffectAmount, EffectDuration, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaCost,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, ReplacementId, SpellTarget, Step, Target,
    TriggerCondition, TypeLine, ZoneId, ZoneType,
};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// A creature ObjectSpec with a non-None card_id so phase-2 (reanimate) is eligible.
/// The card_id is derived from the name to be unique per creature.
fn card_creature(owner: PlayerId, name: &str, power: i32, toughness: i32) -> ObjectSpec {
    ObjectSpec::creature(owner, name, power, toughness)
        .with_card_id(CardId(name.to_lowercase().replace(' ', "-")))
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_by_name_opt(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
}

fn zone_of(state: &GameState, id: ObjectId) -> ZoneId {
    state
        .objects
        .get(&id)
        .map(|o| o.zone)
        .unwrap_or(ZoneId::Exile)
}

fn controller_of(state: &GameState, name: &str) -> PlayerId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(_, o)| o.controller)
        .unwrap_or_else(|| panic!("'{}' not found", name))
}

/// Run DestroyAndReanimate with an explicit target list (DeclaredTarget indices).
fn run_destroy_and_reanimate(
    mut state: GameState,
    controller: PlayerId,
    target_ids: &[ObjectId],
    cant_be_regenerated: bool,
) -> (GameState, Vec<GameEvent>) {
    let source = ObjectId(0);
    let target_specs: Vec<SpellTarget> = target_ids
        .iter()
        .map(|&id| SpellTarget {
            target: Target::Object(id),
            zone_at_cast: Some(ZoneId::Battlefield),
        })
        .collect();
    let mut ctx = EffectContext::new(controller, source, target_specs);

    let targets: Vec<CardEffectTarget> = (0..target_ids.len())
        .map(|i| CardEffectTarget::DeclaredTarget { index: i })
        .collect();

    let effect = Effect::DestroyAndReanimate {
        targets,
        cant_be_regenerated,
    };
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
}

// ── Test 1: Basic destroy-and-reanimate ───────────────────────────────────────

/// CR 701.8 — DestroyAndReanimate destroys a creature target and reanimates the
/// resulting graveyard card under the activating player's control.
#[test]
fn test_l02_destroy_and_reanimate_basic() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(card_creature(p(2), "Opponent Creature", 3, 3))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let target_id = find_by_name(&state, "Opponent Creature");
    let (state_after, events) = run_destroy_and_reanimate(state, p(1), &[target_id], false);

    // The original object should no longer be on the battlefield.
    assert_ne!(
        zone_of(&state_after, target_id),
        ZoneId::Battlefield,
        "CR 701.8: destroyed creature should leave the battlefield"
    );

    // A CreatureDied event should have fired.
    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        died,
        "CR 701.8: CreatureDied event should fire for destroyed creature"
    );

    // The creature should now be on the battlefield again (reanimated).
    let reanimated_on_bf = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Opponent Creature" && o.zone == ZoneId::Battlefield);
    assert!(
        reanimated_on_bf,
        "creature should be reanimated to the battlefield"
    );

    // PermanentEnteredBattlefield event should be present.
    let etb = events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }));
    assert!(
        etb,
        "PermanentEnteredBattlefield event should fire on reanimate"
    );
}

// ── Test 2: Returns under activating player's control ─────────────────────────

/// CR 701.8 / Sorin -6 oracle text — "return each card put into a graveyard this
/// way to the battlefield under your control." The reanimated permanent is
/// controlled by the activating player (P1), not the original controller (P2).
#[test]
fn test_l02_reanimate_under_your_control() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(card_creature(p(2), "Enemy Knight", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let target_id = find_by_name(&state, "Enemy Knight");
    let (state_after, _events) = run_destroy_and_reanimate(state, p(1), &[target_id], false);

    // Find the reanimated copy (new ObjectId, same name, on battlefield).
    let reanimated_controller = state_after
        .objects
        .values()
        .find(|o| o.characteristics.name == "Enemy Knight" && o.zone == ZoneId::Battlefield)
        .map(|o| o.controller);

    assert_eq!(
        reanimated_controller,
        Some(p(1)),
        "CR Sorin -6: reanimated creature should be under the activating player's (P1) control, not P2"
    );
}

// ── Test 3: Token is destroyed but NOT reanimated ─────────────────────────────

/// CR 704.5d — Token objects cease to exist when they leave the battlefield.
/// After a token is destroyed, it exists in the graveyard for state-based action
/// processing but has is_token = true, so phase 2 skips it.
#[test]
fn test_l02_token_destroyed_not_reanimated() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(2), "Goblin Token", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    // Mark the creature as a token.
    let token_id = find_by_name(&state, "Goblin Token");
    if let Some(obj) = state.objects.get_mut(&token_id) {
        obj.is_token = true;
        obj.card_id = None; // tokens have no card_id
    }

    let (state_after, events) = run_destroy_and_reanimate(state, p(1), &[token_id], false);

    // Token should have been destroyed (CreatureDied event).
    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        died,
        "CR 704.5d: token should be destroyed (CreatureDied event)"
    );

    // Token must NOT appear on the battlefield after phase 2.
    let token_on_bf = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Goblin Token" && o.zone == ZoneId::Battlefield);
    assert!(
        !token_on_bf,
        "CR 704.5d: a destroyed token must not be reanimated (is_token gate)"
    );

    // No PermanentEnteredBattlefield event from the reanimate phase.
    let entered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .count();
    assert_eq!(
        entered_count, 0,
        "no PermanentEnteredBattlefield from reanimate for tokens"
    );
}

// ── Test 4: Replacement-redirected-to-exile prevents reanimate ─────────────────

/// CR 614.1a — A replacement effect that exiles the card instead of putting it into
/// the graveyard means the card never enters a graveyard, so phase 2 finds nothing
/// in the graveyard to reanimate.
#[test]
fn test_l02_replacement_redirect_to_exile_not_reanimated() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(card_creature(p(2), "Doomed Creature", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let doomed_id = find_by_name(&state, "Doomed Creature");

    // Install a global replacement effect: any battlefield→graveyard goes to exile instead
    // (simulating Rest in Peace, CR 614.1a).
    state.replacement_effects.push_back(ReplacementEffect {
        id: ReplacementId(9001),
        controller: p(1),
        source: None,
        duration: EffectDuration::Indefinite,
        is_self_replacement: false,
        trigger: ReplacementTrigger::WouldChangeZone {
            from: Some(ZoneType::Battlefield),
            to: ZoneType::Graveyard,
            filter: ObjectFilter::Any,
        },
        modification: ReplacementModification::RedirectToZone(ZoneType::Exile),
    });

    let (state_after, events) = run_destroy_and_reanimate(state, p(1), &[doomed_id], false);

    // Object should be in exile (redirected by replacement effect).
    let in_exile = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Doomed Creature" && o.zone == ZoneId::Exile);
    assert!(
        in_exile,
        "CR 614.1a: replacement effect should redirect to exile"
    );

    // Must NOT appear on battlefield (reanimate phase must be skipped).
    let on_bf = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Doomed Creature" && o.zone == ZoneId::Battlefield);
    assert!(
        !on_bf,
        "CR 614.1a: a creature redirected to exile must not be reanimated"
    );

    // ObjectExiled event (not CreatureDied) should be present.
    let exiled_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectExiled { .. }));
    assert!(
        exiled_event,
        "ObjectExiled event should fire when redirected to exile"
    );

    // No PermanentEnteredBattlefield for the reanimate phase.
    let entered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .count();
    assert_eq!(
        entered_count, 0,
        "no reanimate ETB event when card went to exile"
    );
}

// ── Test 5: Indestructible target survives (no reanimate) ─────────────────────

/// CR 702.12b — Indestructible permanents cannot be destroyed. Phase 1 skips them;
/// they never enter the graveyard, so phase 2 has nothing to reanimate.
#[test]
fn test_l02_indestructible_target_survives() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(2), "Indestructible Angel", 4, 4)
                .with_keyword(KeywordAbility::Indestructible),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let angel_id = find_by_name(&state, "Indestructible Angel");
    let (state_after, events) = run_destroy_and_reanimate(state, p(1), &[angel_id], false);

    // Angel should still be on the battlefield.
    assert_eq!(
        zone_of(&state_after, angel_id),
        ZoneId::Battlefield,
        "CR 702.12b: indestructible creature must survive DestroyAndReanimate"
    );

    // No CreatureDied event.
    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(
        !died,
        "CR 702.12b: no CreatureDied event for indestructible creature"
    );

    // No PermanentEnteredBattlefield (nothing to reanimate).
    let entered = events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }));
    assert!(
        !entered,
        "CR 702.12b: no reanimate ETB for indestructible that never died"
    );
}

// ── Test 6: Multiple targets — partial indestructible ─────────────────────────

/// CR 701.8 — DestroyAndReanimate with 3 targets: one normal creature (Normal Bear), one
/// indestructible creature (Adamantine Titan), one own creature (Own Goblin). The two normal
/// creatures are destroyed and reanimated; the indestructible creature is skipped entirely.
#[test]
fn test_l02_multiple_targets_partial_indestructible() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(card_creature(p(2), "Normal Bear", 2, 2))
        .object(
            card_creature(p(2), "Adamantine Titan", 6, 6)
                .with_keyword(KeywordAbility::Indestructible),
        )
        .object(card_creature(p(1), "Own Goblin", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let bear_id = find_by_name(&state, "Normal Bear");
    let titan_id = find_by_name(&state, "Adamantine Titan");
    let goblin_id = find_by_name(&state, "Own Goblin");

    let (state_after, events) =
        run_destroy_and_reanimate(state, p(1), &[bear_id, titan_id, goblin_id], false);

    // Normal Bear: destroyed and reanimated.
    let bear_on_bf = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Normal Bear" && o.zone == ZoneId::Battlefield);
    assert!(
        bear_on_bf,
        "Normal Bear should be reanimated on battlefield"
    );
    assert_eq!(
        controller_of(&state_after, "Normal Bear"),
        p(1),
        "Normal Bear should be under activating player P1's control after reanimate"
    );

    // Titan: still on battlefield, unchanged.
    assert_eq!(
        zone_of(&state_after, titan_id),
        ZoneId::Battlefield,
        "CR 702.12b: Adamantine Titan (indestructible) must remain on battlefield"
    );

    // Own Goblin: destroyed and reanimated (own creature can also be targeted).
    let goblin_on_bf = state_after
        .objects
        .values()
        .any(|o| o.characteristics.name == "Own Goblin" && o.zone == ZoneId::Battlefield);
    assert!(
        goblin_on_bf,
        "Own Goblin should be reanimated on battlefield"
    );

    // CreatureDied events: 2 (Bear + Goblin, not Titan).
    let died_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CreatureDied { .. }))
        .count();
    assert_eq!(
        died_count, 2,
        "two creatures should die (not the indestructible one)"
    );

    // PermanentEnteredBattlefield events: 2 (Bear + Goblin reanimated).
    let entered_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }))
        .count();
    assert_eq!(
        entered_count, 2,
        "two PermanentEnteredBattlefield events for the two reanimates"
    );

    // No event for the Titan (neither died nor entered).
    let find_opt = find_by_name_opt(&state_after, "Adamantine Titan");
    assert!(find_opt.is_some(), "Titan must still exist");
}

// ── Test 7: ETB triggered ability fires for reanimated permanent ──────────────

/// CR 603.6a — "A permanent's ability that triggers 'when/whenever [it] enters
/// the battlefield' triggers when that object enters the battlefield."
///
/// The reanimate phase calls queue_carddef_etb_triggers, which queues ETB
/// triggered abilities from the permanent's CardDefinition. This test verifies
/// that a creature with a WhenEntersBattlefield triggered ability has that
/// trigger queued (added to state.pending_triggers) after being destroyed and
/// reanimated by DestroyAndReanimate.
#[test]
fn test_l02_destroy_and_reanimate_runs_etb() {
    // Build a CardDefinition for a creature with an ETB "gain 1 life" triggered ability.
    let etb_card_id = CardId("etb-creature".to_string());
    let etb_def = CardDefinition {
        card_id: etb_card_id.clone(),
        name: "ETB Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "When this creature enters the battlefield, gain 1 life.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
    };
    let registry = CardRegistry::new(vec![etb_def]);

    // Build the game state with the registry so queue_carddef_etb_triggers can look up the def.
    let state = GameStateBuilder::new()
        .with_registry(registry)
        .add_player(p(1))
        .add_player(p(2))
        // P2 owns the ETB creature; P1 will destroy-and-reanimate it.
        .object(ObjectSpec::creature(p(2), "ETB Creature", 2, 2).with_card_id(etb_card_id.clone()))
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    let target_id = find_by_name(&state, "ETB Creature");

    // Run DestroyAndReanimate via execute_effect (not process_command) so we can
    // inspect state.pending_triggers directly after the effect resolves.
    let (state_after, events) = run_destroy_and_reanimate(state, p(1), &[target_id], false);

    // Phase 1: creature must have been destroyed.
    let died = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { .. }));
    assert!(died, "CR 701.8: CreatureDied event must fire in phase 1");

    // Phase 2: creature must be reanimated (PermanentEnteredBattlefield event).
    let entered = events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }));
    assert!(
        entered,
        "creature must be reanimated (PermanentEnteredBattlefield event)"
    );

    // CR 603.6a: the reanimate path calls queue_carddef_etb_triggers, which pushes a
    // PendingTrigger for the WhenEntersBattlefield ability. Assert it was queued.
    assert!(
        !state_after.pending_triggers.is_empty(),
        "CR 603.6a: queue_carddef_etb_triggers must queue the ETB triggered ability \
         when a creature with a WhenEntersBattlefield ability is reanimated. \
         Got 0 pending triggers."
    );

    // Verify the trigger is for the reanimated creature (CardDefETB kind).
    use mtg_engine::state::stubs::PendingTriggerKind;
    let has_carddef_etb = state_after
        .pending_triggers
        .iter()
        .any(|t| t.kind == PendingTriggerKind::CardDefETB);
    assert!(
        has_carddef_etb,
        "CR 603.6a: the queued trigger must be a CardDefETB pending trigger"
    );
}
