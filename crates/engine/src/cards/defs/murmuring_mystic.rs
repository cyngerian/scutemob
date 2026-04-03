// Murmuring Mystic — {3}{U}, Creature — Human Wizard 1/5
// Whenever you cast an instant or sorcery spell, create a 1/1 blue Bird Illusion creature
// token with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("murmuring-mystic"),
        name: "Murmuring Mystic".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Whenever you cast an instant or sorcery spell, create a 1/1 blue Bird Illusion creature token with flying.".to_string(),
        power: Some(1),
        toughness: Some(5),
        abilities: vec![
            // Whenever you cast an instant or sorcery spell, create a 1/1 blue Bird Illusion token with flying.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Instant, CardType::Sorcery]),
                    noncreature_only: false,
                    chosen_subtype_filter: false,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Bird Illusion".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Bird".to_string()), SubType("Illusion".to_string())].into_iter().collect(),
                        colors: [Color::Blue].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
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
        ],
        ..Default::default()
    }
}
