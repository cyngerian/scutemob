//! PB-DX55 Half 1 — `OOS-SIM6-3`: auto-tap covers `CastSpell` alone.
//!
//! `LocalGame::auto_tap_commands_for` used to open with
//! `let Command::CastSpell(cast) = command else { return None; };`, so every
//! OTHER mana-charging command was applied with whatever mana happened to
//! already be floating, on both the human `submit` path and the bot path in
//! `advance()`. The offer gate (`legal_actions::can_afford`, pool + untapped
//! sources) and the engine's own charge (the pool alone, no auto-tap) disagreed
//! for `ActivateAbility`, `TapForMana`, `DeclareAttackers`'s CR 508.1h attack
//! tax, `TurnFaceUp`, `ActivateBloodrush`, and a `pay: true`
//! `PayEcho`/`PayCumulativeUpkeep`/`PayRecover` — measured at HEAD (stage 0) as
//! **18** `InsufficientMana` refusals on the `activate` class across the
//! `sim5_bot_cast_discipline` A/B seeds. This file drives the REAL channels
//! (`LocalGame`/`HumanChoice`, `StubProvider`'s offer layer, and
//! `legal_actions::command_mana_cost` directly) rather than asserting the offer
//! alone — the `kaito_shizuki` lesson (PB-DX43): existence is never sufficiency.
//!
//! Engine-side census and the exhaustive `command_mana_cost` match itself live
//! in `crates/simulator/src/legal_actions.rs`; see that function's own doc for
//! the full 45-variant census and its stated, honest limitations
//! (`OOS-DX55-8`, `OOS-DX55-9`).

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, CardDefinition, CounterType, GameState,
    GameStateBuilder, ManaAbility, ManaColor, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId,
    ZoneId,
};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::{
    build_registry, legal_actions, AdvanceOutcome, Bot, HeuristicBot, LegalAction,
    LegalActionProvider, LocalGame, LocalGameLimits, PendingDecision, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 55_55_55;

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

fn id_of(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name)
        .unwrap_or_else(|| panic!("no object named {name:?} in state"))
        .id
}

fn library_filler(state: GameStateBuilder, player: PlayerId) -> GameStateBuilder {
    let mut builder = state;
    for i in 0..30 {
        builder = builder.object(
            ObjectSpec::card(player, &format!("PB-DX55 Library Filler {i}"))
                .in_zone(ZoneId::Library(player)),
        );
    }
    builder
}

/// `p1` controls a real, `Complete`, deck-legal **Karn's Bastion**
/// ("{4}, {T}: Proliferate.", CR 602.2 — not a mana ability; its printed
/// "{T}: Add {C}" ability lowers into a separate `mana_abilities` slot per
/// `mana_ability_lowering`, so `activated_abilities[0]` is the Proliferate
/// ability alone) and 4 real Forests, plus a naked, card-id-less "Counter
/// Bearer" creature carrying one `+1/+1` counter — a resolution-effect witness,
/// not decoration: CR 122.4's Proliferate resolves DETERMINISTICALLY in this
/// engine (every eligible counter-bearing permanent, no interactive choice), so
/// its count going 1 -> 2 is exactly the ability having actually resolved,
/// distinguishing "the activation was accepted" from "the activation did
/// nothing."
///
/// `forests_tapped` selects the two probes' only difference (t1 vs t1b).
fn fixture(forests_tapped: bool) -> GameState {
    let defs = card_defs_by_name();
    let bastion = enrich_spec_from_def(
        ObjectSpec::land(p(1), "Karn's Bastion").with_card_id(card_name_to_id("Karn's Bastion")),
        &defs,
    );
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(bastion)
        .object(
            ObjectSpec::creature(p(1), "PB-DX55 Counter Bearer", 2, 2)
                .with_counter(CounterType::PlusOnePlusOne, 1),
        );
    for i in 0..4 {
        // A synthetic basic-land-shaped source ("{T}: Add {G}", CR 305.6),
        // built directly with `with_mana_ability` rather than
        // `enrich_spec_from_def("Forest", ..)`: `enrich_spec_from_def` looks
        // the def up by the object's OWN `name` field
        // (`defs.get(&spec.name)`), so four objects sharing the literal name
        // "Forest" could not be told apart afterward by `id_of`, and four
        // objects named distinctly ("PB-DX55 Forest 0" etc.) would never
        // match the def at all and would enrich to nothing. This sidesteps
        // that trap entirely while still being a real, printed-shape mana
        // ability (CR 605) `mana_solver::gather_sources` can tap.
        let mut forest = ObjectSpec::land(p(1), &format!("PB-DX55 Forest {i}"))
            .with_mana_ability(ManaAbility::tap_for(ManaColor::Green));
        if forests_tapped {
            forest = forest.tapped();
        }
        builder = builder.object(forest);
    }
    builder = library_filler(builder, p(1));
    builder = library_filler(builder, p(2));
    builder
        .build()
        .expect("PB-DX55 activation fixture must build")
}

fn start_human_game(forests_tapped: bool) -> LocalGame<StubProvider> {
    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p(2), Box::new(HeuristicBot::new(SEED, "p2".to_string())));
    let human: BTreeSet<PlayerId> = [p(1)].into_iter().collect();
    let (game, _events) = LocalGame::start(
        fixture(forests_tapped),
        SEED,
        StubProvider,
        bots,
        human,
        limits(),
        true,
    )
    .expect("PB-DX55 activation game must start");
    game
}

/// Drive the human seat, passing priority, until `want` finds an action in the
/// offered list. **Does not panic on exhaustion** (unlike the PB-DX45 sibling
/// helper) — `t1b` needs to prove the action is NEVER offered, and a helper
/// that only knows how to succeed cannot be reused for that.
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    want: impl Fn(&LegalAction) -> bool,
    max_steps: usize,
) -> Option<(PendingDecision, usize)> {
    for _ in 0..max_steps {
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                if let Some(i) = d.actions.iter().position(&want) {
                    return Some((d, i));
                }
                let pass = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::PassPriority))?;
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: pass,
                        params: ActionParams::default(),
                    },
                )
                .expect("passing priority should always be accepted");
            }
            other => panic!("expected AwaitingHuman while driving, got {other:?}"),
        }
    }
    None
}

fn pool_of(state: &GameState, player: PlayerId) -> ManaPool {
    state
        .players()
        .get(&player)
        .expect("player exists")
        .mana_pool
        .clone()
}

fn counters_on(state: &GameState, name: &str) -> u32 {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name)
        .and_then(|o| o.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0)
}

fn is_tapped(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name)
        .map(|o| o.status.tapped)
        .unwrap_or(false)
}

/// CR 602.2a/602.2b, `OOS-SIM6-3`'s headline: a human with an EMPTY mana pool
/// and untapped lands can activate a real mana-cost ability. Asserted by
/// RESOLUTION EFFECT (the counter actually goes up), never by the offer alone.
#[test]
fn t1_human_activates_a_real_mana_cost_ability_with_an_empty_pool_via_auto_tap() {
    let mut game = start_human_game(false);
    assert_eq!(
        pool_of(game.state(), p(1)).total(),
        0,
        "precondition: p1's pool must be empty, or funding it proves nothing about \
         auto-tap"
    );
    assert_eq!(
        counters_on(game.state(), "PB-DX55 Counter Bearer"),
        1,
        "precondition: the witness creature starts with exactly one +1/+1 counter"
    );

    let bastion_id = id_of(game.state(), "Karn's Bastion");
    let (decision, idx) = drive_until(
        &mut game,
        |a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == bastion_id),
        80,
    )
    .expect("Karn's Bastion's Proliferate ability must be offered even with an empty pool");
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    )
    .expect("activating with an empty pool must succeed once auto-tap funds it");

    // CR 602.2: activating pushes the ability onto the stack; it does not
    // resolve until both players pass priority with an empty stack (nothing
    // holds up Proliferate here -- it has no target). Drive that far, but no
    // further, so this does not risk crossing a later untap step.
    let _ = drive_until(&mut game, |_| false, 4);

    assert_eq!(
        counters_on(game.state(), "PB-DX55 Counter Bearer"),
        2,
        "CR 700-series: the ability actually resolved and Proliferate added a second \
         +1/+1 counter -- the resolution effect, not merely an accepted submission"
    );
    assert!(
        is_tapped(game.state(), "Karn's Bastion"),
        "the ability's own {{T}} cost component must have been paid"
    );
    for i in 0..4 {
        assert!(
            is_tapped(game.state(), &format!("PB-DX55 Forest {i}")),
            "auto-tap must have tapped every Forest to fund the {{4}} generic component"
        );
    }
    assert_eq!(
        pool_of(game.state(), p(1)).total(),
        0,
        "4 green mana tapped, {{4}} generic paid -- nothing should be left floating"
    );
}

/// CR 602.2a/602.2b, SR-38's OTHER direction: with the funding sources already
/// tapped and the pool still empty, the ability must NOT be offered, and a
/// direct `process_command` bypassing the offer layer must be refused.
///
/// **Getting to "tapped lands, empty pool" honestly is not "build the fixture
/// pre-tapped."** `LocalGame::start` runs the real CR 502.1 untap step as a
/// turn-based action on entering turn 1, which UNTAPS every permanent the
/// active player controls -- a fixture built with `.tapped()` Forests comes
/// back untapped the moment the game actually starts, discovered by this
/// probe's own precondition failing when first written. So this probe taps
/// the four Forests for real (an ordinary `TapForMana` action each, no
/// funding needed since a plain "{T}: Add {G}" ability has no activation cost
/// of its own -- see `t2`), then passes priority through a step boundary so
/// CR 500.4 empties the mana pool while the Forests stay tapped (nothing
/// short of the NEXT untap step untaps them again).
#[test]
fn t1b_activation_is_neither_offered_nor_accepted_with_tapped_lands_and_an_empty_pool() {
    let mut game = start_human_game(false);

    for i in 0..4 {
        let forest_id = id_of(game.state(), &format!("PB-DX55 Forest {i}"));
        let (decision, idx) = drive_until(
            &mut game,
            |a| matches!(a, LegalAction::TapForMana { source, .. } if *source == forest_id),
            80,
        )
        .unwrap_or_else(|| panic!("Forest {i} must offer TapForMana"));
        game.submit(
            decision.seq,
            HumanChoice {
                action_index: idx,
                params: ActionParams::default(),
            },
        )
        .unwrap_or_else(|e| panic!("tapping Forest {i} for mana must succeed: {e:?}"));
    }
    assert_eq!(
        pool_of(game.state(), p(1)).total(),
        4,
        "precondition: all four Forests must have actually produced mana"
    );

    // CR 500.4: pass priority (never picking any OTHER action) until a step
    // transition empties the mana pool. Bounded low enough to stay inside
    // turn 1 -- crossing into turn 2's untap step would untap the Forests
    // again and defeat the whole point of this probe.
    let _ = drive_until(&mut game, |_| false, 6);
    assert_eq!(
        pool_of(game.state(), p(1)).total(),
        0,
        "CR 500.4: the mana pool must be empty after at least one step transition"
    );
    for i in 0..4 {
        assert!(
            is_tapped(game.state(), &format!("PB-DX55 Forest {i}")),
            "the Forests must still be tapped -- only an untap step clears that, and              none has happened since they were tapped"
        );
    }

    let bastion_id = id_of(game.state(), "Karn's Bastion");
    // A SNAPSHOT check via `StubProvider::legal_actions` directly, not another
    // `drive_until` -- driving further risks crossing into turn 2's own untap
    // step, which would untap the Forests again and silently fund the very
    // activation this probe is proving is unaffordable.
    let actions = StubProvider.legal_actions(game.state(), p(1));
    let offered = actions.iter().position(
        |a| matches!(a, LegalAction::ActivateAbility { source, .. } if *source == bastion_id),
    );
    assert!(
        offered.is_none(),
        "SR-38: an unaffordable activation (empty pool, no untapped sources) must not \
         be offered at all -- got {:?}",
        offered
    );

    // Force it through directly, bypassing the offer layer entirely, to prove the
    // ENGINE also refuses it (not merely that the offer withheld it).
    let ability_index =
        mtg_engine::rules::layers::calculate_characteristics(game.state(), bastion_id)
            .expect("Karn's Bastion resolves characteristics")
            .activated_abilities
            .iter()
            .position(|_| true)
            .expect("Karn's Bastion has at least one activated ability");
    let result = mtg_engine::process_command(
        game.state().clone(),
        mtg_engine::Command::ActivateAbility {
            player: p(1),
            source: bastion_id,
            ability_index,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );
    assert!(
        result.is_err(),
        "the engine must refuse an activation it cannot fund from the floating pool alone"
    );
}

/// `command_mana_cost` is a focused unit under test on its own: `Some` for a
/// real activation with a mana cost, `None` for a mana ability with no
/// activation cost of its own (the "{T}: Add {C}" half of the SAME permanent
/// this file's other probes activate the OTHER half of).
#[test]
fn t2_command_mana_cost_some_for_the_costed_ability_none_for_the_free_mana_ability() {
    let state = fixture(false);
    let bastion_id = id_of(&state, "Karn's Bastion");

    let activate_cmd = mtg_engine::Command::ActivateAbility {
        player: p(1),
        source: bastion_id,
        ability_index: 0,
        targets: vec![],
        discard_card: None,
        sacrifice_target: None,
        x_value: None,
        modes_chosen: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    };
    let cost = legal_actions::command_mana_cost(&state, p(1), &activate_cmd);
    assert_eq!(
        cost,
        Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        "the Proliferate ability's activation cost must be {{4}} generic, exactly as \
         printed"
    );

    let tap_cmd = mtg_engine::Command::TapForMana {
        player: p(1),
        source: bastion_id,
        ability_index: 0,
        chosen_color: None,
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    };
    assert_eq!(
        legal_actions::command_mana_cost(&state, p(1), &tap_cmd),
        None,
        "a mana ability with no activation cost of its own (\"{{T}}: Add {{C}}\") must \
         report None -- there is nothing for auto-tap to fund"
    );
}

/// CR 702.30a's "under-offer" dual (`OOS-SIM6-3`): with an empty pool and
/// untapped sources, `PayEcho { pay: true }` must now be OFFERED (it was
/// pool-only-gated before this batch) and, once submitted, must actually PAY —
/// the permanent survives rather than being sacrificed. Built by seeding
/// `pending_echo_payments` directly (the established idiom this file's sibling
/// `legal_actions.rs` unit tests already use for this exact channel), which
/// isolates "is the payment channel funded and honoured" from "does the CR
/// 702.30a upkeep trigger correctly queue the payment" -- a property already
/// covered by the engine's own mechanics tests and not this batch's subject.
#[test]
fn t3_pay_echo_true_is_offered_with_an_empty_pool_and_actually_pays() {
    let mut forests = Vec::new();
    for i in 0..2 {
        // See `fixture`'s doc for why these are synthetic mana sources rather
        // than `enrich_spec_from_def("Forest", ..)` instances.
        forests.push(
            ObjectSpec::land(p(1), &format!("PB-DX55 Echo Forest {i}"))
                .with_mana_ability(ManaAbility::tap_for(ManaColor::Green)),
        );
    }
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(build_registry())
        .active_player(p(1))
        .object(ObjectSpec::creature(p(1), "PB-DX55 Echo Permanent", 2, 2));
    for forest in forests {
        builder = builder.object(forest);
    }
    builder = library_filler(builder, p(1));
    builder = library_filler(builder, p(2));
    let mut state = builder.build().expect("PB-DX55 echo fixture must build");
    let perm = id_of(&state, "PB-DX55 Echo Permanent");
    state.pending_echo_payments_mut().push_back((
        p(1),
        perm,
        ManaCost {
            generic: 2,
            ..Default::default()
        },
    ));

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(
        p(2),
        Box::new(HeuristicBot::new(SEED, "p2echo".to_string())),
    );
    let human: BTreeSet<PlayerId> = [p(1)].into_iter().collect();
    let (mut game, _events) =
        LocalGame::start(state, SEED, StubProvider, bots, human, limits(), true)
            .expect("PB-DX55 echo game must start");

    assert_eq!(
        pool_of(game.state(), p(1)).total(),
        0,
        "precondition: p1's pool must be empty"
    );

    let (decision, idx) = drive_until(
        &mut game,
        |a| matches!(a, LegalAction::PayEcho { permanent, pay: true } if *permanent == perm),
        80,
    )
    .expect(
        "PayEcho { pay: true } must be offered with an empty pool when untapped sources \
         can cover it (this is the widened gate this batch ships)",
    );

    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                auto_tap: true,
                ..ActionParams::default()
            },
        },
    )
    .expect("paying echo with an empty pool must succeed once auto-tap funds it");

    let still_exists = game
        .state()
        .objects()
        .values()
        .any(|o| o.id == perm && o.zone == ZoneId::Battlefield);
    assert!(
        still_exists,
        "CR 702.30a: paying echo keeps the permanent on the battlefield -- it must NOT \
         have been sacrificed"
    );
    assert_eq!(
        pool_of(game.state(), p(1)).total(),
        0,
        "both Forests were tapped for exactly the {{2}} generic echo cost"
    );
}

/// A mechanism gate, not a behavioural one: `auto_tap_commands_for` must
/// contain no `let Command::… else` narrowing, and `command_mana_cost`'s match
/// must contain no `_ =>` wildcard arm. Parsed from source, with a
/// NON-VACUITY floor -- a source gate that silently finds nothing has passed
/// for the wrong reason.
#[test]
fn r1_no_command_narrowing_and_no_wildcard_arm() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let local_game_src = std::fs::read_to_string(manifest_dir.join("src/local_game.rs"))
        .expect("src/local_game.rs must be readable");
    let legal_actions_src = std::fs::read_to_string(manifest_dir.join("src/legal_actions.rs"))
        .expect("src/legal_actions.rs must be readable");

    // Isolate `auto_tap_commands_for`'s body.
    let fn_start = local_game_src
        .find("fn auto_tap_commands_for(")
        .expect("auto_tap_commands_for must exist in local_game.rs");
    let body_start = local_game_src[fn_start..]
        .find('{')
        .map(|i| fn_start + i)
        .expect("auto_tap_commands_for must have a body");
    // The function is short; a fixed-size window is enough to capture its whole
    // body without needing a real brace-matcher, and is proven non-vacuous below.
    let body_end = (body_start + 600).min(local_game_src.len());
    let body = &local_game_src[body_start..body_end];
    assert!(
        body.len() > 100,
        "non-vacuity floor: the captured body window is suspiciously short -- the \
         function may have moved or been renamed"
    );
    assert!(
        body.contains("command_mana_cost"),
        "auto_tap_commands_for must call legal_actions::command_mana_cost -- got:\n{body}"
    );
    assert!(
        !body.contains("let Command::"),
        "auto_tap_commands_for must contain no `let Command::… else` narrowing -- the \
         defect this batch closes was exactly one such narrowing to `CastSpell` alone. \
         Body:\n{body}"
    );

    // Isolate `command_mana_cost`'s whole `match` body (from its `match command {`
    // to the function's own closing brace, found by counting braces).
    let fn_start = legal_actions_src
        .find("pub fn command_mana_cost(")
        .expect("command_mana_cost must exist in legal_actions.rs");
    let match_start = legal_actions_src[fn_start..]
        .find("match command {")
        .map(|i| fn_start + i)
        .expect("command_mana_cost must open with `match command {`");
    let mut depth: i32 = 0;
    let mut end = match_start;
    for (offset, ch) in legal_actions_src[match_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = match_start + offset + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(
        end > match_start,
        "brace-matching command_mana_cost's match arm failed to close -- source may \
         have changed shape"
    );
    let match_body = &legal_actions_src[match_start..end];
    assert!(
        match_body.len() > 5_000,
        "non-vacuity floor: command_mana_cost's match body is suspiciously short ({} \
         bytes) -- the exhaustive census may have been collapsed",
        match_body.len()
    );
    // A wildcard arm reads `_ =>` at the top level of the match (not inside a
    // nested `match method { ... }` or similar, which this file's arms do use
    // for `TurnFaceUpMethod`/`CumulativeUpkeepCost`). Detecting "at the top
    // level" precisely needs a real parser; detecting "anywhere at all" is a
    // sound OVER-approximation for this gate's purpose, because ADDING a
    // top-level wildcard arm to satisfy exhaustiveness is exactly the shape
    // this gate exists to forbid, and every one of this file's existing
    // NESTED wildcards is a `_ => None`/`_ => Some(...)` arm inside a small,
    // fully-covered enum match (`TurnFaceUpMethod`, `AbilityDefinition::Morph`
    // family) -- reworded here as an explicit denylist so the gate states its
    // own scope rather than silently trusting the over-approximation.
    let forbidden_top_level_wildcard = "\n        _ =>";
    assert!(
        !match_body.contains(forbidden_top_level_wildcard),
        "command_mana_cost's outer `match command {{ .. }}` must have no `_ =>` \
         wildcard arm at the Command-variant level -- a new mana-charging Command \
         variant must be a compile error until classified, not silently swallowed"
    );
}
