/// Tests for tapped-and-attacking token creation (CR 508.4) and
/// EffectTarget::LastCreatedPermanent for equipment auto-attach.
///
/// CR 508.4: If a creature is put onto the battlefield attacking, its controller
/// chooses which defending player it's attacking (unless the effect specifies).
/// Such creatures are "attacking" but never "attacked" (no AttackersDeclared event).
///
/// CR 508.4c: These creatures are not affected by attack requirements/restrictions.
use mtg_engine::cards::card_definition::{EffectTarget, TokenSpec};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::events::GameEvent;
use mtg_engine::state::combat::{AttackTarget, CombatState};
use mtg_engine::state::game_object::ObjectId;
use mtg_engine::state::turn::{Phase, Step};
use mtg_engine::state::types::{CardType, Color, SubType};
use mtg_engine::state::zone::ZoneId;
use mtg_engine::state::{GameStateBuilder, ObjectSpec, PlayerId};
use mtg_engine::Effect;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Helper: create an EffectContext where the source is an attacker.
fn ec_attacker(controller: PlayerId, source: ObjectId) -> EffectContext {
    EffectContext::new(controller, source, vec![])
}

fn human_token_spec_attacking() -> TokenSpec {
    TokenSpec {
        name: "Human".to_string(),
        power: 1,
        toughness: 1,
        colors: [Color::Red].iter().copied().collect(),
        card_types: [CardType::Creature].iter().copied().collect(),
        subtypes: [SubType("Human".to_string())].iter().cloned().collect(),
        count: 2,
        tapped: true,
        enters_attacking: true,
        ..Default::default()
    }
}

// ── Tapped-and-Attacking Token Tests ─────────────────────────────────────────

#[test]
/// CR 508.4 — tokens created with enters_attacking are registered in combat.attackers.
/// Source: Hanweir Garrison "create two 1/1 red Human tokens tapped and attacking."
fn test_token_enters_tapped_and_attacking() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Attacker")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let attacker_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Attacker")
        .unwrap()
        .id;

    // Set up combat: p(1) attacking p(2), Attacker is declared as attacking.
    state.turn.phase = Phase::Combat;
    state.turn.step = Step::DeclareAttackers;
    let mut combat = CombatState::new(p(1));
    combat
        .attackers
        .insert(attacker_id, AttackTarget::Player(p(2)));
    state.combat = Some(combat);

    let effect = Effect::CreateToken {
        spec: human_token_spec_attacking(),
    };

    let mut ctx = ec_attacker(p(1), attacker_id);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    // Two tokens should have been created.
    let tokens: Vec<_> = state
        .objects
        .values()
        .filter(|o| o.characteristics.name == "Human" && o.is_token)
        .collect();
    assert_eq!(tokens.len(), 2, "should create 2 Human tokens");

    // Both tokens should be tapped.
    for token in &tokens {
        assert!(token.status.tapped, "token should be tapped");
    }

    // Both tokens should be registered as attacking p(2) in combat state.
    let combat = state.combat.as_ref().unwrap();
    for token in &tokens {
        assert!(
            combat.attackers.contains_key(&token.id),
            "token {:?} should be in combat.attackers",
            token.id
        );
        assert_eq!(
            combat.attackers.get(&token.id),
            Some(&AttackTarget::Player(p(2))),
            "token should be attacking p(2)"
        );
    }
}

#[test]
/// CR 508.4 — tokens with enters_attacking but no combat active: tokens enter
/// but are NOT registered as attacking.
fn test_token_enters_attacking_no_combat() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Source")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let source_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Source")
        .unwrap()
        .id;

    // No combat active.
    assert!(state.combat.is_none());

    let effect = Effect::CreateToken {
        spec: human_token_spec_attacking(),
    };

    let mut ctx = ec_attacker(p(1), source_id);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    // Tokens are created (on battlefield, tapped).
    let tokens: Vec<_> = state
        .objects
        .values()
        .filter(|o| o.characteristics.name == "Human" && o.is_token)
        .collect();
    assert_eq!(tokens.len(), 2, "should still create 2 tokens");
    for token in &tokens {
        assert!(
            token.status.tapped,
            "token should be tapped even without combat"
        );
    }

    // No combat state = no attacking registration.
    assert!(state.combat.is_none());
}

#[test]
/// CR 508.4a — if the source creature isn't attacking, tokens enter but are
/// not registered as attacking (no attack target to inherit).
fn test_token_enters_attacking_source_not_attacking() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "NonAttacker")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let source_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "NonAttacker")
        .unwrap()
        .id;

    // Combat is active but source is NOT in attackers.
    state.turn.phase = Phase::Combat;
    state.turn.step = Step::DeclareAttackers;
    let combat = CombatState::new(p(1));
    // No attackers declared.
    state.combat = Some(combat);

    let effect = Effect::CreateToken {
        spec: human_token_spec_attacking(),
    };

    let mut ctx = ec_attacker(p(1), source_id);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    // Tokens created but not registered as attacking.
    let tokens: Vec<_> = state
        .objects
        .values()
        .filter(|o| o.characteristics.name == "Human" && o.is_token)
        .collect();
    assert_eq!(tokens.len(), 2);

    let combat = state.combat.as_ref().unwrap();
    for token in &tokens {
        assert!(
            !combat.attackers.contains_key(&token.id),
            "token should NOT be in attackers when source isn't attacking"
        );
    }
}

#[test]
/// CR 508.4 — TokenCreated and PermanentEnteredBattlefield events are emitted.
fn test_token_enters_attacking_events_emitted() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Warrior")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Creature]),
        )
        .build()
        .unwrap();

    let warrior_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Warrior")
        .unwrap()
        .id;

    state.turn.phase = Phase::Combat;
    state.turn.step = Step::DeclareAttackers;
    let mut combat = CombatState::new(p(1));
    combat
        .attackers
        .insert(warrior_id, AttackTarget::Player(p(2)));
    state.combat = Some(combat);

    let effect = Effect::CreateToken {
        spec: TokenSpec {
            name: "Token".to_string(),
            power: 1,
            toughness: 1,
            colors: im::OrdSet::new(),
            card_types: [CardType::Creature].iter().copied().collect(),
            count: 1,
            tapped: true,
            enters_attacking: true,
            ..Default::default()
        },
    };

    let mut ctx = ec_attacker(p(1), warrior_id);
    let events = execute_effect(&mut state, &effect, &mut ctx);

    let token_created = events
        .iter()
        .any(|e| matches!(e, GameEvent::TokenCreated { .. }));
    let etb = events
        .iter()
        .any(|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }));
    assert!(token_created, "TokenCreated event should fire");
    assert!(etb, "PermanentEnteredBattlefield event should fire");
}

// ── EffectTarget::LastCreatedPermanent Tests ─────────────────────────────────

#[test]
/// EffectTarget::LastCreatedPermanent resolves to the most recently created token.
/// Source: Cryptic Coat "cloak the top card, then attach this Equipment to it."
fn test_last_created_permanent_after_create_token() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Equipment")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Artifact]),
        )
        .build()
        .unwrap();

    let equip_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Equipment")
        .unwrap()
        .id;

    // Create a token, then attach equipment to it via LastCreatedPermanent.
    let effect = Effect::Sequence(vec![
        Effect::CreateToken {
            spec: TokenSpec {
                name: "Germ".to_string(),
                power: 0,
                toughness: 0,
                colors: [Color::Black].iter().copied().collect(),
                card_types: [CardType::Creature].iter().copied().collect(),
                count: 1,
                ..Default::default()
            },
        },
        Effect::AttachEquipment {
            equipment: EffectTarget::Source,
            target: EffectTarget::LastCreatedPermanent,
        },
    ]);

    let mut ctx = ec_attacker(p(1), equip_id);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    // The token should have been created.
    let germ = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Germ" && o.is_token);
    assert!(germ.is_some(), "Germ token should exist");
    let germ = germ.unwrap();

    // Equipment should be attached to the Germ.
    let equip = state.objects.get(&equip_id).unwrap();
    assert_eq!(
        equip.attached_to,
        Some(germ.id),
        "Equipment should be attached to the Germ token"
    );
    assert!(
        germ.attachments.iter().any(|&id| id == equip_id),
        "Germ should have Equipment in its attachments"
    );
}

#[test]
/// EffectTarget::LastCreatedPermanent resolves to empty if no permanent was created.
fn test_last_created_permanent_none() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Equipment")
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Artifact]),
        )
        .build()
        .unwrap();

    let equip_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Equipment")
        .unwrap()
        .id;

    // Try to attach to LastCreatedPermanent without creating anything first.
    let effect = Effect::AttachEquipment {
        equipment: EffectTarget::Source,
        target: EffectTarget::LastCreatedPermanent,
    };

    let mut ctx = ec_attacker(p(1), equip_id);
    let _events = execute_effect(&mut state, &effect, &mut ctx);

    // Equipment should NOT be attached (no target found).
    let equip = state.objects.get(&equip_id).unwrap();
    assert_eq!(
        equip.attached_to, None,
        "Equipment should not be attached when no LastCreatedPermanent"
    );
}
