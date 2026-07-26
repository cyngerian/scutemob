//! PB-RS4 (scutemob-146, OOS-RS-3 / OOS-OS4-2 residuals): face-aware residuals
//! closing the 3 surviving CR 712.8d/e deviations left by PB-OS4b.
//!
//! Three gathering/deregistration sites still read the FRONT face unconditionally:
//!
//! 1. `apply_self_etb_from_definition` (`replacement.rs`) — self-ETB replacements
//!    (enters tapped / enters with counters) gathered from the front face even when
//!    the permanent enters back-face-up (disturb, stack craft).
//! 2. `register_permanent_replacement_abilities` (`replacement.rs`) — non-self
//!    permanent replacement abilities, same front-only defect.
//! 3. `deregister_face_statics` (`face.rs`) — only removed `AbilityDefinition::Static`
//!    on transform; nine other families registered by
//!    `register_static_continuous_effects` were never deregistered, leaking the old
//!    face's TriggerDoubling/SuppressCreatureETBTriggers/StaticRestriction/
//!    CdaPowerToughness/CdaModifyPowerToughness/AdditionalLandPlays/
//!    StaticFlashGrant/StaticPlayFromGraveyard/StaticPlayFromTop.
//!
//! Plus a deviation found during planning (deviation #4, same root cause):
//!
//! 4. `turn_actions.rs`'s CR 714.3b precombat-main Saga sweep and
//!    `fire_saga_chapter_triggers`'s ability-index producer both read
//!    `def.abilities` (front) instead of `def.effective_abilities(is_transformed)`,
//!    disagreeing with the SBA guard (`sba.rs:843`/`:889`) which was already fixed
//!    in PB-OS4b.
//!
//! These tests are written FIRST and verified RED against pre-fix HEAD (AC 5458).
//! See `memory/primitives/pb-plan-RS4.md` §7 for the full test design and
//! `memory/primitive-wip.md` for the recorded fail-before messages.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::rules::replacement::{
    fire_saga_chapter_triggers, register_static_continuous_effects,
};
use mtg_engine::state::stubs::{ActiveRestriction, AdditionalLandPlaySource};
use mtg_engine::{
    all_cards, calculate_characteristics, AbilityDefinition, AltCostKind, CardDefinition, CardFace,
    CardId, CardRegistry, CardType, Command, CounterType, Effect, EffectAmount, FlashGrantFilter,
    GameEvent, GameRestriction, GameState, GameStateBuilder, KeywordAbility, ManaColor, ManaCost,
    ObjectFilter, ObjectId, ObjectSpec, PlayFromTopFilter, PlayerFilter, PlayerId, PlayerTarget,
    ReplacementModification, ReplacementTrigger, Step, SubType, TriggerCondition,
    TriggerDoublerFilter, TypeLine, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ──────────────────────────────────────────────────────────────────

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

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
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
    mtg_engine::enrich_spec_from_def(base, defs)
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = mtg_engine::process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
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

fn empty_cast_spell_disturb(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
        alt_cost: Some(AltCostKind::Disturb),
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        face_down_kind: None,
        additional_costs: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }))
}

// ── Deviation #1/#2 fixtures: a disturb DFC with a configurable back/front ability ──

/// A mock Disturb DFC. Front: {W} 1/1 Human, Disturb {1}{W}, plus `front_extra`.
/// Back: 3/2 White Spirit Flying, plus `back_extra`.
fn disturb_dfc_def(
    card_id: &str,
    front_name: &str,
    back_name: &str,
    front_extra: Vec<AbilityDefinition>,
    back_extra: Vec<AbilityDefinition>,
) -> CardDefinition {
    let mut front_abilities = vec![
        AbilityDefinition::Keyword(KeywordAbility::Disturb),
        AbilityDefinition::Disturb {
            cost: ManaCost {
                white: 1,
                generic: 1,
                ..Default::default()
            },
        },
    ];
    front_abilities.extend(front_extra);
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: front_name.to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Human".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: front_abilities,
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: Some(CardFace {
            name: back_name.to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature, CardType::Enchantment]
                    .into_iter()
                    .collect(),
                subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "".to_string(),
            abilities: back_extra,
            power: Some(3),
            toughness: Some(2),
            color_indicator: Some(vec![mtg_engine::Color::White]),
        }),
        ..Default::default()
    }
}

fn disturb_card_in_graveyard(owner: PlayerId, def: &CardDefinition) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, &def.name)
        .in_zone(ZoneId::Graveyard(owner))
        .with_card_id(def.card_id.clone())
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Disturb)
        .with_mana_cost(ManaCost {
            white: 1,
            ..Default::default()
        });
    spec.power = Some(1);
    spec.toughness = Some(1);
    spec
}

/// Cast `def` with disturb from the graveyard and resolve the stack. Returns the
/// resulting state and the entered (back-face) permanent's ObjectId.
fn cast_disturb_and_resolve(def: CardDefinition) -> (GameState, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(disturb_card_in_graveyard(p1, &def))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let card_id = def.card_id.clone();
    let beggar_id = find_in_zone(&state, &def.name, ZoneId::Graveyard(p1))
        .expect("disturb card should be in graveyard");

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);

    let (state, _) = mtg_engine::process_command(state, empty_cast_spell_disturb(p1, beggar_id))
        .unwrap_or_else(|e| panic!("cast with disturb should succeed: {:?}", e));
    let (state, _) = pass_all(state, &[p1, p2]);

    let entered_id = state
        .objects()
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.card_id == Some(card_id.clone())
                && obj.is_transformed
        })
        .map(|(id, _)| *id)
        .expect("back face should be on the battlefield");
    (state, entered_id)
}

// ── Test 1/2: apply_self_etb_from_definition face-awareness ────────────────

/// CR 614.1c / 614.12 / 712.8e: a disturb DFC whose BACK face declares a self-ETB
/// "enters tapped" replacement (front declares none) must enter TAPPED — the
/// self-ETB gathering loop must read the visible (back) face, not the front.
#[test]
fn test_disturb_back_face_self_etb_replacement_applies() {
    let def = disturb_dfc_def(
        "mock-rs4-back-enters-tapped",
        "Mock RS4 Front A",
        "Mock RS4 Back A",
        vec![],
        vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldEnterBattlefield {
                filter: ObjectFilter::Any,
            },
            modification: ReplacementModification::EntersTapped,
            is_self: true,
            unless_condition: None,
        }],
    );
    let (state, entered_id) = cast_disturb_and_resolve(def);
    assert!(
        state.objects()[&entered_id].status.tapped,
        "back face's self-ETB 'enters tapped' replacement must be gathered and \
         applied when the permanent enters back-face-up (CR 614.12 / 712.8e)"
    );
}

/// CR 712.8e: a disturb DFC whose FRONT face declares a self-ETB "enters with 2
/// +1/+1 counters" replacement (back declares none) must NOT apply that
/// replacement once entering back-face-up — the front face's characteristics
/// (including its self-ETB replacements) are not the ones "actually showing."
#[test]
fn test_disturb_front_face_self_etb_replacement_does_not_apply() {
    let def = disturb_dfc_def(
        "mock-rs4-front-enters-with-counters",
        "Mock RS4 Front B",
        "Mock RS4 Back B",
        vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldEnterBattlefield {
                filter: ObjectFilter::Any,
            },
            modification: ReplacementModification::EntersWithCounters {
                counter: CounterType::PlusOnePlusOne,
                count: Box::new(EffectAmount::Fixed(2)),
            },
            is_self: true,
            unless_condition: None,
        }],
        vec![],
    );
    let (state, entered_id) = cast_disturb_and_resolve(def);
    let counters = state.objects()[&entered_id]
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 0,
        "the FRONT face's 'enters with counters' self-ETB replacement must NOT \
         apply once the permanent enters back-face-up (CR 712.8e); got {} counters",
        counters
    );
}

// ── Test 3/4: register_permanent_replacement_abilities face-awareness ──────

/// CR 614 / 712.8e: a disturb DFC whose BACK face declares a non-self permanent
/// replacement ability (a counter-doubler) must have that replacement REGISTERED
/// into `state.replacement_effects` once it enters back-face-up.
#[test]
fn test_disturb_back_face_permanent_replacement_is_registered() {
    let def = disturb_dfc_def(
        "mock-rs4-back-permanent-replacement",
        "Mock RS4 Front C",
        "Mock RS4 Back C",
        vec![],
        vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldPlaceCounters {
                placer_filter: PlayerFilter::Any,
                receiver_filter: ObjectFilter::Any,
                counter_filter: None,
            },
            modification: ReplacementModification::DoubleCounters,
            is_self: false,
            unless_condition: None,
        }],
    );
    let (state, entered_id) = cast_disturb_and_resolve(def);
    let count = state
        .replacement_effects()
        .iter()
        .filter(|r| r.source == Some(entered_id))
        .count();
    assert_eq!(
        count, 1,
        "the back face's non-self permanent replacement ability must be \
         registered (CR 614, 712.8e); found {} matching entries",
        count
    );
}

/// CR 712.8e: a disturb DFC whose FRONT face declares a non-self permanent
/// replacement ability (back declares none) must NOT register that replacement
/// once entering back-face-up.
#[test]
fn test_disturb_front_face_permanent_replacement_is_not_registered() {
    let def = disturb_dfc_def(
        "mock-rs4-front-permanent-replacement",
        "Mock RS4 Front D",
        "Mock RS4 Back D",
        vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldPlaceCounters {
                placer_filter: PlayerFilter::Any,
                receiver_filter: ObjectFilter::Any,
                counter_filter: None,
            },
            modification: ReplacementModification::DoubleCounters,
            is_self: false,
            unless_condition: None,
        }],
        vec![],
    );
    let (state, entered_id) = cast_disturb_and_resolve(def);
    let count = state
        .replacement_effects()
        .iter()
        .filter(|r| r.source == Some(entered_id))
        .count();
    assert_eq!(
        count, 0,
        "the FRONT face's non-self permanent replacement ability must NOT be \
         registered once the permanent enters back-face-up (CR 712.8e); found {} \
         matching entries",
        count
    );
}

// ── Test 5: CR 714.3b precombat-main Saga sweep face-awareness (deviation #4.1) ──

/// CR 714.3b / 712.8e: `Fable of the Mirror-Breaker`, once transformed to
/// `Reflection of Kiki-Jiki` (no SagaChapter abilities on the back face), must NOT
/// accrue another lore counter at its controller's precombat main phase, and must
/// NOT queue another chapter trigger. `rules/sba.rs:843` already agrees (fixed in
/// PB-OS4b); the `turn_actions.rs` sweep must agree too.
#[test]
fn test_transformed_saga_stops_accruing_lore_counters() {
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
        .object(ObjectSpec::card(p1, "Filler A").in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::card(p1, "Filler B").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let fable_id = find_by_name(&state, "Fable of the Mirror-Breaker");
    let mut ctx = EffectContext::new(p1, fable_id, vec![]);
    let _ = execute_effect(&mut state, &Effect::TransformSelf, &mut ctx);
    assert!(
        state.objects()[&fable_id].is_transformed,
        "sanity: fable should be transformed before advancing to precombat main"
    );

    let lore_before = state.objects()[&fable_id]
        .counters
        .get(&CounterType::Lore)
        .copied()
        .unwrap_or(0);

    let state = advance_to_step(state, Step::PreCombatMain);

    let lore_after = state.objects()[&fable_id]
        .counters
        .get(&CounterType::Lore)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        lore_after, lore_before,
        "a transformed Saga with no back-face SagaChapter abilities must not \
         accrue another lore counter at precombat main (CR 714.3b / 712.8e)"
    );
    assert!(
        state.stack_objects().is_empty(),
        "no chapter trigger should have been queued for the transformed permanent"
    );
}

// ── Test 6: fire_saga_chapter_triggers producer/consumer index parity (deviation #4.2) ──

/// Front: 3 SagaChapter abilities (indices 1..3, index 0 is the Transform keyword).
/// Back: a single non-SagaChapter Triggered ability at index 0.
fn mock_saga_index_parity_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-rs4-saga-index-parity".to_string()),
        name: "Mock RS4 Saga Index Parity Front".to_string(),
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
                chapter: 1,
                effect: Effect::Nothing,
                targets: vec![],
            },
            AbilityDefinition::SagaChapter {
                chapter: 2,
                effect: Effect::Nothing,
                targets: vec![],
            },
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
            name: "Mock RS4 Saga Index Parity Back".to_string(),
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
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            }],
            power: Some(3),
            toughness: Some(3),
            color_indicator: None,
        }),
        ..Default::default()
    }
}

/// CR 714.2b / 712.8d/e: `fire_saga_chapter_triggers`'s producer must index into
/// the currently-visible face's *effective* ability list, matching the namespace
/// the three consumers (`resolution.rs:1996`/`:2028`, `sba.rs:889`) already use. A
/// transformed permanent whose back face has no `SagaChapter` abilities must
/// produce NO chapter trigger at all when the front-face lore-crossing math would
/// otherwise have fired one at front-index 1.
#[test]
fn test_saga_chapter_trigger_index_matches_effective_face() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mock_saga_index_parity_def();
    let registry = registry_with(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Mock RS4 Saga Index Parity Front")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("mock-rs4-saga-index-parity".to_string()))
                .with_types(vec![CardType::Enchantment]),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let saga_id = find_by_name(&state, "Mock RS4 Saga Index Parity Front");
    // Flip to the back face directly (bypassing TransformSelf's own preconditions --
    // this test isolates fire_saga_chapter_triggers, not the flip mechanism).
    if let Some(obj) = state.objects_mut().get_mut(&saga_id) {
        obj.is_transformed = true;
    }
    *state.pending_triggers_mut() = imbl::Vector::new();

    let _events = fire_saga_chapter_triggers(&mut state, saga_id, p1, 0, 1, &def);

    assert!(
        state.pending_triggers().is_empty(),
        "the back face has no SagaChapter abilities -- crossing the front's chapter-1 \
         threshold must not queue any trigger once transformed (CR 714.2b / 712.8d/e); \
         pending_triggers: {:?}",
        state.pending_triggers()
    );
}

// ── Tests 7-13: deregister_face_statics extended to all nine families ───────

/// A generic front-only-ability mock DFC: front carries Transform + `extra`,
/// back carries nothing. Used to probe each of the nine deregistration families
/// in isolation.
fn mock_family_def(
    card_id_str: &str,
    front_name: &str,
    extra: AbilityDefinition,
) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id_str.to_string()),
        name: front_name.to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Transform), extra],
        power: Some(2),
        toughness: Some(2),
        color_indicator: None,
        back_face: Some(CardFace {
            name: format!("{front_name} Back"),
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

fn mock_family_on_battlefield(owner: PlayerId, name: &str, card_id: &str) -> ObjectSpec {
    let mut spec = ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId(card_id.to_string()))
        .with_types(vec![CardType::Creature]);
    spec.power = Some(2);
    spec.toughness = Some(2);
    spec
}

fn build_family_state(
    registry: std::sync::Arc<CardRegistry>,
    p1: PlayerId,
    p2: PlayerId,
    name: &str,
    card_id: &str,
) -> GameState {
    GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mock_family_on_battlefield(p1, name, card_id))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap()
}

fn transform(state: &mut GameState, p1: PlayerId, id: ObjectId) {
    let mut ctx = EffectContext::new(p1, id, vec![]);
    let _ = execute_effect(state, &Effect::TransformSelf, &mut ctx);
}

/// CR 603.2d / 604.1: a Panharmonicon-style `TriggerDoubling` ability must be
/// deregistered once the permanent transforms away from the face that declared it.
#[test]
fn test_transform_deregisters_trigger_doubling() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::TriggerDoubling {
        filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
        additional_triggers: 1,
    };
    let def = mock_family_def(
        "mock-rs4-triggerdoubling",
        "Mock RS4 TriggerDoubling",
        ability,
    );
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 TriggerDoubling",
        "mock-rs4-triggerdoubling",
    );
    let id = find_by_name(&state, "Mock RS4 TriggerDoubling");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    assert!(
        state.trigger_doublers().iter().any(|d| d.source == id),
        "sanity: front TriggerDoubling should be registered"
    );

    transform(&mut state, p1, id);

    assert!(
        !state.trigger_doublers().iter().any(|d| d.source == id),
        "the front's TriggerDoubling must be deregistered once transformed away \
         from it (CR 603.2d / 604.1)"
    );
}

/// CR 614.16a: a Torpor Orb-style `SuppressCreatureETBTriggers` ability must be
/// deregistered once the permanent transforms away from the face that declared it.
#[test]
fn test_transform_deregisters_etb_suppressor() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::SuppressCreatureETBTriggers {
        filter: mtg_engine::ETBSuppressFilter::CreaturesOnly,
    };
    let def = mock_family_def("mock-rs4-etbsuppress", "Mock RS4 ETBSuppress", ability);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 ETBSuppress",
        "mock-rs4-etbsuppress",
    );
    let id = find_by_name(&state, "Mock RS4 ETBSuppress");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    assert!(
        state.etb_suppressors().iter().any(|s| s.source == id),
        "sanity: front SuppressCreatureETBTriggers should be registered"
    );

    transform(&mut state, p1, id);

    assert!(
        !state.etb_suppressors().iter().any(|s| s.source == id),
        "the front's SuppressCreatureETBTriggers must be deregistered once \
         transformed away from it (CR 614.16a)"
    );
}

/// CR 604.1: a `StaticRestriction` (Rule of Law-style) ability must be
/// deregistered once the permanent transforms away from the face that declared it.
#[test]
fn test_transform_deregisters_static_restriction() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::StaticRestriction {
        restriction: GameRestriction::MaxSpellsPerTurn { max: 1 },
    };
    let def = mock_family_def("mock-rs4-restriction", "Mock RS4 Restriction", ability);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 Restriction",
        "mock-rs4-restriction",
    );
    let id = find_by_name(&state, "Mock RS4 Restriction");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    assert!(
        state.restrictions().iter().any(|r| r.source == id),
        "sanity: front StaticRestriction should be registered"
    );

    transform(&mut state, p1, id);

    assert!(
        !state.restrictions().iter().any(|r| r.source == id),
        "the front's StaticRestriction must be deregistered once transformed away \
         from it (CR 604.1)"
    );
}

/// CR 604.3 / 613.4a: a `CdaPowerToughness` characteristic-defining ability must
/// be deregistered (observed through `calculate_characteristics`) once the
/// permanent transforms away from the face that declared it.
#[test]
fn test_transform_deregisters_cda_power_toughness() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::CdaPowerToughness {
        power: EffectAmount::Fixed(5),
        toughness: EffectAmount::Fixed(5),
    };
    let def = mock_family_def("mock-rs4-cda-pt", "Mock RS4 CdaPT", ability);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 CdaPT",
        "mock-rs4-cda-pt",
    );
    let id = find_by_name(&state, "Mock RS4 CdaPT");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    let before = calculate_characteristics(&state, id).unwrap();
    assert_eq!(
        before.power,
        Some(5),
        "sanity: front CDA should set power to 5"
    );
    assert_eq!(before.toughness, Some(5));

    transform(&mut state, p1, id);

    let after = calculate_characteristics(&state, id).unwrap();
    assert_eq!(
        after.power,
        Some(2),
        "the front's CdaPowerToughness must be deregistered once transformed away \
         from it (CR 604.3 / 613.4a); back face's printed power is 2"
    );
    assert_eq!(after.toughness, Some(2));
}

/// CR 604.3 / 613.4c: a `CdaModifyPowerToughness` ability with BOTH power and
/// toughness `Some` registers TWO continuous effects. Both must be deregistered
/// once the permanent transforms away from the declaring face (this is the
/// two-entry case the PB-OS4b doc comment specifically worried about).
#[test]
fn test_transform_deregisters_cda_modify_both_entries() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::CdaModifyPowerToughness {
        power: Some(EffectAmount::Fixed(3)),
        toughness: Some(EffectAmount::Fixed(2)),
    };
    let def = mock_family_def("mock-rs4-cda-modify", "Mock RS4 CdaModify", ability);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 CdaModify",
        "mock-rs4-cda-modify",
    );
    let id = find_by_name(&state, "Mock RS4 CdaModify");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    let before = calculate_characteristics(&state, id).unwrap();
    assert_eq!(
        before.power,
        Some(5),
        "sanity: 2 base + 3 modify = 5 power before transform"
    );
    assert_eq!(
        before.toughness,
        Some(4),
        "sanity: 2 base + 2 modify = 4 toughness before transform"
    );
    let is_cda_before = state
        .continuous_effects()
        .iter()
        .filter(|e| e.source == Some(id) && e.is_cda)
        .count();
    assert_eq!(is_cda_before, 2, "sanity: two CDA entries registered");

    transform(&mut state, p1, id);

    let after = calculate_characteristics(&state, id).unwrap();
    assert_eq!(
        after.power,
        Some(2),
        "both CdaModifyPowerToughness entries must be deregistered (power)"
    );
    assert_eq!(
        after.toughness,
        Some(2),
        "both CdaModifyPowerToughness entries must be deregistered (toughness)"
    );
    let is_cda_after = state
        .continuous_effects()
        .iter()
        .filter(|e| e.source == Some(id) && e.is_cda)
        .count();
    assert_eq!(
        is_cda_after, 0,
        "both CDA continuous-effect entries must be removed, not just one"
    );
}

/// CR 305.2: an `AdditionalLandPlays` ability must be deregistered once the
/// permanent transforms away from the face that declared it.
#[test]
fn test_transform_deregisters_additional_land_plays() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::AdditionalLandPlays { count: 1 };
    let def = mock_family_def("mock-rs4-landplays", "Mock RS4 LandPlays", ability);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 LandPlays",
        "mock-rs4-landplays",
    );
    let id = find_by_name(&state, "Mock RS4 LandPlays");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    assert!(
        state
            .additional_land_play_sources()
            .iter()
            .any(|s| s.source == id),
        "sanity: front AdditionalLandPlays should be registered"
    );

    transform(&mut state, p1, id);

    assert!(
        !state
            .additional_land_play_sources()
            .iter()
            .any(|s| s.source == id),
        "the front's AdditionalLandPlays must be deregistered once transformed \
         away from it (CR 305.2)"
    );
}

/// CR 601.3b: a `StaticFlashGrant` ability must be deregistered once the
/// permanent transforms away from the face that declared it.
#[test]
fn test_transform_deregisters_static_flash_grant() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::StaticFlashGrant {
        filter: FlashGrantFilter::AllSpells,
    };
    let def = mock_family_def("mock-rs4-flashgrant", "Mock RS4 FlashGrant", ability);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 FlashGrant",
        "mock-rs4-flashgrant",
    );
    let id = find_by_name(&state, "Mock RS4 FlashGrant");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    assert!(
        state.flash_grants().iter().any(|f| f.source == Some(id)),
        "sanity: front StaticFlashGrant should be registered"
    );

    transform(&mut state, p1, id);

    assert!(
        !state.flash_grants().iter().any(|f| f.source == Some(id)),
        "the front's StaticFlashGrant must be deregistered once transformed away \
         from it (CR 601.3b)"
    );
}

/// CR 601.3 / 305.1: a `StaticPlayFromGraveyard` permission must be deregistered
/// once the permanent transforms away from the face that declared it.
#[test]
fn test_transform_deregisters_play_from_graveyard() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::StaticPlayFromGraveyard {
        filter: PlayFromTopFilter::All,
        condition: None,
    };
    let def = mock_family_def("mock-rs4-pfg", "Mock RS4 PlayFromGraveyard", ability);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 PlayFromGraveyard",
        "mock-rs4-pfg",
    );
    let id = find_by_name(&state, "Mock RS4 PlayFromGraveyard");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    assert!(
        state
            .play_from_graveyard_permissions()
            .iter()
            .any(|pm| pm.source == id),
        "sanity: front StaticPlayFromGraveyard should be registered"
    );

    transform(&mut state, p1, id);

    assert!(
        !state
            .play_from_graveyard_permissions()
            .iter()
            .any(|pm| pm.source == id),
        "the front's StaticPlayFromGraveyard must be deregistered once \
         transformed away from it (CR 601.3 / 305.1)"
    );
}

/// CR 601.3: a `StaticPlayFromTop` permission must be deregistered once the
/// permanent transforms away from the face that declared it.
#[test]
fn test_transform_deregisters_play_from_top() {
    let p1 = p(1);
    let p2 = p(2);
    let ability = AbilityDefinition::StaticPlayFromTop {
        filter: PlayFromTopFilter::All,
        look_at_top: false,
        reveal_top: false,
        pay_life_instead: false,
        condition: None,
        on_cast_effect: None,
    };
    let def = mock_family_def("mock-rs4-pft", "Mock RS4 PlayFromTop", ability);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(
        registry.clone(),
        p1,
        p2,
        "Mock RS4 PlayFromTop",
        "mock-rs4-pft",
    );
    let id = find_by_name(&state, "Mock RS4 PlayFromTop");
    let card_id = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id.as_ref(), &registry, false);

    assert!(
        state
            .play_from_top_permissions()
            .iter()
            .any(|pm| pm.source == id),
        "sanity: front StaticPlayFromTop should be registered"
    );

    transform(&mut state, p1, id);

    assert!(
        !state
            .play_from_top_permissions()
            .iter()
            .any(|pm| pm.source == id),
        "the front's StaticPlayFromTop must be deregistered once transformed away \
         from it (CR 601.3)"
    );
}

/// CR 712.18: transforming there and back restores all nine families exactly --
/// not doubled (pre-fix: nothing is ever removed, so a round trip would leave
/// stale front-face entries alongside freshly re-registered ones).
#[test]
fn test_transform_there_and_back_restores_all_nine_families() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = "mock-rs4-all-nine";
    let front_name = "Mock RS4 AllNine";
    let mut def = mock_family_def(
        card_id,
        front_name,
        AbilityDefinition::TriggerDoubling {
            filter: TriggerDoublerFilter::ArtifactOrCreatureETB,
            additional_triggers: 1,
        },
    );
    def.abilities.extend(vec![
        AbilityDefinition::SuppressCreatureETBTriggers {
            filter: mtg_engine::ETBSuppressFilter::CreaturesOnly,
        },
        AbilityDefinition::StaticRestriction {
            restriction: GameRestriction::MaxSpellsPerTurn { max: 1 },
        },
        AbilityDefinition::CdaPowerToughness {
            power: EffectAmount::Fixed(5),
            toughness: EffectAmount::Fixed(5),
        },
        AbilityDefinition::CdaModifyPowerToughness {
            power: Some(EffectAmount::Fixed(1)),
            toughness: None,
        },
        AbilityDefinition::AdditionalLandPlays { count: 1 },
        AbilityDefinition::StaticFlashGrant {
            filter: FlashGrantFilter::AllSpells,
        },
        AbilityDefinition::StaticPlayFromGraveyard {
            filter: PlayFromTopFilter::All,
            condition: None,
        },
        AbilityDefinition::StaticPlayFromTop {
            filter: PlayFromTopFilter::All,
            look_at_top: false,
            reveal_top: false,
            pay_life_instead: false,
            condition: None,
            on_cast_effect: None,
        },
    ]);
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(registry.clone(), p1, p2, front_name, card_id);
    let id = find_by_name(&state, front_name);
    let card_id_val = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id_val.as_ref(), &registry, false);

    let count_families = |state: &GameState| -> usize {
        state
            .trigger_doublers()
            .iter()
            .filter(|d| d.source == id)
            .count()
            + state
                .etb_suppressors()
                .iter()
                .filter(|s| s.source == id)
                .count()
            + state
                .restrictions()
                .iter()
                .filter(|r| r.source == id)
                .count()
            + state
                .continuous_effects()
                .iter()
                .filter(|e| e.source == Some(id) && e.is_cda)
                .count()
            + state
                .additional_land_play_sources()
                .iter()
                .filter(|s| s.source == id)
                .count()
            + state
                .flash_grants()
                .iter()
                .filter(|f| f.source == Some(id))
                .count()
            + state
                .play_from_graveyard_permissions()
                .iter()
                .filter(|pm| pm.source == id)
                .count()
            + state
                .play_from_top_permissions()
                .iter()
                .filter(|pm| pm.source == id)
                .count()
    };

    let before = count_families(&state);
    assert_eq!(
        before, 9,
        "sanity: nine families (CdaModifyPowerToughness with only power Some \
         contributes exactly one continuous-effect entry) registered before any \
         transform"
    );

    transform(&mut state, p1, id);
    let after_out = count_families(&state);
    assert_eq!(
        after_out, 0,
        "all nine families must be deregistered on the way out"
    );

    transform(&mut state, p1, id);
    let after_round_trip = count_families(&state);
    assert_eq!(
        after_round_trip, before,
        "after transforming there and back, every collection must return to \
         EXACTLY its pre-transform contents -- not doubled (CR 712.18)"
    );
}

// ── Regression guard: precise removal, not a bulk purge (§7.3) ─────────────

/// Primarily a regression guard (its long-term value is POST-fix): it pins the
/// "remove at most the registered count, first structural match" rule against a
/// future bulk-purge refactor (e.g. `retain(|e| e.source != obj_id)`), which
/// would wrongly delete a same-source registration created by unrelated code
/// (mirrors the Class level-up pattern at `resolution.rs:7447-7470`, which
/// pushes an `AdditionalLandPlaySource` with `source: <the Class permanent>`
/// alongside any of the permanent's own static registrations). It also happens
/// to fail pre-fix today -- not because of over-removal, but because
/// `deregister_face_statics` removes nothing at all pre-fix, so the front's own
/// count:1 entry survives alongside the count:5 one (2 remain, not the expected
/// 1). That is consistent with, not contrary to, deviation #3.
#[test]
fn test_transform_does_not_remove_other_sources_registrations() {
    let p1 = p(1);
    let p2 = p(2);
    let card_id = "mock-rs4-no-overremove";
    let front_name = "Mock RS4 NoOverremove";
    let def = mock_family_def(
        card_id,
        front_name,
        AbilityDefinition::AdditionalLandPlays { count: 1 },
    );
    let registry = registry_with(vec![def]);
    let mut state = build_family_state(registry.clone(), p1, p2, front_name, card_id);
    let id = find_by_name(&state, front_name);
    let card_id_val = state.objects()[&id].card_id.clone();
    register_static_continuous_effects(&mut state, id, card_id_val.as_ref(), &registry, false);

    // A second, unrelated permanent registering into the same collection: must
    // never be touched by `id`'s deregistration.
    let other_id = ObjectId(999_001);
    state
        .additional_land_play_sources_mut()
        .push_back(AdditionalLandPlaySource {
            source: other_id,
            controller: p2,
            count: 3,
        });
    state.restrictions_mut().push_back(ActiveRestriction {
        source: other_id,
        controller: p2,
        restriction: GameRestriction::MaxSpellsPerTurn { max: 2 },
    });

    // A same-SOURCE, structurally-DIFFERENT entry on `id` itself (mirroring a
    // Class level-up pushing its own AdditionalLandPlaySource with a different
    // count, sharing `id` as the source ObjectId).
    state
        .additional_land_play_sources_mut()
        .push_back(AdditionalLandPlaySource {
            source: id,
            controller: p1,
            count: 5,
        });

    transform(&mut state, p1, id);

    // The front's OWN count:1 registration must be gone.
    let remaining: Vec<_> = state
        .additional_land_play_sources()
        .iter()
        .filter(|s| s.source == id)
        .collect();
    assert_eq!(
        remaining.len(),
        1,
        "exactly one entry (the front's own count:1) must be removed, leaving the \
         same-source count:5 entry alone; found {:?}",
        remaining
    );
    assert_eq!(
        remaining[0].count, 5,
        "the surviving same-source entry must be the structurally-different \
         count:5 one, not the removed count:1 one"
    );

    // The other permanent's entries must be completely untouched.
    assert!(
        state
            .additional_land_play_sources()
            .iter()
            .any(|s| s.source == other_id && s.count == 3),
        "the unrelated permanent's AdditionalLandPlaySource must survive"
    );
    assert!(
        state.restrictions().iter().any(|r| r.source == other_id),
        "the unrelated permanent's StaticRestriction must survive"
    );
}
