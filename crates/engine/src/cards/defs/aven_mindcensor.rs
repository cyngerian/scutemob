// Aven Mindcensor — {2}{W}, Creature — Bird Wizard 2/1
// Flash, Flying
// If an opponent would search a library, that player searches the top four
// cards of that library instead.
//
// Flash and Flying are implemented.
// TODO: DSL gap — the replacement effect ("search top four instead of entire
// library") requires a ReplacementTrigger for opponent library searches with
// a ZoneRestriction modification. No such replacement effect exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aven-mindcensor"),
        name: "Aven Mindcensor".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Bird", "Wizard"]),
        oracle_text: "Flash\nFlying\nIf an opponent would search a library, that player searches the top four cards of that library instead.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — opponent library search restriction replacement not expressible.
        ],
        ..Default::default()
    }
}
