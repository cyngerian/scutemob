// Zhalfirin Void — Land, ETB: scry 1. {T}: Add {C}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zhalfirin-void"),
        name: "Zhalfirin Void".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "When this land enters, scry 1. (Look at the top card of your library. You may put that card on the bottom.)\n{T}: Add {C}.".to_string(),
        abilities: vec![
            // CR 701.18: Scry 1 on ETB.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Scry {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
