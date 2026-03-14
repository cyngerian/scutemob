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
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::TheRingTemptsYou,
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "Whenever you choose a creature as your Ring-bearer, you may pay 2 life.
            // If you do, draw a card." — requires TriggerCondition::WhenRingBearerChosen
            // and optional cost payment (may pay 2 life → draw). Deferred.
        ],
        ..Default::default()
    }
}
