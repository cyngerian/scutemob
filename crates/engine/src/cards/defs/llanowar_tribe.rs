// Llanowar Tribe — {T}: Add {G}{G}{G}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("llanowar-tribe"),
        name: "Llanowar Tribe".to_string(),
        mana_cost: Some(ManaCost { green: 3, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "{T}: Add {G}{G}{G}.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 1, 0) },
                timing_restriction: None,
            },
        ],
        ..Default::default()
    }
}
