//! PB-DX49 (`OOS-RR4-1`) — CR 714's blanked-Saga exemption, through the REAL channels.
//!
//! The engine-side probes live in
//! `crates/engine/tests/primitives/pb_dx49_blanked_saga_sites.rs` and prove that the
//! five behavioural sites (`sba.rs`'s CR 714.4 filter and its chapter-on-stack guard,
//! `turn_actions.rs`' CR 714.3b precombat lore counter, and `replacement.rs`' CR 714.3a
//! counter and CR 714.2b chapter triggers) each consult `rules::saga::saga_view` rather
//! than the printed def. This file exists because **existence is never sufficiency** (the
//! `kaito_shizuki` lesson, PB-DX43): a rule the engine applies but no client can reach is
//! not a repaired behaviour. Every probe here drives a real `LocalGame` plus
//! `StubProvider`'s offer layer and `HumanChoice`/`ActionParams` — the same surfaces the
//! browser and the bots go through.
//!
//! | probe | channel | what it discriminates |
//! |---|---|---|
//! | c1 | human seat — `LocalGame` + `HumanChoice` | CR 613.1f Layer-6 `RemoveAllAbilities`: 0 lore accrued, 0 chapters, survives at its printed final chapter |
//! | c2 | human seat, **identical fixture minus the blanker** | lore accrues, chapter II's *resolution effect* lands, and at its final chapter the Saga is sacrificed |
//! | c3 | bot path — `StubProvider::legal_actions` + `Bot::choose_action` + `process_command`, no human seat anywhere | the same four-way discrimination |
//! | c4 | human seat, CR 708.2a face-down (manifest-shaped) | no precombat counter, no chapter, never sacrificed |
//!
//! # Assert by RESOLUTION EFFECT and by exact COUNTS, never by the offer
//!
//! Following `pb_dx48_ward_channel.rs`' standard. Nothing below asserts that an action was
//! *offered*; the verdicts are:
//!
//! - the Saga's **lore counter value** and the **number of `GameEvent::CounterAdded` lore
//!   events naming it** (site 3),
//! - the **number of `GameEvent::AbilityTriggered` events whose `source_object_id` is the
//!   Saga** (site 5) — an exact count, never `>= 1`, because a double-dispatch bug passes a
//!   `>= 1` assertion (PB-DX48's rule),
//! - **battlefield vs graveyard membership** (site 1, the CR 714.4 sacrifice), and
//! - the chapter's **RESOLUTION EFFECT**: chapter II of `Binding the Old Gods` reads
//!   *"Search your library for a Forest card, put it onto the battlefield tapped, then
//!   shuffle"*, so the verdict is that the Forest which started in the library is now on
//!   the **battlefield** and **tapped** — a completed zone change plus a status the effect
//!   itself sets, not the existence of a stack object (which would be satisfied by a stack
//!   entry that resolved to nothing).
//!
//! # Stopping the drive at the right moment, and asserting that it stopped there
//!
//! PB-DX48's coordinator revert caught a probe whose assertion was true for two different
//! reasons because the drive ran through CR 514.2's cleanup. The same trap is live here: a
//! blanked Saga that survived turn 1 would take a lore counter and be sacrificed on a
//! later turn anyway, so a drive that runs to `Halted(MaxTurns)` measures a Saga's absence
//! that CR 714.4's exemption never had a chance to prevent.
//!
//! So every drive below stops at exactly one point — **turn 1, `Step::PreCombatMain`, the
//! Saga's controller active, with `stack_objects()` and `pending_triggers()` both empty**
//! — and every probe re-asserts that stopping state explicitly as a precondition.
//! `Step::PreCombatMain` is where CR 714.3b's turn-based action runs, so a drive that never
//! reaches it measures nothing at all; the helpers panic rather than return in that case.
//!
//! **That predicate alone is not enough, and the first draft of this file proved it.**
//! `GameStateBuilder::build()` defaults to `Step::PreCombatMain` (`builder.rs:311`), so the
//! bot-path drive — which, unlike `LocalGame::start`, does not reset the step — satisfied
//! "settled at turn-1 precombat main" **before issuing a single command**, and a probe
//! passed having driven nothing. That is precisely the shape PB-DX48 warns about: a drive
//! that stopped because it never started, wearing the same assertion as one that stopped
//! because resolution finished. Both drives therefore now (a) start from `Step::Untap` —
//! `LocalGame::start` calls `start_game`, and `drive_bots` calls it explicitly for the same
//! reason — and (b) assert **before the loop** that the state is not already at the
//! stopping point.
//!
//! # Two seeded lore values, because one seeding cannot exercise both halves
//!
//! Each probe runs its fixture at **two** starting lore counts, and each is paired with the
//! identical fixture differing in exactly one input (the blanking channel):
//!
//! - **leg A, `lore = 1`** — the accrual half. The precombat counter takes a live Saga
//!   1 → 2, which crosses **chapter II**, whose search is the resolution-effect verdict.
//!   Under the pre-PB-DX49 engine a blanked Saga took that counter and fired that chapter,
//!   so all three of leg A's numbers move under a revert. The Saga survives either way here
//!   (2 < 3), which is deliberate: it keeps leg A's subject on sites 3 and 5 and out of
//!   site 1.
//! - **leg B, `lore = 3`** — the CR 714.4 half proper. This is the only seeding at which
//!   site 1's *"lore counters ≥ final chapter number"* comparison is reached at all, so it
//!   is the only one that exercises the sacrifice exemption directly. At this seeding the
//!   trigger count does **not** discriminate (3 → 4 crosses no chapter either way), which
//!   is exactly why leg A exists.
//!
//! # Fixture choices, and what they cost
//!
//! - The Saga is **`Binding the Old Gods`**, the only deck-legal `Complete` Saga in the
//!   corpus, built through `card_name_to_id` + `enrich_spec_from_def` against the real
//!   `all_cards()` definition — never a stand-in. `ObjectSpec::card()` creates naked
//!   objects (the standing `memory/gotchas-infra.md` gotcha), so the enrichment is what
//!   gives it its Saga subtype and its three chapter abilities. Its printed final chapter
//!   is **derived from the def** by `printed_final_chapter()` rather than transcribed, so a
//!   change to the card reddens these probes instead of quietly vacating them.
//! - **Chapter II is the resolution-effect verdict, and the other two chapters were each
//!   tried first and each failed for a reason worth recording rather than hiding:**
//!   - **Chapter III** (*"Creatures you control gain deathtouch until end of turn"*) is
//!     unobservable after the fact. **↻ CORRECTED 2026-09-05 by PB-DX39 (`scutemob-230`):
//!     the SYMPTOM below reproduces and the stated CAUSE does not.** This paragraph said
//!     the grant is lost because `EffectFilter::CreaturesYouControl` resolves its
//!     controller through `state.objects.get(&source_id)` and CR 714.4 has already
//!     sacrificed the Saga. PB-DX39 repaired exactly that filter (it now resolves the
//!     source through last known information on the locked path, CR 608.2h / CR 113.7a)
//!     and **chapter III is still unobservable**, because the filter is never reached at
//!     all: measured on a real `LocalGame` drive, the Saga IS sacrificed and its LKI IS
//!     captured, and then `state.continuous_effects()` comes back **EMPTY** — no
//!     `ApplyContinuousEffect` ever ran. The real blocker is one link upstream, in
//!     `rules/resolution.rs`'s card-registry fallback for a `PendingTriggerKind::Normal`
//!     trigger, which opens with `state.fizzle_object(source_object)` — a documented
//!     LIVE-ONLY lookup that returns no LKI — so a departed source falls through to
//!     `(None, None)` and the whole ability resolves as a no-op. That is CR 113.7a-wrong
//!     (*"an ability exists on the stack independently of its source"*) for **every**
//!     registry-fallback triggered ability whose source has left, not only Sagas, and it
//!     is filed as **`OOS-DX39-3`**. Pinned wrong-way-round by
//!     `pb_dx39_source_relative_channel::dx39_c4_binding_chapter_iii_grant_is_still_unreachable_and_the_blocker_is_downstream`,
//!     whose own failure message says how to invert it. A draft of this file used chapter
//!     III and failed — a true observation with a wrong diagnosis attached, which is
//!     PB-DX27's rule (*a blocker note is a claim*) applied to a test-file note.
//!   - **Chapter I** (*"Destroy target nonland permanent an opponent controls"*) destroys
//!     nothing, even with exactly one legal target on the board and the trigger measurably
//!     on the stack. `fire_saga_chapter_triggers` queues a `PendingTriggerKind::Normal`
//!     trigger whose `ability_index` indexes `def.effective_abilities(..)`, but
//!     `flush_sorted`'s target-requirement lookup for a `Normal` trigger reads
//!     `obj.characteristics.triggered_abilities[ability_index]`
//!     (`rules/abilities.rs:8796-8830` and again at `:8918-8960`), and
//!     `AbilityDefinition::SagaChapter` is **never lowered into that vector** — it appears
//!     nowhere in `rules/abilities.rs`. So the requirement list comes back empty, no
//!     CR 603.3d announcement is made, and `EffectTarget::DeclaredTarget { index: 0 }`
//!     resolves at nothing. **Measured, not inferred**: with the fixture below plus an
//!     opponent 2/2, the drive recorded exactly one `AbilityTriggered` from the Saga and
//!     the 2/2 still on the battlefield. That is a live defect on a deck-legal `Complete`
//!     card and is reported for triage; it is **not** in PB-DX49's scope (nothing this
//!     batch touched is on that path), and no probe here is written against it, because a
//!     probe asserting today's broken behaviour would have to be inverted by whoever fixes
//!     it.
//!
//!     Chapter II is free of both problems: its search is a completed zone change with no
//!     dependency on its source surviving, and it declares no targets.
//! - Chapter II's search opens a genuine PB-DP9 CR 701.19a decision
//!   (`EffectChoiceQuestion::SearchLibrary`), so the human drive below **answers a real
//!   question through `HumanChoice`** rather than only passing priority, and the bot drive
//!   routes it through `state.blocking_decision()` exactly as `LocalGame::advance()` does.
//!   The library holds exactly one Forest, so the answer space is a single candidate and
//!   both channels reach the same place.
//! - The fixture is built with `GameStateBuilder`, **not** `mtg_simulator::setup::
//!   build_initial_state` (PB-DX47's standard). The production pregame path deals decks and
//!   shuffles; it cannot put a *named* Saga on the battlefield carrying a *chosen* number
//!   of lore counters, which is the entire independent variable here. **What that costs is
//!   stated rather than glossed:** these probes do not exercise deck validation, the
//!   mulligan, or the opening-hand path, and the Saga arrives on the battlefield without
//!   ever having entered it, so **CR 714.3a's ETB lore counter (site 4) is not exercised by
//!   this file at all** — `pb_dx49_blanked_saga_sites.rs`' `t6`/`t7` cover it. Everything
//!   from `start_game` onward — priority, turn-based actions, the trigger flush, the
//!   CR 608.2d choice channel, resolution and the SBAs — is the real engine on the real
//!   command path.
//! - Blanking is registered as an explicit `ContinuousEffect` via
//!   `GameStateBuilder::add_continuous_effect`, because `build()` registers **no** static
//!   continuous effects (`OOS-DX43-6`): a blanker permanent dropped straight onto the
//!   battlefield confers nothing, and the probe would then fail for a reason it does not
//!   describe.
//!
//! # What these probes discriminate, measured by executed reverts — and one thing they do not
//!
//! Both rows below were executed in a throwaway `git worktree` off this commit, so nothing
//! in the working tree was edited to obtain them.
//!
//! - **R-A — `rules::layers::abilities_are_blanked` short-circuited to `false`** (the
//!   pre-PB-DX49 world, where no site consults the layer axis). **c1, c3 and c4 all RED**,
//!   each on its first assertion: `CounterAdded` count `left: 1, right: 0`. c2 stays GREEN,
//!   and that is a CONTROL row rather than an undiscriminated one — c2 is the arm with no
//!   blanking, so a revert of the blanking predicate *must not* move it.
//! - **R-B — site 1 only** (`sba.rs`'s CR 714.4 filter re-reads the printed def's max
//!   chapter; sites 3 and 5 keep the fix). **c1, c3 and c4 RED on their leg B**, proving
//!   that leg's sacrifice-exemption assertion is load-bearing on its own and not merely a
//!   consequence of site 3 declining the counter. c2 stays GREEN — again a control, since
//!   an un-blanked Saga at its final chapter is sacrificed either way.
//!
//! **Honestly UNDISCRIMINATED here: site 5 in isolation.** Sites 3 and 5 are chained on
//! this path — `turn_actions.rs` only calls `fire_saga_chapter_triggers` for a Saga it has
//! just placed a counter on — so with site 3 fixed a blanked Saga never reaches site 5 at
//! all, and a site-5-only revert cannot redden anything in this file. The chapter-trigger
//! counts below are therefore *corroboration* of site 3 rather than an independent
//! measurement of site 5; `pb_dx49_blanked_saga_sites.rs`' `t5` is what exercises site 5
//! alone, by calling it directly. Stated here rather than only in `memory/`.
//!
//! # c4 and the manifest path — a scoping fact, stated so it is not mistaken for coverage
//!
//! c4's fixture is *manifest-shaped*: `status.face_down` plus
//! `face_down_as = Some(FaceDownKind::Manifest)`, the same conjunct `abilities_are_blanked`
//! and `saga_view` read, poked onto the object because `GameStateBuilder` has no face-down
//! setter. **`Effect::Manifest` and `Effect::Cloak` never call
//! `apply_self_etb_from_definition`**, so CR 714.3a's ETB lore counter never runs on the
//! manifest path in the first place (recorded as `OOS-DX49-2`). c4's live symptom is
//! therefore at the **precombat-counter (site 3), chapter-trigger (site 5) and sacrifice
//! (site 1)** sites, and that is what it measures. A later reader must not conclude from
//! c4's silence that the ETB half is untested by accident — it is not tested here, and on
//! the manifest path there is nothing there to test.

use std::collections::{BTreeSet, HashMap};

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, start_game,
    AbilityDefinition, CardDefinition, Command, ContinuousEffect, CounterType, EffectDuration,
    EffectFilter, EffectId, EffectLayer, FaceDownKind, GameEvent, GameState, GameStateBuilder,
    LayerModification, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};
use mtg_simulator::{
    build_registry, ActionParams, AdvanceOutcome, Bot, HeuristicBot, HumanChoice, LegalAction,
    LegalActionProvider, LocalGame, LocalGameError, LocalGameLimits, StubProvider,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SEED: u64 = 49_49_49;
const SAGA_NAME: &str = "Binding the Old Gods";
const FOREST_NAME: &str = "Forest";

/// The precombat counter takes the Saga 1 -> 2, which crosses chapter II.
const LORE_ONE_BELOW_CHAPTER_II: u32 = 1;
/// `final`: the seeding at which CR 714.4's own comparison is reached.
const LORE_AT_FINAL: u32 = 3;

/// Which blanking channel (if any) the fixture applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Blanking {
    /// No blanker at all — the control arm.
    None,
    /// CR 613.1f: a Layer-6 `RemoveAllAbilities`. The permanent keeps its subtypes, so it
    /// is **still a Saga** with zero chapter abilities.
    Layer6RemoveAllAbilities,
    /// CR 708.2a: face down as a manifest. *"No text, no name, no subtypes"* — so it is not
    /// a Saga at all.
    FaceDownManifest,
}

fn card_defs_by_name() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// CR 714.2d's *"greatest value among chapter abilities it has"*, read off the real card
/// definition rather than transcribed. If `binding_the_old_gods.rs` ever changes shape,
/// this reddens the probes instead of leaving them quietly measuring the wrong threshold.
fn printed_final_chapter() -> u32 {
    all_cards()
        .into_iter()
        .find(|d| d.name == SAGA_NAME)
        .unwrap_or_else(|| panic!("{SAGA_NAME} must exist in all_cards()"))
        .abilities
        .iter()
        .filter_map(|a| match a {
            AbilityDefinition::SagaChapter { chapter, .. } => Some(*chapter),
            _ => None,
        })
        .max()
        .unwrap_or_else(|| panic!("{SAGA_NAME} must declare at least one SagaChapter"))
}

/// A Humility-shaped Layer-6 `RemoveAllAbilities` over every permanent on the battlefield
/// — the same synthetic effect `pb_dx49_blanked_saga_sites.rs` uses, for the same
/// `OOS-DX43-6` reason.
fn blanket_remove_all_abilities() -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(9492),
        source: None,
        timestamp: 1,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::AllPermanents,
        modification: LayerModification::RemoveAllAbilities,
        is_cda: false,
        affected_set: None,
        condition: None,
    }
}

/// The fixture: `p1` controls the Saga (seeded with `start_lore` lore counters) and has
/// exactly one `Forest` in a 20-card library — chapter II's only search candidate, which is
/// what makes its CR 701.19a answer space a single card; `p2` is the opponent with a
/// library of its own so the draw step never triggers CR 704.5b.
///
/// Everything except `blanking` and `start_lore` is held constant across every call, which
/// is what makes the c1/c2, c3 and c4 comparisons attributable to one input.
///
/// The step is set to `Step::Untap` **explicitly**: `build()` otherwise defaults to
/// `Step::PreCombatMain`, which is the stopping point every drive below hunts for, and a
/// fixture that starts there lets a drive "succeed" having issued no commands (see the
/// module doc).
fn fixture(blanking: Blanking, start_lore: u32) -> GameState {
    let defs = card_defs_by_name();
    let (p1, p2) = (p(1), p(2));

    let mut saga = enrich_spec_from_def(
        ObjectSpec::card(p1, SAGA_NAME)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id(SAGA_NAME)),
        &defs,
    );
    if start_lore > 0 {
        saga = saga.with_counter(CounterType::Lore, start_lore);
    }

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(build_registry())
        .object(saga)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, FOREST_NAME)
                .in_zone(ZoneId::Library(p1))
                .with_card_id(card_name_to_id(FOREST_NAME)),
            &defs,
        ))
        .active_player(p1)
        .at_step(Step::Untap);

    // A library apiece: without one, the draw step makes a seat lose to CR 704.5b and the
    // drive would end for a reason that has nothing to do with CR 714. These fillers carry
    // no `card_id`, so they are not search candidates and cannot be cast.
    for player in [p1, p2] {
        for i in 0..20 {
            builder = builder.object(
                ObjectSpec::card(player, &format!("DX49 Library Filler {i}"))
                    .in_zone(ZoneId::Library(player)),
            );
        }
    }

    if blanking == Blanking::Layer6RemoveAllAbilities {
        builder = builder.add_continuous_effect(blanket_remove_all_abilities());
    }

    let mut state = builder.build().expect("PB-DX49 channel fixture must build");

    if blanking == Blanking::FaceDownManifest {
        let saga_id = saga_battlefield_id(&state).expect("the Saga fixture is on the battlefield");
        let obj = state
            .objects_mut()
            .get_mut(&saga_id)
            .expect("live fixture object");
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Manifest);
    }

    state
}

// ── Measurement helpers ─────────────────────────────────────────────────────────────

/// The Saga's `ObjectId` while it is on the battlefield, or `None` once it has been
/// sacrificed (CR 400.7 mints a new id in the graveyard, so this deliberately does not
/// follow it there).
fn saga_battlefield_id(state: &GameState) -> Option<ObjectId> {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == SAGA_NAME && o.zone == ZoneId::Battlefield)
        .map(|o| o.id)
}

fn count_in(state: &GameState, name: &str, zone: ZoneId) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.characteristics.name == name && o.zone == zone)
        .count()
}

fn on_battlefield(state: &GameState, name: &str) -> usize {
    count_in(state, name, ZoneId::Battlefield)
}

/// `true` only if there is exactly one object of that name on the battlefield AND it is
/// tapped — chapter II says *"put it onto the battlefield **tapped**"*, so the status is
/// part of the resolution effect, not decoration.
fn tapped_on_battlefield(state: &GameState, name: &str) -> bool {
    let matching: Vec<_> = state
        .objects()
        .values()
        .filter(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
        .collect();
    matching.len() == 1 && matching[0].status.tapped
}

fn lore(state: &GameState, id: ObjectId) -> u32 {
    state
        .objects()
        .get(&id)
        .and_then(|o| o.counters.get(&CounterType::Lore).copied())
        .unwrap_or(0)
}

/// CR 714.3b's turn-based action emits exactly one `CounterAdded` per lore counter it
/// places, so counting them measures site 3 directly rather than inferring it from the
/// final counter value.
fn lore_counter_events(events: &[GameEvent], saga: ObjectId) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::CounterAdded {
                    object_id,
                    counter: CounterType::Lore,
                    ..
                } if *object_id == saga
            )
        })
        .count()
}

/// CR 714.2b chapter triggers that actually reached the stack, counted by source. An EXACT
/// count, never `>= 1`.
fn chapter_triggers(events: &[GameEvent], saga: ObjectId) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. } if *source_object_id == saga
            )
        })
        .count()
}

// ── The one stopping point, and the two drives that reach it ────────────────────────

/// Turn 1, `Step::PreCombatMain`, `active` is the active player, and both the stack and
/// `pending_triggers` are empty.
///
/// `Step::PreCombatMain` is where CR 714.3b's turn-based action runs; requiring the stack
/// and the trigger queue to be empty means everything that action queued has already
/// resolved. Pinning `turn_number == 1` keeps the measurement on the one precombat main the
/// fixture was built for — a later turn would bring a second lore counter and a second
/// chapter with it.
fn settled_at_precombat_main(state: &GameState, active: PlayerId) -> bool {
    state.turn().turn_number == 1
        && state.turn().step == Step::PreCombatMain
        && state.turn().active_player == active
        && state.stack_objects().is_empty()
        && state.pending_triggers().is_empty()
}

/// What a drive produced: the state at the stopping point, and every event emitted on the
/// way there.
struct Driven {
    state: GameState,
    events: Vec<GameEvent>,
}

/// Assert the drive has somewhere to go. See the module doc: the builder's default step
/// **is** the stopping point, so without this a drive can report success having applied no
/// commands at all — and it did, in this file's first draft.
fn assert_the_drive_has_not_already_arrived(state: &GameState, active: PlayerId) {
    assert!(
        !settled_at_precombat_main(state, active),
        "the fixture must start BEFORE the stopping point, or the drive proves nothing \
         (GameStateBuilder::build() defaults to Step::PreCombatMain)"
    );
}

/// The HUMAN channel: `p1` (the Saga's controller) occupies a real seat; `p2` is a bot
/// driven internally by `LocalGame::advance()`.
///
/// The human passes priority when that is all there is to do — the same thing a browser
/// client with nothing more to click would do — and **answers chapter II's CR 701.19a
/// search through `HumanChoice`** when the engine asks, which is the point: a decision the
/// engine raises but no client can answer is the same shape of nothing this file exists to
/// rule out.
///
/// Events are read from `game.journal()` rather than from `submit()`'s return values:
/// `advance()` drives every bot seat's own commands internally, and those events never pass
/// through a `submit()` return. Reading only the human's returns would make the trigger and
/// counter censuses below silently miss anything a bot seat's priority pass caused.
///
/// **Panics rather than returning early.** A drive that quietly gave up asserts nothing.
fn drive_human(state: GameState, human: PlayerId, active: PlayerId) -> Driven {
    assert_the_drive_has_not_already_arrived(&state, active);

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for seat in [p(1), p(2)] {
        if seat != human {
            bots.insert(seat, Box::new(HeuristicBot::new(SEED, format!("{seat:?}"))));
        }
    }
    let human_seats: BTreeSet<PlayerId> = [human].into_iter().collect();
    let (mut game, start_events) = LocalGame::start(
        state,
        SEED,
        StubProvider,
        bots,
        human_seats,
        LocalGameLimits {
            max_turns: 2,
            max_commands: 600,
            max_consecutive_passes: 400,
            record_journal: true,
        },
        true,
    )
    .unwrap_or_else(|e: LocalGameError| panic!("PB-DX49 channel game must start: {e:?}"));

    for _ in 0..200 {
        if settled_at_precombat_main(game.state(), active) {
            let mut events = start_events.clone();
            events.extend(game.journal().iter().flat_map(|r| r.events.iter().cloned()));
            return Driven {
                state: game.state().clone(),
                events,
            };
        }
        match game.advance() {
            AdvanceOutcome::AwaitingHuman(d) => {
                // Answer a real question if one is owed (chapter II's search); otherwise
                // pass. Deliberately in that order: a decision that offers BOTH would be
                // an ordinary priority window with an unanswered choice hanging off it,
                // and answering is what makes progress.
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
                            "the human seat was offered neither an effect-choice answer \
                             nor PassPriority: {:?}",
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
                .expect("the human's own offered action must be accepted (SR-38)");
            }
            other => panic!(
                "the drive must reach a settled turn-1 precombat main, not {other:?} (a \
                 drive that halts or ends the game measures nothing)"
            ),
        }
    }
    panic!("turn-1 precombat main never settled within 200 human decisions");
}

/// The BOT path: no human seat anywhere.
///
/// `LocalGame::advance()` is opaque for a fully bot-driven game — with nothing to stop on
/// it runs straight through to `Halted`/`GameOver`, i.e. past the point every assertion
/// here cares about. So this drives the SAME three real components `LocalGame`'s own
/// internal bot loop uses — `StubProvider::legal_actions` (the real offer layer),
/// `Bot::choose_action` (the real bot decision) and `mtg_engine::process_command` (the real
/// engine entry point) — with the seat resolved exactly as `advance()` resolves it:
/// blocking decision first (chapter II's search reaches this branch), then the priority
/// holder, then its "nobody holds priority between steps, so pass for the active player"
/// fallback. It is not a synthetic shortcut; it is the identical mechanism with a stopping
/// point. (`advance()` has a fourth branch, `pending_commander_zone_choices`; this fixture
/// registers no commander, so it is unreachable here and is asserted as such.)
///
/// `start_game` is called here for the same reason `LocalGame::start` calls it: it is what
/// resets the turn to `Step::Untap`. Omitting it was this file's first draft's defect (see
/// the module doc).
fn drive_bots(state: GameState, active: PlayerId) -> Driven {
    let (mut state, start_events) =
        start_game(state).unwrap_or_else(|e| panic!("PB-DX49 bot-path game must start: {e:?}"));
    assert_the_drive_has_not_already_arrived(&state, active);

    let mut bots: HashMap<PlayerId, Box<dyn Bot>> = HashMap::new();
    for seat in [p(1), p(2)] {
        bots.insert(seat, Box::new(HeuristicBot::new(SEED, format!("{seat:?}"))));
    }
    let mut events = start_events;
    for _ in 0..400 {
        if settled_at_precombat_main(&state, active) {
            return Driven { state, events };
        }
        assert!(
            state.pending_commander_zone_choices().is_empty(),
            "this fixture registers no commander, so `advance()`'s CR 903.9a branch must \
             stay unreachable -- if it fires, this loop is no longer LocalGame-shaped"
        );
        let acting = if let Some(decision) = state.blocking_decision() {
            decision.player()
        } else {
            state
                .turn()
                .priority_holder
                .unwrap_or(state.turn().active_player)
        };
        let legal = StubProvider.legal_actions(&state, acting);
        let command = if legal.is_empty() {
            Command::PassPriority { player: acting }
        } else {
            let bot = bots
                .get_mut(&acting)
                .unwrap_or_else(|| panic!("no bot registered for seat {acting:?}"));
            bot.choose_action(&state, acting, &legal)
        };
        let (next, ev) = process_command(state, command)
            .expect("a bot's own offered command must be accepted (SR-38)");
        state = next;
        events.extend(ev);
    }
    panic!("the bot-driven game never settled at turn-1 precombat main within 400 commands");
}

/// Shared precondition: the drive stopped where it was supposed to stop.
fn assert_stopped_at_the_right_moment(d: &Driven, active: PlayerId) {
    assert_eq!(d.state.turn().turn_number, 1, "precondition: still turn 1");
    assert_eq!(
        d.state.turn().step,
        Step::PreCombatMain,
        "precondition: the drive stopped at CR 714.3b's own step, so the turn-based action \
         under test has genuinely run"
    );
    assert_eq!(
        d.state.turn().active_player,
        active,
        "precondition: it is the Saga controller's precombat main"
    );
    assert!(
        d.state.stack_objects().is_empty() && d.state.pending_triggers().is_empty(),
        "precondition: everything the turn-based action queued has resolved -- a drive that \
         stopped with work outstanding would make the counts below meaningless"
    );
}

// ── c1: Layer-6 blanked, human channel ──────────────────────────────────────────────

/// **c1** — CR 613.1f × CR 714, end to end through a real `LocalGame` human seat.
///
/// Both legs put `Binding the Old Gods` on the battlefield under an **active**
/// `LayerModification::RemoveAllAbilities` continuous effect and drive turn 1 to its
/// settled precombat main.
///
/// - **leg A (`lore = 1`)**: CR 714.3b's *"each Saga they control **with one or more chapter
///   abilities**"* does not describe it, so **no** lore counter is placed; nothing crosses
///   chapter II, so **zero** chapter triggers are queued, and the Forest stays in the
///   library. Under the pre-PB-DX49 engine all three of those numbers move, which is what
///   makes this leg discriminate.
/// - **leg B (`lore = 3`)**: the CR 714.4 half proper — *"if the number of lore counters on
///   a Saga permanent **with one or more chapter abilities** is greater than or equal to
///   its final chapter number ... that Saga's controller sacrifices it."* The permanent sits
///   at exactly its printed final chapter and is **not** sacrificed. This is the only
///   seeding at which site 1's comparison is reached at all.
///
/// c2 is the identical pair with the blanker absent — one input, nothing else.
#[test]
fn c1_layer6_blanked_saga_accrues_no_lore_fires_no_chapter_and_survives_at_its_final_chapter() {
    let final_chapter = printed_final_chapter();
    assert_eq!(
        final_chapter, LORE_AT_FINAL,
        "CR 714.2d: {SAGA_NAME}'s printed final chapter, derived from its own def"
    );

    // ── leg A: one short of chapter II ──
    let state = fixture(
        Blanking::Layer6RemoveAllAbilities,
        LORE_ONE_BELOW_CHAPTER_II,
    );
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    let driven = drive_human(state, p(1), p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));

    assert_eq!(
        lore_counter_events(&driven.events, saga),
        0,
        "CR 714.3b: a permanent under a Layer-6 RemoveAllAbilities has no chapter \
         abilities, so the precombat turn-based action must place NO lore counter"
    );
    assert_eq!(
        lore(&driven.state, saga),
        LORE_ONE_BELOW_CHAPTER_II,
        "and its counter total is therefore unchanged from the seeding"
    );
    assert_eq!(
        chapter_triggers(&driven.events, saga),
        0,
        "CR 714.2b: the chapter ability must exist at the instant counters are put on; \
         blanked, none does, so no threshold crossing can trigger"
    );
    assert_eq!(
        on_battlefield(&driven.state, FOREST_NAME),
        0,
        "and chapter II's RESOLUTION EFFECT never happens: the Forest is still in the \
         library. This is the verdict -- an unfired trigger and a fired-but-inert one are \
         not the same claim."
    );
    assert_eq!(
        count_in(&driven.state, FOREST_NAME, ZoneId::Library(p(1))),
        1,
        "asserted from both directions: the Forest is where it started"
    );
    assert_eq!(
        on_battlefield(&driven.state, SAGA_NAME),
        1,
        "with no counter placed it never reaches its final chapter, so CR 714.4 never fires \
         either"
    );

    // ── leg B: at the final chapter, where CR 714.4's own comparison is reached ──
    let state = fixture(Blanking::Layer6RemoveAllAbilities, LORE_AT_FINAL);
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    let driven = drive_human(state, p(1), p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));

    // Survival FIRST, deliberately: a sacrificed Saga's ObjectId is dead (CR 400.7), so
    // `lore(..)` reads 0 for it, and putting the counter assertions ahead of this one made
    // a site-1 regression report itself as `left: 0, right: 3` instead of naming CR 714.4.
    // Measured, not guessed -- that is exactly what the R-B revert printed.
    assert_eq!(
        on_battlefield(&driven.state, SAGA_NAME),
        1,
        "CR 714.4 applies only to a Saga permanent WITH one or more chapter abilities; a \
         Layer-6 RemoveAllAbilities leaves none, so the sacrifice does not reach it even at \
         lore >= final chapter"
    );
    assert_eq!(
        count_in(&driven.state, SAGA_NAME, ZoneId::Graveyard(p(1))),
        0,
        "and it is not in the graveyard either -- 'still on the battlefield' is asserted \
         from both directions"
    );
    assert_eq!(
        lore_counter_events(&driven.events, saga),
        0,
        "CR 714.3b again: still no counter"
    );
    assert_eq!(
        lore(&driven.state, saga),
        LORE_AT_FINAL,
        "so it sits on exactly its printed final chapter for the whole drive"
    );
    assert!(
        lore(&driven.state, saga) >= final_chapter,
        "precondition for the assertion above: the permanent IS at or past its printed \
         final chapter, so CR 714.4's comparison is genuinely reached rather than skipped"
    );
    assert_eq!(
        chapter_triggers(&driven.events, saga),
        0,
        "CR 714.2b: no chapters retained, none can trigger"
    );
}

// ── c2: the same Saga un-blanked ────────────────────────────────────────────────────

/// **c2** — the identical fixture and the identical drive as c1, differing in **exactly one
/// input**: the `RemoveAllAbilities` continuous effect is absent.
///
/// - **leg A (`lore = 1`)** is c1 leg A's twin, and it carries the resolution-effect
///   verdict: the precombat turn-based action places one lore counter (1 → 2), that
///   crossing triggers **chapter II** exactly once, the human answers its CR 701.19a search
///   through `HumanChoice`, and the Forest ends up on the **battlefield, tapped**.
/// - **leg B (`lore = 3`)** is c1 leg B's twin: at its final chapter with its chapter
///   abilities intact, it is sacrificed.
#[test]
fn c2_the_same_saga_unblanked_accrues_lore_resolves_chapter_ii_and_is_sacrificed_at_its_final_chapter(
) {
    // ── leg A: accrual, the chapter, and its resolution effect ──
    let state = fixture(Blanking::None, LORE_ONE_BELOW_CHAPTER_II);
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    assert_eq!(
        count_in(&state, FOREST_NAME, ZoneId::Library(p(1))),
        1,
        "precondition: the Forest starts in the library, so the assertion below is a \
         measured MOVE and not a fixture that was already in the answer"
    );
    let driven = drive_human(state, p(1), p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));

    assert_eq!(
        lore_counter_events(&driven.events, saga),
        1,
        "CR 714.3b: un-blanked, the precombat turn-based action places exactly one lore \
         counter"
    );
    assert_eq!(
        lore(&driven.state, saga),
        LORE_ONE_BELOW_CHAPTER_II + 1,
        "and the counter total moves by exactly that much"
    );
    assert_eq!(
        chapter_triggers(&driven.events, saga),
        1,
        "CR 714.2b: the 1 -> 2 crossing triggers chapter II exactly once. An exact count, \
         not `>= 1`: a double dispatch would pass that."
    );
    assert_eq!(
        on_battlefield(&driven.state, FOREST_NAME),
        1,
        "chapter II's RESOLUTION EFFECT -- 'Search your library for a Forest card, put it \
         onto the battlefield tapped' -- observed as a completed zone change. This is the \
         verdict; the trigger count above is what makes it attributable to chapter II."
    );
    assert!(
        tapped_on_battlefield(&driven.state, FOREST_NAME),
        "and TAPPED, which is part of the printed effect -- a Forest that arrived untapped \
         would be a different effect passing the same zone check"
    );
    assert_eq!(
        count_in(&driven.state, FOREST_NAME, ZoneId::Library(p(1))),
        0,
        "and it is no longer in the library"
    );
    assert_eq!(
        on_battlefield(&driven.state, SAGA_NAME),
        1,
        "at two lore counters it is short of its final chapter, so it survives -- which is \
         deliberate: it keeps this leg's subject on sites 3 and 5 and out of site 1"
    );

    // ── leg B: the exactly-one-input twin of c1 leg B ──
    let state = fixture(Blanking::None, LORE_AT_FINAL);
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    let driven = drive_human(state, p(1), p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));

    assert_eq!(
        on_battlefield(&driven.state, SAGA_NAME),
        0,
        "non-vacuity for c1 leg B: the identical fixture WITHOUT the blanker, seeded at the \
         identical lore count, IS sacrificed"
    );
    assert_eq!(
        count_in(&driven.state, SAGA_NAME, ZoneId::Graveyard(p(1))),
        1,
        "into its owner's graveyard"
    );
    assert_eq!(
        chapter_triggers(&driven.events, saga),
        0,
        "and it fires no chapter on the way out: it was already at its final chapter, so \
         nothing crossed. This is why leg A exists -- at this seeding the trigger count does \
         not discriminate between blanked and un-blanked."
    );
}

// ── c3: the bot path ────────────────────────────────────────────────────────────────

/// **c3** — the same four-way discrimination with **no human seat anywhere**, so the
/// exemption is not an artefact of the human channel, and chapter II's CR 701.19a search is
/// answered by `StubProvider` + `HeuristicBot` rather than by a person. `StubProvider` needs
/// no change for any of it, and that is asserted rather than assumed: `drive_bots`'
/// `.expect("a bot's own offered command must be accepted (SR-38)")` is what would catch an
/// offer layer that produced an action the engine then refused.
#[test]
fn c3_bot_path_reaches_the_identical_discrimination_with_no_human_seat() {
    // Blanked, one short of chapter II.
    let state = fixture(
        Blanking::Layer6RemoveAllAbilities,
        LORE_ONE_BELOW_CHAPTER_II,
    );
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    let driven = drive_bots(state, p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));
    assert_eq!(
        lore_counter_events(&driven.events, saga),
        0,
        "CR 714.3b on the bot path: no counter"
    );
    assert_eq!(lore(&driven.state, saga), LORE_ONE_BELOW_CHAPTER_II);
    assert_eq!(
        chapter_triggers(&driven.events, saga),
        0,
        "CR 714.2b on the bot path: no chapter"
    );
    assert_eq!(
        on_battlefield(&driven.state, FOREST_NAME),
        0,
        "and no search: the Forest is still in the library"
    );
    assert_eq!(on_battlefield(&driven.state, SAGA_NAME), 1);

    // Un-blanked, one short of chapter II -- the one-input twin.
    let state = fixture(Blanking::None, LORE_ONE_BELOW_CHAPTER_II);
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    let driven = drive_bots(state, p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));
    assert_eq!(
        lore_counter_events(&driven.events, saga),
        1,
        "non-vacuity: the bot path does place the counter when nothing blanks the Saga"
    );
    assert_eq!(
        chapter_triggers(&driven.events, saga),
        1,
        "and the 1 -> 2 crossing triggers chapter II exactly once"
    );
    assert_eq!(
        on_battlefield(&driven.state, FOREST_NAME),
        1,
        "chapter II's resolution effect, answered and resolved with no human seat anywhere"
    );
    assert!(tapped_on_battlefield(&driven.state, FOREST_NAME));
    assert_eq!(on_battlefield(&driven.state, SAGA_NAME), 1);

    // Blanked, at the final chapter -- CR 714.4's exemption on the bot path.
    let state = fixture(Blanking::Layer6RemoveAllAbilities, LORE_AT_FINAL);
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    let driven = drive_bots(state, p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));
    assert_eq!(
        on_battlefield(&driven.state, SAGA_NAME),
        1,
        "CR 714.4 exemption, bot path"
    );
    assert_eq!(lore(&driven.state, saga), LORE_AT_FINAL);
    assert!(lore(&driven.state, saga) >= printed_final_chapter());

    // Un-blanked, at the final chapter -- the one-input twin.
    let state = fixture(Blanking::None, LORE_AT_FINAL);
    let driven = drive_bots(state, p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));
    assert_eq!(
        on_battlefield(&driven.state, SAGA_NAME),
        0,
        "non-vacuity: un-blanked at its final chapter, the bot path sacrifices it"
    );
    assert_eq!(
        count_in(&driven.state, SAGA_NAME, ZoneId::Graveyard(p(1))),
        1
    );
}

// ── c4: the CR 708.2a face-down channel ─────────────────────────────────────────────

/// **c4** — CR 708.2a: *"no text, no name, **no subtypes**, and no mana cost"*. A manifested
/// permanent is not a Saga at all, so CR 714.3b's *"each Saga they control"* does not
/// describe it, CR 714.2b has no ability to trigger, and CR 714.4's *"a Saga permanent with
/// one or more chapter abilities"* never reaches it.
///
/// **Scope, stated so it is not mistaken for coverage** (see the module doc):
/// `Effect::Manifest` and `Effect::Cloak` never call `apply_self_etb_from_definition`, so
/// CR 714.3a's **ETB** lore counter never runs on the manifest path in the first place —
/// there is nothing for this probe to measure there, and its silence about the ETB half is a
/// scoping fact rather than a gap. `pb_dx49_blanked_saga_sites.rs`' `t7` covers site 4 at the
/// engine level. What c4 measures is the manifest channel's live symptom: sites 3, 5 and 1.
///
/// The face-down state is poked onto the object (`status.face_down` +
/// `face_down_as = Some(FaceDownKind::Manifest)`, the exact conjunct the engine reads)
/// because `GameStateBuilder` has no face-down setter — so the *creation* of the face-down
/// permanent is not driven through a channel here, while everything the probe asserts about
/// is: the turn-based action, the trigger flush, resolution and the SBAs all run on the real
/// command path.
#[test]
fn c4_manifested_face_down_saga_takes_no_precombat_counter_fires_no_chapter_and_is_never_sacrificed(
) {
    // ── leg A: one short of chapter II ──
    let state = fixture(Blanking::FaceDownManifest, LORE_ONE_BELOW_CHAPTER_II);
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    assert!(
        state
            .objects()
            .get(&saga)
            .map(|o| o.status.face_down && o.face_down_as == Some(FaceDownKind::Manifest))
            .unwrap_or(false),
        "precondition: the fixture really is face down as a manifest"
    );
    let driven = drive_human(state, p(1), p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));

    assert_eq!(
        lore_counter_events(&driven.events, saga),
        0,
        "CR 708.2a x CR 714.3b: no subtypes means not a Saga, so no precombat lore counter \
         is placed"
    );
    assert_eq!(
        lore(&driven.state, saga),
        LORE_ONE_BELOW_CHAPTER_II,
        "its counter total is unchanged from the seeding"
    );
    assert_eq!(
        chapter_triggers(&driven.events, saga),
        0,
        "CR 708.2a x CR 714.2b: no text means no chapter ability to trigger"
    );
    assert_eq!(
        on_battlefield(&driven.state, FOREST_NAME),
        0,
        "and a face-down 2/2 searches no library on its controller's behalf"
    );
    assert_eq!(
        on_battlefield(&driven.state, SAGA_NAME),
        1,
        "and it is not sacrificed"
    );

    // Non-vacuity for leg A: the same fixture FACE UP -- one input -- does all three.
    let state = fixture(Blanking::None, LORE_ONE_BELOW_CHAPTER_II);
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    let driven = drive_human(state, p(1), p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));
    assert_eq!(lore_counter_events(&driven.events, saga), 1);
    assert_eq!(chapter_triggers(&driven.events, saga), 1);
    assert_eq!(
        on_battlefield(&driven.state, FOREST_NAME),
        1,
        "non-vacuity: face up, the identical fixture accrues, fires chapter II and puts the \
         Forest onto the battlefield"
    );

    // ── leg B: at the final chapter, where CR 714.4's comparison is reached ──
    let state = fixture(Blanking::FaceDownManifest, LORE_AT_FINAL);
    let saga = saga_battlefield_id(&state).expect("fixture places the Saga on the battlefield");
    let driven = drive_human(state, p(1), p(1));
    assert_stopped_at_the_right_moment(&driven, p(1));

    assert_eq!(
        on_battlefield(&driven.state, SAGA_NAME),
        1,
        "CR 708.2a x CR 714.4: a manifested permanent is never sacrificed as a Saga, no \
         matter how many lore counters it carries"
    );
    assert_eq!(
        count_in(&driven.state, SAGA_NAME, ZoneId::Graveyard(p(1))),
        0
    );
    assert_eq!(lore(&driven.state, saga), LORE_AT_FINAL);
    assert!(
        lore(&driven.state, saga) >= printed_final_chapter(),
        "precondition: CR 714.4's comparison is genuinely reached"
    );
}
