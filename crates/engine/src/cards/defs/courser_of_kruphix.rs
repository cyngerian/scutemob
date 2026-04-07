// Courser of Kruphix — {1}{G}{G}, Enchantment Creature — Centaur 2/4
// Play with the top card of your library revealed.
// You may play lands from the top of your library.
// Landfall — Whenever a land you control enters, you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("courser-of-kruphix"),
        name: "Courser of Kruphix".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 2, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Centaur"]),
        oracle_text: "Play with the top card of your library revealed.\nYou may play lands from the top of your library.\nLandfall — Whenever a land you control enters, you gain 1 life.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // CR 601.3 / CR 305.1 (PB-A): "Play with the top card of your library revealed.
            // You may play lands from the top of your library."
            // reveal_top: true means ALL players can see the top card (stronger than look_at_top).
            // LandsOnly filter: only lands may be played this way (not spells).
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::LandsOnly,
                look_at_top: false,
                reveal_top: true,
                pay_life_instead: false,
                condition: None,
                on_cast_effect: None,
            },
            // Landfall — gain 1 life
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
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
