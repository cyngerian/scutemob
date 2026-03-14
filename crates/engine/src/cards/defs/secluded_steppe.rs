// Secluded Steppe
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("secluded-steppe"),
        name: "Secluded Steppe".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {W}.\nCycling {W} ({W}, Discard this card: Draw a card.)".to_string(),
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
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // CR 702.29: Cycling {W}.
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { white: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
