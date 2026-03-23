// Captain Lannery Storm — {2}{R}, Legendary Creature — Human Pirate 2/2
// Haste
// Whenever Captain Lannery Storm attacks, create a Treasure token.
// Whenever you sacrifice a Treasure, Captain Lannery Storm gets +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("captain-lannery-storm"),
        name: "Captain Lannery Storm".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Pirate"],
        ),
        oracle_text: "Haste\nWhenever Captain Lannery Storm attacks, create a Treasure token.\nWhenever you sacrifice a Treasure, Captain Lannery Storm gets +1/+0 until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken { spec: treasure_token_spec(1) },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "Whenever you sacrifice a Treasure" trigger not in DSL.
        ],
        ..Default::default()
    }
}
