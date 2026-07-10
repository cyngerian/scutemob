// Jetmir's Garden
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("jetmirs-garden"),
        name: "Jetmir's Garden".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Mountain", "Forest", "Plains"]),
        oracle_text: "({T}: Add {R}, {G}, or {W}.)\nThis land enters tapped.\nCycling {3} ({3}, Discard this card: Draw a card.)".to_string(),
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
            // Mana production handled by basic land subtypes Mountain/Forest/Plains (CR 305.6).
            // CR 702.29: Cycling {3}.
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 3, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
