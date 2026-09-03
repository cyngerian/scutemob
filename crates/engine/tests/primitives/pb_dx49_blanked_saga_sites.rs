//! PB-DX49 (CR 714 + CR 613.1f / CR 708.2a): every Saga site reads the printed def, so a
//! blanked Saga kept accruing lore counters, kept firing chapter triggers, and was sacrificed
//! anyway.
//!
//! Engine surface under test: `rules::saga::saga_view` (the one CR 714 query) and
//! `rules::layers::abilities_are_blanked` (the one ability-blanking predicate), consumed by
//! the five behavioural sites — `sba.rs::check_saga_sbas`'s CR 714.4 filter and its
//! chapter-still-on-stack guard, `turn_actions.rs::precombat_main_actions`' CR 714.3b lore
//! counter, `replacement.rs::apply_self_etb_from_definition`'s CR 714.3a lore counter, and
//! `replacement.rs::fire_saga_chapter_triggers`' CR 714.2b threshold crossing.
//!
//! **Every probe here asserts an exact COUNT, never `>= 1`** (PB-DX48's rule: a `>= 1`
//! assertion passes on a double-dispatch bug, and the whole point of `t6` is that one of the
//! two ETB questions must answer 1 while the other answers 0).
//!
//! ## Two blanking channels, and they do NOT answer the same at every site
//!
//! - **CR 613.1f** Layer-6 `RemoveAllAbilities` — the permanent keeps its subtypes, so it is
//!   **still a Saga** with zero chapter abilities.
//! - **CR 708.2a** face-down — *"no text, no name, **no subtypes**, and no mana cost"* — so a
//!   face-down permanent is **not a Saga at all**.
//!
//! `t6` and `t7` are the pair that pins the difference, and `t6` is this batch's correction to
//! its own seed row.
//!
//! ## Fixture notes
//!
//! `GameStateBuilder::build()` registers **no** static continuous effects (`OOS-DX43-6`), so a
//! blanker permanent placed straight onto the battlefield confers nothing. Every blanked
//! fixture here therefore registers the `ContinuousEffect` explicitly via
//! `add_continuous_effect`, exactly as `pb_dx43_intrinsic_land_mana.rs`'s `p9` does.

use mtg_engine::rules::saga::saga_view;
use mtg_engine::state::test_util;
use mtg_engine::{check_and_apply_sbas, *};

// ── Fixtures ─────────────────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

const SAGA_CARD_ID: &str = "pb-dx49-saga";

/// A three-chapter Saga. Chapter effects are deliberately observable in the player's LIFE
/// TOTAL (chapters I and III) and hand size (chapter II) so that `t8` can assert a chapter's
/// **resolution effect** rather than the mere existence of a stack object.
fn saga_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId(SAGA_CARD_ID.to_string()),
        name: "PB-DX49 Test Saga".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            supertypes: imbl::OrdSet::new(),
            card_types: imbl::ordset![CardType::Enchantment],
            subtypes: imbl::ordset![SubType("Saga".to_string())],
        },
        oracle_text: "I — You gain 3 life. II — You gain 5 life. III — You gain 7 life."
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

/// A Humility-shaped Layer-6 `RemoveAllAbilities` over every enchantment on the battlefield.
///
/// Deliberately NOT sourced from a permanent: `OOS-DX43-6` means a conferring permanent placed
/// by the builder registers nothing, and a synthetic effect is what every sibling suite uses.
fn blanket_remove_all_abilities() -> ContinuousEffect {
    ContinuousEffect {
        id: EffectId(9491),
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

/// Build a state with the Saga already on the battlefield (the builder fires no ETB
/// replacements, so it starts with 0 lore counters), optionally under the blanker.
fn saga_on_battlefield(blanked: bool) -> (GameState, ObjectId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![saga_def()]);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "PB-DX49 Test Saga")
                .with_card_id(CardId(SAGA_CARD_ID.to_string()))
                .with_types(vec![CardType::Enchantment])
                .with_subtypes(vec![SubType("Saga".to_string())])
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    if blanked {
        builder = builder.add_continuous_effect(blanket_remove_all_abilities());
    }
    let state = builder.build().unwrap();
    let saga_id = find_saga(&state);
    (state, saga_id, p1)
}

fn find_saga(state: &GameState) -> ObjectId {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "PB-DX49 Test Saga" && o.zone == ZoneId::Battlefield)
        .expect("the Saga fixture is on the battlefield")
        .id
}

fn saga_on_battlefield_count(state: &GameState) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.characteristics.name == "PB-DX49 Test Saga" && o.zone == ZoneId::Battlefield)
        .count()
}

fn lore(state: &GameState, id: ObjectId) -> u32 {
    state
        .objects()
        .get(&id)
        .and_then(|o| o.counters.get(&CounterType::Lore).copied())
        .unwrap_or(0)
}

/// CR 708.2a: turn the permanent face down. `face_down_as` is the conjunct the engine uses
/// everywhere to distinguish a face-down *permanent* (morph/manifest/cloak) from Foretell's
/// and Hideaway's unrelated `face_down` usage — the same spelling `abilities_are_blanked`
/// and `saga_view` read.
fn turn_face_down(state: &mut GameState, id: ObjectId) {
    let obj = state.objects_mut().get_mut(&id).expect("live fixture");
    obj.status.face_down = true;
    obj.face_down_as = Some(FaceDownKind::Manifest);
}

// ── t1 / t2: CR 714.4, the sacrifice SBA ─────────────────────────────────────────────────────

/// CR 714.4: *"If the number of lore counters on a Saga permanent **with one or more chapter
/// abilities** is greater than or equal to its final chapter number ... that Saga's controller
/// sacrifices it."*
///
/// A permanent under an active Layer-6 `RemoveAllAbilities` (CR 613.1f) has no chapter
/// abilities, so CR 714.4 does not reach it and it is **not** sacrificed — even sitting on 3
/// lore counters, which would be at or past the printed final chapter.
///
/// The two halves differ in **exactly one thing**: whether the blanking `ContinuousEffect` is
/// registered. Everything else — the def, the object, the counter count, the SBA call — is
/// identical, which is what makes the discrimination attributable.
#[test]
fn t1_blanked_saga_at_final_chapter_is_not_sacrificed_cr714_4() {
    // Half A: blanked — survives.
    let (mut state, saga_id, _p1) = saga_on_battlefield(true);
    state
        .objects_mut()
        .get_mut(&saga_id)
        .unwrap()
        .counters
        .insert(CounterType::Lore, 3);
    let _ = check_and_apply_sbas(&mut state);
    assert_eq!(
        saga_on_battlefield_count(&state),
        1,
        "CR 714.4 applies only to a Saga permanent WITH one or more chapter abilities; a \
         Layer-6 RemoveAllAbilities leaves none, so the sacrifice SBA must not reach it"
    );

    // Half B: the same fixture without the blanker — sacrificed.
    let (mut state, saga_id, _p1) = saga_on_battlefield(false);
    state
        .objects_mut()
        .get_mut(&saga_id)
        .unwrap()
        .counters
        .insert(CounterType::Lore, 3);
    let _ = check_and_apply_sbas(&mut state);
    assert_eq!(
        saga_on_battlefield_count(&state),
        0,
        "non-vacuity: the identical fixture WITHOUT the blanker must be sacrificed, or half A \
         proves nothing"
    );
}

/// CR 714.4 through the **face-down** channel (CR 708.2a: no text, no name, **no subtypes**).
/// A manifested Saga is not a Saga and has no chapter abilities, so it is not sacrificed no
/// matter how many lore counters it carries.
#[test]
fn t2_face_down_saga_at_final_chapter_is_not_sacrificed_cr714_4() {
    let (mut state, saga_id, _p1) = saga_on_battlefield(false);
    turn_face_down(&mut state, saga_id);
    state
        .objects_mut()
        .get_mut(&saga_id)
        .unwrap()
        .counters
        .insert(CounterType::Lore, 3);

    let _ = check_and_apply_sbas(&mut state);

    assert_eq!(
        saga_on_battlefield_count(&state),
        1,
        "CR 708.2a: a face-down permanent has no subtypes and no abilities, so CR 714.4's \
         'Saga permanent with one or more chapter abilities' does not describe it"
    );
    let view = saga_view(&state, saga_id);
    assert_eq!(
        view.final_chapter(),
        None,
        "CR 714.2d: with no retained chapter abilities there is no final chapter number to \
         compare against"
    );
    assert!(
        !view.is_saga_permanent,
        "CR 708.2a: no subtypes means not a Saga"
    );
}

// ── t3 / t4: CR 714.3b, the precombat main lore counter ──────────────────────────────────────

/// CR 714.3b: *"As a player's precombat main phase begins, that player puts a lore counter on
/// each Saga they control **with one or more chapter abilities**."* The clause is in the rule,
/// so a blanked permanent takes **no** counter.
#[test]
fn t3_blanked_saga_takes_no_precombat_lore_counter_cr714_3b() {
    // Half A: blanked — no counter.
    let (mut state, saga_id, _p1) = saga_on_battlefield(true);
    assert_eq!(lore(&state, saga_id), 0, "fixture starts at zero");
    let _ = mtg_engine::rules::turn_actions::execute_turn_based_actions(&mut state).unwrap();
    assert_eq!(
        lore(&state, saga_id),
        0,
        "CR 714.3b's 'with one or more chapter abilities' clause excludes a permanent under a \
         Layer-6 RemoveAllAbilities"
    );

    // Half B: un-blanked — exactly one counter.
    let (mut state, saga_id, _p1) = saga_on_battlefield(false);
    let _ = mtg_engine::rules::turn_actions::execute_turn_based_actions(&mut state).unwrap();
    assert_eq!(
        lore(&state, saga_id),
        1,
        "non-vacuity: the identical fixture WITHOUT the blanker takes exactly one lore counter"
    );
}

/// CR 714.3b through the face-down channel (CR 708.2a). A manifested Saga is not a Saga.
#[test]
fn t4_face_down_saga_takes_no_precombat_lore_counter_cr714_3b() {
    let (mut state, saga_id, _p1) = saga_on_battlefield(false);
    turn_face_down(&mut state, saga_id);

    let _ = mtg_engine::rules::turn_actions::execute_turn_based_actions(&mut state).unwrap();

    assert_eq!(
        lore(&state, saga_id),
        0,
        "CR 708.2a: a manifested permanent has no subtypes, so CR 714.3b's 'each Saga they \
         control' does not describe it"
    );
}

// ── t5: CR 714.2b, the chapter trigger ───────────────────────────────────────────────────────

/// CR 714.2b: a chapter ability means *"When one or more lore counters are put onto this Saga,
/// if the number of lore counters on it was less than N and became at least N, [effect]"* — so
/// the ability has to **exist** at the moment the counters are put on. A blanked Saga queues
/// nothing.
///
/// This calls `fire_saga_chapter_triggers` directly rather than going through the precombat
/// TBA, because site 3 (`t3`) already declines to place the counter for a blanked Saga; going
/// through the TBA would measure site 3 twice and site 5 not at all.
#[test]
fn t5_blanked_saga_queues_no_chapter_triggers_cr714_2b() {
    // Half A: blanked, lore crossing 0 -> 2 (would cross chapters I and II) — nothing queued.
    let (mut state, saga_id, p1) = saga_on_battlefield(true);
    *state.pending_triggers_mut() = imbl::Vector::new();
    let _ =
        mtg_engine::rules::replacement::fire_saga_chapter_triggers(&mut state, saga_id, p1, 0, 2);
    assert_eq!(
        state.pending_triggers().len(),
        0,
        "CR 714.2b: the chapter ability must exist when the counters are put on; under a \
         Layer-6 RemoveAllAbilities there is none, so no threshold crossing can trigger"
    );

    // Half B: un-blanked, the same crossing — exactly two.
    let (mut state, saga_id, p1) = saga_on_battlefield(false);
    *state.pending_triggers_mut() = imbl::Vector::new();
    let _ =
        mtg_engine::rules::replacement::fire_saga_chapter_triggers(&mut state, saga_id, p1, 0, 2);
    assert_eq!(
        state.pending_triggers().len(),
        2,
        "non-vacuity: chapters I and II both cross on a 0 -> 2 jump (CR 714.2c)"
    );
    let indices: Vec<usize> = state
        .pending_triggers()
        .iter()
        .map(|t| t.ability_index)
        .collect();
    assert_eq!(
        indices,
        vec![0, 1],
        "CR 712.8d/e: the queued indices are enumeration indices into the visible face's \
         effective ability list, which is the namespace every consumer resolves against"
    );
}

// ── t6 / t7: CR 714.3a, the ETB lore counter — the batch's correction to its own seed ────────

/// **CR 714.3a verbatim** (June 2025 wording): *"As a Saga **without the read ahead ability**
/// enters the battlefield, its controller puts a lore counter on it."*
///
/// **Note what is NOT in that sentence: there is no "with one or more chapter abilities"
/// clause**, unlike CR 714.3b and CR 714.4, which both carry it explicitly.
///
/// `OOS-RR4-1`'s row says *"A fix to the first three alone leaves a blanked Saga still taking
/// its ETB counter and still firing chapter I"*, framing the ETB counter as part of the
/// defect. **That is wrong for this channel and this test pins it.** CR 613.1f removes
/// abilities, not subtypes, so a permanent entering under a Layer-6 `RemoveAllAbilities` **is
/// still a Saga** and CR 714.3a still puts a lore counter on it. Suppressing that counter
/// would be CR-*wrong*, and observably so: if the blanker later leaves, a Saga that entered
/// with 1 lore counter must resume at chapter II rather than firing chapter I.
///
/// So the two halves of this site answer differently, and both are asserted here as exact
/// counts: **1 lore counter, 0 chapter triggers.** That combination is precisely the correct
/// outcome — the chapter never triggered while blanked (CR 714.2b), and the counter is on the
/// permanent to be resumed from.
#[test]
fn t6_saga_entering_blanked_takes_its_etb_counter_but_fires_no_chapter_cr714_3a() {
    let (mut state, saga_id, p1) = saga_on_battlefield(true);
    *state.pending_triggers_mut() = imbl::Vector::new();
    let registry = state.card_registry().clone();

    let _ = mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        saga_id,
        p1,
        Some(&CardId(SAGA_CARD_ID.to_string())),
        &registry,
    );

    assert_eq!(
        lore(&state, saga_id),
        1,
        "CR 714.3a has NO 'with one or more chapter abilities' clause: a blanked permanent \
         keeps its subtypes (CR 613.1f removes abilities, not types), so it is still a Saga \
         and still gets its ETB lore counter. This is the correction to OOS-RR4-1's own claim."
    );
    assert_eq!(
        state.pending_triggers().len(),
        0,
        "CR 714.2b: chapter I must NOT trigger, because the ability does not exist at the \
         moment the counter is put on. Keeping the counter and firing nothing is the pair \
         that makes a later un-blanking resume at chapter II."
    );
}

/// CR 714.3a through the face-down channel. CR 708.2a gives a face-down permanent **no
/// subtypes**, so it is not a Saga, so CR 714.3a does not describe it and **no** lore counter
/// is placed. This is the half of `OOS-RR4-1`'s ETB claim that is right — and it is right for
/// a different reason than the row gives.
#[test]
fn t7_saga_entering_face_down_takes_no_etb_counter_cr714_3a() {
    let (mut state, saga_id, p1) = saga_on_battlefield(false);
    turn_face_down(&mut state, saga_id);
    *state.pending_triggers_mut() = imbl::Vector::new();
    let registry = state.card_registry().clone();

    let _ = mtg_engine::rules::replacement::apply_self_etb_from_definition(
        &mut state,
        saga_id,
        p1,
        Some(&CardId(SAGA_CARD_ID.to_string())),
        &registry,
    );

    assert_eq!(
        lore(&state, saga_id),
        0,
        "CR 708.2a: no text, no name, NO SUBTYPES — a manifested permanent is not a Saga, so \
         CR 714.3a's 'as a Saga enters the battlefield' never fires"
    );
    assert_eq!(
        state.pending_triggers().len(),
        0,
        "and no chapter ability exists to trigger either"
    );
}

// ── t8: CR 113.7a, the deliberate EXCLUSION ──────────────────────────────────────────────────

/// CR 113.7a: *"an ability on the stack exists independently of its source"* — once a chapter
/// ability has triggered and gone on the stack, blanking the Saga neither counters it nor
/// changes it.
///
/// This is why `resolution.rs`'s two `AbilityDefinition::SagaChapter` lookups are deliberately
/// **not** consumers of `saga::saga_view` (they carry a source comment saying so). Wiring the
/// query there would make this chapter fizzle.
///
/// The verdict is the chapter's **resolution effect** (the controller's life total), not the
/// existence of the stack object — an existence assertion would be satisfied by a stack entry
/// that resolved to nothing.
#[test]
fn t8_chapter_already_on_the_stack_still_resolves_after_its_source_is_blanked_cr113_7a() {
    let (mut state, saga_id, p1) = saga_on_battlefield(true);
    let life_before = state.players().get(&p1).unwrap().life_total;

    // Chapter I (`ability_index` 0, "gain 3 life") has already triggered and is on the stack.
    let stack_id = test_util::next_object_id(&mut state);
    let stack_obj = StackObject::trigger_default(
        stack_id,
        p1,
        StackObjectKind::TriggeredAbility {
            source_object: saga_id,
            ability_index: 0,
            is_carddef_etb: false,
            embedded_effect: None,
        },
    );
    state.stack_objects_mut().push_back(stack_obj);

    let _ = mtg_engine::rules::resolution::resolve_top_of_stack(&mut state).unwrap();

    assert_eq!(
        state.players().get(&p1).unwrap().life_total,
        life_before + 3,
        "CR 113.7a: the chapter ability on the stack exists independently of its source, so \
         blanking the Saga after it triggered must not stop it resolving. Verdict is the \
         RESOLUTION EFFECT (+3 life), not the stack object's existence."
    );
}

// ── t9: the chapter-still-on-stack guard (site 2) ────────────────────────────────────────────

/// CR 714.4's second clause — *"and it isn't the source of a chapter ability that has
/// triggered but not yet left the stack"* — is now asked through `SagaView::is_chapter_index`.
///
/// For a blanked Saga the guard **no longer matches**: there are no retained chapters, so no
/// index is a chapter index. This test asserts that explicitly **and** asserts that the outer
/// CR 714.4 exemption (`final_chapter() == None`) is what keeps the permanent alive — so the
/// two sites are shown to AGREE rather than one silently masking the other. If the outer
/// filter regressed, this permanent would be sacrificed while the guard that used to protect
/// it now declines to, and this test would go red on the survival assertion.
#[test]
fn t9_blanked_saga_stack_guard_no_longer_matches_and_the_outer_exemption_is_what_saves_it() {
    let (mut state, saga_id, p1) = saga_on_battlefield(true);
    state
        .objects_mut()
        .get_mut(&saga_id)
        .unwrap()
        .counters
        .insert(CounterType::Lore, 3);

    // A chapter-III trigger (printed `ability_index` 2) from this Saga is on the stack.
    let stack_id = test_util::next_object_id(&mut state);
    let stack_obj = StackObject::trigger_default(
        stack_id,
        p1,
        StackObjectKind::TriggeredAbility {
            source_object: saga_id,
            ability_index: 2,
            is_carddef_etb: false,
            embedded_effect: None,
        },
    );
    state.stack_objects_mut().push_back(stack_obj);

    let view = saga_view(&state, saga_id);
    assert!(
        !view.is_chapter_index(2),
        "site 2: with no retained chapter abilities, printed index 2 is not a chapter index — \
         the guard declines to match"
    );
    assert_eq!(
        view.final_chapter(),
        None,
        "site 1: and the outer CR 714.4 filter excludes the permanent entirely, which is what \
         actually keeps it alive"
    );

    let _ = check_and_apply_sbas(&mut state);
    assert_eq!(
        saga_on_battlefield_count(&state),
        1,
        "the two sites agree: the guard no longer matches AND the outer exemption holds, so \
         the permanent survives. A regression in site 1 alone would sacrifice it here, since \
         site 2 no longer covers for it."
    );

    // Non-vacuity: un-blanked, the same stack entry IS a chapter index and IS what saves it.
    let (mut state, saga_id, p1) = saga_on_battlefield(false);
    state
        .objects_mut()
        .get_mut(&saga_id)
        .unwrap()
        .counters
        .insert(CounterType::Lore, 3);
    let stack_id = test_util::next_object_id(&mut state);
    let stack_obj = StackObject::trigger_default(
        stack_id,
        p1,
        StackObjectKind::TriggeredAbility {
            source_object: saga_id,
            ability_index: 2,
            is_carddef_etb: false,
            embedded_effect: None,
        },
    );
    state.stack_objects_mut().push_back(stack_obj);
    assert!(
        saga_view(&state, saga_id).is_chapter_index(2),
        "non-vacuity: un-blanked, printed index 2 IS chapter III"
    );
    let _ = check_and_apply_sbas(&mut state);
    assert_eq!(
        saga_on_battlefield_count(&state),
        1,
        "non-vacuity: un-blanked and at its final chapter, this permanent survives ONLY \
         because of site 2's guard (CR 714.4's second clause)"
    );
}

// ── t10: the fizzle-path delta disclosed in the plan's §2c ───────────────────────────────────

/// **The one behavioural delta this batch's site-5 refactor introduces, pinned rather than
/// left to be discovered later.**
///
/// `fire_saga_chapter_triggers` used to take a `def: &CardDefinition` and enumerate it. When
/// the object had already departed (CR 400.7 — a legal fizzle), it therefore still pushed
/// chapter triggers naming a **dead** id. It now enumerates `saga_view(..).chapters`, and the
/// view returns nothing for a missing object, so nothing is pushed.
///
/// That is the CR-correct answer: a departed Saga has no ability to trigger.
#[test]
fn t10_fire_saga_chapter_triggers_on_a_departed_object_pushes_nothing_cr400_7() {
    let (mut state, saga_id, p1) = saga_on_battlefield(false);
    let owner = state.objects().get(&saga_id).unwrap().owner;

    // Non-vacuity first: while live, this exact call pushes exactly three triggers.
    *state.pending_triggers_mut() = imbl::Vector::new();
    let _ =
        mtg_engine::rules::replacement::fire_saga_chapter_triggers(&mut state, saga_id, p1, 0, 3);
    assert_eq!(
        state.pending_triggers().len(),
        3,
        "non-vacuity: a live Saga crossing 0 -> 3 queues all three chapters"
    );

    // Now let it depart (CR 400.7: the old id is dead) and repeat.
    let mut state2 = state.clone();
    *state2.pending_triggers_mut() = imbl::Vector::new();
    let moved = test_util::move_object_to_zone(&mut state2, saga_id, ZoneId::Graveyard(owner));
    assert!(moved.is_ok(), "the fixture must actually depart");
    let _ =
        mtg_engine::rules::replacement::fire_saga_chapter_triggers(&mut state2, saga_id, p1, 0, 3);
    assert_eq!(
        state2.pending_triggers().len(),
        0,
        "CR 400.7: the old ObjectId is dead. The pre-PB-DX49 body enumerated the passed \
         CardDefinition and pushed three triggers naming a departed object; the view returns \
         no chapters for a missing object, so nothing is pushed."
    );
}
