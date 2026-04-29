// Dictate of Erebos — {3}{B}{B}, Enchantment
// Flash
// Whenever a creature you control dies, each opponent sacrifices a creature of their choice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dictate-of-erebos"),
        name: "Dictate of Erebos".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Flash\nWhenever a creature you control dies, each opponent sacrifices a creature of their choice.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // CR 603.10a: "Whenever a creature you control dies, each opponent
            // sacrifices a creature."
            // PB-SFT (CR 701.21a + CR 109.1): creature-only filter applied.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::SacrificePermanents {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(1),
                        filter: Some(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        }),
                    }),
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
