//! PB-DX54 (`OOS-DX25c-6`): behavioural probes for the resolving stack entry's
//! CR 608.2n departure point, `rules::resolution::depart_resolving_stack_entry`.
//!
//! **Hard constraint, inherited from PB-DX25c/PB-DX25b's durable lesson: every
//! targeting probe below (t1-t6) reaches the code through a real
//! `Command::CastSpell` and real `Command::PassPriority` resolution. No
//! hand-built `StackObject` anywhere in this file.** t7 places its Saga
//! directly on the battlefield via `GameStateBuilder` (never "cast", since it
//! is fixture setup unrelated to the mechanism under test -- exactly the same
//! convention `pb_dx25c_retarget_legality.rs`'s own T1 uses for its battlefield
//! creatures), but the chapter ABILITY that exercises the shipped code path
//! reaches the stack through the engine's real trigger-fire-and-flush
//! machinery and resolves through a real `PassPriority` command, never a
//! fabricated `StackObject`.
//!
//! # What is being probed
//!
//! Before PB-DX54, `resolve_top_of_stack_inner` opened with
//! `state.stack_objects.pop_back()`, so for the whole of a resolution the
//! resolving object's own stack ENTRY did not exist.
//! `state::stack_registry::stack_index_for_announced_target` returned `None`
//! for it, and both single-target retarget requirements
//! (`TargetSpellWithSingleTarget`, `TargetSpellOrAbilityWithSingleTarget`)
//! resolve their candidate through that function -- so a victim spell could
//! never be redirected onto the Misdirection/Bolt Bend that was redirecting
//! it, contrary to Misdirection's 2004-10-04 ruling: *"You can choose to make
//! a spell on the stack target this spell ... This spell is still on the
//! stack when new targets are selected for the spell."* Fixed: the function
//! now PEEKS (`state.stack_objects.back().cloned()`), and the entry departs
//! through `depart_resolving_stack_entry` at the two CR-ordered points inside
//! `resolve_top_of_stack_inner`, before that same function's own
//! `check_and_apply_sbas` call.
//!
//! The CR warrant is CR 608.2n (*"As the final part of an instant or sorcery
//! spell's resolution, the spell is put into its owner's graveyard ..."*),
//! reinforced by CR 608.2's preamble that 608.2n/608.2p run last -- **not**
//! CR 608.2m, which is about an object removed by something ELSE
//! mid-resolution (see `resolution.rs`'s own doc on
//! `depart_resolving_stack_entry` for the full argument, verified against
//! the rules text before this file cited it).
//!
//! # t3's claim, verified before being written down
//!
//! t3 asserts `TargetRequirement::TargetSpellOrAbility` is unaffected by this
//! fix and stays green both before and after it. Checked directly against
//! `casting.rs::validate_object_satisfies_requirement`'s
//! `TargetSpell | TargetSpellWithFilter | TargetSpellOrAbility` arm
//! (`casting.rs:7022-7071`): it reads `obj.zone` from `state.objects.get(&id)`
//! alone and never touches `state.stack_objects` at all -- the resolving
//! spell's CARD stays in `state.objects` with `zone == Stack` for the entire
//! resolution regardless of what happens to its `StackObject` entry (that is
//! exactly why PB-DX25c's own T7/T8 had to route around
//! `TargetSpellWithSingleTarget`'s blind spot through `TargetSpellWithFilter`
//! or a fourth stack object). So the claim is true, not assumed.
//!
//! # Revert matrix (recorded here, not only in the task's own report)
//!
//! Every probe was run against the shipped code AND against a hand-applied,
//! hand-restored revert of `resolve_top_of_stack_inner`'s peek back to
//! `state.stack_objects.pop_back()` (with both `depart_resolving_stack_entry`
//! calls and the `resolve_top_of_stack` backstop removed, since they would be
//! no-ops under a pop anyway). t1, t2, t4, t5 go RED under that revert (the
//! resolving entry vanishes at the start of its own resolution, so it can
//! never be found as a retarget candidate). t3 stays GREEN under it, as a
//! stated CONTROL (see above -- that arm never reads `state.stack_objects`).
//! t6 stays GREEN under it too, disclosed rather than hidden: "never resolved
//! twice" is a property of `depart_resolving_stack_entry`'s own idempotence
//! and of there being exactly one departure call per resolution, which the
//! revert does not disturb -- it only changes WHEN the (single) departure
//! happens, not whether it happens exactly once. t4b is a CONTROL for a
//! different reason, stated in its own doc: it is structurally unreachable at
//! cast time under EITHER version.
//!
//! t7 is split in two. `t7_non_final_chapters_resolve_normally_with_correct_
//! departure_timing` covers chapters I and II of a real Saga (a triggered
//! ability, not a spell, resolving with the correct departure timing) and
//! stays GREEN under the standard revert for the same reason t6 does --
//! nothing about a non-final chapter's own resolution depends on WHEN the
//! departure happens relative to the tail SBA check, only on Ward/redirect-
//! style visibility during resolution, which this Saga never exercises.
//! `t7b_cr_714_4_same_command_sacrifice_is_confounded_by_a_different_bug` is
//! the intended CR 714.4 final-chapter probe, and building it surfaced a
//! SEPARATE, pre-existing, out-of-scope bug that makes it impossible to
//! observe PB-DX54's own property cleanly -- see that test's own doc for the
//! full account, the CR argument that it really is a bug, and every
//! alternative construction considered and rejected.

//!
//! # The CR 714.4 final-chapter half of t7 could NOT be built, and why
//!
//! **This section was an empty `#[test] fn t7b` in the first draft and the coordinator
//! removed the wrapper.** A `#[test]` that asserts nothing always passes, contributes zero
//! coverage, and — the reason it actually mattered here — adds **+1 to this batch's own
//! reported test delta** for a row that tests nothing, which corrupts the one figure every
//! later batch inherits. The content below is worth keeping; the wrapper was not. The
//! finding itself is filed as **`OOS-DX54-4`**, which is where a real defect with no probe
//! belongs — PB-DX49's `OOS-DX49-1` precedent verbatim: *a probe asserting today's behaviour
//! would have to be inverted by whoever fixes it, and nothing this batch touched is on that
//! path*.
//!
//! A 3-chapter Saga, entirely `Effect::GainLife` so each chapter's resolution
//! effect is independently observable in the controller's life total.
//! CR 714.4 / CR 608.2n -- the shipped departure point for a resolving
//! TRIGGERED ABILITY (not just a spell): the chapter I and chapter II
//! abilities of a real Saga resolve normally, their own resolution effects
//! (`Effect::GainLife`) actually fire, and the Saga survives both times
//! (neither is the final chapter, so CR 714.4 does not apply). This is the
//! achievable half of what the brief calls t7 -- see the doc above
//! for why the FINAL-chapter half (CR 714.4's own sacrifice-timing property,
//! the reason this row exists at all) could not be built.
//! **`t7`'s intended final-chapter half, DROPPED per the task's own escape
//! hatch, and the discovery that blocked it recorded here rather than
//! silently dropped.**
//!
//! The brief's original ask: drive a real 3-chapter Saga to its final
//! chapter and assert the Saga is sacrificed (CR 714.4) in the SAME
//! `PassPriority` command that resolves the final chapter ability -- the
//! probe that would discriminate the shipped departure point (inside
//! `resolve_top_of_stack_inner`, before that function's own tail
//! `check_and_apply_sbas` call) from a hypothetical "depart at the function
//! boundary" design.
//!
//! **Building it found a DIFFERENT, PRE-EXISTING, out-of-scope bug that
//! confounds the observation before PB-DX54's own property can ever be
//! exercised, and this file does not fix it (only these two test files are
//! in scope for this task).** Executed and captured verbatim: when chapter
//! III's lore counter crosses the threshold inside
//! `turn_actions::precombat_main_actions` (a turn-based action, executed
//! from inside `rules::engine::enter_step`), the resulting event tail for
//! that SAME `PassPriority` command is:
//!
//! ```text
//! CounterAdded { object_id: ObjectId(1), counter: Lore, count: 3 },
//! PermanentDestroyed { object_id: ObjectId(1), new_grave_id: ObjectId(24), .. },
//! AbilityTriggered { controller: PlayerId(1), source_object_id: ObjectId(1), stack_object_id: ObjectId(25) },
//! ```
//!
//! The Saga is sacrificed (`PermanentDestroyed`) BEFORE its own chapter III
//! trigger is even placed on the stack (`AbilityTriggered` fires
//! AFTERWARDS) -- one whole mechanism earlier than the property this row
//! exists to probe. The chapter III `PendingTrigger` is queued by
//! `fire_saga_chapter_triggers` during `execute_turn_based_actions`, which
//! `enter_step` runs BEFORE its own `check_and_apply_sbas` call; only AFTER
//! that does `enter_step` call `abilities::flush_pending_triggers` to place
//! the trigger on `state.stack_objects`. So `sba::check_saga_sbas`'s
//! `has_pending_chapter` guard -- which scans `state.stack_objects` alone --
//! finds nothing there yet and sacrifices the Saga a full step too early.
//!
//! **This is CR-wrong, checked against the rule's own wording rather than
//! assumed**: CR 714.4's exemption reads *"it isn't the source of a chapter
//! ability that has TRIGGERED but not yet left the stack"* -- CR 603.2's
//! sense of "triggered" is the EVENT occurring (the condition being met),
//! which is exactly when `fire_saga_chapter_triggers` queues the
//! `PendingTrigger`, not the LATER moment the engine gets around to placing
//! it on the stack. CR 704.3's own loop (check state-based actions, THEN
//! place waiting triggered abilities, repeat) is precisely the scenario
//! CR 714.4's wording is written to guard against -- a Saga must not be
//! sacrificed in the SAME state-based-action pass in which its final chapter
//! crossed the threshold, before that chapter's own trigger has had a chance
//! to be placed. `sba::check_saga_sbas`'s guard checking only
//! `state.stack_objects` (never `state.pending_triggers`) is therefore a
//! distinct defect from anything PB-DX54 touches -- `resolve_top_of_stack_
//! inner`'s CR 608.2n departure point is never reached in this trace at all,
//! since the Saga's card is destroyed via a State-Based Action, not via a
//! spell/ability resolution.
//!
//! **Every alternative construction considered and rejected, so this is a
//! floor rather than a guess:**
//! * A single-chapter Saga (chapter I is also the final chapter) hits the
//!   identical ordering inside `replacement::apply_self_etb_from_definition`
//!   -- the same TBA-queues-then-SBA-checks-then-flushes shape, just at ETB
//!   instead of at a later main phase. Confirmed by reading that call site;
//!   not executed separately, since the mechanism is the same code shape.
//! * Pre-setting `lore = 2` directly via `GameStateBuilder` does not help:
//!   the confound is inside `enter_step`'s ordering for the STEP-ENTRY that
//!   crosses `lore == 3`, and that ordering runs identically regardless of
//!   how `lore` reached 2.
//! * Getting chapter III's ability onto the stack through a real spell's OWN
//!   resolution tail (which DOES check SBAs, then flush, in one call) would
//!   sidestep the confound -- but no generic "put a lore counter on target
//!   Saga" `Effect` variant exists in this DSL that also calls
//!   `fire_saga_chapter_triggers` (that function has exactly one caller,
//!   `turn_actions::precombat_main_actions`, per `replacement.rs`'s own
//!   module doc), so there is no real card/effect that reaches the stack
//!   this way.
//! * Calling `replacement::fire_saga_chapter_triggers` and
//!   `abilities::flush_pending_triggers` directly from the test, bypassing
//!   `enter_step` entirely, would place a REAL `StackObject` through REAL
//!   production functions rather than a hand-built one -- but it is still a
//!   bypass of the real `Command::PassPriority` path this file's hard
//!   constraint requires for the mechanism under test, and the brief's own
//!   escape hatch calls for disclosure and dropping the probe over this
//!   kind of workaround.
//!
//! **Marked UNDISCRIMINATED for `OOS-DX25c-6` in this file, as instructed.**
//! No assertion is written against the confounded (buggy) trace: pinning
//! "the Saga is destroyed before its trigger is placed" as an expected
//! outcome would enshrine a CR violation as a maintained test, which a
//! correct future fix to `sba::check_saga_sbas` would then have to break
//! deliberately. This doc is the record instead.
use std::sync::Arc;

use mtg_engine::state::stack_registry::card_in_stack_zone;
use mtg_engine::{
    process_command, start_game, AbilityDefinition, CardDefinition, CardEffectTarget, CardId,
    CardRegistry, CardType, Command, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    GameStateError, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step,
    SubType, Target, TargetRequirement, TypeLine, ZoneId,
};

// ── Generic helpers ──────────────────────────────────────────────────────────

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

/// The `StackObject`'s own entry id for a given card id currently on the
/// stack -- a DIFFERENT id space from the card id itself
/// (`state::stack_registry`'s own doc: a card id and an entry id are minted
/// one line apart from the same monotone counter, so they never collide but
/// they are never interchangeable either). Used to distinguish "the card
/// moved to the graveyard" (a new card id, CR 400.7) from "the STACK ENTRY
/// departed" (the entry's own `.id` stops appearing in `state.stack_objects`)
/// -- t5/t6 need the second one specifically.
fn stack_entry_id_for_card(state: &GameState, card_id: ObjectId) -> ObjectId {
    state
        .stack_objects()
        .iter()
        .find(|so| card_in_stack_zone(&so.kind) == Some(card_id))
        .map(|so| so.id)
        .unwrap_or_else(|| panic!("no stack entry owns card {:?}", card_id))
}

fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell(Box::new(mtg_engine::rules::command::CastSpellData {
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
}

/// Passes priority as whoever the state says currently holds it, repeatedly,
/// until the top of the stack resolves (the stack shrinks). Mirrors
/// `pb_dx25c_retarget_legality.rs`'s own helper of the same name -- copied
/// rather than shared, since the two test files do not share a module.
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
        "stack did not resolve after 20 passes (started at {} objects, still {}); events: {:?}",
        start_len,
        state.stack_objects().len(),
        all_events
    );
}

/// Resolves the ENTIRE stack, collecting every event across every pass.
/// Panics rather than looping forever if the stack never empties.
fn resolve_all(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    for _ in 0..50 {
        if state.stack_objects().is_empty() {
            return (state, all_events);
        }
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder to resolve the stack"));
        let (s, ev) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = s;
        all_events.extend(ev);
    }
    panic!(
        "stack never emptied after 50 passes; events so far: {:?}",
        all_events
    );
}

/// Passes priority as whoever currently holds it, repeatedly, until `pred`
/// holds. Used by t7 to drive the real turn structure forward to a specific
/// (turn number, step, active player) triple without ever hand-building a
/// `StackObject` -- every intervening trigger (including the Saga's own
/// chapter abilities) resolves for real along the way, exactly like
/// `crates/engine/tests/core/turn_structure.rs::pass_until_advance`, just
/// keyed on an arbitrary predicate instead of "the step changed".
fn advance_until(
    mut state: GameState,
    pred: impl Fn(&GameState) -> bool,
) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    for _ in 0..2000 {
        if pred(&state) {
            return (state, all_events);
        }
        let holder = state.turn().priority_holder.unwrap_or_else(|| {
            panic!(
                "no priority holder while advancing (step {:?}, active {:?}, turn {})",
                state.turn().step,
                state.turn().active_player,
                state.turn().turn_number
            )
        });
        let (s, ev) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = s;
        all_events.extend(ev);
    }
    panic!(
        "advance_until: predicate never satisfied after 2000 passes (last step {:?}, turn {})",
        state.turn().step,
        state.turn().turn_number
    );
}

// ── t1-t6 fixtures: the "triangle" (decoy, victim, redirector) ─────────────

/// A "deals 3 damage to any target" instant -- CR 115.4's `TargetAny`, exactly
/// one target. Used as the DECOY: the object VICTIM targets, and (via its own
/// resolution effect) the thing that proves "untouched" means "actually
/// resolves", not merely "wasn't countered".
fn any_target_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: deals 3 damage to any target."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
                source: None,
            },
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A "counter target spell [with a single target / or ability with a single
/// target / or ability]" instant -- the VICTIM in every triangle fixture. Its
/// OWN declared `targets` requirement (parameterised here) is what
/// `rules::retarget::plan_target_change` reads as `reqs` when something else
/// retargets IT -- it is this requirement, not the redirector's, that
/// selects which `validate_object_satisfies_requirement` arm t1/t2/t3
/// exercise.
fn victim_counter_def(name: &str, card_id: &str, req: TargetRequirement) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: counter target spell."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::CounterSpell {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                exile_instead: false,
            },
            targets: vec![req],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A "change the target of target spell with a single target" instant --
/// structurally identical to Misdirection's own Spell ability, but a
/// SEPARATE card def so t4 can build its own triangle without reusing
/// Misdirection's card id for the object being retargeted.
fn misdirection_clone_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        oracle_text: format!("{name}: change the target of target spell with a single target."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ChangeTargets {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                must_change: true,
            },
            targets: vec![TargetRequirement::TargetSpellWithSingleTarget],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn life_of(state: &GameState, player: PlayerId) -> i32 {
    state
        .players()
        .get(&player)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
        .life_total
}

/// Builds a real 3-spell stack -- DECOY (bottom), VICTIM (middle, whose own
/// declared requirement is `victim_req`), REDIRECTOR (top, a real corpus
/// def) -- entirely through `Command::CastSpell`. Returns the state with all
/// three on the stack and each object's CARD id (not its stack-entry id --
/// see `stack_entry_id_for_card` for that).
///
/// `p1` casts the redirector, `p2` casts VICTIM (targeting DECOY), `p3`
/// casts DECOY (targeting `p1`). Three distinct casters, mirroring
/// `pb_dx25c_retarget_legality.rs`'s own T8 triangle, so no single player's
/// mana pool has to cover two spells.
fn build_triangle_on_stack(
    tag: &str,
    victim_req: TargetRequirement,
    redirector_def: CardDefinition,
    redirector_mana: ManaPool,
) -> (
    GameState,
    PlayerId,
    PlayerId,
    PlayerId,
    ObjectId,
    ObjectId,
    ObjectId,
) {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let decoy = any_target_def(
        &format!("PB-DX54 {tag} Decoy"),
        &format!("pb-dx54-{tag}-decoy"),
    );
    let victim = victim_counter_def(
        &format!("PB-DX54 {tag} Victim"),
        &format!("pb-dx54-{tag}-victim"),
        victim_req,
    );
    let redirector_name = redirector_def.name.clone();
    let redirector_card_id_val = redirector_def.card_id.clone();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![redirector_def.clone(), decoy.clone(), victim.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(p1, redirector_mana)
        .player_mana(
            p2,
            ManaPool {
                colorless: 1,
                blue: 1,
                ..Default::default()
            },
        )
        .player_mana(
            p3,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p3, &decoy.name)
                .in_zone(ZoneId::Hand(p3))
                .with_card_id(decoy.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, &victim.name)
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(victim.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, &redirector_name)
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(redirector_card_id_val)
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let decoy_hand_id = find_obj(&state, &decoy.name);
    let (state, _) = cast(state, p3, decoy_hand_id, vec![Target::Player(p1)])
        .unwrap_or_else(|e| panic!("Decoy cast must succeed: {:?}", e));
    let decoy_card_id = find_stack_obj_on_stack(&state, &format!("{tag} Decoy"));

    let victim_hand_id = find_obj(&state, &victim.name);
    let (state, _) = cast(
        state,
        p2,
        victim_hand_id,
        vec![Target::Object(decoy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Victim cast must succeed: {:?}", e));
    let victim_card_id = find_stack_obj_on_stack(&state, &format!("{tag} Victim"));

    let redirector_hand_id = find_obj(&state, &redirector_name);
    let (state, _) = cast(
        state,
        p1,
        redirector_hand_id,
        vec![Target::Object(victim_card_id)],
    )
    .unwrap_or_else(|e| panic!("Redirector cast must succeed: {:?}", e));
    let redirector_card_id = find_stack_obj_on_stack(&state, &redirector_name);

    (
        state,
        p1,
        p2,
        p3,
        decoy_card_id,
        victim_card_id,
        redirector_card_id,
    )
}

// ── t1: the headline ────────────────────────────────────────────────────────

/// CR 608.2n / CR 115.7a -- Misdirection's own 2004-10-04 ruling: *"This
/// spell is still on the stack when new targets are selected for the
/// spell."* Discriminates the shipped peek against pre-PB-DX54's
/// `pop_back()`: pre-fix, `stack_index_for_announced_target` returned `None`
/// for the departed entry, so Misdirection's own card failed the
/// `TargetSpellWithSingleTarget` arm's `target_count != 1` check (0 targets
/// found, since no stack entry named that card) and was never offered as a
/// retarget candidate at all.
///
/// Asserted by RESOLUTION EFFECT, not by the offer alone: after the redirect,
/// VICTIM's sole target (Misdirection's now-departed card, CR 400.7 a new
/// object in the graveyard) is illegal, so VICTIM must fizzle (CR 608.2b);
/// DECOY must be completely untouched, which is checked by actually watching
/// its damage land, not merely by the absence of a counter event.
#[test]
fn t1_headline_resolving_entry_becomes_a_legal_redirect_candidate() {
    let (state, p1, _p2, _p3, _decoy_card_id, _victim_card_id, redirector_card_id) =
        build_triangle_on_stack(
            "T1",
            TargetRequirement::TargetSpellWithSingleTarget,
            mtg_engine::cards::defs::misdirection::card(),
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        );

    // Non-vacuity floor: three real spells really are on the stack.
    assert_eq!(
        state.stack_objects().len(),
        3,
        "the stack must hold decoy+victim+misdirection before anything resolves"
    );

    // Resolve Misdirection (top of stack): it must retarget VICTIM onto ITS
    // OWN card, the only remaining legal candidate (decoy is the current
    // target and thus excluded, victim is self-excluded).
    let (state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Misdirection must redirect VICTIM: {:?}", resolve_events));
    assert_eq!(new_target.len(), 1, "VICTIM has exactly one target slot");
    assert_eq!(
        new_target[0].target,
        Target::Object(redirector_card_id),
        "OOS-DX25c-6: with the resolving Misdirection's own entry visible for the \
         whole resolution, its card is the only remaining legal candidate for \
         VICTIM's TargetSpellWithSingleTarget requirement -- unreachable pre-fix, \
         when stack_index_for_announced_target returned None for the departed entry"
    );

    // VICTIM must fizzle: its sole target (Misdirection's card) is gone to the
    // graveyard by the time VICTIM tries to resolve (CR 608.2b, CR 400.7).
    let (state, victim_events) = resolve_top_of_stack(state);
    assert!(
        victim_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "VICTIM must fizzle once its sole target (Misdirection's departed card) is \
         illegal, CR 608.2b: {:?}",
        victim_events
    );
    assert!(
        !victim_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCountered { .. })),
        "VICTIM's CounterSpell effect must never fire against DECOY -- the redirect \
         succeeded, so DECOY was never VICTIM's target when it tried to resolve: {:?}",
        victim_events
    );

    // DECOY must be completely untouched -- proven by watching its OWN
    // resolution effect actually happen, not merely by the absence of a
    // counter event (which a no-op resolution would also satisfy).
    let life_before_decoy = life_of(&state, p1);
    let (state, decoy_events) = resolve_top_of_stack(state);
    assert!(
        decoy_events
            .iter()
            .any(|e| matches!(e, GameEvent::DamageDealt { .. })),
        "DECOY must actually resolve and deal its damage -- 'untouched' means it \
         executes normally, not merely that it was never countered: {:?}",
        decoy_events
    );
    assert_eq!(
        life_of(&state, p1),
        life_before_decoy - 3,
        "DECOY's 3 damage must land on its ORIGINAL target (p1), proving it was \
         never itself disturbed by VICTIM's failed counter attempt"
    );
    assert!(
        state.stack_objects().is_empty(),
        "the whole stack must have resolved"
    );
}

// ── t2: the same headline, through the real Bolt Bend ──────────────────────

/// Same shape as t1, but the redirector is the REAL corpus `bolt_bend::card()`
/// and VICTIM declares `TargetSpellOrAbilityWithSingleTarget` (Bolt Bend's own
/// printed requirement, exercised here on VICTIM's side too for CR-fidelity
/// with a card that could plausibly print "counter target spell or ability
/// with a single target"). Bolt Bend is paid at full price (`{3}{R}`, no
/// `SelfCostReduction` discount) -- p1's mana pool covers exactly that.
#[test]
fn t2_headline_via_the_real_bolt_bend() {
    let (state, p1, _p2, _p3, _decoy_card_id, _victim_card_id, redirector_card_id) =
        build_triangle_on_stack(
            "T2",
            TargetRequirement::TargetSpellOrAbilityWithSingleTarget,
            mtg_engine::cards::defs::bolt_bend::card(),
            ManaPool {
                colorless: 3,
                red: 1,
                ..Default::default()
            },
        );

    assert_eq!(
        state.stack_objects().len(),
        3,
        "the stack must hold decoy+victim+bolt bend before anything resolves"
    );

    let (state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Bolt Bend must redirect VICTIM: {:?}", resolve_events));
    assert_eq!(
        new_target[0].target,
        Target::Object(redirector_card_id),
        "OOS-DX25c-6 via the TargetSpellOrAbilityWithSingleTarget arm: Bolt Bend's \
         own card, now visible for the whole resolution, is the only remaining \
         legal candidate"
    );

    let (state, victim_events) = resolve_top_of_stack(state);
    assert!(
        victim_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "VICTIM must fizzle once its sole target (Bolt Bend's departed card) is \
         illegal: {:?}",
        victim_events
    );

    let life_before_decoy = life_of(&state, p1);
    let (state, decoy_events) = resolve_top_of_stack(state);
    assert!(
        decoy_events
            .iter()
            .any(|e| matches!(e, GameEvent::DamageDealt { .. })),
        "DECOY must actually resolve: {:?}",
        decoy_events
    );
    assert_eq!(
        life_of(&state, p1),
        life_before_decoy - 3,
        "DECOY untouched"
    );
    assert!(state.stack_objects().is_empty(), "stack fully resolved");
}

// ── t3: TargetSpellOrAbility is a stated CONTROL, not a gap ────────────────

/// CR 115.7d's plain "target spell or ability" (Deflecting Swat's shape),
/// with NO single-target restriction. **Expected GREEN both before and after
/// PB-DX54**, and that is verified rather than assumed: read directly,
/// `casting.rs::validate_object_satisfies_requirement`'s
/// `TargetSpell | TargetSpellWithFilter | TargetSpellOrAbility` arm
/// (`casting.rs:7022-7071`) checks `obj.zone != ZoneId::Stack` from
/// `state.objects.get(&id)` and the `self_id` exclusion -- it never consults
/// `state.stack_objects` at all, so it was never blind to a departed entry
/// in the first place. The redirector here is Misdirection (any real corpus
/// spell would do; VICTIM's own requirement, not the redirector's, is what
/// this test varies).
#[test]
fn t3_target_spell_or_ability_is_a_stated_control() {
    let (state, _p1, _p2, _p3, _decoy_card_id, _victim_card_id, redirector_card_id) =
        build_triangle_on_stack(
            "T3",
            TargetRequirement::TargetSpellOrAbility,
            mtg_engine::cards::defs::misdirection::card(),
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        );

    let (_state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Misdirection must redirect VICTIM: {:?}", resolve_events));
    assert_eq!(
        new_target[0].target,
        Target::Object(redirector_card_id),
        "TargetSpellOrAbility accepts the redirector's card exactly as it did \
         before this fix -- this arm never depended on the stack-entry lookup"
    );
}

// ── t4: self-targeting is STILL refused, and now for the right reason ──────

/// CR 601.2c -- the victim spell must never be redirected onto its OWN card,
/// even though its own entry is now visible for the whole resolution.
///
/// Simplified relative to `pb_dx25c_retarget_legality.rs`'s own T8, which
/// needed a FOURTH stack object ("Alternative") purely because the resolving
/// Misdirection was invisible pre-fix and therefore could not serve as the
/// alternative. Post-fix, Misdirection IS the alternative, so three objects
/// (decoy, clone, real Misdirection) suffice.
#[test]
fn t4_self_targeting_is_still_refused_and_the_alternative_is_the_redirector() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let decoy = any_target_def("PB-DX54 T4 Decoy", "pb-dx54-t4-decoy");
    let clone_ = misdirection_clone_def("PB-DX54 T4 Clone", "pb-dx54-t4-clone");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), decoy.clone(), clone_.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                colorless: 1,
                blue: 1,
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p3, "PB-DX54 T4 Decoy")
                .in_zone(ZoneId::Hand(p3))
                .with_card_id(decoy.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX54 T4 Clone")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(clone_.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let decoy_hand_id = find_obj(&state, "PB-DX54 T4 Decoy");
    let (state, _) = cast(state, p3, decoy_hand_id, vec![Target::Player(p1)])
        .unwrap_or_else(|e| panic!("Decoy cast must succeed: {:?}", e));
    let decoy_card_id = find_stack_obj_on_stack(&state, "T4 Decoy");

    let clone_hand_id = find_obj(&state, "PB-DX54 T4 Clone");
    let (state, _) = cast(
        state,
        p2,
        clone_hand_id,
        vec![Target::Object(decoy_card_id)],
    )
    .unwrap_or_else(|e| panic!("Clone cast must succeed: {:?}", e));
    let clone_card_id = find_stack_obj_on_stack(&state, "T4 Clone");

    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let (state, _) = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(clone_card_id)],
    )
    .unwrap_or_else(|e| panic!("Misdirection cast must succeed: {:?}", e));
    let misdirection_card_id = find_stack_obj_on_stack(&state, "Misdirection");

    assert_eq!(
        state.stack_objects().len(),
        3,
        "decoy+clone+misdirection must all be on the stack"
    );

    let (_, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("clone must redirect: {:?}", resolve_events));
    assert_ne!(
        new_target[0].target,
        Target::Object(clone_card_id),
        "CR 601.2c: the clone's own card must never be a legal redirect target \
         for its own retarget (self_id exclusion), even now that the resolving \
         Misdirection's entry is visible and would otherwise make the candidate \
         set larger"
    );
    assert_eq!(
        new_target[0].target,
        Target::Object(misdirection_card_id),
        "with decoy (current target, excluded) and clone (self, excluded via \
         self_id), Misdirection's own card -- visible only because of the \
         PB-DX54 fix -- is the sole remaining legal candidate"
    );
}

// ── t4b: self-targeting at CAST time ────────────────────────────────────────

/// CR 601.2c at ANNOUNCEMENT, not at retarget. Attempted here by casting
/// Misdirection naming its own (still-in-hand) card as the target.
///
/// **This does NOT exercise the `self_id` guard**, and that is verified
/// against `casting.rs`'s own comment on the `TargetSpellWithSingleTarget`
/// arm rather than assumed: *"At cast time this guard is provably a no-op:
/// self_id there is card, the PRE-zone-move id ... so it can never equal a
/// candidate id, which this arm already requires to be ZoneId::Stack. The
/// guard is therefore live ONLY on the retarget path this batch created."*
/// At the moment cast-time targets are validated, Misdirection's own card is
/// still in `p1`'s hand (the validation runs BEFORE the zone move onto the
/// stack), so ANY candidate that has not yet been cast -- self-referential or
/// not -- fails the plain "must be on the stack" check first. There is no
/// legal way to construct a genuinely self-id-discriminating cast-time probe:
/// the id that would need to collide with `self_id` is only minted one line
/// AFTER validation runs (`state.next_object_id()`), so no `Command` a test
/// can submit can ever announce it in advance.
///
/// What IS reachable, and asserted here instead: casting Misdirection with no
/// spell of any kind on the stack, naming its own hand card as the target,
/// is refused -- for the "not on the stack" reason, disclosed rather than
/// mis-sold as a self-id probe.
#[test]
fn t4b_self_targeting_at_cast_time_is_refused_but_not_by_the_self_id_guard() {
    let p1 = p(1);
    let p2 = p(2);
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![misdirection.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let result = cast(
        state,
        p1,
        misdirection_hand_id,
        vec![Target::Object(misdirection_hand_id)],
    );
    assert!(
        result.is_err(),
        "casting Misdirection naming its own (still-in-hand) card must be refused"
    );
}

// ── t5: the resolving entry is seen EXACTLY ONCE, never double-seen ────────

/// COUNT, not presence (PB-DX48's rule: a `>= 1` assertion passes on a
/// double-dispatch bug). Before Misdirection resolves, `state.stack_objects`
/// must hold EXACTLY one entry whose `card_in_stack_zone` names Misdirection's
/// card -- this is the exact arithmetic `plan_target_change`'s candidate
/// legality check consults DURING the resolution about to run, since nothing
/// removes any entry until the resolving object's own CR 608.2n departure,
/// which happens strictly AFTER its `ChangeTargets` effect executes. After
/// the command returns, that count must be exactly ZERO, and the entry's OWN
/// id (distinct from its card id -- see `stack_entry_id_for_card`) must never
/// again appear in `state.stack_objects`, and the stack must have shrunk by
/// EXACTLY one.
#[test]
fn t5_resolving_entry_is_seen_exactly_once_during_resolution_and_zero_times_after() {
    let (state, _p1, _p2, _p3, _decoy_card_id, _victim_card_id, redirector_card_id) =
        build_triangle_on_stack(
            "T5",
            TargetRequirement::TargetSpellWithSingleTarget,
            mtg_engine::cards::defs::misdirection::card(),
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        );

    let before_len = state.stack_objects().len();
    assert_eq!(before_len, 3, "three real stack entries before resolution");

    let redirector_entry_id_before = stack_entry_id_for_card(&state, redirector_card_id);

    let matches_before = state
        .stack_objects()
        .iter()
        .filter(|so| card_in_stack_zone(&so.kind) == Some(redirector_card_id))
        .count();
    assert_eq!(
        matches_before, 1,
        "PB-DX54's whole thesis in one COUNT: not zero (pre-fix, the popped entry \
         made this arithmetic answer 0, which is the defect) and not two (a \
         double-push would answer 2, PB-DX47's shape)"
    );

    let (state, resolve_events) = resolve_top_of_stack(state);
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::TargetsChanged { .. })),
        "the redirect must actually have happened: {:?}",
        resolve_events
    );

    assert_eq!(
        state.stack_objects().len(),
        before_len - 1,
        "exactly one entry departs per resolution -- not zero (would mean it never \
         left the stack) and not two (would mean something else also vanished)"
    );

    let matches_after = state
        .stack_objects()
        .iter()
        .filter(|so| card_in_stack_zone(&so.kind) == Some(redirector_card_id))
        .count();
    assert_eq!(
        matches_after, 0,
        "the resolving entry's card must be findable via card_in_stack_zone ZERO \
         times once the command returns -- it moved to the graveyard as a new \
         object (CR 400.7)"
    );

    assert!(
        state
            .stack_objects()
            .iter()
            .all(|so| so.id != redirector_entry_id_before),
        "the resolving entry's OWN stack-entry id (distinct from its card id) must \
         never again appear in state.stack_objects once the command returns"
    );
}

// ── t6: never resolved twice, across a mixed multi-object stack ────────────

/// Drives the full triangle to an empty stack and asserts, across every event
/// from every `PassPriority` in the sequence, that each pushed stack entry
/// appears in EXACTLY ONE resolution-terminal event
/// (`SpellResolved`/`SpellFizzled`/`SpellCountered`/`AbilityResolved`) --
/// never zero, never more than one. This triangle exercises all three
/// terminal shapes at once: Misdirection resolves normally, VICTIM fizzles
/// (CR 608.2b, its sole target departed), DECOY resolves normally.
#[test]
fn t6_never_resolved_twice_across_a_mixed_multi_object_stack() {
    let (state, _p1, _p2, _p3, decoy_card_id, victim_card_id, redirector_card_id) =
        build_triangle_on_stack(
            "T6",
            TargetRequirement::TargetSpellWithSingleTarget,
            mtg_engine::cards::defs::misdirection::card(),
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        );

    // Capture each object's STACK-ENTRY id (what SpellCast/terminal events
    // name), not its card id -- two different id spaces.
    let decoy_entry_id = stack_entry_id_for_card(&state, decoy_card_id);
    let victim_entry_id = stack_entry_id_for_card(&state, victim_card_id);
    let redirector_entry_id = stack_entry_id_for_card(&state, redirector_card_id);
    let pushed: std::collections::BTreeSet<ObjectId> =
        [decoy_entry_id, victim_entry_id, redirector_entry_id]
            .into_iter()
            .collect();
    assert_eq!(
        pushed.len(),
        3,
        "three distinct spells must mint three distinct stack-entry ids"
    );

    let (state, all_events) = resolve_all(state);
    assert!(
        state.stack_objects().is_empty(),
        "the whole stack must have resolved"
    );

    let mut terminal_ids: Vec<ObjectId> = Vec::new();
    for e in &all_events {
        match e {
            GameEvent::SpellResolved {
                stack_object_id, ..
            }
            | GameEvent::SpellFizzled {
                stack_object_id, ..
            }
            | GameEvent::SpellCountered {
                stack_object_id, ..
            }
            | GameEvent::AbilityResolved {
                stack_object_id, ..
            } => {
                terminal_ids.push(*stack_object_id);
            }
            _ => {}
        }
    }
    assert_eq!(
        terminal_ids.len(),
        3,
        "exactly one resolution-terminal event per pushed spell -- not zero \
         (something never resolved) and not more than three (something resolved \
         twice): {:?}",
        terminal_ids
    );
    let terminal_set: std::collections::BTreeSet<ObjectId> = terminal_ids.iter().copied().collect();
    assert_eq!(
        terminal_set.len(),
        3,
        "no id may repeat -- depart_resolving_stack_entry's idempotence must \
         never actually be EXERCISED by a double terminal event for the same \
         entry: {:?}",
        terminal_ids
    );
    assert_eq!(
        terminal_set, pushed,
        "the set of resolved entries must equal the set of pushed entries exactly"
    );
}

// ── t7: a real Saga's chapter abilities, driven through real turn structure ──

/// A 3-chapter Saga, entirely `Effect::GainLife` so each chapter's resolution
/// effect is independently observable in the controller's life total.
fn t7_saga_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("pb-dx54-t7-saga".to_string()),
        name: "PB-DX54 T7 Saga".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            supertypes: imbl::OrdSet::new(),
            card_types: imbl::ordset![CardType::Enchantment],
            subtypes: imbl::ordset![SubType("Saga".to_string())],
        },
        oracle_text: "I -- You gain 3 life. II -- You gain 5 life. III -- You gain 7 life."
            .to_string(),
        abilities: vec![
            AbilityDefinition::SagaChapter {
                chapter: 1,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![],
            },
            AbilityDefinition::SagaChapter {
                chapter: 2,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(5),
                },
                targets: vec![],
            },
            AbilityDefinition::SagaChapter {
                chapter: 3,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(7),
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}

fn lore_of(state: &GameState, id: ObjectId) -> u32 {
    state
        .objects()
        .get(&id)
        .and_then(|o| o.counters.get(&mtg_engine::CounterType::Lore).copied())
        .unwrap_or(0)
}

/// CR 714.4 / CR 608.2n -- the shipped departure point for a resolving
/// TRIGGERED ABILITY (not just a spell): the chapter I and chapter II
/// abilities of a real Saga resolve normally, their own resolution effects
/// (`Effect::GainLife`) actually fire, and the Saga survives both times
/// (neither is the final chapter, so CR 714.4 does not apply). This is the
/// achievable half of what the brief calls t7 -- see the doc above
/// this file's module doc ("The CR 714.4 final-chapter half of t7 could NOT
/// be built, and why") and `OOS-DX54-4`
/// for why the FINAL-chapter half (CR 714.4's own sacrifice-timing property,
/// the reason this row exists at all) could not be built.
#[test]
fn t7_non_final_chapters_resolve_normally_with_correct_departure_timing() {
    let p1 = p(1);
    let p2 = p(2);
    let saga = t7_saga_def();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![saga.clone()]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "PB-DX54 T7 Saga")
                .with_card_id(saga.card_id.clone())
                .with_types(vec![CardType::Enchantment])
                .with_subtypes(vec![SubType("Saga".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1);
    // Enough library filler for every draw step both players pass through --
    // unnamed, unregistered "naked" cards, mirroring
    // `crates/engine/tests/core/turn_structure.rs::four_player_with_libraries`
    // exactly; nothing here ever reads their characteristics.
    for i in 0..8 {
        builder = builder
            .object(
                ObjectSpec::card(p1, &format!("PB-DX54 T7 Filler P1 {i}"))
                    .in_zone(ZoneId::Library(p1)),
            )
            .object(
                ObjectSpec::card(p2, &format!("PB-DX54 T7 Filler P2 {i}"))
                    .in_zone(ZoneId::Library(p2)),
            );
    }
    let state = builder.build().unwrap();
    let (state, _) = start_game(state).unwrap();
    let saga_id = find_obj(&state, "PB-DX54 T7 Saga");
    assert_eq!(
        lore_of(&state, saga_id),
        0,
        "no lore counters before any main phase"
    );

    // Chapter I: advance to p1's first PreCombatMain (turn 1) and resolve.
    let (state, ev1) = advance_until(state, |s| {
        s.turn().step == Step::PreCombatMain && s.turn().active_player == p1
    });
    assert!(
        !state.stack_objects().is_empty(),
        "chapter I's trigger must already be on the stack by the time \
         PreCombatMain is entered: {:?}",
        ev1
    );
    let (state, chapter1_events) = resolve_top_of_stack(state);
    assert!(
        chapter1_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeGained { player, amount: 3 } if *player == p1)),
        "chapter I's own effect (gain 3 life) must have executed: {:?}",
        chapter1_events
    );
    assert_eq!(lore_of(&state, saga_id), 1, "CR 714.2b: chapter I crossed");
    assert!(
        state.objects().contains_key(&saga_id),
        "the Saga must survive chapter I (1 < final chapter 3)"
    );

    // Chapter II: advance to the NEXT time p1 reaches PreCombatMain (turn 3,
    // since turn_number increments once per PLAYER-turn and p2's own turn
    // sits in between).
    let turn_after_1 = state.turn().turn_number;
    let (state, ev2) = advance_until(state, |s| {
        s.turn().turn_number > turn_after_1
            && s.turn().step == Step::PreCombatMain
            && s.turn().active_player == p1
    });
    assert!(
        !state.stack_objects().is_empty(),
        "chapter II's trigger must already be on the stack: {:?}",
        ev2
    );
    let (state, chapter2_events) = resolve_top_of_stack(state);
    assert!(
        chapter2_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeGained { player, amount: 5 } if *player == p1)),
        "chapter II's own effect (gain 5 life) must have executed: {:?}",
        chapter2_events
    );
    assert_eq!(lore_of(&state, saga_id), 2, "CR 714.2b: chapter II crossed");
    assert!(
        state.objects().contains_key(&saga_id),
        "the Saga must survive chapter II (2 < final chapter 3)"
    );
}
