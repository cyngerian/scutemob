//! PB-DX25b (`OOS-DX25-3`): positive probes for the announced-target ->
//! stack-entry id space repair.
//!
//! `casting.rs`'s `TargetSpellOrAbilityWithSingleTarget` / `TargetSpellWithSingleTarget`
//! validators, and `effects/mod.rs`'s `Effect::ChangeTargets` / `Effect::CopySpellOnStack`,
//! all took an ANNOUNCED target id (a `state.objects` CARD id, CR 601.2c) and
//! compared it against `StackObject::id` (a disjoint id space minted one line
//! later, `state/mod.rs::next_object_id`, `abilities.rs:1381`/`casting.rs:4425`).
//! The comparison type-checked and was unsatisfiable on any real cast:
//! Misdirection and Bolt Bend were `Complete`, deck-legal, and could never
//! resolve a legal target. `state::stack_registry::stack_index_for_announced_target`
//! is the one shared fix, consumed by all four sites plus
//! `Effect::CounterSpell` (PB-DX25's own consumer, re-expressed through the same
//! helper here).
//!
//! **Hard constraints (plan §5.1 AC 6297):** no direct call to the private
//! `casting::validate_object_satisfies_requirement`, and no hand-built
//! `StackObject` whose `id` equals its `source_object` in T1/T2 — every fixture
//! spell in those two tests reaches the stack by being CAST through a real
//! `Command::CastSpell`.

use std::sync::Arc;

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stack::{StackObject, StackObjectKind};
use mtg_engine::state::test_util;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    CardType, Command, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    GameStateError, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, SpellTarget, Step, Target,
    TargetRequirement, TypeLine, ZoneId,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_stack_obj_on_stack(state: &GameState, name_substr: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Stack && obj.characteristics.name.contains(name_substr)
        })
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no ZoneId::Stack object containing '{}' found", name_substr))
}

/// The `StackObject` entry whose `Spell`/`MutatingCreatureSpell` kind owns
/// `card_id` (CR 601.2c) -- i.e. the real stack-entry id a card id maps to.
fn stack_entry_id_for_card(state: &GameState, card_id: ObjectId) -> ObjectId {
    state
        .stack_objects()
        .iter()
        .find(|so| match &so.kind {
            StackObjectKind::Spell { source_object }
            | StackObjectKind::MutatingCreatureSpell { source_object, .. } => {
                *source_object == card_id
            }
            _ => false,
        })
        .map(|so| so.id)
        .unwrap_or_else(|| panic!("no StackObject entry owning card {:?} found", card_id))
}

fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
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
        })),
    )
}

/// Pass priority once per listed player, in order. If nobody acts in between,
/// this resolves the top of the stack once all have passed in succession
/// (CR 117.4). Mirrors `tests/mechanics_m_z/ward.rs`'s `pass_all`.
fn pass_n(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
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

/// A minimal single-target instant: deals `amount` damage to
/// `TargetRequirement::TargetAny` (CR 115.7a "single target" -- exactly one
/// declared target). Used as the "victim" spell Misdirection/Bolt Bend
/// redirect in T1/T2/T4/T5/T6/T8.
fn victim_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name} deals 3 damage to any target."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
                source: None,
            },
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn life_of(state: &GameState, player: PlayerId) -> i32 {
    state
        .players()
        .get(&player)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
        .life_total
}

// ── T1: misdirection announces AND resolves ─────────────────────────────────

/// CR 115.7a/115.7b/601.2a/601.2c -- Misdirection announces a real cast's
/// target and the redirect actually takes effect at resolution. This is the
/// batch's headline non-vacuity proof: the cast at step 3 is RED at HEAD (the
/// unfixed `casting.rs:6502`-era lookup made `TargetSpellWithSingleTarget`
/// unsatisfiable on any real cast) -- see
/// `memory/primitives/pb-DX25b-execution-notes.md` for the executed revert.
#[test]
fn t1_misdirection_announces_and_resolves() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let victim = victim_def("PB-DX25b T1 Victim", "pb-dx25b-t1-victim");
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![misdirection.clone(), victim.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25b T1 Victim")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // Step 1: p2 casts Victim targeting p3.
    let victim_hand_id = find_obj(&state, "PB-DX25b T1 Victim");
    let (state, cast_events) = cast(state, p2, victim_hand_id, vec![Target::Player(p3)])
        .unwrap_or_else(|e| panic!("p2's Victim cast must succeed: {:?}", e));
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p2)),
        "SpellCast event expected for Victim"
    );

    // Step 2: capture victim_card_id (state.objects, ZoneId::Stack) and
    // victim_entry_id (the StackObject's own id) -- and prove they differ.
    let victim_card_id = find_stack_obj_on_stack(&state, "Victim");
    let victim_entry_id = stack_entry_id_for_card(&state, victim_card_id);
    assert_ne!(
        victim_card_id, victim_entry_id,
        "non-vacuity anchor: a real cast must not collapse the announced-card-id \
         space and the stack-entry-id space onto one id"
    );

    // Step 3: p1 casts Misdirection announcing the victim's CARD id. THIS IS
    // THE ASSERTION THAT IS RED AT HEAD.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, misdirection_cast_events) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(victim_card_id)],
    )
    .unwrap_or_else(|e| {
        panic!(
            "p1's Misdirection cast targeting the victim's card id must succeed \
             -- if this panics, casting.rs's TargetSpellWithSingleTarget lookup \
             is comparing the wrong id space again: {:?}",
            e
        )
    });
    assert!(
        misdirection_cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "SpellCast event expected for Misdirection"
    );

    // Step 4: before resolution, the victim's declared target is unchanged.
    let victim_before = state
        .stack_objects()
        .iter()
        .find(|so| so.id == victim_entry_id)
        .expect("victim stack entry must still exist before resolution");
    assert_eq!(
        victim_before.targets[0].target,
        Target::Player(p3),
        "before Misdirection resolves, the victim must still target p3"
    );

    // Step 5: resolve Misdirection by real priority passes (APNAP: p1, p2, p3).
    let (state, resolve_events) = pass_n(state, &[p1, p2, p3]);
    let targets_changed = resolve_events.iter().find_map(|e| match e {
        GameEvent::TargetsChanged {
            stack_object_id,
            old_targets,
            new_targets,
        } => Some((*stack_object_id, old_targets.clone(), new_targets.clone())),
        _ => None,
    });
    let (changed_id, old_targets, new_targets) =
        targets_changed.expect("TargetsChanged event must be emitted when Misdirection resolves");
    assert_eq!(
        changed_id, victim_entry_id,
        "TargetsChanged.stack_object_id must name the STACK-ENTRY id \
         (victim_entry_id), not the announced card id (victim_card_id) -- \
         the field's OWN doc comment (rules/events.rs:1421-1422, \"The stack \
         object whose targets changed\") says so. No consumer reads this \
         field today (event_view.rs:927 discards it via `..`); this is a \
         contract correction, not a compatibility fix for an existing \
         reader (PB-DX25b review Finding E5)."
    );
    assert_eq!(old_targets[0].target, Target::Player(p3));
    assert_eq!(new_targets[0].target, Target::Player(p1));

    let victim_after = state
        .stack_objects()
        .iter()
        .find(|so| so.id == victim_entry_id)
        .expect("victim stack entry must still exist after Misdirection resolves");
    assert_eq!(
        victim_after.targets[0].target,
        Target::Player(p1),
        "the victim's StackObject must now target p1"
    );

    // Step 6: resolve the victim too (APNAP again) and check the end-to-end
    // life-total observable -- an event-only assertion would pass even if the
    // `iter_mut` write at effects/mod.rs's ChangeTargets arm were dropped.
    let life_p1_before = life_of(&state, p1);
    let life_p3_before = life_of(&state, p3);
    let (state, _victim_resolve_events) = pass_n(state, &[p1, p2, p3]);
    assert_eq!(
        life_of(&state, p1),
        life_p1_before - 3,
        "p1's life must drop by 3 -- the redirected victim resolved against p1"
    );
    assert_eq!(
        life_of(&state, p3),
        life_p3_before,
        "p3's life must be unchanged -- Misdirection redirected the victim away from p3"
    );
}

// ── T2: bolt_bend announces AND resolves against a spell ────────────────────

/// CR 115.7a/115.7b -- Bolt Bend, same shape as T1, through
/// `TargetSpellOrAbilityWithSingleTarget` (C1, `casting.rs:6476`-era).
#[test]
fn t2_bolt_bend_announces_and_resolves() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let victim = victim_def("PB-DX25b T2 Victim", "pb-dx25b-t2-victim");
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone(), victim.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                red: 1,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p1, "Bolt Bend")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(bolt_bend.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25b T2 Victim")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let victim_hand_id = find_obj(&state, "PB-DX25b T2 Victim");
    let (state, _cast_events) = cast(state, p2, victim_hand_id, vec![Target::Player(p3)])
        .unwrap_or_else(|e| panic!("p2's Victim cast must succeed: {:?}", e));

    let victim_card_id = find_stack_obj_on_stack(&state, "Victim");
    let victim_entry_id = stack_entry_id_for_card(&state, victim_card_id);
    assert_ne!(victim_card_id, victim_entry_id);

    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");
    let (state, _bolt_bend_cast_events) = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::Object(victim_card_id)],
    )
    .unwrap_or_else(|e| {
        panic!(
            "p1's Bolt Bend cast targeting the victim's card id must succeed: {:?}",
            e
        )
    });

    let (state, resolve_events) = pass_n(state, &[p1, p2, p3]);
    let targets_changed = resolve_events.iter().find_map(|e| match e {
        GameEvent::TargetsChanged {
            stack_object_id,
            new_targets,
            ..
        } => Some((*stack_object_id, new_targets.clone())),
        _ => None,
    });
    let (changed_id, new_targets) =
        targets_changed.expect("TargetsChanged event must be emitted when Bolt Bend resolves");
    assert_eq!(changed_id, victim_entry_id);
    assert_eq!(new_targets[0].target, Target::Player(p1));

    let life_p1_before = life_of(&state, p1);
    let life_p3_before = life_of(&state, p3);
    let (state, _) = pass_n(state, &[p1, p2, p3]);
    assert_eq!(life_of(&state, p1), life_p1_before - 3);
    assert_eq!(life_of(&state, p3), life_p3_before);
}

// ── T3: the ability half does NOT work (pinned wrong-way-round) ─────────────

/// `OOS-DX25b-1` -- Bolt Bend's "or ability" half is still unreachable. An
/// activated ability's stack entry is minted at `abilities.rs:1381` and never
/// added to `state.objects`, so (a) the offer layer
/// (`queries::legal_targets_per_slot`) cannot enumerate it, and (b) a cast
/// naming it fails, ultimately because `validate_object_satisfies_requirement`'s
/// opening `state.objects.get(&id).ok_or(ObjectNotFound)?` can never find it --
/// though the specific `ObjectNotFound` variant does not reach the
/// `Command::CastSpell` caller: the bipartite target/slot matcher in
/// `casting.rs` swallows it into the generic "could not be matched to a
/// requirement slot" `InvalidTarget` (see (b1)'s own correction below).
/// Closing this needs a NEW target id space (`Target::StackObject`, a wire
/// change) -- out of this batch's scope.
#[test]
fn t3_ability_half_is_still_unreachable() {
    use mtg_engine::state::{ActivatedAbility, ActivationCost};

    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    let ability = ActivatedAbility {
        targets: vec![TargetRequirement::TargetCreature],
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
            remove_counter_cost: None,
            exile_self: false,
            exert: false,
            life_cost: 0,
            sacrifice_exclude_self: false,
            exile_self_from_hand: false,
        },
        description: "{T}: Destroy target creature".to_string(),
        effect: Some(Effect::DestroyPermanent {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: false,
        }),
        sorcery_speed: false,
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: None,
    };
    let ability_source =
        ObjectSpec::creature(p2, "T3 Ability Source", 1, 1).with_activated_ability(ability);
    let victim_creature = ObjectSpec::creature(p1, "T3 Victim Creature", 1, 1);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                red: 1,
                ..Default::default()
            },
        )
        .object(ability_source)
        .object(victim_creature)
        .object(
            ObjectSpec::card(p1, "Bolt Bend")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(bolt_bend.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "T3 Ability Source");
    let target_creature_id = find_obj(&state, "T3 Victim Creature");
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");

    // p2 activates the ability targeting the victim creature.
    let (state, _activate_events) = {
        let mut s = state;
        s.turn_mut().priority_holder = Some(p2);
        process_command(
            s,
            Command::ActivateAbility {
                player: p2,
                source: source_id,
                ability_index: 0,
                targets: vec![Target::Object(target_creature_id)],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
                modes_chosen: vec![],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        )
        .unwrap_or_else(|e| panic!("p2's ability activation must succeed: {:?}", e))
    };

    assert_eq!(
        state.stack_objects().len(),
        1,
        "only the ActivatedAbility should be on the stack"
    );
    let ability_stack_id = state.stack_objects().back().unwrap().id;
    assert!(
        !state.objects().contains_key(&ability_stack_id),
        "an activated ability's stack-entry id is never added to state.objects \
         (abilities.rs:1381) -- this IS OOS-DX25b-1's mechanism, asserted directly"
    );

    // (a) the offer layer cannot enumerate the ability's stack-entry id, nor
    // the ability's source permanent (wrong zone), for
    // TargetSpellOrAbilityWithSingleTarget.
    let candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p1,
        bolt_bend_hand_id,
        &[TargetRequirement::TargetSpellOrAbilityWithSingleTarget],
    );
    assert_eq!(candidates.len(), 1);
    assert!(
        !candidates[0].contains(&Target::Object(ability_stack_id)),
        "the ability's own stack-entry id must never be offered as a candidate \
         (it is not a state.objects key)"
    );
    assert!(
        !candidates[0].contains(&Target::Object(source_id)),
        "the ability's source permanent must never be offered as a candidate \
         (it is on the Battlefield, not the Stack)"
    );

    // (b1) a cast naming the ability's stack-entry id fails. **Correction to
    // the plan's prediction**: `validate_object_satisfies_requirement`'s
    // `ObjectNotFound` never reaches the caller directly -- the bipartite
    // slot-matching pass (`casting.rs:6089-6098`'s `target_satisfies` closure)
    // swallows any `Err` into a bare `.is_ok() == false` and, when no
    // requirement slot matches, reports the GENERIC "declared N target(s) but
    // N could not be matched to a requirement slot" `InvalidTarget`, not the
    // specific `ObjectNotFound`. The underlying mechanism (the lookup fails)
    // is unchanged; only the error VARIANT observed at the `Command::CastSpell`
    // boundary differs from what the plan predicted.
    let state2 = state.clone();
    let result_stack_id = cast(
        state2,
        p1,
        bolt_bend_hand_id,
        vec![Target::Object(ability_stack_id)],
    );
    assert!(
        matches!(result_stack_id, Err(GameStateError::InvalidTarget(_))),
        "OOS-DX25b-1: casting Bolt Bend at the ability's stack-entry id must \
         fail (the lookup can never find it, since it is not a state.objects \
         key) -- got: {:?}",
        result_stack_id.map(|_| ())
    );

    // (b2) a cast naming the ability's source permanent fails with
    // InvalidTarget (wrong zone: Battlefield, not Stack).
    let result_source = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::Object(source_id)],
    );
    assert!(
        matches!(result_source, Err(GameStateError::InvalidTarget(_))),
        "OOS-DX25b-1: casting Bolt Bend at the ability's SOURCE permanent must \
         fail with InvalidTarget (wrong zone) -- got: {:?}",
        result_source.map(|_| ())
    );
}

// ── T4: CR 608.2b fizzle on the newly reachable path ─────────────────────────

/// CR 608.2b -- a victim spell countered before Misdirection resolves makes
/// Misdirection fizzle (its target's card gets a new ObjectId under CR 400.7,
/// which is no longer a live `state.objects` key). Three real casts, LIFO:
/// Victim, Misdirection (targeting Victim), Counterspell (targeting Victim's
/// card) -- the counter resolves first.
#[test]
fn t4_cr_608_2b_fizzle_on_the_newly_reachable_path() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let counterspell = mtg_engine::cards::defs::counterspell::card();
    let victim = victim_def("PB-DX25b T4 Victim", "pb-dx25b-t4-victim");
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![
        misdirection.clone(),
        counterspell.clone(),
        victim.clone(),
    ]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .player_mana(
            p3,
            ManaPool {
                blue: 2,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25b T4 Victim")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p3, "Counterspell")
                .in_zone(ZoneId::Hand(p3))
                .with_card_id(counterspell.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let victim_hand_id = find_obj(&state, "PB-DX25b T4 Victim");
    let (state, _) = cast(state, p2, victim_hand_id, vec![Target::Player(p3)])
        .unwrap_or_else(|e| panic!("Victim cast must succeed: {:?}", e));
    let victim_card_id = find_stack_obj_on_stack(&state, "Victim");

    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(victim_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));
    let misdirection_card_id = find_stack_obj_on_stack(&state, "Misdirection");
    let misdirection_entry_id = stack_entry_id_for_card(&state, misdirection_card_id);

    let counterspell_hand_id = find_obj(&state, "Counterspell");
    let (state, _) = cast(
        state,
        p3,
        counterspell_hand_id,
        vec![Target::Object(victim_card_id)],
    )
    .unwrap_or_else(|e| panic!("Counterspell cast must succeed: {:?}", e));

    assert_eq!(
        state.stack_objects().len(),
        3,
        "Victim + Misdirection + Counterspell should all be on the stack"
    );

    // Resolve the Counterspell (top of stack, LIFO) -- counters the Victim.
    let (state, counter_resolve_events) = pass_n(state, &[p3, p1, p2]);
    assert!(
        counter_resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "the Counterspell must resolve and counter the Victim"
    );
    assert!(
        !state.objects().contains_key(&victim_card_id),
        "CR 400.7: the countered Victim's card must have a NEW ObjectId now -- \
         the old id is dead"
    );

    // Resolve Misdirection next: its target (the victim's OLD card id) is no
    // longer legal (CR 608.2b) -- it must fizzle, not redirect anything.
    //
    // **Correction to the plan's prediction**: `SpellFizzled.source_object_id`
    // is NOT Misdirection's own (pre-fizzle) card id -- `resolution.rs:247-249`
    // moves the fizzling card to the graveyard as part of emitting the event,
    // which mints a NEW ObjectId under CR 400.7 (`move_object_to_zone`) before
    // the event is even constructed. `source_object_id` therefore names the
    // GRAVEYARD object, not the stack object. `stack_object_id` (the
    // stack-ENTRY id, `misdirection_entry_id`) is the field that stably
    // identifies "which spell fizzled" across that zone move.
    let (state, misdirection_resolve_events) = pass_n(state, &[p1, p2, p3]);
    let fizzled = misdirection_resolve_events.iter().find_map(|e| match e {
        GameEvent::SpellFizzled {
            stack_object_id, ..
        } => Some(*stack_object_id),
        _ => None,
    });
    assert_eq!(
        fizzled,
        Some(misdirection_entry_id),
        "Misdirection must fizzle (CR 608.2b), naming its STACK-ENTRY id -- got \
         fizzle events: {:?}",
        misdirection_resolve_events
            .iter()
            .filter(|e| matches!(e, GameEvent::SpellFizzled { .. }))
            .collect::<Vec<_>>()
    );
    assert!(
        !misdirection_resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "a fizzled Misdirection must not emit TargetsChanged"
    );
    assert!(
        state.objects().values().any(|o| {
            o.characteristics.name == "Misdirection" && matches!(o.zone, ZoneId::Graveyard(_))
        }),
        "Misdirection's card must be in a graveyard after fizzling"
    );
}

// ── T5: CR 707.10 -- a copy of a spell is not announceable ──────────────────

/// `OOS-DX25b-2` -- a copy of a spell IS a spell (CR 707.10) but owns no card
/// of its own; the `!so.is_copy` guard in `stack_index_for_announced_target`
/// is what keeps a copy from being findable by the original's card id once
/// the original itself is no longer on the stack. Reaches the copy through the
/// real, `pub` `rules::copy::copy_spell_on_stack` (PB-DX25's own precedent for
/// this style of fixture).
///
/// **PB-DX25b review Finding E7**: this test's second half (removing the
/// ORIGINAL's `StackObject` while deliberately leaving its card in
/// `state.objects` -- a configuration this test's own body already labels
/// synthetic) is the ONLY assertion in the tree that discriminates the
/// `!so.is_copy` guard. PB-DX25's own `pb_dx25_counterspell_stack_shapes`
/// copy probes do NOT discriminate it: their scenarios keep the original
/// present, so `position()`'s first-match-wins semantics land on the original
/// regardless of the guard (confirmed by executing the guard's deletion
/// against that suite during this batch's revert matrix, V2 --
/// `pb-DX25b-execution-notes.md` §4 -- all six of that suite's tests stayed
/// green). A guard shared by five consumers (`Effect::CounterSpell`,
/// `casting.rs` C1/C2, `effects/mod.rs` C3/C4) resting on one synthetic
/// assertion is a real, stated residual, not a gap this test hides.
#[test]
fn t5_copy_is_not_announceable() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ObjectSpec::card(p2, "PB-DX25b T5 Victim").in_zone(ZoneId::Stack))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let victim_card_id = find_obj(&state, "PB-DX25b T5 Victim");
    let victim_entry_id = test_util::next_object_id(&mut state);
    assert_ne!(victim_entry_id, victim_card_id);
    let victim_entry = StackObject {
        id: victim_entry_id,
        controller: p2,
        kind: StackObjectKind::Spell {
            source_object: victim_card_id,
        },
        targets: vec![SpellTarget {
            target: Target::Player(p3),
            zone_at_cast: None,
        }],
        ..blank_stack_object()
    };
    state.stack_objects_mut().push_back(victim_entry);

    // CR 707.10: copy the spell -- `copy.rs` clones `kind` wholesale, so the
    // copy's `source_object` also names `victim_card_id`.
    let (copy_entry_id, _copy_event) =
        mtg_engine::rules::copy::copy_spell_on_stack(&mut state, victim_entry_id, p1, false)
            .unwrap_or_else(|e| panic!("copy_spell_on_stack failed: {:?}", e));
    assert_eq!(
        state.stack_objects().len(),
        2,
        "original + copy on the stack"
    );

    // While the ORIGINAL is still present, the announced card id must resolve
    // to the ORIGINAL, never the copy (`copy.rs` pushes the copy ABOVE the
    // original; `position()` returns the first/lowest-index match).
    let pos_with_original = mtg_engine::state::stack_registry::stack_index_for_announced_target(
        state.stack_objects(),
        victim_card_id,
    );
    assert_eq!(
        pos_with_original.map(|i| state.stack_objects()[i].id),
        Some(victim_entry_id),
        "while the original is present, the announced card id must resolve to \
         the ORIGINAL, never the copy"
    );

    // Remove the ORIGINAL's StackObject entry, leaving only the copy --
    // synthetic (a real removal mints a new card ObjectId under CR 400.7 and
    // would take the card OUT of state.objects; this fixture deliberately
    // leaves the card object in place to isolate the `!so.is_copy`
    // disambiguation from the separate dead-id filter
    // `resolve_effect_target_list_indexed` relies on in production -- PB-DX25's
    // `test_dx25_countering_a_copy_moves_no_card` is the real-cast version of
    // this same argument).
    let original_pos = state
        .stack_objects()
        .iter()
        .position(|s| s.id == victim_entry_id)
        .unwrap();
    state.stack_objects_mut().remove(original_pos);
    assert_eq!(state.stack_objects().len(), 1, "only the copy remains");

    // CR 707.10: the copy owns no card of its own -- with only the copy left,
    // the announced card id must resolve to NOTHING, not to the copy.
    let pos_copy_only = mtg_engine::state::stack_registry::stack_index_for_announced_target(
        state.stack_objects(),
        victim_card_id,
    );
    assert_eq!(
        pos_copy_only, None,
        "OOS-DX25b-2: a copy of a spell must not be announceable by the \
         original's card id, even when it is the only entry left whose cloned \
         kind names that card (CR 707.10 -- a copy owns no card of its own)"
    );

    // A copy's own StackObject id is never added to state.objects, so no cast
    // can ever announce it -- casting.rs's opening
    // `state.objects.get(&id).ok_or(ObjectNotFound)?` rejects it before any
    // TargetSpellWithSingleTarget logic runs.
    assert!(
        !state.objects().contains_key(&copy_entry_id),
        "a copy's own StackObject id must never be a state.objects key -- no \
         cast can ever announce it"
    );
}

fn blank_stack_object() -> StackObject {
    StackObject {
        id: ObjectId(0),
        controller: p(1),
        kind: StackObjectKind::Spell {
            source_object: ObjectId(0),
        },
        targets: vec![],
        target_requirements: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_warped: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_cast_as_adventure: false,
        cast_right_half: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        x_value: 0,
        evidence_collected: false,
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_lki: vec![],
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
        defending_player: None,
    }
}

// ── T6: Bolt Bend's duplicate-target ruling ──────────────────────────────────

/// Bolt Bend, 2024-11-08 ruling: "If a spell or ability targets the same
/// player or object multiple times, you can't target it with Bolt Bend." A
/// duplicated target is TWO declared-target entries, so `targets.len() != 1`
/// rejects it. Pins the count guard on the REPAIRED id space (distinct
/// announced-card-id / stack-entry-id, post-PB-DX25b).
#[test]
fn t6_bolt_bend_rejects_duplicate_target_spell() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .object(ObjectSpec::card(p2, "PB-DX25b T6 Victim").in_zone(ZoneId::Stack))
        .object(
            ObjectSpec::card(p1, "Bolt Bend")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(bolt_bend.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let victim_card_id = find_obj(&state, "PB-DX25b T6 Victim");
    let victim_entry_id = test_util::next_object_id(&mut state);
    assert_ne!(victim_entry_id, victim_card_id);
    // The victim spell targets p3 TWICE -- a duplicated target, CR-legal for
    // e.g. "deals damage to each of up to two targets" naming the same player.
    let victim_entry = StackObject {
        id: victim_entry_id,
        controller: p2,
        kind: StackObjectKind::Spell {
            source_object: victim_card_id,
        },
        targets: vec![
            SpellTarget {
                target: Target::Player(p3),
                zone_at_cast: None,
            },
            SpellTarget {
                target: Target::Player(p3),
                zone_at_cast: None,
            },
        ],
        ..blank_stack_object()
    };
    state.stack_objects_mut().push_back(victim_entry);

    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");
    let result = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::Object(victim_card_id)],
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "Bolt Bend must reject a spell with a duplicated target (2 entries, \
         target_count != 1) -- 2024-11-08 ruling -- got: {:?}",
        result.map(|_| ())
    );
}

// ── T7: Ward still finds its target (regression guard) ──────────────────────

/// Ward is the only production consumer of the DIRECT-id clause in
/// `stack_index_for_announced_target` (`so.id == announced`, CR 702.21a): the
/// engine-internal Ward trigger names the targeting spell/ability's
/// STACK-ENTRY id, never its card id. A "simplification" that dropped that
/// clause (keeping only the card-owning-kind clause) would pass every
/// Misdirection/Bolt Bend probe in this file and still break Ward.
#[test]
fn t7_ward_still_finds_its_target() {
    use mtg_engine::rules::command::CastSpellData as WardCastSpellData;
    use mtg_engine::{KeywordAbility, ManaColor};

    let p1 = p(1);
    let p2 = p(2);

    let doom_blade = CardDefinition {
        card_id: CardId("pb-dx25b-t7-doom-blade".to_string()),
        name: "PB-DX25b T7 Doom Blade".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: "Destroy target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![doom_blade.clone()]);
    let ward_creature =
        ObjectSpec::creature(p1, "T7 Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let spell = ObjectSpec::card(p2, "PB-DX25b T7 Doom Blade")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(doom_blade.card_id.clone())
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward_creature)
        .object(spell)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p2);

    let creature_id = find_obj(&state, "T7 Ward Creature");
    let spell_id = find_obj(&state, "PB-DX25b T7 Doom Blade");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell(Box::new(WardCastSpellData {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(creature_id)],
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
        })),
    )
    .unwrap_or_else(|e| panic!("Doom Blade cast must succeed: {:?}", e));

    assert!(
        cast_events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTargeted { target_id, .. } if *target_id == creature_id
        )),
        "Ward trigger must fire (PermanentTargeted)"
    );
    assert_eq!(
        state.stack_objects().len(),
        2,
        "stack must have Doom Blade + the Ward trigger"
    );

    let (state, resolve_events) = process_command(state, Command::PassPriority { player: p2 })
        .and_then(|(s, mut ev)| {
            let (s2, ev2) = process_command(s, Command::PassPriority { player: p1 })?;
            ev.extend(ev2);
            Ok((s2, ev))
        })
        .unwrap_or_else(|e| panic!("passing to resolve the ward trigger failed: {:?}", e));

    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "CR 702.21a: Ward's CounterSpell must still find Doom Blade through the \
         DIRECT-id clause (the ward trigger targets Doom Blade's STACK-ENTRY \
         id, not its card id) and counter it -- got: {:?}",
        resolve_events
    );
    assert!(
        state.objects().values().any(|o| {
            o.characteristics.name == "T7 Ward Creature" && o.zone == ZoneId::Battlefield
        }),
        "the ward creature must survive"
    );
}

// ── T8: Effect::CopySpellOnStack finds its target (C4, synthetic) ───────────

/// C4 (plan §2.2/§3.2) -- `Effect::CopySpellOnStack` took an ANNOUNCED id and
/// compared it against `StackObject::id` directly, the same defect class as
/// C1-C3. **Synthetic**: no corpus def currently uses
/// `Effect::CopySpellOnStack` (plan §1 fact 9 -- the dispatch brief's claim
/// that `plumb_the_forbidden`/`complete_the_circuit` used it was refuted; both
/// only mention the effect in PROSE/comments, never in code). This probe earns
/// no `completeness` flip; it exists so the fourth site is not "fixed and
/// untested".
#[test]
fn t8_copy_spell_on_stack_finds_its_target() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ObjectSpec::card(p2, "PB-DX25b T8 Victim").in_zone(ZoneId::Stack))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let victim_card_id = find_obj(&state, "PB-DX25b T8 Victim");
    let victim_entry_id = test_util::next_object_id(&mut state);
    assert_ne!(victim_entry_id, victim_card_id);
    let victim_entry = StackObject {
        id: victim_entry_id,
        controller: p2,
        kind: StackObjectKind::Spell {
            source_object: victim_card_id,
        },
        targets: vec![SpellTarget {
            target: Target::Player(p3),
            zone_at_cast: None,
        }],
        ..blank_stack_object()
    };
    state.stack_objects_mut().push_back(victim_entry);
    assert_eq!(state.stack_objects().len(), 1);

    let source = ObjectId(0);
    let mut ctx = EffectContext::new(
        p1,
        source,
        vec![SpellTarget {
            target: Target::Object(victim_card_id),
            zone_at_cast: Some(ZoneId::Stack),
        }],
    );
    let effect = Effect::CopySpellOnStack {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        count: EffectAmount::Fixed(1),
    };
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.stack_objects().len(),
        2,
        "a copy must be created -- the announced CARD id (victim_card_id) must \
         resolve to the victim's stack entry, not fail silently"
    );
    assert!(
        state.stack_objects().iter().any(|s| s.is_copy
            && matches!(&s.kind, StackObjectKind::Spell { source_object } if *source_object == victim_card_id)),
        "the new copy must clone the ORIGINAL's kind (source_object == victim_card_id)"
    );
}

// ── T9: object-target redirect obeys the original requirement ──────────────

/// `OOS-DX25b-3` CLOSED (PB-DX25b review Finding E1, HIGH; fixed by
/// PB-DX25c). CR 115.7a: "each target can be changed only to ANOTHER LEGAL
/// target." Before PB-DX25c, the `Target::Object` branch of
/// `Effect::ChangeTargets` picked the smallest `ObjectId` in the recorded
/// `zone_at_cast` with NO check that the new object satisfied the original
/// spell's `TargetRequirement` -- a "destroy target CREATURE" spell could be
/// redirected onto a LAND. `rules::retarget::plan_target_change` now
/// delegates the whole decision to `casting::validate_targets_inner`, the
/// same collective legality arithmetic a real cast is checked against.
///
/// **This test was PINNED WRONG-WAY-ROUND for one batch** (the
/// `blinkmoth_nexus` pattern, PB-DX19), asserting what the engine DID at
/// HEAD rather than what CR 115.7a requires. PB-DX25c inverts it: on THIS
/// fixture (the only other object on the board is a land, and the victim
/// spell's requirement is `TargetCreature`), CR 115.7a's own fallback
/// applies -- "If a target can't be changed to another legal target, the
/// original target is unchanged." See T9b below for the sibling fixture
/// that proves the redirect DOES fire when a legal alternative exists (a
/// `plan_target_change` that always returned `None` would pass THIS test
/// alone, without T9b).
#[test]
fn t9_object_target_redirect_obeys_the_original_requirement() {
    let p1 = p(1);
    let p2 = p(2);

    let destroy_creature = CardDefinition {
        card_id: CardId("pb-dx25b-t9-destroy-creature".to_string()),
        name: "PB-DX25b T9 Destroy Creature".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: "Destroy target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), destroy_creature.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                black: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::land(p1, "T9 Bystander Land"))
        .object(ObjectSpec::creature(p1, "T9 Victim Creature", 2, 2))
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25b T9 Destroy Creature")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(destroy_creature.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let creature_id = find_obj(&state, "T9 Victim Creature");
    let destroy_hand_id = find_obj(&state, "PB-DX25b T9 Destroy Creature");

    // p2 casts "destroy target creature" at p1's creature.
    let (state, _) = cast(
        state,
        p2,
        destroy_hand_id,
        vec![Target::Object(creature_id)],
    )
    .unwrap_or_else(|e| panic!("Destroy Creature cast must succeed: {:?}", e));
    let destroy_card_id = find_stack_obj_on_stack(&state, "Destroy Creature");

    // p1 casts Misdirection targeting that spell.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(destroy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    // Resolve Misdirection first (LIFO). CR 115.7a requires the redirect to
    // land on ANOTHER LEGAL target -- the victim's own requirement is
    // TargetCreature, and the only other object on the board is a land, so
    // there is no legal alternative. CR 115.7a's fallback applies: "the
    // original target is unchanged."
    let (state, resolve_events) = pass_n(state, &[p1, p2]);
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "OOS-DX25b-3 (CR 115.7a): with no legal alternative (the only other \
         object is a land, and the victim's requirement is TargetCreature), \
         Misdirection must resolve WITHOUT changing the victim's target -- \
         no TargetsChanged event. Got: {:?}",
        resolve_events
    );

    // Resolve the (unchanged) Destroy Creature spell -- it still destroys
    // its ORIGINAL target, the creature.
    let (state, _) = pass_n(state, &[p1, p2]);

    assert!(
        state.objects().values().any(|o| o.characteristics.name == "T9 Bystander Land"
            && o.zone == ZoneId::Battlefield),
        "OOS-DX25b-3 CLOSED (CR 115.7a): the bystander land must survive -- \
         the redirect must not have landed on it, since a land does not \
         satisfy the victim spell's TargetCreature requirement"
    );
    assert!(
        !state.objects().values().any(
            |o| o.characteristics.name == "T9 Victim Creature" && o.zone == ZoneId::Battlefield
        ),
        "the ORIGINAL target (the creature) must be destroyed -- the redirect \
         found no legal alternative, so Destroy Creature resolves against its \
         unchanged original target"
    );
}

// ── T9b: object-target redirect DOES fire when a legal alternative exists ──

/// The sibling of T9 above, using the plan's exact prescription (§5.1): same
/// board, plus a SECOND creature (controlled by p2, the victim spell's
/// caster) so a legal `TargetCreature` alternative actually exists. Proves
/// the fix is not simply "never change anything" -- without this test, a
/// `plan_target_change` that always returned `None` would pass T9 alone.
#[test]
fn t9b_object_target_redirect_fires_with_a_legal_alternative() {
    let p1 = p(1);
    let p2 = p(2);

    let destroy_creature = CardDefinition {
        card_id: CardId("pb-dx25c-t9b-destroy-creature".to_string()),
        name: "PB-DX25c T9b Destroy Creature".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: "Destroy target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), destroy_creature.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                black: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::land(p1, "T9b Bystander Land"))
        .object(ObjectSpec::creature(p1, "T9b Victim Creature", 2, 2))
        .object(ObjectSpec::creature(p2, "T9b Alternative Creature", 3, 3))
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c T9b Destroy Creature")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(destroy_creature.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let creature_id = find_obj(&state, "T9b Victim Creature");
    let alt_creature_id = find_obj(&state, "T9b Alternative Creature");
    let destroy_hand_id = find_obj(&state, "PB-DX25c T9b Destroy Creature");

    // p2 casts "destroy target creature" at p1's creature.
    let (state, _) = cast(
        state,
        p2,
        destroy_hand_id,
        vec![Target::Object(creature_id)],
    )
    .unwrap_or_else(|e| panic!("Destroy Creature cast must succeed: {:?}", e));
    let destroy_card_id = find_stack_obj_on_stack(&state, "Destroy Creature");

    // p1 casts Misdirection targeting that spell.
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(destroy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));

    // Resolve Misdirection first (LIFO). A legal alternative (the second
    // creature) exists, so the redirect must fire.
    let (state, resolve_events) = pass_n(state, &[p1, p2]);
    let changed = resolve_events.iter().find_map(|e| match e {
        GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
        _ => None,
    });
    let new_targets =
        changed.unwrap_or_else(|| panic!("Misdirection must redirect: {:?}", resolve_events));
    assert_eq!(
        new_targets.len(),
        1,
        "the redirected target set must still have exactly one target"
    );
    assert_eq!(
        new_targets[0].target,
        Target::Object(alt_creature_id),
        "the redirect must land on the SECOND creature (the only legal \
         alternative), not the land"
    );
    assert_eq!(
        new_targets[0].zone_at_cast,
        Some(ZoneId::Battlefield),
        "CR 608.2b: zone_at_cast must be rebuilt from the NEW target's own \
         zone"
    );

    // Resolve the (redirected) Destroy Creature spell.
    let (state, _) = pass_n(state, &[p1, p2]);

    assert!(
        state.objects().values().any(
            |o| o.characteristics.name == "T9b Bystander Land" && o.zone == ZoneId::Battlefield
        ),
        "the bystander land must survive -- it was never a candidate"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "T9b Victim Creature"
                && o.zone == ZoneId::Battlefield),
        "the ORIGINAL target must survive -- the redirect moved the spell off it"
    );
    assert!(
        !state.objects().values().any(|o| {
            o.characteristics.name == "T9b Alternative Creature" && o.zone == ZoneId::Battlefield
        }),
        "the ALTERNATIVE creature (the new target) must be destroyed"
    );
}

// ── T10: untimely_malfunction mode 1's modal target index (plan §8 R5) ──────

/// Plan §8 R5 -- verify BY PROBE whether announcing only mode 1's target lands
/// at `ctx.targets[1]` (matching `Effect::ChangeTargets`'s `DeclaredTarget {
/// index: 1 }`) or at `ctx.targets[0]` (in which case mode 1 is still broken
/// after this batch, despite the id-space repair). Untimely Malfunction pools
/// all three modes' `TargetRequirement`s into ONE flat list (`mode_targets:
/// None`), so it casts through the bipartite `validate_targets_with_source`
/// matcher (`casting.rs:3696-3727`, `validate_targets_inner`), not the
/// per-mode-sliced `mode_targets_active` path -- and `target_count_range`
/// (`casting.rs:6025`) requires exactly 3 declared targets (min == max == 3,
/// one per mandatory pooled slot) REGARDLESS of which single mode is chosen
/// (confirmed by executing this probe with only 1 declared target first: `Err
/// InvalidTarget("expected 3..=3 target(s) but got 1")`, before the fix
/// below). So a real cast must announce a target for ALL THREE pooled
/// TargetRequirements every time, and `validate_mapped_targets`'s own doc
/// (`casting.rs:6226-6227`) states the returned `Vec<SpellTarget>` "preserves
/// declaration order (positions are NOT reordered to match requirement/slot
/// order)" -- so `ctx.targets[1]` is whichever target the CALLER declared
/// SECOND, not whichever target satisfied slot 1. This probe declares in
/// pooled-slot order (artifact, spell, creature) to test the id-space repair
/// on its own terms, per the card def's own header comment convention.
#[test]
fn t10_untimely_malfunction_mode1_target_index() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let um = mtg_engine::cards::defs::untimely_malfunction::card();
    let victim = victim_def("PB-DX25b T10 Victim", "pb-dx25b-t10-victim");
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![um.clone(), victim.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 1,
                red: 1,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::artifact(p1, "T10 Artifact Target"))
        .object(ObjectSpec::creature(p1, "T10 Creature Target", 2, 2))
        .object(
            ObjectSpec::card(p1, "Untimely Malfunction")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(um.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25b T10 Victim")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let victim_hand_id = find_obj(&state, "PB-DX25b T10 Victim");
    let (state, _) = cast(state, p2, victim_hand_id, vec![Target::Player(p3)])
        .unwrap_or_else(|e| panic!("Victim cast must succeed: {:?}", e));
    let victim_card_id = find_stack_obj_on_stack(&state, "Victim");
    let victim_entry_id = stack_entry_id_for_card(&state, victim_card_id);

    let artifact_id = find_obj(&state, "T10 Artifact Target");
    let creature_id = find_obj(&state, "T10 Creature Target");
    let um_hand_id = find_obj(&state, "Untimely Malfunction");
    let mut s = state;
    s.turn_mut().priority_holder = Some(p1);
    let (state, _cast_events) = process_command(
        s,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: um_hand_id,
            // Declared in POOLED-SLOT order (artifact, spell, creature) --
            // matching the card def's own header comment convention, since
            // `validate_mapped_targets` preserves declaration order rather
            // than reordering by slot.
            targets: vec![
                Target::Object(artifact_id),
                Target::Object(victim_card_id),
                Target::Object(creature_id),
            ],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            prototype: false,
            modes_chosen: vec![1],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        })),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Untimely Malfunction mode-1 cast (announcing all three pooled \
             targets, per casting.rs's flat target_count_range) must \
             succeed: {:?}",
            e
        )
    });

    let (state, resolve_events) = pass_n(state, &[p1, p2, p3]);
    let targets_changed = resolve_events.iter().any(|e| {
        matches!(e, GameEvent::TargetsChanged { stack_object_id, .. } if *stack_object_id == victim_entry_id)
    });
    assert!(
        targets_changed,
        "Untimely Malfunction mode 1 must actually redirect the victim's \
         target -- if this fails, the single announced target for mode 1 \
         landed at ctx.targets[0] rather than ctx.targets[1] (or the mode-1 \
         effect's DeclaredTarget{{index: 1}} otherwise failed to resolve), and \
         mode 1 is still broken after PB-DX25b's id-space repair (plan §8 R5). \
         Events observed: {:?}",
        resolve_events
    );
    let victim_after = state
        .stack_objects()
        .iter()
        .find(|so| so.id == victim_entry_id)
        .expect("victim stack entry must still exist after Untimely Malfunction resolves");
    assert_eq!(
        victim_after.targets[0].target,
        Target::Player(p1),
        "the victim's target must actually be redirected to p1 (Untimely \
         Malfunction's caster)"
    );
}
