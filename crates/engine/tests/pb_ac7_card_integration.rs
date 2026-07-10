//! PB-AC7 card integration tests — Kenrith's Transformation, Eaten by Piranhas,
//! Darksteel Mutation, Sram, Senior Edificer, Leaf-Crowned Visionary.
//!
//! These exercise the *real* `CardDefinition`s in `crates/card-defs/src/defs/`
//! (not synthetic fixtures) through full `process_command` flows, validating that
//! the PB-AC7 primitives (`LayerModification::SetCreatureTypes`,
//! `LayerModification::SetCardTypes`,
//! `TriggerCondition::WheneverYouCastSpell.spell_subtype_filter`) are wired
//! correctly into each card's abilities.
//!
//! Final Showdown (mode 0) is a PARTIAL clause on a multi-mode card; not given a
//! dedicated integration test here (its now-expressible clause reuses a pattern
//! already covered by the unit tests in `pb_ac7_type_change_ability_removal.rs`),
//! per the backfill scope (fully-clean roster only).
//!
//! Vraska, Betrayal's Sting (-2) DOES get a dedicated integration test below
//! (`test_vraska_betrayals_sting_minus2_full_integration`) — see review finding
//! F-VR1 (`memory/card-authoring/review-pb-ac7-backfill.md`): the -2 ability
//! applies `RemoveAllAbilities` and a granted `AddManaAbility` at the SAME
//! timestamp (within one `Effect::Sequence`), so the granted ability's survival
//! depends on stable-sort insertion order rather than distinct timestamps. This
//! test locks that behavior in as a regression guard.

use mtg_engine::{
    calculate_characteristics, enrich_spec_from_def, process_command, CardDefinition, CardId,
    CardRegistry, CardType, Command, CounterType, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ObjectId, ObjectSpec, PlayerId, Step, SubType, SuperType, Target, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs_of(def: &CardDefinition) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

/// `ObjectSpec::card()` produces a *naked* object: no mana cost, no abilities,
/// no P/T. Without enrichment a cast pays nothing and `ActivateAbility` reports
/// `InvalidAbilityIndex`. Every test below routes specs through here.
fn card_spec(
    player: PlayerId,
    name: &str,
    card_id: &str,
    zone: ZoneId,
    def: &CardDefinition,
) -> ObjectSpec {
    enrich_spec_from_def(ObjectSpec::card(player, name), &defs_of(def))
        .with_card_id(CardId(card_id.to_string()))
        .in_zone(zone)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Hand(player))
        .count()
}

fn cast_spell(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), mtg_engine::GameStateError> {
    process_command(
        state,
        Command::CastSpell {
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
        },
    )
}

/// Pass priority for all listed players once.
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

// ── 1. Kenrith's Transformation ────────────────────────────────────────────────

#[test]
/// Oracle: "Enchant creature / When this Aura enters, draw a card. / Enchanted
/// creature loses all abilities and is a green Elk creature with base power and
/// toughness 3/3." Verifies the ETB draw, and the full Layer 4/5/6/7b composition
/// on the enchanted creature (including Legendary supertype preservation).
fn test_kenriths_transformation_full_integration() {
    let def = mtg_engine::cards::defs::kenriths_transformation::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::creature(p1, "Legendary Flyer", 5, 5)
                .with_keyword(KeywordAbility::Flying)
                .with_supertypes(vec![mtg_engine::SuperType::Legendary]),
        )
        .object(card_spec(
            p1,
            "Kenrith's Transformation",
            "kenriths-transformation",
            ZoneId::Hand(p1),
            &def,
        ))
        .object(ObjectSpec::creature(p1, "Library Filler", 1, 1).in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 1;
    state.players_mut().get_mut(&p1).unwrap().mana_pool.green = 1;
    state.turn_mut().priority_holder = Some(p1);

    let target_id = find_object(&state, "Legendary Flyer");
    let spell_id = find_object(&state, "Kenrith's Transformation");
    let hand_before = hand_count(&state, p1);

    let (state, _) = cast_spell(state, p1, spell_id, vec![Target::Object(target_id)]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);

    // ETB draw fired (net hand: -1 for casting the Aura, +1 for the draw = same size,
    // but the Aura left the hand so this nets to hand_before - the aura + the draw).
    assert_eq!(
        hand_count(&state, p1),
        hand_before, // -1 (cast) + 1 (draw) = net 0 change
        "When this Aura enters, draw a card"
    );

    let chars = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(chars.power, Some(3));
    assert_eq!(chars.toughness, Some(3));
    assert_eq!(
        chars.card_types,
        im::OrdSet::unit(CardType::Creature),
        "loses all other card types"
    );
    assert_eq!(
        chars.subtypes,
        im::OrdSet::unit(SubType("Elk".to_string())),
        "is ... an Elk creature"
    );
    assert!(
        chars.colors.contains(&mtg_engine::Color::Green),
        "is a green ... creature"
    );
    assert!(
        chars.keywords.is_empty(),
        "loses all abilities: Flying must be gone"
    );
    assert!(
        chars.supertypes.contains(&mtg_engine::SuperType::Legendary),
        "Legendary supertype must be preserved (SetCardTypes/SetCreatureTypes, not SetTypeLine)"
    );
}

// ── 2. Eaten by Piranhas ────────────────────────────────────────────────────────

#[test]
/// Oracle: "Flash / Enchant creature / Enchanted creature loses all abilities and
/// is a black Skeleton creature with base power and toughness 1/1." Verifies the
/// full Layer 4/5/6/7b composition.
fn test_eaten_by_piranhas_full_integration() {
    let def = mtg_engine::cards::defs::eaten_by_piranhas::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::creature(p1, "Red Dragon", 4, 4)
                .with_keyword(KeywordAbility::Flying)
                .with_colors(vec![mtg_engine::Color::Red]),
        )
        .object(card_spec(
            p1,
            "Eaten by Piranhas",
            "eaten-by-piranhas",
            ZoneId::Hand(p1),
            &def,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 1;
    state.players_mut().get_mut(&p1).unwrap().mana_pool.blue = 1;
    state.turn_mut().priority_holder = Some(p1);

    let target_id = find_object(&state, "Red Dragon");
    let spell_id = find_object(&state, "Eaten by Piranhas");

    let (state, _) = cast_spell(state, p1, spell_id, vec![Target::Object(target_id)]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(chars.power, Some(1));
    assert_eq!(chars.toughness, Some(1));
    assert_eq!(chars.card_types, im::OrdSet::unit(CardType::Creature));
    assert_eq!(
        chars.subtypes,
        im::OrdSet::unit(SubType("Skeleton".to_string()))
    );
    assert!(chars.colors.contains(&mtg_engine::Color::Black));
    assert!(
        !chars.colors.contains(&mtg_engine::Color::Red),
        "loses all other colors"
    );
    assert!(chars.keywords.is_empty(), "loses all abilities");
}

// ── 3. Darksteel Mutation ────────────────────────────────────────────────────────

#[test]
/// Oracle: "Enchant creature / Enchanted creature is an Insect artifact creature
/// with base power and toughness 0/1 and has indestructible, and it loses all
/// other abilities, card types, and creature types." Verifies indestructible
/// survives the "loses all OTHER abilities" removal (CR 613.7 granted-then-removed
/// ordering via ability-vec order).
fn test_darksteel_mutation_full_integration() {
    let def = mtg_engine::cards::defs::darksteel_mutation::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ObjectSpec::creature(p1, "Big Flyer", 5, 5).with_keyword(KeywordAbility::Flying))
        .object(card_spec(
            p1,
            "Darksteel Mutation",
            "darksteel-mutation",
            ZoneId::Hand(p1),
            &def,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 1;
    state.players_mut().get_mut(&p1).unwrap().mana_pool.white = 1;
    state.turn_mut().priority_holder = Some(p1);

    let target_id = find_object(&state, "Big Flyer");
    let spell_id = find_object(&state, "Darksteel Mutation");

    let (state, _) = cast_spell(state, p1, spell_id, vec![Target::Object(target_id)]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(chars.power, Some(0));
    assert_eq!(chars.toughness, Some(1));
    assert_eq!(
        chars.card_types,
        [CardType::Artifact, CardType::Creature]
            .into_iter()
            .collect(),
        "is an Insect artifact creature"
    );
    assert_eq!(
        chars.subtypes,
        im::OrdSet::unit(SubType("Insect".to_string()))
    );
    assert!(
        chars.keywords.contains(&KeywordAbility::Indestructible),
        "has indestructible (must survive the ability removal, CR 613.7)"
    );
    assert!(
        !chars.keywords.contains(&KeywordAbility::Flying),
        "loses all other abilities: Flying must be gone"
    );
}

// ── 4. Sram, Senior Edificer ─────────────────────────────────────────────────────

#[test]
/// Oracle: "Whenever you cast an Aura, Equipment, or Vehicle spell, draw a card."
/// Verifies OR-semantics across all three named subtypes (Aura, Equipment, Vehicle
/// each independently fire the trigger) AND that a spell with none of those
/// subtypes does NOT fire it.
fn test_sram_senior_edificer_spell_subtype_filter() {
    let def = mtg_engine::cards::defs::sram_senior_edificer::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(card_spec(
            p1,
            "Sram, Senior Edificer",
            "sram-senior-edificer",
            ZoneId::Battlefield,
            &def,
        ))
        .object(
            ObjectSpec::card(p1, "Test Aura")
                .with_types(vec![CardType::Enchantment])
                .with_subtypes(vec![SubType("Aura".to_string())])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Test Equipment")
                .with_types(vec![CardType::Artifact])
                .with_subtypes(vec![SubType("Equipment".to_string())])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Test Vehicle")
                .with_types(vec![CardType::Artifact])
                .with_subtypes(vec![SubType("Vehicle".to_string())])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Test Vanilla Creature")
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Hand(p1)),
        )
        // Library filler: 3 positive-case casts each draw a card.
        .object(ObjectSpec::creature(p1, "Library Filler 1", 1, 1).in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::creature(p1, "Library Filler 2", 1, 1).in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::creature(p1, "Library Filler 3", 1, 1).in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 10;
    state.turn_mut().priority_holder = Some(p1);

    let sram_id = find_object(&state, "Sram, Senior Edificer");

    // Positive case 1: Aura.
    let aura_id = find_object(&state, "Test Aura");
    let hand_before = hand_count(&state, p1);
    let (state, _) = cast_spell(state, p1, aura_id, vec![]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        hand_count(&state, p1),
        hand_before,
        "casting an Aura spell must fire the draw trigger (-1 cast +1 draw = net 0)"
    );

    // Positive case 2: Equipment.
    let equipment_id = find_object(&state, "Test Equipment");
    let hand_before = hand_count(&state, p1);
    let (state, _) = cast_spell(state, p1, equipment_id, vec![]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        hand_count(&state, p1),
        hand_before,
        "casting an Equipment spell must fire the draw trigger"
    );
    assert!(
        find_object(&state, "Sram, Senior Edificer") == sram_id,
        "sanity: Sram is still around"
    );

    // Positive case 3: Vehicle.
    let vehicle_id = find_object(&state, "Test Vehicle");
    let hand_before = hand_count(&state, p1);
    let (state, _) = cast_spell(state, p1, vehicle_id, vec![]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        hand_count(&state, p1),
        hand_before,
        "casting a Vehicle spell must fire the draw trigger"
    );

    // Negative case: vanilla creature spell (none of the three subtypes) must NOT draw.
    let creature_id = find_object(&state, "Test Vanilla Creature");
    let hand_before_2 = hand_count(&state, p1);
    let (state, _) = cast_spell(state, p1, creature_id, vec![]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        hand_count(&state, p1),
        hand_before_2 - 1,
        "casting a vanilla creature spell must NOT fire the draw trigger (only -1 for the cast)"
    );
}

// ── 5. Leaf-Crowned Visionary ────────────────────────────────────────────────────

#[test]
/// Oracle: "Other Elves you control get +1/+1. / Whenever you cast an Elf spell,
/// you may pay {G}. If you do, draw a card." Verifies the static +1/+1 buff, and
/// the may-pay draw trigger firing only on Elf spells (and not on a non-Elf spell).
fn test_leaf_crowned_visionary_full_integration() {
    let def = mtg_engine::cards::defs::leaf_crowned_visionary::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(std::sync::Arc::clone(&registry))
        .object(card_spec(
            p1,
            "Leaf-Crowned Visionary",
            "leaf-crowned-visionary",
            ZoneId::Battlefield,
            &def,
        ))
        .object(
            ObjectSpec::creature(p1, "Other Elf", 1, 1)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .object(
            ObjectSpec::card(p1, "Test Elf Spell")
                .with_types(vec![CardType::Creature])
                .with_subtypes(vec![SubType("Elf".to_string())])
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Test Non-Elf Spell")
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Hand(p1)),
        )
        // Library filler: the positive-case cast draws one card. A second filler is
        // kept in reserve so that if the negative-case assertion below fails (i.e. the
        // engine incorrectly fires the trigger on a non-Elf spell), the failure surfaces
        // as a clean assertion mismatch rather than a draw-from-empty-library game loss.
        .object(ObjectSpec::creature(p1, "Library Filler 1", 1, 1).in_zone(ZoneId::Library(p1)))
        .object(ObjectSpec::creature(p1, "Library Filler 2", 1, 1).in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let mut state = state;
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .colorless = 10;
    state.players_mut().get_mut(&p1).unwrap().mana_pool.green = 10;
    state.turn_mut().priority_holder = Some(p1);

    // `GameStateBuilder` places the permanent directly on the battlefield without
    // going through the ETB path, so `register_static_continuous_effects` (normally
    // called at ETB — see `rules/replacement.rs:2020`) never runs and the static
    // +1/+1 is never registered. Register it manually, mirroring the pattern used
    // by `pb_ac3_dynamic_pt_counts.rs::test_ashaya_pt_equals_lands_you_control`.
    let leaf_id = find_object(&state, "Leaf-Crowned Visionary");
    let card_id = state
        .objects()
        .get(&leaf_id)
        .and_then(|o| o.card_id.clone());
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        leaf_id,
        card_id.as_ref(),
        &registry,
    );

    // Static buff: "Other Elves you control get +1/+1."
    let other_elf_id = find_object(&state, "Other Elf");
    let chars = calculate_characteristics(&state, other_elf_id).unwrap();
    assert_eq!(chars.power, Some(2));
    assert_eq!(chars.toughness, Some(2));

    // Positive case: cast an Elf spell — the may-pay-{G}-to-draw trigger fires and
    // the deterministic non-interactive path pays the cost (see MayPayThenEffect docs).
    let elf_spell_id = find_object(&state, "Test Elf Spell");
    let hand_before = hand_count(&state, p1);
    let (state, _) = cast_spell(state, p1, elf_spell_id, vec![]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);
    // -1 (cast) + 1 (draw, since the {G} cost is paid) = net hand size unchanged.
    assert_eq!(
        hand_count(&state, p1),
        hand_before,
        "casting an Elf spell must fire the may-pay-{{G}}-to-draw trigger"
    );

    // Negative case: cast a non-Elf spell — the trigger must NOT fire.
    //
    // FIXED (2026-07-09): this assertion previously failed due to an
    // `ability_index` namespace desync. The G-4 spell_type_filter/noncreature_only/
    // spell_subtype_filter post-processing in `rules/abilities.rs` used to look the
    // ability up via `def.abilities.get(t.ability_index)` — an index into the raw
    // `CardDefinition::abilities` Vec (which also contains Static/Keyword/Activated
    // abilities) — but `t.ability_index` (set in `collect_triggers_for_event`,
    // `rules/abilities.rs`, via `resolved_chars.triggered_abilities.iter()
    // .enumerate()`) is actually a dense index into the runtime
    // `characteristics.triggered_abilities` list. For Leaf-Crowned Visionary —
    // Static ability at `def.abilities[0]`, Triggered `WheneverYouCastSpell` at
    // `def.abilities[1]`, but the SOLE runtime `triggered_abilities` entry at dense
    // index 0 — the old lookup resolved to the Static ability and silently skipped
    // the filter. Fix: `spell_type_filter`/`noncreature_only`/`spell_subtype_filter`
    // are now carried directly on the runtime `TriggeredAbilityDef.
    // triggering_creature_filter` (reusing the existing `TargetFilter` machinery,
    // populated in `enrich_spec_from_def`), and the post-filter re-resolves the
    // trigger definition from the same dense runtime list the trigger was built
    // from instead of the raw `CardDefinition::abilities` Vec. See
    // `crates/engine/src/rules/abilities.rs` (search "index-namespace fix") for the
    // full writeup. `cards/defs/monastery_mentor.rs` had the identical bug
    // (Prowess keyword at index 0 desynced its `noncreature_only: true` filter);
    // see the regression test in `pb_ac7_type_change_ability_removal.rs` /
    // `abilities.rs`.
    let non_elf_spell_id = find_object(&state, "Test Non-Elf Spell");
    let hand_before_2 = hand_count(&state, p1);
    let (state, _) = cast_spell(state, p1, non_elf_spell_id, vec![]).unwrap();
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        hand_count(&state, p1),
        hand_before_2 - 1,
        "casting a non-Elf spell must NOT fire the may-pay-{{G}}-to-draw trigger (only -1 for the cast)"
    );
}

// ── 6. Vraska, Betrayal's Sting — -2 ────────────────────────────────────────────

#[test]
/// Oracle: "-2: Target creature becomes a Treasure artifact with '{T}, Sacrifice
/// this artifact: Add one mana of any color' and loses all other card types and
/// abilities." Ruling (2023-02-04): "The target ... will lose any other subtypes
/// and card types it previously had and will be only a Treasure artifact. It will
/// retain any supertypes it had."
///
/// Regression guard for review finding F-VR1: the -2 ability's `Effect::Sequence`
/// applies `RemoveAllAbilities` and a granted `AddManaAbility` in the SAME
/// `Effect::ApplyContinuousEffect` "batch" — the timestamp counter (CR 613.7b) is
/// not advanced between pushes within one `Sequence`
/// (`crates/engine/src/effects/mod.rs`, `let ts = state.timestamp_counter();`), so
/// both continuous effects share one timestamp. There is no CR 613.8a dependency
/// arm between `RemoveAllAbilities` and `AddManaAbility`
/// (`crates/engine/src/rules/layers.rs::depends_on`), so `resolve_layer_order`
/// falls back to a stable sort on that shared timestamp
/// (`toposort_with_timestamp_fallback`), which preserves the original push/vec
/// order: `RemoveAllAbilities` is pushed before `AddManaAbility` in the card def,
/// so Remove applies first and the granted mana ability survives. This mirrors
/// the already-tested Darksteel Mutation pattern (`RemoveAllAbilities` before
/// `AddKeyword`), just with `AddManaAbility` instead of `AddKeyword` and equal
/// (not distinct/incrementing) timestamps. If the push order in
/// `vraska_betrayals_sting.rs` is ever reversed, or the sort at
/// `toposort_with_timestamp_fallback` stops being stable for tied timestamps,
/// this test fails.
fn test_vraska_betrayals_sting_minus2_full_integration() {
    let def = mtg_engine::cards::defs::vraska_betrayals_sting::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            card_spec(
                p1,
                "Vraska, Betrayal's Sting",
                "vraska-betrayals-sting",
                ZoneId::Battlefield,
                &def,
            )
            .with_counter(CounterType::Loyalty, 6),
        )
        .object(
            ObjectSpec::creature(p2, "Target Creature", 3, 3)
                .with_keyword(KeywordAbility::Flying)
                .with_supertypes(vec![SuperType::Legendary]),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let vraska_id = find_object(&state, "Vraska, Betrayal's Sting");
    let target_id = find_object(&state, "Target Creature");

    let (state, _) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: vraska_id,
            ability_index: 1, // -2: target creature becomes a Treasure
            targets: vec![Target::Object(target_id)],
            x_value: None,
        },
    )
    .unwrap();
    let state = resolve_stack(state, &[p1, p2]);

    let chars = calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(
        chars.card_types,
        im::OrdSet::unit(CardType::Artifact),
        "loses all other card types (is only a Treasure artifact)"
    );
    assert_eq!(
        chars.subtypes,
        im::OrdSet::unit(SubType("Treasure".to_string())),
        "loses all other subtypes ... only a Treasure artifact"
    );
    assert!(
        chars.supertypes.contains(&SuperType::Legendary),
        "ruling: retains any supertypes it had"
    );
    assert!(
        chars.keywords.is_empty(),
        "loses all other abilities: Flying must be gone"
    );
    assert!(
        chars.activated_abilities.is_empty(),
        "loses all other abilities: no leftover activated abilities"
    );
    assert!(
        chars.triggered_abilities.is_empty(),
        "loses all other abilities: no leftover triggered abilities"
    );
    assert_eq!(
        chars.mana_abilities.len(),
        1,
        "the granted mana ability must survive the 'loses all other abilities' removal \
         (F-VR1 — this is the regression guard for the equal-timestamp ordering)"
    );
    let granted = chars.mana_abilities.front().unwrap();
    assert!(
        granted.requires_tap,
        "granted ability is '{{T}}, Sacrifice this artifact: Add one mana of any color'"
    );
    assert!(granted.sacrifice_self, "granted ability requires Sacrifice");
    assert!(
        granted.any_color,
        "granted ability adds one mana of any color"
    );
    assert!(
        granted.produces.is_empty(),
        "any_color ability has no fixed 'produces' map"
    );
}
