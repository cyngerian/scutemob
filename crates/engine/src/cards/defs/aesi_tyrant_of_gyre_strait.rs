// Aesi, Tyrant of Gyre Strait — {4}{G}{U}, Legendary Creature — Serpent 5/5
// You may play an additional land on each of your turns.
// Landfall — Whenever a land you control enters, you may draw a card.
//
// TODO: AdditionalLandPlays static effect not in DSL.
// TODO: "may draw" — optional draw. Implementing as mandatory draw (approximation).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aesi-tyrant-of-gyre-strait"),
        name: "Aesi, Tyrant of Gyre Strait".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Serpent"],
        ),
        oracle_text: "You may play an additional land on each of your turns.\nLandfall — Whenever a land you control enters, you may draw a card.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // TODO: "You may play an additional land" — AdditionalLandPlays static not in DSL.
            // Landfall — Whenever a land you control enters, draw a card.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
