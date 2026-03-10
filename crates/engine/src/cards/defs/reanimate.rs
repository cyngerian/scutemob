// Reanimate — {B} Sorcery; put target creature card from a graveyard onto
// the battlefield under your control. You lose life equal to its mana value.
// TODO: DSL gap — "target creature card in a graveyard" targeting and
// "lose life equal to its mana value" dynamic life loss are not expressible.
// Only the spell shell is defined.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reanimate"),
        name: "Reanimate".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Put target creature card from a graveyard onto the battlefield under your control. You lose life equal to its mana value.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(0),
            },
            // TODO: target creature in graveyard + move to battlefield + lose life
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
