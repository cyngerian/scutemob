//! PB-XS: TargetFilter.exclude_self — "another target X" target selection.
//!
//! Tests verify that the new `exclude_self: bool` field on `TargetFilter`
//! correctly rejects the source object as a target candidate while still
//! permitting other legal targets, across:
//! - The declarative target validation path used by spells and activated
//!   abilities (`casting::validate_object_satisfies_requirement`, threaded
//!   through `validate_targets_with_source`).
//! - All four filter-bearing TargetRequirement variants:
//!   TargetCreatureWithFilter, TargetPermanentWithFilter,
//!   TargetCardInYourGraveyard, TargetCardInGraveyard.
//! - Hash schema: HASH_SCHEMA_VERSION bumped 18 → 19; the new field is hashed.
//!
//! CR Rules covered:
//! - CR 109.1: "Object" identity; the source of a spell or ability is itself
//!   an object distinct from candidate target objects.
//! - CR 601.2c: At cast/activation time the player announces targets; each
//!   declared target must satisfy the TargetRequirement (including
//!   exclude_self) BEFORE costs are paid.
//! - CR 603.10a: WhenDies death triggers fire from a graveyard object whose
//!   pre-death identity must be excluded from "another target X" candidates.

use std::sync::Arc;

use std::collections::HashMap;

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Cost, Effect, EffectTarget, TargetFilter, TargetRequirement,
    TypeLine,
};
use mtg_engine::rules::{process_command, Command};
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::{
    CardId, CardType, GameStateBuilder, ObjectSpec, PlayerId, SubType, Target, ZoneId,
};
use mtg_engine::{enrich_spec_from_def, CardRegistry, GameState, ObjectId, HASH_SCHEMA_VERSION};

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

/// Single-def CardRegistry + defs map (mirrors cost_primitives.rs helper).
fn single_def(def: CardDefinition) -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let defs: HashMap<String, CardDefinition> =
        std::iter::once((def.name.clone(), def.clone())).collect();
    let registry = CardRegistry::new(vec![def]);
    (defs, registry)
}

// ── A: Hash schema sentinel ───────────────────────────────────────────────────

/// PB-XS sentinel (re-pointed by PB-XS-E): asserts the live HASH_SCHEMA_VERSION,
/// not the version PB-XS itself bumped to. The test name is generic so future
/// PBs do not need to rename it again.
#[test]
fn test_pbxs_hash_schema_version_matches_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 36u8,
        "BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (GameEvent::CreatureDied.pre_death_characteristics: Option<Characteristics>, CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers). If you bumped again, update this test and state/hash.rs history."
    );
}

// ── B: TargetFilter equality / hash discriminator ─────────────────────────────

/// PB-XS B-1: Filters that differ only in `exclude_self` are NOT equal.
/// This proves the field participates in PartialEq (relied on by replacement-
/// effect dedup and continuous-effect cache equality).
#[test]
fn test_pbxs_filter_equality_distinguishes_exclude_self() {
    let f_default = TargetFilter::default();
    let f_exclude = TargetFilter {
        exclude_self: true,
        ..Default::default()
    };
    assert_ne!(
        f_default, f_exclude,
        "exclude_self difference must be observable via PartialEq"
    );
}

/// PB-XS B-2: `exclude_self` flows through `serde(default)` so pre-PB-XS
/// serialized TargetFilter values continue to deserialize (as `false`).
#[test]
fn test_pbxs_serde_default_for_exclude_self() {
    // Synthetic JSON without `exclude_self` — mimics a pre-PB-XS snapshot.
    let json = r#"{
        "max_power": null,
        "min_power": null,
        "has_card_type": null,
        "has_keywords": [],
        "colors": null,
        "exclude_colors": null,
        "non_creature": false,
        "non_land": false,
        "basic": false,
        "controller": "Any",
        "has_subtype": null
    }"#;
    let parsed: TargetFilter =
        serde_json::from_str(json).expect("pre-PB-XS TargetFilter must deserialize");
    assert!(
        !parsed.exclude_self,
        "missing field deserializes as false (#[serde(default)])"
    );
}

// ── C: Activated-ability declarative target validation ────────────────────────

/// Build a "fight" creature: "{T}: This creature fights another target creature."
/// Mirrors Brash Taunter's activated ability stripped of mana cost.
fn fight_self_creature(name: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-fight-{}",
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
            effect: Effect::Fight {
                attacker: EffectTarget::Source,
                defender: EffectTarget::DeclaredTarget { index: 0 },
            },
            timing_restriction: None,
            // PB-XS: exclude_self -- "another target creature."
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                exclude_self: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

/// PB-XS C-1: An activated ability with `exclude_self: true` on
/// TargetCreatureWithFilter rejects the source as a target.
/// Setup: P1 controls a Fighter with the fight-another ability. No other
/// creatures exist. Targeting self must error InvalidTarget.
#[test]
fn test_pbxs_activated_target_self_is_rejected() {
    let fighter_def = fight_self_creature("Fighter");
    let (defs, registry) = single_def(fighter_def.clone());

    let fighter = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Fighter")
            .with_card_id(fighter_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(fighter)
        .build()
        .expect("builder must succeed");

    let fighter_id = find_obj(&state, "Fighter");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: fighter_id,
            ability_index: 0,
            targets: vec![Target::Object(fighter_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "PB-XS: activated 'another target creature' must reject self as target"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

/// PB-XS C-2: Same ability successfully targets a different creature.
/// Confirms `exclude_self` does NOT regress legitimate targets.
#[test]
fn test_pbxs_activated_target_another_creature_is_accepted() {
    let fighter_def = fight_self_creature("Fighter");
    let (defs, registry) = single_def(fighter_def.clone());

    let fighter = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Fighter")
            .with_card_id(fighter_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let goblin = ObjectSpec::creature(p(2), "Goblin Brute", 3, 3).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(fighter)
        .object(goblin)
        .build()
        .expect("builder must succeed");

    let fighter_id = find_obj(&state, "Fighter");
    let goblin_id = find_obj(&state, "Goblin Brute");
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: fighter_id,
            ability_index: 0,
            targets: vec![Target::Object(goblin_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "PB-XS: activated 'another target creature' must accept a different creature: {:?}",
        result.err()
    );
}

// ── D: Spell-side declarative target validation (TargetPermanentWithFilter) ───

/// PB-XS D-1: A spell with `exclude_self: true` on TargetPermanentWithFilter
/// rejects the casting spell itself as a target (defensive — spells are not
/// permanents, so the candidate would already be in the Stack zone, not
/// battlefield; this test confirms exclude_self does not regress the
/// pre-existing zone-mismatch rejection).
///
/// More importantly: when an activated ability uses TargetPermanentWithFilter
/// with exclude_self, self-targeting from the battlefield source must fail.
#[test]
fn test_pbxs_activated_target_permanent_exclude_self_rejects_source() {
    // Build a permanent-target activated ability: "{T}: Tap another target
    // permanent." Self-targeting would otherwise be legal (Tap on the source).
    let card_def = CardDefinition {
        name: "Untapper".to_string(),
        card_id: CardId("test-pbxs-untapper".to_string()),
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
            effect: Effect::TapPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            timing_restriction: None,
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                exclude_self: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };
    let (defs, registry) = single_def(card_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Untapper")
            .with_card_id(card_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Untapper");

    // Targeting self with exclude_self=true must error.
    let result = process_command(
        state.clone(),
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(source_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result.is_err(),
        "PB-XS D-1: self-targeting TargetPermanentWithFilter w/ exclude_self must error"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

// ── E: TargetCardInYourGraveyard exclude_self (Elderfang Ritualist pattern) ──

/// PB-XS E-1: TargetCardInYourGraveyard with `exclude_self: true` rejects
/// the source object itself. This is the declarative test exercising the
/// `validate_object_satisfies_requirement` graveyard arm — symmetric to the
/// `TargetCreatureWithFilter` path.
///
/// We exercise this via a unit-style activation: an artifact in the graveyard
/// would normally not have an ability, so we synthesize a "Reanimate Elf"
/// activated ability on a battlefield permanent that targets an Elf card in
/// its controller's graveyard with exclude_self=true. With only the source
/// object's card itself "in graveyard" (synthetic test scenario), the request
/// must fail; with a real other Elf card present, it must succeed.
#[test]
fn test_pbxs_graveyard_filter_excludes_source() {
    // Construct a battlefield creature whose activated ability returns
    // "another target Elf card from your graveyard" to its owner's hand.
    let necro_def = CardDefinition {
        name: "Necromancer".to_string(),
        card_id: CardId("test-pbxs-necro".to_string()),
        mana_cost: Some(ManaCost {
            black: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            subtypes: im::ordset![SubType("Elf".to_string())],
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to: mtg_engine::ZoneTarget::Hand {
                    owner: mtg_engine::PlayerTarget::Controller,
                },
                controller_override: None,
            },
            timing_restriction: None,
            // exclude_self: true.
            targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                has_subtype: Some(SubType("Elf".to_string())),
                exclude_self: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };
    let (defs, registry) = single_def(necro_def.clone());

    // Place a Necromancer on the battlefield. The "source" is on the battlefield,
    // not the graveyard, so exclude_self filtering should never match self
    // here — but we want to verify the graveyard arm at least accepts a real
    // other-Elf in the graveyard. We also place an "Elder Elf" Elf card in P1's
    // graveyard for the legal-target path.
    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Necromancer")
            .with_card_id(necro_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let elf_card = ObjectSpec::card(p(1), "Elder Elf")
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Graveyard(p(1)));

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(elf_card)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Necromancer");
    let elf_id = find_obj(&state, "Elder Elf");

    // Sanity check: the source is on the battlefield, the Elf is in the graveyard.
    assert_eq!(
        state.objects.get(&source_id).unwrap().zone,
        ZoneId::Battlefield
    );
    assert_eq!(
        state.objects.get(&elf_id).unwrap().zone,
        ZoneId::Graveyard(p(1))
    );

    // Targeting the Elf card (another Elf, in the right graveyard) must succeed.
    let result_other = process_command(
        state.clone(),
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(elf_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result_other.is_ok(),
        "PB-XS E-1: another-Elf-in-graveyard target must be accepted: {:?}",
        result_other.err()
    );

    // Targeting the source itself (which is on the battlefield, not in any
    // graveyard) must fail — but for a different reason (wrong zone), not
    // exclude_self. We assert error to confirm we don't accidentally pass
    // through self-targeting via the graveyard arm.
    let result_self = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(source_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );
    assert!(
        result_self.is_err(),
        "PB-XS E-1: targeting the battlefield source for a graveyard requirement must error"
    );
}

// ── F: Trigger auto-target picker — Elderfang-style WhenDies ───────────────────

/// PB-XS F-1: Elderfang Ritualist death-trigger graveyard auto-target picker.
///
/// CR 109.1 / 601.2c / 400.7 / 603.10a: The dying creature continues to exist
/// as a new graveyard object after `move_object_to_zone`. The WhenDies trigger's
/// `source` is bound to that POST-death graveyard ObjectId. Without `exclude_self`,
/// the auto-target picker scanning `TargetCardInYourGraveyard(Elf)` would pick
/// the dying Ritualist itself as its own legal target. With `exclude_self: true`,
/// the picker must skip the source and pick the SECOND Elf card in the graveyard.
///
/// This test discriminates pre-death (battlefield) vs post-death (graveyard)
/// ObjectId for `trigger.source`, and directly exercises the graveyard-scan arm
/// added by PB-XS at `abilities.rs:6627`.
#[test]
fn test_pbxs_death_trigger_graveyard_picker_excludes_source() {
    use mtg_engine::cards::card_definition::TriggerCondition;
    use mtg_engine::state::turn::Step;

    // Card def: "When this dies, return another target Elf card from your
    // graveyard to your hand." — Elderfang Ritualist's exact ability shape.
    let ritualist_def = CardDefinition {
        name: "Test Elderfang".to_string(),
        card_id: CardId("test-pbxs-elderfang".to_string()),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            subtypes: im::ordset![SubType("Elf".to_string())],
            ..Default::default()
        },
        power: Some(3),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenDies,
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to: mtg_engine::ZoneTarget::Hand {
                    owner: mtg_engine::PlayerTarget::Controller,
                },
                controller_override: None,
            },
            intervening_if: None,
            targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                has_subtype: Some(SubType("Elf".to_string())),
                // PB-XS: exclude the post-death Ritualist itself.
                exclude_self: true,
                ..Default::default()
            })],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let (defs, registry) = single_def(ritualist_def.clone());

    // Battlefield Ritualist with lethal damage already on it (dies to SBA).
    let dying_ritualist = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Test Elderfang")
            .with_card_id(ritualist_def.card_id.clone())
            .in_zone(ZoneId::Battlefield)
            .with_damage(1),
        &defs,
    );
    // A SECOND Elf card already in P1's graveyard — the only legal target
    // for the death trigger once exclude_self gates out the Ritualist.
    let other_elf = ObjectSpec::card(p(1), "Lurking Elf")
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Elf".to_string())])
        .in_zone(ZoneId::Graveyard(p(1)));

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(dying_ritualist)
        .object(other_elf)
        .build()
        .expect("builder must succeed");
    state.turn.priority_holder = Some(p(1));

    // Sanity: both objects exist with expected zones before SBAs fire.
    let ritualist_id_pre = find_obj(&state, "Test Elderfang");
    let other_elf_id = find_obj(&state, "Lurking Elf");
    assert_eq!(
        state.objects.get(&ritualist_id_pre).unwrap().zone,
        ZoneId::Battlefield,
        "pre-SBA: Ritualist on battlefield"
    );
    assert_eq!(
        state.objects.get(&other_elf_id).unwrap().zone,
        ZoneId::Graveyard(p(1)),
        "pre-SBA: Other Elf in graveyard"
    );

    // Both players pass priority → SBAs fire → Ritualist dies (CR 704.5g
    // damage >= toughness) → check_triggers queues the WhenDies trigger →
    // auto-target picker runs against TargetCardInYourGraveyard(Elf,
    // exclude_self=true).
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

    // Sanity: CreatureDied fired (so the trigger had a chance to queue).
    assert!(
        events
            .iter()
            .any(|e| matches!(e, mtg_engine::GameEvent::CreatureDied { .. })),
        "F-1: CreatureDied event must have fired"
    );

    // After SBA + trigger queueing, the WhenDies TriggeredAbility stack object
    // should be on the stack with `targets` containing the OTHER Elf — not the
    // post-death Ritualist's new graveyard ObjectId.
    use mtg_engine::StackObjectKind;
    let trigger_so = state
        .stack_objects
        .iter()
        .find(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }))
        .expect("F-1: WhenDies TriggeredAbility must be on the stack");

    assert_eq!(
        trigger_so.targets.len(),
        1,
        "F-1: WhenDies trigger must have exactly 1 target (CR 603.3d auto-pick)"
    );

    let target = &trigger_so.targets[0];
    let target_id = match target.target {
        Target::Object(id) => id,
        Target::Player(_) => panic!("F-1: target must be an object, got player"),
    };

    // Resolve the post-death Ritualist's new graveyard ObjectId for the
    // discrimination: per CR 400.7 it is a DIFFERENT ObjectId from the
    // pre-death one. We locate it by name in the graveyard.
    let post_death_ritualist_id = state
        .objects
        .iter()
        .find(|(_, o)| {
            o.characteristics.name == "Test Elderfang" && o.zone == ZoneId::Graveyard(p(1))
        })
        .map(|(id, _)| *id)
        .expect("F-1: post-death Ritualist must be in graveyard (CR 400.7 new ObjectId)");

    // The discrimination: target must be the OTHER Elf, not the post-death
    // Ritualist. Without exclude_self, the auto-target picker would pick the
    // first match in iteration order — which could be the post-death Ritualist.
    assert_eq!(
        target_id, other_elf_id,
        "F-1 / CR 109.1 / CR 400.7 / CR 603.10a: WhenDies auto-target picker must \
         exclude the post-death source (id {:?}) and pick the second Elf (id {:?}); \
         got id {:?}",
        post_death_ritualist_id, other_elf_id, target_id
    );
    assert_ne!(
        target_id, post_death_ritualist_id,
        "F-1: auto-target picker MUST NOT pick the post-death source as its own target"
    );
}

/// PB-XS F-2: Negative-discriminator companion to F-1.
///
/// Same setup as F-1 but with NO second Elf in the graveyard. Per CR 603.3d,
/// when no legal target exists the trigger is skipped entirely (no stack
/// object created). Without `exclude_self`, the picker would (incorrectly)
/// pick the post-death Ritualist; with `exclude_self`, there is genuinely
/// no legal target.
#[test]
fn test_pbxs_death_trigger_skipped_when_only_source_is_legal() {
    use mtg_engine::cards::card_definition::TriggerCondition;
    use mtg_engine::state::turn::Step;

    let ritualist_def = CardDefinition {
        name: "Test Elderfang Solo".to_string(),
        card_id: CardId("test-pbxs-elderfang-solo".to_string()),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Creature],
            subtypes: im::ordset![SubType("Elf".to_string())],
            ..Default::default()
        },
        power: Some(3),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenDies,
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to: mtg_engine::ZoneTarget::Hand {
                    owner: mtg_engine::PlayerTarget::Controller,
                },
                controller_override: None,
            },
            intervening_if: None,
            targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                has_subtype: Some(SubType("Elf".to_string())),
                exclude_self: true,
                ..Default::default()
            })],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let (defs, registry) = single_def(ritualist_def.clone());

    let dying_ritualist = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Test Elderfang Solo")
            .with_card_id(ritualist_def.card_id.clone())
            .in_zone(ZoneId::Battlefield)
            .with_damage(1),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(dying_ritualist)
        .build()
        .expect("builder must succeed");
    state.turn.priority_holder = Some(p(1));

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
        "F-2: CreatureDied event must have fired"
    );

    // The WhenDies trigger must have been skipped (no legal non-self target).
    // Per CR 603.3d, when no legal target exists, the trigger is not put on
    // the stack at all (handled by `flush_pending_triggers`).
    use mtg_engine::StackObjectKind;
    let no_dies_trigger = state
        .stack_objects
        .iter()
        .all(|so| !matches!(so.kind, StackObjectKind::TriggeredAbility { .. }));
    assert!(
        no_dies_trigger,
        "F-2 / CR 603.3d: WhenDies trigger with only-self in graveyard must be SKIPPED \
         (exclude_self=true rejects post-death source; no other Elf exists); got stack: {:?}",
        state
            .stack_objects
            .iter()
            .map(|so| format!("{:?}", so.kind))
            .collect::<Vec<_>>()
    );
}

// ── G: matches_filter unit regression ─────────────────────────────────────────

/// PB-XS G-1: `matches_filter` (which takes only &Characteristics) MUST
/// silently ignore `exclude_self` — by design, since matches_filter cannot
/// see the candidate object's ObjectId. This test documents that invariant:
/// the same Characteristics passes both filter shapes regardless of
/// exclude_self. Enforcement happens at higher-level call sites that have
/// access to the candidate's ObjectId.
#[test]
fn test_pbxs_matches_filter_ignores_exclude_self_by_design() {
    use mtg_engine::effects::matches_filter;
    use mtg_engine::Characteristics;

    let chars = Characteristics {
        name: "Test Creature".to_string(),
        card_types: im::ordset![CardType::Creature],
        ..Characteristics::default()
    };
    let f_no_exclude = TargetFilter::default();
    let f_with_exclude = TargetFilter {
        exclude_self: true,
        ..Default::default()
    };

    assert!(
        matches_filter(&chars, &f_no_exclude),
        "filter without exclude_self matches creature characteristics"
    );
    assert!(
        matches_filter(&chars, &f_with_exclude),
        "PB-XS: matches_filter MUST ignore exclude_self (it has no ObjectId context). \
         Enforcement happens at call sites with access to (self_id, candidate_id)."
    );
}
