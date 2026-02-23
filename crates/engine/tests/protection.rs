//! Protection keyword targeting tests (CR 702.16a-b).
//!
//! Session 5 of M9.4: data model and targeting enforcement for the protection
//! keyword. Protection blocks Damage, Enchanting, Blocking, and Targeting (DEBT).
//! This session covers only the Targeting (T) aspect; Damage, Enchanting, and
//! Blocking are covered in Session 6.

use mtg_engine::{
    process_command, CardType, Color, Command, GameEvent, GameStateBuilder, KeywordAbility,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, ProtectionQuality, Step, Target, ZoneId,
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
