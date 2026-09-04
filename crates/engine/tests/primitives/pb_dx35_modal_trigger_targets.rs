//! PB-DX35 Half A (`OOS-DX4-2`): a modal triggered ability's `ModeSelection.
//! mode_targets` is now honoured on the TRIGGER path, not only the casting path.
//!
//! Every probe here asserts by RESOLUTION EFFECT (life total, a token existing, a
//! counter landing), never by the offer -- following the PB-DX28 / PB-DX43 lesson
//! that existence-only assertions can pass vacuously. `t1`/`t2` drive the real
//! `retreat_to_kazandu()` def; `t3`/`t4` drive the real `shambling_ghast()` def;
//! `t5`-`t8` drive synthetic `WhenDies` modal triggers built to isolate one
//! `trigger_modal_plan` arm each. `t9` (site 1/2/D-vs-site-3 agreement) is an
//! internal `#[cfg(test)]` unit test inside `rules/abilities.rs` -- see that
//! module's own doc comment for why (site 3 is a bare private `fn`, unreachable
//! from here).
//!
//! `memory/primitives/pb-plan-DX35.md` §A4 is authoritative for this file's scope.

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardId, CardRegistry, CardType, Command, CounterType, Effect, EffectAmount,
    GameEvent, GameState, GameStateBuilder, ModeSelection, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, Step, TargetController, TargetFilter, TargetRequirement, TriggerCondition,
    ZoneId,
};
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == zone)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found in {zone:?}"))
}

fn any_object_named(state: &GameState, name_prefix: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name.starts_with(name_prefix))
}

/// Pass priority for every listed player once, applying the SBA + trigger-flush
/// cycle `handle_all_passed` runs when the stack empties and both players pass
/// (CR 704.3 / CR 603.3b) -- mirrors `mechanics_e_l::haunt::pass_all`.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Kill `creature_id` by marking lethal damage directly (CR 704.5g), then run
/// ONE `pass_all` round -- the SBA pass that removes it AND the trigger flush
/// that (attempts to) put its death trigger on the stack happen inside that
/// round's `handle_all_passed` (mirrors `mechanics_e_l::haunt`'s pattern).
fn kill_by_damage(
    mut state: GameState,
    creature_id: ObjectId,
    players: &[PlayerId],
) -> (GameState, Vec<GameEvent>) {
    let toughness = state
        .objects()
        .get(&creature_id)
        .and_then(|o| o.characteristics.toughness)
        .unwrap_or(1)
        .max(1) as u32;
    state
        .objects_mut()
        .get_mut(&creature_id)
        .unwrap()
        .damage_marked = toughness;
    pass_all(state, players)
}

/// A synthetic `WhenDies` modal trigger, `min_modes`/`max_modes`/`mode_targets`
/// parameterised per test. Both modes are life-total changes of DISTINCT fixed
/// amounts so which mode ran is unambiguous (the PB-DX47 disambiguation-by-
/// amount idiom).
fn synthetic_modal_subject(
    unique_name: &str,
    min_modes: usize,
    mode_targets: Option<Vec<Vec<TargetRequirement>>>,
) -> CardDefinition {
    CardDefinition {
        card_id: CardId(format!("dx35-synthetic-{unique_name}")),
        name: format!("DX35 Synthetic {unique_name}"),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "When this creature dies, choose one -- you gain 10 life; or you gain \
                      100 life."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenDies,
            effect: Effect::Nothing,
            intervening_if: None,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes,
                max_modes: 1,
                modes: vec![
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(10),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(100),
                    },
                ],
                allow_duplicate_modes: false,
                mode_costs: None,
                mode_targets,
            }),
            trigger_zone: None,
        }],
        ..Default::default()
    }
}

/// The unsatisfiable "another target creature you control" requirement `t5`-`t7`
/// use to make one or both modes CR 700.2b-illegal on a board with no OTHER
/// creature.
fn unsatisfiable_creature_target() -> TargetRequirement {
    TargetRequirement::TargetCreatureWithFilter(TargetFilter {
        controller: TargetController::You,
        exclude_self: true,
        ..Default::default()
    })
}

// ── t1: retreat_to_kazandu, no creature -- the CR 603.3d trap ───────────────

/// The criterion's headline probe. At the merge base, `retreat_to_kazandu`'s
/// mode-0 target ("target creature") was declared FLAT, so with no creature on
/// the battlefield the whole trigger was removed (CR 603.3d) and "You gain 2
/// life" (mode 1, which needs no target) was unreachable. After this batch the
/// target is scoped to mode 0 alone; `trigger_modal_plan` picks the first
/// CR 700.2b-legal mode, so mode 1 is chosen and the trigger resolves.
#[test]
fn t1_retreat_to_kazandu_gains_life_with_no_creature_on_the_board() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let retreat_id = card_name_to_id("Retreat to Kazandu");
    let retreat = enrich_spec_from_def(
        ObjectSpec::card(p1, "Retreat to Kazandu")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(retreat_id),
        &defs,
    );
    let land = ObjectSpec::land(p1, "Other Land").in_zone(ZoneId::Hand(p1));
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![defs["Retreat to Kazandu"].clone()]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(retreat)
        .object(land)
        .build()
        .expect("t1 fixture must build");

    assert!(
        state
            .objects()
            .values()
            .all(|o| !o.characteristics.card_types.contains(&CardType::Creature)),
        "non-vacuity premise: NO creature must exist on this board"
    );

    let life_before = state.players()[&p1].life_total;
    let land_id = find_in_zone(&state, "Other Land", ZoneId::Hand(p1));
    let (state, _) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: land_id,
        },
    )
    .expect("PlayLand must succeed");
    assert_eq!(
        state.stack_objects().len(),
        1,
        "the landfall trigger must reach the stack (CR 700.2b permits choosing \
         mode 1, which needs no target, when mode 0 has no legal candidate)"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.stack_objects().len(),
        0,
        "the trigger must have resolved"
    );

    let life_after = state.players()[&p1].life_total;
    assert_eq!(
        life_after,
        life_before + 2,
        "CR 700.2b: with no legal target for mode 0, mode 1 (\"You gain 2 life\") \
         must be chosen and resolve. At the merge base the whole trigger was \
         removed instead (life unchanged)."
    );
}

// ── t2: retreat_to_kazandu, WITH a legal creature ────────────────────────────

/// Mode 0 is CR 700.2b-legal (a creature exists) and is chosen: a target IS
/// announced and the +1/+1 counter lands on it.
#[test]
fn t2_retreat_to_kazandu_puts_a_counter_on_the_only_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let retreat_id = card_name_to_id("Retreat to Kazandu");
    let retreat = enrich_spec_from_def(
        ObjectSpec::card(p1, "Retreat to Kazandu")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(retreat_id),
        &defs,
    );
    let land = ObjectSpec::land(p1, "Other Land").in_zone(ZoneId::Hand(p1));
    let bear = ObjectSpec::creature(p1, "Ally Bear", 2, 2);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![defs["Retreat to Kazandu"].clone()]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(retreat)
        .object(land)
        .object(bear)
        .build()
        .expect("t2 fixture must build");

    let land_id = find_in_zone(&state, "Other Land", ZoneId::Hand(p1));
    let (state, events) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: land_id,
        },
    )
    .expect("PlayLand must succeed");

    assert_eq!(state.stack_objects().len(), 1);
    let so = &state.stack_objects()[0];
    assert_eq!(
        so.modes_chosen,
        vec![0],
        "mode 0 (put a +1/+1 counter) must be the CR 700.2b-legal choice"
    );
    assert_eq!(
        so.targets.len(),
        1,
        "mode 0's single target slot must be the only announced target"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsAnnounced { .. })),
        "a target must be announced (CR 601.2c) -- the criterion's own wording"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    let bear_id = find_obj(&state, "Ally Bear");
    let counters = state.objects()[&bear_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 1,
        "the +1/+1 counter must land on the target creature"
    );
}

// ── t3: shambling_ghast dies, no opponent creature ───────────────────────────

/// Mode 1 (the -1/-1 mode, which needs an opponent creature) is CR 700.2b-
/// illegal with no opponent creature on the board; mode 0 (Create a Treasure
/// token, which needs no target) is chosen instead of the whole trigger being
/// removed.
#[test]
fn t3_shambling_ghast_makes_a_treasure_with_no_opponent_creature() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let ghast_id = card_name_to_id("Shambling Ghast");
    let ghast = enrich_spec_from_def(
        ObjectSpec::creature(p1, "Shambling Ghast", 1, 1)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(ghast_id),
        &defs,
    );
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![defs["Shambling Ghast"].clone()]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(ghast)
        .build()
        .expect("t3 fixture must build");

    let ghast_obj = find_obj(&state, "Shambling Ghast");
    let (state, _) = kill_by_damage(state, ghast_obj, &[p1, p2]);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "the WhenDies trigger must reach the stack (mode 0 needs no target)"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        any_object_named(&state, "Treasure"),
        "mode 0 (Create a Treasure token) must resolve. At the merge base the \
         whole trigger was removed with no opponent creature on the board."
    );
}

// ── t4: shambling_ghast dies, WITH an opponent creature ──────────────────────

/// Mode 0 (Treasure, no target) is still the CR 700.2b first-legal choice, even
/// though mode 1 (which needs an opponent creature) is now ALSO legal --
/// CR 700.2b picks the FIRST legal mode by declared order, not "any legal
/// mode". No target is announced (mode 0's slice is empty).
#[test]
fn t4_shambling_ghast_still_makes_a_treasure_when_an_opponent_creature_exists() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let ghast_id = card_name_to_id("Shambling Ghast");
    let ghast = enrich_spec_from_def(
        ObjectSpec::creature(p1, "Shambling Ghast", 1, 1)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(ghast_id),
        &defs,
    );
    let opp_creature = ObjectSpec::creature(p2, "Opponent Bear", 3, 3);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![defs["Shambling Ghast"].clone()]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(ghast)
        .object(opp_creature)
        .build()
        .expect("t4 fixture must build");

    let ghast_obj = find_obj(&state, "Shambling Ghast");
    let (state, _) = kill_by_damage(state, ghast_obj, &[p1, p2]);
    assert_eq!(state.stack_objects().len(), 1);
    let so = &state.stack_objects()[0];
    assert_eq!(
        so.modes_chosen,
        vec![0],
        "CR 700.2b picks the FIRST legal mode (0), not any legal mode"
    );
    assert_eq!(
        so.targets.len(),
        0,
        "mode 0's target slice is empty -- pin the announcement count"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        any_object_named(&state, "Treasure"),
        "mode 0 (Create a Treasure token) must resolve"
    );
    let opp_id = find_obj(&state, "Opponent Bear");
    assert_eq!(
        state.objects()[&opp_id].characteristics.power,
        Some(3),
        "mode 1's -1/-1 must NOT have applied -- mode 0 was chosen"
    );
}

// ── t5: synthetic -- mode 0 illegal, mode 1 legal, min_modes: 1 ─────────────

/// With no candidate for mode 0's target, `trigger_modal_plan` falls through
/// to mode 1 (which needs no target). Pin `modes_chosen` directly.
#[test]
fn t5_no_candidate_for_mode_0_falls_through_to_mode_1() {
    let p1 = p(1);
    let p2 = p(2);
    let def = synthetic_modal_subject(
        "t5",
        1,
        Some(vec![vec![unsatisfiable_creature_target()], vec![]]),
    );
    let subject = enrich_spec_from_def(
        ObjectSpec::creature(p1, &def.name, 1, 1)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(def.card_id.clone()),
        &HashMap::from([(def.name.clone(), def.clone())]),
    );
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(subject)
        .build()
        .expect("t5 fixture must build");

    let life_before = state.players()[&p1].life_total;
    let subject_id = find_obj(&state, &def.name);
    let (state, _) = kill_by_damage(state, subject_id, &[p1, p2]);
    assert_eq!(state.stack_objects().len(), 1);
    assert_eq!(
        state.stack_objects()[0].modes_chosen,
        vec![1],
        "mode 0 has no legal candidate (excludes self, no other creature); \
         mode 1 must be chosen"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players()[&p1].life_total,
        life_before + 100,
        "mode 1 (+100 life) must have resolved, not mode 0 (+10)"
    );
}

// ── t6: synthetic -- both modes illegal, min_modes: 0 ────────────────────────

/// CR 700.2b: "choose up to one" legally chooses ZERO modes when none is
/// legal. The ability still reaches the stack (an empty target requirement
/// list is not a CR 603.3d removal) but resolves with NO EFFECT.
#[test]
fn t6_no_legal_mode_with_min_modes_zero_resolves_with_no_effect() {
    let p1 = p(1);
    let p2 = p(2);
    let def = synthetic_modal_subject(
        "t6",
        0,
        Some(vec![
            vec![unsatisfiable_creature_target()],
            vec![unsatisfiable_creature_target()],
        ]),
    );
    let subject = enrich_spec_from_def(
        ObjectSpec::creature(p1, &def.name, 1, 1)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(def.card_id.clone()),
        &HashMap::from([(def.name.clone(), def.clone())]),
    );
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(subject)
        .build()
        .expect("t6 fixture must build");

    let life_before = state.players()[&p1].life_total;
    let subject_id = find_obj(&state, &def.name);
    let (state, _) = kill_by_damage(state, subject_id, &[p1, p2]);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "min_modes: 0 with no legal mode is 'chose zero modes', not a removal"
    );
    assert_eq!(
        state.stack_objects()[0].modes_chosen,
        Vec::<usize>::new(),
        "no mode was chosen"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players()[&p1].life_total,
        life_before,
        "NEITHER mode's effect may run -- the trigger resolves doing nothing. \
         At the merge base an empty modes_chosen fell through to the runtime \
         `effect` field, which for three lowering arms silently executes mode \
         0 anyway."
    );
}

// ── t7: synthetic -- both modes illegal, min_modes: 1 -- CR 700.2b removal ──

/// With `min_modes: 1` and no legal mode, the ability is removed from the
/// stack entirely (CR 700.2b) -- it never reaches the stack.
#[test]
fn t7_no_legal_mode_with_min_modes_one_removes_the_trigger() {
    let p1 = p(1);
    let p2 = p(2);
    let def = synthetic_modal_subject(
        "t7",
        1,
        Some(vec![
            vec![unsatisfiable_creature_target()],
            vec![unsatisfiable_creature_target()],
        ]),
    );
    let subject = enrich_spec_from_def(
        ObjectSpec::creature(p1, &def.name, 1, 1)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(def.card_id.clone()),
        &HashMap::from([(def.name.clone(), def.clone())]),
    );
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(subject)
        .build()
        .expect("t7 fixture must build");

    let life_before = state.players()[&p1].life_total;
    let subject_id = find_obj(&state, &def.name);
    let (state, _) = kill_by_damage(state, subject_id, &[p1, p2]);
    assert_eq!(
        state.stack_objects().len(),
        0,
        "CR 700.2b: 'If no mode is chosen, the ability is removed from the \
         stack.' min_modes: 1 with no legal mode must never reach the stack."
    );
    assert_eq!(
        state.players()[&p1].life_total,
        life_before,
        "non-vacuity: neither mode's life gain fired"
    );
}

// ── t8: mode_targets: None -- byte-identical to the merge base ──────────────

/// The A1 step-3 backward-compatibility pin: an unscoped modal trigger
/// (`mode_targets: None`) always picks mode 0 whenever a mode exists, exactly
/// as it did before this batch -- regardless of board state, because a flat
/// (or absent) requirement list cannot differ by mode.
#[test]
fn t8_unscoped_modal_trigger_always_picks_mode_zero() {
    let p1 = p(1);
    let p2 = p(2);
    let def = synthetic_modal_subject("t8", 1, None);
    let subject = enrich_spec_from_def(
        ObjectSpec::creature(p1, &def.name, 1, 1)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(def.card_id.clone()),
        &HashMap::from([(def.name.clone(), def.clone())]),
    );
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(subject)
        .build()
        .expect("t8 fixture must build");

    let life_before = state.players()[&p1].life_total;
    let subject_id = find_obj(&state, &def.name);
    let (state, _) = kill_by_damage(state, subject_id, &[p1, p2]);
    assert_eq!(state.stack_objects().len(), 1);
    assert_eq!(
        state.stack_objects()[0].modes_chosen,
        vec![0],
        "mode_targets: None must keep picking mode 0 unconditionally"
    );

    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.players()[&p1].life_total,
        life_before + 10,
        "mode 0 (+10 life) must resolve, not mode 1 (+100)"
    );
}
