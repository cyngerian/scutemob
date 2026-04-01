// Ichor Rats — {1}{B}{B}, Creature — Phyrexian Rat 2/1
// Infect (This creature deals damage to creatures in the form of -1/-1 counters and
// to players in the form of poison counters.)
// When this enters, each player gets a poison counter.
// TODO: DSL gap — no Effect variant to give poison counters to players directly.
// "Each player gets a poison counter" requires a player-targeting poison counter add
// (not an object counter). Effect::AddCounter targets permanents only. This ETB
// trigger cannot be represented and is omitted. Core Infect keyword is implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ichor-rats"),
        name: "Ichor Rats".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Rat"]),
        oracle_text: "Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)\nWhen this enters, each player gets a poison counter.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Infect),
            // TODO: "When this enters, each player gets a poison counter." — no Effect
            // variant exists for giving poison counters to players. Omitted until
            // Effect::GivePlayerPoisonCounters is added.
        ],
        ..Default::default()
    }
}
