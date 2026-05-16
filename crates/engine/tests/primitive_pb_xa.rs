//! PB-XA: TargetFilter.is_attacking — runtime enforcement at validate sites and
//! trigger auto-target picker.
//!
//! Tests verify that the pre-existing `is_attacking: bool` field on `TargetFilter`
//! is now correctly enforced at every call site after PB-XA:
//! - The declarative target validation path used by activated abilities
//!   (`casting::validate_object_satisfies_requirement`, CR 601.2c) for
//!   TargetCreatureWithFilter (V1) and TargetPermanentWithFilter (V2).
//! - The graveyard arm (TargetCardInYourGraveyard V3) — graveyard objects are
//!   never in combat.attackers, so is_attacking=true always rejects them.
//! - The trigger auto-target picker in `abilities.rs` for TargetPermanentWithFilter
//!   (T4 top-level) — picks an attacking creature over a non-attacking one, and
//!   skips the trigger entirely (CR 603.3d) when no attacker exists.
//!
//! CR Rules covered:
//! - CR 508.1k: A "creature that's attacking" is one in `CombatState.attackers`;
//!   battlefield only — graveyard objects cannot be attacking.
//! - CR 601.2c: At cast/activation time each declared target must satisfy the
//!   TargetRequirement, including `is_attacking`.
//! - CR 603.3d: A triggered ability that would require targets but has no legal
//!   targets is skipped (not put on the stack).
//!
//! HASH invariant: `is_attacking` was a pre-existing field already hashed in
//! `state/hash.rs`. PB-XA adds NO new fields; HASH_SCHEMA_VERSION remains 20u8.

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

// ── A: Hash schema sentinel ───────────────────────────────────────────────────

/// PB-XA A-1: HASH_SCHEMA_VERSION sentinel.
/// is_attacking is a pre-existing TargetFilter field already hashed at
/// hash.rs:4347. PB-XA adds no new fields, struct changes, or discriminants,
/// so no HASH bump is needed or expected. When the next PB bumps the version,
/// update this sentinel to match (keep it in lockstep with the live constant).
#[test]
fn test_pb_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 27u8,
        "BASELINE-LKI-01 bumped HASH_SCHEMA_VERSION 26→27 (GameEvent::CreatureDied.pre_death_characteristics: Option<Characteristics>, CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers). If you bumped again, update this test and state/hash.rs history."
    );
}

// ── B: TargetFilter equality discriminator ─────────────────────────────────────

/// PB-XA B-1: Filters that differ only in `is_attacking` are NOT equal.
/// The field was already hashed; this confirms it participates in PartialEq.
#[test]
fn test_pbxa_filter_equality_distinguishes_is_attacking() {
    let f_default = TargetFilter::default();
    let f_attacking = TargetFilter {
        is_attacking: true,
        ..Default::default()
    };
    assert_ne!(
        f_default, f_attacking,
        "is_attacking difference must be observable via PartialEq"
    );
}

// ── C: Activated-ability declarative validation — TargetCreatureWithFilter ────

/// Build an activated ability: "{T}: Choose another target attacking creature."
/// Exercises V1 (TargetCreatureWithFilter with is_attacking=true).
fn attacking_creature_target_ability(name: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-pbxa-{}",
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
            // PB-XA: is_attacking=true — only attacking creatures are legal targets.
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                is_attacking: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

/// PB-XA C-1: Activated ability with TargetCreatureWithFilter{is_attacking=true}
/// rejects a non-attacking creature as a target.
/// Exercises V1 negative path.
#[test]
fn test_pbxa_activated_target_creature_not_attacking_rejected() {
    let source_def = attacking_creature_target_ability("Watcher");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Watcher")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    // A creature that is on the battlefield but NOT in combat.
    let non_attacker =
        ObjectSpec::creature(p(2), "Sitting Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(non_attacker)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Watcher");
    let non_attacker_id = find_obj(&state, "Sitting Bear");

    // No combat state — Sitting Bear is not attacking.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(non_attacker_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "PB-XA C-1: is_attacking=true must reject non-attacking creature (no combat state)"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

/// PB-XA C-2: Activated ability with TargetCreatureWithFilter{is_attacking=true}
/// accepts a creature that IS in combat.attackers.
/// Exercises V1 positive path.
#[test]
fn test_pbxa_activated_target_creature_attacking_accepted() {
    let source_def = attacking_creature_target_ability("Watcher");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Watcher")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let attacker = ObjectSpec::creature(p(2), "Charging Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(attacker)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Watcher");
    let attacker_id = find_obj(&state, "Charging Bear");

    // Inject combat state: Charging Bear is attacking.
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
        "PB-XA C-2: is_attacking=true must accept a creature in combat.attackers: {:?}",
        result.err()
    );
}

// ── D: Activated-ability declarative validation — TargetPermanentWithFilter ───

/// Build an activated ability with TargetPermanentWithFilter{is_attacking=true}.
/// Exercises V2.
fn attacking_permanent_target_ability(name: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-pbxa-perm-{}",
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
            // PB-XA: is_attacking=true on TargetPermanentWithFilter.
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                is_attacking: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

/// PB-XA D-1: TargetPermanentWithFilter{is_attacking=true} rejects a non-attacking
/// permanent. Exercises V2 negative path.
#[test]
fn test_pbxa_activated_target_permanent_not_attacking_rejected() {
    let source_def = attacking_permanent_target_ability("Arbiter");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Arbiter")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let non_attacker = ObjectSpec::creature(p(2), "Idle Guard", 1, 4).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(non_attacker)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Arbiter");
    let non_attacker_id = find_obj(&state, "Idle Guard");

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(non_attacker_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "PB-XA D-1: is_attacking=true must reject non-attacking permanent"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

/// PB-XA D-2: TargetPermanentWithFilter{is_attacking=true} accepts a permanent
/// that IS in combat.attackers. Exercises V2 positive path.
#[test]
fn test_pbxa_activated_target_permanent_attacking_accepted() {
    let source_def = attacking_permanent_target_ability("Arbiter");
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Arbiter")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    let attacker = ObjectSpec::creature(p(2), "Charging Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(source)
        .object(attacker)
        .build()
        .expect("builder must succeed");

    let source_id = find_obj(&state, "Arbiter");
    let attacker_id = find_obj(&state, "Charging Bear");

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
        "PB-XA D-2: is_attacking=true must accept a permanent in combat.attackers: {:?}",
        result.err()
    );
}

// ── E: Graveyard arm (TargetCardInYourGraveyard) — always rejects ─────────────

/// PB-XA E-1: TargetCardInYourGraveyard{is_attacking=true} rejects a graveyard
/// card even when combat is active (graveyard objects are never in
/// combat.attackers; CR 508.1k requires a creature on the battlefield).
///
/// Exercises V3: the check uniformly rejects any is_attacking=true graveyard
/// target regardless of the combat state.
#[test]
fn test_pbxa_graveyard_target_with_is_attacking_always_rejected() {
    // A battlefield creature with an activated ability targeting
    // "another attacking Elf card from your graveyard" (contrived but
    // exercises the V3 code path cleanly).
    let source_def = CardDefinition {
        name: "Grave Caller".to_string(),
        card_id: CardId("test-pbxa-grave-caller".to_string()),
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
            targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                has_card_type: Some(CardType::Creature),
                is_attacking: true,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };
    let (defs, registry) = single_def(source_def.clone());

    let source = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Grave Caller")
            .with_card_id(source_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    );
    // A creature card in P1's graveyard — graveyard, not battlefield.
    let gy_card = ObjectSpec::card(p(1), "Dead Bear")
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

    let source_id = find_obj(&state, "Grave Caller");
    let gy_card_id = find_obj(&state, "Dead Bear");

    // Even with an active combat state (with a *different* object attacking),
    // the graveyard card cannot satisfy is_attacking=true.
    // We inject combat with source_id as attacker (battlefield object is in combat;
    // the graveyard card is not).
    let mut state = state;
    state.combat = Some(combat_with_attacker(p(1), source_id));

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: source_id,
            ability_index: 0,
            targets: vec![Target::Object(gy_card_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "PB-XA E-1 / CR 508.1k: graveyard objects are never attacking — \
         TargetCardInYourGraveyard{{is_attacking=true}} must always reject them"
    );
    assert!(matches!(
        result.unwrap_err(),
        mtg_engine::GameStateError::InvalidTarget(_)
    ));
}

// ── F: Trigger auto-target picker — Thousand-Faced-Shadow-shaped ETB ─────────

/// PB-XA F-1: ETB trigger auto-target picker with TargetPermanentWithFilter
/// {is_attacking=true} picks the attacking creature over the non-attacking one.
///
/// Setup:
/// - P1 controls a TFS-shaped creature on the battlefield (the triggering source).
/// - P2 controls two creatures: "Sitter" (non-attacking, SMALLER ObjectId, added
///   first) and "Ravager" (attacker, LARGER ObjectId, added second).
/// - state.combat is injected with Ravager as the sole attacker.
///
/// Expected: the trigger auto-picks Ravager (the attacker), skipping Sitter.
///
/// ObjectId-ordering discriminator (cf. PB-XS E1 HIGH / H-XA-01 review finding):
/// The auto-target picker walks state.objects (BTreeMap, ascending ObjectId) and
/// short-circuits on the first legal match. Sitter is added BEFORE Ravager so it
/// gets the smaller ObjectId and is seen FIRST in iteration order. Without PB-XA
/// (`passes_attacking` removed), the picker finds Sitter legal and returns it —
/// the assertion `target_id == ravager_id` FAILS. With PB-XA enabled, Sitter
/// fails the `passes_attacking` gate; the picker advances to Ravager and returns
/// it — the assertion PASSES. This construction guarantees the test only passes
/// when `passes_attacking` enforcement is active (CR 508.1k / 601.2c).
///
/// Exercises T4 (top-level TargetPermanentWithFilter in abilities.rs:6783-6810).
#[test]
fn test_pbxa_trigger_picker_selects_attacking_creature_positive() {
    // The TFS-shaped ETB source: put it on the battlefield (triggers on ETB).
    // We build the state with it already on battlefield so we can manipulate
    // combat state before the trigger fires via SBA / zone transition.
    // Instead of actually casting it (which would require mana), we build the
    // state with it already on the battlefield and manually inject the ETB
    // by letting the auto-target picker run via a synthetic pending trigger.
    //
    // Practical approach: build a state where the Ninja is already on the
    // battlefield WITH damage that causes it to die (so SBA fires), but
    // that doesn't exercise the ETB. Instead, we simulate ETB by using
    // the established pattern from primitive_pb_xs.rs F-1:
    //
    //   Build dying creature → SBA fires → check_triggers → WhenDies trigger queued
    //
    // We use a WhenDies trigger (not WhenEntersBattlefield) for simplicity — the
    // same T4 TargetPermanentWithFilter auto-target picker code path is exercised
    // regardless of the trigger condition.

    // Re-define as WhenDies to use the SBA-death trigger flow (same T4 code path).
    let ninja_def_dies = CardDefinition {
        name: "Shadow Ninja".to_string(),
        card_id: CardId("test-pbxa-tfs-shadow-ninja".to_string()),
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
            // T4 path: TargetPermanentWithFilter{has_card_type=Creature, is_attacking=true}.
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_type: Some(CardType::Creature),
                is_attacking: true,
                ..Default::default()
            })],
            intervening_if: None,
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let (defs2, registry2) = {
        let def = ninja_def_dies.clone();
        let defs: HashMap<String, CardDefinition> =
            std::iter::once((def.name.clone(), def.clone())).collect();
        let registry = CardRegistry::new(vec![def]);
        (defs, registry)
    };

    // Shadow Ninja: dying (toughness 1, damage 1).
    let dying_ninja = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Shadow Ninja")
            .with_card_id(ninja_def_dies.card_id.clone())
            .in_zone(ZoneId::Battlefield)
            .with_damage(1),
        &defs2,
    );
    // ObjectId-ordering discriminator: Sitter is added BEFORE Ravager so it gets
    // the smaller ObjectId and is visited first by the auto-target picker's BTreeMap
    // iteration. Without `passes_attacking` enforcement, the picker would short-
    // circuit on Sitter (legal non-attacker with smaller ID) and the assertion
    // `target_id == ravager_id` would fail — proving the test is not a tautology.
    let sitter = ObjectSpec::creature(p(2), "Sitter", 2, 2).in_zone(ZoneId::Battlefield);
    let ravager = ObjectSpec::creature(p(2), "Ravager", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry2)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(dying_ninja)
        .object(sitter) // Sitter first → smaller ObjectId (seen first in iteration).
        .object(ravager) // Ravager second → larger ObjectId.
        .build()
        .expect("builder must succeed");
    state.turn.priority_holder = Some(p(1));

    let ravager_id = find_obj(&state, "Ravager");
    let sitter_id = find_obj(&state, "Sitter");

    // Sanity: confirm the ObjectId ordering assumption holds. If the builder ever
    // changes insertion order this assertion will catch the re-tautologisation
    // before a false-green test can slip through review.
    assert!(
        sitter_id < ravager_id,
        "F-1 discriminator invariant: Sitter (added first) must have a smaller ObjectId \
         than Ravager (added second). Got sitter={:?} ravager={:?}. \
         If the builder's ObjectId-assignment order changed, adjust the add order above.",
        sitter_id,
        ravager_id
    );

    // Inject combat state: Ravager is the sole attacker.
    // Note: attacking_player p(1) does not match Ravager's controller p(2) — an MTG-
    // impossible state — but the validate/picker sites only check attackers.contains_key,
    // not attacking_player, so the discrimination is unaffected (L-XA-01).
    state.combat = Some(combat_with_attacker(p(1), ravager_id));

    // Both players pass priority → SBAs fire → Shadow Ninja dies → WhenDies
    // trigger queues → auto-target picker runs on TargetPermanentWithFilter
    // {has_card_type=Creature, is_attacking=true} → must pick Ravager.
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
        "F-1: CreatureDied must have fired"
    );

    // The WhenDies TriggeredAbility must be on the stack with target = Ravager.
    let trigger_so = state
        .stack_objects
        .iter()
        .find(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }))
        .expect("F-1: WhenDies TriggeredAbility must be on the stack");

    assert_eq!(
        trigger_so.targets.len(),
        1,
        "F-1: trigger must have exactly 1 target"
    );

    let target_id = match trigger_so.targets[0].target {
        Target::Object(id) => id,
        Target::Player(_) => panic!("F-1: target must be an object, got player"),
    };

    assert_eq!(
        target_id, ravager_id,
        "PB-XA F-1 / CR 508.1k: auto-target picker must select Ravager (the attacker, id {:?}), \
         not Sitter (non-attacking, id {:?}). Got id {:?}.",
        ravager_id, sitter_id, target_id
    );
    assert_ne!(
        target_id, sitter_id,
        "PB-XA F-1: auto-target picker must NOT select non-attacking Sitter"
    );
}

/// PB-XA F-2: ETB trigger auto-target picker with TargetPermanentWithFilter
/// {is_attacking=true} is SKIPPED when no creature is in combat.attackers.
///
/// Per CR 603.3d, a triggered ability that requires targets but has no legal
/// targets is not put on the stack at all.
///
/// Without PB-XA, the picker would find a battlefield creature regardless of
/// attacking status and put a (now-illegal) trigger on the stack.
#[test]
fn test_pbxa_trigger_picker_skipped_when_no_attacker() {
    // Same WhenDies + TargetPermanentWithFilter{is_attacking=true} shape.
    let ninja_def_dies = CardDefinition {
        name: "Shadow Ninja Solo".to_string(),
        card_id: CardId("test-pbxa-tfs-shadow-ninja-solo".to_string()),
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
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_type: Some(CardType::Creature),
                is_attacking: true,
                ..Default::default()
            })],
            intervening_if: None,
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    };
    let (defs2, registry2) = {
        let def = ninja_def_dies.clone();
        let defs: HashMap<String, CardDefinition> =
            std::iter::once((def.name.clone(), def.clone())).collect();
        let registry = CardRegistry::new(vec![def]);
        (defs, registry)
    };

    // Shadow Ninja Solo: dying.
    let dying_ninja = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Shadow Ninja Solo")
            .with_card_id(ninja_def_dies.card_id.clone())
            .in_zone(ZoneId::Battlefield)
            .with_damage(1),
        &defs2,
    );
    // A non-attacking creature exists on the battlefield — but it's not in combat.
    let sitter = ObjectSpec::creature(p(2), "Peaceful Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry2)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(dying_ninja)
        .object(sitter)
        .build()
        .expect("builder must succeed");
    state.turn.priority_holder = Some(p(1));
    // No combat state — no attackers.

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
        "F-2: CreatureDied must have fired"
    );

    // Per CR 603.3d: trigger with no legal target is not put on the stack.
    let no_trigger = state
        .stack_objects
        .iter()
        .all(|so| !matches!(so.kind, StackObjectKind::TriggeredAbility { .. }));
    assert!(
        no_trigger,
        "PB-XA F-2 / CR 603.3d: WhenDies trigger with is_attacking=true and no attackers \
         must be SKIPPED (no stack object created). Stack: {:?}",
        state
            .stack_objects
            .iter()
            .map(|so| format!("{:?}", so.kind))
            .collect::<Vec<_>>()
    );
}

// ── G: matches_filter regression invariant ────────────────────────────────────

/// PB-XA G-1: `matches_filter` (which takes only &Characteristics) MUST
/// silently ignore `is_attacking` — by design. The field is a runtime property
/// of `CombatState.attackers`, not of `Characteristics`. Enforcement happens at
/// higher-level call sites (validate_object_satisfies_requirement and the
/// trigger auto-target picker) that have access to `state.combat`.
///
/// This test documents the invariant: the same Characteristics passes both
/// filter shapes regardless of is_attacking.
#[test]
fn test_pbxa_matches_filter_ignores_is_attacking_by_design() {
    use mtg_engine::effects::matches_filter;
    use mtg_engine::Characteristics;

    let chars = Characteristics {
        name: "Test Creature".to_string(),
        card_types: im::ordset![CardType::Creature],
        ..Characteristics::default()
    };
    let f_no_attacking = TargetFilter::default();
    let f_with_attacking = TargetFilter {
        is_attacking: true,
        ..Default::default()
    };

    assert!(
        matches_filter(&chars, &f_no_attacking),
        "filter without is_attacking matches creature characteristics"
    );
    assert!(
        matches_filter(&chars, &f_with_attacking),
        "PB-XA: matches_filter MUST ignore is_attacking (no CombatState context). \
         Enforcement happens at call sites that have access to state.combat."
    );
}
