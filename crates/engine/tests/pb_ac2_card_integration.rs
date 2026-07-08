//! PB-AC2 review finding MEDIUM #4: real-card end-to-end integration tests for
//! `Effect::MayPayThenEffect` (CR 118.12) and `Effect::CounterUnlessPays` (CR 118.12a).
//!
//! `crates/engine/tests/optional_cost_and_counter_tax.rs` exercises both primitives
//! only against synthetic effects built by hand. These five tests drive the REAL
//! `CardDefinition`s that shipped in the PB-AC2 backfill (`crossway_troublemakers`,
//! `hazorets_monument`, `springbloom_druid`, `nadir_kraken`, `mana_leak`) through the
//! engine's normal cast / trigger / resolution pipeline (`Command::CastSpell`,
//! `process_command`, SBAs, `check_triggers` / `flush_pending_triggers`), using
//! `enrich_spec_from_def` to populate each object's characteristics and triggered
//! abilities from its registered `CardDefinition` (never a hand-built `Effect`).
//!
//! CR Rules covered:
//! - CR 118.12: beneficial optional cost is paid (or not) at resolution.
//! - CR 118.12a: "counter unless pays" tax semantics (deterministic non-interactive
//!   decline path per PB-AC2's design — see `memory/primitives/pb-plan-AC2.md`).
//! - CR 119.4: paying life.
//! - CR 701.21a: sacrifice is not destruction; dies triggers still fire.
//! - CR 603.2 / 603.3: triggered abilities fire on cast / draw / death and queue to
//!   the stack via `check_and_flush_triggers`.
//! - CR 603.10a: "whenever a creature dies" is a look-back-in-time trigger.
//!
//! Hazoret's Monument regression: prior to PB-AC2 this ability was miscoded as an
//! *unconditional* draw. The real fix wraps it in `MayPayThenEffect{DiscardCard ->
//! DrawCards}`; the assertions below fail if the draw ever happens without a paired
//! discard.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, CardDefinition, CardId, CardRegistry,
    CardType, Command, CounterType, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, StackObjectKind, Step,
    SubType, Target, TypeLine, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_in_hand(state: &GameState, player: PlayerId, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, o)| o.characteristics.name == name && o.zone == ZoneId::Hand(player))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("card '{}' not found in {}'s hand", name, player.0))
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| o.zone == ZoneId::Hand(player))
        .count()
}

fn graveyard_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Graveyard(pid) if pid == player))
        .count()
}

fn trigger_count_for(state: &GameState, source: ObjectId) -> usize {
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

/// Pass priority once for every player in the list (resolves top stack item or
/// advances the turn). Mirrors the pattern used across the PB-AC0/AC1 card
/// integration test suites.
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

/// Cast a spell by name from `player`'s hand. The caller is responsible for
/// pre-floating any mana the cost requires.
fn cast_spell(
    state: GameState,
    player: PlayerId,
    name: &str,
    targets: Vec<Target>,
) -> (GameState, Vec<GameEvent>) {
    let card_id = find_in_hand(&state, player, name);
    let mut state = state;
    state.turn.priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell {
            player,
            card: card_id,
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell({}) failed: {:?}", name, e))
}

/// Push a bare `StackObject::Spell` entry wrapping `source_object` onto
/// `state.stack_objects`. Used for the opposing spell that Mana Leak targets --
/// mirrors the pattern in `optional_cost_and_counter_tax.rs`.
fn push_spell_stack_object(
    state: &mut GameState,
    source_object: ObjectId,
    controller: PlayerId,
) -> ObjectId {
    let stack_id = state.next_object_id();
    state.stack_objects.push_back(mtg_engine::StackObject {
        id: stack_id,
        controller,
        kind: StackObjectKind::Spell { source_object },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback: false,
        kicker_times_paid: 0,
        was_evoked: false,
        was_bestowed: false,
        cast_with_madness: false,
        cast_with_miracle: false,
        was_escaped: false,
        cast_with_foretell: false,
        was_buyback_paid: false,
        was_suspended: false,
        was_overloaded: false,
        cast_with_jump_start: false,
        cast_with_aftermath: false,
        was_dashed: false,
        was_blitzed: false,
        was_plotted: false,
        was_prototyped: false,
        was_impended: false,
        was_bargained: false,
        was_surged: false,
        was_casualty_paid: false,
        was_cleaved: false,
        was_cast_as_adventure: false,
        x_value: 0,
        evidence_collected: false,
        spliced_effects: vec![],
        spliced_card_ids: vec![],
        modes_chosen: vec![],
        is_cast_transformed: false,
        additional_costs: vec![],
        damaged_player: None,
        combat_damage_amount: 0,
        triggering_creature_id: None,
        cast_from_top_with_bonus: false,
        sacrificed_creature_powers: vec![],
        lki_counters: im::OrdMap::new(),
        lki_power: None,
    });
    stack_id
}

/// A minimal vanilla creature `CardDefinition`, used as the spell a real
/// cast-triggered ability (Hazoret's Monument) reacts to.
fn vanilla_creature_def(card_id: &str, name: &str, generic: u32) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic,
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

// ── 1. Crossway Troublemakers -- PayLife on a real death trigger ───────────────

/// CR 118.12 / 119.4 / 603.10a: Crossway Troublemakers, "Whenever a Vampire you
/// control dies, you may pay 2 life. If you do, draw a card." The real card
/// definition (not a synthetic effect) is placed on the battlefield via
/// `enrich_spec_from_def`; a second Vampire dies to the 0-toughness SBA (CR
/// 704.5f), which queues Crossway's death trigger; resolving it pays 2 life and
/// draws a card.
#[test]
fn test_crossway_troublemakers_vampire_death_may_pay_life_draws() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let crossway_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Crossway Troublemakers")
            .with_card_id(CardId("crossway-troublemakers".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    // A disposable Vampire that dies immediately to the 0-toughness SBA.
    let vampire_fodder = ObjectSpec::creature(p1, "Vampire Fodder", 2, 0)
        .with_subtypes(vec![SubType("Vampire".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(crossway_spec)
        .object(vampire_fodder)
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.players.get_mut(&p1).unwrap().life_total = 20;

    let crossway_id = find_by_name(&state, "Crossway Troublemakers");
    let initial_hand = hand_count(&state, p1);

    // SBA kills Vampire Fodder; the death trigger is queued/put on the stack.
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        !state
            .objects
            .values()
            .any(|o| o.characteristics.name == "Vampire Fodder" && o.zone == ZoneId::Battlefield),
        "CR 704.5f: Vampire Fodder (0 toughness) should have died to the SBA"
    );
    assert!(
        trigger_count_for(&state, crossway_id) > 0,
        "CR 603.10a: Crossway's death trigger should fire when a Vampire you control dies"
    );

    // Resolve the trigger: MayPayThenEffect{PayLife(2) -> DrawCards(1)}.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::LifeLost { player, amount } if *player == p1 && *amount == 2)
        ),
        "CR 119.4: paying 2 life should emit LifeLost; events: {:?}",
        events
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 118.12: cost was paid -- `then` (draw a card) should run"
    );
    assert_eq!(
        state.players.get(&p1).unwrap().life_total,
        18,
        "life total should be reduced by the paid 2 life"
    );
    assert_eq!(
        hand_count(&state, p1),
        initial_hand + 1,
        "CR 118.12: exactly 1 card should have been drawn"
    );
}

// ── 2. Hazoret's Monument -- DiscardCard on a real cast trigger (regression) ────

/// CR 118.12 regression: Hazoret's Monument, "Whenever you cast a creature spell,
/// you may discard a card. If you do, draw a card." Prior to PB-AC2 this was
/// miscoded as an *unconditional* draw. Driven end-to-end via `Command::CastSpell`:
/// casting a creature immediately queues Hazoret's trigger (CR 603.3, flushed by
/// `check_and_flush_triggers` inside the CastSpell command handler); resolving it
/// must discard exactly one card AND draw exactly one card -- never draw alone.
#[test]
fn test_hazorets_monument_creature_cast_may_discard_draws() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let hazoret_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Hazoret's Monument")
            .with_card_id(CardId("hazorets-monument".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let bear_def = vanilla_creature_def("hazoret-test-bear", "Hazoret Test Bear", 2);
    let mut cards = all_cards();
    cards.push(bear_def);
    let registry = CardRegistry::new(cards);

    let bear_in_hand = ObjectSpec::creature(p1, "Hazoret Test Bear", 2, 2)
        .with_card_id(CardId("hazoret-test-bear".to_string()))
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(hazoret_spec)
        .object(bear_in_hand)
        .object(ObjectSpec::card(p1, "Filler Card A").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Filler Card B").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    let hazoret_id = find_by_name(&state, "Hazoret's Monument");
    let hand_after_bear_in_hand = hand_count(&state, p1); // 3: Bear, Filler A, Filler B

    // Cast the creature -- CastSpell flushes triggers immediately (CR 603.3), so
    // Hazoret's "whenever you cast a creature spell" trigger is already queued.
    let (state, _) = cast_spell(state, p1, "Hazoret Test Bear", vec![]);
    assert_eq!(
        hand_count(&state, p1),
        hand_after_bear_in_hand - 1,
        "casting the Bear should remove it from hand"
    );
    assert!(
        trigger_count_for(&state, hazoret_id) > 0,
        "CR 603.2/603.3: Hazoret's cast trigger should be queued immediately after casting \
         a creature spell"
    );

    // Resolve Hazoret's trigger: MayPayThenEffect{DiscardCard -> DrawCards(1)}.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { player, .. } if *player == p1)),
        "CR 118.12: cost was paid -- a card should be discarded. Regression: the \
         pre-PB-AC2 bug drew unconditionally with NO discard. events: {:?}",
        events
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 118.12: cost was paid -- `then` (draw a card) should run"
    );
    assert_eq!(
        graveyard_count(&state, p1),
        1,
        "exactly one discarded card should be in the graveyard"
    );
    // Net hand change from the pre-cast baseline: -1 (Bear cast) -1 (discard) +1
    // (draw) = -1. A wrong unconditional-draw implementation would net 0.
    assert_eq!(
        hand_count(&state, p1),
        hand_after_bear_in_hand - 1,
        "CR 118.12 regression guard: hand count must reflect discard+draw \
         (net -1 from the pre-cast baseline), NOT an unconditional draw (which \
         would net 0)"
    );
}

// ── 3. Springbloom Druid -- Sacrifice(land) on a real ETB trigger ───────────────

/// CR 118.12 / 701.21a: Springbloom Druid, "When this creature enters, you may
/// sacrifice a land. If you do, search your library for up to two basic land
/// cards, put them onto the battlefield tapped, then shuffle." Cast from hand so
/// the real `WhenEntersBattlefield` CardDef trigger path (`queue_carddef_etb_triggers`,
/// which requires the object's `card_id` + the state's `CardRegistry`) fires for
/// real, not a hand-wired runtime trigger.
#[test]
fn test_springbloom_druid_etb_may_sacrifice_land_searches() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let druid_in_hand = enrich_spec_from_def(
        ObjectSpec::card(p1, "Springbloom Druid")
            .with_card_id(CardId("springbloom-druid".to_string()))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );

    let land_to_sac = ObjectSpec::card(p1, "Sacrificial Land")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Battlefield);

    // Two distinct real basic lands in the library (CR 205.4a: basic supertype).
    let forest_in_lib = enrich_spec_from_def(
        ObjectSpec::card(p1, "Forest").in_zone(ZoneId::Library(p1)),
        &defs,
    );
    let plains_in_lib = enrich_spec_from_def(
        ObjectSpec::card(p1, "Plains").in_zone(ZoneId::Library(p1)),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(druid_in_hand)
        .object(land_to_sac)
        .object(forest_in_lib)
        .object(plains_in_lib)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
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
        .add(ManaColor::Colorless, 2);

    // Cast Springbloom Druid; resolve it onto the battlefield (queues the real
    // WhenEntersBattlefield CardDef trigger via queue_carddef_etb_triggers).
    let (state, _) = cast_spell(state, p1, "Springbloom Druid", vec![]);
    let (state, _) = pass_all(state, &[p1, p2]);
    assert!(
        state
            .objects
            .values()
            .any(|o| o.characteristics.name == "Springbloom Druid" && o.zone == ZoneId::Battlefield),
        "Springbloom Druid should have resolved onto the battlefield"
    );
    let druid_id = find_by_name(&state, "Springbloom Druid");
    assert!(
        trigger_count_for(&state, druid_id) > 0,
        "CR 603.3: Springbloom Druid's ETB trigger should be queued after it enters"
    );

    // Resolve the ETB trigger: MayPayThenEffect{Sacrifice(land) -> Sequence[Search,
    // Search, Shuffle]}.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentSacrificed { player, .. } if *player == p1)),
        "CR 701.21a: the sacrifice-as-cost should fire PermanentSacrificed; events: {:?}",
        events
    );
    assert!(
        !state
            .objects
            .values()
            .any(|o| o.characteristics.name == "Sacrificial Land" && o.zone == ZoneId::Battlefield),
        "the sacrificed land should have left the battlefield"
    );

    let forest_on_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Forest" && o.zone == ZoneId::Battlefield);
    let plains_on_bf = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Plains" && o.zone == ZoneId::Battlefield);
    assert!(
        forest_on_bf.is_some(),
        "CR 118.12: cost was paid -- the Forest should have been searched onto the battlefield"
    );
    assert!(
        plains_on_bf.is_some(),
        "CR 118.12: cost was paid -- the Plains should have been searched onto the battlefield"
    );
    assert!(
        forest_on_bf.unwrap().status.tapped,
        "the searched Forest should enter tapped"
    );
    assert!(
        plains_on_bf.unwrap().status.tapped,
        "the searched Plains should enter tapped"
    );
    assert!(
        !state
            .objects
            .values()
            .any(|o| o.zone == ZoneId::Library(p1)),
        "both basics should have left the library"
    );
}

// ── 4. Nadir Kraken -- Mana{1} (pre-floated) on a real draw trigger ─────────────

/// CR 118.12 / 118.8 / 500.4: Nadir Kraken, "Whenever you draw a card, you may pay
/// {1}. If you do, put a +1/+1 counter on Nadir Kraken and create a 1/1 blue
/// Tentacle creature token." The real card definition's `WheneverYouDrawACard`
/// trigger is wired via `enrich_spec_from_def`; a real `Effect::DrawCards` fires
/// the trigger (mirroring how a draw step/effect would), and {1} is pre-floated
/// since mana pools empty between steps (CR 500.4).
#[test]
fn test_nadir_kraken_on_draw_may_pay_puts_counter_and_token() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let kraken_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Nadir Kraken")
            .with_card_id(CardId("nadir-kraken".to_string()))
            .in_zone(ZoneId::Battlefield),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(kraken_spec)
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    // Pre-float {1} -- mana pools are empty between steps (CR 500.4); the
    // beneficial pay only fires if the payer has floating mana at trigger time.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let kraken_id = find_by_name(&state, "Nadir Kraken");

    // Draw a card via a real Effect::DrawCards (there is no top-level Command for
    // "draw a card" -- draws are always effect-driven in this engine).
    let mut ctx = EffectContext::new(p1, kraken_id, vec![]);
    let draw_events = execute_effect(
        &mut state,
        &Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
        &mut ctx,
    );
    assert!(
        draw_events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "sanity: the draw should have happened"
    );
    let triggers = check_triggers(&state, &draw_events);
    for t in triggers {
        state.pending_triggers.push_back(t);
    }
    let flush_events = flush_pending_triggers(&mut state);
    assert!(
        flush_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 603.2: Nadir Kraken's draw trigger should fire when its controller draws"
    );

    // Resolve the trigger from the stack.
    while !state.stack_objects.is_empty() {
        let (s, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
        let (s, _) = process_command(s, Command::PassPriority { player: p2 }).unwrap();
        state = s;
    }

    let kraken = state.objects.get(&kraken_id).unwrap();
    assert_eq!(
        kraken
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "CR 118.12: {{1}} was floating and paid -- Nadir Kraken should have a +1/+1 counter"
    );
    let tentacle_count = state
        .objects
        .values()
        .filter(|o| o.characteristics.name == "Tentacle" && o.zone == ZoneId::Battlefield)
        .count();
    assert_eq!(
        tentacle_count, 1,
        "CR 118.12: {{1}} was floating and paid -- exactly 1 Tentacle token should be created"
    );
    assert_eq!(
        state.players.get(&p1).unwrap().mana_pool.colorless,
        0,
        "the floating mana should have been spent"
    );
}

// ── 5. Mana Leak -- CounterUnlessPays end-to-end ────────────────────────────────

/// CR 118.12a / 701.5: Mana Leak, "Counter target spell unless its controller pays
/// {3}." Driven end-to-end via `Command::CastSpell`: the real card definition is
/// cast targeting an opposing spell on the stack; the target's controller has no
/// way to pay {3} (empty mana pool), so the deterministic non-interactive decline
/// path counters it (delegating to `Effect::CounterSpell`).
#[test]
fn test_mana_leak_counters_target_spell() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let target_spell = ObjectSpec::card(p2, "Countable Spell")
        .in_zone(ZoneId::Stack)
        .with_types(vec![CardType::Instant]);

    let mana_leak_in_hand = enrich_spec_from_def(
        ObjectSpec::card(p1, "Mana Leak")
            .with_card_id(CardId("mana-leak".to_string()))
            .in_zone(ZoneId::Hand(p1)),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(target_spell)
        .object(mana_leak_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let target_id = find_by_name(&state, "Countable Spell");
    push_spell_stack_object(&mut state, target_id, p2);

    // p1 has exactly {1}{U} -- enough to cast Mana Leak, nothing left over. p2 (the
    // target's controller) has NO floating mana at all -- no way to pay {3}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let (state, _) = cast_spell(state, p1, "Mana Leak", vec![Target::Object(target_id)]);
    assert!(
        state
            .stack_objects
            .iter()
            .any(|so| matches!(so.kind, StackObjectKind::Spell { source_object } if source_object != target_id)),
        "Mana Leak should be on the stack above the target"
    );

    // Resolve Mana Leak off the stack.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "CR 118.12a: Mana Leak should counter the target spell (deterministic decline \
         path -- the controller has no floating mana to pay {{3}}); events: {:?}",
        events
    );
    assert!(
        state.objects.values().any(|o| {
            o.characteristics.name == "Countable Spell" && matches!(o.zone, ZoneId::Graveyard(_))
        }),
        "the countered spell should move to its owner's graveyard"
    );
    assert!(
        !state
            .stack_objects
            .iter()
            .any(|so| matches!(so.kind, StackObjectKind::Spell { source_object } if source_object == target_id)),
        "the countered spell's stack entry should be gone"
    );
}
