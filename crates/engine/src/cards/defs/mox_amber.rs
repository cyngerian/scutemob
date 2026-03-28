// Mox Amber — {T}: Add one mana of any color among legendary creatures and planeswalkers you c
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mox-amber"),
        name: "Mox Amber".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &[]),
        oracle_text: "{T}: Add one mana of any color among legendary creatures and planeswalkers you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
