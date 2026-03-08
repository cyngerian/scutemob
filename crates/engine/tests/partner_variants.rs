//! Partner variant keyword ability tests (CR 702.124 subrules i, k, m).
//!
//! Covers three partner variant abilities added in Batch 15:
//! - FriendsForever (CR 702.124i): "partner--Friends forever" — both commanders must have it.
//! - ChooseABackground (CR 702.124k): one commander has the keyword, the other is a legendary
//!   Background enchantment (no keyword required on the Background itself).
//! - DoctorsCompanion (CR 702.124m): one commander has the keyword, the other is a legendary
//!   Time Lord Doctor creature with no other creature types.
//!
//! Key rules verified:
//! - Friends Forever: both must have the ability (CR 702.124i).
//! - Friends Forever: cannot combine with plain Partner (CR 702.124f).
//! - Choose a Background: creature + Background enchantment valid (CR 702.124k).
//! - Choose a Background: Background without matching creature rejected (CR 702.124k).
//! - Choose a Background: validate_deck does NOT reject Background enchantment as "not a creature".
//! - Choose a Background: cross-variant rejection with plain Partner (CR 702.124f).
//! - Doctor's Companion: creature + legendary Time Lord Doctor valid (CR 702.124m).
//! - Doctor's Companion: Doctor with extra creature types rejected (CR 702.124m).
//! - Doctor's Companion: Doctor missing Time Lord subtype rejected (CR 702.124m).
//! - Doctor's Companion: cross-variant rejection with Friends Forever (CR 702.124f).
//! - Cross-variant: Friends Forever + Choose a Background rejected (CR 702.124f).
//! - Cross-variant: Friends Forever + Doctor's Companion rejected (CR 702.124f).

use mtg_engine::state::{CardType, SuperType};
use mtg_engine::{
    validate_deck, validate_partner_commanders, AbilityDefinition, CardDefinition, CardId,
    CardRegistry, KeywordAbility, ManaCost, SubType, TypeLine,
};

// ── Test Helpers ──────────────────────────────────────────────────────────────

/// Build a legendary creature CardDefinition with the given ability.
fn legendary_creature_with_ability(
    id: &str,
    name: &str,
    ability: KeywordAbility,
) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![AbilityDefinition::Keyword(ability)],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// Build a legendary enchantment with the Background subtype (CR 702.124k).
fn legendary_background_enchantment(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            subtypes: [SubType("Background".to_string())].into_iter().collect(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: None,
        toughness: None,
        ..Default::default()
    }
}

/// Build a legendary creature with subtypes {Time Lord, Doctor} only (CR 702.124m).
fn legendary_time_lord_doctor(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            subtypes: [
                SubType("Time Lord".to_string()),
                SubType("Doctor".to_string()),
            ]
            .into_iter()
            .collect(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

/// Build a legendary creature with {Time Lord, Doctor, <extra>} subtypes.
fn legendary_time_lord_doctor_extra_type(id: &str, name: &str, extra: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            subtypes: [
                SubType("Time Lord".to_string()),
                SubType("Doctor".to_string()),
                SubType(extra.to_string()),
            ]
            .into_iter()
            .collect(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(3),
        toughness: Some(3),
        ..Default::default()
    }
}

/// A vanilla legendary creature (no partner keywords).
fn legendary_creature_no_ability(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}

/// A legendary enchantment WITHOUT the Background subtype.
fn legendary_enchantment_no_background(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: None,
        toughness: None,
        ..Default::default()
    }
}

/// A plain creature with the plain Partner keyword.
fn legendary_creature_plain_partner(id: &str, name: &str) -> CardDefinition {
    legendary_creature_with_ability(id, name, KeywordAbility::Partner)
}

// ── Friends Forever Tests (CR 702.124i) ──────────────────────────────────────

#[test]
/// CR 702.124i — Two legendary creatures both with FriendsForever are a valid pair.
/// "Partner--Friends forever" requires both commanders to have the exact same ability.
fn test_friends_forever_both_have_ability_valid_pair() {
    let a =
        legendary_creature_with_ability("ff-a", "Forever Friend A", KeywordAbility::FriendsForever);
    let b =
        legendary_creature_with_ability("ff-b", "Forever Friend B", KeywordAbility::FriendsForever);
    let result = validate_partner_commanders(&a, &b);
    assert!(
        result.is_ok(),
        "CR 702.124i: both commanders with FriendsForever should be a valid pair, got: {:?}",
        result
    );
}

#[test]
/// CR 702.124i — One commander has FriendsForever, the other has nothing: invalid.
fn test_friends_forever_only_one_has_ability_rejected() {
    let a =
        legendary_creature_with_ability("ff-a", "Forever Friend A", KeywordAbility::FriendsForever);
    let b = legendary_creature_no_ability("vanilla-b", "Vanilla Legend B");
    let result = validate_partner_commanders(&a, &b);
    assert!(
        result.is_err(),
        "CR 702.124i: only one commander with FriendsForever should fail validation"
    );
}

#[test]
/// CR 702.124f — FriendsForever + plain Partner cannot be combined.
/// Different partner abilities are distinct and cannot pair with each other.
fn test_friends_forever_mixed_with_plain_partner_rejected() {
    let a =
        legendary_creature_with_ability("ff-a", "Forever Friend A", KeywordAbility::FriendsForever);
    let b = legendary_creature_plain_partner("partner-b", "Plain Partner B");
    let result = validate_partner_commanders(&a, &b);
    assert!(
        result.is_err(),
        "CR 702.124f: FriendsForever + plain Partner should fail validation"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("702.124f") || err.contains("incompatible"),
        "CR 702.124f: error should mention incompatible abilities, got: {err}"
    );
}

// ── Choose a Background Tests (CR 702.124k) ───────────────────────────────────

#[test]
/// CR 702.124k — Creature with ChooseABackground + legendary Background enchantment: valid.
fn test_choose_a_background_creature_plus_background_enchantment_valid() {
    let creature = legendary_creature_with_ability(
        "choose-bg-creature",
        "Dungeon Delver",
        KeywordAbility::ChooseABackground,
    );
    let background = legendary_background_enchantment("noble-heritage", "Noble Heritage");
    let result = validate_partner_commanders(&creature, &background);
    assert!(
        result.is_ok(),
        "CR 702.124k: ChooseABackground creature + Background enchantment should be valid, got: {:?}",
        result
    );
    // Also test with reversed argument order.
    let result_rev = validate_partner_commanders(&background, &creature);
    assert!(
        result_rev.is_ok(),
        "CR 702.124k: argument order should not matter"
    );
}

#[test]
/// CR 702.124k — Creature with ChooseABackground + legendary enchantment WITHOUT
/// Background subtype: invalid.
fn test_choose_a_background_missing_background_subtype_rejected() {
    let creature = legendary_creature_with_ability(
        "choose-bg-creature",
        "Dungeon Delver",
        KeywordAbility::ChooseABackground,
    );
    let not_background =
        legendary_enchantment_no_background("cursed-totem", "Cursed Totem Enchantment");
    let result = validate_partner_commanders(&creature, &not_background);
    assert!(
        result.is_err(),
        "CR 702.124k: ChooseABackground + non-Background enchantment should fail"
    );
}

#[test]
/// CR 702.124k — Background enchantment paired with a creature that does NOT have
/// ChooseABackground: invalid.
fn test_choose_a_background_background_without_choose_ability_rejected() {
    let background = legendary_background_enchantment("noble-heritage", "Noble Heritage");
    let regular_creature = legendary_creature_no_ability("regular-legend", "Regular Legend");
    let result = validate_partner_commanders(&background, &regular_creature);
    assert!(
        result.is_err(),
        "CR 702.124k: Background enchantment without a ChooseABackground partner should fail"
    );
}

#[test]
/// CR 702.124k — Two creatures both with ChooseABackground but neither is a Background: invalid.
fn test_choose_a_background_two_creatures_both_choose_rejected() {
    let a = legendary_creature_with_ability(
        "choose-a",
        "Dungeon Delver A",
        KeywordAbility::ChooseABackground,
    );
    let b = legendary_creature_with_ability(
        "choose-b",
        "Dungeon Delver B",
        KeywordAbility::ChooseABackground,
    );
    let result = validate_partner_commanders(&a, &b);
    assert!(
        result.is_err(),
        "CR 702.124k: two ChooseABackground creatures (neither a Background) should fail"
    );
}

#[test]
/// CR 702.124f — ChooseABackground + plain Partner cannot be combined.
fn test_choose_a_background_mixed_with_partner_rejected() {
    let a = legendary_creature_with_ability(
        "choose-bg",
        "Dungeon Delver",
        KeywordAbility::ChooseABackground,
    );
    let b = legendary_creature_plain_partner("partner-b", "Plain Partner B");
    let result = validate_partner_commanders(&a, &b);
    assert!(
        result.is_err(),
        "CR 702.124f: ChooseABackground + plain Partner should fail validation"
    );
}

#[test]
/// CR 702.124k — validate_deck must NOT reject the Background enchantment commander
/// as "not a creature." The Background enchantment is exempt from the creature type
/// requirement when paired with a ChooseABackground commander.
fn test_choose_a_background_commander_type_check_allows_enchantment() {
    let creature = legendary_creature_with_ability(
        "dungeon-delver",
        "Dungeon Delver",
        KeywordAbility::ChooseABackground,
    );
    let background = legendary_background_enchantment("noble-heritage", "Noble Heritage");

    // Build a 100-card deck (fill with copies of a basic land).
    let basic_land = CardDefinition {
        card_id: CardId("forest".to_string()),
        name: "Forest".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Land].into_iter().collect(),
            supertypes: [SuperType::Basic].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: None,
        toughness: None,
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![creature.clone(), background.clone(), basic_land]);

    let commander_ids = vec![
        CardId("dungeon-delver".to_string()),
        CardId("noble-heritage".to_string()),
    ];

    // 98 basic lands + 2 commanders = 100 cards.
    let mut deck_ids = vec![
        CardId("dungeon-delver".to_string()),
        CardId("noble-heritage".to_string()),
    ];
    for _ in 0..98 {
        deck_ids.push(CardId("forest".to_string()));
    }

    let result = validate_deck(&commander_ids, &deck_ids, &registry, &[]);

    // Should be valid — no violation for Background enchantment not being a creature.
    let creature_violation = result.violations.iter().any(|v| {
        if let mtg_engine::DeckViolation::InvalidCommander { name, reason } = v {
            name == "Noble Heritage" && reason.contains("creature")
        } else {
            false
        }
    });
    assert!(
        !creature_violation,
        "CR 702.124k: Noble Heritage (Background enchantment) should NOT be rejected as 'not a creature', \
         violations: {:?}",
        result.violations
    );
    assert!(
        result.valid,
        "CR 702.124k: Dungeon Delver + Noble Heritage should be a valid deck, \
         violations: {:?}",
        result.violations
    );
}

// ── Doctor's Companion Tests (CR 702.124m) ────────────────────────────────────

#[test]
/// CR 702.124m — Creature with DoctorsCompanion + legendary Time Lord Doctor
/// (no extra creature types): valid pair.
fn test_doctors_companion_with_valid_doctor_valid_pair() {
    let companion = legendary_creature_with_ability(
        "rosalind",
        "Rosalind, the Companion",
        KeywordAbility::DoctorsCompanion,
    );
    let doctor = legendary_time_lord_doctor("the-doctor", "The Doctor");
    let result = validate_partner_commanders(&companion, &doctor);
    assert!(
        result.is_ok(),
        "CR 702.124m: DoctorsCompanion + valid Time Lord Doctor should be valid, got: {:?}",
        result
    );
    // Also test reversed argument order.
    let result_rev = validate_partner_commanders(&doctor, &companion);
    assert!(
        result_rev.is_ok(),
        "CR 702.124m: argument order should not matter"
    );
}

#[test]
/// CR 702.124m — Doctor creature missing the "Time Lord" subtype: invalid.
fn test_doctors_companion_doctor_missing_time_lord_subtype_rejected() {
    let companion = legendary_creature_with_ability(
        "rosalind",
        "Rosalind, the Companion",
        KeywordAbility::DoctorsCompanion,
    );
    // A creature with only "Doctor" subtype but NOT "Time Lord".
    let not_time_lord_doctor = CardDefinition {
        card_id: CardId("fake-doctor".to_string()),
        name: "Fake Doctor".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            supertypes: [SuperType::Legendary].into_iter().collect(),
            subtypes: [SubType("Doctor".to_string())].into_iter().collect(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let result = validate_partner_commanders(&companion, &not_time_lord_doctor);
    assert!(
        result.is_err(),
        "CR 702.124m: Doctor missing 'Time Lord' subtype should fail validation"
    );
}

#[test]
/// CR 702.124m — Doctor creature has extra creature types beyond {Time Lord, Doctor}: invalid.
/// "No other creature types" means exactly {Time Lord, Doctor} — no more.
fn test_doctors_companion_doctor_has_extra_creature_types_rejected() {
    let companion = legendary_creature_with_ability(
        "rosalind",
        "Rosalind, the Companion",
        KeywordAbility::DoctorsCompanion,
    );
    let doctor_with_extra =
        legendary_time_lord_doctor_extra_type("hybrid-doctor", "Hybrid Doctor", "Human");
    let result = validate_partner_commanders(&companion, &doctor_with_extra);
    assert!(
        result.is_err(),
        "CR 702.124m: Doctor with extra creature types (Human) should fail validation"
    );
}

#[test]
/// CR 702.124m — DoctorsCompanion + regular legendary creature (not a Doctor): invalid.
fn test_doctors_companion_only_companion_no_doctor_rejected() {
    let companion = legendary_creature_with_ability(
        "rosalind",
        "Rosalind, the Companion",
        KeywordAbility::DoctorsCompanion,
    );
    let regular = legendary_creature_no_ability("regular-legend", "Regular Legend");
    let result = validate_partner_commanders(&companion, &regular);
    assert!(
        result.is_err(),
        "CR 702.124m: DoctorsCompanion + non-Doctor creature should fail validation"
    );
}

#[test]
/// CR 702.124f — DoctorsCompanion + FriendsForever cannot be combined.
fn test_doctors_companion_mixed_with_friends_forever_rejected() {
    let companion = legendary_creature_with_ability(
        "rosalind",
        "Rosalind, the Companion",
        KeywordAbility::DoctorsCompanion,
    );
    let ff = legendary_creature_with_ability(
        "ff-friend",
        "Forever Friend",
        KeywordAbility::FriendsForever,
    );
    let result = validate_partner_commanders(&companion, &ff);
    assert!(
        result.is_err(),
        "CR 702.124f: DoctorsCompanion + FriendsForever should fail validation"
    );
}

// ── Cross-variant Rejection Tests (CR 702.124f) ───────────────────────────────

#[test]
/// CR 702.124f — FriendsForever + ChooseABackground cannot be combined.
/// Different partner variant abilities are distinct and cannot pair with each other.
fn test_cross_variant_friends_forever_plus_choose_background_rejected() {
    let ff = legendary_creature_with_ability(
        "ff-commander",
        "Forever Friend Commander",
        KeywordAbility::FriendsForever,
    );
    let cab = legendary_creature_with_ability(
        "choose-bg-commander",
        "Choose Background Commander",
        KeywordAbility::ChooseABackground,
    );
    let result = validate_partner_commanders(&ff, &cab);
    assert!(
        result.is_err(),
        "CR 702.124f: FriendsForever + ChooseABackground should fail validation"
    );
    // The error may come from the ChooseABackground case (the FF commander is not a Background
    // enchantment) or from cross-variant detection — either way it must be rejected.
}

#[test]
/// CR 702.124f — FriendsForever + DoctorsCompanion cannot be combined.
fn test_cross_variant_friends_forever_plus_doctors_companion_rejected() {
    let ff = legendary_creature_with_ability(
        "ff-commander",
        "Forever Friend Commander",
        KeywordAbility::FriendsForever,
    );
    let dc = legendary_creature_with_ability(
        "dc-commander",
        "Doctor Companion Commander",
        KeywordAbility::DoctorsCompanion,
    );
    let result = validate_partner_commanders(&ff, &dc);
    assert!(
        result.is_err(),
        "CR 702.124f: FriendsForever + DoctorsCompanion should fail validation"
    );
    // The error may come from the DoctorsCompanion case (the FF commander is not a Time Lord
    // Doctor) or from cross-variant detection — either way it must be rejected.
}
