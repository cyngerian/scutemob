// Teysa, Orzhov Scion — {1}{W}{B}, Legendary Creature — Human Advisor 2/3
// Sacrifice three white creatures: Exile target creature.
// Whenever another black creature you control dies, create a 1/1 white Spirit creature token
// with flying.
// TODO: Sacrifice ability — no DSL Cost variant for "sacrifice N permanents of a given type"
//   (only Cost::Sacrifice sacrifices one). "Sacrifice three white creatures" as an activated
//   cost is not expressible. Deferred: pending a future PB that adds multi-sacrifice costs.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teysa-orzhov-scion"),
        name: "Teysa, Orzhov Scion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Advisor"]),
        oracle_text: "Sacrifice three white creatures: Exile target creature.\nWhenever another black creature you control dies, create a 1/1 white Spirit creature token with flying.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // CR 603.10a / CR 603.2: "Whenever another black creature you control dies,
            // create a 1/1 white Spirit creature token with flying."
            // PB-N: black color filter via triggering_creature_filter (colors field).
            // exclude_self: true ("another"), controller: You.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: true,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        colors: Some([Color::Black].into_iter().collect()),
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Spirit".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::White].into_iter().collect(),
                        supertypes: OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Spirit".to_string())].into_iter().collect(),
                        keywords: [KeywordAbility::Flying].into_iter().collect(),
                        count: 1,
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
