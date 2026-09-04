// Felidar Retreat — {3}{W}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • Create a 2/2 white Cat Beast creature token.
// • Put a +1/+1 counter on each creature you control. Those creatures gain
//   vigilance until end of turn.
//
// CR 700.2b / PB-35: Modal triggered ability. Bot fallback: mode 0 (token).
//
// PB-DX35 (OOS-DX4-2, execution-notes §0.5/A3): NOT in the flat-targets-scoped-to-both-modes
// population -- `targets` is already `vec![]`, so there was never a requirement to leak across
// modes. Left unchanged.
//
// **The reason is stated precisely because the obvious short version is FALSE**, which this
// batch's own `/review` caught: mode 1 DOES contain an `EffectTarget::DeclaredTarget { index: 0 }`
// (below, inside the `Effect::ForEach { over: EachCreatureYouControl }`). That is a ForEach
// ITERATION BINDING, not an announced target slot -- nothing is declared for it at CR 601.2c
// time, which is why the flat `targets` list is empty and correct. A reader re-deriving this
// population by grepping for `DeclaredTarget` finds a contradiction; the axis that separates
// them is the flat `targets` list, which is what `core::pb_dx35_modal_trigger_roster` walks.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("felidar-retreat"),
        name: "Felidar Retreat".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            white: 1,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Enchantment], &[]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n\u{2022} \
                      Create a 2/2 white Cat Beast creature token.\n\u{2022} Put a +1/+1 counter \
                      on each creature you control. Those creatures gain vigilance until end of \
                      turn."
            .to_string(),
        abilities: vec![
            // CR 700.2b / PB-35: Landfall modal triggered ability.
            // Mode 0: Create a 2/2 white Cat Beast token.
            // Mode 1: +1/+1 counter on each creature you control + vigilance until EOT.
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
                                count: EffectAmount::Fixed(1),
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
                    mode_targets: None,
                }),
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
