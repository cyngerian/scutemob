//! PB-DX29 — CR 702.140a/c/e: a mutating creature spell can finally go **under**.
//!
//! # What was wrong
//!
//! `crates/simulator/src/params.rs`' `CastWithMutate` arm built
//! `AdditionalCost::Mutate { target, on_top: true }` with `on_top` **hard-coded**, and
//! `LegalAction::CastWithMutate` carried no channel for it. So no client in this tree —
//! browser, TUI, bot or test harness routing through the mapping table — could ever
//! mutate a creature UNDER another one.
//!
//! That is not cosmetic. CR 702.140e / CR 729.2a make the **topmost** component supply
//! the merged permanent's non-ability characteristics — name, card id, mana cost,
//! colours, types and power/toughness — and `resolution.rs`'s merge site sets
//! `target_obj.characteristics` and `target_obj.card_id` from `merged_components.front()`.
//! "Over" and "under" therefore produce genuinely different permanents from the same two
//! cards, on **six deck-legal `Complete` mutate defs**.
//!
//! # The fix, and why it is an ACTION and not a param
//!
//! `legal_actions.rs` now emits one `CastWithMutate` per `(target, on_top)` pair, exactly
//! as it already emitted one per target. That is the `PayEcho` / `ChooseDredge` /
//! `ActivateBloodrush` idiom this codebase has already ruled correct (PB-DX23 §3) for a
//! choice fully determined at offer time: no new `ActionParams` field, no wire change, no
//! picker — the choice IS the button. `on_top: true` is emitted first of each pair, so an
//! index-choosing bot takes the same action it took before on the same board.
//!
//! # The CR deviation this does NOT fix
//!
//! CR 702.140c makes the choice a decision taken **as the spell resolves**, and the
//! engine captures it at ANNOUNCEMENT. Offering it here makes a question the engine
//! already asks at the wrong moment *answerable* instead of hard-coded; moving the moment
//! is `OOS-DX29-2` and needs a resolution-time `EffectChoiceQuestion`. Stated here rather
//! than implied, because a reader who finds this file could otherwise conclude the timing
//! was checked and found correct.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AdditionalCost,
    CardDefinition, Command, GameState, GameStateBuilder, ManaPool, ObjectId, ObjectSpec, PlayerId,
    Step, ZoneId,
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

/// M1 — CR 702.140a/c: the provider offers **both** halves of the choice, once per
/// target, and they are distinguishable.
///
/// **Revert to watch red**: in `legal_actions.rs`'s mutate loop, replace the
/// `for on_top in [true, false]` with a single `on_top: true` push.
#[test]
fn test_dx29_m1_provider_offers_both_on_top_and_under() {
    let state = mutate_board();
    let host = find_object(&state, HOST);
    let card = find_object(&state, MUTATOR);

    let actions = mutate_actions(&state, p(1));
    // Non-vacuity: the board really does produce a mutate offer at all. Without this a
    // provider that stopped offering mutate entirely would satisfy nothing below but
    // also fail nothing above.
    assert!(
        !actions.is_empty(),
        "precondition (CR 702.140a): a Gemrazer in hand with {{G}}{{G}}{{G}} floating and a \
         non-Human creature owned on the battlefield must produce a mutate offer"
    );

    let flags: BTreeSet<bool> = actions
        .iter()
        .map(|a| match a {
            LegalAction::CastWithMutate { on_top, .. } => *on_top,
            _ => unreachable!("filtered above"),
        })
        .collect();
    assert_eq!(
        flags,
        [true, false].into_iter().collect::<BTreeSet<bool>>(),
        "CR 702.140c: the caster chooses whether the mutating card goes on top or under, so \
         the provider must offer BOTH. Before PB-DX29 `params.rs` hard-coded `on_top: true` \
         and no client could ever mutate under. Offered: {actions:?}"
    );

    // Exactly one action per (target, flag) pair — one host on the board, so two.
    assert_eq!(
        actions.len(),
        2,
        "one host creature x two on_top values = two offers; got {actions:?}"
    );
    for action in &actions {
        let LegalAction::CastWithMutate {
            card: c,
            mutate_target,
            ..
        } = action
        else {
            unreachable!("filtered above")
        };
        assert_eq!(*c, card, "every offer must name the card in hand");
        assert_eq!(
            *mutate_target, host,
            "every offer must name the non-Human creature the caster owns (CR 702.140a)"
        );
    }

    // CR 702.140a: the ORDER is pinned, not incidental. `on_top: true` is emitted first
    // so an index-choosing bot takes the same action it took before PB-DX29 on the same
    // board — which is why no recorded seed moved.
    let LegalAction::CastWithMutate { on_top: first, .. } = &actions[0] else {
        unreachable!()
    };
    assert!(
        *first,
        "`on_top: true` must be the LOWER index of each pair, so a bot choosing by index \
         reproduces the pre-PB-DX29 command"
    );
}

/// M2 — CR 702.140a: `params.rs` forwards the action's own choice into the
/// `AdditionalCost::Mutate` it builds, rather than hard-coding it.
///
/// **Revert to watch red**: in `params.rs`'s `CastWithMutate` arm, change
/// `on_top: *on_top` back to `on_top: true`.
#[test]
fn test_dx29_m2_params_forwards_the_actions_on_top_choice() {
    let state = mutate_board();
    let p1 = p(1);

    for action in mutate_actions(&state, p1) {
        let LegalAction::CastWithMutate { on_top: want, .. } = &action else {
            unreachable!()
        };
        let command = action_to_command_with_params(&state, p1, &action, &ActionParams::default())
            .expect("a mutate offer must map to a command");
        let Command::CastSpell(cast) = &command else {
            panic!("CastWithMutate must map to a CastSpell, got {command:?}");
        };
        let got = cast
            .additional_costs
            .iter()
            .find_map(|c| match c {
                AdditionalCost::Mutate { on_top, .. } => Some(*on_top),
                _ => None,
            })
            .expect("CR 702.140a: the mutate cost must be announced");
        assert_eq!(
            got, *want,
            "the mapping table must forward the ACTION's `on_top`, not a hard-coded value. \
             action: {action:?}"
        );
    }
}

/// M3 — CR 702.140e / CR 729.2a, **end to end with the NON-DEFAULT answer**. Mutating
/// UNDER leaves the host's name on the merged permanent; mutating OVER replaces it.
///
/// This is the assertion that makes the channel worth having: it reads the merged
/// permanent by NAME (CR 400.7 — and note that a mutate merge deliberately PRESERVES the
/// target's `ObjectId`, CR 729.2c, so the id survives here where a zone change would
/// have killed it), and the two answers produce different names from the same two cards.
///
/// **Revert to watch red**: either revert named in M1/M2 collapses this to one outcome.
#[test]
fn test_dx29_m3_mutating_under_keeps_the_hosts_characteristics() {
    let p1 = p(1);
    let p2 = p(2);

    let outcome = |want_on_top: bool| -> String {
        let state = mutate_board();
        let action = mutate_actions(&state, p1)
            .into_iter()
            .find(|a| matches!(a, LegalAction::CastWithMutate { on_top, .. } if *on_top == want_on_top))
            .unwrap_or_else(|| panic!("no mutate offer with on_top={want_on_top}"));
        let command = action_to_command_with_params(&state, p1, &action, &ActionParams::default())
            .expect("mapping must succeed");
        // `process_command` takes ownership, so each branch builds its own state.
        let (state, _events) = process_command(state, command).expect("the mutate cast is legal");
        // Both players pass so the spell resolves (CR 117.4).
        let (state, _) = process_command(state, Command::PassPriority { player: p1 })
            .expect("p1 may pass with the spell on the stack");
        let (state, _) = process_command(state, Command::PassPriority { player: p2 })
            .expect("p2 may pass, resolving the top of the stack");

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
        outcome(true),
        MUTATOR,
        "mutating OVER must leave the mutating card's name on the merged permanent"
    );
    assert_eq!(
        outcome(false),
        HOST,
        "CR 702.140e / CR 729.2a: mutating UNDER must leave the HOST's name, mana cost, types \
         and P/T on the merged permanent. This is the outcome no client in the tree could \
         produce before PB-DX29, because `params.rs` hard-coded `on_top: true`."
    );
}
