//! Tests for PB-EWC: `ReplacementModification::EntersWithCounters` with
//! `count: EffectAmount` (CR 614.1c).
//!
//! This primitive unblocks "enters with N where N is dynamic" patterns:
//!   - **Master Biomancer** — non-self ETB replacement on each other creature
//!     the controller controls, with count = the source's live, layer-resolved
//!     power. Ruling 2013-01-24: "use Master Biomancer's power as that creature
//!     is entering."
//!   - **Ingenious Prodigy** — self-ETB replacement, count = X paid at cast
//!     time. Replaces the previous triggered-ETB stub that placed counters
//!     AFTER the permanent was on the battlefield (CR 614.1c DEVIATION).
//!
//! Implementation: the resolver builds an `EffectContext` pinned to the
//! replacement source (read from `ReplacementEffect.source` for global
//! replacements; `new_id` for self-ETB) and calls `resolve_amount`. The
//! source is alive on the battlefield when its replacement fires for
//! another creature, so `EffectAmount::PowerOf(EffectTarget::Source)`
//! resolves via the live arm of `resolve_amount` (layer-resolved P/T).
//!
//! Tests:
//!   (a) Master Biomancer — Elvish Mystic enters with +1/+1 counters equal
//!       to MB's printed power (2/4 → 2 counters).
//!   (b) Master Biomancer — pumping MB by +1/+1 counter (Layer 7d) changes
//!       the subsequent entry's counter count: 4/6 MB → next entry gets 4.
//!   (c) Ingenious Prodigy — cast with X=5 enters with 5 +1/+1 counters
//!       via the replacement (counters present immediately, no trigger).
//!   (d) Hash schema sentinel: `HASH_SCHEMA_VERSION == 18` (PB-EWC bump).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::types::CounterType;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, GameEvent, GameState, GameStateBuilder, ManaColor, ObjectId, ObjectSpec,
    PlayerId, Step, HASH_SCHEMA_VERSION,
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

/// Cast a non-X creature spell from hand, paying its full mana cost.
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

/// Cast an X-cost spell from hand, paying base cost + X * x_count generic.
fn cast_x_spell(
    mut state: GameState,
    caster: PlayerId,
    card: ObjectId,
    mana: &[(ManaColor, u32)],
    x_value: u32,
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
            x_value,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell(x_value={}) failed: {:?}", x_value, e))
}

// ── Test (a): Master Biomancer counters from live power ───────────────────────

/// CR 614.1c / Ruling 2013-01-24 — Master Biomancer (2/4) is on the battlefield.
/// When Elvish Mystic (1/1) enters under the same controller, it must enter with
/// 2 +1/+1 counters: count = MB's live power at ETB time.
#[test]
fn test_master_biomancer_counter_from_live_power_base() {
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

    // Cast Master Biomancer for {2}{G}{U} — goes through full ETB pipeline
    // (apply_self_etb → apply_etb_replacements → register_permanent_replacement_abilities).
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
    // Sanity: MB's global ETB replacement is registered exactly once, with the
    // filter bound to p1 (verifies bind_object_filter ran for WouldEnterBattlefield).
    let mb_repl_count: usize = state
        .replacement_effects
        .iter()
        .filter(|e| {
            matches!(
                &e.trigger,
                mtg_engine::ReplacementTrigger::WouldEnterBattlefield {
                    filter: mtg_engine::ObjectFilter::CreatureControlledBy(pid)
                } if *pid == p1
            )
        })
        .count();
    assert_eq!(
        mb_repl_count, 1,
        "MB's EntersWithCounters replacement should be registered with the \
         CreatureControlledBy filter rebound to p1 (placeholder PlayerId(0) \
         must NOT survive registration)"
    );

    let mystic_id = find_by_name(&state, "Elvish Mystic");

    // Cast Elvish Mystic for {G}. Resolves into a permanent under p1's control.
    let (state, _) = cast_creature(state, p1, mystic_id, &[(ManaColor::Green, 1)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_mystic =
        find_on_battlefield(&state, "Elvish Mystic").expect("Elvish Mystic must be on battlefield");
    let counter_count = state
        .objects
        .get(&bf_mystic)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 2,
        "CR 614.1c / Ruling 2013-01-24: Elvish Mystic must enter with +1/+1 \
         counters equal to Master Biomancer's live power (2). Got {}.",
        counter_count
    );
}

// ── Test (b): Master Biomancer counters track live, layer-resolved power ─────

/// CR 614.12 — replacement effects modifying ETB check the source's
/// characteristics "as it would exist on the battlefield" at the time the
/// replacement fires. Pumping Master Biomancer to 4/6 (via 2 +1/+1 counters,
/// Layer 7d) must affect the count on the *next* entering creature.
#[test]
fn test_master_biomancer_counter_tracks_pumped_power() {
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
    let (mut state, _) = pass_all(state, &[p1, p2]);

    // Pump Master Biomancer to 4/6 via 2 +1/+1 counters (Layer 7d, CR 613.1g).
    let mb_bf = find_on_battlefield(&state, "Master Biomancer")
        .expect("Master Biomancer must be on battlefield after casting");
    state
        .objects
        .get_mut(&mb_bf)
        .unwrap()
        .counters
        .insert(CounterType::PlusOnePlusOne, 2);

    let mystic_id = find_by_name(&state, "Elvish Mystic");
    let (state, _) = cast_creature(state, p1, mystic_id, &[(ManaColor::Green, 1)]);
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_mystic =
        find_on_battlefield(&state, "Elvish Mystic").expect("Elvish Mystic must be on battlefield");
    let counter_count = state
        .objects
        .get(&bf_mystic)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 4,
        "CR 614.12 / CR 613.1g: pumped MB (2 base + 2 counters = 4) must give \
         Elvish Mystic 4 +1/+1 counters via the LIVE power resolver \
         (calculate_characteristics layer 7d). Got {}.",
        counter_count
    );
}

// ── Test (c): Ingenious Prodigy enters with X +1/+1 counters via replacement ──

/// CR 107.3m / CR 614.1c — Ingenious Prodigy cast with X=5 must enter with
/// exactly 5 +1/+1 counters. After PB-EWC the counters are placed by the
/// replacement effect (no stack trigger), so they are present the moment the
/// permanent is on the battlefield.
#[test]
fn test_ingenious_prodigy_x_value_replacement_counts() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let prodigy_spec = enrich(p1, "Ingenious Prodigy", ZoneId::Hand(p1), &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(prodigy_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let prodigy_id = find_by_name(&state, "Ingenious Prodigy");

    // Cast {X}{U} with X=5: pay 5 generic + 1 blue.
    let (state, _) = cast_x_spell(
        state,
        p1,
        prodigy_id,
        &[(ManaColor::Colorless, 5), (ManaColor::Blue, 1)],
        5,
    );
    // Resolve the spell (creature enters battlefield).
    let (state, _) = pass_all(state, &[p1, p2]);

    let bf_prodigy = find_on_battlefield(&state, "Ingenious Prodigy")
        .expect("Ingenious Prodigy must be on battlefield");
    let counter_count = state
        .objects
        .get(&bf_prodigy)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0);
    assert_eq!(
        counter_count, 5,
        "CR 107.3m / CR 614.1c: Ingenious Prodigy cast with X=5 must enter \
         with 5 +1/+1 counters via the EntersWithCounters replacement. Got {}.",
        counter_count
    );
}

// ── Test (d): Hash schema sentinel ────────────────────────────────────────────

/// PB-EWC bumped `HASH_SCHEMA_VERSION` from 17 to 18 to cover the
/// `ReplacementModification::EntersWithCounters { count: EffectAmount }`
/// wire-format change. Pre-PB-EWC saved states (count: u32) are not
/// forward-compatible.
#[test]
fn test_pb_ewc_hash_schema_version_is_18() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 20u8,
        "PB-XS-E bumped HASH_SCHEMA_VERSION 19→20 (TriggerCondition::Whenever{{Creature,Permanent}}EntersBattlefield.exclude_self, CR 109.1 / 603.2). If you bumped again, update this test and state/hash.rs history."
    );
}

// ── Test (e): Ingenious Prodigy with X=0 ─────────────────────────────────────

/// CR 107.3m edge: casting Ingenious Prodigy with X=0 (pay only {U}) must
/// produce a 0/1 creature with no counters. Verifies the replacement-resolver
/// correctly produces 0 counters (no CounterAdded event) without panicking.
#[test]
fn test_ingenious_prodigy_x_zero_no_counters() {
    let p1 = p(1);
    let p2 = p(2);

    let (defs, registry) = build_defs_and_registry();

    let prodigy_spec = enrich(p1, "Ingenious Prodigy", ZoneId::Hand(p1), &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(prodigy_spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let prodigy_id = find_by_name(&state, "Ingenious Prodigy");

    // Cast {X}{U} with X=0: pay only 1 blue.
    let (state, _) = cast_x_spell(state, p1, prodigy_id, &[(ManaColor::Blue, 1)], 0);
    let (state, events) = pass_all(state, &[p1, p2]);

    let bf_prodigy = find_on_battlefield(&state, "Ingenious Prodigy")
        .expect("Ingenious Prodigy must be on battlefield");
    let prodigy_obj = state.objects.get(&bf_prodigy).unwrap();
    // Discriminating absence check: the counters OrdMap must not even contain
    // the PlusOnePlusOne key. A regression that did
    // `obj.counters.insert(PlusOnePlusOne, 0)` would be wrong game state
    // (CR 122 / 704.5d: a permanent with 0 of a counter type is treated as
    // having no counters of that type, but the engine must not retain dead
    // entries either).
    assert!(
        !prodigy_obj
            .counters
            .contains_key(&CounterType::PlusOnePlusOne),
        "X=0 must not insert a (PlusOnePlusOne, 0) entry into the counters map; \
         counters: {:?}",
        prodigy_obj.counters
    );
    let counter_count = prodigy_obj
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counter_count, 0,
        "X=0 must place zero counters; got {}.",
        counter_count
    );

    let counter_added_zero = events.iter().any(|e| {
        matches!(
            e,
            GameEvent::CounterAdded {
                counter: CounterType::PlusOnePlusOne,
                count: 0,
                ..
            }
        )
    });
    assert!(
        !counter_added_zero,
        "Resolver must suppress the CounterAdded event when count resolves to 0 \
         (matches the > 0 guard in emit_etb_modification)."
    );
}
