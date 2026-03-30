// Thornbite Staff — {2} Kindred Artifact — Shaman Equipment
// Equipped creature has "{2}, {T}: This creature deals 1 damage to any target" and
// "Whenever a creature dies, untap this creature."
// Whenever a Shaman creature enters, you may attach this Equipment to it.
// Equip {4}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thornbite-staff"),
        name: "Thornbite Staff".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Shaman", "Equipment"]),
        oracle_text: "Equipped creature has \"{2}, {T}: This creature deals 1 damage to any target\" and \"Whenever a creature dies, untap this creature.\"\nWhenever a Shaman creature enters, you may attach Thornbite Staff to it.\nEquip {4}".to_string(),
        abilities: vec![
            // TODO: Granting activated abilities to equipped creature not in DSL.
            // TODO: "Whenever a creature dies, untap" — trigger not in DSL.
            // TODO: "Whenever a Shaman enters, attach" — auto-attach by subtype not in DSL.
            // Equip {4}
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 4, ..Default::default() }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
