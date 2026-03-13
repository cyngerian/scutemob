// Tainted Observer — {1}{G}{U}, Creature — Phyrexian Bird 2/3
// Flying, toxic 1; whenever another creature you control enters, may pay {2} to proliferate.
// TODO: "Whenever another creature you control enters, you may pay {2}. If you do, proliferate."
// This is a MayPayOrElse / intervening-if with cost payment pattern. The DSL has
// Effect::MayPayOrElse but TriggerCondition::WheneverCreatureEntersBattlefield has no
// "exclude self" for this trigger + the optional cost payment at trigger resolution is
// not yet linked to AbilityDefinition::Triggered (would need cost in trigger, not just effect).
// Deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tainted-observer"),
        name: "Tainted Observer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Bird"]),
        oracle_text: "Flying\nToxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhenever another creature you control enters, you may pay {2}. If you do, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            // TODO: "Whenever another creature you control enters, you may pay {2}. If you do, proliferate."
            // DSL gap: optional cost payment at trigger resolution not supported.
        ],
        ..Default::default()
    }
}
