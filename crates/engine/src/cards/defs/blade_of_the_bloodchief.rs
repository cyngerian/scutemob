// Blade of the Bloodchief — {1}, Artifact — Equipment
// Whenever a creature dies, put a +1/+1 counter on equipped creature. If equipped creature
// is a Vampire, put two +1/+1 counters on it instead.
// Equip {1}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blade-of-the-bloodchief"),
        name: "Blade of the Bloodchief".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Whenever a creature dies, put a +1/+1 counter on equipped creature. If equipped creature is a Vampire, put two +1/+1 counters on it instead.\nEquip {1}".to_string(),
        abilities: vec![
            // TODO: DSL gap — "Whenever a creature dies, put +1/+1 counter on equipped
            // creature (2 if Vampire)." WheneverCreatureDies trigger exists, but
            // EffectTarget::EquippedCreature does not, and conditional counter count
            // based on equipped creature's subtype is not in DSL.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
