//! PB-DX52 (`OOS-DX25b-1` headline; rider `OOS-DX25b-5`): probes for the new
//! `Target::StackObject(ObjectId)` id space.
//!
//! An activated or triggered ability's stack entry (CR 113.1c: "An ability can be an
//! activated or triggered ability on the stack. This kind of ability is an object.")
//! is minted by `state.next_object_id()` and pushed into `state.stack_objects` alone --
//! it owns no card, so it is **never** added to `state.objects`
//! (`state::stack_registry::card_in_stack_zone` returns `None` for every ability/
//! trigger kind, CR 113.1c vs. CR 601.2a). Before this batch there was therefore no id
//! space in which a player could NAME one: the offer layer
//! (`queries::legal_targets_per_slot`) could not enumerate it and
//! `casting::validate_object_satisfies_requirement`'s opening
//! `state.objects.get(&id)?` could never find it. Bolt Bend's printed "Change the
//! target of target spell **or ability** with a single target" (CR 115.7a) was
//! therefore dead on its "or ability" half, and `TargetSpellOrAbilityWithSingleTarget`
//! was behaviourally identical to the spell-only `TargetSpellWithSingleTarget` on
//! every production path (`OOS-DX25b-1`).
//!
//! **The two id spaces, and why a card id and a stack-entry id are never
//! interchangeable.** A SPELL's card moves into `ZoneId::Stack` with a fresh
//! `ObjectId` (CR 400.7/601.2a) -- that CARD id is a `state.objects` key and is what
//! `Target::Object` names (CR 601.2c: the offer layer enumerates it, the player
//! announces it). An ABILITY's stack entry has no card and is named by its own
//! `StackObject::id` -- `Target::StackObject` carries that id. Both numbers are minted
//! from the one monotone `state.next_object_id()` counter, so in a real game an id
//! lives in exactly one of the two spaces -- but nothing in the TYPE SYSTEM forces
//! that; `t9` below constructs a deliberate numeric collision to prove the two `Target`
//! variants are resolved through two different maps (`state.objects` vs
//! `state.stack_objects`) rather than through a shared numeric comparison a colliding
//! id could accidentally satisfy.
//!
//! **Stated coverage gap (PB-DX52 `/review`, LOW 10): `Effect::CounterSpell` with a declared
//! `Target::StackObject` has no probe here.** `t6` exercises
//! `resolution::counter_stack_object` DIRECTLY, which is the second, non-production counter
//! path. The `Effect::CounterSpell` path with a stack-entry target was traced rather than
//! driven (`effects/mod.rs` -> `stack_index_for_announced_target`'s first clause ->
//! `card_in_stack_zone` returns `None` for an ability -> no phantom graveyard move) and is
//! correct; it is untested because **no corpus def can reach it today** -- the only card that
//! would is `siren_stormtamer`, and it is blocked on a filtered requirement that does not
//! exist (`OOS-DX52-3`). Recorded so the absence is a known bound rather than an oversight.
//!
//! **Verdicts are asserted by RESOLUTION EFFECT wherever the fixture allows it, not by
//! the offer or the announcement alone** -- `t1` (the headline) and `t3` (the triggered
//! half) both carry the redirect through to a destroyed-or-surviving creature, not just
//! a `TargetsChanged` event. Where that is not possible today (`t7`, Deflecting Swat)
//! the docstring says so explicitly rather than implying more than is proven.
use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stack::{StackObject, StackObjectKind};
use mtg_engine::state::test_util;
use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    process_command, CardEffectTarget, CardRegistry, CardType, Command, Effect, GameEvent,
    GameState, GameStateBuilder, GameStateError, KeywordAbility, ManaPool, ObjectId, ObjectSpec,
    PlayerId, SpellTarget, Step, Target, TargetRequirement, ZoneId,
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

fn activate(
    state: GameState,
    player: PlayerId,
    source: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index: 0,
            targets,
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
}

/// Pass priority once per listed player, in order. If nobody acts in between, this
/// resolves the top of the stack once all have passed in succession (CR 117.4).
/// Mirrors `pb_dx25b_announced_stack_target_space.rs`'s `pass_n`.
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

/// `{T}: Destroy target creature.` -- one mandatory `TargetCreature` slot. The
/// reusable single-target ability body for t1/t2/t5/t6/t8.
fn destroy_one_creature_ability() -> ActivatedAbility {
    ActivatedAbility {
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
    }
}

/// `{T}: Destroy two target creatures.` -- TWO mandatory `TargetCreature` slots, used
/// by `t4` (CR 115.7a's "with a single target" enforcement) and `t7`
/// (`TargetSpellOrAbility`'s any-count acceptance).
fn destroy_two_creatures_ability() -> ActivatedAbility {
    ActivatedAbility {
        targets: vec![
            TargetRequirement::TargetCreature,
            TargetRequirement::TargetCreature,
        ],
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
        description: "{T}: Destroy two target creatures".to_string(),
        effect: Some(Effect::DestroyPermanent {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: false,
        }),
        sorcery_speed: false,
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: None,
    }
}

// ── T1: the headline, end to end ────────────────────────────────────────────

/// CR 115.7a / CR 601.2c / CR 113.1c -- p2 activates `{T}: Destroy target creature`
/// targeting p1's creature; p1 casts Bolt Bend naming the ABILITY's own stack-entry id
/// (`Target::StackObject`), redirecting it onto p2's own creature. Asserted at every
/// stage: (a) SR-38 offer/validate agreement, (b) the cast is accepted, (c)
/// `GameEvent::TargetsChanged` names the ability's ENTRY id, and (d) **the verdict**
/// -- after everything resolves, the ability's `DestroyPermanent` effect landed on the
/// NEW target and the original survives. (d) is observed game state (which creature
/// is on the battlefield), not the event stream alone.
#[test]
fn t1_headline_ability_redirect_lands_on_the_new_target() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    let ability_source = ObjectSpec::artifact(p2, "T1 Ability Source")
        .with_activated_ability(destroy_one_creature_ability());

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
        .object(ObjectSpec::creature(p1, "T1 Original Victim", 2, 2))
        .object(ObjectSpec::creature(p2, "T1 Alternative Creature", 3, 3))
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

    let source_id = find_obj(&state, "T1 Ability Source");
    let original_id = find_obj(&state, "T1 Original Victim");
    let alt_id = find_obj(&state, "T1 Alternative Creature");
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");

    // p2 activates the ability targeting p1's creature.
    let (state, _activate_events) =
        activate(state, p2, source_id, vec![Target::Object(original_id)])
            .unwrap_or_else(|e| panic!("p2's ability activation must succeed: {:?}", e));
    assert_eq!(
        state.stack_objects().len(),
        1,
        "only the ability should be on the stack"
    );
    let ability_entry_id = state.stack_objects().back().unwrap().id;

    // (a) SR-38: the offer layer must list the ability's stack entry, and only it, for
    // Bolt Bend's TargetSpellOrAbilityWithSingleTarget slot.
    let candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p1,
        bolt_bend_hand_id,
        &[TargetRequirement::TargetSpellOrAbilityWithSingleTarget],
    );
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0],
        vec![Target::StackObject(ability_entry_id)],
        "OOS-DX25b-1 CLOSED: the offer layer must enumerate exactly the ability's own \
         stack-entry id -- nothing else on this board satisfies \
         TargetSpellOrAbilityWithSingleTarget -- got: {:?}",
        candidates[0]
    );

    // (b) the cast is accepted.
    let (state, cast_events) = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::StackObject(ability_entry_id)],
    )
    .unwrap_or_else(|e| {
        panic!(
            "p1's Bolt Bend cast targeting the ability's stack entry must succeed \
             (the offer layer just said this was legal): {:?}",
            e
        )
    });
    assert!(
        cast_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)),
        "SpellCast event expected for Bolt Bend"
    );
    assert_eq!(
        state.stack_objects().len(),
        2,
        "Bolt Bend + the ability, both on the stack"
    );

    // Resolve Bolt Bend first (LIFO). must_change: true (CR 115.7a) forces the redirect
    // onto the only OTHER legal TargetCreature candidate (the original is excluded --
    // "changed only to ANOTHER legal target").
    let (state, resolve_events) = pass_n(state, &[p1, p2]);
    let targets_changed = resolve_events.iter().find_map(|e| match e {
        GameEvent::TargetsChanged {
            stack_object_id,
            new_targets,
            ..
        } => Some((*stack_object_id, new_targets.clone())),
        _ => None,
    });
    // (c) TargetsChanged names the ability's OWN entry id.
    let (changed_id, new_targets) = targets_changed.unwrap_or_else(|| {
        panic!(
            "TargetsChanged must fire when Bolt Bend resolves: {:?}",
            resolve_events
        )
    });
    assert_eq!(
        changed_id, ability_entry_id,
        "TargetsChanged must name the ability's stack-ENTRY id, not Bolt Bend's own"
    );
    assert_eq!(
        new_targets.len(),
        1,
        "the redirected target set must still have exactly one target"
    );
    assert_eq!(
        new_targets[0].target,
        Target::Object(alt_id),
        "the redirect must land on the alternative creature -- the only other legal \
         TargetCreature candidate"
    );

    // Resolve the (redirected) ability.
    let (state, _resolve_events_2) = pass_n(state, &[p1, p2]);

    // (d) THE VERDICT: observed game state, not the event stream.
    assert!(
        state.objects().values().any(
            |o| o.characteristics.name == "T1 Original Victim" && o.zone == ZoneId::Battlefield
        ),
        "the ORIGINAL target must SURVIVE -- Bolt Bend redirected the ability away \
         from it before it resolved"
    );
    assert!(
        !state.objects().values().any(|o| {
            o.characteristics.name == "T1 Alternative Creature" && o.zone == ZoneId::Battlefield
        }),
        "the ALTERNATIVE creature (the ability's NEW target) must be destroyed -- \
         this is the resolution-EFFECT verdict, not just the announcement"
    );
}

// ── T2: the distinctness probe (AC 7348) ────────────────────────────────────

/// CR 115.7a/115.7b -- on ONE fixture holding ONE activated-ability stack entry,
/// `TargetSpellOrAbilityWithSingleTarget` ACCEPTS it and the spell-only
/// `TargetSpellWithSingleTarget` REFUSES it. Before PB-DX52 these two requirements
/// were behaviourally IDENTICAL on every production path (`OOS-DX25b-1`): the ability
/// half of `TargetSpellOrAbilityWithSingleTarget` was unreachable, so nothing could
/// ever exercise the one clause (spell-only) that distinguishes it from
/// `TargetSpellWithSingleTarget`. This is the first assertion in the tree that can
/// tell them apart -- both from the offer layer directly, and via two REAL casts
/// (Bolt Bend accepts, Misdirection refuses) driven from independent clones of the
/// same fixture.
#[test]
fn t2_distinctness_spell_or_ability_vs_spell_only() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![bolt_bend.clone(), misdirection.clone()]);

    let ability_source = ObjectSpec::artifact(p2, "T2 Ability Source")
        .with_activated_ability(destroy_one_creature_ability());

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                red: 1,
                blue: 2,
                ..Default::default()
            },
        )
        .object(ability_source)
        .object(ObjectSpec::creature(p1, "T2 Victim Creature", 2, 2))
        .object(
            ObjectSpec::card(p1, "Bolt Bend")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(bolt_bend.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "T2 Ability Source");
    let victim_id = find_obj(&state, "T2 Victim Creature");
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");
    let misdirection_hand_id = find_obj(&state, "Misdirection");

    let (state, _) = activate(state, p2, source_id, vec![Target::Object(victim_id)])
        .unwrap_or_else(|e| panic!("p2's ability activation must succeed: {:?}", e));
    let ability_entry_id = state.stack_objects().back().unwrap().id;

    // (a) offer-layer distinctness: one call, two requirement lists, same state.
    let candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p1,
        bolt_bend_hand_id,
        &[
            TargetRequirement::TargetSpellOrAbilityWithSingleTarget,
            TargetRequirement::TargetSpellWithSingleTarget,
        ],
    );
    assert_eq!(candidates.len(), 2);
    assert!(
        candidates[0].contains(&Target::StackObject(ability_entry_id)),
        "TargetSpellOrAbilityWithSingleTarget must accept the ability entry \
         (CR 115.7a) -- candidates: {:?}",
        candidates[0]
    );
    assert!(
        !candidates[1].contains(&Target::StackObject(ability_entry_id)),
        "TargetSpellWithSingleTarget must REFUSE the ability entry -- it is \
         spell-only (CR 115.7a/115.7b) -- candidates: {:?}",
        candidates[1]
    );

    // (b) two REAL casts from two independent clones of the SAME fixture.
    let state_for_bolt_bend = state.clone();
    let (_after_bolt_bend, bolt_bend_events) = cast(
        state_for_bolt_bend,
        p1,
        bolt_bend_hand_id,
        vec![Target::StackObject(ability_entry_id)],
    )
    .unwrap_or_else(|e| {
        panic!(
            "Bolt Bend (TargetSpellOrAbilityWithSingleTarget) targeting the ability \
             entry must succeed: {:?}",
            e
        )
    });
    assert!(bolt_bend_events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)));

    let state_for_misdirection = state;
    let result = cast(
        state_for_misdirection,
        p1,
        misdirection_hand_id,
        vec![Target::StackObject(ability_entry_id)],
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "Misdirection (TargetSpellWithSingleTarget, spell-only) targeting the SAME \
         ability entry must FAIL -- got: {:?}",
        result.map(|_| ())
    );
}

// ── T3: a TRIGGERED ability's entry is targetable too ───────────────────────

/// CR 115.7a / CR 113.1c -- the seed and the v4 queue row both say "activated OR
/// triggered"; this file must not measure only the activated half. Synthetic
/// (matching `pb_dx25b_announced_stack_target_space.rs`'s T5/T6/T8 convention): a
/// `StackObjectKind::TriggeredAbility` entry is pushed directly, bypassing the
/// CR 603.2/603.3 trigger-collection pipeline, because that pipeline is not this
/// file's subject -- `card_in_stack_zone`'s `TriggeredAbility { .. } => None` arm
/// treats it identically to `ActivatedAbility { .. } => None`, so a hand-built entry
/// of this kind exercises the same offer/validate/retarget code path a real "whenever
/// ~ deals damage, destroy target creature"-style trigger would.
///
/// Proven through to the entry's OWN `.targets` field actually changing (not just an
/// event) and through an explicit "did not fizzle" check. Full resolution-EFFECT
/// verification (a creature actually destroyed) is deliberately NOT attempted here --
/// the synthetic entry carries `embedded_effect: None` and no card-registry ability at
/// its index, so resolving it further is outside this probe's scope. `t1` already
/// proves the resolution-effect half end to end for the ACTIVATED case, through the
/// IDENTICAL `Effect::ChangeTargets` code path (`stack_index_for_announced_target`'s
/// `so.id == announced` clause does not distinguish `StackObjectKind` at all).
#[test]
fn t3_triggered_ability_entry_is_also_reachable() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    let mut state = GameStateBuilder::new()
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
        .object(ObjectSpec::creature(p1, "T3 Trigger Original Victim", 2, 2))
        .object(ObjectSpec::creature(p2, "T3 Trigger Alternative", 3, 3))
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

    let original_id = find_obj(&state, "T3 Trigger Original Victim");
    let alt_id = find_obj(&state, "T3 Trigger Alternative");
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");

    let entry_id = test_util::next_object_id(&mut state);
    let mut trigger_entry = StackObject::trigger_default(
        entry_id,
        p2,
        StackObjectKind::TriggeredAbility {
            source_object: original_id,
            ability_index: 0,
            is_carddef_etb: false,
            embedded_effect: None,
        },
    );
    trigger_entry.targets = vec![SpellTarget {
        target: Target::Object(original_id),
        zone_at_cast: Some(ZoneId::Battlefield),
    }];
    trigger_entry.target_requirements = vec![TargetRequirement::TargetCreature];
    state.stack_objects_mut().push_back(trigger_entry);
    assert_eq!(state.stack_objects().len(), 1);

    // SR-38: the offer layer must enumerate a TRIGGERED ability's entry exactly like
    // an activated ability's.
    let candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p1,
        bolt_bend_hand_id,
        &[TargetRequirement::TargetSpellOrAbilityWithSingleTarget],
    );
    assert!(
        candidates[0].contains(&Target::StackObject(entry_id)),
        "a TRIGGERED ability's entry must be offered -- candidates: {:?}",
        candidates[0]
    );

    let (state, cast_events) = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::StackObject(entry_id)],
    )
    .unwrap_or_else(|e| {
        panic!(
            "Bolt Bend targeting a triggered ability's entry must succeed: {:?}",
            e
        )
    });
    assert!(cast_events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)));

    let (state, resolve_events) = pass_n(state, &[p1, p2]);
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "Bolt Bend must not fizzle -- the triggered ability's entry is still live: {:?}",
        resolve_events
    );
    let targets_changed = resolve_events.iter().find_map(|e| match e {
        GameEvent::TargetsChanged {
            stack_object_id,
            new_targets,
            ..
        } => Some((*stack_object_id, new_targets.clone())),
        _ => None,
    });
    let (changed_id, new_targets) = targets_changed.unwrap_or_else(|| {
        panic!(
            "TargetsChanged must fire -- the triggered ability's entry is a legal, \
             single-target stack object: {:?}",
            resolve_events
        )
    });
    assert_eq!(changed_id, entry_id);
    assert_eq!(new_targets[0].target, Target::Object(alt_id));

    // The entry's own `.targets` field, re-read from the state (not the event), must
    // agree.
    let entry_after = state
        .stack_objects()
        .iter()
        .find(|so| so.id == entry_id)
        .expect("the triggered ability's entry must still be on the stack");
    assert_eq!(entry_after.targets[0].target, Target::Object(alt_id));
}

// ── T4: CR 115.7a "with a single target" is enforced ────────────────────────

/// CR 115.7a -- an ability entry with TWO declared targets is refused by
/// `TargetSpellOrAbilityWithSingleTarget` and is NOT offered by the offer layer.
#[test]
fn t4_single_target_clause_refuses_a_two_target_entry() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    let ability_source = ObjectSpec::artifact(p2, "T4 Ability Source")
        .with_activated_ability(destroy_two_creatures_ability());

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
        .object(ObjectSpec::creature(p1, "T4 Target One", 2, 2))
        .object(ObjectSpec::creature(p1, "T4 Target Two", 2, 2))
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

    let source_id = find_obj(&state, "T4 Ability Source");
    let t1_id = find_obj(&state, "T4 Target One");
    let t2_id = find_obj(&state, "T4 Target Two");
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");

    let (state, _) = activate(
        state,
        p2,
        source_id,
        vec![Target::Object(t1_id), Target::Object(t2_id)],
    )
    .unwrap_or_else(|e| panic!("p2's TWO-target ability activation must succeed: {:?}", e));
    let ability_entry_id = state.stack_objects().back().unwrap().id;
    assert_eq!(
        state.stack_objects().back().unwrap().targets.len(),
        2,
        "non-vacuity anchor: the ability really does carry two declared targets"
    );

    let candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p1,
        bolt_bend_hand_id,
        &[TargetRequirement::TargetSpellOrAbilityWithSingleTarget],
    );
    assert!(
        !candidates[0].contains(&Target::StackObject(ability_entry_id)),
        "CR 115.7a: a two-target ability entry must NOT be offered for 'target spell \
         or ability WITH A SINGLE TARGET' -- candidates: {:?}",
        candidates[0]
    );

    let result = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::StackObject(ability_entry_id)],
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "Bolt Bend must refuse a two-target ability entry -- got: {:?}",
        result.map(|_| ())
    );
}

// ── T5/T6: CR 608.2b, both directions ────────────────────────────────────────

/// Shared setup for T5/T6: p2 activates `{T}: Destroy target creature` against p1's
/// creature (with a SECOND creature present as the only other legal TargetCreature
/// candidate, so `t5`'s redirect has somewhere to land -- CR 115.7a's fallback, "if a
/// target can't be changed to another legal target, the original target is unchanged,"
/// would otherwise make "no fizzle" and "no redirect for lack of an alternative"
/// indistinguishable); p1 casts Bolt Bend targeting the ability's entry. Returns the
/// state with both stack entries in place, plus the ability's entry id.
fn cr_608_2b_fixture(name_prefix: &str) -> (GameState, ObjectId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    let ability_source = ObjectSpec::artifact(p2, &format!("{name_prefix} Ability Source"))
        .with_activated_ability(destroy_one_creature_ability());

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
        .object(ObjectSpec::creature(
            p1,
            &format!("{name_prefix} Original"),
            2,
            2,
        ))
        .object(ObjectSpec::creature(
            p2,
            &format!("{name_prefix} Alternative"),
            3,
            3,
        ))
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

    let source_id = find_obj(&state, &format!("{name_prefix} Ability Source"));
    let original_id = find_obj(&state, &format!("{name_prefix} Original"));
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");

    let (state, _) = activate(state, p2, source_id, vec![Target::Object(original_id)])
        .unwrap_or_else(|e| panic!("activation must succeed: {:?}", e));
    let ability_entry_id = state.stack_objects().back().unwrap().id;

    let (state, _) = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::StackObject(ability_entry_id)],
    )
    .unwrap_or_else(|e| panic!("Bolt Bend cast must succeed: {:?}", e));
    assert_eq!(state.stack_objects().len(), 2, "ability + Bolt Bend");

    (state, ability_entry_id, original_id)
}

/// CR 608.2b, wrong-way-round: an ability entry that is STILL on the stack IS a legal
/// target at resolution (`resolution::is_target_legal`'s `Target::StackObject` arm
/// answers by EXISTENCE, since a stack entry has no zone to check -- `zone_at_cast` is
/// `None` for this variant, per `Target::StackObject`'s own doc). Asserted as "does
/// NOT fizzle" plus "actually redirects" -- the two are not the same claim: a legal
/// target with no OTHER candidate would also not fizzle (CR 115.7a's own fallback),
/// which is why `t6`'s sibling fixture and `t1` both provide an alternative.
#[test]
fn t5_live_entry_is_a_legal_target_no_fizzle() {
    let p1 = p(1);
    let p2 = p(2);
    let (state, ability_entry_id, _original_id) = cr_608_2b_fixture("T5");
    let alt_id = find_obj(&state, "T5 Alternative");

    let stack_len_before_resolution = state.stack_objects().len();
    let (state, resolve_events) = pass_n(state, &[p1, p2]);
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "Bolt Bend must not fizzle -- its target (the ability's live entry) is legal: {:?}",
        resolve_events
    );
    let targets_changed = resolve_events.iter().find_map(|e| match e {
        GameEvent::TargetsChanged {
            stack_object_id,
            new_targets,
            ..
        } => Some((*stack_object_id, new_targets.clone())),
        _ => None,
    });
    let (changed_id, new_targets) = targets_changed.unwrap_or_else(|| {
        panic!(
            "non-vacuity anchor: Bolt Bend must actually redirect, not merely fail \
             to fizzle -- resolve_events: {:?}",
            resolve_events
        )
    });
    assert_eq!(changed_id, ability_entry_id);
    assert_eq!(
        new_targets[0].target,
        Target::Object(alt_id),
        "the redirect must land on the only other legal TargetCreature candidate"
    );
    assert_eq!(
        state.stack_objects().len(),
        stack_len_before_resolution - 1,
        "Bolt Bend itself left the stack when it resolved"
    );
}

/// CR 608.2b, right-way-round: an ability entry that is COUNTERED before Bolt Bend
/// resolves is gone from `state.stack_objects` and is therefore an ILLEGAL target --
/// Bolt Bend's sole target is now illegal, so the whole spell fizzles (CR 608.2b: "If
/// all its targets ... are now illegal, the spell or ability doesn't resolve. It's
/// removed from the stack and ... put into its owner's graveyard.").
///
/// Removal is via `resolution::counter_stack_object` -- the engine's second,
/// independent counter path, already exercised on an `ActivatedAbility` kind by
/// `pb_dx25_counterspell_stack_shapes::T7` (its own doc: "a `pub` function with zero
/// production callers", used by tests precisely to isolate CR questions like this one
/// from stack-ordering mechanics). This removes the ability's entry WHILE Bolt Bend
/// still sits on top of it, without resolving Bolt Bend first -- simulating CR 608.2n's
/// "it resolves, or it otherwise leaves the stack" for the ability alone.
#[test]
fn t6_countered_entry_is_illegal_bolt_bend_fizzles() {
    let p1 = p(1);
    let p2 = p(2);
    let (mut state, ability_entry_id, _original_id) = cr_608_2b_fixture("T6");

    let counter_events =
        mtg_engine::rules::resolution::counter_stack_object(&mut state, ability_entry_id)
            .unwrap_or_else(|e| {
                panic!("counter_stack_object on the ability must succeed: {:?}", e)
            });
    assert!(
        !counter_events.is_empty()
            || !state
                .stack_objects()
                .iter()
                .any(|so| so.id == ability_entry_id),
        "counter_stack_object must actually remove the ability's entry"
    );
    assert!(
        !state
            .stack_objects()
            .iter()
            .any(|so| so.id == ability_entry_id),
        "the ability's entry must be gone from state.stack_objects"
    );
    assert_eq!(state.stack_objects().len(), 1, "only Bolt Bend remains");

    let (state, resolve_events) = pass_n(state, &[p1, p2]);
    let fizzled = resolve_events.iter().find_map(|e| match e {
        GameEvent::SpellFizzled {
            stack_object_id, ..
        } => Some(*stack_object_id),
        _ => None,
    });
    assert!(
        fizzled.is_some(),
        "Bolt Bend must fizzle -- its target (the removed ability entry) no longer \
         exists on the stack: {:?}",
        resolve_events
    );
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "a fizzled Bolt Bend must not emit TargetsChanged"
    );
    assert!(
        state.objects().values().any(
            |o| o.characteristics.name == "Bolt Bend" && matches!(o.zone, ZoneId::Graveyard(_))
        ),
        "Bolt Bend's card must be in a graveyard after fizzling"
    );
}

// ── T7: TargetSpellOrAbility (Deflecting Swat) -- any target count ─────────

/// CR 115.1a / CR 115.7d -- `TargetSpellOrAbility` accepts an ability entry with ANY
/// declared-target count, where `TargetSpellOrAbilityWithSingleTarget` would refuse
/// the same TWO-target entry. Deflecting Swat's printed line has no "with a single
/// target" clause, which is exactly why PB-DX52 added this variant rather than
/// widening Bolt Bend's.
///
/// **Deflecting Swat's `must_change: false` makes its RESOLUTION a deterministic
/// no-op today** (`OOS-DX25b-4`, open, deferred to PB-DX54 -- CR 115.7d's "you MAY
/// choose new targets" is a player decision needing an `EffectChoiceQuestion` variant
/// this engine does not yet have). This probe therefore asserts the ANNOUNCEMENT half
/// only -- that the entry is legally targetable and the cast is accepted with the
/// entry recorded as Deflecting Swat's own declared target -- and says so rather than
/// implying the redirect fires.
#[test]
fn t7_target_spell_or_ability_accepts_any_target_count() {
    let p1 = p(1);
    let p2 = p(2);

    let deflecting_swat = mtg_engine::cards::defs::deflecting_swat::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![deflecting_swat.clone()]);

    let ability_source = ObjectSpec::artifact(p2, "T7 Ability Source")
        .with_activated_ability(destroy_two_creatures_ability());

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 2,
                red: 1,
                ..Default::default()
            },
        )
        .object(ability_source)
        .object(ObjectSpec::creature(p1, "T7 Target One", 2, 2))
        .object(ObjectSpec::creature(p1, "T7 Target Two", 2, 2))
        .object(
            ObjectSpec::card(p1, "Deflecting Swat")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(deflecting_swat.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source_id = find_obj(&state, "T7 Ability Source");
    let t1_id = find_obj(&state, "T7 Target One");
    let t2_id = find_obj(&state, "T7 Target Two");
    let swat_hand_id = find_obj(&state, "Deflecting Swat");

    let (state, _) = activate(
        state,
        p2,
        source_id,
        vec![Target::Object(t1_id), Target::Object(t2_id)],
    )
    .unwrap_or_else(|e| panic!("2-target activation must succeed: {:?}", e));
    let ability_entry_id = state.stack_objects().back().unwrap().id;
    assert_eq!(
        state.stack_objects().back().unwrap().targets.len(),
        2,
        "non-vacuity anchor: the ability really carries two declared targets"
    );

    let candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p1,
        swat_hand_id,
        &[
            TargetRequirement::TargetSpellOrAbility,
            TargetRequirement::TargetSpellOrAbilityWithSingleTarget,
        ],
    );
    assert!(
        candidates[0].contains(&Target::StackObject(ability_entry_id)),
        "TargetSpellOrAbility (CR 115.1a/115.7d) must accept a TWO-target ability \
         entry -- it asserts nothing about target count -- candidates: {:?}",
        candidates[0]
    );
    assert!(
        !candidates[1].contains(&Target::StackObject(ability_entry_id)),
        "TargetSpellOrAbilityWithSingleTarget must REFUSE the SAME entry -- it is \
         CR 115.7a's single-target clause -- candidates: {:?}",
        candidates[1]
    );

    let (state, cast_events) = cast(
        state,
        p1,
        swat_hand_id,
        vec![Target::StackObject(ability_entry_id)],
    )
    .unwrap_or_else(|e| {
        panic!(
            "Deflecting Swat targeting the 2-target ability entry must succeed: {:?}",
            e
        )
    });
    assert!(cast_events
        .iter()
        .any(|e| matches!(e, GameEvent::SpellCast { player, .. } if *player == p1)));

    let swat_entry = state
        .stack_objects()
        .iter()
        .find(|so| so.id != ability_entry_id)
        .expect("Deflecting Swat's own entry must be on the stack");
    assert_eq!(
        swat_entry.targets[0].target,
        Target::StackObject(ability_entry_id),
        "Deflecting Swat's own declared target must be the ability's entry id"
    );
}

// ── T8: no Ward dispatch for a stack-entry target ────────────────────────────

/// CR 702.21a ("Whenever this PERMANENT becomes the target...") / CR 110.1 ("A
/// permanent is a card or token on the battlefield.") -- an ability on the stack is
/// not a permanent, so a Bolt Bend cast whose SOLE declared target is
/// `Target::StackObject` owes ZERO `GameEvent::PermanentTargeted` and pushes no Ward
/// trigger onto the stack.
///
/// **COUNT, not `>= 0`** (PB-DX36 HIGH 1's lesson, `OOS-DX36-8`): this drives a cast
/// whose sole declared target is a stack entry. It would NOT catch a defect where a
/// Bolt Bend cast that ALSO named a battlefield permanent wrongly suppressed THAT
/// target's `PermanentTargeted` -- this probe is the NEGATIVE case only. The positive
/// case (a real object target correctly dispatching Ward) is already pinned by
/// `pb_dx25b_announced_stack_target_space::t7_ward_still_finds_its_target`.
#[test]
fn t8_no_ward_dispatch_for_a_stack_entry_target() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    let ability_source = ObjectSpec::artifact(p2, "T8 Ability Source")
        .with_activated_ability(destroy_one_creature_ability());
    let ward_creature =
        ObjectSpec::creature(p2, "T8 Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));

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
        .object(ward_creature)
        .object(ObjectSpec::creature(p1, "T8 Non Ward Victim", 2, 2))
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

    let source_id = find_obj(&state, "T8 Ability Source");
    let victim_id = find_obj(&state, "T8 Non Ward Victim");
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");

    let (state, _) = activate(state, p2, source_id, vec![Target::Object(victim_id)])
        .unwrap_or_else(|e| panic!("activation must succeed: {:?}", e));
    let ability_entry_id = state.stack_objects().back().unwrap().id;
    let stack_len_before = state.stack_objects().len();

    let (state, cast_events) = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::StackObject(ability_entry_id)],
    )
    .unwrap_or_else(|e| panic!("Bolt Bend cast must succeed: {:?}", e));

    let permanent_targeted_count = cast_events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentTargeted { .. }))
        .count();
    assert_eq!(
        permanent_targeted_count, 0,
        "CR 702.21a / CR 110.1: a stack-entry target owes NO PermanentTargeted event \
         -- got: {:?}",
        cast_events
    );
    assert_eq!(
        state.stack_objects().len(),
        stack_len_before + 1,
        "only Bolt Bend itself should be pushed -- no Ward trigger entry"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "T8 Ward Creature" && o.zone == ZoneId::Battlefield),
        "non-vacuity anchor: the Ward creature is actually on the board (present, \
         untouched) -- this is not a trivially-empty-board pass"
    );
}

// ── T9: the two id spaces do not collapse ────────────────────────────────────

/// `Target`'s derived `PartialEq` compares the variant discriminant first (pinned
/// directly below), so `Target::Object(n)` and `Target::StackObject(n)` are never
/// equal for the same `n` -- but the DURABLE claim this test makes is behavioural, not
/// just structural: even when a `state.objects` id and a `state.stack_objects` entry
/// id are numerically IDENTICAL, `casting.rs`'s dispatch on the `Target` variant
/// (`Target::Object(id) => validate_object_satisfies_requirement(..)` vs
/// `Target::StackObject(id) => validate_stack_object_satisfies_requirement(..)`)
/// looks each one up in a DIFFERENT map and can never accidentally satisfy the
/// other's requirement.
///
/// Production ids never collide -- both spaces are minted from the one monotone
/// `state.next_object_id()` counter, so an id lives in exactly one of them (CR 400.7:
/// new objects always get fresh ids). This test deliberately constructs the collision
/// anyway (labeled synthetic, matching `pb_dx25b_announced_stack_target_space.rs`'s
/// T5/T6/T8/T9 convention) because it is the only way to prove the dispatch is keyed
/// on the VARIANT and the MAP, not on a shared numeric comparison a colliding id could
/// accidentally satisfy.
#[test]
fn t9_object_and_stack_object_ids_do_not_collapse() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "T9 Creature", 2, 2))
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

    let creature_id = find_obj(&state, "T9 Creature");
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");

    // Baseline: never equal on the same numeric id.
    assert_ne!(
        Target::Object(creature_id),
        Target::StackObject(creature_id),
        "the two variants must never compare equal, even on the same numeric id"
    );

    // Deliberately colliding, synthetic (see module doc): a stack entry whose `id`
    // equals the REAL creature's id.
    let mut colliding_entry = StackObject::trigger_default(
        creature_id,
        p2,
        StackObjectKind::ActivatedAbility {
            source_object: creature_id,
            ability_index: 0,
            embedded_effect: None,
        },
    );
    colliding_entry.targets = vec![SpellTarget {
        target: Target::Player(p2),
        zone_at_cast: None,
    }];
    colliding_entry.target_requirements = vec![TargetRequirement::TargetPlayer];
    state.stack_objects_mut().push_back(colliding_entry);

    // Direction 1: TargetCreature. The REAL creature (Target::Object) satisfies it;
    // the colliding entry (Target::StackObject, SAME number) does not --
    // `validate_stack_object_satisfies_requirement` has no arm for TargetCreature at
    // all (a stack entry has no battlefield presence to check).
    let creature_candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p1,
        bolt_bend_hand_id,
        &[TargetRequirement::TargetCreature],
    );
    assert!(
        creature_candidates[0].contains(&Target::Object(creature_id)),
        "the real creature must satisfy TargetCreature"
    );
    assert!(
        !creature_candidates[0].contains(&Target::StackObject(creature_id)),
        "the colliding StackObject entry must NOT satisfy TargetCreature, even though \
         its id is numerically identical to a real creature's -- got: {:?}",
        creature_candidates[0]
    );

    // Direction 2 (the reverse): TargetSpellOrAbilityWithSingleTarget. The colliding
    // entry (one declared target) satisfies it; the real creature (Battlefield, not
    // Stack) does not -- `validate_object_satisfies_requirement` requires
    // `obj.zone == ZoneId::Stack` for this requirement.
    let ability_candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p1,
        bolt_bend_hand_id,
        &[TargetRequirement::TargetSpellOrAbilityWithSingleTarget],
    );
    assert!(
        ability_candidates[0].contains(&Target::StackObject(creature_id)),
        "the colliding entry must satisfy TargetSpellOrAbilityWithSingleTarget (one \
         declared target) -- got: {:?}",
        ability_candidates[0]
    );
    assert!(
        !ability_candidates[0].contains(&Target::Object(creature_id)),
        "the real creature (zone == Battlefield) must NOT satisfy \
         TargetSpellOrAbilityWithSingleTarget, even sharing the colliding entry's \
         numeric id -- got: {:?}",
        ability_candidates[0]
    );
}

// ── T10: CR 702.16b protection survives an ability-shaped redirect ───────────

/// CR 702.16b / CR 113.7 / CR 115.7a (`OOS-DX25c-3`) -- **the probe that makes this
/// batch's own near-miss a red test rather than a paragraph.**
///
/// PB-DX52 is what makes an ability a reachable `Effect::ChangeTargets` victim, and
/// `rules::retarget::plan_target_change` derives the victim's `source_chars` -- the
/// characteristics CR 702.16b's protection check reads -- from a `stack_registry` helper.
/// Until this batch that helper was `card_in_stack_zone`, which returns `None` for every
/// ability kind because an ability on the stack owns no card. With `None`,
/// `validate_target_protection` has no source to compare a protection quality against and
/// **every protection check silently passes**, so Bolt Bend could have redirected a RED
/// ability onto a creature with protection from red. PB-DX52 changes that read to
/// `stack_registry::source_of` (CR 113.7: *"The source of an ability is the object that
/// generated it"*), which for an ability is its source permanent.
///
/// **Why this test exists at all**: the batch's revert matrix row R6 (put
/// `card_in_stack_zone` back) reddened exactly ONE thing -- `pb_dx52_stack_target_roster::
/// r7b`, a SOURCE gate that reads the call site's text. No behavioural probe moved. A
/// source gate proves the line is spelled a certain way; it cannot prove the line does
/// anything, and a later batch that "simplifies" the helper while keeping the name would
/// satisfy it. This probe closes that: it is RED under R6 on an observable outcome.
///
/// Fixture: p2's **red** artifact holds `{T}: Destroy target creature` and points it at
/// p1's plain creature. p1 controls a second creature with **protection from red**. p1
/// casts Bolt Bend at the ability's stack entry. CR 115.7a demands *another legal target*;
/// the protected creature is NOT one (CR 702.16b: it *"can't be the target of ... spells
/// or abilities"* from a red source), and it is the only other creature on the board -- so
/// CR 115.7a's own fallback applies and the target is **unchanged**.
///
/// **The verdict is the resolution effect, and it is asserted in BOTH directions**: the
/// plain creature dies (the original target was kept) AND the protected creature survives
/// (the redirect did not land on it). Asserting only the second would pass on a fixture
/// where nothing resolved at all.
#[test]
fn t10_protection_from_red_refuses_an_ability_shaped_redirect() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_bend = mtg_engine::cards::defs::bolt_bend::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![bolt_bend.clone()]);

    // The ability's SOURCE is red. That is the whole point: CR 113.7 makes it the
    // ability's source, and CR 702.16b compares the protection quality against it.
    let ability_source = ObjectSpec::artifact(p2, "T10 Red Ability Source")
        .with_colors(vec![mtg_engine::Color::Red])
        .with_activated_ability(destroy_one_creature_ability());

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
        .object(ObjectSpec::creature(p1, "T10 Plain Victim", 2, 2))
        .object(
            ObjectSpec::creature(p1, "T10 Protected Creature", 3, 3).with_keyword(
                KeywordAbility::ProtectionFrom(mtg_engine::ProtectionQuality::FromColor(
                    mtg_engine::Color::Red,
                )),
            ),
        )
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

    let source_id = find_obj(&state, "T10 Red Ability Source");
    let plain_id = find_obj(&state, "T10 Plain Victim");
    let protected_id = find_obj(&state, "T10 Protected Creature");
    let bolt_bend_hand_id = find_obj(&state, "Bolt Bend");

    // NON-VACUITY FLOOR, asserted before the drive rather than assumed: the protected
    // creature must be the ONLY other creature on the board, or CR 115.7a could find a
    // third legal target and this probe would pass for a reason it does not name.
    let creature_count = state
        .objects()
        .iter()
        .filter(|(id, o)| {
            o.zone == ZoneId::Battlefield
                && mtg_engine::rules::layers::expect_characteristics(&state, **id)
                    .card_types
                    .contains(&CardType::Creature)
        })
        .count();
    assert_eq!(
        creature_count, 2,
        "non-vacuity: exactly two creatures must exist, so the protected one is the only \
         candidate CR 115.7a could redirect onto"
    );

    let (state, _) = activate(state, p2, source_id, vec![Target::Object(plain_id)])
        .unwrap_or_else(|e| panic!("p2's ability activation must succeed: {:?}", e));
    let ability_entry_id = state.stack_objects().back().unwrap().id;

    let (state, _) = cast(
        state,
        p1,
        bolt_bend_hand_id,
        vec![Target::StackObject(ability_entry_id)],
    )
    .unwrap_or_else(|e| panic!("Bolt Bend must be castable at the ability entry: {:?}", e));

    // Resolve everything.
    let (state, _) = pass_n(state, &[p1, p2]);
    let (state, _) = pass_n(state, &[p1, p2]);

    // THE VERDICT, both directions.
    let protected_alive = state
        .objects()
        .get(&protected_id)
        .is_some_and(|o| o.zone == ZoneId::Battlefield);
    let plain_alive = state
        .objects()
        .get(&plain_id)
        .is_some_and(|o| o.zone == ZoneId::Battlefield);
    assert!(
        protected_alive,
        "CR 702.16b: the ability's source is RED, so a creature with protection from red \
         is NOT another legal target and CR 115.7a's fallback must leave the original \
         target unchanged. If this fails, `plan_target_change` is deriving the victim's \
         source characteristics from a helper that returns `None` for an ability -- i.e. \
         `card_in_stack_zone` instead of `source_of` (`OOS-DX25c-3`)."
    );
    assert!(
        !plain_alive,
        "the original target must still have been destroyed -- otherwise this probe would \
         pass on a fixture where the ability never resolved at all, which would make the \
         protection assertion above vacuous"
    );
}
