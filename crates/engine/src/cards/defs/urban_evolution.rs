// Urban Evolution — {3}{G}{U}, Sorcery
// Draw three cards. You may play an additional land this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("urban-evolution"),
        name: "Urban Evolution".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw three cards. You may play an additional land this turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 305.2: Draw three cards and grant one additional land play this turn.
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(3),
                },
                Effect::AdditionalLandPlay,
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
