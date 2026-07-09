//! Tests for PB-EAT: `ReplacementModification::EntersAsAdditionalType
//! { subtype: SubType }` (CR 614.1c).
//!
//! This primitive unblocks the type-grant half of Master Biomancer
//! ("...and as a Mutant in addition to its other types"). CR 614.1c entry
//! modification: the subtype is pushed into the entering permanent's
//! `characteristics.subtypes` BEFORE `PermanentEnteredBattlefield` is emitted,
//! so ETB triggers and SBAs observe the augmented type set on the very turn
//! it enters. This is NOT a Layer 4 continuous type-adding effect (which would
//! only apply to permanents already on the battlefield and would not alter the
//! entering object's own characteristics at ETB time).
//!
//! Tests:
//!   (A) Hash schema sentinel: `HASH_SCHEMA_VERSION == 21` (PB-EAT bump).
//!   (B) PartialEq discriminator: two `EntersAsAdditionalType` values that
//!       differ only in subtype are not equal.
//!   (C) Serde forward-compat: pre-PB-EAT-serialized ReplacementModification
//!       values (e.g. `EntersTapped`, `EntersWithCounters`) deserialize
//!       unchanged after the new variant was added (variants are additive).
//!   (D) Master Biomancer functional test: another creature ETBs and gains
//!       BOTH the Mutant subtype AND the +1/+1 counters from PB-EWC's count
//!       side in the same ETB.
//!   (E) Independence: a creature that already has Mutant (printed) gains the
//!       counters but the subtype set is unaffected (idempotent OrdSet insert).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::types::{CounterType, SubType};
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ManaColor, ObjectId, ObjectSpec,
    PlayerId, ReplacementModification, Step, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_on_battlefield(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
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
        let pool = &mut state.players.get_mut(&caster).unwrap().mana_pool;
        for &(color, n) in mana {
            if n > 0 {
                pool.add(color, n);
            }
        }
    }
    state.turn.priority_holder = Some(caster);
    process_command(
        state,
        Command::CastSpell {
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell failed: {:?}", e))
}

// ── Test A: Hash schema sentinel ──────────────────────────────────────────────

/// HASH_SCHEMA_VERSION live sentinel — fails if the schema version drifts
/// without this test being updated. See the `state/hash.rs` history block.
#[test]
fn test_pb_eat_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 34u8,
        "BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (GameEvent::CreatureDied.pre_death_characteristics: Option<Characteristics>, CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers). If you bumped again, update this test and state/hash.rs history."
    );
}

// ── Test B: PartialEq discriminator ───────────────────────────────────────────

/// PB-EAT B-1: two `EntersAsAdditionalType` values that differ only in subtype
/// must not be equal under `PartialEq` (the field participates in equality —
/// relied on by replacement-effect dedup and continuous-effect cache equality).
#[test]
fn test_pb_eat_partial_eq_distinguishes_subtype() {
    let mutant = ReplacementModification::EntersAsAdditionalType {
        subtype: SubType("Mutant".to_string()),
    };
    let zombie = ReplacementModification::EntersAsAdditionalType {
        subtype: SubType("Zombie".to_string()),
    };
    assert_ne!(
        mutant, zombie,
        "EntersAsAdditionalType subtype difference must be observable via PartialEq"
    );
    let mutant_again = ReplacementModification::EntersAsAdditionalType {
        subtype: SubType("Mutant".to_string()),
    };
    assert_eq!(
        mutant, mutant_again,
        "Two EntersAsAdditionalType values with the same SubType must be PartialEq-equal"
    );
}

// ── Test C: Serde forward-compat for pre-PB-EAT snapshots ─────────────────────

/// PB-EAT C-1: Pre-PB-EAT serialized `ReplacementModification` values (which
/// only ever contained the variants present at the time) must continue to
/// deserialize after PB-EAT added a new variant. Adding a variant to an
/// externally-tagged serde enum is additive — it does not break parsing of
/// existing tag values. This is the analog of "serde-default deserialization
/// of pre-PB-EAT snapshots" called out by the PB-EAT acceptance criteria.
#[test]
fn test_pb_eat_serde_pre_pb_eat_snapshots_deserialize() {
    // (i) "EntersTapped" — a unit variant present pre-PB-EAT.
    let json_unit = r#""EntersTapped""#;
    let parsed_unit: ReplacementModification = serde_json::from_str(json_unit)
        .expect("pre-PB-EAT EntersTapped snapshot must deserialize after the new variant");
    assert_eq!(parsed_unit, ReplacementModification::EntersTapped);

    // (ii) "EntersWithCounters" — a struct variant present pre-PB-EAT (PB-EWC
    // changed `count` to EffectAmount::Fixed). Confirms that adding the
    // PB-EAT variant did not break the externally-tagged wire format.
    let json_ewc = r#"{
        "EntersWithCounters": {
            "counter": "PlusOnePlusOne",
            "count": { "Fixed": 1 }
        }
    }"#;
    let parsed_ewc: ReplacementModification = serde_json::from_str(json_ewc)
        .expect("pre-PB-EAT EntersWithCounters snapshot must deserialize after the new variant");
    assert!(
        matches!(
            parsed_ewc,
            ReplacementModification::EntersWithCounters {
                counter: CounterType::PlusOnePlusOne,
                ..
            }
        ),
        "EntersWithCounters round-trip should recover the counter variant; got {:?}",
        parsed_ewc
    );

    // (iii) Round-trip: the new variant itself serializes and deserializes
    // identically (the wire format the engine emits from PB-EAT onward).
    let original = ReplacementModification::EntersAsAdditionalType {
        subtype: SubType("Mutant".to_string()),
    };
    let serialized = serde_json::to_string(&original).expect("EntersAsAdditionalType serializes");
    let round_tripped: ReplacementModification =
        serde_json::from_str(&serialized).expect("EntersAsAdditionalType round-trips");
    assert_eq!(
        round_tripped, original,
        "EntersAsAdditionalType must round-trip via serde"
    );
}

// ── Test D: Master Biomancer adds Mutant subtype AND +1/+1 counters ───────────

/// CR 614.1c — Master Biomancer is on the battlefield. When Elvish Mystic
/// (printed types: Elf Druid) enters under the same controller, BOTH of MB's
/// ETB replacements fire:
///   1. PB-EWC: 2 +1/+1 counters (= MB's live power).
///   2. PB-EAT: the Mutant subtype is added to its `characteristics.subtypes`
///      BEFORE `PermanentEnteredBattlefield` is emitted.
///
/// Both modifications must be observable on the final battlefield state.
#[test]
fn test_master_biomancer_grants_mutant_subtype_and_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let mb_spec = enrich(p1, "Master Biomancer", ZoneId::Hand(p1), &defs);
    let mystic_spec = enrich(p1, "Elvish Mystic", ZoneId::Hand(p1), &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mb_spec)
        .object(mystic_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mb_in_hand = find_by_name(&state, "Master Biomancer");
    let (state, _) = cast_creature(
        state,
        p1,
        mb_in_hand,
        &[
            (ManaColor::Colorless, 2),
            (ManaColor::Green, 1),
            (ManaColor::Blue, 1),
        ],
    );
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        find_on_battlefield(&state, "Master Biomancer").is_some(),
        "Master Biomancer must be on battlefield after casting"
    );

    let mystic_id = find_by_name(&state, "Elvish Mystic");
    let (state, _) = cast_creature(state, p1, mystic_id, &[(ManaColor::Green, 1)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_mystic =
        find_on_battlefield(&state, "Elvish Mystic").expect("Elvish Mystic must be on battlefield");
    let mystic_obj = state.objects.get(&bf_mystic).unwrap();

    // PB-EAT half: Mutant subtype was added during entry.
    assert!(
        mystic_obj
            .characteristics
            .subtypes
            .contains(&SubType("Mutant".to_string())),
        "CR 614.1c (PB-EAT): Elvish Mystic must have the Mutant subtype added \
         by Master Biomancer's EntersAsAdditionalType replacement. \
         Got subtypes: {:?}",
        mystic_obj.characteristics.subtypes
    );

    // Sanity: the printed subtypes are preserved (the OrdSet insert is purely
    // additive — CR 205.3 / 614.1c).
    assert!(
        mystic_obj
            .characteristics
            .subtypes
            .contains(&SubType("Elf".to_string())),
        "Printed subtype 'Elf' must be preserved; got: {:?}",
        mystic_obj.characteristics.subtypes
    );
    assert!(
        mystic_obj
            .characteristics
            .subtypes
            .contains(&SubType("Druid".to_string())),
        "Printed subtype 'Druid' must be preserved; got: {:?}",
        mystic_obj.characteristics.subtypes
    );

    // PB-EWC half: the +1/+1 counters from MB's power are still present
    // (verifies that PB-EAT did not regress PB-EWC's resolver path).
    let counter_count = mystic_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "PB-EWC + PB-EAT must BOTH apply in the same ETB: Elvish Mystic must \
         have 2 +1/+1 counters (MB's printed power). Got {}.",
        counter_count
    );
}

// ── Test E: EntersAsAdditionalType is idempotent when subtype already present ─

/// CR 614.1c / CR 205.3: a creature whose printed type set already includes
/// the granted subtype should remain unchanged in its `subtypes` (the OrdSet
/// insert is a no-op). Counters from PB-EWC still apply.
///
/// Setup: Simic Initiate (printed types: Human Mutant — confirmed in
/// `cards/defs/simic_initiate.rs`). Under Master Biomancer it should still
/// be Mutant (single entry, not double).
#[test]
fn test_eat_idempotent_when_subtype_already_present() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let mb_spec = enrich(p1, "Master Biomancer", ZoneId::Hand(p1), &defs);
    let initiate_spec = enrich(p1, "Simic Initiate", ZoneId::Hand(p1), &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(mb_spec)
        .object(initiate_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mb_in_hand = find_by_name(&state, "Master Biomancer");
    let (state, _) = cast_creature(
        state,
        p1,
        mb_in_hand,
        &[
            (ManaColor::Colorless, 2),
            (ManaColor::Green, 1),
            (ManaColor::Blue, 1),
        ],
    );
    let (state, _) = pass_all(state, &[p1, p2]);

    let initiate_id = find_by_name(&state, "Simic Initiate");
    let (state, _) = cast_creature(state, p1, initiate_id, &[(ManaColor::Green, 1)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_initiate = find_on_battlefield(&state, "Simic Initiate")
        .expect("Simic Initiate must be on battlefield");
    let initiate_obj = state.objects.get(&bf_initiate).unwrap();

    // Mutant is still present (printed + replacement — both contribute, OrdSet
    // dedups).
    assert!(
        initiate_obj
            .characteristics
            .subtypes
            .contains(&SubType("Mutant".to_string())),
        "Mutant must be present (printed + replacement). Got: {:?}",
        initiate_obj.characteristics.subtypes
    );
    // Explicit dedup: there must be exactly ONE "Mutant" entry. Iterate the OrdSet
    // and count. Survives a hypothetical future migration off OrdSet (e.g. to Vec)
    // where idempotency would no longer be structural.
    let mutant_count = initiate_obj
        .characteristics
        .subtypes
        .iter()
        .filter(|st| **st == SubType("Mutant".to_string()))
        .count();
    assert_eq!(
        mutant_count, 1,
        "CR 614.5 / OrdSet idempotency: Mutant must appear exactly once even \
         though both the printed type set and the EntersAsAdditionalType \
         replacement contribute it. Got {} entries.",
        mutant_count
    );

    // PB-EWC counters still applied: 2 from MB's power + 1 from Simic Initiate's
    // own Graft 1 self-ETB replacement (CR 702.58 — "this creature enters with
    // a +1/+1 counter on it"). The two replacement effects compose independently.
    let counter_count = initiate_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 3,
        "PB-EWC must still apply for printed-Mutants: 2 from Master Biomancer's \
         power + 1 from Simic Initiate's Graft 1. Got {}.",
        counter_count
    );
}
