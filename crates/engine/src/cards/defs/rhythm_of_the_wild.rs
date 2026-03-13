// Rhythm of the Wild — {1}{R}{G}, Enchantment
// Creature spells you control can't be countered.
// Nontoken creatures you control have riot. (They enter with your choice of a +1/+1 counter or haste.)
//
// TODO: DSL gap — "creature spells you control can't be countered" requires a
// continuous effect granting uncounterability to spells on the stack; no DSL
// primitive exists for that layer.
// TODO: DSL gap — granting KeywordAbility::Riot to all nontoken creatures you
// control requires a continuous effect with an EffectFilter for nontoken creatures;
// the current ContinuousEffectDef only grants to a single targeted permanent, not
// a blanket "all nontoken creatures you control".
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rhythm-of-the-wild"),
        name: "Rhythm of the Wild".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creature spells you control can't be countered.\nNontoken creatures you control have riot. (They enter with your choice of a +1/+1 counter or haste.)".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
