// Talrand, Sky Summoner — {2}{U}{U}, Legendary Creature — Merfolk Wizard 2/2
// Whenever you cast an instant or sorcery spell, create a 2/2 blue Drake creature token
// with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("talrand-sky-summoner"),
        name: "Talrand, Sky Summoner".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Merfolk", "Wizard"]),
        oracle_text: "Whenever you cast an instant or sorcery spell, create a 2/2 blue Drake creature token with flying.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever you cast an instant or sorcery spell" — WheneverYouCastSpell
            // lacks a spell-type filter (instant/sorcery only). Using unfiltered approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Drake".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Drake".to_string())].into_iter().collect(),
                        colors: [Color::Blue].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
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
