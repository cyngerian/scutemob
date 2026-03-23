// Blightbelly Rat — {1}{B}, Creature — Phyrexian Rat 2/2
// Toxic 1
// When this creature dies, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blightbelly-rat"),
        name: "Blightbelly Rat".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Rat"]),
        oracle_text: "Toxic 1\nWhen Blightbelly Rat dies, proliferate.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
