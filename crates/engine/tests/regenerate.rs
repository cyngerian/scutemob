//! Regenerate keyword action tests (CR 701.19, 614.8).
//!
//! Regenerate creates a one-shot replacement effect ("regeneration shield") on a permanent.
//! When that permanent would be destroyed, the shield intercepts and instead:
//! 1. Removes all damage marked on the permanent (CR 701.19a).
//! 2. Taps the permanent (CR 701.19a).
//! 3. Removes it from combat if it was attacking or blocking (CR 701.19a).
//!
//! Key rules verified:
//! - Shield prevents destruction by spell/ability (Effect::DestroyPermanent).
//! - Shield prevents SBA destruction from lethal damage (CR 704.5g).
//! - Shield prevents SBA destruction from deathtouch damage (CR 704.5h).
//! - Shield does NOT prevent zero-toughness death (CR 704.5f — not destruction).
//! - Shield is one-shot (consumed after one use, CR 701.19a).
//! - Multiple shields stack and are consumed independently (CR 701.19a).
//! - Shield removes regenerated creature from combat (CR 701.19a).
//! - Shield expires at end of turn (CR 701.19a, 514.2).
//! - Indestructible is handled before regeneration — shield NOT consumed (CR 702.12a).

use mtg_engine::{
    check_and_apply_sbas, CardEffectTarget, CardRegistry, CombatState, Effect, EffectDuration,
    GameEvent, GameStateBuilder, ObjectFilter, ObjectId, ObjectSpec, PlayerId, ReplacementEffect,
    ReplacementId, ReplacementModification, ReplacementTrigger, SpellTarget, Step, Target, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_on_battlefield(state: &mtg_engine::GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
}

fn is_tapped(state: &mtg_engine::GameState, id: ObjectId) -> bool {
    state
        .objects
        .get(&id)
        .map(|o| o.status.tapped)
        .unwrap_or(false)
}

fn damage_marked(state: &mtg_engine::GameState, id: ObjectId) -> u32 {
    state.objects.get(&id).map(|o| o.damage_marked).unwrap_or(0)
}

fn deathtouch_damage(state: &mtg_engine::GameState, id: ObjectId) -> bool {
    state
        .objects
        .get(&id)
        .map(|o| o.deathtouch_damage)
        .unwrap_or(false)
}

fn regen_shield(controller: PlayerId, target: ObjectId, shield_id: u64) -> ReplacementEffect {
    ReplacementEffect {
        id: ReplacementId(shield_id),
        source: Some(target),
        controller,
        duration: EffectDuration::UntilEndOfTurn,
        is_self_replacement: true,
        trigger: ReplacementTrigger::WouldBeDestroyed {
            filter: ObjectFilter::SpecificObject(target),
        },
        modification: ReplacementModification::Regenerate,
    }
}

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

// ── Test 1: Shield prevents destruction by spell ──────────────────────────────

#[test]
/// CR 701.19a, 614.8 — Regeneration shield intercepts DestroyPermanent effect.
///
/// Creature with a regeneration shield is targeted by a destroy spell.
/// The shield intercepts: creature stays on battlefield, damage removed, creature tapped.
/// No CreatureDied event emitted.
fn test_regenerate_shield_prevents_destruction_by_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Regen Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "Regen Creature").expect("creature should be on battlefield");

    // Register a regeneration shield.
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 0));
    state.next_replacement_id = 1;

    let (effect, mut ctx) = destroy_effect(creature_id);
    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Creature should still be on the battlefield.
    assert!(
        find_on_battlefield(&state, "Regen Creature").is_some(),
        "CR 701.19a: creature should survive destruction via regeneration shield"
    );

    // Damage should be removed.
    assert_eq!(
        damage_marked(&state, creature_id),
        0,
        "CR 701.19a: damage should be removed after regeneration"
    );

    // Creature should be tapped.
    assert!(
        is_tapped(&state, creature_id),
        "CR 701.19a: creature should be tapped after regeneration"
    );

    // Regenerated event emitted, no CreatureDied.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "CR 701.19a: Regenerated event should be emitted"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 701.19a: CreatureDied should NOT be emitted when creature regenerates"
    );

    // Shield should be consumed (removed).
    assert!(
        state.replacement_effects.is_empty(),
        "CR 701.19a: regeneration shield should be consumed after use"
    );
}

// ── Test 2: Shield prevents SBA lethal damage death ───────────────────────────

#[test]
/// CR 701.19a, 704.5g — Regeneration shield intercepts SBA lethal-damage destruction.
///
/// 2/2 creature with 3 damage marked has a regeneration shield.
/// SBA checks: creature would die (704.5g). Shield intercepts → creature survives.
fn test_regenerate_shield_prevents_sba_lethal_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Tanky", 2, 2)
        .with_damage(3) // lethal damage for a 2/2
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "Tanky").expect("creature should be on battlefield");

    // Register a regeneration shield.
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 0));
    state.next_replacement_id = 1;

    let events = check_and_apply_sbas(&mut state);

    // Creature should still be on the battlefield.
    assert!(
        find_on_battlefield(&state, "Tanky").is_some(),
        "CR 701.19a/704.5g: creature should survive lethal damage via regeneration shield"
    );

    // Damage should be reset to 0.
    assert_eq!(
        damage_marked(&state, creature_id),
        0,
        "CR 701.19a: damage_marked should be reset to 0 after regeneration"
    );

    // Creature should be tapped.
    assert!(
        is_tapped(&state, creature_id),
        "CR 701.19a: creature should be tapped after regeneration"
    );

    // Regenerated event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "CR 701.19a: Regenerated event should be emitted from SBA path"
    );

    // No CreatureDied.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 701.19a: CreatureDied should NOT be emitted when creature regenerates"
    );
}

// ── Test 3: Shield prevents SBA deathtouch damage death ──────────────────────

#[test]
/// CR 701.19a, 704.5h — Regeneration shield intercepts SBA deathtouch destruction.
///
/// 4/4 creature dealt 1 deathtouch damage has a regeneration shield.
/// SBA checks: creature would die (704.5h). Shield intercepts → creature survives.
fn test_regenerate_shield_prevents_sba_deathtouch_damage() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // 4/4 creature with deathtouch damage marked.
    let creature = ObjectSpec::creature(p1, "Toughie", 4, 4)
        .with_damage(1)
        .with_deathtouch_damage()
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "Toughie").expect("creature should be on battlefield");

    // Register a regeneration shield.
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 0));
    state.next_replacement_id = 1;

    let events = check_and_apply_sbas(&mut state);

    // Creature should still be on the battlefield.
    assert!(
        find_on_battlefield(&state, "Toughie").is_some(),
        "CR 701.19a/704.5h: creature should survive deathtouch damage via regeneration shield"
    );

    // Damage should be reset to 0 and deathtouch_damage cleared.
    assert_eq!(
        damage_marked(&state, creature_id),
        0,
        "CR 701.19a: damage_marked should be reset to 0 after regeneration"
    );
    assert!(
        !deathtouch_damage(&state, creature_id),
        "CR 701.19a: deathtouch_damage flag should be cleared after regeneration"
    );

    // Regenerated event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "CR 701.19a: Regenerated event should be emitted from SBA deathtouch path"
    );
}

// ── Test 4: Shield is one-shot (consumed after first use) ─────────────────────

#[test]
/// CR 701.19a — Regeneration shield is one-shot: consumed after intercepting one destruction.
///
/// Creature with one shield. First destruction intercepted. Second destruction kills it.
fn test_regenerate_shield_is_one_shot() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "OnceAlive", 2, 2)
        .with_damage(3) // lethal
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "OnceAlive").expect("creature should be on battlefield");

    // Register one regeneration shield.
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 0));
    state.next_replacement_id = 1;

    // First SBA pass: shield intercepts, creature survives.
    let events1 = check_and_apply_sbas(&mut state);
    assert!(
        find_on_battlefield(&state, "OnceAlive").is_some(),
        "CR 701.19a: creature should survive first destruction"
    );
    assert!(
        events1
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "CR 701.19a: Regenerated event should be emitted on first use"
    );

    // Shield is now consumed.
    assert!(
        state.replacement_effects.is_empty(),
        "CR 701.19a: shield should be consumed after first use"
    );

    // Apply lethal damage again.
    if let Some(obj) = state.objects.get_mut(&creature_id) {
        obj.damage_marked = 3;
    }

    // Second SBA pass: no shield → creature dies.
    let events2 = check_and_apply_sbas(&mut state);
    assert!(
        find_on_battlefield(&state, "OnceAlive").is_none(),
        "CR 701.19a: creature should die on second destruction (no shield remaining)"
    );
    assert!(
        events2
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CR 701.19a: CreatureDied should be emitted when no shield remains"
    );
}

// ── Test 5: Multiple shields — consumed independently ─────────────────────────

#[test]
/// CR 701.19a — Multiple regeneration shields stack and are consumed one at a time.
///
/// Creature with two shields. First destruction uses first shield. Second destruction
/// uses second shield. Third destruction kills it.
fn test_regenerate_multiple_shields() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "DoubleShielded", 2, 2)
        .with_damage(3) // lethal
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "DoubleShielded").expect("creature should be on battlefield");

    // Register two regeneration shields.
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 0));
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 1));
    state.next_replacement_id = 2;

    // First SBA pass: first shield consumed, creature survives.
    let events1 = check_and_apply_sbas(&mut state);
    assert!(
        find_on_battlefield(&state, "DoubleShielded").is_some(),
        "CR 701.19a: creature should survive first destruction (2 shields)"
    );
    assert!(
        events1
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "Regenerated event expected on first interception"
    );
    // One shield remains.
    assert_eq!(
        state.replacement_effects.len(),
        1,
        "CR 701.19a: one shield should remain after first interception"
    );

    // Apply lethal damage again.
    if let Some(obj) = state.objects.get_mut(&creature_id) {
        obj.damage_marked = 3;
    }

    // Second SBA pass: second shield consumed, creature survives again.
    let events2 = check_and_apply_sbas(&mut state);
    assert!(
        find_on_battlefield(&state, "DoubleShielded").is_some(),
        "CR 701.19a: creature should survive second destruction (1 shield remains)"
    );
    assert!(
        events2
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "Regenerated event expected on second interception"
    );
    // No shields remain.
    assert!(
        state.replacement_effects.is_empty(),
        "CR 701.19a: both shields should be consumed after two interceptions"
    );

    // Apply lethal damage a third time.
    if let Some(obj) = state.objects.get_mut(&creature_id) {
        obj.damage_marked = 3;
    }

    // Third SBA pass: no shield → creature dies.
    let events3 = check_and_apply_sbas(&mut state);
    assert!(
        find_on_battlefield(&state, "DoubleShielded").is_none(),
        "CR 701.19a: creature should die on third destruction (no shields remaining)"
    );
    assert!(
        events3
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "CreatureDied expected on third destruction"
    );
}

// ── Test 6: Shield removes creature from combat (attacker) ────────────────────

#[test]
/// CR 701.19a — Regeneration removes attacking creature from combat.
///
/// Creature declared as attacker is destroyed during combat. Shield intercepts.
/// Creature is removed from combat.attackers.
fn test_regenerate_removes_from_combat_attacker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "Attacker", 2, 2)
        .with_damage(3) // lethal
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "Attacker").expect("creature should be on battlefield");

    // Set up combat state with creature as attacker.
    let mut combat = CombatState::new(p1);
    combat
        .attackers
        .insert(creature_id, mtg_engine::AttackTarget::Player(p2));
    state.combat = Some(combat);

    // Register a regeneration shield.
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 0));
    state.next_replacement_id = 1;

    let events = check_and_apply_sbas(&mut state);

    // Creature should still be on the battlefield.
    assert!(
        find_on_battlefield(&state, "Attacker").is_some(),
        "CR 701.19a: creature should survive destruction via regeneration shield"
    );

    // Creature should be tapped.
    assert!(
        is_tapped(&state, creature_id),
        "CR 701.19a: creature should be tapped after regeneration"
    );

    // Creature should be removed from combat.
    let in_combat = state
        .combat
        .as_ref()
        .map(|c| c.attackers.contains_key(&creature_id))
        .unwrap_or(false);
    assert!(
        !in_combat,
        "CR 701.19a: creature should be removed from combat.attackers after regeneration"
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "Regenerated event should be emitted"
    );
}

// ── Test 7: Shield removes creature from combat (blocker) ─────────────────────

#[test]
/// CR 701.19a — Regeneration removes blocking creature from combat.
///
/// Creature declared as blocker is destroyed during combat. Shield intercepts.
/// Creature is removed from combat.blockers.
fn test_regenerate_removes_from_combat_blocker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let attacker = ObjectSpec::creature(p2, "Attacker", 5, 5).in_zone(ZoneId::Battlefield);
    let blocker = ObjectSpec::creature(p1, "Blocker", 2, 2)
        .with_damage(5) // lethal
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .object(blocker)
        .active_player(p2)
        .at_step(Step::DeclareBlockers)
        .build()
        .unwrap();

    let attacker_id =
        find_on_battlefield(&state, "Attacker").expect("attacker should be on battlefield");
    let blocker_id =
        find_on_battlefield(&state, "Blocker").expect("blocker should be on battlefield");

    // Set up combat state with blocker.
    let mut combat = CombatState::new(p2);
    combat
        .attackers
        .insert(attacker_id, mtg_engine::AttackTarget::Player(p1));
    combat.blockers.insert(blocker_id, attacker_id);
    state.combat = Some(combat);

    // Register a regeneration shield on the blocker.
    state
        .replacement_effects
        .push_back(regen_shield(p1, blocker_id, 0));
    state.next_replacement_id = 1;

    let events = check_and_apply_sbas(&mut state);

    // Blocker should still be on the battlefield.
    assert!(
        find_on_battlefield(&state, "Blocker").is_some(),
        "CR 701.19a: blocker should survive destruction via regeneration shield"
    );

    // Blocker should be tapped.
    assert!(
        is_tapped(&state, blocker_id),
        "CR 701.19a: blocker should be tapped after regeneration"
    );

    // Blocker should be removed from combat.blockers.
    let in_combat = state
        .combat
        .as_ref()
        .map(|c| c.blockers.contains_key(&blocker_id))
        .unwrap_or(false);
    assert!(
        !in_combat,
        "CR 701.19a: creature should be removed from combat.blockers after regeneration"
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "Regenerated event should be emitted"
    );
}

// ── Test 8: Shield does NOT prevent zero-toughness death ─────────────────────

#[test]
/// CR 704.5f, 701.19a — Regeneration does NOT prevent death from zero/negative toughness.
///
/// A creature with toughness 0 has a regeneration shield. SBA 704.5f fires (not destruction).
/// The creature dies and the shield is NOT consumed.
fn test_regenerate_does_not_prevent_zero_toughness() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // 1/0 creature (zero toughness — dies to 704.5f, not destruction).
    let creature = ObjectSpec::creature(p1, "ZeroTough", 1, 0).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "ZeroTough").expect("creature should be on battlefield");

    // Register a regeneration shield.
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 0));
    state.next_replacement_id = 1;

    let events = check_and_apply_sbas(&mut state);

    // Creature should be dead (704.5f bypasses regeneration).
    assert!(
        find_on_battlefield(&state, "ZeroTough").is_none(),
        "CR 704.5f: zero-toughness creature should die even with regeneration shield"
    );

    // No Regenerated event (shield was not consumed).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "CR 704.5f: Regenerated event should NOT be emitted for zero-toughness death"
    );

    // Shield should still be present (not consumed).
    assert_eq!(
        state.replacement_effects.len(),
        1,
        "CR 704.5f: regeneration shield should NOT be consumed for zero-toughness death"
    );
}

// ── Test 9: RegenerationShieldCreated event emitted ──────────────────────────

#[test]
/// CR 701.19a — Effect::Regenerate emits RegenerationShieldCreated event.
///
/// Executing Effect::Regenerate on a valid target emits RegenerationShieldCreated
/// and registers a replacement effect.
fn test_regenerate_effect_emits_shield_created_event() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let creature = ObjectSpec::creature(p1, "ShieldTarget", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "ShieldTarget").expect("creature should be on battlefield");

    use mtg_engine::effects::EffectContext;

    let effect = Effect::Regenerate {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
    };
    let mut ctx = EffectContext::new(
        p1,
        ObjectId(999),
        vec![SpellTarget {
            target: Target::Object(creature_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );

    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // RegenerationShieldCreated event emitted.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::RegenerationShieldCreated { object_id, .. } if *object_id == creature_id)),
        "CR 701.19a: RegenerationShieldCreated event should be emitted when Effect::Regenerate executes"
    );

    // Shield registered in state.replacement_effects.
    assert_eq!(
        state.replacement_effects.len(),
        1,
        "CR 701.19a: regeneration shield should be registered in state.replacement_effects"
    );
    let shield = &state.replacement_effects[0];
    assert_eq!(
        shield.modification,
        ReplacementModification::Regenerate,
        "shield modification should be Regenerate"
    );
    assert!(
        shield.is_self_replacement,
        "CR 614.15: regeneration shield should be a self-replacement effect"
    );
    assert_eq!(
        shield.duration,
        EffectDuration::UntilEndOfTurn,
        "CR 701.19a: regeneration shield lasts until end of turn"
    );
}

// ── Test 10: Indestructible — shield NOT consumed ────────────────────────────

#[test]
/// CR 702.12a, 701.19a — Indestructible bypasses regeneration check entirely.
///
/// Indestructible creature with regeneration shield. Destroy spell cast.
/// Creature survives (indestructible). Shield is NOT consumed.
fn test_regenerate_not_applied_when_indestructible() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    use mtg_engine::KeywordAbility;

    let creature = ObjectSpec::creature(p1, "Indestrucible", 2, 2)
        .with_keyword(KeywordAbility::Indestructible)
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let creature_id =
        find_on_battlefield(&state, "Indestrucible").expect("creature should be on battlefield");

    // Register a regeneration shield.
    state
        .replacement_effects
        .push_back(regen_shield(p1, creature_id, 0));
    state.next_replacement_id = 1;

    let (effect, mut ctx) = destroy_effect(creature_id);
    let events = mtg_engine::effects::execute_effect(&mut state, &effect, &mut ctx);

    // Creature should still be on the battlefield (indestructible).
    assert!(
        find_on_battlefield(&state, "Indestrucible").is_some(),
        "CR 702.12a: indestructible creature should survive destroy spell"
    );

    // Shield should NOT be consumed (indestructible handled first).
    assert_eq!(
        state.replacement_effects.len(),
        1,
        "CR 702.12a/701.19a: regeneration shield should NOT be consumed when indestructible prevents destruction"
    );

    // No Regenerated event (indestructible prevented destruction before shield check).
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "CR 702.12a: Regenerated event should NOT be emitted when indestructible applies"
    );
}
