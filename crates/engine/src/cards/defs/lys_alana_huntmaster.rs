// Lys Alana Huntmaster — {2}{G}{G}, Creature — Elf Warrior 3/3
// Whenever you cast an Elf spell, you may create a 1/1 green Elf Warrior creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lys-alana-huntmaster"),
        name: "Lys Alana Huntmaster".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "Whenever you cast an Elf spell, you may create a 1/1 green Elf Warrior creature token.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // Whenever you cast an Elf spell, create a 1/1 green Elf Warrior token.
            // TODO: "you may" — optional create. Using mandatory.
            // Note: "Elf spell" needs subtype filter on spells (not yet in DSL).
            // Using creature spell as approximation (Elf creature spells only).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                },
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
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
