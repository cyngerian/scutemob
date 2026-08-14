//! Tests for PB-DX43 (CR 305.6/305.7): the intrinsic "{T}: Add [symbol]" mana ability every
//! object with the land card type AND a basic land type has, whether or not its text box
//! actually contains that text.
//!
//! Engine surface under test: `rules::layers::derive_intrinsic_land_mana_abilities` (D1 -- runs
//! at the END of each Layer-4 iteration, so it sees the fully dependency-resolved subtype set and
//! is still subordinate to Layer 6), `rules::layers::discharges_intrinsic_mana_ability` (D4 --
//! idempotence predicate), and the `LayerModification::SetLandTypes` arm's CR 305.7
//! ability-clearing (D2 -- gated on the payload actually naming a basic land type).
//!
//! See `pb_dx27_blood_moon_type_scope.rs` for the sibling regression suite this batch must not
//! weaken (`t5`/`t6`/`t7`/`t8`) -- `t6`'s own doc comment predicted this batch by name
//! (`OOS-DX27-10`).

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    calculate_characteristics, enrich_spec_from_def, process_command, CardDefinition, CardId,
    CardRegistry, CardType, Command, Condition, ContinuousEffect, EffectDuration, EffectFilter,
    EffectId, EffectLayer, FaceDownKind, GameEvent, GameState, GameStateBuilder, LayerModification,
    ManaAbility, ManaColor, ObjectId, ObjectSpec, PlayerId, Step, SubType, ZoneId,
};
use std::collections::HashMap;

// ── Helpers (mirrors pb_dx27_blood_moon_type_scope.rs's idiom) ────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs_of(defs: &[&CardDefinition]) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    for d in defs {
        m.insert(d.name.clone(), (*d).clone());
    }
    m
}

fn card_spec(
    player: PlayerId,
    name: &str,
    card_id: &str,
    zone: ZoneId,
    all_defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(ObjectSpec::card(player, name), all_defs)
        .with_card_id(CardId(card_id.to_string()))
        .in_zone(zone)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn cast_spell_no_targets(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<(GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
    process_command(
        state,
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
        })),
    )
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut state = state;
    for &pl in players {
        let (s, ev) = process_command(state, Command::PassPriority { player: pl }).unwrap();
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

/// Gives `player` enough mana for `{2}{R}` (Blood Moon's cost) and casts+resolves the named
/// card from hand.
fn cast_and_resolve_2r(
    state: GameState,
    player: PlayerId,
    card_name: &str,
    all_players: &[PlayerId],
) -> GameState {
    let mut state = state;
    {
        let pool = &mut state.players_mut().get_mut(&player).unwrap().mana_pool;
        pool.colorless = 2;
        pool.red = 1;
    }
    state.turn_mut().priority_holder = Some(player);
    let card_id = find_object(&state, card_name);
    let (state, _) = cast_spell_no_targets(state, player, card_id).unwrap();
    resolve_stack(state, all_players)
}

/// A raw `AddSubtypes` continuous effect mirroring Urborg/Yavimaya's exact shape.
fn add_subtypes_effect(
    id: u64,
    timestamp: u64,
    filter: EffectFilter,
    subtype_name: &str,
) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::Indefinite,
        filter,
        modification: LayerModification::AddSubtypes(imbl::OrdSet::unit(SubType(
            subtype_name.to_string(),
        ))),
        is_cda: false,
        affected_set: None,
        condition: None,
    }
}

/// A raw `SetLandTypes` continuous effect mirroring Blood Moon/Magus of the Moon's exact shape.
fn set_land_types_effect(id: u64, timestamp: u64, subtype_name: &str) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllNonbasicLands,
        modification: LayerModification::SetLandTypes(imbl::OrdSet::unit(SubType(
            subtype_name.to_string(),
        ))),
        is_cda: false,
        affected_set: None,
        condition: None,
    }
}

// ── P1/P2/P3: the three staple "nonbasic land is also a basic type" cards ─────────────────────

/// CR 305.6/305.7: "Each land is a Swamp in addition to its other land types" (Urborg, modelled
/// here as its exact `AddSubtypes(Swamp)`/`AllLands` static). A Plains under Urborg must produce
/// BOTH its own printed {W} AND the intrinsic {B} the gained Swamp subtype carries -- CR 305.7's
/// "if a land gains one or more land types IN ADDITION to its own, it keeps its land types and
/// rules text, and it gains the new land types and mana abilities" clause.
#[test]
fn p1_urborg_grants_swamp_intrinsic_black_to_a_plains() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .object(
            ObjectSpec::land(p1, "Test Plains").with_subtypes(vec![SubType("Plains".to_string())]),
        )
        .add_continuous_effect(add_subtypes_effect(1, 1, EffectFilter::AllLands, "Swamp"))
        .build()
        .unwrap();

    let land_id = find_object(&state, "Test Plains");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars
            .mana_abilities
            .iter()
            .any(|ma| ma.produces.get(&ManaColor::White).copied() == Some(1)),
        "the land's own Plains subtype must still produce {{W}}: {:?}",
        chars.mana_abilities
    );
    assert!(
        chars
            .mana_abilities
            .iter()
            .any(|ma| ma.produces.get(&ManaColor::Black).copied() == Some(1)),
        "Urborg's granted Swamp subtype must produce the CR 305.6 intrinsic {{B}}: {:?}",
        chars.mana_abilities
    );
}

/// Same shape as P1 for Yavimaya, Cradle of Growth's `AddSubtypes(Forest)`/`AllLands` static.
#[test]
fn p2_yavimaya_grants_forest_intrinsic_green_to_a_plains() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .object(
            ObjectSpec::land(p1, "Test Plains").with_subtypes(vec![SubType("Plains".to_string())]),
        )
        .add_continuous_effect(add_subtypes_effect(1, 1, EffectFilter::AllLands, "Forest"))
        .build()
        .unwrap();

    let land_id = find_object(&state, "Test Plains");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars
            .mana_abilities
            .iter()
            .any(|ma| ma.produces.get(&ManaColor::White).copied() == Some(1)),
        "the land's own Plains subtype must still produce {{W}}: {:?}",
        chars.mana_abilities
    );
    assert!(
        chars
            .mana_abilities
            .iter()
            .any(|ma| ma.produces.get(&ManaColor::Green).copied() == Some(1)),
        "Yavimaya's granted Forest subtype must produce the CR 305.6 intrinsic {{G}}: {:?}",
        chars.mana_abilities
    );
}

/// Dryad of the Ilysian Grove's `AddSubtypes(all five basics)`/`LandsYouControl` static. Also
/// proves the `LandsYouControl` filter is real: a land controlled by ANOTHER player must NOT
/// gain the five basic subtypes (or their mana abilities).
#[test]
fn p3_dryad_grants_all_five_basics_only_to_lands_its_controller_controls() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Dryad Source", 2, 4))
        .object(
            ObjectSpec::land(p1, "My Plains").with_subtypes(vec![SubType("Plains".to_string())]),
        )
        // Deliberately NOT a basic land itself -- if `LandsYouControl` wrongly matched this
        // land, it would go from zero mana abilities to five; if it stays isolated (correct),
        // it stays at zero. A land that already prints "Plains" would gain {W} from CR 305.6
        // regardless of Dryad, which would not isolate the LandsYouControl claim.
        .object(ObjectSpec::land(p2, "Their Nonbasic Land"))
        .build()
        .unwrap();

    let dryad_id = find_object(&state, "Dryad Source");
    // `EffectFilter::LandsYouControl` resolves its controller from `effect.source` at
    // layer-application time, so `source` must be a real object id -- can't be known until after
    // `build()`, hence the post-build injection (mirrors `grant_activated_ability.rs`'s
    // `cryptolith_grant` idiom).
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(dryad_id),
        timestamp: 1,
        layer: EffectLayer::TypeChange,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::LandsYouControl,
        modification: LayerModification::AddSubtypes(
            [
                SubType("Plains".to_string()),
                SubType("Island".to_string()),
                SubType("Swamp".to_string()),
                SubType("Mountain".to_string()),
                SubType("Forest".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
        is_cda: false,
        affected_set: None,
        condition: None,
    });

    let my_land_id = find_object(&state, "My Plains");
    let their_land_id = find_object(&state, "Their Nonbasic Land");
    let my_chars = calculate_characteristics(&state, my_land_id).unwrap();
    let their_chars = calculate_characteristics(&state, their_land_id).unwrap();

    for color in [
        ManaColor::White,
        ManaColor::Blue,
        ManaColor::Black,
        ManaColor::Red,
        ManaColor::Green,
    ] {
        assert!(
            my_chars
                .mana_abilities
                .iter()
                .any(|ma| ma.produces.get(&color).copied() == Some(1)),
            "the Dryad's controller's own land must gain all five basic mana abilities, \
             missing {color:?}: {:?}",
            my_chars.mana_abilities
        );
    }
    assert!(
        their_chars.mana_abilities.is_empty(),
        "LandsYouControl must NOT apply to a land controlled by another player: {:?}",
        their_chars.mana_abilities
    );
}

// ── P4: two-moon fixture (OOS-DX27-10) ─────────────────────────────────────────────────────────

/// `OOS-DX27-10`'s closure: Blood Moon AND Magus of the Moon on the battlefield together must
/// grant exactly ONE `{T}: Add {R}`, not two. `pb_dx27_blood_moon_type_scope.rs`'s own `t6`
/// structurally cannot see this -- it builds a ONE-moon board (its doc comment says so
/// explicitly). This is the minimum new evidence the memo calls for: a two-moon fixture.
#[test]
fn p4_two_moons_together_grant_exactly_one_red_mana_ability() {
    let ancient_den = mtg_engine::cards::defs::ancient_den::card();
    let all_defs = defs_of(&[&ancient_den]);
    let registry = CardRegistry::new(vec![ancient_den.clone()]);
    let p1 = p(1);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Ancient Den",
            "ancient-den",
            ZoneId::Battlefield,
            &all_defs,
        ))
        // Blood Moon-shaped and Magus of the Moon-shaped statics -- both apply the SAME
        // SetLandTypes(Mountain) to the SAME nonbasic land, at different timestamps.
        .add_continuous_effect(set_land_types_effect(1, 5, "Mountain"))
        .add_continuous_effect(set_land_types_effect(2, 10, "Mountain"))
        .build()
        .unwrap();

    let land_id = find_object(&state, "Ancient Den");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert_eq!(
        chars.mana_abilities.len(),
        1,
        "two independent 'nonbasic lands are Mountains' sources must not double-grant the \
         intrinsic {{T}}: Add {{R}} (OOS-DX27-10): {:?}",
        chars.mana_abilities
    );
    assert_eq!(
        chars.mana_abilities.front().unwrap().produces,
        imbl::ordmap! { ManaColor::Red => 1 },
        "the surviving ability must produce exactly {{R}}: {:?}",
        chars.mana_abilities
    );
}

// ── P5: idempotence + ability-index stability (OOS-DX26-3) ────────────────────────────────────

/// A basic Swamp under Urborg still has EXACTLY one `{T}: Add {B}` ability, and it is at
/// INDEX 0 -- the printed one, not a derived append. `Command::TapForMana.ability_index` is a
/// dense index into `mana_abilities`, so a client that already knows index 0 taps for {B} must
/// keep working (OOS-DX26-3 hazard).
#[test]
fn p5_urborg_plus_basic_swamp_still_has_exactly_one_black_ability_at_index_zero() {
    let swamp = mtg_engine::cards::defs::swamp::card();
    let all_defs = defs_of(&[&swamp]);
    let registry = CardRegistry::new(vec![swamp.clone()]);
    let p1 = p(1);

    let base_spec = enrich_spec_from_def(ObjectSpec::card(p1, "Swamp"), &all_defs);
    let base_ability = base_spec
        .mana_abilities
        .first()
        .cloned()
        .expect("Swamp's def lowers exactly one printed mana ability");

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Swamp",
            "swamp-1",
            ZoneId::Battlefield,
            &all_defs,
        ))
        .add_continuous_effect(add_subtypes_effect(1, 1, EffectFilter::AllLands, "Swamp"))
        .build()
        .unwrap();

    let land_id = find_object(&state, "Swamp");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert_eq!(
        chars.mana_abilities.len(),
        1,
        "D3/D4 idempotence: Urborg's redundant Swamp grant must not duplicate the printed \
         {{T}}: Add {{B}} ability: {:?}",
        chars.mana_abilities
    );
    assert_eq!(
        chars.mana_abilities.front(),
        Some(&base_ability),
        "the surviving ability must be the ORIGINAL printed one, still at index 0: {:?}",
        chars.mana_abilities
    );
}

// ── P6: CR 305.7 removal, through the real cast path ───────────────────────────────────────────

/// Ancient Den under Blood Moon loses its printed {W} and has exactly {R}. Same assertion as
/// `pb_dx27_blood_moon_type_scope.rs`'s `t5`/`t6`, reproduced here as PB-DX43's own regression
/// pin because the RED ability's SOURCE changed: it used to come from an explicit
/// `AddManaAbility` static on `blood_moon.rs`; it now comes solely from the CR 305.6 intrinsic
/// derivation, through the real cast-and-resolve path (not a synthetic fixture).
#[test]
fn p6_ancient_den_under_blood_moon_loses_white_and_has_exactly_red() {
    let blood_moon = mtg_engine::cards::defs::blood_moon::card();
    let ancient_den = mtg_engine::cards::defs::ancient_den::card();
    let all_defs = defs_of(&[&blood_moon, &ancient_den]);
    let registry = CardRegistry::new(vec![blood_moon.clone(), ancient_den.clone()]);
    let p1 = p(1);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Blood Moon",
            "blood-moon",
            ZoneId::Hand(p1),
            &all_defs,
        ))
        .object(card_spec(
            p1,
            "Ancient Den",
            "ancient-den",
            ZoneId::Battlefield,
            &all_defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let state = cast_and_resolve_2r(state, p1, "Blood Moon", &[p1, p(2)]);
    let land_id = find_object(&state, "Ancient Den");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        !chars
            .mana_abilities
            .iter()
            .any(|ma| ma.produces.contains_key(&ManaColor::White)),
        "the printed {{T}}: Add {{W}} must be gone under Blood Moon: {:?}",
        chars.mana_abilities
    );
    assert_eq!(
        chars.mana_abilities.len(),
        1,
        "exactly one mana ability must survive -- the intrinsic {{R}}: {:?}",
        chars.mana_abilities
    );
    assert_eq!(
        chars.mana_abilities.front().unwrap().produces,
        imbl::ordmap! { ManaColor::Red => 1 },
        "the surviving ability must produce exactly {{R}}: {:?}",
        chars.mana_abilities
    );
}

// ── P7: CR 305.7's final sentence -- Layer-6 grants from OTHER effects survive ────────────────

/// CR 305.7's last sentence: "Note that this doesn't remove any abilities that were granted to
/// the land by other effects." A Layer-6 grant with an EARLIER timestamp than a Blood-Moon-shaped
/// Layer-4 `SetLandTypes` must still survive, because the clearing runs at Layer 4 -- strictly
/// BEFORE Layer 6 -- rather than as a timestamp-ordered Layer-6 peer of the grant.
///
/// **This probe fails on the pre-PB-DX43 shape.** Before this batch, CR 305.7's ability removal
/// was modelled as a SEPARATE Layer-6 `RemoveAllAbilities` static on `blood_moon.rs` itself,
/// timestamp-ordered against every OTHER Layer-6 effect. An executed reproduction (three raw
/// effects: an early-timestamp external grant, a mid-timestamp `RemoveAllAbilities` mimicking
/// Blood Moon's own removal component, and a later-timestamp `AddManaAbility` mimicking Blood
/// Moon's own red grant -- the "deliberately mis-ordered" pair the old `blood_moon.rs` comment
/// described) strips the EARLIER external grant while Blood Moon's own later grant survives --
/// see `memory/primitives/pb-DX43-execution-notes.md` row P7 for the executed proof.
#[test]
fn p7_earlier_timestamped_layer_six_grant_survives_blood_moons_layer_four_clearing() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::land(p1, "Nonbasic Land"))
        // External Layer-6 grant (Cryptolith-Rite-shaped), timestamp 1 -- EARLIER than the
        // moon-shaped effect below.
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 1,
            layer: EffectLayer::Ability,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::AddManaAbility(ManaAbility::tap_for(ManaColor::Green)),
            is_cda: false,
            affected_set: None,
            condition: None,
        })
        // Blood-Moon-shaped Layer-4 SetLandTypes, timestamp 10 -- LATER.
        .add_continuous_effect(set_land_types_effect(2, 10, "Mountain"))
        .build()
        .unwrap();

    let land_id = find_object(&state, "Nonbasic Land");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars
            .mana_abilities
            .iter()
            .any(|ma| ma.produces.get(&ManaColor::Green).copied() == Some(1)),
        "CR 305.7's last sentence: a Layer-6 ability granted by ANOTHER effect must survive the \
         moon's Layer-4 clearing regardless of relative timestamp: {:?}",
        chars.mana_abilities
    );
    assert!(
        chars
            .mana_abilities
            .iter()
            .any(|ma| ma.produces.get(&ManaColor::Red).copied() == Some(1)),
        "the intrinsic {{T}}: Add {{R}} for the newly-set Mountain type must ALSO be present: {:?}",
        chars.mana_abilities
    );
}

// ── P8: CR 708.2a -- face-down derives nothing ─────────────────────────────────────────────────

/// A face-down permanent whose def is a Swamp derives nothing: CR 708.2a's face-down blank
/// (`layers.rs:329-342`, PRE-EXISTING) replaces `card_types` with just `{Creature}` and
/// `subtypes` with `{}` BEFORE the layer loop even starts, so `derive_intrinsic_land_mana_
/// abilities`'s "has the Land card type" guard is never satisfied. D6's "falls out for free"
/// claim, asserted rather than assumed.
#[test]
fn p8_face_down_swamp_derives_nothing() {
    let swamp = mtg_engine::cards::defs::swamp::card();
    let all_defs = defs_of(&[&swamp]);
    let registry = CardRegistry::new(vec![swamp.clone()]);
    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Swamp",
            "swamp-1",
            ZoneId::Battlefield,
            &all_defs,
        ))
        .build()
        .unwrap();

    let land_id = find_object(&state, "Swamp");
    state
        .objects_mut()
        .get_mut(&land_id)
        .unwrap()
        .status
        .face_down = true;
    state.objects_mut().get_mut(&land_id).unwrap().face_down_as = Some(FaceDownKind::Morph);

    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.mana_abilities.is_empty(),
        "CR 708.2a: a face-down permanent has no Land card type and no subtypes, so CR 305.6's \
         derivation (which requires BOTH conjuncts) must find nothing: {:?}",
        chars.mana_abilities
    );
    assert!(
        !chars.card_types.contains(&CardType::Land),
        "face-down blanking replaces card_types with just Creature: {:?}",
        chars.card_types
    );
}

// ── P9: subordination to Layer 6 (D1) ──────────────────────────────────────────────────────────

/// A layer-6 `RemoveAllAbilities` continuous effect (Humility-shaped, NOT a moon) applied to a
/// basic Swamp removes the derived/printed {B}. Proves D1's placement decision is real: the
/// derivation runs at the END of Layer 4, strictly BEFORE Layer 6, so it is NOT immune to Layer-6
/// ability removal -- if the derivation were instead placed AFTER the whole layer walk, this test
/// would go red (the intrinsic would survive Humility, which is CR-wrong: CR 613.1f's Layer 6
/// applies after CR 305.6's Layer-4 grant, not around it).
#[test]
fn p9_layer_six_remove_all_abilities_still_strips_the_derived_ability() {
    let swamp = mtg_engine::cards::defs::swamp::card();
    let all_defs = defs_of(&[&swamp]);
    let registry = CardRegistry::new(vec![swamp.clone()]);
    let p1 = p(1);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Swamp",
            "swamp-1",
            ZoneId::Battlefield,
            &all_defs,
        ))
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 1,
            layer: EffectLayer::Ability,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::RemoveAllAbilities,
            is_cda: false,
            affected_set: None,
            condition: None,
        })
        .build()
        .unwrap();

    let land_id = find_object(&state, "Swamp");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.mana_abilities.is_empty(),
        "D1: the derivation must NOT be immune to a Layer-6 ability-removal effect: {:?}",
        chars.mana_abilities
    );
}

// ── P10: multi-basic and derivation order (D5) ─────────────────────────────────────────────────

/// A land with BOTH Forest and Plains subtypes gets both {G} and {W}, appended in CR 305.6's OWN
/// listed order (Plains, Island, Swamp, Mountain, Forest) rather than alphabetically. Forest
/// sorts BEFORE Plains alphabetically ('F' < 'P'), so this pair is chosen specifically to
/// distinguish "iterates `BASIC_LAND_TYPES`" from "iterates `chars.subtypes`'s `OrdSet` order".
#[test]
fn p10_multi_basic_land_gets_both_colors_in_cr_305_6_order() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .object(ObjectSpec::land(p1, "Synthetic Dual").with_subtypes(vec![
            SubType("Forest".to_string()),
            SubType("Plains".to_string()),
        ]))
        .build()
        .unwrap();

    let land_id = find_object(&state, "Synthetic Dual");
    let chars = calculate_characteristics(&state, land_id).unwrap();
    let resolved: Vec<ManaAbility> = chars.mana_abilities.iter().cloned().collect();

    assert_eq!(
        resolved.len(),
        2,
        "both basic subtypes must produce a mana ability: {:?}",
        resolved
    );
    assert_eq!(
        resolved[0].produces.get(&ManaColor::White).copied(),
        Some(1),
        "CR 305.6's own listed order is Plains, Island, Swamp, Mountain, Forest -- White \
         (Plains) must be appended BEFORE Green (Forest), even though 'Forest' sorts before \
         'Plains' alphabetically: {:?}",
        resolved
    );
    assert_eq!(
        resolved[1].produces.get(&ManaColor::Green).copied(),
        Some(1),
        "Green (Forest) must be second: {:?}",
        resolved
    );
}

// ── P11: both conjuncts required ───────────────────────────────────────────────────────────────

/// CR 305.6 requires BOTH the land card type AND a basic land type. A Creature carrying only the
/// Swamp SUBTYPE (no Land card type) must derive nothing.
#[test]
fn p11_non_land_object_with_a_basic_land_subtype_derives_nothing() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .object(
            ObjectSpec::creature(p1, "Swamp Thing", 3, 3)
                .with_subtypes(vec![SubType("Swamp".to_string())]),
        )
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Swamp Thing");
    let chars = calculate_characteristics(&state, obj_id).unwrap();

    assert!(
        chars.mana_abilities.is_empty(),
        "a Creature with the Swamp subtype but no Land card type must derive nothing: {:?}",
        chars.mana_abilities
    );
}

// ── P12: D4 -- conditioned/costed abilities do not discharge the intrinsic ────────────────────

/// D4: an existing mana ability that is CONDITIONED (`activation_condition: Some(..)`) or COSTED
/// (`life_cost > 0`) does not discharge the intrinsic -- CR 305.6's ability is unconditional and
/// free, so the land ends up with the restricted/costed one AND a derived unrestricted one.
#[test]
fn p12_conditioned_or_costed_existing_ability_does_not_discharge_the_intrinsic() {
    let p1 = p(1);

    // (a) A restricted (activation_condition-gated) {T}: Add {B} does not discharge the
    // intrinsic.
    let mut conditioned = ManaAbility::tap_for(ManaColor::Black);
    conditioned.activation_condition = Some(Box::new(Condition::IsYourTurn));
    let state_a = GameStateBuilder::new()
        .add_player(p1)
        .object(
            ObjectSpec::land(p1, "Conditioned Swamp")
                .with_subtypes(vec![SubType("Swamp".to_string())])
                .with_mana_ability(conditioned),
        )
        .build()
        .unwrap();
    let id_a = find_object(&state_a, "Conditioned Swamp");
    let chars_a = calculate_characteristics(&state_a, id_a).unwrap();
    assert_eq!(
        chars_a.mana_abilities.len(),
        2,
        "a CONDITIONED existing {{T}}: Add {{B}} must not discharge CR 305.6's unconditional \
         intrinsic -- the land must end up with BOTH the restricted one AND a new unrestricted \
         one: {:?}",
        chars_a.mana_abilities
    );
    assert!(
        chars_a
            .mana_abilities
            .iter()
            .any(|ma| ma.activation_condition.is_some()),
        "the restricted ability must survive unchanged: {:?}",
        chars_a.mana_abilities
    );
    assert!(
        chars_a
            .mana_abilities
            .iter()
            .any(|ma| ma.activation_condition.is_none()
                && ma.life_cost == 0
                && ma.produces.get(&ManaColor::Black).copied() == Some(1)),
        "a SECOND, unconditional {{T}}: Add {{B}} must also be present: {:?}",
        chars_a.mana_abilities
    );

    // (b) A costed (life_cost > 0) {T}: Add {B} does not discharge the intrinsic either.
    let mut costed = ManaAbility::tap_for(ManaColor::Black);
    costed.life_cost = 1;
    let state_b = GameStateBuilder::new()
        .add_player(p1)
        .object(
            ObjectSpec::land(p1, "Costed Swamp")
                .with_subtypes(vec![SubType("Swamp".to_string())])
                .with_mana_ability(costed),
        )
        .build()
        .unwrap();
    let id_b = find_object(&state_b, "Costed Swamp");
    let chars_b = calculate_characteristics(&state_b, id_b).unwrap();
    assert_eq!(
        chars_b.mana_abilities.len(),
        2,
        "a COSTED (Pay 1 life) existing {{T}}: Add {{B}} must not discharge the intrinsic \
         either: {:?}",
        chars_b.mana_abilities
    );
    assert!(
        chars_b.mana_abilities.iter().any(|ma| ma.life_cost == 0
            && ma.activation_condition.is_none()
            && ma.produces.get(&ManaColor::Black).copied() == Some(1)),
        "a SECOND, free {{T}}: Add {{B}} must also be present: {:?}",
        chars_b.mana_abilities
    );
}

// ── P13: SetLandTypes' CR 305.7 precondition is real (D2) ─────────────────────────────────────

/// A `SetLandTypes` payload of a NONBASIC land type (Gate) does not trigger CR 305.7's ability
/// clearing and derives nothing -- CR 305.7's own precondition is "sets a land's subtype to one
/// or more of the BASIC land types", and Gate is not basic. The `SetLandTypes` arm's subtype-SET
/// behaviour must still function (the old land type is still replaced).
#[test]
fn p13_nonbasic_set_land_types_payload_does_not_trigger_cr_305_7_clearing_or_derive_anything() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .object(
            ObjectSpec::land(p1, "Would-Be Gate")
                .with_subtypes(vec![SubType("Cave".to_string())])
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Blue)),
        )
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 1,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::SetLandTypes(imbl::OrdSet::unit(SubType(
                "Gate".to_string(),
            ))),
            is_cda: false,
            affected_set: None,
            condition: None,
        })
        .build()
        .unwrap();

    let land_id = find_object(&state, "Would-Be Gate");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.subtypes.contains(&SubType("Gate".to_string())),
        "SetLandTypes must still SET the land subtype even for a nonbasic payload: {:?}",
        chars.subtypes
    );
    assert!(
        !chars.subtypes.contains(&SubType("Cave".to_string())),
        "the old land subtype must still be replaced: {:?}",
        chars.subtypes
    );
    assert_eq!(
        chars.mana_abilities.len(),
        1,
        "CR 305.7's ability-clearing precondition is 'sets ... to one or more of the BASIC \
         land types' -- Gate is not basic, so the printed {{T}}: Add {{U}} must survive \
         untouched and nothing must be derived: {:?}",
        chars.mana_abilities
    );
    assert_eq!(
        chars
            .mana_abilities
            .front()
            .unwrap()
            .produces
            .get(&ManaColor::Blue)
            .copied(),
        Some(1),
        "the surviving ability must be the original Blue one: {:?}",
        chars.mana_abilities
    );
}
