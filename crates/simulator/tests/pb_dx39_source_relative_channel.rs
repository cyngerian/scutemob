//! PB-DX39 (`OOS-DX5-3` + `OOS-DX5-7`'s residual) — a source-relative
//! `EffectFilter` whose source has left the battlefield, through the REAL
//! channels.
//!
//! The engine-side probes live in
//! `crates/engine/tests/primitives/pb_dx39_source_relative_lki.rs` and isolate
//! the mechanism: every source-relative arm of
//! `rules/layers.rs::effect_applies_to` opens with
//! `state.objects.get(&source_id)` and answers `false` unconditionally when
//! that is `None`, so `snapshot_affected_set` (CR 611.2c) locks an EMPTY set.
//!
//! This file exists because **existence is never sufficiency** (the
//! `kaito_shizuki` lesson, PB-DX43): a rule the engine applies but no client
//! can reach is not a repaired behaviour. Both probes below drive a real
//! `LocalGame` with a real human seat, submitting real `HumanChoice`s against
//! `StubProvider`'s own offer layer — the same surfaces the browser goes
//! through — and never poke `GameState` after `LocalGame::start`.
//!
//! * **CR 608.2h** — *"If the effect requires information from a specific
//!   object, including the source of the ability itself, … if it's no longer in
//!   that zone … the effect uses the object's last known information."*
//! * **CR 113.7a** — *"Once activated or triggered, an ability exists on the
//!   stack independently of its source."*
//! * **CR 611.2c** — the affected set is determined when the ability
//!   **resolves**, and never changes afterwards.
//!
//! | probe | subject | channel |
//! |---|---|---|
//! | `dx39_c1_jitte_bonus_survives_a_real_destruction_in_response` | Umezawa's Jitte (`Complete`, deck-legal), `EffectFilter::AttachedCreature` | **human seat — real `LocalGame` + `HumanChoice`**, two submitted intents |
//! | `dx39_c2_mardu_sacrifice_self_pumps_the_board_through_the_offer_layer` | Mardu Ascendancy, `EffectFilter::CreaturesYouControl` | `StubProvider::legal_actions` + `action_to_command_with_params` + `process_command` — see the c2/c3 header for why a `LocalGame` drive is **impossible** for this card |
//! | `dx39_c3_mardu_does_not_pump_an_opponents_creature` | Mardu Ascendancy | as c2; its "you control" half is the defect, its opponent half a CONTROL |
//!
//! **These probes carry no executed fail-before evidence and that is stated
//! rather than implied.** The engine repair landed while this file was being
//! written, so all three were first executed against the FIXED tree. The
//! executed fail-before evidence for the mechanism is in
//! `crates/engine/tests/primitives/pb_dx39_source_relative_lki.rs`, whose RED
//! `left`/`right` values were recorded at the merge base. What these three
//! probes are is **channel-reachability pins**: they prove the repaired
//! behaviour is reachable from the surfaces a client uses, which is a different
//! claim from "they discriminate the fix", and only the first is supported here.
//!
//! # Assert by RESOLUTION EFFECT, never by the offer
//!
//! Following `pb_dx48_ward_channel.rs`' and `pb_dx49_saga_blanking_channel.rs`'
//! standard. Nothing here asserts that an action was *offered*. The verdict in
//! every probe is the **layer-resolved P/T** of a creature on the battlefield
//! after the ability has resolved — the thing a player sees — never
//! `ContinuousEffect::affected_set`, which would be satisfied by a change that
//! never reaches a player.
//!
//! # How c1 destroys the Jitte, and why it is not a poke
//!
//! `Nature's Claim` (`{G}` instant, *"Destroy target artifact or enchantment"*,
//! `Complete` by derive) is cast **by the human seat, through the offer layer,
//! targeting the human's own Jitte**, while the Jitte's +2/+2 mode is already
//! on the stack. That is the exact scenario the 2005-02-01 Gatherer ruling
//! describes — *"If the Jitte leaves the battlefield after the '+2/+2' mode is
//! announced but before it resolves, the bonus is given to the creature that
//! was most recently equipped once the ability resolves"* — reached with no
//! `objects_mut()` call after the game starts.
//!
//! # Fixture choices, and what they cost
//!
//! * The subjects are the real corpus defs, built through `card_name_to_id` +
//!   `enrich_spec_from_def` against `all_cards()` — never a stand-in.
//!   `ObjectSpec::card()` creates **naked** objects (the standing
//!   `memory/gotchas-infra.md` gotcha; PB-DX47 found a probe passing against
//!   one that measured nothing), so each probe **asserts the ability is present
//!   on the object** before driving it.
//! * **The Jitte's charge counter is placed directly in the fixture rather than
//!   earned in combat**, and that is stated rather than glossed: driving
//!   *"whenever equipped creature deals combat damage, put two charge counters
//!   on Umezawa's Jitte"* would need a declared attack, an unblocked hit and a
//!   combat-damage step before the probe's own subject could begin. **So
//!   `TriggerCondition::WhenEquippedCreatureDealsCombatDamage` is NOT exercised
//!   by this file at all** — a later reader must not read c1's green (once it
//!   is green) as coverage of the counter trigger.
//! * Likewise the Jitte's `attached_to` is set in the fixture rather than by
//!   paying Equip {2}, so **CR 702.6a's equip ability is not exercised here
//!   either** (`pb_dx26_equip_surface.rs` covers it). The attachment is the
//!   probe's independent variable, not its subject.
//! * The fixture is built with `GameStateBuilder`, **not**
//!   `mtg_simulator::setup::build_initial_state` (PB-DX47's standard): the
//!   production pregame path deals and shuffles decks and cannot put a *named*
//!   Equipment on the battlefield already attached to a *named* creature with a
//!   *chosen* counter count. What that costs: these probes do not exercise deck
//!   validation, the mulligan or the opening-hand path. Everything from
//!   `start_game` onward — priority, the offer layer, the command path,
//!   resolution and the SBAs — is the real engine.
//! * The step is set to `Step::Untap` explicitly, because `build()` otherwise
//!   defaults to `Step::PreCombatMain` and a drive that starts where it means
//!   to stop proves nothing (`pb_dx49_saga_blanking_channel.rs`' recorded
//!   first-draft defect).
//!
//! # c4/c5 — the SECOND deck-legal `Complete` subject, and what driving it MEASURED
//!
//! The table above is honest about a hole. Of the three probes shipped first, **only c1
//! is a real `LocalGame` drive**, and it exercises `EffectFilter::AttachedCreature`. The
//! `EffectFilter::CreaturesYouControl` half — `OOS-DX5-7`'s own subject — is reachable
//! only through c2/c3's offer-layer-plus-production-mapping path, because
//! `Mardu Ascendancy` is `Completeness::partial` and Architecture Invariant 9 /
//! `validate_deck` refuse to start a game containing it (verbatim refusal in the c2/c3
//! header below).
//!
//! **There is a deck-legal `Complete` member of that half.** `binding_the_old_gods.rs`
//! declares no `Completeness` line at all, so `#[default] Completeness::Complete` applies;
//! its **chapter III** is `LayerModification::AddKeyword(KeywordAbility::Deathtouch)` over
//! `EffectFilter::CreaturesYouControl`. `pb_dx49_saga_blanking_channel.rs`' own module doc
//! says of it, verbatim:
//!
//! > *"**Chapter III** … is unobservable after the fact. Its grant is an
//! > `EffectFilter::CreaturesYouControl` continuous effect, and that filter resolves its
//! > controller through `state.objects.get(&source_id)` at layer-application time …
//! > Chapter III is the final chapter, so **CR 714.4 sacrifices the Saga in the same
//! > window it resolves in, the source id is gone, and the filter matches nothing.** A
//! > draft of this file used it and failed on exactly that — a fact about `EffectFilter`
//! > and a departed source, not about CR 714."*
//!
//! # THE OBSERVATION REPRODUCES AND THE STATED CAUSE IS WRONG, WHICH IS THIS SECTION'S POINT
//!
//! c4/c5 were written to be the missing full-`LocalGame` demonstration of PB-DX39's
//! `CreaturesYouControl` half on a deck-legal card. **They are not, because chapter III's
//! grant is still unreachable at HEAD — and the blocker is NOT the source-relative filter
//! PB-DX39 repaired.** A note in a test file is a claim like any other, and this one was
//! right about the symptom and wrong about the mechanism.
//!
//! Measured on the drive below, at HEAD, with the engine untouched:
//!
//! 1. CR 714.3b's precombat turn-based action puts the crossing lore counter on
//!    (`CounterAdded { object_id: ObjectId(1), counter: Lore, count: 3 }`).
//! 2. CR 117.5 performs state-based actions **before** putting triggered abilities on the
//!    stack, so at the moment CR 714.4's check runs the chapter III trigger is in
//!    `pending_triggers` and **not on the stack** — `rules/sba.rs`' *"don't sacrifice
//!    while a chapter ability from this Saga is on the stack"* guard does not see it, and
//!    the Saga is sacrificed. So the CR 608.2h condition **is** reached, with no response
//!    from anybody and no `objects_mut()` poke after `LocalGame::start`.
//! 3. **PB-DX39's capture half works on this card.** With the chapter ability on the stack
//!    and the Saga's `ObjectId` already retired from `state.objects`,
//!    `GameState::lki_objects()` carries the snapshot:
//!    `Some((PlayerId(1), "Binding the Old Gods"))`. That is exactly what
//!    `rules::layers::source_view_at_resolution` would read. **c5 asserts this**, and it
//!    is the one genuinely positive thing this pair proves.
//! 4. The chapter ability then resolves — `AbilityTriggered` and `AbilityResolved` are
//!    both in the journal — and **`state.continuous_effects()` is EMPTY**. No
//!    `ApplyContinuousEffect` ever executed, so no `snapshot_affected_set` call was ever
//!    made and `EffectFilter::CreaturesYouControl`'s arm was **never consulted at all**.
//!
//! The blocker is one link upstream of PB-DX39, in `rules/resolution.rs`' triggered-ability
//! arm. A `PendingTriggerKind::Normal` trigger whose ability is an
//! `AbilityDefinition::SagaChapter` is looked up through the **card-registry fallback**,
//! and that fallback opens with `let obj = state.fizzle_object(source_object);`.
//! `fizzle_object` is a documented **live-only** `self.objects` lookup that returns
//! **no** last known information, so a departed source yields `None` and the arm falls
//! through to `(None, None)` — the ability resolves with no effect whatsoever. That is
//! CR 113.7a-wrong for this path (*"Once activated or triggered, an ability exists on the
//! stack independently of its source. Destruction or removal of the source after that time
//! won't affect the ability."*), and it is wrong for **every** registry-fallback triggered
//! ability whose source has left, not only for Sagas. It is **out of PB-DX39's scope** —
//! nothing this batch touched is on that path — and no line of the engine was changed to
//! discover it.
//!
//! # So c4/c5 are a wrong-way-round DEFECT PIN plus one positive assertion
//!
//! | probe | what it asserts | direction |
//! |---|---|---|
//! | `dx39_c4_binding_chapter_iii_grant_is_still_unreachable_and_the_blocker_is_downstream` | the CR 608.2h condition is genuinely reached, and **no creature gains deathtouch** | **WRONG WAY ROUND** — invert when the resolution-site lookup is fixed; the inversion is spelled out on the test |
//! | `dx39_c5_binding_chapter_iii_source_lki_is_captured_while_the_chapter_is_on_the_stack` | the Saga's last known information **is** in `lki_objects` with the right controller at exactly the moment `source_view_at_resolution` would want it | RIGHT WAY ROUND |
//!
//! `pb_dx49_saga_blanking_channel.rs` declined to write any probe here, on the ground that
//! *"a probe asserting today's broken behaviour would have to be inverted by whoever fixes
//! it"*. That reasoning is respected and the pin is written anyway for one reason: the
//! symptom now has **two** candidate causes on record, one of them refuted, and a pin that
//! reddens the day either changes is what stops a third batch re-deriving this from
//! scratch. c4 says in its own failure message what to do when it goes red.
//!
//! # Fail-before, executed
//!
//! Two reverts were executed against this file, each restored byte-exactly afterwards
//! (`diff` identical, md5 match) and each recorded verbatim in the task record.
//!
//! * **R1 — the LKI READ.** Deleting the `.or_else(|| state.lki_object_snapshot(source_id))`
//!   branch from `rules::layers::source_view_at_resolution`, the single smallest edit that
//!   undoes PB-DX39's read: **c1, c2 and c3 RED** (`left: Some(1), right: Some(3)` and
//!   `left: Some(2), right: Some(5)`), **c4 and c5 GREEN**. That is not a gap in c4/c5; it
//!   is this section's finding stated as a measurement — on this card the read is never
//!   reached, so a revert of the read cannot move it.
//! * **R2 — the LKI CAPTURE.** Neutralising the `is_source_of_a_pending_ability` disjunct
//!   in `GameState::capture_lki_snapshot` (the call is kept, so the edit is a behaviour
//!   change and not a `dead_code` build failure — deleting it outright does not compile):
//!   **c5 RED** on its own message (*"observed the window 1 time(s) and found no
//!   snapshot"*) and **c1 RED** too, since Umezawa's Jitte carries none of the four
//!   LKI-relevant keywords and is served by the same disjunct. **c2 and c3 stay GREEN**,
//!   which is informative rather than a gap: Mardu Ascendancy departs as an *activation
//!   cost*, so it is captured by `capture_source_lki_for_pending_ability` — the other
//!   clause — and R2 cannot reach it.
//!
//! Two things c4/c5 deliberately do **not** claim:
//!
//! * They do not exercise CR 714.3a's enters-the-battlefield lore counter — the Saga is
//!   placed on the battlefield by the fixture carrying a seeded lore count, the same
//!   `GameStateBuilder` cost `pb_dx49_saga_blanking_channel.rs` states for itself.
//! * They take **no position** on whether the engine's CR 714.4 guard is correct. Whether
//!   a Saga whose final chapter has triggered but has not yet reached the stack should be
//!   sacrificed is a CR 714.4 question; c4/c5 only require that the source is gone when
//!   the chapter resolves, and assert that precondition explicitly rather than assuming
//!   it, so a later change to that guard reddens the precondition instead of quietly
//!   making the probes measure nothing.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, calculate_characteristics, card_name_to_id, enrich_spec_from_def, process_command,
    AbilityDefinition, CardDefinition, Command, CounterType, EffectFilter, GameState,
    GameStateBuilder, KeywordAbility, LayerModification, ObjectId, ObjectSpec, PlayerId, Step,
    Target, ZoneId,
};
use mtg_simulator::{
    action_to_command_with_params, build_registry, ActionParams, AdvanceOutcome, Bot, HeuristicBot,
    HumanChoice, LegalAction, LegalActionProvider, LocalGame, LocalGameError, LocalGameLimits,
    StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 39_39_39;
const JITTE: &str = "Umezawa's Jitte";
const MARDU: &str = "Mardu Ascendancy";
const CLAIM: &str = "Nature's Claim";
const FOREST: &str = "Forest";

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn enriched(owner: PlayerId, name: &str, zone: ZoneId) -> ObjectSpec {
    let defs = card_defs_by_name();
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        &defs,
    )
}

fn find_on_battlefield(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
}

fn expect_on_battlefield(state: &GameState, name: &str) -> ObjectId {
    find_on_battlefield(state, name)
        .unwrap_or_else(|| panic!("'{name}' must be on the battlefield"))
}

fn power(state: &GameState, id: ObjectId) -> Option<i32> {
    calculate_characteristics(state, id).and_then(|c| c.power)
}

fn toughness(state: &GameState, id: ObjectId) -> Option<i32> {
    calculate_characteristics(state, id).and_then(|c| c.toughness)
}

/// The index a `Command::ActivateAbility` must name for the (single) MODAL
/// activated ability on `id`. Located by shape rather than hardcoded —
/// `OOS-DX26-3` records that authoring another activated ability into a def
/// silently renumbers it.
fn modal_ability_index(state: &GameState, id: ObjectId) -> usize {
    calculate_characteristics(state, id)
        .unwrap_or_else(|| panic!("{id:?} must have layer-resolved characteristics"))
        .activated_abilities
        .iter()
        .position(|a| a.modes.is_some())
        .unwrap_or_else(|| panic!("{id:?} declares no modal activated ability"))
}

/// The index of the (single) activated ability on `id` whose cost sacrifices
/// the source itself. Same reason as [`modal_ability_index`].
fn sacrifice_self_ability_index(state: &GameState, id: ObjectId) -> usize {
    calculate_characteristics(state, id)
        .unwrap_or_else(|| panic!("{id:?} must have layer-resolved characteristics"))
        .activated_abilities
        .iter()
        .position(|a| a.cost.sacrifice_self)
        .unwrap_or_else(|| panic!("{id:?} declares no sacrifice-self activated ability"))
}

// ── The scripted human driver ───────────────────────────────────────────────────

/// One thing the human seat means to do, in order. Anything the drive is offered
/// that does not match the head of the queue is answered with `PassPriority` —
/// the same thing a browser client with nothing more to click would do.
#[derive(Clone, Debug)]
enum Intent {
    /// Activate `source`'s ability at `ability_index`, choosing `modes`.
    Activate {
        source: ObjectId,
        ability_index: usize,
        modes: Vec<usize>,
    },
    /// Cast the card named `card_name` from hand, targeting `target`.
    ///
    /// `require_stack` is a **response-ordering floor**: the number of objects
    /// that must already be on the stack at the moment this cast is submitted.
    /// Without it "cast in response" would be an assumption about how the drive
    /// happened to schedule, and a drive that let the earlier ability resolve
    /// first would still reach the same end state by a route that never touches
    /// the defect.
    CastAt {
        card_name: String,
        target: ObjectId,
        require_stack: usize,
    },
}

struct Driven {
    state: GameState,
    /// How many intents the drive actually managed to submit. Asserted by every
    /// caller: a drive that quietly passed its way to the end measures nothing.
    submitted: usize,
}

/// Drive `p1` as a real human seat through `intents`, then keep passing until the
/// stack and the trigger queue are both empty, and stop there.
///
/// Panics rather than returning early: a drive that gave up asserts nothing.
fn drive_human(state: GameState, human: PlayerId, mut intents: Vec<Intent>) -> Driven {
    assert!(
        !(state.stack_objects().is_empty()
            && state.turn().step == Step::PreCombatMain
            && intents.is_empty()),
        "the drive must have somewhere to go"
    );
    let total = intents.len();
    assert!(total > 0, "a drive with no intents measures nothing");

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for seat in [p(1), p(2)] {
        if seat != human {
            bots.insert(seat, Box::new(HeuristicBot::new(SEED, format!("{seat:?}"))));
        }
    }
    let human_seats: BTreeSet<PlayerId> = [human].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        SEED,
        StubProvider,
        bots,
        human_seats,
        LocalGameLimits {
            max_turns: 2,
            max_commands: 800,
            max_consecutive_passes: 500,
            record_journal: true,
        },
        true,
    )
    .unwrap_or_else(|e: LocalGameError| panic!("PB-DX39 channel game must start: {e:?}"));

    for _ in 0..300 {
        let settled = game.state().stack_objects().is_empty()
            && game.state().pending_triggers().is_empty()
            && game.state().blocking_decision().is_none();
        if intents.is_empty() && settled {
            return Driven {
                state: game.state().clone(),
                submitted: total,
            };
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                let mut chosen: Option<(usize, ActionParams)> = None;
                if let Some(next) = intents.first() {
                    match next {
                        Intent::Activate {
                            source,
                            ability_index,
                            modes,
                        } => {
                            if let Some(i) = d.actions.iter().position(|a| {
                                matches!(a, LegalAction::ActivateAbility { source: s, ability_index: ai, .. }
                                    if s == source && ai == ability_index)
                            }) {
                                chosen = Some((
                                    i,
                                    ActionParams {
                                        modes_chosen: modes.clone(),
                                        ..ActionParams::default()
                                    },
                                ));
                            }
                        }
                        Intent::CastAt {
                            card_name,
                            target,
                            require_stack,
                        } => {
                            let hand_id = game
                                .state()
                                .objects()
                                .values()
                                .find(|o| {
                                    o.characteristics.name == *card_name
                                        && o.zone == ZoneId::Hand(d.player)
                                })
                                .map(|o| o.id);
                            if let Some(hand_id) = hand_id {
                                if let Some(i) = d.actions.iter().position(|a| {
                                    matches!(a, LegalAction::CastSpell { card, .. } if *card == hand_id)
                                }) {
                                    assert_eq!(
                                        game.state().stack_objects().len(),
                                        *require_stack,
                                        "response-ordering floor: '{card_name}' must be cast \
                                         with exactly {require_stack} object(s) already on the \
                                         stack, or it is not being cast IN RESPONSE to anything"
                                    );
                                    chosen = Some((
                                        i,
                                        ActionParams {
                                            targets: vec![Target::Object(*target)],
                                            // The browser's own affordance: the client
                                            // asks the engine to solve the mana payment
                                            // rather than scripting taps.
                                            auto_tap: true,
                                            ..ActionParams::default()
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
                let (idx, params) = match chosen {
                    Some(c) => {
                        intents.remove(0);
                        c
                    }
                    None => {
                        // Answer a real pending question if one is owed, else pass.
                        let i = d
                            .actions
                            .iter()
                            .position(|a| matches!(a, LegalAction::AnswerEffectChoice { .. }))
                            .or_else(|| {
                                d.actions
                                    .iter()
                                    .position(|a| matches!(a, LegalAction::PassPriority))
                            })
                            .unwrap_or_else(|| {
                                panic!(
                                    "the human seat was offered neither the queued intent nor \
                                     PassPriority: {:?}",
                                    d.actions
                                )
                            });
                        (i, ActionParams::default())
                    }
                };
                let dbg_action = format!("{:?}", d.actions[idx]);
                let dbg_params = format!("{params:?}");
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: idx,
                        params,
                    },
                )
                .unwrap_or_else(|e| {
                    panic!(
                        "SR-38: the human's own offered action must be accepted: {e:?}\n                           action = {dbg_action}\n  params = {dbg_params}\n  offered = {:?}",
                        d.actions
                    )
                });
            }
            other => panic!(
                "the drive must reach a settled priority window with every intent submitted, \
                 not {other:?} (intents left: {}) — a drive that halts or ends the game \
                 measures nothing",
                intents.len()
            ),
        }
    }
    panic!(
        "the drive never settled with every intent submitted ({} left)",
        intents.len()
    );
}

// ── c1 — Umezawa's Jitte, destroyed in response, through the human channel ──────

#[test]
/// **c1** — CR 608.2h / CR 113.7a / CR 611.2c × Gatherer ruling 2005-02-01,
/// end to end through a real `LocalGame` human seat.
///
/// The human activates the Jitte's *"Equipped creature gets +2/+2 until end of
/// turn"* mode while it is attached to `DX39 Bear`, then — with that ability
/// still on the stack — casts `Nature's Claim` at their own Jitte and destroys
/// it. When the mode finally resolves, the ruling says the bonus goes to *"the
/// creature that was most recently equipped"*.
///
/// Pre-fix, `EffectFilter::AttachedCreature`'s arm answered `false` for every
/// candidate once the Jitte's `ObjectId` was gone from `state.objects`, so the
/// locked set was empty and the Bear stayed 1/1. This probe was first executed
/// against the repaired tree (see the module doc); its
/// `require_stack: 1` floor is what proves it genuinely reaches the
/// destroyed-in-response condition rather than a benign ordering.
fn dx39_c1_jitte_bonus_survives_a_real_destruction_in_response() {
    let (p1, p2) = (p(1), p(2));

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(build_registry())
        .object(enriched(p1, JITTE, ZoneId::Battlefield).with_counter(CounterType::Charge, 1))
        .object(ObjectSpec::creature(p1, "DX39 Bear", 1, 1))
        .object(ObjectSpec::creature(p1, "DX39 Other Bear", 1, 1))
        .object(enriched(p1, FOREST, ZoneId::Battlefield))
        .object(enriched(p1, CLAIM, ZoneId::Hand(p1)))
        .active_player(p1)
        .at_step(Step::Untap);
    for player in [p1, p2] {
        for i in 0..20 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("DX39 Library Filler {i}"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    let mut state = builder.build().expect("PB-DX39 c1 fixture must build");

    let jitte = expect_on_battlefield(&state, JITTE);
    let bear = expect_on_battlefield(&state, "DX39 Bear");
    let other = expect_on_battlefield(&state, "DX39 Other Bear");

    // CR 301.5: the Equipment is attached. Set here, in the FIXTURE, before the
    // game starts — nothing below touches `GameState` outside `LocalGame`.
    state
        .objects_mut()
        .get_mut(&jitte)
        .expect("live fixture object")
        .attached_to = Some(bear);

    // ── non-vacuity floors, on the fixture ────────────────────────────────
    let modal = modal_ability_index(&state, jitte);
    assert!(
        calculate_characteristics(&state, jitte)
            .map(|c| !c.activated_abilities.is_empty())
            .unwrap_or(false),
        "floor: the Jitte object is ENRICHED and really carries its activated abilities — \
         ObjectSpec::card() alone would make this probe measure nothing (PB-DX47)"
    );
    assert_eq!(
        state.objects().get(&jitte).and_then(|o| o.attached_to),
        Some(bear),
        "floor: the Jitte really is attached to DX39 Bear"
    );
    assert_eq!(power(&state, bear), Some(1), "floor: the Bear starts 1/1");

    let driven = drive_human(
        state,
        p1,
        vec![
            Intent::Activate {
                source: jitte,
                ability_index: modal,
                modes: vec![0],
            },
            Intent::CastAt {
                card_name: CLAIM.to_string(),
                target: jitte,
                // CR 601.2: the Jitte's +2/+2 mode is ALREADY on the stack. This
                // is what makes the Claim a response and the destruction happen
                // before the mode resolves (LIFO, CR 608.1).
                require_stack: 1,
            },
        ],
    );
    assert_eq!(
        driven.submitted, 2,
        "floor: both human intents were submitted"
    );
    let state = driven.state;

    // ── non-vacuity floors, on the drive ──────────────────────────────────
    assert!(
        find_on_battlefield(&state, JITTE).is_none(),
        "floor: Nature's Claim really destroyed the Jitte — its source left the battlefield"
    );
    assert!(
        state.objects().get(&jitte).is_none(),
        "floor: CR 400.7 — the ObjectId the ability's stack entry named is retired, which is \
         the exact condition every source-relative EffectFilter arm answers `false` on"
    );
    assert!(
        state.stack_objects().is_empty(),
        "floor: both the spell and the ability resolved rather than being stranded"
    );

    assert_eq!(
        power(&state, bear),
        Some(3),
        "CR 608.2h + Jitte ruling 2005-02-01: through the real human channel, the +2/+2 \
         goes to the creature that was most recently equipped"
    );
    assert_eq!(
        toughness(&state, bear),
        Some(3),
        "CR 608.2h: ModifyBoth(2) is +2/+2, not +2/+0"
    );
    assert_eq!(
        power(&state, other),
        Some(1),
        "CR 301.5: the never-equipped creature gets nothing — a fix that widens \
         AttachedCreature to 'anything' on an empty lookup is caught here"
    );
}

// ── c2/c3 — Mardu Ascendancy through the offer layer + the production mapping ───
//
// **A `LocalGame` / `HumanChoice` drive of Mardu Ascendancy is IMPOSSIBLE at HEAD,
// and that is a finding rather than a shortcut.** `LocalGame::start` calls
// `start_game`, which enforces Architecture Invariant 9 (SR-2) and refuses any
// game containing a non-`Complete` card. `mardu_ascendancy.rs` is
// `Completeness::partial` — it declares
// `TriggerCondition::WheneverCreatureYouControlAttacks` with no nontoken
// predicate — so the card is **not deck-legal** and putting it on a battlefield
// that `start_game` will inspect fails with
// `GameStateError::IncompleteCardsInGame` before a single command is issued.
// Verbatim, from this file's own first draft:
//
// ```text
// PB-DX39 channel game must start: Engine(IncompleteCardsInGame { count: 1,
//   first_name: "Mardu Ascendancy", first_kind: "partial", ... })
// ```
//
// So the `OOS-DX5-7` residual's blast radius through the **production** channel
// is currently **zero on this card**: no validated game can contain it. The
// defect is real in the engine (see the primitive probes) and is reachable on
// deck-legal cards through other routes; Mardu is simply not one of them today,
// and a reader must not take c2/c3's green as evidence that it is.
//
// What c2/c3 therefore drive is everything `HumanChoice` itself routes through,
// minus `start_game`: **`StubProvider::legal_actions`** (the real offer layer,
// the same call `LocalGame::advance` makes to build a human decision),
// **`action_to_command_with_params`** (the production mapping `LocalGame::submit`
// calls on the human's `ActionParams` — not a hand-built `Command`), and
// **`mtg_engine::process_command`** (the real engine entry point). The one thing
// they do not exercise is the pregame/deck-validation path, which is precisely
// the thing that refuses this card. This is `pb_dx50_mutate_legality_channel.rs`'
// idiom, used for the same reason.

/// Build the Mardu fixture and drive its sacrifice-self ability through the offer
/// layer and the production param mapping, then resolve it by passing priority.
///
/// Returns the settled state. Panics rather than returning early — a drive that
/// gave up asserts nothing.
fn drive_mardu_through_the_offer_layer(
    state: GameState,
    p1: PlayerId,
    p2: PlayerId,
    mardu: ObjectId,
    sac: usize,
) -> GameState {
    let mut state = state;
    state.turn_mut().priority_holder = Some(p1);

    let offers = StubProvider.legal_actions(&state, p1);
    let offer = offers
        .iter()
        .find(|a| {
            matches!(a, LegalAction::ActivateAbility { source, ability_index, .. }
                if *source == mardu && *ability_index == sac)
        })
        .unwrap_or_else(|| {
            panic!(
                "SR-38: the real offer layer must offer the sacrifice-self ability it will \
                 then accept; got {offers:?}"
            )
        });

    let command = action_to_command_with_params(&state, p1, offer, &ActionParams::default())
        .expect("the production mapping must build a Command for the offer layer's own action");
    assert!(
        matches!(command, Command::ActivateAbility { .. }),
        "the mapping produced {command:?}, not an ActivateAbility"
    );
    let (next, _ev) = process_command(state, command)
        .expect("SR-38: an action the offer layer emitted must be accepted by the engine");
    state = next;

    assert!(
        state.objects().get(&mardu).is_none(),
        "floor: CR 601.2h — the sacrifice-self cost was really paid at activation, so the \
         source ObjectId is retired BEFORE the ability reaches the stack"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "floor: CR 113.7a — the ability is on the stack even though its source is gone"
    );

    for _ in 0..20 {
        if state.stack_objects().is_empty() {
            return state;
        }
        let acting = state
            .turn()
            .priority_holder
            .unwrap_or(state.turn().active_player);
        let (next, _ev) = process_command(state, Command::PassPriority { player: acting })
            .expect("passing priority must be accepted");
        state = next;
        let _ = p2;
    }
    panic!("the ability never resolved within 20 priority passes");
}

/// The shared Mardu board. `extra` is the creature set, which is the only thing
/// c2 and c3 differ in.
fn mardu_fixture(extra: Vec<ObjectSpec>) -> GameState {
    let (p1, p2) = (p(1), p(2));
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(build_registry())
        .object(enriched(p1, MARDU, ZoneId::Battlefield))
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for spec in extra {
        builder = builder.object(spec);
    }
    builder.build().expect("PB-DX39 Mardu fixture must build")
}

#[test]
/// **c2** — CR 608.2h / CR 113.7a / CR 611.2c. Mardu Ascendancy:
/// *"Sacrifice this enchantment: Creatures you control get +0/+3 until end of
/// turn."* `Cost::SacrificeSelf` is paid at activation, before the ability is
/// pushed to the stack, so the source is gone at EVERY resolution of this
/// ability — no response from anybody is needed to reach the defect.
///
/// Pre-fix, `EffectFilter::CreaturesYouControl` read
/// `state.objects.get(&source_id).map(|src| src.controller)`, got `None`, and
/// its `source_controller.is_some()` conjunct made every candidate `false`.
///
/// The verdict is the creatures' **layer-resolved toughness**, never the locked
/// `affected_set`.
fn dx39_c2_mardu_sacrifice_self_pumps_the_board_through_the_offer_layer() {
    let (p1, p2) = (p(1), p(2));
    let state = mardu_fixture(vec![
        ObjectSpec::creature(p1, "DX39 Bear One", 2, 2),
        ObjectSpec::creature(p1, "DX39 Bear Two", 2, 2),
    ]);

    let mardu = expect_on_battlefield(&state, MARDU);
    let one = expect_on_battlefield(&state, "DX39 Bear One");
    let two = expect_on_battlefield(&state, "DX39 Bear Two");
    let sac = sacrifice_self_ability_index(&state, mardu);

    // ── non-vacuity floors ────────────────────────────────────────────────
    assert!(
        calculate_characteristics(&state, mardu)
            .map(|c| c.activated_abilities[sac].cost.sacrifice_self)
            .unwrap_or(false),
        "floor: the enchantment is ENRICHED and really carries its sacrifice-self ability — \
         ObjectSpec::card() alone would make this probe measure nothing (PB-DX47)"
    );
    assert_eq!(
        toughness(&state, one),
        Some(2),
        "floor: Bear One starts 2/2"
    );
    assert_eq!(
        toughness(&state, two),
        Some(2),
        "floor: Bear Two starts 2/2"
    );

    let state = drive_mardu_through_the_offer_layer(state, p1, p2, mardu, sac);

    assert_eq!(
        toughness(&state, one),
        Some(5),
        "CR 608.2h: 'Creatures you control' is resolved from the source's last known \
         information, so Bear One gets +0/+3"
    );
    assert_eq!(
        toughness(&state, two),
        Some(5),
        "CR 608.2h: ... and so does Bear Two"
    );
    assert_eq!(
        power(&state, one),
        Some(2),
        "ModifyToughness(3) is +0/+3, not +3/+3"
    );
}

#[test]
/// **c3** — CR 608.2h + CR 109.5 (*"you"* is the source's controller), with both
/// directions asserted on ONE board through the same channel.
///
/// Its "you control" half was RED pre-fix; its opponent half is a **CONTROL**
/// that was green pre-fix for the wrong reason (the locked set was empty, so
/// nobody was pumped) and must stay green for the right one. A fix that answers
/// `true` whenever the source lookup fails is caught here.
fn dx39_c3_mardu_does_not_pump_an_opponents_creature() {
    let (p1, p2) = (p(1), p(2));
    let state = mardu_fixture(vec![
        ObjectSpec::creature(p1, "DX39 My Bear", 2, 2),
        ObjectSpec::creature(p2, "DX39 Their Bear", 2, 2),
    ]);

    let mardu = expect_on_battlefield(&state, MARDU);
    let mine = expect_on_battlefield(&state, "DX39 My Bear");
    let theirs = expect_on_battlefield(&state, "DX39 Their Bear");
    let sac = sacrifice_self_ability_index(&state, mardu);

    assert_ne!(
        state.objects().get(&mine).map(|o| o.controller),
        state.objects().get(&theirs).map(|o| o.controller),
        "floor: the two creatures really are controlled by different players"
    );
    assert_eq!(toughness(&state, mine), Some(2), "floor: 2/2 to start");
    assert_eq!(toughness(&state, theirs), Some(2), "floor: 2/2 to start");

    let state = drive_mardu_through_the_offer_layer(state, p1, p2, mardu, sac);

    assert_eq!(
        toughness(&state, mine),
        Some(5),
        "CR 608.2h + CR 109.5: 'you' is the source's controller, read from last known \
         information — p1's creature is in the set"
    );
    assert_eq!(
        toughness(&state, theirs),
        Some(2),
        "CR 109.5: p2's creature is NOT — a fix that answers `true` whenever the source \
         lookup fails is caught here"
    );
}

// ── c4/c5 — Binding the Old Gods, a DECK-LEGAL `Complete` CreaturesYouControl subject ───
//
// Everything below drives a real `LocalGame` with a real human seat. Nothing pokes
// `GameState` after `LocalGame::start`, and no `Intent` is submitted at all: the human
// only ever passes priority, because CR 714.3b's precombat lore counter is a **turn-based
// action** and needs no player action to reach the CR 608.2h condition. Read the module
// doc's c4/c5 section first — c4 is pinned WRONG WAY ROUND on purpose.

const SAGA: &str = "Binding the Old Gods";
const MY_BEAR_A: &str = "DX39 Saga Bear A";
const MY_BEAR_B: &str = "DX39 Saga Bear B";
const THEIR_BEAR: &str = "DX39 Saga Enemy Bear";

/// CR 714.2d's *"greatest value among chapter abilities it has"*, read off the real card
/// definition rather than transcribed (`pb_dx49_saga_blanking_channel.rs`' rule). If
/// `binding_the_old_gods.rs` ever gains a chapter IV, this reddens the seeding floor below
/// instead of leaving the fixture quietly one chapter short.
fn saga_final_chapter() -> u32 {
    all_cards()
        .into_iter()
        .find(|d| d.name == SAGA)
        .unwrap_or_else(|| panic!("{SAGA} must exist in all_cards()"))
        .abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::SagaChapter { chapter, .. } => Some(*chapter),
            _ => None,
        })
        .max()
        .unwrap_or_else(|| panic!("{SAGA} must declare at least one SagaChapter"))
}

/// `true` iff the real definition's FINAL chapter is the
/// `AddKeyword(Deathtouch)` × `CreaturesYouControl` continuous effect these probes are
/// written against — derived, never transcribed, so a card change reddens rather than
/// silently repointing the subject.
fn final_chapter_is_the_deathtouch_grant() -> bool {
    let last = saga_final_chapter();
    all_cards()
        .into_iter()
        .find(|d| d.name == SAGA)
        .into_iter()
        .flat_map(|d| d.abilities.into_iter())
        .any(|a| match a {
            AbilityDefinition::SagaChapter {
                chapter,
                effect: mtg_engine::Effect::ApplyContinuousEffect { effect_def },
                ..
            } => {
                chapter == last
                    && matches!(
                        effect_def.modification,
                        LayerModification::AddKeyword(KeywordAbility::Deathtouch)
                    )
                    && matches!(effect_def.filter, EffectFilter::CreaturesYouControl)
            }
            _ => false,
        })
}

fn has_deathtouch(state: &GameState, id: ObjectId) -> bool {
    calculate_characteristics(state, id)
        .map(|c| c.keywords.contains(&KeywordAbility::Deathtouch))
        .unwrap_or(false)
}

/// The shared c4/c5 board: `p1` controls the Saga seeded **one lore counter below its
/// final chapter**, plus two creatures; `p2` controls one creature. Each seat gets a
/// library so no draw step ends the game by CR 704.5b.
///
/// The seeding is `final - 1` so CR 714.3b's precombat lore counter — a turn-based action
/// needing no player action — crosses the final chapter on turn 1. No lower chapter is
/// crossed, so no CR 701.19a search question is opened and the drive's only job is to pass
/// priority.
///
/// `Step::Untap` explicitly, for the same reason the c1 fixture does it: `build()`
/// otherwise defaults to `Step::PreCombatMain`, which is where these drives STOP, and a
/// drive that starts where it means to stop proves nothing.
fn binding_fixture() -> GameState {
    let (p1, p2) = (p(1), p(2));
    let seed_lore = saga_final_chapter()
        .checked_sub(1)
        .expect("the Saga's final chapter must be >= 1");

    let mut saga = enriched(p1, SAGA, ZoneId::Battlefield);
    if seed_lore > 0 {
        saga = saga.with_counter(CounterType::Lore, seed_lore);
    }

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(build_registry())
        .object(saga)
        .object(ObjectSpec::creature(p1, MY_BEAR_A, 2, 2))
        .object(ObjectSpec::creature(p1, MY_BEAR_B, 2, 2))
        .object(ObjectSpec::creature(p2, THEIR_BEAR, 2, 2))
        .active_player(p1)
        .at_step(Step::Untap);
    for player in [p1, p2] {
        for i in 0..20 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("DX39 Saga Filler {i}"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }
    builder.build().expect("PB-DX39 Binding fixture must build")
}

/// Turn 1, `Step::PreCombatMain`, `active` active, and the stack, the pending-trigger
/// queue and the blocking-decision slot all empty — i.e. CR 714.3b's turn-based action has
/// run and everything it queued has resolved.
fn settled_at_saga_precombat_main(state: &GameState, active: PlayerId) -> bool {
    state.turn().turn_number == 1
        && state.turn().step == Step::PreCombatMain
        && state.turn().active_player == active
        && state.stack_objects().is_empty()
        && state.pending_triggers().is_empty()
        && state.blocking_decision().is_none()
}

/// What the c4/c5 drive produced.
struct SagaDriven {
    /// The state at the stopping point.
    state: GameState,
    /// The Saga's battlefield `ObjectId` (retired by the time the drive stops).
    saga: ObjectId,
    mine_a: ObjectId,
    mine_b: ObjectId,
    theirs: ObjectId,
    /// The **mid-flight** observation, taken at the first moment the chapter ability is on
    /// the stack **and** the Saga's `ObjectId` is already gone from `state.objects` —
    /// exactly the instant `rules::layers::source_view_at_resolution` would consult
    /// `lki_object_snapshot`. `Some((controller, name))` if the snapshot is there.
    ///
    /// Recorded during the drive because `GameState::maybe_clear_lki_objects` empties the
    /// store once the stack and the pending-trigger queue are both empty, which is the
    /// stopping point — so this is unobservable from the settled state and asserting it
    /// there would silently measure nothing.
    lki_while_chapter_on_stack: Option<(PlayerId, String)>,
    /// How many times that instant was actually observed. A `0` here means the drive never
    /// reached the condition and every assertion about it would be vacuous.
    lki_observations: usize,
}

/// Drive a REAL `LocalGame` with `human` in a real seat until the stopping point above,
/// recording the mid-flight LKI observation on the way.
///
/// The human answers a real pending question if one is owed and otherwise passes — the
/// same thing a browser client with nothing more to click would do. Panics rather than
/// returning early: a drive that gave up asserts nothing.
fn drive_saga_to_precombat_main(
    state: GameState,
    human: PlayerId,
    active: PlayerId,
    saga: ObjectId,
) -> (GameState, Option<(PlayerId, String)>, usize) {
    assert!(
        !settled_at_saga_precombat_main(&state, active),
        "the fixture must start BEFORE the stopping point, or the drive proves nothing \
         (GameStateBuilder::build() defaults to Step::PreCombatMain)"
    );

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for seat in [p(1), p(2)] {
        if seat != human {
            bots.insert(seat, Box::new(HeuristicBot::new(SEED, format!("{seat:?}"))));
        }
    }
    let human_seats: BTreeSet<PlayerId> = [human].into_iter().collect();
    let (mut game, _start_events) = LocalGame::start(
        state,
        SEED,
        StubProvider,
        bots,
        human_seats,
        LocalGameLimits {
            max_turns: 2,
            max_commands: 800,
            max_consecutive_passes: 500,
            record_journal: true,
        },
        true,
    )
    .unwrap_or_else(|e: LocalGameError| panic!("PB-DX39 c4/c5 game must start: {e:?}"));

    let mut lki: Option<(PlayerId, String)> = None;
    let mut observations = 0usize;
    for _ in 0..300 {
        {
            // CR 113.7a: the ability is on the stack and its source is gone. This is the
            // one window in which the LKI store is both populated and readable.
            let st = game.state();
            if !st.stack_objects().is_empty() && !st.objects().contains_key(&saga) {
                observations += 1;
                if lki.is_none() {
                    lki = st
                        .lki_objects()
                        .get(&saga)
                        .map(|o| (o.controller, o.characteristics.name.clone()));
                }
            }
        }
        if settled_at_saga_precombat_main(game.state(), active) {
            return (game.state().clone(), lki, observations);
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                let idx = d
                    .actions
                    .iter()
                    .position(|a| matches!(a, LegalAction::AnswerEffectChoice { .. }))
                    .or_else(|| {
                        d.actions
                            .iter()
                            .position(|a| matches!(a, LegalAction::PassPriority))
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "the human seat was offered neither an effect-choice answer nor \
                             PassPriority: {:?}",
                            d.actions
                        )
                    });
                game.submit(
                    d.seq,
                    HumanChoice {
                        action_index: idx,
                        params: ActionParams::default(),
                    },
                )
                .expect("SR-38: the human's own offered action must be accepted");
            }
            other => panic!(
                "the drive must reach a settled turn-1 precombat main, not {other:?} — a drive \
                 that halts or ends the game measures nothing"
            ),
        }
    }
    panic!("turn-1 precombat main never settled within 300 human decisions");
}

/// Build, floor, drive, and re-assert the precondition that makes this a CR 608.2h
/// scenario at all — the Saga's `ObjectId` is RETIRED while its final chapter is on the
/// stack.
fn drive_binding_final_chapter() -> SagaDriven {
    let p1 = p(1);
    let state = binding_fixture();

    let saga = expect_on_battlefield(&state, SAGA);
    let mine_a = expect_on_battlefield(&state, MY_BEAR_A);
    let mine_b = expect_on_battlefield(&state, MY_BEAR_B);
    let theirs = expect_on_battlefield(&state, THEIR_BEAR);

    // ── non-vacuity floors, on the fixture ────────────────────────────────
    assert!(
        final_chapter_is_the_deathtouch_grant(),
        "floor: the subject is derived from `binding_the_old_gods.rs` itself — its FINAL \
         chapter must still be AddKeyword(Deathtouch) over CreaturesYouControl, or these \
         probes are measuring a different card than they describe"
    );
    assert!(
        calculate_characteristics(&state, saga)
            .map(|c| c.subtypes.iter().any(|s| s.0 == "Saga"))
            .unwrap_or(false),
        "floor: the Saga object is ENRICHED and really carries its Saga subtype — \
         ObjectSpec::card() alone would make this probe measure nothing (PB-DX47)"
    );
    assert_eq!(
        state
            .objects()
            .get(&saga)
            .and_then(|o| o.counters.get(&CounterType::Lore).copied())
            .unwrap_or(0),
        saga_final_chapter() - 1,
        "floor: the Saga is seeded exactly one lore counter below its final chapter, so \
         CR 714.3b's turn-based action is what crosses it"
    );
    for (id, name) in [
        (mine_a, MY_BEAR_A),
        (mine_b, MY_BEAR_B),
        (theirs, THEIR_BEAR),
    ] {
        assert!(
            !has_deathtouch(&state, id),
            "floor: '{name}' does NOT have deathtouch before the drive — otherwise the \
             assertions below would be true for a reason that has nothing to do with \
             CR 608.2h"
        );
    }
    assert_ne!(
        state.objects().get(&mine_a).map(|o| o.controller),
        state.objects().get(&theirs).map(|o| o.controller),
        "floor: the two creatures really are controlled by different players"
    );

    let (state, lki_while_chapter_on_stack, lki_observations) =
        drive_saga_to_precombat_main(state, p1, p1, saga);

    // ── the precondition that makes this a CR 608.2h scenario at all ──────
    //
    // CR 117.5 checks state-based actions BEFORE putting triggered abilities on the stack,
    // so CR 714.4's *"isn't the source of a chapter ability ... on the stack"* guard does
    // not see a chapter trigger that is still in `pending_triggers`. The Saga is therefore
    // sacrificed while its final chapter is queued, and the chapter reaches the stack with
    // its source's ObjectId already retired (CR 400.7).
    //
    // Asserted rather than assumed. If a later batch changes that guard, this reddens and
    // says so, instead of the assertions below quietly passing from a LIVE source read.
    assert!(
        find_on_battlefield(&state, SAGA).is_none(),
        "precondition: CR 714.4 really sacrificed the Saga — its source left the battlefield"
    );
    assert!(
        state.objects().get(&saga).is_none(),
        "precondition: CR 400.7 — the ObjectId the chapter ability's stack entry named is \
         retired, which is the exact condition CR 608.2h / CR 113.7a exist for"
    );
    assert!(
        state.stack_objects().is_empty() && state.pending_triggers().is_empty(),
        "precondition: the chapter ability resolved rather than being stranded"
    );
    assert!(
        lki_observations > 0,
        "precondition: the drive really passed through a window in which the chapter \
         ability was on the stack with its source already gone — without it, c5's \
         assertion would be vacuous"
    );

    SagaDriven {
        state,
        saga,
        mine_a,
        mine_b,
        theirs,
        lki_while_chapter_on_stack,
        lki_observations,
    }
}

#[test]
/// **c4 — PINNED WRONG WAY ROUND.** CR 714.4 × CR 608.2h × CR 113.7a on a **deck-legal
/// `Complete`** card, driven end to end through a real `LocalGame` human seat.
///
/// `Binding the Old Gods`' final chapter reads *"Creatures you control gain deathtouch
/// until end of turn"*. The drive reaches the CR 608.2h condition exactly as the module
/// doc describes, and **nobody gains deathtouch** — which is what this test asserts.
///
/// **The blocker is NOT the `EffectFilter::CreaturesYouControl` arm PB-DX39 repaired**,
/// and `pb_dx49_saga_blanking_channel.rs`' module doc says it is. The filter is never
/// consulted, because no `ApplyContinuousEffect` ever executes: `rules/resolution.rs`'
/// card-registry fallback for a `PendingTriggerKind::Normal` trigger opens with
/// `state.fizzle_object(source_object)`, a documented **live-only** lookup that returns no
/// last known information, so a departed source yields `None` and the whole chapter
/// ability resolves as a no-op. That is CR 113.7a-wrong (*"an ability exists on the stack
/// independently of its source"*) for every registry-fallback triggered ability whose
/// source has left, not only for Sagas. Out of PB-DX39's scope; nothing this batch touched
/// is on that path.
///
/// # WHEN THIS TEST GOES RED, INVERT IT — DO NOT DELETE IT
///
/// Flip the three `assert!(!has_deathtouch(..))` below to `assert!(has_deathtouch(..))`
/// for `mine_a` and `mine_b`, and **keep `theirs` negated**: CR 109.5 makes *"you"* the
/// source's controller, so the opponent's creature must still not gain the keyword. The
/// opposing pair is what catches a "fix" that answers `true` whenever the source lookup
/// fails. c5 already proves the last known information needed to get that right is present
/// and correct.
fn dx39_c4_binding_chapter_iii_grant_is_still_unreachable_and_the_blocker_is_downstream() {
    let d = drive_binding_final_chapter();

    assert!(
        d.state
            .continuous_effects()
            .iter()
            .all(|e| e.source != Some(d.saga)),
        "the mechanism, asserted rather than narrated: the chapter ability resolved and \
         registered NO continuous effect at all, so `snapshot_affected_set` was never \
         called and `EffectFilter::CreaturesYouControl` was never consulted. If this line \
         goes red the resolution-site lookup has been fixed — read this test's doc and \
         INVERT the deathtouch assertions below rather than deleting them"
    );
    for (id, name) in [
        (d.mine_a, MY_BEAR_A),
        (d.mine_b, MY_BEAR_B),
        (d.theirs, THEIR_BEAR),
    ] {
        assert!(
            !has_deathtouch(&d.state, id),
            "DEFECT PIN (wrong way round): '{name}' does not gain deathtouch, because \
             `rules/resolution.rs`' registry fallback reads the chapter ability's effect \
             through the live-only `fizzle_object` and the Saga's ObjectId is retired. \
             When this goes red, invert it for the controller's creatures and KEEP it \
             negated for the opponent's — see this test's doc"
        );
    }
    assert!(
        d.state
            .objects()
            .get(&d.mine_a)
            .map(|o| o.zone == ZoneId::Battlefield)
            .unwrap_or(false),
        "floor: the creatures are still on the battlefield, so the assertions above are \
         about live permanents rather than about `calculate_characteristics` answering for \
         something that left"
    );
}

#[test]
/// **c5** — the positive half, and the one thing this pair genuinely proves about
/// PB-DX39 on a deck-legal `Complete` card: **the source's last known information is
/// captured and available at exactly the moment `source_view_at_resolution` wants it.**
///
/// CR 608.2h: *"if it's no longer in that zone … the effect uses the object's last known
/// information."* CR 113.7a: *"Once activated or triggered, an ability exists on the stack
/// independently of its source."* The Saga is sacrificed by CR 714.4 while its final
/// chapter is still in `pending_triggers`; the chapter then sits on the stack with its
/// source's `ObjectId` retired, and `GameState::lki_objects()` carries the snapshot with
/// the **controller** the `CreaturesYouControl` filter would read.
///
/// This is observed **mid-drive**, not from the settled state:
/// `GameState::maybe_clear_lki_objects` empties the store once the stack and the
/// pending-trigger queue are both empty, which is the stopping point — so an assertion
/// taken at the end would measure nothing and pass for the wrong reason.
///
/// **Executed fail-before**: removing the `is_source_of_a_pending_ability` disjunct from
/// `GameState::capture_lki_snapshot` reddens this probe (the snapshot is never taken, so
/// the observation is `None`). Deleting the `.or_else(|| state.lki_object_snapshot(..))`
/// **read** in `rules::layers::source_view_at_resolution` does **not** redden it, and that
/// is the same finding as c4's stated one way round: on this card the read is never
/// reached.
fn dx39_c5_binding_chapter_iii_source_lki_is_captured_while_the_chapter_is_on_the_stack() {
    let d = drive_binding_final_chapter();

    assert!(
        d.lki_observations > 0,
        "floor: the drive really passed through the CR 113.7a window — ability on the \
         stack, source gone"
    );
    let (controller, name) = d.lki_while_chapter_on_stack.clone().unwrap_or_else(|| {
        panic!(
            "CR 608.2h: with the final chapter on the stack and the Saga's ObjectId \
             retired, `lki_objects` must carry its last known information — observed the \
             window {} time(s) and found no snapshot",
            d.lki_observations
        )
    });
    assert_eq!(
        controller,
        p(1),
        "CR 109.5 + CR 608.2h: the snapshot carries the SOURCE's controller, which is what \
         `EffectFilter::CreaturesYouControl` resolves 'you' to"
    );
    assert_eq!(
        name, SAGA,
        "floor: the snapshot is the Saga's own, not some other departed permanent's"
    );
}
