// Nullpriest of Oblivion — {1}{B}, Creature — Vampire Cleric 2/1
// Kicker {3}{B}; Lifelink; Menace
// When this creature enters, if it was kicked, return target creature card from your graveyard
// to the battlefield.
//
// TODO: DSL gap — Kicker ETB trigger omitted.
// "When this creature enters, if it was kicked, return target creature card from your graveyard
// to the battlefield." Requires an ETB triggered ability with an intervening-if condition
// (Condition::WasKicked) and a targeted return-from-graveyard effect. DSL gap:
// return_from_graveyard pattern (ZoneTarget::Battlefield from graveyard with target filter)
// is not currently supported.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nullpriest-of-oblivion"),
        name: "Nullpriest of Oblivion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Cleric"]),
        oracle_text: "Kicker {3}{B} (You may pay an additional {3}{B} as you cast this spell.)\nLifelink\nMenace (This creature can't be blocked except by two or more creatures.)\nWhen this creature enters, if it was kicked, return target creature card from your graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Keyword(KeywordAbility::Menace),
        ],
        ..Default::default()
    }
}
