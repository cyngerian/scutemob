// Hullbreaker Horror — {5}{U}{U}, Creature — Kraken Horror 7/8
// Flash
// This spell can't be countered.
// Whenever you cast a spell, choose up to one —
// • Return target spell you don't control to its owner's hand.
// • Return target nonland permanent to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hullbreaker-horror"),
        name: "Hullbreaker Horror".to_string(),
        mana_cost: Some(ManaCost { generic: 5, blue: 2, ..Default::default() }),
        types: creature_types(&["Kraken", "Horror"]),
        oracle_text: "Flash\nThis spell can't be countered.\nWhenever you cast a spell, choose up to one —\n• Return target spell you don't control to its owner's hand.\n• Return target nonland permanent to its owner's hand.".to_string(),
        power: Some(7),
        toughness: Some(8),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // TODO: "This spell can't be countered" — needs cant_be_countered on the creature spell.
            // Currently only supported on AbilityDefinition::Spell, not on creature cards.
            // TODO: "Whenever you cast a spell, choose up to one" — modal triggered ability
            // with per-mode targets. Mode 1: bounce target opponent's spell. Mode 2: bounce
            // target nonland permanent. "up to one" means may choose zero modes.
        ],
        ..Default::default()
    }
}
