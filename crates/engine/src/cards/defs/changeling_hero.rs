// Changeling Hero — {3}{W}, Creature — Shapeshifter 4/4; Changeling, Lifelink,
// Champion a creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("changeling-hero"),
        name: "Changeling Hero".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: creature_types(&["Shapeshifter"]),
        oracle_text: "Changeling (This card is every creature type.)\nChampion a creature (When this enters, sacrifice it unless you exile another creature you control. When this leaves the battlefield, that card returns to the battlefield.)\nLifelink (Damage dealt by this creature also causes you to gain that much life.)"
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Changeling),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Champion { filter: ChampionFilter::AnyCreature },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
