// Grave Pact
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grave-pact"),
        name: "Grave Pact".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 3, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control dies, each other player sacrifices a creature of their choice.".to_string(),
        abilities: vec![
            // CR 603.10a: "Whenever a creature you control dies, each other player
            // sacrifices a creature."
            // PB-SFT (CR 701.17a + CR 109.1c): creature-only filter applied.
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
