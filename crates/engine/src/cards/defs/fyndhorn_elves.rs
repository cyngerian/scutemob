// Fyndhorn Elves — {G}, Creature — Elf Druid 1/1; {T}: Add {G}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fyndhorn-elves"),
        name: "Fyndhorn Elves".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "{T}: Add {G}.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 0, 0, 1, 0),
            },
            timing_restriction: None,
        }],
        color_indicator: None,
        back_face: None,
    }
}
