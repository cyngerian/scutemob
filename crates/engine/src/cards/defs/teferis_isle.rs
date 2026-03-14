// Teferi's Isle — Legendary Land
// CR 614.1c: enters tapped (self-replacement); CR 702.26a: Phasing.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferis-isle"),
        name: "Teferi's Isle".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "Phasing (This phases in or out before you untap during each of your untap steps. While it's phased out, it's treated as though it doesn't exist.)\nTeferi's Isle enters tapped.\n{T}: Add {U}{U}.".to_string(),
        abilities: vec![
            // CR 702.26a: Phasing — phases in or out before untap.
            AbilityDefinition::Keyword(KeywordAbility::Phasing),
            // CR 614.1c: self-replacement — this permanent enters the battlefield tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {U}{U}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 2, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
