// World Shaper — {3}{G}, Creature — Merfolk Shaman 3/3
// Whenever this creature attacks, you may mill three cards.
// When this creature dies, return all land cards from your graveyard to the battlefield tapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("world-shaper"),
        name: "World Shaper".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Merfolk", "Shaman"]),
        oracle_text: "Whenever World Shaper attacks, you may mill three cards.\nWhen World Shaper dies, return all land cards from your graveyard to the battlefield tapped.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // CR 603.2: "Whenever World Shaper attacks, you may mill three cards."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::MillCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 400.7, 603.6a: "When World Shaper dies, return all land cards from
            // your graveyard to the battlefield tapped."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::ReturnAllFromGraveyardToBattlefield {
                    graveyards: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    tapped: true,
                    controller_override: None,
                    unique_names: false,
                    permanent_cards_only: false,
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
