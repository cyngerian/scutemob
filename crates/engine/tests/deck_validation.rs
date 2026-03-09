//! Deck validation tests for Commander format (CR 903.5).
//!
//! Tests verify that `validate_deck` correctly rejects illegal decks and accepts
//! legal ones.

use std::sync::Arc;

use mtg_engine::cards::{CardDefinition, CardRegistry, TypeLine};
use mtg_engine::state::{CardId, CardType, Color, ManaCost, SubType, SuperType};
use mtg_engine::{compute_color_identity, validate_deck, DeckViolation};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

/// Build a minimal legendary creature definition.
fn legendary_creature(id: &str, name: &str, cost: ManaCost) -> CardDefinition {
    CardDefinition {
        card_id: cid(id),
        name: name.to_string(),
        mana_cost: Some(cost),
        types: TypeLine {
            supertypes: [SuperType::Legendary].iter().copied().collect(),
            card_types: [CardType::Creature].iter().copied().collect(),
            subtypes: [SubType("Human".to_string()), SubType("Wizard".to_string())]
                .iter()
                .cloned()
                .collect(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(2),
        toughness: Some(2),
        back_face: None,
    }
}

/// Build a non-creature non-legendary artifact definition.
fn artifact(id: &str, name: &str, cost: ManaCost) -> CardDefinition {
    CardDefinition {
        card_id: cid(id),
        name: name.to_string(),
        mana_cost: Some(cost),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [CardType::Artifact].iter().copied().collect(),
            subtypes: Default::default(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        ..Default::default()
    }
}

/// Build a basic land definition.
fn basic_land(id: &str, name: &str, subtype: &str) -> CardDefinition {
    CardDefinition {
        card_id: cid(id),
        name: name.to_string(),
        mana_cost: None,
        types: TypeLine {
            supertypes: [SuperType::Basic].iter().copied().collect(),
            card_types: [CardType::Land].iter().copied().collect(),
            subtypes: [SubType(subtype.to_string())].iter().cloned().collect(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        ..Default::default()
    }
}

/// Build a non-legendary creature definition.
fn creature(id: &str, name: &str, cost: ManaCost) -> CardDefinition {
    CardDefinition {
        card_id: cid(id),
        name: name.to_string(),
        mana_cost: Some(cost),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [CardType::Creature].iter().copied().collect(),
            subtypes: [SubType("Human".to_string())].iter().cloned().collect(),
        },
        oracle_text: String::new(),
        abilities: vec![],
        power: Some(1),
        toughness: Some(1),
        back_face: None,
    }
}

/// Build a registry containing a commander and enough filler cards for a 100-card deck.
///
/// Commander: white legendary creature (mana cost {2W}).
/// Filler: 39 unique colorless artifacts + basic land "Plains".
fn build_valid_deck_registry() -> Arc<CardRegistry> {
    let mut defs = Vec::new();

    // Commander: white legendary creature
    defs.push(legendary_creature(
        "test-commander",
        "Test Commander",
        ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        },
    ));

    // Filler: 39 unique colorless artifacts
    for i in 1..=39 {
        defs.push(artifact(
            &format!("filler-{i}"),
            &format!("Filler Artifact {i}"),
            ManaCost {
                generic: 1,
                ..Default::default()
            },
        ));
    }

    // Basic land: Plains (only one definition, used 60 times by CardId)
    defs.push(basic_land("plains", "Plains", "Plains"));

    CardRegistry::new(defs)
}

/// Produce the CardIds for a valid 100-card deck: 1 commander + 39 fillers + 60 plains.
fn valid_deck_ids() -> Vec<CardId> {
    let mut ids = Vec::new();
    ids.push(cid("test-commander"));
    for i in 1..=39 {
        ids.push(cid(&format!("filler-{i}")));
    }
    for _ in 0..60 {
        ids.push(cid("plains"));
    }
    ids
}

// ── CR 903.5a: Deck size ──────────────────────────────────────────────────────

#[test]
/// CR 903.5a — deck with 99 cards is rejected (too few).
fn test_deck_validation_rejects_99_cards() {
    let registry = build_valid_deck_registry();
    let mut ids = valid_deck_ids();
    ids.pop(); // remove one card
    assert_eq!(ids.len(), 99);

    let result = validate_deck(&[cid("test-commander")], &ids, &registry, &[]);

    assert!(!result.valid);
    assert!(
        result.violations.iter().any(|v| matches!(
            v,
            DeckViolation::WrongDeckSize {
                actual: 99,
                expected: 100
            }
        )),
        "expected WrongDeckSize violation, got: {:?}",
        result.violations
    );
}

#[test]
/// CR 903.5a — deck with 101 cards is rejected (too many).
fn test_deck_validation_rejects_101_cards() {
    let registry = build_valid_deck_registry();
    let mut ids = valid_deck_ids();
    ids.push(cid("plains")); // add one extra
    assert_eq!(ids.len(), 101);

    let result = validate_deck(&[cid("test-commander")], &ids, &registry, &[]);

    assert!(!result.valid);
    assert!(
        result.violations.iter().any(|v| matches!(
            v,
            DeckViolation::WrongDeckSize {
                actual: 101,
                expected: 100
            }
        )),
        "expected WrongDeckSize violation, got: {:?}",
        result.violations
    );
}

// ── CR 903.5b: Singleton rule ─────────────────────────────────────────────────

#[test]
/// CR 903.5b — two copies of a non-basic card in the same deck is rejected.
fn test_deck_validation_rejects_duplicate_nonbasic() {
    let mut defs = Vec::new();
    defs.push(legendary_creature(
        "test-commander",
        "Test Commander",
        ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        },
    ));
    // Two distinct CardIds but the same card name — models two copies of the same card.
    defs.push(artifact(
        "duplicate-a",
        "Duplicate Artifact",
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));
    defs.push(artifact(
        "duplicate-b",
        "Duplicate Artifact",
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));
    // Filler to reach exactly 100
    for i in 1..=37 {
        defs.push(artifact(
            &format!("filler-{i}"),
            &format!("Filler {i}"),
            ManaCost {
                generic: 1,
                ..Default::default()
            },
        ));
    }
    defs.push(basic_land("plains", "Plains", "Plains"));
    let registry = CardRegistry::new(defs);

    let mut ids = vec![
        cid("test-commander"),
        cid("duplicate-a"),
        cid("duplicate-b"),
    ];
    for i in 1..=37 {
        ids.push(cid(&format!("filler-{i}")));
    }
    for _ in 0..60 {
        ids.push(cid("plains"));
    }
    assert_eq!(ids.len(), 100);

    let result = validate_deck(&[cid("test-commander")], &ids, &registry, &[]);

    assert!(!result.valid);
    assert!(
        result.violations.iter().any(|v| matches!(
            v,
            DeckViolation::DuplicateCard { name, .. } if name == "Duplicate Artifact"
        )),
        "expected DuplicateCard violation, got: {:?}",
        result.violations
    );
}

#[test]
/// CR 903.5b exception — multiple basic lands of the same name are allowed.
fn test_deck_validation_allows_basic_land_duplicates() {
    let registry = build_valid_deck_registry();
    let ids = valid_deck_ids(); // contains 60 Plains

    let result = validate_deck(&[cid("test-commander")], &ids, &registry, &[]);

    assert!(
        result.valid,
        "basic land duplicates should be allowed; violations: {:?}",
        result.violations
    );
}

// ── CR 903.5c: Color identity ─────────────────────────────────────────────────

#[test]
/// CR 903.5c — off-color card (blue identity in a mono-white deck) is rejected.
fn test_deck_validation_rejects_off_color_identity() {
    let mut defs = Vec::new();
    defs.push(legendary_creature(
        "test-commander",
        "Test Commander",
        ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        },
    ));
    // Blue card — violates mono-white identity
    defs.push(creature(
        "blue-creature",
        "Blue Creature",
        ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        },
    ));
    for i in 1..=38 {
        defs.push(artifact(
            &format!("filler-{i}"),
            &format!("Filler {i}"),
            ManaCost {
                generic: 1,
                ..Default::default()
            },
        ));
    }
    defs.push(basic_land("plains", "Plains", "Plains"));
    let registry = CardRegistry::new(defs);

    let mut ids = vec![cid("test-commander"), cid("blue-creature")];
    for i in 1..=38 {
        ids.push(cid(&format!("filler-{i}")));
    }
    for _ in 0..60 {
        ids.push(cid("plains"));
    }
    assert_eq!(ids.len(), 100);

    let result = validate_deck(&[cid("test-commander")], &ids, &registry, &[]);

    assert!(!result.valid);
    assert!(
        result.violations.iter().any(|v| matches!(
            v,
            DeckViolation::ColorIdentityViolation { card, .. } if card == "Blue Creature"
        )),
        "expected ColorIdentityViolation for Blue Creature, got: {:?}",
        result.violations
    );
}

// ── Banned list ───────────────────────────────────────────────────────────────

#[test]
/// Banned card in deck is rejected.
fn test_deck_validation_rejects_banned_card() {
    let mut defs = Vec::new();
    defs.push(legendary_creature(
        "test-commander",
        "Test Commander",
        ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        },
    ));
    defs.push(artifact(
        "banned-card",
        "Banned Card",
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    ));
    for i in 1..=38 {
        defs.push(artifact(
            &format!("filler-{i}"),
            &format!("Filler {i}"),
            ManaCost {
                generic: 1,
                ..Default::default()
            },
        ));
    }
    defs.push(basic_land("plains", "Plains", "Plains"));
    let registry = CardRegistry::new(defs);

    let mut ids = vec![cid("test-commander"), cid("banned-card")];
    for i in 1..=38 {
        ids.push(cid(&format!("filler-{i}")));
    }
    for _ in 0..60 {
        ids.push(cid("plains"));
    }
    assert_eq!(ids.len(), 100);

    let banned = vec!["Banned Card".to_string()];
    let result = validate_deck(&[cid("test-commander")], &ids, &registry, &banned);

    assert!(!result.valid);
    assert!(
        result.violations.iter().any(|v| matches!(
            v,
            DeckViolation::BannedCard { name } if name == "Banned Card"
        )),
        "expected BannedCard violation, got: {:?}",
        result.violations
    );
}

// ── CR 903.3: Commander must be legendary creature ────────────────────────────

#[test]
/// CR 903.3 — a non-legendary creature cannot be a commander.
fn test_deck_validation_rejects_non_legendary_commander() {
    let mut defs = Vec::new();
    // Commander is not legendary
    defs.push(creature(
        "non-legendary-commander",
        "Regular Creature",
        ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        },
    ));
    for i in 1..=39 {
        defs.push(artifact(
            &format!("filler-{i}"),
            &format!("Filler {i}"),
            ManaCost {
                generic: 1,
                ..Default::default()
            },
        ));
    }
    defs.push(basic_land("plains", "Plains", "Plains"));
    let registry = CardRegistry::new(defs);

    let mut ids = vec![cid("non-legendary-commander")];
    for i in 1..=39 {
        ids.push(cid(&format!("filler-{i}")));
    }
    for _ in 0..60 {
        ids.push(cid("plains"));
    }
    assert_eq!(ids.len(), 100);

    let result = validate_deck(&[cid("non-legendary-commander")], &ids, &registry, &[]);

    assert!(!result.valid);
    assert!(
        result.violations.iter().any(|v| matches!(
            v,
            DeckViolation::InvalidCommander { name, .. } if name == "Regular Creature"
        )),
        "expected InvalidCommander violation, got: {:?}",
        result.violations
    );
}

// ── Full valid deck ───────────────────────────────────────────────────────────

#[test]
/// A properly constructed 100-card Commander deck passes all checks.
fn test_deck_validation_accepts_valid_100_card_deck() {
    let registry = build_valid_deck_registry();
    let ids = valid_deck_ids();
    assert_eq!(ids.len(), 100);

    let result = validate_deck(&[cid("test-commander")], &ids, &registry, &[]);

    assert!(
        result.valid,
        "valid deck should pass; violations: {:?}",
        result.violations
    );
    assert!(result.violations.is_empty());
}

// ── compute_color_identity ────────────────────────────────────────────────────

#[test]
/// CR 903.4 — colorless card (no colored mana symbols) has empty color identity.
fn test_compute_color_identity_colorless() {
    let registry = CardRegistry::new(vec![artifact(
        "sol-ring",
        "Sol Ring",
        ManaCost {
            generic: 1,
            ..Default::default()
        },
    )]);
    let def = registry.get(cid("sol-ring")).unwrap();
    let identity = compute_color_identity(def);
    assert!(
        identity.is_empty(),
        "colorless artifact should have empty identity"
    );
}

#[test]
/// CR 903.4 — white card has White in its color identity.
fn test_compute_color_identity_single_color() {
    let registry = CardRegistry::new(vec![legendary_creature(
        "white-commander",
        "White Commander",
        ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        },
    )]);
    let def = registry.get(cid("white-commander")).unwrap();
    let identity = compute_color_identity(def);
    assert_eq!(identity, vec![Color::White]);
}

#[test]
/// CR 903.4 — multicolor card has all its colors in identity.
fn test_compute_color_identity_multicolor() {
    let registry = CardRegistry::new(vec![legendary_creature(
        "grixis-commander",
        "Grixis Commander",
        ManaCost {
            generic: 2,
            blue: 1,
            black: 1,
            red: 1,
            ..Default::default()
        },
    )]);
    let def = registry.get(cid("grixis-commander")).unwrap();
    let mut identity = compute_color_identity(def);
    identity.sort();
    assert_eq!(identity, vec![Color::Blue, Color::Black, Color::Red]);
}
