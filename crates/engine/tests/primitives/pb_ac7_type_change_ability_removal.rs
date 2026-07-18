//! Tests for PB-AC7: Type-changing & ability-removal.
//!
//! Covers two genuinely-new engine primitives plus regression coverage of three
//! primitives the plan found already-expressible (see `memory/primitives/pb-plan-AC7.md`
//! "Scope reframing"):
//!
//! - `LayerModification::SetCreatureTypes(OrdSet<SubType>)` — Layer 4 (CR 205.1a): SETS
//!   the creature-type subtypes, preserving card types/supertypes/non-creature subtypes.
//! - `LayerModification::SetCardTypes(OrdSet<CardType>)` — Layer 4 (CR 205.1a): SETS the
//!   card types, leaving supertypes/subtypes untouched. Companion to `SetCreatureTypes`.
//! - `TriggerCondition::WheneverYouCastSpell.spell_subtype_filter: Option<Vec<SubType>>` —
//!   OR-semantics spell-subtype filter (CR 205.1a) for "cast an Aura/Equipment/Vehicle
//!   spell" (Sram) / "cast an Elf spell" (Leaf-Crowned Visionary) triggers.
//! - Regression: `LayerModification::RemoveAllAbilities` (CR 613.1f) composed with a
//!   later-timestamp `AddKeyword` (CR 613.7), with the face-down 2/2-no-text override
//!   (CR 708.2), and via `Effect::ApplyContinuousEffect` with an `UntilEndOfTurn`
//!   duration (CR 514.2/611.2a) — all already-expressible per the plan, not net-new
//!   primitives, but validated here per the plan's test list.
//!
//! Hash: `HASH_SCHEMA_VERSION` bumped 33 -> 34 (new `LayerModification` discriminants
//! 30/31, new `WheneverYouCastSpell.spell_subtype_filter` field). No new mutable
//! runtime GameState/PlayerState/GameObject fields this batch (see hash.rs changelog).

use imbl::OrdSet;
use mtg_engine::cards::card_definition::EffectAmount;
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    calculate_characteristics, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardId, CardRegistry, CardType, Command, ContinuousEffect, Effect,
    EffectDuration, EffectFilter, EffectId, EffectLayer, EnchantTarget, FaceDownKind, GameEvent,
    GameState, GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, Step, SubType, Target, TriggerCondition, ZoneId, HASH_SCHEMA_VERSION,
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
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Build a continuous effect with an explicit timestamp (mirrors `tests/layers.rs`
/// and `tests/conditional_statics.rs` helper idiom).
fn effect_at(
    id: u64,
    source: Option<ObjectId>,
    timestamp: u64,
    layer: EffectLayer,
    duration: EffectDuration,
    filter: EffectFilter,
    modification: LayerModification,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source,
        timestamp,
        layer,
        duration,
        filter,
        modification,
        is_cda: false,
        condition: None,
    }
}

fn cast_spell_no_targets(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<(GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
    cast_spell(state, player, card, vec![])
}

fn cast_spell(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
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

/// A single-ability CardDefinition whose only ability is a `WheneverYouCastSpell`
/// trigger with the given `spell_subtype_filter`. Draws a card on fire (unresolved
/// draw is not asserted -- only that the trigger appears on the stack).
fn cast_trigger_def(spell_subtype_filter: Option<Vec<SubType>>) -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-cast-trigger".to_string()),
        name: "Test Cast Trigger".to_string(),
        power: Some(1),
        toughness: Some(1),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever you cast a spell with a matching subtype, draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverYouCastSpell {
                during_opponent_turn: false,
                spell_type_filter: None,
                noncreature_only: false,
                chosen_subtype_filter: false,
                spell_subtype_filter,
            },
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}

/// Build a 2-player state with the cast-trigger creature on p1's battlefield (from
/// `defs`/`registry`), a dummy "Target Bear" creature (for Aura-targeting casts),
/// and one extra spell object placed via `extra`.
fn build_trigger_state(
    p1: PlayerId,
    p2: PlayerId,
    defs: &HashMap<String, CardDefinition>,
    registry: std::sync::Arc<CardRegistry>,
    extra: ObjectSpec,
) -> GameState {
    let trigger_spec = enrich_spec_from_def(
        ObjectSpec::creature(p1, "Test Cast Trigger", 1, 1)
            .with_card_id(CardId("test-cast-trigger".to_string())),
        defs,
    );
    GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(trigger_spec)
        .object(ObjectSpec::creature(p1, "Target Bear", 2, 2))
        .object(extra)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap()
}

fn stack_has_trigger_for(state: &GameState, source_id: ObjectId) -> bool {
    state
        .stack_objects()
        .iter()
        .any(|so| matches!(&so.kind, mtg_engine::StackObjectKind::TriggeredAbility { source_object, .. } if *source_object == source_id))
}

// ---------------------------------------------------------------------------
// SetCreatureTypes (Layer 4, CR 205.1a)
// ---------------------------------------------------------------------------

/// CR 205.1a — `SetCreatureTypes` replaces creature-type subtypes but preserves
/// card types (Artifact + Creature survive) and supertypes.
#[test]
fn test_set_creature_types_replaces_creature_subtypes_keeps_card_types() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Golem Body", 3, 3)
                .with_types(vec![CardType::Artifact, CardType::Creature])
                .with_subtypes(vec![SubType("Golem".to_string())])
                .with_supertypes(vec![mtg_engine::SuperType::Legendary]),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::SetCreatureTypes(OrdSet::unit(SubType("Elk".to_string()))),
        ))
        .build()
        .unwrap();

    let id = find_object(&state, "Golem Body");
    let chars = calculate_characteristics(&state, id).unwrap();

    assert_eq!(
        chars.subtypes,
        OrdSet::unit(SubType("Elk".to_string())),
        "CR 205.1a: creature-type subtypes replaced; Golem removed, Elk added"
    );
    assert!(
        chars.card_types.contains(&CardType::Artifact)
            && chars.card_types.contains(&CardType::Creature),
        "SetCreatureTypes must not touch card types"
    );
    assert!(
        chars.supertypes.contains(&mtg_engine::SuperType::Legendary),
        "SetCreatureTypes must preserve supertypes (Legendary)"
    );
}

/// CR 205.1a — `SetCreatureTypes` preserves non-creature subtypes (e.g. a land
/// subtype on a creature-land) while replacing only the creature-type subtypes.
#[test]
fn test_set_creature_types_preserves_noncreature_subtypes() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Creature Land", 2, 2)
                .with_types(vec![CardType::Land, CardType::Creature])
                .with_subtypes(vec![
                    SubType("Forest".to_string()),
                    SubType("Elemental".to_string()),
                ]),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::SetCreatureTypes(OrdSet::unit(SubType("Elk".to_string()))),
        ))
        .build()
        .unwrap();

    let id = find_object(&state, "Creature Land");
    let chars = calculate_characteristics(&state, id).unwrap();

    assert!(
        chars.subtypes.contains(&SubType("Forest".to_string())),
        "CR 205.1a: non-creature (land) subtype Forest must survive SetCreatureTypes"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Elemental".to_string())),
        "prior creature-type subtype Elemental must be replaced"
    );
    assert!(
        chars.subtypes.contains(&SubType("Elk".to_string())),
        "new creature-type subtype Elk must be present"
    );
}

// ---------------------------------------------------------------------------
// SetCardTypes (Layer 4, CR 205.1a)
// ---------------------------------------------------------------------------

/// CR 205.1a — `SetCardTypes` replaces card types, leaving supertypes and
/// subtypes untouched (Legendary + Golem both survive; Artifact is gone).
#[test]
fn test_set_card_types_replaces_card_types_preserves_supertypes() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Living Statue", 3, 3)
                .with_types(vec![CardType::Artifact, CardType::Creature])
                .with_subtypes(vec![SubType("Golem".to_string())])
                .with_supertypes(vec![mtg_engine::SuperType::Legendary]),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::SetCardTypes(OrdSet::unit(CardType::Creature)),
        ))
        .build()
        .unwrap();

    let id = find_object(&state, "Living Statue");
    let chars = calculate_characteristics(&state, id).unwrap();

    assert_eq!(
        chars.card_types,
        OrdSet::unit(CardType::Creature),
        "CR 205.1a: card types replaced to exactly {{Creature}}"
    );
    assert!(
        chars.subtypes.contains(&SubType("Golem".to_string())),
        "SetCardTypes must not touch subtypes"
    );
    assert!(
        chars.supertypes.contains(&mtg_engine::SuperType::Legendary),
        "SetCardTypes must preserve supertypes"
    );
}

/// CR 205.1a correlated-subtype-removal clause (review PB-AC7 H1 fix) —
/// `SetCardTypes` removing the Artifact card type must ALSO drop the Equipment
/// subtype (correlated only with Artifact), while an unrelated creature-type
/// subtype (Golem, correlated with Creature, which survives) is untouched.
/// This is the exact Kenrith's Transformation / Eaten by Piranhas scenario: an
/// artifact-creature Equipment target losing its Artifact card type.
#[test]
fn test_set_card_types_drops_correlated_subtype_when_card_type_removed() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Equipment Creature", 3, 3)
                .with_types(vec![CardType::Artifact, CardType::Creature])
                .with_subtypes(vec![
                    SubType("Equipment".to_string()),
                    SubType("Golem".to_string()),
                ]),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::SetCardTypes(OrdSet::unit(CardType::Creature)),
        ))
        .build()
        .unwrap();

    let id = find_object(&state, "Equipment Creature");
    let chars = calculate_characteristics(&state, id).unwrap();

    assert_eq!(
        chars.card_types,
        OrdSet::unit(CardType::Creature),
        "CR 205.1a: card types replaced to exactly {{Creature}}"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Equipment".to_string())),
        "CR 205.1a: Equipment (correlated with the now-removed Artifact card type) \
         must be dropped -- this is the Kenrith's Transformation / Eaten by Piranhas bug"
    );
    assert!(
        chars.subtypes.contains(&SubType("Golem".to_string())),
        "CR 205.1a: Golem (correlated with Creature, which survives) must remain"
    );
}

// ---------------------------------------------------------------------------
// Darksteel-Mutation-style integration: SetCardTypes + SetCreatureTypes + timestamp
// ---------------------------------------------------------------------------

/// CR 205.1b + CR 613.7 — Darksteel-Mutation-style Aura: enchanted creature
/// "is an Insect artifact creature with base P/T 0/1 and has indestructible, and
/// it loses all other abilities, card types, and creature types." Composed from
/// `RemoveAllAbilities` (earlier timestamp), `AddKeyword(Indestructible)` (later
/// timestamp), `SetCardTypes({Artifact,Creature})`, `SetCreatureTypes({Insect})`,
/// and `SetPowerToughness{0,1}`. Verifies indestructible survives the "loses all
/// OTHER abilities" removal (granted-then-removed ordering, CR 613.7) and flying
/// (the creature's original ability) does not.
///
/// The target is an enchantment-creature with the "Shrine" subtype (the exact
/// ruled example from Darksteel Mutation's own Gatherer ruling: "If it had any
/// subtypes other than artifact types and creature types (such as Shrine), it
/// won't retain those.") — this exercises the CR 205.1a correlated-subtype-
/// removal clause (PB-AC7 review H1): Enchantment is removed by `SetCardTypes`,
/// so Shrine must be dropped, while Artifact survives so it would keep an
/// Equipment-style artifact subtype if it had one (not applicable here).
#[test]
fn test_darksteel_mutation_keeps_indestructible() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Big Flyer", 5, 5)
                .with_keyword(KeywordAbility::Flying)
                .with_types(vec![CardType::Enchantment, CardType::Creature])
                .with_subtypes(vec![
                    SubType("Shrine".to_string()),
                    SubType("God".to_string()),
                ]),
        )
        // The Aura itself, on the battlefield, as the effects' source (so
        // `EffectDuration::WhileSourceOnBattlefield` is active — CR 613.6).
        .object(ObjectSpec::enchantment(p(1), "Mock Darksteel Mutation"))
        .build()
        .unwrap();

    let target_id = find_object(&state, "Big Flyer");
    let aura_source = find_object(&state, "Mock Darksteel Mutation");

    // List removal BEFORE the indestructible grant (earlier timestamp) so the
    // later-timestamp grant survives the removal (CR 613.7).
    state.continuous_effects_mut().push_back(effect_at(
        1,
        Some(aura_source),
        10,
        EffectLayer::Ability,
        EffectDuration::WhileSourceOnBattlefield,
        EffectFilter::SingleObject(target_id),
        LayerModification::RemoveAllAbilities,
    ));
    state.continuous_effects_mut().push_back(effect_at(
        2,
        Some(aura_source),
        11,
        EffectLayer::Ability,
        EffectDuration::WhileSourceOnBattlefield,
        EffectFilter::SingleObject(target_id),
        LayerModification::AddKeyword(KeywordAbility::Indestructible),
    ));
    state.continuous_effects_mut().push_back(effect_at(
        3,
        Some(aura_source),
        12,
        EffectLayer::TypeChange,
        EffectDuration::WhileSourceOnBattlefield,
        EffectFilter::SingleObject(target_id),
        LayerModification::SetCardTypes(
            [CardType::Artifact, CardType::Creature]
                .into_iter()
                .collect(),
        ),
    ));
    state.continuous_effects_mut().push_back(effect_at(
        4,
        Some(aura_source),
        12,
        EffectLayer::TypeChange,
        EffectDuration::WhileSourceOnBattlefield,
        EffectFilter::SingleObject(target_id),
        LayerModification::SetCreatureTypes(OrdSet::unit(SubType("Insect".to_string()))),
    ));
    state.continuous_effects_mut().push_back(effect_at(
        5,
        Some(aura_source),
        13,
        EffectLayer::PtSet,
        EffectDuration::WhileSourceOnBattlefield,
        EffectFilter::SingleObject(target_id),
        LayerModification::SetPowerToughness {
            power: 0,
            toughness: 1,
        },
    ));

    let chars = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(chars.power, Some(0));
    assert_eq!(chars.toughness, Some(1));
    assert_eq!(
        chars.card_types,
        [CardType::Artifact, CardType::Creature]
            .into_iter()
            .collect(),
        "CR 205.1b: card types become exactly Artifact+Creature"
    );
    assert_eq!(
        chars.subtypes,
        OrdSet::unit(SubType("Insect".to_string())),
        "CR 205.1a: creature-type subtypes become exactly Insect; Shrine (correlated \
         with the now-removed Enchantment card type) must be dropped -- the Darksteel \
         Mutation Gatherer ruling's exact 'Shrine won't retain' example"
    );
    assert!(
        chars.keywords.contains(&KeywordAbility::Indestructible),
        "CR 613.7: Indestructible (later timestamp) survives RemoveAllAbilities"
    );
    assert!(
        !chars.keywords.contains(&KeywordAbility::Flying),
        "CR 613.1f: RemoveAllAbilities strips the creature's original Flying"
    );
}

// ---------------------------------------------------------------------------
// Granted-then-removed timestamp ordering (CR 613.7)
// ---------------------------------------------------------------------------

/// CR 613.7 — a Layer-6 `AddKeyword` with a LATER timestamp than a co-resident
/// `RemoveAllAbilities` survives the removal; with an EARLIER timestamp, it does
/// not. Wedges on `chars.keywords` (the property `RemoveAllAbilities` actually
/// clears — CR 613 gotcha #39).
#[test]
fn test_granted_then_removed_ordering_by_timestamp() {
    // Case A: removal (ts 10) BEFORE grant (ts 20) -> grant (Flying) survives.
    let state_a = GameStateBuilder::new()
        .add_player(p(1))
        .object(ObjectSpec::creature(p(1), "Wedge A", 2, 2))
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::Ability,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::RemoveAllAbilities,
        ))
        .add_continuous_effect(effect_at(
            2,
            None,
            20,
            EffectLayer::Ability,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::AddKeyword(KeywordAbility::Flying),
        ))
        .build()
        .unwrap();
    let id_a = find_object(&state_a, "Wedge A");
    let chars_a = calculate_characteristics(&state_a, id_a).unwrap();
    assert!(
        chars_a.keywords.contains(&KeywordAbility::Flying),
        "CR 613.7: later-timestamp AddKeyword survives an earlier RemoveAllAbilities"
    );

    // Case B: grant (ts 10) BEFORE removal (ts 20) -> grant (Flying) is removed.
    let state_b = GameStateBuilder::new()
        .add_player(p(1))
        .object(ObjectSpec::creature(p(1), "Wedge B", 2, 2))
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::Ability,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::AddKeyword(KeywordAbility::Flying),
        ))
        .add_continuous_effect(effect_at(
            2,
            None,
            20,
            EffectLayer::Ability,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::RemoveAllAbilities,
        ))
        .build()
        .unwrap();
    let id_b = find_object(&state_b, "Wedge B");
    let chars_b = calculate_characteristics(&state_b, id_b).unwrap();
    assert!(
        !chars_b.keywords.contains(&KeywordAbility::Flying),
        "CR 613.7: later-timestamp RemoveAllAbilities removes an earlier-granted keyword"
    );
}

// ---------------------------------------------------------------------------
// RemoveAllAbilities composed with face-down override (CR 708.2)
// ---------------------------------------------------------------------------

/// CR 708.2 / 708.2a — a face-down permanent (empty ability set from the pre-loop
/// override) under a `RemoveAllAbilities` effect stays a 2/2 with no abilities.
/// The face-down override runs BEFORE the layer loop; RemoveAllAbilities is a
/// no-op on an already-empty ability set. No panic, no unexpected interaction.
#[test]
fn test_lose_abilities_vs_face_down_override() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Face Down Thing", 4, 5)
                .with_keyword(KeywordAbility::Flying),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::Ability,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::RemoveAllAbilities,
        ))
        .build()
        .unwrap();

    let id = find_object(&state, "Face Down Thing");
    {
        let obj = state.objects_mut().get_mut(&id).unwrap();
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Morph);
    }

    let chars = calculate_characteristics(&state, id).unwrap();
    assert_eq!(chars.power, Some(2), "CR 708.2a: face-down base P/T is 2/2");
    assert_eq!(chars.toughness, Some(2));
    assert!(
        chars.keywords.is_empty(),
        "CR 708.2a + 613.1f: face-down has no keywords; RemoveAllAbilities is a no-op on empty set"
    );
}

// ---------------------------------------------------------------------------
// One-shot Layer-4 type override with duration (already expressible via
// ApplyContinuousEffect) — expiry at cleanup (CR 514.2)
// ---------------------------------------------------------------------------

/// CR 514.2 / 611.2a — a one-shot `RemoveAllAbilities` registered via
/// `Effect::ApplyContinuousEffect` with `UntilEndOfTurn` duration removes flying
/// while active, then flying returns after `expire_end_of_turn_effects` (CR 514.2
/// Cleanup) removes the expired continuous effect.
#[test]
fn test_lose_abilities_one_shot_until_eot() {
    use mtg_engine::cards::card_definition::ContinuousEffectDef;

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::creature(p(1), "Temp Vanilla", 3, 3).with_keyword(KeywordAbility::Flying),
        )
        .build()
        .unwrap();

    let source_id = find_object(&state, "Temp Vanilla");

    let effect = Effect::ApplyContinuousEffect {
        effect_def: Box::new(ContinuousEffectDef {
            layer: EffectLayer::Ability,
            modification: LayerModification::RemoveAllAbilities,
            filter: EffectFilter::AllCreatures,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    let mut ctx = EffectContext::new(p(1), source_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let chars_during = calculate_characteristics(&state, source_id).unwrap();
    assert!(
        chars_during.keywords.is_empty(),
        "CR 613.1f: flying removed while the one-shot UntilEndOfTurn effect is active"
    );

    // Simulate Cleanup (CR 514.2): expire UntilEndOfTurn continuous effects.
    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    let chars_after = calculate_characteristics(&state, source_id).unwrap();
    assert!(
        chars_after.keywords.contains(&KeywordAbility::Flying),
        "CR 514.2: flying returns after the UntilEndOfTurn effect expires at cleanup"
    );
}

// ---------------------------------------------------------------------------
// Layer 4 dependency / timestamp interaction (CR 613.8)
// ---------------------------------------------------------------------------

/// CR 613.8 — `SetCreatureTypes` (creature-type subtypes only) and a co-resident
/// `AddSubtypes` targeting a DISJOINT subtype set (a land subtype, Urborg-style)
/// produce the same result regardless of timestamp order, because the two
/// modifications never touch the same subset of `subtypes`. No dependency arm was
/// added for this pair (see `rules/layers.rs::depends_on` PB-AC7 comment) — pure
/// timestamp order (CR 613.7) is correct here.
#[test]
fn test_set_creature_types_layer4_dependency_with_add_subtypes() {
    // Order 1: AddSubtypes(Swamp) OLDER, SetCreatureTypes(Elk) NEWER.
    let state_1 = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Order1", 2, 2)
                .with_types(vec![CardType::Land, CardType::Creature])
                .with_subtypes(vec![SubType("Golem".to_string())]),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::AddSubtypes(OrdSet::unit(SubType("Swamp".to_string()))),
        ))
        .add_continuous_effect(effect_at(
            2,
            None,
            20,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::SetCreatureTypes(OrdSet::unit(SubType("Elk".to_string()))),
        ))
        .build()
        .unwrap();
    let id_1 = find_object(&state_1, "Order1");
    let chars_1 = calculate_characteristics(&state_1, id_1).unwrap();

    // Order 2: SetCreatureTypes(Elk) OLDER, AddSubtypes(Swamp) NEWER.
    let state_2 = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "Order2", 2, 2)
                .with_types(vec![CardType::Land, CardType::Creature])
                .with_subtypes(vec![SubType("Golem".to_string())]),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::SetCreatureTypes(OrdSet::unit(SubType("Elk".to_string()))),
        ))
        .add_continuous_effect(effect_at(
            2,
            None,
            20,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::AddSubtypes(OrdSet::unit(SubType("Swamp".to_string()))),
        ))
        .build()
        .unwrap();
    let id_2 = find_object(&state_2, "Order2");
    let chars_2 = calculate_characteristics(&state_2, id_2).unwrap();

    let expected: OrdSet<SubType> = [SubType("Elk".to_string()), SubType("Swamp".to_string())]
        .into_iter()
        .collect();
    assert_eq!(
        chars_1.subtypes, expected,
        "disjoint subtype sets: order-independent result (Swamp older)"
    );
    assert_eq!(
        chars_2.subtypes, expected,
        "disjoint subtype sets: order-independent result (Elk older)"
    );
}

/// CR 613.8 (PB-AC7 review M1 fix) — `SetCreatureTypes({Elk})` and a co-resident
/// `AddSubtypes({Zombie})` are NON-disjoint (Zombie is itself a creature type):
/// applying `AddSubtypes` first lets `SetCreatureTypes` filter Zombie out ({Elk}
/// only); applying `AddSubtypes` second, WITHOUT a dependency, would union it in
/// ({Elk, Zombie}) — order-dependent, which is exactly the CR 613.8a trigger for
/// a dependency arm. With the `(SetCreatureTypes, AddSubtypes)` dependency arm
/// added in `rules/layers.rs::depends_on`, `AddSubtypes` is always forced to
/// apply before `SetCreatureTypes` regardless of timestamp, so BOTH orders below
/// now converge on {Elk} — the reviewer's exact counterexample, resolved.
#[test]
fn test_set_creature_types_layer4_dependency_nondisjoint_creature_subtype() {
    // Order 1: AddSubtypes(Zombie) OLDER, SetCreatureTypes(Elk) NEWER.
    let state_1 = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "NDOrder1", 2, 2)
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![SubType("Golem".to_string())]),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::AddSubtypes(OrdSet::unit(SubType("Zombie".to_string()))),
        ))
        .add_continuous_effect(effect_at(
            2,
            None,
            20,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::SetCreatureTypes(OrdSet::unit(SubType("Elk".to_string()))),
        ))
        .build()
        .unwrap();
    let id_1 = find_object(&state_1, "NDOrder1");
    let chars_1 = calculate_characteristics(&state_1, id_1).unwrap();

    // Order 2: SetCreatureTypes(Elk) OLDER, AddSubtypes(Zombie) NEWER -- without
    // the dependency arm, natural timestamp order would apply AddSubtypes LAST
    // and produce {Elk, Zombie} instead.
    let state_2 = GameStateBuilder::new()
        .add_player(p(1))
        .object(
            ObjectSpec::creature(p(1), "NDOrder2", 2, 2)
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![SubType("Golem".to_string())]),
        )
        .add_continuous_effect(effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::SetCreatureTypes(OrdSet::unit(SubType("Elk".to_string()))),
        ))
        .add_continuous_effect(effect_at(
            2,
            None,
            20,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            LayerModification::AddSubtypes(OrdSet::unit(SubType("Zombie".to_string()))),
        ))
        .build()
        .unwrap();
    let id_2 = find_object(&state_2, "NDOrder2");
    let chars_2 = calculate_characteristics(&state_2, id_2).unwrap();

    let expected = OrdSet::unit(SubType("Elk".to_string()));
    assert_eq!(
        chars_1.subtypes, expected,
        "CR 613.8: non-disjoint (Zombie is a creature type) -- AddSubtypes-older order"
    );
    assert_eq!(
        chars_2.subtypes, expected,
        "CR 613.8: non-disjoint (Zombie is a creature type) -- SetCreatureTypes-older \
         order must now agree via the dependency arm (would be {{Elk, Zombie}} without it)"
    );
}

// ---------------------------------------------------------------------------
// spell_subtype_filter (CR 205.1a) — Sram/Leaf-Crowned-style cast trigger
// ---------------------------------------------------------------------------

/// CR 205.1a — `spell_subtype_filter: Some(vec![Aura, Equipment, Vehicle])` fires
/// on casting any spell carrying one of those subtypes (Sram, Senior Edificer).
#[test]
fn test_spell_subtype_filter_positive() {
    let p1 = p(1);
    let p2 = p(2);
    let def = cast_trigger_def(Some(vec![
        SubType("Aura".to_string()),
        SubType("Equipment".to_string()),
        SubType("Vehicle".to_string()),
    ]));
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut defs: HashMap<String, CardDefinition> = HashMap::new();
    defs.insert(def.name.clone(), def.clone());

    // Equipment spell.
    let equipment = ObjectSpec::card(p1, "Test Equipment")
        .with_types(vec![CardType::Artifact])
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .in_zone(ZoneId::Hand(p1));
    let state = build_trigger_state(p1, p2, &defs, registry.clone(), equipment);
    let source_id = find_object(&state, "Test Cast Trigger");
    let spell_id = find_object(&state, "Test Equipment");
    let (state, _events) = cast_spell_no_targets(state, p1, spell_id).unwrap();
    assert!(
        stack_has_trigger_for(&state, source_id),
        "CR 205.1a: casting an Equipment spell fires the Aura/Equipment/Vehicle filter"
    );

    // Vehicle spell.
    let vehicle = ObjectSpec::card(p1, "Test Vehicle")
        .with_types(vec![CardType::Artifact])
        .with_subtypes(vec![SubType("Vehicle".to_string())])
        .in_zone(ZoneId::Hand(p1));
    let state = build_trigger_state(p1, p2, &defs, registry.clone(), vehicle);
    let source_id = find_object(&state, "Test Cast Trigger");
    let spell_id = find_object(&state, "Test Vehicle");
    let (state, _events) = cast_spell_no_targets(state, p1, spell_id).unwrap();
    assert!(
        stack_has_trigger_for(&state, source_id),
        "CR 205.1a: casting a Vehicle spell fires the Aura/Equipment/Vehicle filter"
    );

    // Aura spell (needs a legal Enchant-creature target: Target Bear).
    let aura = ObjectSpec::card(p1, "Test Aura")
        .with_types(vec![CardType::Enchantment])
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Creature))
        .in_zone(ZoneId::Hand(p1));
    let state = build_trigger_state(p1, p2, &defs, registry, aura);
    let source_id = find_object(&state, "Test Cast Trigger");
    let spell_id = find_object(&state, "Test Aura");
    let target_id = find_object(&state, "Target Bear");
    let (state, _events) =
        cast_spell(state, p1, spell_id, vec![Target::Object(target_id)]).unwrap();
    assert!(
        stack_has_trigger_for(&state, source_id),
        "CR 205.1a: casting an Aura spell fires the Aura/Equipment/Vehicle filter"
    );
}

/// CR 205.1a — casting a spell with none of the filtered subtypes (a vanilla
/// creature spell) does NOT fire the `spell_subtype_filter` trigger.
#[test]
fn test_spell_subtype_filter_negative() {
    let p1 = p(1);
    let p2 = p(2);
    let def = cast_trigger_def(Some(vec![
        SubType("Aura".to_string()),
        SubType("Equipment".to_string()),
        SubType("Vehicle".to_string()),
    ]));
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut defs: HashMap<String, CardDefinition> = HashMap::new();
    defs.insert(def.name.clone(), def.clone());

    let creature = ObjectSpec::card(p1, "Test Vanilla Creature")
        .with_types(vec![CardType::Creature])
        .in_zone(ZoneId::Hand(p1));
    let state = build_trigger_state(p1, p2, &defs, registry, creature);
    let source_id = find_object(&state, "Test Cast Trigger");
    let spell_id = find_object(&state, "Test Vanilla Creature");
    let (state, _events) = cast_spell_no_targets(state, p1, spell_id).unwrap();
    assert!(
        !stack_has_trigger_for(&state, source_id),
        "CR 205.1a: a vanilla creature spell (no matching subtype) must NOT fire"
    );
}

/// Regression: `spell_subtype_filter: None` (all 21 existing card-def sites) still
/// fires on every qualifying spell, unaffected by the new field.
#[test]
fn test_spell_subtype_filter_none_matches_all() {
    let p1 = p(1);
    let p2 = p(2);
    let def = cast_trigger_def(None);
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut defs: HashMap<String, CardDefinition> = HashMap::new();
    defs.insert(def.name.clone(), def.clone());

    let creature = ObjectSpec::card(p1, "Test Vanilla Creature")
        .with_types(vec![CardType::Creature])
        .in_zone(ZoneId::Hand(p1));
    let state = build_trigger_state(p1, p2, &defs, registry, creature);
    let source_id = find_object(&state, "Test Cast Trigger");
    let spell_id = find_object(&state, "Test Vanilla Creature");
    let (state, _events) = cast_spell_no_targets(state, p1, spell_id).unwrap();
    assert!(
        stack_has_trigger_for(&state, source_id),
        "spell_subtype_filter: None must not restrict casts (regression guard)"
    );
}

// ---------------------------------------------------------------------------
// Hash schema (CR N/A — hash infrastructure)
// ---------------------------------------------------------------------------

/// PB-AC7 bumped `HASH_SCHEMA_VERSION` 33 -> 34 (new `LayerModification` discriminants
/// 30/31 + `WheneverYouCastSpell.spell_subtype_filter`). Live sentinel.
#[test]
fn test_hash_schema_version_is_34() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 49u8,
        "PB-AC7 bumped HASH_SCHEMA_VERSION 33->34. If you bumped again, update this test."
    );
}

/// Hash arm coverage: two `ContinuousEffect`s identical except `SetCreatureTypes`
/// payload hash differently, and `SetCreatureTypes` vs `SetTypeLine` (same
/// subtypes) hash differently (validates discriminant 30 is distinct from
/// `SetTypeLine`'s discriminant 2).
#[test]
fn test_hash_distinguishes_set_creature_types_payload() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let base = |modification: LayerModification| -> ContinuousEffect {
        effect_at(
            1,
            None,
            10,
            EffectLayer::TypeChange,
            EffectDuration::WhileSourceOnBattlefield,
            EffectFilter::AllCreatures,
            modification,
        )
    };

    let elk = base(LayerModification::SetCreatureTypes(OrdSet::unit(SubType(
        "Elk".to_string(),
    ))));
    let frog = base(LayerModification::SetCreatureTypes(OrdSet::unit(SubType(
        "Frog".to_string(),
    ))));
    let set_type_line = base(LayerModification::SetTypeLine {
        supertypes: OrdSet::new(),
        card_types: OrdSet::unit(CardType::Creature),
        subtypes: OrdSet::unit(SubType("Elk".to_string())),
    });

    let hash_of = |e: &ContinuousEffect| -> [u8; 32] {
        let mut hasher = Hasher::new();
        e.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    assert_ne!(
        hash_of(&elk),
        hash_of(&frog),
        "SetCreatureTypes(Elk) and SetCreatureTypes(Frog) must hash differently"
    );
    assert_ne!(
        hash_of(&elk),
        hash_of(&set_type_line),
        "SetCreatureTypes and SetTypeLine (same subtypes) must hash differently (disc 30 vs 2)"
    );
}

/// Hash arm coverage: `WheneverYouCastSpell` with `spell_subtype_filter: None` vs
/// `Some(vec![Elf])` hash differently (validates the new destructure + hash line).
#[test]
fn test_hash_distinguishes_spell_subtype_filter() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let none_filter = TriggerCondition::WheneverYouCastSpell {
        during_opponent_turn: false,
        spell_type_filter: None,
        noncreature_only: false,
        chosen_subtype_filter: false,
        spell_subtype_filter: None,
    };
    let elf_filter = TriggerCondition::WheneverYouCastSpell {
        during_opponent_turn: false,
        spell_type_filter: None,
        noncreature_only: false,
        chosen_subtype_filter: false,
        spell_subtype_filter: Some(vec![SubType("Elf".to_string())]),
    };

    let hash_of = |t: &TriggerCondition| -> [u8; 32] {
        let mut hasher = Hasher::new();
        t.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    assert_ne!(
        hash_of(&none_filter),
        hash_of(&elf_filter),
        "spell_subtype_filter: None vs Some(vec![Elf]) must hash differently"
    );
}
