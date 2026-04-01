// Risen Reef — {1}{G}{U}, Creature — Elemental 1/1
// Whenever this or another Elemental you control enters, look at the top card of
// your library. If it's a land, you may put it onto the battlefield tapped.
// If you don't, put it into your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("risen-reef"),
        name: "Risen Reef".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text: "Whenever Risen Reef or another Elemental you control enters, look at the top card of your library. If it's a land card, you may put it onto the battlefield tapped. If you don't put the card onto the battlefield, put it into your hand.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        has_subtype: Some(SubType("Elemental".to_string())),
                        ..Default::default()
                    }),
                },
                // Land → battlefield tapped; else → hand.
                effect: Effect::RevealAndRoute {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    matched_dest: ZoneTarget::Battlefield { tapped: true },
                    unmatched_dest: ZoneTarget::Hand { owner: PlayerTarget::Controller },
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
