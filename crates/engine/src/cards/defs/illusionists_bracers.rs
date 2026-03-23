// Illusionist's Bracers — {2}, Artifact — Equipment
// Whenever an ability of equipped creature is activated, if it isn't a mana ability,
// copy that ability. You may choose new targets for the copy.
// Equip {3}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("illusionists-bracers"),
        name: "Illusionist's Bracers".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Whenever an ability of equipped creature is activated, if it isn't a mana ability, copy that ability. You may choose new targets for the copy.\nEquip {3}".to_string(),
        abilities: vec![
            // TODO: DSL gap — triggered ability that copies activated abilities of
            // equipped creature. Ability copying not in DSL.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
