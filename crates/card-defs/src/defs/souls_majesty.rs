// Soul's Majesty — {4}{G}, Sorcery
// Draw cards equal to the power of target creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("souls-majesty"),
        name: "Soul's Majesty".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw cards equal to the power of target creature you control.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.7a: Draw cards equal to the power of target creature you control.
            // EffectAmount::PowerOf(DeclaredTarget { index: 0 }) reads the power of the
            // targeted creature at resolution time.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::PowerOf(EffectTarget::DeclaredTarget { index: 0 }),
            },
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller: TargetController::You,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
