//! Tests for the PB-DX27 rider `OOS-ADJ-7`: Blood Moon / Magus of the Moon were
//! stripping the Artifact/Creature card type from artifact lands and creature lands.
//!
//! The old implementation used `LayerModification::SetTypeLine`, which replaces the
//! WHOLE type line (supertypes + card types + subtypes). CR 305.7's "Nonbasic lands
//! are Mountains" only ever changes LAND SUBTYPES (CR 205.1a); per the 2020-08-07
//! ruling: "This effect doesn't affect names or supertypes ... Nonbasic lands will
//! lose any other land types and abilities they had. They will gain the land type
//! Mountain and gain the ability '{T}: Add {R}.'"
//!
//! New primitive: `LayerModification::SetLandTypes(OrdSet<SubType>)` (Layer 4, CR
//! 205.1a) — SETS the LAND-type subset of `subtypes`, leaving `card_types`,
//! `supertypes`, and non-land subtypes untouched. Companion to `SetCreatureTypes`/
//! `SetCardTypes` (PB-AC7).
//!
//! Hash: `HASH_SCHEMA_VERSION` bumped 74 -> 75 (new `LayerModification` variant,
//! arm tag `32u8`). `PROTOCOL_VERSION` unmoved (`LayerModification` is not in the
//! SR-8 wire closure).

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::hash::HashInto;
use mtg_engine::{
    calculate_characteristics, enrich_spec_from_def, process_command, CardDefinition, CardId,
    CardRegistry, CardType, Command, ContinuousEffect, EffectDuration, EffectFilter, EffectId,
    EffectLayer, GameEvent, GameState, GameStateBuilder, LayerModification, ManaColor, ObjectId,
    ObjectSpec, PlayerId, Step, SubType, SuperType, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Registers every `CardDefinition` under BOTH the def's own name and (defensively)
/// any alias the caller passes — mirrors `pb_ac7_card_integration.rs`'s `defs_of`
/// but for multiple simultaneous defs.
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

/// Gives `player` enough mana for `{2}{R}` (Blood Moon's and Magus of the Moon's
/// shared cost) and casts+resolves the named card from hand. `all_players` is the
/// full player list (needed to pass priority around the table to resolve the
/// stack — a 1-player game is treated as already over, so every test uses a
/// second, empty-board player).
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

// ── T1/T2: artifact lands keep their Artifact card type ────────────────────────

/// CR 305.7 + 2020-08-07 ruling: Blood Moon changes LAND SUBTYPES only. An
/// artifact land (Ancient Den: `types(&[CardType::Artifact, CardType::Land])`,
/// `Complete` by derive) must keep `CardType::Artifact` after Blood Moon resolves
/// — the pre-rider `SetTypeLine` implementation silently dropped it.
#[test]
fn t1_ancient_den_class_artifact_land_keeps_artifact_card_type_under_blood_moon() {
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
        chars.card_types.contains(&CardType::Artifact),
        "Ancient Den must stay an Artifact under Blood Moon (OOS-ADJ-7): {:?}",
        chars.card_types
    );
    assert!(
        chars.card_types.contains(&CardType::Land),
        "Ancient Den must stay a Land: {:?}",
        chars.card_types
    );
    assert_eq!(
        chars.subtypes,
        imbl::OrdSet::unit(SubType("Mountain".to_string())),
        "Ancient Den's land subtypes must be exactly {{Mountain}}"
    );
}

/// Same as T1 but for Treasure Vault (`types_sub(&[Artifact, Land], &[])`), a
/// second independently-authored artifact land in the corpus.
#[test]
fn t2_treasure_vault_keeps_artifact_card_type_under_blood_moon() {
    let blood_moon = mtg_engine::cards::defs::blood_moon::card();
    let treasure_vault = mtg_engine::cards::defs::treasure_vault::card();
    let all_defs = defs_of(&[&blood_moon, &treasure_vault]);
    let registry = CardRegistry::new(vec![blood_moon.clone(), treasure_vault.clone()]);
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
            "Treasure Vault",
            "treasure-vault",
            ZoneId::Battlefield,
            &all_defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let state = cast_and_resolve_2r(state, p1, "Blood Moon", &[p1, p(2)]);
    let land_id = find_object(&state, "Treasure Vault");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "Treasure Vault must stay an Artifact under Blood Moon (OOS-ADJ-7): {:?}",
        chars.card_types
    );
    assert!(
        chars.card_types.contains(&CardType::Land),
        "Treasure Vault must stay a Land: {:?}",
        chars.card_types
    );
    assert_eq!(
        chars.subtypes,
        imbl::OrdSet::unit(SubType("Mountain".to_string())),
        "Treasure Vault's land subtypes must be exactly {{Mountain}}"
    );
}

// ── T3: Dryad Arbor keeps its Creature card type AND its creature subtype ──────

/// The seed filing (`OOS-ADJ-7`) named only the two artifact lands. Dryad Arbor
/// (`types_sub(&[Land, Creature], &["Forest", "Dryad"])`) is a THIRD live-wrong
/// pair the filing missed: under the old `SetTypeLine` implementation it lost its
/// Creature card type too. `SetLandTypes` must preserve `CardType::Creature` and
/// the non-land `Dryad` subtype while replacing the land subtype `Forest` with
/// `Mountain`.
#[test]
fn t3_dryad_arbor_keeps_creature_card_type_and_dryad_subtype_under_blood_moon() {
    let blood_moon = mtg_engine::cards::defs::blood_moon::card();
    let dryad_arbor = mtg_engine::cards::defs::dryad_arbor::card();
    let all_defs = defs_of(&[&blood_moon, &dryad_arbor]);
    let registry = CardRegistry::new(vec![blood_moon.clone(), dryad_arbor.clone()]);
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
            "Dryad Arbor",
            "dryad-arbor",
            ZoneId::Battlefield,
            &all_defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let state = cast_and_resolve_2r(state, p1, "Blood Moon", &[p1, p(2)]);
    let land_id = find_object(&state, "Dryad Arbor");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.card_types.contains(&CardType::Creature),
        "Dryad Arbor must stay a Creature under Blood Moon (OOS-ADJ-7, missed by the \
         original filing): {:?}",
        chars.card_types
    );
    assert!(
        chars.card_types.contains(&CardType::Land),
        "Dryad Arbor must stay a Land: {:?}",
        chars.card_types
    );
    assert_eq!(
        chars.subtypes,
        [
            SubType("Mountain".to_string()),
            SubType("Dryad".to_string())
        ]
        .into_iter()
        .collect::<imbl::OrdSet<_>>(),
        "Forest (a land subtype) is replaced by Mountain; Dryad (a creature subtype, \
         not a land subtype) survives untouched"
    );
    assert!(
        !chars.subtypes.contains(&SubType("Forest".to_string())),
        "the land subtype Forest must be gone"
    );
}

// ── T4: supertypes preserved ────────────────────────────────────────────────────

/// 2020-08-07 ruling, first sentence: "This effect doesn't affect names or
/// supertypes." A Legendary nonbasic land must stay Legendary under Blood Moon.
/// Uses a synthetic fixture (not a real card def) — `EffectFilter::AllNonbasicLands`
/// matches on type/supertype, not on card identity, so this isolates the
/// supertype-preservation claim from any particular printed card.
#[test]
fn t4_supertypes_preserved_under_blood_moon() {
    let blood_moon = mtg_engine::cards::defs::blood_moon::card();
    let all_defs = defs_of(&[&blood_moon]);
    let registry = CardRegistry::new(vec![blood_moon.clone()]);
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
        .object(
            ObjectSpec::land(p1, "Legendary Nonbasic Land")
                .with_supertypes(vec![SuperType::Legendary])
                .with_subtypes(vec![SubType("Cave".to_string())]),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let state = cast_and_resolve_2r(state, p1, "Blood Moon", &[p1, p(2)]);
    let land_id = find_object(&state, "Legendary Nonbasic Land");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.supertypes.contains(&SuperType::Legendary),
        "the ruling says supertypes are untouched: {:?}",
        chars.supertypes
    );
    assert_eq!(
        chars.subtypes,
        imbl::OrdSet::unit(SubType("Mountain".to_string())),
        "the Cave land subtype must be replaced by Mountain"
    );
}

// ── T5/T6: abilities removed, granted mana ability present, ordering proven ────

/// CR 613.1f (existing, pre-rider behaviour): Blood Moon's Layer-6 RemoveAllAbilities
/// must still strip the land's ORIGINAL mana ability. Regression coverage — this
/// batch must not weaken it.
#[test]
fn t5_ancient_dens_original_white_mana_ability_is_removed() {
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
        !chars.mana_abilities.iter().any(|ma| ma
            .produces
            .get(&ManaColor::White)
            .copied()
            .unwrap_or(0)
            > 0),
        "Ancient Den's printed '{{T}}: Add {{W}}' must be gone under RemoveAllAbilities: {:?}",
        chars.mana_abilities
    );
}

/// 2020-08-07 ruling, third sentence: "They will gain the land type Mountain and
/// gain the ability '{T}: Add {R}.'" Ancient Den must end up with EXACTLY one mana
/// ability, the granted `{T}: Add {R}` — proving both the removal (T5) and the
/// grant survive together, in the right order.
#[test]
fn t6_ancient_den_gains_exactly_the_granted_tap_add_red_ability() {
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

    assert_eq!(
        chars.mana_abilities.len(),
        1,
        "exactly one mana ability must survive (the grant, after the removal): {:?}",
        chars.mana_abilities
    );
    let granted = chars.mana_abilities.front().unwrap();
    assert!(
        granted.requires_tap,
        "the granted ability is '{{T}}: Add {{R}}' — it requires tapping"
    );
    assert!(
        !granted.sacrifice_self,
        "the granted ability does not sacrifice the land"
    );
    assert!(
        !granted.any_color,
        "the granted ability produces Red specifically, not any color"
    );
    assert_eq!(
        granted.produces,
        imbl::ordmap! { ManaColor::Red => 1 },
        "the granted ability produces exactly {{R}}"
    );
}

// ── T7: Blood Moon + Urborg dependency (CR 613.8), re-derived for SetLandTypes ──

/// CR 613.8: Blood Moon's `SetLandTypes` depends on Urborg's `AddSubtypes(Swamp)`,
/// so Urborg must apply first regardless of timestamp order (mirrors
/// `test_613_blood_moon_plus_urborg_blood_moon_older_dependency_wins` in
/// `tests/rules/layers.rs`, which pins the ORIGINAL `SetTypeLine`-based arm and is
/// untouched by this rider). This test pins the NEW `SetLandTypes`-based arm added
/// alongside it. Blood Moon entered FIRST (lower timestamp): without the
/// dependency arm, plain timestamp order would apply Blood Moon's SET before
/// Urborg's ADD, giving the CR-wrong "Mountain, Swamp". With the dependency arm,
/// Urborg's ADD is forced before Blood Moon's SET regardless of timestamp, giving
/// the correct "Mountain" only.
#[test]
fn t7_blood_moon_still_overrides_urborg_dependency() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .object(ObjectSpec::land(p(1), "Nonbasic Land"))
        // Blood Moon effect (timestamp 5, OLDER — entered first).
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(1),
            source: None,
            timestamp: 5,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllNonbasicLands,
            modification: LayerModification::SetLandTypes(imbl::OrdSet::unit(SubType(
                "Mountain".to_string(),
            ))),
            is_cda: false,
            affected_set: None,
            condition: None,
        })
        // Urborg effect (timestamp 10, NEWER — entered second).
        .add_continuous_effect(ContinuousEffect {
            id: EffectId(2),
            source: None,
            timestamp: 10,
            layer: EffectLayer::TypeChange,
            duration: EffectDuration::Indefinite,
            filter: EffectFilter::AllLands,
            modification: LayerModification::AddSubtypes(imbl::OrdSet::unit(SubType(
                "Swamp".to_string(),
            ))),
            is_cda: false,
            affected_set: None,
            condition: None,
        })
        .build()
        .unwrap();

    let land_id = find_object(&state, "Nonbasic Land");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.subtypes.contains(&SubType("Mountain".to_string())),
        "land should be a Mountain: {:?}",
        chars.subtypes
    );
    assert!(
        !chars.subtypes.contains(&SubType("Swamp".to_string())),
        "land should NOT be a Swamp -- the SetLandTypes/AddSubtypes dependency must \
         force Urborg's add before Blood Moon's set, regardless of timestamp: {:?}",
        chars.subtypes
    );
}

// ── T8: Magus of the Moon gets its own probe ────────────────────────────────────

/// Magus of the Moon is a separate card def with the identical three-static
/// pattern — do not test only Blood Moon and assume the sibling is fine.
#[test]
fn t8_magus_of_the_moon_keeps_artifact_land_card_type_and_grants_tap_add_red() {
    let magus = mtg_engine::cards::defs::magus_of_the_moon::card();
    let ancient_den = mtg_engine::cards::defs::ancient_den::card();
    let all_defs = defs_of(&[&magus, &ancient_den]);
    let registry = CardRegistry::new(vec![magus.clone(), ancient_den.clone()]);
    let p1 = p(1);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Magus of the Moon",
            "magus-of-the-moon",
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

    let state = cast_and_resolve_2r(state, p1, "Magus of the Moon", &[p1, p(2)]);
    let land_id = find_object(&state, "Ancient Den");
    let chars = calculate_characteristics(&state, land_id).unwrap();

    assert!(
        chars.card_types.contains(&CardType::Artifact),
        "Ancient Den must stay an Artifact under Magus of the Moon: {:?}",
        chars.card_types
    );
    assert_eq!(
        chars.subtypes,
        imbl::OrdSet::unit(SubType("Mountain".to_string())),
        "land subtypes must be exactly {{Mountain}}"
    );
    assert_eq!(
        chars.mana_abilities.len(),
        1,
        "exactly one mana ability must survive: {:?}",
        chars.mana_abilities
    );
    let granted = chars.mana_abilities.front().unwrap();
    assert_eq!(
        granted.produces,
        imbl::ordmap! { ManaColor::Red => 1 },
        "the granted ability produces exactly {{R}}"
    );
}

// ── T9: HashInto field coverage ─────────────────────────────────────────────────

/// `LayerModification::SetLandTypes` must be hashed. Required because
/// `canonical_fixture()` cannot populate `continuous_effects`
/// (`hash_schema.rs`'s five named exclusions), so this variant's own bytes are
/// otherwise inside NO gate -- the same situation the v74 row records for a
/// sibling field (mirrors `pb_dx25c_retarget_legality.rs::t10_...`).
#[test]
fn t9_set_land_types_is_hashed() {
    let a = LayerModification::SetLandTypes(imbl::OrdSet::unit(SubType("Mountain".to_string())));
    let b = LayerModification::SetLandTypes(imbl::OrdSet::unit(SubType("Island".to_string())));
    let c =
        LayerModification::SetCreatureTypes(imbl::OrdSet::unit(SubType("Mountain".to_string())));

    let hash_of = |m: &LayerModification| -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        m.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    assert_ne!(
        hash_of(&a),
        hash_of(&b),
        "two SetLandTypes differing only in payload must hash differently"
    );
    assert_ne!(
        hash_of(&a),
        hash_of(&c),
        "SetLandTypes and SetCreatureTypes with the SAME payload string must hash \
         differently -- they carry distinct arm tags (32 vs 30)"
    );
    assert_eq!(
        hash_of(&a),
        hash_of(&a),
        "identical SetLandTypes values must hash identically (sanity)"
    );
}
