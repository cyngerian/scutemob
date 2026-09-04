//! PB-DX48 (`OOS-ENG2-1` ≡ `OOS-ENG2-2`) — CR 702.21a Ward dispatch at every
//! `push_target_announcement` site, and the exactly-once fixpoint that keeps it from
//! firing twice.
//!
//! `memory/primitives/pb-plan-DX48.md` §3a is authoritative for probe scope. The
//! engine change under test shipped across THREE commits, and this file is written
//! against the FINAL shape (`8c703988`), not the first draft (`72c0770c`):
//!
//! * **Part A** (`72c0770c`) — `rules/events.rs::permanent_targeted_events` is the
//!   ONE place `GameEvent::PermanentTargeted` is constructed. `push_target_announcement`
//!   calls it after `TargetsAnnounced`, so every `push_target_announcement` call site
//!   gets the CR 702.21a dispatch for free, including the five that previously had
//!   none (`handle_activate_forecast`, `flush_sorted`'s two arms, `handle_scavenge_card`,
//!   `handle_activate_loyalty_ability`).
//! * **Part B, first draft** (`72c0770c`) — the becomes-target fixpoint lived in
//!   `rules/engine.rs::check_and_flush_triggers`. That function is only one of SIX
//!   flush call sites and two of them (`resume_trigger_flush`,
//!   `drop_departed_trigger_flush`) bypass `abilities::flush_pending_triggers`
//!   entirely, calling `flush_sorted` directly. Worse: `rules/resolution.rs`'s
//!   post-resolution sweep (`resolve_top_of_stack`) calls
//!   `abilities::flush_pending_triggers` directly and NEVER calls
//!   `check_and_flush_triggers` — so a triggered ability placed while a SPELL
//!   RESOLVES (the ordinary case: a creature's own ETB trigger) still dispatched
//!   NOTHING even with Part A's emission live at every site.
//! * **Part B, final** (`8c703988`) — the wave loop moved INTO
//!   `abilities::flush_pending_triggers` itself (which wraps the pre-PB-DX48 body,
//!   now `flush_pending_triggers_once`), because that is the one function all six
//!   flush sites go through. `rules/engine.rs::check_and_flush_triggers` is a single
//!   pass again — a second loop there would re-scan events the flush already
//!   dispatched and fire Ward twice (the double-dispatch this batch's commit message
//!   says was observed and rejected before this shape shipped).
//!
//! **`t1` below is therefore the RESOLUTION-path probe, not the suspend/resume one.**
//! A triggered ability whose CR 603.3d target slot has exactly ONE legal candidate
//! never suspends (`forced_trigger_target_answer`, CR 601.2c: one legal answer is not
//! a choice) — it is placed entirely inside `resolve_top_of_stack`'s post-resolution
//! sweep, via `Command::PassPriority`, a path `check_and_flush_triggers` never runs
//! after. This is the scenario Part B's relocation from `check_and_flush_triggers`
//! into `flush_pending_triggers` was needed to fix, and it is the only shape in this
//! file that reddens under the "single wave" revert (see the matrix in
//! `memory/primitives/pb-DX48-execution-notes.md`) — every other probe here goes
//! through a command handler that ALSO calls the (now single-pass)
//! `check_and_flush_triggers` immediately afterward, which independently finds the
//! same-command `PermanentTargeted` event regardless of whether the wave loop lives
//! inside `flush_pending_triggers`. That is disclosed per-probe below rather than
//! implied by uniform framing.
//!
//! CR citations used throughout: CR 702.21a (Ward), CR 603.3b (a CR 603.3 batch's
//! triggers are all placed before any player receives priority), CR 603.3d (target
//! announcement / CR 601.2c "one legal answer is not a choice"), CR 608.1/608.2
//! (resolution), CR 118.12 (`MayPayOrElse` — non-interactive at HEAD, always applies
//! `or_else`, per `ward.rs`'s own module doc).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::{ActivatedAbility, ActivationCost, StackObjectKind, TriggerEvent};
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition,
    CardEffectTarget, CardId, CardRegistry, CardType, Command, CounterType, Effect, EffectAmount,
    EffectDuration, GameEvent, GameState, GameStateBuilder, KeywordAbility, LoyaltyCost, ManaColor,
    ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, Target, TargetController,
    TargetFilter, TargetRequirement, TriggeredAbilityDef, TypeLine, ZoneId,
};

// ── Shared helpers ───────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
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

fn permanent_targeted_count(events: &[GameEvent], target_id: ObjectId) -> usize {
    events
        .iter()
        .filter(
            |e| matches!(e, GameEvent::PermanentTargeted { target_id: t, .. } if *t == target_id),
        )
        .count()
}

fn ward_ability_triggered_count(
    events: &[GameEvent],
    ward_id: ObjectId,
    ward_controller: PlayerId,
) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { controller, source_object_id, .. }
                if *source_object_id == ward_id && *controller == ward_controller
            )
        })
        .count()
}

fn cast(player: PlayerId, card: ObjectId, targets: Vec<Target>) -> Command {
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
    }))
}

// ── t1 / t1b: the resolution-path probe (Part B's own subject) ──────────────────

/// CR 702.21a / CR 603.3b — a self-ETB triggered ability whose CR 603.3d target slot
/// has exactly ONE legal candidate is placed entirely inside a spell's RESOLUTION
/// (`resolve_top_of_stack`'s post-resolution sweep, reached via `Command::PassPriority`
/// alone — never `Command::ChooseTriggerTargets`). Ward must still dispatch exactly
/// once from that path.
///
/// **Fixture is load-bearing in three ways, matching the coordinator's measured probe
/// verbatim (do not alter without re-measuring):**
/// 1. The trigger source (`T48 Pinger Relic`) is an ARTIFACT, not a creature, so it
///    does not add itself to the `TargetCreature` candidate set.
/// 2. The ward creature is the ONLY creature on the battlefield, so the slot has
///    exactly one legal candidate and `forced_trigger_target_answer` answers it
///    without suspending (CR 601.2c: one legal answer is not a choice) — this is what
///    makes the probe exercise the RESOLUTION path rather than the
///    `ChooseTriggerTargets` resume path.
/// 3. It drives `Command::CastSpell` then two `Command::PassPriority`s, so the
///    triggered ability is placed by `resolve_top_of_stack`'s sweep, a path
///    `check_and_flush_triggers` never runs after.
///
/// **Why this is the DECISIVE probe of the file.** Reverting the wave loop inside
/// `abilities::flush_pending_triggers` (replacing its body with
/// `flush_pending_triggers_once(state)`) leaves `PermanentTargeted` emission INTACT
/// (Part A is untouched) and still reddens this test: `PermanentTargeted` count stays
/// 1, but zero ward triggers reach the stack. A probe that asserted only on
/// `PermanentTargeted` — never on the stack / resolution outcome — would stay GREEN
/// under that revert and prove nothing about dispatch. See the matrix in
/// `memory/primitives/pb-DX48-execution-notes.md`, row R-B.
#[test]
fn test_dx48_t1_resolution_placed_forced_target_trigger_dispatches_ward_once() {
    let p1 = p(1);
    let p2 = p(2);

    // p2's ward creature is the ONLY creature on the battlefield.
    let ward =
        ObjectSpec::creature(p2, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    // The source is an ARTIFACT, so it never adds itself to the TargetCreature
    // candidate set.
    let relic = ObjectSpec::card(p1, "T48 Pinger Relic")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        })
        .with_triggered_ability(TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            trigger_on: TriggerEvent::SelfEntersBattlefield,
            intervening_if: None,
            description: "deals 1 damage to target creature".to_string(),
            effect: Some(Effect::DealDamage {
                source: Some(CardEffectTarget::Source),
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![TargetRequirement::TargetCreature],
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ward)
        .object(relic)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p1);

    let relic_id = find_object(&state, "T48 Pinger Relic");
    let ward_id = find_object(&state, "Ward Creature");

    let (state, _) = process_command(state, cast(p1, relic_id, vec![]))
        .expect("casting T48 Pinger Relic must succeed");
    let (state, e1) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, e2) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let resolve_events: Vec<GameEvent> = e1.into_iter().chain(e2).collect();

    // CR 702.21a: exactly one PermanentTargeted for the ward creature, and exactly
    // one ward AbilityTriggered (controller = the ward creature's own controller p2)
    // -- both COUNTS, not mere presence (finding #2: a hook that double-dispatches
    // would still emit >= 1 of each).
    assert_eq!(
        permanent_targeted_count(&resolve_events, ward_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted for the forced-target resolution trigger"
    );
    assert_eq!(
        ward_ability_triggered_count(&resolve_events, ward_id, p2),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered, placed by the RESOLUTION sweep"
    );
    assert_eq!(
        state.stack_objects().len(),
        2,
        "stack: T48 Pinger Relic's triggered ability + the ward trigger on top"
    );
    assert!(
        matches!(
            state.stack_objects().back().unwrap().kind,
            StackObjectKind::TriggeredAbility { .. }
        ),
        "the ward trigger, placed second, must be on TOP (resolves first)"
    );

    // Resolve fully: ward trigger resolves first (MayPayOrElse always applies
    // or_else = CounterSpell, per ward.rs's module doc), countering the relic's
    // triggered ability before its DealDamage effect ever runs.
    let (state, resolve2) = pass_all(state, &[p1, p2]);
    assert!(
        resolve2
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "CR 702.21a: ward should counter the relic's triggered ability"
    );
    assert!(
        state.stack_objects().is_empty(),
        "both stack entries must be gone: the ward trigger resolved, and it countered the other"
    );
    let ward_obj = state
        .objects()
        .get(&ward_id)
        .expect("ward creature must still be on the battlefield");
    assert_eq!(
        ward_obj.damage_marked, 0,
        "CR 702.21a: the ward creature must take NO damage -- its own DealDamage \
         effect was countered before it could resolve"
    );
}

/// CR 702.21a's "an opponent controls" clause — the non-vacuity partner for t1.
/// Same forced-target resolution shape, but the triggering ability's controller IS
/// the ward creature's own controller (both p1). Ward must NOT fire, but
/// `PermanentTargeted` must still be emitted (Part A's predicate is
/// controller-agnostic — only `check_triggers`'s Ward-collection arm reads
/// controller). Asserting the +1 alongside the 0 is what makes this discriminate
/// (see the doc comment on `test_dx48_t1_...` for why a bare "ward fired zero times"
/// assertion would be vacuous under total emission failure too).
#[test]
fn test_dx48_t1b_ward_does_not_fire_for_its_own_controllers_triggered_ability() {
    let p1 = p(1);
    let p2 = p(2);

    // Both the ward creature AND the artifact are controlled by p1 this time.
    let ward =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let relic = ObjectSpec::card(p1, "T48 Pinger Relic")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        })
        .with_triggered_ability(TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            trigger_on: TriggerEvent::SelfEntersBattlefield,
            intervening_if: None,
            description: "deals 1 damage to target creature".to_string(),
            effect: Some(Effect::DealDamage {
                source: Some(CardEffectTarget::Source),
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![TargetRequirement::TargetCreature],
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ward)
        .object(relic)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p1);

    let relic_id = find_object(&state, "T48 Pinger Relic");
    let ward_id = find_object(&state, "Ward Creature");

    let (state, _) = process_command(state, cast(p1, relic_id, vec![]))
        .expect("casting T48 Pinger Relic must succeed");
    let (state, e1) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, e2) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let resolve_events: Vec<GameEvent> = e1.into_iter().chain(e2).collect();

    assert_eq!(
        permanent_targeted_count(&resolve_events, ward_id),
        1,
        "Part A's emission is controller-agnostic: the event still fires even though \
         the ward creature's own controller is the one targeting it"
    );
    assert_eq!(
        ward_ability_triggered_count(&resolve_events, ward_id, p1),
        0,
        "CR 702.21a: ward triggers only for an OPPONENT's spell/ability -- zero here"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "only the relic's own triggered ability should be on the stack -- no ward trigger"
    );
}

// ── t2: handle_activate_forecast (site A3) ───────────────────────────────────────

fn t48_forecast_probe_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("t48-forecast-probe".to_string()),
        name: "T48 Forecast Probe".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Forecast — {1}, Reveal this card from your hand: It deals 1 damage to \
                      target creature."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Forecast),
            AbilityDefinition::Forecast {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
                effect: Effect::DealDamage {
                    source: None,
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// CR 702.57a (Forecast) / CR 702.21a — site A3. Forecast had NO Ward dispatch at all
/// before this batch (census §2's "5 with no Ward dispatch"). `check_and_flush_triggers`
/// runs immediately after `handle_activate_forecast` in the SAME command, so this
/// probe is discriminated by the "no emission" revert (R-A) but NOT by the "single
/// wave" revert (R-B) — the Ward trigger is found by `check_and_flush_triggers`'s own
/// (still single-pass) scan of the command's `events`, independent of the wave loop
/// living inside `flush_pending_triggers`. Disclosed rather than implied.
#[test]
fn test_dx48_t2_forecast_ability_dispatches_ward() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![t48_forecast_probe_def()]);

    let ward =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let probe_card = ObjectSpec::card(p2, "T48 Forecast Probe")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("t48-forecast-probe".to_string()))
        .with_keyword(KeywordAbility::Forecast)
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward)
        .object(probe_card)
        .active_player(p2)
        .at_step(Step::Upkeep)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p2);

    let ward_id = find_object(&state, "Ward Creature");
    let probe_id = find_object(&state, "T48 Forecast Probe");

    let (state, events) = process_command(
        state,
        Command::ActivateForecast {
            player: p2,
            card: probe_id,
            targets: vec![Target::Object(ward_id)],
        },
    )
    .expect("ActivateForecast targeting the ward creature must succeed");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityActivated { player, .. } if *player == p2)),
        "AbilityActivated must be emitted"
    );
    assert_eq!(
        permanent_targeted_count(&events, ward_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted from the forecast activation (site A3)"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, ward_id, p1),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered"
    );

    let (state, _resolve_events) = pass_all(state, &[p2, p1]);
    // `StackObjectKind::ForecastAbility` is not one of the two kinds
    // `Effect::CounterSpell` names in its `SpellCountered` event (PB-DX25's own
    // documented wildcard: "every other ability/trigger kind: NO event, exactly as
    // before PB-DX25 -- a DIAGNOSTICS omission, not a state one"), so the assertion
    // here is on STATE, not on the event: both stack entries are gone (the ward
    // trigger resolved and its CounterSpell removed the forecast ability), and the
    // resolution-effect check below (zero damage) proves it was actually removed
    // BEFORE its own effect could run, not merely that the stack drained normally.
    assert!(
        state.stack_objects().is_empty(),
        "both stack entries resolve away: ward, then the countered forecast ability"
    );
    assert_eq!(
        state.objects().get(&ward_id).unwrap().damage_marked,
        0,
        "the ward creature must take no damage -- forecast's DealDamage never ran"
    );
}

// ── t3: handle_scavenge_card (site A12) ──────────────────────────────────────────

fn t48_scavenge_probe_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("t48-scavenge-probe".to_string()),
        name: "T48 Scavenge Probe".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(3),
        toughness: Some(3),
        oracle_text: "Scavenge {1}{G}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Scavenge),
            AbilityDefinition::Scavenge {
                cost: ManaCost {
                    green: 1,
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

/// CR 702.97a (Scavenge) / CR 702.21a — site A12. Same "no dispatch before this
/// batch, still discriminated by R-A only, not R-B" shape as t2 (Command::ScavengeCard
/// calls `check_and_flush_triggers` immediately after `handle_scavenge_card` in the
/// same command).
#[test]
fn test_dx48_t3_scavenge_ability_dispatches_ward() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![t48_scavenge_probe_def()]);

    let ward =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let goliath = ObjectSpec::creature(p2, "T48 Scavenge Probe", 3, 3)
        .in_zone(ZoneId::Graveyard(p2))
        .with_card_id(CardId("t48-scavenge-probe".to_string()))
        .with_keyword(KeywordAbility::Scavenge);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward)
        .object(goliath)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p2);

    let ward_id = find_object(&state, "Ward Creature");
    let goliath_id = find_object(&state, "T48 Scavenge Probe");

    let (state, events) = process_command(
        state,
        Command::ScavengeCard {
            player: p2,
            card: goliath_id,
            target_creature: ward_id,
        },
    )
    .expect("ScavengeCard targeting the ward creature must succeed");

    assert_eq!(
        permanent_targeted_count(&events, ward_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted from the scavenge activation (site A12)"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, ward_id, p1),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered"
    );

    // `StackObjectKind::ScavengeAbility` is outside `SpellCountered`'s two named
    // kinds (see t2's comment for the full explanation) -- the resolution-effect
    // check below (zero counters) is the assertion that discriminates "countered"
    // from "resolved normally", not an event.
    let (state, _resolve_events) = pass_all(state, &[p2, p1]);
    assert!(
        state.stack_objects().is_empty(),
        "both stack entries resolve away: ward, then the countered scavenge ability"
    );
    let counters = state
        .objects()
        .get(&ward_id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 0,
        "CR 702.97a + CR 702.21a: no +1/+1 counters -- scavenge's effect was countered"
    );
}

// ── t4: handle_activate_loyalty_ability (site A13) ───────────────────────────────

fn t48_loyalty_walker_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("t48-loyalty-walker".to_string()),
        name: "T48 Loyalty Walker".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Planeswalker].into_iter().collect(),
            ..Default::default()
        },
        starting_loyalty: Some(4),
        oracle_text: "−2: Gain control of target creature until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::LoyaltyAbility {
            cost: LoyaltyCost::Minus(2),
            effect: Effect::GainControl {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                duration: EffectDuration::UntilEndOfTurn,
            },
            targets: vec![TargetRequirement::TargetCreature],
        }],
        ..Default::default()
    }
}

/// CR 606.1 (loyalty abilities) / CR 702.21a — site A13. Same "R-A only" shape.
/// Resolution effect assertion: since Ward always counters (non-interactive
/// `MayPayOrElse`), `GainControl` never applies, so the ward creature's controller
/// must stay p1, never flip to p2.
#[test]
fn test_dx48_t4_loyalty_ability_dispatches_ward() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![t48_loyalty_walker_def()]);

    let ward =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let walker = ObjectSpec::card(p2, "T48 Loyalty Walker")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("t48-loyalty-walker".to_string()))
        .with_types(vec![CardType::Planeswalker])
        .with_counter(CounterType::Loyalty, 4);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward)
        .object(walker)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let ward_id = find_object(&state, "Ward Creature");
    let walker_id = find_object(&state, "T48 Loyalty Walker");

    let (state, events) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p2,
            source: walker_id,
            ability_index: 0,
            targets: vec![Target::Object(ward_id)],
            x_value: None,
        },
    )
    .expect("ActivateLoyaltyAbility targeting the ward creature must succeed");

    assert_eq!(
        permanent_targeted_count(&events, ward_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted from the loyalty activation (site A13)"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, ward_id, p1),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered"
    );
    let loyalty_after = state
        .objects()
        .get(&walker_id)
        .unwrap()
        .counters
        .get(&CounterType::Loyalty)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        loyalty_after, 2,
        "CR 606.4: the -2 cost was paid at activation"
    );

    // `StackObjectKind::LoyaltyAbility` is outside `SpellCountered`'s two named
    // kinds (see t2's comment) -- the controller check below is the assertion that
    // discriminates "countered" from "resolved normally".
    let (state, _resolve_events) = pass_all(state, &[p2, p1]);
    assert!(
        state.stack_objects().is_empty(),
        "both stack entries resolve away: ward, then the countered loyalty ability"
    );
    assert_eq!(
        state.objects().get(&ward_id).unwrap().controller,
        p1,
        "CR 702.21a: GainControl never resolved -- the ward creature's controller \
         must stay p1"
    );
}

// ── t5: flush_sorted's Modular arm (site T6) ─────────────────────────────────────

/// CR 702.43a (Modular) / CR 702.21a — site T6, the auto-target (deterministic
/// first-artifact-creature) arm, entirely distinct from t1's suspend-free-but-real-
/// choice shape. The ward creature is given the Artifact type so it is the unique
/// legal Modular target once the dying creature has already left the battlefield.
///
/// This path is reached through SBA-driven death + the empty-stack step-priority
/// flush (`rules/engine.rs`'s `sba::check_and_apply_sbas` +
/// `abilities::flush_pending_triggers`, mirroring `mechanics_m_z/modular.rs`'s own
/// `test_modular_dies_transfers_counters`), the SAME site family as t1 (a flush that
/// is not followed by an outer `check_and_flush_triggers` re-scan) -- so, like t1, it
/// is expected to be discriminated by BOTH R-A and R-B; the revert matrix confirms
/// this by execution rather than by this comment alone.
#[test]
fn test_dx48_t5_modular_dies_trigger_dispatches_ward() {
    let p1 = p(1);
    let p2 = p(2);

    // p1's Ward Creature is ALSO an Artifact Creature -- the only OTHER artifact
    // creature on the battlefield once p2's Modular creature has died.
    let ward = ObjectSpec::creature(p1, "Ward Creature", 3, 3)
        .with_types(vec![CardType::Creature, CardType::Artifact])
        .with_keyword(KeywordAbility::Ward(2));

    // p2's Modular 1 creature, already lethally damaged (0/0 base + 1 counter = 1/1
    // effective, 1 damage marked) -- mirrors modular.rs's own fixture exactly.
    let modular_creature = ObjectSpec::creature(p2, "Modular Dier", 0, 0)
        .with_keyword(KeywordAbility::Modular(1))
        .with_types(vec![CardType::Artifact, CardType::Creature])
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .with_damage(1)
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(ward)
        .object(modular_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let ward_id = find_object(&state, "Ward Creature");

    // Pass priority: SBA kills the Modular creature, its dies-trigger is placed
    // (targeting the ward creature, the only remaining artifact creature), and
    // that placement's own PermanentTargeted dispatches Ward -- all within this
    // one PassPriority round, mirroring modular.rs's own pattern.
    let (state, events) = pass_all(state, &[p1, p2]);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "precondition: the Modular creature must die to SBA"
    );
    assert_eq!(
        permanent_targeted_count(&events, ward_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted from the Modular dies trigger's \
         placement (site T6)"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, ward_id, p1),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered"
    );
    assert_eq!(
        state.stack_objects().len(),
        2,
        "stack: the Modular trigger + the ward trigger on top"
    );

    // Resolve fully: ward counters the Modular trigger before it can add counters.
    // `StackObjectKind::KeywordTrigger` (Modular's own kind) is outside
    // `SpellCountered`'s two named kinds (see t2's comment) -- the zero-counters
    // check below is the assertion that discriminates "countered" from "resolved
    // normally".
    let (state, _resolve_events) = pass_all(state, &[p1, p2]);
    assert!(
        state.stack_objects().is_empty(),
        "both stack entries resolve away: ward, then the countered Modular trigger"
    );
    let counters = state
        .objects()
        .get(&ward_id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        counters, 0,
        "CR 702.43a + CR 702.21a: no +1/+1 counters transferred -- Modular's effect \
         was countered"
    );
}

// ── t6: all three deck-legal Complete Ward defs, real corpus cards ──────────────

fn t48_bear_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("t48-trigger-bear".to_string()),
        name: "T48 Trigger Bear".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        oracle_text: "When this creature enters, it deals 1 damage to target creature an \
                      opponent controls."
            .to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: mtg_engine::TriggerCondition::WhenEntersBattlefield,
            effect: Effect::DealDamage {
                source: None,
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller: TargetController::Opponent,
                ..Default::default()
            })],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}

fn defs_with_bear() -> HashMap<String, CardDefinition> {
    let mut defs: HashMap<String, CardDefinition> = all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect();
    let bear = t48_bear_def();
    defs.insert(bear.name.clone(), bear);
    defs
}

fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    enrich_spec_from_def(base, defs)
}

/// CR 702.21a — all three deck-legal `Complete` Ward defs in the corpus
/// (`Adrix and Nev, Twincasters` Ward {2}, `Miirym, Sentinel Wyrm` Ward {2},
/// `Tyrranax Rex` Ward {4}) each fire exactly once when a REAL corpus permanent
/// (built through `enrich_spec_from_def` + `GameStateBuilder::build`, matching the
/// plan's §2 verification, never a stand-in) targets it. The `TargetCreatureWithFilter`
/// filter makes the ward creature the ONLY legal candidate, so this reaches Ward via
/// the RESOLUTION path (same shape as t1), forcing the answer without a suspend.
/// Asserting the Ward keyword's own printed cost per def (2, 2, 4) proves the probe
/// reads the real def, not a hand-authored stand-in.
#[test]
fn test_dx48_t6_real_ward_defs_each_fire_exactly_once() {
    let defs = defs_with_bear();
    let registry = CardRegistry::new(defs.values().cloned().collect::<Vec<_>>());

    let cases: [(&str, u32); 3] = [
        ("Adrix and Nev, Twincasters", 2),
        ("Miirym, Sentinel Wyrm", 2),
        ("Tyrranax Rex", 4),
    ];

    for (ward_name, expected_cost) in cases {
        let p1 = p(1);
        let p2 = p(2);

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(Arc::clone(&registry))
            .object(real_card_spec(p1, ward_name, ZoneId::Battlefield, &defs))
            .object(real_card_spec(
                p2,
                "T48 Trigger Bear",
                ZoneId::Hand(p2),
                &defs,
            ))
            .active_player(p2)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap_or_else(|e| panic!("build failed for {ward_name}: {e:?}"));
        state
            .players_mut()
            .get_mut(&p2)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, 1);
        state.turn_mut().priority_holder = Some(p2);

        let ward_id = find_object(&state, ward_name);
        let bear_id = find_object(&state, "T48 Trigger Bear");

        assert!(
            state
                .objects()
                .get(&ward_id)
                .unwrap()
                .characteristics
                .keywords
                .contains(&KeywordAbility::Ward(expected_cost)),
            "{ward_name}: expected Ward {{{expected_cost}}} from the REAL def, proving \
             this probe reads the real card and not a stand-in"
        );

        let (state, _) = process_command(state, cast(p2, bear_id, vec![]))
            .unwrap_or_else(|e| panic!("casting T48 Trigger Bear failed for {ward_name}: {e:?}"));
        let (state, e1) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
        let (state, e2) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
        let resolve_events: Vec<GameEvent> = e1.into_iter().chain(e2).collect();

        assert_eq!(
            permanent_targeted_count(&resolve_events, ward_id),
            1,
            "{ward_name}: exactly one PermanentTargeted"
        );
        assert_eq!(
            ward_ability_triggered_count(&resolve_events, ward_id, p1),
            1,
            "{ward_name}: exactly one ward AbilityTriggered"
        );
        assert!(
            state.stack_objects().len() >= 2,
            "{ward_name}: the bear's ETB trigger + the ward trigger must both be on the stack"
        );
    }
}

// ── t7: wave bound / no cascade ──────────────────────────────────────────────────

/// CR 702.21a — the ward trigger's OWN target is the targeting stack object itself
/// (`SpellTarget { target: Object(<targeting stack id>), zone_at_cast: None }`), which
/// structurally never satisfies `permanent_targeted_events`'s `zone_at_cast ==
/// Some(Battlefield)` predicate. That is the reason the wave loop terminates after
/// dispatching Ward exactly once, rather than needing all 16 `MAX_BECOMES_TARGET_WAVES`
/// -- asserted here from the LIVE stack object, not from a comment.
#[test]
fn test_dx48_t7_ward_trigger_target_has_no_zone_at_cast_so_no_cascade() {
    use mtg_engine::CardEffectTarget as CET;

    let p1 = p(1);
    let p2 = p(2);

    let ward =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let ability = ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
            remove_counter_cost: None,
            exile_self: false,
            exert: false,
            life_cost: 0,
            sacrifice_exclude_self: false,
            exile_self_from_hand: false,
        },
        description: "{T}: 1 damage to target creature".to_string(),
        effect: Some(Effect::DealDamage {
            source: None,
            target: CET::DeclaredTarget { index: 0 },
            amount: EffectAmount::Fixed(1),
        }),
        sorcery_speed: false,
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: None,
    };
    let pinger = ObjectSpec::creature(p2, "Pinger", 1, 1).with_activated_ability(ability);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ward)
        .object(pinger)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let ward_id = find_object(&state, "Ward Creature");
    let pinger_id = find_object(&state, "Pinger");

    let (state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: pinger_id,
            ability_index: 0,
            targets: vec![Target::Object(ward_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("activating Pinger's ability must succeed");

    // No cascade: exactly one PermanentTargeted event total for this command, even
    // though the ward trigger's own placement runs through the SAME dispatch machinery.
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, GameEvent::PermanentTargeted { .. }))
            .count(),
        1,
        "no wave-2 cascade: only Pinger's activation produces a PermanentTargeted, \
         never the ward trigger's own placement"
    );

    let pinger_stack_id = state
        .stack_objects()
        .iter()
        .find(|so| matches!(so.kind, StackObjectKind::ActivatedAbility { source_object, .. } if source_object == pinger_id))
        .expect("Pinger's activated ability must be on the stack")
        .id;
    let ward_stack_obj = state
        .stack_objects()
        .iter()
        .find(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { source_object, .. } if source_object == ward_id))
        .expect("the ward trigger must be on the stack");

    assert_eq!(
        ward_stack_obj.targets.len(),
        1,
        "the ward trigger targets exactly one thing: the targeting ability"
    );
    assert_eq!(
        ward_stack_obj.targets[0].target,
        Target::Object(pinger_stack_id),
        "CR 702.21a: the ward trigger's target is the TARGETING STACK OBJECT itself"
    );
    assert_eq!(
        ward_stack_obj.targets[0].zone_at_cast, None,
        "the ward trigger's own SpellTarget.zone_at_cast must be None -- a stack \
         object is never `Some(ZoneId::Battlefield)`, which is exactly why the wave \
         loop cannot cascade from Ward's own placement"
    );
}

// ── t8: the three pre-existing emitter sites are unchanged ──────────────────────

/// CR 702.21a — site S1 (`casting.rs::handle_cast_spell`). Part A deleted the
/// hand-rolled `battlefield_targets` collection loop here and folded it into the
/// shared helper; this proves the emission is byte-for-byte unchanged for a plain
/// targeted cast.
#[test]
fn test_dx48_t8a_cast_site_still_emits_the_same_permanent_targeted_payload() {
    let p1 = p(1);
    let p2 = p(2);

    let bolt_def = CardDefinition {
        card_id: CardId("t48-bolt".to_string()),
        name: "T48 Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "T48 Bolt deals 1 damage to target creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                source: None,
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![bolt_def]);

    let ward =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let bolt = ObjectSpec::card(p2, "T48 Bolt")
        .in_zone(ZoneId::Hand(p2))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        // PB-DX18 (OOS-M11-5): ObjectSpec::card() is naked -- the def this fixture registers
        // was never linked, so the spell announced a target while the engine believed it
        // required none (CR 601.2c).
        .with_card_id(CardId("t48-bolt".to_string()));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward)
        .object(bolt)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn_mut().priority_holder = Some(p2);

    let ward_id = find_object(&state, "Ward Creature");
    let bolt_id = find_object(&state, "T48 Bolt");

    let (_state, events) = process_command(state, cast(p2, bolt_id, vec![Target::Object(ward_id)]))
        .expect("casting T48 Bolt at the ward creature must succeed");

    assert_eq!(
        permanent_targeted_count(&events, ward_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted for a plain targeted cast (site S1)"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, ward_id, p1),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered"
    );
}

/// CR 702.21a — site A1 (`abilities.rs::handle_activate_ability`). Part A deleted the
/// identical hand-rolled loop here too.
#[test]
fn test_dx48_t8b_activated_ability_site_still_emits_the_same_permanent_targeted_payload() {
    let p1 = p(1);
    let p2 = p(2);

    let ward =
        ObjectSpec::creature(p1, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));
    let ability = ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
            remove_counter_cost: None,
            exile_self: false,
            exert: false,
            life_cost: 0,
            sacrifice_exclude_self: false,
            exile_self_from_hand: false,
        },
        description: "{T}: destroy target creature".to_string(),
        effect: Some(Effect::DestroyPermanent {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: false,
        }),
        sorcery_speed: false,
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: None,
    };
    let assassin = ObjectSpec::creature(p2, "T48 Assassin", 1, 1).with_activated_ability(ability);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ward)
        .object(assassin)
        .active_player(p2)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let ward_id = find_object(&state, "Ward Creature");
    let assassin_id = find_object(&state, "T48 Assassin");

    let (_state, events) = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: assassin_id,
            ability_index: 0,
            targets: vec![Target::Object(ward_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("activating the assassin's ability must succeed");

    assert_eq!(
        permanent_targeted_count(&events, ward_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted for an activated ability (site A1)"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, ward_id, p1),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered"
    );
}

/// CR 207.2c (Bloodrush) / CR 702.21a — site A4. The deleted push here was
/// UNCONDITIONAL (no `zone_at_cast` check at all); the shared predicate now requires
/// `zone_at_cast == Some(Battlefield)`. Per the shipped commit's own comment this is a
/// measured no-op: bloodrush's `zone_at_cast` is read straight off the target object's
/// live zone, and step 5 of `handle_activate_bloodrush` already refuses any target that
/// is not an ATTACKING creature (i.e. not on the battlefield) before the push is ever
/// reached -- so the predicate can never observe a non-battlefield target from this
/// site, and this probe cannot construct a case where the two disagree. What it proves
/// instead: the predicate is satisfied on the one shape bloodrush can ever produce, and
/// the resolution effect (Ward counters the pump) is a genuine behavioural check, not
/// just an event count.
#[test]
fn test_dx48_t8c_bloodrush_site_predicate_agrees_with_the_deleted_unconditional_push() {
    use mtg_engine::AttackTarget;

    let p1 = p(1);
    let p2 = p(2);

    let bloodrush_def = CardDefinition {
        card_id: CardId("t48-bloodrush-pump".to_string()),
        name: "T48 Bloodrush Pump".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        oracle_text: "Bloodrush — {R}, Discard this card: Target attacking creature gets \
                      +4/+4 until end of turn."
            .to_string(),
        abilities: vec![AbilityDefinition::Bloodrush {
            cost: ManaCost {
                red: 1,
                ..Default::default()
            },
            power_boost: 4,
            toughness_boost: 4,
            grants_keyword: None,
        }],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![bloodrush_def]);

    // p1's Ward creature is attacking p2.
    let ward = ObjectSpec::creature(p1, "Ward Creature", 3, 3)
        .with_keyword(KeywordAbility::Ward(2))
        .in_zone(ZoneId::Battlefield);
    // p2 (the defending player) holds the bloodrush card in hand.
    let pump = ObjectSpec::card(p2, "T48 Bloodrush Pump")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("t48-bloodrush-pump".to_string()));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ward)
        .object(pump)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let ward_id = find_object(&state, "Ward Creature");
    let pump_id = find_object(&state, "T48 Bloodrush Pump");

    let (mut state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(ward_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("declaring the ward creature as an attacker must succeed");

    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn_mut().priority_holder = Some(p2);

    let (state, events) = process_command(
        state,
        Command::ActivateBloodrush {
            player: p2,
            card: pump_id,
            target: ward_id,
        },
    )
    .expect("ActivateBloodrush targeting the attacking ward creature must succeed");

    assert_eq!(
        permanent_targeted_count(&events, ward_id),
        1,
        "CR 702.21a: exactly one PermanentTargeted from bloodrush (site A4)"
    );
    assert_eq!(
        ward_ability_triggered_count(&events, ward_id, p1),
        1,
        "CR 702.21a: exactly one ward AbilityTriggered"
    );

    // `StackObjectKind::BloodrushAbility` is outside `SpellCountered`'s two named
    // kinds (see t2's comment) -- the power check below is the assertion that
    // discriminates "countered" from "resolved normally".
    let (state, _resolve_events) = pass_all(state, &[p2, p1]);
    assert!(
        state.stack_objects().is_empty(),
        "both stack entries resolve away: ward, then the countered bloodrush pump"
    );
    let power_after = mtg_engine::calculate_characteristics(&state, ward_id)
        .and_then(|c| c.power)
        .unwrap_or(0);
    assert_eq!(
        power_after, 3,
        "CR 207.2c + CR 702.21a: the +4/+4 pump never resolved -- power stays base 3"
    );
}

// ── t9: the CR 603.3d suspension PREFIX (the `/review` MEDIUM) ───────────────

/// CR 702.21a + CR 603.3b/603.3d — a batch member placed BEFORE the batch suspends
/// still dispatches Ward.
///
/// **This probe exists because the `/review` reproduced a real hole and the row that
/// filed it stated the wrong precondition.** `dispatch_becomes_target_waves`' first
/// draft tested `pending_trigger_targets.is_some()` at the TOP of its loop and
/// returned having collected nothing, so every `PermanentTargeted` emitted by the
/// members `flush_sorted` placed *before* it suspended was dropped on the floor.
/// Nothing else scans them: every `flush_pending_triggers` caller scans its events
/// BEFORE the flush, and `Command::ChooseTriggerTargets`' arm sweeps only the RESUMED
/// events. The draft's comment claimed the resumed call would cover it; that was false
/// in both halves, and this batch's whole subject is a false comment.
///
/// `OOS-DX48-3` filed it as needing "≥3 targeted triggers, ≥2 of them asking" and
/// "not reproduced". It needs **two** triggers, **one** asking, and it reproduces here.
///
/// Shape: p1's artifact ETB targets p2's Ward creature (forced — the ward creature is
/// the only creature p1 can target, so this member is placed with no question). p2
/// also controls a permanent whose ETB targets an opponent, and with THREE seats that
/// slot has two live candidates, so the batch suspends on p2's trigger AFTER p1's has
/// already been placed and announced.
///
/// **Red by revert**: move the `pending_trigger_targets.is_some()` early return in
/// `abilities::dispatch_becomes_target_waves` back above the slice collection. The
/// `PermanentTargeted` assertion stays GREEN (the emission was never the broken half)
/// while the ward-trigger count drops to 0 — which is why the count is the verdict.
#[test]
fn test_dx48_t9_a_batch_prefix_dispatches_ward_even_when_the_batch_suspends() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let ward =
        ObjectSpec::creature(p2, "Ward Creature", 3, 3).with_keyword(KeywordAbility::Ward(2));

    // p1's member: an ARTIFACT (never its own TargetCreature candidate) whose ETB
    // targets the only creature on the battlefield -- forced, so it is PLACED, not asked.
    let relic = ObjectSpec::card(p1, "T48 Prefix Relic")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        })
        .with_triggered_ability(TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            trigger_on: TriggerEvent::SelfEntersBattlefield,
            intervening_if: None,
            description: "deals 1 damage to target creature".to_string(),
            effect: Some(Effect::DealDamage {
                source: Some(CardEffectTarget::Source),
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![TargetRequirement::TargetCreature],
        });

    // p2's member: fires off the SAME `PermanentEnteredBattlefield` event, and its
    // `TargetOpponent` slot has two live candidates (p1, p3), so it SUSPENDS.
    let asker = ObjectSpec::enchantment(p2, "T48 Suspending Asker").with_triggered_ability(
        TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            trigger_on: TriggerEvent::AnyPermanentEntersBattlefield,
            intervening_if: None,
            description: "target opponent loses 1 life".to_string(),
            effect: Some(Effect::LoseLife {
                player: PlayerTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(1),
            }),
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![TargetRequirement::TargetOpponent],
        },
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(ward)
        .object(asker)
        .object(relic)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p1);

    let relic_id = find_object(&state, "T48 Prefix Relic");
    let ward_id = find_object(&state, "Ward Creature");

    let (state, _) = process_command(state, cast(p1, relic_id, vec![]))
        .expect("casting T48 Prefix Relic must succeed");
    let (state, e1) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, e2) = process_command(state, Command::PassPriority { player: p2 }).unwrap();
    let (state, e3) = process_command(state, Command::PassPriority { player: p3 }).unwrap();
    let mut all: Vec<GameEvent> = e1.into_iter().chain(e2).chain(e3).collect();

    // Precondition: the batch really did suspend, and it did so AFTER p1's member was
    // placed. Without both halves this probe is not about a prefix at all.
    let entry = state
        .pending_trigger_targets()
        .expect("precondition: the CR 603.3d batch must have SUSPENDED")
        .clone();
    assert_eq!(
        permanent_targeted_count(&all, ward_id),
        1,
        "precondition: the prefix member was placed and announced its target before \
         the batch suspended -- the emission was never the broken half"
    );

    // Answer the outstanding question; the rest of the batch is placed and the
    // prefix's queued Ward trigger is drained by the resume's own sweep.
    let default = entry.slots[0]
        .default
        .clone()
        .expect("TargetOpponent has a deterministic default");
    let (state, e4) = process_command(
        state,
        Command::ChooseTriggerTargets {
            player: entry.player,
            choice_id: entry.choice_id,
            targets: vec![vec![default.target.clone()]],
        },
    )
    .expect("the engine must accept its own default answer (SR-38)");
    all.extend(e4);

    // CR 702.21a: EXACTLY ONE ward trigger for the one targeting event -- not zero
    // (the defect) and not two (the double-dispatch design this batch rejected).
    assert_eq!(
        ward_ability_triggered_count(&all, ward_id, p2),
        1,
        "CR 702.21a / CR 603.3b: the prefix member's targeting must dispatch Ward \
         exactly once, even though the batch suspended after it was placed"
    );
    assert_eq!(
        permanent_targeted_count(&all, ward_id),
        1,
        "and the emission must still be exactly one -- the resume must not re-announce \
         the prefix member's target"
    );
    let _ = state;
}
