//! PB-RS3 / F-Rabble probe (`memory/card-authoring/review-pb-rs3-roster.md`).
//!
//! `goblin_rabblemaster.rs`'s completeness note claims "Other Goblin creatures
//! you control attack each combat if able" needs a new subtype-filtered
//! must-attack `GameRestriction` variant. The review flagged that framing as
//! stale: the engine already implements must-attack via
//! `KeywordAbility::MustAttackEachCombat`, read from LAYER-RESOLVED
//! characteristics at `combat.rs:378-390` (`expect_characteristics`, not the
//! object's own printed keyword list). Every piece needed to grant that
//! keyword to "other Goblins you control" already exists on the shelf:
//!
//! - `LayerModification::AddKeyword(KeywordAbility::MustAttackEachCombat)`
//! - `EffectFilter::OtherCreaturesYouControlWithSubtype(SubType)` -- live in
//!   `galadhrim_brigade.rs` (P/T) and `camellia_the_seedmiser.rs` (Menace)
//! - `AbilityDefinition::Static { .. }` + `EffectDuration::WhileSourceOnBattlefield`
//!
//! The open question this probe answers: does `AddKeyword` on the Layer 6
//! ability-grant layer compose with `expect_characteristics` for a
//! **non-source** object, for THIS specific keyword? This test builds a mock
//! "Rabblemaster-shaped" static grant (mirroring the exact
//! `register_static_continuous_effects` + `calculate_characteristics` pattern
//! already proven for transform statics in `pb_os4b_face_aware_abilities.rs`)
//! and drives it all the way through `Command::DeclareAttackers` enforcement
//! (`combat.rs:375-430`), not just a characteristics snapshot.

use mtg_engine::rules::layers::expect_characteristics;
use mtg_engine::rules::replacement::register_static_continuous_effects;
use mtg_engine::{
    AbilityDefinition, AttackTarget, CardContinuousEffectDef, CardDefinition, CardId, CardRegistry,
    CardType, Command, EffectDuration, EffectFilter, EffectLayer, GameState, GameStateBuilder,
    KeywordAbility, LayerModification, ObjectId, ObjectSpec, PlayerId, Step, SubType, TypeLine,
    ZoneId,
};

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

fn declare_cmd(player: PlayerId, attackers: Vec<(ObjectId, AttackTarget)>) -> Command {
    Command::DeclareAttackers {
        player,
        attackers,
        enlist_choices: vec![],
        exert_choices: vec![],
    }
}

/// Mock "Rabblemaster-shaped" card: a Goblin with a Static ability granting
/// `MustAttackEachCombat` to OTHER Goblins the controller controls. This is
/// exactly the shape `goblin_rabblemaster.rs` would carry if the review's
/// F-Rabble lead holds up.
fn mock_rabblemaster_grant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-rs3-rabblemaster-probe".to_string()),
        name: "Mock RS3 Rabblemaster Probe".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "".to_string(),
        abilities: vec![AbilityDefinition::Static {
            continuous_effect: CardContinuousEffectDef {
                layer: EffectLayer::Ability,
                modification: LayerModification::AddKeyword(KeywordAbility::MustAttackEachCombat),
                filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType(
                    "Goblin".to_string(),
                )),
                duration: EffectDuration::WhileSourceOnBattlefield,
                condition: None,
            },
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

fn other_goblin_spec(owner: PlayerId) -> ObjectSpec {
    ObjectSpec::creature(owner, "Other Goblin Token", 1, 1)
        .with_subtypes(vec![SubType("Goblin".to_string())])
}

/// Probe: `AddKeyword(MustAttackEachCombat)` granted by a Static ability with
/// `OtherCreaturesYouControlWithSubtype` DOES reach a non-source object's
/// layer-resolved characteristics, and `combat.rs`'s must-attack enforcement
/// (which reads exactly that path) DOES pick it up.
#[test]
fn test_addkeyword_mustattack_grant_composes_for_non_source_object() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![mock_rabblemaster_grant_def()]);

    let rabblemaster_spec = ObjectSpec::card(p1, "Mock RS3 Rabblemaster Probe")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-rs3-rabblemaster-probe".to_string()))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Goblin".to_string())]);
    let mut rabblemaster_spec = rabblemaster_spec;
    rabblemaster_spec.power = Some(2);
    rabblemaster_spec.toughness = Some(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(rabblemaster_spec)
        .object(other_goblin_spec(p1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let rabblemaster_id = find_by_name(&state, "Mock RS3 Rabblemaster Probe");
    let other_goblin_id = find_by_name(&state, "Other Goblin Token");

    // GameStateBuilder does not replay ETB -- register the Static grant
    // manually, matching what a real ETB would have done (same pattern as
    // pb_os4b_face_aware_abilities.rs).
    let card_id = state.objects()[&rabblemaster_id].card_id.clone();
    register_static_continuous_effects(
        &mut state,
        rabblemaster_id,
        card_id.as_ref(),
        &registry,
        false,
    );

    // Step 1: does the keyword reach the OTHER Goblin's layer-resolved
    // characteristics?
    let other_chars = expect_characteristics(&state, other_goblin_id);
    assert!(
        other_chars
            .keywords
            .contains(&KeywordAbility::MustAttackEachCombat),
        "AddKeyword(MustAttackEachCombat) granted by a Static ability with \
         OtherCreaturesYouControlWithSubtype must reach the non-source \
         object's layer-resolved characteristics"
    );

    // Step 2 (the CR-faithful negative control): the SOURCE itself must NOT
    // get the keyword -- "OTHER Goblin creatures you control attack each
    // combat if able" excludes Rabblemaster itself.
    let source_chars = expect_characteristics(&state, rabblemaster_id);
    assert!(
        !source_chars
            .keywords
            .contains(&KeywordAbility::MustAttackEachCombat),
        "the filter is OTHER creatures -- the source itself must not be \
         forced to attack by its own grant"
    );

    // Step 3: full enforcement path. combat.rs's must-attack check
    // (`combat.rs:375-430`) reads exactly `expect_characteristics` for every
    // battlefield object the active player controls -- declaring an empty
    // attack (the other Goblin is able to attack: untapped, no summoning
    // sickness, no Defender) must be rejected.
    let result = process_command_declare(state.clone(), declare_cmd(p1, vec![]));
    assert!(
        result.is_err(),
        "CR 508.1d: a Goblin forced to attack via the granted keyword must \
         not be able to sit out of combat when able to attack: {:?}",
        result.ok().map(|_| ())
    );

    // Step 4 (positive control): declaring the forced Goblin as an attacker
    // must succeed.
    let ok_result = process_command_declare(
        state,
        declare_cmd(p1, vec![(other_goblin_id, AttackTarget::Player(p2))]),
    );
    assert!(
        ok_result.is_ok(),
        "declaring the forced Goblin as an attacker must be legal: {:?}",
        ok_result.err()
    );
}

fn process_command_declare(
    state: GameState,
    cmd: Command,
) -> Result<(GameState, Vec<mtg_engine::GameEvent>), mtg_engine::GameStateError> {
    mtg_engine::process_command(state, cmd)
}
