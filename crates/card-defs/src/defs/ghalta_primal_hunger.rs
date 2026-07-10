// Ghalta, Primal Hunger — {10}{G}{G}, Legendary Creature — Elder Dinosaur 12/12
// This spell costs {X} less to cast, where X is the total power of creatures you control.
// Trample
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ghalta-primal-hunger"),
        name: "Ghalta, Primal Hunger".to_string(),
        mana_cost: Some(ManaCost { generic: 10, green: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elder", "Dinosaur"]),
        oracle_text: "This spell costs {X} less to cast, where X is the total power of creatures you control.\nTrample (This creature can deal excess combat damage to the player or planeswalker it's attacking.)".to_string(),
        power: Some(12),
        toughness: Some(12),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
        ],
        self_cost_reduction: Some(SelfCostReduction::TotalPowerOfCreatures),
        ..Default::default()
    }
}
