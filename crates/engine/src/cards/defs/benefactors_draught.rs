// Benefactor's Draught — {1}{G}, Instant
// Untap all creatures. Until end of turn, whenever a creature an opponent controls
// blocks, draw a card.
// Draw a card.
//
// TODO: "Untap all creatures" — Effect::UntapAll not in DSL.
// TODO: "Whenever opponent's creature blocks" trigger not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("benefactors-draught"),
        name: "Benefactor's Draught".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Untap all creatures. Until end of turn, whenever a creature an opponent controls blocks, draw a card.\nDraw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Untap all + block trigger not expressible. Draw only.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
