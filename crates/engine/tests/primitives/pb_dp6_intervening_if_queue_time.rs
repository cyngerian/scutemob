//! PB-DP6 (DP-15, CR 603.4): intervening-if is now checked at QUEUE time, not
//! only at resolution.
//!
//! Before this batch, nearly every trigger-queue site read a card-def
//! `AbilityDefinition::Triggered { intervening_if: Option<Condition>, .. }` and
//! queued the trigger unconditionally, deferring the condition check to
//! resolution (`rules/resolution.rs` ~2139-2156). CR 603.4's first sentence --
//! "a triggered ability ... doesn't trigger unless the stated condition is true
//! at that time" -- was therefore only half-enforced: the false-POSITIVE
//! direction (queue a trigger that should never have triggered at all) was
//! live-wrong on every `AtBeginningOfYourUpkeep` / `AtBeginningOfYourEndStep` /
//! `AtBeginningOfCombat` / `AtBeginningOfFirstMainPhase` /
//! `AtBeginningOfPostcombatMain` card-def trigger in the corpus.
//!
//! This file exercises the shared queue-time gate
//! (`rules::abilities::carddef_intervening_if_holds_at_queue_time`, built on
//! the exhaustive `effects::condition_is_queue_time_evaluable` predicate) at
//! representative sites from the plan's 14-site roster
//! (`memory/primitives/pb-plan-DP6.md` §2/§4/§7). The resolution-time
//! re-check is retained (hard constraint 2) and T6 pins that directly.

use mtg_engine::cards::card_definition::TriggerZone;
use mtg_engine::effects::{condition_is_queue_time_evaluable, execute_effect, EffectContext};
use mtg_engine::rules::abilities::check_triggers;
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::rules::replacement::queue_carddef_etb_triggers;
use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition,
    CardEffectTarget, CardFace, CardId, CardRegistry, CardType, Command, Completeness, Condition,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor,
    ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TargetController, TargetFilter, TokenSpec,
    TriggerCondition, TypeLine, ZoneId, ZoneTarget,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn one_def_map(def: &CardDefinition) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

fn token_spec(name: &str) -> TokenSpec {
    TokenSpec {
        name: name.to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        power: 1,
        toughness: 1,
        count: EffectAmount::Fixed(1),
        ..Default::default()
    }
}

fn count_tokens(state: &GameState, name: &str) -> usize {
    state
        .objects()
        .values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.is_token && o.characteristics.name == name)
        .count()
}

/// A single-ability vanilla 2/2 creature with `ability` as its only ability.
fn conditional_creature_def(id: &str, name: &str, ability: AbilityDefinition) -> CardDefinition {
    CardDefinition {
        card_id: cid(id),
        name: name.to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![ability],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}

/// A card-def triggered ability with the given trigger condition and
/// intervening-if, creating a distinctly-named 1/1 token on resolution.
fn conditional_trigger_ability(
    trigger_condition: TriggerCondition,
    intervening_if: Option<Condition>,
    token_name: &str,
) -> AbilityDefinition {
    AbilityDefinition::Triggered {
        once_per_turn: false,
        trigger_condition,
        effect: Effect::CreateToken {
            spec: token_spec(token_name),
        },
        intervening_if,
        targets: vec![],
        modes: None,
        trigger_zone: None,
    }
}

/// Place `def` on the battlefield under `owner`'s control.
fn place_on_battlefield(owner: PlayerId, def: &CardDefinition) -> ObjectSpec {
    let defs = one_def_map(def);
    enrich_spec_from_def(
        ObjectSpec::card(owner, &def.name)
            .with_card_id(def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &defs,
    )
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (stuck at {:?}, wanted {:?})",
            state.turn().step,
            target
        );
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder at step {:?}", state.turn().step));
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
    }
}

fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

fn empty_cast_spell_data(player: PlayerId, card: ObjectId, kicker_times: u32) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times,
        alt_cost: None,
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        face_down_kind: None,
        additional_costs: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }))
}

// ── T1: ETB WasKicked gate uses the object's live kicker count ──────────────

/// CR 702.33d/e + 603.4: Nullpriest of Oblivion's ETB reanimation trigger has
/// `intervening_if: Some(Condition::WasKicked)`. Pre-fix, the ETB gate built
/// `EffectContext::new` (zero-filled `kicker_times_paid`), so `WasKicked`
/// always read false at queue time even for a genuinely kicked permanent --
/// the trigger never even reached the stack. Cast kicked (should queue) and
/// unkicked (should not queue) through the real cast/resolve path.
///
/// This asserts the full end-to-end reanimation, not just queuing: the
/// resolution-time re-check (`resolution.rs`'s `condition_holds` closure) had
/// its own, sibling zero-fill bug -- it also built `EffectContext::new`, so
/// even after the queue-time gate was fixed the trigger would queue and then
/// immediately fizzle at resolution, and GY Fodder would never actually
/// return to the battlefield. That resolution-time bug is fixed in the same
/// fix cycle as this test (PB-DP6 fix cycle finding 1), so both halves of CR
/// 603.4 now agree and the reanimation is observable end to end.
#[test]
fn test_dp6_etb_waskicked_gate_uses_object_kicker_count() {
    let nullpriest_def = all_cards()
        .into_iter()
        .find(|d| d.name == "Nullpriest of Oblivion")
        .expect("Nullpriest of Oblivion should be in the corpus");

    // -- Kicked: the ETB trigger should queue and reanimate. --
    {
        let p1 = p(1);
        let p2 = p(2);
        let registry = CardRegistry::new(vec![nullpriest_def.clone()]);
        let spell = ObjectSpec::card(p1, "Nullpriest of Oblivion")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(nullpriest_def.card_id.clone())
            .with_types(vec![CardType::Creature])
            .with_mana_cost(ManaCost {
                generic: 1,
                black: 1,
                ..Default::default()
            })
            .with_keyword(KeywordAbility::Kicker);
        let gy_creature =
            ObjectSpec::creature(p1, "GY Fodder", 1, 1).in_zone(ZoneId::Graveyard(p1));

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(spell)
            .object(gy_creature)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        // {1}{B} base + {3}{B} kicker = {4}{B}{B}.
        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Black, 2);
        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, 4);
        state.turn_mut().priority_holder = Some(p1);

        let spell_id = find_object(&state, "Nullpriest of Oblivion");
        let (state, _) = process_command(state, empty_cast_spell_data(p1, spell_id, 1))
            .unwrap_or_else(|e| panic!("cast kicked Nullpriest failed: {:?}", e));

        // Resolve the spell: the permanent enters and (post-fix) its ETB
        // trigger is queued and immediately flushed to the stack.
        let (state, _) = pass_all(state, &[p1, p2]);
        assert_eq!(
            state.stack_objects().len(),
            1,
            "CR 702.33d/PB-DP6: kicked Nullpriest's ETB trigger should be queued \
             (put on the stack) -- pre-fix, EffectContext::new zero-filled \
             kicker_times_paid so WasKicked always read false here"
        );
        // The auto-selected target must be GY Fodder (the only legal candidate) --
        // confirms the queued trigger is well-formed, not merely present.
        let target_is_gy_fodder = match state.stack_objects()[0].targets.first() {
            Some(mtg_engine::SpellTarget {
                target: mtg_engine::Target::Object(id),
                ..
            }) => state
                .objects()
                .get(id)
                .map(|o| o.characteristics.name == "GY Fodder")
                .unwrap_or(false),
            _ => false,
        };
        assert!(
            target_is_gy_fodder,
            "the queued trigger's auto-selected target should be GY Fodder"
        );

        // CR 702.33d/603.4 (fix cycle finding 1): resolve the stack and confirm
        // GY Fodder actually returns to the battlefield -- the resolution-time
        // re-check must agree with the queue-time gate that this permanent was
        // kicked, not fizzle the ability on a zero-filled kicker count.
        let state = resolve_stack(state, &[p1, p2]);
        assert!(
            state
                .objects()
                .values()
                .any(|o| o.characteristics.name == "GY Fodder" && o.zone == ZoneId::Battlefield),
            "CR 702.33d: kicked Nullpriest's reanimation effect should actually \
             return GY Fodder to the battlefield -- pre-fix, the resolution-time \
             re-check's zero-filled EffectContext::new made WasKicked read false \
             and fizzled the ability even after the queue-time gate let it \
             through"
        );
        assert!(
            !state
                .objects()
                .values()
                .any(|o| o.characteristics.name == "GY Fodder" && o.zone == ZoneId::Graveyard(p1)),
            "GY Fodder should have left the graveyard"
        );
    }

    // -- Unkicked: the ETB trigger should NOT queue. --
    {
        let p1 = p(1);
        let p2 = p(2);
        let registry = CardRegistry::new(vec![nullpriest_def.clone()]);
        let spell = ObjectSpec::card(p1, "Nullpriest of Oblivion")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(nullpriest_def.card_id.clone())
            .with_types(vec![CardType::Creature])
            .with_mana_cost(ManaCost {
                generic: 1,
                black: 1,
                ..Default::default()
            })
            .with_keyword(KeywordAbility::Kicker);
        let gy_creature =
            ObjectSpec::creature(p1, "GY Fodder", 1, 1).in_zone(ZoneId::Graveyard(p1));

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(spell)
            .object(gy_creature)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Black, 1);
        state
            .players_mut()
            .get_mut(&p1)
            .unwrap()
            .mana_pool
            .add(ManaColor::Colorless, 1);
        state.turn_mut().priority_holder = Some(p1);

        let spell_id = find_object(&state, "Nullpriest of Oblivion");
        let (state, _) = process_command(state, empty_cast_spell_data(p1, spell_id, 0))
            .unwrap_or_else(|e| panic!("cast unkicked Nullpriest failed: {:?}", e));

        let (state, _) = pass_all(state, &[p1, p2]);
        assert_eq!(
            state.stack_objects().len(),
            0,
            "CR 603.4/702.33d: unkicked Nullpriest's ETB trigger must not queue at all"
        );
        assert!(
            state
                .objects()
                .values()
                .any(|o| o.characteristics.name == "GY Fodder" && o.zone == ZoneId::Graveyard(p1)),
            "GY Fodder should remain in the graveyard when Nullpriest is not kicked"
        );
    }
}

// ── T2/T3: upkeep sweep gate (Land-Tax-shaped) ───────────────────────────────

/// CR 603.4 (A1): a Land-Tax-shaped upkeep trigger with
/// `Condition::OpponentControlsMoreLandsThanYou` false must not be queued at
/// all -- the stack must never contain it, and no token results.
#[test]
fn test_dp6_upkeep_trigger_not_queued_when_condition_false() {
    let p1 = p(1);
    let p2 = p(2);
    let def = conditional_creature_def(
        "mock-dp6-upkeep-false",
        "Mock DP6 Upkeep False",
        conditional_trigger_ability(
            TriggerCondition::AtBeginningOfYourUpkeep,
            Some(Condition::OpponentControlsMoreLandsThanYou),
            "DP6UpkeepToken",
        ),
    );
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(place_on_battlefield(p1, &def))
        .object(ObjectSpec::land(p1, "P1 Land"))
        .object(ObjectSpec::land(p2, "P2 Land"))
        .active_player(p1)
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let state = advance_to_step(state, Step::Upkeep);
    assert!(
        state
            .stack_objects()
            .iter()
            .all(|so| !matches!(so.kind, mtg_engine::state::StackObjectKind::TriggeredAbility { source_object, .. } if state.objects().get(&source_object).map(|o| o.characteristics.name == def.name).unwrap_or(false))),
        "CR 603.4: equal land counts (condition false) must never queue the upkeep trigger"
    );
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        count_tokens(&state, "DP6UpkeepToken"),
        0,
        "CR 603.4: no token should ever be created when the intervening-if is false at queue time"
    );
}

/// CR 603.4 (A1), non-regression: the same shape with the condition TRUE must
/// still queue and resolve normally.
#[test]
fn test_dp6_upkeep_trigger_queued_when_condition_true() {
    let p1 = p(1);
    let p2 = p(2);
    let def = conditional_creature_def(
        "mock-dp6-upkeep-true",
        "Mock DP6 Upkeep True",
        conditional_trigger_ability(
            TriggerCondition::AtBeginningOfYourUpkeep,
            Some(Condition::OpponentControlsMoreLandsThanYou),
            "DP6UpkeepToken2",
        ),
    );
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(place_on_battlefield(p1, &def))
        .object(ObjectSpec::land(p1, "P1 Land"))
        .object(ObjectSpec::land(p2, "P2 Land A"))
        .object(ObjectSpec::land(p2, "P2 Land B"))
        .active_player(p1)
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let state = advance_to_step(state, Step::Upkeep);
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        count_tokens(&state, "DP6UpkeepToken2"),
        1,
        "CR 603.4: an opponent controlling more lands should queue and resolve the trigger"
    );
}

// ── T4: end step gate (Searslicer-shaped) ────────────────────────────────────

/// CR 603.4 (A4): an end-step trigger with `Condition::YouAttackedThisTurn`
/// false (no attack was declared) must not be queued at all.
#[test]
fn test_dp6_end_step_trigger_not_queued_when_condition_false() {
    let p1 = p(1);
    let p2 = p(2);
    let def = conditional_creature_def(
        "mock-dp6-endstep",
        "Mock DP6 End Step",
        conditional_trigger_ability(
            TriggerCondition::AtBeginningOfYourEndStep,
            Some(Condition::YouAttackedThisTurn),
            "DP6EndStepToken",
        ),
    );
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(place_on_battlefield(p1, &def))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    // No attackers declared -- `advance_step` auto-skips DeclareBlockers/
    // CombatDamage straight to EndOfCombat, then on to EndStep.
    let state = advance_to_step(state, Step::End);
    // Check QUEUING directly, right at the step transition, before resolving
    // anything. This is load-bearing: the pre-existing resolution-time
    // re-check (retained, unmodified by this PB) would ALSO catch a false
    // `YouAttackedThisTurn` and produce 0 tokens even if the trigger had been
    // wrongly queued -- so a final-token-count-only assertion cannot tell
    // "never queued" apart from "queued, then fizzled at resolution" and
    // would silently pass both pre-fix and post-fix.
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: with no attack declared this turn, the end-step trigger must \
         not even be queued (not merely fizzle at resolution)"
    );
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        count_tokens(&state, "DP6EndStepToken"),
        0,
        "CR 603.4: with no attack declared this turn, no token should ever be created"
    );
}

// ── T5: begin-combat gate (Loyal-Apprentice-shaped, no commander) ──────────

/// CR 603.4 (A5): a begin-combat trigger with
/// `Condition::YouControlYourCommander` false (no commander registered at
/// all) must not be queued.
#[test]
fn test_dp6_begin_combat_trigger_not_queued_without_commander() {
    let p1 = p(1);
    let p2 = p(2);
    let def = conditional_creature_def(
        "mock-dp6-combat-no-cmdr",
        "Mock DP6 Combat No Commander",
        conditional_trigger_ability(
            TriggerCondition::AtBeginningOfCombat,
            Some(Condition::YouControlYourCommander),
            "DP6CombatToken",
        ),
    );
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(place_on_battlefield(p1, &def))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let state = advance_to_step(state, Step::BeginningOfCombat);
    // Check QUEUING directly (see T4's comment for why a final-token-count-only
    // assertion would silently pass both pre-fix and post-fix here too: the
    // pre-existing, retained resolution-time re-check would already catch a
    // false YouControlYourCommander and produce 0 tokens even if the trigger
    // had been wrongly queued).
    assert!(
        state.stack_objects().is_empty(),
        "CR 903.3d/603.4: with no commander at all, YouControlYourCommander is \
         false -- the trigger must not even be queued at the beginning of combat"
    );
    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        count_tokens(&state, "DP6CombatToken"),
        0,
        "CR 903.3d/603.4: no token should ever be created"
    );
}

// ── T6: resolution-time re-check retained (hard constraint 2 pin) ──────────

/// CR 603.4 (both sentences): the commander is present when the trigger is
/// QUEUED (true) but removed before it RESOLVES. The queue-time gate must let
/// it through; the resolution-time re-check (untouched by PB-DP6) must still
/// catch the now-false condition and produce no effect.
#[test]
fn test_dp6_resolution_recheck_retained() {
    let p1 = p(1);
    let p2 = p(2);
    let commander_cid = cid("mock-dp6-t6-commander");

    let def = conditional_creature_def(
        "mock-dp6-combat-recheck",
        "Mock DP6 Combat Recheck",
        conditional_trigger_ability(
            TriggerCondition::AtBeginningOfCombat,
            Some(Condition::YouControlYourCommander),
            "DP6RecheckToken",
        ),
    );
    let commander_def = CardDefinition {
        card_id: commander_cid.clone(),
        name: "Mock DP6 T6 Commander".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(3),
        toughness: Some(3),
        completeness: Completeness::Complete,
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![def.clone(), commander_def.clone()]);
    let commander_obj = ObjectSpec::creature(p1, "Mock DP6 T6 Commander", 3, 3)
        .with_card_id(commander_cid.clone())
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .player_commander(p1, commander_cid)
        .object(place_on_battlefield(p1, &def))
        .object(commander_obj)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    // Commander present -- the trigger must queue.
    let state = advance_to_step(state, Step::BeginningOfCombat);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "CR 603.4: with the commander controlled at queue time, the trigger should be on the stack"
    );

    let mut state = state;
    let commander_id = find_object(&state, "Mock DP6 T6 Commander");
    execute_effect(
        &mut state,
        &Effect::DestroyPermanent {
            target: CardEffectTarget::Source,
            cant_be_regenerated: false,
        },
        &mut EffectContext::new(p1, commander_id, vec![]),
    );

    let state = resolve_stack(state, &[p1, p2]);
    assert_eq!(
        count_tokens(&state, "DP6RecheckToken"),
        0,
        "CR 603.4: the commander was removed before the trigger resolved -- the \
         RETAINED resolution-time re-check should fail and no token should be created"
    );
}

// ── T7: first-main and postcombat-main sweeps gate (A2/A3) ─────────────────

/// CR 603.4 (A2/A3): both the first-main-phase and postcombat-main-phase
/// card-def sweeps must respect a false intervening-if.
#[test]
fn test_dp6_first_main_and_postcombat_main_gates() {
    let p1 = p(1);
    let p2 = p(2);
    let first_main_def = conditional_creature_def(
        "mock-dp6-first-main",
        "Mock DP6 First Main",
        conditional_trigger_ability(
            TriggerCondition::AtBeginningOfFirstMainPhase,
            Some(Condition::OpponentControlsMoreLandsThanYou),
            "DP6FirstMainToken",
        ),
    );
    let postcombat_def = conditional_creature_def(
        "mock-dp6-postcombat-main",
        "Mock DP6 Postcombat Main",
        conditional_trigger_ability(
            TriggerCondition::AtBeginningOfPostcombatMain,
            Some(Condition::OpponentControlsMoreLandsThanYou),
            "DP6PostcombatToken",
        ),
    );
    let registry = CardRegistry::new(vec![first_main_def.clone(), postcombat_def.clone()]);
    let mut b = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(place_on_battlefield(p1, &first_main_def))
        .object(place_on_battlefield(p1, &postcombat_def))
        .object(ObjectSpec::land(p1, "P1 Land"))
        .object(ObjectSpec::land(p2, "P2 Land"));
    // Advancing all the way to PostCombatMain crosses the Draw step; give both
    // players a small library so drawing does not deck them out mid-traversal.
    for i in 0..3 {
        b = b.object(
            ObjectSpec::creature(p1, &format!("P1 Library {i}"), 1, 1).in_zone(ZoneId::Library(p1)),
        );
        b = b.object(
            ObjectSpec::creature(p2, &format!("P2 Library {i}"), 1, 1).in_zone(ZoneId::Library(p2)),
        );
    }
    let mut state = b.active_player(p1).at_step(Step::Untap).build().unwrap();
    state.turn_mut().priority_holder = Some(p1);

    // Check QUEUING directly at each step transition -- a final-token-count-only
    // assertion would silently pass both pre-fix and post-fix here, because the
    // pre-existing, retained resolution-time re-check already catches a false
    // `OpponentControlsMoreLandsThanYou` and produces 0 tokens even if the
    // trigger had been wrongly queued (see T4's comment for the full argument).
    let state = advance_to_step(state, Step::PreCombatMain);
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: equal land counts must not even queue the first-main-phase trigger"
    );
    let state = advance_to_step(state, Step::PostCombatMain);
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.4: equal land counts must not even queue the postcombat-main-phase trigger"
    );
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        count_tokens(&state, "DP6FirstMainToken"),
        0,
        "CR 603.4: no token should ever be created from the first-main-phase trigger"
    );
    assert_eq!(
        count_tokens(&state, "DP6PostcombatToken"),
        0,
        "CR 603.4: equal land counts must never queue the postcombat-main-phase trigger"
    );
}

// ── T8: an unanswerable condition must NOT suppress the trigger ────────────

/// CR 603.4 hard constraint 3: `Condition::TargetIsLegal` cannot be
/// meaningfully answered at queue time (no targets exist yet). The gate must
/// default to `true` -- the trigger is still queued (put on the stack) -- even
/// though it may separately fizzle at resolution for unrelated reasons.
#[test]
fn test_dp6_unevaluable_condition_does_not_suppress() {
    let p1 = p(1);
    let p2 = p(2);
    let def = conditional_creature_def(
        "mock-dp6-unevaluable",
        "Mock DP6 Unevaluable",
        conditional_trigger_ability(
            TriggerCondition::AtBeginningOfYourUpkeep,
            Some(Condition::TargetIsLegal { index: 0 }),
            "DP6UnevaluableToken",
        ),
    );
    let registry = CardRegistry::new(vec![def.clone()]);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(place_on_battlefield(p1, &def))
        .active_player(p1)
        .at_step(Step::Untap)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let state = advance_to_step(state, Step::Upkeep);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "PB-DP6 hard constraint 3: an unanswerable condition (TargetIsLegal with \
         no targets yet) must default to `true` and still be queued -- \
         suppressing it would be worse than the pre-existing over-fire"
    );
}

// ── T9: graveyard-zone sweep is behaviour-neutral (A14 refactor) ───────────

/// CR 603.3 / `TriggerZone::Graveyard`: a Bloodghast-shaped Landfall trigger
/// dispatched from the graveyard must still respect its `intervening_if`,
/// both before and after the A14 refactor onto the shared helper.
#[test]
fn test_dp6_graveyard_gate_unchanged() {
    let p1 = p(1);
    let p2 = p(2);

    let graveyard_def = |id: &str, name: &str, intervening_if: Option<Condition>| CardDefinition {
        card_id: cid(id),
        name: name.to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                filter: Some(TargetFilter {
                    has_card_type: Some(CardType::Land),
                    controller: TargetController::You,
                    ..Default::default()
                }),
                exclude_self: false,
            },
            effect: Effect::MoveZone {
                target: CardEffectTarget::Source,
                to: ZoneTarget::Battlefield { tapped: false },
                controller_override: None,
            },
            intervening_if,
            targets: vec![],
            modes: None,
            trigger_zone: Some(TriggerZone::Graveyard),
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    };

    // -- False condition: must not fire. --
    {
        let def = graveyard_def(
            "mock-dp6-gy-false",
            "Mock DP6 GY False",
            Some(Condition::OpponentControlsMoreLandsThanYou),
        );
        let card_id = def.card_id.clone();
        let spec = enrich_spec_from_def(
            ObjectSpec::card(p1, &def.name)
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(card_id),
            &one_def_map(&def),
        );
        let p1_land = ObjectSpec::land(p1, "GY Test Forest").in_zone(ZoneId::Battlefield);
        let p2_land = ObjectSpec::land(p2, "GY Test Island").in_zone(ZoneId::Battlefield);

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(vec![def.clone()]))
            .object(spec)
            .object(p1_land)
            .object(p2_land)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let source_id = find_object(&state, "Mock DP6 GY False");
        let land_id = find_object(&state, "GY Test Forest");
        let events = vec![GameEvent::PermanentEnteredBattlefield {
            object_id: land_id,
            player: p1,
        }];
        let triggers = check_triggers(&state, &events);
        assert!(
            triggers.iter().all(|t| t.source != source_id),
            "CR 603.4: equal land counts must not fire the graveyard Landfall trigger"
        );
    }

    // -- True condition: must fire. --
    {
        let def = graveyard_def(
            "mock-dp6-gy-true",
            "Mock DP6 GY True",
            Some(Condition::OpponentControlsMoreLandsThanYou),
        );
        let card_id = def.card_id.clone();
        let spec = enrich_spec_from_def(
            ObjectSpec::card(p1, &def.name)
                .in_zone(ZoneId::Graveyard(p1))
                .with_card_id(card_id),
            &one_def_map(&def),
        );
        let p1_land = ObjectSpec::land(p1, "GY Test Forest 2").in_zone(ZoneId::Battlefield);
        let p2_land_a = ObjectSpec::land(p2, "GY Test Island A").in_zone(ZoneId::Battlefield);
        let p2_land_b = ObjectSpec::land(p2, "GY Test Island B").in_zone(ZoneId::Battlefield);

        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(vec![def.clone()]))
            .object(spec)
            .object(p1_land)
            .object(p2_land_a)
            .object(p2_land_b)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let source_id = find_object(&state, "Mock DP6 GY True");
        let land_id = find_object(&state, "GY Test Forest 2");
        let events = vec![GameEvent::PermanentEnteredBattlefield {
            object_id: land_id,
            player: p1,
        }];
        let triggers = check_triggers(&state, &events);
        assert!(
            triggers.iter().any(|t| t.source == source_id),
            "CR 603.4: an opponent controlling more lands must fire the graveyard Landfall trigger"
        );
    }
}

// ── T10: TributeNotPaid AND-in ──────────────────────────────────────────────

/// CR 702.104b / 603.4 (A8): `TributeNotPaid`'s hardcoded gate is ANDed with
/// the def's own `intervening_if`. Neither check alone should be sufficient.
#[test]
fn test_dp6_tribute_not_paid_respects_intervening_if() {
    let tribute_def = |id: &str, name: &str, intervening_if: Option<Condition>| CardDefinition {
        card_id: cid(id),
        name: name.to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::TributeNotPaid,
            effect: Effect::CreateToken {
                spec: token_spec("DP6TributeToken"),
            },
            intervening_if,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    };

    // -- Tribute not paid, but the def's own intervening-if is false. --
    {
        let p1 = p(1);
        let p2 = p(2);
        let def = tribute_def(
            "mock-dp6-tribute-false",
            "Mock DP6 Tribute False",
            Some(Condition::OpponentControlsMoreLandsThanYou),
        );
        let registry = CardRegistry::new(vec![def.clone()]);
        let spec = place_on_battlefield(p1, &def);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(spec)
            .object(ObjectSpec::land(p1, "Tribute P1 Land"))
            .object(ObjectSpec::land(p2, "Tribute P2 Land"))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let obj_id = find_object(&state, &def.name);
        // tribute_was_paid defaults false (CR 702.104b).
        let card_id = def.card_id.clone();
        let registry_arc = state.card_registry().clone();
        queue_carddef_etb_triggers(&mut state, obj_id, p1, Some(&card_id), &registry_arc);
        assert!(
            state.pending_triggers().iter().all(|t| t.source != obj_id),
            "CR 603.4: tribute was not paid but the def's own intervening-if is \
             false -- the trigger must still not queue"
        );
    }

    // -- Tribute not paid, and the def's own intervening-if is true. --
    {
        let p1 = p(1);
        let p2 = p(2);
        let def = tribute_def(
            "mock-dp6-tribute-true",
            "Mock DP6 Tribute True",
            Some(Condition::OpponentControlsMoreLandsThanYou),
        );
        let registry = CardRegistry::new(vec![def.clone()]);
        let spec = place_on_battlefield(p1, &def);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(spec)
            .object(ObjectSpec::land(p1, "Tribute P1 Land 2"))
            .object(ObjectSpec::land(p2, "Tribute P2 Land A"))
            .object(ObjectSpec::land(p2, "Tribute P2 Land B"))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let obj_id = find_object(&state, &def.name);
        let card_id = def.card_id.clone();
        let registry_arc = state.card_registry().clone();
        queue_carddef_etb_triggers(&mut state, obj_id, p1, Some(&card_id), &registry_arc);
        assert!(
            state.pending_triggers().iter().any(|t| t.source == obj_id),
            "CR 603.4: tribute was not paid AND the def's own intervening-if is \
             true -- the trigger should queue"
        );
    }
}

// ── T11: face-aware gate reads the currently-visible face's condition ─────

/// CR 712.8d/e (PB-OS4b/PB-RS4 contract) + CR 603.4: a transformed
/// permanent's upkeep trigger must be gated by the BACK face's condition, not
/// the front face's -- inherited automatically because the gate reads
/// `intervening_if` from whichever `effective_abilities(is_transformed)` list
/// the caller already walked (§3.2's doc comment).
#[test]
fn test_dp6_face_aware_gate_reads_back_face_condition() {
    let p1 = p(1);
    let p2 = p(2);

    let def = CardDefinition {
        card_id: cid("mock-dp6-face"),
        name: "Mock DP6 Face Front".to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        // Front face's condition is unconditionally true (`None`). If the
        // sweep wrongly read the front face while `is_transformed`, this
        // would fire and create FrontToken.
        abilities: vec![conditional_trigger_ability(
            TriggerCondition::AtBeginningOfYourUpkeep,
            None,
            "DP6FrontToken",
        )],
        back_face: Some(CardFace {
            name: "Mock DP6 Face Back".to_string(),
            mana_cost: None,
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                ..Default::default()
            },
            oracle_text: String::new(),
            abilities: vec![conditional_trigger_ability(
                TriggerCondition::AtBeginningOfYourUpkeep,
                Some(Condition::OpponentControlsMoreLandsThanYou),
                "DP6BackToken",
            )],
            power: Some(3),
            toughness: Some(3),
            color_indicator: None,
        }),
        completeness: Completeness::Complete,
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![def.clone()]);
    let spec = place_on_battlefield(p1, &def);
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .object(ObjectSpec::land(p1, "Face P1 Land"))
        .object(ObjectSpec::land(p2, "Face P2 Land"))
        .active_player(p1)
        .at_step(Step::Untap)
        .build()
        .unwrap();

    let obj_id = find_object(&state, "Mock DP6 Face Front");
    state.objects_mut().get_mut(&obj_id).unwrap().is_transformed = true;
    state.turn_mut().priority_holder = Some(p1);

    let state = advance_to_step(state, Step::Upkeep);
    // Check QUEUING directly, before anything resolves. A post-resolution
    // token count cannot distinguish "the back face's own condition gated at
    // queue time" from "queued anyway, then fizzled at the pre-existing,
    // retained resolution-time re-check" -- both land on 0 tokens either way
    // (see T4's comment for the full argument; this is the same pattern).
    assert!(
        state.stack_objects().is_empty(),
        "CR 712.8d/e + 603.4: the back face's own false condition must gate at \
         QUEUE time -- nothing may reach the stack (a post-resolution token \
         count cannot distinguish this from a resolution-time fizzle)"
    );
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        count_tokens(&state, "DP6FrontToken"),
        0,
        "the permanent is showing its back face -- the front face's (always-true) \
         ability must never even be considered"
    );
    assert_eq!(
        count_tokens(&state, "DP6BackToken"),
        0,
        "the back face's OWN condition (equal land counts) should gate its trigger \
         at queue time -- proves intervening_if came from the same effective_abilities \
         list the caller already selected, not a mismatched one"
    );
}

// ── T12: condition_is_queue_time_evaluable is exhaustive and correct ──────

/// CR 603.4 hard constraint 3: pure unit test over the predicate. The 7
/// documented `false` variants, `Not`/`And`/`Or` propagation, and one
/// representative state-only `true` variant.
#[test]
fn test_dp6_condition_evaluability_predicate_is_exhaustive() {
    assert!(!condition_is_queue_time_evaluable(
        &Condition::TargetIsLegal { index: 0 }
    ));
    assert!(!condition_is_queue_time_evaluable(
        &Condition::WasOverloaded
    ));
    assert!(!condition_is_queue_time_evaluable(&Condition::WasBargained));
    assert!(!condition_is_queue_time_evaluable(&Condition::WasCleaved));
    assert!(!condition_is_queue_time_evaluable(
        &Condition::EvidenceWasCollected
    ));
    assert!(!condition_is_queue_time_evaluable(&Condition::GiftWasGiven));
    assert!(!condition_is_queue_time_evaluable(
        &Condition::SacrificeFired
    ));

    // Not/And/Or propagate: one unanswerable arm makes the whole clause
    // unanswerable.
    assert!(!condition_is_queue_time_evaluable(&Condition::Not(
        Box::new(Condition::WasOverloaded)
    )));
    assert!(condition_is_queue_time_evaluable(&Condition::Not(
        Box::new(Condition::Always)
    )));
    assert!(!condition_is_queue_time_evaluable(&Condition::And(
        Box::new(Condition::Always),
        Box::new(Condition::SacrificeFired),
    )));
    assert!(condition_is_queue_time_evaluable(&Condition::And(
        Box::new(Condition::Always),
        Box::new(Condition::WasKicked),
    )));
    assert!(!condition_is_queue_time_evaluable(&Condition::Or(
        Box::new(Condition::Always),
        Box::new(Condition::WasBargained),
    )));
    assert!(condition_is_queue_time_evaluable(&Condition::Or(
        Box::new(Condition::Always),
        Box::new(Condition::WasKicked),
    )));

    // A representative state-only variant answers true.
    assert!(condition_is_queue_time_evaluable(
        &Condition::YouControlYourCommander
    ));
}
