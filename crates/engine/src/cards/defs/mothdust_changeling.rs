// Mothdust Changeling — {U}, Creature — Shapeshifter 1/1
// Changeling (This card is every creature type.)
// Tap an untapped creature you control: This creature gains flying until end of turn.
//
// Changeling is implemented.
//
// TODO: DSL gap — "Tap an untapped creature you control" as an activated ability cost
// has no Cost variant. The flying-granting activated ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mothdust-changeling"),
        name: "Mothdust Changeling".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Shapeshifter"]),
        oracle_text: "Changeling (This card is every creature type.)\nTap an untapped creature you control: This creature gains flying until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Changeling),
        ],
        ..Default::default()
    }
}
