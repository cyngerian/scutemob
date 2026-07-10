// Call of the Ring — {1}{B}, Enchantment
// At the beginning of your upkeep, the Ring tempts you.
// Whenever you choose a creature as your Ring-bearer, you may pay 2 life.
// If you do, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("call-of-the-ring"),
        name: "Call of the Ring".to_string(),
        mana_cost: Some(ManaCost { black: 1, generic: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your upkeep, the Ring tempts you.\nWhenever you choose a creature as your Ring-bearer, you may pay 2 life. If you do, draw a card.".to_string(),
        abilities: vec![
            // CR 701.54a: At the beginning of your upkeep, the Ring tempts you.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::TheRingTemptsYou,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // ENGINE-BLOCKED: "Whenever you choose a creature as your Ring-bearer, you
            // may pay 2 life. If you do, draw a card." PB-AC2's Effect::MayPayThenEffect
            // (CR 118.12) now covers the "may pay 2 life -> draw" rider, but
            // TriggerCondition::WhenRingBearerChosen does not exist (verified: the only
            // Ring-related trigger in the enum is WheneverRingTemptsYou, which fires on
            // temptation, not on choosing a Ring-bearer). Genuine remaining gap, out of
            // PB-AC2 scope.
        ],
        completeness: Completeness::partial("'Whenever you choose a creature as your Ring-bearer, you may pay 2 life. If you do, draw a card.' PB-AC2's..."),
        ..Default::default()
    }
}
