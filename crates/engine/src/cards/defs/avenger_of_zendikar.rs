// Avenger of Zendikar — {5}{G}{G}, Creature — Elemental 5/5
// When this creature enters, create a 0/1 green Plant creature token for each land you control.
// Landfall — Whenever a land you control enters, you may put a +1/+1 counter on each
// Plant creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("avenger-of-zendikar"),
        name: "Avenger of Zendikar".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "When this creature enters, create a 0/1 green Plant creature token for each land you control.\nLandfall \u{2014} Whenever a land you control enters, you may put a +1/+1 counter on each Plant creature you control.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // ETB: Create Plant tokens equal to lands you control
            // TODO: EffectAmount lacks "count of lands you control" variant.
            //   Using fixed 5 as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Plant".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Plant".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 0,
                        toughness: 1,
                        count: 5,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
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
            // Landfall: +1/+1 counter on each Plant
            // TODO: "Each Plant you control" counter distribution not in DSL.
        ],
        ..Default::default()
    }
}
