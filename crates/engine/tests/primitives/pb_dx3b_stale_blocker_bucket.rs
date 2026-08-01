//! PB-DX3b (OOS-DX3-1): the remainder of the `pb-plan-DP6.md:395` stale-blocker
//! bucket, closed the same way PB-DX3 closed `garruks_uprising` /
//! `inventors_fair` — `Condition::YouControlNOrMoreWithFilter { count, filter
//! }` (optionally wrapped in `Condition::Not`) is queue-time evaluable
//! (`effects::condition_is_queue_time_evaluable`) and has been available since
//! well before every blocker note in this file claimed otherwise. This file
//! loads every def from the real corpus via `all_cards()` and drives them
//! through the full command/resolution pipeline — never a re-declared copy.
//!
//! - `jadar_ghoulcaller_of_nephalia` — **`Complete`, live-wrong pre-fix.**
//!   The stored `oracle_text` AND the blocker note both chased a filter the
//!   printed card never had ("no tokens named Shambling Ghast"); MCP-verified
//!   printed text is "if you control no creatures with decayed." T1-T4, T13
//!   (opponent's decayed creature must not suppress -- fix cycle, review
//!   Finding 2).
//! - `ophiomancer` — `partial` -> `Complete`. Its own note already said
//!   "Blocker stale". T5-T7.
//! - `dwynen_s_elite` — `inert` (ability absent) -> `Complete`. Authored from
//!   scratch, same shape as PB-DX3's `inventors_fair`. T8-T10.
//! - `emeria_the_sky_ruin` — **`Complete` only by `#[default]`, live-wrong
//!   pre-fix** (found this batch, not named by the seed) -> explicit
//!   `partial` (the "you may" clause remains genuinely unimplemented; see the
//!   def's own completeness note). T11-T12, T14 (opponent's Plains must not
//!   count -- fix cycle, review Finding 7).
//!
//! ## Pre-fix observations (recorded before/alongside the card-def edits, per
//! plan §3 -- and per the PB-DX3 fix-cycle MEDIUM: every "pre-fix, X happened"
//! sentence below was RUN, not reasoned to. Each was produced by temporarily
//! reverting the relevant def's `intervening_if`/ability back to its pre-fix
//! shape, re-running the exact scenario below with panicking assertions
//! swapped for `eprintln!` so execution could reach the read, reading the
//! actual numbers, then restoring the def and the test.
//!
//! - **T1** (Jadar, controls a creature WITH decayed) -- **RE-VERIFIED
//!   EMPIRICALLY** (both halves, via a second instrumented re-run with the
//!   panicking assertions swapped for `eprintln!` so execution could reach
//!   the resolution). With `intervening_if` reverted to `None`: `stack len =
//!   1` (queued unconditionally, even with a Decayed creature already
//!   controlled) and, after resolving, `decayed count = 2` (a second Zombie
//!   token was created on top of the pre-placed one).
//! - **T3** (Jadar, no decayed creature at queue time, one appears before
//!   resolution) -- **RE-VERIFIED EMPIRICALLY.** With `intervening_if`
//!   reverted to `None`, the trigger queued (as with T2) and then, after the
//!   decayed creature was created between queue and resolution (simulating a
//!   board change during the end step), resolution still created ANOTHER
//!   Zombie token -- no re-check happened. Post-fix, the resolution-time
//!   re-check (`InterveningIf::CardDef`, PB-DX1) declines.
//!   **Deviation from the plan's literal wording**: plan §3's T1 row says
//!   "decayed creature present at queue time, gone by resolution" -- but
//!   Jadar's intervening-if is NEGATED ("no creatures with decayed"), so a
//!   decayed creature present AT QUEUE TIME means the condition is false and
//!   the trigger would not even queue, leaving nothing for a resolution-time
//!   re-check to decline. The CR-correct analogue of "queue-time true,
//!   resolution-time false" for a negated condition is "no decayed creature
//!   at queue time (condition true, trigger queues), a decayed creature
//!   appears before resolution (condition now false, resolution declines)" --
//!   implemented here instead, with this note recording the correction rather
//!   than silently following the plan's inverted wording.
//! - **T5** (Ophiomancer, controls a Snake, own upkeep) -- **RE-VERIFIED
//!   EMPIRICALLY** (both halves, same instrumented-rerun technique as T1).
//!   With `intervening_if` reverted to `None`: `stack len = 1` (queued
//!   unconditionally, even with a Snake already controlled) and, after
//!   resolving, `snake count = 2` (a second Snake token was created on top of
//!   the pre-placed one).
//! - **T8/T9/T10** (Dwynen's Elite) -- vacuous pre-fix, labelled honestly, not
//!   manufactured: `abilities` was EMPTY before this batch (same shape as
//!   `inventors_fair`'s T5 in PB-DX3), so there was no triggered ability to
//!   observe any pre-fix number against. No revert was performed for these
//!   three.
//! - **T11** (Emeria, 6 Plains, upkeep) -- **RE-VERIFIED EMPIRICALLY** (both
//!   halves, same instrumented-rerun technique as T1/T5). With
//!   `intervening_if` reverted to `None` (this def had no explicit
//!   `completeness:` field pre-fix, and no intervening-if at all -- the
//!   `#[default] Complete` trap named in the def's completeness comment):
//!   `stack len = 1` (queued unconditionally with only 6 Plains controlled,
//!   the graveyard target auto-filled per CR 601.2c/PB-DP8) and, after
//!   resolving, "Dead Beast" was on the battlefield and NOT in the graveyard
//!   -- the live-wrong "free reanimation every upkeep" behaviour the seed and
//!   this batch's plan both named, genuinely reproduced.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, CardDefinition,
    CardEffectTarget, Command, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor,
    ObjectId, ObjectSpec, PlayerId, Step, SubType, TokenSpec, ZoneId,
};
use std::collections::HashMap;

// ── Shared helpers ───────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Load a card def from the real corpus by exact name. Never re-declared
/// inline -- the probes must exercise the shipped def.
fn card_def(name: &str) -> CardDefinition {
    all_cards()
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("{name} should be in the corpus"))
}

fn one_def_map(def: &CardDefinition) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Place `def` on the battlefield under `owner`'s control, fully enriched.
fn place_on_battlefield(owner: PlayerId, def: &CardDefinition) -> ObjectSpec {
    let defs = one_def_map(def);
    enrich_spec_from_def(
        ObjectSpec::card(owner, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    )
}

/// Put `def` in `owner`'s hand, fully enriched (for casting).
fn place_in_hand(owner: PlayerId, def: &CardDefinition) -> ObjectSpec {
    let defs = one_def_map(def);
    enrich_spec_from_def(
        ObjectSpec::card(owner, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Hand(owner)),
        &defs,
    )
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

fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (stuck at {:?}, wanted {:?})",
            state.turn().step,
            target
        );
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder at step {:?}", state.turn().step));
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
    }
}

fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

fn empty_cast_spell_data(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
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
    }))
}

/// Destroy a specific object directly, bypassing the stack.
fn destroy_object(state: &mut GameState, controller: PlayerId, obj_id: ObjectId) {
    execute_effect(
        state,
        &mtg_engine::Effect::DestroyPermanent {
            target: CardEffectTarget::Source,
            cant_be_regenerated: false,
        },
        &mut EffectContext::new(controller, obj_id, vec![]),
    );
}

/// Create a token directly, bypassing the stack -- used to simulate a board
/// change happening between a trigger's queue time and its resolution.
fn create_token_directly(state: &mut GameState, controller: PlayerId, spec: TokenSpec) {
    let some_source = *state.objects().keys().next().unwrap();
    execute_effect(
        state,
        &mtg_engine::Effect::CreateToken { spec },
        &mut EffectContext::new(controller, some_source, vec![]),
    );
}

fn decayed_zombie_spec() -> TokenSpec {
    mtg_engine::zombie_decayed_token_spec(1)
}

fn count_decayed_creatures(state: &GameState, controller: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(id, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == controller
                && calculate_characteristics(state, **id)
                    .map(|c| c.keywords.contains(&KeywordAbility::Decayed))
                    .unwrap_or(false)
        })
        .count()
}

fn count_snakes(state: &GameState, controller: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(id, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == controller
                && calculate_characteristics(state, **id)
                    .map(|c| c.subtypes.contains(&SubType("Snake".to_string())))
                    .unwrap_or(false)
        })
        .count()
}

fn count_elf_warriors(state: &GameState, controller: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(id, obj)| {
            obj.zone == ZoneId::Battlefield
                && obj.controller == controller
                && calculate_characteristics(state, **id)
                    .map(|c| c.name == "Elf Warrior")
                    .unwrap_or(false)
        })
        .count()
}

// ── T1: Jadar -- end step, already controls a decayed creature: no trigger ──

/// CR 603.4: with a decayed creature already controlled, "you control no
/// creatures with decayed" is false immediately after the end step begins, so
/// the trigger must not even be queued.
#[test]
fn test_dx3b_jadar_controls_decayed_creature_no_trigger() {
    let def = card_def("Jadar, Ghoulcaller of Nephalia");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(
            ObjectSpec::creature(p(1), "Old Zombie", 2, 2).with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let state = advance_to_step(state, Step::End);
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: a decayed creature is already controlled -- Jadar's trigger \
         must not queue; stack is {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_decayed_creatures(&state, p(1)),
        1,
        "no second Zombie should have been created"
    );
}

// ── T2: Jadar -- end step, no decayed creature: queues, creates a token ─────

/// Non-regression positive direction: with no decayed creature controlled,
/// the trigger queues, resolves, and creates one Zombie token with decayed.
#[test]
fn test_dx3b_jadar_no_decayed_creature_creates_zombie() {
    let def = card_def("Jadar, Ghoulcaller of Nephalia");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let state = advance_to_step(state, Step::End);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4: with no decayed creature controlled, Jadar's trigger \
         should be on the stack"
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_decayed_creatures(&state, p(1)),
        1,
        "exactly one Zombie token with decayed should have been created"
    );
}

// ── T3: Jadar -- no decayed creature at queue time, one appears before ──────
// ── resolution: resolution re-check declines (see module doc deviation note) ─

/// CR 603.4, resolution-time re-check. See the module doc for why this test's
/// scenario is the CR-correct analogue of the plan's literal wording, not a
/// verbatim implementation of it.
#[test]
fn test_dx3b_jadar_decayed_creature_appears_before_resolution_no_second_token() {
    let def = card_def("Jadar, Ghoulcaller of Nephalia");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let mut state = advance_to_step(state, Step::End);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "the trigger should have queued while no decayed creature was controlled"
    );

    // Simulate a board change during the end step: a decayed creature appears
    // (e.g. from an unrelated effect) before Jadar's trigger resolves.
    create_token_directly(&mut state, p(1), decayed_zombie_spec());
    assert_eq!(
        count_decayed_creatures(&state, p(1)),
        1,
        "sanity: exactly one decayed creature now controlled, before resolution"
    );

    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_decayed_creatures(&state, p(1)),
        1,
        "CR 603.4: a decayed creature was controlled at resolution -- Jadar's \
         own trigger must not have created a second one"
    );
}

// ── T4: Jadar -- stored oracle_text matches the printed card, not the stale ──
// ── "Shambling Ghast" text ───────────────────────────────────────────────────

/// Directly readable from source, no dynamic claim: pre-fix the field read
/// "...if you control no tokens named Shambling Ghast..." (captured verbatim
/// before this batch's edit); MCP-verified printed text is "...if you control
/// no creatures with decayed...".
#[test]
fn test_dx3b_jadar_oracle_text_matches_printed_card() {
    let def = card_def("Jadar, Ghoulcaller of Nephalia");
    assert!(
        def.oracle_text.contains("no creatures with decayed"),
        "oracle_text should match the printed card; got: {}",
        def.oracle_text
    );
    assert!(
        !def.oracle_text.contains("Shambling Ghast"),
        "the stale 'no tokens named Shambling Ghast' text must be gone; got: {}",
        def.oracle_text
    );
}

// ── T5: Ophiomancer -- own upkeep, controls a Snake: no trigger ─────────────

/// CR 603.4 (ruling 2013-10-17 #1): with a Snake already controlled, the
/// trigger must not queue.
#[test]
fn test_dx3b_ophiomancer_controls_snake_no_trigger() {
    let def = card_def("Ophiomancer");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(
            ObjectSpec::creature(p(1), "Garter Snake", 1, 1)
                .with_subtypes(vec![SubType("Snake".to_string())]),
        )
        .active_player(p(1))
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let state = advance_to_step(state, Step::Upkeep);
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: a Snake is already controlled -- Ophiomancer's trigger \
         must not queue; stack is {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_snakes(&state, p(1)),
        1,
        "no second Snake should have been created"
    );
}

// ── T6: Ophiomancer -- own upkeep, no Snake: queues, creates a Snake ────────

/// Non-regression positive direction, and the disambiguator for T5.
#[test]
fn test_dx3b_ophiomancer_no_snake_creates_one() {
    let def = card_def("Ophiomancer");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .active_player(p(1))
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let state = advance_to_step(state, Step::Upkeep);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4: with no Snake controlled, Ophiomancer's trigger should be \
         on the stack"
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_snakes(&state, p(1)),
        1,
        "exactly one Snake token with deathtouch should have been created"
    );
}

// ── T7: Ophiomancer -- opponent's upkeep, gate reads the CONTROLLER's board ─

/// CR 603.4 + `AtBeginningOfEachUpkeep`: Ophiomancer (controlled by p1) must
/// fire on the ACTIVE player's (p2's) upkeep too -- and the intervening-if
/// must gate against p1's board, not p2's. p2 (active, whose upkeep this is)
/// controls a Snake; p1 (Ophiomancer's controller) does not. If the engine
/// mistakenly checked the active player's board, this would wrongly decline;
/// if it correctly checks the controller's board, it fires.
#[test]
fn test_dx3b_ophiomancer_opponents_upkeep_gates_on_controller_not_active_player() {
    let def = card_def("Ophiomancer");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(
            ObjectSpec::creature(p(2), "Opponent's Snake", 1, 1)
                .with_subtypes(vec![SubType("Snake".to_string())]),
        )
        .active_player(p(2))
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(2));

    let state = advance_to_step(state, Step::Upkeep);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4 + AtBeginningOfEachUpkeep: Ophiomancer must fire on p2's \
         upkeep too, gated on p1's (its controller's) board -- p1 controls no \
         Snake, so it should trigger even though p2 (active) does; stack is \
         {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(2), p(1)]);
    assert_eq!(
        count_snakes(&state, p(1)),
        1,
        "the created Snake should belong to Ophiomancer's controller (p1), \
         not the active player (p2)"
    );
    assert_eq!(
        count_snakes(&state, p(2)),
        1,
        "p2's own Snake should be untouched (sanity: still exactly the one \
         placed in the fixture)"
    );
}

// ── T8: Dwynen's Elite -- ETB alone (no other Elf): no token ────────────────

/// CR 109.1 "another" -- proven via `exclude_self: true`. Vacuous pre-fix
/// (the ability did not exist), labelled honestly per the module doc.
#[test]
fn test_dx3b_dwynen_s_elite_etb_alone_no_token() {
    let def = card_def("Dwynen's Elite");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_in_hand(p(1), &def))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn_mut().priority_holder = Some(p(1));

    let spell_id = find_object(&state, "Dwynen's Elite");
    let (state, _) =
        process_command(state, empty_cast_spell_data(p(1), spell_id)).expect("cast should succeed");
    let (state, _) = pass_all(state, &[p(1), p(2)]);

    assert!(
        state.stack_objects().is_empty(),
        "CR 109.1: Dwynen's Elite alone controls no OTHER Elf -- the ETB \
         trigger must not queue; stack is {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_elf_warriors(&state, p(1)),
        0,
        "no Elf Warrior token should have been created"
    );
}

// ── T9: Dwynen's Elite -- ETB with another Elf: creates a token ────────────

/// Non-regression positive direction. Vacuous pre-fix, labelled honestly.
#[test]
fn test_dx3b_dwynen_s_elite_etb_with_another_elf_creates_token() {
    let def = card_def("Dwynen's Elite");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_in_hand(p(1), &def))
        .object(
            ObjectSpec::creature(p(1), "Elvish Ranger", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn_mut().priority_holder = Some(p(1));

    let spell_id = find_object(&state, "Dwynen's Elite");
    let (state, _) =
        process_command(state, empty_cast_spell_data(p(1), spell_id)).expect("cast should succeed");
    let (state, _) = pass_all(state, &[p(1), p(2)]);

    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4: with another Elf controlled, the ETB trigger should be on \
         the stack"
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_elf_warriors(&state, p(1)),
        1,
        "exactly one Elf Warrior token should have been created"
    );
}

// ── T10: Dwynen's Elite -- another Elf at queue time, gone by resolution ────

/// CR 603.4 resolution-time re-check. Vacuous pre-fix, labelled honestly.
#[test]
fn test_dx3b_dwynen_s_elite_elf_leaves_before_resolution_no_token() {
    let def = card_def("Dwynen's Elite");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_in_hand(p(1), &def))
        .object(
            ObjectSpec::creature(p(1), "Fleeting Elf", 2, 2)
                .with_subtypes(vec![SubType("Elf".to_string())]),
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn_mut().priority_holder = Some(p(1));

    let spell_id = find_object(&state, "Dwynen's Elite");
    let (state, _) =
        process_command(state, empty_cast_spell_data(p(1), spell_id)).expect("cast should succeed");
    let (mut state, _) = pass_all(state, &[p(1), p(2)]);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "the trigger should have queued while the other Elf was still controlled"
    );

    let elf_id = find_object(&state, "Fleeting Elf");
    destroy_object(&mut state, p(1), elf_id);

    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_elf_warriors(&state, p(1)),
        0,
        "CR 603.4: the other Elf left before the trigger resolved -- no token \
         should have been created"
    );
}

// ── T11: Emeria -- upkeep, 6 Plains: no trigger ─────────────────────────────

/// CR 603.4: 6 Plains is below the printed threshold of 7 -- the trigger must
/// not queue. Pre-fix (see module doc): the def had no intervening-if at all
/// (`Complete` only by `#[default]`), so this reanimated unconditionally --
/// the live-wrong half this batch closes.
#[test]
fn test_dx3b_emeria_six_plains_no_trigger() {
    let def = card_def("Emeria, the Sky Ruin");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(ObjectSpec::creature(p(1), "Dead Beast", 2, 2).in_zone(ZoneId::Graveyard(p(1))));
    for i in 0..6u32 {
        builder = builder.object(
            ObjectSpec::land(p(1), &format!("Plains {i}"))
                .with_subtypes(vec![SubType("Plains".to_string())]),
        );
    }
    let mut state = builder
        .active_player(p(1))
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let state = advance_to_step(state, Step::Upkeep);
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: 6 Plains is below the threshold of 7 -- Emeria's trigger \
         must not queue; stack is {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Dead Beast" && o.zone == ZoneId::Graveyard(p(1))),
        "the creature must still be in the graveyard -- no reanimation should \
         have happened"
    );
}

// ── T12: Emeria -- upkeep, 7 Plains: queues, reanimates ─────────────────────

/// Non-regression positive direction: with 7 Plains controlled, the trigger
/// queues (with its single legal graveyard target auto-filled -- CR 601.2c,
/// PB-DP8 -- since exactly one creature card sits in the graveyard) and
/// resolves, returning the creature to the battlefield.
#[test]
fn test_dx3b_emeria_seven_plains_reanimates() {
    let def = card_def("Emeria, the Sky Ruin");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(ObjectSpec::creature(p(1), "Dead Beast", 2, 2).in_zone(ZoneId::Graveyard(p(1))));
    for i in 0..7u32 {
        builder = builder.object(
            ObjectSpec::land(p(1), &format!("Plains {i}"))
                .with_subtypes(vec![SubType("Plains".to_string())]),
        );
    }
    let mut state = builder
        .active_player(p(1))
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let state = advance_to_step(state, Step::Upkeep);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4: with 7 Plains controlled, Emeria's trigger should be on \
         the stack"
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Dead Beast" && o.zone == ZoneId::Battlefield),
        "CR 603.4: 7 Plains satisfies the threshold -- the creature should \
         have been reanimated"
    );
    assert!(
        !state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Dead Beast" && o.zone == ZoneId::Graveyard(p(1))),
        "the graveyard copy should be gone (CR 400.7 -- new object on the \
         battlefield)"
    );
}

// ── T13: Jadar -- an OPPONENT's decayed creature must not suppress ─────────

/// CR 603.4: the printed clause is "if **you** control no creatures with
/// decayed" -- an opponent's board is not consulted. Added per PB-DX3b review
/// Finding 2: this is the single most load-bearing claim in the def's own
/// comment (`TargetFilter.controller` is deliberately left at `Any` because
/// the `YouControlNOrMoreWithFilter` evaluator does its own
/// `obj.controller == controller` check, effects/mod.rs), and until this
/// test, nothing pinned it -- mirrors T7's shape for Ophiomancer. p2 controls
/// a decayed creature; p1 (Jadar's controller) does not, so the trigger must
/// still queue and resolve at p1's end step.
#[test]
fn test_dx3b_jadar_opponents_decayed_creature_does_not_suppress() {
    let def = card_def("Jadar, Ghoulcaller of Nephalia");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(
            ObjectSpec::creature(p(2), "Opponent's Old Zombie", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let state = advance_to_step(state, Step::End);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4: p2 (not Jadar's controller) controls a decayed creature -- \
         p1 controls none, so Jadar's trigger must still queue at p1's end \
         step; stack is {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert_eq!(
        count_decayed_creatures(&state, p(1)),
        1,
        "p1 should have exactly one decayed creature -- the newly created \
         Zombie -- unaffected by p2's own decayed creature"
    );
    assert_eq!(
        count_decayed_creatures(&state, p(2)),
        1,
        "p2's own decayed creature should be untouched (sanity: still \
         exactly the one placed in the fixture)"
    );
}

// ── T14: Emeria -- an OPPONENT's Plains must not count toward the 7 ────────

/// CR 603.4: "if you control seven or more Plains" -- an opponent's Plains
/// are not consulted. Added per PB-DX3b review Finding 7. p1 controls 6
/// Plains (below the threshold) and p2 controls 2 Plains; the board-wide
/// total is 8, but the trigger must not queue because p1's own count is
/// still 6.
#[test]
fn test_dx3b_emeria_opponents_plains_do_not_count() {
    let def = card_def("Emeria, the Sky Ruin");
    let registry = mtg_engine::CardRegistry::new(vec![def.clone()]);
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(place_on_battlefield(p(1), &def))
        .object(ObjectSpec::creature(p(1), "Dead Beast", 2, 2).in_zone(ZoneId::Graveyard(p(1))));
    for i in 0..6u32 {
        builder = builder.object(
            ObjectSpec::land(p(1), &format!("Plains {i}"))
                .with_subtypes(vec![SubType("Plains".to_string())]),
        );
    }
    for i in 0..2u32 {
        builder = builder.object(
            ObjectSpec::land(p(2), &format!("Opponent's Plains {i}"))
                .with_subtypes(vec![SubType("Plains".to_string())]),
        );
    }
    let mut state = builder
        .active_player(p(1))
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let state = advance_to_step(state, Step::Upkeep);
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: p1 controls only 6 Plains -- p2's 2 Plains must not count \
         toward p1's threshold of 7; stack is {:?}",
        state.stack_objects()
    );
    let state = resolve_stack(state, &[p(1), p(2)]);
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Dead Beast" && o.zone == ZoneId::Graveyard(p(1))),
        "the creature must still be in the graveyard -- no reanimation \
         should have happened"
    );
}
