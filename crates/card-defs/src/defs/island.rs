// Island
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("island"),
        name: "Island".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Basic], &[CardType::Land], &["Island"]),
        oracle_text: "{T}: Add {U}.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 1, 0, 0, 0, 0),
            },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
                activation_zone: None,
        once_per_turn: false,
}],
        ..Default::default()
    }
}
