// Nature's Will — {2}{G}{G}, Enchantment
// Whenever one or more creatures you control deal combat damage to a player,
// tap all lands that player controls and untap all lands you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("natures-will"),
        name: "Nature's Will".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever one or more creatures you control deal combat damage to a player, tap all lands that player controls and untap all lands you control.".to_string(),
        abilities: vec![
            // CR 510.3a / CR 603.2c: Full implementation.
            // Sequence: (1) tap all lands that the damaged player controls (DamagedPlayer ForEach),
            // (2) untap all lands you control (You ForEach).
            // DeclaredTarget { index: 0 } inside ForEach resolves to the current iteration object.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter: None },
                effect: Effect::Sequence(vec![
                    // Tap all lands that the damaged player controls.
                    Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            controller: TargetController::DamagedPlayer,
                            ..Default::default()
                        })),
                        effect: Box::new(Effect::TapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                    // Untap all lands you control.
                    Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            controller: TargetController::You,
                            ..Default::default()
                        })),
                        effect: Box::new(Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
