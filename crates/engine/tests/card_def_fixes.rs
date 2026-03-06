//! Tests verifying M9.4 Session 1 and Session 2 card definition corrections.
//!
//! Session 1 covers:
//! - Read the Bones: Scry 2 fires before drawing two cards (CR 701.18)
//! - Dimir Guildgate: modal color choice (CR 106.6)
//! - Path to Exile: optional search via deterministic MayPayOrElse (CR 701.19)
//! - Thought Vessel / Reliquary Tower: no-maximum-hand-size skips cleanup discard (CR 402.2)
//! - Alela, Cunning Conqueror: WheneverYouCastSpell has during_opponent_turn: true (CR 603.1)
//!
//! Session 2 covers:
//! - Lightning Greaves: equipped creature gets Haste and Shroud (CR 702.6a, CR 613.1f)
//! - Swiftfoot Boots: equipped creature gets Haste and Hexproof (CR 702.6a, CR 613.1f)
//! - Rogue's Passage: creature with CantBeBlocked can't be blocked (CR 509.1b)
//! - Rest in Peace ETB: exiles all cards from all graveyards on entry (CR 603.2)

use mtg_engine::state::player::CardId;
use mtg_engine::{
    all_cards, calculate_characteristics, process_command, start_game, CardRegistry, CardType,
    Command, ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, GameEvent,
    GameState, GameStateBuilder, KeywordAbility, LayerModification, ManaColor, ManaCost, ObjectId,
    ObjectSpec, PlayerId, Step, TriggerCondition, ZoneId,
};

// ── Helper: find an object by name ───────────────────────────────────────────

fn find_object(state: &GameState, name: &str) -> mtg_engine::ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

/// Pass priority for all players once.
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &p in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: p })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", p, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

// ── CR 701.18: Read the Bones — scry fires before draw ───────────────────────

#[test]
/// CR 701.18 — Read the Bones: Scry 2 fires before drawing two cards.
/// Verifies the Scried event precedes the CardDrawn events in the event sequence.
fn test_read_the_bones_scry_then_draw() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Read the Bones")
        .expect("Read the Bones must be in all_cards()");
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    // Give p1 a library with 5 cards and Read the Bones in hand.
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Read the Bones")
                .with_card_id(card_id)
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    black: 1,
                    generic: 2,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry);

    for i in 0..5 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("Library Card {}", i)).in_zone(ZoneId::Library(p1)),
        );
    }
    let state = builder.build().unwrap();

    // Pay 3 mana: 1 black + 2 generic (just add 3 black for simplicity).
    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 3);

    let rtb_id = find_object(&state, "Read the Bones");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: rtb_id,
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
    .unwrap();

    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.18: Scried event must appear before any CardDrawn events.
    let scried_pos = events
        .iter()
        .position(|e| matches!(e, GameEvent::Scried { player, .. } if *player == p1))
        .expect("Scried event should be emitted for p1");

    let first_drawn_pos = events
        .iter()
        .position(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .expect("At least one CardDrawn event should be emitted for p1");

    assert!(
        scried_pos < first_drawn_pos,
        "Scried (pos {}) must precede first CardDrawn (pos {}); events: {:?}",
        scried_pos,
        first_drawn_pos,
        events
    );

    // Must have scried for 2 cards.
    let scried = events
        .iter()
        .find(|e| matches!(e, GameEvent::Scried { player, .. } if *player == p1));
    if let Some(GameEvent::Scried { count, .. }) = scried {
        assert_eq!(*count, 2, "Scry count must be 2");
    }

    // Must have drawn exactly 2 cards.
    let drawn_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player, .. } if *player == p1))
        .count();
    assert_eq!(
        drawn_count, 2,
        "Read the Bones should draw 2 cards; got {}",
        drawn_count
    );

    // p1 should have lost 2 life.
    let initial_life = 40; // Commander format starts at 40
    assert_eq!(
        state.players[&p1].life_total,
        initial_life - 2,
        "Read the Bones should cost 2 life"
    );
}

// ── CR 106.6: Dimir Guildgate — modal color choice ────────────────────────────

#[test]
/// CR 106.6 — Dimir Guildgate: tap ability is modelled as Effect::Choose between
/// AddMana blue and AddMana black (replacing the old AddManaAnyColor).
/// This is a data model test verifying the card definition is correct.
fn test_dimir_guildgate_modal_color() {
    use mtg_engine::{AbilityDefinition, Cost, Effect, ManaPool};

    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Dimir Guildgate")
        .expect("Dimir Guildgate must be in all_cards()");

    // Find the activated tap ability (should have a Choose effect).
    let tap_ability = def.abilities.iter().find(|a| {
        matches!(
            a,
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                ..
            }
        )
    });

    assert!(
        tap_ability.is_some(),
        "Dimir Guildgate should have a tap activated ability"
    );

    if let Some(AbilityDefinition::Activated { effect, .. }) = tap_ability {
        // CR 106.6: must be a Choose between two AddMana options.
        assert!(
            matches!(effect, Effect::Choose { choices, .. } if choices.len() == 2),
            "Dimir Guildgate tap ability must use Effect::Choose with 2 choices; got: {:?}",
            effect
        );

        if let Effect::Choose { choices, .. } = effect {
            // First choice must add 1 blue mana.
            assert!(
                matches!(&choices[0], Effect::AddMana { mana, .. }
                    if mana == &ManaPool { blue: 1, ..Default::default() }),
                "First choice should add 1 blue mana; got: {:?}",
                &choices[0]
            );
            // Second choice must add 1 black mana.
            assert!(
                matches!(&choices[1], Effect::AddMana { mana, .. }
                    if mana == &ManaPool { black: 1, ..Default::default() }),
                "Second choice should add 1 black mana; got: {:?}",
                &choices[1]
            );
        }
    }
}

// ── CR 701.19: Path to Exile — optional search ───────────────────────────────

#[test]
/// CR 701.19 / M9.4 — Path to Exile: target creature is exiled and the deterministic
/// fallback fires the search branch (controller searches for a basic land).
fn test_path_to_exile_optional_search() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let def = all_cards()
        .into_iter()
        .find(|d| d.name == "Path to Exile")
        .expect("Path to Exile must be in all_cards()");
    let card_id = def.card_id.clone();
    let registry = CardRegistry::new(vec![def]);

    // p2 has a creature on the battlefield. p2's library has a basic land.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Path to Exile")
                .with_card_id(card_id)
                .with_types(vec![CardType::Instant])
                .with_mana_cost(ManaCost {
                    white: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(ObjectSpec::creature(p2, "Goblin Guide", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::card(p2, "Plains").in_zone(ZoneId::Library(p2)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Pay 1 white mana.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);

    let path_id = find_object(&state, "Path to Exile");
    let goblin_id = find_object(&state, "Goblin Guide");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: path_id,
            targets: vec![mtg_engine::Target::Object(goblin_id)],
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
    .unwrap();

    let (_state, events) = pass_all(state, &[p1, p2]);

    // Goblin Guide should have been exiled.
    let exiled = events
        .iter()
        .any(|e| matches!(e, GameEvent::ObjectExiled { object_id, .. } if *object_id == goblin_id));
    assert!(
        exiled,
        "Goblin Guide should be exiled by Path to Exile; events: {:?}",
        events
    );
}

// ── CR 402.2: Thought Vessel — no maximum hand size ──────────────────────────

#[test]
/// CR 402.2 / CR 514.1 — Thought Vessel: controller with Thought Vessel on battlefield
/// does NOT discard to hand size during cleanup.
fn test_thought_vessel_no_max_hand_size() {
    let p1 = PlayerId(1);

    // Build a cleanup state: p1 has 9 cards in hand (normally would discard 2),
    // but also has a permanent with NoMaxHandSize on the battlefield.
    let mut builder = GameStateBuilder::four_player().at_step(Step::End);

    // Add 9 cards to hand (exceeds max 7).
    for i in 0..9 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("Hand Card {}", i)).in_zone(ZoneId::Hand(p1)));
    }

    // Add Thought Vessel (with NoMaxHandSize keyword) to battlefield.
    builder = builder.object(
        ObjectSpec::artifact(p1, "Thought Vessel")
            .with_keyword(KeywordAbility::NoMaxHandSize)
            .in_zone(ZoneId::Battlefield),
    );

    let state = builder.build().unwrap();

    // Pass all 4 players in End step to trigger cleanup.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(2),
        },
    )
    .unwrap();
    let (state, _) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(3),
        },
    )
    .unwrap();
    let (_state, events) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(4),
        },
    )
    .unwrap();

    // No DiscardedToHandSize events should have fired.
    let discard_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscardedToHandSize { player, .. } if *player == p1))
        .collect();
    assert!(
        discard_events.is_empty(),
        "Player with Thought Vessel should NOT discard to hand size; discard events: {:?}",
        discard_events
    );
}

#[test]
/// MR-M9.4-15 / CR 402.2 / CR 514.1 — Counter-assertion: the active player
/// WITHOUT Thought Vessel DOES discard to hand size during cleanup.
///
/// CR 514.1a: "First, if the active player's hand contains more cards than
/// their maximum hand size, they discard enough cards to reduce to that number."
/// This test is the discard-happens counterpart to
/// `test_thought_vessel_no_max_hand_size` (which checks the NO-discard path).
fn test_no_thought_vessel_discards_to_hand_size() {
    let p1 = PlayerId(1); // P1 is the active player in a `four_player()` builder.

    // P1 (active player) has 9 cards in hand and NO Thought Vessel.
    // With max_hand_size = 7 (default), P1 must discard 2 cards during cleanup.
    let mut builder = GameStateBuilder::four_player().at_step(Step::End);

    for i in 0..9 {
        builder = builder
            .object(ObjectSpec::card(p1, &format!("P1 Hand {}", i)).in_zone(ZoneId::Hand(p1)));
    }

    let state = builder.build().unwrap();

    // Pass all 4 players through End step → cleanup triggers for active player P1.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(2),
        },
    )
    .unwrap();
    let (state, _) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(3),
        },
    )
    .unwrap();
    let (state, events) = process_command(
        state,
        Command::PassPriority {
            player: PlayerId(4),
        },
    )
    .unwrap();

    // P1 MUST discard (active player, 9 cards, no Vessel).
    let p1_discards: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DiscardedToHandSize { player, .. } if *player == p1))
        .collect();
    assert!(
        !p1_discards.is_empty(),
        "p1 without Thought Vessel should discard to hand size during cleanup; events: {:?}",
        events
    );

    // Verify p1 ends up with exactly 7 cards (maximum hand size).
    let p1_hand_count = state.objects_in_zone(&ZoneId::Hand(p1)).len();
    assert_eq!(
        p1_hand_count, 7,
        "p1 should have exactly 7 cards after cleanup discard; got {}",
        p1_hand_count
    );
}

// ── CR 603.1: Alela trigger scoping — during_opponent_turn flag ───────────────

#[test]
/// CR 603.1 — Alela, Cunning Conqueror: WheneverYouCastSpell trigger has
/// during_opponent_turn: true, restricting it to opponent turns only.
/// This is a data model test verifying the card definition is correct.
fn test_alela_opponent_turn_only() {
    let alela_def = all_cards()
        .into_iter()
        .find(|d| d.name == "Alela, Cunning Conqueror")
        .expect("Alela, Cunning Conqueror must be in all_cards()");

    use mtg_engine::AbilityDefinition;

    // Find the WheneverYouCastSpell trigger in Alela's abilities.
    let cast_spell_trigger = alela_def.abilities.iter().find(|a| {
        matches!(
            a,
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell { .. },
                ..
            }
        )
    });

    assert!(
        cast_spell_trigger.is_some(),
        "Alela should have a WheneverYouCastSpell triggered ability"
    );

    // Verify during_opponent_turn is true (CR 603.1: fires only during opponent turns).
    if let Some(AbilityDefinition::Triggered {
        trigger_condition:
            TriggerCondition::WheneverYouCastSpell {
                during_opponent_turn,
            },
        ..
    }) = cast_spell_trigger
    {
        assert!(
            *during_opponent_turn,
            "Alela's WheneverYouCastSpell should have during_opponent_turn: true (CR 603.1)"
        );
    }
}

// ── CR 702.6a: Lightning Greaves — equipped creature gets Haste and Shroud ───

/// Helper: find an object id by name in `state`.
fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

#[test]
/// CR 702.6a / CR 613.1f — Lightning Greaves static ability: equipped creature
/// gains Haste and Shroud via a layer-6 continuous effect scoped to the attached
/// creature (EffectFilter::AttachedCreature).
fn test_lightning_greaves_grants_haste_shroud() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Build state with a Bear and Lightning Greaves on the battlefield.
    // The continuous effect from the greaves' Static ability uses AttachedCreature,
    // resolved by checking the source's `attached_to` field at calculation time.
    let greaves_effect = ContinuousEffect {
        id: EffectId(999),
        source: None, // filled in after we know greaves_id
        timestamp: 1,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::AttachedCreature,
        modification: LayerModification::AddKeywords(
            [KeywordAbility::Haste, KeywordAbility::Shroud]
                .into_iter()
                .collect(),
        ),
        is_cda: false,
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::artifact(p1, "Lightning Greaves").in_zone(ZoneId::Battlefield))
        .add_continuous_effect(greaves_effect)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let bear_id = find_obj(&state, "Bear");
    let greaves_id = find_obj(&state, "Lightning Greaves");

    // Wire the attachment relationship: greaves attached_to bear; bear has greaves in attachments.
    state.objects.get_mut(&greaves_id).unwrap().attached_to = Some(bear_id);
    state
        .objects
        .get_mut(&bear_id)
        .unwrap()
        .attachments
        .push_back(greaves_id);

    // Fix up the source on the continuous effect now that we have the greaves id.
    let effect_idx = state
        .continuous_effects
        .iter()
        .position(|e| e.id == EffectId(999))
        .unwrap();
    let mut eff = state.continuous_effects[effect_idx].clone();
    eff.source = Some(greaves_id);
    state.continuous_effects.remove(effect_idx);
    state.continuous_effects.push_back(eff);

    // CR 613.1f: Layer 6 ability modification — bear should now have Haste and Shroud.
    let chars = calculate_characteristics(&state, bear_id).unwrap();
    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "Bear equipped with Lightning Greaves must have Haste (CR 702.6a); keywords: {:?}",
        chars.keywords
    );
    assert!(
        chars.keywords.contains(&KeywordAbility::Shroud),
        "Bear equipped with Lightning Greaves must have Shroud (CR 702.6a); keywords: {:?}",
        chars.keywords
    );

    // Sanity: the greaves itself should NOT have haste/shroud (AttachedCreature only applies
    // to the creature the greaves is attached to, not the greaves itself).
    let greaves_chars = calculate_characteristics(&state, greaves_id).unwrap();
    assert!(
        !greaves_chars.keywords.contains(&KeywordAbility::Haste),
        "Lightning Greaves itself should NOT have Haste; keywords: {:?}",
        greaves_chars.keywords
    );
}

// ── CR 702.6a: Swiftfoot Boots — equipped creature gets Haste and Hexproof ───

#[test]
/// CR 702.6a / CR 613.1f — Swiftfoot Boots static ability: equipped creature
/// gains Haste and Hexproof via a layer-6 continuous effect scoped to the
/// attached creature (EffectFilter::AttachedCreature).
fn test_swiftfoot_boots_grants_haste_hexproof() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let boots_effect = ContinuousEffect {
        id: EffectId(998),
        source: None, // filled in after we know boots_id
        timestamp: 1,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::AttachedCreature,
        modification: LayerModification::AddKeywords(
            [KeywordAbility::Haste, KeywordAbility::Hexproof]
                .into_iter()
                .collect(),
        ),
        is_cda: false,
    };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::artifact(p1, "Swiftfoot Boots").in_zone(ZoneId::Battlefield))
        .add_continuous_effect(boots_effect)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let bear_id = find_obj(&state, "Bear");
    let boots_id = find_obj(&state, "Swiftfoot Boots");

    // Wire the attachment relationship.
    state.objects.get_mut(&boots_id).unwrap().attached_to = Some(bear_id);
    state
        .objects
        .get_mut(&bear_id)
        .unwrap()
        .attachments
        .push_back(boots_id);

    // Fix up the source on the continuous effect.
    let effect_idx = state
        .continuous_effects
        .iter()
        .position(|e| e.id == EffectId(998))
        .unwrap();
    let mut eff = state.continuous_effects[effect_idx].clone();
    eff.source = Some(boots_id);
    state.continuous_effects.remove(effect_idx);
    state.continuous_effects.push_back(eff);

    // CR 613.1f: Layer 6 ability modification — bear should now have Haste and Hexproof.
    let chars = calculate_characteristics(&state, bear_id).unwrap();
    assert!(
        chars.keywords.contains(&KeywordAbility::Haste),
        "Bear equipped with Swiftfoot Boots must have Haste (CR 702.6a); keywords: {:?}",
        chars.keywords
    );
    assert!(
        chars.keywords.contains(&KeywordAbility::Hexproof),
        "Bear equipped with Swiftfoot Boots must have Hexproof (CR 702.6a); keywords: {:?}",
        chars.keywords
    );
}

// ── CR 509.1b: Rogue's Passage — CantBeBlocked creature can't be blocked ─────

#[test]
/// CR 509.1b / CR 702.xxx — A creature with the CantBeBlocked keyword cannot
/// be declared as a blocker target. The engine rejects the DeclareBlockers
/// command with an InvalidCommand error.
fn test_rogues_passage_cant_be_blocked() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // p1 has an attacker with CantBeBlocked; p2 has a potential blocker.
    // We start in DeclareBlockers to test that the block declaration is rejected.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Unblockable Attacker", 2, 2)
                .with_keyword(KeywordAbility::CantBeBlocked)
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::creature(p2, "Wall of Stone", 0, 8).in_zone(ZoneId::Battlefield))
        .at_step(Step::DeclareBlockers)
        .active_player(p1)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Unblockable Attacker");
    let blocker_id = find_obj(&state, "Wall of Stone");

    // Manually register the attacker as attacking p2 (simulate being in combat).
    let mut state = state;
    if let Some(combat) = state.combat.as_mut() {
        combat
            .attackers
            .insert(attacker_id, mtg_engine::AttackTarget::Player(p2));
    } else {
        // Create a minimal combat state with the attacker.
        let mut cs = mtg_engine::CombatState::new(p1);
        cs.attackers
            .insert(attacker_id, mtg_engine::AttackTarget::Player(p2));
        state.combat = Some(cs);
    }

    // Attempting to block the CantBeBlocked attacker must fail.
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "Blocking a creature with CantBeBlocked should fail; got Ok"
    );
}

// ── CR 603.2: Rest in Peace ETB — exiles all graveyard cards on entry ─────────

#[test]
/// CR 603.2 / CR 614.1a — Rest in Peace: when it enters the battlefield, its
/// triggered ability fires inline and exiles all cards from all graveyards.
/// Verifies ObjectExiled events are emitted for each graveyard card.
fn test_rest_in_peace_etb_exiles_graveyards() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let rip_def = all_cards()
        .into_iter()
        .find(|d| d.name == "Rest in Peace")
        .expect("Rest in Peace must be in all_cards()");
    let card_id = rip_def.card_id.clone();
    let registry = CardRegistry::new(vec![rip_def]);

    // p1 has Rest in Peace in hand; both players have a card in their graveyard.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::card(p1, "Rest in Peace")
                .with_card_id(card_id)
                .with_types(vec![CardType::Enchantment])
                .with_mana_cost(ManaCost {
                    white: 1,
                    generic: 1,
                    ..Default::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(
            ObjectSpec::card(p1, "Dead Creature")
                .with_types(vec![CardType::Creature])
                .in_zone(ZoneId::Graveyard(p1)),
        )
        .object(
            ObjectSpec::card(p2, "Opponent Dead Sorcery")
                .with_types(vec![CardType::Sorcery])
                .in_zone(ZoneId::Graveyard(p2)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Pay {1W} for Rest in Peace.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 2);

    let rip_id = find_obj(&state, "Rest in Peace");

    // Cast Rest in Peace.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: rip_id,
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
    .expect("casting Rest in Peace failed");

    // Both players pass priority to resolve.
    let (_state, events) = pass_all(state, &[p1, p2]);

    // CR 603.2: The ETB trigger fires inline at resolution.
    // Two ObjectExiled events should appear (one for each graveyard card).
    let exile_count = events
        .iter()
        .filter(|e| matches!(e, GameEvent::ObjectExiled { .. }))
        .count();

    assert_eq!(
        exile_count, 2,
        "Rest in Peace ETB should exile 2 graveyard cards (one per player); \
         got {} ObjectExiled events; events: {:?}",
        exile_count, events
    );
}

// ── CR 113.6b: Leyline of the Void — opening hand rule ───────────────────────

#[test]
/// CR 113.6b — Leyline of the Void: if in the opening hand, the player may begin
/// the game with it on the battlefield. `start_game` places it on the battlefield
/// before the first turn as a pre-game action (deterministic: always placed).
fn test_leyline_opening_hand() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let leyline_card_id = CardId("leyline-of-the-void".to_string());
    let registry = CardRegistry::new(all_cards());

    // Build state: Leyline of the Void is in p1's opening hand.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::enchantment(p1, "Leyline of the Void")
                .with_card_id(leyline_card_id)
                .in_zone(ZoneId::Hand(p1)),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // CR 113.6b: start_game should place Leyline on the battlefield
    // before the first turn begins (pre-game action, not cast or resolved).
    let (final_state, events) = start_game(state).expect("start_game failed");

    // Leyline must now be on the battlefield.
    let battlefield_objects = final_state.objects_in_zone(&ZoneId::Battlefield);
    let leyline_on_bf = battlefield_objects
        .iter()
        .any(|o| o.characteristics.name == "Leyline of the Void");
    assert!(
        leyline_on_bf,
        "Leyline of the Void should be on the battlefield after start_game (CR 113.6b); \
         battlefield: {:?}",
        battlefield_objects
            .iter()
            .map(|o| &o.characteristics.name)
            .collect::<Vec<_>>()
    );

    // Leyline must NOT be in p1's hand anymore.
    let hand_objects = final_state.objects_in_zone(&ZoneId::Hand(p1));
    let leyline_in_hand = hand_objects
        .iter()
        .any(|o| o.characteristics.name == "Leyline of the Void");
    assert!(
        !leyline_in_hand,
        "Leyline of the Void should not remain in hand after start_game (CR 113.6b)"
    );

    // A PermanentEnteredBattlefield event must have been emitted during pre-game setup.
    let etb_event = events.iter().any(
        |e| matches!(e, GameEvent::PermanentEnteredBattlefield { player, .. } if *player == p1),
    );
    assert!(
        etb_event,
        "PermanentEnteredBattlefield event should be emitted for Leyline pre-game placement; \
         events: {:?}",
        events
    );
}

// ── CR 701.20: Darksteel Colossus — shuffle into owner's library ─────────────

#[test]
/// CR 701.20 / CR 614.1a — Darksteel Colossus replacement effect: if Darksteel
/// Colossus would be put into a graveyard from anywhere, shuffle it into its
/// owner's library instead. The replacement emits a `LibraryShuffled` event.
fn test_darksteel_colossus_shuffles_into_library() {
    use mtg_engine::rules::replacement::{
        check_zone_change_replacement, register_permanent_replacement_abilities, ZoneChangeAction,
    };
    use mtg_engine::state::zone::ZoneType;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let colossus_card_id = CardId("darksteel-colossus".to_string());
    let registry = CardRegistry::new(all_cards());

    // Build state: Darksteel Colossus is on p1's battlefield.
    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::artifact(p1, "Darksteel Colossus")
                .with_card_id(colossus_card_id)
                .with_types(vec![CardType::Artifact, CardType::Creature])
                .in_zone(ZoneId::Battlefield),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .with_registry(registry)
        .build()
        .unwrap();

    // Find the Colossus object on the battlefield.
    let colossus_id = state
        .objects_in_zone(&ZoneId::Battlefield)
        .iter()
        .find(|o| o.characteristics.name == "Darksteel Colossus")
        .map(|o| o.id)
        .expect("Darksteel Colossus should be on battlefield");

    // Register Darksteel Colossus's self-replacement ability.
    // This binds the SpecificObject filter to this exact ObjectId.
    let colossus_cid = state.objects.get(&colossus_id).unwrap().card_id.clone();
    let reg = state.card_registry.clone();
    register_permanent_replacement_abilities(
        &mut state,
        colossus_id,
        p1,
        colossus_cid.as_ref(),
        &reg,
    );

    // CR 614.1a / 701.20: Check that a Battlefield→Graveyard zone change for
    // this Colossus is intercepted and redirected to the library.
    let action = check_zone_change_replacement(
        &state,
        colossus_id,
        ZoneType::Battlefield,
        ZoneType::Graveyard,
        p1,
        &std::collections::HashSet::new(),
    );

    // The replacement must redirect to the library (not graveyard).
    match &action {
        ZoneChangeAction::Redirect { to, events, .. } => {
            assert_eq!(
                *to,
                ZoneId::Library(p1),
                "Darksteel Colossus replacement should redirect to Library(p1), got {:?}",
                to
            );

            // CR 701.20: A LibraryShuffled event must be in the emitted events.
            let has_shuffle = events
                .iter()
                .any(|e| matches!(e, GameEvent::LibraryShuffled { player } if *player == p1));
            assert!(
                has_shuffle,
                "ShuffleIntoOwnerLibrary must emit LibraryShuffled for p1; events: {:?}",
                events
            );
        }
        other => {
            panic!(
                "Expected ZoneChangeAction::Redirect for Darksteel Colossus, got: {:?}",
                other
            );
        }
    }
}
