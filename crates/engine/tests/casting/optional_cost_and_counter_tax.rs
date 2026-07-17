//! PB-AC2: Optional-cost (beneficial-pay) wrapper & counter-tax primitive tests.
//!
//! `Effect::MayPayThenEffect { cost, payer, then }` (CR 118.12): "[player] may pay
//! [cost]. If they do, [then]." Distinct from the existing `Effect::MayPayOrElse`
//! (tax semantics: `or_else` fires when the payer DECLINES). `MayPayThenEffect` fires
//! `then` only when the cost is actually PAID.
//!
//! `Effect::CounterUnlessPays { target, cost }` (CR 118.12a): "Counter target spell
//! unless its controller pays [cost]." Equivalent to "controller may pay [cost]; if
//! they don't, counter." Delegates to `Effect::CounterSpell`.
//!
//! Both primitives resolve the optional cost non-interactively and deterministically:
//! the payer pays when able (CR 118.8/119.4), otherwise the cost is not paid. This is
//! a legal, replayable game choice (architecture invariant #9) pending M10+ interactive
//! pay-vs-decline.
//!
//! Card integration tests (crossway_troublemakers, hazorets_monument, springbloom_druid,
//! nadir_kraken, mana_leak, etc.) are deferred to the PB-AC2 backfill phase, since the
//! card definitions themselves are not modified in this implement phase.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::test_util;
use mtg_engine::{
    AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry, CardType, Command,
    Cost, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, ManaColor, ManaCost,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, SpellTarget, StackObject, StackObjectKind, Step,
    Target, TargetFilter, TargetRequirement, TypeLine, ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Execute `Effect::MayPayThenEffect { cost, payer: Controller, then }` directly
/// against `state` for `controller`. Returns (state, events).
fn run_may_pay_then(
    mut state: GameState,
    controller: PlayerId,
    cost: Cost,
    then: Effect,
) -> (GameState, Vec<GameEvent>) {
    let effect = Effect::MayPayThenEffect {
        cost,
        payer: PlayerTarget::Controller,
        then: Box::new(then),
    };
    // Placeholder source: not read by any cost/then combination used in this file.
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
}

/// A "then" effect that draws one card for the controller.
fn draw_one() -> Effect {
    Effect::DrawCards {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(1),
    }
}

/// Push a bare `StackObject::Spell` entry onto `state.stack_objects()` wrapping
/// `source_object`. Mirrors the manual-stack-push pattern used elsewhere in the
/// test suite (see `tests/forecast.rs`, `tests/dungeon_resolution.rs`) since
/// `Effect::CounterSpell`/`CounterUnlessPays` operate on `StackObject`, not on
/// `state.objects()` alone.
fn push_spell_stack_object(
    state: &mut GameState,
    source_object: ObjectId,
    controller: PlayerId,
    cast_with_flashback: bool,
) -> ObjectId {
    let stack_id = test_util::next_object_id(state);
    state.stack_objects_mut().push_back(StackObject {
        id: stack_id,
        controller,
        kind: StackObjectKind::Spell { source_object },
        targets: vec![],
        cant_be_countered: false,
        is_copy: false,
        cast_with_flashback,
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
        was_warped: false,
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
        lki_counters: imbl::OrdMap::new(),
        lki_power: None,
    });
    stack_id
}

/// Execute `Effect::CounterUnlessPays { target: DeclaredTarget{0}, cost }` against a
/// spell whose card object id is `target_card_id` (declared target index 0).
fn run_counter_unless_pays(
    mut state: GameState,
    controller: PlayerId,
    target_card_id: ObjectId,
    cost: Cost,
) -> (GameState, Vec<GameEvent>) {
    let effect = Effect::CounterUnlessPays {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        cost,
    };
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(
        controller,
        source,
        vec![SpellTarget {
            target: Target::Object(target_card_id),
            zone_at_cast: None,
        }],
    );
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
}

// ── Beneficial-pay: PayLife ─────────────────────────────────────────────────────

#[test]
/// CR 118.12 / 119.4: "You may pay 2 life. If you do, draw a card." Payer has enough
/// life -> pays, `then` runs.
fn test_may_pay_then_effect_paylife_pays_and_runs() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state.players_mut().get_mut(&p1).unwrap().life_total = 20;

    let (state, events) = run_may_pay_then(state, p1, Cost::PayLife(2), draw_one());

    assert!(
        events.iter().any(
            |e| matches!(e, GameEvent::LifeLost { player, amount } if *player == p1 && *amount == 2)
        ),
        "CR 119.4: paying life is losing life -- LifeLost should be emitted"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 118.12: cost was paid -- `then` (draw a card) should run"
    );
    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        18,
        "life total should be reduced by the paid amount"
    );
}

#[test]
/// CR 119.4 (negative): a player can pay life only if their life total is >= the
/// amount. Insufficient life -> cost not paid -> `then` does not run.
fn test_may_pay_then_effect_paylife_insufficient_declines() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state.players_mut().get_mut(&p1).unwrap().life_total = 1;

    let (state, events) = run_may_pay_then(state, p1, Cost::PayLife(2), draw_one());

    assert!(
        events.is_empty(),
        "CR 119.4: life total (1) < cost (2) -- no life should be lost and `then` \
         should not run; got events: {:?}",
        events
    );
    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        1,
        "life total should be unchanged when the cost cannot be paid"
    );
}

// ── Beneficial-pay: DiscardCard ─────────────────────────────────────────────────

#[test]
/// CR 118.12: "You may discard a card. If you do, draw a card." Hand has a card ->
/// pays (discards), `then` runs.
fn test_may_pay_then_effect_discard_pays_and_runs() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Filler Card A").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Filler Card B").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_may_pay_then(state, p1, Cost::DiscardCard, draw_one());

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { player, .. } if *player == p1)),
        "CR 118.12: cost was paid -- a card should be discarded"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 118.12: cost was paid -- `then` (draw a card) should run"
    );
    let hand_count = state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Hand(p1))
        .count();
    let grave_count = state
        .objects()
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Graveyard(pid) if pid == p1))
        .count();
    assert_eq!(
        grave_count, 1,
        "one card should have been discarded to graveyard"
    );
    assert_eq!(
        hand_count, 2,
        "net hand change: started with 2, discarded 1 (-> 1), drew 1 (-> 2)"
    );
}

#[test]
/// CR 118.12 (negative): empty hand -> nothing to discard -> cost not paid ->
/// `then` does not run.
fn test_may_pay_then_effect_discard_empty_hand_declines() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let (state, events) = run_may_pay_then(state, p1, Cost::DiscardCard, draw_one());

    assert!(
        events.is_empty(),
        "CR 118.12: empty hand -- discard cost cannot be paid, `then` should not run; \
         got events: {:?}",
        events
    );
    let grave_count = state
        .objects()
        .values()
        .filter(|o| matches!(o.zone, ZoneId::Graveyard(pid) if pid == p1))
        .count();
    assert_eq!(grave_count, 0, "no card should have been discarded");
}

// ── Beneficial-pay: Sacrifice ────────────────────────────────────────────────────

#[test]
/// CR 118.12 / 701.21a: "You may sacrifice a creature. If you do, draw a card."
/// Controls a matching creature -> pays (sacrifices), `then` runs, dies trigger fires.
fn test_may_pay_then_effect_sacrifice_pays_and_runs() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Sac Fodder", 1, 1))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    };
    let (state, events) = run_may_pay_then(state, p1, Cost::Sacrifice(filter), draw_one());

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { controller, .. } if *controller == p1)),
        "CR 701.21a: sacrificing a creature should fire the dies event"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PermanentSacrificed { player, .. } if *player == p1)),
        "CR 701.21a: PermanentSacrificed should be emitted alongside the dies event"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 118.12: cost was paid -- `then` (draw a card) should run"
    );
    assert!(
        !state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Sac Fodder" && o.zone == ZoneId::Battlefield),
        "Sac Fodder should have left the battlefield"
    );
}

#[test]
/// CR 118.12 (negative): no matching permanent to sacrifice -> cost not paid ->
/// `then` does not run.
fn test_may_pay_then_effect_sacrifice_none_declines() {
    let p1 = p(1);
    // p1 controls a noncreature permanent only -- no eligible sacrifice target
    // under a creature filter.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::card(p1, "Some Land")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Land]),
        )
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    };
    let (state, events) = run_may_pay_then(state, p1, Cost::Sacrifice(filter), draw_one());

    assert!(
        events.is_empty(),
        "CR 118.12: no eligible creature to sacrifice -- cost cannot be paid, \
         `then` should not run; got events: {:?}",
        events
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Some Land" && o.zone == ZoneId::Battlefield),
        "the land should be untouched"
    );
}

// ── Beneficial-pay: Mana ────────────────────────────────────────────────────────

#[test]
/// CR 118.12 / 118.8: "You may pay {2}. If you do, draw a card." Mana pools empty
/// between steps (CR 500.4), so this only pays when mana is pre-floated.
fn test_may_pay_then_effect_mana_requires_floating() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    // Pre-float {2}.
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    let cost = Cost::Mana(ManaCost {
        generic: 2,
        ..Default::default()
    });
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1)),
        "CR 118.12: floating mana was available -- `then` should run"
    );
    let pool = &state.players().get(&p1).unwrap().mana_pool;
    assert_eq!(
        pool.colorless, 0,
        "the floating mana should have been spent"
    );
}

#[test]
/// CR 118.12 (negative): mana pools empty between steps (CR 500.4) -- with no
/// floating mana, the mana cost cannot be paid and `then` does not run.
fn test_may_pay_then_effect_mana_empty_pool_declines() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let cost = Cost::Mana(ManaCost {
        generic: 2,
        ..Default::default()
    });
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    assert!(
        events.is_empty(),
        "CR 500.4 / 118.12: empty mana pool -- cost cannot be paid, `then` should \
         not run; got events: {:?}",
        events
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.zone == ZoneId::Library(p1)),
        "the library card should not have been drawn"
    );
}

// ── Beneficial-pay: Sequence atomicity ──────────────────────────────────────────

#[test]
/// CR 118.12 / 601.2g: "You may pay {1} and 1 life. If you do, draw a card." A
/// `Sequence` cost is atomic -- when every sub-cost is payable, all sub-costs are
/// paid and `then` runs.
fn test_may_pay_then_effect_sequence_pays_all_when_available() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state.players_mut().get_mut(&p1).unwrap().life_total = 20;
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let cost = Cost::Sequence(vec![
        Cost::Mana(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        Cost::PayLife(1),
    ]);
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { .. })),
        "CR 118.12: both sub-costs payable -- life should be paid"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "CR 118.12: both sub-costs payable -- `then` should run"
    );
    assert_eq!(state.players().get(&p1).unwrap().life_total, 19);
    assert_eq!(state.players().get(&p1).unwrap().mana_pool.colorless, 0);
}

#[test]
/// CR 118.12 / 601.2g (negative, atomicity): if ANY sub-cost of a `Sequence` cannot
/// be paid, NONE of the sub-costs are paid (no partial life loss) and `then` does
/// not run.
fn test_may_pay_then_effect_sequence_declines_all_when_any_unavailable() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state.players_mut().get_mut(&p1).unwrap().life_total = 20;
    // No mana floated -- the Mana{1} sub-cost is unpayable.

    let cost = Cost::Sequence(vec![
        Cost::Mana(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        Cost::PayLife(1),
    ]);
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    assert!(
        events.is_empty(),
        "CR 118.12: Sequence cost is atomic -- one unpayable sub-cost means NONE \
         are paid; got events: {:?}",
        events
    );
    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        20,
        "life must NOT be paid when the mana sub-cost is unavailable (atomicity)"
    );
}

// ── Beneficial-pay: Sequence cumulative-depletion atomicity (review fix) ────────
//
// CR 118.12: "the entire cost must be paid" -- a `Sequence` is atomic across its
// FULL combined demand, not just checked sub-cost-by-sub-cost against the
// untouched starting state. A homogeneous-resource sequence (two sub-costs
// drawing on the SAME pool: life, the same sacrificeable-permanent set, or the
// same floating mana) must fail the pre-check -- and pay nothing -- when the
// combined demand exceeds what's available, even though each sub-cost checked
// in isolation against the starting state would appear payable.

#[test]
/// CR 118.12 (negative, cumulative depletion): `Sequence[Sacrifice(creature),
/// Sacrifice(creature)]` with only ONE eligible creature on the battlefield.
/// Checked independently against the untouched state, both sub-costs "look"
/// payable (1 >= 1 eligible target, twice) -- but only one creature actually
/// exists, so the combined demand (2 distinct sacrifices) is not satisfiable.
/// The whole cost must be declined: `then` does not run and the creature survives.
fn test_may_pay_then_effect_sequence_sacrifice_sacrifice_one_eligible_declines() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Only Fodder", 1, 1))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    };
    let cost = Cost::Sequence(vec![
        Cost::Sacrifice(filter.clone()),
        Cost::Sacrifice(filter),
    ]);
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    assert!(
        events.is_empty(),
        "CR 118.12: only 1 eligible creature exists for a 2-sacrifice Sequence -- \
         cumulative demand is unsatisfiable, so NOTHING should be paid and `then` \
         should not run; got events: {:?}",
        events
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Only Fodder" && o.zone == ZoneId::Battlefield),
        "the sole creature must survive -- atomicity means no partial sacrifice"
    );
}

#[test]
/// CR 118.12 (positive control): `Sequence[Sacrifice(creature), Sacrifice(creature)]`
/// with TWO eligible creatures -- the combined demand is satisfiable, so both are
/// sacrificed and `then` runs.
fn test_may_pay_then_effect_sequence_sacrifice_sacrifice_two_eligible_pays() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::creature(p1, "Fodder A", 1, 1))
        .object(ObjectSpec::creature(p1, "Fodder B", 1, 1))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let filter = TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    };
    let cost = Cost::Sequence(vec![
        Cost::Sacrifice(filter.clone()),
        Cost::Sacrifice(filter),
    ]);
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    let sac_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::PermanentSacrificed { .. }))
        .count();
    assert_eq!(
        sac_count, 2,
        "CR 118.12: 2 eligible creatures satisfy the combined demand -- both should \
         be sacrificed; got events: {:?}",
        events
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "CR 118.12: full cost paid -- `then` should run"
    );
    assert!(
        !state
            .objects()
            .values()
            .any(|o| o.zone == ZoneId::Battlefield
                && (o.characteristics.name == "Fodder A" || o.characteristics.name == "Fodder B")),
        "both creatures should have left the battlefield"
    );
}

#[test]
/// CR 118.12 / 119.4 (negative, cumulative depletion): `Sequence[PayLife(5),
/// PayLife(5)]` at life 6. Each sub-cost checked independently against the
/// untouched state looks payable (6 >= 5, twice) -- but the combined demand (10)
/// exceeds the life total. The whole cost must be declined: life is unchanged and
/// `then` does not run.
fn test_may_pay_then_effect_sequence_paylife_paylife_insufficient_declines() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state.players_mut().get_mut(&p1).unwrap().life_total = 6;

    let cost = Cost::Sequence(vec![Cost::PayLife(5), Cost::PayLife(5)]);
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    assert!(
        events.is_empty(),
        "CR 119.4: life total (6) cannot cover the combined demand of a \
         PayLife(5)+PayLife(5) Sequence (10) -- nothing should be paid and `then` \
         should not run; got events: {:?}",
        events
    );
    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        6,
        "life must be unchanged -- atomicity means no partial life payment"
    );
}

#[test]
/// CR 118.12 / 119.4 (positive control): `Sequence[PayLife(5), PayLife(5)]` at life
/// 20 -- the combined demand (10) is satisfiable, so both sub-costs are paid and
/// `then` runs.
fn test_may_pay_then_effect_sequence_paylife_paylife_sufficient_pays() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state.players_mut().get_mut(&p1).unwrap().life_total = 20;

    let cost = Cost::Sequence(vec![Cost::PayLife(5), Cost::PayLife(5)]);
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    let life_lost_total: u32 = events
        .iter()
        .filter_map(|e| match e {
            GameEvent::LifeLost { amount, .. } => Some(*amount),
            _ => None,
        })
        .sum();
    assert_eq!(
        life_lost_total, 10,
        "CR 119.4: both PayLife(5) sub-costs should be paid in full; got events: {:?}",
        events
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "CR 118.12: full cost paid -- `then` should run"
    );
    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        10,
        "life should be reduced by the combined 10 total"
    );
}

#[test]
/// CR 118.12 / 118.8 (negative, cumulative depletion): `Sequence[Mana{1}, Mana{1}]`
/// with only 1 floating colorless mana. Each sub-cost checked independently
/// against the untouched pool looks payable (1 >= 1, twice) -- but the combined
/// demand (2) exceeds what's floating. The whole cost must be declined: the pool
/// is unchanged and `then` does not run.
fn test_may_pay_then_effect_sequence_mana_mana_one_floating_declines() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);

    let mana_one = || {
        Cost::Mana(ManaCost {
            generic: 1,
            ..Default::default()
        })
    };
    let cost = Cost::Sequence(vec![mana_one(), mana_one()]);
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    assert!(
        events.is_empty(),
        "CR 118.8/118.12: only 1 floating mana for a Mana{{1}}+Mana{{1}} Sequence \
         (combined demand 2) -- nothing should be paid and `then` should not run; \
         got events: {:?}",
        events
    );
    assert_eq!(
        state.players().get(&p1).unwrap().mana_pool.colorless,
        1,
        "the floating mana must be untouched -- atomicity means no partial spend"
    );
}

#[test]
/// CR 118.12 / 118.8 (positive control): `Sequence[Mana{1}, Mana{1}]` with 2
/// floating colorless mana -- the combined demand (2) is satisfiable, so both
/// sub-costs are paid and `then` runs.
fn test_may_pay_then_effect_sequence_mana_mana_two_floating_pays() {
    let p1 = p(1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Library Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);

    let mana_one = || {
        Cost::Mana(ManaCost {
            generic: 1,
            ..Default::default()
        })
    };
    let cost = Cost::Sequence(vec![mana_one(), mana_one()]);
    let (state, events) = run_may_pay_then(state, p1, cost, draw_one());

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "CR 118.12: 2 floating mana satisfies the combined demand -- `then` should \
         run; got events: {:?}",
        events
    );
    assert_eq!(
        state.players().get(&p1).unwrap().mana_pool.colorless,
        0,
        "both floating mana should have been spent"
    );
}

// ── Counter-tax: CounterUnlessPays ──────────────────────────────────────────────

#[test]
/// CR 118.12a / 701.5: "Counter target spell unless its controller pays {3}."
/// Deterministic path: the controller declines (never has an incentive to pay
/// without interactive choice) -- the target spell is countered.
fn test_counter_unless_pays_counters_when_declined() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::card(p2, "Target Spell")
                .in_zone(ZoneId::Stack)
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    let target_card_id = find_by_name(&state, "Target Spell");
    push_spell_stack_object(&mut state, target_card_id, p2, false);

    let cost = Cost::Mana(ManaCost {
        generic: 3,
        ..Default::default()
    });
    let (state, events) = run_counter_unless_pays(state, p1, target_card_id, cost);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "CR 118.12a: controller declined -- the spell should be countered"
    );
    assert!(
        state.stack_objects().is_empty(),
        "the countered spell's stack entry should be removed"
    );
    assert!(
        state.objects().values().any(|o| {
            o.characteristics.name == "Target Spell" && matches!(o.zone, ZoneId::Graveyard(_))
        }),
        "the countered spell should move to its owner's graveyard"
    );
}

#[test]
/// CR 702.34a regression: `CounterUnlessPays` delegates to `Effect::CounterSpell`,
/// which must preserve flashback-exile-at-counter -- a spell cast with flashback
/// that gets countered is exiled, not put into the graveyard.
fn test_counter_unless_pays_flashback_exiles() {
    let p1 = p(1);
    let p2 = p(2);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(
            ObjectSpec::card(p2, "Flashback Spell")
                .in_zone(ZoneId::Stack)
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    let target_card_id = find_by_name(&state, "Flashback Spell");
    push_spell_stack_object(
        &mut state,
        target_card_id,
        p2,
        true, /* cast_with_flashback */
    );

    let cost = Cost::Mana(ManaCost {
        generic: 1,
        ..Default::default()
    });
    let (state, events) = run_counter_unless_pays(state, p1, target_card_id, cost);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "CR 118.12a: controller declined -- the spell should be countered"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| { o.characteristics.name == "Flashback Spell" && o.zone == ZoneId::Exile }),
        "CR 702.34a: a flashback-cast spell countered by an effect must be exiled, \
         not put into the graveyard"
    );
    assert!(
        !state.objects().values().any(|o| {
            o.characteristics.name == "Flashback Spell" && matches!(o.zone, ZoneId::Graveyard(_))
        }),
        "the flashback spell must NOT end up in the graveyard"
    );
}

#[test]
/// CR 118.12a (Spell Pierce shape): "Counter target noncreature spell unless its
/// controller pays {2}." Target validation is existing `TargetSpellWithFilter`
/// infrastructure: a noncreature spell is a legal target; a creature spell is not.
fn test_counter_unless_pays_noncreature_filter() {
    fn spell_pierce_test_def() -> CardDefinition {
        CardDefinition {
            card_id: CardId("spell-pierce-test".to_string()),
            name: "Spell Pierce Test".to_string(),
            mana_cost: Some(ManaCost {
                blue: 1,
                ..Default::default()
            }),
            types: TypeLine {
                card_types: [CardType::Instant].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: "Counter target noncreature spell unless its controller pays {2}."
                .to_string(),
            abilities: vec![AbilityDefinition::Spell {
                effect: Effect::CounterUnlessPays {
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    cost: Cost::Mana(ManaCost {
                        generic: 2,
                        ..Default::default()
                    }),
                },
                targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                    non_creature: true,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            }],
            ..Default::default()
        }
    }

    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![spell_pierce_test_def()]);

    // ── Legal target: a noncreature (Sorcery) spell on the stack. ──────────────
    let noncreature_spell = ObjectSpec::card(p2, "Noncreature Spell")
        .in_zone(ZoneId::Stack)
        .with_types(vec![CardType::Sorcery]);
    let piercer = ObjectSpec::card(p1, "Spell Pierce Test")
        .with_card_id(CardId("spell-pierce-test".to_string()))
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            blue: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry.clone())
        .object(noncreature_spell)
        .object(piercer)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn_mut().priority_holder = Some(p1);

    let noncreature_id = find_by_name(&state, "Noncreature Spell");
    push_spell_stack_object(&mut state, noncreature_id, p2, false);
    let piercer_id = find_by_name(&state, "Spell Pierce Test");

    let legal_result = mtg_engine::process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: piercer_id,
            targets: vec![Target::Object(noncreature_id)],
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
    );
    assert!(
        legal_result.is_ok(),
        "CR 601.2c: a noncreature spell is a legal target for TargetSpellWithFilter{{non_creature: true}}: {:?}",
        legal_result.err()
    );

    // ── Illegal target: a creature spell on the stack. ──────────────────────────
    let creature_spell = ObjectSpec::card(p2, "Creature Spell")
        .in_zone(ZoneId::Stack)
        .with_types(vec![CardType::Creature]);
    let piercer2 = ObjectSpec::card(p1, "Spell Pierce Test")
        .with_card_id(CardId("spell-pierce-test".to_string()))
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            blue: 1,
            ..Default::default()
        });

    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(creature_spell)
        .object(piercer2)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    state2
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state2.turn_mut().priority_holder = Some(p1);

    let creature_id = find_by_name(&state2, "Creature Spell");
    push_spell_stack_object(&mut state2, creature_id, p2, false);
    let piercer2_id = find_by_name(&state2, "Spell Pierce Test");

    let illegal_result = mtg_engine::process_command(
        state2,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: piercer2_id,
            targets: vec![Target::Object(creature_id)],
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
    );
    assert!(
        illegal_result.is_err(),
        "CR 601.2c: a creature spell is NOT a legal target for \
         TargetSpellWithFilter{{non_creature: true}}"
    );
}

// ── Hash schema ──────────────────────────────────────────────────────────────────

#[test]
/// PB-AC2 bumped `HASH_SCHEMA_VERSION` 28 -> 29 (new `Effect` variants
/// `MayPayThenEffect` discriminant 88 and `CounterUnlessPays` discriminant 89).
/// If you bumped again, update this test and the `state/hash.rs` history block.
fn test_hash_schema_version_is_29() {
    assert_eq!(HASH_SCHEMA_VERSION, 41u8);
}

#[test]
/// Hash soundness: `MayPayThenEffect` and `CounterUnlessPays` must hash distinctly
/// from each other and encode all their fields (PB-AC1's sole HIGH finding was a
/// dropped hash field -- both new variants hash every field, including the
/// execution-inert `cost` on `CounterUnlessPays`).
fn test_hash_distinguishes_new_effect_variants() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let hash_of = |e: &Effect| -> [u8; 32] {
        let mut h = Hasher::new();
        e.hash_into(&mut h);
        *h.finalize().as_bytes()
    };

    let may_pay_then = Effect::MayPayThenEffect {
        cost: Cost::PayLife(2),
        payer: PlayerTarget::Controller,
        then: Box::new(draw_one()),
    };
    let counter_unless_a = Effect::CounterUnlessPays {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        cost: Cost::PayLife(2),
    };
    let counter_unless_b = Effect::CounterUnlessPays {
        target: CardEffectTarget::DeclaredTarget { index: 0 },
        cost: Cost::Mana(ManaCost {
            generic: 3,
            ..Default::default()
        }),
    };

    assert_ne!(
        hash_of(&may_pay_then),
        hash_of(&counter_unless_a),
        "MayPayThenEffect and CounterUnlessPays must hash differently"
    );
    assert_ne!(
        hash_of(&counter_unless_a),
        hash_of(&counter_unless_b),
        "CounterUnlessPays must encode its `cost` field in the hash \
         (PB-AC1's HIGH finding was exactly a dropped hash field)"
    );
}
