// Toski, Bearer of Secrets — {3}{G}, Legendary Creature — Squirrel 1/1
// This spell can't be countered.
// Indestructible
// Toski attacks each combat if able.
// Whenever a creature you control deals combat damage to a player, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("toski-bearer-of-secrets"),
        name: "Toski, Bearer of Secrets".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Squirrel"]),
        oracle_text: "This spell can't be countered.\nIndestructible\nToski attacks each combat if able.\nWhenever a creature you control deals combat damage to a player, draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // TODO: All abilities stripped per W5 policy — Indestructible without must-attack
            // constraint produces wrong game state (unkillable blocker instead of forced attacker).
            // Needs: CantBeCountered, Indestructible, MustAttack, and
            // WhenAnyCreatureYouControlDealsCombatDamage trigger (DSL gaps).
        ],
        ..Default::default()
    }
}
