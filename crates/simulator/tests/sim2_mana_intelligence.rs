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
        builder =
            builder.object(ObjectSpec::land(PlayerId(1), name).with_mana_ability(ability.clone()));
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
        let idx = action_index(
            &decision.actions,
            |a| matches!(a, LegalAction::TapForMana { source: s, .. } if s == source),
        );
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
    let idx = action_index(
        &decision.actions,
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == spell),
    );
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
    let idx = action_index(
        &decision.actions,
        |a| matches!(a, LegalAction::CastSpell { card, .. } if *card == spell),
    );
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

// ---------------------------------------------------------------------------
// `plannable_tap_ability` — every arm mirrors a rejection in `rules/mana.rs`
//
// Each of these is a plan the pre-SIM-2 solver would happily have emitted and the engine
// would then have refused, failing the whole atomic tap-and-cast sequence on the human
// path (the `422`) and silently degrading to `PassPriority` on the bot path.
// ---------------------------------------------------------------------------

/// Build a one-source battlefield with control over the controller's life total, and
/// optionally mutate the source object after the build (for the flags `ObjectSpec` has no
/// setter for, e.g. `status.face_down`).
fn battlefield_with_life(
    name: &str,
    ability: ManaAbility,
    life: i32,
    types: Vec<mtg_engine::CardType>,
    tweak: impl FnOnce(&mut mtg_engine::GameObject),
) -> (GameState, ObjectId) {
    let mut spec = ObjectSpec::card(PlayerId(1), name)
        .with_mana_ability(ability)
        .in_zone(ZoneId::Battlefield);
    if !types.is_empty() {
        spec = spec.with_types(types);
    }
    let mut state = GameStateBuilder::new()
        .add_player_with(PlayerId(1), |p| p.life(life))
        .add_player(PlayerId(2))
        .active_player(PlayerId(1))
        .object(spec)
        .build()
        .expect("fixture should build");
    let id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name)
        .map(|(id, _)| *id)
        .expect("the source must exist");
    if let Some(obj) = state.objects_mut().get_mut(&id) {
        tweak(obj);
    }
    (state, id)
}

/// CR 302.6 / CR 702.10: a summoning-sick creature cannot pay a `{T}` cost — so a mana
/// dork played this turn is not a mana source, and `can_afford` must not count it. With
/// haste (CR 613.1f resolves it through the layer system) it is.
#[test]
fn t11_summoning_sick_mana_creature_is_not_planned() {
    use mtg_engine::{CardType, KeywordAbility};

    let (sick, _) = battlefield_with_life(
        "SIM2 Dork",
        tap_for(ManaColor::Green, 1),
        40,
        vec![CardType::Creature],
        |obj| obj.has_summoning_sickness = true,
    );
    assert!(
        solve_mana_payment(&sick, PlayerId(1), &generic(1)).is_none(),
        "a summoning-sick mana creature cannot pay a {{T}} cost (CR 302.6)"
    );

    let (hasty, id) = battlefield_with_life(
        "SIM2 Dork",
        tap_for(ManaColor::Green, 1),
        40,
        vec![CardType::Creature],
        |obj| {
            obj.has_summoning_sickness = true;
            obj.characteristics.keywords.insert(KeywordAbility::Haste);
        },
    );
    assert_eq!(
        tapped_sources(
            &solve_mana_payment(&hasty, PlayerId(1), &generic(1))
                .expect("haste beats summoning sickness (CR 702.10b)")
        ),
        vec![id],
    );

    // And the non-creature control: a land is never summoning-sick.
    let (land, land_id) = battlefield_with_life(
        "SIM2 Forest",
        tap_for(ManaColor::Green, 1),
        40,
        vec![CardType::Land],
        |obj| obj.has_summoning_sickness = true,
    );
    assert_eq!(
        tapped_sources(
            &solve_mana_payment(&land, PlayerId(1), &generic(1))
                .expect("a land with the flag set is still tappable — CR 302.6 is creatures only")
        ),
        vec![land_id],
    );
}

/// CR 605.3a / CR 118.3a: a Signet's `{1}` is paid from the pool at activation. The solver
/// does not model that interleaving, and crediting the gross `{C}{C}` while ignoring the
/// `{1}` would OVER-credit — the direction that produces a plan the engine refuses. It
/// refuses to plan them instead (`OOS-SIM2-2`).
#[test]
fn t12_ability_with_its_own_mana_cost_is_not_planned() {
    let signet = ManaAbility {
        produces: [(ManaColor::White, 1u32), (ManaColor::Blue, 1u32)]
            .into_iter()
            .collect(),
        requires_tap: true,
        mana_cost: Some(generic(1)),
        ..Default::default()
    };
    let (state, _) = battlefield_with_life("SIM2 Signet", signet, 40, vec![], |_| {});
    assert!(
        solve_mana_payment(&state, PlayerId(1), &generic(1)).is_none(),
        "a Signet is not a free two mana; the solver must not plan it"
    );
}

/// CR 119.4 / CR 118.3b: a horizon land's "Pay 1 life" is rejected outright when the
/// player cannot pay, and legal at exactly the cost (CR 119.4 permits paying to 0).
#[test]
fn t13_life_cost_source_respects_the_life_total() {
    let horizon = ManaAbility {
        produces: [(ManaColor::Black, 1u32)].into_iter().collect(),
        requires_tap: true,
        life_cost: 1,
        ..Default::default()
    };
    let (broke, _) = battlefield_with_life("SIM2 Horizon", horizon.clone(), 0, vec![], |_| {});
    assert!(
        solve_mana_payment(&broke, PlayerId(1), &generic(1)).is_none(),
        "a player at 0 life cannot pay 1 life for mana"
    );
    let (solvent, id) = battlefield_with_life("SIM2 Horizon", horizon, 1, vec![], |_| {});
    assert_eq!(
        tapped_sources(
            &solve_mana_payment(&solvent, PlayerId(1), &generic(1))
                .expect("paying down to 0 is legal (CR 119.4)")
        ),
        vec![id],
    );
}

/// CR 602.2c / CR 118.3: a counter cost with too few counters present. **No def in the
/// corpus lowers to this today** (`sim2_mana_source_roster` R4 pins that at 0), which is
/// exactly why the arm needs a synthetic test — an unexercised filter rots silently.
#[test]
fn t14_counter_cost_source_respects_the_counters_present() {
    use mtg_engine::CounterType;

    let workhorse = ManaAbility {
        produces: [(ManaColor::Colorless, 1u32)].into_iter().collect(),
        requires_tap: true,
        remove_counter: Some((CounterType::PlusOnePlusOne, 1)),
        ..Default::default()
    };
    let (empty, _) = battlefield_with_life("SIM2 Workhorse", workhorse.clone(), 40, vec![], |_| {});
    assert!(
        solve_mana_payment(&empty, PlayerId(1), &generic(1)).is_none(),
        "no counters on the permanent means the activation cost cannot be paid"
    );
    let (stocked, id) = battlefield_with_life("SIM2 Workhorse", workhorse, 40, vec![], |obj| {
        obj.counters.insert(CounterType::PlusOnePlusOne, 1);
    });
    assert_eq!(
        tapped_sources(
            &solve_mana_payment(&stocked, PlayerId(1), &generic(1))
                .expect("one counter present pays a one-counter cost")
        ),
        vec![id],
    );
}

/// CR 602.5b (SR-37): "Activate only if ..." is enforced by `handle_tap_for_mana` via
/// `check_condition`, so the solver asks the identical question rather than a lookalike.
#[test]
fn t15_activation_condition_is_honoured() {
    use mtg_engine::Condition;

    let conditioned = ManaAbility {
        produces: [(ManaColor::Red, 1u32)].into_iter().collect(),
        requires_tap: true,
        activation_condition: Some(Box::new(Condition::ControllerLifeAtLeast(30))),
        ..Default::default()
    };
    let (unmet, _) =
        battlefield_with_life("SIM2 Conditioned", conditioned.clone(), 10, vec![], |_| {});
    assert!(
        solve_mana_payment(&unmet, PlayerId(1), &generic(1)).is_none(),
        "the condition is false at 10 life, so the engine would refuse the activation"
    );
    let (met, id) = battlefield_with_life("SIM2 Conditioned", conditioned, 40, vec![], |_| {});
    assert_eq!(
        tapped_sources(
            &solve_mana_payment(&met, PlayerId(1), &generic(1)).expect("the condition holds at 40")
        ),
        vec![id],
    );
}

/// **The regression the S8 scripted playthrough caught** (seed 42, turn 21): CR 707.2 — a
/// face-down permanent has no abilities, and `layers.rs` clears `mana_abilities`
/// accordingly. Reading base characteristics planned a tap the engine answered with
/// `"object ObjectId(487) has no mana ability at index 0"`.
#[test]
fn t16_face_down_permanent_is_not_a_mana_source() {
    use mtg_engine::FaceDownKind;

    let (state, _) = battlefield_with_life(
        "SIM2 Morph",
        tap_for(ManaColor::Green, 1),
        40,
        vec![mtg_engine::CardType::Creature],
        |obj| {
            obj.status.face_down = true;
            obj.face_down_as = Some(FaceDownKind::Morph);
        },
    );
    assert!(
        solve_mana_payment(&state, PlayerId(1), &generic(1)).is_none(),
        "a face-down permanent has no mana ability to plan (CR 707.2)"
    );
}

/// CR 111.10a / CR 605.3b: an `any_color` source makes exactly ONE mana, and the colour is
/// chosen on the activation command — `handle_tap_for_mana` rejects a missing choice, and
/// equally rejects a `chosen_color` supplied for a fixed-colour ability.
#[test]
fn t17_any_color_source_announces_its_colour_and_counts_as_one() {
    let any = ManaAbility {
        produces: Default::default(),
        requires_tap: true,
        any_color: true,
        ..Default::default()
    };
    let (state, _) = battlefield_with_life("SIM2 Prism", any, 40, vec![], |_| {});

    let cost = ManaCost {
        red: 1,
        ..ManaCost::default()
    };
    let plan = solve_mana_payment(&state, PlayerId(1), &cost).expect("any colour pays {R}");
    assert!(
        matches!(
            plan.as_slice(),
            [Command::TapForMana {
                chosen_color: Some(ManaColor::Red),
                ..
            }]
        ),
        "the pip's colour must be the announced choice: {plan:?}"
    );

    let generic_plan =
        solve_mana_payment(&state, PlayerId(1), &generic(1)).expect("any colour pays {1}");
    assert!(
        matches!(
            generic_plan.as_slice(),
            [Command::TapForMana {
                chosen_color: Some(ManaColor::White),
                ..
            }]
        ),
        "a generic pip takes the deterministic White, mirroring legal_actions.rs"
    );

    // CR 106.1b: colorless is not a colour, so an any-colour source can never pay `{C}`.
    let colorless = ManaCost {
        colorless: 1,
        ..ManaCost::default()
    };
    assert!(
        solve_mana_payment(&state, PlayerId(1), &colorless).is_none(),
        "'add one mana of any color' cannot produce colorless (CR 106.1b)"
    );

    // One mana, not two: `{1}{R}` needs a second source.
    let two = ManaCost {
        red: 1,
        generic: 1,
        ..ManaCost::default()
    };
    assert!(
        solve_mana_payment(&state, PlayerId(1), &two).is_none(),
        "an any-colour source makes exactly one mana (CR 111.10a)"
    );
}

// ---------------------------------------------------------------------------
// The residual, at the solver
// ---------------------------------------------------------------------------

/// The pool subtraction must mirror `ManaPool::can_spend`: a floating `{G}` pays the `{G}`
/// pip of `{1}{G}` and NOT the generic one, leaving exactly one source to tap.
#[test]
fn t18_residual_spends_coloured_pool_on_matching_pips_first() {
    use mtg_engine::ManaPool;

    let mut pool = ManaPool::default();
    pool.add(ManaColor::Green, 1);
    let state = GameStateBuilder::new()
        .add_player_with(PlayerId(1), |p| p.mana(pool))
        .add_player(PlayerId(2))
        .active_player(PlayerId(1))
        .object(
            ObjectSpec::land(PlayerId(1), "SIM2 Forest A")
                .with_mana_ability(tap_for(ManaColor::Green, 1)),
        )
        .object(
            ObjectSpec::land(PlayerId(1), "SIM2 Forest B")
                .with_mana_ability(tap_for(ManaColor::Green, 1)),
        )
        .build()
        .expect("pool fixture should build");

    let cost = ManaCost {
        generic: 1,
        green: 1,
        ..ManaCost::default()
    };
    let plan = mtg_simulator::mana_solver::solve_mana_payment_with_pool(&state, PlayerId(1), &cost)
        .expect("{1}{G} with {G} floating and two Forests up");
    assert_eq!(
        plan.len(),
        1,
        "one floating green covers the {{G}} pip; only the generic pip needs a tap: {plan:?}"
    );

    // A pool that covers the whole cost plans nothing at all — the case the deleted
    // `can_pay_cost` early return used to handle as a special case.
    let mut full = ManaPool::default();
    full.add(ManaColor::Green, 2);
    let covered = GameStateBuilder::new()
        .add_player_with(PlayerId(1), |p| p.mana(full))
        .add_player(PlayerId(2))
        .active_player(PlayerId(1))
        .object(
            ObjectSpec::land(PlayerId(1), "SIM2 Forest A")
                .with_mana_ability(tap_for(ManaColor::Green, 1)),
        )
        .build()
        .expect("covered fixture should build");
    assert_eq!(
        mtg_simulator::mana_solver::solve_mana_payment_with_pool(&covered, PlayerId(1), &cost)
            .expect("a covering pool is solvable"),
        Vec::new(),
        "a pool that covers the cost must plan an EMPTY tap list, not a tap"
    );
}

// ---------------------------------------------------------------------------
// The offer side of the same predicate — OOS-CARDS2-9
// ---------------------------------------------------------------------------

/// SR-38, the offer half: `StubProvider` must not OFFER a `TapForMana` the engine will
/// refuse. Before SIM-2 it checked only `life_cost` (SG-1), so an unmet
/// `activation_condition` and a summoning-sick creature were both offered — and the
/// play-server test driver carried both refusal strings in its `KNOWN_FALSE_OFFERS`
/// allowlist to drive past them.
#[test]
fn t19_unactivatable_mana_abilities_are_not_offered() {
    use mtg_engine::{CardType, Condition};

    let (sick, _) = battlefield_with_life(
        "SIM2 Dork",
        tap_for(ManaColor::Green, 1),
        40,
        vec![CardType::Creature],
        |obj| obj.has_summoning_sickness = true,
    );
    assert!(
        !StubProvider
            .legal_actions(&sick, PlayerId(1))
            .iter()
            .any(|a| matches!(a, LegalAction::TapForMana { .. })),
        "a summoning-sick creature's mana ability must not be offered (CR 302.6)"
    );

    let conditioned = ManaAbility {
        produces: [(ManaColor::Red, 1u32)].into_iter().collect(),
        requires_tap: true,
        activation_condition: Some(Box::new(Condition::ControllerLifeAtLeast(30))),
        ..Default::default()
    };
    let (unmet, _) =
        battlefield_with_life("SIM2 Conditioned", conditioned.clone(), 10, vec![], |_| {});
    assert!(
        !StubProvider
            .legal_actions(&unmet, PlayerId(1))
            .iter()
            .any(|a| matches!(a, LegalAction::TapForMana { .. })),
        "an unmet activation condition must not be offered (CR 602.5b)"
    );

    // Non-vacuity: the same ability with the condition MET is still offered, so the two
    // assertions above are about the condition and not about the fixture.
    let (met, _) = battlefield_with_life("SIM2 Conditioned", conditioned, 40, vec![], |_| {});
    assert!(
        StubProvider
            .legal_actions(&met, PlayerId(1))
            .iter()
            .any(|a| matches!(a, LegalAction::TapForMana { .. })),
        "with the condition met the tap must still be offered"
    );
}

// ---------------------------------------------------------------------------
// The bot path goes through the same helper
// ---------------------------------------------------------------------------

/// `advance()`'s bot auto-tap used to call `solve_mana_payment` on the taxed printed cost
/// directly — a second code path with its own idea of the cost, justified by an asymmetry
/// argument ("a bot never has a reason to prefer its existing pool over a fresh tap") that
/// only held because the solver was pool-blind. It now calls `auto_tap_commands_for`, the
/// human path's own helper.
///
/// Proven end to end rather than by reading the call: an all-bot game where the only
/// untapped source is a `{C}{C}` rock and the only castable card costs `{2}`. Pre-SIM-2
/// the provider would not even offer the cast (F4's under-offer dual), so the bot could
/// never take it; now it is offered, planned with ONE tap, and accepted by the engine.
#[test]
fn t20_bot_seat_casts_through_the_shared_residual_helper() {
    use mtg_engine::state::turn::Step;
    use mtg_engine::{AbilityDefinition, CardType, Effect, EffectAmount, PlayerTarget, TypeLine};

    let def = CardDefinition {
        name: "SIM2 Bot Drop".to_string(),
        card_id: CardId("sim2-bot-drop".to_string()),
        mana_cost: Some(generic(2)),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
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
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(generic(2))
                .in_zone(ZoneId::Hand(PlayerId(1))),
        )
        .object(
            ObjectSpec::artifact(PlayerId(1), "SIM2 Sol Ring")
                .with_mana_ability(tap_for(ManaColor::Colorless, 2)),
        )
        .build()
        .expect("bot fixture should build");
    state.turn_mut().priority_holder = Some(PlayerId(1));

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(PlayerId(1), Box::new(HeuristicBot::new(3, "Bot-1".into())));
    bots.insert(PlayerId(2), Box::new(HeuristicBot::new(4, "Bot-2".into())));
    let (mut game, _events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        BTreeSet::new(),
        LocalGameLimits {
            max_commands: 400,
            max_turns: 3,
            max_consecutive_passes: 100,
            record_journal: true,
        },
        true,
    )
    .expect("game should start");
    let _ = game.advance();

    let casts: Vec<&Command> = game
        .journal()
        .iter()
        .map(|r| &r.command)
        .filter(|c| matches!(c, Command::CastSpell(_)))
        .collect();
    assert!(
        !casts.is_empty(),
        "the bot must be offered and take the {{2}} cast its Sol Ring pays for"
    );
    let taps = game
        .journal()
        .iter()
        .filter(|r| matches!(r.command, Command::TapForMana { .. }))
        .count();
    assert_eq!(
        taps, 1,
        "one {{C}}{{C}} activation covers the whole {{2}} cost — the bot must not tap twice"
    );
}

/// The discriminating pin for "the bot path is the SAME helper" — and therefore for the
/// bot half of `OOS-M11-8`.
///
/// `t20` proves the bot benefits from the production fix, but it would pass just as well
/// against a second, private `solve_mana_payment` call inside `advance()`. This one does
/// not, because it exercises the one thing only the shared helper knows: the **announced
/// `{X}`** (CR 107.3 / CR 601.2b — X is part of the cost from the moment it is announced).
///
/// `advance()` used to call `solve_mana_payment` on the taxed *printed* cost, so a bot
/// announcing X = 2 on an `{X}{R}` spell had one Mountain tapped for it and the engine then
/// refused the cast for want of the other two — absorbed silently by the `PassPriority`
/// fallback, which is why nothing ever caught it. S8 recorded `OOS-M11-8` as CLOSED on the
/// strength of a fix that only ever ran on the human path.
///
/// It was **latent, not live**: `RandomBot`/`HeuristicBot` both build their command from
/// `ActionParams::default()` (`random_bot::action_to_command`), so no shipped bot announces
/// a non-zero X. `XBot` below is the smallest thing that does.
struct XBot {
    x: u32,
    inner: HeuristicBot,
}

impl Bot for XBot {
    fn choose_action(
        &mut self,
        state: &GameState,
        player: PlayerId,
        legal: &[LegalAction],
    ) -> Command {
        if let Some(LegalAction::CastSpell { card, .. }) = legal
            .iter()
            .find(|a| matches!(a, LegalAction::CastSpell { .. }))
        {
            let params = ActionParams {
                x_value: self.x,
                ..Default::default()
            };
            let action = LegalAction::CastSpell {
                card: *card,
                from_zone: ZoneId::Hand(player),
                additional_costs: Default::default(),
            };
            return mtg_simulator::action_to_command_with_params(state, player, &action, &params)
                .expect("the X cast must build");
        }
        self.inner.choose_action(state, player, legal)
    }
    fn choose_targets(
        &mut self,
        state: &GameState,
        valid: &[ObjectId],
        count: usize,
    ) -> Vec<ObjectId> {
        self.inner.choose_targets(state, valid, count)
    }
    fn choose_attackers(
        &mut self,
        state: &GameState,
        eligible: &[ObjectId],
        targets: &[mtg_engine::AttackTarget],
    ) -> Vec<(ObjectId, mtg_engine::AttackTarget)> {
        self.inner.choose_attackers(state, eligible, targets)
    }
    fn choose_blockers(
        &mut self,
        state: &GameState,
        eligible: &[ObjectId],
        attackers: &[ObjectId],
    ) -> Vec<(ObjectId, ObjectId)> {
        self.inner.choose_blockers(state, eligible, attackers)
    }
    fn choose_mulligan_bottom(&mut self, hand: &[ObjectId], count: usize) -> Vec<ObjectId> {
        self.inner.choose_mulligan_bottom(hand, count)
    }
    fn name(&self) -> &str {
        self.inner.name()
    }
}

#[test]
fn t21_bot_auto_tap_includes_the_announced_x() {
    use mtg_engine::state::turn::Step;
    use mtg_engine::{AbilityDefinition, CardType, Effect, EffectAmount, PlayerTarget, TypeLine};

    let x_cost = ManaCost {
        red: 1,
        x_count: 1,
        ..ManaCost::default()
    };
    let def = CardDefinition {
        name: "SIM2 Fireball".to_string(),
        card_id: CardId("sim2-fireball".to_string()),
        mana_cost: Some(x_cost.clone()),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
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
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(x_cost)
                .in_zone(ZoneId::Hand(PlayerId(1))),
        );
    for i in 0..3 {
        builder = builder.object(
            ObjectSpec::land(PlayerId(1), &format!("SIM2 Mountain {i}"))
                .with_mana_ability(tap_for(ManaColor::Red, 1)),
        );
    }
    let mut state = builder.build().expect("X fixture should build");
    state.turn_mut().priority_holder = Some(PlayerId(1));

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        PlayerId(1),
        Box::new(XBot {
            x: 2,
            inner: HeuristicBot::new(5, "XBot-1".into()),
        }),
    );
    bots.insert(PlayerId(2), Box::new(HeuristicBot::new(6, "Bot-2".into())));
    let (mut game, _events) = LocalGame::start(
        state,
        1,
        StubProvider,
        bots,
        BTreeSet::new(),
        LocalGameLimits {
            max_commands: 200,
            max_turns: 2,
            max_consecutive_passes: 100,
            record_journal: true,
        },
        true,
    )
    .expect("game should start");
    let _ = game.advance();

    let cast = game
        .journal()
        .iter()
        .find_map(|r| match &r.command {
            Command::CastSpell(data) => Some(data.clone()),
            _ => None,
        })
        .expect(
            "the bot's X = 2 cast must be accepted — pre-SIM-2 the bot path solved for the \
             printed cost only, the engine refused the cast, and the PassPriority fallback \
             swallowed it",
        );
    assert_eq!(cast.x_value, 2, "the announced X must reach the engine");
    let taps = game
        .journal()
        .iter()
        .take_while(|r| !matches!(r.command, Command::CastSpell(_)))
        .filter(|r| matches!(r.command, Command::TapForMana { .. }))
        .count();
    assert_eq!(
        taps, 3,
        "{{X}}{{R}} with X = 2 is three mana, so three Mountains must be tapped"
    );
}

// ---------------------------------------------------------------------------
// Found by SIM-2's own `/review`: two more ways to plan a tap the engine refuses
// ---------------------------------------------------------------------------

/// CR 605.3 + CR 101.2 (`rules/mana.rs` step 1b): a `GameRestriction` that stops an
/// activated ability stops a **mana** ability. Collector Ouphe / Stony Silence
/// (`ArtifactAbilitiesCantBeActivated`) refuse a Sol Ring's tap outright.
///
/// **This class was mirrored NOWHERE on the tap path** — not in the solver, not in the
/// provider's offer loop — while four separate comments in this batch's first pass claimed
/// the mirror of `handle_tap_for_mana` was complete. Live and reachable: Collector Ouphe
/// and Stony Silence are both `Complete` and deck-legal, so an opponent playing one made
/// `can_afford` count a Sol Ring, the cast was offered, and the atomic tap-and-cast
/// sequence was then refused — the 422 this whole batch exists to remove.
#[test]
fn t22_a_stax_restricted_mana_ability_is_neither_planned_nor_offered() {
    use mtg_engine::state::ActiveRestriction;
    use mtg_engine::{CardType, GameRestriction};

    let build = |restricted: bool| {
        let mut state = GameStateBuilder::new()
            .add_player(PlayerId(1))
            .add_player(PlayerId(2))
            .active_player(PlayerId(1))
            .object(
                ObjectSpec::artifact(PlayerId(1), "SIM2 Sol Ring")
                    .with_mana_ability(tap_for(ManaColor::Colorless, 2)),
            )
            .object(
                ObjectSpec::card(PlayerId(2), "SIM2 Ouphe")
                    .with_types(vec![CardType::Creature])
                    .in_zone(ZoneId::Battlefield),
            )
            .build()
            .expect("stax fixture should build");
        let ouphe = state
            .objects()
            .iter()
            .find(|(_, o)| o.characteristics.name == "SIM2 Ouphe")
            .map(|(id, _)| *id)
            .expect("the Ouphe must exist");
        if restricted {
            state.restrictions_mut().push_back(ActiveRestriction {
                source: ouphe,
                controller: PlayerId(2),
                restriction: GameRestriction::ArtifactAbilitiesCantBeActivated,
            });
        }
        state
    };

    let restricted = build(true);
    assert!(
        solve_mana_payment(&restricted, PlayerId(1), &generic(2)).is_none(),
        "an artifact's mana ability cannot be activated under Collector Ouphe (CR 605.3)"
    );
    assert!(
        !StubProvider
            .legal_actions(&restricted, PlayerId(1))
            .iter()
            .any(|a| matches!(a, LegalAction::TapForMana { .. })),
        "and it must not be offered either (SR-38)"
    );

    // Non-vacuity: the identical board without the restriction plans and offers the tap.
    let free = build(false);
    assert!(
        solve_mana_payment(&free, PlayerId(1), &generic(2)).is_some(),
        "control: with no restriction in play the Sol Ring pays {{2}}"
    );
    assert!(
        StubProvider
            .legal_actions(&free, PlayerId(1))
            .iter()
            .any(|a| matches!(a, LegalAction::TapForMana { .. })),
        "control: with no restriction in play the tap is offered"
    );
}

/// SR-36 + CR 605.1a: a **scaled** mana ability's `produces` is a `1`-per-colour marker,
/// and `rules/mana.rs` adds `resolve_amount(..).max(0)` mana with **no error at zero**.
///
/// The batch's first pass called the marker a safe under-count that "can only under-offer".
/// That is false at zero, which is a state a `Complete` deck-legal def reaches trivially:
/// `growing_rites_of_itlimoc`'s Itlimoc face taps for one green **per creature you
/// control**, so with no creatures it produces nothing while the marker promises one — an
/// over-credit, and therefore an offered cast the engine refuses. Scaled abilities are now
/// excluded from planning outright.
#[test]
fn t23_a_scaled_mana_ability_is_never_planned() {
    use mtg_engine::{EffectAmount, TargetFilter};

    let cradle = ManaAbility {
        produces: [(ManaColor::Green, 1u32)].into_iter().collect(),
        requires_tap: true,
        scaled_amount: Some(Box::new(EffectAmount::PermanentCount {
            filter: TargetFilter {
                has_card_type: Some(mtg_engine::CardType::Creature),
                ..Default::default()
            },
            controller: mtg_engine::PlayerTarget::Controller,
        })),
        ..Default::default()
    };
    let (state, _) = battlefield_with_life("SIM2 Cradle", cradle, 40, vec![], |_| {});
    assert!(
        solve_mana_payment(&state, PlayerId(1), &generic(1)).is_none(),
        "a scaled ability's marker is not a promise of one mana — with no creatures out it \
         produces zero and the engine refuses the cast the marker bought"
    );
}
