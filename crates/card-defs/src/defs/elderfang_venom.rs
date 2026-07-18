// Elderfang Venom — {2}{B}{G}, Enchantment
// Attacking Elves you control have deathtouch.
// Whenever an Elf you control dies, each opponent loses 1 life and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elderfang-venom"),
        name: "Elderfang Venom".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking Elves you control have deathtouch.\nWhenever an Elf you control \
                      dies, each opponent loses 1 life and you gain 1 life."
            .to_string(),
        abilities: vec![
            // CR 613.1f / CR 611.3a: "Attacking Elves you control have deathtouch."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(SubType(
                        "Elf".to_string(),
                    )),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // "Whenever an Elf you control dies, each opponent loses 1 life and you gain 1 life."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::Sequence(vec![
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::LoseLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
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
