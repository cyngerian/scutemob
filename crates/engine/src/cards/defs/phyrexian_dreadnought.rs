// Phyrexian Dreadnought — {1}, Artifact Creature — Phyrexian Dreadnought 12/12
// Trample
// When this creature enters, sacrifice it unless you sacrifice any number of creatures
// with total power 12 or greater.
//
// Trample is implemented.
// TODO: DSL gap — ETB "sacrifice unless you sacrifice creatures with total power >= 12"
// requires a conditional sacrifice with a power-sum predicate. Not expressible in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyrexian-dreadnought"),
        name: "Phyrexian Dreadnought".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Phyrexian", "Dreadnought"]),
        oracle_text: "Trample\nWhen this creature enters, sacrifice it unless you sacrifice any number of creatures with total power 12 or greater.".to_string(),
        power: Some(12),
        toughness: Some(12),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: ETB trigger — sacrifice self unless you sacrifice creatures totaling power 12+
        ],
        ..Default::default()
    }
}
