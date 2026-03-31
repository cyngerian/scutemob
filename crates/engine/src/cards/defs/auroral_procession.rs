// Auroral Procession — {G}{U}, Instant
// Return target card from your graveyard to your hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("auroral-procession"),
        name: "Auroral Procession".to_string(),
        mana_cost: Some(ManaCost { green: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return target card from your graveyard to your hand.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
