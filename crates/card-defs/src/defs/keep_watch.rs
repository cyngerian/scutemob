// Keep Watch — {2}{U}, Instant
// Draw a card for each attacking creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("keep-watch"),
        name: "Keep Watch".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw a card for each attacking creature.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 508.1/509: PB-AC3 AttackingCreatureCount. EachPlayer gives the CR-correct
            // "number of attacking creatures" reading (unrestricted by controller).
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::AttackingCreatureCount {
                    controller: PlayerTarget::EachPlayer,
                    filter: None,
                },
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
