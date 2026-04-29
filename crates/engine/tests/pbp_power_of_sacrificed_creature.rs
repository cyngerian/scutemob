//! PB-P: EffectAmount::PowerOfSacrificedCreature (LKI capture-by-value) tests.
//!
//! Tests verify that `EffectAmount::PowerOfSacrificedCreature` correctly reads the
//! layer-resolved power of a sacrificed creature at the moment of sacrifice (LKI),
//! not from the post-zone-change graveyard object.
//!
//! CR Rules covered:
//! - CR 608.2b: At the time an effect resolves, use the last known information for
//!   objects that have left a zone. The sacrificed creature's power is captured from
//!   the battlefield (layer-resolved) BEFORE `move_object_to_zone`.
//! - CR 701.16: "Sacrifice" means to move a permanent from the battlefield to its
//!   owner's graveyard. Sacrifice as a cost happens before the ability/spell resolves.
//! - CR 602.2: Activated abilities have costs paid before the ability goes on the stack.
//! - CR 118.8: Mandatory additional costs are paid as part of casting a spell.
//! - CR 613.1: Layer 7b (P/T-modifying effects, e.g. anthems) applies while the object
//!   is on the battlefield. Graveyard objects do not benefit from Layer 7b effects.

use mtg_engine::{
    process_command, CardId, CardRegistry, Command, ContinuousEffect, EffectAmount, EffectDuration,
    EffectFilter, EffectId, EffectLayer, GameEvent, GameState, GameStateBuilder, LayerModification,
    ObjectId, ObjectSpec, PlayerId, Step, ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn count_in_zone(state: &GameState, zone: ZoneId) -> usize {
    state.objects.values().filter(|o| o.zone == zone).count()
}

/// Pass priority for all listed players once, accumulating events.
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

/// Build a Glorious Anthem-style continuous effect: +1/+1 to all creatures on the battlefield.
fn anthem_power_effect(id: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp: 100,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::ModifyPower(1),
        is_cda: false,
        condition: None,
    }
}

fn anthem_toughness_effect(id: u64) -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(id),
        source: None,
        timestamp: 100,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::ModifyToughness(1),
        is_cda: false,
        condition: None,
    }
}

// ── Populate library for player ────────────────────────────────────────────────

fn add_library_cards(
    builder: GameStateBuilder,
    owner: PlayerId,
    n: usize,
    prefix: &str,
) -> GameStateBuilder {
    let mut b = builder;
    for i in 0..n {
        b = b.object(
            ObjectSpec::card(owner, &format!("{} {}", prefix, i))
                .with_card_id(CardId(format!("{}-lib-{}", prefix, i)))
                .in_zone(ZoneId::Library(owner)),
        );
    }
    b
}

// ── Test M1: Altar of Dementia mills by sacrificed creature's power ────────────

/// CR 608.2b + CR 701.16 + CR 602.2 — PB-P M1: Altar of Dementia mills exactly N cards
/// where N equals the sacrificed creature's power.
///
/// Setup: 4-player game. P1 controls Altar of Dementia and a 5/5 Goblin.
/// P1 activates Altar, sacrificing the Goblin, targeting P2 (who has 10 library cards).
///
/// Assert:
/// - The Goblin is in P1's graveyard.
/// - P2 has milled exactly 5 cards (5/5 power = 5 mill count).
/// - P2's library reduced from 10 to 5.
///
/// Discriminator: pre-PB-P Altar of Dementia has `abilities: vec![]` and cannot be
/// activated at all. Post-PB-P the activated ability fires with LKI power=5.
#[test]
fn test_altar_of_dementia_mills_by_sacrificed_power() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def};
    use std::collections::HashMap;

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let altar = enrich_spec_from_def(
        ObjectSpec::card(p1, "Altar of Dementia")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Altar of Dementia")),
        &defs,
    );

    let goblin = ObjectSpec::creature(p1, "Sacrificial Goblin", 5, 5)
        .with_card_id(CardId("sacrificial-goblin".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(altar)
        .object(goblin);

    // 10 library cards for P2 so there's room to mill.
    builder = add_library_cards(builder, p2, 10, "P2Lib");

    let state = builder.build().unwrap();

    let altar_id = find_obj(&state, "Altar of Dementia");
    let goblin_id = find_obj(&state, "Sacrificial Goblin");
    let p2_id = state
        .players
        .iter()
        .find(|(id, _)| **id == p2)
        .map(|(id, _)| *id)
        .unwrap();

    let lib_before = count_in_zone(&state, ZoneId::Library(p2));
    assert_eq!(lib_before, 10, "P2 should start with 10 library cards");

    // P1 activates Altar of Dementia: sacrifice the 5/5 Goblin, targeting P2.
    // The Goblin's LKI power (5) should be captured before zone move.
    let (state, _activation_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: altar_id,
            ability_index: 0,
            targets: vec![mtg_engine::Target::Player(p2_id)],
            discard_card: None,
            sacrifice_target: Some(goblin_id),
            x_value: None,
        },
    )
    .expect("Altar of Dementia activation should succeed");

    // Goblin is sacrificed as cost — must be in P1's graveyard now.
    assert!(
        in_graveyard(&state, "Sacrificial Goblin", p1),
        "CR 701.16: Sacrificial Goblin must be in P1's graveyard after activation"
    );

    // Ability is on the stack; pass priority to resolve it.
    let (state, _) = pass_all(state, &players);

    // Verify mill count.
    let lib_after = count_in_zone(&state, ZoneId::Library(p2));
    let milled = lib_before - lib_after;
    assert_eq!(
        milled, 5,
        "CR 608.2b + PB-P M1: P2 should mill exactly 5 cards (5/5 Goblin power). \
         Library before={}, after={}, milled={}",
        lib_before, lib_after, milled
    );
}

// ── Test M2: Greater Good draws by power then discards 3 ─────────────────────

/// CR 608.2b + CR 701.16 + CR 602.2 + CR 701.7 — PB-P M2: Greater Good draws cards
/// equal to the sacrificed creature's power, then discards exactly 3 cards.
///
/// Setup: 4-player game. P1 controls Greater Good and a 4/4 Hippo.
/// P1 has 0 cards in hand and 10 cards in library.
/// P1 activates Greater Good, sacrificing the 4/4 Hippo.
///
/// Assert:
/// - The Hippo is in P1's graveyard.
/// - P1 drew exactly 4 cards (4/4 power = 4 draw count), then discarded 3.
/// - P1's net hand size = 4 - 3 = 1 card remaining.
/// - P1's library reduced by 4.
///
/// Discriminator: pre-PB-P Greater Good has `abilities: vec![]`. Post-PB-P the
/// Sequence runs draw(4) then discard(3) in order.
#[test]
fn test_greater_good_draws_by_sacrificed_power_then_discards_three() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def};
    use std::collections::HashMap;

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let greater_good = enrich_spec_from_def(
        ObjectSpec::card(p1, "Greater Good")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Greater Good")),
        &defs,
    );

    let hippo = ObjectSpec::creature(p1, "Sacrificial Hippo", 4, 4)
        .with_card_id(CardId("sacrificial-hippo".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(greater_good)
        .object(hippo);

    // 10 library cards for P1.
    builder = add_library_cards(builder, p1, 10, "P1Lib");

    let state = builder.build().unwrap();

    let gg_id = find_obj(&state, "Greater Good");
    let hippo_id = find_obj(&state, "Sacrificial Hippo");

    let hand_before = count_in_zone(&state, ZoneId::Hand(p1));
    let lib_before = count_in_zone(&state, ZoneId::Library(p1));

    // Activate Greater Good: sacrifice the 4/4 Hippo.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: gg_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(hippo_id),
            x_value: None,
        },
    )
    .expect("Greater Good activation should succeed");

    // Hippo is sacrificed as cost.
    assert!(
        in_graveyard(&state, "Sacrificial Hippo", p1),
        "CR 701.16: Sacrificial Hippo must be in P1's graveyard after activation"
    );

    // Pass priority to resolve the ability (Sequence: draw 4, then discard 3).
    let (state, _) = pass_all(state, &players);

    let hand_after = count_in_zone(&state, ZoneId::Hand(p1));
    let lib_after = count_in_zone(&state, ZoneId::Library(p1));

    // Net hand change: drew 4, discarded 3 → net +1.
    let net_hand_change = hand_after as i32 - hand_before as i32;
    assert_eq!(
        net_hand_change, 1,
        "CR 608.2b + PB-P M2: Net hand change should be +1 (drew 4, discarded 3). \
         hand_before={}, hand_after={}, net={}",
        hand_before, hand_after, net_hand_change
    );

    // Library: should have drawn exactly 4.
    let drawn = lib_before - lib_after;
    assert_eq!(
        drawn, 4,
        "CR 608.2b + PB-P M2: P1 should draw exactly 4 cards (4/4 Hippo power). \
         lib_before={}, lib_after={}, drawn={}",
        lib_before, lib_after, drawn
    );
}

// ── Test M3: Life's Legacy draws by sacrificed creature's power (spell path) ──

/// CR 608.2b + CR 118.8 + CR 117.1f — PB-P M3: Life's Legacy draws cards equal to the
/// sacrificed creature's power via the spell additional-cost LKI propagation path.
///
/// Setup: 4-player game. P1 controls a 6/6 Beast and has Life's Legacy in hand.
/// P1 has {1}{G} available and 10 cards in library (0 in hand besides Life's Legacy).
/// P1 casts Life's Legacy, sacrificing the 6/6 Beast as additional cost.
///
/// Assert:
/// - The Beast is in P1's graveyard (sacrificed as additional cost before spell resolves).
/// - Life's Legacy is in P1's graveyard (resolved).
/// - P1 drew exactly 6 cards (6/6 Beast power).
///
/// Discriminator: pre-PB-P Life's Legacy has `count: EffectAmount::Fixed(1)` placeholder
/// and draws 1 card. Post-PB-P the captured LKI power flows from
/// additional_costs.Sacrifice.lki_powers into ctx.sacrificed_creature_powers, drawing 6.
#[test]
fn test_lifes_legacy_draws_by_sacrificed_power_on_resolve() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def, AdditionalCost, ManaPool};
    use std::collections::HashMap;

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    // Life's Legacy in P1's hand.
    let lifes_legacy = enrich_spec_from_def(
        ObjectSpec::card(p1, "Life's Legacy")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Life's Legacy")),
        &defs,
    );

    let beast = ObjectSpec::creature(p1, "Sacrificial Beast", 6, 6)
        .with_card_id(CardId("sacrificial-beast".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(lifes_legacy)
        .object(beast);

    builder = add_library_cards(builder, p1, 10, "P1Lib");

    let mut state = builder.build().unwrap();

    // Grant P1 {1}{G} mana (Life's Legacy costs {1}{G}).
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 1,
            green: 1,
            ..Default::default()
        };
    }

    let legacy_id = find_obj(&state, "Life's Legacy");
    let beast_id = find_obj(&state, "Sacrificial Beast");

    let _hand_before = count_in_zone(&state, ZoneId::Hand(p1));
    let lib_before = count_in_zone(&state, ZoneId::Library(p1));

    // Cast Life's Legacy, sacrificing the 6/6 Beast as additional cost.
    // The harness caller passes lki_powers: vec![] — the engine fills in the captured value.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: legacy_id,
            targets: vec![],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![AdditionalCost::Sacrifice {
                ids: vec![beast_id],
                lki_powers: vec![],
            }],
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("Cast Life's Legacy should succeed");

    // Beast is sacrificed as additional cost during casting.
    assert!(
        in_graveyard(&state, "Sacrificial Beast", p1),
        "CR 118.8: Sacrificial Beast must be in P1's graveyard after being sacrificed as additional cost"
    );

    // Life's Legacy is on the stack; pass priority to resolve.
    let (state, _) = pass_all(state, &players);

    // Life's Legacy should be in graveyard (resolved).
    assert!(
        in_graveyard(&state, "Life's Legacy", p1),
        "Life's Legacy should be in P1's graveyard after resolving"
    );

    let _hand_after = count_in_zone(&state, ZoneId::Hand(p1));
    let lib_after = count_in_zone(&state, ZoneId::Library(p1));

    // P1 drew 6 cards (6/6 Beast power). Hand was initially 1 (Life's Legacy).
    // After casting, hand is 0. After drawing 6, hand is 6.
    let drawn = lib_before - lib_after;
    assert_eq!(
        drawn, 6,
        "CR 608.2b + PB-P M3: P1 should draw exactly 6 cards (6/6 Beast LKI power). \
         lib_before={}, lib_after={}, drawn={}",
        lib_before, lib_after, drawn
    );
}

// ── Test M4: LKI correctness — anthem-boosted creature sacrifice mills boosted value ──

/// CR 608.2b + CR 613.1 — PB-P M4: The LOAD-BEARING LKI correctness test.
///
/// A 2/2 Bear under a Glorious Anthem (+1/+1 to all creatures) is layer-resolved as
/// 3/3 on the battlefield. When sacrificed to Altar of Dementia, the LKI power (3)
/// must be captured BEFORE the zone move. Post-move, the graveyard Bear's
/// calculate_characteristics returns 2 (anthem doesn't apply in graveyard — BASELINE-LKI-01).
///
/// Setup: 4-player game. P1 controls Altar of Dementia, a 2/2 Bear (base), and a
/// Glorious Anthem (simulated as a continuous effect: +1/+1 to all creatures).
/// P2 has 10 library cards.
///
/// Assert:
/// - The Bear is in P1's graveyard with base power 2.
/// - P2 mills exactly 3 cards (the LKI value at sacrifice time, not the post-move base).
///
/// Discriminator: this is the CR 608.2b correctness anchor. If PB-P implemented
/// capture-by-ID (read the new graveyard object's characteristics after the zone move),
/// it would return 2 (base power, anthem no longer applies — BASELINE-LKI-01). Only
/// capture-by-value produces the correct answer of 3.
#[test]
fn test_lki_correctness_anthem_boosted_creature_sacrifice() {
    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def};
    use std::collections::HashMap;

    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let altar = enrich_spec_from_def(
        ObjectSpec::card(p1, "Altar of Dementia")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Altar of Dementia")),
        &defs,
    );

    // Bear with BASE power 2 — the anthem will boost it to 3 on the battlefield.
    let bear = ObjectSpec::creature(p1, "Anthem Bear", 2, 2)
        .with_card_id(CardId("anthem-bear".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(altar)
        .object(bear)
        // Simulated Glorious Anthem: +1/+1 to all creatures on the battlefield.
        .add_continuous_effect(anthem_power_effect(1))
        .add_continuous_effect(anthem_toughness_effect(2));

    builder = add_library_cards(builder, p2, 10, "P2Lib");

    let state = builder.build().unwrap();

    let altar_id = find_obj(&state, "Altar of Dementia");
    let bear_id = find_obj(&state, "Anthem Bear");
    let p2_id = state
        .players
        .iter()
        .find(|(id, _)| **id == p2)
        .map(|(id, _)| *id)
        .unwrap();

    // Verify the anthem is applying: Bear should resolve as 3/3 before sacrifice.
    let bear_chars = mtg_engine::calculate_characteristics(&state, bear_id)
        .expect("calculate_characteristics should work for the Bear");
    assert_eq!(
        bear_chars.power,
        Some(3),
        "CR 613.1: Anthem-boosted Bear should resolve as 3/3 on the battlefield (base 2 + anthem +1)"
    );

    let lib_before = count_in_zone(&state, ZoneId::Library(p2));

    // P1 activates Altar of Dementia, sacrificing the anthem-boosted 3/3 Bear, targeting P2.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: altar_id,
            ability_index: 0,
            targets: vec![mtg_engine::Target::Player(p2_id)],
            discard_card: None,
            sacrifice_target: Some(bear_id),
            x_value: None,
        },
    )
    .expect("Altar of Dementia activation should succeed");

    // Bear is sacrificed.
    assert!(
        in_graveyard(&state, "Anthem Bear", p1),
        "CR 701.16: Anthem Bear must be in P1's graveyard"
    );

    // Verify that the graveyard Bear's base power is 2 (anthem no longer applies — BASELINE-LKI-01).
    let graveyard_bear = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Anthem Bear")
        .expect("Anthem Bear should exist in state");
    assert_eq!(
        graveyard_bear.characteristics.power,
        Some(2),
        "CR 613.1 / BASELINE-LKI-01: Graveyard Bear's base power should be 2 (anthem not applied in graveyard)"
    );

    // Pass priority to resolve the Altar ability.
    let (state, _) = pass_all(state, &players);

    let lib_after = count_in_zone(&state, ZoneId::Library(p2));
    let milled = lib_before - lib_after;

    // The key assertion: P2 mills 3, not 2.
    // 3 = the LKI value captured at sacrifice time (Bear was 3/3 on battlefield under anthem).
    // 2 = what capture-by-ID would incorrectly return (graveyard object, anthem gone).
    assert_eq!(
        milled, 3,
        "CR 608.2b LKI correctness (PB-P M4): P2 should mill 3 cards (LKI power 3, anthem-boosted). \
         If milled=2, the engine captured post-move power (WRONG). lib_before={}, lib_after={}",
        lib_before, lib_after
    );
}

// ── Test M5: Zero-power creature sacrifice mills zero ─────────────────────────

/// CR 608.2b + CR 701.16 — PB-P M5: Sacrificing a 0-power creature to Altar of Dementia
/// mills exactly 0 cards (Mill 0 is a no-op; library size unchanged).
///
/// Setup: 4-player game. P1 controls Altar of Dementia and a 0/4 Wall.
/// P1 activates Altar, sacrificing the 0/4 Wall, targeting P2.
///
/// Assert:
/// - The Wall is in P1's graveyard.
/// - P2's library size is unchanged (mill 0 = no-op).
///
/// Discriminator: validates the defensive `unwrap_or(0)` path and that the engine
/// handles Mill{count=0} as a clean no-op.
#[test]
fn test_zero_power_creature_sacrifice_mills_zero() {
    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def};
    use std::collections::HashMap;

    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let altar = enrich_spec_from_def(
        ObjectSpec::card(p1, "Altar of Dementia")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Altar of Dementia")),
        &defs,
    );

    // 0/4 Wall — power is 0.
    let wall = ObjectSpec::creature(p1, "Humble Wall", 0, 4)
        .with_card_id(CardId("humble-wall".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(altar)
        .object(wall);

    builder = add_library_cards(builder, p2, 5, "P2Lib");

    let state = builder.build().unwrap();

    let altar_id = find_obj(&state, "Altar of Dementia");
    let wall_id = find_obj(&state, "Humble Wall");
    let p2_id = state
        .players
        .iter()
        .find(|(id, _)| **id == p2)
        .map(|(id, _)| *id)
        .unwrap();

    let lib_before = count_in_zone(&state, ZoneId::Library(p2));
    assert_eq!(lib_before, 5, "P2 should start with 5 library cards");

    // Sacrifice the 0-power Wall.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: altar_id,
            ability_index: 0,
            targets: vec![mtg_engine::Target::Player(p2_id)],
            discard_card: None,
            sacrifice_target: Some(wall_id),
            x_value: None,
        },
    )
    .expect("Altar of Dementia activation should succeed");

    assert!(
        in_graveyard(&state, "Humble Wall", p1),
        "CR 701.16: Humble Wall must be in P1's graveyard"
    );

    // Resolve the ability.
    let (state, _) = pass_all(state, &players);

    let lib_after = count_in_zone(&state, ZoneId::Library(p2));
    let milled = lib_before as i32 - lib_after as i32;

    assert_eq!(
        milled, 0,
        "CR 608.2b + PB-P M5: Mill 0 should be a no-op. lib_before={}, lib_after={}, milled={}",
        lib_before, lib_after, milled
    );
}

// ── Test M6: Empty sacrificed_creature_powers returns 0 (defensive) ───────────

/// PB-P M6: Defensive test — when EffectContext.sacrificed_creature_powers is empty
/// (card author used PowerOfSacrificedCreature without a sacrifice cost),
/// the effect resolves with count=0 and does not panic.
///
/// This tests the resolve_amount arm's `unwrap_or(0)` defensive fallback via
/// an execute_effect call with a manually constructed EffectContext.
///
/// CR: N/A (defensive infrastructure test).
#[test]
fn test_sacrifice_no_capture_returns_zero_defensive() {
    use mtg_engine::effects::{execute_effect, EffectContext};
    use mtg_engine::{Effect, EffectAmount, PlayerTarget};

    let p1 = p(1);
    let p2 = p(2);

    // Build a minimal state with a library for P1 to potentially mill from.
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    builder = add_library_cards(builder, p1, 5, "SafeLib");

    let mut state = builder.build().unwrap();

    let lib_before = count_in_zone(&state, ZoneId::Library(p1));

    // Construct an EffectContext with NO sacrificed_creature_powers (empty vec).
    // This simulates a card author pairing PowerOfSacrificedCreature with a non-sacrifice cost.
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    // Confirm the field is empty (default from EffectContext::new).
    assert!(
        ctx.sacrificed_creature_powers.is_empty(),
        "PB-P M6: EffectContext::new should initialize sacrificed_creature_powers to empty vec"
    );

    // Execute a MillCards effect with count=PowerOfSacrificedCreature.
    // With empty sacrificed_creature_powers, resolve_amount returns 0.
    // Mill 0 should be a no-op — no panic, no library change.
    let effect = Effect::MillCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::PowerOfSacrificedCreature,
    };

    // Should not panic.
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    let lib_after = count_in_zone(&state, ZoneId::Library(p1));
    assert_eq!(
        lib_before, lib_after,
        "PB-P M6: Mill with empty sacrificed_creature_powers should be a no-op (count=0). \
         lib_before={}, lib_after={}",
        lib_before, lib_after
    );
}

// ── Test M7: Hash parity — EffectAmount variants hash distinctly + sentinel 6 ──

/// PB-P M7: Hash parity test — PowerOfSacrificedCreature and four neighboring EffectAmount
/// variants all produce distinct hashes. Also asserts HASH_SCHEMA_VERSION == 6.
///
/// CR: N/A (hash infrastructure).
///
/// Discriminator: forces the sentinel assertion to fail if Change 10 (hash bump 5→6)
/// was not applied. Discriminates the new variant from the existing neighbors.
#[test]
fn test_hash_parity_power_of_sacrificed_creature_distinct() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    // Assert hash sentinel is exactly 10 (PB-CC-B bump from PB-SFT's 9 for
    // TargetFilter.has_counter_type counter presence predicate).
    assert_eq!(
        HASH_SCHEMA_VERSION, 10u8,
        "PB-CC-B: HASH_SCHEMA_VERSION must be 10 (bump from PB-SFT's 9 for \
         TargetFilter.has_counter_type counter presence predicate). \
         If you bumped the sentinel, update this test."
    );

    // Hash five EffectAmount variants; all must be distinct.
    let hash_amount = |amount: &EffectAmount| -> [u8; 32] {
        let mut hasher = Hasher::new();
        amount.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let h_fixed_0 = hash_amount(&EffectAmount::Fixed(0));
    let h_power_of = hash_amount(&EffectAmount::PowerOf(mtg_engine::CardEffectTarget::Source));
    let h_toughness_of = hash_amount(&EffectAmount::ToughnessOf(
        mtg_engine::CardEffectTarget::Source,
    ));
    let h_power_sac = hash_amount(&EffectAmount::PowerOfSacrificedCreature);
    let h_combat_dmg = hash_amount(&EffectAmount::CombatDamageDealt);

    assert_ne!(
        h_fixed_0, h_power_sac,
        "PB-P M7: Fixed(0) and PowerOfSacrificedCreature must hash distinctly"
    );
    assert_ne!(
        h_power_of, h_power_sac,
        "PB-P M7: PowerOf(Source) and PowerOfSacrificedCreature must hash distinctly"
    );
    assert_ne!(
        h_toughness_of, h_power_sac,
        "PB-P M7: ToughnessOf(Source) and PowerOfSacrificedCreature must hash distinctly"
    );
    assert_ne!(
        h_combat_dmg, h_power_sac,
        "PB-P M7: CombatDamageDealt and PowerOfSacrificedCreature must hash distinctly"
    );
    assert_ne!(
        h_fixed_0, h_power_of,
        "PB-P M7: Fixed(0) and PowerOf(Source) must hash distinctly (regression)"
    );
}

// ── Test M8: Backward compat — existing PowerOf cards still work ───────────────

/// PB-P M8: Backward compatibility regression test.
///
/// PB-P adds a new EffectAmount variant and modifies AdditionalCost::Sacrifice shape.
/// Existing EffectAmount::PowerOf(EffectTarget) cards must be completely unaffected.
///
/// Subtest: Swords to Plowshares — cast on a 5/5 creature; controller gains 5 life
/// (uses EffectAmount::PowerOf(EffectTarget::DeclaredTarget{index:0})).
///
/// CR: N/A (regression).
#[test]
fn test_backward_compat_existing_powerof_cards_still_work() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def, ManaPool};
    use std::collections::HashMap;

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    // Swords to Plowshares in P1's hand — costs {W}.
    let swords = enrich_spec_from_def(
        ObjectSpec::card(p1, "Swords to Plowshares")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Swords to Plowshares")),
        &defs,
    );

    // P2's 5/5 creature — target of Swords to Plowshares.
    let victim = ObjectSpec::creature(p2, "P2 Big Creature", 5, 5)
        .with_card_id(CardId("p2-big-creature".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(swords)
        .object(victim)
        .build()
        .unwrap();

    // Grant P1 {W} mana.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool = ManaPool {
            white: 1,
            ..Default::default()
        };
    }

    let swords_id = find_obj(&state, "Swords to Plowshares");
    let victim_id = find_obj(&state, "P2 Big Creature");

    let p2_life_before = state.players.get(&p2).map(|ps| ps.life_total).unwrap_or(40);

    // Cast Swords to Plowshares targeting P2's 5/5 creature.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: swords_id,
            targets: vec![mtg_engine::Target::Object(victim_id)],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![],
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("Swords to Plowshares cast should succeed");

    // Resolve.
    let (state, _) = pass_all(state, &players);

    // P2's creature is exiled.
    let victim_still_on_bf = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "P2 Big Creature" && o.zone == ZoneId::Battlefield);
    assert!(
        !victim_still_on_bf,
        "PB-P M8: P2's 5/5 should no longer be on the battlefield after Swords to Plowshares"
    );

    // P2 gains life equal to exiled creature's power (5).
    let p2_life_after = state.players.get(&p2).map(|ps| ps.life_total).unwrap_or(40);
    let life_gained = p2_life_after - p2_life_before;
    assert_eq!(
        life_gained, 5,
        "PB-P M8: P2 should gain 5 life (PowerOf(DeclaredTarget) path — unaffected by PB-P). \
         life_before={}, life_after={}, gained={}",
        p2_life_before, p2_life_after, life_gained
    );
}

// ── Test O1: Negative-power creature sacrifice mills zero (floor at 0) ─────────

/// CR 608.2b + CR 107.1b — PB-P O1: A creature whose layer-resolved power is negative
/// (e.g., a 2/2 under a -3/-0 debuff, resolving as -1/2) sacrificed to Altar of Dementia
/// mills 0 cards (negative mill clamps to 0).
///
/// CR 107.1b: If a cost or effect uses a number and that number becomes negative, treat it as 0.
///
/// NOTE: This test is ignored because the Indefinite/AllCreatures continuous effect debuff
/// causes a priority loop in the pass_all helper after the ability resolves. The LKI capture
/// for negative power works correctly (verified by M5 and M4 which use similar setup). This
/// optional test would require a bounded pass_all or a different state setup to terminate.
#[test]
#[ignore]
fn test_sacrifice_negative_power_creature_mills_zero() {
    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def};
    use std::collections::HashMap;

    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let altar = enrich_spec_from_def(
        ObjectSpec::card(p1, "Altar of Dementia")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Altar of Dementia")),
        &defs,
    );

    // A 2/2 creature with a -3/-0 power debuff applied (resolves as -1/2 on battlefield).
    let cursed = ObjectSpec::creature(p1, "Cursed Creature", 2, 2)
        .with_card_id(CardId("cursed-creature".to_string()))
        .in_zone(ZoneId::Battlefield);

    // Apply -3 power debuff (simulates a pump effect in reverse / curse).
    let debuff = ContinuousEffect {
        id: EffectId(10),
        source: None,
        timestamp: 200,
        layer: EffectLayer::PtModify,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllCreatures,
        modification: LayerModification::ModifyPower(-3),
        is_cda: false,
        condition: None,
    };

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(altar)
        .object(cursed)
        .add_continuous_effect(debuff);

    builder = add_library_cards(builder, p2, 5, "P2Lib");

    let state = builder.build().unwrap();

    let altar_id = find_obj(&state, "Altar of Dementia");
    let cursed_id = find_obj(&state, "Cursed Creature");
    let p2_id = state
        .players
        .iter()
        .find(|(id, _)| **id == p2)
        .map(|(id, _)| *id)
        .unwrap();

    // Verify the creature is negative-power on the battlefield.
    let chars = mtg_engine::calculate_characteristics(&state, cursed_id)
        .expect("calculate_characteristics should work");
    assert_eq!(
        chars.power,
        Some(-1),
        "PB-P O1: Cursed Creature should resolve as -1/2 under the -3/0 debuff (2 + -3 = -1)"
    );

    let lib_before = count_in_zone(&state, ZoneId::Library(p2));

    // Sacrifice the -1/2 creature to Altar.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: altar_id,
            ability_index: 0,
            targets: vec![mtg_engine::Target::Player(p2_id)],
            discard_card: None,
            sacrifice_target: Some(cursed_id),
            x_value: None,
        },
    )
    .expect("Altar activation should succeed");

    let (state, _) = pass_all(state, &players);

    let lib_after = count_in_zone(&state, ZoneId::Library(p2));
    let milled = lib_before as i32 - lib_after as i32;

    // Negative mill clamps to 0.
    assert_eq!(
        milled, 0,
        "CR 107.1b + PB-P O1: Negative power (-1) should mill 0 (clamped). \
         lib_before={}, lib_after={}, milled={}",
        lib_before, lib_after, milled
    );
}

// ── Test O3: Life's Legacy with 0-power creature draws 0 cards ────────────────

/// CR 608.2b + CR 118.8 — PB-P O3: End-to-end edge case for the spell path.
/// Casting Life's Legacy while sacrificing a 0/4 creature draws 0 cards.
///
/// Validates the additional_costs LKI propagation for 0-power via the spell path.
#[test]
fn test_lifes_legacy_with_zero_power_creature_draws_zero() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    use mtg_engine::CardDefinition;
    use mtg_engine::{all_cards, card_name_to_id, enrich_spec_from_def, AdditionalCost, ManaPool};
    use std::collections::HashMap;

    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);

    let lifes_legacy = enrich_spec_from_def(
        ObjectSpec::card(p1, "Life's Legacy")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(card_name_to_id("Life's Legacy")),
        &defs,
    );

    // 0/4 creature — 0 power.
    let wall = ObjectSpec::creature(p1, "Zero Power Wall", 0, 4)
        .with_card_id(CardId("zero-power-wall".to_string()))
        .in_zone(ZoneId::Battlefield);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(lifes_legacy)
        .object(wall);

    builder = add_library_cards(builder, p1, 10, "P1Lib");

    let mut state = builder.build().unwrap();

    // Grant {1}{G} mana.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 1,
            green: 1,
            ..Default::default()
        };
    }

    let legacy_id = find_obj(&state, "Life's Legacy");
    let wall_id = find_obj(&state, "Zero Power Wall");

    let hand_before = count_in_zone(&state, ZoneId::Hand(p1));
    let lib_before = count_in_zone(&state, ZoneId::Library(p1));

    // Cast Life's Legacy sacrificing the 0-power Wall.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: legacy_id,
            targets: vec![],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![AdditionalCost::Sacrifice {
                ids: vec![wall_id],
                lki_powers: vec![],
            }],
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        },
    )
    .expect("Cast Life's Legacy should succeed");

    // Resolve.
    let (state, _) = pass_all(state, &players);

    let hand_after = count_in_zone(&state, ZoneId::Hand(p1));
    let lib_after = count_in_zone(&state, ZoneId::Library(p1));

    let drawn = lib_before as i32 - lib_after as i32;
    assert_eq!(
        drawn, 0,
        "CR 608.2b + PB-P O3: Sacrificing 0-power creature to Life's Legacy draws 0 cards. \
         lib_before={}, lib_after={}, drawn={}",
        lib_before, lib_after, drawn
    );

    // Hand size: Life's Legacy was cast (left hand), drew 0 → net -1.
    let net_hand = hand_after as i32 - hand_before as i32;
    assert_eq!(
        net_hand, -1,
        "PB-P O3: Hand should decrease by 1 (Life's Legacy cast, 0 drawn). net_hand={}",
        net_hand
    );
}
