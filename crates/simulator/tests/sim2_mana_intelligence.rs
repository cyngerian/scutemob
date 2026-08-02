//! SIM-2 — mana intelligence: residual auto-tap, true-production accounting, bot tap
//! discipline.
//!
//! Evidence: `memory/playtest-triage-2026-08-02.md` F3 / F4 / F5, all three observed by a
//! human in the browser client before any code was read.
//!
//! CR references used throughout:
//!   * CR 106.1b / 106.4 — colorless is a mana *type*, not a colour; `{C}` is paid only
//!     with colorless mana (CR 107.4c).
//!   * CR 107.4 — generic (`{N}`) is paid with mana of any type.
//!   * CR 500.4 — pools empty at the end of each step and phase, which is why floating
//!     mana that the auto-tapper refuses to spend is *destroyed*, not saved.
//!   * CR 605.1a / 605.3a — a mana ability resolves immediately; its own activation cost
//!     may itself be mana, paid from the pool first.
//!   * CR 602.2c / 302.6 — a `{T}` cost cannot be paid by a summoning-sick creature.
//!
//! Every test in this file was **watched failing** on the pre-fix tree; the recorded
//! failure text is in `memory/workstream-state.md`'s SIM-2 handoff.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    CardDefinition, CardId, CardRegistry, Command, GameState, GameStateBuilder, ManaAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, ZoneId,
};
use mtg_simulator::{
    solve_mana_payment, ActionParams, AdvanceOutcome, Bot, HeuristicBot, HumanChoice, LegalAction,
    LegalActionProvider, LocalGame, LocalGameLimits, RandomBot, StubProvider,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A `{T}`-only mana ability producing `amount` of `color`.
fn tap_for(color: ManaColor, amount: u32) -> ManaAbility {
    ManaAbility {
        produces: [(color, amount)].into_iter().collect(),
        requires_tap: true,
        ..Default::default()
    }
}

/// A battlefield-only state for `PlayerId(1)` holding the named `(name, ability)`
/// sources. No spell, no library — the solver only ever reads the battlefield.
fn battlefield_with(sources: &[(&str, ManaAbility)]) -> (GameState, Vec<ObjectId>) {
    let mut builder = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .active_player(PlayerId(1));
    for (name, ability) in sources {
        builder = builder.object(
            ObjectSpec::land(PlayerId(1), name).with_mana_ability(ability.clone()),
        );
    }
    let state = builder.build().expect("solver fixture should build");
    let ids = sources
        .iter()
        .map(|(name, _)| {
            state
                .objects()
                .iter()
                .find(|(_, o)| o.characteristics.name == *name)
                .map(|(id, _)| *id)
                .unwrap_or_else(|| panic!("{name} must exist"))
        })
        .collect();
    (state, ids)
}

fn generic(n: u32) -> ManaCost {
    ManaCost {
        generic: n,
        ..ManaCost::default()
    }
}

/// How many distinct permanents a solved plan taps.
fn tapped_sources(plan: &[Command]) -> Vec<ObjectId> {
    plan.iter()
        .map(|c| match c {
            Command::TapForMana { source, .. } => *source,
            other => panic!("solver emitted a non-TapForMana command: {other:?}"),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// F4 — the solver counts SOURCES, not MANA
// ---------------------------------------------------------------------------

/// **The under-offer dual** (playtest F4): a `{2}` spell with only a Sol Ring untapped
/// is payable — `{C}{C}` is two mana (CR 107.4: generic takes any type) — but the
/// pre-fix solver credited the source as **1**, returned `None`, and
/// `legal_actions::can_afford` then suppressed the offer entirely. The human never saw
/// the card as castable.
#[test]
fn t1_sol_ring_pays_two_generic_alone() {
    let (state, ids) = battlefield_with(&[("SIM2 Sol Ring", tap_for(ManaColor::Colorless, 2))]);
    let plan = solve_mana_payment(&state, PlayerId(1), &generic(2))
        .expect("a {C}{C} source alone must pay a {2} cost (CR 107.4)");
    assert_eq!(
        tapped_sources(&plan),
        vec![ids[0]],
        "exactly the one source, tapped once"
    );
}

/// **The over-tap** (playtest F4, observed): Sol Ring + 2 Forests tapped for `{2}{G}` —
/// 4 mana produced, 3 spent, 1 stranded and then destroyed at the step boundary
/// (CR 500.4). With production counted truthfully the plan is 2 taps: one Forest for the
/// `{G}` pip and the Sol Ring for the `{2}`.
#[test]
fn t2_generic_phase_does_not_strand_mana() {
    let (state, ids) = battlefield_with(&[
        ("SIM2 Sol Ring", tap_for(ManaColor::Colorless, 2)),
        ("SIM2 Forest A", tap_for(ManaColor::Green, 1)),
        ("SIM2 Forest B", tap_for(ManaColor::Green, 1)),
    ]);
    let cost = ManaCost {
        generic: 2,
        green: 1,
        ..ManaCost::default()
    };
    let plan = solve_mana_payment(&state, PlayerId(1), &cost).expect("{2}{G} is payable here");
    let tapped = tapped_sources(&plan);
    assert_eq!(
        tapped.len(),
        2,
        "one Forest for the pip and the Sol Ring for the generic — not three taps: {tapped:?}"
    );
    assert!(
        tapped.contains(&ids[0]),
        "the Sol Ring is the exact fit for {{2}} and must be the generic payer"
    );
}

/// The least-waste rule is a *preference*, not a fixed order: a **1**-generic cost must
/// take the Forest and leave the Sol Ring untapped (a big source is never spent on a
/// small cost while a small one is available).
#[test]
fn t3_generic_phase_prefers_the_small_producer_for_a_small_cost() {
    let (state, ids) = battlefield_with(&[
        ("SIM2 Sol Ring", tap_for(ManaColor::Colorless, 2)),
        ("SIM2 Forest A", tap_for(ManaColor::Green, 1)),
    ]);
    let plan = solve_mana_payment(&state, PlayerId(1), &generic(1)).expect("{1} is payable");
    assert_eq!(
        tapped_sources(&plan),
        vec![ids[1]],
        "the Forest pays the single generic pip; the Sol Ring stays up"
    );
}

/// CR 107.4c: `{C}` is paid only with colorless mana. A `{C}{C}` cost must be satisfied
/// by the *two* colorless mana one Sol Ring makes, and a Forest can never contribute.
#[test]
fn t4_colorless_pips_credit_true_production() {
    let (state, ids) = battlefield_with(&[
        ("SIM2 Sol Ring", tap_for(ManaColor::Colorless, 2)),
        ("SIM2 Forest A", tap_for(ManaColor::Green, 1)),
    ]);
    let cost = ManaCost {
        colorless: 2,
        ..ManaCost::default()
    };
    let plan = solve_mana_payment(&state, PlayerId(1), &cost).expect("{C}{C} is payable");
    assert_eq!(tapped_sources(&plan), vec![ids[0]]);

    // And the Forest alone cannot pay {C} at all (CR 107.4c).
    let (green_only, _) = battlefield_with(&[("SIM2 Forest A", tap_for(ManaColor::Green, 1))]);
    let cost1 = ManaCost {
        colorless: 1,
        ..ManaCost::default()
    };
    assert!(
        solve_mana_payment(&green_only, PlayerId(1), &cost1).is_none(),
        "green mana cannot pay a {{C}} pip (CR 107.4c)"
    );
}

/// A single source producing two mana of the SAME colour pays two pips of that colour
/// (Teferi's Isle, `{U}{U}`) — the colored phase has the same source-vs-mana bug.
#[test]
fn t5_colored_pips_credit_true_production() {
    let (state, ids) = battlefield_with(&[("SIM2 Teferi's Isle", tap_for(ManaColor::Blue, 2))]);
    let cost = ManaCost {
        blue: 2,
        ..ManaCost::default()
    };
    let plan = solve_mana_payment(&state, PlayerId(1), &cost).expect("{U}{U} from one source");
    assert_eq!(tapped_sources(&plan), vec![ids[0]]);
}

/// A karoo/bounce land (`{B}{G}` from one tap) pays a `{B}{G}` cost with ONE tap, and its
/// surplus colour is spendable on generic afterwards.
#[test]
fn t6_two_colour_source_pays_both_pips_and_spills_into_generic() {
    let (state, ids) = battlefield_with(&[(
        "SIM2 Rot Farm",
        ManaAbility {
            produces: [(ManaColor::Black, 1u32), (ManaColor::Green, 1u32)]
                .into_iter()
                .collect(),
            requires_tap: true,
            ..Default::default()
        },
    )]);
    let both = ManaCost {
        black: 1,
        green: 1,
        ..ManaCost::default()
    };
    assert_eq!(
        tapped_sources(&solve_mana_payment(&state, PlayerId(1), &both).expect("{B}{G}")),
        vec![ids[0]],
    );
    // {B} + {1}: the green half of the same activation pays the generic pip (CR 107.4).
    let mixed = ManaCost {
        black: 1,
        generic: 1,
        ..ManaCost::default()
    };
    assert_eq!(
        tapped_sources(&solve_mana_payment(&state, PlayerId(1), &mixed).expect("{1}{B}")),
        vec![ids[0]],
    );
}

// ---------------------------------------------------------------------------
// F3 — auto-tap is all-or-nothing (the residual)
// ---------------------------------------------------------------------------

/// A 2-player state where `PlayerId(1)` holds a `{3}` instant and controls five
/// `{C}` sources. Five, not three: the pre-fix all-or-nothing solver plans the WHOLE
/// printed cost from untapped sources, so with only three sources the pre-fix plan
/// would fail to solve and the test would pass for the wrong reason (source exhaustion,
/// exactly the vacuity trap `state_for_auto_tap_test` documents in `local_game.rs`).
fn state_for_residual_test() -> (GameState, ObjectId, Vec<ObjectId>) {
    use mtg_engine::state::turn::Step;
    use mtg_engine::{AbilityDefinition, CardType, Effect, EffectAmount, PlayerTarget, TypeLine};

    let def = CardDefinition {
        name: "SIM2 Cantrip".to_string(),
        card_id: CardId("sim2-cantrip".to_string()),
        mana_cost: Some(generic(3)),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let mut builder = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(PlayerId(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(generic(3))
                .in_zone(ZoneId::Hand(PlayerId(1))),
        );
    for i in 0..5 {
        builder = builder.object(
            ObjectSpec::land(PlayerId(1), &format!("SIM2 Source {i}"))
                .with_mana_ability(tap_for(ManaColor::Colorless, 1)),
        );
    }
    let mut state = builder.build().expect("residual fixture should build");
    state.turn_mut().priority_holder = Some(PlayerId(1));

    let find = |name: &str| {
        state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == name)
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("{name} must exist"))
    };
    let spell = find("SIM2 Cantrip");
    let sources = (0..5).map(|i| find(&format!("SIM2 Source {i}"))).collect();
    (state, spell, sources)
}

fn start_local_game(state: GameState) -> LocalGame<StubProvider> {
    let human_seats: BTreeSet<PlayerId> = [PlayerId(1)].into_iter().collect();
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(2),
        Box::new(RandomBot::new(1, "Bot-2".to_string())),
    );
    let (game, _events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        human_seats,
        LocalGameLimits {
            max_commands: 5_000,
            max_turns: 50,
            max_consecutive_passes: 100,
            record_journal: true,
        },
        true,
    )
    .expect("game should start");
    game
}

fn action_index(actions: &[LegalAction], pred: impl Fn(&LegalAction) -> bool) -> usize {
    actions
        .iter()
        .position(pred)
        .unwrap_or_else(|| panic!("expected action not offered: {actions:?}"))
}

/// **F3, the headline**: two mana already floating and a `{3}` cast must tap exactly ONE
/// more source. Pre-fix, `auto_tap_commands_for` handed `solve_mana_payment` the entire
/// printed cost whenever the pool did not *fully* cover it, so it tapped three — and the
/// two floating mana were destroyed at the step boundary (CR 500.4).
#[test]
fn t7_auto_tap_solves_for_the_residual_after_the_pool() {
    let (state, spell, sources) = state_for_residual_test();
    let mut game = start_local_game(state);

    // Fill the pool with 2 by tapping two sources by hand, inside the same step.
    for source in sources.iter().take(2) {
        let decision = match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => d,
            other => panic!("expected AwaitingHuman, got {other:?}"),
        };
        let idx = action_index(&decision.actions, |a| {
            matches!(a, LegalAction::TapForMana { source: s, .. } if s == source)
        });
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: idx,
                params: ActionParams::default(),
            },
        )
        .expect("manual tap must succeed");
    }
    assert_eq!(
        game.state()
            .player(PlayerId(1))
            .expect("player 1")
            .mana_pool
            .total(),
        2,
        "two mana must actually be floating before the cast"
    );

    let decision = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {other:?}"),
    };
    let idx = action_index(&decision.actions, |a| {
        matches!(a, LegalAction::CastSpell { card, .. } if *card == spell)
    });
    let before = game.journal().len();
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .expect("the cast must be accepted");

    let taps = game.journal()[before..]
        .iter()
        .filter(|r| matches!(r.command, Command::TapForMana { .. }))
        .count();
    assert_eq!(
        taps, 1,
        "2 floating + a {{3}} cast must tap exactly 1 more source, not 3"
    );
    let still_untapped = sources
        .iter()
        .filter(|id| {
            !game
                .state()
                .object(**id)
                .expect("source must exist")
                .status
                .tapped
        })
        .count();
    assert_eq!(
        still_untapped, 2,
        "two sources must be left standing after a residual solve"
    );
}

// ---------------------------------------------------------------------------
// F5 — HeuristicBot taps out every empty upkeep
// ---------------------------------------------------------------------------

/// **F5**: with nothing to spend mana on, `TapForMana` (+5) beat `PassPriority` (+1) and
/// the bot deterministically emptied its board every upkeep; CR 500.4 then destroyed the
/// pool at the step boundary and it reached its main phase tapped out.
#[test]
fn t8_bot_does_not_tap_out_with_nothing_to_spend_on() {
    let (state, ids) = battlefield_with(&[
        ("SIM2 Forest A", tap_for(ManaColor::Green, 1)),
        ("SIM2 Forest B", tap_for(ManaColor::Green, 1)),
    ]);
    let legal = vec![
        LegalAction::TapForMana {
            source: ids[0],
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
        LegalAction::TapForMana {
            source: ids[1],
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
        LegalAction::PassPriority,
    ];
    let mut bot = HeuristicBot::new(7, "Bot-1".to_string());
    for _ in 0..8 {
        let cmd = bot.choose_action(&state, PlayerId(1), &legal);
        assert!(
            matches!(cmd, Command::PassPriority { .. }),
            "a bot with nothing to spend mana on must pass, not tap: {cmd:?}"
        );
    }
}

/// The non-vacuity half: scoring `TapForMana` **below** `PassPriority` must not remove
/// it. A scored-0 action is still chosen when it is the only one offered, exactly as the
/// capped-repeat damper's 0 is (`heuristic_bot.rs`'s `is_capped_repeat`) — the bot can
/// never be made unable to act.
#[test]
fn t9_a_demoted_tap_is_still_chosen_when_it_is_the_only_action() {
    let (state, ids) = battlefield_with(&[("SIM2 Forest A", tap_for(ManaColor::Green, 1))]);
    let legal = vec![LegalAction::TapForMana {
        source: ids[0],
        ability_index: 0,
        chosen_color: None,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }];
    let mut bot = HeuristicBot::new(7, "Bot-1".to_string());
    assert!(
        matches!(
            bot.choose_action(&state, PlayerId(1), &legal),
            Command::TapForMana { .. }
        ),
        "a demoted action must remain choosable when nothing else is offered"
    );
}

/// The other side of the demotion: an action that CAN consume mana still outranks both
/// the tap and the pass, so the change costs the bot no real play. `ActivateAbility`
/// scores 40 — this is the assertion that makes "every spend target already outscored a
/// tap" a checked claim rather than a comment.
#[test]
fn t9b_a_spend_target_still_outranks_both_tap_and_pass() {
    let (state, ids) = battlefield_with(&[("SIM2 Forest A", tap_for(ManaColor::Green, 1))]);
    let legal = vec![
        LegalAction::TapForMana {
            source: ids[0],
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
        LegalAction::ActivateAbility {
            source: ids[0],
            ability_index: 0,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
        LegalAction::PassPriority,
    ];
    let mut bot = HeuristicBot::new(7, "Bot-1".to_string());
    assert!(
        matches!(
            bot.choose_action(&state, PlayerId(1), &legal),
            Command::ActivateAbility { .. }
        ),
        "an activatable ability outscores both the tap and the pass"
    );
}

// ---------------------------------------------------------------------------
// Offer-gate agreement (SR-38): what the provider offers, the engine accepts
// ---------------------------------------------------------------------------

/// The offer half of F4's under-offer dual: `StubProvider` must OFFER the `{2}` spell when
/// the only untapped source is a Sol Ring, and the cast must then succeed end to end.
#[test]
fn t10_two_generic_spell_with_only_a_sol_ring_is_offered_and_succeeds() {
    use mtg_engine::state::turn::Step;
    use mtg_engine::{AbilityDefinition, CardType, Effect, EffectAmount, PlayerTarget, TypeLine};

    let def = CardDefinition {
        name: "SIM2 Two Drop".to_string(),
        card_id: CardId("sim2-two-drop".to_string()),
        mana_cost: Some(generic(2)),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };
    let mut state = GameStateBuilder::new()
        .add_player(PlayerId(1))
        .add_player(PlayerId(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(PlayerId(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(PlayerId(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(generic(2))
                .in_zone(ZoneId::Hand(PlayerId(1))),
        )
        .object(
            ObjectSpec::artifact(PlayerId(1), "SIM2 Sol Ring")
                .with_mana_ability(tap_for(ManaColor::Colorless, 2)),
        )
        .build()
        .expect("sol-ring fixture should build");
    state.turn_mut().priority_holder = Some(PlayerId(1));

    let spell = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "SIM2 Two Drop")
        .map(|(id, _)| *id)
        .expect("the spell must exist");

    let offered = StubProvider.legal_actions(&state, PlayerId(1));
    assert!(
        offered
            .iter()
            .any(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == spell)),
        "a {{2}} spell must be offered when a {{C}}{{C}} source is untapped: {offered:?}"
    );

    let mut game = start_local_game(state);
    let decision = match game.advance() {
        AdvanceOutcome::AwaitingHuman(d) => d,
        other => panic!("expected AwaitingHuman, got {other:?}"),
    };
    let idx = action_index(&decision.actions, |a| {
        matches!(a, LegalAction::CastSpell { card, .. } if *card == spell)
    });
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                auto_tap: true,
                ..Default::default()
            },
        },
    )
    .expect("the offered cast must be accepted by the engine (SR-38)");
}
