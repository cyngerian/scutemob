// Zurgo Stormrender — {R}{W}{B}, Legendary Creature — Orc Warrior 3/3
// Mobilize 1 (Whenever this creature attacks, create a tapped and attacking 1/1 red
// Warrior creature token. Sacrifice it at the beginning of the next end step.)
// Whenever a creature token you control leaves the battlefield, draw a card if it was
// attacking. Otherwise, each opponent loses 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zurgo-stormrender"),
        name: "Zurgo Stormrender".to_string(),
        mana_cost: Some(ManaCost { red: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Orc", "Warrior"],
        ),
        oracle_text: "Mobilize 1 (Whenever this creature attacks, create a tapped and attacking 1/1 red Warrior creature token. Sacrifice it at the beginning of the next end step.)\nWhenever a creature token you control leaves the battlefield, draw a card if it was attacking. Otherwise, each opponent loses 1 life.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // Mobilize 1: create token on attack
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Warrior".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Warrior".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
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
                        sacrifice_at_end_step: true, // Mobilize: sacrifice at next end step
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "Whenever a creature token you control leaves" trigger not in DSL.
        ],
        ..Default::default()
    }
}
