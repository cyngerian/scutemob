//! PB-OS4b (scutemob-134, OOS-OS4-2): face-aware ability gathering for
//! transformed permanents (CR 712.8d/e).
//!
//! A transformed permanent has only its BACK face's characteristics --
//! including its triggered/activated/mana/static abilities. Before this PB,
//! two independent "ability channels" stayed front-face-only after transform:
//!
//! - Channel A: the runtime `Characteristics.{mana,activated,triggered}_abilities`
//!   vectors, lowered once at object construction and never rebuilt on transform.
//! - Channel B: static continuous effect registration and several `def.abilities`
//!   direct-index sites (ETB queue, upkeep sweep, mana-tap sweep, Saga SBA,
//!   CardDefETB consumers), which never consulted the back face either.
//!
//! `apply_face_change` (rules/face.rs) is now the single choke point that keeps
//! both channels correct whenever `is_transformed` flips on a battlefield
//! permanent.
//!
//! Probes (AC 5058, by execution against REAL card defs):
//! - `docent_of_perfection`: back Wizard anthem + back cast-trigger fire; front
//!   cast-trigger's "then transform" clause is gone post-transform.
//! - `bloodline_keeper`: back token ability activatable; front transform
//!   ability index gone; back Vampire anthem applies.
//! - `growing_rites_of_itlimoc` / `thaumatic_compass`: back mana abilities,
//!   dead pre-fix, now function.
//! - `fable_of_the_mirror_breaker`: back Reflection of Kiki-Jiki activated
//!   ability is reachable/activatable post-transform (does not assert full
//!   copy correctness -- the Kiki-Jiki nonlegendary `TargetFilter` gap is
//!   separate, OOS-OS4-2 scope only covers reachability).
//!
//! Decoys (AC 5057, synthetic DFCs pinning specific mechanisms):
//! - front static removed / re-added on transform-there-and-back
//!   (`deregister_face_statics` + `register_static_continuous_effects`).
//! - back upkeep trigger fires ONLY when transformed (Channel-B upkeep sweep
//!   producer/consumer index parity).
//! - a transformed Saga with no back-face `SagaChapter` abilities is no longer
//!   a Saga (CR 714.4, `sba.rs` effective-abilities).
//! - a non-DFC's ability set is a no-op under `Effect::TransformSelf`.
//! - an off-battlefield DFC reports front-face abilities (CR 400.7/712.8a).

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::rules::replacement::register_static_continuous_effects;
use mtg_engine::{
    all_cards, calculate_characteristics, check_and_apply_sbas, enrich_spec_from_def,
    AbilityDefinition, CardContinuousEffectDef, CardDefinition, CardFace, CardId, CardRegistry,
    CardType, Command, Completeness, CounterType, Effect, EffectDuration, EffectFilter,
    EffectLayer, GameEvent, GameState, GameStateBuilder, GameStateError, KeywordAbility,
    LayerModification, ManaColor, ObjectId, ObjectSpec, PlayerId, Step, SubType, Target,
    TriggerCondition, TypeLine, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: &ZoneId) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && &obj.zone == zone)
        .map(|(id, _)| *id)
}

fn count_on_battlefield(state: &GameState, name: &str) -> usize {
    state
        .objects()
        .iter()
        .filter(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .count()
}

fn registry_with(defs: Vec<CardDefinition>) -> std::sync::Arc<CardRegistry> {
    CardRegistry::new(defs)
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
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

/// Pass priority for all listed players once (resolves top of stack or advances turn).
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command_pass(current, pl);
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

fn process_command_pass(state: GameState, player: PlayerId) -> (GameState, Vec<GameEvent>) {
    mtg_engine::process_command(state, Command::PassPriority { player })
        .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", player, e))
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

fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(guard < 500, "advance_to_step exceeded safety guard");
        let holder = state.turn().priority_holder.expect("no priority holder");
        let (new_state, _) =
            mtg_engine::process_command(state, Command::PassPriority { player: holder })
                .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
    }
}

fn empty_cast_spell(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
        targets: vec![],
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
    }))
}

// ── PROBE: docent_of_perfection ───────────────────────────────────────────────

/// Transform the real `Docent of Perfection` in place and cast an instant.
/// Returns the resulting state after the cast + full stack resolution.
fn docent_transformed_then_cast_instant() -> GameState {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Docent of Perfection",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(
            ObjectSpec::creature(p1, "Test Wizard", 1, 1)
                .with_subtypes(vec![SubType("Wizard".to_string())]),
        )
        .object(real_card_spec(p1, "Opt", ZoneId::Hand(p1), &defs))
        .object(ObjectSpec::card(p1, "Filler A").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Filler B").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let docent_id = find_by_name(&state, "Docent of Perfection");
    let mut ctx = EffectContext::new(p1, docent_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
    assert!(
        state.objects()[&docent_id].is_transformed,
        "sanity: docent should be transformed before casting"
    );

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn_mut().priority_holder = Some(p1);
    let opt_id = find_by_name(&state, "Opt");
    let (state, _) = mtg_engine::process_command(state, empty_cast_spell(p1, opt_id))
        .unwrap_or_else(|e| panic!("CastSpell(Opt) failed: {:?}", e));
    resolve_stack(state, &[p1, p2])
}

/// CR 712.8d/e: after Docent of Perfection transforms, casting an instant fires
/// the BACK face's simpler trigger (create exactly one Wizard token; no further
/// "then transform" clause -- that clause only exists on the front face).
#[test]
fn test_docent_back_cast_trigger_fires_after_transform() {
    let state = docent_transformed_then_cast_instant();
    assert_eq!(
        count_on_battlefield(&state, "Human Wizard"),
        1,
        "the back trigger should create exactly one Wizard token"
    );
}

/// CR 712.8d/e decoy: the front face's "then if you control 3+ Wizards, transform"
/// clause must NOT re-fire post-transform (it does not exist on the back face).
/// Pre-fix, Channel A kept serving the front trigger, which re-triggered
/// TransformSelf back to the front face on every subsequent instant/sorcery cast.
#[test]
fn test_docent_front_cast_trigger_stops_after_transform() {
    let state = docent_transformed_then_cast_instant();
    let docent_id = find_by_name(&state, "Docent of Perfection");
    assert!(
        state.objects()[&docent_id].is_transformed,
        "docent must remain transformed -- the front's conditional re-transform clause \
         must not fire from the back face"
    );
}

/// CR 613.1c/613.1f: Docent's back face static anthem ("Wizards you control get
/// +2/+1 and have flying") applies only after transform -- the front face has no
/// static ability at all.
#[test]
fn test_docent_back_wizard_anthem_applies() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Docent of Perfection",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(
            ObjectSpec::creature(p1, "Test Wizard", 1, 1)
                .with_subtypes(vec![SubType("Wizard".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let docent_id = find_by_name(&state, "Docent of Perfection");
    let wizard_id = find_by_name(&state, "Test Wizard");

    // Before transform: front has no static, no anthem.
    let chars_before = calculate_characteristics(&state, wizard_id).unwrap();
    assert_eq!(chars_before.power, Some(1));
    assert_eq!(chars_before.toughness, Some(1));
    assert!(!chars_before.keywords.contains(&KeywordAbility::Flying));

    let mut ctx = EffectContext::new(p1, docent_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);

    // After transform: back face's anthem applies (+2/+1, flying).
    let chars_after = calculate_characteristics(&state, wizard_id).unwrap();
    assert_eq!(chars_after.power, Some(3));
    assert_eq!(chars_after.toughness, Some(2));
    assert!(chars_after.keywords.contains(&KeywordAbility::Flying));
}

#[test]
fn test_docent_of_perfection_stays_complete() {
    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Docent of Perfection")
        .expect("Docent of Perfection should have a CardDefinition");
    assert_eq!(
        def.completeness,
        Completeness::Complete,
        "docent should verify Complete now that face-aware gathering is fixed"
    );
}

// ── PROBE: bloodline_keeper ───────────────────────────────────────────────────

fn build_bloodline_state() -> (GameState, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Bloodline Keeper",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(
            ObjectSpec::creature(p1, "Test Vampire", 1, 1)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bloodline_id = find_by_name(&state, "Bloodline Keeper");
    let mut ctx = EffectContext::new(p1, bloodline_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
    assert!(state.objects()[&bloodline_id].is_transformed);
    state.turn_mut().priority_holder = Some(p1);
    (state, bloodline_id)
}

/// CR 712.8d/e: after Bloodline Keeper transforms to Lord of Lineage, its back
/// face's `{T}: Create a 2/2 black Vampire creature token with flying.` ability
/// (activated_abilities index 0 on the back face) is activatable.
#[test]
fn test_bloodline_back_token_ability_activatable_after_transform() {
    let (state, bloodline_id) = build_bloodline_state();
    let before = count_on_battlefield(&state, "Vampire");
    let (state, _) = mtg_engine::process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: bloodline_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("back token ability should be activatable: {:?}", e));
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_on_battlefield(&state, "Vampire"),
        before + 1,
        "activating the back face's token ability should create a Vampire token"
    );
}

/// CR 712.8d/e decoy: the FRONT face's `{B}: Transform this creature.` activated
/// ability (front index 1) must be absent from the back face's activated-ability
/// list -- attempting to activate index 1 (which the back face doesn't have)
/// fails with `InvalidAbilityIndex`.
#[test]
fn test_bloodline_front_transform_ability_gone() {
    let (state, bloodline_id) = build_bloodline_state();
    let result = mtg_engine::process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: bloodline_id,
            ability_index: 1,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidAbilityIndex { .. })),
        "the front's {{B}}: transform ability must not be reachable at index 1 on the \
         back face; got {:?}",
        result.map(|_| ())
    );
}

/// CR 613.1c: Lord of Lineage's "Other Vampire creatures you control get +2/+2"
/// static applies only after transform -- the front face has no static ability.
#[test]
fn test_bloodline_back_vampire_anthem_applies_after_transform() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Bloodline Keeper",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(
            ObjectSpec::creature(p1, "Test Vampire", 1, 1)
                .with_subtypes(vec![SubType("Vampire".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let bloodline_id = find_by_name(&state, "Bloodline Keeper");
    let vampire_id = find_by_name(&state, "Test Vampire");

    let before = calculate_characteristics(&state, vampire_id).unwrap();
    assert_eq!(before.power, Some(1));
    assert_eq!(before.toughness, Some(1));

    let mut ctx = EffectContext::new(p1, bloodline_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);

    let after = calculate_characteristics(&state, vampire_id).unwrap();
    assert_eq!(
        after.power,
        Some(3),
        "CR 613.1c: +2/+2 anthem should apply post-transform"
    );
    assert_eq!(after.toughness, Some(3));
}

#[test]
fn test_bloodline_keeper_stays_complete() {
    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Bloodline Keeper")
        .expect("Bloodline Keeper should have a CardDefinition");
    assert_eq!(
        def.completeness,
        Completeness::Complete,
        "bloodline_keeper should verify Complete now that face-aware gathering is fixed"
    );
}

// ── PROBE: growing_rites_of_itlimoc / thaumatic_compass mana abilities ───────

/// CR 712.8d/e: Growing Rites of Itlimoc's back face (Itlimoc, Cradle of the Sun)
/// mana ability `{T}: Add {G}` was dead pre-fix (Channel A still held the front's
/// empty ability list). Post-fix, tapping for mana succeeds.
#[test]
fn test_growing_rites_itlimoc_taps_for_mana_after_transform() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Growing Rites of Itlimoc",
            ZoneId::Battlefield,
            &defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let itlimoc_id = find_by_name(&state, "Growing Rites of Itlimoc");
    let mut ctx = EffectContext::new(p1, itlimoc_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
    assert!(state.objects()[&itlimoc_id].is_transformed);

    let (state, events) = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: itlimoc_id,
            ability_index: 0,
            chosen_color: None,
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "back mana ability should be tappable post-transform: {:?}",
            e
        )
    });
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                color: ManaColor::Green,
                ..
            }
        )),
        "tapping Itlimoc should add {{G}}; events: {:?}",
        events
    );
    let _ = state;
}

/// CR 712.8d/e: Thaumatic Compass's back face (Spires of Orazca) mana ability
/// `{T}: Add {C}` was dead pre-fix. Post-fix, tapping for mana succeeds.
#[test]
fn test_thaumatic_compass_spires_taps_for_colorless_after_transform() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Thaumatic Compass",
            ZoneId::Battlefield,
            &defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let compass_id = find_by_name(&state, "Thaumatic Compass");
    let mut ctx = EffectContext::new(p1, compass_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
    assert!(state.objects()[&compass_id].is_transformed);

    let (_state, events) = mtg_engine::process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: compass_id,
            ability_index: 0,
            chosen_color: None,
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "back mana ability should be tappable post-transform: {:?}",
            e
        )
    });
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                color: ManaColor::Colorless,
                ..
            }
        )),
        "tapping Spires of Orazca should add {{C}}; events: {:?}",
        events
    );
}

// ── PROBE: fable_of_the_mirror_breaker ────────────────────────────────────────

/// CR 712.8d/e: Fable of the Mirror-Breaker's chapter III
/// (`Effect::ExileSourceAndReturnTransformed`) returns it as Reflection of
/// Kiki-Jiki. Post-fix, the back face's `{1},{T}: Create a token copy...`
/// activated ability is reachable/activatable -- this test only asserts
/// reachability (the command succeeds), not full copy correctness (the
/// Kiki-Jiki "nonlegendary" `TargetFilter` gap is a separate, pre-existing
/// residual, OOS-EF5 scope, unaffected by this PB).
#[test]
fn test_fable_reflection_activated_reachable_after_transform() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Fable of the Mirror-Breaker",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Test Vanilla Creature", 2, 2))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let fable_id = find_by_name(&state, "Fable of the Mirror-Breaker");
    let target_id = find_by_name(&state, "Test Vanilla Creature");
    let mut ctx = EffectContext::new(p1, fable_id, vec![]);
    let _ = execute_effect(
        &mut state,
        &Effect::ExileSourceAndReturnTransformed,
        &mut ctx,
    );

    let reflection_id = state
        .objects()
        .iter()
        .find(|(_, obj)| obj.zone == ZoneId::Battlefield && obj.is_transformed && obj.owner == p1)
        .map(|(id, _)| *id)
        .expect("Reflection of Kiki-Jiki should be on the battlefield");
    let chars = calculate_characteristics(&state, reflection_id).unwrap();
    assert_eq!(chars.name, "Reflection of Kiki-Jiki");

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);
    // Reflection of Kiki-Jiki is a NEW object (CR 400.7 -- the exile-and-return
    // path, unlike an in-place TransformSelf flip) and would otherwise have
    // summoning sickness; clear it so this test can isolate "is the back-face
    // ability reachable" from the unrelated summoning-sickness gate.
    state
        .objects_mut()
        .get_mut(&reflection_id)
        .unwrap()
        .has_summoning_sickness = false;

    let result = mtg_engine::process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: reflection_id,
            ability_index: 0,
            targets: vec![Target::Object(target_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
        },
    );
    assert!(
        result.is_ok(),
        "Reflection of Kiki-Jiki's back-face activated ability should be reachable \
         post-transform; got {:?}",
        result.err()
    );
}

// ── DECOY: front static removed / re-added on transform-there-and-back ──────

/// Front: Creature 2/2 with a self-only Static +1/+0. Back: Creature 2/2, no
/// abilities.
fn mock_static_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-os4b-static-dfc".to_string()),
        name: "Mock OS4b Static Front".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            AbilityDefinition::Static {
                continuous_effect: CardContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Mock OS4b Static Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "".to_string(),
            abilities: vec![],
            power: Some(2),
            toughness: Some(2),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

fn mock_static_dfc_on_battlefield(owner: PlayerId) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, "Mock OS4b Static Front")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-os4b-static-dfc".to_string()))
        .with_types(vec![CardType::Creature]);
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

/// CR 613 / 712.8d/e decoy: the front face's Static +1/+0 anthem must be
/// deregistered when the permanent transforms away from that face (pins
/// `deregister_face_statics` -- a miss here leaves the exact C1-class front
/// static leak this PB fixes).
#[test]
fn test_front_static_removed_on_transform() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_static_dfc_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(mock_static_dfc_on_battlefield(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let id = find_by_name(&state, "Mock OS4b Static Front");
    let card_id = state.objects()[&id].card_id.clone();
    // GameStateBuilder does not replay ETB -- register the front face's static
    // manually, matching what a real ETB would have done.
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    let before = calculate_characteristics(&state, id).unwrap();
    assert_eq!(
        before.power,
        Some(3),
        "sanity: front static should apply (2 base + 1)"
    );

    let mut ctx = EffectContext::new(p1, id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);

    let after = calculate_characteristics(&state, id).unwrap();
    assert_eq!(
        after.power,
        Some(2),
        "the front's +1/+0 static must be deregistered once transformed away from it"
    );
}

/// CR 712.8d/e decoy: transforming there and back restores the front's ability
/// set exactly (pins bidirectional deregister/register + index stability).
#[test]
fn test_transform_there_and_back_restores_front_ability_set() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_static_dfc_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(mock_static_dfc_on_battlefield(p1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let id = find_by_name(&state, "Mock OS4b Static Front");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    let mut ctx = EffectContext::new(p1, id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
    assert!(state.objects()[&id].is_transformed);
    assert_eq!(
        calculate_characteristics(&state, id).unwrap().power,
        Some(2)
    );

    let mut ctx2 = EffectContext::new(p1, id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx2);
    assert!(
        !state.objects()[&id].is_transformed,
        "should be back to the front face"
    );
    assert_eq!(
        calculate_characteristics(&state, id).unwrap().power,
        Some(3),
        "the front's static must be re-registered after transforming back"
    );
}

// ── DECOY: back-only upkeep trigger fires only when transformed ─────────────

/// Front: Creature, no upkeep trigger. Back: Creature,
/// `AtBeginningOfYourUpkeep -> gain 1 life`.
fn mock_upkeep_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-os4b-upkeep-dfc".to_string()),
        name: "Mock OS4b Upkeep Front".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Transform)],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Mock OS4b Upkeep Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "".to_string(),
            abilities: vec![AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::GainLife {
                    player: mtg_engine::PlayerTarget::Controller,
                    amount: mtg_engine::EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            }],
            power: Some(2),
            toughness: Some(2),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

fn build_upkeep_decoy_state(pretransform: bool) -> (GameState, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_upkeep_dfc_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock OS4b Upkeep Front")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("mock-os4b-upkeep-dfc".to_string()))
                .with_types(vec![CardType::Creature]),
        )
        .active_player(p1)
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let id = find_by_name(&state, "Mock OS4b Upkeep Front");
    if !pretransform {
        let mut ctx = EffectContext::new(p1, id, vec![]);
        let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
        assert!(state.objects()[&id].is_transformed);
    }
    (state, id)
}

/// CR 712.8d/e decoy: while showing the FRONT face, no upkeep trigger fires --
/// the front declares none.
#[test]
fn test_front_upkeep_no_trigger() {
    let (state, _) = build_upkeep_decoy_state(true);
    let p1 = p(1);
    let life_before = state.players()[&p1].life_total;
    let state = advance_to_step(state, Step::Upkeep);
    let state = resolve_stack(state, &[p1, p(2)]);
    assert_eq!(
        state.players()[&p1].life_total,
        life_before,
        "front face has no upkeep trigger -- life must not change"
    );
}

/// CR 712.8d/e decoy: once transformed, the BACK face's upkeep trigger fires
/// (pins the `turn_actions.rs` upkeep sweep producer + `abilities.rs`
/// CardDefETB consumer index parity, "is_transformed at consume time").
#[test]
fn test_back_upkeep_trigger_fires_only_when_transformed() {
    let (state, _) = build_upkeep_decoy_state(false);
    let p1 = p(1);
    let life_before = state.players()[&p1].life_total;
    let state = advance_to_step(state, Step::Upkeep);
    let state = resolve_stack(state, &[p1, p(2)]);
    assert_eq!(
        state.players()[&p1].life_total,
        life_before + 1,
        "back face's upkeep trigger should gain 1 life once transformed"
    );
}

// ── DECOY: transformed Saga with no back-face SagaChapter is not sacrificed ──

/// Front: Enchantment Saga with a chapter III ability. Back: Creature, no
/// SagaChapter abilities.
fn mock_saga_transform_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-os4b-saga-transform".to_string()),
        name: "Mock OS4b Saga Front".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            subtypes: [SubType("Saga".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Transform),
            AbilityDefinition::SagaChapter {
                chapter: 3,
                effect: Effect::Nothing,
                targets: vec![],
            },
        ],
        power: None,
        toughness: None,
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Mock OS4b Saga Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "".to_string(),
            abilities: vec![],
            power: Some(3),
            toughness: Some(3),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

/// CR 714.4 / 712.8d/e decoy: an UNtransformed Saga at 3+ lore counters (==
/// final chapter) IS sacrificed by the SBA (baseline sanity for the next test).
#[test]
fn test_saga_untransformed_is_sacrificed() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_saga_transform_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock OS4b Saga Front")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("mock-os4b-saga-transform".to_string()))
                .with_types(vec![CardType::Enchantment])
                .with_counter(CounterType::Lore, 3),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let saga_id = find_by_name(&state, "Mock OS4b Saga Front");
    let _ = check_and_apply_sbas(&mut state);
    assert!(
        !state.objects().contains_key(&saga_id),
        "decoy: an untransformed Saga at 3+ lore IS sacrificed by CR 714.4"
    );
}

/// CR 714.4 / 712.8d/e: a Saga that transforms in place to a back face with no
/// `SagaChapter` abilities is no longer a Saga and must NOT be sacrificed, even
/// though its lore counter count is still >= the (front's) final chapter.
#[test]
fn test_saga_transform_not_sacrificed() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = registry_with(vec![mock_saga_transform_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock OS4b Saga Front")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("mock-os4b-saga-transform".to_string()))
                .with_types(vec![CardType::Enchantment])
                .with_counter(CounterType::Lore, 3),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let saga_id = find_by_name(&state, "Mock OS4b Saga Front");
    let mut ctx = EffectContext::new(p1, saga_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
    assert!(state.objects()[&saga_id].is_transformed);

    let sba_events = check_and_apply_sbas(&mut state);
    assert!(
        state.objects().contains_key(&saga_id),
        "CR 712.8d/e / 714.4: a transformed Saga with no back-face SagaChapter \
         abilities must NOT be sacrificed"
    );
    assert!(!sba_events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { object_id, .. } if *object_id == saga_id)));
}

// ── NEGATIVE: non-DFC transform is a no-op ability set ──────────────────────

fn mock_nondfc_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-os4b-nondfc".to_string()),
        name: "Mock OS4b Non-DFC".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: mtg_engine::Cost::Tap,
            effect: Effect::GainLife {
                player: mtg_engine::PlayerTarget::Controller,
                amount: mtg_engine::EffectAmount::Fixed(1),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: None,
        ..Default::default()
    }
}

/// CR 712.8d/e negative: a non-DFC's ability set is untouched by
/// `Effect::TransformSelf` -- `is_transformed` never flips and its
/// mana/activated/triggered vectors are unchanged.
#[test]
fn test_non_dfc_transform_is_noop_ability_set() {
    let p1 = p(1);
    let p2 = p(2);
    let defs_map_local: HashMap<String, CardDefinition> =
        vec![("Mock OS4b Non-DFC".to_string(), mock_nondfc_def())]
            .into_iter()
            .collect();
    let registry = registry_with(vec![mock_nondfc_def()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Mock OS4b Non-DFC",
            ZoneId::Battlefield,
            &defs_map_local,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let id = find_by_name(&state, "Mock OS4b Non-DFC");
    let activated_before = state.objects()[&id]
        .characteristics
        .activated_abilities
        .len();
    let mana_before = state.objects()[&id].characteristics.mana_abilities.len();
    let triggered_before = state.objects()[&id]
        .characteristics
        .triggered_abilities
        .len();

    let mut ctx = EffectContext::new(p1, id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);

    assert!(
        !state.objects()[&id].is_transformed,
        "a non-DFC must never become is_transformed"
    );
    assert_eq!(
        state.objects()[&id]
            .characteristics
            .activated_abilities
            .len(),
        activated_before
    );
    assert_eq!(
        state.objects()[&id].characteristics.mana_abilities.len(),
        mana_before
    );
    assert_eq!(
        state.objects()[&id]
            .characteristics
            .triggered_abilities
            .len(),
        triggered_before
    );
}

// ── NEGATIVE: off-battlefield DFC uses front-face abilities ─────────────────

/// CR 400.7 / 712.8a negative: a DFC that has never been on the battlefield
/// (`is_transformed == false`, the constructed default) reports the front
/// face's abilities via the Channel-A vectors built at construction time --
/// exactly what `effective_abilities(false)` returns.
#[test]
fn test_offbattlefield_uses_front_abilities() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mock_static_dfc_def();
    let defs_map_local: HashMap<String, CardDefinition> =
        vec![(def.name.clone(), def.clone())].into_iter().collect();
    let registry = registry_with(vec![def.clone()]);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Mock OS4b Static Front",
            ZoneId::Hand(p1),
            &defs_map_local,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let id = find_by_name(&state, "Mock OS4b Static Front");
    assert!(
        !state.objects()[&id].is_transformed,
        "off-battlefield default is front-face"
    );
    assert!(
        find_in_zone(&state, "Mock OS4b Static Front", &ZoneId::Hand(p1)).is_some(),
        "sanity: the object should be in hand"
    );
    // Fixture sanity: front and back ability lists genuinely differ, so this
    // negative test means something (front has the Static, back has none).
    assert_ne!(
        def.effective_abilities(true).len(),
        def.effective_abilities(false).len()
    );
    assert_eq!(def.effective_abilities(false).len(), def.abilities.len());
}
