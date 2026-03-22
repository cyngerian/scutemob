// Scavenger Regent // Exude Toxin — {3}{B} Creature — Dragon 4/4
// Oracle: "Flying\nWard—Discard a card."
// Note: Ward(N) only supports generic mana cost. Ward—Discard is a DSL gap.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scavenger-regent"),
        name: "Scavenger Regent // Exude Toxin".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWard—Discard a card.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — Ward(u32) only supports generic mana cost.
            // "Ward—Discard a card" requires a non-mana ward cost variant.
        ],
        ..Default::default()
    }
}
