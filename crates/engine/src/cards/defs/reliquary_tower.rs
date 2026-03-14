// 12. Reliquary Tower — Land; tap: add {C}; you have no maximum hand size.
// CR 402.2: KeywordAbility::NoMaxHandSize signals the engine to skip the
// cleanup discard for the controller.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reliquary-tower"),
        name: "Reliquary Tower".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "You have no maximum hand size.\n{T}: Add {C}.".to_string(),
        abilities: vec![
            // CR 402.2: no maximum hand size for controller.
            AbilityDefinition::Keyword(KeywordAbility::NoMaxHandSize),
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
