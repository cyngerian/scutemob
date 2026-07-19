//! Tests for PB-OS11 Part B: the batch filtered-attack trigger (OOS-TS-1 reframed,
//! CR 508.1 / CR 508.1m / CR 603.2c).
//!
//! `TriggerCondition::WheneverYouAttack` went from a bare unit to
//! `{ filter: Option<TargetFilter> }`. The trigger is a BATCH trigger — it fires ONCE
//! per combat iff at least one declared attacker controlled by the trigger's
//! controller matches `filter` — distinct from `WheneverCreatureYouControlAttacks
//! {filter}`, which fires once PER matching attacker (would over-trigger).
//!
//! Card coverage: Anim Pakal, Thousandth Moon (`exclude_subtypes: [Gnome]`), General
//! Kreat, the Boltbringer (`has_subtype: Goblin`), Hermes, Overseer of Elpis
//! (`has_subtype: Bird`).
//!
//! Pattern follows `rules/trigger_variants.rs`'s `test_whenever_you_attack_fires_once_
//! per_combat` (combat via `Command::DeclareAttackers` + `pass_all` to drain the
//! stack) combined with `primitive_pb_ewc.rs`'s `all_cards()` + `enrich_spec_from_def`
//! real-card-definition pattern.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AttackTarget,
    CardDefinition, CardRegistry, Command, CounterType, GameEvent, GameState, GameStateBuilder,
    ObjectId, ObjectSpec, PlayerId, Step, SubType, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
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

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Fully drain the stack (in case a resolved trigger's effect puts something else on
/// the stack, or multiple pass_all rounds are needed).
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut guard = 0;
    while !state.stack_objects().is_empty() && guard < 10 {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
        guard += 1;
    }
    (state, all_events)
}

fn counters_on(state: &GameState, id: ObjectId, counter: CounterType) -> u32 {
    state
        .objects()
        .get(&id)
        .and_then(|o| o.counters.get(&counter).copied())
        .unwrap_or(0)
}

fn count_creatures_named(state: &GameState, name: &str) -> usize {
    state
        .objects()
        .iter()
        .filter(|(_, o)| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
        .count()
}

// ── Anim Pakal, Thousandth Moon ──────────────────────────────────────────────────

/// Build p1's battlefield with Anim Pakal plus `attackers` (name -> is_gnome), all
/// untapped and attack-ready (no summoning sickness, per `ObjectSpec::creature`
/// defaults). Anim Pakal itself does not attack (ruling: it need not be one of the
/// attackers).
fn build_anim_pakal_state(attackers: &[(&str, bool)]) -> (GameState, ObjectId, Vec<ObjectId>) {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_defs_and_registry();
    let anim_pakal_spec = enrich(
        p1,
        "Anim Pakal, Thousandth Moon",
        ZoneId::Battlefield,
        &defs,
    );

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(anim_pakal_spec);

    for &(name, is_gnome) in attackers {
        let mut spec = ObjectSpec::creature(p1, name, 1, 1);
        if is_gnome {
            spec = spec.with_subtypes(vec![SubType("Gnome".to_string())]);
        }
        builder = builder.object(spec);
    }

    let mut state = builder
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let anim_pakal_id = find_by_name(&state, "Anim Pakal, Thousandth Moon");
    let attacker_ids: Vec<ObjectId> = attackers
        .iter()
        .map(|(name, _)| find_by_name(&state, name))
        .collect();
    (state, anim_pakal_id, attacker_ids)
}

fn declare_attack_all(
    state: GameState,
    player: PlayerId,
    defender: PlayerId,
    attackers: &[ObjectId],
) -> (GameState, Vec<GameEvent>) {
    process_command(
        state,
        Command::DeclareAttackers {
            player,
            attackers: attackers
                .iter()
                .map(|&id| (id, AttackTarget::Player(defender)))
                .collect(),
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("DeclareAttackers failed: {e:?}"))
}

/// CR 508.1m: attacking with ONLY a Gnome creature must NOT fire Anim Pakal's
/// trigger — `exclude_subtypes: [Gnome]` correctly excludes it, so no attacker
/// matches the filter.
#[test]
fn test_anim_pakal_gnome_only_attack_does_not_fire() {
    let (state, anim_pakal_id, attacker_ids) = build_anim_pakal_state(&[("Gnome Attacker", true)]);
    let before_gnomes = count_creatures_named(&state, "Gnome");

    let (state, _) = declare_attack_all(state, p(1), p(2), &attacker_ids);
    let (state, _) = drain_stack(state, &[p(1), p(2)]);

    assert_eq!(
        counters_on(&state, anim_pakal_id, CounterType::PlusOnePlusOne),
        0,
        "a Gnome-only attack must NOT put a +1/+1 counter on Anim Pakal"
    );
    assert_eq!(
        count_creatures_named(&state, "Gnome"),
        before_gnomes,
        "no new Gnome tokens should be created"
    );
}

/// CR 508.1m: attacking with ONE non-Gnome creature fires the trigger exactly once:
/// +1 counter, then create X=1 Gnome tokens (post-increment count).
#[test]
fn test_anim_pakal_nongnome_attack_fires_once() {
    let (state, anim_pakal_id, attacker_ids) =
        build_anim_pakal_state(&[("Non-Gnome Attacker", false)]);

    let (state, _) = declare_attack_all(state, p(1), p(2), &attacker_ids);
    let (state, _) = drain_stack(state, &[p(1), p(2)]);

    assert_eq!(
        counters_on(&state, anim_pakal_id, CounterType::PlusOnePlusOne),
        1,
        "one non-Gnome attacker must fire the trigger once, adding one +1/+1 counter"
    );
    assert_eq!(
        count_creatures_named(&state, "Gnome"),
        1,
        "exactly one Gnome token must be created (X = 1, the post-increment counter count)"
    );
}

/// The batch-once correctness core (CR 508.1/508.1m): THREE non-Gnome attackers must
/// fire the trigger exactly ONCE, not three times. `WheneverCreatureYouControlAttacks
/// {filter}` would wrongly fire per-creature (3 counters, 3 token batches); the batch
/// `WheneverYouAttack{filter}` must produce 1 counter and 1 token (X=1).
#[test]
fn test_anim_pakal_multiple_nongnome_attackers_fires_once() {
    let (state, anim_pakal_id, attacker_ids) = build_anim_pakal_state(&[
        ("Attacker A", false),
        ("Attacker B", false),
        ("Attacker C", false),
    ]);

    let (state, _) = declare_attack_all(state, p(1), p(2), &attacker_ids);
    let (state, _) = drain_stack(state, &[p(1), p(2)]);

    assert_eq!(
        counters_on(&state, anim_pakal_id, CounterType::PlusOnePlusOne),
        1,
        "three non-Gnome attackers must still fire the batch trigger exactly ONCE \
         (one +1/+1 counter), not once per attacker"
    );
    assert_eq!(
        count_creatures_named(&state, "Gnome"),
        1,
        "exactly one Gnome token batch (X=1) must be created, not three"
    );
}

/// CR 508.1m: successive combats scale the token count with the live counter total —
/// first combat: 0 -> 1 counter, 1 token; second combat: 1 -> 2 counters, 2 tokens
/// created that combat (X reads the post-increment total each time). Both attacker
/// creatures are placed on the battlefield up front (a fresh mid-test object-add API
/// isn't needed); only one attacks in each wave.
#[test]
fn test_anim_pakal_token_count_scales_with_counters() {
    let (state, anim_pakal_id, attacker_ids) = build_anim_pakal_state(&[
        ("First Wave Attacker", false),
        ("Second Wave Attacker", false),
    ]);
    let first_attacker_id = attacker_ids[0];
    let second_attacker_id = attacker_ids[1];

    let (state, _) = declare_attack_all(state, p(1), p(2), &[first_attacker_id]);
    let (mut state, _) = drain_stack(state, &[p(1), p(2)]);
    assert_eq!(
        counters_on(&state, anim_pakal_id, CounterType::PlusOnePlusOne),
        1
    );
    assert_eq!(count_creatures_named(&state, "Gnome"), 1);

    // Simulate a second combat this test (not a full turn-cycle re-entry — the point
    // under test is CounterCount reading the live post-increment total, not turn
    // structure). The first attacker is now tapped from combat and does not attack
    // again; the second, still-untapped attacker declares this wave.
    *state.combat_mut() = None;
    state.turn_mut().step = Step::DeclareAttackers;
    state.turn_mut().priority_holder = Some(p(1));

    let (state, _) = declare_attack_all(state, p(1), p(2), &[second_attacker_id]);
    let (state, _) = drain_stack(state, &[p(1), p(2)]);

    assert_eq!(
        counters_on(&state, anim_pakal_id, CounterType::PlusOnePlusOne),
        2,
        "the second combat must add a second +1/+1 counter (running total 2)"
    );
    assert_eq!(
        count_creatures_named(&state, "Gnome"),
        3,
        "the second combat's token count (X) must read the post-increment ABSOLUTE \
         total of 2 -- i.e. X=2 new Gnomes are created that combat, on top of the 1 \
         from the first combat (running total 1 + 2 = 3), not a fixed +1 per combat"
    );
}

/// Decoy: the Gnome tokens created by a resolved trigger ENTER attacking rather than
/// being DECLARED as attackers (ruling 2023-11-10) — they must not re-fire Anim
/// Pakal's own trigger within the same combat (the counter must land on exactly 1,
/// not 2, even though a new Gnome token entered "attacking" during resolution).
#[test]
fn test_anim_pakal_created_gnomes_do_not_inflate_next_trigger() {
    let (state, anim_pakal_id, attacker_ids) =
        build_anim_pakal_state(&[("Solo Non-Gnome Attacker", false)]);

    let (state, _) = declare_attack_all(state, p(1), p(2), &attacker_ids);
    let (state, _) = drain_stack(state, &[p(1), p(2)]);

    assert_eq!(
        counters_on(&state, anim_pakal_id, CounterType::PlusOnePlusOne),
        1,
        "the created Gnome token entering attacking must NOT re-trigger the batch \
         attack trigger — the counter must land on exactly 1, not 2"
    );
    assert_eq!(
        count_creatures_named(&state, "Gnome"),
        1,
        "exactly one Gnome token total — no runaway recursive creation"
    );
}

// ── General Kreat, the Boltbringer ───────────────────────────────────────────────

fn build_general_kreat_state(attackers: &[(&str, bool)]) -> (GameState, Vec<ObjectId>) {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_defs_and_registry();
    let kreat_spec = enrich(
        p1,
        "General Kreat, the Boltbringer",
        ZoneId::Battlefield,
        &defs,
    );

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(kreat_spec);

    for &(name, is_goblin) in attackers {
        let mut spec = ObjectSpec::creature(p1, name, 1, 1);
        if is_goblin {
            spec = spec.with_subtypes(vec![SubType("Goblin".to_string())]);
        }
        builder = builder.object(spec);
    }

    let mut state = builder
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let attacker_ids: Vec<ObjectId> = attackers
        .iter()
        .map(|(name, _)| find_by_name(&state, name))
        .collect();
    (state, attacker_ids)
}

/// CR 508.1m: attacking with a Goblin you control fires General Kreat's batch trigger
/// once, creating one tapped-and-attacking 1/1 red Goblin token.
#[test]
fn test_general_kreat_goblin_attack_fires_once() {
    let (state, attacker_ids) = build_general_kreat_state(&[("Goblin Raider", true)]);
    let before = count_creatures_named(&state, "Goblin");

    let (state, _) = declare_attack_all(state, p(1), p(2), &attacker_ids);
    let (state, _) = drain_stack(state, &[p(1), p(2)]);

    assert_eq!(
        count_creatures_named(&state, "Goblin"),
        before + 1,
        "attacking with a Goblin must create exactly one Goblin token"
    );
}

/// Negative: attacking with a NON-Goblin creature must NOT fire General Kreat's
/// batch trigger — no Goblin token created.
#[test]
fn test_general_kreat_no_goblin_attack_does_not_fire() {
    let (state, attacker_ids) = build_general_kreat_state(&[("Non-Goblin Attacker", false)]);
    let before = count_creatures_named(&state, "Goblin");

    let (state, _) = declare_attack_all(state, p(1), p(2), &attacker_ids);
    let (state, _) = drain_stack(state, &[p(1), p(2)]);

    assert_eq!(
        count_creatures_named(&state, "Goblin"),
        before,
        "a non-Goblin-only attack must NOT create a Goblin token"
    );
}

// ── Hermes, Overseer of Elpis ─────────────────────────────────────────────────────

fn build_hermes_state(attackers: &[(&str, bool)]) -> (GameState, Vec<ObjectId>) {
    let p1 = p(1);
    let p2 = p(2);
    let (defs, registry) = build_defs_and_registry();
    let hermes_spec = enrich(p1, "Hermes, Overseer of Elpis", ZoneId::Battlefield, &defs);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(hermes_spec);

    for &(name, is_bird) in attackers {
        let mut spec = ObjectSpec::creature(p1, name, 1, 1);
        if is_bird {
            spec = spec.with_subtypes(vec![SubType("Bird".to_string())]);
        }
        builder = builder.object(spec);
    }

    // Give p1's library a couple of cards so Scry 2 has something to look at (not
    // strictly required for the Scried event count, but keeps state realistic).
    let mut state = builder
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let attacker_ids: Vec<ObjectId> = attackers
        .iter()
        .map(|(name, _)| find_by_name(&state, name))
        .collect();
    (state, attacker_ids)
}

/// CR 508.1m: attacking with a Bird you control fires Hermes's batch trigger once,
/// scrying 2 (a `GameEvent::Scried{player, count: 2}` is emitted exactly once).
#[test]
fn test_hermes_bird_attack_scry_2() {
    let (state, attacker_ids) = build_hermes_state(&[("Bird Scout", true)]);

    let (state, _) = declare_attack_all(state, p(1), p(2), &attacker_ids);
    let (_state, events) = drain_stack(state, &[p(1), p(2)]);

    let scry_count = events
        .iter()
        .filter(
            |e| matches!(e, GameEvent::Scried { player, count } if *player == p(1) && *count == 2),
        )
        .count();
    assert_eq!(
        scry_count, 1,
        "attacking with a Bird must scry 2 exactly once (batch trigger, not per-attacker)"
    );
}

/// Negative: attacking with a NON-Bird creature must NOT fire Hermes's batch
/// trigger — no `Scried` event emitted.
#[test]
fn test_hermes_no_bird_attack_no_scry() {
    let (state, attacker_ids) = build_hermes_state(&[("Non-Bird Attacker", false)]);

    let (state, _) = declare_attack_all(state, p(1), p(2), &attacker_ids);
    let (_state, events) = drain_stack(state, &[p(1), p(2)]);

    assert!(
        !events.iter().any(|e| matches!(e, GameEvent::Scried { .. })),
        "a non-Bird-only attack must NOT scry"
    );
}

// ── Regression: legacy `{ filter: None }` unfiltered attack fires on any attack ──

/// CR 508.1 regression guard: a card whose trigger was migrated to
/// `WheneverYouAttack {{ filter: None }}` (the 5 legacy cards: legions_landing,
/// caesar_legions_emperor, mishra_claimed_by_gix, chivalric_alliance,
/// seasoned_dungeoneer) must still fire on ANY attack, unfiltered — pinned here via
/// a synthetic `TriggeredAbilityDef` to avoid depending on any one legacy card's full
/// implementation.
#[test]
fn test_you_attack_filter_none_fires_on_any_attack() {
    let p1 = p(1);
    let p2 = p(2);
    let watcher = ObjectSpec::creature(p1, "Unfiltered Attack Watcher", 0, 0)
        .with_triggered_ability(mtg_engine::TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            trigger_on: mtg_engine::TriggerEvent::ControllerAttacks,
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            intervening_if: None,
            description: "Whenever you attack, gain 1 life. (CR 508.1)".to_string(),
            effect: Some(mtg_engine::cards::card_definition::Effect::GainLife {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                amount: mtg_engine::cards::card_definition::EffectAmount::Fixed(1),
            }),
            triggering_creature_filter: None,
            targets: vec![],
        });
    let attacker = ObjectSpec::creature(p1, "Any Attacker", 1, 1)
        .with_subtypes(vec![SubType("Gnome".to_string())]); // subtype must not matter with filter: None

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(watcher)
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let initial_life = state.player(p1).unwrap().life_total;
    let attacker_id = find_by_name(&state, "Any Attacker");

    let (state, _) = declare_attack_all(state, p1, p2, &[attacker_id]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert_eq!(
        state.player(p1).unwrap().life_total,
        initial_life + 1,
        "an unfiltered WheneverYouAttack{{filter: None}} must fire on any attack, \
         regardless of the attacker's subtype"
    );
}
