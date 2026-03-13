// Gingerbrute — {1}, Artifact Creature — Food Golem 1/1; Haste.
// {1}: can't be blocked this turn except by haste creatures — TODO: no "can't be blocked
// with filter" effect in DSL.
// {2}, {T}, Sacrifice: gain 3 life — TODO: sacrifice-as-cost in activated ability
// not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gingerbrute"),
        name: "Gingerbrute".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Food", "Golem"]),
        oracle_text: "Haste (This creature can attack and {T} as soon as it comes under your control.)\n{1}: This creature can't be blocked this turn except by creatures with haste.\n{2}, {T}, Sacrifice this creature: You gain 3 life.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        // TODO: {1} activated — can't be blocked except by haste creatures (filtered evasion)
        // TODO: {2},{T},sacrifice — sacrifice as cost not expressible in DSL
        ..Default::default()
    }
}
