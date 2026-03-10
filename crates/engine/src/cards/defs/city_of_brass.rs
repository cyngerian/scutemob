// City of Brass — Land; {T}: Add one mana of any color.
// "Whenever City of Brass becomes tapped, it deals 1 damage to you."
// TODO: DSL gap — the damage trigger fires on ANY tap (not just mana ability),
// including tap from opponents' effects. Modeled as damage on the mana ability
// itself (like Ancient Tomb) which is functionally close but misses opponent-taps.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("city-of-brass"),
        name: "City of Brass".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "Whenever City of Brass becomes tapped, it deals 1 damage to you.\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::Sequence(vec![
                Effect::DealDamage {
                    target: EffectTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                Effect::AddManaAnyColor { player: PlayerTarget::Controller },
            ]),
            timing_restriction: None,
        }],
        ..Default::default()
    }
}
