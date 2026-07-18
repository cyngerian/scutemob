// Icetill Explorer — {2}{G}{G}, Creature — Insect Scout 2/4
// You may play an additional land on each of your turns.
// You may play lands from your graveyard.
// Landfall — Whenever a land you control enters, mill a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("icetill-explorer"),
        name: "Icetill Explorer".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 2,
            ..Default::default()
        }),
        types: creature_types(&["Insect", "Scout"]),
        oracle_text: "You may play an additional land on each of your turns.\nYou may play lands \
                      from your graveyard.\nLandfall \u{2014} Whenever a land you control enters, \
                      mill a card."
            .to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // CR 305.2 (PB-32): "You may play an additional land on each of your turns."
            AbilityDefinition::AdditionalLandPlays { count: 1 },
            // CR 601.3, CR 305.1: "You may play lands from your graveyard."
            AbilityDefinition::StaticPlayFromGraveyard {
                filter: PlayFromTopFilter::LandsOnly,
                condition: None,
            },
            // Landfall — Whenever a land you control enters, mill a card.
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
                effect: Effect::MillCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
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
