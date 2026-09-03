//! PB-DX29 → **PB-DX50**: CR 702.140a/c/e — the over/under choice, and the two
//! separate things that were wrong with it.
//!
//! # What PB-DX29 fixed, and what it deliberately did not
//!
//! `params.rs`' `CastWithMutate` arm built `AdditionalCost::Mutate { on_top: true }`
//! with `on_top` **hard-coded**, so no client in this tree could ever mutate a creature
//! UNDER another one. PB-DX29 put an `on_top: bool` on `LegalAction::CastWithMutate` and
//! emitted one offer per `(target, on_top)` pair, making the choice answerable.
//!
//! It said, in this file, that it was not fixing the timing:
//!
//! > CR 702.140c makes the choice a decision taken **as the spell resolves**, and the
//! > engine captures it at ANNOUNCEMENT. … moving the moment is `OOS-DX29-2` and needs a
//! > resolution-time `EffectChoiceQuestion`.
//!
//! # PB-DX50 is that seed, and this file is REWRITTEN rather than deleted
//!
//! The choice now suspends at resolution as `EffectChoiceQuestion::MutateOnTop` on
//! PB-DP9's CR 608.2d channel. `LegalAction::CastWithMutate` has **no `on_top` field**,
//! `AdditionalCost::Mutate` has none either, and the mutate offer is one action per
//! target — the offer count HALVES.
//!
//! Two of this file's three tests therefore **invert**, and they are disclosed by name
//! rather than netted out of a count:
//!
//! * `test_dx29_m1_provider_offers_both_on_top_and_under`
//!   → `test_dx50_m1_provider_offers_exactly_one_action_per_mutate_target`
//! * `test_dx29_m2_params_forwards_the_actions_on_top_choice`
//!   → `test_dx50_m2_params_builds_the_mutate_cost_with_no_over_under_answer`
//!
//! The third, `m3`, is the proof AC 7302 says must SURVIVE — that mutating UNDER leaves
//! the host's name on the merged permanent, end to end on real corpus cards — and it is
//! **re-homed onto the resolution-time answer**, not deleted. It is now proven twice:
//! through the bot path (default params → the pre-batch `on_top: true`) and through the
//! human channel (`ActionParams::effect_choice_answer` → UNDER), which is the answer no
//! channel could produce at resolution time before this batch.
//!
//! CR 702.140e / CR 729.2a are why any of it matters: the **topmost** component supplies
//! the merged permanent's non-ability characteristics — name, card id, mana cost,
//! colours, types and power/toughness — so "over" and "under" are genuinely different
//! permanents from the same two cards, on **six deck-legal `Complete` mutate defs**.

use std::collections::HashMap;

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AdditionalCost,
    CardDefinition, Command, EffectChoiceAnswer, GameState, GameStateBuilder, ManaPool, ObjectId,
    ObjectSpec, PlayerId, Step, ZoneId,
};
use mtg_simulator::build_registry;
use mtg_simulator::legal_actions::{LegalAction, LegalActionProvider, StubProvider};
use mtg_simulator::params::{action_to_command_with_params, ActionParams};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// The corpus's mutate def used throughout: `Complete`, deck-legal, mutate cost
/// `{1}{G}{G}`, a Beast (so it is not itself a Human and the merged permanent's name is
/// unambiguous against the target's).
const MUTATOR: &str = "Gemrazer";
/// A non-Human creature the caster owns, for the mutate target. `Arbor Elf` is an Elf
/// Druid (CR 702.140a needs a NON-Human), `Complete`, and deck-legal.
const HOST: &str = "Arbor Elf";

/// Every card definition keyed by NAME — the shape `enrich_spec_from_def` wants,
/// mirroring `pb_dx23_dredge_answer_channel.rs`'s own helper.
fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object {name:?} not found"))
}

/// A main phase with P1 holding `MUTATOR` and controlling+owning a `HOST`, with enough
/// green mana floating to pay the mutate cost outright.
///
/// Mana is put in the POOL rather than on lands deliberately: `can_afford` answers
/// "the pool alone covers this" first, so the offer this test reads is not entangled
/// with `mana_solver`'s planning (`OOS-M11-2`'s open half).
fn mutate_board() -> GameState {
    let defs = card_defs_by_name();
    let p1 = p(1);
    let p2 = p(2);
    GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(build_registry())
        .player_mana(
            p1,
            ManaPool {
                green: 3,
                ..Default::default()
            },
        )
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, MUTATOR)
                .with_card_id(card_name_to_id(MUTATOR))
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        ))
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, HOST)
                .with_card_id(card_name_to_id(HOST))
                .in_zone(ZoneId::Battlefield),
            &defs,
        ))
        .build()
        .expect("PB-DX29 mutate fixture must build")
}

fn mutate_actions(state: &GameState, player: PlayerId) -> Vec<LegalAction> {
    StubProvider
        .legal_actions(state, player)
        .into_iter()
        .filter(|a| matches!(a, LegalAction::CastWithMutate { .. }))
        .collect()
}

/// M1 (**INVERTED by PB-DX50**) — CR 702.140a/c: the provider offers exactly **one**
/// action per legal mutate target, and it carries no over/under answer at all.
///
/// This test previously asserted the opposite (both halves of a `(target, on_top)`
/// pair). PB-DX29's pair loop was the right fix for "no client can mutate under" and the
/// wrong MOMENT for CR 702.140c, which says the controller chooses *as the spell
/// resolves*. Offering it here meant the opponent saw the choice before deciding whether
/// to respond, and the controller could not change it afterwards.
///
/// **Revert to watch red**: restore `for on_top in [true, false]` around the push in
/// `legal_actions.rs`'s mutate loop (with a field to put it in) — or, without touching
/// the type, push each action twice; the count assertion reddens either way.
#[test]
fn test_dx50_m1_provider_offers_exactly_one_action_per_mutate_target() {
    let state = mutate_board();
    let host = find_object(&state, HOST);
    let card = find_object(&state, MUTATOR);

    let actions = mutate_actions(&state, p(1));
    // Non-vacuity: the board really does produce a mutate offer at all. Without this a
    // provider that stopped offering mutate entirely would satisfy the count assertion
    // below only by accident of arithmetic.
    assert!(
        !actions.is_empty(),
        "precondition (CR 702.140a): a Gemrazer in hand with {{G}}{{G}}{{G}} floating and a \
         non-Human creature owned on the battlefield must produce a mutate offer"
    );
    // ONE host on the board => ONE offer. Before PB-DX50 this was two.
    assert_eq!(
        actions.len(),
        1,
        "CR 702.140c (PB-DX50): the over/under choice is made AS THE SPELL RESOLVES, so \
         the offer layer emits one action per target and not one per (target, on_top) \
         pair. Got: {actions:?}"
    );
    let LegalAction::CastWithMutate {
        card: c,
        mutate_target,
    } = &actions[0]
    else {
        unreachable!("filtered above")
    };
    assert_eq!(*c, card, "the offer must name the card in hand");
    assert_eq!(
        *mutate_target, host,
        "the offer must name the non-Human creature the caster owns (CR 702.140a)"
    );
}

/// M2 (**INVERTED by PB-DX50**) — CR 702.140a: the mapping table announces the mutate
/// HOST and nothing else. There is no over/under value to forward, because CR 702.140c
/// puts that decision at resolution.
///
/// **Revert to watch red**: make `params.rs`'s `CastWithMutate` arm announce something
/// other than the action's `mutate_target` (e.g. the card itself), or drop the
/// `AdditionalCost::Mutate` entry entirely.
#[test]
fn test_dx50_m2_params_builds_the_mutate_cost_with_no_over_under_answer() {
    let state = mutate_board();
    let p1 = p(1);
    let host = find_object(&state, HOST);

    let actions = mutate_actions(&state, p1);
    assert_eq!(actions.len(), 1, "M1's precondition, restated");
    let action = &actions[0];
    let command = action_to_command_with_params(&state, p1, action, &ActionParams::default())
        .expect("a mutate offer must map to a command");
    let Command::CastSpell(cast) = &command else {
        panic!("CastWithMutate must map to a CastSpell, got {command:?}");
    };
    let mutate_entries: Vec<ObjectId> = cast
        .additional_costs
        .iter()
        .filter_map(|c| match c {
            AdditionalCost::Mutate { target } => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(
        mutate_entries,
        vec![host],
        "CR 702.140a: exactly one mutate cost, naming the host. The struct has no \
         `on_top` field at all any more (PB-DX50) -- if this stops compiling because \
         something re-added one, that is the regression, not the test. Got: {cast:?}"
    );
}

/// M3 (**RE-HOMED by PB-DX50, not deleted**) — CR 702.140e / CR 729.2a, end to end with
/// the NON-DEFAULT answer, through the **resolution-time** channel.
///
/// This is the assertion AC 7302 requires to survive: mutating UNDER leaves the host's
/// name on the merged permanent; mutating OVER replaces it. It reads the merged permanent
/// by NAME (CR 400.7 — and note that a mutate merge deliberately PRESERVES the target's
/// `ObjectId`, CR 729.2c, so the id survives here where a zone change would have killed
/// it), and the two answers produce different names from the same two cards.
///
/// **Both channels, and the difference between them is the point.** The BOT path submits
/// the offered action's own `answer` verbatim (`ActionParams::default()`), which is
/// `default_effect_choice_answer`'s `on_top: true` — the exact recovery of the pre-batch
/// hard-coded value, and the reason every bot game and fuzz seed is behaviourally
/// unchanged. The HUMAN channel supplies `effect_choice_answer` and gets UNDER, which is
/// the answer no channel could produce **at resolution time** before this batch.
///
/// **Revert to watch red**: hard-code `mutate_on_top = true` at the ask site in
/// `resolution.rs`'s `MutatingCreatureSpell` arm; the UNDER half collapses onto the OVER
/// half and the `assert_ne!` fires.
#[test]
fn test_dx50_m3_mutating_under_keeps_the_hosts_characteristics() {
    let p1 = p(1);
    let p2 = p(2);

    // `answer`: `None` = the bot path (submit the offer's own default verbatim);
    // `Some(v)` = the human channel naming its own answer.
    let outcome = |answer: Option<bool>| -> String {
        let state = mutate_board();
        let actions = mutate_actions(&state, p1);
        assert_eq!(actions.len(), 1, "M1's precondition, restated");
        let command =
            action_to_command_with_params(&state, p1, &actions[0], &ActionParams::default())
                .expect("mapping must succeed");
        // `process_command` takes ownership, so each branch builds its own state.
        let (state, _events) = process_command(state, command).expect("the mutate cast is legal");
        // CR 702.140c: nothing has been asked yet -- the opponent still has priority and
        // must not have learned the choice. This is the whole batch, asserted inside the
        // end-to-end probe rather than only in the unit file.
        assert!(
            state.pending_effect_choice().is_none(),
            "CR 702.140c: the over/under question must not be asked at announcement"
        );
        // Both players pass so the spell resolves (CR 117.4).
        let (state, _) = process_command(state, Command::PassPriority { player: p1 })
            .expect("p1 may pass with the spell on the stack");
        let (mut state, _) = process_command(state, Command::PassPriority { player: p2 })
            .expect("p2 may pass, resolving the top of the stack");

        // Now the question is live, and it comes through the OFFER LAYER -- not
        // hand-built -- so this probe exercises the real client path.
        let offers: Vec<LegalAction> = StubProvider
            .legal_actions(&state, p1)
            .into_iter()
            .filter(|a| matches!(a, LegalAction::AnswerEffectChoice { .. }))
            .collect();
        assert_eq!(
            offers.len(),
            1,
            "CR 608.2d: resolution must offer exactly one over/under answer. Got {offers:?}"
        );
        let params = match answer {
            None => ActionParams::default(),
            Some(on_top) => ActionParams {
                effect_choice_answer: Some(EffectChoiceAnswer::MutateOnTop { on_top }),
                ..Default::default()
            },
        };
        let cmd = action_to_command_with_params(&state, p1, &offers[0], &params)
            .expect("the answer must map to a command");
        let (next, _) = process_command(state, cmd).expect("both answers are legal (CR 702.140c)");
        state = next;

        let merged: Vec<String> = state
            .objects()
            .values()
            .filter(|o| o.zone == ZoneId::Battlefield && o.controller == p1)
            .map(|o| o.characteristics.name.clone())
            .collect();
        assert_eq!(
            merged.len(),
            1,
            "CR 729.2b: the mutating spell is absorbed into the host and does NOT enter as a \
             separate permanent, so exactly one permanent must remain. Got: {merged:?}"
        );
        merged.into_iter().next().expect("checked above")
    };

    // CR 729.2a: the topmost component supplies the merged permanent's characteristics.
    assert_eq!(
        outcome(Some(true)),
        MUTATOR,
        "mutating OVER must leave the mutating card's name on the merged permanent"
    );
    assert_eq!(
        outcome(Some(false)),
        HOST,
        "CR 702.140e / CR 729.2a: mutating UNDER must leave the HOST's name, mana cost, types \
         and P/T on the merged permanent. Before PB-DX50 this was reachable only by choosing \
         it at ANNOUNCEMENT, which is not when CR 702.140c puts the choice."
    );
    // The BOT path, submitting the offer's own default verbatim, reproduces the
    // pre-batch hard-coded `on_top: true`. This is what keeps every bot game, every
    // recorded fuzz seed and the one golden mutate script behaviourally identical.
    assert_eq!(
        outcome(None),
        MUTATOR,
        "PB-DX50 §5: `default_effect_choice_answer(MutateOnTop) == {{ on_top: true }}` is \
         the exact recovery of the pre-batch value, so a bot submitting the default plays \
         the identical game"
    );
    // The pair must DIFFER, or the two halves above are two spellings of one
    // measurement (this queue's own recurring failure mode).
    assert_ne!(
        outcome(Some(true)),
        outcome(Some(false)),
        "CR 702.140c/e: the choice must be observable, or it is not a choice"
    );
}
