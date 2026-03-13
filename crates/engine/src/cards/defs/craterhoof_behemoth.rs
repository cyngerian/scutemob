// Craterhoof Behemoth — {5}{G}{G}{G}, Creature — Beast 5/5; Haste.
// ETB trigger: creatures you control gain trample and get +X/+X where X = creature count.
// TODO: DSL gap — ETB pump based on count of creatures you control (count_threshold pattern)
// and mass trample grant not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("craterhoof-behemoth"),
        name: "Craterhoof Behemoth".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 3, ..Default::default() }),
        types: creature_types(&["Beast"]),
        oracle_text: "Haste\nWhen this creature enters, creatures you control gain trample and get +X/+X until end of turn, where X is the number of creatures you control.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        // TODO: ETB trigger — mass trample grant + X/X pump based on creature count
        ..Default::default()
    }
}
