//! Tests for `rules::queries` (M11-local Session 3 §B): the read-only advisory query
//! surface a UI/simulator caller uses to populate target dropdowns before submitting a
//! `Command`. Every test cites the CR section the query mirrors.

use mtg_engine::state::game_object::ActivatedAbility;
use mtg_engine::{
    ability_target_requirements, all_cards, card_name_to_id, enrich_spec_from_def,
    legal_targets_per_slot, spell_target_requirements, target_count_range, AltCostKind,
    CardDefinition, CardType, Color, GameStateBuilder, KeywordAbility, ObjectId, ObjectSpec,
    PlayerId, ProtectionQuality, Step, Target, TargetRequirement, ZoneId,
};
use std::collections::HashMap;
use std::sync::Arc;

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// Build card def map + registry from `all_cards()` (mirrors `mana_triggers.rs`).
fn build_registry() -> (
    HashMap<String, CardDefinition>,
    Arc<mtg_engine::CardRegistry>,
) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = mtg_engine::CardRegistry::new(cards);
    (defs, registry)
}

/// Build an `ObjectSpec` for a named card, enriched from its `CardDefinition`.
fn make_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

// ── CR 601.2c: legal_targets_per_slot ──────────────────────────────────────────

#[test]
/// CR 601.2c / 702.11b / 702.18a / 702.16b — `legal_targets_per_slot` for a
/// `TargetCreature` slot excludes a shrouded creature and a creature with protection
/// from the source's color, but still includes the *caster's own* hexproof creature
/// (CR 702.11b: hexproof only blocks opponents) — pinning that this delegates to the
/// real checker (`casting::validate_targets_inner`) rather than a blunt keyword filter.
fn test_601_2c_legal_targets_excludes_shroud_and_protected_creatures() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let plain = ObjectSpec::creature(p2, "Plain Creature", 2, 2);
    let shrouded =
        ObjectSpec::creature(p2, "Shrouded Creature", 2, 2).with_keyword(KeywordAbility::Shroud);
    let protected = ObjectSpec::creature(p2, "Protected Creature", 2, 2).with_keyword(
        KeywordAbility::ProtectionFrom(ProtectionQuality::FromColor(Color::Red)),
    );
    let own_hexproof = ObjectSpec::creature(p1, "Own Hexproof Creature", 2, 2)
        .with_keyword(KeywordAbility::Hexproof);
    // The targeting source: a red spell in the caster's hand.
    let source = ObjectSpec::card(p1, "Red Source Spell")
        .with_types(vec![CardType::Instant])
        .with_colors(vec![Color::Red])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(plain)
        .object(shrouded)
        .object(protected)
        .object(own_hexproof)
        .object(source)
        .build()
        .unwrap();

    let plain_id = find_object(&state, "Plain Creature");
    let shrouded_id = find_object(&state, "Shrouded Creature");
    let protected_id = find_object(&state, "Protected Creature");
    let own_hexproof_id = find_object(&state, "Own Hexproof Creature");
    let source_id = find_object(&state, "Red Source Spell");

    let candidates =
        legal_targets_per_slot(&state, p1, source_id, &[TargetRequirement::TargetCreature]);
    assert_eq!(candidates.len(), 1, "one slot in, one slot out");

    let slot0: Vec<ObjectId> = candidates[0]
        .iter()
        .map(|t| match t {
            Target::Object(id) => *id,
            Target::Player(_) => panic!("TargetCreature slot should never yield a player target"),
        })
        .collect();

    assert!(
        slot0.contains(&plain_id),
        "a plain creature is a legal target"
    );
    assert!(
        slot0.contains(&own_hexproof_id),
        "CR 702.11b: hexproof only blocks opponents — the caster's own hexproof \
         creature is still targetable by the caster"
    );
    assert!(
        !slot0.contains(&shrouded_id),
        "CR 702.18a: shroud blocks targeting by anyone"
    );
    assert!(
        !slot0.contains(&protected_id),
        "CR 702.16b: protection from red blocks targeting by a red source"
    );
}

#[test]
/// CR 601.2c — a `TargetPlayer` slot's candidates are every live player, and no objects.
fn test_601_2c_legal_targets_includes_players_for_target_player() {
    let p1 = PlayerId(1);
    let source = ObjectSpec::card(p1, "Source Spell")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Source Spell");

    let candidates =
        legal_targets_per_slot(&state, p1, source_id, &[TargetRequirement::TargetPlayer]);
    assert_eq!(candidates.len(), 1);
    let slot0 = &candidates[0];
    assert_eq!(slot0.len(), 4, "all 4 seats are live players");
    for pid in [PlayerId(1), PlayerId(2), PlayerId(3), PlayerId(4)] {
        assert!(slot0.contains(&Target::Player(pid)));
    }
    assert!(
        slot0.iter().all(|t| matches!(t, Target::Player(_))),
        "a TargetPlayer slot must never surface an object candidate"
    );
}

// ── CR 601.2c: target_count_range ──────────────────────────────────────────────

#[test]
/// CR 601.2c — `UpToN` contributes `(0, count)`; a mandatory requirement contributes
/// `(1, 1)`; a mixed list sums both independently.
fn test_601_2c_target_count_range_up_to_n() {
    let up_to_three = TargetRequirement::UpToN {
        count: 3,
        inner: Box::new(TargetRequirement::TargetCreature),
    };
    assert_eq!(
        target_count_range(std::slice::from_ref(&up_to_three)),
        (0, 3)
    );

    let mandatory = TargetRequirement::TargetCreature;
    assert_eq!(target_count_range(std::slice::from_ref(&mandatory)), (1, 1));

    let mixed = vec![mandatory, up_to_three];
    assert_eq!(target_count_range(&mixed), (1, 4));
}

// ── CR 702.96b: spell_target_requirements + Overload ───────────────────────────

#[test]
/// CR 702.96b — an overloaded spell has no targets; without the Overload alt-cost the
/// same spell's printed target requirement is returned unchanged. Uses Cyclonic Rift, a
/// real `Complete` card pairing `AbilityDefinition::Keyword(KeywordAbility::Overload)`
/// with a non-empty `Spell.targets` list.
fn test_702_96b_overload_reports_no_target_requirements() {
    let p1 = PlayerId(1);
    let (defs, registry) = build_registry();
    assert_eq!(
        defs.get("Cyclonic Rift").map(|d| d.completeness.clone()),
        Some(mtg_engine::Completeness::Complete),
        "test fixture assumption: Cyclonic Rift is Complete"
    );
    let spec = make_spec(p1, "Cyclonic Rift", ZoneId::Hand(p1), &defs);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(registry)
        .object(spec)
        .build()
        .unwrap();

    let card_obj_id = find_object(&state, "Cyclonic Rift");

    let normal = spell_target_requirements(&state, card_obj_id, &[], None, false);
    assert!(
        !normal.is_empty(),
        "without overload, Cyclonic Rift carries its printed target requirement"
    );

    let overloaded =
        spell_target_requirements(&state, card_obj_id, &[], Some(AltCostKind::Overload), false);
    assert!(
        overloaded.is_empty(),
        "CR 702.96b: an overloaded spell has no targets"
    );
}

#[test]
/// `spell_target_requirements` never panics on a missing object — it returns `vec![]`.
fn test_spell_target_requirements_missing_object_is_empty() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let bogus = ObjectId(999_999);
    assert!(spell_target_requirements(&state, bogus, &[], None, false).is_empty());
}

// ── CR 602.2b: ability_target_requirements ─────────────────────────────────────

#[test]
/// CR 602.2b — `ability_target_requirements` reads the layer-resolved activated-ability
/// list at `ability_index`, and an out-of-range index yields `vec![]` rather than
/// panicking.
fn test_602_2b_ability_target_requirements_reads_layer_resolved_list() {
    let p1 = PlayerId(1);
    let ability = ActivatedAbility {
        targets: vec![TargetRequirement::TargetCreature],
        ..Default::default()
    };
    let source_spec =
        ObjectSpec::creature(p1, "Ability Source", 2, 2).with_activated_ability(ability);

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(source_spec)
        .build()
        .unwrap();

    let source_id = find_object(&state, "Ability Source");

    let reqs = ability_target_requirements(&state, source_id, 0);
    assert_eq!(reqs, vec![TargetRequirement::TargetCreature]);

    let out_of_range = ability_target_requirements(&state, source_id, 5);
    assert!(
        out_of_range.is_empty(),
        "an out-of-range ability_index must yield vec![], never panic"
    );
}

#[test]
/// `ability_target_requirements` never panics on a missing object — it returns `vec![]`.
fn test_ability_target_requirements_missing_object_is_empty() {
    let state = GameStateBuilder::four_player().build().unwrap();
    let bogus = ObjectId(999_999);
    assert!(ability_target_requirements(&state, bogus, 0).is_empty());
}
