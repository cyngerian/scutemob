//! Tests for PB-EWC-D: `ObjectFilter::CreatureControlledByOfSubtype` +
//! `bind_object_filter` `OwnedByOpponentsOf` rebind.
//!
//! This primitive unblocks Dragonstorm Globe: "Each Dragon you control enters
//! with an additional +1/+1 counter on it." The existing `ObjectFilter` enum
//! (PB-CD) can express `CreatureControlledBy(PlayerId)` but could not pin a
//! subtype receiver. The new variant adds subtype-specific receiver filtering
//! (CR 614.1c / CR 613.1d).
//!
//! Also closes sub-gap E2 from pb-review-EWC.md: `bind_object_filter` now
//! rebinds `OwnedByOpponentsOf(PlayerId(0))` → `OwnedByOpponentsOf(controller)`,
//! symmetrically to the WouldChangeZone direct pattern already present.
//!
//! Tests:
//!   (1) `test_pb_ewcd_hash_schema_version_is_23` — HASH-23 sentinel.
//!   (2) `test_pb_ewcd_partial_eq_discriminates_subtype_variant` — PartialEq
//!       variant equality and inequality.
//!   (3) `test_pb_ewcd_serde_default_backward_compat` — JSON roundtrip for
//!       the new variant.
//!   (4) `test_dragonstorm_globe_dragon_etb_gets_extra_counter` — positive
//!       functional: Dragon entering under Globe gets 1 +1/+1 counter.
//!   (5) `test_dragonstorm_globe_non_dragon_etb_no_counter` — negative
//!       functional: non-Dragon entering under Globe gets no counter.
//!   (6) `test_bind_object_filter_rebinds_owned_by_opponents_of_for_wouldenterbattlefield`
//!       — E2 regression: OwnedByOpponentsOf(PlayerId(0)) placeholder is rebound
//!       to the actual controller at registration time.

use mtg_engine::rules::command::CastSpellData;
use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::types::{CounterType, SubType};
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ManaColor, ObjectFilter,
    ObjectId, ObjectSpec, PlayerId, ReplacementModification, ReplacementTrigger, Step,
    HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

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

fn find_on_battlefield(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
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

fn build_defs_and_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

fn enrich(
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

fn cast_creature(
    mut state: GameState,
    caster: PlayerId,
    card: ObjectId,
    mana: &[(ManaColor, u32)],
) -> (GameState, Vec<GameEvent>) {
    {
        let pool = &mut state.players_mut().get_mut(&caster).unwrap().mana_pool;
        for &(color, n) in mana {
            if n > 0 {
                pool.add(color, n);
            }
        }
    }
    state.turn_mut().priority_holder = Some(caster);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: caster,
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
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e))
}

// ── Test 1: Hash schema sentinel ──────────────────────────────────────────────

/// HASH_SCHEMA_VERSION live sentinel — fails if the schema version drifts
/// without this test being updated. See the `state/hash.rs` history block.
#[test]
fn test_pb_ewcd_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 55u8,
        "HASH_SCHEMA_VERSION drifted without this sentinel being updated. Bump this assertion and the state/hash.rs history block together; the authoritative check is the SR-17 machine gate in tests/core/hash_schema.rs."
    );
}

// ── Test 2: PartialEq discriminator ───────────────────────────────────────────

/// PB-EWC-D (2): The new `CreatureControlledByOfSubtype` variant correctly
/// implements `PartialEq` — two values differ when controller OR subtype differs,
/// and are equal only when both fields match. Also verifies it differs from the
/// unrelated `CreatureControlledBy` variant.
///
/// CR 613.1d — subtype membership is part of the match predicate; both fields
/// must participate in equality for correctness.
#[test]
fn test_pb_ewcd_partial_eq_discriminates_subtype_variant() {
    let p1 = p(1);
    let p2 = p(2);

    let dragon_p1 = ObjectFilter::CreatureControlledByOfSubtype {
        controller: p1,
        subtype: SubType("Dragon".to_string()),
    };
    let dragon_p2 = ObjectFilter::CreatureControlledByOfSubtype {
        controller: p2,
        subtype: SubType("Dragon".to_string()),
    };
    let zombie_p1 = ObjectFilter::CreatureControlledByOfSubtype {
        controller: p1,
        subtype: SubType("Zombie".to_string()),
    };
    let dragon_p1_again = ObjectFilter::CreatureControlledByOfSubtype {
        controller: p1,
        subtype: SubType("Dragon".to_string()),
    };
    let creature_controlled_by_p1 = ObjectFilter::CreatureControlledBy(p1);

    // Same controller, same subtype → equal.
    assert_eq!(
        dragon_p1, dragon_p1_again,
        "Two CreatureControlledByOfSubtype values with same controller and subtype must be equal"
    );

    // Same subtype, different controller → not equal.
    assert_ne!(
        dragon_p1, dragon_p2,
        "Different controller must produce inequality"
    );

    // Same controller, different subtype → not equal.
    assert_ne!(
        dragon_p1, zombie_p1,
        "Different subtype must produce inequality"
    );

    // Different variant (CreatureControlledBy vs CreatureControlledByOfSubtype) → not equal.
    assert_ne!(
        dragon_p1, creature_controlled_by_p1,
        "CreatureControlledByOfSubtype must not equal CreatureControlledBy with same PlayerId"
    );
}

// ── Test 3: Serde roundtrip ────────────────────────────────────────────────────

/// PB-EWC-D (3): The new `ObjectFilter::CreatureControlledByOfSubtype` variant
/// round-trips via JSON serde. The variant is additive — no pre-EWC-D serialized
/// state will contain it, so backward compatibility (old → new deserialization)
/// is not a concern. The forward-compat check (new variant round-trips) proves
/// the wire format is stable.
///
/// CR 613.1d / 614.1c — the filter is part of `WouldEnterBattlefield` replacement
/// triggers which are serialized in `GameState`.
#[test]
fn test_pb_ewcd_serde_default_backward_compat() {
    let p1 = p(1);
    let original = ObjectFilter::CreatureControlledByOfSubtype {
        controller: p1,
        subtype: SubType("Dragon".to_string()),
    };

    let json = serde_json::to_string(&original)
        .expect("CreatureControlledByOfSubtype must serialize to JSON");
    let round_tripped: ObjectFilter =
        serde_json::from_str(&json).expect("CreatureControlledByOfSubtype must deserialize");

    assert_eq!(
        round_tripped, original,
        "CreatureControlledByOfSubtype must round-trip via serde identically"
    );

    // Confirm the existing variants still deserialize (additive change verification).
    let json_any = r#""Any""#;
    let parsed: ObjectFilter =
        serde_json::from_str(json_any).expect("ObjectFilter::Any must still deserialize");
    assert_eq!(parsed, ObjectFilter::Any);

    let json_creature = r#"{"CreatureControlledBy": 1}"#;
    let parsed_creature: ObjectFilter =
        serde_json::from_str(json_creature).expect("CreatureControlledBy must still deserialize");
    assert_eq!(parsed_creature, ObjectFilter::CreatureControlledBy(p(1)));
}

// ── Test 4: Dragon ETB gets extra counter from Dragonstorm Globe ──────────────

/// CR 614.1c / CR 613.1d — Dragonstorm Globe is on the battlefield (registered
/// its ETB replacement for `CreatureControlledByOfSubtype { controller: p1,
/// subtype: Dragon }`). When a Dragon creature (Parapet Thrasher, 4/3) enters
/// under the same controller, it must enter with exactly 1 additional +1/+1
/// counter from Globe's replacement.
///
/// Parapet Thrasher is a Creature — Dragon {2}{R}{R} 4/3.
#[test]
fn test_dragonstorm_globe_dragon_etb_gets_extra_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    // Globe needs 3 generic mana; Parapet Thrasher needs {2}{R}{R} = 4 mana.
    let globe_spec = enrich(p1, "Dragonstorm Globe", ZoneId::Hand(p1), &defs);
    let dragon_spec = enrich(p1, "Parapet Thrasher", ZoneId::Hand(p1), &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(globe_spec)
        .object(dragon_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let globe_id = find_by_name(&state, "Dragonstorm Globe");

    // Cast Dragonstorm Globe for {3} (Artifact, no creature types).
    let (state, _) = cast_creature(state, p1, globe_id, &[(ManaColor::Colorless, 3)]);
    // Resolve Globe (passes priority for both players to resolve the spell).
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        find_on_battlefield(&state, "Dragonstorm Globe").is_some(),
        "Dragonstorm Globe must be on battlefield after casting"
    );

    // Verify the replacement is registered with the Dragon subtype filter, bound
    // to p1 (not the PlayerId(0) placeholder).
    let globe_repl_count: usize = state
        .replacement_effects()
        .iter()
        .filter(|e| {
            matches!(
                &e.trigger,
                ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::CreatureControlledByOfSubtype {
                        controller: pid,
                        subtype,
                    }
                } if *pid == p1 && subtype.0 == "Dragon"
            ) && matches!(
                &e.modification,
                ReplacementModification::EntersWithCounters { .. }
            )
        })
        .count();
    assert_eq!(
        globe_repl_count, 1,
        "Dragonstorm Globe's EntersWithCounters replacement should be registered \
         with CreatureControlledByOfSubtype bound to p1 (placeholder must not survive \
         registration)"
    );

    // Now cast a Dragon — Parapet Thrasher {2}{R}{R} = 2 colorless + 2 red.
    let dragon_id = find_by_name(&state, "Parapet Thrasher");
    let (state, _) = cast_creature(
        state,
        p1,
        dragon_id,
        &[(ManaColor::Colorless, 2), (ManaColor::Red, 2)],
    );
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_dragon = find_on_battlefield(&state, "Parapet Thrasher")
        .expect("Parapet Thrasher must be on battlefield after casting");
    let counter_count = state
        .objects()
        .get(&bf_dragon)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);

    assert_eq!(
        counter_count, 1,
        "CR 614.1c: Dragon (Parapet Thrasher) entering under Dragonstorm Globe \
         must receive exactly 1 additional +1/+1 counter. Got {}.",
        counter_count
    );
}

// ── Test 5: Non-Dragon ETB gets no counter from Dragonstorm Globe ─────────────

/// CR 614.1c / CR 613.1d — Dragonstorm Globe is on the battlefield. When a
/// non-Dragon creature (Elvish Mystic, Elf Druid 1/1) enters under the same
/// controller, it must NOT receive a +1/+1 counter — the ObjectFilter subtype
/// check must filter it out.
#[test]
fn test_dragonstorm_globe_non_dragon_etb_no_counter() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let globe_spec = enrich(p1, "Dragonstorm Globe", ZoneId::Hand(p1), &defs);
    let non_dragon_spec = enrich(p1, "Elvish Mystic", ZoneId::Hand(p1), &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(globe_spec)
        .object(non_dragon_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let globe_id = find_by_name(&state, "Dragonstorm Globe");

    // Cast Dragonstorm Globe for {3}.
    let (state, _) = cast_creature(state, p1, globe_id, &[(ManaColor::Colorless, 3)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        find_on_battlefield(&state, "Dragonstorm Globe").is_some(),
        "Dragonstorm Globe must be on battlefield after casting"
    );

    // Cast Elvish Mystic for {G}.
    let mystic_id = find_by_name(&state, "Elvish Mystic");
    let (state, _) = cast_creature(state, p1, mystic_id, &[(ManaColor::Green, 1)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_mystic = find_on_battlefield(&state, "Elvish Mystic")
        .expect("Elvish Mystic must be on battlefield after casting");
    let counter_count = state
        .objects()
        .get(&bf_mystic)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);

    assert_eq!(
        counter_count, 0,
        "CR 614.1c: Non-Dragon (Elvish Mystic, Elf Druid) entering under \
         Dragonstorm Globe must NOT receive a +1/+1 counter. Got {}.",
        counter_count
    );
}

// ── Test 6: OwnedByOpponentsOf(PlayerId(0)) rebind regression ────────────────

/// PB-EWC-D (E2 fix from pb-review-EWC.md): `bind_object_filter` now rebinds
/// `OwnedByOpponentsOf(PlayerId(0))` → `OwnedByOpponentsOf(controller)` for any
/// filter passed through `bind_object_filter`, including `WouldEnterBattlefield`.
///
/// This test calls `bind_object_filter` directly and asserts the result equals
/// `OwnedByOpponentsOf(actual_controller)` — NOT the placeholder
/// `OwnedByOpponentsOf(PlayerId(0))`. Also asserts that passthrough cases (Any,
/// AnyCreature, ControlledBy(non-zero), etc.) are returned unchanged. And asserts
/// the same rebind for `CreatureControlledByOfSubtype` (the EWC-D primary variant).
///
/// CR 614.12: replacement effects bind the controller's identity at registration
/// time; the `PlayerId(0)` placeholder must never survive into `state.replacement_effects()`.
#[test]
fn test_bind_object_filter_rebinds_owned_by_opponents_of_for_wouldenterbattlefield() {
    use mtg_engine::bind_object_filter;
    use mtg_engine::state::types::SubType;
    use mtg_engine::ObjectFilter;

    let controller = PlayerId(7);

    // ── Rebind: OwnedByOpponentsOf(PlayerId(0)) → OwnedByOpponentsOf(controller) ──
    let placeholder_ooo = ObjectFilter::OwnedByOpponentsOf(PlayerId(0));
    let rebound_ooo = bind_object_filter(&placeholder_ooo, controller);
    assert_eq!(
        rebound_ooo,
        ObjectFilter::OwnedByOpponentsOf(controller),
        "bind_object_filter must rebind OwnedByOpponentsOf(PlayerId(0)) to \
         OwnedByOpponentsOf(controller); got {:?}",
        rebound_ooo
    );

    // Non-placeholder OwnedByOpponentsOf must be returned unchanged.
    let non_placeholder_ooo = ObjectFilter::OwnedByOpponentsOf(PlayerId(3));
    let unchanged_ooo = bind_object_filter(&non_placeholder_ooo, controller);
    assert_eq!(
        unchanged_ooo,
        ObjectFilter::OwnedByOpponentsOf(PlayerId(3)),
        "bind_object_filter must NOT rebind OwnedByOpponentsOf with a non-zero PlayerId"
    );

    // ── Rebind: CreatureControlledByOfSubtype{PlayerId(0), Dragon} → {controller, Dragon} ──
    let placeholder_ccs = ObjectFilter::CreatureControlledByOfSubtype {
        controller: PlayerId(0),
        subtype: SubType("Dragon".to_string()),
    };
    let rebound_ccs = bind_object_filter(&placeholder_ccs, controller);
    assert_eq!(
        rebound_ccs,
        ObjectFilter::CreatureControlledByOfSubtype {
            controller,
            subtype: SubType("Dragon".to_string()),
        },
        "bind_object_filter must rebind CreatureControlledByOfSubtype{{PlayerId(0), Dragon}} \
         to {{controller, Dragon}}; got {:?}",
        rebound_ccs
    );

    // ── Passthrough cases — should be returned unchanged ──────────────────────
    let passthrough_cases: &[ObjectFilter] = &[
        ObjectFilter::Any,
        ObjectFilter::AnyCreature,
        ObjectFilter::ControlledBy(PlayerId(3)), // non-zero, must not be rebound
        ObjectFilter::OwnedByOpponentsOf(PlayerId(5)), // non-zero, must not be rebound
        ObjectFilter::CreatureControlledByOfSubtype {
            controller: PlayerId(2), // non-zero, must not be rebound
            subtype: SubType("Goblin".to_string()),
        },
    ];
    for filter in passthrough_cases {
        let result = bind_object_filter(filter, controller);
        assert_eq!(
            &result, filter,
            "bind_object_filter must return {:?} unchanged (passthrough case); got {:?}",
            filter, result
        );
    }
}

// ── Test 7: Hash determinism for CreatureControlledByOfSubtype ────────────────

/// PB-EWC-D (E1): The new `ObjectFilter::CreatureControlledByOfSubtype` variant
/// produces deterministic hashes. Two structurally equal values must produce
/// identical `HashInto` bytes; structurally different values must not.
///
/// CR 613.1d — subtype membership is part of the receiver filter predicate;
/// the hash must encode both `controller` and `subtype` to be sound.
#[test]
fn test_pb_ewcd_hash_determinism_for_creature_controlled_by_of_subtype() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;
    use mtg_engine::state::types::SubType;
    use mtg_engine::ObjectFilter;

    let hash = |f: &ObjectFilter| {
        let mut h = Hasher::new();
        f.hash_into(&mut h);
        *h.finalize().as_bytes()
    };

    let dragon_p1_a = ObjectFilter::CreatureControlledByOfSubtype {
        controller: PlayerId(1),
        subtype: SubType("Dragon".to_string()),
    };
    let dragon_p1_b = ObjectFilter::CreatureControlledByOfSubtype {
        controller: PlayerId(1),
        subtype: SubType("Dragon".to_string()),
    };
    let dragon_p2 = ObjectFilter::CreatureControlledByOfSubtype {
        controller: PlayerId(2),
        subtype: SubType("Dragon".to_string()),
    };
    let goblin_p1 = ObjectFilter::CreatureControlledByOfSubtype {
        controller: PlayerId(1),
        subtype: SubType("Goblin".to_string()),
    };
    let creature_controlled_by_p1 = ObjectFilter::CreatureControlledBy(PlayerId(1));

    // Two equal instances must hash identically.
    assert_eq!(
        hash(&dragon_p1_a),
        hash(&dragon_p1_b),
        "CreatureControlledByOfSubtype{{p1, Dragon}} must have deterministic hash"
    );

    // Different controller must produce a different hash.
    assert_ne!(
        hash(&dragon_p1_a),
        hash(&dragon_p2),
        "CreatureControlledByOfSubtype{{p1, Dragon}} vs {{p2, Dragon}} must hash differently \
         (controller contributes to hash)"
    );

    // Different subtype must produce a different hash.
    assert_ne!(
        hash(&dragon_p1_a),
        hash(&goblin_p1),
        "CreatureControlledByOfSubtype{{p1, Dragon}} vs {{p1, Goblin}} must hash differently \
         (subtype contributes to hash)"
    );

    // Different variant must produce a different hash.
    assert_ne!(
        hash(&dragon_p1_a),
        hash(&creature_controlled_by_p1),
        "CreatureControlledByOfSubtype must hash differently from CreatureControlledBy \
         with the same PlayerId (discriminant byte 9 vs 8)"
    );
}
