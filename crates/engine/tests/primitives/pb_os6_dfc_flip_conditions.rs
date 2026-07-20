//! PB-OS6 (OOS-EF5-4 a/b/g): DFC flip-condition sub-batch.
//!
//! Three new primitives, each closing a "surviving clause" gap on an already-shipped
//! `Effect::TransformSelf` (PB-EF5) DFC:
//!
//! - (a) `Condition::TopCardIsInstantOrSorcery` (CR 400.2/614.1c) -- delver_of_secrets'
//!   upkeep flip. Card flips `partial` to `Complete`.
//! - (b) `Condition::YouAttackedWithNOrMore(u32)` (CR 508.1/508.4) backed by
//!   `PlayerState.attackers_declared_this_turn`, captured in `handle_declare_attackers`
//!   and reset in `reset_turn_state` -- legions_landing (NEW card, Complete).
//! - (g) `Effect::RemoveFromCombat { target }` (CR 506.4) plus `GameEvent::RemovedFromCombat`
//!   and a shared `remove_from_combat` helper factored out of `apply_regeneration` --
//!   thaumatic_compass' Spires of Orazca back face. Card flips `partial` to `Complete`.
//!
//! All card-facing probes drive real `Command`s (SR-34/36 execution-probing, not
//! source-tracing). PROTOCOL_VERSION moves 20 to 21 and HASH_SCHEMA_VERSION moves 57
//! to 58: four wire-closure shape moves (two new `Condition` variants, one new `Effect`
//! variant, one new `GameEvent` variant); `PlayerState.attackers_declared_this_turn` is
//! GameState-only, so that field is HASH-only.
//!
//! Patterns mirrored: `tests/primitives/pb_os5_relative_attacker_count.rs` (attack-count
//! and sentinel layout), `tests/mechanics_a_d/chosen_creature_type.rs` (top-of-library
//! condition), `tests/mechanics_m_z/regenerate.rs` (combat-removal assertions),
//! `tests/mechanics_m_z/pb_ef5_transform_self.rs` (DFC end-step transform harness),
//! `tests/primitives/pb_ac6_card_integration.rs` (upkeep-trigger step-crossing harness).

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    all_cards, calculate_characteristics, check_and_apply_sbas, enrich_spec_from_def,
    process_command, AttackTarget, CardDefinition, CardEffectTarget, CardRegistry, CardType,
    CombatState, Command, Effect, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec,
    PlayerId, Step, Target, ZoneId, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    enrich_spec_from_def(base, defs)
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

/// Pass priority repeatedly until `target` step is reached.
fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (infinite loop?)"
        );
        let holder = state.turn().priority_holder.expect("no priority holder");
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
    }
}

/// Resolve everything currently on the stack by passing priority in turn order.
fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

fn declare_attackers(
    state: GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
) -> GameState {
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player,
            attackers,
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");
    state
}

fn is_transformed(state: &GameState, id: ObjectId) -> bool {
    state
        .objects()
        .get(&id)
        .map(|o| o.is_transformed)
        .unwrap_or(false)
}

// ── (a) delver_of_secrets: Condition::TopCardIsInstantOrSorcery ───────────────

/// CR 400.2/614.1c/701.28 -- Delver of Secrets flips to Insectile Aberration when the
/// top card of its controller's library is an instant/sorcery, revealed at the
/// beginning of its controller's upkeep. Drives the real upkeep step-entry trigger
/// (`upkeep_actions`) across a step crossing, then resolves the stack.
#[test]
fn test_delver_flips_when_top_is_instant() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let delver = real_card_spec(p1, "Delver of Secrets", ZoneId::Battlefield, &defs);
    let top_instant = ObjectSpec::card(p1, "Library Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(delver)
        .object(top_instant)
        .active_player(p1)
        .at_step(Step::Untap)
        .build()
        .unwrap();
    // CR 502: the untap step grants no priority; seed it to walk out of Untap.
    state.turn_mut().priority_holder = Some(p1);

    let delver_id = find_object(&state, "Delver of Secrets");
    assert!(
        !is_transformed(&state, delver_id),
        "precondition: Delver starts untransformed"
    );

    let state = advance_to_step(state, Step::Upkeep);
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        is_transformed(&state, delver_id),
        "CR 400.2/614.1c: top card is an instant -- Delver should transform at upkeep"
    );
    let chars = calculate_characteristics(&state, delver_id).expect("should have chars");
    assert_eq!(chars.name, "Insectile Aberration");
}

/// Decoy: top card is a creature -- Condition::TopCardIsInstantOrSorcery is false,
/// so Delver's upkeep trigger fires (goes on the stack, CR 603.3) but resolves to
/// Effect::Nothing. Delver must NOT transform.
#[test]
fn test_delver_no_flip_when_top_is_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let delver = real_card_spec(p1, "Delver of Secrets", ZoneId::Battlefield, &defs);
    let top_creature = ObjectSpec::creature(p1, "Library Bear", 2, 2).in_zone(ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(delver)
        .object(top_creature)
        .active_player(p1)
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let delver_id = find_object(&state, "Delver of Secrets");

    let state = advance_to_step(state, Step::Upkeep);
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        !is_transformed(&state, delver_id),
        "top card is a creature, not instant/sorcery -- Delver must NOT transform"
    );
}

// ── (b) legions_landing: Condition::YouAttackedWithNOrMore(u32) ───────────────

/// CR 508.1/508.4 -- Legion's Landing transforms into Adanto, the First Fort when its
/// controller attacks with three or more creatures. Drives the real
/// `Command::DeclareAttackers` -> `WheneverYouAttack` trigger -> `Effect::Conditional`
/// path.
#[test]
fn test_legions_landing_transforms_on_three_attackers() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let landing = real_card_spec(p1, "Legion's Landing", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(landing)
        .object(ObjectSpec::creature(p1, "Attacker A", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker B", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker C", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let landing_id = find_object(&state, "Legion's Landing");
    let a = find_object(&state, "Attacker A");
    let b = find_object(&state, "Attacker B");
    let c = find_object(&state, "Attacker C");

    let state = declare_attackers(
        state,
        p1,
        vec![
            (a, AttackTarget::Player(p2)),
            (b, AttackTarget::Player(p2)),
            (c, AttackTarget::Player(p2)),
        ],
    );
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        is_transformed(&state, landing_id),
        "CR 508.1: attacking with 3+ creatures should transform Legion's Landing"
    );
    let chars = calculate_characteristics(&state, landing_id).expect("should have chars");
    assert_eq!(chars.name, "Adanto, the First Fort");
}

/// Decoy: attacking with only two creatures -- Condition::YouAttackedWithNOrMore(3) is
/// false (count 2 < 3), so the WheneverYouAttack trigger fires (any attack) but its
/// Effect::Conditional resolves to Nothing. Legion's Landing must NOT transform. This
/// pins the COUNT gate, not a bare "you attacked" check.
#[test]
fn test_legions_landing_no_transform_on_two_attackers() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let landing = real_card_spec(p1, "Legion's Landing", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(landing)
        .object(ObjectSpec::creature(p1, "Attacker A", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker B", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let landing_id = find_object(&state, "Legion's Landing");
    let a = find_object(&state, "Attacker A");
    let b = find_object(&state, "Attacker B");

    let state = declare_attackers(
        state,
        p1,
        vec![(a, AttackTarget::Player(p2)), (b, AttackTarget::Player(p2))],
    );
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        !is_transformed(&state, landing_id),
        "CR 508.1: attacking with only 2 creatures must NOT transform Legion's Landing \
         (YouAttackedWithNOrMore(3) is false)"
    );
}

/// CR 508.1: WheneverYouAttack fires once per player declaration, not once per
/// attacking creature -- exactly one `PermanentTransformed` event, not three.
#[test]
fn test_legions_landing_fires_once_not_per_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let landing = real_card_spec(p1, "Legion's Landing", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(landing)
        .object(ObjectSpec::creature(p1, "Attacker A", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker B", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker C", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let landing_id = find_object(&state, "Legion's Landing");
    let a = find_object(&state, "Attacker A");
    let b = find_object(&state, "Attacker B");
    let c = find_object(&state, "Attacker C");

    let state = declare_attackers(
        state,
        p1,
        vec![
            (a, AttackTarget::Player(p2)),
            (b, AttackTarget::Player(p2)),
            (c, AttackTarget::Player(p2)),
        ],
    );

    let mut transform_events = 0usize;
    let mut state = state;
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        let (new_state, events) = pass_all(state, &[p1, p2]);
        state = new_state;
        transform_events += events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    GameEvent::PermanentTransformed { object_id, to_back_face: true }
                        if *object_id == landing_id
                )
            })
            .count();
    }

    assert_eq!(
        transform_events, 1,
        "the attack trigger must fire exactly once (per player declaration), producing \
         exactly one PermanentTransformed event, not one per attacking creature"
    );
    assert!(is_transformed(&state, landing_id));
}

/// CR 508.1 ruling (2017-09-29): once you've attacked with three or more creatures,
/// Legion's Landing transforms even if some of those creatures leave combat before the
/// trigger resolves -- the count is CAPTURED at declare-attackers time, not re-read at
/// resolution.
#[test]
fn test_legions_landing_transforms_even_if_attacker_leaves() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let landing = real_card_spec(p1, "Legion's Landing", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(landing)
        .object(ObjectSpec::creature(p1, "Attacker A", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker B", 2, 2))
        .object(ObjectSpec::creature(p1, "Attacker C", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let landing_id = find_object(&state, "Legion's Landing");
    let a = find_object(&state, "Attacker A");
    let b = find_object(&state, "Attacker B");
    let c = find_object(&state, "Attacker C");

    let mut state = declare_attackers(
        state,
        p1,
        vec![
            (a, AttackTarget::Player(p2)),
            (b, AttackTarget::Player(p2)),
            (c, AttackTarget::Player(p2)),
        ],
    );

    // The WheneverYouAttack trigger is already on the stack at this point (the
    // DeclareAttackers command handler flushes pending triggers). Simulate one
    // attacker leaving combat BEFORE the trigger resolves -- equivalent to any real
    // removal path (Path to Exile, Effect::RemoveFromCombat, etc.).
    assert!(
        !state.stack_objects().is_empty(),
        "precondition: the attack trigger should already be on the stack"
    );
    if let Some(combat) = state.combat_mut() {
        combat.attackers.remove(&c);
    }

    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        is_transformed(&state, landing_id),
        "CR 508.1 ruling: the captured attacker count does not decrease when an \
         attacker later leaves combat -- Legion's Landing should still transform"
    );
}

// ── (g) thaumatic_compass: Effect::RemoveFromCombat ────────────────────────────

/// Build a Thaumatic Compass already transformed into Spires of Orazca (via the real
/// end-step TransformSelf path, 7+ lands controlled), plus an opponent's tapped
/// attacking creature. Returns (state, spires_id, attacker_id).
fn spires_with_attacking_opponent_creature() -> (GameState, ObjectId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let registry = CardRegistry::new(all_cards());

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Thaumatic Compass",
            ZoneId::Battlefield,
            &defs,
        ));
    for i in 0..7 {
        builder = builder.object(
            ObjectSpec::land(p1, &format!("Compass Land {}", i)).in_zone(ZoneId::Battlefield),
        );
    }
    builder = builder.object(
        ObjectSpec::creature(p2, "Opponent Attacker", 3, 3)
            .in_zone(ZoneId::Battlefield)
            .tapped(),
    );
    let state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let compass_id = find_object(&state, "Thaumatic Compass");
    let attacker_id = find_object(&state, "Opponent Attacker");

    let state = advance_to_step(state, Step::End);
    let mut state = resolve_stack(state, &[p1, p2]);

    assert!(
        is_transformed(&state, compass_id),
        "precondition: 7+ lands should have transformed Thaumatic Compass into Spires"
    );
    let chars = calculate_characteristics(&state, compass_id).expect("should have chars");
    assert_eq!(chars.name, "Spires of Orazca");

    // The opponent's creature is attacking (an opponent of Spires' controller, p1).
    let mut combat = CombatState::new(p2);
    combat
        .attackers
        .insert(attacker_id, AttackTarget::Player(p1));
    *state.combat_mut() = Some(combat);

    (state, compass_id, attacker_id)
}

/// CR 506.4/508 -- Spires of Orazca's `{T}: Untap target attacking creature an opponent
/// controls and remove it from combat` both untaps the target AND removes it from
/// combat via the two-step Sequence (UntapPermanent + RemoveFromCombat). Drives the
/// real `Command::ActivateAbility` path.
#[test]
fn test_thaumatic_spires_untaps_and_removes_from_combat() {
    let (state, compass_id, attacker_id) = spires_with_attacking_opponent_creature();

    assert!(
        state.objects()[&attacker_id].status.tapped,
        "precondition: the opponent's attacker starts tapped"
    );
    assert!(
        state
            .combat()
            .as_ref()
            .map(|c| c.attackers.contains_key(&attacker_id))
            .unwrap_or(false),
        "precondition: the opponent's attacker starts in combat.attackers"
    );

    // ability_index 0: the mana ability ({T}: Add {C}) is filtered out of
    // characteristics.activated_abilities, so the untap-and-remove ability is index 0.
    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: compass_id,
            ability_index: 0,
            targets: vec![Target::Object(attacker_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("Spires' untap-and-remove ability should activate");
    let state = resolve_stack(state, &[p(1), p(2)]);

    assert!(
        !state.objects()[&attacker_id].status.tapped,
        "CR 701.21: the target should be untapped"
    );
    assert!(
        !state
            .combat()
            .as_ref()
            .map(|c| c.attackers.contains_key(&attacker_id))
            .unwrap_or(false),
        "CR 506.4: the target should no longer be in combat.attackers"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { .. })),
        "sanity: the activation itself should be observable"
    );
}

/// Decoy: pins that the combat-removal comes from `Effect::RemoveFromCombat`, not from
/// `Effect::UntapPermanent` alone (CR 506.4b -- tapping/untapping does not by itself
/// remove a permanent from combat). Executes ONLY an UntapPermanent effect against the
/// same attacking-creature setup and asserts the target is STILL in combat.attackers
/// afterward.
#[test]
fn test_thaumatic_spires_decoy_untap_only_leaves_in_combat() {
    let (mut state, _compass_id, attacker_id) = spires_with_attacking_opponent_creature();

    assert!(
        state.objects()[&attacker_id].status.tapped,
        "precondition: the opponent's attacker starts tapped"
    );
    assert!(
        state
            .combat()
            .as_ref()
            .map(|c| c.attackers.contains_key(&attacker_id))
            .unwrap_or(false),
        "precondition: the opponent's attacker starts in combat.attackers"
    );

    let mut ctx = EffectContext::new(p(1), attacker_id, vec![]);
    let _events = execute_effect(
        &mut state,
        &Effect::UntapPermanent {
            target: CardEffectTarget::Source,
        },
        &mut ctx,
    );

    assert!(
        !state.objects()[&attacker_id].status.tapped,
        "the lone UntapPermanent effect should still untap the target"
    );
    assert!(
        state
            .combat()
            .as_ref()
            .map(|c| c.attackers.contains_key(&attacker_id))
            .unwrap_or(false),
        "CR 506.4b: untapping alone must NOT remove the target from combat.attackers -- \
         only the paired RemoveFromCombat effect does that"
    );
}

// ── Engine-level: remove_from_combat helper (via public Effect::RemoveFromCombat) ──

/// CR 506.4 -- `Effect::RemoveFromCombat` (and the `remove_from_combat` helper it
/// dispatches to) clears an attacker from `combat.attackers`, `combat.blocked_attackers`,
/// and its `combat.damage_assignment_order` key; and clears a blocker from
/// `combat.blockers` AND strips it out of every attacker's blocker-order list.
#[test]
fn test_remove_from_combat_helper_clears_attacker_and_damage_order() {
    let p1 = p(1);
    let p2 = p(2);

    // ── Attacker-side clearing ──
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(ObjectSpec::creature(p1, "Removed Attacker", 3, 3))
            .object(ObjectSpec::creature(p2, "Some Blocker", 1, 1))
            .at_step(Step::DeclareBlockers)
            .active_player(p1)
            .build()
            .unwrap();
        let attacker_id = find_object(&state, "Removed Attacker");
        let blocker_id = find_object(&state, "Some Blocker");

        let mut combat = CombatState::new(p1);
        combat
            .attackers
            .insert(attacker_id, AttackTarget::Player(p2));
        combat.blockers.insert(blocker_id, attacker_id);
        combat
            .damage_assignment_order
            .insert(attacker_id, vec![blocker_id]);
        combat.blocked_attackers.insert(attacker_id);
        *state.combat_mut() = Some(combat);

        let mut ctx = EffectContext::new(p1, attacker_id, vec![]);
        let events = execute_effect(
            &mut state,
            &Effect::RemoveFromCombat {
                target: CardEffectTarget::Source,
            },
            &mut ctx,
        );

        let combat = state.combat().as_ref().expect("combat still active");
        assert!(
            !combat.attackers.contains_key(&attacker_id),
            "attacker should be removed from combat.attackers"
        );
        assert!(
            !combat.blocked_attackers.contains(&attacker_id),
            "attacker should be removed from combat.blocked_attackers"
        );
        assert!(
            !combat.damage_assignment_order.contains_key(&attacker_id),
            "attacker's damage_assignment_order entry should be removed"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, GameEvent::RemovedFromCombat { object_id } if *object_id == attacker_id)),
            "a RemovedFromCombat event should be emitted"
        );
    }

    // ── Blocker-side clearing (strip from OTHER attackers' order lists) ──
    {
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(ObjectSpec::creature(p1, "Attacker One", 2, 2))
            .object(ObjectSpec::creature(p1, "Attacker Two", 2, 2))
            .object(ObjectSpec::creature(p2, "Removed Blocker", 1, 1))
            .at_step(Step::DeclareBlockers)
            .active_player(p1)
            .build()
            .unwrap();
        let attacker_one = find_object(&state, "Attacker One");
        let attacker_two = find_object(&state, "Attacker Two");
        let blocker_id = find_object(&state, "Removed Blocker");

        let mut combat = CombatState::new(p1);
        combat
            .attackers
            .insert(attacker_one, AttackTarget::Player(p2));
        combat
            .attackers
            .insert(attacker_two, AttackTarget::Player(p2));
        combat.blockers.insert(blocker_id, attacker_one);
        // Contrived (not reachable via real declare-blockers) but exercises the
        // "strip from every attacker's order list" branch of the helper directly.
        combat
            .damage_assignment_order
            .insert(attacker_one, vec![blocker_id]);
        combat
            .damage_assignment_order
            .insert(attacker_two, vec![blocker_id]);
        *state.combat_mut() = Some(combat);

        let mut ctx = EffectContext::new(p2, blocker_id, vec![]);
        execute_effect(
            &mut state,
            &Effect::RemoveFromCombat {
                target: CardEffectTarget::Source,
            },
            &mut ctx,
        );

        let combat = state.combat().as_ref().expect("combat still active");
        assert!(
            !combat.blockers.contains_key(&blocker_id),
            "blocker should be removed from combat.blockers"
        );
        assert!(
            !combat
                .damage_assignment_order
                .get(&attacker_one)
                .unwrap()
                .contains(&blocker_id),
            "blocker should be stripped from Attacker One's damage_assignment_order list"
        );
        assert!(
            !combat
                .damage_assignment_order
                .get(&attacker_two)
                .unwrap()
                .contains(&blocker_id),
            "blocker should be stripped from Attacker Two's damage_assignment_order list too"
        );
        // Attackers themselves are untouched -- only the blocker was removed.
        assert!(combat.attackers.contains_key(&attacker_one));
        assert!(combat.attackers.contains_key(&attacker_two));
    }
}

/// Regression guard: the `apply_regeneration` refactor (PB-OS6(g) factored its combat-
/// removal step 3 into the shared `remove_from_combat` helper) must still remove a
/// regenerated attacker from `combat.attackers`. Condensed mirror of
/// `tests/mechanics_m_z/regenerate.rs::test_regenerate_removes_from_combat_attacker`.
#[test]
fn test_regenerate_still_removes_from_combat() {
    use mtg_engine::{
        EffectDuration, ObjectFilter, ReplacementEffect, ReplacementId, ReplacementModification,
        ReplacementTrigger, SpellTarget,
    };

    let p1 = p(1);
    let p2 = p(2);

    let creature = ObjectSpec::creature(p1, "Regen Attacker", 2, 2)
        .with_damage(3)
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

    let creature_id = find_object(&state, "Regen Attacker");

    let mut combat = CombatState::new(p1);
    combat
        .attackers
        .insert(creature_id, AttackTarget::Player(p2));
    *state.combat_mut() = Some(combat);

    state
        .replacement_effects_mut()
        .push_back(ReplacementEffect {
            id: ReplacementId(0),
            source: Some(creature_id),
            controller: p1,
            duration: EffectDuration::UntilEndOfTurn,
            is_self_replacement: true,
            trigger: ReplacementTrigger::WouldBeDestroyed {
                filter: ObjectFilter::SpecificObject(creature_id),
            },
            modification: ReplacementModification::Regenerate,
        });
    *state.next_replacement_id_mut() = 1;

    let events = check_and_apply_sbas(&mut state);

    assert!(
        state.objects().get(&creature_id).is_some(),
        "creature should survive via the regeneration shield"
    );
    assert!(
        !state
            .combat()
            .as_ref()
            .map(|c| c.attackers.contains_key(&creature_id))
            .unwrap_or(false),
        "regenerated creature should still be removed from combat.attackers (refactor \
         regression guard)"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::Regenerated { .. })),
        "Regenerated event should still be emitted"
    );

    let _unused = SpellTarget {
        target: Target::Object(creature_id),
        zone_at_cast: None,
    };
}

// ── Registration smoke test ─────────────────────────────────────────────────────

/// `all_cards()` contains the three shipped defs, and legions_landing (NEW) is present
/// via the file-discovery build (SR-36 -- verified via `all_cards()`, not grep).
#[test]
fn test_os6_cards_registered() {
    let names = ["Delver of Secrets", "Legion's Landing", "Thaumatic Compass"];
    let all = all_cards();
    for name in names {
        assert!(
            all.iter().any(|d| d.name == name),
            "'{}' should be present in all_cards()",
            name
        );
    }
    let legions = all
        .iter()
        .find(|d| d.name == "Legion's Landing")
        .expect("Legion's Landing should be registered");
    assert!(
        legions.completeness.is_complete(),
        "legions_landing should be Complete"
    );
    let delver = all
        .iter()
        .find(|d| d.name == "Delver of Secrets")
        .expect("Delver of Secrets should be registered");
    assert!(
        delver.completeness.is_complete(),
        "delver_of_secrets should be Complete after PB-OS6(a)"
    );
    let thaumatic = all
        .iter()
        .find(|d| d.name == "Thaumatic Compass")
        .expect("Thaumatic Compass should be registered");
    assert!(
        thaumatic.completeness.is_complete(),
        "thaumatic_compass should be Complete after PB-OS6(g)"
    );
}

// ── Wire sentinels ───────────────────────────────────────────────────────────────

/// PB-OS6 bumped PROTOCOL_VERSION 20 -> 21 and HASH_SCHEMA_VERSION 57 -> 58 (four
/// closure-shape moves: Condition gained TopCardIsInstantOrSorcery /
/// YouAttackedWithNOrMore, Effect gained RemoveFromCombat, GameEvent gained
/// RemovedFromCombat). See crates/engine/src/rules/protocol.rs and
/// crates/engine/src/state/hash.rs for the authoritative bump.
#[test]
fn test_os6_version_sentinels() {
    assert_eq!(
        PROTOCOL_VERSION, 27,
        "PROTOCOL_VERSION should be 21 after PB-OS6"
    );
    assert_eq!(
        HASH_SCHEMA_VERSION, 63u8,
        "HASH_SCHEMA_VERSION should be 58 after PB-OS6"
    );
}
