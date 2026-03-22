// Courser of Kruphix — {1}{G}{G}, Enchantment Creature — Centaur 2/4
// Play with the top card of your library revealed.
// You may play lands from the top of your library.
// Landfall — Whenever a land you control enters, you gain 1 life.
//
// TODO: "Play with the top card of your library revealed" — needs continuous info-reveal effect.
// TODO: "You may play lands from the top of your library" — needs play-from-top-of-library static.
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
            },
        ],
        ..Default::default()
    }
}
