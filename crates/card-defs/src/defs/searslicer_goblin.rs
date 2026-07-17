// Searslicer Goblin — {1}{R}, Creature — Goblin Warrior 2/1
// Raid — At the beginning of your end step, if you attacked this turn, create a 1/1 red
// Goblin creature token.
//
// CR 508.1 (Raid): PB-AC6 added Condition::YouAttackedThisTurn, used here as the
// intervening-if on the AtBeginningOfYourEndStep trigger.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("searslicer-goblin"),
        name: "Searslicer Goblin".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Raid \u{2014} At the beginning of your end step, if you attacked this turn, \
                      create a 1/1 red Goblin creature token."
            .to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
            effect: Effect::CreateToken {
                spec: TokenSpec {
                    name: "Goblin".to_string(),
                    card_types: [CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                    colors: [Color::Red].into_iter().collect(),
                    power: 1,
                    toughness: 1,
                    count: EffectAmount::Fixed(1),
                    supertypes: imbl::OrdSet::new(),
                    keywords: imbl::OrdSet::new(),
                    tapped: false,
                    enters_attacking: false,
                    mana_color: None,
                    mana_abilities: vec![],
                    activated_abilities: vec![],
                    ..Default::default()
                },
            },
            intervening_if: Some(Condition::YouAttackedThisTurn),
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
