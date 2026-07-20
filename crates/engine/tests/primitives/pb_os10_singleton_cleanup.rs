//! Tests for PB-OS10 — singleton cleanup pair.
//!
//! Two independent primitives shipped in one batch:
//!
//! 1. `TargetRequirement::TargetPermanentDistinctFrom(usize)` (OOS-XS-1) — CR 601.2c
//!    "another target permanent" inter-target distinctness. Type-legality is identical
//!    to `TargetPermanent`; the distinctness constraint is enforced in a post-slot-
//!    assignment pass in `casting.rs`. `hidden_strings` wires the variant but stays
//!    `known_wrong` (tap/untap "may" optionality still unmodeled).
//! 2. `TriggerCondition::WhenEquippedCreatureDealsCombatDamage` +
//!    `TriggerEvent::EquippedCreatureDealsCombatDamage` (OOS-EF7-1) — CR 510.3a/603.2c
//!    any-recipient equipped-creature combat-damage trigger, distinct from the existing
//!    `...ToPlayer` pair. `umezawas_jitte` is execution-verified end-to-end (any-recipient
//!    trigger, `Cost::RemoveCounter` payment, and all three modal effects) and flips to
//!    `Complete`.
//!
//! `HASH_SCHEMA_VERSION` bumped 61 -> 62 (`TargetRequirement::TargetPermanentDistinctFrom`
//! discriminant 20; `TriggerEvent`/`TriggerCondition::EquippedCreatureDealsCombatDamage` /
//! `WhenEquippedCreatureDealsCombatDamage` discriminant 48 each). `PROTOCOL_VERSION` bumped
//! 24 -> 25 (`TargetRequirement` is reachable from `AbilityDefinition.targets`, part of the
//! wire closure; `TriggerEvent`/`TriggerCondition` are NOT wire-closure types).

use std::collections::HashMap;

use mtg_engine::{
    enrich_spec_from_def, process_command, AbilityDefinition, AttackTarget, CardDefinition,
    CardEffectTarget as EffectTarget, CardId, CardRegistry, CardType, CastSpellData, Command,
    CounterType, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, GameStateError,
    ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, SpellTarget, Step, Target,
    TargetRequirement, TypeLine, ZoneId, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
};

use mtg_engine::effects::{execute_effect, EffectContext};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn counter_count(state: &GameState, name: &str, counter: CounterType) -> u32 {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .and_then(|(_, obj)| obj.counters.get(&counter).copied())
        .unwrap_or(0)
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

fn attach(state: &mut GameState, equip_id: ObjectId, creature_id: ObjectId) {
    if let Some(creature_obj) = state.objects_mut().get_mut(&creature_id) {
        creature_obj.attachments = creature_obj
            .attachments
            .clone()
            .into_iter()
            .chain(std::iter::once(equip_id))
            .collect();
    }
    if let Some(equip_obj) = state.objects_mut().get_mut(&equip_id) {
        equip_obj.attached_to = Some(creature_id);
    }
}

// ── Version sentinel ─────────────────────────────────────────────────────────

/// CR 601.2c / 510.3a: PB-OS10 bumped both wire versions. Authoritative machine gates
/// are `tests/core/hash_schema.rs` / `tests/core/protocol_schema.rs`; this sentinel just
/// forces a deliberate edit here (and to `state/hash.rs` / `rules/protocol.rs`) on any
/// future bump, mirroring the convention on every other PB test module.
#[test]
fn test_pb_os10_version_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 63u8,
        "PB-OS10 added TargetRequirement::TargetPermanentDistinctFrom plus \
         TriggerEvent/TriggerCondition::(When)EquippedCreatureDealsCombatDamage \
         (HASH 61->62). Update this sentinel and the state/hash.rs history block together."
    );
    assert_eq!(
        PROTOCOL_VERSION, 26,
        "PB-OS10 added TargetRequirement::TargetPermanentDistinctFrom (PROTOCOL 24->25). \
         Update this sentinel and the rules/protocol.rs history block together."
    );
}

// ── Inter-target distinctness (OOS-XS-1) ─────────────────────────────────────

/// A minimal sorcery with `targets: [TargetPermanent, TargetPermanentDistinctFrom(0)]` —
/// the effect does nothing; only target validation is under test.
fn distinct_test_spell() -> CardDefinition {
    CardDefinition {
        name: "OS10 Distinct Test Spell".to_string(),
        card_id: CardId("test-os10-distinct-spell".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Sorcery],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![
                TargetRequirement::TargetPermanent,
                TargetRequirement::TargetPermanentDistinctFrom(0),
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a 2-player state with `distinct_test_spell` in p1's hand plus a creature and
/// a land on the battlefield (both controlled by p1). Returns (state, spell_id,
/// creature_id, land_id).
fn build_distinct_state() -> (GameState, ObjectId, ObjectId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let spell_def = distinct_test_spell();
    let registry: std::sync::Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let test_spell = ObjectSpec::card(p1, "OS10 Distinct Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone())
        .with_types(vec![CardType::Sorcery]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 1,
                ..ManaPool::default()
            },
        )
        .object(test_spell)
        .object(ObjectSpec::creature(p1, "OS10 Creature", 2, 2))
        .object(ObjectSpec::land(p1, "OS10 Land"))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .expect("build_distinct_state: GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "OS10 Distinct Test Spell");
    let creature_id = find_obj(&state, "OS10 Creature");
    let land_id = find_obj(&state, "OS10 Land");
    state.turn_mut().priority_holder = Some(p1);

    (state, spell_id, creature_id, land_id)
}

fn cast_spell(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
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
        })),
    )
}

/// CR 601.2c: the SAME permanent chosen for both a `TargetPermanent` slot and a
/// `TargetPermanentDistinctFrom(0)` slot is rejected.
#[test]
fn test_distinct_from_rejects_same_permanent() {
    let (state, spell_id, creature_id, _land_id) = build_distinct_state();
    let result = cast_spell(
        state,
        p(1),
        spell_id,
        vec![Target::Object(creature_id), Target::Object(creature_id)],
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "CR 601.2c: the same permanent cannot fill both a TargetPermanent slot and a \
         TargetPermanentDistinctFrom(0) slot, got: {:?}",
        result.map(|_| ())
    );
}

/// CR 601.2c: two DIFFERENT permanents are accepted, both bound.
#[test]
fn test_distinct_from_accepts_two_different() {
    let (state, spell_id, creature_id, land_id) = build_distinct_state();
    let (state, _events) = cast_spell(
        state,
        p(1),
        spell_id,
        vec![Target::Object(creature_id), Target::Object(land_id)],
    )
    .unwrap_or_else(|e| {
        panic!(
            "two different permanents must be accepted by TargetPermanentDistinctFrom: {:?}",
            e
        )
    });
    // CR 400.7: the cast mints a new stack ObjectId for the spell; a non-vacuous sanity
    // check is that the hand card is gone and a stack object now exists.
    assert!(
        state.objects().values().any(
            |o| o.characteristics.name == "OS10 Distinct Test Spell" && o.zone == ZoneId::Stack
        ),
        "the spell should be on the stack after a successful cast"
    );
}

/// CR 601.2c: `TargetPermanentDistinctFrom(0)` has the SAME type-legality as
/// `TargetPermanent` — it accepts any battlefield permanent (not just the same type as
/// slot 0) and rejects a non-battlefield object, independent of distinctness.
#[test]
fn test_distinct_from_type_legality() {
    // Part A: a non-battlefield object (the spell card itself, still in hand at
    // declaration time) is illegal for the second slot, exactly like TargetPermanent
    // would reject it.
    let (state, spell_id, creature_id, _land_id) = build_distinct_state();
    let result = cast_spell(
        state,
        p(1),
        spell_id,
        vec![Target::Object(creature_id), Target::Object(spell_id)],
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "a non-battlefield object is illegal for TargetPermanentDistinctFrom (same \
         type-legality as TargetPermanent), got: {:?}",
        result.map(|_| ())
    );

    // Part B: a DIFFERENT-TYPE battlefield permanent (a land vs. the creature in slot 0)
    // is legal — proving the check is "any permanent", not "same type as slot 0".
    let (state, spell_id, creature_id, land_id) = build_distinct_state();
    let result = cast_spell(
        state,
        p(1),
        spell_id,
        vec![Target::Object(creature_id), Target::Object(land_id)],
    );
    assert!(
        result.is_ok(),
        "a different-type battlefield permanent (land) is legal for \
         TargetPermanentDistinctFrom(0) even though slot 0 is a creature, got: {:?}",
        result.map(|_| ()).err()
    );
}

/// OOS-XS-1: Hidden Strings' second target slot IS `TargetPermanentDistinctFrom(0)`
/// (the primitive is wired), and casting it at the same permanent twice is rejected.
/// The card itself stays `known_wrong` (tap/untap "may" optionality unmodeled) — this
/// test only pins the primitive.
#[test]
fn test_hidden_strings_second_slot_distinct() {
    let card = mtg_engine::cards::defs::hidden_strings::card();
    let AbilityDefinition::Spell { targets, .. } = &card.abilities[0] else {
        panic!("expected Hidden Strings' first ability to be AbilityDefinition::Spell");
    };
    assert_eq!(
        targets,
        &vec![
            TargetRequirement::TargetPermanent,
            TargetRequirement::TargetPermanentDistinctFrom(0),
        ],
        "Hidden Strings must declare TargetPermanentDistinctFrom(0) on its second target"
    );

    let p1 = p(1);
    let p2 = p(2);
    let defs_map: HashMap<String, CardDefinition> =
        [(card.name.clone(), card.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![card.clone()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                blue: 1,
                colorless: 1,
                ..ManaPool::default()
            },
        )
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Hidden Strings")
                .with_card_id(card.card_id.clone())
                .in_zone(ZoneId::Hand(p1)),
            &defs_map,
        ))
        .object(ObjectSpec::creature(p1, "HS Creature", 2, 2))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let spell_id = find_obj(&state, "Hidden Strings");
    let creature_id = find_obj(&state, "HS Creature");

    let result = cast_spell(
        state,
        p1,
        spell_id,
        vec![Target::Object(creature_id), Target::Object(creature_id)],
    );
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "Hidden Strings cannot legally target the same permanent twice (CR 601.2c \
         'another target permanent'), got: {:?}",
        result.map(|_| ())
    );
}

// ── Jitte any-recipient combat trigger + modal ability (OOS-EF7-1) ───────────

/// Build a 2-player state with Umezawa's Jitte and a "Wielder" creature (p1's control),
/// plus any `extra` battlefield permanents. Jitte is NOT attached by default -- callers
/// use `attach` explicitly. `wielder_pt` sets the Wielder's power/toughness.
fn build_jitte_state(
    wielder_pt: (i32, i32),
    extra: Vec<ObjectSpec>,
) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::umezawas_jitte::card();
    let defs_map: HashMap<String, CardDefinition> =
        [(def.name.clone(), def.clone())].into_iter().collect();
    let registry = CardRegistry::new(vec![def.clone()]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Umezawa's Jitte")
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs_map,
        ))
        .object(ObjectSpec::creature(
            p1,
            "Wielder",
            wielder_pt.0,
            wielder_pt.1,
        ));
    for spec in extra {
        builder = builder.object(spec);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    (state, p1, p2)
}

fn activate(
    state: GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    targets: Vec<Target>,
    modes_chosen: Vec<usize>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index,
            targets,
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
}

/// Jitte's modal ability is its only entry in `activated_abilities` (Equip is not
/// wired through the indexed activated-ability path; the counters trigger is a
/// `TriggeredAbilityDef`, also not indexed there).
const JITTE_MODAL_ABILITY_INDEX: usize = 0;

/// CR 510.3a / 603.2c (core OOS-EF7-1 proof): equipped creature deals combat damage to
/// a CREATURE (a blocker) -- the any-recipient trigger fires, putting 2 charge counters
/// on Jitte. This is exactly the case the old `...ToPlayer` variant missed.
#[test]
fn test_jitte_triggers_on_damage_to_creature() {
    let (state, p1, p2) =
        build_jitte_state((3, 3), vec![ObjectSpec::creature(p(2), "Blocker", 3, 3)]);
    let jitte_id = find_obj(&state, "Umezawa's Jitte");
    let wielder_id = find_obj(&state, "Wielder");
    let blocker_id = find_obj(&state, "Blocker");
    let mut state = state;
    attach(&mut state, jitte_id, wielder_id);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(wielder_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers");
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, wielder_id)],
        },
    )
    .expect("DeclareBlockers");

    let mut state = state;
    for _ in 0..6 {
        let (s, _) = pass_all(state, &[p1, p2]);
        state = s;
    }

    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        2,
        "CR 510.3a: equipped creature dealing combat damage to a CREATURE must add 2 \
         charge counters (the any-recipient trigger, the surviving OOS-EF7-1 blocker)"
    );
}

/// CR 510.3a: damage to a PLAYER also fires the any-recipient trigger, adding 2 counters.
#[test]
fn test_jitte_triggers_on_damage_to_player() {
    let (state, p1, p2) = build_jitte_state((3, 3), vec![]);
    let jitte_id = find_obj(&state, "Umezawa's Jitte");
    let wielder_id = find_obj(&state, "Wielder");
    let mut state = state;
    attach(&mut state, jitte_id, wielder_id);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(wielder_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let mut state = state;
    for _ in 0..6 {
        let (s, _) = pass_all(state, &[p1, p2]);
        state = s;
    }

    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        2,
        "CR 510.3a: equipped creature dealing combat damage to a player must add 2 charge \
         counters"
    );
}

/// DECOY: non-combat damage from the equipped creature does NOT fire the trigger --
/// the trigger is combat-only (fired only from the combat-damage TBA collector).
#[test]
fn test_jitte_no_trigger_on_noncombat_damage() {
    let (state, p1, p2) = build_jitte_state((3, 3), vec![]);
    let jitte_id = find_obj(&state, "Umezawa's Jitte");
    let wielder_id = find_obj(&state, "Wielder");
    let mut state = state;
    attach(&mut state, jitte_id, wielder_id);

    let mut ctx = EffectContext::new(
        p1,
        wielder_id,
        vec![SpellTarget {
            target: Target::Player(p2),
            zone_at_cast: None,
        }],
    );
    let effect = Effect::DealDamage {
        target: EffectTarget::DeclaredTarget { index: 0 },
        amount: EffectAmount::Fixed(3),
        source: None,
    };
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        0,
        "DECOY: non-combat damage from the equipped creature must NOT add charge counters"
    );
}

/// DECOY: Jitte on the battlefield but NOT attached to anything -- a creature dealing
/// combat damage does not fire the trigger (requires attachment).
#[test]
fn test_jitte_no_trigger_when_unequipped() {
    let (state, p1, p2) = build_jitte_state((3, 3), vec![]);
    let wielder_id = find_obj(&state, "Wielder");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(wielder_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers");

    let mut state = state;
    for _ in 0..6 {
        let (s, _) = pass_all(state, &[p1, p2]);
        state = s;
    }

    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        0,
        "DECOY: an unattached Jitte must NOT gain charge counters from unrelated combat \
         damage"
    );
}

/// CR 603.2c: the trigger fires ONCE per equipped source creature per combat-damage
/// step, regardless of how many recipients it damaged. A 5/5 equipped attacker blocked
/// by two 2/2s (damage split via OrderBlockers) must add exactly 2 counters, not 4.
#[test]
fn test_jitte_fires_once_per_multiblock() {
    let (state, p1, p2) = build_jitte_state(
        (5, 5),
        vec![
            ObjectSpec::creature(p(2), "MB Blocker1", 2, 2),
            ObjectSpec::creature(p(2), "MB Blocker2", 2, 2),
        ],
    );
    let jitte_id = find_obj(&state, "Umezawa's Jitte");
    let wielder_id = find_obj(&state, "Wielder");
    let blocker1_id = find_obj(&state, "MB Blocker1");
    let blocker2_id = find_obj(&state, "MB Blocker2");
    let mut state = state;
    attach(&mut state, jitte_id, wielder_id);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(wielder_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers");
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker1_id, wielder_id), (blocker2_id, wielder_id)],
        },
    )
    .expect("DeclareBlockers");

    let (state, _) = process_command(
        state,
        Command::OrderBlockers {
            player: p1,
            attacker: wielder_id,
            order: vec![blocker1_id, blocker2_id],
        },
    )
    .expect("OrderBlockers");

    let mut state = state;
    for _ in 0..6 {
        let (s, _) = pass_all(state, &[p1, p2]);
        state = s;
    }

    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        2,
        "CR 603.2c: dealing combat damage to TWO blockers in one step must add exactly 2 \
         counters (one trigger), not 4"
    );
}

/// CR 510.3a: discriminant separation -- an Equipment carrying the OLD
/// `WhenEquippedCreatureDealsCombatDamageToPlayer` trigger does NOT fire from the new
/// any-recipient path when its equipped creature damages a CREATURE (only Jitte's new
/// variant covers that case).
#[test]
fn test_jitte_distinct_from_toplayer_variant() {
    let p1 = p(1);
    let p2 = p(2);
    let old_variant_trigger = mtg_engine::TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        trigger_on: mtg_engine::TriggerEvent::EquippedCreatureDealsCombatDamageToPlayer,
        intervening_if: None,
        description: "old to-player-only variant test trigger".to_string(),
        effect: Some(Effect::AddCounter {
            target: EffectTarget::Source,
            counter: CounterType::Charge,
            count: 2,
        }),
        etb_filter: None,
        death_filter: None,
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
    };
    let old_sword = ObjectSpec::artifact(p1, "OldVariantSword")
        .with_card_id(CardId("old-variant-sword".to_string()))
        .with_triggered_ability(old_variant_trigger);
    let wielder = ObjectSpec::creature(p1, "OV Wielder", 3, 3);
    let blocker = ObjectSpec::creature(p2, "OV Blocker", 3, 3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(old_sword)
        .object(wielder)
        .object(blocker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let sword_id = find_obj(&state, "OldVariantSword");
    let wielder_id = find_obj(&state, "OV Wielder");
    let blocker_id = find_obj(&state, "OV Blocker");
    let mut state = state;
    attach(&mut state, sword_id, wielder_id);

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(wielder_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers");
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, wielder_id)],
        },
    )
    .expect("DeclareBlockers");

    // Only 2 passes: one to resolve combat damage, one more in case a (spurious) trigger
    // needs resolving. No trigger fires here, so the state stays in the current turn's
    // combat steps -- unlike the Jitte-triggers-fire tests, this decoy must NOT cycle far
    // enough to rotate the active player, or a further PassPriority would target the
    // wrong player.
    let mut state = state;
    for _ in 0..2 {
        let (s, _) = pass_all(state, &[p1, p2]);
        state = s;
    }

    assert_eq!(
        counter_count(&state, "OldVariantSword", CounterType::Charge),
        0,
        "the OLD WhenEquippedCreatureDealsCombatDamageToPlayer variant must NOT fire when \
         its equipped creature damages a creature (discriminant separation from the new \
         any-recipient variant)"
    );
    // Non-vacuity: damage was in fact dealt (the blocker took a hit).
    assert!(
        !state.objects().values().any(|o| o.id == blocker_id),
        "sanity: the 3/3 blocker took 3 damage and should have died"
    );
}

// ── Jitte modal ability (RemoveCounter cost + 3 modes) ───────────────────────

/// CR 602.2 / 700.2a: with 0 charge counters, the modal ability cannot be activated
/// (the RemoveCounter cost cannot be paid).
#[test]
fn test_jitte_cost_requires_counter() {
    let (state, p1, _p2) = build_jitte_state((3, 3), vec![]);
    let jitte_id = find_obj(&state, "Umezawa's Jitte");

    let result = activate(
        state,
        p1,
        jitte_id,
        JITTE_MODAL_ABILITY_INDEX,
        vec![],
        vec![0],
    );
    assert!(
        result.is_err(),
        "CR 602.2: Umezawa's Jitte with 0 charge counters cannot activate the \
         RemoveCounter-cost ability"
    );
}

/// CR 602.2 / 700.2a: mode 0 selected -- equipped creature gets +2/+2 EOT, and the
/// RemoveCounter cost removes exactly one charge counter at activation.
#[test]
fn test_jitte_mode0_pumps_equipped() {
    let (state, p1, _p2) = build_jitte_state((3, 3), vec![]);
    let jitte_id = find_obj(&state, "Umezawa's Jitte");
    let wielder_id = find_obj(&state, "Wielder");
    let mut state = state;
    attach(&mut state, jitte_id, wielder_id);
    if let Some(obj) = state.objects_mut().get_mut(&jitte_id) {
        obj.counters.insert(CounterType::Charge, 1);
    }

    let (state, _) = activate(
        state,
        p1,
        jitte_id,
        JITTE_MODAL_ABILITY_INDEX,
        vec![],
        vec![0],
    )
    .unwrap_or_else(|e| panic!("mode-0 activation should succeed: {:?}", e));

    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        0,
        "CR 602.2c: the RemoveCounter cost is paid at activation, before resolution"
    );

    let (state, _) = pass_all(state, &[p1, p(2)]);
    let wielder_chars = mtg_engine::calculate_characteristics(&state, wielder_id).unwrap();
    assert_eq!(
        wielder_chars.power,
        Some(5),
        "CR 700.2a: mode 0 (equipped creature +2/+2) must apply to the equipped creature \
         (base 3 -> 5), even in an activated-modal context"
    );
    assert_eq!(wielder_chars.toughness, Some(5));
}

/// CR 602.2 / 700.2a: mode 1 selected -- a targeted creature (not the equipped one) gets
/// -1/-1 EOT.
#[test]
fn test_jitte_mode1_shrinks_target() {
    let (state, p1, p2) = build_jitte_state(
        (3, 3),
        vec![ObjectSpec::creature(p(2), "Shrink Target", 2, 2)],
    );
    let jitte_id = find_obj(&state, "Umezawa's Jitte");
    let wielder_id = find_obj(&state, "Wielder");
    let target_id = find_obj(&state, "Shrink Target");
    let mut state = state;
    attach(&mut state, jitte_id, wielder_id);
    if let Some(obj) = state.objects_mut().get_mut(&jitte_id) {
        obj.counters.insert(CounterType::Charge, 1);
    }

    let (state, _) = activate(
        state,
        p1,
        jitte_id,
        JITTE_MODAL_ABILITY_INDEX,
        vec![Target::Object(target_id)],
        vec![1],
    )
    .unwrap_or_else(|e| panic!("mode-1 activation should succeed: {:?}", e));

    let (state, _) = pass_all(state, &[p1, p2]);
    let target_chars = mtg_engine::calculate_characteristics(&state, target_id).unwrap();
    assert_eq!(
        target_chars.power,
        Some(1),
        "CR 700.2a: mode 1 (-1/-1 to a targeted creature) must apply (base 2 -> 1)"
    );
    assert_eq!(target_chars.toughness, Some(1));
}

/// CR 602.2 / 700.2c: mode 2 selected -- controller gains 2 life. Mode 2 has an EMPTY
/// `mode_targets` slice; activating it with no target supplied must succeed.
#[test]
fn test_jitte_mode2_gains_life() {
    let (state, p1, _p2) = build_jitte_state((3, 3), vec![]);
    let jitte_id = find_obj(&state, "Umezawa's Jitte");
    let mut state = state;
    if let Some(obj) = state.objects_mut().get_mut(&jitte_id) {
        obj.counters.insert(CounterType::Charge, 1);
    }
    let life_before = state.players().get(&p1).unwrap().life_total;

    let (state, _) = activate(
        state,
        p1,
        jitte_id,
        JITTE_MODAL_ABILITY_INDEX,
        vec![],
        vec![2],
    )
    .unwrap_or_else(|e| {
        panic!(
            "CR 700.2c: mode 2 (empty mode_targets slice) must not require a target: {:?}",
            e
        )
    });

    let (state, _) = pass_all(state, &[p1, p(2)]);
    let life_after = state.players().get(&p1).unwrap().life_total;
    assert_eq!(
        life_after,
        life_before + 2,
        "CR 700.2a: mode 2 must gain the controller 2 life"
    );
}

/// CR 122.6 / 602.2c: two combat-damage events add 4 counters total; spending 2 across
/// two mode-2 activations leaves 2 remaining (accumulation + spend round-trip).
#[test]
fn test_jitte_counter_accumulation_roundtrip() {
    let (state, p1, p2) = build_jitte_state((3, 3), vec![]);
    let jitte_id = find_obj(&state, "Umezawa's Jitte");
    let wielder_id = find_obj(&state, "Wielder");
    let mut state = state;
    attach(&mut state, jitte_id, wielder_id);

    // First combat step: unblocked attack, +2 counters.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(wielder_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers");
    let mut state = state;
    for _ in 0..6 {
        let (s, _) = pass_all(state, &[p1, p2]);
        state = s;
    }
    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        2
    );

    // Force a second combat-damage event by directly re-adding 2 more counters
    // (simulating a second attack step without re-running the full turn cycle,
    // which combat_damage_triggers.rs establishes fires reliably per-step already).
    if let Some(obj) = state.objects_mut().get_mut(&jitte_id) {
        let cur = obj.counters.get(&CounterType::Charge).copied().unwrap_or(0);
        obj.counters.insert(CounterType::Charge, cur + 2);
    }
    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        4
    );

    // Spend 2 counters across two mode-2 (gain life) activations.
    let (state, _) = activate(
        state,
        p1,
        jitte_id,
        JITTE_MODAL_ABILITY_INDEX,
        vec![],
        vec![2],
    )
    .unwrap_or_else(|e| panic!("first spend should succeed: {:?}", e));
    let (state, _) = pass_all(state, &[p1, p2]);

    let (state, _) = activate(
        state,
        p1,
        jitte_id,
        JITTE_MODAL_ABILITY_INDEX,
        vec![],
        vec![2],
    )
    .unwrap_or_else(|e| panic!("second spend should succeed: {:?}", e));
    let (state, _) = pass_all(state, &[p1, p2]);

    assert_eq!(
        counter_count(&state, "Umezawa's Jitte", CounterType::Charge),
        2,
        "4 counters accumulated, 2 spent across two activations -- 2 must remain"
    );
}
