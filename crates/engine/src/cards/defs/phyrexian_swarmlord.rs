// Phyrexian Swarmlord — {4}{G}{G}, Creature — Phyrexian Insect Horror 4/4
// Infect (This creature deals damage to creatures in the form of -1/-1 counters and
// to players in the form of poison counters.)
// At the beginning of your upkeep, create a 1/1 green Phyrexian Insect creature token
// with infect for each poison counter your opponents have.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phyrexian-swarmlord"),
        name: "Phyrexian Swarmlord".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Insect", "Horror"]),
        oracle_text: "Infect (This creature deals damage to creatures in the form of -1/-1 counters and to players in the form of poison counters.)\nAt the beginning of your upkeep, create a 1/1 green Phyrexian Insect creature token with infect for each poison counter your opponents have.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Infect),
            // TODO: Upkeep trigger creates X tokens where X = total poison counters on
            //   opponents. EffectAmount::Fixed only; no CountOpponentPoisonCounters variant.
            //   W5 policy: omitted.
        ],
        ..Default::default()
    }
}
