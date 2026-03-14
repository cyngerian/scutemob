// 6. Darksteel Ingot — {3}, Artifact (Indestructible), tap: add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darksteel-ingot"),
        name: "Darksteel Ingot".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Indestructible\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
