// Windfall — {2}{U}, Sorcery
// Each player discards their hand, then draws cards equal to the greatest
// number of cards a player discarded this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("windfall"),
        name: "Windfall".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each player discards their hand, then draws cards equal to the greatest \
                      number of cards a player discarded this way."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 121.1: each player discards their hand, then all players draw
            // the GREATEST number of cards any player discarded this way.
            effect: Effect::WheelHand {
                player: PlayerTarget::EachPlayer,
                disposal: WheelDisposal::Discard,
                draw: WheelDraw::GreatestDiscarded,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
