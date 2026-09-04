//! PB-DX52 (`OOS-DX25b-1`) — Bolt Bend's printed "or ability" half, through the REAL
//! channels.
//!
//! The engine-side probes live in `crates/engine/tests/primitives/` and
//! `crates/engine/tests/core/` (a sibling agent's files, not this one). This file
//! exists because **existence is never sufficiency** (the `kaito_shizuki` lesson,
//! PB-DX43): a target space the engine can now express is not a repaired decision
//! until a real client can be OFFERED it and have its answer ACCEPTED. Every probe
//! here drives `LocalGame`/`HumanChoice` or the `StubProvider`/`plan_targets`/
//! `RandomBot` bot layer — the same surfaces the browser and the bots go through.
//!
//! # The fixture, and why it needs no synthetic card
//!
//! `Bolt Bend` (`Complete`, deck-legal) is the CastSpell under test:
//! *"Change the target of target spell or ability with a single target"*
//! (CR 115.7a, `TargetRequirement::TargetSpellOrAbilityWithSingleTarget`). The
//! "ability" it must be able to target is `Goblin Sharpshooter`'s printed
//! `{T}: This creature deals 1 damage to any target` — real corpus, `Complete`,
//! deck-legal, and its cost is `Cost::Tap` alone, so p2 can activate it with **no
//! mana at all**, which keeps the fixture's mana-payment surface entirely on p1's
//! side (Bolt Bend's own `{3}{R}`).
//!
//! # The narrative every probe drives
//!
//! p2 activates Goblin Sharpshooter's ability targeting **p1** (a real burn spell
//! aimed at the human). p1, holding priority next (CR 117.3c: the activator gets
//! priority back first, so p1's window comes right after p2's mandatory pass),
//! casts Bolt Bend at the ability's stack entry. CR 115.7a's own candidate-ordering
//! rule (`rules::retarget::retarget_candidates`) tries the redirecting player
//! (p1, Bolt Bend's controller) first, but p1 IS the ability's current target and
//! `!= current` excludes it — so the next candidate, the other player (p2), is what
//! the redirect lands on. **This makes the resolution-effect assertion
//! deterministic without touching player choice at resolution time**: CR 115.7a's
//! "which object or player becomes the new target" is an ENGINE decision
//! (`rules::retarget::plan_target_change`), not a further human choice, so a
//! probe can compute the expected outcome in advance rather than merely observing
//! whatever the engine happened to pick.
use std::collections::{BTreeSet, HashMap};

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition, Command,
    GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, StackObjectKind, Step, Target,
    ZoneId,
};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::targeting::plan_targets;
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, LegalAction, LegalActionProvider, LocalGame,
    LocalGameLimits, PendingDecision, RandomBot, StubProvider, TargetPlan,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 52_52_52;

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 3,
        max_commands: 600,
        max_consecutive_passes: 500,
        record_journal: true,
    }
}

/// `p1` holds a real `Bolt Bend` in hand plus four untapped Mountains (covers the
/// printed `{3}{R}` with `auto_tap: true`, and stays clear of `ConditionalPowerThreshold`
/// — p1 controls no creature, so the cost reduction never fires and every candidate
/// pays the full four). `p2` controls a real, non-summoning-sick `Goblin Sharpshooter`
/// (builder-placed permanents are never summoning sick — `has_summoning_sickness:
/// false` unconditionally, `state/builder.rs`) and holds a real `Lightning Bolt` plus
/// one Mountain, used only by the c4 control (a SPELL, not an ability, on the stack).
///
/// `active_player(p2)` + `at_step(PreCombatMain)` matter only for the RAW
/// `process_command` probes (c3/c4), which never call `LocalGame::start` and so never
/// have this reset. The `LocalGame`-driven probes (c1/c2) start a fresh turn at
/// `Step::Untap` regardless — `LocalGame::start` always resets to it (PB-DX45's own
/// documented gotcha) — and reach `PreCombatMain` by the ordinary `drive_until`
/// pass-through.
fn fixture() -> GameState {
    let defs = card_defs_by_name();
    let bolt_bend = enrich_spec_from_def(
        ObjectSpec::card(p(1), "Bolt Bend")
            .in_zone(ZoneId::Hand(p(1)))
            .with_card_id(card_name_to_id("Bolt Bend")),
        &defs,
    );
    let sharpshooter = enrich_spec_from_def(
        ObjectSpec::creature(p(2), "Goblin Sharpshooter", 1, 1)
            .with_card_id(card_name_to_id("Goblin Sharpshooter")),
        &defs,
    );
    let lightning_bolt = enrich_spec_from_def(
        ObjectSpec::card(p(2), "Lightning Bolt")
            .in_zone(ZoneId::Hand(p(2)))
            .with_card_id(card_name_to_id("Lightning Bolt")),
        &defs,
    );
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(2))
        .at_step(Step::PreCombatMain)
        .object(bolt_bend)
        .object(sharpshooter)
        .object(lightning_bolt);
    for _ in 0..4 {
        builder = builder.object(enrich_spec_from_def(
            ObjectSpec::land(p(1), "Mountain").with_card_id(card_name_to_id("Mountain")),
            &defs,
        ));
    }
    builder = builder.object(enrich_spec_from_def(
        ObjectSpec::land(p(2), "Mountain").with_card_id(card_name_to_id("Mountain")),
        &defs,
    ));
    for player in [p(1), p(2)] {
        for i in 0..30 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("PB-DX52 Library Filler {i} ({player:?})"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().expect("PB-DX52 channel fixture must build")
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found in state"))
}

/// CR 113.1c / CR 602.2: the ONE `ActivatedAbility` stack entry -- named by the
/// `StackObject`'s own id, per `Target::StackObject`'s doc (PB-DX52).
fn find_ability_stack_id(state: &GameState) -> ObjectId {
    state
        .stack_objects()
        .iter()
        .find(|so| matches!(so.kind, StackObjectKind::ActivatedAbility { .. }))
        .map(|so| so.id)
        .unwrap_or_else(|| {
            panic!(
                "no ActivatedAbility stack object found: {:?}",
                state.stack_objects()
            )
        })
}

fn life(state: &GameState, player: PlayerId) -> i32 {
    state
        .players()
        .get(&player)
        .unwrap_or_else(|| panic!("{player:?} exists"))
        .life_total
}

fn start_human_game() -> LocalGame<StubProvider> {
    let bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    let human: BTreeSet<PlayerId> = [p(1), p(2)].into_iter().collect();
    let (game, _events) =
        LocalGame::start(fixture(), SEED, StubProvider, bots, human, limits(), true)
            .expect("PB-DX52 channel game must start");
    game
}

/// Drive human seats, passing priority for whichever player is asked, until `want`
/// finds an action in the offered list. Returns the decision and the index of that
/// action.
///
/// **Panics rather than returning `None`** -- a probe that silently ends early is a
/// probe that asserts nothing, and every assertion in this file is downstream of
/// actually reaching the offer. Player-agnostic on purpose: `want`'s own predicate
/// (matching a specific `source`/`card` id) is what disambiguates which seat's turn
/// it is to act, mirroring `pb_dx45_optional_cost_channel.rs`'s idiom.
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    label: &str,
    want: impl Fn(&LegalAction) -> bool,
) -> (PendingDecision, usize) {
    for _ in 0..80 {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                if let Some(i) = d.actions.iter().position(&want) {
                    return (d, i);
                }
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!(
                            "no {label} offer and no PassPriority either: {:?}",
                            d.actions
                        )
                    });
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: pass,
                        params: ActionParams::default(),
                    },
                )
                .expect("passing priority should be accepted");
            }
            other => panic!("expected AwaitingHuman while hunting {label}, got {other:?}"),
        }
    }
    panic!("no {label} offer within 80 human decisions");
}

/// Drive p2 to activate Goblin Sharpshooter's `{T}` ability against `target`.
/// Returns the ability's own stack-entry id (CR 113.1c).
fn activate_sharpshooter(
    game: &mut LocalGame<StubProvider>,
    sharpshooter_id: ObjectId,
    target: Target,
) -> ObjectId {
    let (decision, idx) = drive_until(
        game,
        "ActivateAbility(Goblin Sharpshooter)",
        |a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == sharpshooter_id),
    );
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![target],
                ..ActionParams::default()
            },
        },
    )
    .expect("activating Goblin Sharpshooter should be accepted");
    find_ability_stack_id(game.state())
}

/// Drive to the exact moment p1 is offered `CastSpell(Bolt Bend)` with the ability
/// on the stack, WITHOUT submitting it -- the caller decides what to assert about
/// the offer (c1) or simply completes the cast and reads the resolution effect
/// (c2). Returns `(game, decision, action_index, bolt_bend_hand_id, ability_stack_id)`.
fn drive_to_bolt_bend_offer() -> (
    LocalGame<StubProvider>,
    PendingDecision,
    usize,
    ObjectId,
    ObjectId,
) {
    let mut game = start_human_game();
    let bolt_bend_id = find_obj(game.state(), "Bolt Bend");
    let sharpshooter_id = find_obj(game.state(), "Goblin Sharpshooter");
    let ability_id = activate_sharpshooter(&mut game, sharpshooter_id, Target::Player(p(1)));
    let (decision, cast_index) = drive_until(
        &mut game,
        "CastSpell(Bolt Bend)",
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == bolt_bend_id),
    );
    (game, decision, cast_index, bolt_bend_id, ability_id)
}

/// Pass priority for whichever player is asked until the stack is empty. Every
/// intervening decision in this fixture is an ordinary priority window (no further
/// `EffectChoice`/trigger-target asks are owed by either `Effect::ChangeTargets` or
/// `Effect::DealDamage`), so a blanket pass is correct, not merely convenient.
fn resolve_stack_fully(game: &mut LocalGame<StubProvider>) {
    for _ in 0..40 {
        if game.state().stack_objects().is_empty() {
            return;
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))
                    .unwrap_or_else(|| {
                        panic!("no PassPriority while resolving the stack: {:?}", d.actions)
                    });
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: pass,
                        params: ActionParams::default(),
                    },
                )
                .expect("pass should be accepted");
            }
            other => panic!("expected AwaitingHuman while resolving the stack, got {other:?}"),
        }
    }
    panic!("stack did not resolve within 40 human decisions");
}

#[test]
/// **c1** -- CR 115.7a / CR 601.2c: the offer reaches the human, and the exact
/// candidate the engine names is what it accepts (SR-38 in both directions: never
/// offer what the engine refuses, never refuse what it offers).
///
/// `LegalAction::CastSpell` itself carries no candidate list (verified in source:
/// its variant has `card`/`from_zone`/`additional_costs`/`alt_cost` and nothing
/// target-shaped) -- the candidate universe is a SEPARATE query
/// (`mtg_engine::spell_target_requirements` + `mtg_engine::legal_targets_per_slot`),
/// the exact pair `tools/play-server/src/view.rs::action_option_view` and
/// `mtg_simulator::targeting::plan_targets` both call. This probe calls that same
/// pair directly, which is why it proves something about the OFFER a browser or a
/// bot would actually see, not merely about the engine having a `Target::StackObject`
/// variant.
fn c1_the_offer_reaches_the_human_and_is_accepted() {
    let (mut game, decision, cast_index, bolt_bend_id, ability_id) = drive_to_bolt_bend_offer();

    let reqs = mtg_engine::spell_target_requirements(game.state(), bolt_bend_id, &[], None, false);
    assert_eq!(
        reqs.len(),
        1,
        "Bolt Bend has exactly one target slot (TargetSpellOrAbilityWithSingleTarget), \
         got {reqs:?}"
    );
    let per_slot = mtg_engine::legal_targets_per_slot(game.state(), p(1), bolt_bend_id, &reqs);
    assert_eq!(per_slot.len(), 1, "one slot in, one candidate list out");
    assert!(
        per_slot[0].contains(&Target::StackObject(ability_id)),
        "CR 115.7a / OOS-DX25b-1: the ability's own stack entry must be a legal \
         candidate for Bolt Bend's single target slot, got {:?}",
        per_slot[0]
    );

    // SR-38: submit EXACTLY the candidate the offer computation named -- not a
    // hand-picked id that merely happens to equal it.
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: cast_index,
            params: ActionParams {
                auto_tap: true,
                targets: vec![Target::StackObject(ability_id)],
                ..ActionParams::default()
            },
        },
    )
    .expect("the engine must accept the exact target it offered as a legal candidate (SR-38)");
}

#[test]
/// **c2** -- the VERDICT is the resolution effect, never the offer and never
/// `GameEvent::TargetsChanged` (that event corroborates; it is not what this test
/// reads to decide pass/fail). Goblin Sharpshooter's ability was announced against
/// p1; Bolt Bend redirects it; CR 115.7a's own candidate order excludes p1 (the
/// current target) and offers the redirecting player (p1, Bolt Bend's controller)
/// FIRST among the rest -- which is also excluded since it equals the current
/// target, so the engine lands on the next candidate, p2. The observable fact is
/// life totals: p1 takes none of Goblin Sharpshooter's damage, p2 takes all of it.
fn c2_the_ability_resolves_against_the_new_target() {
    let (mut game, decision, cast_index, _bolt_bend_id, ability_id) = drive_to_bolt_bend_offer();

    let p1_before = life(game.state(), p(1));
    let p2_before = life(game.state(), p(2));

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: cast_index,
            params: ActionParams {
                auto_tap: true,
                targets: vec![Target::StackObject(ability_id)],
                ..ActionParams::default()
            },
        },
    )
    .expect("casting Bolt Bend at the ability's stack entry should be accepted");

    resolve_stack_fully(&mut game);

    let p1_after = life(game.state(), p(1));
    let p2_after = life(game.state(), p(2));

    // THE VERDICT.
    assert_eq!(
        p1_after, p1_before,
        "CR 115.7a: the redirect must move Goblin Sharpshooter's target OFF p1 -- p1 \
         must take none of its damage (before {p1_before}, after {p1_after})"
    );
    assert_eq!(
        p2_after,
        p2_before - 1,
        "CR 115.7a: the ability must resolve against its NEW target, p2, and deal the \
         printed 1 damage there (before {p2_before}, after {p2_after})"
    );

    // Corroboration only, read AFTER the verdict above, not instead of it.
    assert!(
        game.state().stack_objects().is_empty(),
        "both Bolt Bend and the redirected ability must have fully resolved"
    );
}

/// Build the ability-on-stack state for the raw-`process_command` probes (c3/c4),
/// which never call `LocalGame::start` and so never suffer its `Step::Untap` reset
/// (PB-DX45's documented gotcha) -- deliberately NOT going through `LocalGame` here,
/// since c3's subject is the STANDALONE bot-layer functions
/// (`StubProvider::legal_actions` + `mtg_simulator::targeting::plan_targets` +
/// `RandomBot::choose_action`), mirroring `pb_dx25c_bot_retarget_is_legal.rs`'s own
/// choice of drive for the identical reason.
///
/// p2 activates Goblin Sharpshooter targeting p1, then passes priority (CR 117.3c:
/// the activator holds priority first, so this pass is what hands it to p1). p1's
/// four Mountains are tapped for mana directly (`Command::TapForMana`, mirroring
/// what `LocalGame::submit`'s `auto_tap` does, since a raw `process_command` cast
/// spends only FLOATING mana and taps nothing itself).
fn build_ability_on_stack_with_p1_funded() -> (GameState, ObjectId, ObjectId) {
    let mut state = fixture();
    state.turn_mut().priority_holder = Some(p(2));
    let sharpshooter_id = find_obj(&state, "Goblin Sharpshooter");
    let bolt_bend_id = find_obj(&state, "Bolt Bend");

    let (state, _events) = process_command(
        state,
        Command::ActivateAbility {
            player: p(2),
            source: sharpshooter_id,
            ability_index: 0,
            targets: vec![Target::Player(p(1))],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("p2 activating Goblin Sharpshooter must succeed");

    let (mut state, _events) = process_command(state, Command::PassPriority { player: p(2) })
        .expect("p2 passing priority (CR 117.3c) must succeed");

    let ability_id = find_ability_stack_id(&state);

    let mountain_ids: Vec<ObjectId> = state
        .objects()
        .iter()
        .filter(|(_, o)| o.controller == p(1) && o.characteristics.name == "Mountain")
        .map(|(id, _)| *id)
        .collect();
    assert!(
        mountain_ids.len() >= 4,
        "the fixture must give p1 at least four Mountains to fund Bolt Bend's {{3}}{{R}}: \
         {mountain_ids:?}"
    );
    for mid in mountain_ids.into_iter().take(4) {
        let (s, _events) = process_command(
            state,
            Command::TapForMana {
                player: p(1),
                source: mid,
                ability_index: 0,
                chosen_color: None,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("tapping a Mountain for mana must succeed");
        state = s;
    }

    (state, bolt_bend_id, ability_id)
}

#[test]
/// **c3** -- the BOT path. `StubProvider` offers `CastSpell(Bolt Bend)`,
/// `mtg_simulator::targeting::plan_targets` builds the announcement from the SAME
/// `legal_targets_per_slot` c1 calls directly, and `RandomBot` turns that plan into
/// a `Command`.
///
/// This fixture has exactly ONE legal candidate for Bolt Bend's mandatory single
/// target slot (the ability's own stack entry -- Goblin Sharpshooter's creature
/// object on the battlefield fails `TargetSpellOrAbilityWithSingleTarget`, and
/// neither player is a spell or ability), so "either announces it legally or
/// declines cleanly" resolves to the ANNOUNCE branch here by construction:
/// `plan_targets`'s mandatory-slot loop can only reach `TargetPlan::Unsatisfiable`
/// (the "decline cleanly" branch) when the candidate list is EMPTY, and the DECLINE
/// side of that same disjunction -- the plan comes back `Unsatisfiable` and a
/// resulting empty-target cast is CLEANLY REFUSED by the engine rather than
/// silently accepted -- is what c4(a) below proves, on a fixture built to have no
/// candidate rather than one. (The offer itself is NOT withheld in that case --
/// verified, not assumed, by c4(a)'s own first-draft correction.)
fn c3_the_bot_path_offers_and_accepts_the_stack_object_target() {
    let (state, bolt_bend_id, ability_id) = build_ability_on_stack_with_p1_funded();

    let action = StubProvider
        .legal_actions(&state, p(1))
        .into_iter()
        .find(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == bolt_bend_id))
        .unwrap_or_else(|| panic!("StubProvider must offer casting Bolt Bend"));

    // Non-vacuity anchor (PB-DX25's T6 lesson): `plan_targets` must announce a REAL
    // target, not nothing, and it must be the ability -- the only legal candidate.
    let plan = plan_targets(&state, p(1), &action);
    let TargetPlan::Announce(announced) = &plan else {
        panic!("plan_targets must announce a target for Bolt Bend, got {plan:?}");
    };
    assert_eq!(
        announced,
        &vec![Target::StackObject(ability_id)],
        "the bot layer must announce the ability's stack entry -- the ONLY legal \
         candidate on this board"
    );

    let mut bot = RandomBot::new(1, "pb-dx52-c3-bot".into());
    let cmd = bot.choose_action(&state, p(1), std::slice::from_ref(&action));
    let Command::CastSpell(cast_data) = &cmd else {
        panic!("expected a CastSpell command from the bot, got {cmd:?}");
    };
    assert_eq!(
        cast_data.targets,
        vec![Target::StackObject(ability_id)],
        "the bot-built Command::CastSpell must carry the plan's own announced target"
    );

    // SR-38: the engine must accept exactly what the offer/plan/bot layer produced.
    process_command(state, cmd)
        .expect("the engine must accept the bot-built cast targeting the ability (SR-38)");
}

#[test]
/// **c4** -- the control. Two halves, each closing a different way `c1` could pass
/// for the wrong reason.
///
/// (a) With NOTHING on the stack, Bolt Bend's mandatory single target slot has no
///     legal candidate at all. The `CastSpell` offer is NOT withheld for this
///     reason (measured, not assumed -- `StubProvider`'s only target-shaped
///     suppression is Aura-adjacent and unrelated); what SR-38 owes instead is
///     that the bot-layer PLAN declines (`TargetPlan::Unsatisfiable`) and that
///     submitting the resulting empty-target cast is cleanly refused.
/// (b) With a SPELL (not an ability) on the stack, the candidate is
///     `Target::Object(spell_stack_id)` and `Target::StackObject` must NOT
///     spuriously appear. Without this half, c1 passing could mean "every stack
///     entry, spell or ability alike, is offered as a `StackObject`" rather than
///     "an ability's entry specifically is now reachable" -- CR 115.7a's own text
///     ("target spell OR ability") depends on the two being distinguishable, which
///     is exactly what `casting::validate_stack_object_satisfies_requirement`'s
///     `is_spell` discriminator is for.
fn c4_control_no_ability_on_the_stack_yields_no_stack_object_candidate() {
    // (a). **Corrected in place, because the first draft of this probe assumed the
    // wrong shape and execution refuted it**: `StubProvider.legal_actions`
    // suppresses a `CastSpell` offer only for a COST-shaped reason
    // (`offerable_cast_plan`'s CR 118.8 mandatory-sacrifice check) -- there is no
    // sibling suppression keyed on TARGET availability, so Bolt Bend's `CastSpell`
    // action is offered here even though nothing legal exists to target. What IS
    // true, and is asserted below instead: the bot-layer PLAN for that offer is
    // `TargetPlan::Unsatisfiable` (CR 601.2c -- "if a mandatory target slot has no
    // legal candidate, the whole announcement is impossible"), and submitting the
    // resulting empty-target cast is CLEANLY REFUSED by the engine -- SR-38's other
    // half, "never silently accept a doomed command", not "never offer a doomed
    // one". Mana is tapped first so the refusal below is provably about the
    // target and not about affordability.
    let mut state_a = fixture();
    state_a.turn_mut().priority_holder = Some(p(1));
    let bolt_bend_id_a = find_obj(&state_a, "Bolt Bend");
    let mountain_ids_a: Vec<ObjectId> = state_a
        .objects()
        .iter()
        .filter(|(_, o)| o.controller == p(1) && o.characteristics.name == "Mountain")
        .map(|(id, _)| *id)
        .collect();
    assert!(mountain_ids_a.len() >= 4, "{mountain_ids_a:?}");
    for mid in mountain_ids_a.into_iter().take(4) {
        let (s, _events) = process_command(
            state_a,
            Command::TapForMana {
                player: p(1),
                source: mid,
                ability_index: 0,
                chosen_color: None,
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            },
        )
        .expect("tapping a Mountain for mana must succeed");
        state_a = s;
    }

    let offered = StubProvider.legal_actions(&state_a, p(1));
    let cast_action = offered
        .iter()
        .find(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == bolt_bend_id_a))
        .unwrap_or_else(|| {
            panic!(
                "StubProvider must still offer casting Bolt Bend here -- target \
                 suppression is not part of this offer gate: {offered:?}"
            )
        });
    let plan = plan_targets(&state_a, p(1), cast_action);
    assert!(
        matches!(plan, TargetPlan::Unsatisfiable),
        "CR 601.2c: with no legal candidate anywhere for Bolt Bend's mandatory \
         single target slot, plan_targets must decline rather than announce \
         nothing, got {plan:?}"
    );

    let cmd = Command::CastSpell(Box::new(CastSpellData {
        player: p(1),
        card: bolt_bend_id_a,
        targets: plan.announced(),
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
    }));
    let result = process_command(state_a, cmd);
    assert!(
        result.is_err(),
        "SR-38: an unsatisfiable target plan must be cleanly REFUSED by the engine, \
         never silently accepted into a wrong game state, got {result:?}"
    );

    // (b).
    let mut state_b = fixture();
    state_b.turn_mut().priority_holder = Some(p(2));
    let lightning_bolt_hand_id = find_obj(&state_b, "Lightning Bolt");
    let mountain_p2 = state_b
        .objects()
        .iter()
        .find(|(_, o)| o.controller == p(2) && o.characteristics.name == "Mountain")
        .map(|(id, _)| *id)
        .expect("fixture must give p2 a Mountain");

    let (state_b, _events) = process_command(
        state_b,
        Command::TapForMana {
            player: p(2),
            source: mountain_p2,
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("tapping p2's Mountain must succeed");

    let (state_b, _events) = process_command(
        state_b,
        Command::CastSpell(Box::new(CastSpellData {
            player: p(2),
            card: lightning_bolt_hand_id,
            targets: vec![Target::Player(p(1))],
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
    )
    .expect("p2 casting Lightning Bolt must succeed");

    let lightning_bolt_stack_id = find_obj(&state_b, "Lightning Bolt");
    let bolt_bend_id_b = find_obj(&state_b, "Bolt Bend");
    let reqs = mtg_engine::spell_target_requirements(&state_b, bolt_bend_id_b, &[], None, false);
    assert_eq!(reqs.len(), 1);
    let per_slot = mtg_engine::legal_targets_per_slot(&state_b, p(1), bolt_bend_id_b, &reqs);
    assert_eq!(per_slot.len(), 1);

    assert!(
        per_slot[0]
            .iter()
            .any(|t| *t == Target::Object(lightning_bolt_stack_id)),
        "CR 115.4/115.7a: a single-target SPELL on the stack must still be a legal \
         candidate, got {:?}",
        per_slot[0]
    );
    assert!(
        !per_slot[0]
            .iter()
            .any(|t| matches!(t, Target::StackObject(_))),
        "OOS-DX25b-1 control: no ability is on the stack here, so Target::StackObject \
         must not appear among the candidates -- this is what proves c1 is not passing \
         because every stack entry gets offered as a StackObject regardless of kind: {:?}",
        per_slot[0]
    );
}
