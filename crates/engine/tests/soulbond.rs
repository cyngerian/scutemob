//! Soulbond keyword ability tests (CR 702.95).
//!
//! Soulbond represents two triggered abilities:
//! 1. Self-ETB: When a creature with soulbond enters the battlefield, if its
//!    controller controls another unpaired creature, the controller may pair
//!    this creature with that creature (CR 702.95a, first sentence).
//! 2. Other-ETB: Whenever another creature the soulbond creature's controller
//!    controls enters, if both are unpaired, the controller may pair them
//!    (CR 702.95a, second sentence).
//!
//! Key rules verified:
//! - Pairing is symmetric: both creatures have paired_with set (CR 702.95b).
//! - A creature can be paired with only one other creature (CR 702.95d).
//! - Pairing breaks on zone change (CR 702.95e + CR 400.7).
//! - Pairing breaks on controller change (CR 702.95e).
//! - Pairing breaks when either creature stops being a creature (CR 702.95e).
//! - Resolution fizzle if either creature is invalid at resolution (CR 702.95c).
//! - WhilePaired CEs grant effects to both creatures while paired.
//! - No trigger fires if no other unpaired creature exists (CR 702.95a intervening-if).

use mtg_engine::{
    calculate_characteristics, check_and_apply_sbas, process_command, AbilityDefinition,
    CardDefinition, CardId, CardRegistry, CardType, Command, EffectLayer, GameEvent, GameState,
    GameStateBuilder, KeywordAbility, LayerModification, ManaColor, ManaCost, ObjectId, ObjectSpec,
    PlayerId, SoulbondGrant, Step, TypeLine, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
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

/// Cast a creature from hand and resolve it (both players pass priority).
fn cast_and_resolve(
    state: GameState,
    caster: PlayerId,
    card_name: &str,
    other_player: PlayerId,
) -> (GameState, Vec<GameEvent>) {
    let card_id = find_object(&state, card_name);
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: caster,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell '{}' failed: {:?}", card_name, e));

    pass_all(state, &[caster, other_player])
}

// ── Card definitions ──────────────────────────────────────────────────────────

/// 4/4 soulbond creature that grants +4/+4 to both while paired.
fn soulbond_4_4_grant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-soulbond-4-4".to_string()),
        name: "Mock Silverheart".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Soulbond\nAs long as Mock Silverheart is paired, both creatures get +4/+4."
            .to_string(),
        abilities: vec![AbilityDefinition::Soulbond {
            grants: vec![SoulbondGrant {
                layer: EffectLayer::PtModify,
                modification: LayerModification::ModifyBoth(4),
            }],
        }],
        power: Some(4),
        toughness: Some(4),
        ..Default::default()
    }
}

/// 2/2 soulbond creature with no grants (minimal soulbond creature).
fn soulbond_2_2_no_grant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-soulbond-no-grant".to_string()),
        name: "Mock Soulbond Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Soulbond".to_string(),
        abilities: vec![AbilityDefinition::Soulbond { grants: vec![] }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// 2/2 vanilla creature (no keywords, no abilities).
fn vanilla_2_2_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-vanilla-2-2".to_string()),
        name: "Mock Vanilla".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// 3/3 vanilla creature (different stats to distinguish from 2/2).
fn vanilla_3_3_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("mock-vanilla-3-3".to_string()),
        name: "Mock Big Vanilla".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

// ── Build helpers ──────────────────────────────────────────────────────────────

/// Build a state with a soulbond creature in hand and some battlefield permanents.
fn build_state(
    p1: PlayerId,
    p2: PlayerId,
    extra_defs: Vec<CardDefinition>,
    extra_objects: Vec<ObjectSpec>,
    soulbond_in_hand: bool,
    soulbond_def: CardDefinition,
) -> GameState {
    let soulbond_cid = soulbond_def.card_id.clone();
    let mut defs = vec![soulbond_def];
    defs.extend(extra_defs);
    let registry = CardRegistry::new(defs);

    let soulbond_spec = ObjectSpec::creature(p1, "Mock Silverheart", 4, 4)
        .in_zone(if soulbond_in_hand {
            ZoneId::Hand(p1)
        } else {
            ZoneId::Battlefield
        })
        .with_card_id(soulbond_cid)
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Soulbond)
        .with_mana_cost(ManaCost {
            generic: 5,
            green: 1,
            ..Default::default()
        });

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(soulbond_spec);

    for obj in extra_objects {
        builder = builder.object(obj);
    }

    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Add mana for {5}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn.priority_holder = Some(p1);

    state
}

// ── Test 1: Self-ETB trigger pairs soulbond creature with unpaired partner ────

/// CR 702.95a (first sentence) — When a creature with soulbond enters, if the
/// controller has another unpaired creature, both become paired.
#[test]
fn test_soulbond_self_etb_pairs_with_unpaired_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let vanilla = ObjectSpec::creature(p1, "Mock Vanilla", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-vanilla-2-2".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = build_state(
        p1,
        p2,
        vec![vanilla_2_2_def()],
        vec![vanilla],
        true,
        soulbond_4_4_grant_def(),
    );

    let vanilla_id = find_object(&state, "Mock Vanilla");

    // Cast the soulbond creature and resolve it + any ETB triggers.
    // pass_all resolves the spell (both pass) → creature ETB → trigger on stack → resolve.
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);

    // After spell resolves, SoulbondTrigger should be on stack.
    // One more pass_all to resolve the trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Mock Silverheart should be on battlefield");

    let silverheart_obj = state.objects.get(&silverheart_id).unwrap();
    let vanilla_obj = state.objects.get(&vanilla_id).unwrap();

    // CR 702.95b: Pairing is symmetric.
    assert_eq!(
        silverheart_obj.paired_with,
        Some(vanilla_id),
        "Silverheart should be paired with vanilla"
    );
    assert_eq!(
        vanilla_obj.paired_with,
        Some(silverheart_id),
        "Vanilla should be paired with Silverheart"
    );
}

// ── Test 2: Other-ETB trigger — soulbond creature already on battlefield ──────

/// CR 702.95a (second sentence) — When another creature enters the battlefield
/// controlled by the soulbond creature's controller, and both are unpaired,
/// the soulbond trigger fires and pairs them.
#[test]
fn test_soulbond_other_etb_pairs_with_entering_creature() {
    let p1 = p(1);
    let p2 = p(2);

    let vanilla_in_hand = ObjectSpec::creature(p1, "Mock Vanilla", 2, 2)
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("mock-vanilla-2-2".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        });

    // Silverheart starts on battlefield, vanilla in hand.
    let mut state = build_state(
        p1,
        p2,
        vec![vanilla_2_2_def()],
        vec![vanilla_in_hand],
        false,
        soulbond_4_4_grant_def(),
    );
    state.turn.priority_holder = Some(p1);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should be on battlefield");

    // Silverheart starts unpaired.
    assert_eq!(
        state.objects.get(&silverheart_id).unwrap().paired_with,
        None,
        "Silverheart should start unpaired"
    );

    // Add mana for vanilla {1}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    // Cast vanilla and resolve → OtherETB trigger for Silverheart fires → resolve.
    let (state, _) = cast_and_resolve(state, p1, "Mock Vanilla", p2);
    // Resolve the soulbond trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let vanilla_id = find_object_in_zone(&state, "Mock Vanilla", ZoneId::Battlefield)
        .expect("Vanilla should be on battlefield");

    let silverheart_obj = state.objects.get(&silverheart_id).unwrap();
    let vanilla_obj = state.objects.get(&vanilla_id).unwrap();

    // CR 702.95b: Symmetric pairing.
    assert_eq!(
        silverheart_obj.paired_with,
        Some(vanilla_id),
        "Silverheart should be paired with Vanilla"
    );
    assert_eq!(
        vanilla_obj.paired_with,
        Some(silverheart_id),
        "Vanilla should be paired with Silverheart"
    );
}

// ── Test 3: WhilePaired CEs grant P/T bonus ────────────────────────────────────

/// CR 702.95a "for as long as both remain creatures on the battlefield" —
/// While paired, both creatures receive the soulbond grants via ContinuousEffects.
#[test]
fn test_soulbond_grants_apply_while_paired() {
    let p1 = p(1);
    let p2 = p(2);

    let vanilla = ObjectSpec::creature(p1, "Mock Vanilla", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-vanilla-2-2".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = build_state(
        p1,
        p2,
        vec![vanilla_2_2_def()],
        vec![vanilla],
        true,
        soulbond_4_4_grant_def(),
    );

    let vanilla_id = find_object(&state, "Mock Vanilla");

    // Cast and resolve the soulbond creature.
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);
    // Resolve the soulbond trigger.
    let (state, _) = pass_all(state, &[p1, p2]);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should be on battlefield");

    // Both should be paired.
    assert!(
        state
            .objects
            .get(&silverheart_id)
            .unwrap()
            .paired_with
            .is_some(),
        "Silverheart should be paired"
    );
    assert!(
        state
            .objects
            .get(&vanilla_id)
            .unwrap()
            .paired_with
            .is_some(),
        "Vanilla should be paired"
    );

    // CR 702.95a: WhilePaired CE grants +4/+4 to both.
    // Silverheart base 4/4 + 4/4 grant = 8/8.
    let sh_chars = calculate_characteristics(&state, silverheart_id)
        .expect("Should calculate Silverheart characteristics");
    assert_eq!(
        sh_chars.power,
        Some(8),
        "Silverheart should have +4/+4 grant → power 8"
    );
    assert_eq!(
        sh_chars.toughness,
        Some(8),
        "Silverheart toughness should be 8"
    );

    // Vanilla base 2/2 + 4/4 grant = 6/6.
    let van_chars = calculate_characteristics(&state, vanilla_id)
        .expect("Should calculate Vanilla characteristics");
    assert_eq!(
        van_chars.power,
        Some(6),
        "Vanilla should have +4/+4 grant → power 6"
    );
    assert_eq!(
        van_chars.toughness,
        Some(6),
        "Vanilla toughness should be 6"
    );
}

// ── Test 4: Unpairing on zone change ─────────────────────────────────────────

/// CR 702.95e + CR 400.7 — When a paired creature changes zones, it becomes a
/// new object. The remaining creature should have paired_with = None.
#[test]
fn test_soulbond_unpair_on_zone_change() {
    let p1 = p(1);
    let p2 = p(2);

    let vanilla = ObjectSpec::creature(p1, "Mock Vanilla", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-vanilla-2-2".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = build_state(
        p1,
        p2,
        vec![vanilla_2_2_def()],
        vec![vanilla],
        true,
        soulbond_4_4_grant_def(),
    );

    let vanilla_id = find_object(&state, "Mock Vanilla");

    // Cast and resolve the soulbond creature, then resolve the trigger.
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);
    let (mut state, _) = pass_all(state, &[p1, p2]);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should be on battlefield");

    // Confirm they are paired.
    assert_eq!(
        state.objects.get(&silverheart_id).unwrap().paired_with,
        Some(vanilla_id)
    );

    // Manually move vanilla to graveyard (simulating death).
    // This calls move_object_to_zone which clears paired_with on both.
    state
        .move_object_to_zone(vanilla_id, ZoneId::Graveyard(p1))
        .unwrap_or_else(|e| panic!("move_object_to_zone failed: {:?}", e));

    // Silverheart should now be unpaired.
    let silverheart_obj = state.objects.get(&silverheart_id).unwrap();
    assert_eq!(
        silverheart_obj.paired_with, None,
        "Silverheart should be unpaired after vanilla died"
    );
}

// ── Test 5: No trigger if no other unpaired creature exists ───────────────────

/// CR 702.95a intervening-if — The trigger only fires if the controller has
/// another unpaired creature. If none, no trigger is generated.
#[test]
fn test_soulbond_no_trigger_if_no_unpaired_partner() {
    let p1 = p(1);
    let p2 = p(2);

    // No other creatures for p1 on the battlefield.
    let mut state = build_state(p1, p2, vec![], vec![], true, soulbond_4_4_grant_def());

    // Cast Silverheart and resolve.
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);
    // Try to resolve another trigger cycle — but none should exist.
    let (state, _) = pass_all(state, &[p1, p2]);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should be on battlefield");

    // Should be unpaired because no other creature was available.
    assert_eq!(
        state.objects.get(&silverheart_id).unwrap().paired_with,
        None,
        "Silverheart should be unpaired (no other creature)"
    );
}

// ── Test 6: Already-paired creature cannot be paired again ────────────────────

/// CR 702.95d — A creature can be paired with only one other creature.
/// If a soulbond trigger tries to pair an already-paired creature, it fizzles.
#[test]
fn test_soulbond_already_paired_cannot_repair() {
    let p1 = p(1);
    let p2 = p(2);

    // Start with two creatures on the battlefield for p1.
    let vanilla1 = ObjectSpec::creature(p1, "Mock Vanilla", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-vanilla-2-2".to_string()))
        .with_types(vec![CardType::Creature]);

    let vanilla2 = ObjectSpec::creature(p1, "Mock Big Vanilla", 3, 3)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-vanilla-3-3".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = build_state(
        p1,
        p2,
        vec![vanilla_2_2_def(), vanilla_3_3_def()],
        vec![vanilla1, vanilla2],
        true,
        soulbond_4_4_grant_def(),
    );

    let vanilla_id = find_object_in_zone(&state, "Mock Vanilla", ZoneId::Battlefield)
        .expect("vanilla should be on battlefield");

    // Manually pair vanilla1 with vanilla2 to simulate them already being paired.
    let vanilla2_id = find_object_in_zone(&state, "Mock Big Vanilla", ZoneId::Battlefield)
        .expect("vanilla2 should be on battlefield");
    state.objects.get_mut(&vanilla_id).unwrap().paired_with = Some(vanilla2_id);
    state.objects.get_mut(&vanilla2_id).unwrap().paired_with = Some(vanilla_id);

    // Cast Silverheart — it enters, checks for unpaired creatures.
    // Both vanillas are paired, so the intervening-if should not find an unpaired partner.
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);
    let (state, _) = pass_all(state, &[p1, p2]);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should be on battlefield");

    // Silverheart should remain unpaired — no unpaired partner available.
    assert_eq!(
        state.objects.get(&silverheart_id).unwrap().paired_with,
        None,
        "Silverheart should be unpaired (no unpaired creatures available)"
    );

    // The pre-existing pair should be undisturbed.
    assert_eq!(
        state.objects.get(&vanilla_id).unwrap().paired_with,
        Some(vanilla2_id),
        "Pre-existing pair should remain intact"
    );
}

// ── Test 7: Resolution fizzle — target leaves before resolution ───────────────

/// CR 702.95c — If either creature is no longer on the battlefield as a creature
/// controlled by the soulbond ability's controller, neither becomes paired.
#[test]
fn test_soulbond_resolution_fizzle_target_leaves() {
    let p1 = p(1);
    let p2 = p(2);

    let vanilla = ObjectSpec::creature(p1, "Mock Vanilla", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-vanilla-2-2".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = build_state(
        p1,
        p2,
        vec![vanilla_2_2_def()],
        vec![vanilla],
        true,
        soulbond_4_4_grant_def(),
    );

    let vanilla_id = find_object(&state, "Mock Vanilla");

    // Cast Silverheart — ETB trigger fires, goes on stack.
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);

    // At this point, the soulbond trigger should be on the stack.
    // Before it resolves, kill the vanilla creature (zone change = fizzle).
    let mut state = state;
    state
        .move_object_to_zone(vanilla_id, ZoneId::Graveyard(p1))
        .unwrap_or_else(|e| panic!("move_object_to_zone failed: {:?}", e));

    // Now resolve the trigger — it should fizzle.
    let (state, _) = pass_all(state, &[p1, p2]);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should still be on battlefield");

    // Neither creature is paired — fizzle.
    assert_eq!(
        state.objects.get(&silverheart_id).unwrap().paired_with,
        None,
        "Silverheart should be unpaired after fizzle"
    );
}

// ── Test 8: SBA unpairing on controller change ────────────────────────────────

/// CR 702.95e — A paired creature becomes unpaired if its controller changes.
#[test]
fn test_soulbond_unpair_on_controller_change() {
    let p1 = p(1);
    let p2 = p(2);

    let vanilla = ObjectSpec::creature(p1, "Mock Vanilla", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-vanilla-2-2".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = build_state(
        p1,
        p2,
        vec![vanilla_2_2_def()],
        vec![vanilla],
        true,
        soulbond_4_4_grant_def(),
    );

    let vanilla_id = find_object(&state, "Mock Vanilla");

    // Pair them up.
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);
    let (mut state, _) = pass_all(state, &[p1, p2]);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should be on battlefield");

    // Confirm paired.
    assert_eq!(
        state.objects.get(&silverheart_id).unwrap().paired_with,
        Some(vanilla_id)
    );

    // Simulate controller change: give vanilla to p2.
    state.objects.get_mut(&vanilla_id).unwrap().controller = p2;

    // Run SBAs — CR 702.95e should clear the pairing.
    let _events = check_and_apply_sbas(&mut state);

    assert_eq!(
        state.objects.get(&silverheart_id).unwrap().paired_with,
        None,
        "Silverheart should be unpaired after vanilla changed controller"
    );
    assert_eq!(
        state.objects.get(&vanilla_id).unwrap().paired_with,
        None,
        "Vanilla should be unpaired after controller change"
    );
}

// ── Test 9: Grants removed when unpairing occurs ──────────────────────────────

/// CR 702.95a "for as long as" — WhilePaired CEs stop applying once paired_with
/// is cleared, so P/T grants revert to base values.
#[test]
fn test_soulbond_grants_removed_when_unpaired() {
    let p1 = p(1);
    let p2 = p(2);

    let vanilla = ObjectSpec::creature(p1, "Mock Vanilla", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-vanilla-2-2".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = build_state(
        p1,
        p2,
        vec![vanilla_2_2_def()],
        vec![vanilla],
        true,
        soulbond_4_4_grant_def(),
    );

    let vanilla_id = find_object(&state, "Mock Vanilla");

    // Pair them.
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);
    let (mut state, _) = pass_all(state, &[p1, p2]);

    let silverheart_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should be on battlefield");

    // Confirm grant applies: vanilla should be 6/6.
    let van_chars = calculate_characteristics(&state, vanilla_id).unwrap();
    assert_eq!(
        van_chars.power,
        Some(6),
        "Vanilla should be 6/6 while paired"
    );

    // Break the pair by moving vanilla to graveyard.
    state
        .move_object_to_zone(vanilla_id, ZoneId::Graveyard(p1))
        .unwrap();

    // Silverheart is now unpaired — its grants should not apply to itself either.
    let sh_chars = calculate_characteristics(&state, silverheart_id).unwrap();
    assert_eq!(
        sh_chars.power,
        Some(4),
        "Silverheart should revert to base 4/4 after unpairing"
    );
    assert_eq!(sh_chars.toughness, Some(4));
}

// ── Test 10: Two soulbond creatures pair with each other ─────────────────────

/// CR 702.95a — A creature with soulbond can pair with another soulbond creature.
/// When Silverheart enters, it triggers SoulbondSelfETB (target=sb2) and the
/// existing sb2 triggers SoulbondOtherETB (source=sb2, target=Silverheart).
/// Both triggers go on the stack; whichever resolves first succeeds.
/// The second trigger fizzles (CR 702.95d: already paired).
/// Pairing is symmetric (CR 702.95b).
#[test]
fn test_soulbond_self_etb_pairs_with_other_soulbond_creature() {
    let p1 = p(1);
    let p2 = p(2);

    // A second soulbond creature (no-grant variant) already on battlefield.
    let soulbond2 = ObjectSpec::creature(p1, "Mock Soulbond Creature", 2, 2)
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("mock-soulbond-no-grant".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Soulbond);

    let state = build_state(
        p1,
        p2,
        vec![soulbond_2_2_no_grant_def()],
        vec![soulbond2],
        true,
        soulbond_4_4_grant_def(),
    );

    let sb2_id = find_object_in_zone(&state, "Mock Soulbond Creature", ZoneId::Battlefield)
        .expect("Second soulbond creature should be on battlefield");

    // Cast Silverheart — two triggers fire (SelfETB + OtherETB).
    // Resolve the spell and all triggered abilities (3 pass_all: spell + trigger1 + trigger2).
    let (state, _) = cast_and_resolve(state, p1, "Mock Silverheart", p2);
    let (state, _) = pass_all(state, &[p1, p2]);
    // Drain any second trigger (fizzles, but still needs to be resolved).
    let (state, _) = pass_all(state, &[p1, p2]);

    let sh_id = find_object_in_zone(&state, "Mock Silverheart", ZoneId::Battlefield)
        .expect("Silverheart should be on battlefield");
    let sb2_id_after = find_object_in_zone(&state, "Mock Soulbond Creature", ZoneId::Battlefield)
        .expect("Second soulbond creature should be on battlefield");

    // sb2_id may have changed if zone transitions occurred; use the current id.
    let _ = sb2_id; // the pre-cast id (may differ from post-cast on battlefield)

    let sh_obj = state.objects.get(&sh_id).unwrap();
    let sb2_obj = state.objects.get(&sb2_id_after).unwrap();

    // CR 702.95b: Symmetric pairing — one of the two triggers succeeded.
    assert_eq!(
        sh_obj.paired_with,
        Some(sb2_id_after),
        "Silverheart should be paired with the other soulbond creature"
    );
    assert_eq!(
        sb2_obj.paired_with,
        Some(sh_id),
        "Second soulbond creature should be paired with Silverheart"
    );
}
