// Elenda's Hierophant — {2}{W}, Creature — Vampire Cleric 1/1
// Flying
// Whenever you gain life, put a +1/+1 counter on this creature.
// When this creature dies, create X 1/1 white Vampire creature tokens with lifelink,
// where X is its power.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elendas-hierophant"),
        name: "Elenda's Hierophant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Cleric"]),
        oracle_text: "Flying\nWhenever you gain life, put a +1/+1 counter on this creature.\nWhen \
                      this creature dies, create X 1/1 white Vampire creature tokens with \
                      lifelink, where X is its power."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Whenever you gain life, put a +1/+1 counter on this creature.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouGainLife,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // "When this creature dies, create X 1/1 white Vampire creature tokens with
            // lifelink, where X is its power." Mirrors elenda_the_dusk_rose.rs's death
            // trigger shape.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Vampire".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Vampire".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::SourcePowerAtLastKnownInformation,
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Lifelink].into_iter().collect(),
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
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
