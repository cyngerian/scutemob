//! Tests for graveyard-targeted spells and abilities (CR 115.1, CR 608.2b).
//!
//! PB-10: TargetCardInYourGraveyard / TargetCardInGraveyard validation,
//! resolution with MoveZone, filter matching (card type, subtype OR),
//! fizzle when target leaves graveyard.

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Effect, EffectTarget, PlayerTarget, TargetFilter,
    TargetRequirement, TypeLine, ZoneTarget,
};
use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::turn::Step;
use mtg_engine::state::{
    CardId, CardType, GameStateBuilder, ManaPool, ObjectSpec, PlayerId, SubType, Target, ZoneId,
};
use mtg_engine::{CardRegistry, GameState, ObjectId};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn st(s: &str) -> SubType {
    SubType(s.to_string())
}

/// Pass priority for all four players in order.
fn pass_all_four(
    state: GameState,
    turn_order: [PlayerId; 4],
) -> (GameState, Vec<GameEvent>) {
    let mut s = state;
    let mut all_events = Vec::new();
    for player in &turn_order {
        let (ns, evs) = process_command(s, Command::PassPriority { player: *player }).unwrap();
        all_events.extend(evs);
        s = ns;
    }
    (s, all_events)
}

/// Build a CardDefinition for a spell that targets a card in graveyard
/// with the given filter, and moves it to the given destination zone.
fn return_from_gy_spell(
    name: &str,
    card_id: &str,
    filter: TargetFilter,
    to: ZoneTarget,
    any_graveyard: bool,
    is_instant: bool,
) -> CardDefinition {
    let target_req = if any_graveyard {
        TargetRequirement::TargetCardInGraveyard(filter)
    } else {
        TargetRequirement::TargetCardInYourGraveyard(filter)
    };
    let card_type = if is_instant {
        CardType::Instant
    } else {
        CardType::Sorcery
    };
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(card_id.to_string()),
        mana_cost: Some(ManaCost {
            black: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![card_type],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to,
            },
            targets: vec![target_req],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

fn default_cast(player: PlayerId, card: ObjectId, targets: Vec<Target>) -> Command {
    Command::CastSpell {
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
    }
}

// ---------------------------------------------------------------------------
// CR 115.1: Target validation at cast time — graveyard zone
// ---------------------------------------------------------------------------

#[test]
/// CR 115.1 — targeting a creature card in your graveyard is valid at cast time.
fn test_115_1_target_creature_in_your_graveyard_valid() {
    let p1 = p(1);
    let spell_def = return_from_gy_spell(
        "Raise Dead",
        "raise_dead",
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        ZoneTarget::Battlefield { tapped: false },
        false,
        false,
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Raise Dead")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    let dead_creature = ObjectSpec::creature(p1, "Dead Bear", 2, 2)
        .in_zone(ZoneId::Graveyard(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(dead_creature)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let creature_id = *state.zones.get(&ZoneId::Graveyard(p1)).unwrap().object_ids().first().unwrap();

    let result = process_command(state, default_cast(p1, spell_id, vec![Target::Object(creature_id)]));
    assert!(result.is_ok(), "targeting a creature in your graveyard should succeed");
    let (new_state, events) = result.unwrap();
    assert_eq!(new_state.stack_objects.len(), 1);
    assert!(events.iter().any(|e| matches!(e, GameEvent::SpellCast { .. })));
}

#[test]
/// CR 115.1, CR 608.2b — cast and resolve: creature returns from graveyard to battlefield.
fn test_115_1_resolve_return_creature_from_gy_to_battlefield() {
    let p1 = p(1);
    let spell_def = return_from_gy_spell(
        "Raise Dead",
        "raise_dead",
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            ..Default::default()
        },
        ZoneTarget::Battlefield { tapped: false },
        false,
        false,
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Raise Dead")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    let dead_creature = ObjectSpec::creature(p1, "Dead Bear", 2, 2)
        .in_zone(ZoneId::Graveyard(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(dead_creature)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let creature_id = *state.zones.get(&ZoneId::Graveyard(p1)).unwrap().object_ids().first().unwrap();

    let (state, _) = process_command(state, default_cast(p1, spell_id, vec![Target::Object(creature_id)])).unwrap();

    // Resolve.
    let turn_order = [p(1), p(2), p(3), p(4)];
    let (state, _) = pass_all_four(state, turn_order);

    // Creature should be on the battlefield (new ObjectId — CR 400.7).
    let bf_bears: Vec<_> = state.objects.values()
        .filter(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Dead Bear")
        .collect();
    assert_eq!(bf_bears.len(), 1, "Dead Bear should be on the battlefield after resolve");

    // Graveyard should have only the sorcery.
    let gy_names: Vec<_> = state.zones.get(&ZoneId::Graveyard(p1)).unwrap().object_ids().iter()
        .filter_map(|id| state.objects.get(id))
        .map(|o| o.characteristics.name.as_str())
        .collect();
    assert!(gy_names.contains(&"Raise Dead"), "Sorcery should be in graveyard");
    assert!(!gy_names.contains(&"Dead Bear"), "Dead Bear should not be in graveyard");
}

#[test]
/// CR 115.1 — TargetCardInYourGraveyard rejects cards in opponent's graveyard.
fn test_115_1_your_gy_rejects_opponent_gy_card() {
    let p1 = p(1);
    let p2 = p(2);
    let spell_def = return_from_gy_spell(
        "Raise Dead", "raise_dead",
        TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
        ZoneTarget::Battlefield { tapped: false },
        false, false,
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Raise Dead")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    // Creature in p2's graveyard.
    let dead = ObjectSpec::creature(p2, "Enemy Bear", 2, 2).in_zone(ZoneId::Graveyard(p2));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(dead)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let creature_id = *state.zones.get(&ZoneId::Graveyard(p2)).unwrap().object_ids().first().unwrap();

    let result = process_command(state, default_cast(p1, spell_id, vec![Target::Object(creature_id)]));
    assert!(result.is_err(), "TargetCardInYourGraveyard should reject opponent's graveyard");
}

#[test]
/// CR 115.1 — TargetCardInGraveyard (any) accepts opponent's graveyard.
fn test_115_1_any_gy_accepts_opponent_gy_card() {
    let p1 = p(1);
    let p2 = p(2);
    let spell_def = return_from_gy_spell(
        "Reanimate", "reanimate",
        TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
        ZoneTarget::Battlefield { tapped: false },
        true, false, // any graveyard
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Reanimate")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    let dead = ObjectSpec::creature(p2, "Enemy Dragon", 5, 5).in_zone(ZoneId::Graveyard(p2));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(dead)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let creature_id = *state.zones.get(&ZoneId::Graveyard(p2)).unwrap().object_ids().first().unwrap();

    let result = process_command(state, default_cast(p1, spell_id, vec![Target::Object(creature_id)]));
    assert!(result.is_ok(), "TargetCardInGraveyard should accept any player's graveyard");
}

#[test]
/// CR 115.1 — filter rejects wrong card type (creature filter vs artifact in GY).
fn test_115_1_filter_rejects_wrong_card_type() {
    let p1 = p(1);
    let spell_def = return_from_gy_spell(
        "Raise Dead", "raise_dead",
        TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
        ZoneTarget::Battlefield { tapped: false },
        false, false,
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Raise Dead")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    let dead_artifact = ObjectSpec::card(p1, "Dead Artifact")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Graveyard(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(dead_artifact)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let artifact_id = *state.zones.get(&ZoneId::Graveyard(p1)).unwrap().object_ids().first().unwrap();

    let result = process_command(state, default_cast(p1, spell_id, vec![Target::Object(artifact_id)]));
    assert!(result.is_err(), "creature filter should reject artifact cards in graveyard");
}

#[test]
/// CR 115.1 — graveyard targeting rejects cards on the battlefield.
fn test_115_1_gy_targeting_rejects_battlefield_card() {
    let p1 = p(1);
    let spell_def = return_from_gy_spell(
        "Raise Dead", "raise_dead",
        TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
        ZoneTarget::Battlefield { tapped: false },
        false, false,
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Raise Dead")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    let living = ObjectSpec::creature(p1, "Living Bear", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(living)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let creature_id = state.objects.values()
        .find(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Living Bear")
        .unwrap().id;

    let result = process_command(state, default_cast(p1, spell_id, vec![Target::Object(creature_id)]));
    assert!(result.is_err(), "graveyard targeting should reject battlefield cards");
}

#[test]
/// PB-10 — has_subtypes OR-semantics: Wizard matches "Vampire or Wizard" filter.
fn test_has_subtypes_or_filter_matches() {
    let p1 = p(1);
    let spell_def = return_from_gy_spell(
        "Necro Recall", "necro_recall",
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            has_subtypes: vec![st("Vampire"), st("Wizard")],
            ..Default::default()
        },
        ZoneTarget::Battlefield { tapped: false },
        false, false,
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Necro Recall")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    let dead_wizard = ObjectSpec::creature(p1, "Dead Wizard", 1, 1)
        .with_subtypes(vec![st("Human"), st("Wizard")])
        .in_zone(ZoneId::Graveyard(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(dead_wizard)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let wizard_id = *state.zones.get(&ZoneId::Graveyard(p1)).unwrap().object_ids().first().unwrap();

    let result = process_command(state, default_cast(p1, spell_id, vec![Target::Object(wizard_id)]));
    assert!(result.is_ok(), "Wizard should match 'Vampire or Wizard' OR filter");
}

#[test]
/// PB-10 — has_subtypes OR-semantics: Goblin is rejected by "Vampire or Wizard" filter.
fn test_has_subtypes_or_filter_rejects_no_match() {
    let p1 = p(1);
    let spell_def = return_from_gy_spell(
        "Necro Recall", "necro_recall",
        TargetFilter {
            has_card_type: Some(CardType::Creature),
            has_subtypes: vec![st("Vampire"), st("Wizard")],
            ..Default::default()
        },
        ZoneTarget::Battlefield { tapped: false },
        false, false,
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Necro Recall")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    let dead_goblin = ObjectSpec::creature(p1, "Dead Goblin", 1, 1)
        .with_subtypes(vec![st("Goblin")])
        .in_zone(ZoneId::Graveyard(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(dead_goblin)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let goblin_id = *state.zones.get(&ZoneId::Graveyard(p1)).unwrap().object_ids().first().unwrap();

    let result = process_command(state, default_cast(p1, spell_id, vec![Target::Object(goblin_id)]));
    assert!(result.is_err(), "Goblin should be rejected by 'Vampire or Wizard' filter");
}

#[test]
/// CR 115.1 — return artifact from graveyard to hand (Buried Ruin pattern).
fn test_115_1_return_artifact_to_hand() {
    let p1 = p(1);
    let spell_def = return_from_gy_spell(
        "Artifact Recovery", "artifact_recovery",
        TargetFilter { has_card_type: Some(CardType::Artifact), ..Default::default() },
        ZoneTarget::Hand { owner: PlayerTarget::Controller },
        false, false,
    );
    let spell_cid = spell_def.card_id.clone();
    let registry = CardRegistry::new(vec![spell_def]);

    let spell = ObjectSpec::card(p1, "Artifact Recovery")
        .with_card_id(spell_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));

    let dead_artifact = ObjectSpec::card(p1, "Sol Ring")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Graveyard(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 1, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(spell)
        .object(dead_artifact)
        .with_registry(registry)
        .build()
        .unwrap();

    let spell_id = *state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().first().unwrap();
    let artifact_id = *state.zones.get(&ZoneId::Graveyard(p1)).unwrap().object_ids().first().unwrap();

    let (state, _) = process_command(state, default_cast(p1, spell_id, vec![Target::Object(artifact_id)])).unwrap();

    let turn_order = [p(1), p(2), p(3), p(4)];
    let (state, _) = pass_all_four(state, turn_order);

    let hand_names: Vec<_> = state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().iter()
        .filter_map(|id| state.objects.get(id))
        .map(|o| o.characteristics.name.as_str())
        .collect();
    assert!(hand_names.contains(&"Sol Ring"), "Sol Ring should be returned to hand: got {:?}", hand_names);
}

#[test]
/// CR 608.2b — spell fizzles when graveyard target leaves before resolution.
fn test_608_2b_fizzle_gy_target_exiled_before_resolution() {
    let p1 = p(1);
    let raise_def = return_from_gy_spell(
        "Raise Dead", "raise_dead",
        TargetFilter { has_card_type: Some(CardType::Creature), ..Default::default() },
        ZoneTarget::Battlefield { tapped: false },
        false, false,
    );
    let raise_cid = raise_def.card_id.clone();

    let exile_def = CardDefinition {
        name: "Exile GY".to_string(),
        card_id: CardId("exile_gy".to_string()),
        mana_cost: Some(ManaCost { black: 1, ..ManaCost::default() }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to: ZoneTarget::Exile,
            },
            targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };
    let exile_cid = exile_def.card_id.clone();
    let registry = CardRegistry::new(vec![raise_def, exile_def]);

    let raise = ObjectSpec::card(p1, "Raise Dead")
        .with_card_id(raise_cid)
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));
    let exile = ObjectSpec::card(p1, "Exile GY")
        .with_card_id(exile_cid)
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost { black: 1, ..ManaCost::default() })
        .in_zone(ZoneId::Hand(p1));
    let dead = ObjectSpec::creature(p1, "Dead Bear", 2, 2).in_zone(ZoneId::Graveyard(p1));

    let state = GameStateBuilder::four_player()
        .player_mana(p1, ManaPool { black: 2, ..ManaPool::default() })
        .active_player(p1)
        .at_step(Step::PreCombatMain) // main phase for sorcery-speed Raise Dead
        .object(raise)
        .object(exile)
        .object(dead)
        .with_registry(registry)
        .build()
        .unwrap();

    let hand_ids: Vec<ObjectId> = state.zones.get(&ZoneId::Hand(p1)).unwrap().object_ids().iter().copied().collect();
    let raise_id = hand_ids.iter()
        .find(|id| state.objects.get(id).unwrap().characteristics.name == "Raise Dead")
        .copied().unwrap();
    let exile_id = hand_ids.iter()
        .find(|id| state.objects.get(id).unwrap().characteristics.name == "Exile GY")
        .copied().unwrap();
    let creature_id = *state.zones.get(&ZoneId::Graveyard(p1)).unwrap().object_ids().first().unwrap();

    // Cast Raise Dead first (bottom of stack).
    let (state, _) = process_command(state, default_cast(p1, raise_id, vec![Target::Object(creature_id)])).unwrap();
    // Cast Exile GY (top of stack — resolves first).
    let (state, _) = process_command(state, default_cast(p1, exile_id, vec![Target::Object(creature_id)])).unwrap();
    assert_eq!(state.stack_objects.len(), 2);

    let turn_order = [p(1), p(2), p(3), p(4)];
    // Resolve Exile GY — creature moves to exile.
    let (state, events1) = pass_all_four(state, turn_order);
    assert_eq!(state.stack_objects.len(), 1, "one spell remaining");

    // Resolve Raise Dead — target gone from GY → should fizzle.
    let (state, events2) = pass_all_four(state, turn_order);
    assert!(state.stack_objects.is_empty());

    let all_events: Vec<_> = events1.into_iter().chain(events2).collect();
    assert!(
        all_events.iter().any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "Raise Dead should fizzle because target left graveyard"
    );

    let bf_has_bear = state.objects.values()
        .any(|o| o.zone == ZoneId::Battlefield && o.characteristics.name == "Dead Bear");
    assert!(!bf_has_bear, "Dead Bear should NOT be on the battlefield");
}
