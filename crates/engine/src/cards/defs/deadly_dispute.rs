// Deadly Dispute — {1}{B} Instant; as an additional cost, sacrifice an artifact
// or creature. Draw two cards. Create a Treasure token.
// TODO: DSL gap — "sacrifice an artifact or creature" as additional cost
// requires typed sacrifice. The draw + treasure portion is expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deadly-dispute"),
        name: "Deadly Dispute".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, sacrifice an artifact or creature.\nDraw two cards. Create a Treasure token.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // TODO: sacrifice additional cost not modeled
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
