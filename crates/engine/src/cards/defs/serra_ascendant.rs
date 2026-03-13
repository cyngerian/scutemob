// Serra Ascendant — {W}, Creature — Human Monk 1/1
// Lifelink (Damage dealt by this creature also causes you to gain that much life.)
// As long as you have 30 or more life, this creature gets +5/+5 and has flying.
//
// Lifelink is implemented.
//
// TODO: DSL gap — "as long as you have 30 or more life, this creature gets +5/+5 and has
// flying" is a conditional static ability. EffectDuration has no "while condition X" variant,
// so there is no way to register a P/T modify + Flying grant that is active only while
// the controller's life total is >= 30. Omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("serra-ascendant"),
        name: "Serra Ascendant".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Monk"]),
        oracle_text: "Lifelink (Damage dealt by this creature also causes you to gain that much life.)\nAs long as you have 30 or more life, this creature gets +5/+5 and has flying.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
