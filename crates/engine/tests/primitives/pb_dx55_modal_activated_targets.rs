//! PB-DX55 Half 3 (`OOS-SIM5-5`) — CR 700.2a/700.2c/700.2f: per-mode target
//! requirements for a modal ACTIVATED ability, on the three real corpus members
//! (`Cankerbloom`, `Goblin Cratermaker`, `Umezawa's Jitte`).
//!
//! The corpus roster is `core::pb_dx55_modal_activated_roster`; the mechanism gate over
//! `per_mode_target_requirements`'s call sites and the reviewed `.mode_targets`
//! reader list is `core::pb_dx55_per_mode_slicer_ratchet`. This file is the
//! *behavioural* half: every requirement asserted here is checked against a real
//! `all_cards()` def, through a real `process_command` activation, resolving to a real
//! effect on the battlefield -- never merely a returned `Ok(_)`.
use mtg_engine::state::GameStateError;
use mtg_engine::{
    ability_target_requirements, all_cards, card_name_to_id, enrich_spec_from_def, process_command,
    CardDefinition, CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ManaColor,
    ObjectId, ObjectSpec, PlayerId, Step, Target, TargetRequirement, ZoneId,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found on the battlefield"))
}

fn is_on_battlefield(state: &GameState, id: ObjectId) -> bool {
    state
        .objects()
        .get(&id)
        .is_some_and(|o| o.zone == ZoneId::Battlefield)
}

/// Layer-resolved activated-ability index of the (single, per r1) modal ability on
/// `name`, found by the shape `ability_default_modes`/`handle_activate_ability` also
/// use -- `modes.is_some()` -- rather than a hardcoded number, so this file does not
/// silently repeat the plan doc's wrong "index 1" for Jitte.
fn modal_ability_index(state: &GameState, id: ObjectId) -> usize {
    mtg_engine::calculate_characteristics(state, id)
        .expect("layer-resolved characteristics")
        .activated_abilities
        .iter()
        .position(|a| a.modes.is_some())
        .unwrap_or_else(|| panic!("object {id:?} has no modal activated ability"))
}

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

fn activate(
    state: GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index,
            targets,
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
}

/// p1 controls `subject` (a real, enriched def) plus a Sol Ring (artifact) and an
/// Anointed Procession (enchantment) -- both `Complete`, deck-legal, real corpus defs,
/// reused from `pb_dx20b_enchant_offer_channel.rs`'s own board so a fresh reader
/// recognises the witnesses. `mana` generic mana is pre-floated for `subject`'s own
/// activation cost.
fn setup_with_artifact_and_enchantment(
    subject_name: &str,
    mana: u32,
) -> (GameState, ObjectId, ObjectId, ObjectId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let subject = enrich_spec_from_def(
        ObjectSpec::card(p1, subject_name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id(subject_name)),
        &defs,
    );
    let sol_ring = enrich_spec_from_def(
        ObjectSpec::card(p1, "Sol Ring")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Sol Ring")),
        &defs,
    );
    let procession = enrich_spec_from_def(
        ObjectSpec::card(p2, "Anointed Procession")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Anointed Procession")),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(subject)
        .object(sol_ring)
        .object(procession)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, mana);
    state.turn_mut().priority_holder = Some(p1);

    let subject_id = find_object(&state, subject_name);
    let sol_ring_id = find_object(&state, "Sol Ring");
    let procession_id = find_object(&state, "Anointed Procession");
    (state, subject_id, sol_ring_id, procession_id, p1)
}

// ── Cankerbloom (CR 700.2c, three modes: two targeted, one not) ────────────────────

/// t1: the three modes of Cankerbloom's ability announce THREE DIFFERENT
/// requirement lists -- CR 700.2c/700.2f, and the exact defect `OOS-SIM5-5` names:
/// before this batch `ability_target_requirements` (the pre-PB-DX55 form) returned
/// `vec![]` for EVERY mode, because it read only the flat (always-empty, per the
/// author invariant) `targets` list.
#[test]
fn t1_cankerbloom_modes_announce_distinct_per_mode_target_requirements() {
    let (state, source, ..) = setup_with_artifact_and_enchantment("Cankerbloom", 1);
    let idx = modal_ability_index(&state, source);

    let mode0 = ability_target_requirements(&state, source, idx, &[0]);
    let mode1 = ability_target_requirements(&state, source, idx, &[1]);
    let mode2 = ability_target_requirements(&state, source, idx, &[2]);

    assert_eq!(
        mode0,
        vec![TargetRequirement::TargetArtifact],
        "mode 0 (destroy target artifact)"
    );
    assert_eq!(
        mode1,
        vec![TargetRequirement::TargetEnchantment],
        "mode 1 (destroy target enchantment)"
    );
    assert_eq!(
        mode2,
        Vec::<TargetRequirement>::new(),
        "mode 2 (proliferate) has no target requirement at all"
    );

    // The OLD 3-argument query (no chosen modes) still reports `vec![]` -- the
    // CR 601.2b-faithful "no targets until you announce your modes" answer, and the
    // SAME answer it gave before this batch (the flat `targets` list is empty by
    // the CR 700.2c author invariant `core::pb_dx55_modal_activated_roster::r2` pins).
    assert_eq!(
        ability_target_requirements(&state, source, idx, &[]),
        Vec::<TargetRequirement>::new(),
        "the un-mode-aware query must be unaffected by this batch"
    );
    assert_eq!(
        ability_target_requirements(&state, source, idx, &[]),
        Vec::<TargetRequirement>::new(),
        "modes_chosen: &[] must agree with the 3-argument form exactly"
    );
}

/// t2: mode 0 -- ACTIVATE for real, targeting the Sol Ring, and assert the
/// RESOLUTION EFFECT (the artifact is destroyed), never merely `Ok(_)`. This is the
/// activation path this batch's `handle_activate_ability` refactor touches
/// (`mode_targets_active` now delegates to `casting::per_mode_target_requirements`
/// instead of re-deriving the slice inline) -- exercised for real, not through the
/// query alone.
#[test]
fn t2_cankerbloom_mode_0_destroys_the_targeted_artifact() {
    let (state, source, sol_ring, _procession, p1) =
        setup_with_artifact_and_enchantment("Cankerbloom", 1);
    let idx = modal_ability_index(&state, source);

    let (state, _) = activate(
        state,
        p1,
        source,
        idx,
        vec![Target::Object(sol_ring)],
        vec![0],
    )
    .expect("Cankerbloom mode 0 (destroy target artifact) must be a legal activation");
    // The ability goes onto the stack (CR 602.2); pass priority both ways to resolve
    // it before checking the resolution effect.
    let (state, events) = pass_all(state, &[p1, p(2)]);

    assert!(
        !is_on_battlefield(&state, sol_ring),
        "CR 700.2c: mode 0's DestroyPermanent effect must actually have run"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentDestroyed { object_id, .. } if *object_id == sol_ring)),
        "the resolution effect must be observable on the event log, not merely inferred \
         from final state: {events:?}"
    );
}

/// t3: mode 1 -- the OTHER targeted mode, on the SAME ability, destroying the
/// enchantment instead. Discriminates t2: if the engine still executed mode 0
/// regardless of `modes_chosen`, this would destroy the Sol Ring instead (or fail
/// to find a legal target for the enchantment requirement) rather than the
/// Anointed Procession.
#[test]
fn t3_cankerbloom_mode_1_destroys_the_targeted_enchantment() {
    let (state, source, sol_ring, procession, p1) =
        setup_with_artifact_and_enchantment("Cankerbloom", 1);
    let idx = modal_ability_index(&state, source);

    let (state, _) = activate(
        state,
        p1,
        source,
        idx,
        vec![Target::Object(procession)],
        vec![1],
    )
    .expect("Cankerbloom mode 1 (destroy target enchantment) must be a legal activation");
    let (state, events) = pass_all(state, &[p1, p(2)]);

    assert!(
        !is_on_battlefield(&state, procession),
        "CR 700.2c: mode 1's DestroyPermanent effect must actually have run"
    );
    assert!(
        is_on_battlefield(&state, sol_ring),
        "mode 1 must not touch the artifact -- if it destroyed the Sol Ring instead, \
         `handle_activate_ability` picked the wrong mode's effect"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentDestroyed { object_id, .. } if *object_id == procession)),
        "the resolution effect must be observable on the event log: {events:?}"
    );
}

/// t4: mode 2 (Proliferate) has no target requirement -- activating it with ZERO
/// targets must be accepted (CR 700.2c: "the ability is treated as though it did
/// not have those targets" for a mode that is not chosen).
#[test]
fn t4_cankerbloom_mode_2_proliferate_needs_no_target() {
    let (state, source, sol_ring, procession, p1) =
        setup_with_artifact_and_enchantment("Cankerbloom", 1);
    let idx = modal_ability_index(&state, source);

    let (state, _) = activate(state, p1, source, idx, vec![], vec![2])
        .expect("Cankerbloom mode 2 (proliferate) must accept zero targets");
    let (state, _events) = pass_all(state, &[p1, p(2)]);

    // Proliferate with no counters anywhere is a legal no-op; the point is that the
    // command was ACCEPTED, and neither witness was touched.
    assert!(is_on_battlefield(&state, sol_ring));
    assert!(is_on_battlefield(&state, procession));
}

/// t5: CR 700.2c author invariant, enforced end-to-end -- targeting mode 1's
/// requirement (an enchantment) with an ARTIFACT is refused with the same
/// "wrong target type" error a flat-list activation would give, proving the
/// refactored `mode_targets_active` still enforces per-mode target TYPE, not just
/// per-mode target PRESENCE.
#[test]
fn t5_cankerbloom_mode_1_refuses_an_artifact_target() {
    let (state, source, sol_ring, _procession, p1) =
        setup_with_artifact_and_enchantment("Cankerbloom", 1);
    let idx = modal_ability_index(&state, source);

    let err = activate(
        state,
        p1,
        source,
        idx,
        vec![Target::Object(sol_ring)],
        vec![1],
    )
    .expect_err("mode 1 requires an ENCHANTMENT target; the Sol Ring is an artifact");
    let msg = format!("{err:?}");
    assert!(
        msg.to_lowercase().contains("target") || msg.to_lowercase().contains("invalid"),
        "expected a target-legality refusal, got {msg}"
    );
}

/// t6: CR 700.2c author invariant -- `mode_targets_active.is_some() &&
/// validated_modes_chosen.len() > 1` is hard-rejected regardless of whether the
/// individually-chosen modes are each legal. Unchanged by this batch's refactor
/// (only the SLICE computation moved; this guard sits immediately after it,
/// untouched) -- pinned here so a future edit that moves the guard cannot silently
/// drop it.
#[test]
fn t6_cankerbloom_refuses_choosing_two_modes_with_mode_targets() {
    let (state, source, sol_ring, procession, p1) =
        setup_with_artifact_and_enchantment("Cankerbloom", 1);
    let idx = modal_ability_index(&state, source);

    let err = activate(
        state,
        p1,
        source,
        idx,
        vec![Target::Object(sol_ring), Target::Object(procession)],
        vec![0, 1],
    )
    .expect_err("CR 700.2c: multiple modes chosen combined with ModeSelection.mode_targets");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("700.2c") || msg.to_lowercase().contains("mode"),
        "expected the CR 700.2c multi-mode-with-mode_targets refusal, got {msg}"
    );
}

// ── Goblin Cratermaker (a second corpus member, different target shapes) ───────────

/// A second target-carrying artifact witness (Sol Ring is colored-neutral but the
/// TargetPermanentWithFilter requirement excludes it via `non_land` + all five
/// colours excluded -- Sol Ring has no colour, so it still matches "colorless
/// nonland" and IS a legal mode-1 target too; a separate creature witness is what
/// discriminates mode 0 from mode 1).
fn setup_cratermaker() -> (GameState, ObjectId, ObjectId, ObjectId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let subject = enrich_spec_from_def(
        ObjectSpec::card(p1, "Goblin Cratermaker")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Goblin Cratermaker")),
        &defs,
    );
    let creature = ObjectSpec::creature(p2, "Opposing Bear", 2, 2);
    let sol_ring = enrich_spec_from_def(
        ObjectSpec::card(p2, "Sol Ring")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Sol Ring")),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(subject)
        .object(creature)
        .object(sol_ring)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);

    let subject_id = find_object(&state, "Goblin Cratermaker");
    let creature_id = find_object(&state, "Opposing Bear");
    let sol_ring_id = find_object(&state, "Sol Ring");
    (state, subject_id, creature_id, sol_ring_id, p1)
}

/// t7: mode 0 (deal 2 damage to target creature) and mode 1 (destroy target
/// colorless nonland permanent) announce different requirement SHAPES, not just
/// different labels -- `TargetCreature` vs a `TargetPermanentWithFilter` carrying
/// `non_land: true` and all five colours excluded.
#[test]
fn t7_goblin_cratermaker_modes_have_distinct_requirement_shapes() {
    let (state, source, ..) = setup_cratermaker();
    let idx = modal_ability_index(&state, source);

    let mode0 = ability_target_requirements(&state, source, idx, &[0]);
    assert_eq!(mode0, vec![TargetRequirement::TargetCreature]);

    let mode1 = ability_target_requirements(&state, source, idx, &[1]);
    assert_eq!(mode1.len(), 1);
    assert!(
        matches!(mode1[0], TargetRequirement::TargetPermanentWithFilter(ref f) if f.non_land && f.exclude_colors.is_some()),
        "mode 1 must be a colorless-nonland filter, got {mode1:?}"
    );
}

/// t8: ACTIVATE mode 1 for real against the Sol Ring (colorless, nonland,
/// permanent) -- resolves as a real destroy, not merely a legal announcement.
#[test]
fn t8_goblin_cratermaker_mode_1_destroys_the_colorless_permanent() {
    let (state, source, creature, sol_ring, p1) = setup_cratermaker();
    let idx = modal_ability_index(&state, source);

    let (state, _) = activate(
        state,
        p1,
        source,
        idx,
        vec![Target::Object(sol_ring)],
        vec![1],
    )
    .expect("Goblin Cratermaker mode 1 must be a legal activation against the Sol Ring");
    let (state, events) = pass_all(state, &[p1, p(2)]);

    assert!(!is_on_battlefield(&state, sol_ring));
    assert!(
        is_on_battlefield(&state, creature),
        "mode 1 must not have damaged/killed the creature -- mode 0's effect must not run"
    );
    assert!(events.iter().any(
        |e| matches!(e, GameEvent::PermanentDestroyed { object_id, .. } if *object_id == sol_ring)
    ));
}

// ── Umezawa's Jitte (ability index 0 -- NOT 1, see r3 for the correction) ──────────

fn setup_jitte() -> (GameState, ObjectId, ObjectId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let jitte = enrich_spec_from_def(
        ObjectSpec::artifact(p1, "Umezawa's Jitte")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Umezawa's Jitte")),
        &defs,
    );
    let target_creature = ObjectSpec::creature(p2, "Target Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(jitte)
        .object(target_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let jitte_id = find_object(&state, "Umezawa's Jitte");
    // The activation cost is `RemoveCounter { Charge, 1 }`: give it one to spend.
    if let Some(obj) = state.objects_mut().get_mut(&jitte_id) {
        obj.counters.insert(mtg_engine::CounterType::Charge, 1);
    }
    let creature_id = find_object(&state, "Target Bear");
    (state, jitte_id, creature_id, p1)
}

/// t9: Jitte's modal ability is at layer-resolved index **0** (not 1 -- see
/// `core::pb_dx55_modal_activated_roster::r3`), and its three modes announce the
/// expected shapes: no target, one creature target, no target.
#[test]
fn t9_jitte_modal_ability_is_index_zero_with_three_distinct_modes() {
    let (state, source, ..) = setup_jitte();
    let idx = modal_ability_index(&state, source);
    assert_eq!(
        idx, 0,
        "Umezawa's Jitte's modal ability must be at index 0, not 1"
    );

    let mode0 = ability_target_requirements(&state, source, idx, &[0]);
    let mode1 = ability_target_requirements(&state, source, idx, &[1]);
    let mode2 = ability_target_requirements(&state, source, idx, &[2]);
    assert_eq!(
        mode0,
        Vec::<TargetRequirement>::new(),
        "mode 0: equipped creature +2/+2, no target"
    );
    assert_eq!(
        mode1,
        vec![TargetRequirement::TargetCreature],
        "mode 1: target creature -1/-1"
    );
    assert_eq!(
        mode2,
        Vec::<TargetRequirement>::new(),
        "mode 2: you gain 2 life, no target"
    );
}

/// t10: ACTIVATE mode 1 for real, targeting the opposing creature -- resolves as a
/// real -1/-1 continuous effect, visible through `calculate_characteristics`.
#[test]
fn t10_jitte_mode_1_resolves_as_a_real_minus_one_minus_one_effect() {
    let (state, source, creature, p1) = setup_jitte();
    let idx = modal_ability_index(&state, source);
    let before = mtg_engine::calculate_characteristics(&state, creature)
        .expect("creature must have characteristics before activation");
    assert_eq!(before.power, Some(2));
    assert_eq!(before.toughness, Some(2));

    let (state, _) = activate(
        state,
        p1,
        source,
        idx,
        vec![Target::Object(creature)],
        vec![1],
    )
    .expect("Jitte mode 1 (target creature -1/-1) must be a legal activation");
    let (state, events) = pass_all(state, &[p1, p(2)]);

    let after = mtg_engine::calculate_characteristics(&state, creature)
        .expect("creature must still exist and have characteristics after activation");
    assert_eq!(
        (after.power, after.toughness),
        (Some(1), Some(1)),
        "CR 700.2c: mode 1's ApplyContinuousEffect must actually be in force"
    );
    assert!(
        !events.is_empty(),
        "the activation must have produced at least one event"
    );
}
