// Crystal Barricade — {1}{W}, Artifact Creature — Wall 0/4
// Defender
// You have hexproof.
// Prevent all noncombat damage that would be dealt to other creatures you control.
// TODO: DSL gap — "you have hexproof" (player hexproof) and blanket noncombat damage
// prevention for other creatures are not expressible in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crystal-barricade"),
        name: "Crystal Barricade".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Wall"]),
        oracle_text: "Defender (This creature can't attack.)\nYou have hexproof. (You can't be the target of spells or abilities your opponents control.)\nPrevent all noncombat damage that would be dealt to other creatures you control.".to_string(),
        power: Some(0),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Defender),
            // TODO: player hexproof (you have hexproof)
            // TODO: prevent noncombat damage to other creatures you control
        ],
        ..Default::default()
    }
}
