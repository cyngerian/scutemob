// Return Upon the Tide — {4}{B}, Sorcery
// Return target creature card from your graveyard to the battlefield. If it's an Elf,
// create two 1/1 green Elf Warrior creature tokens.
// Foretell {3}{B}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("return-upon-the-tide"),
        name: "Return Upon the Tide".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Sorcery], &[]),
        oracle_text: "Return target creature card from your graveyard to the battlefield. If it's an Elf, create two 1/1 green Elf Warrior creature tokens.\nForetell {3}{B}".to_string(),
        abilities: vec![
            // TODO: Return creature from graveyard + conditional Elf check for token creation
            // — subtype-conditional effect branch not in DSL
            AbilityDefinition::Keyword(KeywordAbility::Foretell),
        ],
        ..Default::default()
    }
}
