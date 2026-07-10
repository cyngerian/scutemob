//! PB-XS-E: trigger-side `exclude_self` for `Whenever another creature/permanent enters`.
//!
//! Counterpart to PB-XS (target-side). Adds `exclude_self: bool` to:
//!   - `TriggerCondition::WheneverCreatureEntersBattlefield`
//!   - `TriggerCondition::WheneverPermanentEntersBattlefield`
//!
//! Previously the runtime conversion in `enrich_spec_from_def` hardcoded
//! `ETBTriggerFilter.exclude_self = true` for the Creature variant (latent bug
//! for "this or another X" cards) and `false` for the Permanent variant
//! (Landfall default). Now the per-card field threads through end-to-end:
//! CardDef → enrich_spec_from_def → ETBTriggerFilter → collect_triggers_for_event.
//!
//! CR rules covered:
//! - CR 109.1 — object identity (the entering object is distinct from the trigger source).
//! - CR 603.2 — triggered abilities fire when their trigger event occurs.
//! - CR 207.2c — ability-word triggers (Alliance, Landfall) using "you control" /
//!   "another" qualifiers.
//! - CR 400.7 — zone-change identity for the graveyard-side dispatch path.
//!
//! Schema/serde:
//! - HASH_SCHEMA_VERSION sentinel: 19 → 20.
//! - `#[serde(default)]` keeps pre-PB-XS-E serialized states round-tripping with
//!   `exclude_self = false`.

use std::collections::HashMap;

use mtg_engine::{
    enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry,
    CardType, Color, Command, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step, SubType,
    TargetController, TargetFilter, TriggerCondition, TypeLine, ZoneId, HASH_SCHEMA_VERSION,
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

fn life_total(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(&player)
        .map(|p| p.life_total)
        .unwrap_or_default()
}

/// Build a minimal creature CardDefinition with a given generic mana cost.
fn creature_def(card_id: &str, name: &str, generic: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}

/// Cast a creature from a player's hand by name, paying with colorless mana.
fn cast_creature(
    state: GameState,
    player: PlayerId,
    name: &str,
    mana_amount: u32,
) -> (GameState, Vec<GameEvent>) {
    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Hand(player))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("card '{}' not found in {}'s hand", name, player.0));

    let mut state = state;
    state.players.get_mut(&player).unwrap().mana_pool.colorless = mana_amount;
    state.turn.priority_holder = Some(player);

    process_command(
        state,
        Command::CastSpell {
            player,
            card: card_id,
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
    .expect("CastSpell failed")
}

/// Count triggers (pending + on stack) emitted from a specific source.
fn triggers_for(state: &GameState, source: ObjectId) -> usize {
    let pending = state
        .pending_triggers
        .iter()
        .filter(|t| t.source == source)
        .count();
    let on_stack = state
        .stack_objects
        .iter()
        .filter(|so| {
            matches!(
                so.kind,
                StackObjectKind::TriggeredAbility { source_object, .. }
                if source_object == source
            )
        })
        .count();
    pending + on_stack
}

// ── A: Hash schema sentinel ───────────────────────────────────────────────────

/// HASH_SCHEMA_VERSION live sentinel — fails if the schema version drifts
/// without this test being updated. See the `state/hash.rs` history block.
#[test]
fn test_pbxse_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 35u8,
        "BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (GameEvent::CreatureDied.pre_death_characteristics: Option<Characteristics>, CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers). If you bumped again, update this test and state/hash.rs history."
    );
}

// ── B: PartialEq / serde-default round-trip ───────────────────────────────────

/// PB-XS-E B-1: Two `WheneverCreatureEntersBattlefield` conditions differing only
/// in `exclude_self` are NOT equal — the field participates in `PartialEq`. This
/// is load-bearing for downstream comparisons (registry equality, replacement-effect
/// dedup) that rely on `Eq`.
#[test]
fn test_pbxse_creature_variant_partialeq_distinguishes_exclude_self() {
    let inclusive = TriggerCondition::WheneverCreatureEntersBattlefield {
        filter: None,
        exclude_self: false,
    };
    let exclusive = TriggerCondition::WheneverCreatureEntersBattlefield {
        filter: None,
        exclude_self: true,
    };
    assert_ne!(
        inclusive, exclusive,
        "PB-XS-E: WheneverCreatureEntersBattlefield.exclude_self must affect PartialEq"
    );
}

/// PB-XS-E B-2: Same property for `WheneverPermanentEntersBattlefield`.
#[test]
fn test_pbxse_permanent_variant_partialeq_distinguishes_exclude_self() {
    let inclusive = TriggerCondition::WheneverPermanentEntersBattlefield {
        filter: None,
        exclude_self: false,
    };
    let exclusive = TriggerCondition::WheneverPermanentEntersBattlefield {
        filter: None,
        exclude_self: true,
    };
    assert_ne!(
        inclusive, exclusive,
        "PB-XS-E: WheneverPermanentEntersBattlefield.exclude_self must affect PartialEq"
    );
}

/// PB-XS-E B-3: `#[serde(default)]` means pre-PB-XS-E serialized states
/// (without `exclude_self` in the JSON) deserialize as `exclude_self: false`.
/// This preserves backward compatibility for replays/snapshots predating PB-XS-E.
#[test]
fn test_pbxse_creature_variant_serde_default_for_exclude_self() {
    let json = r#"{
        "WheneverCreatureEntersBattlefield": {
            "filter": null
        }
    }"#;
    let parsed: TriggerCondition = serde_json::from_str(json)
        .expect("pre-PB-XS-E WheneverCreatureEntersBattlefield must deserialize");
    match parsed {
        TriggerCondition::WheneverCreatureEntersBattlefield {
            filter,
            exclude_self,
        } => {
            assert!(filter.is_none(), "PB-XS-E: filter must round-trip as None");
            assert!(
                !exclude_self,
                "PB-XS-E: missing exclude_self deserializes as false (#[serde(default)])"
            );
        }
        other => panic!(
            "Expected WheneverCreatureEntersBattlefield, got {:?}",
            other
        ),
    }
}

/// PB-XS-E B-4: Same serde-default round-trip for the Permanent variant.
#[test]
fn test_pbxse_permanent_variant_serde_default_for_exclude_self() {
    let json = r#"{
        "WheneverPermanentEntersBattlefield": {
            "filter": null
        }
    }"#;
    let parsed: TriggerCondition = serde_json::from_str(json)
        .expect("pre-PB-XS-E WheneverPermanentEntersBattlefield must deserialize");
    match parsed {
        TriggerCondition::WheneverPermanentEntersBattlefield {
            filter,
            exclude_self,
        } => {
            assert!(filter.is_none(), "PB-XS-E: filter must round-trip as None");
            assert!(
                !exclude_self,
                "PB-XS-E: missing exclude_self deserializes as false (#[serde(default)])"
            );
        }
        other => panic!(
            "Expected WheneverPermanentEntersBattlefield, got {:?}",
            other
        ),
    }
}

// Build a watcher CardDef that triggers "Whenever [another?] creature you control enters,
// you gain 1 life." `exclude_self` is parameterized so each test scenario picks its semantics.
fn build_watcher_def(card_id: &str, name: &str, exclude_self: bool) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: if exclude_self {
            "Whenever another creature you control enters, you gain 1 life.".to_string()
        } else {
            "Whenever a creature you control enters, you gain 1 life.".to_string()
        },
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                filter: Some(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                }),
                exclude_self,
            },
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

// ── C: Creature-ETB positive discriminator (source enters → trigger SKIPPED) ──

/// PB-XS-E C-1: `WheneverCreatureEntersBattlefield { exclude_self: true }` —
/// when the source creature itself enters the battlefield, the trigger MUST NOT
/// fire (the "another" qualifier). End-to-end CardDef → enrich_spec_from_def →
/// ETBTriggerFilter → collect_triggers_for_event dispatch.
#[test]
fn test_pbxse_creature_trigger_excludes_self_on_own_etb() {
    let watcher_def = build_watcher_def("pbxse-watcher", "Self-Excluding Watcher", true);
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((watcher_def.name.clone(), watcher_def.clone())).collect();
    let registry = CardRegistry::new(vec![watcher_def.clone()]);
    // Enrich the spec so the runtime triggered_abilities (built from the
    // CardDef's WheneverCreatureEntersBattlefield) are present when the card
    // moves Hand → Stack → Battlefield.
    let in_hand = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Self-Excluding Watcher")
            .with_card_id(watcher_def.card_id.clone())
            .in_zone(ZoneId::Hand(p(1))),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(in_hand)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_life = life_total(&state, p(1));

    let (state, _) = cast_creature(state, p(1), "Self-Excluding Watcher", 2);
    let (state, resolution_events) = pass_all(state, &[p(1), p(2)]);

    let fired = resolution_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { .. }));
    assert!(
        !fired,
        "PB-XS-E C-1: WheneverCreatureEntersBattlefield {{ exclude_self: true }} must NOT \
         fire when the source itself enters. Got events: {:?}",
        resolution_events
            .iter()
            .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        life_total(&state, p(1)),
        initial_life,
        "PB-XS-E C-1: no life gain when trigger does not fire"
    );
}

// ── D: Creature-ETB negative discriminator (another enters → trigger FIRES) ───

/// PB-XS-E D-1: `WheneverCreatureEntersBattlefield { exclude_self: true }` —
/// when a DIFFERENT creature enters, the trigger MUST fire. Confirms the
/// gate does not regress legitimate triggers.
#[test]
fn test_pbxse_creature_trigger_fires_when_another_creature_enters() {
    let watcher_def = build_watcher_def("pbxse-watcher-2", "Watcher", true);
    let ally_def = creature_def("pbxse-ally", "Ally Wolf", 2);
    let defs: HashMap<String, CardDefinition> = [
        (watcher_def.name.clone(), watcher_def.clone()),
        (ally_def.name.clone(), ally_def.clone()),
    ]
    .into_iter()
    .collect();
    let registry = CardRegistry::new(vec![watcher_def.clone(), ally_def.clone()]);

    let watcher_on_bf = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Watcher")
            .with_card_id(watcher_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let ally_in_hand = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Ally Wolf")
            .with_card_id(ally_def.card_id.clone())
            .in_zone(ZoneId::Hand(p(1))),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(watcher_on_bf)
        .object(ally_in_hand)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let watcher_id = find_object(&state, "Watcher");
    let initial_life = life_total(&state, p(1));

    let (state, _) = cast_creature(state, p(1), "Ally Wolf", 2);
    let (state, _resolution_events) = pass_all(state, &[p(1), p(2)]);

    let n = triggers_for(&state, watcher_id);
    assert_eq!(
        n, 1,
        "PB-XS-E D-1: exactly 1 trigger should fire from Watcher when Ally Wolf enters (got {})",
        n
    );

    let (state, _) = pass_all(state, &[p(1), p(2)]);
    assert_eq!(
        life_total(&state, p(1)),
        initial_life + 1,
        "PB-XS-E D-1: life gain must occur after the resolved trigger"
    );
}

// ── E: Creature-ETB inclusive default (exclude_self: false → source fires) ────

/// PB-XS-E E-1: `WheneverCreatureEntersBattlefield { exclude_self: false }` —
/// the source IS allowed to fire its own ETB trigger. This matches oracle text
/// of cards like Witty Roastmaster ("Whenever a creature enters under your
/// control") and Risen Reef ("Whenever this or another Elemental..."). Confirms
/// the default-false behavior — historically these cards latently failed to
/// fire on self because the runtime hardcoded `exclude_self: true`.
#[test]
fn test_pbxse_creature_trigger_inclusive_fires_on_self_etb() {
    let inclusive_def = build_watcher_def("pbxse-inclusive", "Inclusive Witness", false);
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((inclusive_def.name.clone(), inclusive_def.clone())).collect();
    let registry = CardRegistry::new(vec![inclusive_def.clone()]);
    let in_hand = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Inclusive Witness")
            .with_card_id(inclusive_def.card_id.clone())
            .in_zone(ZoneId::Hand(p(1))),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(in_hand)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_life = life_total(&state, p(1));

    let (state, _) = cast_creature(state, p(1), "Inclusive Witness", 2);
    let (state, _resolution_events) = pass_all(state, &[p(1), p(2)]);

    let witness_id = find_object(&state, "Inclusive Witness");
    let n = triggers_for(&state, witness_id);
    assert_eq!(
        n, 1,
        "PB-XS-E E-1: inclusive (exclude_self=false) trigger must fire on self ETB (got {})",
        n
    );

    let (state, _) = pass_all(state, &[p(1), p(2)]);
    assert_eq!(
        life_total(&state, p(1)),
        initial_life + 1,
        "PB-XS-E E-1: life gain confirms the self-firing trigger resolved"
    );
}

// ── F: Permanent-ETB positive (source enters → SKIPPED) ───────────────────────

/// Build a permanent-watcher CardDef: "Whenever [another?] red permanent you control enters,
/// you gain 1 life." Source is a red creature (so its own ETB matches the color filter).
fn build_permanent_watcher_def(card_id: &str, name: &str, exclude_self: bool) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: if exclude_self {
            "Whenever another red permanent you control enters, you gain 1 life.".to_string()
        } else {
            "Whenever a red permanent you control enters, you gain 1 life.".to_string()
        },
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                filter: Some(TargetFilter {
                    controller: TargetController::You,
                    colors: Some([Color::Red].iter().copied().collect()),
                    ..Default::default()
                }),
                exclude_self,
            },
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// PB-XS-E F-1: `WheneverPermanentEntersBattlefield { exclude_self: true }` —
/// the source is excluded from triggering itself when it enters. The Permanent
/// variant defaults to `false` for backward compatibility with Landfall, but
/// can opt into self-exclusion. We use a synthetic "Whenever any permanent enters"
/// trigger and verify that the source's own ETB does not fire it.
///
/// Color filter (red) makes the source matchable when it shares the color, but
/// `exclude_self: true` still must skip its own ETB.
#[test]
fn test_pbxse_permanent_trigger_excludes_self_on_own_etb() {
    let watcher_def = build_permanent_watcher_def("pbxse-pw-1", "Permanent Watcher", true);
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((watcher_def.name.clone(), watcher_def.clone())).collect();
    let registry = CardRegistry::new(vec![watcher_def.clone()]);
    let in_hand = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Permanent Watcher")
            .with_card_id(watcher_def.card_id.clone())
            .in_zone(ZoneId::Hand(p(1))),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(in_hand)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give P1 enough red mana.
    let mut state = state;
    state.players.get_mut(&p(1)).unwrap().mana_pool.red = 1;
    state.players.get_mut(&p(1)).unwrap().mana_pool.colorless = 1;
    state.turn.priority_holder = Some(p(1));

    // Find the card in hand.
    let card_id = state
        .objects
        .iter()
        .find(|(_, obj)| {
            obj.characteristics.name == "Permanent Watcher" && obj.zone == ZoneId::Hand(p(1))
        })
        .map(|(id, _)| *id)
        .unwrap();
    let initial_life = life_total(&state, p(1));

    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p(1),
            card: card_id,
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
    .expect("CastSpell failed");
    let (state, resolution_events) = pass_all(state, &[p(1), p(2)]);

    let fired = resolution_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { .. }));
    assert!(
        !fired,
        "PB-XS-E F-1: WheneverPermanentEntersBattlefield {{ exclude_self: true }} must NOT \
         fire on the source's own ETB. Got events: {:?}",
        resolution_events
            .iter()
            .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        life_total(&state, p(1)),
        initial_life,
        "PB-XS-E F-1: no life gain when self-trigger is suppressed"
    );
}

// ── G: Permanent-ETB negative (another enters → FIRES) ────────────────────────

/// PB-XS-E G-1: `WheneverPermanentEntersBattlefield { exclude_self: true }` —
/// another matching permanent entering MUST still fire the trigger.
#[test]
fn test_pbxse_permanent_trigger_fires_when_another_red_permanent_enters() {
    let watcher_def = build_permanent_watcher_def("pbxse-pw-2", "Watcher", true);
    let goblin_def = CardDefinition {
        card_id: CardId("pbxse-goblin".to_string()),
        name: "Red Goblin".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    };
    let defs: HashMap<String, CardDefinition> = [
        (watcher_def.name.clone(), watcher_def.clone()),
        (goblin_def.name.clone(), goblin_def.clone()),
    ]
    .into_iter()
    .collect();
    let registry = CardRegistry::new(vec![watcher_def.clone(), goblin_def.clone()]);

    let watcher_on_bf = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Watcher")
            .with_card_id(watcher_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let goblin_in_hand = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Red Goblin")
            .with_card_id(goblin_def.card_id.clone())
            .in_zone(ZoneId::Hand(p(1))),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(watcher_on_bf)
        .object(goblin_in_hand)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let mut state = state;
    state.players.get_mut(&p(1)).unwrap().mana_pool.red = 1;
    state.turn.priority_holder = Some(p(1));

    let goblin_card_id = state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == "Red Goblin" && obj.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .unwrap();

    let watcher_id = find_object(&state, "Watcher");
    let initial_life = life_total(&state, p(1));

    let (state, _cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p(1),
            card: goblin_card_id,
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
    .expect("CastSpell failed");
    let (state, _resolution_events) = pass_all(state, &[p(1), p(2)]);

    let n = triggers_for(&state, watcher_id);
    assert_eq!(
        n, 1,
        "PB-XS-E G-1: exactly 1 trigger should fire from Watcher when Red Goblin enters (got {})",
        n
    );

    let (state, _) = pass_all(state, &[p(1), p(2)]);
    assert_eq!(
        life_total(&state, p(1)),
        initial_life + 1,
        "PB-XS-E G-1: life gain confirms the trigger resolved"
    );
}

// ── H: Subtype-filter regression discriminator (Forerunner pattern) ───────────

/// PB-XS-E H-1: Subtype-filtered exclude_self — `WheneverCreatureEntersBattlefield`
/// with `has_subtype: Some(Vampire)` and `exclude_self: true` represents
/// Forerunner of the Legion: "Whenever another Vampire you control enters".
/// The source itself is a Vampire, so without the gate the source's own ETB
/// would match the filter — exclude_self must suppress it.
#[test]
fn test_pbxse_subtype_exclude_self_suppresses_source_match() {
    // Use a colorless cost so the test doesn't need to wire up white mana.
    let forerunner_def = CardDefinition {
        card_id: CardId("pbxse-forerunner".to_string()),
        name: "Vampire Forerunner".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].iter().cloned().collect(),
            subtypes: [SubType("Vampire".to_string())].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever another Vampire you control enters, you gain 1 life.".to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                filter: Some(TargetFilter {
                    controller: TargetController::You,
                    has_subtype: Some(SubType("Vampire".to_string())),
                    ..Default::default()
                }),
                exclude_self: true,
            },
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((forerunner_def.name.clone(), forerunner_def.clone())).collect();
    let registry = CardRegistry::new(vec![forerunner_def.clone()]);
    let in_hand = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Vampire Forerunner")
            .with_card_id(forerunner_def.card_id.clone())
            .in_zone(ZoneId::Hand(p(1))),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(in_hand)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let initial_life = life_total(&state, p(1));

    let (state, _) = cast_creature(state, p(1), "Vampire Forerunner", 3);
    let (state, resolution_events) = pass_all(state, &[p(1), p(2)]);

    let fired = resolution_events
        .iter()
        .any(|e| matches!(e, GameEvent::AbilityTriggered { .. }));
    assert!(
        !fired,
        "PB-XS-E H-1: subtype-filtered trigger with exclude_self=true must NOT fire on \
         the source Vampire's own ETB. Got: {:?}",
        resolution_events
            .iter()
            .filter(|e| matches!(e, GameEvent::AbilityTriggered { .. }))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        life_total(&state, p(1)),
        initial_life,
        "PB-XS-E H-1: no life gain when self-trigger is suppressed by subtype + exclude_self"
    );
}
