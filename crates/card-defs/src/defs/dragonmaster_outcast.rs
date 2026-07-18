// Dragonmaster Outcast — {R}, Creature — Human Shaman 1/1
// At the beginning of your upkeep, if you control six or more lands, create a 5/5 red Dragon
// creature token with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonmaster-outcast"),
        name: "Dragonmaster Outcast".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Shaman"]),
        oracle_text: "At the beginning of your upkeep, if you control six or more lands, create a \
                      5/5 red Dragon creature token with flying."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 603.4: intervening-if re-checked at resolution.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Dragon".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Dragon".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 5,
                        toughness: 5,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 6,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                }),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
