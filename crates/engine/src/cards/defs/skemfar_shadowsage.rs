// Skemfar Shadowsage — {3}{B}, Creature — Elf Cleric 2/5
// When this enters, choose one —
// • Each opponent loses X life, where X is the greatest number of creatures you
//   control that share a creature type.
// • You gain X life, where X is the greatest number of creatures you control
//   that share a creature type.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skemfar-shadowsage"),
        name: "Skemfar Shadowsage".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Cleric"]),
        oracle_text: "When this creature enters, choose one —\n• Each opponent loses X life, where X is the greatest number of creatures you control that share a creature type.\n• You gain X life, where X is the greatest number of creatures you control that share a creature type.".to_string(),
        power: Some(2),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                // TODO: X = greatest number of creatures sharing a creature type.
                // No EffectAmount variant for this. Using Fixed(0) stub.
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
