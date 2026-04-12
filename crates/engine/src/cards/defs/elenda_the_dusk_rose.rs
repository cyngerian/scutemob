// Elenda, the Dusk Rose — {2}{W}{B}, Legendary Creature — Vampire Knight 1/1
// Lifelink
// Whenever another creature dies, put a +1/+1 counter on Elenda.
// When Elenda dies, create X 1/1 white Vampire creature tokens with lifelink, where X
// is Elenda's power.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elenda-the-dusk-rose"),
        name: "Elenda, the Dusk Rose".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Knight"],
        ),
        oracle_text: "Lifelink\nWhenever another creature dies, put a +1/+1 counter on Elenda, the Dusk Rose.\nWhen Elenda, the Dusk Rose dies, create X 1/1 white Vampire creature tokens with lifelink, where X is Elenda's power.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            // CR 603.10a: "Whenever another creature dies, put a +1/+1 counter on Elenda."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: None, exclude_self: true, nontoken_only: false, filter: None,
},
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
            // TODO: "When dies, create X tokens where X = power" — EffectAmount lacks
            //   power-based count. Using fixed 3 as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Vampire".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Vampire".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 3,
                        supertypes: im::OrdSet::new(),
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
        ..Default::default()
    }
}
