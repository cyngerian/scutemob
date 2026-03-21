// 48. Elvish Mystic — {G}, Creature — Elf Druid 1/1; same as Llanowar Elves.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elvish-mystic"),
        name: "Elvish Mystic".to_string(),
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
            targets: vec![],
                activation_condition: None,
        }],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
