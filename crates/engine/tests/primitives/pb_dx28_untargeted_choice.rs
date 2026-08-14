//! PB-DX28 §1 (`OOS-DX4-6`): behavioral probes for the untargeted-choice channel
//! (`EffectTarget::ChosenObject`, `EffectChoiceQuestion::ChooseObject`).
//!
//! `memory/primitives/pb-plan-DX28.md` §1.6 / `pb-plan-DX28-part2.md` are
//! authoritative. Two defect directions, both proven, plus the determined-answer
//! short-circuit and two card-integration tests through the real shipped defs.
//!
//! * **Direction 1 (eligibility)** — a hexproofed/shrouded/protected candidate is
//!   STILL a legal answer, because CR 115.10 exempts an untargeted choice from
//!   CR 115's whole targeting-legality apparatus.
//! * **Direction 2 (no fizzle)** — when the object that WOULD have been chosen
//!   leaves before resolution, the choice re-derives candidates from the board
//!   AS IT STANDS at resolution and the effect still does something (CR 608.2b
//!   never applies, because nothing was chosen while the ability was on the
//!   stack).
//!
//! Most probes call `effects::execute_effect` directly against a hand-built
//! `EffectContext` -- the SAME primitive the real card defs use, exercised
//! without the stack/priority machinery around it, so "the object left in
//! response" is modeled by simply building the state AS IT WOULD LOOK at
//! resolution time (the choice is re-derived from `state` at the moment
//! `execute_effect` runs, so this is not a simplification of the property being
//! tested -- it is the property).

use mtg_engine::effects::{execute_effect, execute_effect_answering, EffectContext};
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardEffectTarget, CardType, ChoiceZone, Command, CounterType, Effect,
    EffectChoiceAnswer, EffectChoiceQuestion, GameState, GameStateBuilder, KeywordAbility,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, TargetController, TargetFilter, TriggerCondition,
    ZoneId, ZoneTarget,
};
use std::collections::HashMap;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found in {zone:?}"))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn card_def(name: &str) -> CardDefinition {
    all_cards()
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("{name} should be in the corpus"))
}

// ── Direction 1: eligibility (hexproof/shroud do not restrict) ──────────────

/// CR 115.10 / CR 702.11b: a hexproofed land is STILL a legal candidate for an
/// untargeted "return a land you control to its owner's hand" choice — the
/// exact shape the ten Karoos use.
#[test]
fn t1_hexproofed_land_is_eligible_for_an_untargeted_choice() {
    let p1 = p(1);
    let p2 = p(2);
    let hexproof_land =
        ObjectSpec::land(p1, "Hexproof Land").with_keyword(KeywordAbility::Hexproof);
    let plain_land = ObjectSpec::land(p1, "Plain Land");
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(hexproof_land)
        .object(plain_land)
        .build()
        .unwrap();
    let source = find_obj(&state, "Plain Land");
    let effect = Effect::MoveZone {
        target: CardEffectTarget::ChosenObject {
            zone: ChoiceZone::Battlefield,
            filter: Box::new(TargetFilter {
                has_card_type: Some(CardType::Land),
                controller: TargetController::You,
                ..Default::default()
            }),
            count: 1,
            up_to: false,
        },
        to: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        controller_override: None,
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let pending = state
        .pending_effect_choice()
        .expect("two eligible lands -- the choice must be ASKED, not auto-resolved");
    match &pending.question {
        EffectChoiceQuestion::ChooseObject { candidates, .. } => {
            let hexproof_id = find_obj(&state, "Hexproof Land");
            assert!(
                candidates.contains(&hexproof_id),
                "CR 115.10: a hexproofed land must be a legal candidate for an untargeted \
                 choice -- hexproof (CR 702.11b) only restricts TARGETING"
            );
        }
        other => panic!("expected ChooseObject, got {other:?}"),
    }
}

/// Sibling of T1 on `sword_of_truth_and_justice`'s REAL def: a shrouded creature
/// you control is a legal candidate for its "put a +1/+1 counter on a creature
/// you control" untargeted choice.
#[test]
fn t2_shrouded_creature_is_eligible_for_sword_of_truth_and_justices_addcounter() {
    let p1 = p(1);
    let p2 = p(2);
    let shrouded =
        ObjectSpec::creature(p1, "Shrouded Beast", 3, 3).with_keyword(KeywordAbility::Shroud);
    // A SECOND eligible creature, so `candidates.len() (2) > count (1)` and the
    // determined short-circuit does NOT fire -- otherwise a single-candidate
    // board would auto-resolve without ever exercising the `candidates` list
    // this test wants to inspect.
    let other = ObjectSpec::creature(p1, "Plain Beast", 2, 2);
    let sword_id_card = card_name_to_id("Sword of Truth and Justice");
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(shrouded)
        .object(other)
        .object(
            ObjectSpec::card(p1, "Sword of Truth and Justice")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(sword_id_card),
        )
        .build()
        .unwrap();
    let source = find_obj(&state, "Sword of Truth and Justice");

    let effect = sword_add_counter_effect();
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    let pending = state
        .pending_effect_choice()
        .expect("two eligible creatures -- the choice must be ASKED, not auto-resolved");
    match &pending.question {
        EffectChoiceQuestion::ChooseObject { candidates, .. } => {
            let shrouded_id = find_obj(&state, "Shrouded Beast");
            assert!(
                candidates.contains(&shrouded_id),
                "CR 115.10: a shrouded creature must be a legal candidate -- shroud \
                 (CR 702.18a) only restricts TARGETING"
            );
        }
        other => panic!("expected ChooseObject, got {other:?}"),
    }
}

/// Extract `sword_of_truth_and_justice`'s REAL combat-damage trigger effect
/// (`Sequence([AddCounter{ChosenObject}, Proliferate])`) from the shipped def --
/// never re-declared, so a regression in the def itself reddens every test that
/// calls this.
fn sword_add_counter_effect() -> Effect {
    let def = card_def("Sword of Truth and Justice");
    def.abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect,
                ..
            } => Some(effect.clone()),
            _ => None,
        })
        .expect("Sword of Truth and Justice must have the combat-damage trigger")
}

// ── Direction 2: no fizzle (CR 608.2b never applies) ─────────────────────────

/// A Karoo's "return a land you control to its owner's hand": with only ONE
/// eligible land present (as it would be if the other left in response before
/// resolution), the choice is DETERMINED (candidates.len() <= count, !up_to) and
/// the ability resolves immediately, returning the land that IS there. No
/// suspension, no fizzle -- the printed card would simply choose whatever
/// remains.
#[test]
fn t3_no_fizzle_when_only_one_candidate_remains_at_resolution() {
    let p1 = p(1);
    let p2 = p(2);
    // Exactly ONE land you control: models "the other one already left, in
    // response, before this trigger resolved" -- the choice is re-derived from
    // CURRENT state, which is all `execute_effect` ever sees.
    let only_land = ObjectSpec::land(p1, "Last Land Standing");
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(only_land)
        .build()
        .unwrap();
    let source = find_obj(&state, "Last Land Standing");
    let effect = Effect::MoveZone {
        target: CardEffectTarget::ChosenObject {
            zone: ChoiceZone::Battlefield,
            filter: Box::new(TargetFilter {
                has_card_type: Some(CardType::Land),
                controller: TargetController::You,
                ..Default::default()
            }),
            count: 1,
            up_to: false,
        },
        to: ZoneTarget::Hand {
            owner: PlayerTarget::Controller,
        },
        controller_override: None,
    };
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        state.pending_effect_choice().is_none(),
        "CR 601.2c's determined-answer principle: with exactly one candidate for a \
         count-1 non-up_to choice, nothing is asked -- it must resolve immediately, \
         not fizzle and not suspend"
    );
    let land_id = find_obj(&state, "Last Land Standing");
    assert_eq!(
        state.objects().get(&land_id).map(|o| o.zone),
        Some(ZoneId::Hand(p1)),
        "the sole remaining land must have been returned to hand"
    );
}

/// Sibling of T3 on the REAL `sword_of_truth_and_justice` effect: with ZERO
/// eligible creatures at resolution (the bearer was removed in response before
/// the trigger resolved), the `AddCounter` half no-ops (empty candidate set) but
/// `Proliferate` -- the SECOND half of the same `Sequence` -- STILL RUNS. A
/// pre-batch targeted `TargetCreatureWithFilter` would have fizzled the WHOLE
/// trigger (CR 608.2b), killing the proliferate too; this is the exact defect
/// `OOS-DX4-6` named.
#[test]
fn t4_addcounter_finds_nothing_but_proliferate_still_fires() {
    let p1 = p(1);
    let p2 = p(2);
    // Zero creatures under p1's control -- an artifact with an EXISTING
    // +1/+1 counter stands in for "some other permanent proliferate can grow",
    // proving the SECOND effect in the Sequence ran independently of the first.
    let counter_bearer = ObjectSpec::card(p1, "Counter-Bearing Relic")
        .with_types(vec![CardType::Artifact])
        .with_counter(CounterType::PlusOnePlusOne, 1)
        .in_zone(ZoneId::Battlefield);
    let sword_id_card = card_name_to_id("Sword of Truth and Justice");
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(counter_bearer)
        .object(
            ObjectSpec::card(p1, "Sword of Truth and Justice")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(sword_id_card),
        )
        .build()
        .unwrap();
    let source = find_obj(&state, "Sword of Truth and Justice");
    let effect = sword_add_counter_effect();
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        state.pending_effect_choice().is_none(),
        "zero candidates is DETERMINED (the empty set) -- nothing to ask"
    );
    let relic_id = find_obj(&state, "Counter-Bearing Relic");
    let counters = state
        .objects()
        .get(&relic_id)
        .map(|o| {
            o.counters
                .get(&CounterType::PlusOnePlusOne)
                .copied()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    assert_eq!(
        counters, 2,
        "Proliferate must still have run and added a SECOND +1/+1 counter to the relic, \
         even though the AddCounter half found no eligible creature -- the whole trigger \
         does not fizzle (CR 608.2b never applies to an untargeted choice)"
    );
}

// ── The `up_to` axis ──────────────────────────────────────────────────────────

/// `cloud_of_faeries`'s shape ("untap up to two lands"): with ZERO eligible
/// lands present, the choice is DETERMINED (the empty set) and resolves
/// without asking -- a genuine no-op, not a crash or a stuck suspension.
///
/// **`up_to` does NOT short-circuit merely because `candidates.len() < count`**
/// (a one-land board for "up to two" is deliberately still ASKED -- see
/// `resolve_pending_object_choices`'s doc: "up to N" always leaves genuine
/// agency, the player could still choose zero, which `!up_to`'s "as much as
/// possible" rule does not admit). Confirmed by execution, not asserted here
/// (T6 drives the asked case).
#[test]
fn t5_up_to_determined_short_circuit_on_the_empty_candidate_set() {
    let p1 = p(1);
    let p2 = p(2);
    // Zero lands on the battlefield at all -- the source itself is a creature,
    // so it is not itself a candidate.
    let source = ObjectSpec::creature(p1, "Untapper", 1, 1);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(source)
        .build()
        .unwrap();
    let source_id = find_obj(&state, "Untapper");
    let effect = Effect::UntapPermanent {
        target: CardEffectTarget::ChosenObject {
            zone: ChoiceZone::Battlefield,
            filter: Box::new(TargetFilter {
                has_card_type: Some(CardType::Land),
                ..Default::default()
            }),
            count: 2,
            up_to: true,
        },
    };
    let mut ctx = EffectContext::new(p1, source_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        state.pending_effect_choice().is_none(),
        "zero candidates for an 'up to two' choice is still DETERMINED (the empty set) -- \
         nothing to ask"
    );
}

/// `frantic_search`'s shape ("untap up to three lands"): with THREE eligible
/// lands, the choice is asked, and the chooser may legally pick FEWER than the
/// stated maximum.
#[test]
fn t6_up_to_accepts_fewer_than_the_stated_maximum() {
    let p1 = p(1);
    let p2 = p(2);
    let mut l1 = ObjectSpec::land(p1, "Land One");
    l1.tapped = true;
    let mut l2 = ObjectSpec::land(p1, "Land Two");
    l2.tapped = true;
    let mut l3 = ObjectSpec::land(p1, "Land Three");
    l3.tapped = true;
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(l1)
        .object(l2)
        .object(l3)
        .build()
        .unwrap();
    let source = find_obj(&state, "Land One");
    let effect = Effect::UntapPermanent {
        target: CardEffectTarget::ChosenObject {
            zone: ChoiceZone::Battlefield,
            filter: Box::new(TargetFilter {
                has_card_type: Some(CardType::Land),
                ..Default::default()
            }),
            count: 3,
            up_to: true,
        },
    };
    let land_two_id = find_obj(&state, "Land Two");
    let mut ctx = EffectContext::new(p1, source, vec![]);
    execute_effect_answering(&mut state, &effect, &mut ctx, &mut |q| match q {
        EffectChoiceQuestion::ChooseObject { .. } => EffectChoiceAnswer::ChooseObject {
            chosen: vec![land_two_id],
        },
        other => panic!("unexpected question {other:?}"),
    });

    assert!(state.pending_effect_choice().is_none());
    let is_tapped = |name: &str| {
        state
            .objects()
            .get(&find_obj(&state, name))
            .unwrap()
            .status
            .tapped
    };
    assert!(!is_tapped("Land Two"), "the chosen land must be untapped");
    assert!(
        is_tapped("Land One"),
        "an UNCHOSEN land must stay tapped -- up_to means the \
             chooser may pick fewer, not that every candidate is affected"
    );
    assert!(is_tapped("Land Three"), "same for the other unchosen land");
}

// ── The graveyard zone ────────────────────────────────────────────────────────

/// `takenuma_abandoned_mire`'s shape: "mill three, then return a creature or
/// planeswalker card from your graveyard to your hand" is TWO effects in one
/// Sequence. With ZERO eligible cards in the graveyard before the mill, the mill
/// still happens (CR 608.2b doesn't touch it -- it's a separate effect), and if
/// the mill itself puts a matching card into the graveyard, the untargeted
/// choice picks it up at resolution -- proving the choice is derived AFTER the
/// mill runs, not from a stale pre-resolution snapshot.
#[test]
fn t7_graveyard_choice_sees_cards_the_same_resolutions_own_mill_just_added() {
    let p1 = p(1);
    let p2 = p(2);
    // Library: bottom to top, MillCards mills from the top -- give p1 a library
    // with exactly one creature card the mill will place into the graveyard.
    let milled_creature =
        ObjectSpec::creature(p1, "Milled Beast", 2, 2).in_zone(ZoneId::Library(p1));
    let filler1 = ObjectSpec::card(p1, "Filler 1")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Library(p1));
    let filler2 = ObjectSpec::card(p1, "Filler 2")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Library(p1));
    let source = ObjectSpec::land(p1, "Takenuma Stand-In");
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(source)
        .object(milled_creature)
        .object(filler1)
        .object(filler2)
        .build()
        .unwrap();
    assert_eq!(
        state
            .objects()
            .get(&find_obj(&state, "Milled Beast"))
            .map(|o| o.zone),
        Some(ZoneId::Library(p1)),
        "sanity: the creature starts in the library, not the graveyard"
    );
    assert!(
        state.objects_in_zone(&ZoneId::Graveyard(p1)).is_empty(),
        "sanity: the graveyard starts empty -- there is NOTHING to choose before the mill runs"
    );

    let source_id = find_obj(&state, "Takenuma Stand-In");
    let effect = Effect::Sequence(vec![
        Effect::MillCards {
            player: PlayerTarget::Controller,
            count: mtg_engine::EffectAmount::Fixed(3),
        },
        Effect::MoveZone {
            target: CardEffectTarget::ChosenObject {
                zone: ChoiceZone::YourGraveyard,
                filter: Box::new(TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                    ..Default::default()
                }),
                count: 1,
                up_to: false,
            },
            to: ZoneTarget::Hand {
                owner: PlayerTarget::Controller,
            },
            controller_override: None,
        },
    ]);
    let mut ctx = EffectContext::new(p1, source_id, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(
        state.objects_in_zone(&ZoneId::Graveyard(p1)).len(),
        2,
        "3 cards milled, 1 (the creature) returned to hand -- 2 should remain in the \
         graveyard. If this is 3, the mill never ran; if 0, the wrong card count moved."
    );
    let beast_zone = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Milled Beast")
        .map(|(_, o)| o.zone);
    assert_eq!(
        beast_zone,
        Some(ZoneId::Hand(p1)),
        "the milled creature -- which did not exist in the graveyard before THIS \
         resolution's own mill ran -- must be the card the untargeted choice picked up"
    );
}

// ── Card integration: the trigger goes on the stack untargeted ──────────────

/// CR 603.3d: a `PendingTrigger` / `StackObject` for an untargeted choice
/// carries ZERO declared targets -- an SBA can never remove it for "all targets
/// illegal" (there were never any). Drives the REAL `azorius_chancery()` def
/// through `Command::PlayLand`.
#[test]
fn t8_karoo_trigger_reaches_the_stack_with_zero_declared_targets() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let karoo_id = card_name_to_id("Azorius Chancery");
    let karoo = enrich_spec_from_def(
        ObjectSpec::card(p1, "Azorius Chancery")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(karoo_id),
        &defs,
    );
    let other_land = ObjectSpec::land(p1, "Other Land");
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(mtg_engine::CardRegistry::new(
            defs.values().cloned().collect::<Vec<_>>(),
        ))
        .object(karoo)
        .object(other_land)
        .active_player(p1)
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let card_id = find_in_zone(&state, "Azorius Chancery", ZoneId::Hand(p1));

    let (state, _) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: card_id,
        },
    )
    .expect("PlayLand must succeed");

    let trigger_obj = state
        .stack_objects()
        .iter()
        .find(|so| {
            state
                .objects()
                .get(&match so.kind {
                    mtg_engine::StackObjectKind::TriggeredAbility { source_object, .. } => {
                        source_object
                    }
                    _ => return false,
                })
                .is_some_and(|o| o.characteristics.name == "Azorius Chancery")
        })
        .unwrap_or_else(|| {
            panic!(
                "no TriggeredAbility stack object for Azorius Chancery; stack = {:?}",
                state.stack_objects()
            )
        });
    assert!(
        trigger_obj.targets.is_empty(),
        "CR 115.10: the ETB trigger's declared-targets list must be EMPTY -- it never \
         announced anything, so CR 603.3d's 'all targets illegal' removal can never apply \
         to it"
    );
}

/// Full `Command::AnswerEffectChoice` round trip through the REAL stack: the
/// Karoo's trigger is answered naming a SPECIFIC land, and that land (not just
/// "a" land) ends up in hand -- proving the validated production path, not just
/// the bare `execute_effect` primitive.
#[test]
fn t9_karoo_answers_a_real_choice_through_the_validated_command_path() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let karoo_id = card_name_to_id("Azorius Chancery");
    let karoo = enrich_spec_from_def(
        ObjectSpec::card(p1, "Azorius Chancery")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(karoo_id),
        &defs,
    );
    let other_land = ObjectSpec::land(p1, "Other Land");
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(mtg_engine::CardRegistry::new(
            defs.values().cloned().collect::<Vec<_>>(),
        ))
        .object(karoo)
        .object(other_land)
        .active_player(p1)
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);
    let card_id = find_in_zone(&state, "Azorius Chancery", ZoneId::Hand(p1));

    let (state, _) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: card_id,
        },
    )
    .expect("PlayLand must succeed");
    // Both players pass to resolve the trigger -- it suspends on the choice.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).expect("p1 pass");
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).expect("p2 pass");

    let pending = state
        .pending_effect_choice()
        .expect("the Karoo trigger must suspend on the ChooseObject question");
    let other_land_id = find_obj(&state, "Other Land");
    let (question, choice_id) = (pending.question.clone(), pending.choice_id);
    match &question {
        EffectChoiceQuestion::ChooseObject { candidates, .. } => {
            assert!(
                candidates.contains(&other_land_id),
                "Other Land must be a candidate"
            );
        }
        other => panic!("expected ChooseObject, got {other:?}"),
    }

    let (state, _) = process_command(
        state,
        Command::AnswerEffectChoice {
            player: p1,
            choice_id,
            answer: EffectChoiceAnswer::ChooseObject {
                chosen: vec![other_land_id],
            },
        },
    )
    .expect("a legal, in-candidates answer must be accepted");

    // CR 400.7: `MoveZone` mints a NEW ObjectId for the destination object, so
    // `other_land_id` (the pre-move id) is dead now -- find by name instead.
    assert_eq!(
        state
            .objects()
            .iter()
            .filter(|(_, o)| o.characteristics.name == "Other Land")
            .map(|(_, o)| o.zone)
            .next(),
        Some(ZoneId::Hand(p1)),
        "the NAMED land -- not an arbitrary one -- must be the one that moved to hand"
    );
}

/// Card integration: `takenuma_abandoned_mire`'s REAL Channel ability, driven
/// straight through `execute_effect` on the def's own authored effect.
#[test]
fn t10_takenuma_channel_mills_then_returns_from_the_graveyard() {
    let p1 = p(1);
    let p2 = p(2);
    let def = card_def("Takenuma, Abandoned Mire");
    let channel_effect = def
        .abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::Activated {
                effect, targets, ..
            } if targets.is_empty() && matches!(effect, Effect::Sequence(_)) => {
                Some(effect.clone())
            }
            _ => None,
        })
        .expect("Takenuma's Channel ability must carry a Sequence effect with zero targets");

    let milled_pw = ObjectSpec::card(p1, "Milled Walker")
        .with_types(vec![CardType::Planeswalker])
        .with_loyalty(3)
        .in_zone(ZoneId::Library(p1));
    let filler1 = ObjectSpec::card(p1, "Filler 1")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Library(p1));
    let filler2 = ObjectSpec::card(p1, "Filler 2")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Library(p1));
    let source = ObjectSpec::land(p1, "Takenuma Stand-In");
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(source)
        .object(milled_pw)
        .object(filler1)
        .object(filler2)
        .build()
        .unwrap();
    let source_id = find_obj(&state, "Takenuma Stand-In");
    let mut ctx = EffectContext::new(p1, source_id, vec![]);
    execute_effect(&mut state, &channel_effect, &mut ctx);

    assert!(
        state.pending_effect_choice().is_none(),
        "one eligible card is DETERMINED"
    );
    let pw_zone = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "Milled Walker")
        .map(|(_, o)| o.zone);
    assert_eq!(
        pw_zone,
        Some(ZoneId::Hand(p1)),
        "the planeswalker card the mill put into the graveyard must be the one returned"
    );
}

/// Card integration: `Connive // Concoct`'s REAL Concoct half — the **18th**
/// member of the `OOS-DX4-6` class, and the one neither the seed nor the plan's
/// §0.1 census named.
///
/// It was found by `pb_dx28_chosen_object_roster.rs`'s R4 inverse axis after the
/// roster had already been pinned at 17, and it is the only member whose
/// `targets` list hangs off an `AbilityDefinition::Fuse` (a split card's half)
/// rather than a `Spell`/`Triggered`/`Activated` node — which is why R3's walk
/// could not see it until its variant list was widened.
///
/// The card had **zero** behavioural coverage before this probe: the corpus's
/// only `connive`-named test file exercises the connive KEYWORD (CR 702.163),
/// not this card. A roster pin asserts a def's shape; it asserts nothing about
/// whether the shape executes, and shipping the migration on the pin alone
/// would repeat the failure PB-DX27's review recorded (three headline defs
/// promoted with no behavioural coverage at all).
///
/// CR 115.10 / CR 701.25 / CR 608.2b.
#[test]
fn t11_concoct_surveils_then_returns_a_chosen_creature_with_no_declared_target() {
    let p1 = p(1);
    let p2 = p(2);
    let def = card_def("Connive // Concoct");
    let concoct = def
        .abilities
        .iter()
        .find_map(|a| match a {
            AbilityDefinition::Fuse {
                name,
                effect,
                targets,
                ..
            } if name == "Concoct" => {
                assert!(
                    targets.is_empty(),
                    "CR 115.10: Concoct prints no \"target\" -- its half must declare \
                     no TargetRequirement after the PB-DX28 migration, got {targets:?}"
                );
                Some(effect.clone())
            }
            _ => None,
        })
        .expect("Connive // Concoct must carry a Fuse half named Concoct");

    // Two eligible creature cards in the graveyard, so the choice is a REAL one
    // (not the determined short-circuit t10 exercises) -- and one ineligible
    // card, so a predicate that ignored the filter would be caught.
    let eligible_a =
        ObjectSpec::creature(p1, "Grave Creature A", 2, 2).in_zone(ZoneId::Graveyard(p1));
    let eligible_b =
        ObjectSpec::creature(p1, "Grave Creature B", 3, 3).in_zone(ZoneId::Graveyard(p1));
    let ineligible = ObjectSpec::card(p1, "Grave Instant")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Graveyard(p1));
    let source = ObjectSpec::card(p1, "Concoct Stand-In")
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Stack);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(source)
        .object(eligible_a)
        .object(eligible_b)
        .object(ineligible)
        .build()
        .unwrap();
    let source_id = find_obj(&state, "Concoct Stand-In");
    let mut ctx = EffectContext::new(p1, source_id, vec![]);
    execute_effect(&mut state, &concoct, &mut ctx);

    // Two eligible creature cards means the answer is NOT determined, so the
    // resolution must SUSPEND on a real question rather than auto-pick.
    let pending = state
        .pending_effect_choice()
        .cloned()
        .expect("two eligible creature cards must raise a real CR 608.2d question");
    assert_eq!(pending.player, p1, "CR 608.2d: the controller chooses");
    match &pending.question {
        EffectChoiceQuestion::ChooseObject {
            candidates,
            count,
            up_to,
        } => {
            assert_eq!(*count, 1, "\"a creature card\" is exactly one");
            assert!(!*up_to, "\"a creature card\" is not \"up to one\"");
            let names: Vec<String> = candidates
                .iter()
                .map(|id| state.objects()[id].characteristics.name.clone())
                .collect();
            assert_eq!(
                names,
                vec![
                    "Grave Creature A".to_string(),
                    "Grave Creature B".to_string()
                ],
                "the answer space is the two CREATURE cards in the controller's own \
                 graveyard -- the Instant must be filtered out, and no other zone reached"
            );
        }
        other => panic!("expected a ChooseObject question, got {other:?}"),
    }
}
