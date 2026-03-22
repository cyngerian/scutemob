// Paradise Mantle — {0}, Artifact — Equipment
// Equipped creature has "{T}: Add one mana of any color."
// Equip {1}
//
// TODO: "Equipped creature has '{T}: Add any color'" — granting activated abilities
//   via equipment not in DSL (only keyword grants via AddKeyword).
// Implementing Equip cost only.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("paradise-mantle"),
        name: "Paradise Mantle".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has \"{T}: Add one mana of any color.\"\nEquip {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // TODO: grant "{T}: Add any color" to equipped creature (no GrantActivatedAbility in DSL)
        ],
        ..Default::default()
    }
}
