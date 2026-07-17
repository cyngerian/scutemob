// Puresteel Paladin — {W}{W}, Creature — Human Knight 2/2
// Whenever an Equipment you control enters, you may draw a card.
// Metalcraft — Equipment you control have equip {0} as long as you control
// three or more artifacts.
//
// TODO: "Equipment enters" trigger — WheneverPermanentEntersBattlefield with
//   Equipment subtype filter. TargetFilter has has_card_type but Equipment is
//   a subtype, not a card type. Using has_subtype.
// TODO: Metalcraft conditional equip cost reduction not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("puresteel-paladin"),
        name: "Puresteel Paladin".to_string(),
        mana_cost: Some(ManaCost { white: 2, ..Default::default() }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Whenever an Equipment you control enters, you may draw a card.\nMetalcraft — Equipment you control have equip {0} as long as you control three or more artifacts.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // Whenever an Equipment enters — approximation with subtype filter.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Equipment".to_string())),
                        controller: TargetController::You,
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
            // TODO: Metalcraft equip cost reduction not expressible.
        ],
        completeness: Completeness::partial("Blocked on Metalcraft: 'Equipment you control have equip {0} as long as you control three or more artifacts' — no condition-gated grant of an equip cost to other permanents. Also: the ETB trigger's 'you may draw' is wired as an unconditional draw (Effect::Choose is non-interactive). The Equipment-subtype filter is implemented and is not a blocker."),
        ..Default::default()
    }
}
