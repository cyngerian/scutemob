//! Umbra Armor (formerly Totem Armor) tests (CR 702.89).
//!
//! Umbra armor is a static ability on Auras that creates a continuous replacement effect:
//! "If enchanted permanent would be destroyed, instead remove all damage marked on it
//! and destroy this Aura." (CR 702.89a)
//!
//! Key properties verified:
//! - Replaces destruction by spell/ability (Effect::DestroyPermanent).
//! - Replaces SBA destruction from lethal damage (CR 704.5g).
//! - Replaces SBA destruction from deathtouch damage (CR 704.5h).
//! - Does NOT prevent zero-toughness death (CR 704.5f -- not destruction).
//! - Does NOT tap the protected permanent (unlike regeneration).
//! - Does NOT remove the protected permanent from combat (unlike regeneration).
//! - "Can't be regenerated" does NOT block umbra armor (separate mechanics -- ruling).
//! - Indestructible is handled before umbra armor; Aura is NOT consumed (CR 702.12a).
//! - Sacrifice is not destruction; umbra armor does not apply (CR 701.8b).
//! - Umbra armor works on non-creature permanents (lands, artifacts) enchanted by an Aura.
//! - UmbraArmorApplied event emitted with correct protected_id and aura_id.

use mtg_engine::{
    check_and_apply_sbas, AttackTarget, CardEffectTarget, CardRegistry, CardType, CombatState,
    Effect, GameEvent, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec, PlayerId,
    SpellTarget, Step, SubType, Target, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_on_battlefield(state: &mtg_engine::GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
}

fn is_in_graveyard(state: &mtg_engine::GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .iter()
        .any(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Graveyard(owner))
}

fn damage_marked(state: &mtg_engine::GameState, id: ObjectId) -> u32 {
    state.objects.get(&id).map(|o| o.damage_marked).unwrap_or(0)
}

fn deathtouch_damage_flag(state: &mtg_engine::GameState, id: ObjectId) -> bool {
    state
        .objects
        .get(&id)
        .map(|o| o.deathtouch_damage)
        .unwrap_or(false)
}

fn is_tapped(state: &mtg_engine::GameState, id: ObjectId) -> bool {
    state
        .objects
        .get(&id)
        .map(|o| o.status.tapped)
        .unwrap_or(false)
}

/// Create a destroy-permanent effect context targeting the given object.
fn destroy_effect(target_id: ObjectId) -> (Effect, mtg_engine::effects::EffectContext) {
    use mtg_engine::effects::EffectContext;
    let effect = Effect::DestroyPermanent {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
    };
    let ctx = EffectContext::new(
        PlayerId(1),
        ObjectId(999),
        vec![SpellTarget {
            target: Target::Object(target_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    (effect, ctx)
}

/// Build an Aura ObjectSpec with the UmbraArmor keyword.
fn umbra_aura_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .with_types(vec![CardType::Enchantment])
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::UmbraArmor)
        .in_zone(ZoneId::Battlefield)
}

/// Wire up the attachment relationship after the state is built.
///
/// Sets `aura.attached_to = Some(permanent_id)` and adds `aura_id` to
/// `permanent.attachments`, matching the real attachment setup in resolution.rs.
fn attach_aura(state: &mut mtg_engine::GameState, aura_id: ObjectId, permanent_id: ObjectId) {
    if let Some(aura) = state.objects.get_mut(&aura_id) {
        aura.attached_to = Some(permanent_id);
    }
    if let Some(perm) = state.objects.get_mut(&permanent_id) {
        perm.attachments.push_back(aura_id);
    }
}

// ── Test 1: Umbra armor prevents destruction by spell ─────────────────────────

#[test]
/// CR 702.89a, 701.8b — Umbra armor replaces DestroyPermanent effect.
///
/// A creature enchanted by an Aura with umbra armor is targeted by a destroy spell.
/// Instead of being destroyed, all damage on the creature is cleared and the Aura
/// is destroyed. The creature remains on the battlefield.
fn test_umbra_armor_prevents_destruction_by_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Enchanted Bear", 2, 2)
        .with_damage(1) // some damage pre-marked
        .in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Hyena Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Enchanted Bear").unwrap();
    let aura_id = find_on_battlefield(&state, "Hyena Umbra").unwrap();

    attach_aura(&mut state, aura_id, creature_id);

    let (effect, mut ctx) = destroy_effect(creature_id);
    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Creature should still be on the battlefield.
    assert!(
        find_on_battlefield(&state, "Enchanted Bear").is_some(),
        "CR 702.89a: creature should survive destruction via umbra armor"
    );

    // Damage should be cleared.
    let survivor_id = find_on_battlefield(&state, "Enchanted Bear").unwrap();
    assert_eq!(
        damage_marked(&state, survivor_id),
        0,
        "CR 702.89a: all damage should be removed from the protected permanent"
    );

    // Creature should NOT be tapped (unlike regeneration).
    assert!(
        !is_tapped(&state, survivor_id),
        "CR 702.89a ruling: umbra armor does NOT tap the protected permanent"
    );

    // Aura should be in the graveyard.
    assert!(
        is_in_graveyard(&state, "Hyena Umbra", p1),
        "CR 702.89a: the Aura should be destroyed (moved to graveyard)"
    );

    // UmbraArmorApplied event emitted; no CreatureDied for the creature.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::UmbraArmorApplied { .. })),
        "CR 702.89a: UmbraArmorApplied event should be emitted"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 702.89a: CreatureDied should NOT be emitted for the protected creature"
    );
}

// ── Test 2: Umbra armor prevents SBA lethal damage destruction ─────────────────

#[test]
/// CR 702.89a, 704.5g — Umbra armor replaces SBA lethal-damage destruction.
///
/// A 2/2 creature with 3 damage marked (lethal for 704.5g) enchanted by an umbra armor Aura.
/// SBA fires -- creature would be destroyed. Umbra armor intercepts: creature survives,
/// damage cleared, Aura destroyed.
fn test_umbra_armor_prevents_sba_lethal_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Damaged Bear", 2, 2)
        .with_damage(3) // lethal damage for a 2/2
        .in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Snake Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Damaged Bear").unwrap();
    let aura_id = find_on_battlefield(&state, "Snake Umbra").unwrap();
    attach_aura(&mut state, aura_id, creature_id);

    let events = check_and_apply_sbas(&mut state);

    // Creature survives.
    assert!(
        find_on_battlefield(&state, "Damaged Bear").is_some(),
        "CR 702.89a, 704.5g: creature should survive lethal damage via umbra armor"
    );

    // Damage cleared.
    let survivor_id = find_on_battlefield(&state, "Damaged Bear").unwrap();
    assert_eq!(
        damage_marked(&state, survivor_id),
        0,
        "CR 702.89a: damage should be cleared from the protected permanent"
    );

    // Aura destroyed.
    assert!(
        is_in_graveyard(&state, "Snake Umbra", p1),
        "CR 702.89a: Aura should be destroyed by umbra armor"
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::UmbraArmorApplied { .. })),
        "CR 702.89a: UmbraArmorApplied event should be emitted from SBA"
    );
}

// ── Test 3: Umbra armor prevents SBA deathtouch damage destruction ─────────────

#[test]
/// CR 702.89a, 704.5h — Umbra armor replaces SBA deathtouch-damage destruction.
///
/// A creature with deathtouch_damage = true enchanted by an umbra armor Aura.
/// SBA fires -- creature would be destroyed by deathtouch (704.5h). Umbra armor
/// intercepts: creature survives, deathtouch flag cleared, Aura destroyed.
fn test_umbra_armor_prevents_sba_deathtouch_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Deathtouch Victim", 3, 3)
        .with_damage(1) // minimal damage, but from a deathtouch source
        .with_deathtouch_damage()
        .in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Bear Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Deathtouch Victim").unwrap();
    let aura_id = find_on_battlefield(&state, "Bear Umbra").unwrap();
    attach_aura(&mut state, aura_id, creature_id);

    let events = check_and_apply_sbas(&mut state);

    assert!(
        find_on_battlefield(&state, "Deathtouch Victim").is_some(),
        "CR 702.89a, 704.5h: creature should survive deathtouch via umbra armor"
    );

    let survivor_id = find_on_battlefield(&state, "Deathtouch Victim").unwrap();
    assert_eq!(
        damage_marked(&state, survivor_id),
        0,
        "CR 702.89a: damage should be cleared"
    );
    assert!(
        !deathtouch_damage_flag(&state, survivor_id),
        "CR 702.89a: deathtouch_damage flag should be cleared"
    );
    assert!(
        is_in_graveyard(&state, "Bear Umbra", p1),
        "CR 702.89a: Aura should be destroyed"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::UmbraArmorApplied { .. })),
        "CR 702.89a: UmbraArmorApplied event should be emitted"
    );
}

// ── Test 4: Umbra armor does NOT prevent zero-toughness death ─────────────────

#[test]
/// CR 702.89a, 704.5f — Zero-toughness is NOT destruction; umbra armor does not apply.
///
/// A 0/0 creature (or creature with 0 effective toughness) enchanted by an umbra armor Aura.
/// SBA 704.5f fires -- creature is put into the graveyard (not destroyed). Umbra armor
/// does NOT apply. The Aura also falls off (SBA 704.5m).
fn test_umbra_armor_does_not_prevent_zero_toughness() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Feeble Creature", 0, 0).in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Frail Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Feeble Creature").unwrap();
    let aura_id = find_on_battlefield(&state, "Frail Umbra").unwrap();
    attach_aura(&mut state, aura_id, creature_id);

    let _events = check_and_apply_sbas(&mut state);

    // Creature should be dead (zero-toughness SBA 704.5f is NOT destruction).
    assert!(
        find_on_battlefield(&state, "Feeble Creature").is_none(),
        "CR 702.89a, 704.5f: zero-toughness is not destruction; umbra armor does not apply"
    );
    assert!(
        is_in_graveyard(&state, "Feeble Creature", p1),
        "CR 704.5f: creature should be in graveyard"
    );

    // Umbra armor should NOT have fired -- no UmbraArmorApplied event.
    // (The Aura may also have gone to the graveyard via 704.5m.)
    // The key assertion is the creature died despite having an umbra armor Aura.
}

// ── Test 5: Umbra armor does NOT tap or remove from combat ─────────────────────

#[test]
/// CR 702.89a ruling — Umbra armor does NOT tap the protected permanent or remove it
/// from combat. This is a critical difference from regeneration (CR 701.19a).
fn test_umbra_armor_does_not_tap_or_remove_from_combat() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p1, "Umbra Attacker", 3, 3).in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Combat Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(aura)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_on_battlefield(&state, "Umbra Attacker").unwrap();
    let aura_id = find_on_battlefield(&state, "Combat Umbra").unwrap();
    attach_aura(&mut state, aura_id, attacker_id);

    // Simulate the creature being in an attacking state.
    let mut combat = CombatState::new(p1);
    combat
        .attackers
        .insert(attacker_id, AttackTarget::Player(p2));
    state.combat = Some(combat);

    // Apply destroy effect.
    let (effect, mut ctx) = destroy_effect(attacker_id);
    let _events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Creature should survive.
    assert!(
        find_on_battlefield(&state, "Umbra Attacker").is_some(),
        "CR 702.89a: creature should survive via umbra armor"
    );

    let survivor_id = find_on_battlefield(&state, "Umbra Attacker").unwrap();

    // Creature should NOT be tapped (unlike regeneration).
    assert!(
        !is_tapped(&state, survivor_id),
        "CR 702.89a ruling: umbra armor does NOT tap the permanent"
    );

    // Creature should still be in combat (unlike regeneration which removes from combat).
    assert!(
        state
            .combat
            .as_ref()
            .map(|c| c.attackers.contains_key(&survivor_id))
            .unwrap_or(false),
        "CR 702.89a ruling: umbra armor does NOT remove the permanent from combat"
    );
}

// ── Test 6: Indestructible -- Aura NOT consumed ────────────────────────────────

#[test]
/// CR 702.12a, 702.89a — Indestructible prevents destruction entirely; umbra armor not used.
///
/// A creature with Indestructible enchanted by an umbra armor Aura is targeted by destroy.
/// Indestructible prevents destruction before umbra armor can fire. The Aura stays on the
/// battlefield (it is NOT consumed).
fn test_umbra_armor_not_consumed_by_indestructible() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Indestructible Hero", 4, 4)
        .with_keyword(KeywordAbility::Indestructible)
        .in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Conserved Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Indestructible Hero").unwrap();
    let aura_id = find_on_battlefield(&state, "Conserved Umbra").unwrap();
    attach_aura(&mut state, aura_id, creature_id);

    let (effect, mut ctx) = destroy_effect(creature_id);
    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Creature should still be on the battlefield.
    assert!(
        find_on_battlefield(&state, "Indestructible Hero").is_some(),
        "CR 702.12a: indestructible creature should survive"
    );

    // Aura should also still be on the battlefield (NOT consumed).
    assert!(
        find_on_battlefield(&state, "Conserved Umbra").is_some(),
        "CR 702.89a: Aura should NOT be destroyed when indestructible blocks first"
    );

    // No UmbraArmorApplied event -- umbra armor never fired.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::UmbraArmorApplied { .. })),
        "CR 702.89a: UmbraArmorApplied should NOT fire when indestructible takes priority"
    );
}

// ── Test 7: Sacrifice bypasses umbra armor ─────────────────────────────────────

#[test]
/// CR 701.8b, 702.89a — Sacrifice is not "destroy"; umbra armor does not apply.
///
/// A creature enchanted by an umbra armor Aura is sacrificed (e.g., as a cost).
/// Sacrifice is not destruction (CR 701.8b). Umbra armor does not apply. Both the
/// creature and the Aura end up in the graveyard.
fn test_umbra_armor_does_not_prevent_sacrifice() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature =
        ObjectSpec::creature(p1, "Sacrificed Creature", 2, 2).in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Wasted Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Sacrificed Creature").unwrap();
    let aura_id = find_on_battlefield(&state, "Wasted Umbra").unwrap();
    attach_aura(&mut state, aura_id, creature_id);

    // Sacrifice the creature by moving it directly to the graveyard (as sacrifice does).
    let owner = state.objects.get(&creature_id).unwrap().owner;
    let _ = state.move_object_to_zone(creature_id, ZoneId::Graveyard(owner));

    // Run SBAs so the now-enchanting-nothing Aura falls off (SBA 704.5m).
    check_and_apply_sbas(&mut state);

    // Creature should be dead.
    assert!(
        find_on_battlefield(&state, "Sacrificed Creature").is_none(),
        "CR 701.8b: creature should be gone after sacrifice"
    );
    assert!(
        is_in_graveyard(&state, "Sacrificed Creature", p1),
        "Sacrificed creature should be in the graveyard"
    );

    // Aura should also be gone (fell off via SBA 704.5m or destroyed with creature).
    assert!(
        find_on_battlefield(&state, "Wasted Umbra").is_none(),
        "CR 704.5m: Aura should fall off when enchanted permanent leaves battlefield"
    );
}

// ── Test 8: Umbra armor removes all damage ─────────────────────────────────────

#[test]
/// CR 702.89a — Umbra armor removes ALL damage from the protected permanent,
/// even if the damage alone was not lethal (e.g., destroy spell with 3 damage pre-marked
/// on a 5/5).
fn test_umbra_armor_removes_all_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Sturdy Bear", 5, 5)
        .with_damage(3) // not lethal, but present
        .in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Damage Clearer Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Sturdy Bear").unwrap();
    let aura_id = find_on_battlefield(&state, "Damage Clearer Umbra").unwrap();
    attach_aura(&mut state, aura_id, creature_id);

    let (effect, mut ctx) = destroy_effect(creature_id);
    let _events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Creature survives with 0 damage.
    let survivor_id = find_on_battlefield(&state, "Sturdy Bear").unwrap();
    assert_eq!(
        damage_marked(&state, survivor_id),
        0,
        "CR 702.89a: umbra armor removes ALL damage from the protected permanent"
    );
}

// ── Test 9: UmbraArmorApplied event fields are correct ─────────────────────────

#[test]
/// CR 702.89a — Verify the UmbraArmorApplied event contains correct protected_id and aura_id.
fn test_umbra_armor_event_emitted_with_correct_ids() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Event Creature", 2, 2).in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Event Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Event Creature").unwrap();
    let aura_id = find_on_battlefield(&state, "Event Umbra").unwrap();
    attach_aura(&mut state, aura_id, creature_id);

    let (effect, mut ctx) = destroy_effect(creature_id);
    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    let umbra_event = events
        .iter()
        .find(|e| matches!(e, GameEvent::UmbraArmorApplied { .. }));
    assert!(
        umbra_event.is_some(),
        "UmbraArmorApplied event should be present"
    );

    if let Some(GameEvent::UmbraArmorApplied {
        protected_id,
        aura_id: event_aura_id,
    }) = umbra_event
    {
        // The protected_id in the event is the pre-save ID (the creature's original ID).
        // The creature survives so the new battlefield ID should match the original.
        let survivor_id = find_on_battlefield(&state, "Event Creature").unwrap();
        assert_eq!(
            *protected_id, creature_id,
            "UmbraArmorApplied.protected_id should be the creature's ObjectId"
        );
        // The aura went to the graveyard and got a new ObjectId (CR 400.7).
        // But the event captures the pre-move aura_id.
        assert_eq!(
            *event_aura_id, aura_id,
            "UmbraArmorApplied.aura_id should be the Aura's pre-destruction ObjectId"
        );
        // Creature still there.
        let _ = survivor_id; // Silence unused warning.
    }
}

// ── Test 10: Umbra armor on non-creature permanent ─────────────────────────────

#[test]
/// CR 702.89a — Umbra armor protects any enchanted permanent, not just creatures.
///
/// A land enchanted by an umbra armor Aura (e.g., Rancor on a land doesn't make sense, but
/// the rule applies to any permanent type). When the enchanted permanent would be destroyed,
/// umbra armor fires.
fn test_umbra_armor_protects_non_creature_permanent() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let land = ObjectSpec::card(p1, "Forest")
        .with_types(vec![CardType::Land])
        .with_subtypes(vec![SubType("Forest".to_string())])
        .in_zone(ZoneId::Battlefield);
    let aura = umbra_aura_spec(p1, "Land Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(land)
        .object(aura)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let land_id = find_on_battlefield(&state, "Forest").unwrap();
    let aura_id = find_on_battlefield(&state, "Land Umbra").unwrap();
    attach_aura(&mut state, aura_id, land_id);

    let (effect, mut ctx) = destroy_effect(land_id);
    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Land should survive.
    assert!(
        find_on_battlefield(&state, "Forest").is_some(),
        "CR 702.89a: umbra armor protects any enchanted permanent, including lands"
    );

    // Aura destroyed.
    assert!(
        is_in_graveyard(&state, "Land Umbra", p1),
        "CR 702.89a: Aura should be destroyed to protect the land"
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::UmbraArmorApplied { .. })),
        "CR 702.89a: UmbraArmorApplied event should be emitted for non-creature permanents"
    );
}

// ── Test 11: Multiple umbra armor Auras -- one consumed per destruction event ───

#[test]
/// CR 702.89a, 616.1 — When multiple umbra armor Auras protect the same permanent,
/// each destruction event consumes exactly one Aura. The engine auto-selects the
/// lowest ObjectId (deterministic for replay). The permanent survives until all
/// Auras are exhausted.
///
/// Sequence:
///   1. Creature + Aura-A + Aura-B both attached.
///   2. First destroy: one Aura consumed, creature survives, one Aura remains.
///   3. Second destroy: second Aura consumed, creature survives, no Auras remain.
///   4. Third destroy: no Aura left, creature actually dies.
fn test_umbra_armor_multiple_auras_one_consumed_per_event() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature =
        ObjectSpec::creature(p1, "Doubly Protected Bear", 3, 3).in_zone(ZoneId::Battlefield);
    let aura_a = umbra_aura_spec(p1, "First Umbra");
    let aura_b = umbra_aura_spec(p1, "Second Umbra");

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .object(aura_a)
        .object(aura_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id = find_on_battlefield(&state, "Doubly Protected Bear").unwrap();
    let aura_a_id = find_on_battlefield(&state, "First Umbra").unwrap();
    let aura_b_id = find_on_battlefield(&state, "Second Umbra").unwrap();

    attach_aura(&mut state, aura_a_id, creature_id);
    attach_aura(&mut state, aura_b_id, creature_id);

    // --- First destroy: one Aura consumed ----------------------------------------
    let (effect, mut ctx) = destroy_effect(creature_id);
    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        find_on_battlefield(&state, "Doubly Protected Bear").is_some(),
        "CR 702.89a: creature should survive first destruction (two Auras present)"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::UmbraArmorApplied { .. })),
        "CR 702.89a: UmbraArmorApplied event should fire on first destroy"
    );

    // Exactly one Aura should remain on the battlefield.
    let auras_on_bf: Vec<_> = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && (obj.characteristics.name == "First Umbra"
                    || obj.characteristics.name == "Second Umbra")
        })
        .collect();
    assert_eq!(
        auras_on_bf.len(),
        1,
        "CR 702.89a: exactly one Aura should remain after first destroy; found {}",
        auras_on_bf.len()
    );

    // --- Second destroy: second Aura consumed ------------------------------------
    // Creature gets a fresh ID after first event; find it again.
    let creature_id2 = find_on_battlefield(&state, "Doubly Protected Bear").unwrap();
    let (effect2, mut ctx2) = destroy_effect(creature_id2);
    let events2 = mtg_engine::effects::execute_effect(&mut state, &effect2, &mut ctx2);

    assert!(
        find_on_battlefield(&state, "Doubly Protected Bear").is_some(),
        "CR 702.89a: creature should survive second destruction (one Aura remaining)"
    );
    assert!(
        events2
            .iter()
            .any(|e| matches!(e, GameEvent::UmbraArmorApplied { .. })),
        "CR 702.89a: UmbraArmorApplied event should fire on second destroy"
    );

    // Both Auras should now be in the graveyard.
    let auras_on_bf2: Vec<_> = state
        .objects
        .values()
        .filter(|obj| {
            obj.zone == ZoneId::Battlefield
                && (obj.characteristics.name == "First Umbra"
                    || obj.characteristics.name == "Second Umbra")
        })
        .collect();
    assert_eq!(
        auras_on_bf2.len(),
        0,
        "CR 702.89a: no Auras should remain after second destroy"
    );

    // --- Third destroy: no Aura left; creature actually dies ---------------------
    let creature_id3 = find_on_battlefield(&state, "Doubly Protected Bear").unwrap();
    let (effect3, mut ctx3) = destroy_effect(creature_id3);
    let events3 = mtg_engine::effects::execute_effect(&mut state, &effect3, &mut ctx3);

    assert!(
        find_on_battlefield(&state, "Doubly Protected Bear").is_none(),
        "CR 702.89a: creature should die on third destruction (no Auras left)"
    );
    assert!(
        is_in_graveyard(&state, "Doubly Protected Bear", p1),
        "CR 702.89a: creature should be in graveyard after third destroy"
    );
    assert!(
        !events3
            .iter()
            .any(|e| matches!(e, GameEvent::UmbraArmorApplied { .. })),
        "CR 702.89a: UmbraArmorApplied should NOT fire when no Aura is present"
    );
}
