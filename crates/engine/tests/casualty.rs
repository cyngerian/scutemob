//! Casualty keyword ability tests (CR 702.153).
//!
//! Casualty N is an optional additional cost keyword that allows copying a spell:
//! "As an additional cost to cast this spell, you may sacrifice a creature with power
//! N or greater. When you cast this spell, if a casualty cost was paid for it, copy it."
//! (CR 702.153a)
//!
//! Key rules verified:
//! - Casualty is optional -- the spell may be cast without sacrifice (CR 702.153a).
//! - The sacrificed creature must have power >= N (CR 702.153a).
//! - Only the caster's own creatures may be sacrificed.
//! - Non-creatures are rejected (unlike Bargain which accepts artifacts/enchantments).
//! - When paid, a CasualtyTrigger appears on the stack; resolving it produces one copy.
//! - The copy has `is_copy: true` and does NOT increment `spells_cast_this_turn` (ruling 2022-04-29).
//! - Providing a sacrifice for a spell without the Casualty keyword is rejected.
//! - A creature with power > N is accepted (power >= N, not power == N).

use mtg_engine::cards::card_definition::{EffectAmount, PlayerTarget};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId,
    Step, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_in_zone(
    state: &mtg_engine::GameState,
    name: &str,
    zone: ZoneId,
) -> Option<mtg_engine::ObjectId> {
    state.objects.iter().find_map(|(&id, obj)| {
        if obj.characteristics.name == name && obj.zone == zone {
            Some(id)
        } else {
            None
        }
    })
}

/// Pass priority for all listed players once (round-robin, one pass each).
fn pass_all(
    state: mtg_engine::GameState,
    players: &[PlayerId],
) -> (mtg_engine::GameState, Vec<GameEvent>) {
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

// ── Card definitions ──────────────────────────────────────────────────────────

/// Synthetic Casualty 1 instant: deal 2 damage to target player.
/// Using GainLife (controller gains life) to avoid target requirements.
///
/// Mana cost: {R}. Casualty 1 optional additional cost: sacrifice creature with power >= 1.
fn casualty_bolt_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("casualty-bolt".to_string()),
        name: "Casualty Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Casualty 1 (As an additional cost to cast this spell, you may sacrifice a creature with power 1 or greater. If you do, copy this spell.)\nYou gain 2 life."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Casualty(1)),
            AbilityDefinition::Spell {
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic Casualty 2 instant -- requires power >= 2.
fn casualty2_bolt_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("casualty2-bolt".to_string()),
        name: "Casualty2 Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Casualty 2\nYou gain 2 life.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Casualty(2)),
            AbilityDefinition::Spell {
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic spell without Casualty -- used to verify providing a sacrifice for
/// a non-casualty spell is rejected.
fn plain_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-instant".to_string()),
        name: "Plain Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "You gain 1 life.".to_string(),
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
    }
}

/// Helper: build a 2-player state with the Casualty Bolt (N=1) in p1's hand.
/// `extra_objects` are additional objects to place.
fn setup_casualty_state(
    extra_objects: Vec<ObjectSpec>,
) -> (
    mtg_engine::GameState,
    PlayerId,
    PlayerId,
    mtg_engine::ObjectId,
) {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![casualty_bolt_def()]);

    let spell = ObjectSpec::card(p1, "Casualty Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("casualty-bolt".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Casualty(1));

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain);

    for obj in extra_objects {
        builder = builder.object(obj);
    }

    let mut state = builder.build().unwrap();

    // Add {R} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Casualty Bolt");
    (state, p1, p2, spell_id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// CR 702.153a — Casualty 1: Sacrifice a 1/1 creature. Verify:
/// (a) creature goes to graveyard,
/// (b) CasualtyTrigger appears on stack above the original spell,
/// (c) resolving the trigger creates a copy on the stack,
/// (d) copy resolves before original (LIFO order).
#[test]
fn test_casualty_basic_copy() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Soldier Token", 1, 1).token();
    let (state, _p1, _p2, spell_id) = setup_casualty_state(vec![creature]);

    let creature_id = find_object(&state, "Soldier Token");
    let initial_life = state.players[&p1].life_total;

    // Cast Casualty Bolt while sacrificing the 1/1 soldier token.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(creature_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with Casualty 1 (1/1 token) failed: {:?}", e));

    // CR 702.153a: The creature must have been sacrificed -- no longer on battlefield.
    let on_bf = find_object_in_zone(&state, "Soldier Token", ZoneId::Battlefield);
    assert!(
        on_bf.is_none(),
        "CR 702.153a: creature sacrificed as casualty cost must leave the battlefield"
    );

    // The stack should have 2 objects: the trigger on top, then the spell below.
    // (Trigger is pushed after the spell, so it resolves first via LIFO.)
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.153a: after paying casualty cost, stack should have the spell + the casualty trigger"
    );

    // The original spell should be marked was_casualty_paid = true.
    let original = state.stack_objects.iter().find(|o| o.was_casualty_paid);
    assert!(
        original.is_some(),
        "CR 702.153a: original spell's was_casualty_paid should be true after paying casualty cost"
    );

    // The top of stack should be the CasualtyTrigger.
    use mtg_engine::StackObjectKind;
    let top = state
        .stack_objects
        .back()
        .expect("stack must have at least 1 object");
    assert!(
        matches!(top.kind, StackObjectKind::CasualtyTrigger { .. }),
        "CR 702.153a: CasualtyTrigger should be on top of stack after paying casualty cost"
    );

    // Both players pass priority -- the trigger resolves, creating a copy.
    let (state, _) = pass_all(state, &[p1, p(2)]);

    // After trigger resolves, the stack should now have 2 items: the copy + the original.
    // (Copy was pushed on top when trigger resolved, then the trigger was consumed.)
    // Actually: trigger resolves -> copy pushed -> stack = [original, copy]; copy is on top.
    assert!(
        state.stack_objects.len() >= 2,
        "CR 702.153a: after CasualtyTrigger resolves, both copy and original spell should be on stack"
    );

    // The copy should have is_copy = true.
    let copy_obj = state.stack_objects.iter().find(|o| o.is_copy).expect(
        "CR 702.153a: a copy spell (is_copy=true) should be on stack after trigger resolves",
    );

    assert!(
        !copy_obj.was_casualty_paid,
        "Copy should not have was_casualty_paid set (copies don't pay costs)"
    );

    // Resolve the copy: both players pass priority.
    let (state, _) = pass_all(state, &[p1, p(2)]);
    // Copy resolves -> gain 2 life.
    let life_after_copy = state.players[&p1].life_total;
    assert_eq!(
        life_after_copy,
        initial_life + 2,
        "CR 702.153a: copy should resolve and gain life before original"
    );

    // Resolve the original: both players pass priority.
    let (state, _) = pass_all(state, &[p1, p(2)]);
    let life_after_original = state.players[&p1].life_total;
    assert_eq!(
        life_after_original,
        initial_life + 4,
        "CR 702.153a: original spell should also resolve and gain life (LIFO order)"
    );
}

/// CR 702.153a — Casualty power threshold: Sacrificing a creature with power < N is rejected;
/// sacrificing a creature with power == N is accepted.
#[test]
fn test_casualty_power_threshold() {
    let p1 = p(1);
    let p2 = p(2);

    let weak_creature = ObjectSpec::creature(p1, "Weak Creature", 1, 1);
    let strong_creature = ObjectSpec::creature(p1, "Strong Creature", 2, 2);

    let registry = CardRegistry::new(vec![casualty2_bolt_def()]);

    let spell = ObjectSpec::card(p1, "Casualty2 Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("casualty2-bolt".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Casualty(2));

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(weak_creature)
        .object(strong_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Casualty2 Bolt");
    let weak_id = find_object(&state, "Weak Creature");
    let strong_id = find_object(&state, "Strong Creature");

    // Attempt to sacrifice a creature with power 1 for Casualty 2 -- must be rejected.
    let result = process_command(
        state.clone(),
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(weak_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_err(),
        "CR 702.153a: sacrificing a creature with power < N for Casualty N must be rejected"
    );

    // Sacrificing a creature with power 2 (== N) should be accepted.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(strong_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );
    assert!(
        result.is_ok(),
        "CR 702.153a: sacrificing a creature with power == N for Casualty N must be accepted"
    );
}

/// CR 702.153a — Casualty is optional: casting without sacrifice is valid.
/// The spell resolves normally with no copy produced.
#[test]
fn test_casualty_optional_no_sacrifice() {
    let p1 = p(1);
    let (state, _p1, _p2, spell_id) = setup_casualty_state(vec![]);
    let initial_life = state.players[&p1].life_total;

    // Cast without providing a sacrifice.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell without casualty should succeed: {:?}", e));

    // Only the spell should be on the stack -- no trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.153a: no CasualtyTrigger when sacrifice was not paid"
    );

    // The stack object should not be marked was_casualty_paid.
    assert!(
        !state.stack_objects[0].was_casualty_paid,
        "CR 702.153a: was_casualty_paid must be false when casualty was not paid"
    );

    // Resolve -- both players pass priority.
    let (state, _) = pass_all(state, &[p1, p(2)]);

    // Spell resolves once -- gain 2 life.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 2,
        "CR 702.153a: non-casualty cast should gain life exactly once"
    );
}

/// CR 702.153a — Casualty requires a creature: attempting to sacrifice a non-creature
/// (artifact) as the casualty cost is rejected.
#[test]
fn test_casualty_not_a_creature() {
    let p1 = p(1);
    let artifact = ObjectSpec::artifact(p1, "Sol Ring");
    let (state, _p1, _p2, spell_id) = setup_casualty_state(vec![artifact]);

    let artifact_id = find_object(&state, "Sol Ring");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(artifact_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.153a: sacrificing a non-creature as casualty cost must be rejected"
    );
}

/// CR 702.153a — Casualty sacrifice must be controlled by the caster:
/// sacrificing an opponent's creature is rejected.
#[test]
fn test_casualty_wrong_controller() {
    let p1 = p(1);
    let p2 = p(2);

    // Opponent's creature on the battlefield.
    let opponent_creature = ObjectSpec::creature(p2, "Opponent Creature", 2, 2);
    let (state, _p1, _p2, spell_id) = setup_casualty_state(vec![opponent_creature]);

    let opp_creature_id = find_object(&state, "Opponent Creature");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(opp_creature_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.153a: sacrificing an opponent's creature as casualty cost must be rejected"
    );
}

/// Engine validation: Providing a casualty sacrifice for a spell without the Casualty
/// keyword is rejected.
#[test]
fn test_casualty_spell_without_keyword() {
    let p1 = p(1);
    let p2 = p(2);

    let creature = ObjectSpec::creature(p1, "Any Creature", 2, 2);

    let registry = CardRegistry::new(vec![plain_instant_def()]);

    let spell = ObjectSpec::card(p1, "Plain Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-instant".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Plain Instant");
    let creature_id = find_object(&state, "Any Creature");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(creature_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "Engine: providing casualty sacrifice for a spell without Casualty keyword must be rejected"
    );
}

/// Ruling 2022-04-29 — Casualty copy is NOT cast: the `is_copy` flag should be true on
/// the copy, and `spells_cast_this_turn` should only be incremented once (for the original).
#[test]
fn test_casualty_copy_is_not_cast() {
    let p1 = p(1);
    let creature = ObjectSpec::creature(p1, "Soldier Token", 1, 1).token();
    let (state, _p1, _p2, spell_id) = setup_casualty_state(vec![creature]);

    let creature_id = find_object(&state, "Soldier Token");
    let spells_before = state.players[&p1].spells_cast_this_turn;

    // Cast with casualty.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(creature_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with Casualty failed: {:?}", e));

    // After cast, spells_cast_this_turn should be spells_before + 1 (original only).
    assert_eq!(
        state.players[&p1].spells_cast_this_turn,
        spells_before + 1,
        "Ruling 2022-04-29: only the original spell is 'cast'; spells_cast_this_turn should be +1"
    );

    // Resolve the casualty trigger (both players pass priority).
    let (state, _) = pass_all(state, &[p1, p(2)]);

    // After trigger resolves, spells_cast_this_turn should still be +1, not +2.
    assert_eq!(
        state.players[&p1].spells_cast_this_turn,
        spells_before + 1,
        "Ruling 2022-04-29: casualty copy is not cast; spells_cast_this_turn should remain +1"
    );

    // Verify the copy exists on stack with is_copy = true.
    let copy = state.stack_objects.iter().find(|o| o.is_copy);
    assert!(
        copy.is_some(),
        "Ruling 2022-04-29: copy created by casualty trigger should have is_copy = true"
    );
}

/// CR 702.153a — Sacrificing a creature with power strictly greater than N is accepted.
/// Casualty requires power >= N, not power == N.
#[test]
fn test_casualty_higher_power_accepted() {
    let p1 = p(1);
    // Casualty 1 spell; sacrifice a 3/3 (power 3 > 1).
    let big_creature = ObjectSpec::creature(p1, "Big Creature", 3, 3);
    let (state, _p1, _p2, spell_id) = setup_casualty_state(vec![big_creature]);

    let big_id = find_object(&state, "Big Creature");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(big_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.153a: sacrificing a creature with power > N for Casualty N must be accepted"
    );
}

/// CR 702.153a — Sacrificing a creature not on the battlefield is rejected.
#[test]
fn test_casualty_creature_not_on_battlefield() {
    let p1 = p(1);
    // Creature in graveyard, not on battlefield.
    let graveyard_creature =
        ObjectSpec::creature(p1, "Dead Creature", 2, 2).in_zone(ZoneId::Graveyard(p1));
    let (state, _p1, _p2, spell_id) = setup_casualty_state(vec![graveyard_creature]);

    let dead_id = find_object(&state, "Dead Creature");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: None,
            escape_exile_cards: vec![],
            retrace_discard_land: None,
            jump_start_discard: None,
            prototype: false,
            bargain_sacrifice: None,
            emerge_sacrifice: None,
            casualty_sacrifice: Some(dead_id),
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
            devour_sacrifices: vec![],
            modes_chosen: vec![],
            fuse: false,
            x_value: 0,
            collect_evidence_cards: vec![],
            squad_count: 0,
            offspring_paid: false,
            gift_opponent: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.153a: casualty sacrifice must be a creature on the battlefield"
    );
}
