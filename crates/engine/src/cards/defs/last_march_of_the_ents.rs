// Last March of the Ents — {6}{G}{G}, Sorcery
// This spell can't be countered.
// Draw cards equal to the greatest toughness among creatures you control, then put
// any number of creature cards from your hand onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("last-march-of-the-ents"),
        name: "Last March of the Ents".to_string(),
        mana_cost: Some(ManaCost { generic: 6, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "This spell can't be countered.\nDraw cards equal to the greatest toughness among creatures you control, then put any number of creature cards from your hand onto the battlefield.".to_string(),
        abilities: vec![
            // TODO: "Draw cards equal to greatest toughness" — needs
            // EffectAmount::GreatestToughnessAmongCreaturesYouControl.
            // TODO: "put any number of creature cards from your hand onto the battlefield"
            // — needs Effect::PutCardsFromHandOntoBattlefield with type filter.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: None,
                cant_be_countered: true,
            },
        ],
        ..Default::default()
    }
}
