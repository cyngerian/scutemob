// Retreat to Coralhelm — {2}{U}, Enchantment
// Landfall — Whenever a land you control enters, choose one —
// • You may tap or untap target creature.
// • Scry 1.
//
// CR 700.2b / PB-35: Modal triggered ability. Bot fallback: mode 0 (untap target creature).
// Note: "tap or untap" is approximated as "untap" (mode 0). The bot always untaps.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("retreat-to-coralhelm"),
        name: "Retreat to Coralhelm".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Landfall — Whenever a land you control enters, choose one —\n• You may tap or untap target creature.\n• Scry 1.".to_string(),
        abilities: vec![
            // CR 700.2b / PB-35: Landfall modal triggered ability.
            // Mode 0: You may tap or untap target creature (approximated as untap).
            // Mode 1: Scry 1.
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
                targets: vec![
                    // Mode 0 target: any creature.
                    TargetRequirement::TargetCreature,
                ],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: Untap target creature (approximation of "tap or untap").
                        Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        },
                        // Mode 1: Scry 1.
                        Effect::Scry {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(1),
                        },
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
