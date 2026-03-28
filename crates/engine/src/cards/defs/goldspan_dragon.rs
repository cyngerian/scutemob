// Goldspan Dragon — {3}{R}{R}, Creature — Dragon 4/4
// Flying, haste
// Whenever this creature attacks or becomes the target of a spell, create a Treasure token.
// Treasures you control have "{T}, Sacrifice this artifact: Add two mana of any one color."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goldspan-dragon"),
        name: "Goldspan Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying, haste\nWhenever this creature attacks or becomes the target of a spell, create a Treasure token.\nTreasures you control have \"{T}, Sacrifice this artifact: Add two mana of any one color.\"".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // Attack trigger: create Treasure
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken { spec: treasure_token_spec(1) },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "Becomes target of a spell" trigger not in DSL.
            // TODO: "Treasures add two mana" static override not in DSL.
        ],
        ..Default::default()
    }
}
