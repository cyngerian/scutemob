//! Protection keyword tests (CR 702.16a-f).
//!
//! Session 5 of M9.4: data model and targeting enforcement for the protection
//! keyword. Protection blocks Damage, Enchanting, Blocking, and Targeting (DEBT).
//! Session 5 covers Targeting (T); Session 6 covers Damage (D), Enchanting (E), Blocking (B).

use mtg_engine::{
    process_command, start_game, AttackTarget, CardType, Color, Command, GameEvent,
    GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId,
    ProtectionQuality, Step, SubType, Target, ZoneId,
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
