// Leaden Myr — {T}: Add {B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("leaden-myr"),
        name: "Leaden Myr".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Myr"]),
        oracle_text: "{T}: Add {B}.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 0, 0) },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
