// Bala Ged Recovery // Bala Ged Sanctuary — {2}{G} Sorcery // Land (MDFC)
// Oracle: "Return target card from your graveyard to your hand."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bala-ged-recovery"),
        name: "Bala Ged Recovery // Bala Ged Sanctuary".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return target card from your graveyard to your hand.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to: ZoneTarget::Hand {
                    owner: PlayerTarget::Controller,
                },
                controller_override: None,
            },
            targets: vec![TargetRequirement::TargetCardInYourGraveyard(
                TargetFilter::default(),
            )],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
