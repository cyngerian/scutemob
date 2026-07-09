//! Protection keyword tests (CR 702.16a-f).
//!
//! Session 5 of M9.4: data model and targeting enforcement for the protection
//! keyword. Protection blocks Damage, Enchanting, Blocking, and Targeting (DEBT).
//! Session 5 covers Targeting (T); Session 6 covers Damage (D), Enchanting (E), Blocking (B).

use mtg_engine::CombatDamageTarget;
use mtg_engine::{
    process_command, start_game, AttackTarget, CardType, Color, Command, GameEvent,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId,
    ProtectionQuality, Step, SubType, SuperType, Target, ZoneId,
};

// ── Helper: find object by name ───────────────────────────────────────────────

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

// ── CR 702.16b: Protection from color blocks targeting ────────────────────────

#[test]
/// CR 702.16b — A creature with protection from red cannot be targeted by a red spell.
/// Source: CR 702.16b example ("can't be the target of red spells")
fn test_protection_from_red_blocks_red_spell_targeting() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Target: p1's creature with protection from red.
    let target_spec = ObjectSpec::creature(p1, "White Knight", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
    );

    // Source: a red instant spell controlled by p2.
    let bolt_spec = ObjectSpec::card(p2, "Lightning Bolt")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(target_spec)
        .object(bolt_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    let target_id = find_object(&state, "White Knight");
    let bolt_id = find_object(&state, "Lightning Bolt");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: bolt_id,
            targets: vec![Target::Object(target_id)],
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
        },
    );

    assert!(
        result.is_err(),
        "CR 702.16b: a red spell should not be able to target a creature with protection from red"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("protection"),
        "Error should mention protection, got: {}",
        err_msg
    );
}

#[test]
/// CR 702.16b — A creature with protection from red CAN be targeted by a green spell.
/// Protection only blocks the specified quality (CR 702.16a).
fn test_protection_from_red_allows_green_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Target: p1's creature with protection from red.
    let target_spec = ObjectSpec::creature(p1, "White Knight", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
    );

    // Source: a green instant spell.
    let vines_spec = ObjectSpec::card(p2, "Vines of Vastwood")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            green: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Green])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(target_spec)
        .object(vines_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p2);

    let target_id = find_object(&state, "White Knight");
    let vines_id = find_object(&state, "Vines of Vastwood");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: vines_id,
            targets: vec![Target::Object(target_id)],
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
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.16b: a green spell should be able to target a creature with protection from red; got: {:?}",
        result.err()
    );

    // Verify SpellCast event is emitted.
    let (_, events) = result.unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellCast { .. })),
        "SpellCast event should have fired"
    );
}

#[test]
/// CR 702.16b — Protection from creatures blocks targeting by creature sources.
///
/// A creature permanent's activated ability cannot target a permanent with
/// protection from creatures (card type = Creature).
fn test_protection_from_creatures_blocks_creature_ability() {
    use mtg_engine::ManaCost;

    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Target: p1's creature with protection from creatures.
    let target_spec = ObjectSpec::creature(p1, "Paladin", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromCardType(CardType::Creature)),
    );

    // Source: a creature with an activated ability that deals damage to target creature.
    // We model the activated ability via the creature's object spec.
    // For this test we directly call validate_target_protection via the protection module.
    // Use a simpler approach: attempt to cast a spell that has the Creature card type.
    let creature_spell_spec = ObjectSpec::card(p2, "Creature Spell")
        .with_types(vec![CardType::Creature, CardType::Instant])
        .with_mana_cost(ManaCost {
            colorless: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(target_spec)
        .object(creature_spell_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p2);

    let target_id = find_object(&state, "Paladin");
    let spell_id = find_object(&state, "Creature Spell");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(target_id)],
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
        },
    );

    assert!(
        result.is_err(),
        "CR 702.16b: a creature-type spell should not target a creature with protection from creatures"
    );
}

#[test]
/// CR 702.16b — Protection from everything blocks targeting by any source.
/// A permanent with protection from everything cannot be targeted by anything.
fn test_protection_from_all_blocks_all_targeting() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Target: p1's creature with protection from everything.
    let target_spec = ObjectSpec::creature(p1, "Progenitus", 10, 10)
        .with_keyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromAll));

    // Source: any colored instant spell — even white should be blocked.
    let spell_spec = ObjectSpec::card(p2, "Divine Verdict")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            white: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::White])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(target_spec)
        .object(spell_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state.turn.priority_holder = Some(p2);

    let target_id = find_object(&state, "Progenitus");
    let spell_id = find_object(&state, "Divine Verdict");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(target_id)],
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
        },
    );

    assert!(
        result.is_err(),
        "CR 702.16b: protection from everything should block targeting by any source"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("protection"),
        "Error should mention protection, got: {}",
        err_msg
    );
}

// ── Session 6: Damage (D), Blocking (B), Aura/Equipment SBAs (E) ─────────────

#[test]
/// CR 702.16e — A creature with protection from red takes 0 damage from a red source.
/// Source: CR 702.16e ("all damage that would be dealt ... is prevented")
fn test_protection_from_red_prevents_red_damage() {
    let p1 = PlayerId(1);

    let target_spec = ObjectSpec::creature(p1, "White Knight", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .object(target_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let target_id = find_object(&state, "White Knight");
    let target_chars = mtg_engine::calculate_characteristics(&state, target_id).unwrap();

    // Verify: protection from red prevents damage from a red source.
    let red_source = mtg_engine::state::game_object::Characteristics {
        colors: [Color::Red].iter().cloned().collect(),
        ..Default::default()
    };
    let prevents = mtg_engine::rules::protection::protection_prevents_damage(
        &target_chars.keywords,
        &red_source,
        None,
    );
    assert!(
        prevents,
        "CR 702.16e: protection from red should prevent all damage from a red source"
    );

    // Verify: protection from red does NOT prevent damage from a green source.
    let green_source = mtg_engine::state::game_object::Characteristics {
        colors: [Color::Green].iter().cloned().collect(),
        ..Default::default()
    };
    let prevents_green = mtg_engine::rules::protection::protection_prevents_damage(
        &target_chars.keywords,
        &green_source,
        None,
    );
    assert!(
        !prevents_green,
        "CR 702.16e: protection from red should NOT prevent damage from a green source"
    );
}

#[test]
/// CR 702.16f — A red creature cannot block a creature with protection from red.
/// Source: CR 702.16f ("can't be blocked by [sources with the quality]")
fn test_protection_from_red_blocks_red_blocker() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Attacker: creature with protection from red.
    let attacker_spec = ObjectSpec::creature(p1, "White Knight", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
    );
    // Blocker: p2's red creature.
    let blocker_spec =
        ObjectSpec::creature(p2, "Goblin Warrior", 2, 2).with_colors(vec![Color::Red]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(attacker_spec)
        .object(blocker_spec)
        .at_step(Step::DeclareAttackers)
        .active_player(p1)
        .build()
        .unwrap();

    let mut state = state;
    state.turn.priority_holder = Some(p1);

    let attacker_id = find_object(&state, "White Knight");
    let blocker_id = find_object(&state, "Goblin Warrior");

    // Declare attacker.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    // Pass priority to reach DeclareBlockers step.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).unwrap();
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).unwrap();

    assert_eq!(state.turn.step, Step::DeclareBlockers);

    // p2 attempts to block with the red creature — should fail.
    let result = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(blocker_id, attacker_id)],
        },
    );

    assert!(
        result.is_err(),
        "CR 702.16f: a red creature should not block a creature with protection from red"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("protection"),
        "Error should mention protection, got: {}",
        err_msg
    );
}

#[test]
/// CR 702.16c — An Aura on a creature with protection from the aura's color falls off (SBA 704.5m).
/// Source: CR 702.16c ("can't be enchanted by [sources with the quality]")
fn test_protection_from_red_aura_falls_off() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "White Knight", 2, 2).with_keyword(
            KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
        ))
        .object(
            // A red Aura.
            ObjectSpec::enchantment(p1, "Burning Anger")
                .with_subtypes(vec![SubType("Aura".to_string())])
                .with_colors(vec![Color::Red]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let knight_id = find_object(&state, "White Knight");
    let aura_id = find_object(&state, "Burning Anger");

    // Manually attach the aura to the knight (simulating an illegal attachment state).
    state.objects.get_mut(&aura_id).unwrap().attached_to = Some(knight_id);
    state
        .objects
        .get_mut(&knight_id)
        .unwrap()
        .attachments
        .push_back(aura_id);

    let (_, events) = start_game(state).unwrap();

    let aura_fell_off = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if *object_id == aura_id));
    assert!(
        aura_fell_off,
        "CR 702.16c + CR 704.5m: a red aura on a creature with protection from red \
         should fall off via SBA; events: {:?}",
        events
    );
}

#[test]
/// CR 702.16d — Equipment attached to a creature with protection from the equipment detaches (SBA 704.5n).
/// Source: CR 702.16d ("can't be equipped by [sources with the quality]")
fn test_protection_from_red_equipment_detaches() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "White Knight", 2, 2).with_keyword(
            KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
        ))
        .object(
            // A red Equipment.
            ObjectSpec::artifact(p1, "Blazing Sword")
                .with_subtypes(vec![SubType("Equipment".to_string())])
                .with_colors(vec![Color::Red]),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let knight_id = find_object(&state, "White Knight");
    let equip_id = find_object(&state, "Blazing Sword");

    // Manually attach the equipment to the knight (simulating an illegal attachment state).
    state.objects.get_mut(&equip_id).unwrap().attached_to = Some(knight_id);
    state
        .objects
        .get_mut(&knight_id)
        .unwrap()
        .attachments
        .push_back(equip_id);

    let (_, events) = start_game(state).unwrap();

    let unattached = events.iter().any(
        |e| matches!(e, GameEvent::EquipmentUnattached { object_id } if *object_id == equip_id),
    );
    assert!(
        unattached,
        "CR 702.16d + CR 704.5n: red equipment on a creature with protection from red \
         should detach via SBA; events: {:?}",
        events
    );
}

#[test]
/// CR 702.16 — Protection does NOT block non-targeted global effects.
///
/// A global destroy effect (Wrath of God) affects all creatures regardless of
/// protection. Protection only blocks Damage, Enchanting, Blocking, and Targeting
/// (DEBT). A non-targeted effect bypasses all four restrictions.
///
/// Source: CR 702.16a note ("the 'T' part requires a targeting relationship")
fn test_protection_global_effect_still_works() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Progenitus", 10, 10)
                .with_keyword(KeywordAbility::ProtectionFrom(ProtectionQuality::FromAll)),
        )
        .object(ObjectSpec::creature(p2, "Goblin Token", 1, 1))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let progenitus_id = find_object(&state, "Progenitus");
    let goblin_id = find_object(&state, "Goblin Token");

    // Use execute_effect directly to simulate a global destroy (non-targeted).
    use mtg_engine::cards::EffectTarget as CardEffectTarget;
    use mtg_engine::Effect;
    let global_destroy = Effect::DestroyPermanent {
        target: CardEffectTarget::AllCreatures,
        cant_be_regenerated: false,
    };

    let mut ctx = mtg_engine::effects::EffectContext::new(p1, progenitus_id, vec![]);
    let mut state_mut = state;
    let events = mtg_engine::effects::execute_effect(&mut state_mut, &global_destroy, &mut ctx);

    // DestroyPermanent on a creature emits CreatureDied (it moves to graveyard).
    let progenitus_destroyed = events.iter().any(
        |e| matches!(e, GameEvent::CreatureDied { object_id, .. } if *object_id == progenitus_id),
    );
    let goblin_destroyed = events
        .iter()
        .any(|e| matches!(e, GameEvent::CreatureDied { object_id, .. } if *object_id == goblin_id));

    assert!(
        progenitus_destroyed,
        "CR 702.16: protection from everything does NOT block global non-targeted effects; \
         Progenitus should be destroyed by a destroy-all-creatures effect; events: {:?}",
        events
    );
    assert!(
        goblin_destroyed,
        "Goblin Token should also be destroyed by the global effect; events: {:?}",
        events
    );
}

// ── CR 702.16b: Player targets skip protection check (Finding 1 fix) ──────────

#[test]
/// CR 702.16b — A player with protection from red cannot be targeted by a red spell.
///
/// Players can gain protection qualities (e.g., from continuous effects). When a player
/// has `protection_qualities` containing `FromColor(Red)`, a red spell must not be able
/// to target that player.
fn test_protection_player_target_blocked_by_red_spell() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // p2 is protected from red (simulating e.g. Teferi's Protection granting protection
    // from the player's own color). We set protection_qualities directly on PlayerState.
    let bolt_spec = ObjectSpec::card(p1, "Lightning Bolt")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(bolt_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let mut state = state;
    // Grant p2 protection from red via the PlayerState field.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .protection_qualities
        .push(ProtectionQuality::FromColor(Color::Red));
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let bolt_id = find_object(&state, "Lightning Bolt");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: bolt_id,
            targets: vec![Target::Player(p2)],
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
        },
    );

    assert!(
        result.is_err(),
        "CR 702.16b: a red spell should not be able to target a player with protection from red"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("protection"),
        "Error should mention protection, got: {}",
        err_msg
    );
}

#[test]
/// CR 702.16b — A player without protection CAN be targeted by any spell.
///
/// Positive case: without `protection_qualities`, player targeting proceeds normally.
fn test_protection_player_target_allowed_without_protection() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let bolt_spec = ObjectSpec::card(p1, "Lightning Bolt")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(bolt_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let bolt_id = find_object(&state, "Lightning Bolt");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: bolt_id,
            targets: vec![Target::Player(p2)],
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
        },
    );

    assert!(
        result.is_ok(),
        "CR 702.16b: a player without protection can be targeted by any spell; got: {:?}",
        result.unwrap_err()
    );
}

// ── CR 702.16e: Player damage skips protection check (Finding 2 fix) ──────────

#[test]
/// CR 702.16e — Damage from a red source to a player with protection from red is prevented.
///
/// Uses `apply_damage_prevention` directly (same as the creature damage test above).
/// A red source dealing damage to a player with `protection_qualities: [FromColor(Red)]`
/// should return 0 damage.
fn test_protection_player_damage_prevented() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Place a red source on the battlefield so `source_characteristics` can resolve it.
    let attacker_spec =
        ObjectSpec::creature(p1, "Goblin Guide", 2, 2).with_colors(vec![Color::Red]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(attacker_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Goblin Guide");

    let mut state = state;
    // Grant p2 protection from red.
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .protection_qualities
        .push(ProtectionQuality::FromColor(Color::Red));

    let (amount, _events) = mtg_engine::rules::replacement::apply_damage_prevention(
        &mut state,
        source_id,
        &CombatDamageTarget::Player(p2),
        3,
    );

    assert_eq!(
        amount, 0,
        "CR 702.16e: damage from a red source to a player with protection from red \
         must be prevented (amount must be 0, got {})",
        amount
    );
}

// ── SR-PRO-03: Multicolor source triggers protection when ANY color matches ────

#[test]
/// SR-PRO-03 / CR 702.16a — Protection from red is triggered by ANY red source,
/// including multicolor sources. A source that shares even one color with the
/// protection quality is blocked.
///
/// Scenario: target has protection from red. Source is a green/red multicolor spell.
/// The source shares red with the protection quality → targeting is blocked.
/// Source: CR 702.16a ("having that quality")
fn test_protection_from_red_blocks_multicolor_red_source() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Target: creature with protection from red.
    let target_spec = ObjectSpec::creature(p1, "White Knight Multicolor Test", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
    );

    // Source: a multicolor (red+green) instant spell.
    let multicolor_spec = ObjectSpec::card(p2, "Gruul Bolt")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            red: 1,
            green: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Red, Color::Green])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(target_spec)
        .object(multicolor_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p2);

    let target_id = find_object(&state, "White Knight Multicolor Test");
    let spell_id = find_object(&state, "Gruul Bolt");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(target_id)],
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
        },
    );

    assert!(
        result.is_err(),
        "SR-PRO-03 / CR 702.16a: a red+green multicolor spell should not target \
         a creature with protection from red (source shares red)"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("protection"),
        "Error should mention protection, got: {}",
        err_msg
    );
}

#[test]
/// SR-PRO-03 / CR 702.16a — A pure-green spell targeting a creature with protection
/// from red is NOT blocked (the source doesn't share the red quality).
///
/// Positive control for `test_protection_from_red_blocks_multicolor_red_source`.
fn test_protection_from_red_allows_green_only_multicolor_source() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let target_spec = ObjectSpec::creature(p1, "White Knight Green Test", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
    );

    let green_only_spec = ObjectSpec::card(p2, "Giant Growth")
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            green: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Green])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(target_spec)
        .object(green_only_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p2);

    let target_id = find_object(&state, "White Knight Green Test");
    let spell_id = find_object(&state, "Giant Growth");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(target_id)],
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
        },
    );

    assert!(
        result.is_ok(),
        "SR-PRO-03: a green-only spell should be able to target a creature \
         with protection from red (no shared color); got: {:?}",
        result.err()
    );
}

// ── SR-PRO-04: Subtype-based protection ───────────────────────────────────────

#[test]
/// SR-PRO-04 / CR 702.16b — Protection from a subtype (e.g. "Goblins") blocks targeting
/// by sources that have that subtype.
///
/// A source with creature subtype Goblin cannot target a permanent with
/// protection from Goblins (ProtectionQuality::FromSubType(SubType("Goblin"))).
/// Source: CR 702.16b ("having that quality" — applies to any property)
fn test_protection_from_subtype_goblin_blocks_goblin_source() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Target: creature with protection from Goblins.
    let target_spec = ObjectSpec::creature(p1, "Goblin-Hater", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromSubType(SubType(
            "Goblin".to_string(),
        ))),
    );

    // Source: a Goblin instant (creature subtype = Goblin).
    let goblin_spell_spec = ObjectSpec::card(p2, "Goblin Grenade")
        .with_types(vec![CardType::Instant])
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .with_mana_cost(ManaCost {
            red: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(target_spec)
        .object(goblin_spell_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p2);

    let target_id = find_object(&state, "Goblin-Hater");
    let spell_id = find_object(&state, "Goblin Grenade");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(target_id)],
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
        },
    );

    assert!(
        result.is_err(),
        "SR-PRO-04 / CR 702.16b: a Goblin-subtype spell should not target a \
         creature with protection from Goblins"
    );
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("protection"),
        "Error should mention protection, got: {}",
        err_msg
    );
}

#[test]
/// SR-PRO-04 / CR 702.16b — A non-Goblin spell CAN target a creature with protection
/// from Goblins (the source doesn't share the subtype quality).
///
/// Positive control: a Wizard instant has no Goblin subtype → targeting allowed.
fn test_protection_from_subtype_goblin_allows_wizard_source() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Target: creature with protection from Goblins.
    let target_spec = ObjectSpec::creature(p1, "Goblin-Hater Positive", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromSubType(SubType(
            "Goblin".to_string(),
        ))),
    );

    // Source: a Wizard instant (no Goblin subtype).
    let wizard_spell_spec = ObjectSpec::card(p2, "Wizard Bolt")
        .with_types(vec![CardType::Instant])
        .with_subtypes(vec![SubType("Wizard".to_string())])
        .with_mana_cost(ManaCost {
            blue: 1,
            ..Default::default()
        })
        .with_colors(vec![Color::Blue])
        .in_zone(ZoneId::Hand(p2));

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(target_spec)
        .object(wizard_spell_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let mut state = state;
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p2);

    let target_id = find_object(&state, "Goblin-Hater Positive");
    let spell_id = find_object(&state, "Wizard Bolt");

    let result = process_command(
        state,
        Command::CastSpell {
            player: p2,
            card: spell_id,
            targets: vec![Target::Object(target_id)],
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
        },
    );

    assert!(
        result.is_ok(),
        "SR-PRO-04: a Wizard-subtype (non-Goblin) spell should be able to target \
         a creature with protection from Goblins; got: {:?}",
        result.err()
    );
}

#[test]
/// CR 702.16e — Damage from a green source to a player with protection from red is NOT prevented.
///
/// Protection from red only prevents damage from red sources (CR 702.16a).
fn test_protection_player_damage_not_prevented_wrong_color() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    // Place a green source on the battlefield.
    let attacker_spec =
        ObjectSpec::creature(p1, "Llanowar Elves", 1, 1).with_colors(vec![Color::Green]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(attacker_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Llanowar Elves");

    let mut state = state;
    // Grant p2 protection from red (not green).
    state
        .players
        .get_mut(&p2)
        .unwrap()
        .protection_qualities
        .push(ProtectionQuality::FromColor(Color::Red));

    let (amount, _events) = mtg_engine::rules::replacement::apply_damage_prevention(
        &mut state,
        source_id,
        &CombatDamageTarget::Player(p2),
        3,
    );

    assert_eq!(
        amount, 3,
        "CR 702.16e: damage from a green source to a player with protection from red \
         must NOT be prevented (amount must be 3, got {})",
        amount
    );
}

// ── SR-PRO-01: Protection from a supertype ────────────────────────────────────

#[test]
/// SR-PRO-01 / CR 702.16b — Protection from a supertype (e.g. "legendary") blocks
/// targeting by sources that have that supertype, and does NOT block sources that
/// lack it.
///
/// Source: CR 702.16a ("having that quality" — applies to any property, including
/// supertypes).
fn test_protection_from_supertype_legendary() {
    fn try_cast(source_name: &str, supertypes: Vec<SuperType>) -> bool {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);

        // Target: creature with protection from legendary.
        let target_spec = ObjectSpec::creature(p1, "Legend-Hater", 2, 2).with_keyword(
            KeywordAbility::ProtectionFrom(ProtectionQuality::FromSuperType(SuperType::Legendary)),
        );

        let source_spec = ObjectSpec::card(p2, source_name)
            .with_types(vec![CardType::Instant])
            .with_supertypes(supertypes)
            .with_mana_cost(ManaCost {
                red: 1,
                ..Default::default()
            })
            .with_colors(vec![Color::Red])
            .in_zone(ZoneId::Hand(p2));

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(target_spec)
            .object(source_spec)
            .at_step(Step::PreCombatMain)
            .active_player(p2)
            .build()
            .unwrap();
        state
            .players
            .get_mut(&p2)
            .unwrap()
            .mana_pool
            .add(ManaColor::Red, 1);
        state.turn.priority_holder = Some(p2);

        let target_id = find_object(&state, "Legend-Hater");
        let spell_id = find_object(&state, source_name);

        process_command(
            state,
            Command::CastSpell {
                player: p2,
                card: spell_id,
                targets: vec![Target::Object(target_id)],
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
            },
        )
        .is_err()
    }

    assert!(
        try_cast("Legendary Bolt", vec![SuperType::Legendary]),
        "SR-PRO-01 / CR 702.16b: a legendary spell must not target a creature with \
         protection from legendary"
    );
    assert!(
        !try_cast("Plain Bolt", vec![]),
        "SR-PRO-01: a non-legendary spell must be able to target a creature with \
         protection from legendary"
    );
}

// ── SR-PRO-01: Protection from a name ─────────────────────────────────────────

#[test]
/// SR-PRO-01 / CR 702.16b — Protection from a card name (e.g. "protection from
/// Nicol Bolas") blocks targeting by a source whose name matches exactly, and does
/// NOT block sources with a different name.
///
/// Source: CR 702.16a ("having that quality" — names are a permitted quality).
fn test_protection_from_name() {
    fn try_cast(source_name: &str, protected_from: &str) -> bool {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);

        let target_spec = ObjectSpec::creature(p1, "Name-Hater", 2, 2).with_keyword(
            KeywordAbility::ProtectionFrom(ProtectionQuality::FromName(protected_from.to_string())),
        );

        let source_spec = ObjectSpec::card(p2, source_name)
            .with_types(vec![CardType::Instant])
            .with_mana_cost(ManaCost {
                red: 1,
                ..Default::default()
            })
            .with_colors(vec![Color::Red])
            .in_zone(ZoneId::Hand(p2));

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(target_spec)
            .object(source_spec)
            .at_step(Step::PreCombatMain)
            .active_player(p2)
            .build()
            .unwrap();
        state
            .players
            .get_mut(&p2)
            .unwrap()
            .mana_pool
            .add(ManaColor::Red, 1);
        state.turn.priority_holder = Some(p2);

        let target_id = find_object(&state, "Name-Hater");
        let spell_id = find_object(&state, source_name);

        process_command(
            state,
            Command::CastSpell {
                player: p2,
                card: spell_id,
                targets: vec![Target::Object(target_id)],
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
            },
        )
        .is_err()
    }

    assert!(
        try_cast("Nicol Bolas", "Nicol Bolas"),
        "SR-PRO-01 / CR 702.16b: a spell named 'Nicol Bolas' must not target a \
         creature with protection from Nicol Bolas"
    );
    assert!(
        !try_cast("Shock", "Nicol Bolas"),
        "SR-PRO-01: a spell with a different name must be able to target a creature \
         with protection from Nicol Bolas"
    );
}

// ── SR-PRO-02: Protection from a player (CR 702.16k) ──────────────────────────

#[test]
/// SR-PRO-02 / CR 702.16k — Protection from a player blocks targeting by spells that
/// player controls, and allows targeting by everyone else.
///
/// A creature with "protection from <p2>" cannot be targeted by p2's spells, but
/// its own controller (p1) can still target it.
fn test_protection_from_player_targeting() {
    fn try_cast(caster: PlayerId) -> bool {
        let p1 = PlayerId(1);
        let p2 = PlayerId(2);

        // p1's creature has protection from player p2.
        let target_spec = ObjectSpec::creature(p1, "Bolas-Hater", 2, 2).with_keyword(
            KeywordAbility::ProtectionFrom(ProtectionQuality::FromPlayer(p2)),
        );

        let source_spec = ObjectSpec::card(caster, "Targeted Bolt")
            .with_types(vec![CardType::Instant])
            .with_mana_cost(ManaCost {
                red: 1,
                ..Default::default()
            })
            .with_colors(vec![Color::Red])
            .in_zone(ZoneId::Hand(caster));

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .object(target_spec)
            .object(source_spec)
            .at_step(Step::PreCombatMain)
            .active_player(caster)
            .build()
            .unwrap();
        state
            .players
            .get_mut(&caster)
            .unwrap()
            .mana_pool
            .add(ManaColor::Red, 1);
        state.turn.priority_holder = Some(caster);

        let target_id = find_object(&state, "Bolas-Hater");
        let spell_id = find_object(&state, "Targeted Bolt");

        process_command(
            state,
            Command::CastSpell {
                player: caster,
                card: spell_id,
                targets: vec![Target::Object(target_id)],
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
            },
        )
        .is_err()
    }

    assert!(
        try_cast(PlayerId(2)),
        "SR-PRO-02 / CR 702.16k: a spell controlled by p2 must not target a creature \
         with protection from p2"
    );
    assert!(
        !try_cast(PlayerId(1)),
        "SR-PRO-02 / CR 702.16k: a spell controlled by p1 (not p2) must be able to \
         target a creature with protection from p2"
    );
}

#[test]
/// SR-PRO-02 / CR 702.16k/702.16e — Damage from a source controlled by the
/// protected-from player is prevented; damage from a source controlled by any other
/// player is not.
///
/// Exercises the `FromPlayer` wiring in the damage-prevention path, where the
/// source's controller is read from the source game object.
fn test_protection_from_player_damage_prevented() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    // A source creature controlled by p2.
    let source_spec = ObjectSpec::creature(p2, "Bolas Minion", 3, 3);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .object(source_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p2)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Bolas Minion");

    // p1 has protection from player p2 → damage from p2's source is prevented.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .protection_qualities
        .push(ProtectionQuality::FromPlayer(p2));

    let (amount, _events) = mtg_engine::rules::replacement::apply_damage_prevention(
        &mut state,
        source_id,
        &CombatDamageTarget::Player(p1),
        4,
    );
    assert_eq!(
        amount, 0,
        "SR-PRO-02 / CR 702.16e: damage from a source controlled by p2 to a player \
         with protection from p2 must be fully prevented (got {})",
        amount
    );

    // Swap p1's protection to be from p3 instead → p2's source damage now lands.
    {
        let ps = state.players.get_mut(&p1).unwrap();
        ps.protection_qualities.clear();
        ps.protection_qualities
            .push(ProtectionQuality::FromPlayer(p3));
    }
    let (amount2, _events2) = mtg_engine::rules::replacement::apply_damage_prevention(
        &mut state,
        source_id,
        &CombatDamageTarget::Player(p1),
        4,
    );
    assert_eq!(
        amount2, 4,
        "SR-PRO-02 / CR 702.16k: damage from p2's source to a player with protection \
         only from p3 must NOT be prevented (got {})",
        amount2
    );
}

// ── SR-PRO-03: Multicolor source — damage prevention path ─────────────────────

#[test]
/// SR-PRO-03 / CR 702.16e — Protection from a color is triggered by ANY source that
/// shares that color, even a multicolor one, in the *damage-prevention* path.
///
/// Exercises `protection_prevents_damage` directly (the existing SR-PRO-03 tests
/// cover only the targeting path): a creature with protection from red takes 0
/// damage from a red/green multicolor source, but full damage from a blue/green
/// multicolor source that shares no color with the quality.
fn test_protection_from_multicolor_source_damage_prevention() {
    let p1 = PlayerId(1);

    let target_spec = ObjectSpec::creature(p1, "Multicolor Damage Target", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(PlayerId(2))
        .object(target_spec)
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    let target_id = find_object(&state, "Multicolor Damage Target");
    let target_chars = mtg_engine::calculate_characteristics(&state, target_id).unwrap();

    // A red/green multicolor source shares red with the protection quality.
    let red_green_source = mtg_engine::state::game_object::Characteristics {
        colors: [Color::Red, Color::Green].iter().cloned().collect(),
        ..Default::default()
    };
    assert!(
        mtg_engine::rules::protection::protection_prevents_damage(
            &target_chars.keywords,
            &red_green_source,
            None,
        ),
        "SR-PRO-03 / CR 702.16e: protection from red must prevent damage from a \
         red+green multicolor source (source shares red)"
    );

    // A blue/green multicolor source shares no color with the protection quality.
    let blue_green_source = mtg_engine::state::game_object::Characteristics {
        colors: [Color::Blue, Color::Green].iter().cloned().collect(),
        ..Default::default()
    };
    assert!(
        !mtg_engine::rules::protection::protection_prevents_damage(
            &target_chars.keywords,
            &blue_green_source,
            None,
        ),
        "SR-PRO-03 / CR 702.16e: protection from red must NOT prevent damage from a \
         blue+green multicolor source (no shared color)"
    );
}

// ── SR-PRO-04: Subtype-based protection — blocking path ───────────────────────

#[test]
/// SR-PRO-04 / CR 702.16f — Protection from a subtype prevents blocking by a creature
/// that has that subtype, in the *blocking* path.
///
/// Exercises `protection_prevents_blocking` / `can_block` directly (the existing
/// SR-PRO-04 tests cover only the targeting path): an attacker with protection from
/// Goblins cannot be blocked by a Goblin creature, but can be blocked by a Wizard.
fn test_protection_from_subtype_goblin_prevents_blocking() {
    let attacker_keywords: im::OrdSet<KeywordAbility> = [KeywordAbility::ProtectionFrom(
        ProtectionQuality::FromSubType(SubType("Goblin".to_string())),
    )]
    .iter()
    .cloned()
    .collect();

    // A Goblin blocker shares the protected-from subtype → cannot block.
    let goblin_blocker = mtg_engine::state::game_object::Characteristics {
        subtypes: [SubType("Goblin".to_string())].iter().cloned().collect(),
        ..Default::default()
    };
    assert!(
        !mtg_engine::rules::protection::can_block(&attacker_keywords, &goblin_blocker, None),
        "SR-PRO-04 / CR 702.16f: a Goblin creature must not be able to block an \
         attacker with protection from Goblins"
    );

    // A Wizard blocker does not share the subtype → can block.
    let wizard_blocker = mtg_engine::state::game_object::Characteristics {
        subtypes: [SubType("Wizard".to_string())].iter().cloned().collect(),
        ..Default::default()
    };
    assert!(
        mtg_engine::rules::protection::can_block(&attacker_keywords, &wizard_blocker, None),
        "SR-PRO-04 / CR 702.16f: a Wizard creature (no Goblin subtype) must be able to \
         block an attacker with protection from Goblins"
    );
}
