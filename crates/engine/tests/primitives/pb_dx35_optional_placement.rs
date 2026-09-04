//! PB-DX35 Half B (`OOS-DX4-5`): `Effect::LookAtTopThenPlace.optional` is a real CR
//! 118.12 player decision.
//!
//! Through PB-OS8 the arm destructured `optional: _` and always placed the best
//! candidate when one existed -- so five `Complete` defs printed "you may" and the
//! engine never asked. This batch makes `optional == true` with a nonempty candidate
//! set ask `EffectChoiceQuestion::ChooseObject { count: 1, up_to: true, .. }` on the
//! CR 608.2d suspend-and-replay channel PB-DX45's `place_cost` already uses, addressed
//! to the LOOKING player (`p`), not `ctx.controller`.
//!
//! Every probe here asserts by RESOLUTION EFFECT (the zone an object ends up in, or a
//! life total), never by the offer alone -- an offer-shaped assertion would pass on a
//! fixture where the question is asked and the answer thrown away (the PB-DX45
//! lesson, restated because this file's subject is the same primitive).

use mtg_engine::effects::{
    default_effect_choice_answer, execute_effect, execute_effect_answering,
    execute_effect_with_default_choices, EffectContext,
};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, CardType, Command, Effect, EffectAmount, EffectChoiceAnswer,
    EffectChoiceQuestion, GameEvent, GameState, GameStateBuilder, ManaCost, ManaPool, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, Step, TargetFilter, ZoneId, ZoneTarget,
};
use std::collections::HashMap;

// ── Helpers (self-contained, mirroring `pb_os8_look_at_top_then_place.rs` /
// `pb_dx45_optional_cost.rs` -- these engine integration-test files do not share a
// util module, per existing convention) ─────────────────────────────────────────

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

fn in_zone(state: &GameState, name: &str, zone: ZoneId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == zone)
}

fn in_hand(state: &GameState, name: &str, owner: PlayerId) -> bool {
    in_zone(state, name, ZoneId::Hand(owner))
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    in_zone(state, name, ZoneId::Graveyard(owner))
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    in_zone(state, name, ZoneId::Battlefield)
}

fn land_card(owner: PlayerId, name: &str, zone: ZoneId) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .with_card_id(CardId(format!(
            "dx35-{}",
            name.to_lowercase().replace(' ', "-")
        )))
        .with_types(vec![CardType::Land])
        .in_zone(zone)
}

fn creature_with_mv(owner: PlayerId, name: &str, mv: u32, zone: ZoneId) -> ObjectSpec {
    ObjectSpec::creature(owner, name, mv as i32, mv as i32)
        .with_card_id(CardId(format!(
            "dx35-{}",
            name.to_lowercase().replace(' ', "-")
        )))
        .with_mana_cost(ManaCost {
            generic: mv,
            ..Default::default()
        })
        .in_zone(zone)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn real_card_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    let def = defs
        .get(name)
        .unwrap_or_else(|| panic!("no real CardDefinition for '{}'", name));
    let base = ObjectSpec::card(owner, name)
        .in_zone(zone)
        .with_card_id(def.card_id.clone());
    enrich_spec_from_def(base, defs)
}

/// A "look at the top `n`, place a matching Land, `optional` per the caller,
/// destination = hand, rest_to = graveyard" fixture effect. Graveyard, not the
/// library bottom that the real 5 defs mostly use, is the destination-independent
/// choice deliberately: it makes a DECLINE distinguishable from "still in the
/// library" (a bottomed card and an untouched card can coincide in id since
/// PB-DX15a) with a single zone check.
fn base_effect(count: i32, optional: bool) -> Effect {
    Effect::LookAtTopThenPlace {
        player: PlayerTarget::Controller,
        count: EffectAmount::Fixed(count),
        filter: TargetFilter {
            has_card_type: Some(CardType::Land),
            ..Default::default()
        },
        place_cost: None,
        destination: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        rest_to: ZoneTarget::Graveyard {
            owner: PlayerTarget::Controller,
        },
        optional,
    }
}

fn bare_state(p1: PlayerId, p2: PlayerId, objects: Vec<ObjectSpec>) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for spec in objects {
        builder = builder.object(spec);
    }
    builder.build().unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
// t1 -- `optional: false` places the winner and asks NOTHING
// ═══════════════════════════════════════════════════════════════════════════

/// CR 118.12: `optional: false` keeps the M7 deterministic take-when-able fallback
/// byte-for-byte -- no `EffectChoiceQuestion` is ever raised.
///
/// **Stated as a CONTROL, not a discriminator**: this behaviour is unchanged by B1
/// and stays GREEN under every revert row in this batch's own matrix (it pins the
/// `optional: false` arm, which none of them touch).
#[test]
fn t1_optional_false_places_the_winner_and_asks_nothing() {
    let p1 = p(1);
    let p2 = p(2);

    let winner = land_card(p1, "T1 Winner Land", ZoneId::Library(p1));
    let mut state = bare_state(p1, p2, vec![winner]);

    let effect = base_effect(1, false);
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        in_hand(&state, "T1 Winner Land", p1),
        "optional: false must place the winner unconditionally"
    );
    assert!(
        state.pending_effect_choice().is_none(),
        "optional: false must never raise a CR 608.2d question -- it is not a player \
         decision at all"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// t2 -- `optional: true` with candidates ASKS, and the DEFAULT answer reproduces t1
// ═══════════════════════════════════════════════════════════════════════════

/// The behaviour-preservation pin: `default_effect_choice_answer`'s `ChooseObject`
/// arm returns `candidates.take(count)` = the first (lowest-id, post-sort) candidate
/// = exactly what `optional: false` places unconditionally. A bot submitting the
/// engine's own default therefore plays byte-identically to the pre-PB-DX35 engine.
///
/// **Stated as a CONTROL, not a discriminator**: by design its assertions hold
/// EITHER way (with the ask, answered by default; or under a full revert to the
/// pre-batch inert design, which places the same card unconditionally) -- that
/// equivalence IS the property this test pins. `t3`/`t6`/`t7`/`t8` are what prove
/// the ask is real; this one proves the default answer doesn't change anything for
/// the bot/fuzzer paths that submit it.
#[test]
fn t2_optional_true_asks_and_the_default_answer_reproduces_optional_false() {
    let p1 = p(1);
    let p2 = p(2);

    let winner = land_card(p1, "T2 Winner Land", ZoneId::Library(p1));
    let mut state = bare_state(p1, p2, vec![winner]);

    let effect = base_effect(1, true);
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect_with_default_choices(&mut state, &effect, &mut ctx);

    assert!(
        in_hand(&state, "T2 Winner Land", p1),
        "the default ChooseObject answer must place the same card optional: false does"
    );
    assert!(
        state.pending_effect_choice().is_none(),
        "the resolution must have completed, not be left suspended"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// t3 -- the DECLINE (`chosen: []`) leaves the card unplaced, routed to `rest_to`
// ═══════════════════════════════════════════════════════════════════════════

/// CR 118.12: declining is a real, distinct answer. Asserted by ZONE, not by the
/// absence of a `PermanentEnteredBattlefield`/hand event -- the card must be routed
/// to `rest_to` (here, the graveyard) exactly as a no-match top-N always was.
#[test]
fn t3_decline_leaves_the_card_unplaced_and_routes_it_to_rest_to() {
    let p1 = p(1);
    let p2 = p(2);

    let candidate = land_card(p1, "T3 Candidate Land", ZoneId::Library(p1));
    let mut state = bare_state(p1, p2, vec![candidate]);

    let effect = base_effect(1, true);
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let mut answer_fn = |q: &EffectChoiceQuestion| -> EffectChoiceAnswer {
        match q {
            EffectChoiceQuestion::ChooseObject { .. } => {
                EffectChoiceAnswer::ChooseObject { chosen: vec![] }
            }
            other => default_effect_choice_answer(other),
        }
    };
    let _events = execute_effect_answering(&mut state, &effect, &mut ctx, &mut answer_fn);

    assert!(
        !on_battlefield(&state, "T3 Candidate Land") && !in_hand(&state, "T3 Candidate Land", p1),
        "a declined card must NOT be placed to `destination`"
    );
    assert!(
        in_graveyard(&state, "T3 Candidate Land", p1),
        "a declined card must be routed to `rest_to` -- CR 118.12's 'if you don't' \
         fallback"
    );
    assert!(
        state.pending_effect_choice().is_none(),
        "the resolution must complete after a decline, not be left suspended"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// t4 -- `optional: true` with an EMPTY candidate set asks NOTHING
// ═══════════════════════════════════════════════════════════════════════════

/// CR 118.12: there is nothing to ask "whether" or "which" about when nothing in the
/// top-N window matches. Every looked-at card is still routed to `rest_to`
/// unconditionally.
///
/// **Stated as a CONTROL, not a discriminator**: an empty candidate set behaves
/// identically before and after this batch (the `continue`/empty-candidates path
/// predates PB-DX35), so this stays GREEN under a full revert too.
#[test]
fn t4_optional_true_with_no_match_asks_nothing() {
    let p1 = p(1);
    let p2 = p(2);

    let non_match_a = creature_with_mv(p1, "T4 Non Match A", 2, ZoneId::Library(p1));
    let non_match_b = creature_with_mv(p1, "T4 Non Match B", 3, ZoneId::Library(p1));
    let mut state = bare_state(p1, p2, vec![non_match_a, non_match_b]);

    let effect = base_effect(2, true);
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        state.pending_effect_choice().is_none(),
        "an empty candidate set must never raise a CR 608.2d question"
    );
    assert!(
        in_graveyard(&state, "T4 Non Match A", p1) && in_graveyard(&state, "T4 Non Match B", p1),
        "both non-matching cards must still be routed to rest_to unconditionally"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// t5 -- candidates.first() == the pre-batch min_by_key(|id| id.0) winner, proven on
// a fixture where top_ids order and ascending-ObjectId order genuinely disagree
// ═══════════════════════════════════════════════════════════════════════════

/// `Zone::top_n` returns TOP-FIRST order (`v.iter().rev()`), and the builder's push
/// order mints ascending ObjectIds bottom-to-top -- so for ANY 2+-candidate window
/// built the ordinary way, top-first order is the REVERSE of ascending-id order.
/// This fixture makes that disagreement explicit and asserts BOTH the raw fact (the
/// true top card of the library has the HIGHER id) and the consequence (the winner
/// PLACED is the LOWER id, i.e. `candidates.first()` after the required sort, not
/// `top_ids[0]` before it -- which would place the wrong card).
///
/// **Revert row (executed)**: a full revert to the pre-batch inert design leaves
/// this GREEN -- `min_by_key(|id| id.0)` was already correct pre-batch, so `t5`
/// does not discriminate "the ask feature exists" (`t3`/`t6`/`t7`/`t8` do that).
/// It discriminates a NARROWER regression: deleting only the `candidates.sort_by_key`
/// call while keeping the rest of B1's shape (an unsorted `candidates.first()`,
/// i.e. the physically topmost matching card) reddens `t5` on the assertion below,
/// proven by executing that exact revert.
#[test]
fn t5_candidates_first_is_the_ascending_id_winner_not_the_top_first_one() {
    let p1 = p(1);
    let p2 = p(2);

    // Push order is bottom-to-top: "Lower Id Land" is pushed first (lower ObjectId,
    // ends up BELOW in the library); "Higher Id Land" is pushed second (higher
    // ObjectId, ends up ABOVE -- the true top of a 2-card library).
    let lower_id_land = land_card(p1, "T5 Lower Id Land", ZoneId::Library(p1));
    let higher_id_land = land_card(p1, "T5 Higher Id Land", ZoneId::Library(p1));
    let mut state = bare_state(p1, p2, vec![lower_id_land, higher_id_land]);

    let lower_id = find_obj(&state, "T5 Lower Id Land");
    let higher_id = find_obj(&state, "T5 Higher Id Land");
    assert!(
        lower_id.0 < higher_id.0,
        "sanity: push order must mint ascending ids in push order"
    );

    let top_ids_before = state.zones().get(&ZoneId::Library(p1)).unwrap().top_n(2);
    assert_eq!(
        top_ids_before,
        vec![higher_id, lower_id],
        "the fixture's whole premise: `Zone::top_n`'s TOP-FIRST order must list the \
         HIGHER-id card first, the exact reverse of ascending-id order -- if this \
         assertion ever fails the fixture no longer discriminates a missing sort"
    );

    let effect = base_effect(2, false);
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        in_hand(&state, "T5 Lower Id Land", p1),
        "the LOWER-id card (candidates.first() after the required ascending sort) must \
         be placed -- an unsorted `top_ids.first()` would place the higher-id card \
         instead"
    );
    assert!(
        in_graveyard(&state, "T5 Higher Id Land", p1),
        "the higher-id card, though physically on top of the library, must be routed \
         to rest_to, not placed"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// t6 -- Birthing Ritual asks BOTH `EffectChoiceQuestion`s in the printed order
// ═══════════════════════════════════════════════════════════════════════════

/// CR 118.12, verbatim: *"Then you may sacrifice a creature. If you do, you may put a
/// creature card ... onto the battlefield."* Birthing Ritual is the corpus's only
/// member of BOTH `try_pay_optional_cost` call sites in one resolution (`place_cost`
/// AND the placement itself), so it is the only def that can pin the ORDER. Driven on
/// the REAL def's effect, extracted from `all_cards()` rather than hand-copied.
#[test]
fn t6_birthing_ritual_asks_pay_optional_cost_then_choose_object_in_that_order() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let ritual_def = defs
        .get("Birthing Ritual")
        .expect("Birthing Ritual def exists");
    let effect = ritual_def
        .abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::Triggered { effect, .. } => Some(effect.clone()),
            _ => None,
        })
        .expect("Birthing Ritual has a Triggered ability carrying its LookAtTopThenPlace");
    assert!(
        matches!(effect, Effect::LookAtTopThenPlace { .. }),
        "sanity: Birthing Ritual's triggered effect must still be LookAtTopThenPlace"
    );

    let sac_fodder = creature_with_mv(p1, "T6 Sac Fodder", 2, ZoneId::Battlefield);
    let good_target = creature_with_mv(p1, "T6 Good Target", 3, ZoneId::Library(p1));
    let mut state = bare_state(p1, p2, vec![sac_fodder, good_target]);

    let mut order: Vec<&'static str> = Vec::new();
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    {
        let mut answer_fn = |q: &EffectChoiceQuestion| -> EffectChoiceAnswer {
            match q {
                EffectChoiceQuestion::PayOptionalCost { .. } => {
                    order.push("PayOptionalCost");
                    EffectChoiceAnswer::PayOptionalCost { pay: true }
                }
                EffectChoiceQuestion::ChooseObject { candidates, .. } => {
                    order.push("ChooseObject");
                    EffectChoiceAnswer::ChooseObject {
                        chosen: candidates.first().copied().into_iter().collect(),
                    }
                }
                other => default_effect_choice_answer(other),
            }
        };
        let _events = execute_effect_answering(&mut state, &effect, &mut ctx, &mut answer_fn);
    }

    assert_eq!(
        order,
        vec!["PayOptionalCost", "ChooseObject"],
        "CR 118.12's printed order must be the ORDER the engine ASKS in, not merely the \
         order it validates in"
    );
    assert!(
        in_graveyard(&state, "T6 Sac Fodder", p1),
        "the sacrifice (answered `pay: true`) must have been paid"
    );
    assert!(
        on_battlefield(&state, "T6 Good Target"),
        "the placement (answered by taking the first candidate) must have happened"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// t7 -- the choice is a real choice of WHICH, not only whether
// ═══════════════════════════════════════════════════════════════════════════

/// With two matching candidates, answering with the SECOND (by ascending id, i.e.
/// `candidates[1]`) places the SECOND -- proving `ChooseObject`'s candidate set is a
/// real menu, not merely a yes/no gate around the deterministic winner.
#[test]
fn t7_answering_with_the_second_candidate_places_the_second() {
    let p1 = p(1);
    let p2 = p(2);

    let first_land = land_card(p1, "T7 First Land", ZoneId::Library(p1));
    let second_land = land_card(p1, "T7 Second Land", ZoneId::Library(p1));
    let mut state = bare_state(p1, p2, vec![first_land, second_land]);

    let first_id = find_obj(&state, "T7 First Land");
    let second_id = find_obj(&state, "T7 Second Land");
    assert!(
        first_id.0 < second_id.0,
        "sanity: push order mints ascending ids"
    );

    let effect = base_effect(2, true);
    let mut ctx = EffectContext::new(p1, ObjectId(9999), vec![]);
    let mut answer_fn = |q: &EffectChoiceQuestion| -> EffectChoiceAnswer {
        match q {
            EffectChoiceQuestion::ChooseObject { candidates, .. } => {
                assert_eq!(
                    candidates.len(),
                    2,
                    "sanity: both lands must be legal candidates"
                );
                assert_eq!(
                    candidates[0], first_id,
                    "candidates must be sorted ascending by ObjectId (first == lower id)"
                );
                EffectChoiceAnswer::ChooseObject {
                    chosen: vec![candidates[1]],
                }
            }
            other => default_effect_choice_answer(other),
        }
    };
    let _events = execute_effect_answering(&mut state, &effect, &mut ctx, &mut answer_fn);

    assert!(
        in_hand(&state, "T7 Second Land", p1),
        "the SECOND (answered) candidate must be placed, not the deterministic \
         (lowest-id) default"
    );
    assert!(
        in_graveyard(&state, "T7 First Land", p1),
        "the FIRST (lowest-id, non-chosen) candidate must be routed to rest_to instead"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// t8 -- Risen Reef, declined, puts the card into hand (its printed `rest_to`)
// ═══════════════════════════════════════════════════════════════════════════

/// Repeatedly pass priority (for whoever currently holds it) until either a CR
/// 608.2d question is outstanding or the stack empties. Bounded so a genuine stall
/// fails loudly rather than hanging the suite.
fn advance_until_choice_or_empty_stack(mut state: GameState, guard_max: u32) -> GameState {
    let mut guard = 0;
    loop {
        if state.pending_effect_choice().is_some() {
            return state;
        }
        if state.stack_objects().is_empty() {
            return state;
        }
        guard += 1;
        assert!(
            guard < guard_max,
            "advance_until_choice_or_empty_stack exceeded its guard -- neither a \
             CR 608.2d question nor an empty stack was reached"
        );
        let holder = state
            .turn()
            .priority_holder
            .expect("no priority holder while the stack is nonempty");
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {holder:?} failed: {e:?}"));
        state = new_state;
    }
}

/// Risen Reef prints *"look at the top card of your library. If it's a land card,
/// you may put it onto the battlefield tapped. If you don't put the card onto the
/// battlefield, put it into your hand."* -- CR 118.12's decline fallback, verbatim.
/// Its own ETB fires its own trigger (`exclude_self: false`), which is what this
/// fixture uses to reach a REAL question through the full `process_command` cast ->
/// resolve -> trigger -> resolve pipeline, not a direct-executor shortcut.
#[test]
fn t8_risen_reef_declined_puts_the_land_into_hand() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let reef = real_card_spec(p1, "Risen Reef", ZoneId::Hand(p1), &defs);
    let lib_land = land_card(p1, "T8 Fixture Land", ZoneId::Library(p1));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(reef)
        .object(lib_land)
        .build()
        .unwrap();

    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool = ManaPool {
            colorless: 1,
            green: 1,
            blue: 1,
            ..Default::default()
        };
    }

    let reef_id = find_obj(&state, "Risen Reef");

    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player: p1,
            card: reef_id,
            targets: vec![],
            modes_chosen: vec![],
            x_value: 0,
            kicker_times: 0,
            additional_costs: vec![],
            alt_cost: None,
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            prototype: false,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
            face_down_kind: None,
        })),
    )
    .expect("cast Risen Reef");

    let state = advance_until_choice_or_empty_stack(state, 50);

    assert!(
        on_battlefield(&state, "Risen Reef"),
        "sanity: Risen Reef must have resolved onto the battlefield before its own \
         ETB trigger can fire"
    );

    let pending = state
        .pending_effect_choice()
        .cloned()
        .expect("Risen Reef's own ETB trigger must raise a real CR 608.2d question");
    assert!(
        matches!(pending.question, EffectChoiceQuestion::ChooseObject { .. }),
        "the outstanding question must be a ChooseObject: {:?}",
        pending.question
    );

    let (state, events) = process_command(
        state,
        Command::AnswerEffectChoice {
            player: pending.player,
            choice_id: pending.choice_id,
            answer: EffectChoiceAnswer::ChooseObject { chosen: vec![] },
        },
    )
    .expect("the decline must be accepted");

    assert!(
        in_hand(&state, "T8 Fixture Land", p1),
        "a declined Risen Reef dig must put the card into hand -- the printed \
         fallback, verbatim"
    );
    assert!(
        !on_battlefield(&state, "T8 Fixture Land"),
        "a declined card must NOT enter the battlefield"
    );
    assert!(
        !events.iter().any(
            |e| matches!(e, GameEvent::PermanentEnteredBattlefield { object_id, .. }
                if state.objects().get(object_id).map(|o| o.characteristics.name.as_str())
                    == Some("T8 Fixture Land"))
        ),
        "no PermanentEnteredBattlefield event may reference the declined card"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Version sentinels
// ═══════════════════════════════════════════════════════════════════════════

/// PROTOCOL_VERSION and HASH_SCHEMA_VERSION are predicted UNMOVED by Half B
/// (execution-notes §0.2): `Effect::LookAtTopThenPlace` and
/// `EffectChoiceQuestion::ChooseObject` are both already in the wire closure with no
/// field, type or variant added.
#[test]
fn test_pb_dx35_half_b_version_sentinels() {
    assert_eq!(
        mtg_engine::PROTOCOL_VERSION,
        42,
        "PROTOCOL_VERSION must be unmoved by PB-DX35 Half B"
    );
    assert_eq!(
        mtg_engine::HASH_SCHEMA_VERSION,
        83u8,
        "HASH_SCHEMA_VERSION must be unmoved by PB-DX35 Half B"
    );
}
