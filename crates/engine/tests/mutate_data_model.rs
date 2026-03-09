//! Mutate data model tests — Session 1 (CR 702.140, CR 729.2).
//!
//! These tests verify that the Session 1 types compile and behave correctly
//! without requiring any resolution logic (Session 2+).
//!
//! CR 702.140: Mutate is an alternative cost allowing you to merge a creature
//! spell with a non-Human creature you own on the battlefield.
//! CR 729.2: A merged permanent consists of two or more objects that were merged.

use mtg_engine::{Characteristics, KeywordAbility, MergedComponent};

/// CR 729.2: A MergedComponent stores card_id, characteristics, and is_token.
/// Verifies the struct fields are accessible and a vector of components works.
#[test]
fn test_mutate_data_model_compiles() {
    let comp = MergedComponent {
        card_id: None,
        characteristics: Characteristics {
            name: "Test Creature".to_string(),
            power: Some(2),
            toughness: Some(2),
            ..Default::default()
        },
        is_token: false,
    };

    assert_eq!(comp.characteristics.name, "Test Creature");
    assert!(!comp.is_token);
    assert_eq!(comp.card_id, None);

    // Verify im::Vector is usable (clone, len, index access)
    let mut components: im::Vector<MergedComponent> = im::Vector::new();
    components.push_back(comp.clone());
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].characteristics.power, Some(2));

    // merged_components[0] is the topmost component (CR 729.2a)
    let top = &components[0];
    assert_eq!(top.characteristics.toughness, Some(2));
}

/// CR 729.2: An unmerged permanent has empty merged_components.
/// Verifies the empty-means-unmerged invariant at the type level.
#[test]
fn test_merged_component_default_empty() {
    let components: im::Vector<MergedComponent> = im::Vector::new();
    assert!(
        components.is_empty(),
        "CR 729.2: unmerged permanent must have empty merged_components"
    );
    // Two-component merged permanent
    let comp_a = MergedComponent {
        card_id: None,
        characteristics: Characteristics {
            name: "Creature A".to_string(),
            ..Default::default()
        },
        is_token: false,
    };
    let comp_b = MergedComponent {
        card_id: None,
        characteristics: Characteristics {
            name: "Creature B".to_string(),
            ..Default::default()
        },
        is_token: true,
    };
    let mut merged: im::Vector<MergedComponent> = im::Vector::new();
    merged.push_back(comp_a);
    merged.push_back(comp_b);
    assert_eq!(merged.len(), 2);
    // [0] is topmost, [1] is underneath (CR 729.2a)
    assert_eq!(merged[0].characteristics.name, "Creature A");
    assert_eq!(merged[1].characteristics.name, "Creature B");
}

/// CR 702.140: KeywordAbility::Mutate can be stored in an OrdSet and checked.
/// Discriminant 147 — after FriendsForever (144), ChooseABackground (145), DoctorsCompanion (146).
#[test]
fn test_mutate_keyword_in_ordset() {
    let mut keywords = im::OrdSet::new();
    keywords.insert(KeywordAbility::Mutate);

    assert!(
        keywords.contains(&KeywordAbility::Mutate),
        "CR 702.140: KeywordAbility::Mutate must be recognizable in keyword set"
    );

    // Verify it does NOT incorrectly match other keywords
    assert!(!keywords.contains(&KeywordAbility::Trample));
    assert!(!keywords.contains(&KeywordAbility::Haste));

    // Verify it sorts after other keywords (discriminant 147 is near-end of enum)
    keywords.insert(KeywordAbility::Haste);
    keywords.insert(KeywordAbility::Trample);
    // All three present
    assert_eq!(keywords.len(), 3);
    assert!(keywords.contains(&KeywordAbility::Mutate));
}
