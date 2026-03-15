//! Tests for hybrid mana, Phyrexian mana, and X costs (PB-9).
//!
//! CR 107.3 (X), 107.4e (hybrid), 107.4f (Phyrexian),
//! CR 202.2d (hybrid color identity), 202.3e-g (mana value).

use mtg_engine::{
    process_command, CardDefinition, CardId, CardType, Command, GameEvent, GameStateBuilder,
    HybridMana, HybridManaPayment, ManaCost, ManaColor, ObjectSpec, PhyrexianMana,
    PlayerId, Step, ZoneId, compute_color_identity,
};
use mtg_engine::rules::casting::flatten_hybrid_phyrexian;

// ── Mana Value Tests ─────────────────────────────────────────────────────────

/// CR 202.3f: Hybrid {W/U} contributes 1 to mana value (max(1, 1) = 1).
#[test]
fn test_hybrid_color_color_mana_value() {
    let cost = ManaCost {
        generic: 1,
        hybrid: vec![
            HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
            HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
        ],
        ..Default::default()
    };
    // Kitchen Finks: {{1}}{{G/W}}{{G/W}} → MV 3
    assert_eq!(cost.mana_value(), 3, "CR 202.3f: {{1}}{{G/W}}{{G/W}} = MV 3");
}

/// CR 202.3f: Hybrid {{2/W}} contributes 2 to mana value (max(2, 1) = 2).
#[test]
fn test_hybrid_generic_color_mana_value() {
    let cost = ManaCost {
        hybrid: vec![HybridMana::GenericColor(ManaColor::White)],
        ..Default::default()
    };
    assert_eq!(cost.mana_value(), 2, "CR 202.3f: {{2/W}} = MV 2");
}

/// CR 202.3g: Phyrexian {W/P} contributes 1 to mana value.
#[test]
fn test_phyrexian_mana_value() {
    let cost = ManaCost {
        generic: 1,
        phyrexian: vec![
            PhyrexianMana::Single(ManaColor::Blue),
            PhyrexianMana::Single(ManaColor::Blue),
        ],
        ..Default::default()
    };
    // {{1}}{{U/P}}{{U/P}} → MV 3
    assert_eq!(cost.mana_value(), 3, "CR 202.3g: {{1}}{{U/P}}{{U/P}} = MV 3");
}

/// CR 202.3e: X is 0 off the stack; x_count doesn't contribute to mana_value().
#[test]
fn test_x_mana_value_off_stack() {
    let cost = ManaCost {
        blue: 1,
        x_count: 1,
        ..Default::default()
    };
    // {{X}}{{U}} → MV 1 (X = 0 off stack)
    assert_eq!(cost.mana_value(), 1, "CR 202.3e: {{X}}{{U}} off stack = MV 1");
}

/// CR 202.3e: {{X}}{{X}} has MV 0 off stack (+ any fixed pips).
#[test]
fn test_double_x_mana_value() {
    let cost = ManaCost {
        x_count: 2,
        ..Default::default()
    };
    // {{X}}{{X}} → MV 0 off stack
    assert_eq!(cost.mana_value(), 0, "CR 202.3e: {{X}}{{X}} off stack = MV 0");
}

/// CR 202.3g + 202.2d: Hybrid Phyrexian {G/W/P} contributes 1 to MV.
#[test]
fn test_hybrid_phyrexian_mana_value() {
    let cost = ManaCost {
        generic: 1,
        green: 1,
        white: 1,
        phyrexian: vec![PhyrexianMana::Hybrid(ManaColor::Green, ManaColor::White)],
        ..Default::default()
    };
    // {{1}}{{G}}{{W}}{{G/W/P}} → MV 4
    assert_eq!(cost.mana_value(), 4, "CR 202.3g: {{1}}{{G}}{{W}}{{G/W/P}} = MV 4");
}

// ── Payment Tests ────────────────────────────────────────────────────────────

fn p1() -> PlayerId {
    PlayerId(0)
}

/// CR 107.4e: Hybrid {G/W} can be paid with green.
#[test]
fn test_hybrid_payment_first_color() {
    let cost = ManaCost {
        generic: 1,
        hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
        ..Default::default()
    };
    let choices = vec![HybridManaPayment::Color(ManaColor::Green)];
    let phyrexian = vec![];
    let (flat, life) = flatten_hybrid_phyrexian(&cost, &choices, &phyrexian);
    assert_eq!(flat.green, 1, "Hybrid paid with green");
    assert_eq!(flat.white, 0, "White not used");
    assert_eq!(flat.generic, 1, "Generic preserved");
    assert_eq!(life, 0, "No life cost");
}

/// CR 107.4e: Hybrid {G/W} can be paid with white.
#[test]
fn test_hybrid_payment_second_color() {
    let cost = ManaCost {
        hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
        ..Default::default()
    };
    let choices = vec![HybridManaPayment::Color(ManaColor::White)];
    let (flat, _) = flatten_hybrid_phyrexian(&cost, &choices, &[]);
    assert_eq!(flat.white, 1, "Hybrid paid with white");
    assert_eq!(flat.green, 0, "Green not used");
}

/// CR 107.4e: Hybrid {{2/W}} can be paid with 2 generic.
#[test]
fn test_hybrid_generic_color_pay_generic() {
    let cost = ManaCost {
        hybrid: vec![HybridMana::GenericColor(ManaColor::White)],
        ..Default::default()
    };
    let choices = vec![HybridManaPayment::Generic];
    let (flat, _) = flatten_hybrid_phyrexian(&cost, &choices, &[]);
    assert_eq!(flat.generic, 2, "{{2/W}} paid as 2 generic");
    assert_eq!(flat.white, 0, "White not used");
}

/// CR 107.4e: Hybrid {{2/W}} can be paid with white (cheaper).
#[test]
fn test_hybrid_generic_color_pay_color() {
    let cost = ManaCost {
        hybrid: vec![HybridMana::GenericColor(ManaColor::White)],
        ..Default::default()
    };
    let choices = vec![HybridManaPayment::Color(ManaColor::White)];
    let (flat, _) = flatten_hybrid_phyrexian(&cost, &choices, &[]);
    assert_eq!(flat.white, 1, "{{2/W}} paid as 1 white");
    assert_eq!(flat.generic, 0, "No generic cost");
}

/// CR 107.4f: Phyrexian {U/P} paid with mana.
#[test]
fn test_phyrexian_payment_mana() {
    let cost = ManaCost {
        phyrexian: vec![PhyrexianMana::Single(ManaColor::Blue)],
        ..Default::default()
    };
    let (flat, life) = flatten_hybrid_phyrexian(&cost, &[], &[false]);
    assert_eq!(flat.blue, 1, "Phyrexian paid with blue mana");
    assert_eq!(life, 0, "No life cost");
}

/// CR 107.4f: Phyrexian {U/P} paid with 2 life.
#[test]
fn test_phyrexian_payment_life() {
    let cost = ManaCost {
        phyrexian: vec![PhyrexianMana::Single(ManaColor::Blue)],
        ..Default::default()
    };
    let (flat, life) = flatten_hybrid_phyrexian(&cost, &[], &[true]);
    assert_eq!(flat.blue, 0, "No blue mana when paying life");
    assert_eq!(life, 2, "2 life paid for Phyrexian");
}

/// CR 107.4f: Multiple Phyrexian pips — mix of life and mana.
#[test]
fn test_phyrexian_mixed_payment() {
    let cost = ManaCost {
        phyrexian: vec![
            PhyrexianMana::Single(ManaColor::Red),
            PhyrexianMana::Single(ManaColor::Red),
        ],
        ..Default::default()
    };
    // First: pay with mana, second: pay with life
    let (flat, life) = flatten_hybrid_phyrexian(&cost, &[], &[false, true]);
    assert_eq!(flat.red, 1, "One red from mana payment");
    assert_eq!(life, 2, "2 life from second pip");
}

// ── X Cost Payment (Integration) ────────────────────────────────────────────

// ── Color Identity Tests ─────────────────────────────────────────────────────

/// CR 903.4 / CR 202.2d: Hybrid {G/W} adds both G and W to color identity.
#[test]
fn test_hybrid_color_identity() {
    let def = CardDefinition {
        card_id: CardId("hybrid_card".to_string()),
        name: "Hybrid Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            hybrid: vec![HybridMana::ColorColor(ManaColor::Green, ManaColor::White)],
            ..Default::default()
        }),
        ..Default::default()
    };

    let colors = compute_color_identity(&def);
    assert!(colors.contains(&mtg_engine::Color::Green), "Hybrid adds green to identity");
    assert!(colors.contains(&mtg_engine::Color::White), "Hybrid adds white to identity");
}

/// CR 903.4: Phyrexian {B/P} adds B to color identity.
#[test]
fn test_phyrexian_color_identity() {
    let def = CardDefinition {
        card_id: CardId("phyrexian_card".to_string()),
        name: "Phyrexian Card".to_string(),
        mana_cost: Some(ManaCost {
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Black)],
            ..Default::default()
        }),
        ..Default::default()
    };

    let colors = compute_color_identity(&def);
    assert!(colors.contains(&mtg_engine::Color::Black), "Phyrexian adds black to identity");
}

/// CR 903.4: Hybrid Phyrexian {G/W/P} adds both G and W to color identity.
#[test]
fn test_hybrid_phyrexian_color_identity() {
    let def = CardDefinition {
        card_id: CardId("hybrid_phyrexian_card".to_string()),
        name: "Hybrid Phyrexian Card".to_string(),
        mana_cost: Some(ManaCost {
            phyrexian: vec![PhyrexianMana::Hybrid(ManaColor::Green, ManaColor::White)],
            ..Default::default()
        }),
        ..Default::default()
    };

    let colors = compute_color_identity(&def);
    assert!(colors.contains(&mtg_engine::Color::Green), "Hybrid Phyrexian adds green");
    assert!(colors.contains(&mtg_engine::Color::White), "Hybrid Phyrexian adds white");
}
