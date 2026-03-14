// War Room — Land, {T}: Add {C}; {3},{T}, Pay X life: Draw a card (TODO)
// TODO: {3}, {T}, Pay life equal to # colors in commander color identity: Draw a card
// — life payment scaled to commander color identity count not expressible in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("war-room"),
        name: "War Room".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{3}, {T}, Pay life equal to the number of colors in your commanders' color identity: Draw a card.".to_string(),
        abilities: vec![
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
