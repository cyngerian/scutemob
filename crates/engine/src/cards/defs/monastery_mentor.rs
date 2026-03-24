// Monastery Mentor — {2}{W}, Creature — Human Monk 2/2
// Prowess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end
// of turn.)
// Whenever you cast a noncreature spell, create a 1/1 white Monk creature token with
// prowess.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("monastery-mentor"),
        name: "Monastery Mentor".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Monk"]),
        oracle_text: "Prowess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)\nWhenever you cast a noncreature spell, create a 1/1 white Monk creature token with prowess.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Prowess),
            // Whenever you cast a noncreature spell, create a 1/1 white Monk token with prowess.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: None,
                    noncreature_only: true,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Monk".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Monk".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Prowess].into_iter().collect(),
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
