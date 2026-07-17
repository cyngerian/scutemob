// Anointed Procession — {3}{W}, Enchantment
// If an effect would create one or more tokens under your control, it creates twice that
// many of those tokens instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("anointed-procession"),
        name: "Anointed Procession".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            white: 1,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Enchantment], &[]),
        oracle_text: "If an effect would create one or more tokens under your control, it creates \
                      twice that many of those tokens instead."
            .to_string(),
        abilities: vec![
            // CR 111.1 / CR 614.1: Token-doubling replacement effect.
            // PlayerId(0) placeholder — bound to the controller at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldCreateTokens {
                    controller_filter: PlayerFilter::Specific(PlayerId(0)),
                },
                modification: ReplacementModification::DoubleTokens,
                is_self: false,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
