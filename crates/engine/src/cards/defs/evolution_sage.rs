// Evolution Sage — {2}{G}, Creature — Elf Druid 3/2
// Landfall — Whenever a land you control enters, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("evolution-sage"),
        name: "Evolution Sage".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "Landfall — Whenever a land you control enters, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // Landfall — Whenever a land you control enters, proliferate.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
