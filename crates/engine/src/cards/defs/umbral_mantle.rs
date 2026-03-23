// Umbral Mantle — {3}, Artifact — Equipment
// Equipped creature has "{3}, {Q}: This creature gets +2/+2 until end of turn."
// Equip {0}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("umbral-mantle"),
        name: "Umbral Mantle".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has \"{3}, {Q}: This creature gets +2/+2 until end of turn.\"\nEquip {0}".to_string(),
        abilities: vec![
            // TODO: DSL gap — granting activated ability to equipped creature ({Q} = untap
            // symbol cost). GrantActivatedAbility not in DSL.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
