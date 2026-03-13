// Hellkite Courser — {4}{R}{R}, Creature — Dragon 6/5
// Flying
// When this creature enters, you may put a commander you own from the command
// zone onto the battlefield. It gains haste. Return it to the command zone at
// the beginning of the next end step.
//
// Flying is implemented.
// TODO: DSL gap — the ETB triggered ability involves command zone manipulation
// (put commander from command zone to battlefield + haste grant + end step
// return-to-command-zone trigger). No ETB effect targets the command zone or
// grants temporary haste to a specific permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hellkite-courser"),
        name: "Hellkite Courser".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhen this creature enters, you may put a commander you own from the command zone onto the battlefield. It gains haste. Return it to the command zone at the beginning of the next end step.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — ETB command zone manipulation not expressible.
        ],
        ..Default::default()
    }
}
