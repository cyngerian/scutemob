//! Bargain keyword ability tests (CR 702.166).
//!
//! Bargain is an optional additional cost found on some spells.
//! "As an additional cost to cast this spell, you may sacrifice an artifact,
//! enchantment, or token." (CR 702.166a)
//!
//! Key rules verified:
//! - Bargain is an optional additional cost — not mandatory (CR 702.166a).
//! - The sacrifice target must be an artifact, enchantment, OR token (CR 702.166a).
//! - Non-token creatures that are not artifacts/enchantments are rejected (CR 702.166a).
//! - Only the caster's own permanents may be sacrificed (CR 702.166a).
//! - Providing a bargain sacrifice for a spell without Bargain is rejected (engine validation).
//! - Spell resolves with `was_bargained = true` when sacrifice was paid (CR 702.166b).
//! - Spell resolves with `was_bargained = false` when sacrifice was not paid (CR 702.166b).
//! - Permanent cast with bargain has `was_bargained = true` after entering battlefield (CR 702.166b).
//! - Permanent cast without bargain has `was_bargained = false` after entering battlefield (CR 702.166b).

use mtg_engine::cards::card_definition::{Condition, EffectAmount, PlayerTarget};
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, GameEvent, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectSpec, PlayerId,
    Step, TypeLine, ZoneId,
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

/// Pass priority for all listed players once.
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

/// Synthetic bargain instant: "Witching Well variant" — when bargained, gain 3 life;
/// when not bargained, gain 1 life.
///
/// Mana cost: {1}{W}. Bargain optional additional cost: sacrifice artifact/enchantment/token.
fn bargain_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("bargain-instant".to_string()),
        name: "Bargain Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Bargain (As an additional cost to cast this spell, you may sacrifice an artifact, enchantment, or token.)\nIf this spell was bargained, you gain 3 life. Otherwise, you gain 1 life.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Bargain),
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasBargained,
                    if_true: Box::new(Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(3),
                    }),
                    if_false: Box::new(Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// Synthetic bargain permanent: artifact that when bargained, controller draws a card on ETB.
///
/// Mana cost: {2}. Bargain optional additional cost: sacrifice artifact/enchantment/token.
/// ETB: if this was bargained, draw a card.
fn bargain_artifact_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("bargain-artifact".to_string()),
        name: "Bargain Artifact".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Artifact].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Bargain (As an additional cost to cast this spell, you may sacrifice an artifact, enchantment, or token.)\nWhen Bargain Artifact enters the battlefield, if it was bargained, draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Bargain),
        ],
        ..Default::default()
    }
}

/// Synthetic spell without Bargain — used to verify that providing a sacrifice
/// for a non-bargain spell is rejected.
fn plain_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-instant".to_string()),
        name: "Plain Instant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
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

/// Helper: build a 2-player state with the bargain instant in p1's hand,
/// plus given objects. Returns (state, p1, p2, spell_id).
fn setup_bargain_state(
    extra_objects: Vec<ObjectSpec>,
) -> (
    mtg_engine::GameState,
    PlayerId,
    PlayerId,
    mtg_engine::ObjectId,
) {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bargain_instant_def()]);

    let spell = ObjectSpec::card(p1, "Bargain Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bargain-instant".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Bargain);

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

    // Add {1}{W} mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Bargain Instant");
    (state, p1, p2, spell_id)
}

// ── Test 1: Basic bargain with token sacrifice ─────────────────────────────────

/// CR 702.166a — Casting a spell with Bargain while sacrificing a token as
/// additional cost: the token should enter graveyard, the spell should resolve
/// with `was_bargained = true`, and the bargained effect (3 life) should apply.
#[test]
fn test_bargain_basic_instant_with_sacrifice() {
    let p1 = p(1);

    let token = ObjectSpec::card(p1, "Creature Token")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .token(); // is_token: true

    let (state, _p1, _p2, spell_id) = setup_bargain_state(vec![token]);
    let initial_life = state.players[&p1].life_total;

    let token_id = find_object(&state, "Creature Token");

    // Cast Bargain Instant while sacrificing the creature token.
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
            bargain_sacrifice: Some(token_id),
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with bargain (token) failed: {:?}", e));

    // CR 702.166b: The spell on the stack should be marked as bargained.
    assert!(
        state.stack_objects[0].was_bargained,
        "CR 702.166b: was_bargained should be true on stack object after paying bargain cost"
    );

    // CR 702.166a: The token must have been sacrificed — no longer on battlefield.
    let token_on_battlefield = find_object_in_zone(&state, "Creature Token", ZoneId::Battlefield);
    assert!(
        token_on_battlefield.is_none(),
        "CR 702.166a: token sacrificed as bargain cost must no longer be on the battlefield"
    );

    // Both players pass priority — spell resolves.
    let (state, events) = pass_all(state, &[p1, _p2]);

    // CR 702.166b: Bargained effect (3 life) should have applied.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 3,
        "CR 702.166b: bargained spell should grant 3 life (not 1)"
    );

    // There should be a LifeGained event with amount 3.
    let life_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::LifeGained { amount, .. } if *amount == 3));
    assert!(
        life_event,
        "CR 702.166b: LifeGained event with amount 3 expected; events: {:?}",
        events
    );
}

// ── Test 2: Bargain without sacrifice (optional cost not paid) ─────────────────

/// CR 702.166a — Bargain is optional. Casting the spell without providing a
/// sacrifice should succeed and resolve with `was_bargained = false`, applying
/// the base effect (1 life).
#[test]
fn test_bargain_basic_instant_without_sacrifice() {
    let p1 = p(1);

    let (state, _p1, _p2, spell_id) = setup_bargain_state(vec![]);
    let initial_life = state.players[&p1].life_total;

    // Cast Bargain Instant without bargaining.
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
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell without bargain failed: {:?}", e));

    // CR 702.166b: The spell on the stack should NOT be marked as bargained.
    assert!(
        !state.stack_objects[0].was_bargained,
        "CR 702.166b: was_bargained should be false when sacrifice was not provided"
    );

    // Both players pass priority — spell resolves.
    let (state, events) = pass_all(state, &[p1, _p2]);

    // Base effect (1 life) should apply, not the bargained effect.
    assert_eq!(
        state.players[&p1].life_total,
        initial_life + 1,
        "CR 702.166a: un-bargained spell should grant only 1 life (not 3)"
    );

    // There should be a LifeGained event with amount 1.
    let life_event = events
        .iter()
        .any(|e| matches!(e, GameEvent::LifeGained { amount, .. } if *amount == 1));
    assert!(
        life_event,
        "CR 702.166a: LifeGained event with amount 1 expected; events: {:?}",
        events
    );

    // There must NOT be a LifeGained event with amount 3.
    let life_event_3 = events
        .iter()
        .any(|e| matches!(e, GameEvent::LifeGained { amount, .. } if *amount == 3));
    assert!(
        !life_event_3,
        "CR 702.166a: no LifeGained with amount 3 expected when not bargained; events: {:?}",
        events
    );
}

// ── Test 3: Bargain — sacrifice an artifact ────────────────────────────────────

/// CR 702.166a — An artifact (non-token) is a valid bargain sacrifice target.
/// Sacrificing an artifact is accepted and the spell is bargained.
#[test]
fn test_bargain_sacrifice_artifact() {
    let p1 = p(1);

    let artifact = ObjectSpec::card(p1, "Sol Ring")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);

    let (state, _p1, _p2, spell_id) = setup_bargain_state(vec![artifact]);

    let artifact_id = find_object(&state, "Sol Ring");

    // Sacrificing an artifact should succeed.
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
            bargain_sacrifice: Some(artifact_id),
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell bargaining with artifact failed: {:?}", e));

    // CR 702.166b: spell should be bargained.
    assert!(
        state.stack_objects[0].was_bargained,
        "CR 702.166a: artifact sacrifice should mark spell as bargained"
    );

    // Sol Ring must have been moved off the battlefield.
    let artifact_on_battlefield = find_object_in_zone(&state, "Sol Ring", ZoneId::Battlefield);
    assert!(
        artifact_on_battlefield.is_none(),
        "CR 702.166a: sacrificed artifact must no longer be on the battlefield"
    );
}

// ── Test 4: Bargain — sacrifice an enchantment ─────────────────────────────────

/// CR 702.166a — An enchantment (non-token) is a valid bargain sacrifice target.
/// Sacrificing an enchantment is accepted and the spell is bargained.
#[test]
fn test_bargain_sacrifice_enchantment() {
    let p1 = p(1);

    let enchantment = ObjectSpec::card(p1, "Pacifism")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Enchantment]);

    let (state, _p1, _p2, spell_id) = setup_bargain_state(vec![enchantment]);

    let enchantment_id = find_object(&state, "Pacifism");

    // Sacrificing an enchantment should succeed.
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
            bargain_sacrifice: Some(enchantment_id),
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell bargaining with enchantment failed: {:?}", e));

    // CR 702.166b: spell should be bargained.
    assert!(
        state.stack_objects[0].was_bargained,
        "CR 702.166a: enchantment sacrifice should mark spell as bargained"
    );

    // Pacifism must have been moved off the battlefield.
    let enchantment_on_battlefield = find_object_in_zone(&state, "Pacifism", ZoneId::Battlefield);
    assert!(
        enchantment_on_battlefield.is_none(),
        "CR 702.166a: sacrificed enchantment must no longer be on the battlefield"
    );
}

// ── Test 5: Bargain — sacrifice a creature token ───────────────────────────────

/// CR 702.166a — A token of any type (including creature tokens) is a valid
/// bargain sacrifice target. Tokens qualify as tokens regardless of their types.
#[test]
fn test_bargain_sacrifice_creature_token() {
    let p1 = p(1);

    // A 1/1 creature token — not an artifact, not an enchantment, but IS a token.
    let creature_token = ObjectSpec::card(p1, "Soldier Token")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature])
        .token(); // is_token: true marks it as a token

    let (state, _p1, _p2, spell_id) = setup_bargain_state(vec![creature_token]);

    let token_id = find_object(&state, "Soldier Token");

    // Sacrificing a creature token should succeed — tokens qualify even if not artifact/enchantment.
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
            bargain_sacrifice: Some(token_id),
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell bargaining with creature token failed: {:?}", e));

    // CR 702.166b: spell should be bargained.
    assert!(
        state.stack_objects[0].was_bargained,
        "CR 702.166a: creature token sacrifice should mark spell as bargained (tokens qualify)"
    );
}

// ── Test 6: Bargain — invalid sacrifice target: non-token creature ─────────────

/// CR 702.166a — A non-token, non-artifact, non-enchantment creature is NOT a
/// valid bargain sacrifice target. Attempting to sacrifice one must be rejected.
#[test]
fn test_bargain_sacrifice_invalid_creature() {
    let p1 = p(1);

    // A non-token creature that is neither an artifact nor an enchantment.
    let invalid_creature = ObjectSpec::card(p1, "Llanowar Elves")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Creature]);
    // NOTE: is_token is false by default — this is a regular creature.

    let (state, _p1, _p2, spell_id) = setup_bargain_state(vec![invalid_creature]);

    let creature_id = find_object(&state, "Llanowar Elves");

    // Attempting to sacrifice a non-token creature should be rejected.
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
            bargain_sacrifice: Some(creature_id),
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.166a: non-token creature should be rejected as a bargain sacrifice target"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "CR 702.166a: error should be InvalidCommand; got: {:?}",
        err
    );
}

// ── Test 7: Bargain — cannot sacrifice opponent's permanent ────────────────────

/// CR 702.166a — The bargain sacrifice must be controlled by the caster.
/// Attempting to sacrifice an opponent's artifact must be rejected.
#[test]
fn test_bargain_sacrifice_opponent_permanent() {
    let p1 = p(1);
    let p2 = p(2);

    // An artifact controlled by the opponent (p2), not by the caster (p1).
    let opponent_artifact = ObjectSpec::card(p2, "Opponent Artifact")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);

    let (state, _p1, _p2, spell_id) = setup_bargain_state(vec![opponent_artifact]);

    let opponent_artifact_id = find_object(&state, "Opponent Artifact");

    // Attempting to sacrifice an opponent's artifact should be rejected.
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
            bargain_sacrifice: Some(opponent_artifact_id),
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.166a: sacrificing an opponent's permanent should be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "CR 702.166a: error should be InvalidCommand; got: {:?}",
        err
    );
}

// ── Test 8: No Bargain keyword → sacrifice rejected ────────────────────────────

/// CR 702.166a — Providing a bargain sacrifice for a spell that does NOT have
/// the Bargain keyword must be rejected by the engine.
#[test]
fn test_bargain_no_keyword_rejects_sacrifice() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![plain_instant_def()]);

    // Plain instant has no Bargain keyword.
    let spell = ObjectSpec::card(p1, "Plain Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("plain-instant".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });

    // An artifact controlled by p1.
    let artifact = ObjectSpec::card(p1, "Sol Ring")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(artifact)
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
    let artifact_id = find_object(&state, "Sol Ring");

    // Attempting to provide a sacrifice for a non-bargain spell should be rejected.
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
            bargain_sacrifice: Some(artifact_id),
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    );

    assert!(
        result.is_err(),
        "CR 702.166a: providing a bargain sacrifice for a spell without Bargain must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, mtg_engine::GameStateError::InvalidCommand(_)),
        "CR 702.166a: error should be InvalidCommand; got: {:?}",
        err
    );
}

// ── Test 9: Bargain permanent — was_bargained propagates to GameObject ─────────

/// CR 702.166b — When a permanent with Bargain is cast WITH the bargain cost paid
/// and enters the battlefield, the resulting GameObject should have
/// `was_bargained = true`. This is required for ETB triggers that check
/// "if this permanent was bargained" (e.g., Hylda's Crown of Winter).
#[test]
fn test_bargain_permanent_etb_was_bargained() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bargain_artifact_def()]);

    // The bargain artifact spell.
    let spell = ObjectSpec::card(p1, "Bargain Artifact")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bargain-artifact".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Bargain);

    // An enchantment for p1 to sacrifice.
    let enchantment = ObjectSpec::card(p1, "Pacifism")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Enchantment]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(enchantment)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Bargain Artifact");
    let enchantment_id = find_object(&state, "Pacifism");

    // Cast the bargain artifact while bargaining.
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
            bargain_sacrifice: Some(enchantment_id),
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell bargain artifact failed: {:?}", e));

    // Spell is on the stack and is bargained.
    assert!(
        state.stack_objects[0].was_bargained,
        "CR 702.166b: was_bargained must be true on StackObject before resolution"
    );

    // Both players pass priority — spell resolves, permanent enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Find the Bargain Artifact on the battlefield.
    let artifact_on_bf = state.objects.values().find(|obj| {
        obj.characteristics.name == "Bargain Artifact" && obj.zone == ZoneId::Battlefield
    });

    assert!(
        artifact_on_bf.is_some(),
        "CR 702.166b: Bargain Artifact should be on the battlefield after resolution"
    );

    let artifact_obj = artifact_on_bf.unwrap();
    assert!(
        artifact_obj.was_bargained,
        "CR 702.166b: permanent's was_bargained must be true — ETB triggers checking \
         Condition::WasBargained require this to be propagated from StackObject to GameObject"
    );
}

// ── Test 10: Bargain permanent — was_bargained is false when not bargained ──────

/// CR 702.166b — When a permanent with Bargain is cast WITHOUT the bargain cost paid
/// and enters the battlefield, the resulting GameObject should have
/// `was_bargained = false`. This prevents ETB triggers from incorrectly
/// treating the permanent as bargained.
#[test]
fn test_bargain_permanent_etb_not_bargained() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bargain_artifact_def()]);

    // The bargain artifact spell.
    let spell = ObjectSpec::card(p1, "Bargain Artifact")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("bargain-artifact".to_string()))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 2,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Bargain);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Bargain Artifact");

    // Cast the bargain artifact WITHOUT bargaining.
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
            bargain_sacrifice: None, // No bargain sacrifice
            emerge_sacrifice: None,
            casualty_sacrifice: None,
            assist_player: None,
            assist_amount: 0,
            replicate_count: 0,
            splice_cards: vec![],
            entwine_paid: false,
            escalate_modes: 0,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell without bargain failed: {:?}", e));

    // Spell is on the stack and is NOT bargained.
    assert!(
        !state.stack_objects[0].was_bargained,
        "CR 702.166b: was_bargained must be false on StackObject when no sacrifice provided"
    );

    // Both players pass priority — spell resolves, permanent enters battlefield.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Find the Bargain Artifact on the battlefield.
    let artifact_on_bf = state.objects.values().find(|obj| {
        obj.characteristics.name == "Bargain Artifact" && obj.zone == ZoneId::Battlefield
    });

    assert!(
        artifact_on_bf.is_some(),
        "Bargain Artifact should be on the battlefield after resolution without bargaining"
    );

    let artifact_obj = artifact_on_bf.unwrap();
    assert!(
        !artifact_obj.was_bargained,
        "CR 702.166b: permanent's was_bargained must be false when cast without bargaining"
    );
}
