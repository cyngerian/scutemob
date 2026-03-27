// Wolverine Riders — {4}{G}{G}, Creature — Elf Warrior 4/4
// At the beginning of each upkeep, create a 1/1 green Elf Warrior creature token.
// Whenever another Elf you control enters, you gain life equal to its toughness.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wolverine-riders"),
        name: "Wolverine Riders".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "At the beginning of each upkeep, create a 1/1 green Elf Warrior creature token.\nWhenever another Elf you control enters, you gain life equal to its toughness.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfEachUpkeep,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Elf Warrior".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Elf".to_string()), SubType("Warrior".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
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
            },
            // TODO: "Whenever another Elf enters, gain life equal to toughness" —
            //   EffectAmount lacks toughness-of-entering-creature variant.
        ],
        ..Default::default()
    }
}
