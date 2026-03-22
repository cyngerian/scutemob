// Ruthless Lawbringer — {1}{W}{B}, Creature — Vampire Assassin 3/2
// When this creature enters, you may sacrifice another creature. When you do,
// destroy target nonland permanent.
// TODO: DSL gap — "when you do" reflexive trigger: the ETB trigger allows sacrificing another
// creature, and only when that happens does a second trigger fire to destroy a nonland permanent.
// The DSL has no reflexive trigger chaining; the sacrifice-then-destroy sequence cannot be
// faithfully expressed without producing wrong game state (the destroy would always fire).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ruthless-lawbringer"),
        name: "Ruthless Lawbringer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Assassin"]),
        oracle_text: "When this creature enters, you may sacrifice another creature. When you do, destroy target nonland permanent.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // TODO: ETB — "you may sacrifice another creature. When you do, destroy target
            // nonland permanent." The "when you do" reflexive trigger cannot be expressed:
            // the sacrifice is a choice that conditionally fires a second trigger. No DSL
            // support for optional-sacrifice ETB with conditional chained trigger.
        ],
        ..Default::default()
    }
}
