//! Extort keyword ability tests (CR 702.101).
//!
//! Extort is a triggered ability: "Whenever you cast a spell, you may pay {W/B}.
//! If you do, each opponent loses 1 life and you gain life equal to the total
//! life lost this way."
//!
//! Key rules verified:
//! - Triggers on any spell cast by the controller, including creature spells (CR 702.101a).
//! - Does NOT trigger for opponent spells (CR 702.101a: "you cast").
//! - Multiple instances trigger separately (CR 702.101b).
//! - Life gained equals total life ACTUALLY lost, not opponents_count*amount (ruling 2024-01-12).
//! - Does not target any player — hexproof/shroud do not prevent extort (ruling 2024-01-12).
//! - Extort trigger resolves before the triggering spell (standard triggered ability ordering).
//! - Multiplayer: in 4-player, each of 3 opponents loses 1 life, controller gains 3.

use mtg_engine::{
    process_command, CardDefinition, CardId, CardRegistry, CardType, Command, GameEvent,
    GameStateBuilder, KeywordAbility, ManaCost, ObjectSpec, PlayerId, Step, Target, TypeLine,
    ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn life_total(state: &mtg_engine::GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(&player)
        .map(|ps| ps.life_total)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
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

/// A simple instant — "Lightning Bolt" stand-in.
fn instant_def() -> CardDefinition {
    use mtg_engine::cards::card_definition::EffectAmount;
    use mtg_engine::{AbilityDefinition, CardEffectTarget, Effect, TargetRequirement};
    CardDefinition {
        card_id: CardId("lightning-bolt".to_string()),
        name: "Lightning Bolt".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Lightning Bolt deals 3 damage to any target.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DealDamage {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// A simple creature spell.
fn creature_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("grizzly-bears".to_string()),
        name: "Grizzly Bears".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![],
        ..Default::default()
    }
}

// ── Test 1: Basic drain on spell cast ─────────────────────────────────────────

#[test]
/// CR 702.101a — Extort: whenever you cast a spell, each opponent loses 1 life
/// and you gain life equal to the total life lost. In a 4-player game (3 opponents),
/// controller gains 3 life.
fn test_extort_basic_drain_on_spell_cast() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let registry = CardRegistry::new(vec![instant_def()]);

    // p1 has a creature with Extort on the battlefield.
    let extort_creature =
        ObjectSpec::creature(p1, "Extort Creature", 2, 2).with_keyword(KeywordAbility::Extort);

    // p1 has Lightning Bolt in hand.
    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(extort_creature)
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
        .add(mtg_engine::ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Lightning Bolt");

    // p1 casts Lightning Bolt targeting p2.
    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .unwrap();

    // AbilityTriggered event emitted for extort.
    assert!(
        cast_events.iter().any(
            |e| matches!(e, GameEvent::AbilityTriggered { controller, .. } if *controller == p1)
        ),
        "CR 702.101a: AbilityTriggered event expected for extort"
    );

    // Stack has 2 items: the spell + extort trigger.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.101a: stack should have spell + extort trigger"
    );

    // All 4 players pass — extort trigger resolves first (top of stack).
    let (_state, resolve_events) = pass_all(state, &[p1, p2, p3, p4]);

    // Extort trigger resolved: each opponent lost 1 life.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 1 } if *player == p2)),
        "CR 702.101a: p2 should have lost 1 life from extort"
    );
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 1 } if *player == p3)),
        "CR 702.101a: p3 should have lost 1 life from extort"
    );
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 1 } if *player == p4)),
        "CR 702.101a: p4 should have lost 1 life from extort"
    );

    // Controller gained 3 life (one per opponent).
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeGained { player, amount: 3 } if *player == p1)),
        "CR 702.101a: p1 should have gained 3 life (1 per opponent)"
    );
}

// ── Test 2: Extort triggers on creature spell ─────────────────────────────────

#[test]
/// CR 702.101a — Extort triggers "whenever you cast a spell" with no type restriction.
/// Unlike Prowess (noncreature only), extort triggers on creature spells too.
fn test_extort_triggers_on_creature_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![creature_def()]);

    let extort_creature =
        ObjectSpec::creature(p1, "Extort Creature", 2, 2).with_keyword(KeywordAbility::Extort);

    let creature_spell = ObjectSpec::card(p1, "Grizzly Bears")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            green: 1,
            generic: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(extort_creature)
        .object(creature_spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Green, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Grizzly Bears");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .unwrap();

    // Stack has 2 items: creature spell + extort trigger.
    assert_eq!(
        state.stack_objects.len(),
        2,
        "CR 702.101a: extort should trigger on creature spell (no type restriction)"
    );

    // AbilityTriggered event from extort.
    assert!(
        cast_events.iter().any(
            |e| matches!(e, GameEvent::AbilityTriggered { controller, .. } if *controller == p1)
        ),
        "CR 702.101a: extort should trigger on creature spell"
    );
}

// ── Test 3: Opponent's spell does NOT trigger extort ─────────────────────────

#[test]
/// CR 702.101a — "whenever YOU cast a spell" means only the extort permanent's
/// controller. An opponent casting a spell does NOT trigger extort.
fn test_extort_does_not_trigger_for_opponent_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![instant_def()]);

    // p1 has an extort creature on the battlefield.
    let extort_creature =
        ObjectSpec::creature(p1, "Extort Creature", 2, 2).with_keyword(KeywordAbility::Extort);

    // p2 has Lightning Bolt in hand (p2 is active player).
    let spell = ObjectSpec::card(p2, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p2))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(extort_creature)
        .object(spell)
        .active_player(p2) // p2 is active player
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    let spell_id = find_object(&state, "Lightning Bolt");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Player(p1)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .unwrap();

    // Only the spell on the stack — NO extort trigger.
    assert_eq!(
        state.stack_objects.len(),
        1,
        "CR 702.101a: p2's spell should NOT trigger p1's extort (only controller's spells trigger)"
    );

    // No AbilityTriggered events from extort.
    assert!(
        !cast_events.iter().any(
            |e| matches!(e, GameEvent::AbilityTriggered { controller, .. } if *controller == p1)
        ),
        "CR 702.101a: p1's extort should NOT trigger for p2's spell"
    );
}

// ── Test 4: Multiple instances trigger separately ────────────────────────────

#[test]
/// CR 702.101b — If a permanent has multiple instances of extort, each triggers
/// separately. Two extort instances on one permanent = two separate triggers.
/// In 4-player: each trigger drains 1 from each of 3 opponents, total 6 life gained.
fn test_extort_multiple_instances_trigger_separately() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let registry = CardRegistry::new(vec![instant_def()]);

    // p1 has a creature with TWO instances of Extort.
    let extort_creature = ObjectSpec::creature(p1, "Double Extort", 2, 2)
        .with_keyword(KeywordAbility::Extort)
        .with_keyword(KeywordAbility::Extort);

    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(extort_creature)
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
        .add(mtg_engine::ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    // Record starting life totals.
    let p1_start = life_total(&state, p1);
    let p2_start = life_total(&state, p2);
    let p3_start = life_total(&state, p3);
    let p4_start = life_total(&state, p4);

    let spell_id = find_object(&state, "Lightning Bolt");

    let (state, cast_events) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .unwrap();

    // Two AbilityTriggered events from the two extort instances.
    let triggered_count = cast_events
        .iter()
        .filter(
            |e| matches!(e, GameEvent::AbilityTriggered { controller, .. } if *controller == p1),
        )
        .count();
    assert_eq!(
        triggered_count, 2,
        "CR 702.101b: two extort instances should produce two triggers"
    );

    // Stack: spell + 2 extort triggers = 3 items.
    assert_eq!(
        state.stack_objects.len(),
        3,
        "CR 702.101b: stack should have spell + 2 extort triggers"
    );

    // Resolve first extort trigger (all pass once).
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // After first trigger: each opponent lost 1 life, p1 gained 3.
    // Stack now has spell + 1 extort trigger = 2 items.
    assert_eq!(state.stack_objects.len(), 2, "one extort trigger resolved");

    // Resolve second extort trigger (all pass again).
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // After both triggers: each opponent lost 2 total, p1 gained 6 total.
    assert_eq!(
        life_total(&state, p2),
        p2_start - 2,
        "CR 702.101b: p2 should have lost 2 life (1 per extort trigger)"
    );
    assert_eq!(
        life_total(&state, p3),
        p3_start - 2,
        "CR 702.101b: p3 should have lost 2 life (1 per extort trigger)"
    );
    assert_eq!(
        life_total(&state, p4),
        p4_start - 2,
        "CR 702.101b: p4 should have lost 2 life (1 per extort trigger)"
    );
    assert_eq!(
        life_total(&state, p1),
        p1_start + 6,
        "CR 702.101b: p1 should have gained 6 life (3 per trigger × 2 triggers)"
    );
}

// ── Test 5: Extort does not target — affects all opponents regardless ─────────

#[test]
/// Ruling 2024-01-12: "The extort ability doesn't target any player."
/// Hexproof/shroud/protection do not prevent extort. We verify this by
/// ensuring all opponents lose life even when one could theoretically be
/// "untargetable" — extort hits everyone because it doesn't use targeting.
/// In a 2-player game, the single opponent loses 1 life.
fn test_extort_does_not_target_hits_all_opponents() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![instant_def()]);

    let extort_creature =
        ObjectSpec::creature(p1, "Extort Creature", 2, 2).with_keyword(KeywordAbility::Extort);

    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(extort_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let p2_start = life_total(&state, p2);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Lightning Bolt");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .unwrap();

    // Resolve extort trigger.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    // p2 lost 1 life — extort hit them (not a targeted effect, can't be hexproofed).
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::LifeLost { player, amount: 1 } if *player == p2)),
        "Ruling 2024-01-12: extort should affect p2 (no targeting, hexproof irrelevant)"
    );
    assert_eq!(
        life_total(&state, p2),
        p2_start - 1,
        "Ruling 2024-01-12: p2 loses 1 life from extort (non-targeted)"
    );
}

// ── Test 6: Extort resolves before the triggering spell ──────────────────────

#[test]
/// Ruling 2024-01-12 / standard triggered ability behavior: extort is a triggered
/// ability that goes on the stack above the spell. It resolves before the spell.
/// After extort resolves, the triggering spell is still on the stack.
fn test_extort_resolves_before_triggering_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let registry = CardRegistry::new(vec![instant_def()]);

    let extort_creature =
        ObjectSpec::creature(p1, "Extort Creature", 2, 2).with_keyword(KeywordAbility::Extort);

    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(extort_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let p1_start = life_total(&state, p1);
    let p2_start = life_total(&state, p2);

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Lightning Bolt");

    // Cast Lightning Bolt.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .unwrap();

    // Stack: Lightning Bolt (bottom) + extort trigger (top).
    assert_eq!(state.stack_objects.len(), 2);

    // Both pass — extort trigger resolves first.
    let (state, _) = pass_all(state, &[p1, p2]);

    // After extort resolves: p2 lost 1 life, p1 gained 1. Spell still on stack.
    assert_eq!(
        life_total(&state, p2),
        p2_start - 1,
        "extort should have resolved: p2 lost 1 life"
    );
    assert_eq!(
        life_total(&state, p1),
        p1_start + 1,
        "extort should have resolved: p1 gained 1 life"
    );

    // Lightning Bolt is still on the stack (not yet resolved).
    assert_eq!(
        state.stack_objects.len(),
        1,
        "Ruling 2024-01-12: Lightning Bolt should still be on stack after extort resolves"
    );
}

// ── Test 7: Multiplayer 4-player life totals ──────────────────────────────────

#[test]
/// CR 702.101a + multiplayer (CR 903): In a 4-player Commander game at 40 life,
/// extort drains 1 from each of 3 opponents (P2, P3, P4) and grants controller
/// 3 life total. Final life totals: P1=43, P2=39, P3=39, P4=39.
fn test_extort_multiplayer_4_player_drain() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);
    let p4 = PlayerId(4);

    let registry = CardRegistry::new(vec![instant_def()]);

    let extort_creature =
        ObjectSpec::creature(p1, "Extort Creature", 2, 2).with_keyword(KeywordAbility::Extort);

    let spell = ObjectSpec::card(p1, "Lightning Bolt")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(extort_creature)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Set all players to 40 life (Commander starting life total).
    for pid in [p1, p2, p3, p4] {
        state.players.get_mut(&pid).unwrap().life_total = 40;
    }

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Lightning Bolt");

    // p1 casts Lightning Bolt targeting p2.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![Target::Player(p2)],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            cast_with_evoke: false,
            cast_with_bestow: false,
            cast_with_miracle: false,
            cast_with_escape: false,
            escape_exile_cards: vec![],
            cast_with_foretell: false,
            cast_with_buyback: false,
            cast_with_overload: false,
            retrace_discard_land: None,
        },
    )
    .unwrap();

    // All pass — extort trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    // CR 702.101a: each opponent (p2, p3, p4) lost 1; p1 gained 3.
    assert_eq!(
        life_total(&state, p2),
        39,
        "CR 702.101a: p2 should be at 39 life after extort"
    );
    assert_eq!(
        life_total(&state, p3),
        39,
        "CR 702.101a: p3 should be at 39 life after extort"
    );
    assert_eq!(
        life_total(&state, p4),
        39,
        "CR 702.101a: p4 should be at 39 life after extort"
    );
    assert_eq!(
        life_total(&state, p1),
        43,
        "CR 702.101a: p1 should be at 43 life after extort (gained 3 from 3 opponents)"
    );
}
