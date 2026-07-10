// Savai Triome
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("savai-triome"),
        name: "Savai Triome".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Plains", "Swamp", "Mountain"]),
        oracle_text: "({T}: Add {R}, {W}, or {B}.)\nThis land enters tapped.\nCycling {3} ({3}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // Mana production handled by basic land subtypes Plains/Swamp/Mountain (CR 305.6).
            // CR 702.29: Cycling {3}.
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 3, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
