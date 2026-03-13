// Mindleecher — {4}{B}{B}, Creature — Nightmare 5/5
// Mutate {4}{B}; Flying
// Whenever this creature mutates, exile the top card of each opponent's library face down.
// You may look at and play those cards for as long as they remain exiled.
//
// TODO: DSL gap — mutate trigger omitted.
// "Whenever this creature mutates, exile the top card of each opponent's library face down.
// You may look at and play those cards for as long as they remain exiled."
// Requires a mutate triggered ability (TriggerCondition::WheneverThisCreatureMutates) with
// a ForEach-over-opponents effect that exiles the top card face down and grants play permission.
// No Effect variant for "exile top card of library face down with play-while-exiled grant"
// in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mindleecher"),
        name: "Mindleecher".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: creature_types(&["Nightmare"]),
        oracle_text: "Mutate {4}{B} (If you cast this spell for its mutate cost, put it over or under target non-Human creature you own. They mutate into the creature on top plus all abilities from under it.)\nFlying\nWhenever this creature mutates, exile the top card of each opponent's library face down. You may look at and play those cards for as long as they remain exiled.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Mutate),
            AbilityDefinition::MutateCost {
                cost: ManaCost { generic: 4, black: 1, ..Default::default() },
            },
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}
