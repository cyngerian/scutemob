// Legion Loyalist — {R}, Creature — Goblin Soldier 1/1
// "Haste
// Battalion — Whenever this creature and at least two other creatures attack, creatures you
// control gain first strike and trample until end of turn and can't be blocked by creature
// tokens this turn."
//
// Haste is implemented.
//
// TODO: DSL gap — Battalion trigger requires checking that THIS creature is attacking plus
// at least two others. TriggerCondition has no "when this creature and at least two other
// creatures attack" variant. The triggered effect (grant first strike + trample + can't be
// blocked by tokens) also has no DSL representation for token-blocker restrictions.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("legion-loyalist"),
        name: "Legion Loyalist".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Soldier"]),
        oracle_text: "Haste\nBattalion \u{2014} Whenever this creature and at least two other creatures attack, creatures you control gain first strike and trample until end of turn and can't be blocked by creature tokens this turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        ..Default::default()
    }
}
