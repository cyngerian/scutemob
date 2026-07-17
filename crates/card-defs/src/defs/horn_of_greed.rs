// Horn of Greed — {3}, Artifact
// Whenever a player plays a land, that player draws a card.
//
// Modeled as WheneverPermanentEntersBattlefield with Land filter, any controller.
// TODO: "that player" — DrawCards always targets Controller (the Horn's controller),
// not the player who played the land. Needs PlayerTarget::TriggeringPlayer.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("horn-of-greed"),
        name: "Horn of Greed".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever a player plays a land, that player draws a card.".to_string(),
        abilities: vec![
            // Whenever a land enters under any player's control, that player draws a card.
            // TODO: DrawCards targets Controller (Horn's controller), not land's controller.
            // Needs PlayerTarget::TriggeringPermanentController or similar.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::Any,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong("the Horn's controller draws whenever ANY player's land enters (should be 'that player draws'). PlayerTarget::TriggeringPlayer exists but the PermanentEnteredBattlefield dispatch never sets trigger.triggering_player, so it is unresolvable here. Also fires on lands PUT onto the battlefield, not just lands played (CR 305.1). Prefer abilities: vec![] per W5 until the dispatch tags the entering permanent's controller."),
        ..Default::default()
    }
}
