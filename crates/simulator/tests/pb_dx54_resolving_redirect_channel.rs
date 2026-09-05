//! PB-DX54 (`OOS-DX25c-6`) — a resolving spell can BE its own redirect victim's
//! new target, through the REAL channels.
//!
//! The engine-side change lives in `crates/engine/src/rules/resolution.rs`
//! (`depart_resolving_stack_entry`, CR 608.2n): `resolve_top_of_stack_inner` used
//! to `pop_back()` the resolving stack entry before its own effect ran, so for
//! the whole of a resolution `state::stack_registry::stack_index_for_announced_
//! target` returned `None` for it. Misdirection's own 2004-10-04 ruling —
//! *"You can choose to make a spell on the stack target this spell … This spell
//! is still on the stack when new targets are selected for the spell"* — was
//! therefore unimplementable on any channel: a spell being redirected by
//! Misdirection could never have its NEW target become Misdirection's own card,
//! because Misdirection's stack entry did not exist at the moment the redirect's
//! candidate universe was built.
//!
//! **The CR warrant is CR 608.2n, not CR 608.2m** — the seed row, the v4 memo row
//! and this task's own acceptance criterion all cite 608.2m, and that citation is
//! wrong. CR 608.2m is about an object removed from the stack by SOMETHING ELSE
//! mid-resolution; it says nothing about when the resolving object's own
//! departure happens. `resolution.rs`'s own module doc corrects this in place.
//!
//! # What this file drives, and what it does not
//!
//! **C1 mixes channels, and the split is stated rather than glossed.** DECOY and
//! VICTIM are cast through a real `Command::CastSpell` (mirroring
//! `pb_dx25c_bot_retarget_is_legal.rs`'s own division of labour: getting a
//! legally-shaped `StackObject` with real `target_requirements` recorded onto it
//! requires going through the actual cast path — a hand-built `StackObject`
//! literal would have no `target_requirements` for `plan_target_change` to read,
//! which would make the whole redirect a silent no-op and defeat the point of
//! the probe). Only Misdirection's OWN cast goes through
//! `LocalGame`/`HumanChoice` — the human seat submits it via `game.submit`, not
//! a hand-built `Command`. This is deliberate, not a shortcut: DECOY and VICTIM
//! must exist on the stack *before* the `LocalGame` is started, because
//! `LocalGame::start` → `mtg_engine::start_game` unconditionally resets
//! `state.turn.step` to `Untap` and, on its way past that step (CR 502.3: no
//! player receives priority during Untap), immediately empties every floating
//! mana pool (`turn_actions::empty_all_mana_pools`, called from
//! `rules::engine::enter_step`'s auto-advance branch) — this is the mechanism
//! behind `pb_dx45_optional_cost_channel.rs`'s own note that *"a pool set on
//! `GameStateBuilder` does not survive `LocalGame::start`'s reset to
//! `Step::Untap`"*. Casting DECOY and VICTIM (which pay real mana) BEFORE that
//! reset, from mana paid out of a pool that is fully spent by the time the reset
//! runs, sidesteps the drain entirely. Misdirection itself uses CR 118.9's PITCH
//! alternative cost (exile a blue card from hand) specifically so its OWN cast,
//! which genuinely happens after the reset, needs no mana at all.
//!
//! Non-empty stack objects survive the Untap/Upkeep auto-advance inside
//! `start_game` untouched — nothing in that path reads or clears
//! `state.stack_objects` — and CR 503.1 grants the active player priority at
//! Upkeep with the stack exactly as it was left, which is the window C1 drives
//! Misdirection's cast in.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    process_command, AbilityDefinition, AdditionalCost, AltCostKind, CardDefinition,
    CardEffectTarget, CardId, CardRegistry, CardType, Color, Command, Effect, EffectAmount,
    GameEvent, GameState, GameStateBuilder, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId,
    PlayerTarget, Step, Target, TargetRequirement, TypeLine, ZoneId,
};
use mtg_simulator::params::{ActionParams, HumanChoice};
use mtg_simulator::targeting::plan_targets;
use mtg_simulator::{
    build_registry, AdvanceOutcome, Bot, HeuristicBot, LegalAction, LegalActionProvider, LocalGame,
    LocalGameLimits, PendingDecision, RandomBot, StubProvider, TargetPlan,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 54_54_54;

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Finds a `ZoneId::Stack` object by a substring of its printed name. Robust to
/// CR 400.7 id churn across zone moves, since the printed name survives a move
/// but the `ObjectId` never does.
fn find_stack_obj_on_stack(state: &GameState, name_substr: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Stack && obj.characteristics.name.contains(name_substr)
        })
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no ZoneId::Stack object containing '{}' found", name_substr))
}

/// Casts a spell via a real `Command::CastSpell`, setting priority to the caster
/// first (mirroring `pb_dx25c_bot_retarget_is_legal.rs`'s own `cast()` helper).
fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> (GameState, Vec<GameEvent>) {
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
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
    .unwrap_or_else(|e| panic!("cast must succeed: {:?}", e))
}

/// Same shape as `pb_dx25c_bot_retarget_is_legal.rs`'s own helper: pass priority
/// as whoever currently holds it until the top of the stack resolves.
fn resolve_top_of_stack(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let start_len = state.stack_objects().len();
    let mut all_events = Vec::new();
    for _ in 0..20 {
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder to resolve the stack"));
        let (s, ev) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = s;
        all_events.extend(ev);
        if state.stack_objects().len() < start_len {
            return (state, all_events);
        }
    }
    panic!(
        "stack did not resolve after 20 passes; events: {:?}",
        all_events
    );
}

/// "Target opponent loses 3 life" — a plain single-target instant. Used as
/// DECOY (C1) and as the offer-layer VICTIM (C2), mirroring
/// `pb_dx25c_bot_retarget_is_legal.rs`'s `life_loss_player_def`.
fn life_loss_player_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("{name}: target opponent loses 3 life."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::LoseLife {
                player: PlayerTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetOpponent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// "Counter target spell with a single target" — CR 115.7a's own restriction is
/// on Misdirection's requirement; VICTIM here carries the SAME requirement,
/// because it is Misdirection's own printed target announced at DECOY (a
/// single-target spell). Used as VICTIM in C1.
fn counter_single_target_spell_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("{name}: Counter target spell with a single target."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                exile_instead: false,
            },
            targets: vec![TargetRequirement::TargetSpellWithSingleTarget],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// "Destroy target creature" — a plain OBJECT-typed single-target instant,
/// mirroring `pb_dx25c_bot_retarget_is_legal.rs`'s `destroy_creature_def`. Used
/// as VICTIM in C3, the stated CONTROL: Misdirection (a `ZoneId::Stack` object,
/// never `Battlefield`, never a creature) cannot satisfy `TargetCreature`, so it
/// can never become this spell's new target no matter how the CR 608.2n fix
/// widens the candidate universe.
fn destroy_creature_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("{name}: Destroy target creature."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn limits() -> LocalGameLimits {
    LocalGameLimits {
        max_turns: 2,
        max_commands: 300,
        max_consecutive_passes: 200,
        record_journal: true,
    }
}

/// Drive the human seat, passing priority, until `want` finds an action in the
/// offered list. Returns the decision and the index of that action.
///
/// **Panics rather than returning `None`** — a probe that silently ends early is
/// a probe that asserts nothing (`pb_dx45_optional_cost_channel.rs`'s own rule).
fn drive_until(
    game: &mut LocalGame<StubProvider>,
    label: &str,
    want: impl Fn(&LegalAction) -> bool,
) -> (PendingDecision, usize) {
    for _ in 0..40 {
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
    panic!("no {label} offer within 40 human decisions");
}

/// Drains the stack by repeatedly submitting `PassPriority` for the human seat,
/// until the stack is empty or `max_iters` is exhausted.
///
/// **Deliberately does NOT collect events from `submit`'s own return value.**
/// `LocalGame::advance()`'s own doc is explicit that a human seat is the ONLY
/// thing that stops its internal loop -- every bot turn in between (here, p2's
/// own `PassPriority`, and the STACK RESOLUTION that pass triggers once both
/// players have passed) runs entirely INSIDE one `advance()` call and is never
/// returned to the caller. So the actual `TargetsChanged`/`SpellFizzled`/
/// resolution events this test cares about happen on the bot's turn, not on
/// ours, and collecting only what `submit` hands back for OUR OWN passes would
/// silently observe nothing. `game.journal()` (armed by `record_journal: true`
/// in `limits()`) is the one place that records every applied command's
/// events regardless of which seat drove it -- see
/// `drive_and_collect_from_journal`, which reads it instead.
fn drain_stack(game: &mut LocalGame<StubProvider>, max_iters: usize) {
    for _ in 0..max_iters {
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
                        panic!(
                            "expected PassPriority to be offered while draining the stack, got: {:?}",
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
                .expect("passing priority to drain the stack must be accepted");
            }
            other => panic!("expected AwaitingHuman while draining the stack, got {other:?}"),
        }
    }
    panic!("stack did not drain within {max_iters} human decisions");
}

/// Every event recorded in `game.journal()` from `cursor` onward, flattened in
/// order. `cursor` should be `game.journal().len()` taken immediately before
/// the drive whose events are being collected, so this reads only the NEW
/// entries -- both the human's own submitted commands and every bot-driven
/// command `advance()` applied internally on their behalf.
fn journal_events_since(game: &LocalGame<StubProvider>, cursor: usize) -> Vec<GameEvent> {
    game.journal_since(cursor)
        .iter()
        .flat_map(|record| record.events.clone())
        .collect()
}

/// C1 — CR 608.2n / Misdirection's 2004-10-04 ruling, end to end. DECOY and
/// VICTIM are real `Command::CastSpell` casts (see the module doc for why);
/// Misdirection's OWN cast goes through `LocalGame`/`HumanChoice`.
///
/// Asserted by RESOLUTION EFFECT, not by the offer or by the redirect event
/// alone: DECOY's own effect must actually land (p1 loses 3 life), and no
/// `GameEvent::SpellCountered` may ever appear — proving VICTIM never got to
/// execute its `CounterSpell` effect against DECOY, because its own redirected
/// target (Misdirection's card) vanished (CR 400.7: Misdirection becomes a NEW
/// object on its way to the graveyard, CR 608.2n) by the time VICTIM's own
/// CR 608.2b legality check ran.
#[test]
fn dx54_c1_misdirection_redirects_the_victim_onto_its_own_resolving_card_and_the_decoy_survives() {
    let p1 = p(1);
    let p2 = p(2);

    let decoy = life_loss_player_def("PB-DX54 C1 Decoy", "pb-dx54-c1-decoy");
    let victim = counter_single_target_spell_def("PB-DX54 C1 Victim", "pb-dx54-c1-victim");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), decoy.clone(), victim.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                blue: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(misdirection.mana_cost.clone().unwrap()),
        )
        // CR 118.9's pitch cost: a blue card in hand, otherwise unrelated to
        // this probe (never cast, so it needs no `card_id` -- the "naked
        // object" convention `pb_dx45_optional_cost_channel.rs`'s own fixture
        // uses for its "Library Filler" objects).
        .object(
            ObjectSpec::card(p1, "PB-DX54 C1 Blue Filler")
                .in_zone(ZoneId::Hand(p1))
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .with_colors(vec![Color::Blue]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX54 C1 Decoy")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(decoy.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(decoy.mana_cost.clone().unwrap()),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX54 C1 Victim")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(victim.mana_cost.clone().unwrap()),
        )
        .build()
        .expect("PB-DX54 C1 fixture must build");

    // Cast DECOY (p2 -> p1) and VICTIM (p2, targeting DECOY) BEFORE the
    // `LocalGame` exists at all -- see the module doc for why this order is
    // load-bearing, not incidental.
    let decoy_hand_id = find_obj(&state, "PB-DX54 C1 Decoy");
    let (state, _) = cast(state, p2, decoy_hand_id, vec![Target::Player(p1)]);
    let decoy_card_id = find_stack_obj_on_stack(&state, "C1 Decoy");

    let victim_hand_id = find_obj(&state, "PB-DX54 C1 Victim");
    let (state, _) = cast(
        state,
        p2,
        victim_hand_id,
        vec![Target::Object(decoy_card_id)],
    );
    let victim_card_id = find_stack_obj_on_stack(&state, "C1 Victim");

    assert_eq!(
        state.stack_objects().len(),
        2,
        "precondition: DECOY and VICTIM must both be on the stack before the \
         LocalGame is even started, or the whole point of pre-loading them is moot"
    );
    let p1_life_start = state.players().get(&p1).unwrap().life_total;

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    bots.insert(p2, Box::new(HeuristicBot::new(SEED, "p2".to_string())));
    let human: BTreeSet<PlayerId> = [p1].into_iter().collect();
    let (mut game, _start_events) =
        LocalGame::start(state, SEED, StubProvider, bots, human, limits(), true)
            .expect("PB-DX54 C1 game must start");

    // CR 503.1: the active player (p1) gets priority at Upkeep with the stack
    // untouched -- so the very first `AwaitingHuman` should already offer
    // Misdirection's pitch cast, with no PassPriority needed first.
    let misdirection_hand_id = find_obj(game.state(), "Misdirection");
    let (decision, idx) = drive_until(&mut game, "CastSpell(Misdirection, Pitch)", |a| {
        matches!(
            a,
            LegalAction::CastSpell {
                card,
                alt_cost: Some(AltCostKind::Pitch),
                ..
            } if *card == misdirection_hand_id
        )
    });

    let cursor = game.journal().len();
    game.submit(
        decision.seq,
        HumanChoice {
            action_index: idx,
            params: ActionParams {
                targets: vec![Target::Object(victim_card_id)],
                ..ActionParams::default()
            },
        },
    )
    .expect("casting Misdirection via pitch, targeting VICTIM, must be accepted");

    // Non-vacuity floor (the plan's own instruction): the stack must really
    // hold three entries right after the cast, before anything resolves.
    assert_eq!(
        game.state().stack_objects().len(),
        3,
        "CR 601.2a: DECOY + VICTIM + MISDIRECTION must all be on the stack \
         immediately after the human's cast is accepted, before any resolution"
    );
    let misdirection_stack_card_id = find_stack_obj_on_stack(game.state(), "Misdirection");

    // `advance()` stops ONLY for a human seat (see `drain_stack`'s own doc): p2's
    // own passes, and the stack resolutions they trigger, all happen INSIDE the
    // next `advance()` calls below and are recorded in the journal, not
    // returned from any `submit` we make here.
    drain_stack(&mut game, 20);
    let all_events = journal_events_since(&game, cursor);

    // 1. `TargetsChanged` really fired, naming Misdirection's own stack-resident
    //    card as VICTIM's new target -- CR 608.2n's whole point: Misdirection's
    //    entry was still ON the stack, and therefore a legal
    //    `TargetSpellWithSingleTarget` candidate, at the moment VICTIM's
    //    redirect was computed.
    let targets_changed: Vec<_> = all_events
        .iter()
        .filter_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        targets_changed.len(),
        1,
        "expected exactly one CR 115.7a retarget (Misdirection resolving once), \
         got: {:?}",
        all_events
    );
    assert_eq!(
        targets_changed[0][0].target,
        Target::Object(misdirection_stack_card_id),
        "Misdirection's own 2004-10-04 ruling: VICTIM's new target must be \
         Misdirection's own resolving card, not left unchanged and not some \
         other object"
    );

    // 2. VICTIM fizzles (CR 608.2b): by the time VICTIM's own resolution
    //    legality-checks its (redirected) target, Misdirection has already left
    //    the stack as a NEW object (CR 400.7, CR 608.2n) -- the id VICTIM was
    //    pointed at no longer names anything on the stack.
    let fizzle_count = all_events
        .iter()
        .filter(|e| matches!(e, GameEvent::SpellFizzled { .. }))
        .count();
    assert_eq!(
        fizzle_count, 1,
        "CR 608.2b: VICTIM's sole (redirected) target must be illegal by the \
         time VICTIM resolves, so it fizzles instead of countering anything; \
         events: {:?}",
        all_events
    );

    // 3. DECOY was NEVER countered -- the direct consequence of (2). Checked
    //    both by the absence of the event AND by the resolution effect it
    //    would have prevented (DECOY's own life-loss).
    assert!(
        !all_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "DECOY must never be countered -- VICTIM's CounterSpell effect never \
         runs, because VICTIM itself fizzles before its effect executes; \
         events: {:?}",
        all_events
    );
    let p1_life_end = game.state().players().get(&p1).unwrap().life_total;
    assert_eq!(
        p1_life_end,
        p1_life_start - 3,
        "DECOY's own effect (target opponent loses 3 life) must actually land \
         -- 'the original target is untouched' means p1's life total moved \
         exactly as if Misdirection and VICTIM had never been cast"
    );
    assert!(
        all_events.iter().any(
            |e| matches!(e, GameEvent::LifeLost { player, amount } if *player == p1 && *amount == 3)
        ),
        "DECOY's own LifeLost event must appear in the resolution trace; \
         events: {:?}",
        all_events
    );

    // The stack is empty -- everything that was pushed has resolved (as a
    // spell or a fizzle) by the time `drain_stack` returns.
    assert!(
        game.state().stack_objects().is_empty(),
        "the whole 3-entry stack must have resolved by the end of the drive"
    );
}

/// C2 — the bot layer OFFERS casting Misdirection at the resolving VICTIM, and
/// the engine ACCEPTS the bot-built cast (SR-38: an offer the engine then
/// refuses is worse than no offer at all).
///
/// Reached through `StubProvider::legal_actions` + `targeting::plan_targets` +
/// `RandomBot::choose_action`, mirroring `pb_dx25c_bot_retarget_is_legal.rs`'s
/// own S1 -- never a hand-built `LegalAction` or `Command`.
#[test]
fn dx54_c2_the_bot_layer_offers_and_the_engine_accepts_the_pitch_cast_at_the_victim() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let victim = life_loss_player_def("PB-DX54 C2 Victim", "pb-dx54-c2-victim");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![misdirection.clone(), victim.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(misdirection.mana_cost.clone().unwrap()),
        )
        // CR 118.9 pitch fodder -- see C1's identical fixture comment.
        .object(
            ObjectSpec::card(p1, "PB-DX54 C2 Blue Filler")
                .in_zone(ZoneId::Hand(p1))
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .with_colors(vec![Color::Blue]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX54 C2 Victim")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(victim.mana_cost.clone().unwrap()),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .expect("PB-DX54 C2 fixture must build");

    // p2 casts VICTIM at p3 -- a real cast, not this probe's subject (mirroring
    // `pb_dx25c_bot_retarget_is_legal.rs`'s own division of labour).
    let victim_hand_id = find_obj(&state, "PB-DX54 C2 Victim");
    let (state, _) = cast(state, p2, victim_hand_id, vec![Target::Player(p3)]);
    let victim_card_id = find_stack_obj_on_stack(&state, "C2 Victim");

    let mut state = state;
    state.turn_mut().priority_holder = Some(p1);
    let misdirection_hand_id = find_obj(&state, "Misdirection");

    // SR-38: assert by MEMBERSHIP in `StubProvider`'s own answer, never by a
    // hand-built action.
    let offers = StubProvider.legal_actions(&state, p1);
    let action = offers
        .iter()
        .find(|a| {
            matches!(
                a,
                LegalAction::CastSpell {
                    card,
                    alt_cost: Some(AltCostKind::Pitch),
                    ..
                } if *card == misdirection_hand_id
            )
        })
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "StubProvider must offer casting Misdirection via its pitch cost, got: {:?}",
                offers
            )
        });

    // Non-vacuity anchor (PB-DX25's T6 lesson, re-cited by
    // `pb_dx25c_bot_retarget_is_legal.rs`): `plan_targets` must announce a REAL
    // target, not nothing -- and VICTIM is the only single-target spell on the
    // stack, so it is the only legal candidate at cast time.
    let plan = plan_targets(&state, p1, &action);
    let TargetPlan::Announce(announced) = &plan else {
        panic!(
            "plan_targets must announce a target for Misdirection, got {:?}",
            plan
        );
    };
    assert_eq!(
        announced,
        &vec![Target::Object(victim_card_id)],
        "the bot layer must announce VICTIM (the only single-target spell on \
         the stack) as Misdirection's target"
    );

    let mut bot = RandomBot::new(1, "dx54-c2-bot".into());
    let cmd = bot.choose_action(&state, p1, std::slice::from_ref(&action));
    let Command::CastSpell(cast_data) = &cmd else {
        panic!("expected a CastSpell command from the bot, got {:?}", cmd);
    };
    assert_eq!(
        cast_data.alt_cost,
        Some(AltCostKind::Pitch),
        "the offered action's own alt_cost judgment must be forwarded verbatim"
    );
    assert_eq!(
        cast_data.targets,
        vec![Target::Object(victim_card_id)],
        "the bot-built Command::CastSpell must carry the same target plan_targets announced"
    );
    assert!(
        cast_data
            .additional_costs
            .iter()
            .any(|c| matches!(c, AdditionalCost::ExileFromHand { .. })),
        "the bot-built cast must carry a real CR 118.9 pitch payment, not an \
         empty additional-costs list that would make casting.rs refuse it: {:?}",
        cast_data.additional_costs
    );

    let (state, _) = process_command(state, cmd).unwrap_or_else(|e| {
        panic!(
            "SR-38: the engine must accept the bot-built Misdirection cast it \
             itself offered: {:?}",
            e
        )
    });

    assert_eq!(
        state.stack_objects().len(),
        2,
        "the bot-driven cast must actually push a SECOND entry onto the stack \
         (VICTIM was already there) -- an offer that is accepted but pushes \
         nothing would be a different, worse defect than a refusal"
    );
}

/// C3 — a stated CONTROL, expected GREEN both before and after the CR 608.2n
/// fix. VICTIM's own requirement here is `TargetCreature`: Misdirection's card
/// sits in `ZoneId::Stack`, never `Battlefield`, and is never a creature, so no
/// widening of the redirect's candidate universe can ever make it a legal
/// `TargetCreature` candidate. CR 115.7a's own fallback applies -- "if a target
/// can't be changed to another legal target, the original target is
/// unchanged" -- because there is no SECOND creature on the battlefield for the
/// redirect to land on either.
#[test]
fn dx54_c3_control_misdirection_cannot_satisfy_a_creature_shaped_victim_requirement() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let destroy = destroy_creature_def("PB-DX54 C3 Destroy", "pb-dx54-c3-destroy");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), destroy.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p2,
            ManaPool {
                black: 1,
                colorless: 1,
                ..Default::default()
            },
        )
        .object(ObjectSpec::creature(p3, "C3 Sole Creature", 2, 2))
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(misdirection.mana_cost.clone().unwrap()),
        )
        // CR 118.9 pitch fodder -- see C1's identical fixture comment.
        .object(
            ObjectSpec::card(p1, "PB-DX54 C3 Blue Filler")
                .in_zone(ZoneId::Hand(p1))
                .with_mana_cost(ManaCost {
                    blue: 1,
                    ..Default::default()
                })
                .with_colors(vec![Color::Blue]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX54 C3 Destroy")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(destroy.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(destroy.mana_cost.clone().unwrap()),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .expect("PB-DX54 C3 fixture must build");

    let sole_creature_id = find_obj(&state, "C3 Sole Creature");
    let destroy_hand_id = find_obj(&state, "PB-DX54 C3 Destroy");
    let (state, _) = cast(
        state,
        p2,
        destroy_hand_id,
        vec![Target::Object(sole_creature_id)],
    );
    let destroy_card_id = find_stack_obj_on_stack(&state, "C3 Destroy");

    let blue_filler_id = find_obj(&state, "PB-DX54 C3 Blue Filler");
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = {
        let mut state = state;
        state.turn_mut().priority_holder = Some(p1);
        process_command(
            state,
            Command::CastSpell(Box::new(CastSpellData {
                player: p1,
                card: misdirection_hand_id,
                targets: vec![Target::Object(destroy_card_id)],
                convoke_creatures: vec![],
                improvise_artifacts: vec![],
                delve_cards: vec![],
                kicker_times: 0,
                alt_cost: Some(AltCostKind::Pitch),
                prototype: false,
                modes_chosen: vec![],
                x_value: 0,
                face_down_kind: None,
                additional_costs: vec![AdditionalCost::ExileFromHand {
                    card: blue_filler_id,
                }],
                hybrid_choices: vec![],
                phyrexian_life_payments: vec![],
            })),
        )
        .unwrap_or_else(|e| panic!("casting Misdirection via pitch must succeed: {:?}", e))
    };

    assert_eq!(
        state.stack_objects().len(),
        2,
        "precondition: DESTROY + MISDIRECTION must both be on the stack before \
         Misdirection resolves"
    );

    // Resolve Misdirection. CONTROL: no TargetsChanged, because Misdirection
    // cannot satisfy TargetCreature and there is no second creature to redirect
    // onto either -- CR 115.7a's fallback ("the original target is unchanged").
    let (state, resolve_events) = resolve_top_of_stack(state);
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "CONTROL: Misdirection resolving against a TargetCreature-shaped victim \
         must never fire TargetsChanged -- it is not a creature and there is no \
         alternative one, so CR 115.7a's fallback applies; events: {:?}",
        resolve_events
    );

    // Resolve DESTROY. It must still see its ORIGINAL target and destroy it --
    // proving the fallback really left the target alone rather than merely
    // suppressing the event.
    let (state, resolve_events) = resolve_top_of_stack(state);
    assert!(
        !state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "C3 Sole Creature" && o.zone == ZoneId::Battlefield),
        "CONTROL: the unredirected DESTROY must still destroy its original \
         (unchanged) target; resolve events: {:?}",
        resolve_events
    );
    assert!(
        state.stack_objects().is_empty(),
        "both stack entries must have resolved by now"
    );
}

// Exercise `build_registry` so the import is not flagged as unused by a build
// configuration that does not otherwise reach it in this file -- kept as a
// cheap non-vacuity sentinel that the simulator's own default registry is at
// least constructible, since every other test here builds a narrow ad hoc one.
#[test]
fn dx54_sentinel_build_registry_is_constructible() {
    let registry = build_registry();
    assert!(
        registry.get(CardId("misdirection".to_string())).is_some(),
        "the simulator's default registry must carry the real Misdirection def, \
         since C1/C2/C3 all cast a hand-assembled registry containing it \
         directly rather than this one -- this sentinel is what proves the two \
         registries are not silently diverging on that card's presence"
    );
}
