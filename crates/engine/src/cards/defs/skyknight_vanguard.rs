// Skyknight Vanguard — {R}{W}, Creature — Human Knight 1/2
// Flying
// Whenever this creature attacks, create a 1/1 white Soldier creature token that's
// tapped and attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skyknight-vanguard"),
        name: "Skyknight Vanguard".to_string(),
        mana_cost: Some(ManaCost { red: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Flying\nWhenever this creature attacks, create a 1/1 white Soldier creature token that's tapped and attacking.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Soldier".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: true,
                        enters_attacking: true,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
