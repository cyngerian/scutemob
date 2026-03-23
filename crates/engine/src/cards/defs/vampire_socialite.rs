// Vampire Socialite — {B}{R}, Creature — Vampire Noble 2/2
// Menace
// When this creature enters, if an opponent lost life this turn, put a +1/+1 counter on
// each other Vampire you control.
// As long as an opponent lost life this turn, each other Vampire you control enters with
// an additional +1/+1 counter on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-socialite"),
        name: "Vampire Socialite".to_string(),
        mana_cost: Some(ManaCost { black: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Noble"]),
        oracle_text: "Menace\nWhen this creature enters, if an opponent lost life this turn, put a +1/+1 counter on each other Vampire you control.\nAs long as an opponent lost life this turn, each other Vampire you control enters with an additional +1/+1 counter on it.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: DSL gap — intervening-if "if an opponent lost life this turn"
            // (Condition::OpponentLostLifeThisTurn) does not exist.
            // TODO: DSL gap — replacement effect for "enters with an additional +1/+1 counter"
            // conditional on opponent life loss. Needs conditional ETB replacement.
        ],
        ..Default::default()
    }
}
