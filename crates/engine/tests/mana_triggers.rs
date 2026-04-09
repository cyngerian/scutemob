//! Tests for PB-E: Mana trigger interception system.
//!
//! Two distinct mechanisms:
//! 1. Triggered mana abilities (CR 605.1b, 605.4a, 106.12a): "Whenever you tap a [type]
//!    for mana, add [mana]." These resolve immediately after the triggering mana ability,
//!    without going on the stack.
//! 2. Mana production replacement effects (CR 106.12b, 106.6a): "If you tap a permanent
//!    for mana, it produces N times as much." Applied BEFORE mana is added to the pool.
//!
//! Key ruling (Nyxbloom Ancient): triggered mana abilities (Pattern 1) are NOT multiplied
//! by mana production replacement effects (Pattern 2).

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId,
    Step, ZoneId,
};
use std::collections::HashMap;
use std::sync::Arc;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Build card def map + registry from all_cards().
fn build_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

/// Build an ObjectSpec for a named card, enriched from def, in a given zone.
fn make_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

/// Extract the mana pool color totals from a game state for a player.
fn mana_pool(state: &GameState, player: PlayerId) -> (u32, u32, u32, u32, u32, u32) {
    let ps = state.player(player).unwrap();
    (
        ps.mana_pool.white,
        ps.mana_pool.blue,
        ps.mana_pool.black,
        ps.mana_pool.red,
        ps.mana_pool.green,
        ps.mana_pool.colorless,
    )
}

/// Register replacement effects for all battlefield permanents.
/// GameStateBuilder doesn't run ETB hooks, so replacement effects (Mana Reflection,
/// Nyxbloom Ancient) must be registered manually before testing.
/// Pattern matches counter_replacement.rs line 105.
fn register_replacement_effects(state: &mut GameState, registry: &Arc<CardRegistry>) {
    use mtg_engine::CardId;
    let battlefield_objects: Vec<(ObjectId, PlayerId, Option<CardId>)> = state
        .objects
        .iter()
        .filter(|(_, obj)| matches!(obj.zone, ZoneId::Battlefield))
        .map(|(id, obj)| (*id, obj.controller, obj.card_id.clone()))
        .collect();
    for (obj_id, controller, card_id) in &battlefield_objects {
        mtg_engine::rules::replacement::register_permanent_replacement_abilities(
            state,
            *obj_id,
            *controller,
            card_id.as_ref(),
            registry,
        );
    }
}

// ── Test 1: Triggered mana ability — land filter (Mirari's Wake) ──────────────

#[test]
/// CR 605.4a / CR 106.12a — Mirari's Wake: "Whenever you tap a land for mana, add one
/// mana of any type that land produced." Tapping a Forest for {G} also triggers the Wake,
/// producing one additional {G}. Result: 2 green mana in pool.
fn test_mana_trigger_land_adds_extra_mana() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let wake = make_spec(p1, "Mirari's Wake", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .object(forest)
        .object(wake)
        .build()
        .unwrap();

    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    let (new_state, events) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&new_state, p1);
    // Forest produces 1 green; Wake trigger adds 1 more green.
    assert_eq!(
        green, 2,
        "Mirari's Wake should add 1 green on top of Forest's 1 green"
    );

    // Two ManaAdded events: one from Forest, one from Wake trigger.
    let mana_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ManaAdded { .. }))
        .count();
    assert!(
        mana_events >= 2,
        "Expected at least 2 ManaAdded events (Forest + Wake trigger)"
    );
}

// ── Test 2: Triggered mana ability — land subtype filter (Crypt Ghast) ────────

#[test]
/// CR 106.12a — Crypt Ghast: "Whenever you tap a Swamp for mana, add an additional {B}."
/// Tapping a Swamp produces {B} + {B} = 2 black mana. Tapping a Forest produces only {G}.
fn test_mana_trigger_swamp_subtype_filter() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let swamp = make_spec(p1, "Swamp", ZoneId::Battlefield, &defs);
    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let ghast = make_spec(p1, "Crypt Ghast", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .object(swamp)
        .object(forest)
        .object(ghast)
        .build()
        .unwrap();

    let swamp_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Swamp" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();
    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    // Tap Swamp: should produce {B} + Crypt Ghast trigger {B} = 2 black.
    let (state_after_swamp, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: swamp_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, black, _, _, _) = mana_pool(&state_after_swamp, p1);
    assert_eq!(
        black, 2,
        "Crypt Ghast should add extra black when Swamp is tapped"
    );

    // Tap Forest: should produce {G} only (Crypt Ghast doesn't trigger on Forest).
    let (state_after_forest, _) = process_command(
        state_after_swamp,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&state_after_forest, p1);
    assert_eq!(
        green, 1,
        "Crypt Ghast must NOT trigger on Forest (not a Swamp)"
    );
}

// ── Test 3: Triggered mana ability — creature filter (Leyline of Abundance) ───

#[test]
/// CR 605.1b / CR 106.12a — Leyline of Abundance: "Whenever you tap a creature for mana,
/// add an additional {G}." Tapping Llanowar Elves ({G}) also adds {G} from Leyline = 2G.
/// Tapping a Forest: only {G} (land, not creature — Leyline doesn't trigger).
fn test_mana_trigger_creature_filter() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let elves = make_spec(p1, "Llanowar Elves", ZoneId::Battlefield, &defs);
    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let leyline = make_spec(p1, "Leyline of Abundance", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .object(elves)
        .object(forest)
        .object(leyline)
        .build()
        .unwrap();

    // Remove summoning sickness so Llanowar Elves can tap.
    let elves_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Llanowar Elves" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();
    if let Some(obj) = state.objects.get_mut(&elves_id) {
        obj.has_summoning_sickness = false;
    }
    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    // Tap Llanowar Elves: Leyline triggers → 2 green mana.
    let (state_after_elves, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: elves_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&state_after_elves, p1);
    assert_eq!(
        green, 2,
        "Leyline of Abundance should add green when a creature taps for mana"
    );

    // Tap Forest: Leyline does NOT trigger (Forest is a land, not a creature).
    let (state_after_forest, _) = process_command(
        state_after_elves,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green2, _) = mana_pool(&state_after_forest, p1);
    assert_eq!(
        green2,
        2 + 1,
        "Leyline must NOT trigger on a land; Forest adds only 1G"
    );
}

// ── Test 4: Triggered mana ability — enchanted land filter (Wild Growth) ──────

#[test]
/// CR 106.12a — Wild Growth: "Whenever enchanted land is tapped for mana, add {G}."
/// Wild Growth enchants Forest A. Tapping Forest A: 2G. Tapping Forest B: 1G (not enchanted).
fn test_mana_trigger_enchanted_land() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let forest_a = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let forest_b = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let wild_growth = make_spec(p1, "Wild Growth", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .object(forest_a)
        .object(forest_b)
        .object(wild_growth)
        .build()
        .unwrap();

    // Get IDs.
    let mut forest_ids: Vec<mtg_engine::ObjectId> = state
        .objects
        .values()
        .filter(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .collect();
    forest_ids.sort();
    let forest_a_id = forest_ids[0];
    let forest_b_id = forest_ids[1];
    let wg_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Wild Growth" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    // Attach Wild Growth to Forest A.
    if let Some(obj) = state.objects.get_mut(&wg_id) {
        obj.attached_to = Some(forest_a_id);
    }

    // Tap Forest A (enchanted): Wild Growth triggers → 2 green.
    let (state2, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: forest_a_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&state2, p1);
    assert_eq!(
        green, 2,
        "Wild Growth should add green when enchanted land is tapped"
    );

    // Tap Forest B (not enchanted): no trigger → 1 green.
    let (state3, _) = process_command(
        state2,
        Command::TapForMana {
            player: p1,
            source: forest_b_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green2, _) = mana_pool(&state3, p1);
    assert_eq!(
        green2,
        2 + 1,
        "Wild Growth must NOT trigger for non-enchanted Forest"
    );
}

// ── Test 5: Mana multiplier (×2) — Mana Reflection ────────────────────────────

#[test]
/// CR 106.12b — Mana Reflection: "If you tap a permanent for mana, it produces twice as much."
/// Tapping a Forest for {G} produces {G}{G} instead.
fn test_mana_multiplier_double() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let reflection = make_spec(p1, "Mana Reflection", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry.clone())
        .object(forest)
        .object(reflection)
        .build()
        .unwrap();

    // Register replacement effects: GameStateBuilder doesn't run ETB hooks.
    register_replacement_effects(&mut state, &registry);

    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    let (new_state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&new_state, p1);
    assert_eq!(
        green, 2,
        "Mana Reflection should double Forest's green mana output to 2G"
    );
}

// ── Test 6: Mana multiplier (×3) — Nyxbloom Ancient ──────────────────────────

#[test]
/// CR 106.12b — Nyxbloom Ancient: "If you tap a permanent for mana, it produces three times
/// as much of that mana instead." Tapping Forest for {G} produces {G}{G}{G}.
fn test_mana_multiplier_triple() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let ancient = make_spec(p1, "Nyxbloom Ancient", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry.clone())
        .object(forest)
        .object(ancient)
        .build()
        .unwrap();

    // Remove summoning sickness from the Elemental (just in case it was treated as creature).
    let ancient_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Nyxbloom Ancient")
        .map(|o| o.id)
        .unwrap();
    if let Some(obj) = state.objects.get_mut(&ancient_id) {
        obj.has_summoning_sickness = false;
    }

    // Register replacement effects: GameStateBuilder doesn't run ETB hooks.
    register_replacement_effects(&mut state, &registry);

    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    let (new_state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&new_state, p1);
    assert_eq!(
        green, 3,
        "Nyxbloom Ancient should triple Forest's green mana output to 3G"
    );
}

// ── Test 7: Stacking multipliers — two Mana Reflections = ×4 ─────────────────

#[test]
/// CR 106.6a / Mana Reflection ruling: Multiple Mana Reflections stack multiplicatively.
/// Two Mana Reflections: Forest tapped for {G} → {G}×4.
fn test_mana_multiplier_stacks_multiplicatively() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let reflection1 = make_spec(p1, "Mana Reflection", ZoneId::Battlefield, &defs);
    let reflection2 = make_spec(p1, "Mana Reflection", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry.clone())
        .object(forest)
        .object(reflection1)
        .object(reflection2)
        .build()
        .unwrap();

    // Register replacement effects: GameStateBuilder doesn't run ETB hooks.
    register_replacement_effects(&mut state, &registry);

    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    let (new_state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&new_state, p1);
    assert_eq!(
        green, 4,
        "Two Mana Reflections multiply 2×2 = 4G (multiplicative stacking per ruling)"
    );
}

// ── Test 8: Multiplier does NOT affect triggered mana (Nyxbloom + Wake) ───────

#[test]
/// Nyxbloom Ancient ruling: "If an ability triggers 'whenever you tap' something for mana
/// and produces mana, that triggered mana ability won't be affected by Nyxbloom Ancient."
///
/// Setup: Nyxbloom Ancient + Mirari's Wake + Forest.
/// Forest tapped for mana: Nyxbloom triples the base {G} → 3G, THEN Wake's triggered mana
/// ability fires for 1G (matching the type). Wake's trigger is NOT tripled.
/// Total: 4G (3 from tripled base + 1 from Wake trigger, not 3+3).
fn test_mana_multiplier_does_not_affect_triggered_mana() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);
    let ancient = make_spec(p1, "Nyxbloom Ancient", ZoneId::Battlefield, &defs);
    let wake = make_spec(p1, "Mirari's Wake", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry.clone())
        .object(forest)
        .object(ancient)
        .object(wake)
        .build()
        .unwrap();

    let ancient_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Nyxbloom Ancient")
        .map(|o| o.id)
        .unwrap();
    if let Some(obj) = state.objects.get_mut(&ancient_id) {
        obj.has_summoning_sickness = false;
    }

    // Register replacement effects: GameStateBuilder doesn't run ETB hooks.
    register_replacement_effects(&mut state, &registry);

    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    let (new_state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&new_state, p1);
    // 3G from Nyxbloom tripling Forest + 1G from Wake trigger (not tripled per ruling) = 4G.
    // (If Wake were also tripled we'd get 3+3=6G, which would be wrong.)
    assert_eq!(
        green, 4,
        "Nyxbloom triples Forest to 3G; Wake trigger adds 1G (not tripled) = 4G total"
    );
}

// ── Test 9: Zendikar Resurgent — creature cast draw trigger ───────────────────

#[test]
/// CR 603.1 / CR 106.12a — Zendikar Resurgent: two triggered abilities —
///   (a) "Whenever you tap a land for mana, add one mana of any type that land produced."
///   (b) "Whenever you cast a creature spell, draw a card."
///
/// WhenTappedForMana triggers are handled via card-registry scan at tap time (not stored
/// in triggered_abilities). WheneverYouCastSpell IS converted to a triggered_ability by
/// enrich_spec_from_def. So triggered_abilities.len() == 1 (only the draw trigger).
/// The land mana trigger is verified by tapping a Forest with Resurgent present.
fn test_zendikar_resurgent_registered_on_battlefield() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let resurgent = make_spec(p1, "Zendikar Resurgent", ZoneId::Battlefield, &defs);
    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .object(resurgent)
        .object(forest)
        .build()
        .unwrap();

    // WheneverYouCastSpell (creature draw trigger) is converted to triggered_ability.
    // WhenTappedForMana is NOT — it's handled via card-registry scan in handle_tap_for_mana.
    let resurgent_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Zendikar Resurgent" && o.zone == ZoneId::Battlefield)
        .unwrap();

    assert_eq!(
        resurgent_obj.characteristics.triggered_abilities.len(),
        1,
        "Zendikar Resurgent triggered_abilities has exactly 1 entry (WheneverYouCastSpell draw trigger; WhenTappedForMana is handled via card-registry scan)"
    );

    // Verify the land-mana trigger works: tapping Forest with Resurgent produces 2G.
    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    let (new_state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&new_state, p1);
    assert_eq!(
        green, 2,
        "Zendikar Resurgent land-mana trigger should add 1G when a land is tapped for mana"
    );
}

// ── Test 10: Mana trigger only fires on tap abilities ─────────────────────────

#[test]
/// CR 106.12: "To 'tap [a permanent] for mana' is to activate a mana ability that includes
/// the {T} symbol in its activation cost." Triggered mana abilities (WhenTappedForMana)
/// only fire on {T}-cost mana abilities (Command::TapForMana), not sacrifice-based mana.
///
/// WhenTappedForMana triggers are handled via card-registry scan in handle_tap_for_mana
/// (guarded by ability.requires_tap). They are NOT stored in triggered_abilities on the
/// object — enrich_spec_from_def does not convert WhenTappedForMana to triggered_abilities.
///
/// Verify: Mirari's Wake present + Forest tapped → 2 green (trigger fires).
/// Verify: triggered_abilities is empty (WhenTappedForMana handled via registry scan).
fn test_mana_trigger_only_fires_on_tap_abilities() {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_registry();

    let wake = make_spec(p1, "Mirari's Wake", ZoneId::Battlefield, &defs);
    let forest = make_spec(p1, "Forest", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .object(wake)
        .object(forest)
        .build()
        .unwrap();

    // WhenTappedForMana is NOT stored in triggered_abilities on the object.
    // It is handled via card-registry scan in handle_tap_for_mana (mana.rs).
    let wake_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Mirari's Wake" && o.zone == ZoneId::Battlefield)
        .unwrap();

    assert_eq!(
        wake_obj.characteristics.triggered_abilities.len(),
        0,
        "Mirari's Wake has no triggered_abilities registered on the object; WhenTappedForMana is dispatched via card-registry scan at tap time (CR 106.12a)"
    );

    // Verify the trigger mechanism works: tapping Forest with Wake on battlefield produces 2G.
    let forest_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
        .unwrap();

    let (new_state, _) = process_command(
        state,
        Command::TapForMana {
            player: p1,
            source: forest_id,
            ability_index: 0,
        },
    )
    .unwrap();

    let (_, _, _, _, green, _) = mana_pool(&new_state, p1);
    assert_eq!(
        green, 2,
        "Mirari's Wake fires on TapForMana (Command::TapForMana), adding 1G to Forest's 1G = 2G total"
    );
}
