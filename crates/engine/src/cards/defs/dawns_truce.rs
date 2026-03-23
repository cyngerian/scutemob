// Dawn's Truce — {1}{W}, Instant
// Gift a card
// You and permanents you control gain hexproof until end of turn. If the gift was
// promised, permanents you control also gain indestructible until end of turn.
//
// TODO: Gift mechanic draw + hexproof/indestructible continuous effects complex for DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dawns-truce"),
        name: "Dawn's Truce".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Gift a card (You may promise an opponent a gift as you cast this spell. If you do, they draw a card before its other effects.)\nYou and permanents you control gain hexproof until end of turn. If the gift was promised, permanents you control also gain indestructible until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Gift),
            // TODO: Hexproof + conditional indestructible not easily expressible.
        ],
        ..Default::default()
    }
}
