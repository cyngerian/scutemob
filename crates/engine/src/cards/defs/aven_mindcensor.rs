// Aven Mindcensor — {2}{W}, Creature — Bird Wizard 2/1
// Flash, Flying
// If an opponent would search a library, that player searches the top four
// cards of that library instead.
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
            // CR 701.19 / CR 614.1: Restrict opponent library searches to top 4.
            // PlayerId(0) placeholder — bound to controller at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldSearchLibrary {
                    searcher_filter: PlayerFilter::OpponentsOf(PlayerId(0)),
                },
                modification: ReplacementModification::RestrictSearchTopN(4),
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
