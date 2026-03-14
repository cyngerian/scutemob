// Desert of the Fervent
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("desert-of-the-fervent"),
        name: "Desert of the Fervent".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Desert"]),
        oracle_text: "This land enters tapped.\n{T}: Add {R}.\nCycling {1}{R} ({1}{R}, Discard this card: Draw a card.)".to_string(),
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
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // CR 702.29: Cycling {1}{R}.
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 1, red: 1, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
