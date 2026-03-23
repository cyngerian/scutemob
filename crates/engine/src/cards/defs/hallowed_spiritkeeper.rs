// Hallowed Spiritkeeper — {1}{W}{W}, Creature — Avatar 3/2
// Vigilance
// When this creature dies, create X 1/1 white Spirit creature tokens with flying, where
// X is the number of creature cards in your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hallowed-spiritkeeper"),
        name: "Hallowed Spiritkeeper".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 2, ..Default::default() }),
        types: creature_types(&["Avatar"]),
        oracle_text: "Vigilance\nWhen this creature dies, create X 1/1 white Spirit creature tokens with flying, where X is the number of creature cards in your graveyard.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // TODO: "When dies, create X Spirits where X = creature cards in graveyard"
            // — count-based token amount (EffectAmount::CreatureCardsInYourGraveyard) not in DSL
        ],
        ..Default::default()
    }
}
