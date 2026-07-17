// Whispering Wizard — {3}{U}, Creature — Human Wizard 3/2
// Whenever you cast a noncreature spell, create a 1/1 white Spirit creature token with flying.
// This ability triggers only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("whispering-wizard"),
        name: "Whispering Wizard".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Whenever you cast a noncreature spell, create a 1/1 white Spirit creature \
                      token with flying. This ability triggers only once each turn."
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // CR 603.2h: "Whenever you cast a noncreature spell, create a 1/1 white Spirit
            // creature token with flying. This ability triggers only once each turn."
            AbilityDefinition::Triggered {
                once_per_turn: true,
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: None,
                    noncreature_only: true,
                    chosen_subtype_filter: false,
                    spell_subtype_filter: None,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Spirit".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
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
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
