// Avenger of Zendikar — {5}{G}{G}, Creature — Elemental 5/5
// When this creature enters, create a 0/1 green Plant creature token for each land you control.
// Landfall — Whenever a land you control enters, you may put a +1/+1 counter on each
// Plant creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("avenger-of-zendikar"),
        name: "Avenger of Zendikar".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            green: 2,
            ..Default::default()
        }),
        types: creature_types(&["Elemental"]),
        oracle_text: "When this creature enters, create a 0/1 green Plant creature token for each \
                      land you control.\nLandfall \u{2014} Whenever a land you control enters, \
                      you may put a +1/+1 counter on each Plant creature you control."
            .to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // ETB: Create a Plant token for each land you control.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Plant".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Plant".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 0,
                        toughness: 1,
                        count: EffectAmount::PermanentCount {
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Land),
                                controller: TargetController::You,
                                ..Default::default()
                            },
                            controller: PlayerTarget::Controller,
                        },
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
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Landfall — Whenever a land you control enters, you may put a +1/+1 counter on
            // each Plant creature you control. Modeled unconditionally (always beneficial;
            // same "you may" -> mandatory-take convention as khalni_heart_expedition.rs).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Plant".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    })),
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    }),
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
