//! Unit tests for ManaPool::can_spend() and ManaPool::spend() (CR 118.3, CR 106.12).
//!
//! These test the encapsulated mana spending API on ManaPool directly,
//! without going through the casting pipeline.
//!
//! Addresses MR-M1-15 (ManaPool::spend encapsulation) and MR-M1-07 (ManaPool unit tests).

use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::player::{ManaPool, SpellContext};
use mtg_engine::state::types::ManaColor;

fn pool(w: u32, u: u32, b: u32, r: u32, g: u32, c: u32) -> ManaPool {
    ManaPool {
        white: w,
        blue: u,
        black: b,
        red: r,
        green: g,
        colorless: c,
        restricted: vec![],
    }
}

fn cost(w: u32, u: u32, b: u32, r: u32, g: u32, c: u32, generic: u32) -> ManaCost {
    ManaCost {
        white: w,
        blue: u,
        black: b,
        red: r,
        green: g,
        colorless: c,
        generic,
        ..Default::default()
    }
}

// --- can_spend ---

/// CR 118.3: A player can pay a cost if they have enough mana.
#[test]
fn can_spend_exact_colored() {
    let p = pool(1, 1, 0, 0, 0, 0);
    // Cost: {W}{U}
    assert!(p.can_spend(&cost(1, 1, 0, 0, 0, 0, 0), None));
}

/// CR 118.3: Insufficient colored mana means can't pay.
#[test]
fn can_spend_insufficient_colored() {
    let p = pool(1, 0, 0, 0, 0, 0);
    // Cost: {W}{U} — missing blue
    assert!(!p.can_spend(&cost(1, 1, 0, 0, 0, 0, 0), None));
}

/// Generic mana can be paid with any color (CR 107.4b).
#[test]
fn can_spend_generic_from_colored() {
    let p = pool(0, 0, 0, 3, 0, 0);
    // Cost: {2}{R} — 1R colored + 2 generic from remaining red
    assert!(p.can_spend(&cost(0, 0, 0, 1, 0, 0, 2), None));
}

/// CR 107.4b: Generic can be paid with colorless mana.
#[test]
fn can_spend_generic_from_colorless() {
    let p = pool(0, 0, 0, 0, 0, 5);
    // Cost: {3}
    assert!(p.can_spend(&cost(0, 0, 0, 0, 0, 0, 3), None));
}

/// Not enough total mana for generic portion.
#[test]
fn can_spend_insufficient_generic() {
    let p = pool(1, 0, 0, 0, 0, 0);
    // Cost: {2}{W} — has W but only 0 left for generic (need 2)
    assert!(!p.can_spend(&cost(1, 0, 0, 0, 0, 0, 2), None));
}

/// Zero cost is always payable.
#[test]
fn can_spend_zero_cost() {
    let p = pool(0, 0, 0, 0, 0, 0);
    assert!(p.can_spend(&cost(0, 0, 0, 0, 0, 0, 0), None));
}

/// Colorless-specific cost ({C}) requires colorless mana, not generic.
#[test]
fn can_spend_colorless_specific() {
    // Has only green, no colorless
    let p = pool(0, 0, 0, 0, 3, 0);
    // Cost: {C}{C} — needs 2 colorless specifically
    assert!(!p.can_spend(&cost(0, 0, 0, 0, 0, 2, 0), None));

    // Has colorless
    let p2 = pool(0, 0, 0, 0, 0, 2);
    assert!(p2.can_spend(&cost(0, 0, 0, 0, 0, 2, 0), None));
}

/// Mixed colored + generic cost with excess mana.
#[test]
fn can_spend_with_excess_mana() {
    let p = pool(5, 5, 5, 5, 5, 5);
    // Cost: {1}{W}{U}{B}
    assert!(p.can_spend(&cost(1, 1, 1, 0, 0, 0, 1), None));
}

// --- spend ---

/// CR 118.3: Spending exact colored mana empties those colors.
#[test]
fn spend_exact_colored() {
    let mut p = pool(2, 1, 0, 0, 0, 0);
    p.spend(&cost(2, 1, 0, 0, 0, 0, 0), None);
    assert_eq!(p.white, 0);
    assert_eq!(p.blue, 0);
}

/// CR 107.4b: Generic mana is deducted from colorless first,
/// then green, red, black, blue, white.
#[test]
fn spend_generic_prefers_colorless() {
    let mut p = pool(1, 1, 1, 1, 1, 3);
    // Cost: {3} generic — should take all from colorless
    p.spend(&cost(0, 0, 0, 0, 0, 0, 3), None);
    assert_eq!(p.colorless, 0);
    assert_eq!(p.white, 1);
    assert_eq!(p.blue, 1);
    assert_eq!(p.black, 1);
    assert_eq!(p.red, 1);
    assert_eq!(p.green, 1);
}

/// Generic mana spills over: colorless first, then green, red, black, blue, white.
#[test]
fn spend_generic_overflow_order() {
    let mut p = pool(1, 1, 1, 1, 1, 1);
    // Cost: {4} generic — takes 1 colorless + 1 green + 1 red + 1 black
    p.spend(&cost(0, 0, 0, 0, 0, 0, 4), None);
    assert_eq!(p.colorless, 0);
    assert_eq!(p.green, 0);
    assert_eq!(p.red, 0);
    assert_eq!(p.black, 0);
    assert_eq!(p.blue, 1);
    assert_eq!(p.white, 1);
}

/// Spend colored + generic mixed cost.
#[test]
fn spend_mixed_colored_and_generic() {
    let mut p = pool(2, 0, 0, 3, 0, 0);
    // Cost: {2}{R} — 1R from red, 2 generic from remaining (2R then... but only 2R left + 2W)
    // After R deducted: W=2, R=2, generic=2 → takes 2 from colorless(0)→green(0)→red(2)
    p.spend(&cost(0, 0, 0, 1, 0, 0, 2), None);
    assert_eq!(p.red, 0);
    assert_eq!(p.white, 2);
}

/// Spending from an empty pool with zero cost is a no-op.
#[test]
fn spend_zero_cost_noop() {
    let mut p = pool(3, 2, 1, 0, 0, 4);
    let before = p.clone();
    p.spend(&cost(0, 0, 0, 0, 0, 0, 0), None);
    assert_eq!(p, before);
}

/// Spending all mana leaves pool empty.
#[test]
fn spend_all_mana() {
    let mut p = pool(1, 1, 1, 1, 1, 1);
    // Cost: {W}{U}{B}{R}{G}{C}
    p.spend(&cost(1, 1, 1, 1, 1, 1, 0), None);
    assert!(p.is_empty());
}

/// CR 106.12: Restricted mana matching spell context is included in can_spend.
#[test]
fn can_spend_with_restricted_mana() {
    use mtg_engine::cards::card_definition::ManaRestriction;
    use mtg_engine::state::player::RestrictedMana;
    use mtg_engine::state::types::SubType;

    let mut p = pool(0, 0, 0, 0, 1, 0);
    // Add 2 restricted green (creature spells only)
    p.restricted.push(RestrictedMana {
        color: ManaColor::Green,
        amount: 2,
        restriction: ManaRestriction::CreatureSpellsOnly,
    });

    let creature_ctx = SpellContext {
        is_creature: true,
        subtypes: vec![SubType("Elf".into())],
    };
    let non_creature_ctx = SpellContext {
        is_creature: false,
        subtypes: vec![],
    };

    // Cost: {2}{G} — 1G colored + 2 generic
    let c = cost(0, 0, 0, 0, 1, 0, 2);

    // With creature context: restricted green counts → 1 unrestricted G + 2 restricted G = 3 total
    assert!(p.can_spend(&c, Some(&creature_ctx)));

    // Without creature context: restricted green doesn't count → only 1G, can't pay {2}{G}
    assert!(!p.can_spend(&c, Some(&non_creature_ctx)));
}

/// CR 106.12: Restricted mana is spent first when matching spell context.
#[test]
fn spend_prefers_restricted_mana() {
    use mtg_engine::cards::card_definition::ManaRestriction;
    use mtg_engine::state::player::RestrictedMana;
    use mtg_engine::state::types::SubType;

    let mut p = pool(0, 0, 0, 0, 2, 0);
    p.restricted.push(RestrictedMana {
        color: ManaColor::Green,
        amount: 1,
        restriction: ManaRestriction::CreatureSpellsOnly,
    });

    let ctx = SpellContext {
        is_creature: true,
        subtypes: vec![SubType("Beast".into())],
    };

    // Cost: {G} — should spend restricted green first, leaving unrestricted green intact
    p.spend(&cost(0, 0, 0, 0, 1, 0, 0), Some(&ctx));
    assert_eq!(p.green, 2, "unrestricted green should be untouched");
    assert!(
        p.restricted.is_empty(),
        "restricted entry should be depleted and removed"
    );
}

/// ManaPool::get() returns the unrestricted amount for each color.
#[test]
fn get_returns_correct_color() {
    let p = pool(1, 2, 3, 4, 5, 6);
    assert_eq!(p.get(ManaColor::White), 1);
    assert_eq!(p.get(ManaColor::Blue), 2);
    assert_eq!(p.get(ManaColor::Black), 3);
    assert_eq!(p.get(ManaColor::Red), 4);
    assert_eq!(p.get(ManaColor::Green), 5);
    assert_eq!(p.get(ManaColor::Colorless), 6);
}
