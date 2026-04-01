// Fire Covenant — {1}{B}{R}, Instant
// As an additional cost to cast this spell, pay X life.
// Fire Covenant deals X damage divided as you choose among any number of target creatures.
//
// TODO: DSL gap — "pay X life as additional cost" is not expressible in SpellAdditionalCost
// (no PayLife or PayXLife variant). Divided damage among multiple targets also requires
// Effect::DealDamageDivided which does not exist. Both gaps must be resolved before
// this card can be fully authored. Approximated as Nothing to avoid wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fire-covenant"),
        name: "Fire Covenant".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "As an additional cost to cast this spell, pay X life.\nFire Covenant deals X damage divided as you choose among any number of target creatures.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "pay X life as additional cost" — SpellAdditionalCost has no PayLife/PayXLife variant.
            // TODO: "X damage divided among any number of target creatures" — no multi-target divide effect.
            // Both gaps need engine support before this card can be correctly implemented.
            effect: Effect::Nothing,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
