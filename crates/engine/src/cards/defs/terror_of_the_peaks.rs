// Terror of the Peaks — {3}{R}{R}, Creature — Dragon 5/4
// Flying
// Spells your opponents cast that target this creature cost an additional 3 life to cast.
// Whenever another creature you control enters, this creature deals damage equal to that creature's power to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("terror-of-the-peaks"),
        name: "Terror of the Peaks".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nSpells your opponents cast that target this creature cost an additional 3 life to cast.\nWhenever another creature you control enters, this creature deals damage equal to that creature's power to any target.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: Static ability — spells opponents cast that target this creature cost 3 additional
            // life. DSL gap: no life-additional-cost-when-targeted static.
            // TODO: Triggered ability — whenever another creature you control enters, deal damage
            // equal to that creature's power to any target.
            // DSL gap: no "entering creature's power" dynamic damage amount; requires targeted_trigger.
        ],
        ..Default::default()
    }
}
