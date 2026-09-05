//! PB-DX53 (`scutemob-231`) — CR 508.3d/508.4, ruling 2007-10-01 (`OOS-DX21-1`):
//! the extra-combat raid-count split.
//!
//! `rules/combat.rs::handle_declare_attackers` used to write a single
//! `PlayerState.attackers_declared_this_turn: u32`, ASSIGNED (not accumulated) at
//! each declaration. On a turn with an extra combat phase (CR 500.8), attacking
//! with three creatures in combat 1 and one in combat 2 dropped the count to
//! **one**, so `windbrisk_heights` (ruling 2007-10-01: "at any point in the turn")
//! went dead for the rest of the turn -- and the field could not deduplicate by
//! creature at all, because a `u32` does not know which creatures it counted.
//!
//! **The root defect was one DSL identifier carrying two CR concepts.**
//! `Condition::YouAttackedWithNOrMore(u32)` had exactly two readers wanting
//! OPPOSITE semantics: `legions_landing` (CR 508.3d, per-DECLARATION) and
//! `windbrisk_heights` (ruling 2007-10-01, per-TURN, deduplicated). Making the
//! field accumulate would have repaired Windbrisk and REGRESSED Legion's Landing
//! (2 attackers in combat 1 + 2 in combat 2 would wrongly transform it). So the
//! DSL split: `Condition::YouAttackedWithNOrMoreThisDeclaration(u32)` (renamed,
//! reads `PlayerState.latest_attacker_declaration_size`, semantics unchanged) and
//! `Condition::YouAttackedWithNOrMoreCreaturesThisTurn(u32)` (new, reads
//! `PlayerState.creatures_declared_as_attackers_this_turn: OrdSet<ObjectId>`,
//! deduplicated by CR 400.7 identity, CR 508.4 entrants excluded by construction --
//! the write site reads the DECLARATION command's own attacker list, never
//! `combat.attackers`, which also holds CR 508.4 entrants via
//! `CombatState::add_attacker`, PB-DX51).
//!
//! `memory/primitives/pb-DX53-plan.md` §8 is authoritative for t1-t7 below.

use mtg_engine::effects::{check_condition, EffectContext};
use mtg_engine::rules::turn_actions::reset_turn_state;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AttackTarget, CardDefinition, CardRegistry,
    CombatState, Command, Condition, GameState, GameStateBuilder, KeywordAbility, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};
use std::collections::HashMap;
use std::sync::Arc;

// ── Helpers (mirrors pb_dx21_declare_attackers_once_per_combat.rs) ────────────

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
    // Pull `card_id` from the REAL `CardDefinition`, never derive it from the
    // name via `card_name_to_id` -- a DFC's id incorporates BOTH face names
    // (`legions-landing-adanto-the-first-fort`), which the naming convention
    // cannot reconstruct from the front face's name alone.
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for {name:?}"));
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(def.card_id.clone()),
        defs,
    )
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
}

fn declare_cmd(player: PlayerId, attackers: Vec<(ObjectId, AttackTarget)>) -> Command {
    Command::DeclareAttackers {
        player,
        attackers,
        enlist_choices: vec![],
        exert_choices: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }
}

fn declare(
    state: GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
) -> GameState {
    process_command(state, declare_cmd(player, attackers))
        .expect("DeclareAttackers should succeed")
        .0
}

/// Simulate the `BeginningOfCombat` re-init CR 500.8 performs for an extra combat
/// phase: a fresh `CombatState` replaces the old one (clearing
/// `attackers_declared`, per PB-DX21/PB-DX51), and priority returns to the active
/// player. `PlayerState`'s turn-scoped fields are untouched -- they live outside
/// `CombatState` precisely so they survive this boundary.
fn begin_new_combat(state: &mut GameState, active_player: PlayerId) {
    *state.combat_mut() = Some(CombatState::new(active_player));
    state.turn_mut().priority_holder = Some(active_player);
}

fn pass_all(state: GameState, players: &[PlayerId]) -> GameState {
    let mut current = state;
    for &pl in players {
        let (s, _) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
    }
    current
}

/// Resolve everything currently on the stack by passing priority in turn order.
fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players);
    }
    state
}

fn is_transformed(state: &GameState, id: ObjectId) -> bool {
    state
        .objects()
        .get(&id)
        .map(|o| o.is_transformed)
        .unwrap_or(false)
}

/// A minimal `EffectContext` sufficient for `check_condition` on the two
/// PB-DX53 `Condition` variants, neither of which reads `ctx.targets` or any
/// resolution-scoped field.
fn minimal_ctx(controller: PlayerId, source: ObjectId) -> EffectContext {
    EffectContext::new(controller, source, vec![])
}

// ─────────────────────────────────────────────────────────────────────────────
// t1 — extra combat: 3 in combat 1 + 1 in combat 2 -> per-turn set is 4
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// Ruling 2007-10-01 (Windbrisk Heights): "you'll get to play the card if you
/// declared three different creatures as attackers at any point in the turn."
/// 3 distinct creatures in combat 1 + 1 MORE distinct creature in combat 2 must
/// accumulate to 4 in `creatures_declared_as_attackers_this_turn`, and
/// `Condition::YouAttackedWithNOrMoreCreaturesThisTurn(3)` must read TRUE off
/// that set even though the SECOND declaration alone only declared 1 creature.
fn t1_extra_combat_accumulates_and_condition_is_true() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear A", 2, 2))
        .object(ObjectSpec::creature(p1, "Bear B", 2, 2))
        .object(ObjectSpec::creature(p1, "Bear C", 2, 2))
        .object(ObjectSpec::creature(p1, "Bear D", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let a = find_by_name(&state, "Bear A");
    let b = find_by_name(&state, "Bear B");
    let c = find_by_name(&state, "Bear C");
    let d = find_by_name(&state, "Bear D");

    // Combat 1: declare three.
    let state = declare(
        state,
        p1,
        vec![
            (a, AttackTarget::Player(p2)),
            (b, AttackTarget::Player(p2)),
            (c, AttackTarget::Player(p2)),
        ],
    );

    // CR 500.8: a fresh CombatState for the extra combat.
    let mut state = state;
    begin_new_combat(&mut state, p1);

    // Combat 2: declare one MORE, distinct creature.
    let state = declare(state, p1, vec![(d, AttackTarget::Player(p2))]);

    let set = &state
        .player(p1)
        .unwrap()
        .creatures_declared_as_attackers_this_turn;
    assert_eq!(
        set.len(),
        4,
        "3 (combat 1) + 1 NEW (combat 2) must accumulate to 4 distinct creatures, got {set:?}"
    );
    assert!(set.contains(&a) && set.contains(&b) && set.contains(&c) && set.contains(&d));

    let ctx = minimal_ctx(p1, a);
    assert!(
        check_condition(
            &state,
            &Condition::YouAttackedWithNOrMoreCreaturesThisTurn(3),
            &ctx
        ),
        "ruling 2007-10-01: 4 distinct creatures declared this turn satisfies 'three or more \
         at any point in the turn', even though the SECOND declaration alone was only 1"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t2 — dedup: the SAME creature declared in both combats counts once
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// Ruling 2007-10-01, verbatim: "A creature declared as an attacker in two
/// different attack phases counts only once." A Vigilant creature (so it stays
/// untapped and eligible to attack again) declared in combat 1 and again in
/// combat 2 must leave the per-turn set at length 1, not 2.
fn t2_dedup_same_creature_twice_counts_once() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Vigilant Bear", 2, 2).with_keyword(KeywordAbility::Vigilance),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let v = find_by_name(&state, "Vigilant Bear");

    let state = declare(state, p1, vec![(v, AttackTarget::Player(p2))]);
    assert!(
        !state.objects().get(&v).unwrap().status.tapped,
        "Vigilance must leave the attacker untapped, or this test cannot exercise the \
         SAME-creature-twice case at all"
    );

    let mut state = state;
    begin_new_combat(&mut state, p1);

    let state = declare(state, p1, vec![(v, AttackTarget::Player(p2))]);

    let set = &state
        .player(p1)
        .unwrap()
        .creatures_declared_as_attackers_this_turn;
    assert_eq!(
        set.len(),
        1,
        "the SAME creature declared in two different attack phases must count ONLY ONCE \
         (ruling 2007-10-01, verbatim), got {set:?}"
    );
    assert!(set.contains(&v));
}

// ─────────────────────────────────────────────────────────────────────────────
// t3 — CR 508.4: an entrant does NOT enter the set
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// Ruling 2007-10-01, third sentence: "A creature that entered attacking ...
/// doesn't count because you never attacked with it." `CombatState::add_attacker`
/// (PB-DX51's single production mutator for `combat.attackers`, also used by
/// every CR 508.4 entrant site -- two token paths, Myriad, Ninjutsu) is called
/// directly here to simulate an entrant WITHOUT going through
/// `Command::DeclareAttackers`. The CR 508.4 exclusion holds BY CONSTRUCTION:
/// `handle_declare_attackers`'s write site reads only its own COMMAND parameter,
/// never `combat.attackers`, so an entrant can never reach the set.
fn t3_cr_508_4_entrant_does_not_enter_the_set() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Real Attacker", 2, 2))
        .object(ObjectSpec::creature(p1, "Token Entrant", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let real = find_by_name(&state, "Real Attacker");
    let entrant = find_by_name(&state, "Token Entrant");

    // A real, declared attack -- this DOES enter the set.
    let mut state = declare(state, p1, vec![(real, AttackTarget::Player(p2))]);

    // Simulate a CR 508.4 entrant (e.g. Ninjutsu, a token created attacking)
    // entering combat OUTSIDE the declare-attackers command entirely.
    if let Some(combat) = state.combat_mut().as_mut() {
        combat.add_attacker(entrant, AttackTarget::Player(p2));
    }

    // The entrant IS in combat...
    assert!(
        state
            .combat()
            .as_ref()
            .unwrap()
            .attackers
            .contains_key(&entrant),
        "sanity: the entrant must actually be in combat.attackers for this test to mean \
         anything"
    );

    // ...but the per-turn DECLARED set contains only the real attacker.
    let set = &state
        .player(p1)
        .unwrap()
        .creatures_declared_as_attackers_this_turn;
    assert_eq!(
        set.len(),
        1,
        "a CR 508.4 entrant must NEVER enter the declared-attacker set: {set:?}"
    );
    assert!(set.contains(&real));
    assert!(
        !set.contains(&entrant),
        "the entrant reached combat.attackers but must NOT reach the declared set"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t4 — per-declaration u32 is still ASSIGNED, not accumulated
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// CR 508.3d's per-declaration field, `latest_attacker_declaration_size`, must
/// keep its PB-OS6 semantics unchanged by this batch: it is OVERWRITTEN by each
/// new declaration, not accumulated. After 3 in combat 1 and 1 in combat 2, it
/// must read 1 (the size of the MOST RECENT declaration), not 4.
fn t4_per_declaration_field_still_assigned_not_accumulated() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear A", 2, 2))
        .object(ObjectSpec::creature(p1, "Bear B", 2, 2))
        .object(ObjectSpec::creature(p1, "Bear C", 2, 2))
        .object(ObjectSpec::creature(p1, "Bear D", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let a = find_by_name(&state, "Bear A");
    let b = find_by_name(&state, "Bear B");
    let c = find_by_name(&state, "Bear C");
    let d = find_by_name(&state, "Bear D");

    let state = declare(
        state,
        p1,
        vec![
            (a, AttackTarget::Player(p2)),
            (b, AttackTarget::Player(p2)),
            (c, AttackTarget::Player(p2)),
        ],
    );
    assert_eq!(
        state.player(p1).unwrap().latest_attacker_declaration_size,
        3,
        "combat 1's own declaration size"
    );

    let mut state = state;
    begin_new_combat(&mut state, p1);

    let state = declare(state, p1, vec![(d, AttackTarget::Player(p2))]);
    assert_eq!(
        state.player(p1).unwrap().latest_attacker_declaration_size,
        1,
        "CR 508.3d reads the MOST RECENT declaration's size (1), not the per-turn total (4) \
         -- overwritten, not accumulated"
    );
    // Non-vacuity: the per-turn SET, in contrast, IS 4 -- the two fields
    // genuinely disagree, which is the whole point of the split.
    assert_eq!(
        state
            .player(p1)
            .unwrap()
            .creatures_declared_as_attackers_this_turn
            .len(),
        4
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t5 — turn boundary clears both
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// Both PB-DX53 fields reset for ALL players at the turn boundary
/// (`turn_actions::reset_turn_state`), mirroring every other per-turn
/// `PlayerState` field in this file (`attacked_this_turn`,
/// `created_token_this_turn`, etc.).
fn t5_turn_boundary_clears_both_fields() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear A", 2, 2))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let a = find_by_name(&state, "Bear A");
    let mut state = declare(state, p1, vec![(a, AttackTarget::Player(p2))]);

    assert_eq!(
        state.player(p1).unwrap().latest_attacker_declaration_size,
        1
    );
    assert_eq!(
        state
            .player(p1)
            .unwrap()
            .creatures_declared_as_attackers_this_turn
            .len(),
        1
    );

    reset_turn_state(&mut state, p1);

    assert_eq!(
        state.player(p1).unwrap().latest_attacker_declaration_size,
        0,
        "the per-declaration field must clear at the turn boundary"
    );
    assert!(
        state
            .player(p1)
            .unwrap()
            .creatures_declared_as_attackers_this_turn
            .is_empty(),
        "the per-turn declared-attacker set must clear at the turn boundary"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t6 / t7 — Legion's Landing across an extra combat (CR 508.3d, unmoved by this batch)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// PB-DX21 review finding M3, re-verified structurally by this batch: Legion's
/// Landing's `Whenever you attack with three or more creatures` is CR 508.3d,
/// per-DECLARATION, and reads the SAME field as before this batch
/// (`latest_attacker_declaration_size`, renamed but behaviourally identical).
/// 2 attackers in combat 1 (transform check: 2 < 3, false) and 2 MORE, distinct,
/// attackers in combat 2 (transform check: 2 < 3, false again) must NOT
/// transform it -- even though 4 DISTINCT creatures attacked this turn in total,
/// which is exactly the count that WOULD satisfy the sibling per-turn
/// `Condition::YouAttackedWithNOrMoreCreaturesThisTurn`. The two fields must
/// disagree here, wrong-way-round from t1/t4's own numbers, which is the proof
/// this is a control and not a duplicate of them.
fn t6_legions_landing_extra_combat_two_plus_two_does_not_transform() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let landing = enrich(p1, "Legion's Landing", ZoneId::Battlefield, &defs);
    let a = ObjectSpec::creature(p1, "Bear A", 2, 2).in_zone(ZoneId::Battlefield);
    let b = ObjectSpec::creature(p1, "Bear B", 2, 2).in_zone(ZoneId::Battlefield);
    let c = ObjectSpec::creature(p1, "Bear C", 2, 2).in_zone(ZoneId::Battlefield);
    let d = ObjectSpec::creature(p1, "Bear D", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(landing)
        .object(a)
        .object(b)
        .object(c)
        .object(d)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let landing_id = find_by_name(&state, "Legion's Landing");
    let a_id = find_by_name(&state, "Bear A");
    let b_id = find_by_name(&state, "Bear B");
    let c_id = find_by_name(&state, "Bear C");
    let d_id = find_by_name(&state, "Bear D");

    // Combat 1: 2 attackers.
    let state = declare(
        state,
        p1,
        vec![
            (a_id, AttackTarget::Player(p2)),
            (b_id, AttackTarget::Player(p2)),
        ],
    );
    let state = resolve_stack(state, &[p1, p2]);
    assert!(
        !is_transformed(&state, landing_id),
        "2 attackers in combat 1 must not transform (2 < 3)"
    );

    // CR 500.8: extra combat.
    let mut state = state;
    begin_new_combat(&mut state, p1);

    // Combat 2: 2 MORE, distinct, attackers.
    let state = declare(
        state,
        p1,
        vec![
            (c_id, AttackTarget::Player(p2)),
            (d_id, AttackTarget::Player(p2)),
        ],
    );
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        !is_transformed(&state, landing_id),
        "CR 508.3d is per-DECLARATION: 2 attackers in combat 2 alone must not transform \
         Legion's Landing, even though 4 distinct creatures attacked this turn in total"
    );
    // Non-vacuity for the claim above: the per-turn set really did reach 4.
    assert_eq!(
        state
            .player(p1)
            .unwrap()
            .creatures_declared_as_attackers_this_turn
            .len(),
        4,
        "sanity: 4 distinct creatures really were declared across the two combats"
    );
}

#[test]
/// Non-vacuity floor for t6: the CR 508.3d mechanism itself still works when the
/// per-declaration count genuinely meets 3 in a SINGLE combat -- proving t6's
/// negative result is a scope discrimination, not a broken trigger.
fn t7_legions_landing_three_in_one_combat_does_transform() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let landing = enrich(p1, "Legion's Landing", ZoneId::Battlefield, &defs);
    let a = ObjectSpec::creature(p1, "Bear A", 2, 2).in_zone(ZoneId::Battlefield);
    let b = ObjectSpec::creature(p1, "Bear B", 2, 2).in_zone(ZoneId::Battlefield);
    let c = ObjectSpec::creature(p1, "Bear C", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(landing)
        .object(a)
        .object(b)
        .object(c)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let landing_id = find_by_name(&state, "Legion's Landing");
    let a_id = find_by_name(&state, "Bear A");
    let b_id = find_by_name(&state, "Bear B");
    let c_id = find_by_name(&state, "Bear C");

    let state = declare(
        state,
        p1,
        vec![
            (a_id, AttackTarget::Player(p2)),
            (b_id, AttackTarget::Player(p2)),
            (c_id, AttackTarget::Player(p2)),
        ],
    );
    let state = resolve_stack(state, &[p1, p2]);

    assert!(
        is_transformed(&state, landing_id),
        "3 attackers declared in ONE combat must transform Legion's Landing (CR 508.3d)"
    );
}
