//! Split Second keyword ability tests (CR 702.61).
//!
//! Split second is a static ability that functions only while the spell with split
//! second is on the stack. "Split second" means "As long as this spell is on the
//! stack, players can't cast other spells or activate abilities that aren't mana
//! abilities." (CR 702.61a)
//!
//! Key rules verified:
//! - While a split second spell is on the stack, no spells can be cast (CR 702.61a).
//! - While a split second spell is on the stack, non-mana abilities cannot be
//!   activated (CR 702.61a). This includes cycling (CR 702.29a is an activated ability).
//! - Mana abilities (TapForMana) are still allowed (CR 702.61b).
//! - PassPriority is always allowed (CR 702.61b — players still get priority).
//! - The restriction applies to ALL players, including the caster (CR 702.61a).
//! - After the split second spell resolves (leaves the stack), the restriction ends
//!   and spells may be cast normally (Krosan Grip ruling 2021-03-19).
//! - Triggered abilities still fire and are put on the stack normally (CR 702.61b).

use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::turn::Step;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{
    ActivatedAbility, ActivationCost, GameStateBuilder, ManaColor, ManaCost, ObjectSpec, PlayerId,
    StackObjectKind, TriggerEvent, TriggeredAbilityDef,
};
use mtg_engine::{
    AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, GameState, KeywordAbility,
    ManaAbility, TypeLine,
};
use std::sync::Arc;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
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

// ── Card definitions ───────────────────────────────────────────────────────────

/// A split second instant card. No mana cost (free), split second only.
fn split_second_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("split-second-instant".to_string()),
        name: "Split Second Instant".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Split second (As long as this spell is on the stack, players can't cast \
                      spells or activate abilities that aren't mana abilities.)"
            .to_string(),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::SplitSecond)],
        ..Default::default()
    }
}

/// A plain instant card (no special abilities). Used as the spell opponents try to cast.
fn plain_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("plain-instant".to_string()),
        name: "Plain Instant".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Do nothing.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}

/// A card with Cycling {0}. Used to test that cycling is blocked by split second.
fn cycling_instant_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("cycling-instant".to_string()),
        name: "Cycling Instant".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cycling {0}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}

// ── Test 1: Split second blocks casting spells ─────────────────────────────────

#[test]
/// CR 702.61a — While a spell with split second is on the stack, players can't cast
/// other spells. P2 tries to cast an instant but is blocked.
fn test_split_second_blocks_casting_spells() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = Arc::new(CardRegistry::new(vec![
        split_second_instant_def(),
        plain_instant_def(),
    ]));

    let ss_card = ObjectSpec::card(p1, "Split Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("split-second-instant".to_string()))
        .with_keyword(KeywordAbility::SplitSecond);

    // Instant type on ObjectSpec ensures it's instant-speed eligible (not just sorcery-speed).
    let plain_card = ObjectSpec::card(p2, "Plain Instant")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("plain-instant".to_string()))
        .with_types(vec![CardType::Instant]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry((*registry).clone())
        .object(ss_card)
        .object(plain_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let ss_id = find_object(&state, "Split Second Instant");

    // P1 casts the split second spell — succeeds because split second isn't on stack yet.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ss_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell (split second) failed: {:?}", e));

    assert_eq!(
        state.stack_objects.len(),
        1,
        "split second spell should be on the stack"
    );

    // P2 has priority (after p1's cast, active player gets it, but let's give it to p2).
    let mut state = state;
    state.turn.priority_holder = Some(p2);

    let plain_id = find_object(&state, "Plain Instant");

    // P2 tries to cast the instant — should be blocked by split second.
    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: plain_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    );

    match result {
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("split second"),
                "CR 702.61a: error message should mention split second, got: {}",
                msg
            );
        }
        Ok(_) => panic!("CR 702.61a: CastSpell should be blocked while split second is on stack"),
    }
}

// ── Test 2: Split second blocks activated abilities ────────────────────────────

#[test]
/// CR 702.61a — While a spell with split second is on the stack, non-mana abilities
/// cannot be activated. P2 has a creature with a tap ability and is blocked.
fn test_split_second_blocks_activated_abilities() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = Arc::new(CardRegistry::new(vec![split_second_instant_def()]));

    let ss_card = ObjectSpec::card(p1, "Split Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("split-second-instant".to_string()))
        .with_keyword(KeywordAbility::SplitSecond);

    // P2's creature on battlefield with a tap activated ability.
    let creature = ObjectSpec::creature(p2, "Sparkmage", 2, 2)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                requires_tap: true,
                mana_cost: None,
                sacrifice_self: false,
            },
            description: "{T}: Does something".to_string(),
            effect: None,
            sorcery_speed: false,
        })
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry((*registry).clone())
        .object(ss_card)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let ss_id = find_object(&state, "Split Second Instant");

    // P1 casts the split second spell.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ss_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell (split second) failed: {:?}", e));

    let mut state = state;
    state.turn.priority_holder = Some(p2);

    let creature_id = find_object(&state, "Sparkmage");

    // P2 tries to activate the tap ability — should be blocked.
    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: creature_id,
            ability_index: 0,
            targets: vec![],
        },
    );

    match result {
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("split second"),
                "CR 702.61a: error message should mention split second, got: {}",
                msg
            );
        }
        Ok(_) => {
            panic!("CR 702.61a: ActivateAbility should be blocked while split second is on stack")
        }
    }
}

// ── Test 3: Split second blocks cycling ───────────────────────────────────────

#[test]
/// CR 702.61a — Cycling is an activated ability (not a mana ability). It cannot be
/// activated while a spell with split second is on the stack.
fn test_split_second_blocks_cycling() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = Arc::new(CardRegistry::new(vec![
        split_second_instant_def(),
        cycling_instant_def(),
    ]));

    let ss_card = ObjectSpec::card(p1, "Split Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("split-second-instant".to_string()))
        .with_keyword(KeywordAbility::SplitSecond);

    let cycling_card = ObjectSpec::card(p2, "Cycling Instant")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("cycling-instant".to_string()))
        .with_keyword(KeywordAbility::Cycling);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry((*registry).clone())
        .object(ss_card)
        .object(cycling_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let ss_id = find_object(&state, "Split Second Instant");

    // P1 casts the split second spell.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ss_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell (split second) failed: {:?}", e));

    let mut state = state;
    state.turn.priority_holder = Some(p2);

    let cycling_id = find_object(&state, "Cycling Instant");

    // P2 tries to cycle the card — should be blocked by split second.
    let result = process_command(
        state,
        Command::CycleCard {
            player: p2,
            card: cycling_id,
        },
    );

    match result {
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("split second"),
                "CR 702.61a: error message should mention split second, got: {}",
                msg
            );
        }
        Ok(_) => panic!("CR 702.61a: CycleCard should be blocked while split second is on stack"),
    }
}

// ── Test 4: Split second allows mana abilities ─────────────────────────────────

#[test]
/// CR 702.61b — Mana abilities (TapForMana) are still allowed while a spell with
/// split second is on the stack.
fn test_split_second_allows_mana_abilities() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = Arc::new(CardRegistry::new(vec![split_second_instant_def()]));

    let ss_card = ObjectSpec::card(p1, "Split Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("split-second-instant".to_string()))
        .with_keyword(KeywordAbility::SplitSecond);

    // P2 has a basic forest with a mana ability on the battlefield.
    let forest = ObjectSpec::land(p2, "Forest")
        .with_mana_ability(ManaAbility::tap_for(ManaColor::Green))
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry((*registry).clone())
        .object(ss_card)
        .object(forest)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let ss_id = find_object(&state, "Split Second Instant");

    // P1 casts the split second spell.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ss_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell (split second) failed: {:?}", e));

    let mut state = state;
    state.turn.priority_holder = Some(p2);

    let forest_id = find_object(&state, "Forest");

    // P2 taps the forest for mana — should succeed (mana abilities are exempt per CR 702.61b).
    let result = process_command(
        state,
        Command::TapForMana {
            player: p2,
            source: forest_id,
            ability_index: 0,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.61b: TapForMana should succeed while split second is on the stack, got: {:?}",
        result.err()
    );

    let (new_state, events) = result.unwrap();
    assert_eq!(
        new_state.player(p2).unwrap().mana_pool.green,
        1,
        "CR 702.61b: green mana should be added to p2's pool"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ManaAdded {
                player,
                color: ManaColor::Green,
                ..
            } if *player == p2
        )),
        "CR 702.61b: ManaAdded event expected"
    );
}

// ── Test 5: Split second allows pass priority ──────────────────────────────────

#[test]
/// CR 702.61b — Players still receive priority and can pass while split second is
/// on the stack.
fn test_split_second_allows_pass_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = Arc::new(CardRegistry::new(vec![split_second_instant_def()]));

    let ss_card = ObjectSpec::card(p1, "Split Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("split-second-instant".to_string()))
        .with_keyword(KeywordAbility::SplitSecond);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry((*registry).clone())
        .object(ss_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let ss_id = find_object(&state, "Split Second Instant");

    // P1 casts the split second spell. After casting, p1 gets priority.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ss_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell (split second) failed: {:?}", e));

    // P1 still has priority after casting.
    assert_eq!(state.turn.priority_holder, Some(p1));

    // P1 passes priority — should succeed even with split second on stack.
    let result = process_command(state, Command::PassPriority { player: p1 });

    assert!(
        result.is_ok(),
        "CR 702.61b: PassPriority should always succeed, even with split second on stack, got: {:?}",
        result.err()
    );

    let (new_state, _) = result.unwrap();
    // Priority should now be with p2 (next player in turn order).
    assert_eq!(
        new_state.turn.priority_holder,
        Some(p2),
        "priority should pass to next player after PassPriority"
    );
}

// ── Test 6: Split second applies to caster too ────────────────────────────────

#[test]
/// CR 702.61a — Split second restricts ALL players, including the caster of the
/// split second spell itself.
fn test_split_second_blocks_caster_too() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = Arc::new(CardRegistry::new(vec![split_second_instant_def()]));

    let ss_card = ObjectSpec::card(p1, "Split Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("split-second-instant".to_string()))
        .with_keyword(KeywordAbility::SplitSecond);

    // A second instant card in p1's hand (no card definition needed — naked object).
    let second_instant = ObjectSpec::card(p1, "Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_types(vec![CardType::Instant]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry((*registry).clone())
        .object(ss_card)
        .object(second_instant)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let ss_id = find_object(&state, "Split Second Instant");

    // P1 casts the split second spell — succeeds.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ss_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell (split second) failed: {:?}", e));

    // P1 still has priority after casting.
    assert_eq!(state.turn.priority_holder, Some(p1));

    let second_id = find_object(&state, "Second Instant");

    // P1 tries to cast another spell — should be blocked by split second (applies to caster too).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: second_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    );

    match result {
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("split second"),
                "CR 702.61a: caster is also blocked by split second, got: {}",
                msg
            );
        }
        Ok(_) => {
            panic!(
                "CR 702.61a: caster should also be blocked from casting while split second is on stack"
            )
        }
    }
}

// ── Test 7: Restriction ends after resolution ──────────────────────────────────

#[test]
/// CR 702.61a — Split second only functions while the spell is on the stack.
/// Once the spell resolves (or leaves the stack for any reason), the restriction
/// ends and players can cast spells normally again. (Krosan Grip ruling 2021-03-19)
fn test_split_second_restriction_ends_after_resolution() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = Arc::new(CardRegistry::new(vec![
        split_second_instant_def(),
        plain_instant_def(),
    ]));

    let ss_card = ObjectSpec::card(p1, "Split Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("split-second-instant".to_string()))
        .with_keyword(KeywordAbility::SplitSecond);

    // P2's instant: must have Instant card type in ObjectSpec so it can be cast at any time.
    let plain_card = ObjectSpec::card(p2, "Plain Instant")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("plain-instant".to_string()))
        .with_types(vec![CardType::Instant]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry((*registry).clone())
        .object(ss_card)
        .object(plain_card)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let ss_id = find_object(&state, "Split Second Instant");

    // P1 casts the split second spell.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ss_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap();

    assert_eq!(state.stack_objects.len(), 1, "split second spell on stack");

    // Both players pass priority — spell resolves and leaves the stack.
    let (state, _) = pass_all(state, &[p1, p2]);

    // Stack should now be empty (split second spell resolved).
    assert_eq!(
        state.stack_objects.len(),
        0,
        "stack should be empty after split second spell resolves"
    );

    // P1 is the active player. Give p2 priority to test that a non-active player
    // can cast after split second ends. P2 can cast instants at any time.
    let mut state = state;
    state.turn.priority_holder = Some(p2);

    let plain_id = find_object(&state, "Plain Instant");

    // P2 casts the plain instant — should now succeed (split second is gone from stack).
    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: plain_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.61a: After split second spell resolves, new spells should be castable. Got: {:?}",
        result.err()
    );
}

// ── Test 8: Triggered abilities still fire ─────────────────────────────────────

#[test]
/// CR 702.61b — Triggered abilities trigger and are put on the stack as normal
/// while a spell with split second is on the stack.
/// A creature with a triggered ability "whenever a spell is cast" should trigger
/// when P1 casts the split second spell.
fn test_split_second_triggered_abilities_still_fire() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = Arc::new(CardRegistry::new(vec![split_second_instant_def()]));

    let ss_card = ObjectSpec::card(p1, "Split Second Instant")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("split-second-instant".to_string()))
        .with_keyword(KeywordAbility::SplitSecond);

    // P1 has a creature on the battlefield with a "whenever a spell is cast" trigger.
    let creature = ObjectSpec::creature(p1, "Spell Watcher", 1, 1)
        .with_triggered_ability(TriggeredAbilityDef {
            trigger_on: TriggerEvent::AnySpellCast,
            intervening_if: None,
            description:
                "Whenever a spell is cast, this creature gets +1/+1 until end of turn (stub)"
                    .to_string(),
            effect: None,
        })
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry((*registry).clone())
        .object(ss_card)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let ss_id = find_object(&state, "Split Second Instant");

    // P1 casts the split second spell — should trigger the "whenever a spell is cast" ability.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: ss_id,
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
            cast_with_jump_start: false,
            jump_start_discard: None,
            cast_with_aftermath: false,
            cast_with_dash: false,
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell (split second) failed: {:?}", e));

    // After casting, priority goes to active player (p1). Pass priority to flush triggers.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();

    // The triggered ability should now be on the stack (above the split second spell).
    // Stack should have: [split second spell (bottom), triggered ability (top)].
    assert!(
        state.stack_objects.len() >= 2,
        "CR 702.61b: triggered ability should be on the stack above split second spell; \
         stack_objects.len() = {}",
        state.stack_objects.len()
    );

    // There should be a triggered ability (TriggeredAbility kind) on the stack.
    let has_trigger = state
        .stack_objects
        .iter()
        .any(|so| matches!(so.kind, StackObjectKind::TriggeredAbility { .. }));
    assert!(
        has_trigger,
        "CR 702.61b: a triggered ability should be on the stack while split second spell is present"
    );
}
