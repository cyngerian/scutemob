//! PB-EF5 (scutemob-106): `Effect::TransformSelf` -- a unit `Effect` variant that flips
//! the resolving ability's own source (`ctx.source`) double-faced permanent to its other
//! face in place (CR 701.27a/f, 712.18), reusing the existing `Command::Transform` /
//! `transform_permanent_in_place` machinery rather than forking it.
//!
//! Key rules verified:
//! - CR 701.27a/712.18: `TransformSelf` flips `ctx.source` in place -- same `ObjectId`.
//! - CR 701.27a: `TransformSelf` targets only `ctx.source`, never "some DFC" on the
//!   battlefield (decoy: a second controlled DFC must NOT flip).
//! - CR 701.27f / 701.28e: once-per-instruction -- a second `TransformSelf` within the
//!   same resolving instruction (e.g. a `Sequence`) is ignored.
//! - CR 701.27c: non-DFC source is a no-op.
//! - CR 701.27d / 712.10: a DFC whose back face is an instant/sorcery is a no-op.
//! - CR 702.145: a daybound/nightbound source is a no-op via `TransformSelf` (it may only
//!   transform through its own keyword enforcement system); `Command::Transform` on the
//!   same permanent still rejects with `Err` (unchanged from before the refactor).
//! - `Command::Transform` / `handle_transform` behavior is unchanged by the
//!   `transform_permanent_in_place` extraction.
//! - Card integration: `thaumatic_compass` (end-step intervening-if transform),
//!   `docent_of_perfection` (cast-trigger token-then-conditional-transform), and the
//!   `delver_of_secrets` integrity demote (PB-EF5 §6a).

use std::collections::HashMap;

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardFace, CardId, CardRegistry, CardType, Command, Completeness, Effect,
    GameEvent, GameState, GameStateBuilder, GameStateError, KeywordAbility, ManaColor, ObjectId,
    ObjectSpec, PlayerId, Step, SubType, Target, TypeLine, ZoneId,
};

// ── Generic helpers ───────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn count_wizards(state: &GameState, controller: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(_, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == controller
                && calculate_characteristics(state, obj.id)
                    .map(|c| c.subtypes.contains(&SubType("Wizard".to_string())))
                    .unwrap_or(false)
        })
        .count()
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

// ── Mock DFC definitions (engine-primitive tests) ────────────────────────────

/// Front: "Mock Front" 2/2 Creature. Back: "Mock Back" 4/4 Creature with Flying.
fn mock_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-transform-self-dfc".to_string()),
        name: "Mock Front".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Transform".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Transform)],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Mock Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "Flying".to_string(),
            abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Flying)],
            power: Some(4),
            toughness: Some(4),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

fn mock_dfc_on_battlefield(owner: PlayerId, name: &str) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-transform-self-dfc".to_string()))
        .with_types(vec![CardType::Creature]);
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

/// A DFC whose back face is an Instant (CR 701.27d: cannot transform to it).
fn mock_instant_back_dfc_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-instant-back-dfc".to_string()),
        name: "Mock Adventurer".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Transform".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Transform)],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Mock Spell".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Instant].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "".to_string(),
            abilities: vec![],
            power: None,
            toughness: None,
            color_indicator: None,
        }),
        ..Default::default()
    }
}

/// A mock Daybound card, modeled on `mechanics_a_d/daybound.rs`'s Brutal Cathar.
fn mock_daybound_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-daybound-dfc".to_string()),
        name: "Mock Daybound Front".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Daybound".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Daybound)],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: "Mock Nightbound Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "Nightbound".to_string(),
            abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Nightbound)],
            power: Some(4),
            toughness: Some(4),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

/// A mock single-faced card -- cannot transform (CR 701.27c).
fn plain_creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-plain-creature".to_string()),
        name: "Mock Plain Creature".to_string(),
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
        back_face: None,
        ..Default::default()
    }
}

// ── 1: TransformSelf flips ctx.source ─────────────────────────────────────────

/// CR 701.27a/712.18: `Effect::TransformSelf` flips `ctx.source` front->back in place --
/// same `ObjectId`, `PermanentTransformed` emitted. Reverse: a fresh resolving instruction
/// flips it back->front.
#[test]
fn test_transform_self_flips_source() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_on_battlefield(p1, "Mock Front"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Mock Front");
    assert!(!state.objects()[&obj_id].is_transformed);

    let mut ctx = EffectContext::new(p1, obj_id, vec![]);
    let events = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);

    assert!(
        state.objects()[&obj_id].is_transformed,
        "TransformSelf should flip ctx.source"
    );
    assert_eq!(
        state.objects()[&obj_id].id,
        obj_id,
        "CR 712.18: same ObjectId after transform"
    );
    let chars = calculate_characteristics(&state, obj_id).expect("should have chars");
    assert_eq!(chars.name, "Mock Back");
    assert_eq!(chars.power, Some(4));
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::PermanentTransformed { object_id, to_back_face: true } if *object_id == obj_id
        )),
        "should emit PermanentTransformed(to_back_face: true)"
    );
    assert!(
        ctx.source_transformed_this_resolution,
        "guard should latch after a real flip"
    );

    // Reverse: a NEW resolving instruction (fresh ctx) flips back->front.
    let mut ctx2 = EffectContext::new(p1, obj_id, vec![]);
    let events2 = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx2);
    assert!(
        !state.objects()[&obj_id].is_transformed,
        "second TransformSelf (fresh instruction) should flip back to front"
    );
    assert!(events2.iter().any(|e| matches!(
        e,
        GameEvent::PermanentTransformed { object_id, to_back_face: false } if *object_id == obj_id
    )));
}

// ── 2: TransformSelf targets only ctx.source (decoy) ──────────────────────────

/// CR 701.27a: `TransformSelf` flips ONLY `ctx.source`. A second controlled DFC on the
/// battlefield must not be affected -- proves the effect targets the source, not "a DFC".
#[test]
fn test_transform_self_does_not_flip_a_second_dfc() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_on_battlefield(p1, "Mock Front A"))
        .object(mock_dfc_on_battlefield(p1, "Mock Front B"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Mock Front A");
    let decoy_id = find_object(&state, "Mock Front B");

    let mut ctx = EffectContext::new(p1, source_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);

    assert!(
        state.objects()[&source_id].is_transformed,
        "ctx.source should transform"
    );
    assert!(
        !state.objects()[&decoy_id].is_transformed,
        "a different controlled DFC must NOT transform (TransformSelf targets ctx.source only)"
    );
}

// ── 3: once-per-instruction guard ─────────────────────────────────────────────

/// CR 701.27f/701.28e: a `Sequence[TransformSelf, TransformSelf]` in ONE resolving
/// instruction transforms the source only once -- ends on the back face, not flipped
/// twice back to front. Decoy: reverting the guard latch would flip twice.
#[test]
fn test_transform_self_once_per_instruction() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![mock_dfc_def()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_on_battlefield(p1, "Mock Front"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Mock Front");

    let mut ctx = EffectContext::new(p1, obj_id, vec![]);
    let effect = Effect::Sequence(vec![Effect::TransformSelf, Effect::TransformSelf]);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        state.objects()[&obj_id].is_transformed,
        "a doubled TransformSelf in one instruction should still end on the back face \
         (CR 701.27f) -- if the guard were not latching, this would be false again"
    );
    let flip_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentTransformed { .. }))
        .count();
    assert_eq!(
        flip_count, 1,
        "only ONE PermanentTransformed event should be emitted, not two"
    );
}

// ── 4: non-DFC no-op ───────────────────────────────────────────────────────────

/// CR 701.27c: `TransformSelf` on a non-DFC source does nothing -- no event, guard not
/// latched.
#[test]
fn test_transform_self_non_dfc_noop() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![plain_creature_def()]);

    let mut plain_spec = ObjectSpec::card(p1, "Mock Plain Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-plain-creature".to_string()))
        .with_types(vec![CardType::Creature]);
    plain_spec.power = Some(2);
    plain_spec.toughness = Some(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(plain_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Mock Plain Creature");
    let mut ctx = EffectContext::new(p1, obj_id, vec![]);
    let events = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);

    assert!(!state.objects()[&obj_id].is_transformed);
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentTransformed { .. })));
    assert!(
        !ctx.source_transformed_this_resolution,
        "guard must not latch on a no-op"
    );
}

// ── 5: instant/sorcery back face no-op ────────────────────────────────────────

/// CR 701.27d/712.10: a DFC whose back face is an instant/sorcery cannot transform.
#[test]
fn test_transform_self_instant_sorcery_back_noop() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![mock_instant_back_dfc_def()]);

    let spec = {
        let mut s = ObjectSpec::card(p1, "Mock Adventurer")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(CardId("mock-instant-back-dfc".to_string()))
            .with_types(vec![CardType::Creature]);
        s.power = Some(2);
        s.toughness = Some(2);
        s
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Mock Adventurer");
    let mut ctx = EffectContext::new(p1, obj_id, vec![]);
    let events = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);

    assert!(
        !state.objects()[&obj_id].is_transformed,
        "cannot transform into an instant/sorcery back face (CR 701.27d)"
    );
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentTransformed { .. })));
}

// ── 6: daybound no-op (TransformSelf) + Command::Transform still Errs ─────────

/// CR 702.145: a daybound/nightbound permanent cannot transform via `TransformSelf` --
/// it only flips through its own keyword enforcement system. `Command::Transform` on the
/// same permanent still rejects with `Err` (unchanged).
#[test]
fn test_transform_self_daybound_noop() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![mock_daybound_def()]);

    let spec = {
        let mut s = ObjectSpec::card(p1, "Mock Daybound Front")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(CardId("mock-daybound-dfc".to_string()))
            .with_types(vec![CardType::Creature])
            .with_keyword(KeywordAbility::Daybound);
        s.power = Some(2);
        s.toughness = Some(2);
        s
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Mock Daybound Front");

    let mut ctx = EffectContext::new(p1, obj_id, vec![]);
    let events = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
    assert!(
        !state.objects()[&obj_id].is_transformed,
        "TransformSelf must no-op on a daybound source (CR 702.145)"
    );
    assert!(!events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentTransformed { .. })));

    // Command::Transform on the same permanent still Errs (unchanged behavior).
    let result = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: obj_id,
        },
    );
    assert!(
        result.is_err(),
        "Command::Transform must still reject a daybound permanent"
    );
}

// ── 7: Command::Transform behavior is unchanged by the refactor ──────────────

/// Guards the `transform_permanent_in_place` extraction: `Command::Transform` still
/// flips a controlled battlefield DFC, and still `Err`s on a non-controlled permanent,
/// an off-battlefield permanent, and a daybound permanent.
#[test]
fn test_command_transform_unchanged() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![mock_dfc_def(), mock_daybound_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_dfc_on_battlefield(p1, "Mock Front"))
        .object({
            let mut s = ObjectSpec::card(p2, "Opponent's DFC")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("mock-transform-self-dfc".to_string()))
                .with_types(vec![CardType::Creature]);
            s.power = Some(2);
            s.toughness = Some(2);
            s
        })
        .object({
            let mut s = ObjectSpec::card(p1, "Graveyard DFC")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("mock-transform-self-dfc".to_string()))
                .with_types(vec![CardType::Creature]);
            s.power = Some(2);
            s.toughness = Some(2);
            s
        })
        .object({
            let mut s = ObjectSpec::card(p1, "Mock Daybound Front")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("mock-daybound-dfc".to_string()))
                .with_types(vec![CardType::Creature])
                .with_keyword(KeywordAbility::Daybound);
            s.power = Some(2);
            s.toughness = Some(2);
            s
        })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let controlled_id = find_object(&state, "Mock Front");
    let opponents_id = find_object(&state, "Opponent's DFC");
    let graveyard_id = find_object(&state, "Graveyard DFC");
    let daybound_id = find_object(&state, "Mock Daybound Front");

    // Success case: controlled battlefield DFC flips.
    let (state, events) = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: controlled_id,
        },
    )
    .expect("Transform of a controlled battlefield DFC should succeed");
    assert!(state.objects()[&controlled_id].is_transformed);
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentTransformed { .. })));

    // Non-controlled: Err.
    let result = process_command(
        state,
        Command::Transform {
            player: p1,
            permanent: opponents_id,
        },
    );
    assert!(matches!(result, Err(GameStateError::InvalidCommand(_))));

    // Rebuild state for the remaining two Err cases (process_command consumed it above).
    let registry2 = CardRegistry::new(vec![mock_dfc_def()]);
    let state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry2)
        .object({
            let mut s = ObjectSpec::card(p1, "Graveyard DFC")
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(CardId("mock-transform-self-dfc".to_string()))
                .with_types(vec![CardType::Creature]);
            s.power = Some(2);
            s.toughness = Some(2);
            s
        })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let _ = graveyard_id; // silence unused-in-this-branch warning across rebuild
    let graveyard_id2 = find_object(&state2, "Graveyard DFC");
    let result = process_command(
        state2,
        Command::Transform {
            player: p1,
            permanent: graveyard_id2,
        },
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(_))),
        "off-battlefield permanent must Err"
    );

    let _ = daybound_id; // covered by test_transform_self_daybound_noop's Err assertion too
}

// ── Card integration: thaumatic_compass ───────────────────────────────────────

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

fn land_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Land])
}

/// CR 714/604.2 (intervening-if) + 701.27: with 7+ lands, Thaumatic Compass transforms
/// at the beginning of its controller's end step.
#[test]
fn test_thaumatic_compass_transforms_at_end_step_with_7_lands() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
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
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for i in 0..7 {
        builder = builder.object(land_spec(p1, &format!("Land {}", i)));
    }
    let state = builder.build().unwrap();

    let compass_id = find_object(&state, "Thaumatic Compass");
    assert!(!state.objects()[&compass_id].is_transformed);

    let state = advance_to_step(state, Step::End);
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        state.objects()[&compass_id].is_transformed,
        "with 7+ lands, Thaumatic Compass should transform at the beginning of the end step"
    );
    let chars = calculate_characteristics(&state, compass_id).expect("should have chars");
    assert_eq!(chars.name, "Spires of Orazca");
}

/// Decoy: with only 6 lands (below the threshold), Thaumatic Compass does NOT transform.
#[test]
fn test_thaumatic_compass_stays_untransformed_with_6_lands() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
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
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for i in 0..6 {
        builder = builder.object(land_spec(p1, &format!("Land {}", i)));
    }
    let state = builder.build().unwrap();

    let compass_id = find_object(&state, "Thaumatic Compass");

    let state = advance_to_step(state, Step::End);
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        !state.objects()[&compass_id].is_transformed,
        "with only 6 lands, Thaumatic Compass must NOT transform"
    );
}

// ── Card integration: docent_of_perfection ────────────────────────────────────

fn wizard_creature_spec(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::creature(owner, name, 1, 1).with_subtypes(vec![SubType("Wizard".to_string())])
}

fn cast_lightning_bolt(state: GameState, caster: PlayerId, target: PlayerId) -> GameState {
    let defs = defs_map();
    let spell_id = find_object(&state, "Lightning Bolt");
    let _ = &defs; // Lightning Bolt is already in the registry via all_cards()
    state.players().get(&caster).expect("caster exists");
    let mut state = state;
    if let Some(ps) = state.players_mut().get_mut(&caster) {
        ps.mana_pool.add(ManaColor::Red, 1);
    }
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: caster,
            card: spell_id,
            targets: vec![Target::Player(target)],
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
    .expect("casting Lightning Bolt should succeed");
    state
}

/// CR 603 + 701.27f: casting an instant/sorcery with Docent of Perfection and 2 other
/// Wizards on the battlefield creates a 3rd Wizard token, hitting the "3 or more Wizards"
/// threshold, so Docent transforms (Sequence order: token created BEFORE the check).
#[test]
fn test_docent_of_perfection_transforms_on_third_wizard() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Docent of Perfection",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(wizard_creature_spec(p1, "Wizard One"))
        .object(wizard_creature_spec(p1, "Wizard Two"))
        .object(real_card_spec(
            p1,
            "Lightning Bolt",
            ZoneId::Hand(p1),
            &defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let docent_id = find_object(&state, "Docent of Perfection");
    assert!(!state.objects()[&docent_id].is_transformed);
    assert_eq!(count_wizards(&state, p1), 2, "should start with 2 Wizards");

    let state = cast_lightning_bolt(state, p1, p2);
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        count_wizards(&state, p1),
        3,
        "casting the instant should create a 3rd Wizard token"
    );
    assert!(
        state.objects()[&docent_id].is_transformed,
        "with 3+ Wizards, Docent of Perfection should transform"
    );
    let chars = calculate_characteristics(&state, docent_id).expect("should have chars");
    assert_eq!(chars.name, "Final Iteration");
}

/// Decoy: with only 1 existing Wizard, casting the instant creates a 2nd Wizard token
/// (still short of 3), so Docent does NOT transform.
#[test]
fn test_docent_of_perfection_stays_untransformed_below_threshold() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = defs_map();
    let registry = CardRegistry::new(all_cards());

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(real_card_spec(
            p1,
            "Docent of Perfection",
            ZoneId::Battlefield,
            &defs,
        ))
        .object(wizard_creature_spec(p1, "Wizard One"))
        .object(real_card_spec(
            p1,
            "Lightning Bolt",
            ZoneId::Hand(p1),
            &defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let docent_id = find_object(&state, "Docent of Perfection");

    let state = cast_lightning_bolt(state, p1, p2);
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        count_wizards(&state, p1),
        2,
        "casting the instant should create only a 2nd Wizard token"
    );
    assert!(
        !state.objects()[&docent_id].is_transformed,
        "with only 2 Wizards, Docent of Perfection must NOT transform"
    );
}

// ── Integrity regression: delver_of_secrets is marked partial ────────────────

/// Integrity regression guard for PB-EF5 §6a: `delver_of_secrets` was mismarked
/// `Complete` despite never actually transforming (no upkeep trigger modeled). It must
/// stay non-`Complete` until the "top card is instant/sorcery" reveal condition exists.
#[test]
fn test_delver_of_secrets_marked_partial() {
    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Delver of Secrets")
        .expect("Delver of Secrets should have a CardDefinition");
    assert!(
        !def.completeness.is_complete(),
        "delver_of_secrets must NOT be Complete -- its upkeep transform trigger is unmodeled"
    );
    assert!(
        matches!(def.completeness, Completeness::Partial(_)),
        "delver_of_secrets should be marked Partial specifically (not KnownWrong/Inert)"
    );
}
