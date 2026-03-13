// Sting, the Glinting Dagger — {2}, Legendary Artifact — Equipment
// TODO: DSL gap — static "Equipped creature gets +1/+1 and has haste."
//   (equipment continuous effects require EffectFilter::EquippedCreature which may not exist)
// TODO: DSL gap — triggered "At the beginning of each combat, untap equipped creature."
//   (AtBeginningOfCombat trigger targeting equipped creature not supported)
// TODO: DSL gap — conditional keyword "Equipped creature has first strike as long as it's
//   blocking or blocked by a Goblin or Orc." (combat-conditional keyword grant not supported)
// Equip {2} is a keyword but Equipment Equip activated ability requires target-creature
// activated ability which is also a DSL gap.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sting-the-glinting-dagger"),
        name: "Sting, the Glinting Dagger".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Equipped creature gets +1/+1 and has haste.\nAt the beginning of each combat, untap equipped creature.\nEquipped creature has first strike as long as it's blocking or blocked by a Goblin or Orc.\nEquip {2}".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
