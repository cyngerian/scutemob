// Hammer of Nazahn — {4}, Legendary Artifact — Equipment
// ETB trigger: attach this or another Equipment to target creature you control
// Equipped creature gets +2/+0 and has indestructible; Equip {4}
// TODO: ETB trigger to attach equipment not in DSL; continuous effect on equipped creature not expressible
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hammer-of-nazahn"),
        name: "Hammer of Nazahn".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Whenever Hammer of Nazahn or another Equipment you control enters, you may attach that Equipment to target creature you control.\nEquipped creature gets +2/+0 and has indestructible.\nEquip {4}".to_string(),
        abilities: vec![
            // TODO: ETB trigger watching for any Equipment entering (not just self) and
            // attaching to target creature requires triggered_trigger with equipment filter
            // — not in DSL.
            // TODO: Continuous "+2/+0 and indestructible while equipped" effect requires
            // equipment-grant continuous effect layer — not in DSL.
            // TODO: Equip {4} activated ability not in DSL.
        ],
        ..Default::default()
    }
}
