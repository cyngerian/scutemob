// Felidar Retreat — {3}{W}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • Create a 2/2 white Cat Beast creature token.
// • Put a +1/+1 counter on each creature you control. Those creatures gain
//   vigilance until end of turn.
//
// CR 700.2b / PB-35: Modal triggered ability. Bot fallback: mode 0 (token).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("felidar-retreat"),
        name: "Felidar Retreat".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Enchantment], &[]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n\u{2022} Create a 2/2 white Cat Beast creature token.\n\u{2022} Put a +1/+1 counter on each creature you control. Those creatures gain vigilance until end of turn.".to_string(),
        abilities: vec![
            // CR 700.2b / PB-35: Landfall modal triggered ability.
            // Mode 0: Create a 2/2 white Cat Beast token.
            // Mode 1: +1/+1 counter on each creature you control + vigilance until EOT.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: Create a 2/2 white Cat Beast creature token.
                        Effect::CreateToken {
                            spec: TokenSpec {
                                name: "Cat Beast".to_string(),
                                card_types: [CardType::Creature].into_iter().collect(),
                                subtypes: [
                                    SubType("Cat".to_string()),
                                    SubType("Beast".to_string()),
                                ]
                                .into_iter()
                                .collect(),
                                colors: [Color::White].into_iter().collect(),
                                power: 2,
                                toughness: 2,
                                count: 1,
                                supertypes: im::OrdSet::new(),
                                keywords: im::OrdSet::new(),
                                tapped: false,
                                enters_attacking: false,
                                mana_color: None,
                                mana_abilities: vec![],
                                activated_abilities: vec![],
                                ..Default::default()
                            },
                        },
                        // Mode 1: Put a +1/+1 counter on each creature you control +
                        // those creatures gain vigilance until end of turn.
                        Effect::Sequence(vec![
                            Effect::ForEach {
                                over: ForEachTarget::EachCreatureYouControl,
                                effect: Box::new(Effect::AddCounter {
                                    target: EffectTarget::DeclaredTarget { index: 0 },
                                    counter: CounterType::PlusOnePlusOne,
                                    count: 1,
                                }),
                            },
                            Effect::ApplyContinuousEffect {
                                effect_def: Box::new(ContinuousEffectDef {
                                    layer: EffectLayer::Ability,
                                    modification: LayerModification::AddKeyword(
                                        KeywordAbility::Vigilance,
                                    ),
                                    filter: EffectFilter::CreaturesYouControl,
                                    duration: EffectDuration::UntilEndOfTurn,
                                    condition: None,
                                }),
                            },
                        ]),
                    ],
                    allow_duplicate_modes: false,
                    mode_costs: None,
                }),
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
