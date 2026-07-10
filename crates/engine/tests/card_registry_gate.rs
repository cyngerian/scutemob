//! Registry-level enforcement of Architecture Invariant 9.
//!
//! Two machine gates, previously maintained only by worker discipline:
//!
//! 1. `CardRegistry::try_new` rejects duplicate `CardId`s instead of silently
//!    dropping one definition into a `HashMap` collision.
//! 2. Every definition in `all_cards()` that registers no abilities for a card
//!    with printed rules text carries a non-`Complete` `completeness` marker,
//!    so the deck-build gate can surface it.

use mtg_engine::all_cards;
use mtg_engine::cards::{CardDefinition, CardRegistry, Completeness, RegistryError, TypeLine};
use mtg_engine::state::{CardId, CardType};

fn artifact(id: &str, name: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(id.to_string()),
        name: name.to_string(),
        types: TypeLine {
            supertypes: Default::default(),
            card_types: [CardType::Artifact].iter().copied().collect(),
            subtypes: Default::default(),
        },
        ..Default::default()
    }
}

// ── Duplicate CardId detection ────────────────────────────────────────────────

#[test]
/// Two definitions claiming one CardId is a hard error, not a silent overwrite.
///
/// Before this gate, `HashMap::collect` kept whichever definition came last, so a
/// typo'd `cid(...)` in a newly authored card would quietly replace an unrelated
/// card's abilities with no diagnostic anywhere in the build or the test suite.
fn test_try_new_rejects_duplicate_card_id() {
    let defs = vec![
        artifact("sol-ring", "Sol Ring"),
        artifact("mana-crypt", "Mana Crypt"),
        artifact("sol-ring", "Sol Ring (typo'd copy)"),
    ];

    let err = CardRegistry::try_new(defs).expect_err("duplicate CardId must be rejected");
    let RegistryError::DuplicateCardId {
        card_id,
        first_name,
        second_name,
    } = &err;
    assert_eq!(card_id.0, "sol-ring");
    assert_eq!(first_name, "Sol Ring");
    assert_eq!(second_name, "Sol Ring (typo'd copy)");

    // The message must name the colliding id and both definitions.
    let rendered = err.to_string();
    assert!(rendered.contains("sol-ring"), "{rendered}");
    assert!(rendered.contains("Sol Ring (typo'd copy)"), "{rendered}");
}

#[test]
#[should_panic(expected = "duplicate CardId")]
/// `CardRegistry::new` is the infallible convenience wrapper — it panics rather than
/// building a registry that has silently lost a definition.
fn test_new_panics_on_duplicate_card_id() {
    CardRegistry::new(vec![
        artifact("sol-ring", "Sol Ring"),
        artifact("sol-ring", "Sol Ring again"),
    ]);
}

#[test]
/// Distinct CardIds are accepted, and every definition is retrievable.
fn test_try_new_accepts_unique_card_ids() {
    let registry = CardRegistry::try_new(vec![
        artifact("sol-ring", "Sol Ring"),
        artifact("mana-crypt", "Mana Crypt"),
    ])
    .expect("unique CardIds must be accepted");

    assert_eq!(registry.len(), 2);
    assert_eq!(
        registry.get(CardId("mana-crypt".to_string())).unwrap().name,
        "Mana Crypt"
    );
}

#[test]
/// The shipped corpus itself has no CardId collisions. This is the regression test
/// that makes `try_new`'s error unreachable in practice — if a new def collides, this
/// fails by name in CI rather than in a player's game.
fn test_all_cards_have_unique_card_ids() {
    CardRegistry::try_new(all_cards()).expect("all_cards() must not contain duplicate CardIds");
}

// ── Completeness markers on the shipped corpus ────────────────────────────────

#[test]
/// A definition that registers zero abilities for a card with printed rules text is
/// inert: it is a blank permanent wearing the card's name. Every such def must carry a
/// non-`Complete` marker so `validate_deck` rejects it.
///
/// This is the guard that keeps the SR-2 sweep from rotting. A newly authored def with
/// `abilities: vec![]` and a non-empty `oracle_text` fails here until it is either
/// implemented or explicitly marked.
fn test_inert_definitions_are_marked_incomplete() {
    let unmarked: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| {
            d.abilities.is_empty()
                && !d.oracle_text.trim().is_empty()
                && d.completeness.is_complete()
        })
        .map(|d| d.name)
        .collect();

    assert!(
        unmarked.is_empty(),
        "these defs have oracle text but no abilities, and are not marked incomplete \
         (add `completeness: Completeness::inert(\"...\")`): {unmarked:?}"
    );
}

#[test]
/// A vanilla card (no printed rules text, e.g. Memnite) is complete with zero
/// abilities. The inert guard above must not force a marker onto it.
fn test_vanilla_definitions_stay_complete() {
    let marked_vanilla: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| {
            d.abilities.is_empty()
                && d.oracle_text.trim().is_empty()
                && !d.completeness.is_complete()
        })
        .map(|d| d.name)
        .collect();

    assert!(
        marked_vanilla.is_empty(),
        "these defs have no rules text and no abilities — they are vanilla and should be \
         Complete: {marked_vanilla:?}"
    );
}

#[test]
/// Every non-`Complete` marker carries a note explaining what is missing or wrong.
/// An empty note produces a deck-build message the player cannot act on.
fn test_incomplete_markers_carry_a_note() {
    let noteless: Vec<String> = all_cards()
        .into_iter()
        .filter(|d| !d.completeness.is_complete() && d.completeness.note().trim().is_empty())
        .map(|d| d.name)
        .collect();

    assert!(
        noteless.is_empty(),
        "these defs are marked incomplete with an empty note: {noteless:?}"
    );
}

#[test]
/// `Completeness::Complete` is the `Default`, so a def that omits the field is legal
/// in a deck. Every other variant is a deck-build error.
fn test_completeness_default_is_complete() {
    assert_eq!(Completeness::default(), Completeness::Complete);
    assert!(Completeness::default().is_complete());
    assert!(!Completeness::inert("x").is_complete());
    assert!(!Completeness::partial("x").is_complete());
    assert!(!Completeness::known_wrong("x").is_complete());
    assert_eq!(Completeness::inert("x").kind(), "inert");
    assert_eq!(Completeness::partial("x").kind(), "partial");
    assert_eq!(Completeness::known_wrong("x").kind(), "known-wrong");
    assert_eq!(Completeness::known_wrong("why").note(), "why");
    assert_eq!(Completeness::Complete.note(), "");
}
