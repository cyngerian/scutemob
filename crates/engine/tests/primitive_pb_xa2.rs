//! PB-XA2: TargetFilter.is_blocking + is_tapped + is_untapped — runtime enforcement at
//! validate sites and trigger auto-target picker.
//!
//! Tests verify the three new bool fields on `TargetFilter` added in PB-XA2:
//! - `is_blocking: bool` — gated via `CombatState::is_blocking(id)` (CR 509.1c)
//! - `is_tapped: bool` — gated via `GameObject.status.tapped` (CR 701.20a)
//! - `is_untapped: bool` — gated via `!GameObject.status.tapped` (CR 701.21a)
//!
//! The OR-semantics for "attacking or blocking creature" (Eiganjo Channel half) are
//! implemented via a `passes_combat_role` four-way match on (is_attacking, is_blocking).
//! When both are `true`, either role is accepted (the (T,T) branch).
//!
//! CR Rules covered:
//! - CR 508.1k: "attacking creature" = creature in `CombatState.attackers`.
//! - CR 509.1 / 509.1c: "blocking creature" = creature in `CombatState.blockers`.
//! - CR 601.2c: At cast/activation time each target must satisfy the TargetRequirement.
//! - CR 603.3d: A triggered ability with no legal targets is not put on the stack.
//! - CR 701.20a: "tap" — affects `GameObject.status.tapped`.
//! - CR 701.21a: "untap" — mutually exclusive with tapped state.
//!
//! HASH invariant: PB-XA2 adds three new fields to `TargetFilter`, each hashed
//! in `state/hash.rs`. `HASH_SCHEMA_VERSION` bumped from 21 to 22.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Cost, Effect, EffectAmount, EffectTarget, TargetFilter,
    TargetRequirement, TriggerCondition, TypeLine,
};
use mtg_engine::rules::{process_command, Command};
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::PlayerTarget;
use mtg_engine::{
    enrich_spec_from_def, AttackTarget, CardId, CardRegistry, CardType, CombatState, GameState,
    GameStateBuilder, ObjectId, ObjectSpec, PlayerId, StackObjectKind, Step, Target, ZoneId,
    HASH_SCHEMA_VERSION,
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

/// Single-def CardRegistry + defs map.
fn single_def(def: CardDefinition) -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((def.name.clone(), def.clone())).collect();
    let registry = CardRegistry::new(vec![def]);
    (defs, registry)
}

/// Build a fresh CombatState with a single attacker mapped to p(2).
fn combat_with_attacker(attacking_player: PlayerId, attacker_id: ObjectId) -> CombatState {
    CombatState {
        attacking_player,
        attackers: [(attacker_id, AttackTarget::Player(p(2)))]
            .into_iter()
            .collect(),
        blockers: im::OrdMap::new(),
        damage_assignment_order: im::OrdMap::new(),
        first_strike_participants: im::OrdSet::new(),
        defenders_declared: im::OrdSet::new(),
        forced_blocks: im::OrdMap::new(),
        enlist_pairings: Vec::new(),
        blocked_attackers: im::OrdSet::new(),
    }
}

/// Build a fresh CombatState with one attacker and one blocker declared against it.
/// The blocker's ObjectId keys into `CombatState.blockers` — `is_blocking(blocker_id)` is `true`.
fn combat_with_blocker(blocker_id: ObjectId, attacker_id: ObjectId) -> CombatState {
    CombatState {
        attacking_player: p(1),
        attackers: [(attacker_id, AttackTarget::Player(p(2)))]
            .into_iter()
            .collect(),
        blockers: [(blocker_id, attacker_id)].into_iter().collect(),
        damage_assignment_order: im::OrdMap::new(),
        first_strike_participants: im::OrdSet::new(),
        defenders_declared: im::OrdSet::new(),
        forced_blocks: im::OrdMap::new(),
        enlist_pairings: Vec::new(),
        blocked_attackers: im::OrdSet::new(),
    }
}

// ── A: Hash schema sentinel ───────────────────────────────────────────────────

/// HASH_SCHEMA_VERSION live sentinel — fails if the schema version drifts
/// without this test being updated. See the `state/hash.rs` history block.
#[test]
fn test_pb_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 27u8,
        "BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (GameEvent::CreatureDied.pre_death_characteristics: Option<Characteristics>, CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers). If you bumped again, update this test and state/hash.rs history."
    );
}

/// PB-XA2 A-2: Pre-PB-XA2 JSON snapshot missing the three new fields deserializes
/// with all three defaulting to `false` (backward compatible via `#[serde(default)]`).
/// Mirrors the pre-PB-XA2 snapshot format: includes all required non-default fields
/// of TargetFilter but omits is_blocking, is_tapped, and is_untapped.
#[test]
fn test_pbxa2_serde_default_deserialize_pre_xa2_snapshot() {
    // A TargetFilter JSON without is_blocking, is_tapped, or is_untapped fields.
    // Mirrors the pre-PB-XA2 snapshot shape: only includes fields that existed
    // before PB-XA2. Note: TargetFilter has several fields without #[serde(default)]
    // so we must supply them all to form a valid snapshot (they are required serde fields
    // even though they are logically optional in the filter). is_blocking, is_tapped, and
    // is_untapped are absent — this tests backward compatibility.
    let json = r#"{
        "max_power": null,
        "min_power": null,
        "has_card_type": "Creature",
        "has_keywords": [],
        "colors": null,
        "exclude_colors": null,
        "non_creature": false,
        "non_land": false,
        "basic": false,
        "controller": "Any",
        "has_subtype": null,
        "is_attacking": true
    }"#;
    let filter: TargetFilter =
        serde_json::from_str(json).expect("pre-PB-XA2 snapshot must deserialize");
    assert!(
        filter.is_attacking,
        "is_attacking must round-trip from JSON"
    );
    assert!(
        !filter.is_blocking,
        "is_blocking must default to false when absent from JSON snapshot"
    );
    assert!(
        !filter.is_tapped,
        "is_tapped must default to false when absent from JSON snapshot"
    );
    assert!(
        !filter.is_untapped,
        "is_untapped must default to false when absent from JSON snapshot"
    );
}

// ── B: TargetFilter equality discriminators ────────────────────────────────────

/// PB-XA2 B-1: Filters differing only in `is_blocking` are not equal.
#[test]
fn test_pbxa2_filter_equality_distinguishes_is_blocking() {
    let f_default = TargetFilter::default();
    let f_blocking = TargetFilter {
        is_blocking: true,
        ..Default::default()
    };
    assert_ne!(
        f_default, f_blocking,
        "is_blocking difference must be observable via PartialEq"
    );
}

/// PB-XA2 B-2: Filters differing only in `is_tapped` are not equal.
#[test]
fn test_pbxa2_filter_equality_distinguishes_is_tapped() {
    let f_default = TargetFilter::default();
    let f_tapped = TargetFilter {
        is_tapped: true,
        ..Default::default()
    };
    assert_ne!(
        f_default, f_tapped,
        "is_tapped difference must be observable via PartialEq"
    );
}

/// PB-XA2 B-3: Filters differing only in `is_untapped` are not equal.
#[test]
fn test_pbxa2_filter_equality_distinguishes_is_untapped() {
    let f_default = TargetFilter::default();
    let f_untapped = TargetFilter {
        is_untapped: true,
        ..Default::default()
    };
    assert_ne!(
        f_default, f_untapped,
        "is_untapped difference must be observable via PartialEq"
    );
}

// ── Shared test helpers ────────────────────────────────────────────────────────

/// A source creature with an activated ability that targets a blocking creature.
/// Exercises V1 (TargetCreatureWithFilter with is_blocking=true).
fn blocking_creature_target_ability(name: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-pbxa2-blocking-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            timing_restriction: None,
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                is_blocking: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

/// A source creature with an activated ability that targets a tapped creature.
fn tapped_creature_target_ability(name: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-pbxa2-tapped-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            timing_restriction: None,
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                is_tapped: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

/// A source creature with an activated ability that targets an untapped creature.
fn untapped_creature_target_ability(name: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-pbxa2-untapped-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        mana_cost: Some(ManaCost {
            green: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            timing_restriction: None,
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                is_untapped: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

// ── C: is_blocking validate — positive and negative ───────────────────────────

/// PB-XA2 C-1: TargetCreatureWithFilter{is_blocking=true} REJECTS a creature that
/// is NOT in `combat.blockers`. Exercises V1 negative path.
///
/// CR 509.1c: a creature is "blocking" only if its ObjectId keys into
/// `CombatState.blockers`. A creature on the battlefield outside of blockers
/// does not satisfy this filter.
#[test]
fn test_pbxa2_activated_target_is_blocking_non_blocker_rejected() {
    let source_def = blocking_creature_target_ability("Shield Caller");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Shield Caller")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let non_blocker = ObjectSpec::creature(p(2), "Idle Wall", 0, 4).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(non_blocker)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Shield Caller");
    let non_blocker_id = find_obj(&state, "Idle Wall");

    // No combat state — Idle Wall is not blocking.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(non_blocker_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "PB-XA2 C-1: is_blocking=true must reject creature not in combat.blockers"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

/// PB-XA2 C-2: TargetCreatureWithFilter{is_blocking=true} ACCEPTS a creature
/// whose ObjectId keys into `combat.blockers`. Exercises V1 positive path.
///
/// CR 509.1c: the live `blockers` map keys on blocker ObjectId.
#[test]
fn test_pbxa2_activated_target_is_blocking_blocker_accepted() {
    let source_def = blocking_creature_target_ability("Shield Caller");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Shield Caller")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let attacker = ObjectSpec::creature(p(1), "Charging Bear", 2, 2).in_zone(ZoneId::Battlefield);
    let blocker = ObjectSpec::creature(p(2), "Steadfast Guard", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(attacker)
        .object(blocker)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Shield Caller");
    let attacker_id = find_obj(&state, "Charging Bear");
    let blocker_id = find_obj(&state, "Steadfast Guard");

    // Inject combat: Steadfast Guard is blocking Charging Bear.
    let mut state = state;
    state.combat = Some(combat_with_blocker(blocker_id, attacker_id));

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(blocker_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "PB-XA2 C-2: is_blocking=true must accept a creature in combat.blockers: {:?}",
        result.err()
    );
}

// ── D: is_tapped validate — positive and negative ─────────────────────────────

/// PB-XA2 D-1: TargetCreatureWithFilter{is_tapped=true} REJECTS a creature whose
/// `status.tapped == false`. Exercises V1 negative path.
///
/// CR 701.20a: "tap" = turn the permanent sideways. `status.tapped` tracks this.
#[test]
fn test_pbxa2_activated_target_is_tapped_untapped_rejected() {
    let source_def = tapped_creature_target_ability("Tap Seeker");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Tap Seeker")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    // An untapped creature — status.tapped defaults to false.
    let target = ObjectSpec::creature(p(2), "Upright Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(target)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Tap Seeker");
    let target_id = find_obj(&state, "Upright Bear");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(target_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "PB-XA2 D-1: is_tapped=true must reject an untapped creature (status.tapped == false)"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

/// PB-XA2 D-2: TargetCreatureWithFilter{is_tapped=true} ACCEPTS a creature
/// whose `status.tapped == true`. Exercises V1 positive path.
///
/// CR 701.20a: creature is tapped when `status.tapped == true`.
#[test]
fn test_pbxa2_activated_target_is_tapped_tapped_accepted() {
    let source_def = tapped_creature_target_ability("Tap Seeker");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Tap Seeker")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let tapped_target =
        ObjectSpec::creature(p(2), "Tapped Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(tapped_target)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Tap Seeker");
    let tapped_id = find_obj(&state, "Tapped Bear");

    // Manually set the target to tapped state.
    let mut state = state;
    state
        .objects
        .get_mut(&tapped_id)
        .expect("Tapped Bear must exist")
        .status
        .tapped = true;

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(tapped_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "PB-XA2 D-2: is_tapped=true must accept a creature with status.tapped == true: {:?}",
        result.err()
    );
}

// ── E: is_untapped validate — positive and negative ───────────────────────────

/// PB-XA2 E-1: TargetCreatureWithFilter{is_untapped=true} REJECTS a creature
/// whose `status.tapped == true`. Exercises V1 negative path.
///
/// CR 701.21a: "untap" = rotate back to upright. The predicate is `!status.tapped`.
#[test]
fn test_pbxa2_activated_target_is_untapped_tapped_rejected() {
    let source_def = untapped_creature_target_ability("Untap Seeker");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Untap Seeker")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let tapped_target =
        ObjectSpec::creature(p(2), "Sideways Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(tapped_target)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Untap Seeker");
    let tapped_id = find_obj(&state, "Sideways Bear");

    let mut state = state;
    state
        .objects
        .get_mut(&tapped_id)
        .expect("Sideways Bear must exist")
        .status
        .tapped = true;

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(tapped_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "PB-XA2 E-1: is_untapped=true must reject a tapped creature (status.tapped == true)"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

/// PB-XA2 E-2: TargetCreatureWithFilter{is_untapped=true} ACCEPTS a creature
/// whose `status.tapped == false` (default). Exercises V1 positive path.
///
/// CR 701.21a: an untapped creature has `status.tapped == false`.
#[test]
fn test_pbxa2_activated_target_is_untapped_untapped_accepted() {
    let source_def = untapped_creature_target_ability("Untap Seeker");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Untap Seeker")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    // An untapped creature — status.tapped defaults to false.
    let target = ObjectSpec::creature(p(2), "Ready Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(target)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Untap Seeker");
    let target_id = find_obj(&state, "Ready Bear");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(target_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "PB-XA2 E-2: is_untapped=true must accept a creature with status.tapped == false: {:?}",
        result.err()
    );
}

// ── F: OR-semantics "attacking or blocking" (Eiganjo Channel shape) ───────────

/// Source creature with activated ability: {T}: 4 damage to target attacking or blocking
/// creature. This is the Eiganjo Channel shape — (is_attacking=true, is_blocking=true).
fn attacking_or_blocking_target_ability(name: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-pbxa2-aob-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        mana_cost: Some(ManaCost {
            white: 1,
            generic: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(3),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::DealDamage {
                target: EffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(4),
            },
            timing_restriction: None,
            // PB-XA2: "attacking OR blocking" — (T,T) branch of passes_combat_role.
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                is_attacking: true,
                is_blocking: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

/// PB-XA2 F-1: Filter with is_attacking=true AND is_blocking=true ACCEPTS an
/// attacking creature (in combat.attackers, NOT in blockers).
///
/// OR-semantics: the (T,T) branch of passes_combat_role accepts either role.
/// This exercises the attacker-arm of the Eiganjo Channel half.
#[test]
fn test_pbxa2_activated_target_attacking_or_blocking_accepts_attacker() {
    let source_def = attacking_or_blocking_target_ability("Sentinel");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Sentinel")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let attacker = ObjectSpec::creature(p(2), "Charging Bear", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(attacker)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Sentinel");
    let attacker_id = find_obj(&state, "Charging Bear");

    // Only attackers — no blockers. The (T,T) branch should accept via the attacker arm.
    let mut state = state;
    state.combat = Some(combat_with_attacker(p(1), attacker_id));

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(attacker_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "PB-XA2 F-1: (is_attacking=T, is_blocking=T) must accept an attacking creature \
         via OR-semantics — attacker-arm of passes_combat_role (T,T): {:?}",
        result.err()
    );
}

/// PB-XA2 F-2: Filter with is_attacking=true AND is_blocking=true ACCEPTS a
/// blocking creature (in combat.blockers, NOT in attackers).
///
/// OR-semantics: the (T,T) branch of passes_combat_role accepts either role.
/// This exercises the blocker-arm of the Eiganjo Channel half.
#[test]
fn test_pbxa2_activated_target_attacking_or_blocking_accepts_blocker() {
    let source_def = attacking_or_blocking_target_ability("Sentinel");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Sentinel")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let attacker = ObjectSpec::creature(p(1), "Charging Bear", 3, 3).in_zone(ZoneId::Battlefield);
    let blocker = ObjectSpec::creature(p(2), "Stone Wall", 0, 6).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(attacker)
        .object(blocker)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Sentinel");
    let attacker_id = find_obj(&state, "Charging Bear");
    let blocker_id = find_obj(&state, "Stone Wall");

    // Stone Wall is blocking Charging Bear — in blockers but NOT in attackers.
    let mut state = state;
    state.combat = Some(combat_with_blocker(blocker_id, attacker_id));

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(blocker_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "PB-XA2 F-2: (is_attacking=T, is_blocking=T) must accept a blocking creature \
         via OR-semantics — blocker-arm of passes_combat_role (T,T): {:?}",
        result.err()
    );
}

/// PB-XA2 F-3: Filter with is_attacking=true AND is_blocking=true REJECTS a
/// creature that is neither attacking nor blocking.
///
/// OR-semantics: the (T,T) branch requires membership in EITHER role. A creature
/// not in either role fails passes_combat_role and is rejected.
#[test]
fn test_pbxa2_activated_target_attacking_or_blocking_rejects_non_combatant() {
    let source_def = attacking_or_blocking_target_ability("Sentinel");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Sentinel")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let attacker = ObjectSpec::creature(p(1), "Charging Bear", 3, 3).in_zone(ZoneId::Battlefield);
    let non_combatant =
        ObjectSpec::creature(p(2), "Peaceful Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(attacker)
        .object(non_combatant)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Sentinel");
    let attacker_id = find_obj(&state, "Charging Bear");
    let non_combatant_id = find_obj(&state, "Peaceful Bear");

    // Charging Bear is attacking, but Peaceful Bear is not in combat at all.
    let mut state = state;
    state.combat = Some(combat_with_attacker(p(1), attacker_id));

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(non_combatant_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "PB-XA2 F-3: (is_attacking=T, is_blocking=T) must reject a creature not in \
         either combat.attackers or combat.blockers"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

// ── G: Trigger auto-target picker — is_blocking discriminator ─────────────────

/// PB-XA2 G-1: Trigger auto-target picker with TargetPermanentWithFilter
/// {is_blocking=true} picks the blocker over the non-blocker.
///
/// ObjectId-ordering discriminator (per H-XA-01 lesson from PB-XA review):
/// Sitter (non-blocker) is added FIRST → smaller ObjectId, seen first in
/// BTreeMap iteration. Without PB-XA2 enforcement (`passes_combat_role`
/// removed at T4), the picker would short-circuit on Sitter and the assertion
/// `target_id == defender_id` FAILS. With PB-XA2 enabled, Sitter fails the
/// is_blocking gate; the picker advances to Defender and returns it — PASSES.
///
/// Mental-toggle check verified during implementation: with passes_combat_role
/// toggled off at T4, test fails with "Got id ObjectId(<sitter_id>)".
///
/// Exercises T4 (top-level TargetPermanentWithFilter in abilities.rs).
///
/// CR 509.1c: blocker membership is `CombatState.blockers.contains_key`.
/// CR 603.3d: trigger with no legal target is not put on the stack.
#[test]
fn test_pbxa2_trigger_picker_selects_blocking_creature_positive() {
    // WhenDies trigger: "when this creature dies, put target blocking permanent on
    // top of library" (contrived — exercises T4 TargetPermanentWithFilter is_blocking).
    let trigger_source_def = CardDefinition {
        name: "Blocking Scout".to_string(),
        card_id: CardId("test-pbxa2-blocking-scout".to_string()),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDies,
            effect: Effect::CreateTokenCopy {
                source: EffectTarget::DeclaredTarget { index: 0 },
                enters_tapped_and_attacking: false,
                except_not_legendary: false,
                gains_haste: false,
                delayed_action: None,
            },
            // T4 path: TargetPermanentWithFilter{has_card_type=Creature, is_blocking=true}.
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_type: Some(CardType::Creature),
                is_blocking: true,
                ..Default::default()
            })],
            intervening_if: None,
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let (defs2, registry2) = {
        let def = trigger_source_def.clone();
        let defs: HashMap<String, CardDefinition> =
            std::iter::once((def.name.clone(), def.clone())).collect();
        let registry = CardRegistry::new(vec![def]);
        (defs, registry)
    };

    // Blocking Scout: dying (toughness 1, damage 1).
    let dying_scout = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Blocking Scout")
            .with_card_id(trigger_source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield)
            .with_damage(1),
        &defs2,
    );

    // ObjectId-ordering discriminator: Sitter is added BEFORE Defender so Sitter
    // gets the smaller ObjectId and is visited first by the auto-target picker's
    // BTreeMap (ascending ObjectId) iteration. Without `passes_combat_role`
    // enforcement at T4, the picker would find Sitter legal (creature, no is_blocking
    // check) and return it — the assertion `target_id == defender_id` FAILS.
    // With enforcement, Sitter fails the is_blocking gate; the picker advances to
    // Defender and returns it — the assertion PASSES.
    let sitter = ObjectSpec::creature(p(2), "Sitter", 2, 2).in_zone(ZoneId::Battlefield);
    let attacker = ObjectSpec::creature(p(1), "Attacker", 2, 2).in_zone(ZoneId::Battlefield);
    let defender = ObjectSpec::creature(p(2), "Defender", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry2)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(dying_scout)
        .object(sitter) // Sitter first → smaller ObjectId.
        .object(attacker)
        .object(defender) // Defender last → larger ObjectId.
        .build()
        .expect("builder must succeed");
    state.turn.priority_holder = Some(p(1));

    let sitter_id = find_obj(&state, "Sitter");
    let attacker_id = find_obj(&state, "Attacker");
    let defender_id = find_obj(&state, "Defender");

    // Sanity guard: ObjectId ordering must hold for the discriminator to be valid.
    // If the builder ever changes insertion order, this assertion catches the
    // re-tautologisation before a false-green test can slip through review.
    assert!(
        sitter_id < defender_id,
        "G-1 discriminator invariant: Sitter (added first) must have a smaller ObjectId \
         than Defender (added second). Got sitter={:?} defender={:?}. \
         If the builder's ObjectId-assignment order changed, adjust the add order above.",
        sitter_id,
        defender_id
    );

    // Inject combat: Defender is blocking Attacker. Sitter is on the battlefield
    // but is NOT in combat.blockers.
    state.combat = Some(combat_with_blocker(defender_id, attacker_id));

    // Both players pass priority → SBAs fire → Blocking Scout dies → WhenDies
    // trigger queues → auto-target picker runs on TargetPermanentWithFilter
    // {has_card_type=Creature, is_blocking=true} → must pick Defender.
    let (state, events) = {
        let mut all = Vec::new();
        let mut s = state;
        for pl in [p(1), p(2)] {
            let (ns, ev) = process_command(s, Command::PassPriority { player: pl })
                .expect("pass priority must succeed");
            all.extend(ev);
            s = ns;
        }
        (s, all)
    };

    // Sanity: CreatureDied fired (so trigger had a chance to queue).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, mtg_engine::GameEvent::CreatureDied { .. })),
        "G-1: CreatureDied must have fired"
    );

    // The WhenDies trigger must be on the stack with target = Defender.
    let trigger_so = state
        .stack_objects
        .iter()
        .find(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }))
        .expect("G-1: WhenDies TriggeredAbility must be on the stack");

    assert_eq!(
        trigger_so.targets.len(),
        1,
        "G-1: trigger must have exactly 1 target"
    );

    let target_id = match trigger_so.targets[0].target {
        Target::Object(id) => id,
        Target::Player(_) => panic!("G-1: target must be an object, got player"),
    };

    assert_eq!(
        target_id, defender_id,
        "PB-XA2 G-1 / CR 509.1c: auto-target picker must select Defender (the blocker, id {:?}), \
         not Sitter (non-blocking, id {:?}). Got id {:?}.",
        defender_id, sitter_id, target_id
    );
    assert_ne!(
        target_id, sitter_id,
        "PB-XA2 G-1: auto-target picker must NOT select non-blocking Sitter"
    );
}

/// PB-XA2 G-2: Trigger auto-target picker with TargetPermanentWithFilter
/// {is_blocking=true} is SKIPPED when no creature is in `combat.blockers`.
///
/// Per CR 603.3d, a triggered ability that requires targets but has no legal
/// targets is not put on the stack at all.
#[test]
fn test_pbxa2_trigger_picker_skipped_when_no_blocker() {
    let trigger_source_def = CardDefinition {
        name: "Blocking Scout Solo".to_string(),
        card_id: CardId("test-pbxa2-blocking-scout-solo".to_string()),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDies,
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_type: Some(CardType::Creature),
                is_blocking: true,
                ..Default::default()
            })],
            intervening_if: None,
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let (defs2, registry2) = {
        let def = trigger_source_def.clone();
        let defs: HashMap<String, CardDefinition> =
            std::iter::once((def.name.clone(), def.clone())).collect();
        let registry = CardRegistry::new(vec![def]);
        (defs, registry)
    };

    // Dying scout.
    let dying_scout = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Blocking Scout Solo")
            .with_card_id(trigger_source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield)
            .with_damage(1),
        &defs2,
    );
    // A battlefield creature NOT in blockers.
    let sitter = ObjectSpec::creature(p(2), "Non-Blocker", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry2)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(dying_scout)
        .object(sitter)
        .build()
        .expect("builder must succeed");
    state.turn.priority_holder = Some(p(1));
    // No combat state — no blockers at all.

    let (state, events) = {
        let mut all = Vec::new();
        let mut s = state;
        for pl in [p(1), p(2)] {
            let (ns, ev) = process_command(s, Command::PassPriority { player: pl })
                .expect("pass priority must succeed");
            all.extend(ev);
            s = ns;
        }
        (s, all)
    };

    assert!(
        events
            .iter()
            .any(|e| matches!(e, mtg_engine::GameEvent::CreatureDied { .. })),
        "G-2: CreatureDied must have fired"
    );

    // Per CR 603.3d: trigger with no legal target is not put on the stack.
    let no_trigger = state
        .stack_objects
        .iter()
        .all(|so| !matches!(so.kind, StackObjectKind::TriggeredAbility { .. }));
    assert!(
        no_trigger,
        "PB-XA2 G-2 / CR 603.3d: WhenDies trigger with is_blocking=true and no blockers \
         must be SKIPPED (no stack object created). Stack: {:?}",
        state
            .stack_objects
            .iter()
            .map(|so| format!("{:?}", so.kind))
            .collect::<Vec<_>>()
    );
}

// ── H: Graveyard arm — runtime-field behavior for graveyard objects ────────────

/// PB-XA2 H-1: Graveyard arms with is_blocking=true always reject (graveyard
/// objects are never in combat.blockers). is_tapped=true always rejects
/// (graveyard status.tapped defaults to false → !o.status.tapped is true
/// meaning the "must be tapped" predicate `o.status.tapped` is false).
/// is_untapped=true always accepts graveyard cards that match other filters
/// (status.tapped defaults false, so !status.tapped is true → passes_untapped).
///
/// This is a degenerate edge case. No card legitimately uses is_tapped/is_untapped
/// on graveyard filters (CR 110.5 — tapped is a battlefield-only concept). The
/// test locks in the observed behavior so future changes are intentional.
///
/// Exercises V3 (TargetCardInYourGraveyard).
#[test]
fn test_pbxa2_graveyard_target_with_runtime_fields_rejects() {
    // Shared source: battlefield creature with activated abilities.
    fn make_gy_source(name: &str, card_id_str: &str, filter: TargetFilter) -> CardDefinition {
        CardDefinition {
            name: name.to_string(),
            card_id: CardId(card_id_str.to_string()),
            mana_cost: Some(ManaCost {
                black: 1,
                ..ManaCost::default()
            }),
            types: TypeLine {
                card_types: im::ordset![CardType::Creature],
                ..Default::default()
            },
            power: Some(1),
            toughness: Some(1),
            abilities: vec![AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(filter)],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            }],
            ..Default::default()
        }
    }

    // Case 1: is_blocking=true on graveyard target — always rejects.
    {
        let source_def = make_gy_source(
            "Grave Blocker",
            "test-pbxa2-grave-blocker",
            TargetFilter {
                has_card_type: Some(CardType::Creature),
                is_blocking: true,
                ..Default::default()
            },
        );
        let (defs, registry) = single_def(source_def.clone());

        let source = enrich_spec_from_def(
            ObjectSpec::card(p(1), "Grave Blocker")
                .with_card_id(source_def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs,
        );
        let gy_card = ObjectSpec::card(p(1), "Graveyard Creature")
            .with_types(vec![CardType::Creature])
            .in_zone(ZoneId::Graveyard(p(1)));

        let state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry)
            .object(source)
            .object(gy_card)
            .build()
            .expect("builder must succeed");

        let source_id = find_obj(&state, "Grave Blocker");
        let gy_id = find_obj(&state, "Graveyard Creature");

        let result = process_command(
            state,
            Command::ActivateAbility {
                player: p(1),
                source: source_id,
                ability_index: 0,
                targets: vec![Target::Object(gy_id)],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        );

        assert!(
            result.is_err(),
            "H-1a: is_blocking=true on graveyard filter must always reject \
             (graveyard objects are never in combat.blockers)"
        );
    }

    // Case 2: is_tapped=true on graveyard target — always rejects.
    // Rationale: graveyard objects have status.tapped = false (default), so
    // `o.status.tapped` is false → passes_tapped = false → rejected.
    {
        let source_def = make_gy_source(
            "Grave Tapper",
            "test-pbxa2-grave-tapper",
            TargetFilter {
                has_card_type: Some(CardType::Creature),
                is_tapped: true,
                ..Default::default()
            },
        );
        let (defs, registry) = single_def(source_def.clone());

        let source = enrich_spec_from_def(
            ObjectSpec::card(p(1), "Grave Tapper")
                .with_card_id(source_def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs,
        );
        let gy_card2 = ObjectSpec::card(p(1), "Graveyard Creature 2")
            .with_types(vec![CardType::Creature])
            .in_zone(ZoneId::Graveyard(p(1)));

        let state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry)
            .object(source)
            .object(gy_card2)
            .build()
            .expect("builder must succeed");

        let source_id = find_obj(&state, "Grave Tapper");
        let gy_id = find_obj(&state, "Graveyard Creature 2");

        let result = process_command(
            state,
            Command::ActivateAbility {
                player: p(1),
                source: source_id,
                ability_index: 0,
                targets: vec![Target::Object(gy_id)],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        );

        assert!(
            result.is_err(),
            "H-1b: is_tapped=true on graveyard filter must always reject \
             (graveyard status.tapped defaults false; the 'must be tapped' predicate rejects)"
        );
    }

    // Case 3: is_untapped=true on graveyard target — accepts matching graveyard cards.
    // This is the degenerate edge case: graveyard status.tapped defaults false,
    // so !status.tapped is true → passes_untapped = true. The card passes via the
    // is_untapped filter (even though tapped/untapped is meaningless for graveyard).
    // Locked in here to make future behavioral changes intentional.
    {
        let source_def = make_gy_source(
            "Grave Untapper",
            "test-pbxa2-grave-untapper",
            TargetFilter {
                has_card_type: Some(CardType::Creature),
                is_untapped: true,
                ..Default::default()
            },
        );
        let (defs, registry) = single_def(source_def.clone());

        let source = enrich_spec_from_def(
            ObjectSpec::card(p(1), "Grave Untapper")
                .with_card_id(source_def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs,
        );
        let gy_card3 = ObjectSpec::card(p(1), "Graveyard Creature 3")
            .with_types(vec![CardType::Creature])
            .in_zone(ZoneId::Graveyard(p(1)));

        let state = GameStateBuilder::new()
            .add_player(p(1))
            .add_player(p(2))
            .with_registry(registry)
            .object(source)
            .object(gy_card3)
            .build()
            .expect("builder must succeed");

        let source_id = find_obj(&state, "Grave Untapper");
        let gy_id = find_obj(&state, "Graveyard Creature 3");

        let result = process_command(
            state,
            Command::ActivateAbility {
                player: p(1),
                source: source_id,
                ability_index: 0,
                targets: vec![Target::Object(gy_id)],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        );

        // Design-quirk behavior locked in: is_untapped=true on graveyard filter
        // ACCEPTS matching cards (graveyard status.tapped defaults false).
        // This is intentional documentation of the degenerate case — no real card
        // legitimately uses is_untapped on a graveyard filter (CR 110.5).
        assert!(
            result.is_ok(),
            "H-1c: is_untapped=true on graveyard filter ACCEPTS matching cards \
             (degenerate-but-documented edge case: graveyard status.tapped defaults false, \
             so !status.tapped is true → passes_untapped). \
             If this changes, update the comment in V3/V4 in casting.rs. \
             Got: {:?}",
            result.err()
        );
    }
}
